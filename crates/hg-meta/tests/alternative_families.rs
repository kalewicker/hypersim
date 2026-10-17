use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_meta::*;
use hg_rules::{EpochPolicy, RuleSetDelta, RuleSource};

fn core() -> CandidateRule {
    CandidateRule {
        labels: vec![None; 2],
        lhs: vec![vec![0, 1]],
        rhs: vec![vec![0, 1]],
        ..CandidateRule::default()
    }
}
fn encoded(active: bool) -> (HypergraphStore, EncodedRegion) {
    let mut s = HypergraphStore::new();
    let mut p = core();
    p.priority = 1;
    let r = encode_family(&mut s, &core(), &p, active, &MetaLimits::default()).unwrap();
    (s, r)
}
fn parts(s: &HypergraphStore, r: &EncodedRegion) -> FamilyParts {
    family_parts(&s.snapshot(), r.root, &MetaLimits::default()).unwrap()
}

#[test]
fn explicit_modes_roundtrip_expand_and_deduplicate_only_exact_execution() {
    for active in [false, true] {
        let (s, r) = encoded(active);
        let p = read_program(&s.snapshot(), r.root, &MetaLimits::default()).unwrap();
        assert_eq!(p.core, core());
        assert_eq!(p.active, active);
        assert_eq!(p.candidates().len(), if active { 2 } else { 1 });
        assert_eq!(
            p.compile(&MetaLimits::default(), false).unwrap().len(),
            p.candidates().len()
        );
        let mut same = p.clone();
        same.proposal = Some(core());
        same.active = true;
        assert_eq!(same.candidates().len(), 1);
        assert_eq!(same.rcf_bytes(), core().rcf_bytes());
    }
}

#[test]
fn dormant_edits_do_not_change_executable_identity_but_core_edits_do() {
    let (mut s, r) = encoded(false);
    let a = read_program(&s.snapshot(), r.root, &MetaLimits::default()).unwrap();
    let p = parts(&s, &r);
    s.get_node_mut(p.proposal.root)
        .unwrap()
        .set_property(K_PRIORITY, PropertyValue::Int(7));
    let b = read_program(&s.snapshot(), r.root, &MetaLimits::default()).unwrap();
    assert_eq!(a.rcf_bytes(), b.rcf_bytes());
    assert_eq!(a.execution_bytes(), b.execution_bytes());
    s.get_node_mut(p.core.root)
        .unwrap()
        .set_property(K_PRIORITY, PropertyValue::Int(8));
    let c = read_program(&s.snapshot(), r.root, &MetaLimits::default()).unwrap();
    assert_ne!(b.rcf_bytes(), c.rcf_bytes());
    s.get_node_mut(r.root)
        .unwrap()
        .set_property(K_GRAFT_MODE, PropertyValue::Int(1));
    assert_eq!(
        read_program(&s.snapshot(), r.root, &MetaLimits::default())
            .unwrap()
            .candidates()
            .len(),
        2
    );
}

#[test]
fn malformed_dormant_proposal_quarantines_the_whole_family_without_cached_fallback() {
    let (mut s, r) = encoded(false);
    let p = parts(&s, &r);
    let mut src = GraphRuleSource::new(
        EpochPolicy::Fixed(1.try_into().unwrap()),
        MetaLimits::default(),
        false,
    );
    if let RuleSetDelta::Replace { rules, report } = src.rules(&s.snapshot(), 0) {
        assert_eq!(rules.len(), 1);
        assert_eq!(report.validated_regions, 1);
    } else {
        panic!();
    }
    s.get_node_mut(p.proposal.root)
        .unwrap()
        .set_property(K_PRIORITY, PropertyValue::Bool(true));
    let before = s.canonical_hash();
    assert_eq!(
        read_program(&s.snapshot(), r.root, &MetaLimits::default()),
        Err(Validator::Compilability)
    );
    if let RuleSetDelta::Replace { rules, report } = src.rules(&s.snapshot(), 1) {
        assert!(rules.is_empty());
        assert_eq!(report.quarantined, 1);
        assert_eq!(report.validated_regions, 0);
        assert_eq!(report.retired, 1);
        assert_eq!(report.quarantine_by_validator[8], 1);
    } else {
        panic!();
    }
    assert_eq!(before, s.canonical_hash());
}

#[test]
fn family_modes_links_and_unknown_scaffolding_fail_closed() {
    for mode in [
        PropertyValue::Bool(true),
        PropertyValue::Int(-1),
        PropertyValue::Int(2),
    ] {
        let (mut s, r) = encoded(false);
        s.get_node_mut(r.root)
            .unwrap()
            .set_property(K_GRAFT_MODE, mode);
        assert_eq!(
            read_program(&s.snapshot(), r.root, &MetaLimits::default()),
            Err(Validator::Compilability)
        );
    }
    let (mut s, r) = encoded(false);
    let link = s.get_edge(r.edges[0]).unwrap().clone();
    s.add_edge(link);
    assert_eq!(
        read_program(&s.snapshot(), r.root, &MetaLimits::default()),
        Err(Validator::Members)
    );
    let (mut s, r) = encoded(false);
    s.remove_edge(r.edges[1]);
    assert_eq!(
        read_program(&s.snapshot(), r.root, &MetaLimits::default()),
        Err(Validator::Sides)
    );
    let (mut s, r) = encoded(false);
    let extra = s.add_node(NodeData::new(Label(META_BASE + 99), 0));
    s.add_edge(HyperEdge::new(
        Label(META_BASE + 100),
        [r.root, extra].into_iter().collect(),
    ));
    assert_eq!(
        read_program(&s.snapshot(), r.root, &MetaLimits::default()),
        Err(Validator::Members)
    );
}

#[test]
fn cross_family_and_cross_component_sharing_are_invalid() {
    let (mut s, r) = encoded(true);
    let p = parts(&s, &r);
    let other = encode_family(&mut s, &core(), &core(), false, &MetaLimits::default()).unwrap();
    let q = parts(&s, &other);
    s.add_edge(HyperEdge::new(
        Label(E_GRAFT),
        [r.root, q.proposal.root].into_iter().collect(),
    ));
    assert!(read_program(&s.snapshot(), r.root, &MetaLimits::default()).is_err());
    assert!(read_program(&s.snapshot(), other.root, &MetaLimits::default()).is_err());
    let (mut s, r) = encoded(true);
    let q = parts(&s, &r);
    let pat = q
        .core
        .nodes
        .iter()
        .find(|&&n| s.get_node(n).unwrap().label.0 == L_PAT_EDGE)
        .copied()
        .unwrap();
    let var = q
        .proposal
        .nodes
        .iter()
        .find(|&&n| s.get_node(n).unwrap().label.0 == L_VAR)
        .copied()
        .unwrap();
    s.add_edge(HyperEdge::new(
        Label(E_SLOT_BASE + 2),
        [pat, var].into_iter().collect(),
    ));
    assert_eq!(
        read_program(
            &s.snapshot(),
            r.root,
            &MetaLimits {
                radius: 20,
                ..MetaLimits::default()
            }
        ),
        Err(Validator::Members)
    );
    assert_ne!(p.core.root, p.proposal.root);
}

#[test]
fn whole_family_caps_apply_and_writer_rejects_before_modifying_caller() {
    let (s, r) = encoded(false);
    for limits in [
        MetaLimits {
            max_region_nodes: r.nodes.len() - 1,
            ..MetaLimits::default()
        },
        MetaLimits {
            max_region_edges: r.edges.len() - 1,
            ..MetaLimits::default()
        },
        MetaLimits {
            radius: 2,
            ..MetaLimits::default()
        },
    ] {
        assert_eq!(
            read_program(&s.snapshot(), r.root, &limits),
            Err(Validator::Region)
        );
        let mut caller = HypergraphStore::new();
        caller.add_node(NodeData::new(Label(0), 0));
        let before = caller.canonical_hash();
        assert!(encode_family(&mut caller, &core(), &core(), false, &limits).is_err());
        assert_eq!(before, caller.canonical_hash());
    }
}

#[test]
fn canonical_family_order_and_rule_id_attribution_are_allocation_independent() {
    let mut p = core();
    p.priority = 3;
    let mut a = HypergraphStore::new();
    let mut b = HypergraphStore::new();
    b.add_node(NodeData::new(Label(0), 0));
    let ar = encode_family(&mut a, &core(), &p, true, &MetaLimits::default()).unwrap();
    let br = encode_family(&mut b, &p, &core(), true, &MetaLimits::default()).unwrap();
    let ap = read_program(&a.snapshot(), ar.root, &MetaLimits::default()).unwrap();
    let bp = read_program(&b.snapshot(), br.root, &MetaLimits::default()).unwrap();
    assert_eq!(ap.rcf_bytes(), bp.rcf_bytes());
    assert_eq!(ap.execution_bytes(), bp.execution_bytes());
    for s in [&a, &b] {
        let mut src = GraphRuleSource::new(EpochPolicy::Never, MetaLimits::default(), false);
        if let RuleSetDelta::Replace { rules, report } = src.rules(&s.snapshot(), 0) {
            assert_eq!(rules.len(), 2);
            assert_eq!(report.validated_regions, 1);
            assert_eq!(report.installed, 1);
            assert_eq!(
                report.classes,
                ap.candidates()
                    .iter()
                    .map(CandidateRule::rcf_hash)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                rules.iter().map(|r| r.priority).collect::<Vec<_>>(),
                ap.candidates()
                    .iter()
                    .map(|c| c.priority)
                    .collect::<Vec<_>>()
            );
        } else {
            panic!();
        }
    }
}

#[test]
fn exact_duplicate_activation_does_not_change_the_executable_table() {
    let mut s = HypergraphStore::new();
    let r = encode_family(&mut s, &core(), &core(), false, &MetaLimits::default()).unwrap();
    let mut source = GraphRuleSource::new(
        EpochPolicy::Fixed(1.try_into().unwrap()),
        MetaLimits::default(),
        false,
    );
    if let RuleSetDelta::Replace { rules, report } = source.rules(&s.snapshot(), 0) {
        assert!(report.changed);
        assert_eq!(rules.len(), 1);
    } else {
        panic!();
    }
    s.get_node_mut(r.root)
        .unwrap()
        .set_property(K_GRAFT_MODE, PropertyValue::Int(1));
    if let RuleSetDelta::Replace { rules, report } = source.rules(&s.snapshot(), 1) {
        assert!(!report.changed);
        assert_eq!(rules.len(), 1);
        assert_eq!(report.installed, 0);
        assert_eq!(report.retired, 0);
        assert_eq!(report.validated_regions, 1);
        assert_eq!(report.classes, vec![core().rcf_hash()]);
    } else {
        panic!();
    }
}
