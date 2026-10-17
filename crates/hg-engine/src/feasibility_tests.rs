use super::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyOp};
use hg_rules::{
    CompiledDegreeCompare, CompiledPropertyCompare, CompiledPropertyGuard, CompiledQuantityCompare,
    PatternEdge, RewriteTemplate,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn rule(n: usize, edges: &[&[usize]]) -> CompiledRule {
    CompiledRule {
        meta_visible: false,
        name: "feasibility-test".into(),
        priority: 7,
        weight: 1.0,
        enabled: true,
        schedule: None,
        lhs_var_names: (0..n).map(|i| format!("v{i}")).collect(),
        template: RewriteTemplate::default(),
        pattern: MatchPattern {
            n_vars: n,
            node_label_constraints: vec![None; n],
            node_label_ceiling: Some(0xFFF00000),
            match_work_limit: Some(40_000),
            match_result_limit: Some(50_000),
            edges: edges
                .iter()
                .map(|e| PatternEdge {
                    var_ixs: e.to_vec(),
                    label: None,
                    ordered: false,
                })
                .collect(),
            ..MatchPattern::default()
        },
    }
}

fn key(bs: &[Binding]) -> Vec<(usize, i32, Vec<NodeId>, Vec<EdgeId>)> {
    bs.iter()
        .map(|b| {
            (
                b.rule_id,
                b.priority,
                b.bound_nodes.to_vec(),
                b.matched_edges.to_vec(),
            )
        })
        .collect()
}

fn guard_key(mut stats: GuardStats) -> GuardStats {
    stats.enumeration_work = 0;
    stats
}

fn pair_store(repeated: bool) -> HypergraphStore {
    let mut s = HypergraphStore::new();
    let a = s.add_node(NodeData::new(Label(0), 0));
    let b = s.add_node(NodeData::new(Label(1), 0));
    s.add_edge(HyperEdge::new(
        Label(0),
        SmallVec::from_slice(&[a, if repeated { a } else { b }]),
    ));
    s
}

#[test]
fn complete_ordered_bindings_and_all_guard_counters_match_legacy() {
    let patterns: &[(usize, &[&[usize]])] = &[
        (1, &[&[0, 0]]),
        (3, &[&[0, 1], &[2, 2]]),
        (2, &[&[0, 0], &[0, 1]]),
        (1, &[&[0, 0], &[0, 0]]),
        (2, &[&[0, 0], &[1, 1]]),
        (4, &[&[0, 1, 2], &[3, 3]]),
    ];
    for seed in 0..32 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut s = HypergraphStore::new_metadata_isolated();
        let nodes: Vec<_> = (0..4)
            .map(|i| {
                let mut data = NodeData::new(Label(if i == 3 { 0xFFF00005 } else { i % 2 }), 0);
                data.set_property(1, PropertyValue::Int(i as i64 - 1));
                s.add_node(data)
            })
            .collect();
        for _ in 0..8 {
            let arity = rng.gen_range(1..=3);
            let ns: SmallVec<[NodeId; 8]> =
                (0..arity).map(|_| nodes[rng.gen_range(0..4)]).collect();
            s.add_edge(HyperEdge::new(Label(rng.gen_range(0..2)), ns));
        }
        for &(n, es) in patterns {
            let mut r = rule(n, es);
            r.pattern.property_guards.push(CompiledPropertyGuard {
                var_ix: 0,
                key: 1,
                op: PropertyOp::Gte,
                value: PropertyValue::Int(0),
            });
            if seed % 3 == 0 {
                r.pattern.node_label_constraints[0] = Some(Label(0));
            }
            if seed % 4 == 0 && n > 1 {
                r.pattern.distinct_pairs.push((0, n - 1));
            }
            for e in &mut r.pattern.edges {
                if seed % 5 == 0 {
                    e.label = Some(Label(1));
                }
                e.ordered = seed % 2 == 0;
            }
            let before: Vec<_> = s
                .nodes()
                .map(|(id, d)| (id, d.label, d.sorted_properties()))
                .collect();
            let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 23);
            let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 23);
            assert!(
                !os.enumeration_truncated && !ns.enumeration_truncated,
                "seed {seed}"
            );
            assert_eq!(
                key(&new),
                key(&old),
                "seed {seed}, exact unsorted binding order"
            );
            assert_eq!(guard_key(ns), guard_key(os), "seed {seed}, guards");
            assert_eq!(
                s.nodes()
                    .map(|(id, d)| (id, d.label, d.sorted_properties()))
                    .collect::<Vec<_>>(),
                before
            );
        }
    }
}

#[test]
fn no_self_pair_is_proved_with_charged_scan_and_negative_decision() {
    let s = pair_store(false);
    for (budget, work, truncated) in [
        (0, 0, true),
        (1, 1, true),
        (2, 2, true),
        (3, 3, true),
        (4, 4, false),
        (5, 4, false),
    ] {
        let mut r = rule(1, &[&[0, 0]]);
        r.pattern.match_work_limit = Some(budget);
        let (bs, stats) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
        assert!(bs.is_empty());
        assert_eq!(stats.enumeration_work, work);
        assert_eq!(stats.enumeration_truncated, truncated, "budget {budget}");
    }
}

#[test]
fn wrong_arity_and_label_inspections_are_still_charged() {
    let mut s = pair_store(false);
    let a = s.nodes().next().unwrap().0;
    s.add_edge(HyperEdge::new(Label(3), SmallVec::from_slice(&[a])));
    let mut r = rule(1, &[&[0, 0]]);
    r.pattern.edges[0].label = Some(Label(99));
    let (bs, stats) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
    assert!(bs.is_empty() && !stats.enumeration_truncated);
    assert_eq!(stats.enumeration_work, 4); // probe + 2 inspections + proof
}

#[test]
fn positive_probe_keeps_legacy_dfs_prefix_after_debiting_overhead() {
    let s = pair_store(true);
    for budget in 0..40 {
        let mut r = rule(1, &[&[0, 0]]);
        r.pattern.match_work_limit = Some(budget);
        let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 5);
        if budget < 4 {
            assert!(new.is_empty() && ns.enumeration_truncated);
            assert_eq!(ns.enumeration_work, budget);
        } else {
            r.pattern.match_work_limit = Some(budget - 4);
            let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 5);
            assert_eq!(key(&new), key(&old));
            assert_eq!(ns.enumeration_work, os.enumeration_work + 4);
            assert_eq!(guard_key(ns), guard_key(os));
        }
    }
}

#[test]
fn zero_result_allowance_fails_before_even_a_negative_proof() {
    let s = pair_store(false);
    let mut r = rule(1, &[&[0, 0]]);
    r.pattern.match_result_limit = Some(0);
    let (bs, stats) = find_bindings_counted_feasible(&s.snapshot(), &r, 0);
    assert!(bs.is_empty() && stats.enumeration_truncated);
    assert_eq!(stats.enumeration_work, 0);
}

#[test]
fn positive_result_limit_retains_exact_prefix_and_conservative_truncation() {
    let mut s = pair_store(true);
    let a = s.nodes().next().unwrap().0;
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, a])));
    for result_limit in 0..4 {
        let mut r = rule(1, &[&[0, 0]]);
        r.pattern.match_result_limit = Some(result_limit);
        let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
        let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 1);
        assert_eq!(key(&new), key(&old));
        assert_eq!(guard_key(ns), guard_key(os));
        assert!(new.len() <= result_limit);
    }
}

#[test]
fn positive_probe_does_not_bypass_edge_injectivity() {
    let s = pair_store(true);
    let r = rule(1, &[&[0, 0], &[0, 0]]);
    let (bs, stats) = find_bindings_counted_feasible(&s.snapshot(), &r, 0);
    assert!(bs.is_empty() && !stats.enumeration_truncated);
}

#[test]
fn distinct_variables_can_still_share_a_node() {
    let mut s = pair_store(true);
    let a = s.nodes().next().unwrap().0;
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, a])));
    let mut r = rule(2, &[&[0, 0], &[1, 1]]);
    let (bs, stats) = find_bindings_counted_feasible(&s.snapshot(), &r, 0);
    assert_eq!(bs.len(), 2);
    assert!(!stats.enumeration_truncated);
    assert!(bs.iter().all(|b| b.bound_nodes[0] == b.bound_nodes[1]));
    r.pattern.distinct_pairs.push((0, 1));
    assert!(find_bindings_counted_feasible(&s.snapshot(), &r, 0)
        .0
        .is_empty());
}

#[test]
fn metadata_visibility_node_and_edge_labels_are_not_shortcut() {
    let mut s = HypergraphStore::new_metadata_isolated();
    let a = s.add_node(NodeData::new(Label(0xFFF00001), 0));
    s.add_edge(HyperEdge::new(Label(7), SmallVec::from_slice(&[a, a])));
    for visible in [false, true] {
        let mut r = rule(1, &[&[0, 0]]);
        r.meta_visible = visible;
        r.pattern.node_label_ceiling = if visible { None } else { Some(0xFFF00000) };
        r.pattern.node_label_constraints[0] = Some(Label(0xFFF00001));
        r.pattern.edges[0].label = Some(Label(7));
        let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
        let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 1);
        assert_eq!(key(&new), key(&old));
        assert_eq!(new.len(), usize::from(visible));
        assert_eq!(guard_key(ns), guard_key(os));
        r.pattern.node_label_constraints[0] = Some(Label(0));
        assert!(find_bindings_counted_feasible(&s.snapshot(), &r, 1)
            .0
            .is_empty());
    }
}

#[test]
fn property_guards_are_not_evaluated_early_or_skipped_on_positive_probe() {
    let s = pair_store(true);
    let mut r = rule(1, &[&[0, 0]]);
    r.pattern.property_guards.push(CompiledPropertyGuard {
        var_ix: 0,
        key: 123,
        op: PropertyOp::Gte,
        value: PropertyValue::Int(0),
    });
    let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
    let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 1);
    assert!(new.is_empty() && old.is_empty());
    assert_eq!(ns.property_guards, vec![(1, 1)]);
    assert_eq!(guard_key(ns), guard_key(os));
}

#[test]
fn all_guard_families_keep_evaluation_and_short_circuit_order() {
    let mut s = pair_store(true);
    let a = s.nodes().next().unwrap().0;
    s.get_node_mut(a)
        .unwrap()
        .set_property(1, PropertyValue::Int(2));
    for rejecting_family in 0..=4 {
        let mut r = rule(1, &[&[0, 0]]);
        r.pattern.property_guards.push(CompiledPropertyGuard {
            var_ix: 0,
            key: 1,
            op: PropertyOp::Gte,
            value: PropertyValue::Int(if rejecting_family == 1 { 3 } else { 0 }),
        });
        r.pattern.property_compares.push(CompiledPropertyCompare {
            var_a_ix: 0,
            key_a: 1,
            op: if rejecting_family == 2 {
                PropertyOp::Gt
            } else {
                PropertyOp::Gte
            },
            var_b_ix: 0,
            key_b: 1,
            offset: None,
        });
        r.pattern.degree_compares.push(CompiledDegreeCompare {
            var_ix: 0,
            op: PropertyOp::Gte,
            value: if rejecting_family == 3 { 2 } else { 1 },
            offset: None,
        });
        r.pattern.quantity_compares.push(CompiledQuantityCompare {
            var_a_ix: 0,
            quantity: NodeQuantity::Degree,
            op: if rejecting_family == 4 {
                PropertyOp::Gt
            } else {
                PropertyOp::Gte
            },
            var_b_ix: 0,
            offset: None,
        });
        let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
        let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 1);
        assert_eq!(key(&new), key(&old));
        assert_eq!(new.len(), usize::from(rejecting_family == 0));
        assert_eq!(guard_key(ns.clone()), guard_key(os));
        let counts = [
            ns.property_guards[0],
            ns.property_compares[0],
            ns.degree_compares[0],
            ns.quantity_compares[0],
        ];
        for (index, count) in counts.into_iter().enumerate() {
            let family = index + 1;
            assert_eq!(
                count,
                if rejecting_family == 0 || family < rejecting_family {
                    (1, 0)
                } else if family == rejecting_family {
                    (1, 1)
                } else {
                    (0, 0)
                }
            );
        }
    }
}

#[test]
fn no_work_budget_and_non_self_pair_rules_are_bit_exact_legacy_fallbacks() {
    let s = pair_store(true);
    for edges in [&[&[0, 1][..]][..], &[&[0, 0, 0][..]][..]] {
        let r = rule(2, edges);
        let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
        let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 1);
        assert_eq!(key(&new), key(&old));
        assert_eq!(ns, os);
    }
    for result_limit in [None, Some(0), Some(1), Some(10)] {
        let mut r = rule(1, &[&[0, 0]]);
        r.pattern.match_work_limit = None;
        r.pattern.match_result_limit = result_limit;
        let (new, ns) = find_bindings_counted_feasible(&s.snapshot(), &r, 1);
        let (old, os) = find_bindings_counted_legacy(&s.snapshot(), &r, 1);
        assert_eq!(key(&new), key(&old));
        assert_eq!(ns, os);
    }
}

#[test]
fn public_dispatch_matches_the_declared_build_feature() {
    assert_eq!(
        MATCHER_WORK_MODEL,
        if cfg!(feature = "repeated-edge-feasibility") {
            "repeated-self-pair-feasibility-v1"
        } else {
            "legacy-dfs-v1"
        }
    );
    let s = pair_store(false);
    let r = rule(1, &[&[0, 0]]);
    let (got, gs) = find_bindings_counted(&s.snapshot(), &r, 1);
    let (expected, es) = if cfg!(feature = "repeated-edge-feasibility") {
        find_bindings_counted_feasible(&s.snapshot(), &r, 1)
    } else {
        find_bindings_counted_legacy(&s.snapshot(), &r, 1)
    };
    assert_eq!(key(&got), key(&expected));
    assert_eq!(gs, es);
}
