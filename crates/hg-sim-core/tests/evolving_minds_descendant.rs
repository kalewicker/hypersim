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
fn fixture(target: Option<u32>, source: Option<u8>) -> (FundedAdmissionConfig, DescendantConfig) {
    let mut c = descendant_cases()[0].config.clone();
    c.seed = 17;
    (
        c,
        DescendantConfig {
            policy: DESCENDANT_POLICY.into(),
            target_id: target,
            new_source: source,
        },
    )
}
#[test]
fn paid_one_shot_targets_and_death_pruned_continuation_are_exact() {
    for edit in [
        (None, None),
        (Some(104), Some(1)),
        (Some(103), Some(7)),
        (Some(103), Some(1)),
    ] {
        let (c, d) = fixture(edit.0, edit.1);
        let mut full = FamilySimulation::new_descendant(c.clone(), d.clone()).unwrap();
        let mut live = FamilySimulation::new_descendant_stream(c, d).unwrap();
        let mut records = vec![];
        for tick in 1..=64 {
            full.advance_tick().unwrap();
            live.advance_tick().unwrap();
            let b = live.drain_stream_tick().unwrap();
            assert_eq!(b.profile, DESCENDANT_STREAM_PROFILE);
            assert_eq!(b.row, *full.rows.last().unwrap());
            assert_eq!(
                serde_json::to_value(&b.entries).unwrap(),
                serde_json::to_value(&full.journal[full.journal.len() - 46..]).unwrap()
            );
            records.extend(b.records);
            if [8, 16, 24, 64].contains(&tick) {
                let cp = live.live_checkpoint(cursor(tick)).unwrap();
                let att = live.live_attestation(cursor(tick)).unwrap();
                assert_eq!(&cp[..8], b"HGEMDN01");
                let mut bad = att.clone();
                bad["native"]["descendant"]["targets"] = serde_json::json!({});
                if !live.descendant_state().unwrap().targets.is_empty() {
                    assert!(FamilySimulation::resume_live(&cp, &cursor(tick), &bad).is_err());
                }
                let mut cross = cp.clone();
                cross[..8].copy_from_slice(b"HGEMHO01");
                assert!(FamilySimulation::resume_live(&cross, &cursor(tick), &att).is_err());
                live = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
                assert!(full.checkpoint().is_err());
            }
        }
        records.sort_by_key(|r| r.request_id);
        assert_eq!(records, full.admission.as_ref().unwrap().records);
        assert_eq!(full.descendant_state(), live.descendant_state());
        assert_eq!(
            serde_json::to_value(full.final_frame()).unwrap(),
            serde_json::to_value(live.final_frame()).unwrap()
        );
        let state = live.descendant_state().unwrap();
        assert_eq!(state.targets.len(), 2);
        assert_eq!(
            records
                .iter()
                .filter(|r| r.operator == "supplied_descendant")
                .count(),
            2
        );
        for sample in state.targets.values() {
            assert!(sample.tick <= 24);
            assert!(!sample.settled);
            assert_eq!(sample.request_charge, 1.0);
            let r = records
                .iter()
                .find(|r| r.request_id == sample.request_id)
                .unwrap();
            assert!(r.settled);
            if edit.0 == Some(104) {
                assert!(!r.syntax_valid);
                assert_eq!(r.refunded, 29.0);
                assert_eq!(r.child_id, None);
            } else {
                assert_eq!(r.copying_charge, 5.0);
                assert_eq!(r.refunded, 0.0);
                assert!(r.child_id.is_some());
            }
            if edit == (Some(103), Some(7)) {
                assert!(!r.material_reachable);
            }
        }
        assert!(records
            .iter()
            .filter(|r| r.parent > 2 && r.operator != "supplied_descendant")
            .all(|r| r.parent_program == r.proposed_program));
    }
}
#[test]
fn target_order_is_semantic_under_workers_and_reversed_allocation() {
    let run = |workers, reverse| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| {
                let (mut c, d) = fixture(Some(103), Some(1));
                c.reversed = reverse;
                let mut s = FamilySimulation::new_descendant(c, d).unwrap();
                for _ in 0..24 {
                    s.advance_tick().unwrap();
                }
                (
                    s.descendant_state().unwrap().clone(),
                    s.admission.as_ref().unwrap().records.clone(),
                    s.lineage,
                )
            })
    };
    assert_eq!(run(1, false), run(4, false));
    assert_eq!(run(4, false), run(4, true));
}
#[test]
fn fixed_denominator_and_policy_refuse_unregistered_edits() {
    let rows = descendant_cases();
    assert_eq!(rows.len(), 73);
    for seed in [601, 607] {
        assert_eq!(rows.iter().filter(|x| x.config.seed == seed).count(), 36);
    }
    let (c, mut d) = fixture(None, None);
    for edit in [
        (Some(100), Some(0)),
        (Some(99), Some(1)),
        (None, Some(1)),
        (Some(100), None),
        (Some(100), Some(8)),
    ] {
        d.target_id = edit.0;
        d.new_source = edit.1;
        assert!(FamilySimulation::new_descendant(c.clone(), d.clone()).is_err());
    }
}
#[test]
fn committed_recovery_fixture_preserves_targets_and_strict_latest_commit() {
    let p = std::env::var("EVOLVING_MINDS_DESCENDANT_FIXTURE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::temp_dir().join(format!("em-descendant-test-{}", std::process::id()))
        });
    std::fs::create_dir(&p).unwrap();
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, source) in [
        (
            "stream_reference.py",
            "scripts/verify_evolving_minds_streaming.py",
        ),
        (
            "admission_reference.py",
            "scripts/verify_evolving_minds_admission.py",
        ),
        ("replay.py", "scripts/verify_evolving_minds.py"),
        ("d0_reference.py", "scripts/verify_evolving_minds_d0.py"),
        (
            "schedule_reference.py",
            "scripts/verify_evolving_minds_choice.py",
        ),
        (
            "opportunity_reference.py",
            "scripts/evolving_minds_opportunity.py",
        ),
        (
            "shared_replay.py",
            "scripts/evolving_minds_shared_replay.py",
        ),
    ] {
        std::fs::copy(repo.join(source), p.join(name)).unwrap();
    }
    let (cfg, d) = fixture(Some(103), Some(1));
    let a = write_descendant_stream_segment(
        &p.join("world17_recovery"),
        cfg.clone(),
        d.clone(),
        StreamFault::CleanStop,
    )
    .unwrap();
    assert_eq!(a.state, "Interrupted");
    assert_eq!(a.committed_final_tick, 24);
    let b = resume_stream_segment(
        &p.join("world17_recovery"),
        &p.join("world17_recovery_resumed"),
    )
    .unwrap();
    assert_eq!(b.state, "Completed");
    assert_eq!(b.committed_final_tick, 64);
    let rules = std::fs::read_to_string(p.join("world17_recovery/rules.toml")).unwrap();
    let base = rules.split("[[rules]]\nname = \"world_").next().unwrap();
    std::fs::write(p.join("b24_base_rules.toml"), base).unwrap();
    let bad = p.join("world17_recovery/commit_0024.json");
    let original = std::fs::read(&bad).unwrap();
    std::fs::write(&bad, b"corrupt latest marker").unwrap();
    assert!(
        resume_stream_segment(&p.join("world17_recovery"), &p.join("forbidden_fallback")).is_err()
    );
    // Restore the exact test input bytes solely to publish a usable external replay fixture.
    std::fs::write(&bad, original).unwrap();
    let commit = std::fs::read(p.join("world17_recovery_resumed/segment.json")).unwrap();
    let header: serde_json::Value = serde_json::from_slice(&commit).unwrap();
    assert!(header["parent"]["sha256"].as_str().is_some());
}
