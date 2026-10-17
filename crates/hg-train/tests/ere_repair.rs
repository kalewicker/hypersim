#![cfg(feature = "ere")]
use hg_sim_core::ere::{EreArm, MetaOperation, VOCABULARY_PROTOCOL};
use hg_train::ere::repair;
fn base() -> hg_sim_core::SimConfig {
    hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml")).unwrap()
}
#[test]
fn fixed_conditions_fuel_and_fresh_directory_are_explicit() {
    for i in 0..6 {
        let (cfg, emission) = repair::configure(&base(), 42, i).unwrap();
        assert_eq!(emission.families.len(), 4);
        assert_eq!(cfg.engine.max_steps, 100);
        let ere = cfg.ere.unwrap();
        let v = ere.vocabulary.unwrap();
        assert_eq!(v.protocol, repair::PROTOCOL);
        assert_eq!(v.instructions.len(), 5);
        assert_eq!(v.instructions[0].operation, MetaOperation::WitnessActivate);
        assert_eq!(v.instructions[0].fuel, 1.0);
        assert!(v.instructions.iter().map(|s| s.fuel).sum::<f64>() <= 97.0);
        assert!(ere.meta_rules_toml.is_none());
        assert_eq!(ere.meta_seed_energy, 0.0);
        assert_eq!(ere.arm, if i < 4 { EreArm::A3 } else { EreArm::A2 });
    }
    let tmp = tempfile::tempdir().unwrap();
    assert!(repair::run_assay(&base(), tmp.path())
        .unwrap_err()
        .contains("overwrite"));
    assert!(!tmp.path().join("manifest.json").exists());
}
#[test]
fn short_shared_loop_is_paid_read_only_and_tampering_is_rejected() {
    let (mut cfg, emission) = repair::configure(&base(), 42, 5).unwrap();
    cfg.engine.max_steps = 12;
    let (normal, rows) = hg_sim_core::runner::run_ere_diagnostic(&cfg);
    let r = repair::evaluate(cfg.clone(), emission, 0, "unit_observer_parity");
    assert_eq!(r.error, None);
    assert!(r.accounting_ok);
    let normal = normal.unwrap();
    assert_eq!(
        normal.canonical_hash,
        u64::from_str_radix(&r.observation.as_ref().unwrap().final_hash, 16).unwrap()
    );
    assert_eq!(r.ere_windows, rows);
    assert!(r
        .step_observations
        .iter()
        .all(|s| s.families.iter().all(|f| f.valid)));
    let mut bad = r.clone();
    bad.ere_windows[0]
        .encoded_meta_operation_traces
        .as_mut()
        .unwrap()[0]
        .fuel_after = Some(1.0);
    assert!(!repair::verify_accounting(&bad));
    let mut bad = r.clone();
    bad.step_observations.pop();
    assert!(!repair::verify_accounting(&bad));
    assert!(repair::contrasts(&[r]).is_err());
    cfg.ere
        .as_mut()
        .unwrap()
        .vocabulary
        .as_mut()
        .unwrap()
        .protocol = VOCABULARY_PROTOCOL.into();
    let (invalid, rows) = hg_sim_core::runner::run_ere_diagnostic(&cfg);
    assert!(invalid.is_err());
    assert!(rows.is_empty());
}
#[test]
fn cap_and_missing_observations_cannot_be_promoted_to_recovery() {
    let (mut cfg, emission) = repair::configure(&base(), 42, 3).unwrap();
    cfg.ere.as_mut().unwrap().match_ceiling = 1;
    let r = repair::evaluate(cfg, emission, 0, "unit_cap");
    assert!(r.error.is_some());
    assert_eq!(r.completed_steps, Some(0));
    assert!(r.step_observations.is_empty());
}
