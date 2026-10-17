//! E1 region installation: CandidateRule → graph (the reader's inverse).
//!
//! Used by arm A2/A3's meta-rewrites and by the R3 fixtures. Deterministic
//! given insertion order; regions installed from identical candidates in
//! different orders read back identically (D1 discipline applies to the
//! reader side, so the writer is free to allocate in any order — but tests
//! pin both directions).

use crate::schema::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use smallvec::smallvec;

/// Install one rule region; returns the RULE_ROOT id.
pub fn install_region(s: &mut HypergraphStore, cand: &CandidateRule) -> NodeId {
    let mut root = NodeData::new(Label(L_RULE_ROOT), 0);
    root.set_property(K_PRIORITY, PropertyValue::Int(cand.priority as i64));
    let root = s.add_node(root);
    let l = s.add_node(NodeData::new(Label(L_SIDE_L), 0));
    let r = s.add_node(NodeData::new(Label(L_SIDE_R), 0));
    s.add_edge(HyperEdge::new(Label(E_BECOMES), smallvec![root, l, r]));

    // Var nodes first (dense ids already), then label lits.
    let vars: Vec<NodeId> = (0..cand.n_vars())
        .map(|_| s.add_node(NodeData::new(Label(L_VAR), 0)))
        .collect();
    for (v, label) in cand.var_labels.iter().enumerate() {
        if let Some(lv) = label {
            let mut lit = NodeData::new(Label(L_LABEL_LIT), 0);
            lit.set_property(K_LABEL_VALUE, PropertyValue::Int(*lv as i64));
            let lit = s.add_node(lit);
            s.add_edge(HyperEdge::new(Label(E_HASLABEL), smallvec![vars[v], lit]));
        }
    }

    // Pattern edges per side, slots at their compacted positions.
    let sides: [(bool, NodeId); 2] = [(true, l), (false, r)];
    for (is_lhs, side) in sides {
        let edges = if is_lhs { &cand.lhs } else { &cand.rhs };
        for e in edges {
            let pe = s.add_node(NodeData::new(Label(L_PAT_EDGE), 0));
            s.add_edge(HyperEdge::new(Label(E_MEMBER), smallvec![side, pe]));
            for (k, v) in e.slots.iter().enumerate() {
                s.add_edge(HyperEdge::new(
                    Label(E_SLOT_BASE + k as u32),
                    smallvec![pe, vars[*v]],
                ));
            }
        }
    }

    // E1-G guards: GUARD node per guard, wired to its var by E_GUARDS.
    for g in &cand.guards {
        let mut gn = NodeData::new(Label(L_GUARD), 0);
        gn.set_property(K_GUARD_KEY, PropertyValue::Int(g.key as i64));
        gn.set_property(K_GUARD_OP, PropertyValue::Int(g.op as i64));
        gn.set_property(K_GUARD_VAL, g.val);
        let gn = s.add_node(gn);
        s.add_edge(HyperEdge::new(Label(E_GUARDS), smallvec![vars[g.var], gn]));
    }

    // E1-V var guards: one VAR_GUARD node per vars_distinct, wired to the
    // rule root by E_VAR_GUARD; var indices under K_VGUARD_VARS + i.
    for vg in &cand.var_guards {
        let mut gn = NodeData::new(Label(L_GUARD), 0);
        for (i, v) in vg.vars.iter().enumerate() {
            gn.set_property(K_VGUARD_VARS + i as u32, PropertyValue::Int(*v as i64));
        }
        let gn = s.add_node(gn);
        s.add_edge(HyperEdge::new(Label(E_VAR_GUARD), smallvec![root, gn]));
    }

    // E2-lite effects: one EFFECT node per effect, wired to its target var
    // by E_EFFECT; source-var ops add an E_GUARDS edge from the effect node
    // to the source var (the band's generic reference role).
    for e in &cand.effects {
        let mut en = NodeData::new(Label(L_EFFECT), 0);
        en.set_property(K_EFFECT_KEY, PropertyValue::Int(e.key as i64));
        en.set_property(K_EFFECT_OP, PropertyValue::Int(e.op as i64));
        if let Some(v) = e.val {
            en.set_property(K_EFFECT_VAL, v);
        } else if let Some(sk) = e.source_key {
            en.set_property(K_EFFECT_VAL, PropertyValue::Int(sk as i64));
        }
        let en = s.add_node(en);
        s.add_edge(HyperEdge::new(
            Label(E_EFFECT),
            smallvec![vars[e.target], en],
        ));
        if let Some(sv) = e.source {
            s.add_edge(HyperEdge::new(Label(E_GUARDS), smallvec![en, vars[sv]]));
        }
    }

    root
}
