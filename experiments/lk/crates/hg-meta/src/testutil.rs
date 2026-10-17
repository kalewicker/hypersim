//! Shared region-building helpers for `hg-meta` tests (cfg(test) only).
//!
//! These construct E1 regions directly against `HypergraphStore`, mirroring
//! the plan's §3.4 worked example, with knobs for insertion order so the D1
//! and RCF fixtures can build the SAME rule under different construction
//! sequences.

use crate::schema::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use smallvec::smallvec;

/// Mutable handle set while building one region.
pub struct Region {
    pub root: NodeId,
    pub side_l: NodeId,
    pub side_r: NodeId,
}

/// Seed an empty region: RULE_ROOT (+ priority) + BECOMES + two sides.
pub fn seed_region(s: &mut HypergraphStore, priority: i32) -> Region {
    let mut root = NodeData::new(Label(L_RULE_ROOT), 0);
    root.set_property(K_PRIORITY, PropertyValue::Int(priority as i64));
    let r = s.add_node(root);
    let l = s.add_node(NodeData::new(Label(L_SIDE_L), 0));
    let rt = s.add_node(NodeData::new(Label(L_SIDE_R), 0));
    s.add_edge(HyperEdge::new(Label(E_BECOMES), smallvec![r, l, rt]));
    Region {
        root: r,
        side_l: l,
        side_r: rt,
    }
}

pub fn add_var(s: &mut HypergraphStore) -> NodeId {
    s.add_node(NodeData::new(Label(L_VAR), 0))
}

/// Add a pattern edge to a side with `(var, slot_k)` pairs. Returns pe id.
pub fn add_pat_edge(
    s: &mut HypergraphStore,
    reg: &Region,
    lhs: bool,
    var_slots: &[(NodeId, usize)],
) -> NodeId {
    let pe = s.add_node(NodeData::new(Label(L_PAT_EDGE), 0));
    let side = if lhs { reg.side_l } else { reg.side_r };
    s.add_edge(HyperEdge::new(Label(E_MEMBER), smallvec![side, pe]));
    for (v, k) in var_slots {
        s.add_edge(HyperEdge::new(
            Label(E_SLOT_BASE + *k as u32),
            smallvec![pe, *v],
        ));
    }
    pe
}

pub fn add_label_lit(s: &mut HypergraphStore, v: NodeId, label: u32) {
    let mut lit = NodeData::new(Label(L_LABEL_LIT), 0);
    lit.set_property(K_LABEL_VALUE, PropertyValue::Int(label as i64));
    let lit = s.add_node(lit);
    s.add_edge(HyperEdge::new(Label(E_HASLABEL), smallvec![v, lit]));
}

/// Build the plan's §3.4 worked example `triangle_subdivide`
/// `[a,b,c] → [a,b,d],[b,c,d],[a,c,d]`, priority 0.
///
/// Knobs vary the CONSTRUCTION sequence without varying the rule:
/// - `reverse_vars`: create the four var nodes in reverse order (different
///   NodeIds per logical variable).
/// - `reverse_pes`: add the three RHS pattern edges in reverse order
///   (different allocation order among identical-content PAT_EDGE nodes,
///   which is exactly the axis `ffi` tie-breaking in the reader exposes).
///
/// The read-back `CandidateRule` may therefore differ in edge order between
/// knob settings; anything downstream of RCF must not care. That is the
/// point of having both knobs.
pub fn build_triangle(
    s: &mut HypergraphStore,
    reverse_vars: bool,
    reverse_pes: bool,
) -> Vec<NodeId> {
    let mut order: Vec<u8> = vec![0, 1, 2, 3]; // logical a b c d
    if reverse_vars {
        order.reverse();
    }
    let vars: Vec<NodeId> = order.iter().map(|_| add_var(s)).collect();
    let at = |logical: usize| -> NodeId {
        let pos = order.iter().position(|&x| x == logical as u8).unwrap();
        vars[pos]
    };
    let (a, b, c, d) = (at(0), at(1), at(2), at(3));

    let reg = seed_region(s, 0);
    add_pat_edge(s, &reg, true, &[(a, 0), (b, 1), (c, 2)]);
    let mut rhs: Vec<Vec<(NodeId, usize)>> = vec![
        vec![(a, 0), (b, 1), (d, 2)],
        vec![(b, 0), (c, 1), (d, 2)],
        vec![(a, 0), (c, 1), (d, 2)],
    ];
    if reverse_pes {
        rhs.reverse();
    }
    for slots in rhs {
        add_pat_edge(s, &reg, false, &slots);
    }
    vec![reg.root]
}
