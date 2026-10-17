#![cfg(feature = "ere")]
use hg_train::ere::{emit, Budget, PARAMS};

#[test]
fn grammar_is_reproducible_finite_and_all_productions_parse() {
    let theta = vec![0.0; PARAMS];
    assert_eq!(emit(&theta, 42).unwrap(), emit(&theta, 42).unwrap());
    assert!(emit(&[0.0; 63], 42).is_err());
    let mut bad = theta.clone();
    bad[0] = f32::NAN;
    assert!(emit(&bad, 42).is_err());
    for production in 0..16 {
        let mut theta = vec![-20.0; PARAMS];
        for group in 0..4 {
            theta[group * 16 + production] = 20.0;
        }
        let rules = hg_rules::parse_rules_toml(&emit(&theta, 42).unwrap()).unwrap();
        assert_eq!(rules.len(), 4);
        assert!(hg_rules::compile_rules(&rules).is_ok());
    }
}

#[test]
fn user_budget_and_gate_failure_are_preserved_before_any_training() {
    let budget = Budget::default();
    assert_eq!(
        (budget.generations, budget.population, budget.steps_per_eval),
        (20, 16, 10_000)
    );
    let base = hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_native.toml"))
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("campaign");
    let result =
        hg_train::ere::run_campaign(&base, &out, &dir.path().join("missing_receipts"), &budget);
    assert!(result.is_err());
    assert!(out.join("manifest.json").exists());
    assert!(out.join("report.md").exists());
    assert!(!out.join("campaign.csv").exists());
}

#[test]
fn pooled_campaign_rejects_split_receipts_before_any_probe_or_optimizer() {
    let base =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    let dir = tempfile::tempdir().unwrap();
    // Even an aggregate passing split E2 receipt cannot unlock pooled research.
    let split = serde_json::json!({"protocol":"E2-alternatives-v1","pass":true,
        "samples":10000,"trajectory_steps":100,"seeds":168,"valid":8000,
        "quarantined":2000,"neutral":3000,"near_neutral":1500,"divergent":2000,
        "catastrophic":1500,"unmeasured":0,"at_cap_samples":100,
        "neutral_soundness_checked":50,"neutral_soundness_failures":0});
    std::fs::write(
        dir.path().join("exp_r01.json"),
        serde_json::to_vec(&split).unwrap(),
    )
    .unwrap();
    let out = dir.path().join("pooled_campaign");
    let error =
        hg_train::ere::run_campaign(&base, &out, dir.path(), &Budget::default()).unwrap_err();
    assert!(error.contains("E2-alternatives-pooled-v1"));
    let status: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("status.json")).unwrap()).unwrap();
    for field in [
        "optimizer_evaluation_steps",
        "precheck_evaluation_steps",
        "held_out_evaluation_steps",
    ] {
        assert_eq!(status[field], 0);
    }
    assert!(!out.join("campaign.csv").exists());
    assert!(!out.join("prechecks.json").exists());
}

#[test]
fn e2_grammar_catalogue_is_valid_reproducible_and_all_parameters_are_finite() {
    use hg_train::ere::e2;
    let theta = vec![0.0; e2::PARAMS];
    assert_eq!(
        serde_json::to_value(e2::emit(&theta, 42).unwrap()).unwrap(),
        serde_json::to_value(e2::emit(&theta, 42).unwrap()).unwrap()
    );
    assert!(e2::emit(&[0.0; 39], 42).is_err());
    let mut bad = theta;
    bad[0] = f32::NAN;
    assert!(e2::emit(&bad, 42).is_err());
    for core in 0..4 {
        for proposal in 0..4 {
            for active in [false, true] {
                let emitted = e2::emit(&e2::forced(core, proposal, active), 42).unwrap();
                assert_eq!(emitted.families.len(), 4);
                assert!(emitted.families.iter().all(|f| f.active == active));
                let rules = hg_rules::parse_rules_toml(&emitted.core_toml).unwrap();
                assert_eq!(rules.len(), 4);
                assert!(hg_rules::compile_rules(&rules).is_ok());
                assert!(hg_sim_core::ere::seed_family_signature(&emitted.families, true).is_ok());
            }
        }
    }
}

#[test]
fn e2_seed_signature_distinguishes_genotype_from_executable_activation() {
    use hg_train::ere::e2;
    let dormant = e2::emit(&e2::forced(0, 1, false), 42).unwrap();
    let active = e2::emit(&e2::forced(0, 1, true), 42).unwrap();
    let altered = e2::emit(&e2::forced(0, 2, false), 42).unwrap();
    let sig = hg_sim_core::ere::seed_family_signature;
    assert_ne!(
        sig(&dormant.families, true).unwrap(),
        sig(&altered.families, true).unwrap()
    );
    assert_eq!(
        sig(&dormant.families, false).unwrap(),
        sig(&altered.families, false).unwrap()
    );
    assert_ne!(
        sig(&dormant.families, false).unwrap(),
        sig(&active.families, false).unwrap()
    );
}

#[test]
fn e2_pilot_records_budget_then_blocks_on_missing_gate_profile_before_evaluation() {
    let base =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("pilot");
    assert!(
        hg_train::ere::e2::run_pilot(&base, &out, &dir.path().join("missing"), dir.path()).is_err()
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["maximum_simulation_steps"], 58000);
    assert_eq!(manifest["max_evaluations"], 58);
    assert_eq!(manifest["full_campaign"], "not_launched");
    assert!(!out.join("evaluations.json").exists());
    assert!(!out.join("campaign.csv").exists());
}

#[test]
fn e2_production_and_rollout_share_explicit_families_and_role_metrics() {
    use hg_sim_core::ere::EreArm;
    use hg_train::ere::e2;
    let base =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    let emitted = e2::emit(&e2::forced(0, 1, true), 42).unwrap();
    let mut cfg = e2::configure(&base, &emitted, 42, 10, EreArm::A1).unwrap();
    let temp = tempfile::tempdir().unwrap();
    cfg.out_dir = temp.path().into();
    let a = hg_sim_core::run_headless(&cfg, None, None, None).unwrap();
    let b = hg_sim_core::runner::run_ere_rollout(&cfg).unwrap();
    assert_eq!(a.history, b.history);
    assert_eq!(a.canonical_hash, b.canonical_hash);
    assert_eq!(b.final_step, 10);
    let row = b.history.last().unwrap().ere.as_ref().unwrap();
    assert_eq!(row.rule_rcf_novel_cumulative, Some(0));
    assert!(
        row.structural_proposal_reuse_count_by_class
            .as_ref()
            .unwrap()
            .values()
            .sum::<usize>()
            > 0
    );
}

#[test]
fn e2_observation_reads_legacy_meta_label_but_writes_accurate_label() {
    use hg_train::ere::e2::Observation;
    let legacy = serde_json::json!({
        "final_step":1000, "final_hash":"0000000000000001",
        "structural_proposal_score":0.0, "proposal_world_fires":0,
        "structural_proposal_world_fires":0, "activation_count":0,
        "anchor_survival_fraction":1.0, "cleanup_edges_lost":0,
        "quarantine_decisions":0, "meta_editor_events":8000,
        "novel_classes":0, "ceiling_bound":false, "valid":true
    });
    let observation: Observation = serde_json::from_value(legacy).unwrap();
    assert_eq!(observation.meta_rewrite_events, 8000);
    let accurate = serde_json::to_value(observation).unwrap();
    assert_eq!(accurate["meta_rewrite_events"], 8000);
    assert!(accurate.get("meta_editor_events").is_none());
}

#[test]
fn joint_objective_reconstructs_receipts_and_penalizes_body_loss_not_just_anchors() {
    use hg_schema::{ere::EreFamilyMaintenance, EreMetricsRow};
    use hg_train::ere::maintenance::summarize;
    let families = (0..4)
        .map(|family_index| EreFamilyMaintenance {
            family_index,
            anchor: format!("{family_index}"),
            checks: 1000,
            valid_steps: 1000,
            final_valid: true,
            ..Default::default()
        })
        .collect();
    let mut row = EreMetricsRow {
        seeded_family_maintenance: Some(families),
        structural_proposal_reuse_count_by_class: Some([(1, 8000)].into_iter().collect()),
        seeded_editor_rewrite_events: Some(128),
        ordinary_meta_rewrite_events: Some(50),
        meta_rewrite_events: Some(178),
        rules_quarantined: Some(0),
        meta_edges_lost_to_cleanup: Some(0),
        ..Default::default()
    };
    let a = summarize(&[row.clone()], 1000, "hash".into(), 1000, 16).unwrap();
    assert!(a.reuse_score > 0.0);
    assert_eq!(a.joint_score, Some(a.reuse_score));
    let f = &mut row.seeded_family_maintenance.as_mut().unwrap()[0];
    f.valid_steps = 500;
    f.final_valid = false;
    f.final_failure = Some("damaged".into());
    f.first_invalid_step = Some(501);
    let b = summarize(&[row.clone()], 1000, "hash".into(), 1000, 16).unwrap();
    assert_eq!(b.time_valid_fraction, 0.875);
    assert_eq!(b.final_valid_fraction, 0.75);
    assert_eq!(b.joint_score, Some(a.reuse_score * 0.875 * 0.75));
    for f in row.seeded_family_maintenance.as_mut().unwrap() {
        f.final_valid = false;
        f.final_failure = Some("damaged".into());
    }
    assert_eq!(
        summarize(&[row.clone()], 1000, "hash".into(), 1000, 16)
            .unwrap()
            .joint_score,
        Some(0.0)
    );
    row.seeded_family_maintenance.as_mut().unwrap().pop();
    assert!(summarize(&[row.clone()], 1000, "hash".into(), 1000, 16).is_err());
}

#[test]
fn maintenance_rollout_receipts_are_complete_and_editor_events_are_separate() {
    use hg_sim_core::ere::EreArm;
    use hg_train::ere::{e2, maintenance};
    let base =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    let emission = e2::emit(&e2::forced(0, 1, true), 42).unwrap();
    let mut cfg = e2::configure(&base, &emission, 42, 10, EreArm::A2).unwrap();
    cfg.ere.as_mut().unwrap().measure_family_maintenance = true;
    let run = hg_sim_core::runner::run_ere_rollout(&cfg).unwrap();
    let rows: Vec<_> = run
        .history
        .into_iter()
        .filter_map(|r| r.ere.map(|r| *r))
        .collect();
    let observation = maintenance::summarize(
        &rows,
        run.final_step,
        "hash".into(),
        10,
        cfg.engine.max_pairings_per_step,
    )
    .unwrap();
    assert_eq!(observation.time_valid_fraction, 1.0);
    assert_eq!(observation.final_valid_fraction, 1.0);
    assert!(observation.seeded_editor_events > 0);
    assert_eq!(observation.ordinary_meta_events, 0);
    assert_eq!(observation.joint_score, Some(observation.reuse_score));
    let mut capped = cfg;
    capped.ere.as_mut().unwrap().match_ceiling = 1;
    assert!(matches!(
        hg_sim_core::runner::run_ere_rollout(&capped),
        Err(hg_sim_core::runner::SimError::EreHalt {
            completed_steps: 0,
            ..
        })
    ));
}

#[test]
fn maintenance_pilot_admission_and_fresh_directory_guard_precede_evaluation() {
    let base =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("new_pilot");
    assert!(hg_train::ere::maintenance::run_pilot(&base, &out, temp.path(), temp.path()).is_err());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["maximum_simulation_steps"], 58000);
    assert_eq!(
        manifest["base_config"]["ere"]["measure_family_maintenance"],
        true
    );
    assert_eq!(manifest["base_config"]["ere"]["measure_ei"], false);
    assert!(!out.join("evaluations.json").exists());
    assert!(
        hg_train::ere::maintenance::run_pilot(&base, &out, temp.path(), temp.path())
            .unwrap_err()
            .contains("overwrite")
    );
}
