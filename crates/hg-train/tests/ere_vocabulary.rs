#![cfg(feature = "ere")]
use hg_sim_core::ere::{EreArm, MetaVocabularyConfig, VOCABULARY_PROTOCOL};
use hg_train::ere::{e2, maintenance, vocabulary, Budget};
fn base() -> hg_sim_core::SimConfig {
    hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml")).unwrap()
}

#[test]
fn old_research_entrypoints_refuse_new_profile_before_evaluation() {
    let mut cfg = base();
    cfg.ere.as_mut().unwrap().vocabulary = Some(MetaVocabularyConfig {
        protocol: VOCABULARY_PROTOCOL.into(),
        instructions: vec![],
    });
    let emission = e2::emit(&vec![0.0; e2::PARAMS], 42).unwrap();
    assert!(e2::configure(&cfg, &emission, 42, 1000, EreArm::A3)
        .unwrap_err()
        .contains("explicit E2 pooled"));
    let temp = tempfile::tempdir().unwrap();
    let sweep_out = temp.path().join("sweep");
    assert!(
        hg_sim_core::ere::experiments::run_arm_experiments(&cfg, &sweep_out, false)
            .unwrap_err()
            .contains("vocabulary")
    );
    assert!(!sweep_out.exists());
    assert!(maintenance::run_pilot(
        &cfg,
        &temp.path().join("maintenance"),
        temp.path(),
        temp.path()
    )
    .unwrap_err()
    .contains("vocabulary"));
    let out = temp.path().join("training");
    assert!(
        hg_train::ere::run_campaign(&cfg, &out, temp.path(), &Budget::default())
            .unwrap_err()
            .contains("vocabulary")
    );
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
}

#[test]
fn diagnostic_uses_shared_loop_and_retains_partial_cap_measurements() {
    let (mut cfg, _) = vocabulary::configure(&base(), vocabulary::SEED, EreArm::A2).unwrap();
    cfg.engine.max_steps = 3;
    cfg.engine.metrics_window_interval = 2;
    let ordinary = hg_sim_core::runner::run_ere_rollout(&cfg).unwrap();
    let (diagnostic, rows) = hg_sim_core::runner::run_ere_diagnostic(&cfg);
    let diagnostic = diagnostic.unwrap();
    assert_eq!(ordinary.canonical_hash, diagnostic.canonical_hash);
    let expected: Vec<_> = ordinary
        .history
        .into_iter()
        .filter_map(|r| r.ere.map(|r| *r))
        .collect();
    assert_eq!(rows, expected);
    cfg.ere.as_mut().unwrap().match_ceiling = 1;
    let (result, partial) = hg_sim_core::runner::run_ere_diagnostic(&cfg);
    assert!(matches!(
        result,
        Err(hg_sim_core::runner::SimError::EreHalt {
            completed_steps: 0,
            ..
        })
    ));
    assert_eq!(partial.len(), 1);
    assert!(partial[0].match_ceiling_binding);
    assert_eq!(
        partial[0].seeded_family_maintenance.as_ref().unwrap()[0].checks,
        0
    );
    assert_eq!(partial[0].encoded_meta_rewrite_events, Some(0));
}

#[test]
fn assay_profile_and_budget_are_explicit_and_cannot_be_overwritten() {
    assert_eq!(vocabulary::ATTEMPTS, 12);
    assert_eq!(vocabulary::STEPS, 1000);
    for arm in [EreArm::A1, EreArm::A2, EreArm::A3] {
        let (cfg, emission) = vocabulary::configure(&base(), vocabulary::SEED, arm).unwrap();
        let e = cfg.ere.unwrap();
        assert_eq!(e.protocol().unwrap(), VOCABULARY_PROTOCOL);
        assert_eq!(emission.families.len(), 4);
        assert!(e.meta_rules_toml.is_none());
        assert_eq!(e.meta_seed_energy, 0.0);
        assert!(e.measure_family_maintenance && !e.measure_ei);
        assert_eq!(
            e.vocabulary
                .unwrap()
                .instructions
                .iter()
                .map(|s| s.fuel)
                .sum::<f64>(),
            128.0
        );
    }
    let temp = tempfile::tempdir().unwrap();
    assert!(vocabulary::run_assay(&base(), temp.path())
        .unwrap_err()
        .contains("overwrite"));
    assert!(!temp.path().join("manifest.json").exists());
}
