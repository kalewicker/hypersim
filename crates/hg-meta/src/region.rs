use crate::schema::*;
use hg_core::{
    EdgeId, GraphSnapshot, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue,
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug)]
pub struct EncodedRegion {
    pub root: NodeId,
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
}
fn node(s: &mut HypergraphStore, label: u32, props: &[(u32, PropertyValue)]) -> NodeId {
    let mut n = NodeData::new(Label(label), s.step());
    for &(k, v) in props {
        n.set_property(k, v);
    }
    s.add_node(n)
}
fn edge(s: &mut HypergraphStore, label: u32, nodes: &[NodeId]) -> EdgeId {
    s.add_edge(HyperEdge::new(
        Label(label),
        nodes.iter().copied().collect(),
    ))
}

/// Seed-only encoding. Runtime mutation is performed by the ordinary engine,
/// never by this writer. Shared var nodes are the cross-side identity.
pub fn encode(
    s: &mut HypergraphStore,
    c: &CandidateRule,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    encode_versioned(s, c, limits, EncodingVersion::LegacyV1)
}

pub fn encode_versioned(
    s: &mut HypergraphStore,
    c: &CandidateRule,
    limits: &MetaLimits,
    version: EncodingVersion,
) -> Result<EncodedRegion, Validator> {
    if version == EncodingVersion::AlternativeFamiliesV3 {
        return crate::encode_family(s, c, c, false, limits);
    }
    c.validate(limits)?;
    // Pair identical occurrences one-to-one. A physical pattern never
    // represents two occurrences on the same side.
    let mut patterns: Vec<(Option<usize>, Option<usize>, &Vec<usize>)> = c
        .lhs
        .iter()
        .enumerate()
        .map(|(i, e)| (Some(i), None, e))
        .collect();
    for (j, endpoints) in c.rhs.iter().enumerate() {
        if version == EncodingVersion::SharedPatternsV2 {
            if let Some(p) = patterns
                .iter_mut()
                .find(|(l, r, e)| l.is_some() && r.is_none() && *e == endpoints)
            {
                p.1 = Some(j);
                continue;
            }
        }
        patterns.push((None, Some(j), endpoints));
    }
    let n_count = 3
        + c.labels.len()
        + patterns.len()
        + c.labels.iter().filter(|l| l.is_some()).count()
        + c.guards.len();
    let e_count = 1
        + c.lhs.len()
        + c.rhs.len()
        + patterns.iter().map(|p| p.2.len()).sum::<usize>()
        + c.labels.iter().filter(|l| l.is_some()).count()
        + c.guards.len();
    if n_count > limits.max_region_nodes || e_count > limits.max_region_edges {
        return Err(Validator::Region);
    }
    let root = node(
        s,
        L_RULE_ROOT,
        &[(K_PRIORITY, PropertyValue::Int(c.priority as i64))],
    );
    if version == EncodingVersion::SharedPatternsV2 {
        s.get_node_mut(root)
            .unwrap()
            .set_property(K_ENCODING, PropertyValue::Int(2));
    }
    let l = node(s, L_SIDE_L, &[]);
    let r = node(s, L_SIDE_R, &[]);
    let mut nodes = vec![root, l, r];
    let mut edges = vec![edge(s, E_BECOMES, &[root, l, r])];
    let vars: Vec<NodeId> = c
        .labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let v = node(s, L_VAR, &[(K_ORDER, PropertyValue::Int(i as i64))]);
            nodes.push(v);
            if let Some(label) = label {
                let lit = node(
                    s,
                    L_LABEL_LIT,
                    &[(K_LITERAL, PropertyValue::Int(*label as i64))],
                );
                nodes.push(lit);
                edges.push(edge(s, E_HASLABEL, &[v, lit]));
            }
            v
        })
        .collect();
    for (li, ri, endpoints) in patterns {
        let p = node(
            s,
            L_PAT_EDGE,
            &[(K_ORDER, PropertyValue::Int(li.or(ri).unwrap() as i64))],
        );
        nodes.push(p);
        if li.is_some() {
            edges.push(edge(s, E_MEMBER, &[l, p]));
        }
        if let Some(j) = ri {
            edges.push(edge(s, E_MEMBER, &[r, p]));
            if version == EncodingVersion::SharedPatternsV2 {
                s.get_node_mut(p)
                    .unwrap()
                    .set_property(K_ORDER_R, PropertyValue::Int(j as i64));
            }
        }
        for (k, v) in endpoints.iter().enumerate() {
            edges.push(edge(s, E_SLOT_BASE + k as u32, &[p, vars[*v]]));
        }
    }
    for g in &c.guards {
        let n = node(
            s,
            L_GUARD,
            &[
                (K_GKEY, PropertyValue::Int(g.key as i64)),
                (K_GOP, PropertyValue::Int(op_code(g.op) as i64)),
                (K_GVAL, g.value),
            ],
        );
        nodes.push(n);
        edges.push(edge(s, E_GUARDS, &[vars[g.var], n]));
    }
    Ok(EncodedRegion { root, nodes, edges })
}

fn int(s: &GraphSnapshot<'_>, n: NodeId, k: u32) -> Option<i64> {
    match s.get_node(n)?.get_property(k)? {
        PropertyValue::Int(v) => Some(v),
        _ => None,
    }
}
fn has_label(s: &GraphSnapshot<'_>, n: NodeId, l: u32) -> bool {
    s.get_node(n).is_some_and(|x| x.label.0 == l)
}

/// Budget every inspected incidence, not just distinct edges; malformed,
/// duplicate-incidence or high-degree input cannot escape the work bound.
pub fn walk(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    walk_impl(s, root, limits, has_label(s, root, L_ALLELE_ROOT))
}
pub fn walk_family(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    walk_impl(s, root, limits, false)
}
fn walk_impl(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
    component: bool,
) -> Result<EncodedRegion, Validator> {
    if !has_label(s, root, L_RULE_ROOT) && !has_label(s, root, L_ALLELE_ROOT) {
        return Err(Validator::Region);
    }
    let mut nodes = vec![root];
    let mut edges = Vec::new();
    let mut seen_n = HashSet::from([root]);
    let mut seen_e = HashSet::new();
    let mut q = VecDeque::from([(root, 0)]);
    let mut work = 0usize;
    while let Some((n, depth)) = q.pop_front() {
        for &eid in s.edges_incident_to(n) {
            work += 1;
            if work > limits.max_region_edges.saturating_mul(16) {
                return Err(Validator::Region);
            }
            let e = s.get_edge(eid).ok_or(Validator::Region)?;
            if component && matches!(e.label.0, E_CORE | E_GRAFT) {
                continue;
            }
            if e.label.0 < META_BASE || !seen_e.insert(eid) {
                continue;
            }
            // E1 relations are binary except ternary E_BECOMES. Bound
            // endpoint inspection too, not just incidence enumeration.
            if e.nodes.len() > 3 {
                return Err(Validator::Region);
            }
            edges.push(eid);
            if edges.len() > limits.max_region_edges {
                return Err(Validator::Region);
            }
            for &v in &e.nodes {
                if s.get_node(v).is_none_or(|x| x.label.0 < META_BASE) {
                    return Err(Validator::Region);
                }
                if seen_n.insert(v) {
                    if depth >= limits.radius || nodes.len() >= limits.max_region_nodes {
                        return Err(Validator::Region);
                    }
                    nodes.push(v);
                    q.push_back((v, depth + 1));
                }
            }
        }
    }
    Ok(EncodedRegion { root, nodes, edges })
}

/// Each rejected region increments exactly the first failing validator.
pub fn read(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
) -> Result<CandidateRule, Validator> {
    let region = walk(s, root, limits)?;
    let version = match s.get_node(root).and_then(|n| n.get_property(K_ENCODING)) {
        None | Some(PropertyValue::Int(1)) => EncodingVersion::LegacyV1,
        Some(PropertyValue::Int(2)) => EncodingVersion::SharedPatternsV2,
        _ => return Err(Validator::Region),
    };
    if region.nodes.iter().any(|&n| {
        s.get_node(n).is_none_or(|n| {
            ![
                L_RULE_ROOT,
                L_ALLELE_ROOT,
                L_SIDE_L,
                L_SIDE_R,
                L_PAT_EDGE,
                L_VAR,
                L_LABEL_LIT,
                L_GUARD,
            ]
            .contains(&n.label.0)
        })
    }) {
        return Err(Validator::Region);
    }
    if region
        .nodes
        .iter()
        .filter(|&&n| has_label(s, n, L_RULE_ROOT) || has_label(s, n, L_ALLELE_ROOT))
        .count()
        != 1
    {
        return Err(Validator::Sides);
    }
    let becomes: Vec<_> = region
        .edges
        .iter()
        .filter_map(|&e| s.get_edge(e))
        .filter(|e| e.label.0 == E_BECOMES)
        .collect();
    if becomes.len() != 1 || becomes[0].nodes.len() != 3 || becomes[0].nodes[0] != root {
        return Err(Validator::Sides);
    }
    let l = becomes[0].nodes[1];
    let r = becomes[0].nodes[2];
    if !has_label(s, l, L_SIDE_L) || !has_label(s, r, L_SIDE_R) {
        return Err(Validator::Sides);
    }
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut pat_sides: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
    for e in region
        .edges
        .iter()
        .filter_map(|&e| s.get_edge(e))
        .filter(|e| e.label.0 == E_MEMBER)
    {
        if e.nodes.len() != 2 || !has_label(s, e.nodes[1], L_PAT_EDGE) {
            return Err(Validator::Members);
        }
        let side = e.nodes[0];
        if side != l && side != r {
            return Err(Validator::Members);
        }
        let memberships = pat_sides.entry(e.nodes[1]).or_default();
        if !memberships.insert(side)
            || (version == EncodingVersion::LegacyV1 && memberships.len() > 1)
        {
            return Err(Validator::Members);
        }
        if side == l {
            left.push(e.nodes[1]);
        } else {
            right.push(e.nodes[1]);
        }
    }
    if region
        .nodes
        .iter()
        .filter(|&&n| has_label(s, n, L_PAT_EDGE))
        .count()
        != pat_sides.len()
    {
        return Err(Validator::Members);
    }
    let mut slot_map: HashMap<NodeId, Vec<(u32, NodeId)>> = HashMap::new();
    for e in region.edges.iter().filter_map(|&e| s.get_edge(e)) {
        if (E_SLOT_BASE..E_HASLABEL).contains(&e.label.0) {
            if e.label.0 >= E_SLOT_BASE + limits.max_arity.min(8) as u32
                || e.nodes.len() != 2
                || !pat_sides.contains_key(&e.nodes[0])
                || !has_label(s, e.nodes[1], L_VAR)
            {
                return Err(Validator::Slots);
            }
            slot_map
                .entry(e.nodes[0])
                .or_default()
                .push((e.label.0, e.nodes[1]));
        }
    }
    for &p in pat_sides.keys() {
        let slots = slot_map.entry(p).or_default();
        slots.sort_by_key(|x| x.0);
        if slots.len() < 2 || slots.windows(2).any(|w| w[0].0 == w[1].0) {
            return Err(Validator::Slots);
        }
    }
    let mut vars: Vec<_> = region
        .nodes
        .iter()
        .copied()
        .filter(|&n| has_label(s, n, L_VAR))
        .collect();
    if vars.is_empty() || vars.len() > limits.max_vars.min(8) {
        return Err(Validator::Variables);
    }
    if version == EncodingVersion::SharedPatternsV2 {
        let mut seen = HashSet::new();
        for &v in &vars {
            if let Some(p) = s.get_node(v).unwrap().get_property(K_ORDER) {
                if !matches!(p, PropertyValue::Int(i) if (0..8).contains(&i) && seen.insert(i)) {
                    return Err(Validator::Variables);
                }
            }
        }
    }
    let hinted = vars.iter().all(|&v| int(s, v, K_ORDER).is_some());
    if hinted {
        vars.sort_by_key(|&v| int(s, v, K_ORDER));
        if vars
            .iter()
            .any(|&v| int(s, v, K_ORDER).is_none_or(|i| !(0..8).contains(&i)))
            || vars
                .windows(2)
                .any(|w| int(s, w[0], K_ORDER) == int(s, w[1], K_ORDER))
        {
            return Err(Validator::Variables);
        }
    }
    let ix: HashMap<_, _> = vars.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let mut c = CandidateRule {
        labels: vec![None; vars.len()],
        ..CandidateRule::default()
    };
    c.priority = int(s, root, K_PRIORITY)
        .and_then(|p| i32::try_from(p).ok())
        .ok_or(Validator::Compilability)?;
    let mut used_lits = HashSet::new();
    let mut used_guards = HashSet::new();
    for e in region.edges.iter().filter_map(|&e| s.get_edge(e)) {
        match e.label.0 {
            E_HASLABEL => {
                if e.nodes.len() != 2
                    || !ix.contains_key(&e.nodes[0])
                    || !has_label(s, e.nodes[1], L_LABEL_LIT)
                {
                    return Err(Validator::Labels);
                }
                let value = int(s, e.nodes[1], K_LITERAL)
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or(Validator::Labels)?;
                let v = ix[&e.nodes[0]];
                if c.labels[v].replace(value).is_some() {
                    return Err(Validator::Labels);
                }
                used_lits.insert(e.nodes[1]);
            }
            E_GUARDS => {
                if e.nodes.len() != 2
                    || !ix.contains_key(&e.nodes[0])
                    || !has_label(s, e.nodes[1], L_GUARD)
                    || !used_guards.insert(e.nodes[1])
                {
                    return Err(Validator::Guards);
                }
                let n = e.nodes[1];
                c.guards.push(Guard {
                    var: ix[&e.nodes[0]],
                    key: int(s, n, K_GKEY)
                        .and_then(|k| u32::try_from(k).ok())
                        .ok_or(Validator::Guards)?,
                    op: int(s, n, K_GOP)
                        .and_then(decode_op)
                        .ok_or(Validator::Guards)?,
                    value: s
                        .get_node(n)
                        .and_then(|n| n.get_property(K_GVAL))
                        .ok_or(Validator::Guards)?,
                });
            }
            E_BECOMES | E_MEMBER => (),
            label if (E_SLOT_BASE..E_HASLABEL).contains(&label) => (),
            _ => return Err(Validator::Bounds),
        }
    }
    if region
        .nodes
        .iter()
        .filter(|&&n| has_label(s, n, L_LABEL_LIT))
        .count()
        != used_lits.len()
    {
        return Err(Validator::Labels);
    }
    if region
        .nodes
        .iter()
        .filter(|&&n| has_label(s, n, L_GUARD))
        .count()
        != used_guards.len()
    {
        return Err(Validator::Guards);
    }
    for (ps, key) in [
        (&mut left, K_ORDER),
        (
            &mut right,
            if version == EncodingVersion::SharedPatternsV2 {
                K_ORDER_R
            } else {
                K_ORDER
            },
        ),
    ] {
        if version == EncodingVersion::SharedPatternsV2 {
            let mut seen = HashSet::new();
            for &p in ps.iter() {
                let n = s.get_node(p).unwrap();
                if let Some(hint) = n.get_property(key).or_else(|| n.get_property(K_ORDER)) {
                    match hint {
                        PropertyValue::Int(i)
                            if i >= 0 && i <= u32::MAX as i64 && seen.insert(i) =>
                        {
                            ()
                        }
                        _ => return Err(Validator::Members),
                    }
                }
            }
        }
        ps.sort_by_key(|&p| {
            (
                int(s, p, key).or_else(|| int(s, p, K_ORDER)).unwrap_or(0),
                slot_map[&p].iter().map(|(_, v)| ix[v]).collect::<Vec<_>>(),
            )
        });
    }
    c.lhs = left
        .iter()
        .map(|p| slot_map[p].iter().map(|(_, v)| ix[v]).collect())
        .collect();
    c.rhs = right
        .iter()
        .map(|p| slot_map[p].iter().map(|(_, v)| ix[v]).collect())
        .collect();
    c.validate(limits)?;
    // Without writer hints, all allocation history must be erased before
    // assigning executable variable indices. Edges and guards also sorted.
    if !hinted {
        c = canonical_candidate(&c);
    }
    c.guards
        .sort_by_key(|g| (g.var, g.key, op_code(g.op), value_key(g.value)));
    Ok(c)
}
fn value_key(v: PropertyValue) -> (u8, u64) {
    match v {
        PropertyValue::Int(x) => (0, x as u64),
        PropertyValue::Float(x) => (1, x.to_bits()),
        PropertyValue::Bool(x) => (2, x as u64),
    }
}
/// Canonicalize executable presentation too when no ordering hints exist.
pub fn canonical_candidate(c: &CandidateRule) -> CandidateRule {
    fn renamed(c: &CandidateRule, p: &[usize]) -> CandidateRule {
        let mut r = c.clone();
        for (i, &v) in p.iter().enumerate() {
            r.labels[v] = c.labels[i];
        }
        for e in r.lhs.iter_mut().chain(&mut r.rhs) {
            for v in e {
                *v = p[*v];
            }
        }
        for g in &mut r.guards {
            g.var = p[g.var];
        }
        r.lhs.sort();
        r.rhs.sort();
        r.guards
            .sort_by_key(|g| (g.var, g.key, op_code(g.op), value_key(g.value)));
        r
    }
    fn visit(
        c: &CandidateRule,
        p: &mut [usize],
        k: usize,
        best: &mut CandidateRule,
        bytes: &mut Vec<u8>,
    ) {
        if k == p.len() {
            let r = renamed(c, p);
            let b = r.execution_bytes();
            if b < *bytes {
                *best = r;
                *bytes = b;
            }
            return;
        }
        for i in k..p.len() {
            p.swap(k, i);
            visit(c, p, k + 1, best, bytes);
            p.swap(k, i);
        }
    }
    let mut p: Vec<_> = (0..c.labels.len()).collect();
    let mut best = renamed(c, &p);
    let mut bytes = best.execution_bytes();
    visit(c, &mut p, 0, &mut best, &mut bytes);
    best
}
