#![cfg(feature = "ere")]
use hg_sim_core::ere::EreArm;
use hg_train::ere::persistence;
fn base() -> hg_sim_core::SimConfig {
    hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml")).unwrap()
}

#[test]
fn fixed_fixture_budget_and_no_reverse_or_replenishment_are_explicit() {
    assert_eq!((persistence::ATTEMPTS, persistence::STEPS), (24, 100));
    for edit in [false, true] {
        for arm in [EreArm::A1, EreArm::A2, EreArm::A3] {
            let (cfg, emission) = persistence::configure(&base(), 42, arm, edit).unwrap();
            assert_eq!(emission.families.len(), 4);
            assert!(emission.families.iter().all(|f| !f.active));
            for family in emission.families {
                let c = hg_rules::parse_rules_toml(&family.core_toml).unwrap();
                let p = hg_rules::parse_rules_toml(&family.proposal_toml).unwrap();
                assert_eq!(c[0].lhs.edges, c[0].rhs.edges);
                assert_ne!(p[0].lhs.edges, p[0].rhs.edges);
                assert_eq!((c[0].priority, p[0].priority), (0, 1));
            }
            let e = cfg.ere.unwrap();
            let v = e.vocabulary.unwrap();
            assert_eq!(v.instructions.len(), 1);
            assert_eq!(v.instructions[0].fuel, if edit { 1.0 } else { 0.0 });
            assert!(e.meta_rules_toml.is_none());
            assert_eq!(e.meta_seed_energy, 0.0);
            assert_eq!(cfg.engine.metrics_window_interval, 1);
        }
    }
    let tmp = tempfile::tempdir().unwrap();
    assert!(persistence::run_assay(&base(), tmp.path())
        .unwrap_err()
        .contains("overwrite"));
    assert!(!tmp.path().join("manifest.json").exists());
}

#[test]
fn step_observation_is_read_only_and_records_table_used_by_the_shared_loop() {
    let (mut cfg, _) = persistence::configure(&base(), 42, EreArm::A2, true).unwrap();
    cfg.engine.max_steps = 12;
    let (normal, normal_rows) = hg_sim_core::runner::run_ere_diagnostic(&cfg);
    let (probed, probed_rows, steps) = hg_sim_core::runner::run_ere_instrumented_diagnostic(&cfg);
    let normal = normal.unwrap();
    let probed = probed.unwrap();
    assert_eq!(normal.canonical_hash, probed.canonical_hash);
    assert_eq!(normal.history, probed.history);
    assert_eq!(normal_rows, probed_rows);
    assert_eq!(steps.len(), 12);
    assert_eq!(steps[9].world_rule_table.len(), 4);
    assert_eq!(steps[10].world_rule_table.len(), 5);
    assert!(steps
        .iter()
        .all(|s| s.rng_before_compile == s.rng_after_compile));
    assert!(steps
        .iter()
        .all(|s| s.world_rule_table.iter().all(Option::is_some)));
    for pair in steps.windows(2) {
        assert_eq!(pair[0].world_hash_after, pair[1].world_hash_before);
        assert_eq!(pair[0].rng_after_step, pair[1].rng_before_compile);
    }
    let paid: usize = probed_rows
        .iter()
        .filter_map(|r| r.encoded_meta_rewrite_events)
        .sum();
    assert_eq!(paid, 1);
}

#[test]
fn cap_has_no_invented_completed_step_or_rng_consumption() {
    let (mut cfg, _) = persistence::configure(&base(), 42, EreArm::A2, true).unwrap();
    cfg.ere.as_mut().unwrap().match_ceiling = 1;
    let (result, rows, steps) = hg_sim_core::runner::run_ere_instrumented_diagnostic(&cfg);
    assert!(matches!(
        result,
        Err(hg_sim_core::runner::SimError::EreHalt {
            completed_steps: 0,
            ..
        })
    ));
    assert!(steps.is_empty());
    assert_eq!(rows.len(), 1);
    assert!(rows[0].match_ceiling_binding);
    let mut cfg = base();
    cfg.ere = None;
    let (result, rows, steps) = hg_sim_core::runner::run_ere_instrumented_diagnostic(&cfg);
    assert!(result.is_err());
    assert!(rows.is_empty() && steps.is_empty());
}

#[test]
fn incomplete_receipts_and_missing_controls_cannot_pass_contrasts() {
    let mut group = Vec::new();
    for arm in [EreArm::A1, EreArm::A2, EreArm::A3] {
        for edit in [false, true] {
            let (cfg, emission) = persistence::configure(&base(), 42, arm, edit).unwrap();
            group.push(persistence::Receipt {
                evaluation: group.len(),
                seed: 42,
                arm,
                edit,
                requested_steps: 100,
                effective_config: cfg,
                emission,
                completed_steps: Some(100),
                error: None,
                observation: None,
                ere_windows: vec![],
                step_observations: vec![],
                accounting_ok: true,
            });
        }
    }
    let contrast = persistence::contrast(&group).unwrap();
    assert!(!contrast.protected_persistence_pass);
    assert!(!contrast.protected_checks["complete_protected_and_frozen_controls"]);
    assert!(persistence::contrast(&group[..5]).is_err());
}
