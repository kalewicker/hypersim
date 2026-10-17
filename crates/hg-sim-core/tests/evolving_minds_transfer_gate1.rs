#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;

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
fn schedules_are_balanced_explicit_shifts_with_original_coins() {
    for seed in [17, 23] {
        let original = ChoiceSchedule::new(seed, CueMode::Informative);
        for pattern in [
            TemporalPattern::Shuffled,
            TemporalPattern::Alternating,
            TemporalPattern::Blocks4,
        ] {
            let e = TransferEnvironment {
                pattern,
                cue_mode: CueMode::Informative,
            };
            let s = e.schedule(seed);
            assert_eq!(s.resources.iter().filter(|&&x| x).count(), 32);
            assert_eq!(s.cues, s.resources);
            assert_eq!(s.coins, original.coins);
            match pattern {
                TemporalPattern::Shuffled => assert_eq!(s, original),
                TemporalPattern::Alternating => {
                    assert!(s.resources.windows(2).all(|w| w[0] != w[1]))
                }
                TemporalPattern::Blocks4 => {
                    assert!(s.resources.chunks(4).all(|w| w.iter().all(|x| *x == w[0])))
                }
            }
            let withheld = TransferEnvironment {
                pattern,
                cue_mode: CueMode::Withheld,
            }
            .schedule(seed);
            assert_eq!(withheld.resources, s.resources);
            assert!(withheld.cues.iter().all(|x| !*x));
        }
    }
}
#[test]
fn causal_interventions_and_unexposed_registration_are_fixed() {
    let cases = transfer_gate1_cases().unwrap();
    assert_eq!(cases.len(), 13);
    for c in &cases {
        assert_eq!(c.config.seed, 17);
        assert_eq!(
            c.interventions.last().unwrap().program,
            c.config.founders[0].program().unwrap()
        );
    }
    assert_eq!(
        transfer_interventions("reversion").unwrap().0,
        FundedProgram::Delayed
    );
    assert_eq!(
        transfer_interventions("restoration").unwrap().0,
        FundedProgram::Immediate
    );
    assert!(transfer_interventions("choose_after_outcome").is_err());
    let reg = transfer_gate1_evaluation_registration().unwrap();
    assert_eq!(reg["cells"], 96);
    assert_eq!(reg["launch_enabled"], false);
    assert!(reg["scientific_budget"].is_null());
    assert!(reg["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|b| b["seed"] != 17 && b["exposure"] == "registered_unexposed"));
}
#[test]
fn temporal_codec_continuation_matches_full_native_history() {
    let c = transfer_gate1_cases().unwrap()[1].clone();
    let mut full = FamilySimulation::new_transfer(c.config.clone(), c.environment.clone()).unwrap();
    let mut live = FamilySimulation::new_transfer_stream(c.config, c.environment).unwrap();
    let mut rows = vec![];
    for tick in 1..=64 {
        full.advance_tick().unwrap();
        live.advance_tick().unwrap();
        let b = live.drain_stream_tick().unwrap();
        assert_eq!(b.profile, TRANSFER_ENV_STREAM_PROFILE);
        rows.push(b.row);
        if [8, 24].contains(&tick) {
            let cp = live.live_checkpoint(cursor(tick)).unwrap();
            let att = live.live_attestation(cursor(tick)).unwrap();
            assert_eq!(&cp[..8], b"HGEMTA01");
            let mut wrong = cp.clone();
            wrong[..8].copy_from_slice(b"HGEMFS01");
            assert!(FamilySimulation::resume_live(&wrong, &cursor(tick), &att).is_err());
            let mut wrong = att.clone();
            wrong["native"]["transfer_environment"]["pattern"] = "Blocks4".into();
            assert!(FamilySimulation::resume_live(&cp, &cursor(tick), &wrong).is_err());
            live = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
        }
    }
    assert_eq!(full.rows, rows);
    assert_eq!(
        serde_json::to_value(full.final_frame()).unwrap(),
        serde_json::to_value(live.final_frame()).unwrap()
    );
}
#[test]
fn temporal_workers_and_allocation_preserve_outcomes() {
    let run = |workers, reversed| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| {
                let mut c = transfer_gate1_cases().unwrap()[6].clone();
                c.config.reversed = reversed;
                let mut s = FamilySimulation::new_transfer(c.config, c.environment).unwrap();
                for _ in 0..16 {
                    s.advance_tick().unwrap();
                }
                (
                    s.rows.clone(),
                    s.lineage.clone(),
                    s.admission.as_ref().unwrap().records.clone(),
                )
            })
    };
    let one = run(1, false);
    assert_eq!(one, run(4, false));
    assert_eq!(one, run(1, true));
}

#[test]
fn temporal_clean_stop_uses_its_declared_tick24_boundary() {
    let c = transfer_gate1_cases().unwrap().last().unwrap().clone();
    let tmp = tempfile::tempdir().unwrap();
    let r = write_transfer_stream_segment(
        &tmp.path().join("control"),
        c.config,
        c.environment,
        StreamFault::CleanStop,
    )
    .unwrap();
    assert_eq!(r.state, "Interrupted");
    assert_eq!(r.native_final_tick, 24);
    assert_eq!(r.committed_final_tick, 24);
}
