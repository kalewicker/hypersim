#![cfg(feature = "hg-meta")]
use hg_core::PropertyValue;
use hg_engine::Engine;
use hg_meta::{CandidateRule, EncodingVersion, MetaLimits, K_GRAFT_MODE, L_RULE_ROOT};
use hg_rules::{EpochPolicy, MatchBudgetPolicy, StepMode};
use hg_schema::MetricsRow;
use hg_sim_core::ere::{self, experiments, EreArm, EreConfig, EreRuntime, EreSeedFamily};
use serde::Serialize;

fn family(core: &CandidateRule, proposal: &CandidateRule, active: bool) -> EreSeedFamily {
    #[derive(Serialize)]
    struct Rules {
        rules: Vec<hg_rules::Rule>,
    }
    let text = |c: &CandidateRule| {
        toml::to_string(&Rules {
            rules: vec![c.to_rule("allele".into())],
        })
        .unwrap()
    };
    EreSeedFamily {
        core_toml: text(core),
        proposal_toml: text(proposal),
        active,
    }
}
fn structural() -> CandidateRule {
    let mut p = experiments::seed_bank()[0].clone();
    p.priority = 1;
    p.labels = vec![None; 3];
    p.lhs = vec![vec![0, 1], vec![1, 2]];
    p.rhs = vec![vec![0, 1], vec![0, 2]];
    p
}
fn config(seeds: Vec<EreSeedFamily>, arm: EreArm) -> EreConfig {
    EreConfig {
        encoding_version: EncodingVersion::AlternativeFamiliesV3,
        match_budget_policy: MatchBudgetPolicy::Pooled,
        seed_families: Some(seeds),
        arm,
        epoch: EpochPolicy::Fixed(1.try_into().unwrap()),
        measure_ei: false,
        ..EreConfig::default()
    }
}

#[test]
fn maintenance_detects_between_epoch_damage_recovery_and_missing_anchors() {
    let c = experiments::seed_bank()[0].clone();
    let mut s = experiments::fixture(&c);
    let mut engine = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut cfg = config(vec![family(&c, &structural(), false)], EreArm::A1);
    cfg.measure_family_maintenance = true;
    let mut runtime = EreRuntime::install(&cfg, &mut s, &mut engine).unwrap();
    let root = s.nodes().find(|(_, n)| n.label.0 == L_RULE_ROOT).unwrap().0;
    for step in 1..=4 {
        if step == 2 {
            s.get_node_mut(root)
                .unwrap()
                .set_property(K_GRAFT_MODE, PropertyValue::Int(2));
        }
        if step == 4 {
            s.get_node_mut(root)
                .unwrap()
                .set_property(K_GRAFT_MODE, PropertyValue::Int(0));
        }
        runtime.before_step(&s, &mut engine).unwrap();
        let st = engine.step(&mut s);
        runtime.after_step(&s, &st);
    }
    let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
    let m = &row.seeded_family_maintenance.as_ref().unwrap()[0];
    assert_eq!((m.checks, m.valid_steps, m.final_valid), (4, 2, true));
    assert_eq!(
        (
            m.first_invalid_step,
            m.invalid_transitions,
            m.recovery_transitions
        ),
        (Some(2), 1, 1)
    );
    assert_eq!(row.anchor_survival_fraction, Some(1.0));
    assert_eq!(row.rules_quarantined, Some(0)); // Frozen compiler does not see the transient damage.
    s.remove_node(root);
    runtime.before_step(&s, &mut engine).unwrap();
    let st = engine.step(&mut s);
    runtime.after_step(&s, &st);
    let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
    let m = &row.seeded_family_maintenance.unwrap()[0];
    assert_eq!((m.checks, m.valid_steps, m.final_valid), (5, 2, false));
    assert_eq!(m.invalid_transitions, 2);
    assert!(m.final_failure.as_ref().unwrap().contains("missing"));
}

#[test]
fn opt_in_measurement_preserves_hash_counts_rng_and_separates_editor_roles() {
    let c = experiments::seed_bank()[0].clone();
    for arm in [EreArm::A2, EreArm::A3] {
        let mut a = experiments::fixture(&c);
        let mut b = a.clone();
        let rules = vec![c.compile(&MetaLimits::default(), false).unwrap()];
        let mut ea = Engine::new(rules.clone(), StepMode::MaxParallel, 42, 16);
        let mut eb = Engine::new(rules, StepMode::MaxParallel, 42, 16);
        let mut cfg = config(vec![family(&c, &c, false)], arm);
        cfg.meta_rules_toml = Some(experiments::seeded_editor_versioned(
            EncodingVersion::AlternativeFamiliesV3,
        ));
        let mut ra = EreRuntime::install(&cfg, &mut a, &mut ea).unwrap();
        cfg.measure_family_maintenance = true;
        let mut rb = EreRuntime::install(&cfg, &mut b, &mut eb).unwrap();
        // Deliberate ordinary-meta engagement control, outside encoded bodies.
        // An isolated meta-labeled node and a zero-labeled binary edge make a
        // guaranteed match; A2 blocks it, A3 must expose it. Both legs get it.
        for store in [&mut a, &mut b] {
            let meta = store.add_node(hg_core::NodeData::new(
                hg_core::Label(hg_meta::META_BASE + 999),
                0,
            ));
            let world = store.add_node(hg_core::NodeData::new(hg_core::Label(0), 0));
            store.add_edge(hg_core::HyperEdge::new(
                hg_core::Label(0),
                vec![meta, world].into(),
            ));
        }
        for _ in 0..40 {
            ra.before_step(&a, &mut ea).unwrap();
            rb.before_step(&b, &mut eb).unwrap();
            let sa = ea.step(&mut a);
            let sb = eb.step(&mut b);
            assert_eq!(
                (sa.accepted_rewrites, sa.attempted_matches),
                (sb.accepted_rewrites, sb.attempted_matches)
            );
            ra.after_step(&a, &sa);
            rb.after_step(&b, &sb);
            assert_eq!(a.canonical_hash(), b.canonical_hash());
            assert_eq!(ea.rng_state_probe(), eb.rng_state_probe());
        }
        let old = ra.close_window(&a, MetricsRow::default(), MetricsRow::default());
        assert!(old.seeded_family_maintenance.is_none());
        let row = rb.close_window(&b, MetricsRow::default(), MetricsRow::default());
        let editor = row.seeded_editor_rewrite_events.unwrap();
        let ordinary = row.ordinary_meta_rewrite_events.unwrap();
        assert!(editor > 0 && editor <= 32);
        assert_eq!(editor + ordinary, row.meta_rewrite_events.unwrap());
        if arm == EreArm::A2 {
            assert_eq!(ordinary, 0);
        } else {
            assert!(ordinary > 0);
        }
        assert_eq!(row.seeded_family_maintenance.unwrap()[0].checks, 40);
    }
}

#[test]
fn malformed_dormant_seed_and_mismatched_core_reject_before_store_writes() {
    let c = experiments::seed_bank()[0].clone();
    let p = structural();
    let mut s = experiments::fixture(&c);
    let hash = s.canonical_hash();
    let nodes = s.node_count();
    let mut engine = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut seed = family(&c, &p, false);
    seed.proposal_toml = "malformed = [".into();
    assert!(EreRuntime::install(&config(vec![seed], EreArm::A1), &mut s, &mut engine).is_err());
    assert_eq!(s.canonical_hash(), hash);
    assert_eq!(s.node_count(), nodes);
    let seed = family(&p, &c, false);
    assert!(
        EreRuntime::install(&config(vec![seed], EreArm::A1), &mut s, &mut engine)
            .err()
            .unwrap()
            .contains("core")
    );
    assert_eq!(s.canonical_hash(), hash);
    assert_eq!(s.node_count(), nodes);
}

#[test]
fn active_seeded_proposal_fires_without_initial_novelty_or_activation_credit() {
    let c = experiments::seed_bank()[0].clone();
    let p = structural();
    let mut s = experiments::fixture(&p);
    let mut engine = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut runtime = EreRuntime::install(
        &config(vec![family(&c, &p, true)], EreArm::A2),
        &mut s,
        &mut engine,
    )
    .unwrap();
    for _ in 0..10 {
        runtime.before_step(&s, &mut engine).unwrap();
        let st = engine.step(&mut s);
        runtime.after_step(&s, &st);
    }
    let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.rule_rcf_novel_cumulative, Some(0));
    assert_eq!(row.runtime_activation_count, Some(0));
    assert_eq!(
        row.proposal_rule_reuse_count_by_class
            .as_ref()
            .unwrap()
            .get(&p.rcf_hash()),
        Some(&10)
    );
    let mut structural = p.clone();
    structural.priority = 0;
    assert_eq!(
        row.structural_proposal_reuse_count_by_class
            .as_ref()
            .unwrap()
            .get(&structural.rcf_hash()),
        Some(&10)
    );
}

#[test]
fn unmatched_proposal_and_priority_copy_cannot_earn_structural_proposal_credit() {
    let c = experiments::seed_bank()[0].clone();
    for unmatched in [false, true] {
        let mut p = if unmatched { structural() } else { c.clone() };
        p.priority = 1;
        if unmatched {
            p.labels[0] = Some(1);
        }
        let mut s = experiments::fixture(&c);
        let mut engine = Engine::new(
            vec![c.compile(&MetaLimits::default(), false).unwrap()],
            StepMode::MaxParallel,
            42,
            1,
        );
        let mut runtime = EreRuntime::install(
            &config(vec![family(&c, &p, true)], EreArm::A2),
            &mut s,
            &mut engine,
        )
        .unwrap();
        for _ in 0..10 {
            runtime.before_step(&s, &mut engine).unwrap();
            let st = engine.step(&mut s);
            runtime.after_step(&s, &st);
        }
        let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
        assert!(row
            .structural_proposal_reuse_count_by_class
            .as_ref()
            .unwrap()
            .is_empty());
        assert_eq!(
            row.proposal_rule_reuse_count_by_class
                .as_ref()
                .unwrap()
                .values()
                .sum::<usize>(),
            if unmatched { 0 } else { 10 }
        );
    }
}

#[test]
fn dormant_proposal_only_gets_credit_after_runtime_activation() {
    let c = experiments::seed_bank()[0].clone();
    let p = structural();
    let mut s = experiments::fixture(&p);
    let mut engine = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut runtime = EreRuntime::install(
        &config(vec![family(&c, &p, false)], EreArm::A2),
        &mut s,
        &mut engine,
    )
    .unwrap();
    runtime.before_step(&s, &mut engine).unwrap();
    let st = engine.step(&mut s);
    runtime.after_step(&s, &st);
    let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert!(row.proposal_rule_reuse_count_by_class.unwrap().is_empty());
    let root = s.nodes().find(|(_, n)| n.label.0 == L_RULE_ROOT).unwrap().0;
    s.get_node_mut(root)
        .unwrap()
        .set_property(K_GRAFT_MODE, PropertyValue::Int(1));
    runtime.before_step(&s, &mut engine).unwrap();
    let st = engine.step(&mut s);
    runtime.after_step(&s, &st);
    let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.runtime_activation_count, Some(1));
    assert_eq!(row.rule_rcf_novel_cumulative, Some(1));
    let mut structural = p.clone();
    structural.priority = 0;
    assert_eq!(
        row.structural_proposal_reuse_count_by_class
            .unwrap()
            .get(&structural.rcf_hash()),
        Some(&1)
    );
}

#[test]
fn explicit_static_and_frozen_families_have_same_hash_counts_rng_and_baseline() {
    let c = experiments::seed_bank()[0].clone();
    let p = structural();
    let mut a = experiments::fixture(&p);
    let mut b = a.clone();
    let table = vec![c.compile(&MetaLimits::default(), false).unwrap()];
    let mut ea = Engine::new(table.clone(), StepMode::SequentialRandom, 42, 1);
    let mut eb = Engine::new(table, StepMode::SequentialRandom, 42, 1);
    let mut ra = EreRuntime::install(
        &config(vec![family(&c, &p, true)], EreArm::A0),
        &mut a,
        &mut ea,
    )
    .unwrap();
    let mut rb = EreRuntime::install(
        &config(vec![family(&c, &p, true)], EreArm::A1),
        &mut b,
        &mut eb,
    )
    .unwrap();
    for _ in 0..100 {
        ra.before_step(&a, &mut ea).unwrap();
        rb.before_step(&b, &mut eb).unwrap();
        let sa = ea.step(&mut a);
        let sb = eb.step(&mut b);
        ra.after_step(&a, &sa);
        rb.after_step(&b, &sb);
        assert_eq!(a.canonical_hash(), b.canonical_hash());
        assert_eq!(sa.accepted_rewrites, sb.accepted_rewrites);
        assert_eq!(sa.attempted_matches, sb.attempted_matches);
        assert_eq!(ea.rng_state_probe(), eb.rng_state_probe());
    }
    let xa = ra.close_window(&a, MetricsRow::default(), MetricsRow::default());
    let xb = rb.close_window(&b, MetricsRow::default(), MetricsRow::default());
    assert_eq!(xa.rule_rcf_novel_cumulative, Some(0));
    assert_eq!(xb.rule_rcf_novel_cumulative, Some(0));
    assert_eq!(
        xa.structural_proposal_reuse_count_by_class,
        xb.structural_proposal_reuse_count_by_class
    );
}

#[test]
fn priority_scaffolding_cannot_split_structural_reward_buckets() {
    let c = experiments::seed_bank()[0].clone();
    let p = structural();
    let mut s = experiments::fixture(&p);
    let mut engine = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut runtime = EreRuntime::install(
        &config(vec![family(&c, &p, true)], EreArm::A2),
        &mut s,
        &mut engine,
    )
    .unwrap();
    let root = s.nodes().find(|(_, n)| n.label.0 == L_RULE_ROOT).unwrap().0;
    let parts = hg_meta::family_parts(&s.snapshot(), root, &MetaLimits::default()).unwrap();
    for priority in 1..=5 {
        s.get_node_mut(parts.proposal.root)
            .unwrap()
            .set_property(hg_meta::K_PRIORITY, PropertyValue::Int(priority));
        runtime.before_step(&s, &mut engine).unwrap();
        let st = engine.step(&mut s);
        runtime.after_step(&s, &st);
    }
    let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
    let structural = row.structural_proposal_reuse_count_by_class.unwrap();
    assert_eq!(structural.len(), 1);
    assert_eq!(structural.values().sum::<usize>(), 5);
    assert_eq!(row.proposal_rule_reuse_count_by_class.unwrap().len(), 5);
}

#[test]
fn core_proposal_class_overlap_is_reported_ambiguous_not_rewarded() {
    let c = experiments::seed_bank()[0].clone();
    let p = structural();
    let mut s = experiments::fixture(&p);
    let mut copy = p.clone();
    copy.priority = 2;
    let mut engine = Engine::new(
        vec![
            c.compile(&MetaLimits::default(), false).unwrap(),
            p.compile(&MetaLimits::default(), false).unwrap(),
        ],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut runtime = EreRuntime::install(
        &config(
            vec![family(&c, &p, true), family(&p, &copy, false)],
            EreArm::A2,
        ),
        &mut s,
        &mut engine,
    )
    .unwrap();
    for _ in 0..8 {
        runtime.before_step(&s, &mut engine).unwrap();
        let st = engine.step(&mut s);
        runtime.after_step(&s, &st);
    }
    let row = runtime.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert!(row.proposal_rule_reuse_count_by_class.unwrap().is_empty());
    assert!(row
        .structural_proposal_reuse_count_by_class
        .unwrap()
        .is_empty());
    assert_eq!(
        row.ambiguous_role_reuse_count_by_class
            .unwrap()
            .values()
            .sum::<usize>(),
        8
    );
}

#[test]
fn admitted_resource_profile_recomputes_each_gate_and_preserves_legacy_rejection() {
    use serde_json::{json, Value};
    let dir = tempfile::tempdir().unwrap();
    let class = dir.path().join("class");
    let gates = dir.path().join("gates");
    std::fs::create_dir(&class).unwrap();
    std::fs::create_dir(&gates).unwrap();
    let row = |samples, neutral, near, div| {
        json!({"samples":samples,"valid":samples,"neutral":neutral,
        "near_neutral":near,"divergent":div,"quarantined":0,"frozen":0,"exploding":0,"unmeasured":0})
    };
    let one = json!({"protocol":ere::pooled::TRIAL_PROTOCOL,"match_budget_policy":"Pooled",
        "global_work":400000,"global_results":50000,"rng_seed":experiments::MUTATION_SEED,
        "pass":true,"samples":10000,"seeds":168,"trajectory_steps":100,"valid":10000,
        "neutral":7000,"near_neutral":2000,"divergent":1000,"quarantined":0,"catastrophic":0,
        "unmeasured":0,"neutral_soundness_checked":50,"neutral_soundness_failures":0,"at_cap_samples":100,
        "semantic":{"structural":row(8463,6000,1463,1000),"activation":row(1537,1000,537,0),
            "active_parent_structural":row(4250,3000,1000,250),"mode_changes":751,
            "trajectory_changing_activation":276,"target_roles":{"Core":row(4263,3000,763,500),
                "Proposal":row(4200,3000,700,500)}},
        "resource_usage":{"budget_violations":0,"match_ceiling_steps":0,"graph_ceiling_steps":0,
            "attempted_steps":487000,"peak_step_work":100,"peak_step_results":2}});
    let two:Vec<_>=(0..45).map(|i|json!({"file":format!("rule{i}.toml"),"name":format!("rule{i}"),
        "status":if i<2 {"identical"}else{"outside-envelope:distinctness"},"steps":if i<2 {1000}else{0}})).collect();
    let header = json!({"protocol":ere::POOLED_PROTOCOL,"match_budget_policy":"Pooled",
        "encoding":"AlternativeFamiliesV3","global_work":2000000,"global_results":100000,
        "trajectory_steps":1000,"library_rules":45});
    let three = json!({"protocol":ere::POOLED_PROTOCOL,"match_budget_policy":"Pooled",
        "pass":true,"graph_hashes_equal":true,"canonical_csv_equal":true,"rng_states_equal":true,
        "rng_consumed":true,"every_step_advanced":true,"steps":100000,"completed_steps":100000,
        "a0_world_step":100000,"a1_world_step":100000,"match_ceiling_bound":false,
        "accepted_rewrites":100000,"attempted_matches":200000,"canonical_csv_bytes":3000000,
        "global_work":50000,"global_results":50000,"peak_step_work":24,"peak_step_results":2});
    let f = json!({"protocol":ere::POOLED_PROTOCOL,"match_budget_policy":"Pooled","pass":true,
        "gr0":true,"gr4":true,"gr5":true,"gr6":true,"rcf_pairs":40,
        "quarantine_by_validator":[1,1,1,1,1,1,1,1,1],"protection_steps":10000,
        "protection_completed_steps":10000,"protection_world_step":10000,
        "protection_accepted_rewrites":10000,"meta_editor_events":32,"global_work":50000,
        "global_results":50000,"peak_step_work":56,"peak_step_results":5,
        "changed_rule_set_stable":false,"changed_meta_included_ei":null,"changed_meta_excluded_ei":null});
    let write = |folder: &std::path::Path, name: &str, value: &Value| {
        std::fs::write(folder.join(name), serde_json::to_vec(value).unwrap()).unwrap()
    };
    write(&class, "exp_r01.json", &one);
    write(&gates, "exp_r02.json", &json!(two));
    write(&gates, "exp_r02_encoding.json", &header);
    write(&gates, "exp_r03.json", &three);
    write(&gates, "fixture_gates.json", &f);
    assert_eq!(
        ere::admission::require_pooled_400k50k_gates(&class, &gates).unwrap()["pass"],
        true
    );
    assert!(experiments::require_research_gates_with_policy(
        &class,
        EncodingVersion::AlternativeFamiliesV3,
        MatchBudgetPolicy::Pooled
    )
    .is_err());
    for key in [
        "pass",
        "graph_hashes_equal",
        "rng_states_equal",
        "every_step_advanced",
    ] {
        let mut bad = three.clone();
        bad[key] = false.into();
        write(&gates, "exp_r03.json", &bad);
        assert!(ere::admission::require_pooled_400k50k_gates(&class, &gates).is_err());
    }
    write(&gates, "exp_r03.json", &three);
    let mut bad = one.clone();
    bad["global_work"] = 200000.into();
    write(&class, "exp_r01.json", &bad);
    assert!(ere::admission::require_pooled_400k50k_gates(&class, &gates).is_err());
    write(&class, "exp_r01.json", &one);
    let mut bad = f;
    bad["changed_meta_included_ei"] = 0.1.into();
    write(&gates, "fixture_gates.json", &bad);
    assert!(ere::admission::require_pooled_400k50k_gates(&class, &gates).is_err());
}
