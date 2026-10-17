#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
fn checkpoint() -> Vec<u8> {
    let mut s = FamilySimulation::new_lifecycle(LifecycleArm::Sham).unwrap();
    for _ in 0..3 {
        s.advance_tick().unwrap()
    }
    s.checkpoint().unwrap()
}
fn normalized_journal(s: &FamilySimulation) -> serde_json::Value {
    let mut journal = serde_json::to_value(&s.journal).unwrap();
    for entry in journal.as_array_mut().unwrap() {
        for event in entry["events"].as_array_mut().unwrap() {
            if event["rule"] == "communication_rescue" {
                event["rule"] = "communication_sham".into();
            }
        }
    }
    journal
}
#[test]
fn lawful_full_lifecycle_forks_keep_costs_and_restore_exact_material() {
    let bytes = checkpoint();
    let mut sham = FamilySimulation::fork_lifecycle(&bytes, LifecycleArm::Sham).unwrap();
    let mut rescue = FamilySimulation::fork_lifecycle(&bytes, LifecycleArm::Rescue).unwrap();
    let mut freeze = FamilySimulation::fork_lifecycle(&bytes, LifecycleArm::Freeze).unwrap();
    for s in [&mut sham, &mut rescue, &mut freeze] {
        s.run().unwrap();
        s.validate().unwrap();
        assert_eq!(s.journal.len(), 2816);
        assert!(s.rows.iter().all(|r| r.residual == 0.0));
    }
    assert_eq!(sham.rows, rescue.rows);
    assert_eq!(sham.lineage, rescue.lineage);
    assert_eq!(normalized_journal(&sham), normalized_journal(&rescue));
    assert_eq!(
        serde_json::to_value(sham.final_frame()).unwrap(),
        serde_json::to_value(rescue.final_frame()).unwrap()
    );
    assert!(freeze
        .journal
        .iter()
        .any(|s| s.events.iter().any(|e| e.rule == "communication_freeze")));
    assert!(freeze.rows[63].deaths > 0);
}
#[test]
fn release_boundary_replays_history_and_fork_rejects_wrong_checkpoint() {
    let bytes = checkpoint();
    let mut s = FamilySimulation::fork_lifecycle(&bytes, LifecycleArm::Release).unwrap();
    for _ in 3..8 {
        s.advance_tick().unwrap()
    }
    let cp = s.checkpoint().unwrap();
    assert_eq!(&cp[..8], b"HGEML001");
    let mut restored = FamilySimulation::resume(&cp).unwrap();
    s.run().unwrap();
    restored.run().unwrap();
    assert_eq!(s.rows, restored.rows);
    assert_eq!(normalized_journal(&s), normalized_journal(&restored));
    assert!(s
        .journal
        .iter()
        .filter(|e| e.tick == 9 && e.phase == "communication_")
        .flat_map(|e| &e.events)
        .all(|e| e.rule == "communication_sham"));
    let bad = FamilySimulation::new_lifecycle(LifecycleArm::Sham)
        .unwrap()
        .checkpoint()
        .unwrap();
    assert!(FamilySimulation::fork_lifecycle(&bad, LifecycleArm::Freeze).is_err());
    let mut corrupted = cp;
    let n = corrupted.len() - 1;
    corrupted[n] ^= 1;
    assert!(FamilySimulation::resume(&corrupted).is_err());
}
#[test]
fn arbitration_separates_counter_only_conflict_and_physical_contention() {
    for topology in [OfferTopology::Disjoint, OfferTopology::Competing] {
        for policy in [BirthPolicy::Global, BirthPolicy::Local] {
            for rotation in 0..8 {
                for reversed in [false, true] {
                    let r = run_arbitration(ArbitrationRecipe {
                        policy,
                        topology,
                        rotation,
                        reversed,
                    })
                    .unwrap();
                    assert_eq!(r.offers.len(), 2);
                    assert_eq!(r.residual, 0.0);
                    assert_eq!(
                        r.accepted_parents.len(),
                        if policy == BirthPolicy::Local && topology == OfferTopology::Disjoint {
                            2
                        } else {
                            1
                        }
                    );
                    if policy == BirthPolicy::Global && topology == OfferTopology::Disjoint {
                        assert!(r.offers.iter().all(|o| o.physical_conflicts.is_empty()
                            && o.counter_only_conflicts.len() == 1));
                    }
                }
            }
        }
    }
}
#[test]
fn worker_threads_preserve_fairness_fixture_and_lifecycle_events() {
    let run = |n| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .unwrap()
            .install(|| {
                let fair = run_arbitration(ArbitrationRecipe {
                    policy: BirthPolicy::Local,
                    topology: OfferTopology::Disjoint,
                    rotation: 7,
                    reversed: true,
                })
                .unwrap();
                let mut s =
                    FamilySimulation::fork_lifecycle(&checkpoint(), LifecycleArm::Release).unwrap();
                s.run().unwrap();
                (serde_json::to_value(fair).unwrap(), normalized_journal(&s))
            })
    };
    assert_eq!(run(1), run(4));
}
