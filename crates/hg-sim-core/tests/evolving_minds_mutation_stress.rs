#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
#[test]
fn grammar_neighborhood_contains_unreachable_edits_and_no_invalid_syntax() {
    let p = FundedProgram::Immediate.program().unwrap();
    let candidates = grammar_source_candidates(&p).unwrap();
    assert_eq!(candidates.len(), 14);
    assert!(candidates.iter().all(|(_, _, q)| q.validate().is_ok()));
    assert!(candidates
        .iter()
        .any(|(id, source, _)| *id == 103 && *source == 7));
    assert!(candidates
        .windows(2)
        .all(|w| (w[0].0, w[0].1) < (w[1].0, w[1].1)));
    for seed in 0..128 {
        let mut sampler = MutationState::new(MutationKernel::GrammarSourceKernel, seed);
        let (q, id, _, source, raw) = sampler.proposal(&p).unwrap();
        assert_eq!(id.is_some(), raw[0] % 4 != 0);
        if let (Some(id), Some(source)) = (id, source) {
            assert!(candidates
                .iter()
                .any(|(i, s, x)| *i == id && *s == source && *x == q));
        } else {
            assert_eq!(q, p);
        }
    }
    let halt = D0Program {
        instructions: vec![p.instructions.last().unwrap().clone()],
    };
    assert!(grammar_source_candidates(&halt).unwrap().is_empty());
    let (q, target, _, _, raw) = MutationState::new(MutationKernel::GrammarSourceKernel, 17)
        .proposal(&halt)
        .unwrap();
    assert_eq!(q, halt);
    assert!(target.is_none());
    assert_eq!(raw.len(), 1);
}
#[test]
fn old_kernel_encoding_and_frozen_transfer_envelope_remain_unchanged() {
    for (index, k) in [
        MutationKernel::Faithful,
        MutationKernel::SourceKernel,
        MutationKernel::Coin,
        MutationKernel::Invalid,
        MutationKernel::MissingPort,
        MutationKernel::GrammarSourceKernel,
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(bincode::serialize(k).unwrap(), (index as u32).to_le_bytes());
    }
    let c = mutation_stress_cases().unwrap()[3].clone();
    assert!(FamilySimulation::new_funded_admission(c.config.clone()).is_err());
    assert!(FamilySimulation::new_transfer(c.config.clone(), c.environment.clone()).is_err());
    let mut bad = c.config;
    bad.kernels[0] = MutationKernel::Faithful;
    assert!(FamilySimulation::new_mutation_stress(bad, c.environment).is_err());
}
fn cursor(t: u64) -> EvidenceCursor {
    EvidenceCursor {
        segment_sha256: "a".repeat(64),
        tick: t,
        step: 46 * t,
        local_ticks: t,
        offset: 0,
        log_sha256: "b".repeat(64),
        previous_commit_sha256: None,
    }
}
#[test]
fn grammar_native_live_continuation_and_certificates_match_full_history() {
    let c = mutation_stress_cases().unwrap()[3].clone();
    let mut full =
        FamilySimulation::new_mutation_stress(c.config.clone(), c.environment.clone()).unwrap();
    let mut live = FamilySimulation::new_mutation_stress_stream(c.config, c.environment).unwrap();
    let mut rows = vec![];
    let mut records = vec![];
    for tick in 1..=64 {
        full.advance_tick().unwrap();
        live.advance_tick().unwrap();
        let bundle = live.drain_stream_tick().unwrap();
        assert_eq!(bundle.profile, MUTATION_STRESS_STREAM_PROFILE);
        rows.push(bundle.row);
        records.extend(bundle.records);
        if [8, 24].contains(&tick) {
            let cp = live.live_checkpoint(cursor(tick)).unwrap();
            let att = live.live_attestation(cursor(tick)).unwrap();
            assert_eq!(&cp[..8], b"HGEMMS01");
            let mut cross = cp.clone();
            cross[..8].copy_from_slice(b"HGEMTA01");
            assert!(FamilySimulation::resume_live(&cross, &cursor(tick), &att).is_err());
            let mut forged = att.clone();
            forged["native"]["mutation_stress_policy"] = "unknown".into();
            assert!(FamilySimulation::resume_live(&cp, &cursor(tick), &forged).is_err());
            live = FamilySimulation::resume_live(&cp, &cursor(tick), &att).unwrap();
        }
    }
    assert_eq!(rows, full.rows);
    assert_eq!(records, full.admission.as_ref().unwrap().records);
    assert_eq!(
        serde_json::to_value(full.final_frame()).unwrap(),
        serde_json::to_value(live.final_frame()).unwrap()
    );
    assert!(!records.is_empty());
    for r in &records {
        let cert = grammar_certificate(r).unwrap();
        assert_eq!(cert.candidate_count, 14);
        assert_eq!(cert.each_candidate_probability_numerator, 3);
        assert_eq!(cert.each_candidate_probability_denominator, 56);
    }
}
#[test]
fn grammar_worker_allocation_and_stop_boundary_are_bound() {
    let run = |workers, reversed| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| {
                let mut c = mutation_stress_cases().unwrap()[3].clone();
                c.config.reversed = reversed;
                let mut s = FamilySimulation::new_mutation_stress(c.config, c.environment).unwrap();
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
    let c = mutation_stress_cases().unwrap().last().unwrap().clone();
    let tmp = tempfile::tempdir().unwrap();
    let r = write_mutation_stress_stream_segment(
        &tmp.path().join("source"),
        c.config,
        c.environment,
        StreamFault::CleanStop,
    )
    .unwrap();
    assert_eq!(r.committed_final_tick, 24);
    assert_eq!(r.state, "Interrupted");
    assert_eq!(
        resume_stream_segment(&tmp.path().join("source"), &tmp.path().join("resumed"))
            .unwrap()
            .committed_final_tick,
        64
    );
}
