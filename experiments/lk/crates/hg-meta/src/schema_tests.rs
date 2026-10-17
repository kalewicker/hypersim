//! Region-reader tests: D1 determinism, structural errors, worked example.

use super::schema::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use smallvec::smallvec;

/// Mutable handle set while building one region.
struct Region {
    root: NodeId,
    side_l: NodeId,
    side_r: NodeId,
}

/// Seed an empty region: RULE_ROOT + BECOMES + two sides.
fn seed_region(s: &mut HypergraphStore, priority: i32) -> Region {
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

/// Add a pattern edge to a side with vars in given slots. Returns pe id.
fn add_pat_edge(
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

fn add_var(s: &mut HypergraphStore) -> NodeId {
    s.add_node(NodeData::new(Label(L_VAR), 0))
}

fn add_label_lit(s: &mut HypergraphStore, v: NodeId, label: u32) {
    let mut lit = NodeData::new(Label(L_LABEL_LIT), 0);
    lit.set_property(K_LABEL_VALUE, PropertyValue::Int(label as i64));
    let lit = s.add_node(lit);
    s.add_edge(HyperEdge::new(Label(E_HASLABEL), smallvec![v, lit]));
}

/// Build the plan's §3.4 worked example `triangle_subdivide`
/// [a,b,c] → [a,b,d],[b,c,d],[a,c,d], priority 0.
///
/// `reverse_insertion` builds the SAME region but adds nodes/edges in
/// reverse order — the D1 fixture's second store.
fn build_triangle(s: &mut HypergraphStore, reverse_insertion: bool) -> Vec<NodeId> {
    // Order of node creation differs between the two stores; the region
    // content is identical.
    let mut order: Vec<u8> = vec![0, 1, 2, 3]; // a b c d
    if reverse_insertion {
        order.reverse();
    }
    let vars: Vec<NodeId> = order.iter().map(|_| add_var(s)).collect();
    // Map logical var (a,b,c,d) → created id by position of its index in order.
    let at = |logical: usize| -> NodeId {
        let pos = order.iter().position(|&x| x == logical as u8).unwrap();
        vars[pos]
    };
    let (a, b, c, d) = (at(0), at(1), at(2), at(3));

    let reg = seed_region(s, 0);
    // LHS [a,b,c]
    add_pat_edge(s, &reg, true, &[(a, 0), (b, 1), (c, 2)]);
    // RHS [a,b,d] [b,c,d] [a,c,d]
    add_pat_edge(s, &reg, false, &[(a, 0), (b, 1), (d, 2)]);
    add_pat_edge(s, &reg, false, &[(b, 0), (c, 1), (d, 2)]);
    add_pat_edge(s, &reg, false, &[(a, 0), (c, 1), (d, 2)]);
    vec![reg.root]
}

#[test]
fn d1_different_insertion_orders_read_identically() {
    let mut s1 = HypergraphStore::new();
    let roots1 = build_triangle(&mut s1, false);
    let mut s2 = HypergraphStore::new();
    let roots2 = build_triangle(&mut s2, true);

    let snap1 = s1.snapshot();
    let snap2 = s2.snapshot();

    // Root enumeration must find exactly one root in each, and the read
    // must be identical despite different insertion orders / ffi keys.
    assert_eq!(find_rule_roots(&snap1).len(), 1);
    assert_eq!(find_rule_roots(&snap2).len(), 1);

    let r1 = read_region(&snap1, roots1[0]).expect("region 1 reads");
    let r2 = read_region(&snap2, roots2[0]).expect("region 2 reads");
    assert_eq!(r1, r2, "D1 violated: insertion order changed the reading");

    // And it is the worked example: LHS 1 edge arity-3, RHS 3 edges,
    // 4 vars, none labeled, priority 0.
    assert_eq!(r1.priority, 0);
    assert_eq!(r1.lhs.len(), 1);
    assert_eq!(r1.rhs.len(), 3);
    assert_eq!(r1.n_vars(), 4);
    assert!(r1.var_labels.iter().all(|l| l.is_none()));
    assert_eq!(
        r1.lhs[0].slots.len(),
        3,
        "slot compaction keeps all three LHS slots"
    );
}

#[test]
fn reader_errors_on_missing_becomes() {
    let mut s = HypergraphStore::new();
    let _ = s.add_node(NodeData::new(Label(L_RULE_ROOT), 0)); // bare root, no sides
    let snap = s.snapshot();
    let root = find_rule_roots(&snap)[0];
    assert!(read_region(&snap, root).is_err());
}

#[test]
fn reader_rejects_var_on_both_sides_via_same_pat_edge() {
    // One PAT_EDGE wired into BOTH sides: survive/create/delete becomes
    // ambiguous — reader must reject rather than guess.
    let mut s = HypergraphStore::new();
    let reg = seed_region(&mut s, 0);
    let v = add_var(&mut s);
    let pe = s.add_node(NodeData::new(Label(L_PAT_EDGE), 0));
    s.add_edge(HyperEdge::new(Label(E_MEMBER), smallvec![reg.side_l, pe]));
    s.add_edge(HyperEdge::new(Label(E_MEMBER), smallvec![reg.side_r, pe]));
    s.add_edge(HyperEdge::new(Label(E_SLOT_BASE), smallvec![pe, v]));
    let snap = s.snapshot();
    assert!(
        read_region(&snap, reg.root).is_err(),
        "both-sides MEMBER must error"
    );
}

#[test]
fn slot_compaction_leaves_no_hole() {
    // Slots 0 and 2 present, 1 deleted: compacted to positions 0,1.
    let mut s = HypergraphStore::new();
    let reg = seed_region(&mut s, 5);
    let v0 = add_var(&mut s);
    let v2 = add_var(&mut s);
    let pe = s.add_node(NodeData::new(Label(L_PAT_EDGE), 0));
    s.add_edge(HyperEdge::new(Label(E_MEMBER), smallvec![reg.side_l, pe]));
    s.add_edge(HyperEdge::new(Label(E_SLOT_BASE), smallvec![pe, v0]));
    s.add_edge(HyperEdge::new(Label(E_SLOT_BASE + 2), smallvec![pe, v2]));
    let snap = s.snapshot();
    let r = read_region(&snap, reg.root).expect("reads");
    assert_eq!(r.lhs.len(), 1);
    assert_eq!(
        r.lhs[0].slots,
        vec![0usize, 1usize],
        "slots renumbered densely after deletion"
    );
    assert_eq!(r.n_vars(), 2);
}

#[test]
fn label_constraint_reads_through_haslabel() {
    let mut s = HypergraphStore::new();
    let reg = seed_region(&mut s, -3);
    let va = add_var(&mut s);
    let vb = add_var(&mut s);
    add_pat_edge(&mut s, &reg, true, &[(va, 0), (vb, 1)]);
    add_label_lit(&mut s, va, 7);
    let snap = s.snapshot();
    let r = read_region(&snap, reg.root).expect("reads");
    assert_eq!(r.var_labels, vec![Some(7), None]);
    assert_eq!(
        r.priority, -3,
        "negative priority survives the property round-trip"
    );
}
