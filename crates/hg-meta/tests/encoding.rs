use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_meta::*;
use hg_rules::{EpochPolicy, RuleSetDelta, RuleSource};
use proptest::prelude::*;

fn seed() -> CandidateRule {
    CandidateRule {
        priority: 0,
        labels: vec![None; 4],
        lhs: vec![vec![0, 1, 2]],
        rhs: vec![vec![0, 1, 3], vec![1, 2, 3], vec![0, 2, 3]],
        guards: vec![],
    }
}
fn reverse(c: &CandidateRule) -> CandidateRule {
    let n = c.labels.len();
    let mut r = c.clone();
    r.labels.reverse();
    for e in r.lhs.iter_mut().chain(&mut r.rhs) {
        for v in e {
            *v = n - 1 - *v;
        }
    }
    for g in &mut r.guards {
        g.var = n - 1 - g.var;
    }
    r
}

#[test]
fn gr0_twenty_collide_twenty_distinct_and_topological_roundtrip() {
    let limits = MetaLimits::default();
    for i in 0..20 {
        let mut a = seed();
        a.priority = i;
        a.labels[0] = Some(i as u32);
        let mut b = reverse(&a);
        b.rhs.reverse();
        assert_eq!(a.rcf_bytes(), b.rcf_bytes());
        b.priority += 1;
        assert_ne!(a.rcf_bytes(), b.rcf_bytes());
        let mut s = HypergraphStore::new();
        // Different region locations and installation histories.
        for _ in 0..i {
            s.add_node(NodeData::new(Label(12), 0));
        }
        let region = encode(&mut s, &a, &limits).unwrap();
        assert_eq!(read(&s.snapshot(), region.root, &limits).unwrap(), a);
        let cr = a.compile(&limits, false).unwrap();
        assert_eq!(cr.template.n_rhs_vars, 4);
        assert_eq!(
            cr.template.rhs_from_lhs,
            vec![Some(0), Some(1), Some(2), None]
        );
    }
    let a = seed();
    let mut b = a.clone();
    b.rhs[0].swap(0, 1);
    assert_ne!(a.rcf_bytes(), b.rcf_bytes(), "slot order is content");
    b = a.clone();
    b.rhs.push(vec![0, 2]);
    assert_ne!(a.rcf_bytes(), b.rcf_bytes());
}

#[test]
fn gr4_all_nine_validators_engage_and_clean_counters_are_present() {
    let base = seed();
    let limits = MetaLimits::default();
    let mut sum = [0; 9];
    for v in 0..9 {
        let mut s = HypergraphStore::new();
        let region = encode(&mut s, &base, &limits).unwrap();
        let first = |label| {
            region
                .nodes
                .iter()
                .copied()
                .find(|&n| s.get_node(n).unwrap().label.0 == label)
                .unwrap()
        };
        let side = first(L_SIDE_L);
        let pat = first(L_PAT_EDGE);
        let var = first(L_VAR);
        let mut local_limits = limits.clone();
        match v {
            0 => local_limits.max_region_nodes = 2,
            1 => {
                let e = region.edges[0];
                s.remove_edge(e);
            }
            2 => {
                s.add_edge(HyperEdge::new(
                    Label(E_MEMBER),
                    [side, pat].into_iter().collect(),
                ));
            }
            3 => {
                s.add_edge(HyperEdge::new(
                    Label(E_SLOT_BASE),
                    [pat, var].into_iter().collect(),
                ));
            }
            4 => {
                s.get_node_mut(var)
                    .unwrap()
                    .set_property(K_ORDER, PropertyValue::Int(99));
            }
            5 => {
                let n = s.add_node(NodeData::new(Label(L_LABEL_LIT), 0));
                s.add_edge(HyperEdge::new(
                    Label(E_HASLABEL),
                    [var, n].into_iter().collect(),
                ));
            }
            6 => {
                let n = s.add_node(NodeData::new(Label(L_GUARD), 0));
                s.add_edge(HyperEdge::new(
                    Label(E_GUARDS),
                    [var, n].into_iter().collect(),
                ));
            }
            7 => local_limits.max_pattern_edges = 0,
            8 => s
                .get_node_mut(region.root)
                .unwrap()
                .set_property(K_PRIORITY, PropertyValue::Bool(true)),
            _ => unreachable!(),
        }
        let mut source = GraphRuleSource::new(EpochPolicy::Never, local_limits, false);
        let RuleSetDelta::Replace { report, rules } = source.rules(&s.snapshot(), 0) else {
            panic!()
        };
        assert!(rules.is_empty());
        assert_eq!(report.quarantined, 1);
        assert_eq!(report.quarantine_by_validator[v], 1, "validator {v}");
        for (a, b) in sum.iter_mut().zip(report.quarantine_by_validator) {
            *a += b;
        }
    }
    assert_eq!(sum, [1; 9]);
    let mut s = HypergraphStore::new();
    encode(&mut s, &base, &limits).unwrap();
    let mut source = GraphRuleSource::new(EpochPolicy::Never, limits, false);
    let RuleSetDelta::Replace { report, .. } = source.rules(&s.snapshot(), 0) else {
        panic!()
    };
    assert_eq!(report.quarantine_by_validator, [0; 9]);
}

#[test]
fn d1_content_order_and_frozen_compilation() {
    let limits = MetaLimits::default();
    let mut a = seed();
    a.priority = 1;
    let mut b = seed();
    b.labels[0] = Some(9);
    let compile = |specs: &[CandidateRule]| {
        let mut s = HypergraphStore::new();
        for c in specs {
            encode(&mut s, c, &limits).unwrap();
        }
        let mut source = GraphRuleSource::new(EpochPolicy::Never, limits.clone(), false);
        let RuleSetDelta::Replace { report, rules } = source.rules(&s.snapshot(), 0) else {
            panic!()
        };
        assert!(matches!(
            source.rules(&s.snapshot(), 1),
            RuleSetDelta::Unchanged
        ));
        (
            report.classes,
            rules.into_iter().map(|r| r.name).collect::<Vec<_>>(),
        )
    };
    assert_eq!(compile(&[a.clone(), b.clone()]), compile(&[b, a]));
}

#[test]
fn compact_slots_after_deletion_and_guard_variant_roundtrip() {
    let limits = MetaLimits::default();
    let mut c = seed();
    c.guards.push(Guard {
        var: 0,
        key: 1,
        op: hg_core::PropertyOp::Gte,
        value: PropertyValue::Float(2.5),
    });
    let mut s = HypergraphStore::new();
    let r = encode(&mut s, &c, &limits).unwrap();
    assert_eq!(read(&s.snapshot(), r.root, &limits).unwrap(), c);
    let lhs_pat = r
        .nodes
        .iter()
        .copied()
        .find(|&n| s.get_node(n).unwrap().label.0 == L_PAT_EDGE)
        .unwrap();
    let slot = s
        .edges_incident_to(lhs_pat)
        .iter()
        .copied()
        .find(|&e| s.get_edge(e).unwrap().label.0 == E_SLOT_BASE + 1)
        .unwrap();
    s.remove_edge(slot);
    let decoded = read(&s.snapshot(), r.root, &limits).unwrap();
    assert_eq!(decoded.lhs, vec![vec![0, 2]]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn reader_is_total_under_random_single_edge_deletion(index in 0usize..100) {
        let mut s = HypergraphStore::new(); let limits = MetaLimits::default();
        let r = encode(&mut s, &seed(), &limits).unwrap();
        s.remove_edge(r.edges[index % r.edges.len()]);
        let _ = read(&s.snapshot(), r.root, &limits);
    }
}
