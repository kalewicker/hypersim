//! `RuleSpec` -> rule-subgraph in the RuleStore.

use crate::schema::*;
use crate::store_ext::*;
use hg_core::{HypergraphStore, NodeId, PropertyValue};

/// Monotone counter carried by the caller; stamped on every new RuleRoot as
/// K_SEQ, the canonical-enumeration tie-break.
pub fn install_rule(s: &mut HypergraphStore, spec: &RuleSpec, seq: &mut i64) -> NodeId {
    let root = add_rule_node(
        s,
        RULE_ROOT,
        vec![
            (K_PRIORITY, PropertyValue::Int(spec.priority as i64)),
            (K_NVARS, PropertyValue::Int(spec.n_vars as i64)),
            (K_RESOURCE, PropertyValue::Int(0)),
            (K_SEQ, PropertyValue::Int(*seq)),
        ],
    );
    *seq += 1;

    for (i, vl) in spec.var_labels.iter().enumerate() {
        if let Some(lab) = vl {
            let p = add_rule_node(
                s,
                PNODE,
                vec![
                    (K_VAR_ID, PropertyValue::Int(i as i64)),
                    (K_LABEL_CONSTRAINT, PropertyValue::Int(*lab as i64)),
                ],
            );
            add_role_edge(s, root, p, HAS_PNODE);
        }
    }
    for e in &spec.pedges {
        let mut props = vec![(K_ARITY, PropertyValue::Int(e.len() as i64))];
        for (i, r) in e.iter().enumerate() {
            props.push((K_EP0 + i as u32, PropertyValue::Int(*r as i64)));
        }
        let pe = add_rule_node(s, PEDGE, props);
        add_role_edge(s, root, pe, HAS_PEDGE);
    }
    for g in &spec.guards {
        let gn = add_rule_node(
            s,
            GUARD,
            vec![
                (K_GVAR, PropertyValue::Int(g.var as i64)),
                (K_GKEY, PropertyValue::Int(g.key as i64)),
                (K_GOP, PropertyValue::Int(g.op as i64)),
                (K_GVAL, g.val),
            ],
        );
        add_role_edge(s, root, gn, HAS_GUARD);
    }
    for d in &spec.deletes {
        let dn = add_rule_node(s, DELETE, vec![(K_VAR_ID, PropertyValue::Int(*d as i64))]);
        add_role_edge(s, root, dn, HAS_DELETE);
    }
    for (v, l) in &spec.relabels {
        let rn = add_rule_node(
            s,
            RELABEL,
            vec![
                (K_VAR_ID, PropertyValue::Int(*v as i64)),
                (K_NEW_LABEL, PropertyValue::Int(*l as i64)),
            ],
        );
        add_role_edge(s, root, rn, HAS_RELABEL);
    }
    for (k, l) in spec.creates.iter().enumerate() {
        let cn = add_rule_node(
            s,
            CREATE,
            vec![
                (K_CREATE_INDEX, PropertyValue::Int(k as i64)),
                (K_CREATE_LABEL, PropertyValue::Int(*l as i64)),
            ],
        );
        add_role_edge(s, root, cn, HAS_CREATE);
    }
    for e in &spec.redges {
        let mut props = vec![(K_ARITY, PropertyValue::Int(e.len() as i64))];
        for (i, r) in e.iter().enumerate() {
            props.push((K_EP0 + i as u32, PropertyValue::Int(*r as i64)));
        }
        let re = add_rule_node(s, REDGE, props);
        add_role_edge(s, root, re, HAS_REDGE);
    }
    root
}
