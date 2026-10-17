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
fn fixture(edit: SuppliedEdit, kernel: MutationKernel) -> (FundedAdmissionConfig, HandoffConfig) {
    let mut c = handoff_cases()[0].config.clone();
    c.seed = 17;
    c.kernels = [kernel; 2];
    (
        c,
        HandoffConfig {
            policy: HANDOFF_POLICY.into(),
            supplied: edit,
            descendant_kernel: kernel,
        },
    )
}
#[test]
fn supplied_paid_boundaries_and_live_resume_are_exact() {
    for (edit, kernel) in [
        (SuppliedEdit::Useful, MutationKernel::Faithful),
        (SuppliedEdit::Useful, MutationKernel::SourceKernel),
        (SuppliedEdit::Unchanged, MutationKernel::Faithful),
        (SuppliedEdit::Invalid, MutationKernel::Faithful),
        (SuppliedEdit::Unreachable, MutationKernel::Faithful),
    ] {
        let (c, h) = fixture(edit, kernel);
        let mut full = FamilySimulation::new_handoff(c.clone(), h.clone()).unwrap();
        let mut live = FamilySimulation::new_handoff_stream(c, h).unwrap();
        let mut records = vec![];
        for tick in 1..=64 {
            full.advance_tick().unwrap();
            live.advance_tick().unwrap();
            let b = live.drain_stream_tick().unwrap();
            assert_eq!(b.profile, HANDOFF_STREAM_PROFILE);
            assert_eq!(b.row, *full.rows.last().unwrap());
            assert_eq!(
                serde_json::to_value(&b.entries).unwrap(),
                serde_json::to_value(&full.journal[full.journal.len() - 46..]).unwrap()
            );
            records.extend(b.records);
            if [4, 8, 16, 24].contains(&tick) {
                let cp = live.live_checkpoint(cursor(tick)).unwrap();
                let att = live.live_attestation(cursor(tick)).unwrap();
                assert_eq!(&cp[..8], b"HGEMHO01");
                live = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
                let mut bad = att.clone();
                bad["native"]["handoff_config"]["supplied"] = "Unchanged".into();
                if edit == SuppliedEdit::Unchanged {
                    bad["native"]["handoff_config"]["policy"] = "unknown".into();
                }
                assert!(FamilySimulation::resume_live(&cp, &cursor(tick), &bad).is_err());
                let mut cross = cp.clone();
                cross[..8].copy_from_slice(b"HGEMFS01");
                assert!(FamilySimulation::resume_live(&cross, &cursor(tick), &att).is_err());
                assert!(full.checkpoint().is_err());
                if tick == 4 {
                    assert!(records.is_empty());
                }
            }
        }
        records.sort_by_key(|r| r.request_id);
        assert_eq!(records, full.admission.as_ref().unwrap().records);
        assert_eq!(
            serde_json::to_value(full.final_frame()).unwrap(),
            serde_json::to_value(live.final_frame()).unwrap()
        );
        for r in &records {
            assert_eq!(r.request_charge, 1.0);
            assert!(r.settled);
            if r.parent <= 2 {
                assert_eq!(r.operator, "supplied_founder");
                assert!(r.raw_draws.is_empty());
                assert_eq!(r.draw_end, 0);
                assert_eq!(r.parent_program, FundedProgram::Delayed.program().unwrap());
                match edit {
                    SuppliedEdit::Useful => {
                        assert_eq!(r.target_id, Some(103));
                        assert_eq!(r.old_source, Some(2));
                        assert_eq!(r.new_source, Some(0));
                        assert_eq!(r.copying_charge, 5.0);
                    }
                    SuppliedEdit::Invalid => {
                        assert!(!r.syntax_valid);
                        assert_eq!(r.refunded, 29.0);
                        assert_eq!(r.copying_charge, 0.0);
                        assert!(r.child_id.is_none());
                    }
                    SuppliedEdit::Unreachable => {
                        assert!(r.syntax_valid);
                        assert!(!r.material_reachable);
                        assert_eq!(r.copying_charge, 5.0);
                        assert_eq!(r.refunded, 0.0);
                    }
                    SuppliedEdit::Unchanged => assert_eq!(r.parent_program, r.proposed_program),
                }
            } else {
                assert_eq!(r.operator, kernel.name());
            }
        }
        if edit == SuppliedEdit::Unreachable {
            assert!(full.rows.last().unwrap().development_failures >= 2);
        }
    }
}
#[test]
fn handoff_workers_allocation_and_config_binding() {
    let run = |workers, reversed| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| {
                let (mut c, h) = fixture(SuppliedEdit::Useful, MutationKernel::SourceKernel);
                c.reversed = reversed;
                let mut s = FamilySimulation::new_handoff(c, h).unwrap();
                s.run().unwrap();
                (
                    serde_json::to_value(&s.journal).unwrap(),
                    s.lineage.clone(),
                    s.admission.as_ref().unwrap().records.clone(),
                )
            })
    };
    let one = run(1, false);
    assert_eq!(one, run(4, false));
    let rev = run(1, true);
    assert_eq!(one.1, rev.1);
    assert_eq!(one.2, rev.2);
    let (c, mut h) = fixture(SuppliedEdit::Useful, MutationKernel::Faithful);
    h.policy = "other".into();
    assert!(FamilySimulation::new_handoff(c.clone(), h.clone()).is_err());
    h.policy = HANDOFF_POLICY.into();
    h.descendant_kernel = MutationKernel::Invalid;
    assert!(FamilySimulation::new_handoff(c.clone(), h.clone()).is_err());
    h.descendant_kernel = MutationKernel::SourceKernel;
    assert!(FamilySimulation::new_handoff(c, h).is_err());
}
#[test]
fn handoff_strict_publication_recovers() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("source");
    let (c, h) = fixture(SuppliedEdit::Useful, MutationKernel::Faithful);
    let r = write_handoff_stream_segment(&p, c, h, StreamFault::BeforeCommit).unwrap();
    assert_eq!(r.committed_final_tick, 8);
    let r = resume_stream_segment(&p, &tmp.path().join("child")).unwrap();
    assert_eq!(r.committed_final_tick, 64);
    assert_eq!(r.profile, HANDOFF_STREAM_PROFILE);
    let cp = p.join("checkpoint_0008.bin");
    let mut b = std::fs::read(&cp).unwrap();
    let last = b.len() - 1;
    b[last] ^= 1;
    std::fs::write(cp, b).unwrap();
    assert!(resume_stream_segment(&p, &tmp.path().join("refused")).is_err());
    assert!(!tmp.path().join("refused").exists());
}
