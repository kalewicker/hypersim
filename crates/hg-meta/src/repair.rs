//! Explicit witness-backed representation. Decoding never repairs the live graph.
use crate::*;
use hg_core::{
    GraphSnapshot, HyperEdge, HypergraphStore, Label, NodeId, PropertyOp, PropertyValue,
};
use hg_rules::{
    compile_rule, CompiledRule, EffectOpDecl, NodeDegreeCompare, NodePropertyGuard,
    PropertyEffectDecl, Rule,
};
use std::collections::{HashMap, HashSet};

pub const REPAIR_PROTOCOL: &str = "E2-witness-repair-v2";
pub const WITNESS_VERSION: i64 = 7;
pub const REPAIR_INSTRUCTION_VERSION: i64 = 8;
pub const SEALED_REPAIR_PROTOCOL: &str = "E2-witness-sealed-repair-v1";
pub const SEALED_REPAIR_INSTRUCTION_VERSION: i64 = 9;
pub const WITNESS_BASE: u32 = META_BASE + 0x100;
pub const WITNESS_ROLES: [u32; 13] = [
    E_CORE,
    E_GRAFT,
    E_MEMBER,
    E_SLOT_BASE,
    E_SLOT_BASE + 1,
    E_SLOT_BASE + 2,
    E_SLOT_BASE + 3,
    E_SLOT_BASE + 4,
    E_SLOT_BASE + 5,
    E_SLOT_BASE + 6,
    E_SLOT_BASE + 7,
    E_HASLABEL,
    E_GUARDS,
];

pub fn encode_witness_family(
    s: &mut HypergraphStore,
    core: &CandidateRule,
    proposal: &CandidateRule,
    active: bool,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    // Validate aggregate witness representation on an isolated store first.
    let mut test = HypergraphStore::new();
    let checked = add_witnesses(&mut test, core, proposal, active, limits)?;
    read_witness_program(&test.snapshot(), checked.root, limits)?;
    add_witnesses(s, core, proposal, active, limits)
}
fn add_witnesses(
    s: &mut HypergraphStore,
    core: &CandidateRule,
    proposal: &CandidateRule,
    active: bool,
    limits: &MetaLimits,
) -> Result<EncodedRegion, Validator> {
    let mut r = encode_family(s, core, proposal, active, limits)?;
    let pairs: Vec<_> = r
        .edges
        .iter()
        .filter_map(|&id| {
            let e = s.get_edge(id).unwrap();
            (e.nodes.len() == 2).then(|| (e.label.0, e.nodes[0], e.nodes[1]))
        })
        .collect();
    if r.edges.len() + pairs.len() > limits.max_region_edges {
        return Err(Validator::Region);
    }
    for (label, a, b) in pairs {
        let role = WITNESS_ROLES
            .iter()
            .position(|&v| v == label)
            .ok_or(Validator::Bounds)?;
        r.edges.push(s.add_edge(HyperEdge::new(
            Label(WITNESS_BASE + role as u32),
            [r.root, a, b].into_iter().collect(),
        )));
    }
    s.get_node_mut(r.root)
        .unwrap()
        .set_property(K_ENCODING, PropertyValue::Int(WITNESS_VERSION));
    Ok(r)
}

pub fn read_witness_program(
    s: &GraphSnapshot<'_>,
    root: NodeId,
    limits: &MetaLimits,
) -> Result<RuleProgram, Validator> {
    let r = walk_family(s, root, limits)?;
    if s.get_node(root).and_then(|n| n.get_property(K_ENCODING))
        != Some(PropertyValue::Int(WITNESS_VERSION))
    {
        return Err(Validator::Region);
    }
    let mut witnesses = HashSet::new();
    let mut consumed = HashSet::new();
    let mut payloads = Vec::new();
    let mut work = 0usize;
    for &id in &r.edges {
        let w = s.get_edge(id).unwrap();
        if !(WITNESS_BASE..WITNESS_BASE + WITNESS_ROLES.len() as u32).contains(&w.label.0) {
            continue;
        }
        if w.nodes.len() != 3 || w.nodes[0] != root {
            return Err(Validator::Members);
        }
        let role = WITNESS_ROLES[(w.label.0 - WITNESS_BASE) as usize];
        let (a, b) = (w.nodes[1], w.nodes[2]);
        if !witnesses.insert((role, a, b)) {
            return Err(Validator::Members);
        }
        let mut matched = None;
        for &eid in s.edges_incident_to(a) {
            work += 1;
            if work > limits.max_region_edges.saturating_mul(16) {
                return Err(Validator::Region);
            }
            let e = s.get_edge(eid).ok_or(Validator::Region)?;
            let correct = e.label.0 == role
                && (e.nodes.as_slice() == [a, b] || e.nodes.as_slice() == [a, b, root]);
            if correct && matched.replace(eid).is_some_and(|old| old != eid) {
                return Err(Validator::Members);
            }
        }
        let eid = matched.ok_or(Validator::Members)?;
        if !consumed.insert(eid) {
            return Err(Validator::Members);
        }
        // Only a present correctly typed payload is projected; no missing edge is supplied.
        payloads.push((role, a, b));
    }
    if witnesses.is_empty() {
        return Err(Validator::Members);
    }
    let mut projection = HypergraphStore::new();
    let mut map = HashMap::new();
    for &id in &r.nodes {
        let mut data = s.get_node(id).ok_or(Validator::Region)?.clone();
        if id == root {
            data.set_property(K_ENCODING, PropertyValue::Int(3));
        }
        map.insert(id, projection.add_node(data));
    }
    for (role, a, b) in payloads {
        projection.add_edge(HyperEdge::new(
            Label(role),
            [map[&a], map[&b]].into_iter().collect(),
        ));
    }
    for &id in &r.edges {
        let e = s.get_edge(id).unwrap();
        if consumed.contains(&id)
            || (WITNESS_BASE..WITNESS_BASE + WITNESS_ROLES.len() as u32).contains(&e.label.0)
        {
            continue;
        }
        if e.label.0 != E_BECOMES || e.nodes.len() != 3 {
            return Err(Validator::Bounds);
        }
        projection.add_edge(HyperEdge::new(
            e.label,
            e.nodes.iter().map(|n| map[n]).collect(),
        ));
    }
    read_program(&projection.snapshot(), map[&root], limits)
}

pub fn compile_repair_operations(
    operation: MetaOperation,
    limits: &MetaLimits,
) -> Result<Vec<CompiledRule>, Validator> {
    compile_witness_operations(operation, limits, WITNESS_VERSION, operation.instruction_version(), operation == MetaOperation::RepairSealed)
}

pub(crate) fn compile_witness_operations(
    operation: MetaOperation,
    limits: &MetaLimits,
    family_version: i64,
    instruction_version: i64,
    ordered: bool,
) -> Result<Vec<CompiledRule>, Validator> {
    if !matches!(
        operation,
        MetaOperation::Seal | MetaOperation::Repair | MetaOperation::WitnessActivate | MetaOperation::RepairSealed
    ) {
        return Err(Validator::Compilability);
    }
    let roles: Vec<_> = if operation == MetaOperation::WitnessActivate {
        vec![1]
    } else {
        (0..WITNESS_ROLES.len()).collect()
    };
    let mut out = Vec::new();
    let variants: Vec<_> = roles
        .into_iter()
        .flat_map(|ix| {
            let role = WITNESS_ROLES[ix];
            let pairs = match role {
                E_CORE | E_GRAFT => vec![(L_RULE_ROOT, L_ALLELE_ROOT)],
                E_MEMBER => vec![(L_SIDE_L, L_PAT_EDGE), (L_SIDE_R, L_PAT_EDGE)],
                E_HASLABEL => vec![(L_VAR, L_LABEL_LIT)],
                E_GUARDS => vec![(L_VAR, L_GUARD)],
                _ => vec![(L_PAT_EDGE, L_VAR)],
            };
            pairs.into_iter().map(move |(a, b)| (ix, a, b))
        })
        .collect();
    for (ix, a_label, b_label) in variants {
        let role = WITNESS_ROLES[ix];
        let sealed = operation == MetaOperation::RepairSealed;
        // Family-root links must actually share the root, not borrow another
        // same-labelled root through the non-injective matcher.
        let a = if ordered && matches!(role, E_CORE | E_GRAFT) {
            "root"
        } else {
            "a"
        };
        let mut r = Rule {
            name: format!("encoded_meta_{}_{ix}_{a_label}", operation.name()),
            priority: if operation == MetaOperation::WitnessActivate {
                110
            } else {
                100
            },
            ..Rule::default()
        };
        let names = if a == "root" {
            vec!["op", "root", "b"]
        } else {
            vec!["op", "root", "a", "b"]
        };
        r.lhs.nodes = names.into_iter().map(str::to_owned).collect();
        r.rhs.nodes = r.lhs.nodes.clone();
        r.lhs
            .node_labels
            .insert("op".into(), Label(L_META_INSTRUCTION));
        r.lhs.node_labels.insert("root".into(), Label(L_RULE_ROOT));
        // Hyperedge matching permutes endpoints. Typed roles orient every
        // witness/payload binding before a paid RHS is emitted.
        r.lhs.node_labels.insert(a.into(), Label(a_label));
        r.lhs.node_labels.insert("b".into(), Label(b_label));
        r.lhs
            .edges
            .push(vec!["root".into(), a.into(), "b".into()]);
        r.lhs.edge_labels.insert(0, Label(WITNESS_BASE + ix as u32));
        r.rhs.edges = r.lhs.edges.clone();
        r.rhs.edge_labels = r.lhs.edge_labels.clone();
        if operation != MetaOperation::WitnessActivate {
            r.lhs.edges.push(if sealed {
                vec![a.into(), "b".into(), "root".into()]
            } else {
                vec![a.into(), "b".into()]
            });
            r.lhs.edge_labels.insert(
                1,
                Label(if operation == MetaOperation::Seal {
                    role
                } else {
                    0
                }),
            );
            r.rhs
                .edges
                .push(vec![a.into(), "b".into(), "root".into()]);
            r.rhs.edge_labels.insert(1, Label(role));
        }
        for (var, key, op, value) in [
            (
                "op",
                K_ENCODING,
                PropertyOp::Eq,
                PropertyValue::Int(instruction_version),
            ),
            (
                "op",
                K_OPCODE,
                PropertyOp::Eq,
                PropertyValue::Int(operation.code()),
            ),
            ("op", K_FUEL, PropertyOp::Gte, PropertyValue::Float(1.0)),
            ("op", K_FUEL, PropertyOp::Lte, PropertyValue::Float(32.0)),
            (
                "root",
                K_ENCODING,
                PropertyOp::Eq,
                PropertyValue::Int(family_version),
            ),
        ] {
            r.lhs_node_property_guards.push(NodePropertyGuard {
                var: var.into(),
                property: key,
                op,
                value,
            });
        }
        r.lhs_node_degree_compares.push(NodeDegreeCompare {
            var: "op".into(),
            op: PropertyOp::Eq,
            value: 0,
            offset: None,
        });
        r.rhs_property_effects.push(PropertyEffectDecl {
            var: "op".into(),
            key: K_FUEL,
            op: EffectOpDecl::Subtract {
                value: PropertyValue::Float(1.0),
            },
        });
        if operation == MetaOperation::WitnessActivate {
            r.lhs_node_property_guards.push(NodePropertyGuard {
                var: "root".into(),
                property: K_GRAFT_MODE,
                op: PropertyOp::Eq,
                value: PropertyValue::Int(0),
            });
            r.rhs_property_effects.push(PropertyEffectDecl {
                var: "root".into(),
                key: K_GRAFT_MODE,
                op: EffectOpDecl::Set {
                    value: PropertyValue::Int(1),
                },
            });
        }
        let mut c = compile_rule(&r).map_err(|_| Validator::Compilability)?;
        if c.pattern.n_vars > limits.max_vars.min(8)
            || c.template.n_rhs_vars > limits.max_vars.min(8)
            || c.pattern.edges.len() > limits.max_pattern_edges
            || c.template.rhs_edges.len() > limits.max_pattern_edges
            || limits.max_arity < 3
        {
            return Err(Validator::Bounds);
        }
        c.meta_visible = true;
        c.pattern.node_label_ceiling = None;
        if ordered {
            // Never canonicalize a malformed witness or reordered payload.
            // This native constraint is absent from all previous productions.
            for edge in &mut c.pattern.edges {
                edge.ordered = true;
            }
        }
        out.push(c);
    }
    Ok(out)
}
