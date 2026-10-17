use super::*;
use hg_core::Label;
use hg_rules::{compile_rules, parse_rules_toml};

fn rules() -> Vec<CompiledRule> {
    compile_rules(
        &parse_rules_toml(
            r#"
[[rules]]
name="busy"
[rules.lhs]
nodes=["a","b"]
edges=[["a","b"]]
[rules.rhs]
nodes=["a","b"]
edges=[["a","b"]]
[[rules]]
name="cold"
priority=5
[rules.lhs]
nodes=["a","b"]
edges=[["a","b"]]
[rules.lhs.node_labels]
a=99
[rules.rhs]
nodes=["a","b"]
edges=[["a","b"]]
"#,
        )
        .unwrap(),
    )
    .unwrap()
}
fn fixture() -> HypergraphStore {
    let mut s = HypergraphStore::new();
    let ids: Vec<_> = (0..6)
        .map(|_| s.add_node(NodeData::new(Label(0), 0)))
        .collect();
    for pair in ids.windows(2) {
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(pair)));
    }
    s
}
fn engine(policy: MatchBudgetPolicy, work: usize, results: usize) -> Engine {
    let mut e = Engine::new(rules(), StepMode::MaxParallel, 42, 1);
    e.set_match_budget_policy(work, results, policy);
    e
}
fn full_key(b: &Binding) -> String {
    // Include ordered variables and matched edges, not only conflict sort key.
    format!(
        "{}:{}:{:?}:{:?}",
        b.rule_id, b.priority, b.bound_nodes, b.matched_edges
    )
}

#[cfg(feature = "repeated-edge-feasibility")]
fn self_pair_rules() -> Vec<CompiledRule> {
    compile_rules(&parse_rules_toml(r#"
[[rules]]
name="pair_control"
[rules.lhs]
nodes=["a","b"]
edges=[["a","b"]]
[rules.rhs]
nodes=["a","b"]
edges=[["a","b"]]
[[rules]]
name="self_pair"
priority=1
[rules.lhs]
nodes=["a"]
edges=[["a","a"]]
[rules.rhs]
nodes=["a","b"]
edges=[["a","a"],["b","b"]]
"#).unwrap()).unwrap()
}

#[cfg(feature = "repeated-edge-feasibility")]
#[test]
fn prepass_truncation_never_applies_an_already_completed_core_pool() {
    use crate::matcher::find_bindings_counted_legacy;
    let mut s = fixture();
    let rs = self_pair_rules();
    let mut core = rs[0].clone();
    core.pattern.match_work_limit = Some(10_000);
    core.pattern.match_result_limit = Some(1000);
    let (pool, stats) = find_bindings_counted_legacy(&s.snapshot(), &core, 0);
    assert!(!pool.is_empty() && !stats.enumeration_truncated);
    let mut e = Engine::new(rs, StepMode::SequentialRandom, 42, 1);
    e.set_match_budget_policy(stats.enumeration_work + 1, 1000, MatchBudgetPolicy::Pooled);
    let before = s.canonical_hash();
    let rng = e.rng_state_probe();
    let step = s.step();
    assert_eq!(e.step(&mut s).accepted_rewrites, 0);
    assert!(e.match_ceiling_binding());
    assert_eq!(e.match_budget_usage().rules[0].results, pool.len());
    assert_eq!(e.match_budget_usage().rules[1].work_allowance, Some(1));
    assert_eq!(e.match_budget_usage().rules[1].work, 1);
    assert!(e.match_budget_usage().rules[1].truncated);
    assert_eq!(s.canonical_hash(), before);
    assert_eq!(s.step(), step);
    assert_eq!(e.rng_state_probe(), rng);
}

#[cfg(feature = "repeated-edge-feasibility")]
#[test]
fn self_pair_complete_pools_rewrites_allocation_and_rng_equal_legacy_pipeline() {
    use crate::matcher::find_bindings_counted_legacy;
    use rand::RngCore;
    // Bounded same-build legacy selection/apply control, not an independent
    // scientific Engine or a replay of the draw-9730 trajectory.
    let store_key = |s: &HypergraphStore| {
        (s.step(), s.nodes().map(|(id, n)| format!("{id:?}:{}:{}:{:?}:{}:{:?}", n.label.0, n.generation, n.sorted_properties(), n.energy.to_bits(), s.edges_incident_to(id))).collect::<Vec<_>>(),
         s.edges().map(|(id, e)| format!("{id:?}:{}:{:?}", e.label.0, e.nodes)).collect::<Vec<_>>(),
         s.edges().map(|(_, e)| e.label).collect::<std::collections::BTreeSet<_>>().into_iter()
             .map(|l| (l, s.edges_with_label(l).to_vec())).collect::<Vec<_>>())
    };
    for mode in [StepMode::MaxParallel, StepMode::SequentialRandom, StepMode::SequentialWeighted] {
        let mut s = fixture();
        let a = s.nodes().next().unwrap().0;
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, a])));
        let mut reference = s.clone();
        let mut rs = self_pair_rules();
        rs[1].weight = 3.0;
        let mut engine = Engine::new(rs.clone(), mode, 42, 2);
        engine.set_match_budget_policy(400_000, 50_000, MatchBudgetPolicy::Pooled);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..8 {
            let mut reference_pool = Vec::new();
            let mut work = 0;
            let mut results = 0;
            for (id, rule) in rs.iter().enumerate() {
                let mut bounded = rule.clone();
                bounded.pattern.match_work_limit = Some(400_000 - work);
                bounded.pattern.match_result_limit = Some(50_000 - results);
                let (bindings, stats) = find_bindings_counted_legacy(&reference.snapshot(), &bounded, id);
                assert!(!stats.enumeration_truncated);
                work += stats.enumeration_work; results += bindings.len();
                reference_pool.extend(bindings);
            }
            reference_pool.sort_by_key(|b| b.canonical_key());
            let before_rng = engine.rng_state_probe();
            let actual_pool = engine.collect_candidates(&s);
            assert!(!engine.match_budget_usage().truncated);
            assert_eq!(actual_pool.iter().map(full_key).collect::<Vec<_>>(), reference_pool.iter().map(full_key).collect::<Vec<_>>());
            assert_eq!(engine.rng_state_probe(), before_rng);
            let chosen: Vec<_> = match mode {
                StepMode::MaxParallel => resolve_conflicts(&reference_pool).into_iter().take(2).collect(),
                StepMode::SequentialRandom => reference_pool.choose(&mut rng).cloned().into_iter().collect(),
                StepMode::SequentialWeighted => reference_pool.choose_weighted(&mut rng, |b| rs[b.rule_id].weight).ok().cloned().into_iter().collect(),
            };
            for binding in &chosen { apply_binding(&mut reference, &rs[binding.rule_id], binding, &mut rng); }
            reference.set_step(reference.step() + 1);
            let stats = engine.step(&mut s);
            assert!(!engine.match_ceiling_binding());
            assert_eq!(stats.accepted_rewrites, chosen.len());
            assert_eq!(store_key(&s), store_key(&reference));
            assert_eq!(engine.rng_state_probe(), rng.clone().next_u64());
            assert!(engine.match_budget_usage().work <= 400_000 && engine.match_budget_usage().results <= 50_000);
        }
        // A further allocation checks hidden reuse history, not just hashes.
        let next = s.add_node(NodeData::new(Label(0), 0));
        let control_next = reference.add_node(NodeData::new(Label(0), 0));
        assert_eq!(next, control_next);
        assert_eq!(s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[next, next]))),
            reference.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[control_next, control_next]))));
        assert_eq!(store_key(&s), store_key(&reference));
    }
}

#[test]
fn pooled_result_capacity_is_borrowed_in_either_rule_order() {
    for reversed in [false, true] {
        let mut s = fixture();
        let mut rs = rules();
        if reversed {
            rs.reverse();
        }
        let mut split = Engine::new(rs.clone(), StepMode::MaxParallel, 42, 1);
        split.set_match_budget(10_000, 12);
        let mut copy = s.clone();
        let hash = copy.canonical_hash();
        assert_eq!(split.step(&mut copy).accepted_rewrites, 0);
        assert!(split.match_ceiling_binding());
        assert_eq!(copy.canonical_hash(), hash);
        let mut pooled = Engine::new(rs, StepMode::MaxParallel, 42, 1);
        pooled.set_match_budget_policy(10_000, 12, MatchBudgetPolicy::Pooled);
        assert_eq!(pooled.step(&mut s).accepted_rewrites, 1);
        assert!(!pooled.match_ceiling_binding());
        assert_eq!(pooled.match_budget_usage().results, 10);
        assert!(pooled.match_budget_usage().work <= 10_000);
    }
}

#[test]
fn pooled_work_capacity_debits_actual_work_not_each_rules_allowance() {
    let s = fixture();
    let mut reference = engine(MatchBudgetPolicy::Split, 10_000, 10_000);
    reference.collect_candidates(&s);
    let total = reference.match_budget_usage().work;
    let busy = reference.match_budget_usage().rules[0].work;
    assert!(busy > (total + 1) / 2);
    let mut split = engine(MatchBudgetPolicy::Split, total + 1, 1000);
    split.collect_candidates(&s);
    assert!(split.match_budget_usage().truncated);
    let mut pooled = engine(MatchBudgetPolicy::Pooled, total + 1, 1000);
    pooled.collect_candidates(&s);
    assert!(!pooled.match_budget_usage().truncated);
    assert_eq!(pooled.match_budget_usage().work, total);
    assert_eq!(
        pooled.match_budget_usage().rules[1].work_allowance,
        Some(total + 1 - busy)
    );
}

#[test]
fn global_caps_and_zero_capacity_never_apply_partial_pools_or_consume_rng() {
    for (work, results) in [(0, 100), (100, 0), (1, 100), (10_000, 5)] {
        let mut s = fixture();
        let before = s.canonical_hash();
        let step = s.step();
        let mut e = engine(MatchBudgetPolicy::Pooled, work, results);
        let rng = e.rng_state_probe();
        let stats = e.step(&mut s);
        assert!(e.match_ceiling_binding());
        assert_eq!(stats.accepted_rewrites, 0);
        assert_eq!(s.canonical_hash(), before);
        assert_eq!(s.step(), step);
        assert_eq!(e.rng_state_probe(), rng);
        assert!(e.match_budget_usage().work <= work);
        assert!(e.match_budget_usage().results <= results);
    }
}

#[test]
fn exact_boundaries_are_conservative_and_no_one_unit_clamp_is_added() {
    let s = fixture();
    let mut e = engine(MatchBudgetPolicy::Pooled, 10_000, 10);
    e.collect_candidates(&s);
    assert!(e.match_budget_usage().truncated);
    assert_eq!(e.match_budget_usage().results, 10);
    assert_eq!(e.match_budget_usage().rules[1].result_allowance, Some(0));
}

#[test]
fn pooled_complete_bindings_equal_unbounded_including_ties_and_noninjectivity() {
    let mut rs = rules();
    rs[1] = rs[0].clone();
    let mut s = fixture();
    let node = s.nodes().next().unwrap().0;
    s.add_edge(HyperEdge::new(
        Label(0),
        SmallVec::from_slice(&[node, node]),
    ));
    let mut direct = Engine::new(rs.clone(), StepMode::MaxParallel, 42, 1);
    let mut pool = Engine::new(rs, StepMode::MaxParallel, 42, 1);
    pool.set_match_budget_policy(10_000, 1000, MatchBudgetPolicy::Pooled);
    let a = direct.collect_candidates(&s);
    let b = pool.collect_candidates(&s);
    assert_eq!(
        a.iter().map(full_key).collect::<Vec<_>>(),
        b.iter().map(full_key).collect::<Vec<_>>()
    );
    assert_eq!(a.len(), 22);
    assert!(a.iter().any(|b| b.bound_nodes[0] == b.bound_nodes[1]));
    assert!(!pool.match_budget_usage().truncated);
}

#[test]
fn completed_trajectories_and_rng_match_every_step_mode() {
    for mode in [
        StepMode::MaxParallel,
        StepMode::SequentialRandom,
        StepMode::SequentialWeighted,
    ] {
        let mut a = fixture();
        let mut b = a.clone();
        let mut rs = rules();
        rs[1] = rs[0].clone();
        rs[0].weight = 2.0;
        rs[1].weight = 5.0;
        let mut ea = Engine::new(rs.clone(), mode, 42, 3);
        let mut eb = Engine::new(rs, mode, 42, 3);
        eb.set_match_budget_policy(10_000, 1000, MatchBudgetPolicy::Pooled);
        for _ in 0..20 {
            let x = ea.step(&mut a);
            let y = eb.step(&mut b);
            assert!(!eb.match_ceiling_binding());
            assert_eq!(a.canonical_hash(), b.canonical_hash());
            assert_eq!(x.accepted_rewrites, y.accepted_rewrites);
            assert_eq!(x.attempted_matches, y.attempted_matches);
            assert_eq!(ea.rng_state_probe(), eb.rng_state_probe());
        }
    }
}

#[test]
fn disabled_rules_do_not_spend_capacity_and_no_rules_spend_nothing() {
    let mut rs = rules();
    rs[1].enabled = false;
    let mut e = Engine::new(rs, StepMode::MaxParallel, 42, 1);
    e.set_match_budget_policy(1000, 11, MatchBudgetPolicy::Pooled);
    e.step(&mut fixture());
    assert!(!e.match_ceiling_binding());
    assert_eq!(e.match_budget_usage().rules.len(), 1);
    let mut empty = Engine::new(Vec::new(), StepMode::MaxParallel, 42, 1);
    empty.set_match_budget_policy(0, 0, MatchBudgetPolicy::Pooled);
    empty.step(&mut fixture());
    assert!(!empty.match_ceiling_binding());
    assert_eq!(empty.match_budget_usage(), &MatchBudgetUsage::default());
}

#[test]
fn budget_resets_each_step_and_after_a_halted_step_without_draining_guards() {
    let mut s = fixture();
    let mut e = engine(MatchBudgetPolicy::Pooled, 1000, 11);
    e.step(&mut s);
    let usage = e.match_budget_usage().clone();
    e.step(&mut s);
    assert_eq!(&usage, e.match_budget_usage());
    e.set_match_budget_policy(0, 0, MatchBudgetPolicy::Pooled);
    e.step(&mut s);
    assert!(e.match_ceiling_binding());
    e.set_match_budget_policy(1000, 11, MatchBudgetPolicy::Pooled);
    assert_eq!(e.step(&mut s).accepted_rewrites, 1);
    assert!(!e.match_ceiling_binding());
}

#[test]
fn scheduled_bindings_keep_schedule_semantics_but_do_not_apply_on_truncation() {
    let creation = compile_rules(
        &parse_rules_toml(
            r#"
[[rules]]
name="birth"
schedule={mode="every_n_steps",n=1}
[rules.lhs]
nodes=[]
edges=[]
[rules.rhs]
nodes=["a"]
edges=[]
"#,
        )
        .unwrap(),
    )
    .unwrap()
    .pop()
    .unwrap();
    let mut rs = rules();
    rs.push(creation.clone());
    let mut s = fixture();
    let mut e = Engine::new(rs, StepMode::MaxParallel, 42, 1);
    e.set_match_budget_policy(1000, 11, MatchBudgetPolicy::Pooled);
    assert_eq!(e.step(&mut s).accepted_rewrites, 2);
    assert_eq!(e.match_budget_usage().results, 10);
    let before = s.canonical_hash();
    e.set_match_budget_policy(0, 0, MatchBudgetPolicy::Pooled);
    assert_eq!(e.step(&mut s).accepted_rewrites, 0);
    assert_eq!(s.canonical_hash(), before);
    e.replace_static_rules(vec![creation]);
    assert_eq!(e.step(&mut s).accepted_rewrites, 1);
    assert!(!e.match_ceiling_binding());
}

#[test]
fn static_table_replacement_retains_pooled_budget_policy() {
    let mut e = engine(MatchBudgetPolicy::Pooled, 1000, 11);
    e.replace_static_rules(rules());
    assert_eq!(e.rules()[0].pattern.match_result_limit, Some(11));
    e.step(&mut fixture());
    assert!(!e.match_ceiling_binding());
}

#[cfg(feature = "parallel")]
#[test]
fn pooled_enumeration_is_thread_count_independent() {
    let run = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                let mut s = fixture();
                let mut e = engine(MatchBudgetPolicy::Pooled, 1000, 11);
                for _ in 0..10 {
                    e.step(&mut s);
                    assert!(!e.match_ceiling_binding());
                }
                (
                    s.canonical_hash(),
                    e.rng_state_probe(),
                    e.match_budget_usage().clone(),
                )
            })
    };
    assert_eq!(run(1), run(4));
}
