#![cfg(feature = "hg-meta")]
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_engine::Engine;
use hg_meta::*;
use hg_rules::{EpochPolicy, StepMode};
use hg_schema::MetricsRow;
use hg_sim_core::ere::{experiments, semantic, EreArm, EreConfig, EreRuntime};

#[test]
fn e2_identity_validators_protection_and_live_activation_tpm_control() {
    let v = EncodingVersion::AlternativeFamiliesV3;
    assert!(experiments::identity_gate_versioned(1000, v).unwrap());
    let gates = experiments::fixture_gates_versioned(100, v).unwrap();
    assert_eq!(gates["protocol"], v.protocol());
    assert_eq!(gates["pass"], true, "{gates}");
    assert_eq!(
        gates["quarantine_by_validator"],
        serde_json::json!([1, 1, 1, 1, 1, 1, 1, 1, 1])
    );
    assert!(gates["meta_editor_events"].as_u64().unwrap() > 0);
}

#[test]
fn unmatched_proposal_does_not_steal_core_reuse_then_matching_proposal_is_credited() {
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
            arm: EreArm::A2,
            encoding_version: EncodingVersion::AlternativeFamiliesV3,
            epoch: EpochPolicy::Fixed(1.try_into().unwrap()),
            ..EreConfig::default()
        },
        &mut s,
        &mut e,
    )
    .unwrap();
    let root = s.nodes().find(|(_, n)| n.label.0 == L_RULE_ROOT).unwrap().0;
    let parts = family_parts(&s.snapshot(), root, &MetaLimits::default()).unwrap();
    let var = parts
        .proposal
        .nodes
        .iter()
        .find(|&&n| {
            s.get_node(n).is_some_and(|n| {
                n.label.0 == L_VAR && n.get_property(K_ORDER) == Some(PropertyValue::Int(0))
            })
        })
        .copied()
        .unwrap();
    let mut lit = NodeData::new(Label(L_LABEL_LIT), 0);
    lit.set_property(K_LITERAL, PropertyValue::Int(1));
    let lit = s.add_node(lit);
    s.add_edge(HyperEdge::new(
        Label(E_HASLABEL),
        [var, lit].into_iter().collect(),
    ));
    s.get_node_mut(parts.proposal.root)
        .unwrap()
        .set_property(K_PRIORITY, PropertyValue::Int(1));
    s.get_node_mut(root)
        .unwrap()
        .set_property(K_GRAFT_MODE, PropertyValue::Int(1));
    let cold = read_program(&s.snapshot(), root, &MetaLimits::default())
        .unwrap()
        .proposal
        .unwrap();
    let rng = e.rng_state_probe();
    r.before_step(&s, &mut e).unwrap();
    assert_eq!(
        rng,
        e.rng_state_probe(),
        "family expansion may not draw RNG"
    );
    assert_eq!(e.rules().len(), 2);
    let stats = e.step(&mut s);
    r.after_step(&s, &stats);
    for _ in 1..10 {
        r.before_step(&s, &mut e).unwrap();
        let stats = e.step(&mut s);
        r.after_step(&s, &stats);
    }
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.rule_rcf_novel_cumulative, Some(1));
    assert_eq!(row.novel_nontrivial_classes, Some(0));
    let reuse = row.nontrivial_rule_reuse_count_by_class.unwrap();
    assert_eq!(reuse.get(&c.rcf_hash()), Some(&10));
    assert!(!reuse.contains_key(&cold.rcf_hash()));
    s.remove_node(lit);
    let matching = read_program(&s.snapshot(), root, &MetaLimits::default())
        .unwrap()
        .proposal
        .unwrap();
    r.before_step(&s, &mut e).unwrap();
    let stats = e.step(&mut s);
    r.after_step(&s, &stats);
    let row = r.close_window(&s, MetricsRow::default(), MetricsRow::default());
    assert_eq!(row.novel_nontrivial_classes, Some(1));
    assert_eq!(
        row.nontrivial_rule_reuse_count_by_class
            .unwrap()
            .get(&matching.rcf_hash()),
        Some(&1)
    );
}

#[test]
fn activation_really_changes_trajectories_and_deactivation_removes_alternative() {
    let c = experiments::seed_bank()[0].clone();
    let mut p = c.clone();
    p.priority += 1;
    p.rhs.push(vec![0, 0]);
    let mut graph = HypergraphStore::new();
    let encoded = encode_family(&mut graph, &c, &p, false, &MetaLimits::default()).unwrap();
    let dormant = read_program(&graph.snapshot(), encoded.root, &MetaLimits::default()).unwrap();
    graph
        .get_node_mut(encoded.root)
        .unwrap()
        .set_property(K_GRAFT_MODE, PropertyValue::Int(1));
    let active = read_program(&graph.snapshot(), encoded.root, &MetaLimits::default()).unwrap();
    let mut a = experiments::fixture(&c);
    let mut b = a.clone();
    let mut ea = Engine::new(
        dormant.compile(&MetaLimits::default(), false).unwrap(),
        StepMode::MaxParallel,
        42,
        1,
    );
    let mut eb = Engine::new(
        active.compile(&MetaLimits::default(), false).unwrap(),
        StepMode::MaxParallel,
        42,
        1,
    );
    ea.step(&mut a);
    eb.step(&mut b);
    assert_ne!(a.canonical_hash(), b.canonical_hash());
    graph
        .get_node_mut(encoded.root)
        .unwrap()
        .set_property(K_GRAFT_MODE, PropertyValue::Int(0));
    assert_eq!(
        read_program(&graph.snapshot(), encoded.root, &MetaLimits::default())
            .unwrap()
            .execution_bytes(),
        dormant.execution_bytes()
    );
}

#[test]
fn characterization_has_all_states_roles_operations_and_no_short_pass() {
    let bank = semantic::parent_bank();
    assert_eq!(bank.len(), 168);
    assert_eq!(bank.iter().filter(|(_, p)| p.active).count(), 84);
    let report = semantic::mutation_gate(100, 5);
    assert!(!report.pass);
    assert_eq!(report.by_operator.len(), 13);
    assert_eq!(report.valid + report.quarantined, 100);
    let s = report.semantic.unwrap();
    assert_eq!(s.structural.samples + s.activation.samples, 100);
    assert_eq!(s.target_roles.len(), 2);
    assert!(s.active_parent_structural.samples > 0);
    assert!(s.mode_changes > 0);
    assert!(s.trajectory_changing_activation > 0);
}

#[test]
fn aggregate_pass_cannot_bypass_missing_or_failed_semantic_strata() {
    let dir = tempfile::tempdir().unwrap();
    let receipt = serde_json::json!({
        "protocol":"E2-alternatives-v1", "pass":true, "samples":10000,
        "seeds":168, "trajectory_steps":100, "valid":8000, "quarantined":2000,
        "neutral":3000,"near_neutral":1500,"divergent":2000,"catastrophic":1500,
        "unmeasured":0,"at_cap_samples":100,"neutral_soundness_checked":50,"neutral_soundness_failures":0
    });
    std::fs::write(
        dir.path().join("exp_r01.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    assert!(experiments::require_research_gates_versioned(
        dir.path(),
        EncodingVersion::AlternativeFamiliesV3
    )
    .unwrap_err()
    .contains("strata"));
    let mut report = serde_json::to_value(semantic::mutation_gate(100, 5)).unwrap();
    report["pass"] = true.into();
    assert!(semantic::validate_receipt(&report).is_err());
    assert!(semantic::validate_receipt(&receipt).is_err());
}

#[test]
fn active_structural_failure_cannot_be_diluted_by_dormant_or_mode_noops() {
    use experiments::MutationBreakdown;
    let row = |samples, near| MutationBreakdown {
        samples,
        valid: samples,
        near_neutral: near,
        neutral: samples - near,
        ..MutationBreakdown::default()
    };
    let mut s = semantic::SemanticStrata {
        structural: row(8000, 1000),
        active_parent_structural: row(4000, 0),
        activation: row(2000, 0),
        mode_changes: 1000,
        trajectory_changing_activation: 100,
        ..semantic::SemanticStrata::default()
    };
    s.target_roles.insert("Core".into(), row(4000, 500));
    s.target_roles.insert("Proposal".into(), row(4000, 500));
    assert!(
        !s.pass(),
        "overall and structural NearNeutral rates cannot hide active failure"
    );
    s.active_parent_structural = row(4000, 400);
    assert!(s.pass());
    s.trajectory_changing_activation = 0;
    assert!(!s.pass(), "only no-op activations cannot pass");
}

#[test]
fn e2_production_and_rollout_share_the_same_loop_and_write_metrics() {
    let dir = tempfile::tempdir().unwrap();
    let mut config =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2.toml")).unwrap();
    config.engine.max_steps = 20;
    config.engine.metrics_window_interval = 10;
    config.out_dir = dir.path().into();
    config.ere.as_mut().unwrap().arm = EreArm::A2;
    config.ere.as_mut().unwrap().epoch = EpochPolicy::Fixed(1.try_into().unwrap());
    config.ere.as_mut().unwrap().meta_rules_toml = Some(experiments::seeded_editor_versioned(
        EncodingVersion::AlternativeFamiliesV3,
    ));
    let run = hg_sim_core::run_headless(&config, None, None, None).unwrap();
    assert!(run.run_dir.join("ere_metrics.jsonl").exists());
    assert!(run
        .history
        .iter()
        .any(|r| r.ere.as_ref().unwrap().meta_rewrite_events.unwrap() > 0));
    let rollout = hg_sim_core::runner::run_ere_rollout(&config).unwrap();
    assert_eq!(run.canonical_hash, rollout.canonical_hash);
    assert_eq!(run.history, rollout.history);
}
