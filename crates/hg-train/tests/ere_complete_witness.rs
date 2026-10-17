#![cfg(feature = "ere")]
use hg_sim_core::ere::{EreArm, MetaInstructionSeed, MetaOperation, COMPLETE_PROTOCOL};
use hg_train::ere::{e2, maintenance, repair, Budget};

fn configured() -> hg_sim_core::SimConfig {
    let base =
        hg_sim_core::SimConfig::from_toml(include_str!("../../../configs/ere_e2_pooled.toml"))
            .unwrap();
    let (mut cfg, _) = repair::configure(&base, 42, 5).unwrap();
    let v = cfg.ere.as_mut().unwrap().vocabulary.as_mut().unwrap();
    v.protocol = COMPLETE_PROTOCOL.into();
    v.instructions[1] = MetaInstructionSeed {
        operation: MetaOperation::RepairSealed,
        fuel: 1.0,
    };
    cfg.engine.max_steps = 12;
    cfg
}

#[test]
fn complete_shared_loop_parity_and_versioned_instruction_metrics_do_not_reuse_v2_ledger() {
    let cfg = configured();
    let (normal, normal_rows) = hg_sim_core::runner::run_ere_diagnostic(&cfg);
    let (instrumented, rows, steps) = hg_sim_core::runner::run_ere_instrumented_diagnostic(&cfg);
    let normal = normal.unwrap();
    let instrumented = instrumented.unwrap();
    assert_eq!(normal.canonical_hash, instrumented.canonical_hash);
    assert_eq!(normal_rows, rows);
    assert_eq!(steps.len(), 12);
    assert!(steps
        .iter()
        .all(|s| s.rng_before_compile == s.rng_after_compile));
    assert!(rows
        .iter()
        .all(|r| r.encoded_meta_vocabulary_protocol.as_deref() == Some(COMPLETE_PROTOCOL)));
    assert!(rows
        .last()
        .unwrap()
        .encoded_meta_instruction_states
        .as_ref()
        .unwrap()
        .iter()
        .all(|s| s.valid));
    let emission = e2::emit(&vec![0.0; e2::PARAMS], 42).unwrap();
    let receipt = repair::evaluate(cfg, emission, 0, "complete_not_v2");
    assert!(!receipt.accounting_ok);
    assert!(receipt.error.is_some());
    assert!(!repair::verify_accounting(&receipt));
}

#[test]
fn complete_profile_still_refuses_training_and_legacy_maintenance_pilot() {
    let cfg = configured();
    let emission = e2::emit(&vec![0.0; e2::PARAMS], 42).unwrap();
    assert!(e2::configure(&cfg, &emission, 42, 1000, EreArm::A3).is_err());
    let tmp = tempfile::tempdir().unwrap();
    assert!(maintenance::run_pilot(
        &cfg,
        &tmp.path().join("maintenance"),
        tmp.path(),
        tmp.path()
    )
    .unwrap_err()
    .contains("vocabulary"));
    assert!(!tmp.path().join("maintenance").exists());
    let out = tmp.path().join("training");
    assert!(
        hg_train::ere::run_campaign(&cfg, &out, tmp.path(), &Budget::default())
            .unwrap_err()
            .contains("vocabulary")
    );
    let status: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("status.json")).unwrap()).unwrap();
    for key in [
        "optimizer_evaluation_steps",
        "precheck_evaluation_steps",
        "held_out_evaluation_steps",
    ] {
        assert_eq!(status[key], 0);
    }
    assert!(!out.join("campaign.csv").exists());
}
