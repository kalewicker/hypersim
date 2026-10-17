use hg_core::{HyperEdge, HypergraphStore, Label, PropertyValue};
use hg_meta::*;

fn identity() -> CandidateRule {
    CandidateRule {
        labels: vec![None; 3],
        lhs: vec![vec![0, 1, 2]],
        rhs: vec![vec![0, 1, 2]],
        ..CandidateRule::default()
    }
}
fn shared(c: &CandidateRule) -> (HypergraphStore, EncodedRegion) {
    let mut s = HypergraphStore::new();
    let r = encode_versioned(
        &mut s,
        c,
        &MetaLimits::default(),
        EncodingVersion::SharedPatternsV2,
    )
    .unwrap();
    (s, r)
}

#[test]
fn duplicates_pair_one_to_one_and_preserve_each_sides_execution_order() {
    let c = CandidateRule {
        labels: vec![None; 3],
        lhs: vec![vec![0, 1], vec![1, 2], vec![0, 1]],
        rhs: vec![vec![0, 1], vec![0, 1], vec![2, 1]],
        ..CandidateRule::default()
    };
    let (s, r) = shared(&c);
    assert_eq!(
        s.nodes().filter(|(_, n)| n.label.0 == L_PAT_EDGE).count(),
        4
    );
    let decoded = read(&s.snapshot(), r.root, &MetaLimits::default()).unwrap();
    assert_eq!(decoded, c);
    assert_eq!(decoded.execution_bytes(), c.execution_bytes());
    let mut legacy = HypergraphStore::new();
    let old = encode(&mut legacy, &c, &MetaLimits::default()).unwrap();
    assert_eq!(
        read(&legacy.snapshot(), old.root, &MetaLimits::default())
            .unwrap()
            .rcf_bytes(),
        decoded.rcf_bytes()
    );
    assert!(s.node_count() < legacy.node_count());
}

#[test]
fn a_shared_slot_edit_changes_both_sides_without_reader_repair() {
    let c = identity();
    let (mut s, r) = shared(&c);
    let slot = s
        .edges()
        .find(|(_, e)| e.label.0 == E_SLOT_BASE + 2)
        .unwrap()
        .0;
    let pat = s.get_edge(slot).unwrap().nodes[0];
    let var = s
        .edges()
        .find(|(_, e)| e.label.0 == E_SLOT_BASE)
        .unwrap()
        .1
        .nodes[1];
    s.remove_edge(slot);
    s.add_edge(HyperEdge::new(
        Label(E_SLOT_BASE + 2),
        [pat, var].into_iter().collect(),
    ));
    let before = s.canonical_hash();
    let decoded = read(&s.snapshot(), r.root, &MetaLimits::default()).unwrap();
    assert_eq!(
        s.canonical_hash(),
        before,
        "reader must never repair its graph"
    );
    assert_eq!(decoded.lhs, vec![vec![0, 1, 0]]);
    assert_eq!(decoded.lhs, decoded.rhs);
    assert_ne!(decoded.rcf_bytes(), c.rcf_bytes());
}

#[test]
fn side_detachment_is_an_ordinary_asymmetric_rule() {
    let (mut s, r) = shared(&identity());
    let member = s
        .edges()
        .find(|(_, e)| e.label.0 == E_MEMBER && s.get_node(e.nodes[0]).unwrap().label.0 == L_SIDE_R)
        .unwrap()
        .0;
    s.remove_edge(member);
    let decoded = read(&s.snapshot(), r.root, &MetaLimits::default()).unwrap();
    assert_eq!(decoded.lhs, vec![vec![0, 1, 2]]);
    assert!(decoded.rhs.is_empty());
}

#[test]
fn membership_versions_and_hints_fail_closed() {
    let limits = MetaLimits::default();
    let (mut s, r) = shared(&identity());
    let member = s
        .edges()
        .find(|(_, e)| e.label.0 == E_MEMBER)
        .unwrap()
        .1
        .clone();
    s.add_edge(member);
    assert_eq!(
        read(&s.snapshot(), r.root, &limits),
        Err(Validator::Members)
    );
    let (mut s, r) = shared(&identity());
    s.get_node_mut(r.root)
        .unwrap()
        .set_property(K_ENCODING, PropertyValue::Int(1));
    assert_eq!(
        read(&s.snapshot(), r.root, &limits),
        Err(Validator::Members)
    );
    s.get_node_mut(r.root)
        .unwrap()
        .set_property(K_ENCODING, PropertyValue::Int(99));
    assert_eq!(read(&s.snapshot(), r.root, &limits), Err(Validator::Region));
    let (mut s, r) = shared(&identity());
    let pat = s.nodes().find(|(_, n)| n.label.0 == L_PAT_EDGE).unwrap().0;
    s.get_node_mut(pat)
        .unwrap()
        .set_property(K_ORDER_R, PropertyValue::Bool(true));
    assert_eq!(
        read(&s.snapshot(), r.root, &limits),
        Err(Validator::Members)
    );
}

#[test]
fn presentation_hint_gaps_are_independent_of_pattern_count_ceiling() {
    let mut c = identity();
    c.lhs.push(c.lhs[0].clone());
    c.rhs.push(c.rhs[0].clone());
    let (mut s, r) = shared(&c);
    let first = s
        .nodes()
        .find(|(_, n)| {
            n.label.0 == L_PAT_EDGE && n.get_property(K_ORDER) == Some(PropertyValue::Int(0))
        })
        .unwrap()
        .0;
    s.remove_node(first);
    let mut limits = MetaLimits {
        max_pattern_edges: 1,
        ..MetaLimits::default()
    };
    let decoded = read(&s.snapshot(), r.root, &limits).unwrap();
    assert_eq!(decoded.lhs, identity().lhs);
    assert_eq!(decoded.rhs, identity().rhs);
    limits.max_pattern_edges = 0;
    assert_eq!(read(&s.snapshot(), r.root, &limits), Err(Validator::Bounds));
}
