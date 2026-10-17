#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
fn cursor(tick: u64) -> EvidenceCursor {
    EvidenceCursor {
        segment_sha256: "a".repeat(64),
        tick,
        step: 46 * tick,
        local_ticks: tick,
        offset: 0,
        log_sha256: "b".repeat(64),
        previous_commit_sha256: None,
    }
}
#[test]
fn configured_ecology_live_resume_is_exact_and_parameter_bound() {
    for case in funded_admission_cases()
        .into_iter()
        .filter(|c| !c.name.starts_with("immediate_r3"))
    {
        let config = case.config;
        let mut full = FamilySimulation::new_funded_admission(config.clone()).unwrap();
        let mut live = FamilySimulation::new_funded_stream(config.clone()).unwrap();
        let mut entries = vec![];
        let mut rows = vec![];
        let mut rounds = vec![];
        let mut records = vec![];
        let mut lineage = full.lineage.clone();
        assert!(full.checkpoint().is_err());
        for tick in 1..=64 {
            full.advance_tick().unwrap();
            live.advance_tick().unwrap();
            assert!(live.live_checkpoint(cursor(tick)).is_err());
            let bundle = live.drain_stream_tick().unwrap();
            assert_eq!(bundle.profile, FUNDED_STREAM_PROFILE);
            entries.extend(bundle.entries);
            rows.push(bundle.row);
            rounds.push(bundle.round);
            records.extend(bundle.records);
            for b in bundle.lineage_delta {
                lineage.insert(b.id, b);
            }
            if [8, 16].contains(&tick) {
                let cp = live.live_checkpoint(cursor(tick)).unwrap();
                assert_eq!(&cp[..8], b"HGEMFS01");
                let att = live.live_attestation(cursor(tick)).unwrap();
                let resumed = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
                assert_eq!(resumed.funded_config(), Some(&config));
                assert_eq!(
                    serde_json::to_value(resumed.final_frame()).unwrap(),
                    serde_json::to_value(live.final_frame()).unwrap()
                );
                live = resumed;
                let mut forged = att.clone();
                forged["native"]["funded_config"]["endowment"] = 99.into();
                assert!(FamilySimulation::resume_live(&cp, &cursor(tick), &forged).is_err());
                let mut cross = cp.clone();
                cross[..8].copy_from_slice(b"HGEMS001");
                assert!(FamilySimulation::resume_live(&cross, &cursor(tick), &att).is_err());
                assert!(FamilySimulation::resume(&cp).is_err());
            }
        }
        records.sort_by_key(|r| r.request_id);
        assert_eq!(full.rows, rows);
        assert!(rows.iter().all(|r| r.residual == 0.0));
        assert_eq!(
            serde_json::to_value(&full.journal).unwrap(),
            serde_json::to_value(entries).unwrap()
        );
        assert_eq!(full.lineage, lineage);
        assert_eq!(full.admission.as_ref().unwrap().records, records);
        assert_eq!(full.admission.as_ref().unwrap().rounds, rounds);
        assert_eq!(
            serde_json::to_value(full.final_frame()).unwrap(),
            serde_json::to_value(live.final_frame()).unwrap()
        );
        if case.name == "mixed_reject_b24" {
            let invalid: Vec<_> = records.iter().filter(|r| r.parent == 1).collect();
            assert!(invalid.len() > 1);
            assert!(invalid
                .iter()
                .all(|r| !r.syntax_valid && r.refunded == 29.0 && r.request_charge == 1.0));
            assert!(rounds.iter().any(|r| r.selected.len() == 2));
        }
    }
}
#[test]
fn workers_and_allocation_keep_funded_semantics_and_bad_configs_fail() {
    let run = |workers, reversed| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| {
                let mut cfg = funded_admission_cases()[7].config.clone();
                cfg.reversed = reversed;
                let mut s = FamilySimulation::new_funded_admission(cfg).unwrap();
                s.run().unwrap();
                (
                    serde_json::to_value(&s.journal).unwrap(),
                    serde_json::to_value(&s.lineage).unwrap(),
                    s.admission.as_ref().unwrap().records.clone(),
                    s.admission.as_ref().unwrap().rounds.clone(),
                )
            })
    };
    let one = run(1, false);
    assert_eq!(one, run(4, false));
    let reverse = run(1, true);
    assert_eq!(one.1, reverse.1);
    assert_eq!(one.2, reverse.2);
    assert_eq!(one.3, reverse.3);
    let mut cfg = funded_admission_cases()[0].config.clone();
    cfg.horizon = 65;
    assert!(FamilySimulation::new_funded_admission(cfg.clone()).is_err());
    cfg.horizon = 64;
    cfg.home_offsets = [0, 0];
    assert!(cfg.validate().is_err());
}
#[test]
fn funded_pending_state_ignored_and_corrupt_published_state_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source");
    let cfg = funded_admission_cases()[4].config.clone();
    let stopped =
        write_funded_stream_segment(&source, cfg.clone(), StreamFault::BeforeCommit).unwrap();
    assert_eq!(stopped.committed_final_tick, 8);
    let cp = std::fs::read(source.join("checkpoint_0008.bin")).unwrap();
    let report = resume_stream_segment(&source, &tmp.path().join("child")).unwrap();
    assert_eq!(report.funded_config, Some(cfg));
    assert_eq!(report.committed_final_tick, 64);
    let mut damaged = cp;
    let last = damaged.len() - 1;
    damaged[last] ^= 1;
    std::fs::write(source.join("checkpoint_0008.bin"), damaged).unwrap();
    let refused = tmp.path().join("refused");
    assert!(resume_stream_segment(&source, &refused).is_err());
    assert!(!refused.exists());
}
#[test]
fn configured_founders_and_competing_homes_remain_bound_on_resume() {
    let mut cfg = funded_admission_cases()[1].config.clone();
    cfg.founders = [FundedProgram::Delayed, FundedProgram::Immediate];
    cfg.home_offsets = [0, 2];
    cfg.rotation = 5;
    let mut bad = serde_json::to_value(&cfg).unwrap();
    bad["undeclared_parameter"] = true.into();
    assert!(serde_json::from_value::<FundedAdmissionConfig>(bad).is_err());
    let mut full = FamilySimulation::new_funded_admission(cfg.clone()).unwrap();
    let mut live = FamilySimulation::new_funded_stream(cfg.clone()).unwrap();
    assert_eq!(full.lineage[&1].site, 5);
    assert_eq!(full.lineage[&2].site, 7);
    assert_ne!(full.lineage[&1].program, full.lineage[&2].program);
    let mut parallel = false;
    for tick in 1..=16 {
        full.advance_tick().unwrap();
        live.advance_tick().unwrap();
        let b = live.drain_stream_tick().unwrap();
        parallel |= b.round.selected.len() > 1;
        assert_eq!(b.row, *full.rows.last().unwrap());
        assert_eq!(
            serde_json::to_value(&b.entries).unwrap(),
            serde_json::to_value(&full.journal[full.journal.len() - 46..]).unwrap()
        );
        if tick == 8 {
            let cp = live.live_checkpoint(cursor(tick)).unwrap();
            let att = live.live_attestation(cursor(tick)).unwrap();
            live = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
            assert_eq!(live.funded_config(), Some(&cfg));
        }
    }
    assert!(parallel);
    assert_eq!(
        serde_json::to_value(full.final_frame()).unwrap(),
        serde_json::to_value(live.final_frame()).unwrap()
    );
}
