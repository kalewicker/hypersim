#![cfg(feature = "hg-meta")]
use hg_core::{HypergraphStore, PropertyValue};
use hg_engine::Engine;
use hg_meta::{family_parts, EncodingVersion, MetaLimits, K_GRAFT_MODE, K_PRIORITY, L_RULE_ROOT};
use hg_rules::{EpochPolicy, MatchBudgetPolicy, StepMode};
use hg_schema::MetricsRow;
use hg_sim_core::ere::{
    self, experiments, semantic, EreArm, EreConfig, EreRuntime, POOLED_PROTOCOL,
};

#[test]
fn default_serialization_hash_is_unchanged_and_pooled_is_explicit() {
    let split = EreConfig::default();
    let value = serde_json::to_value(&split).unwrap();
    assert!(value.get("match_budget_policy").is_none());
    let mut pool = split;
    pool.encoding_version = EncodingVersion::AlternativeFamiliesV3;
    pool.match_budget_policy = MatchBudgetPolicy::Pooled;
    assert_eq!(
        serde_json::to_value(&pool).unwrap()["match_budget_policy"],
        "Pooled"
    );
    assert!(ere::gate_protocol(EncodingVersion::LegacyV1, MatchBudgetPolicy::Pooled).is_err());
    let config =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    assert_eq!(
        config.ere.unwrap().match_budget_policy,
        MatchBudgetPolicy::Pooled
    );
}

#[test]
fn source_expansion_and_epoch_changes_retain_pooling_and_dont_draw_rng() {
    let c = experiments::seed_bank()[0].clone();
    let mut s = experiments::fixture(&c);
    let mut e = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut r = EreRuntime::install(
        &EreConfig {
            encoding_version: EncodingVersion::AlternativeFamiliesV3,
            match_budget_policy: MatchBudgetPolicy::Pooled,
            arm: EreArm::A2,
            epoch: EpochPolicy::Fixed(1.try_into().unwrap()),
            ..EreConfig::default()
        },
        &mut s,
        &mut e,
    )
    .unwrap();
    let rng = e.rng_state_probe();
    r.before_step(&s, &mut e).unwrap();
    assert_eq!(rng, e.rng_state_probe());
    let st = e.step(&mut s);
    r.after_step(&s, &st);
    let root = s.nodes().find(|(_, n)| n.label.0 == L_RULE_ROOT).unwrap().0;
    let p = family_parts(&s.snapshot(), root, &MetaLimits::default()).unwrap();
    s.get_node_mut(p.proposal.root)
        .unwrap()
        .set_property(K_PRIORITY, PropertyValue::Int(1));
    s.get_node_mut(root)
        .unwrap()
        .set_property(K_GRAFT_MODE, PropertyValue::Int(1));
    let rng = e.rng_state_probe();
    r.before_step(&s, &mut e).unwrap();
    assert_eq!(rng, e.rng_state_probe());
    assert_eq!(e.rules().len(), 2);
    assert!(e
        .rules()
        .iter()
        .all(|r| r.pattern.match_work_limit == Some(50_000)));
    let st = e.step(&mut s);
    r.after_step(&s, &st);
    assert!(!e.match_ceiling_binding());
    assert_eq!(e.match_budget_usage().rules.len(), 2);
    assert!(e.match_budget_usage().work <= 50_000);
    assert!(e.match_budget_usage().results <= 50_000);
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.tpm_rule_set_stable, Some(false));
}

#[test]
fn pooled_small_assay_and_positive_fixtures_are_policy_bound() {
    let report = semantic::mutation_gate_with_policy(100, 5, MatchBudgetPolicy::Pooled);
    assert!(!report.pass);
    assert_eq!(report.protocol, POOLED_PROTOCOL);
    assert_eq!(report.match_budget_policy, Some(MatchBudgetPolicy::Pooled));
    let gates = experiments::fixture_gates_with_policy(
        100,
        EncodingVersion::AlternativeFamiliesV3,
        MatchBudgetPolicy::Pooled,
    )
    .unwrap();
    assert_eq!(gates["pass"], true);
    assert_eq!(gates["protocol"], POOLED_PROTOCOL);
    assert_eq!(gates["match_budget_policy"], "Pooled");
    assert!(experiments::identity_gate_with_policy(
        1000,
        EncodingVersion::AlternativeFamiliesV3,
        MatchBudgetPolicy::Pooled
    )
    .unwrap());
}

#[test]
fn larger_budget_trial_has_distinct_receipt_and_preserves_small_completed_assay() {
    let original = semantic::mutation_gate_with_policy(100, 5, MatchBudgetPolicy::Pooled);
    let mut expected = serde_json::to_value(original).unwrap();
    let mut trial = semantic::mutation_gate_400k50k(100, 5);
    assert_eq!(trial["protocol"], ere::pooled::TRIAL_PROTOCOL);
    assert_eq!(trial["global_work"], 400_000);
    assert_eq!(trial["global_results"], 50_000);
    assert_eq!(trial["pass"], false); // A short assay is never an acceptance receipt.
    let usage = &trial["resource_usage"];
    assert!(usage["attempted_steps"].as_u64().unwrap() > 0);
    assert!(usage["peak_step_work"].as_u64().unwrap() <= 400_000);
    assert!(usage["peak_step_results"].as_u64().unwrap() <= 50_000);
    assert_eq!(usage["budget_violations"], 0);
    for key in ["protocol", "global_work", "global_results", "rng_seed", "kind",
        "resource_usage", "previously_unmeasured_classifications"] {
        trial.as_object_mut().unwrap().remove(key);
    }
    expected.as_object_mut().unwrap().remove("protocol");
    assert_eq!(trial, expected);
}

#[test]
fn larger_budget_receipt_does_not_unlock_original_pooled_campaign() {
    let dir = tempfile::tempdir().unwrap();
    let trial = serde_json::json!({"protocol":ere::pooled::TRIAL_PROTOCOL,
        "match_budget_policy":"Pooled","pass":true});
    std::fs::write(dir.path().join("exp_r01.json"), serde_json::to_vec(&trial).unwrap()).unwrap();
    assert!(experiments::require_research_gates_with_policy(dir.path(),
        EncodingVersion::AlternativeFamiliesV3, MatchBudgetPolicy::Pooled)
        .unwrap_err().contains("G-R1"));
    // Existing artifact protection is checked before any expensive trial work.
    assert!(experiments::cli(&["gr1-e2p-400k50k".into(),
        dir.path().to_str().unwrap().into()]).unwrap_err().contains("fresh receipt"));
}

#[test]
fn larger_budget_replay_requires_all_preregistered_targets() {
    let cases: Vec<_> = (0..14).map(|i| serde_json::json!({"sample_index_zero_based":i})).collect();
    let original = serde_json::json!({"protocol":EncodingVersion::AlternativeFamiliesV3.protocol(),
        "kind":"diagnostic_replay_not_gate","genotype_draws_replayed":10000,
        "trajectory_steps":100,"rng_seed":experiments::MUTATION_SEED,"cases":cases});
    assert!(ere::pooled::replay_400k50k(&original).unwrap_err().contains("missing preregistered"));
}

#[test]
fn identity_receipt_records_progress_engagement_rng_and_caps_and_zero_steps_cannot_pass() {
    let report = experiments::identity_gate_report(1000,
        EncodingVersion::AlternativeFamiliesV3, MatchBudgetPolicy::Pooled).unwrap();
    for key in ["pass", "graph_hashes_equal", "canonical_csv_equal", "rng_states_equal",
        "rng_consumed", "every_step_advanced"] {
        assert_eq!(report[key], true, "{key}");
    }
    for key in ["steps", "completed_steps", "a0_world_step", "a1_world_step"] {
        assert_eq!(report[key], 1000, "{key}");
    }
    assert!(report["accepted_rewrites"].as_u64().unwrap() > 0);
    assert!(report["attempted_matches"].as_u64().unwrap() > 0);
    assert_eq!(report["match_ceiling_bound"], false);
    assert!(report["peak_step_work"].as_u64().unwrap() <= 50_000);
    assert!(report["peak_step_results"].as_u64().unwrap() <= 50_000);
    assert_eq!(experiments::identity_gate_report(0,
        EncodingVersion::AlternativeFamiliesV3, MatchBudgetPolicy::Pooled).unwrap()["pass"], false);
}

#[test]
fn protection_receipt_records_real_steps_live_editor_and_stationarity_suppression() {
    let report = experiments::fixture_gates_with_policy(100,
        EncodingVersion::AlternativeFamiliesV3, MatchBudgetPolicy::Pooled).unwrap();
    assert_eq!(report["pass"], true);
    assert_eq!(report["protection_completed_steps"], 100);
    assert_eq!(report["protection_world_step"], 100);
    assert!(report["protection_accepted_rewrites"].as_u64().unwrap() > 0);
    assert!(report["meta_editor_events"].as_u64().unwrap() > 0);
    assert_eq!(report["changed_rule_set_stable"], false);
    assert!(report["changed_meta_included_ei"].is_null());
    assert!(report["changed_meta_excluded_ei"].is_null());
    assert_eq!(report["quarantine_by_validator"], serde_json::json!([1,1,1,1,1,1,1,1,1]));
}

#[test]
fn pooled_and_split_receipts_cannot_cross_unlock_research() {
    let dir = tempfile::tempdir().unwrap();
    let base = serde_json::json!({"protocol":POOLED_PROTOCOL,"match_budget_policy":"Pooled",
        "pass":true,"samples":10000,"trajectory_steps":100,"seeds":168,"valid":8000,
        "quarantined":2000,"neutral":3000,"near_neutral":1500,"divergent":2000,
        "catastrophic":1500,"unmeasured":0,"at_cap_samples":100,
        "neutral_soundness_checked":50,"neutral_soundness_failures":0});
    let write = |v: &serde_json::Value| {
        std::fs::write(
            dir.path().join("exp_r01.json"),
            serde_json::to_vec(v).unwrap(),
        )
        .unwrap()
    };
    write(&base);
    let split = experiments::require_research_gates_versioned(
        dir.path(),
        EncodingVersion::AlternativeFamiliesV3,
    )
    .unwrap_err();
    assert!(split.contains("G-R1"));
    let pool = experiments::require_research_gates_with_policy(
        dir.path(),
        EncodingVersion::AlternativeFamiliesV3,
        MatchBudgetPolicy::Pooled,
    )
    .unwrap_err();
    assert!(pool.contains("strata"));
    let mut old = base.clone();
    old["protocol"] = EncodingVersion::AlternativeFamiliesV3.protocol().into();
    old.as_object_mut().unwrap().remove("match_budget_policy");
    write(&old);
    assert!(experiments::require_research_gates_with_policy(
        dir.path(),
        EncodingVersion::AlternativeFamiliesV3,
        MatchBudgetPolicy::Pooled
    )
    .unwrap_err()
    .contains("G-R1"));
    let mut forged = base.clone();
    forged["match_budget_policy"] = "Split".into();
    write(&forged);
    assert!(experiments::require_research_gates_with_policy(
        dir.path(),
        EncodingVersion::AlternativeFamiliesV3,
        MatchBudgetPolicy::Pooled
    )
    .unwrap_err()
    .contains("G-R1"));
}

#[test]
fn production_output_and_rollout_use_pooled_policy_on_the_same_loop() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    cfg.out_dir = dir.path().into();
    cfg.engine.max_steps = 20;
    cfg.engine.metrics_window_interval = 10;
    cfg.ere.as_mut().unwrap().arm = EreArm::A2;
    cfg.ere.as_mut().unwrap().epoch = EpochPolicy::Fixed(1.try_into().unwrap());
    cfg.ere.as_mut().unwrap().meta_rules_toml = Some(experiments::seeded_editor_versioned(
        EncodingVersion::AlternativeFamiliesV3,
    ));
    let a = hg_sim_core::run_headless(&cfg, None, None, None).unwrap();
    let b = hg_sim_core::runner::run_ere_rollout(&cfg).unwrap();
    assert_eq!(a.history, b.history);
    assert_eq!(a.canonical_hash, b.canonical_hash);
    assert!(a.run_dir.join("ere_metrics.jsonl").exists());
}

#[test]
fn unsupported_pooled_encoding_fails_before_writing_the_store() {
    let c = experiments::seed_bank()[0].clone();
    let mut s = HypergraphStore::new();
    let hash = s.canonical_hash();
    let mut e = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    assert!(EreRuntime::install(
        &EreConfig {
            match_budget_policy: MatchBudgetPolicy::Pooled,
            ..EreConfig::default()
        },
        &mut s,
        &mut e
    )
    .is_err());
    assert_eq!(s.canonical_hash(), hash);
}
