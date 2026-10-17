#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
fn simulation(kernel: MutationKernel) -> FamilySimulation {
    FamilySimulation::new_mutation(
        family_program("cue").unwrap(),
        17,
        CueMode::Informative,
        kernel,
    )
    .unwrap()
}
#[test]
fn invalid_proposal_is_charged_once_refunded_without_child_or_retry() {
    let mut s = simulation(MutationKernel::Invalid);
    s.run().unwrap();
    let m = s.mutation.as_ref().unwrap();
    assert!(!m.records.is_empty());
    assert!(m.pending.is_none());
    assert_eq!(s.lineage.len(), 1);
    for r in &m.records {
        assert!(!r.syntax_valid && r.settled);
        assert_eq!(r.request_charge, 1.0);
        assert_eq!(r.refunded, 16.5);
        assert_eq!(r.copying_charge, 0.0);
        assert!(r.child_id.is_none() && r.raw_draws.is_empty());
        assert_eq!(r.target_id, Some(102));
    }
    assert!(s.rows.iter().all(|r| r.residual == 0.0));
    assert_eq!(
        s.journal
            .iter()
            .flat_map(|e| &e.events)
            .filter(|e| e.rule.starts_with("reserve_"))
            .count(),
        m.records.len()
    );
    assert_eq!(
        s.journal
            .iter()
            .flat_map(|e| &e.events)
            .filter(|e| e.rule == "transaction_reject")
            .count(),
        m.records.len()
    );
    assert!(s.store().nodes().all(|(_, n)| n.label.0 != 0xE1020000));
}
#[test]
fn valid_edit_is_copied_and_runtime_failure_is_paid() {
    for kernel in [MutationKernel::Coin, MutationKernel::MissingPort] {
        let mut s = simulation(kernel);
        s.run().unwrap();
        let m = s.mutation.as_ref().unwrap();
        assert!(m.records.iter().any(|r| r.child_id.is_some()));
        for r in &m.records {
            assert!(r.syntax_valid && r.settled);
            assert_eq!(r.refunded, 0.0);
            assert_eq!(r.copying_charge, 4.5);
            let b = &s.lineage[&r.child_id.unwrap()];
            assert_eq!(b.program, r.proposed_program);
            assert_eq!(
                b.program.instructions[1].source,
                if kernel == MutationKernel::Coin { 1 } else { 7 }
            );
        }
        if kernel == MutationKernel::MissingPort {
            assert!(s.rows[63].development_failures > 0);
            assert!(m.records.iter().all(|r| !r.material_reachable));
        }
    }
}
#[test]
fn semantic_sampling_uses_id_order_and_records_all_draws() {
    let mut p = family_program("cue").unwrap();
    p.instructions[0].id = 700;
    p.instructions[1].id = 22;
    p.instructions[2].id = 1000;
    let mut a = MutationState::new(MutationKernel::SourceKernel, 17);
    let mut b = a.clone();
    for _ in 0..40 {
        assert_eq!(a.proposal(&p).unwrap(), b.proposal(&p).unwrap());
    }
    assert_eq!(a, b);
    assert!(a.draw_index > 40);
    assert_eq!(
        source_neighborhood(&p)["neighbors"]
            .as_array()
            .unwrap()
            .len(),
        21
    );
    // Arbitrary arena padding cannot enter this API; target domain is sorted semantic IDs.
    let mut bad = p.clone();
    bad.instructions[2].id = 22;
    let old = a.clone();
    assert!(a.proposal(&bad).is_err());
    assert_eq!(old, a);
}
#[test]
fn kernel_continuation_preserves_proposals_rng_native_journal_and_material() {
    let mut s = simulation(MutationKernel::SourceKernel);
    let mut checkpoints = vec![];
    for tick in 0..=64 {
        if [0, 3, 16, 64].contains(&tick) {
            checkpoints.push(s.checkpoint().unwrap());
        }
        if tick < 64 {
            s.advance_tick().unwrap();
        }
    }
    for cp in checkpoints {
        assert_eq!(&cp[..8], b"HGEMM002");
        let mut resumed = FamilySimulation::resume(&cp).unwrap();
        resumed.run().unwrap();
        assert_eq!(resumed.mutation, s.mutation);
        assert_eq!(
            serde_json::to_value(&resumed.journal).unwrap(),
            serde_json::to_value(&s.journal).unwrap()
        );
        assert_eq!(
            serde_json::to_value(resumed.final_frame()).unwrap(),
            serde_json::to_value(s.final_frame()).unwrap()
        );
        let mut damaged = cp;
        let n = damaged.len() - 1;
        damaged[n] ^= 1;
        assert!(FamilySimulation::resume(&damaged).is_err());
    }
}
#[test]
fn mutation_history_is_identical_at_one_and_four_threads() {
    let run = |n| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .unwrap()
            .install(|| {
                let mut s = simulation(MutationKernel::SourceKernel);
                s.run().unwrap();
                (s.mutation, serde_json::to_value(s.journal).unwrap())
            })
    };
    assert_eq!(run(1), run(4));
}

#[test]
fn reversed_arena_allocation_preserves_first_semantic_proposal() {
    let p = family_program("cue").unwrap();
    let mut a = simulation(MutationKernel::SourceKernel);
    let mut b = FamilySimulation::new_mutation_reversed(
        p,
        17,
        CueMode::Informative,
        MutationKernel::SourceKernel,
    )
    .unwrap();
    assert_ne!(
        serde_json::to_value(&a.initial).unwrap(),
        serde_json::to_value(&b.initial).unwrap()
    );
    for s in [&mut a, &mut b] {
        while s.mutation.as_ref().unwrap().records.is_empty() {
            s.advance_tick().unwrap();
        }
    }
    let x = &a.mutation.as_ref().unwrap().records[0];
    let y = &b.mutation.as_ref().unwrap().records[0];
    assert_eq!(x.parent_program, y.parent_program);
    assert_eq!(x.proposed_bytes, y.proposed_bytes);
    assert_eq!(x.target_id, y.target_id);
    assert_eq!(x.raw_draws, y.raw_draws);
    assert_eq!(x.syntax_valid, y.syntax_valid);
    let mut continued = FamilySimulation::resume(&b.checkpoint().unwrap()).unwrap();
    b.run().unwrap();
    continued.run().unwrap();
    assert_eq!(continued.mutation, b.mutation);
    assert_eq!(
        serde_json::to_value(continued.journal).unwrap(),
        serde_json::to_value(b.journal).unwrap()
    );
    // Site/handle contention remains explicitly outside this semantic-target assertion.
}
