//! Rule-subgraph -> `RuleSpec` -> engine-native `CompiledRule`.
//!
//! Well-formedness lives HERE, not in the mutators (junk-DNA discipline): a
//! subgraph that fails any validity check decodes to `None`, stays in the
//! store, still pays rent, and is counted (`decode_failures`). Never panics.

use crate::schema::*;
use crate::store_ext::*;
use hg_core::{HypergraphStore, Label, NodeId, PropertyValue};
use hg_rules::{compile_rule, CompiledRule, PatternGraph, Rule};

fn get_prop(s: &HypergraphStore, n: NodeId, key: u32) -> Option<PropertyValue> {
    s.get_node(n)?.get_property(key)
}

/// Decode one rule-subgraph. `None` = malformed (inert junk DNA).
pub fn decode_spec(s: &HypergraphStore, root: NodeId, caps: &Caps) -> Option<RuleSpec> {
    let n_vars_i = get_int(s, root, K_NVARS)?;
    if n_vars_i < 1 || n_vars_i > caps.lhs_max_nodes as i64 {
        return None;
    }
    let n_vars = n_vars_i as u8;
    let priority = get_int(s, root, K_PRIORITY)? as i32;

    // Var label constraints. Out-of-range PNodes are ignored (v2 behavior:
    // a stale constraint left behind by drop_lhs_var is inert, not fatal).
    let mut var_labels = vec![None; n_vars as usize];
    for pn in children(s, root, HAS_PNODE) {
        let vid = get_int(s, pn, K_VAR_ID).unwrap_or(-1);
        if vid >= 0 && (vid as u8) < n_vars {
            if let Some(l) = get_int(s, pn, K_LABEL_CONSTRAINT) {
                var_labels[vid as usize] = Some(l as u32);
            }
        }
    }

    // Ordered endpoint list of a PEdge/REdge node; None if malformed.
    let read_refs = |en: NodeId, max_ref: i64| -> Option<Vec<u8>> {
        let arity = get_int(s, en, K_ARITY)?;
        if arity < 2 || arity > caps.max_arity as i64 {
            return None;
        }
        let mut refs = Vec::with_capacity(arity as usize);
        for i in 0..arity {
            let r = get_int(s, en, K_EP0 + i as u32)?;
            if r < 0 || r >= max_ref {
                return None;
            }
            refs.push(r as u8);
        }
        Some(refs)
    };

    let mut pedges = Vec::new();
    for pe in children(s, root, HAS_PEDGE) {
        pedges.push(read_refs(pe, n_vars as i64)?);
    }
    if pedges.len() > caps.lhs_max_edges {
        return None;
    }

    let mut guards = Vec::new();
    for gn in children(s, root, HAS_GUARD) {
        let g = GuardSpec {
            var: get_int(s, gn, K_GVAR).filter(|v| *v >= 0 && *v < n_vars as i64)? as u8,
            key: get_int(s, gn, K_GKEY)? as u32,
            op: get_int(s, gn, K_GOP).filter(|o| (0..=2).contains(o))? as u8,
            val: get_prop(s, gn, K_GVAL)?,
        };
        g.property_op()?;
        guards.push(g);
    }

    let mut deletes = Vec::new();
    for dn in children(s, root, HAS_DELETE) {
        let v = get_int(s, dn, K_VAR_ID)?;
        if v < 0 || v >= n_vars as i64 {
            return None;
        }
        deletes.push(v as u8);
    }
    deletes.sort_unstable();
    deletes.dedup();

    let mut relabels = Vec::new();
    for rn in children(s, root, HAS_RELABEL) {
        let v = get_int(s, rn, K_VAR_ID)?;
        let l = get_int(s, rn, K_NEW_LABEL)?;
        if v < 0 || v >= n_vars as i64 {
            return None;
        }
        // Relabel of a deleted var is inert, not fatal (deletion wins —
        // recon note (a)); drop it here so the compiled rule is clean.
        if !deletes.contains(&(v as u8)) {
            relabels.push((v as u8, l as u32));
        }
    }
    relabels.sort_unstable();
    relabels.dedup_by_key(|(v, _)| *v); // one relabel per var (first in sorted order wins)

    // Creates, ordered by index; indices must be exactly 0..len (junk otherwise).
    let mut cvec: Vec<(i64, u32)> = Vec::new();
    for cn in children(s, root, HAS_CREATE) {
        cvec.push((
            get_int(s, cn, K_CREATE_INDEX)?,
            get_int(s, cn, K_CREATE_LABEL)? as u32,
        ));
    }
    cvec.sort();
    if cvec.len() > caps.rhs_max_create {
        return None;
    }
    for (i, (k, _)) in cvec.iter().enumerate() {
        if *k != i as i64 {
            return None;
        }
    }
    let creates: Vec<u32> = cvec.into_iter().map(|x| x.1).collect();

    let n_refs = n_vars as i64 + creates.len() as i64;
    let mut redges = Vec::new();
    for en in children(s, root, HAS_REDGE) {
        let refs = read_refs(en, n_refs)?;
        // Refs to deleted vars make the rule malformed (junk DNA).
        if refs.iter().any(|r| deletes.contains(r)) {
            return None;
        }
        redges.push(refs);
    }

    Some(RuleSpec {
        priority,
        n_vars,
        var_labels,
        pedges,
        guards,
        deletes,
        relabels,
        creates,
        redges,
    })
}

/// `RuleSpec` -> engine-native rule, via `hg_rules::Rule` + `compile_rule`.
/// LHS vars are named `v0..`, creates `c0..`. LHS pattern edges are consumed
/// by the engine on apply; survivors are every var not in `deletes`.
pub fn spec_to_compiled(spec: &RuleSpec, name: &str) -> Option<CompiledRule> {
    let mut lhs = PatternGraph::new();
    for i in 0..spec.n_vars {
        let vn = format!("v{i}");
        if let Some(l) = spec.var_labels[i as usize] {
            lhs.node_labels.insert(vn.clone(), Label(l));
        }
        lhs.nodes.push(vn);
    }
    for e in &spec.pedges {
        lhs.edges.push(e.iter().map(|r| format!("v{r}")).collect());
    }

    let mut rhs = PatternGraph::new();
    let mut rhs_node_labels = ahash_map();
    for i in 0..spec.n_vars {
        if spec.deletes.contains(&i) {
            continue;
        }
        let vn = format!("v{i}");
        if let Some((_, l)) = spec.relabels.iter().find(|(v, _)| *v == i) {
            rhs_node_labels.insert(vn.clone(), Label(*l));
        }
        rhs.nodes.push(vn);
    }
    for (k, l) in spec.creates.iter().enumerate() {
        let cn = format!("c{k}");
        rhs.node_labels.insert(cn.clone(), Label(*l));
        rhs.nodes.push(cn);
    }
    let ref_name = |r: u8| -> String {
        if r < spec.n_vars {
            format!("v{r}")
        } else {
            format!("c{}", r - spec.n_vars)
        }
    };
    for e in &spec.redges {
        rhs.edges.push(e.iter().map(|r| ref_name(*r)).collect());
    }

    let mut rule = Rule {
        name: name.to_string(),
        priority: spec.priority,
        ..Rule::default()
    };
    rule.lhs = lhs;
    rule.rhs = rhs;
    rule.rhs_node_labels = rhs_node_labels;
    for g in &spec.guards {
        rule.lhs_node_property_guards
            .push(hg_rules::NodePropertyGuard {
                var: format!("v{}", g.var),
                property: g.key,
                op: g.property_op()?,
                value: g.val,
            });
    }
    compile_rule(&rule).ok()
}

fn ahash_map() -> ahash::AHashMap<String, Label> {
    ahash::AHashMap::default()
}

/// One decoded, live rule.
pub struct DecodedRule {
    pub root: NodeId,
    pub spec: RuleSpec,
    pub class_hash: u64,
    pub compiled: CompiledRule,
}

/// Decode every RuleRoot; canonical order = (class_hash, K_SEQ). Returns the
/// decoded set plus the number of malformed (inert) roots.
pub fn decode_all(s: &HypergraphStore, caps: &Caps) -> (Vec<DecodedRule>, usize) {
    let mut out = Vec::new();
    let mut failures = 0usize;
    for root in rule_roots(s) {
        let Some(spec) = decode_spec(s, root, caps) else {
            failures += 1;
            continue;
        };
        let ch = spec.class_hash();
        let Some(compiled) = spec_to_compiled(&spec, &format!("r{ch:016x}")) else {
            failures += 1;
            continue;
        };
        out.push(DecodedRule {
            root,
            spec,
            class_hash: ch,
            compiled,
        });
    }
    out.sort_by_key(|d| (d.class_hash, get_int(s, d.root, K_SEQ).unwrap_or(i64::MAX)));
    (out, failures)
}
