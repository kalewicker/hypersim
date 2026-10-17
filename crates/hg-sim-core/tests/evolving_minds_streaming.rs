#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
fn recipe(case: &str) -> AdmissionRecipe {
    AdmissionRecipe {
        case: case.into(),
        rotation: 0,
        reversed: false,
    }
}
fn cursor(tick: u64) -> EvidenceCursor {
    EvidenceCursor {
        segment_sha256: "a".repeat(64),
        tick,
        step: tick * 46,
        local_ticks: tick,
        offset: 0,
        log_sha256: "b".repeat(64),
        previous_commit_sha256: None,
    }
}
#[test]
fn bounded_live_history_matches_full_fixture_and_fast_resume() {
    for case in ["faithful_disjoint", "mixed_competing", "mixed_settlement"] {
        let mut full = FamilySimulation::new_admission(recipe(case)).unwrap();
        let mut live = FamilySimulation::new_stream(recipe(case)).unwrap();
        let mut deltas = full.lineage.clone();
        let mut all_records = vec![];
        let mut all_rounds = vec![];
        let mut all_entries = vec![];
        let mut rows = vec![];
        for tick in 1..=64 {
            full.advance_tick().unwrap();
            live.advance_tick().unwrap();
            assert!(live.live_checkpoint(cursor(tick)).is_err());
            assert!(live.advance_tick().is_err());
            let b = live.drain_stream_tick().unwrap();
            for delta in b.lineage_delta {
                deltas.insert(delta.id, delta);
            }
            all_records.extend(b.records);
            all_records.sort_by_key(|r| r.request_id);
            all_rounds.push(b.round);
            all_entries.extend(b.entries);
            rows.push(b.row);
            assert!(live.rows.is_empty() && live.journal.is_empty());
            assert!(live.admission.as_ref().unwrap().records.is_empty());
            assert!(live.lineage.len() <= 8);
            assert!(live.lineage.values().all(|b| b.death_tick.is_none()));
            if [8, 16].contains(&tick) {
                let cp = live.live_checkpoint(cursor(tick)).unwrap();
                assert!(cp.len() < 2 * 1024 * 1024);
                let att = live.live_attestation(cursor(tick)).unwrap();
                let resumed = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
                assert_eq!(
                    serde_json::to_value(resumed.final_frame()).unwrap(),
                    serde_json::to_value(live.final_frame()).unwrap()
                );
                live = resumed;
                let mut bad = cp.clone();
                let n = bad.len() - 1;
                bad[n] ^= 1;
                assert!(FamilySimulation::resume_live(&bad, &cursor(tick), &att).is_err());
                let mut wrong = cursor(tick);
                wrong.tick += 1;
                assert!(FamilySimulation::resume_live(&cp, &wrong, &att).is_err());
                let mut forged = att;
                forged["native"]["live"]["maximum_depth"] = 999.into();
                assert!(FamilySimulation::resume_live(&cp, &cursor(tick), &forged).is_err());
                assert!(FamilySimulation::resume(&cp).is_err());
                assert!(live.checkpoint().is_err());
            }
        }
        assert_eq!(full.rows, rows);
        assert_eq!(
            serde_json::to_value(&full.journal).unwrap(),
            serde_json::to_value(all_entries).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&full.lineage).unwrap(),
            serde_json::to_value(deltas).unwrap()
        );
        assert_eq!(full.admission.as_ref().unwrap().records, all_records);
        assert_eq!(full.admission.as_ref().unwrap().rounds, all_rounds);
        assert_eq!(
            serde_json::to_value(full.final_frame()).unwrap(),
            serde_json::to_value(live.final_frame()).unwrap()
        );
    }
}
#[test]
fn worker_threads_keep_stream_bundles_exact() {
    let run = |n| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .unwrap()
            .install(|| {
                let mut s = FamilySimulation::new_stream(recipe("mixed_settlement")).unwrap();
                let mut bundles = vec![];
                for _ in 0..64 {
                    s.advance_tick().unwrap();
                    bundles.push(s.drain_stream_tick().unwrap());
                }
                serde_json::to_value(bundles).unwrap()
            })
    };
    assert_eq!(run(1), run(4));
}

#[test]
fn unpublished_state_is_ignored_and_corrupt_published_state_has_no_fallback() {
    use std::fs;
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source");
    let report = write_stream_segment(
        &source,
        recipe("mixed_settlement"),
        StreamFault::BeforeCommit,
    )
    .unwrap();
    assert_eq!(report.committed_final_tick, 8);
    assert!(source.join("checkpoint_0016.bin").exists());
    let before: std::collections::BTreeMap<_, _> = fs::read_dir(&source)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            (p.clone(), fs::read(p).unwrap())
        })
        .collect();
    let resumed = resume_stream_segment(&source, &tmp.path().join("child")).unwrap();
    assert_eq!(resumed.start_tick, 8);
    assert_eq!(resumed.committed_final_tick, 64);
    for (path, bytes) in &before {
        assert_eq!(fs::read(path).unwrap(), *bytes);
    }
    let mut damaged = fs::read(source.join("checkpoint_0008.bin")).unwrap();
    let end = damaged.len() - 1;
    damaged[end] ^= 1;
    fs::write(source.join("checkpoint_0008.bin"), damaged).unwrap();
    let refused = tmp.path().join("refused");
    assert!(resume_stream_segment(&source, &refused).is_err());
    assert!(!refused.exists());
    fs::write(
        source.join("checkpoint_0008.bin"),
        &before[&source.join("checkpoint_0008.bin")],
    )
    .unwrap();
    fs::copy(
        source.join("commit_0016.json.pending"),
        source.join("commit_0016.json"),
    )
    .unwrap();
    assert!(resume_stream_segment(&source, &refused).is_err());
    assert!(!refused.exists());
}
