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
fn retained_native_live_and_checkpoint_continue_exactly() {
    for case in retained_cases().into_iter().take(5) {
        let mut full = FamilySimulation::new_retained(case.config.clone()).unwrap();
        let mut live = FamilySimulation::new_retained_stream(case.config.clone()).unwrap();
        let mut records = vec![];
        for tick in 1..=64 {
            full.advance_tick().unwrap();
            live.advance_tick().unwrap();
            let b = live.drain_stream_tick().unwrap();
            assert_eq!(b.profile, RETAINED_STREAM_PROFILE);
            assert_eq!(b.row, *full.rows.last().unwrap());
            assert_eq!(
                serde_json::to_value(&b.entries).unwrap(),
                serde_json::to_value(&full.journal[full.journal.len() - 46..]).unwrap()
            );
            records.extend(b.records);
            if [4, 8, 16, 24].contains(&tick) {
                let cp = live.live_checkpoint(cursor(tick)).unwrap();
                assert_eq!(&cp[..8], b"HGEMRP01");
                let att = live.live_attestation(cursor(tick)).unwrap();
                live = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
                assert_eq!(full.retained_state(), live.retained_state());
                if tick == 4 {
                    assert!(live.retained_state().unwrap().samples.is_empty());
                }
                let mut bad = att.clone();
                bad["native"]["retained"]["policy"] = "changed".into();
                assert!(FamilySimulation::resume_live(&cp, &cursor(tick), &bad).is_err());
                let mut cross = cp.clone();
                cross[..8].copy_from_slice(b"HGEMFS01");
                assert!(FamilySimulation::resume_live(&cross, &cursor(tick), &att).is_err());
                assert!(full.checkpoint().is_err());
            }
        }
        records.sort_by_key(|r| r.request_id);
        assert_eq!(records, full.admission.as_ref().unwrap().records);
        assert_eq!(full.retained_state(), live.retained_state());
        assert_eq!(
            serde_json::to_value(full.final_frame()).unwrap(),
            serde_json::to_value(live.final_frame()).unwrap()
        );
        assert_eq!(live.retained_state().unwrap().samples.len(), 2);
        for r in &records {
            if r.parent > 2 {
                assert_eq!(r.operator, "faithful");
                assert_eq!(r.parent_program, r.proposed_program);
            } else if r.request_id != live.retained_state().unwrap().samples[&r.parent].request_id {
                assert_eq!(r.operator, "retained_founder");
                assert!(r.raw_draws.is_empty());
                assert_eq!(r.draw_end, 0);
                assert_eq!(
                    r.proposed_bytes,
                    live.retained_state().unwrap().samples[&r.parent].proposed_bytes
                );
            }
        }
        if case.name == "invalid_immediate" {
            assert!(records.len() > 2);
            assert!(records
                .iter()
                .all(|r| !r.syntax_valid && r.refunded == 29.0 && r.copying_charge == 0.0));
            assert_eq!(full.rows.last().unwrap().births, 0);
        }
        if case.name == "unreachable_immediate" {
            assert!(full.rows.last().unwrap().development_failures >= 2);
        }
    }
}
#[test]
fn retained_workers_allocation_and_legacy_identity() {
    let run = |workers, reversed| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| {
                let mut cfg = retained_cases()[1].config.clone();
                cfg.reversed = reversed;
                let mut s = FamilySimulation::new_retained(cfg).unwrap();
                s.run().unwrap();
                (
                    serde_json::to_value(&s.journal).unwrap(),
                    s.lineage.clone(),
                    s.admission.clone(),
                    s.retained_state().cloned(),
                )
            })
    };
    let one = run(1, false);
    assert_eq!(one, run(4, false));
    let reversed = run(1, true);
    assert_eq!(one.1, reversed.1);
    assert_eq!(one.3, reversed.3);
    assert_eq!(
        one.2.as_ref().unwrap().records,
        reversed.2.as_ref().unwrap().records
    );
    assert_eq!(
        one.2.as_ref().unwrap().rounds,
        reversed.2.as_ref().unwrap().rounds
    );
}
#[test]
fn retained_publication_recovers_and_corrupt_latest_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("source");
    let cfg = retained_cases()[3].config.clone();
    let stopped = write_retained_stream_segment(&p, cfg, StreamFault::BeforeCommit).unwrap();
    assert_eq!(stopped.committed_final_tick, 8);
    assert_eq!(stopped.retained_policy.as_deref(), Some(RETAINED_POLICY));
    let report = resume_stream_segment(&p, &tmp.path().join("child")).unwrap();
    assert_eq!(report.profile, RETAINED_STREAM_PROFILE);
    assert_eq!(report.committed_final_tick, 64);
    let cp = p.join("checkpoint_0008.bin");
    let mut b = std::fs::read(&cp).unwrap();
    let last = b.len() - 1;
    b[last] ^= 1;
    std::fs::write(cp, b).unwrap();
    assert!(resume_stream_segment(&p, &tmp.path().join("refused")).is_err());
    assert!(!tmp.path().join("refused").exists());
}
