#![cfg(feature = "hg-meta")]
use hg_core::{HypergraphStore, Label, NodeData};
use hg_engine::Engine;
use hg_meta::{MetaLimits, META_BASE};
use hg_rules::{compile_rule, EpochPolicy, StepMode};
use hg_schema::MetricsRow;
use hg_sim_core::{
    ere::{experiments, meta_excluded_store, EreArm, EreConfig, EreRuntime},
    SimConfig,
};

fn setup(arm: EreArm, epoch: EpochPolicy) -> (HypergraphStore, Engine, EreRuntime) {
    let c = experiments::seed_bank()[0].clone();
    let mut store = experiments::fixture(&c);
    let mut engine = Engine::new(
        vec![compile_rule(&c.to_rule("fixture".into())).unwrap()],
        StepMode::MaxParallel,
        42,
        1,
    );
    let config = EreConfig {
        arm,
        epoch,
        min_tpm_transitions: 1,
        ..EreConfig::default()
    };
    let runtime = EreRuntime::install(&config, &mut store, &mut engine).unwrap();
    (store, engine, runtime)
}
fn step(s: &mut HypergraphStore, e: &mut Engine, r: &mut EreRuntime) {
    r.before_step(s, e).unwrap();
    let stats = e.step(s);
    r.after_step(s, &stats);
    assert!(!e.match_ceiling_binding());
}

#[test]
fn gr3_frozen_identity_short_and_nontrivial_engagement() {
    assert!(experiments::identity_gate(1000).unwrap());
    let (mut s, mut e, mut r) = setup(EreArm::A1, EpochPolicy::Never);
    for _ in 0..10 {
        step(&mut s, &mut e, &mut r);
    }
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(
        row.rule_replicator_kind,
        Some(hg_schema::RuleReplicatorKind::NonTrivial)
    );
    assert_eq!(row.quarantine_by_validator, Some([0; 9]));
    assert_eq!(row.rule_rcf_novel_cumulative, Some(0));
    let included = row.meta_included.as_ref().unwrap();
    let excluded = row.meta_excluded.as_ref().unwrap();
    assert!(included.accepted_rewrites.unwrap() > 0);
    assert_eq!(excluded.accepted_rewrites, included.accepted_rewrites);
    assert!(included.attempted_matches.unwrap() > 0);
    assert_eq!(excluded.attempted_matches, None);
    assert!(excluded
        .unmeasured_reasons
        .contains_key("attempted_matches"));
    assert!(included.ce_structural.is_some());
    assert!(included.ce_energetic.is_some());
    assert!(included.wl_class_count.unwrap() > 0);
}

#[test]
fn d2_epoch_compile_never_draws_engine_rng_and_positive_control_does() {
    let (s, mut e, _) = setup(EreArm::A1, EpochPolicy::Never);
    let before = e.rng_state_probe();
    e.compile_at_boundary(&s);
    assert_eq!(before, e.rng_state_probe());
    let c = experiments::seed_bank()[0].clone();
    let mut s = experiments::fixture(&c);
    let mut sequential = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::SequentialRandom,
        42,
        1,
    );
    let before = sequential.rng_state_probe();
    sequential.step(&mut s);
    assert_ne!(
        before,
        sequential.rng_state_probe(),
        "positive sampling control must consume RNG"
    );
}

#[test]
fn native_fixture_gates_have_live_meta_editor_engagement() {
    let gates = experiments::fixture_gates(100).unwrap();
    assert_eq!(gates["pass"], true);
}

#[test]
fn temporal_block_ci_has_explicit_short_window_limit_and_reproducibility() {
    use hg_core::WlHash;
    let ticks = vec![vec![(WlHash(1), WlHash(1)), (WlHash(2), WlHash(2))]; 64];
    let a = hg_sim_core::ere::block_bootstrap_ei_width(&ticks, 42);
    assert_eq!(a, Some(0.0));
    assert_eq!(a, hg_sim_core::ere::block_bootstrap_ei_width(&ticks, 42));
    assert_eq!(
        hg_sim_core::ere::block_bootstrap_ei_width(&ticks[..10], 42),
        None
    );
}

#[test]
fn gr5_protected_anchors_and_no_meta_cleanup_for_1000_steps() {
    let (mut s, mut e, mut r) = setup(EreArm::A2, EpochPolicy::Fixed(10.try_into().unwrap()));
    for _ in 0..1000 {
        step(&mut s, &mut e, &mut r);
        assert!(r.protection_intact(&s));
    }
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.anchor_survival_fraction, Some(1.0));
    assert_eq!(row.meta_edges_lost_to_cleanup, Some(0));
    assert_eq!(row.epoch_len_effective, Some(10));
}

#[test]
fn gr6_rule_change_invalidates_both_tpm_measurements() {
    let (mut s, mut e, mut r) = setup(EreArm::A3, EpochPolicy::Fixed(1.try_into().unwrap()));
    step(&mut s, &mut e, &mut r);
    let root = s
        .nodes()
        .find(|(_, n)| n.label.0 == hg_meta::L_RULE_ROOT)
        .unwrap()
        .0;
    s.get_node_mut(root)
        .unwrap()
        .set_property(hg_meta::K_PRIORITY, hg_core::PropertyValue::Int(2));
    step(&mut s, &mut e, &mut r);
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.tpm_rule_set_stable, Some(false));
    assert!(row.meta_included.as_ref().unwrap().ei_micro.is_none());
    assert!(row.meta_excluded.as_ref().unwrap().ei_micro.is_none());
    assert!(row.ei_unmeasured_reason.as_ref().unwrap().contains("G-R6"));
    assert_eq!(row.rule_rcf_novel_cumulative, Some(1));
}

#[test]
fn meta_projection_removes_incident_edges_before_wl() {
    let (s, _, _) = setup(EreArm::A1, EpochPolicy::Never);
    let projected = meta_excluded_store(&s);
    assert!(projected.nodes().all(|(_, n)| n.label.0 < META_BASE));
    assert_eq!(projected.node_count(), 2);
    assert_eq!(projected.edge_count(), 1);
}

#[test]
fn malformed_epoch_quarantines_instead_of_panicking() {
    let (mut s, mut e, mut r) = setup(EreArm::A2, EpochPolicy::Fixed(1.try_into().unwrap()));
    step(&mut s, &mut e, &mut r);
    let root = s
        .nodes()
        .find(|(_, n)| n.label.0 == hg_meta::L_RULE_ROOT)
        .unwrap()
        .0;
    s.get_node_mut(root)
        .unwrap()
        .set_property(hg_meta::K_PRIORITY, hg_core::PropertyValue::Bool(true));
    step(&mut s, &mut e, &mut r);
    assert!(e.rules().is_empty());
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.rules_quarantined, Some(1));
    assert_eq!(row.rules_retired, Some(1));
}

#[test]
fn meta_guard_applies_to_unconstrained_and_isolated_vars() {
    let raw = hg_rules::parse_rules_toml(
        r#"
[[rules]]
name="isolated"
[rules.lhs]
nodes=["a"]
edges=[]
[rules.rhs]
nodes=[]
edges=[]
"#,
    )
    .unwrap();
    let mut r = compile_rule(&raw[0]).unwrap();
    r.pattern.node_label_ceiling = Some(META_BASE);
    let mut s = HypergraphStore::new();
    s.add_node(NodeData::new(Label(hg_meta::L_RULE_ROOT), 0));
    assert!(hg_engine::find_bindings(&s.snapshot(), &r, 0).is_empty());
}

#[test]
fn binding_match_ceiling_does_not_apply_partial_pool() {
    let c = experiments::seed_bank()[0].clone();
    let mut s = experiments::fixture(&c);
    let hash = s.canonical_hash();
    let mut e = Engine::new(
        vec![c.compile(&MetaLimits::default(), false).unwrap()],
        StepMode::MaxParallel,
        42,
        100,
    );
    e.set_match_budget(1, 1);
    e.step(&mut s);
    assert!(e.match_ceiling_binding());
    assert_eq!(s.canonical_hash(), hash);
    assert_eq!(s.step(), 0);
}

#[test]
fn metrics_attachment_preserves_legacy_bincode_and_csv_shape() {
    let a = MetricsRow::default();
    let mut b = a.clone();
    b.ere = Some(Box::default());
    assert_eq!(
        bincode::serialize(&a).unwrap(),
        bincode::serialize(&b).unwrap()
    );
    let restored: MetricsRow = bincode::deserialize(&bincode::serialize(&b).unwrap()).unwrap();
    assert!(restored.ere.is_none());
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
}

#[test]
fn production_pipeline_writes_ere_artifacts_and_rollout_uses_same_loop() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = SimConfig::from_toml(
        r#"
experiment_name="ere_integration"
seed=42
run_id="test"
[rules]
source="Inline"
inline_toml='''
[[rules]]
name="identity"
[rules.lhs]
nodes=["a","b"]
edges=[["a","b"]]
[rules.rhs]
nodes=["a","b"]
edges=[["a","b"]]
'''
[initial]
node_count=8
[engine]
max_steps=20
metrics_window_interval=10
[evaluator]
enabled=false
[causal]
enabled=false
[output]
fixed_provenance=true
[ere]
arm="A1"
"#,
    )
    .unwrap();
    config.out_dir = dir.path().into();
    let run = hg_sim_core::run_headless(&config, None, None, None).unwrap();
    assert!(run.history.iter().all(|row| row.ere.is_some()));
    assert!(run.run_dir.join("ere_metrics.jsonl").exists());
    let rollout = hg_sim_core::runner::run_ere_rollout(&config).unwrap();
    assert_eq!(run.canonical_hash, rollout.canonical_hash);
    assert_eq!(run.history, rollout.history);
}

#[test]
fn gr1_small_protocol_smoke_never_claims_full_gate() {
    let report = experiments::mutation_gate(200, 10);
    assert!(!report.pass);
    assert!(report.seeds >= 20);
    assert_eq!(report.valid + report.quarantined, 200);
    assert_eq!(
        report.neutral
            + report.near_neutral
            + report.divergent
            + report.catastrophic
            + report.unmeasured,
        report.valid
    );
}

#[test]
fn research_gate_recomputes_thresholds_instead_of_trusting_pass_flag() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = serde_json::json!({
        "protocol":"E1-native-v2", "pass":true, "samples":10000,
        "seeds":28, "trajectory_steps":100, "valid":8184, "quarantined":1816,
        "neutral":2052, "near_neutral":1344, "divergent":1761,
        "catastrophic":3027, "unmeasured":0, "at_cap_samples":1428,
        "neutral_soundness_checked":50, "neutral_soundness_failures":0
    });
    let write = |value: &serde_json::Value| {
        std::fs::write(
            dir.path().join("exp_r01.json"),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap();
    };
    write(&receipt);
    assert!(experiments::require_research_gates(dir.path())
        .unwrap_err()
        .contains("G-R1"));
    receipt["catastrophic"] = 2000.into();
    receipt["divergent"] = 2788.into();
    write(&receipt);
    // A valid G-R1 receipt advances to the next prerequisite, never to research.
    assert!(experiments::require_research_gates(dir.path())
        .unwrap_err()
        .contains("exp_r02.json"));
}

#[test]
fn shared_encoding_roundtrips_and_engages_all_live_fixture_gates() {
    let version = hg_meta::EncodingVersion::SharedPatternsV2;
    assert!(experiments::identity_gate_versioned(1000, version).unwrap());
    let gates = experiments::fixture_gates_versioned(100, version).unwrap();
    assert_eq!(gates["pass"], true);
    assert_eq!(gates["protocol"], version.protocol());
    assert_eq!(
        gates["quarantine_by_validator"],
        serde_json::json!(vec![1; 9])
    );
    let report = experiments::mutation_gate_versioned(100, 10, version);
    assert!(!report.pass, "a smoke test must never claim the full gate");
    assert_eq!(report.protocol, version.protocol());
    assert_eq!(
        report
            .by_operator
            .values()
            .map(|r| r.samples)
            .sum::<usize>(),
        100
    );
    assert_eq!(
        report
            .by_seed_shape
            .values()
            .map(|r| r.samples)
            .sum::<usize>(),
        100
    );
    assert_eq!(
        report
            .by_operator
            .values()
            .map(|r| r.frozen + r.exploding)
            .sum::<usize>(),
        report.catastrophic
    );
}

#[test]
fn encoding_version_is_explicit_and_cannot_reuse_legacy_research_receipts() {
    use hg_meta::EncodingVersion;
    let legacy = EreConfig::default();
    let value = serde_json::to_value(&legacy).unwrap();
    assert!(
        value.get("encoding_version").is_none(),
        "preserve old config hashes"
    );
    let shared = EreConfig {
        encoding_version: EncodingVersion::SharedPatternsV2,
        ..legacy
    };
    let value = serde_json::to_value(shared).unwrap();
    assert_eq!(value["encoding_version"], "SharedPatternsV2");
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = serde_json::json!({"protocol":"E1-native-v2", "pass":true,
        "samples":10000,"seeds":28,"trajectory_steps":100,"valid":8000,"quarantined":2000,
        "neutral":2000,"near_neutral":1200,"divergent":2800,"catastrophic":2000,
        "unmeasured":0,"at_cap_samples":1000,"neutral_soundness_checked":50,"neutral_soundness_failures":0});
    let write = |r: &serde_json::Value| {
        std::fs::write(
            dir.path().join("exp_r01.json"),
            serde_json::to_vec(r).unwrap(),
        )
        .unwrap()
    };
    write(&receipt);
    assert!(experiments::require_research_gates_versioned(
        dir.path(),
        EncodingVersion::SharedPatternsV2
    )
    .unwrap_err()
    .contains("G-R1"));
    receipt["protocol"] = EncodingVersion::SharedPatternsV2.protocol().into();
    write(&receipt);
    assert!(experiments::require_research_gates_versioned(
        dir.path(),
        EncodingVersion::SharedPatternsV2
    )
    .unwrap_err()
    .contains("exp_r02.json"));
    std::fs::write(
        dir.path().join("exp_r02.json"),
        serde_json::to_vec(&serde_json::json!([{"status":"identical","steps":1000}])).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("exp_r02_encoding.json"),
        serde_json::to_vec(&serde_json::json!({"protocol":EncodingVersion::LegacyV1.protocol()}))
            .unwrap(),
    )
    .unwrap();
    assert!(experiments::require_research_gates_versioned(
        dir.path(),
        EncodingVersion::SharedPatternsV2
    )
    .unwrap_err()
    .contains("different encoding"));
    // Version binding is symmetric: E1.1 receipts cannot unlock legacy runs.
    receipt["protocol"] = EncodingVersion::LegacyV1.protocol().into();
    write(&receipt);
    std::fs::write(
        dir.path().join("exp_r02_encoding.json"),
        serde_json::to_vec(
            &serde_json::json!({"protocol":EncodingVersion::SharedPatternsV2.protocol()}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(experiments::require_research_gates(dir.path())
        .unwrap_err()
        .contains("different encoding"));
}
