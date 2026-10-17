#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
fn recipe(id: Option<u32>, source: Option<u8>) -> NeighborhoodRecipe {
    NeighborhoodRecipe {
        program: "delayed_cue".into(),
        target_id: id,
        new_source: source,
    }
}
#[test]
fn parent_funded_development_and_failure_refunds() {
    for (id, source) in [(None, None), (Some(104), Some(1)), (Some(102), Some(7))] {
        let r = recipe(id, source);
        let mut s = FamilySimulation::new_funding(r.clone(), 24).unwrap();
        s.run().unwrap();
        let o = neighborhood_outcome(&s, r).unwrap();
        assert!(s.rows.iter().all(|r| r.residual == 0.0));
        assert_eq!(
            s.mutation
                .as_ref()
                .unwrap()
                .records
                .iter()
                .filter(|r| r.parent == 1)
                .count(),
            1
        );
        assert!(s.checkpoint().is_err());
        if id == None {
            assert!(o.first_child.unwrap().paid_construction_completed);
        } else if id == Some(104) {
            let m = s.mutation.as_ref().unwrap();
            assert_eq!(m.records.len(), 1);
            assert_eq!(m.records[0].refunded, 29.0);
            assert_eq!(m.records[0].request_charge, 1.0);
            assert_eq!(m.records[0].copying_charge, 0.0);
            assert_eq!(s.lineage.len(), 1);
        } else {
            let c = o.first_child.unwrap();
            assert_eq!(c.development_failures, 1);
            assert!(!c.paid_construction_completed);
        }
    }
    assert!(FamilySimulation::new_funding(recipe(None, None), 23).is_err());
    assert_eq!(funding_recipes().len(), 116);
}
#[test]
fn baseline_exact_legacy_and_funded_worker_identity() {
    let run = |workers, budget| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| {
                let mut s = if budget == 0 {
                    FamilySimulation::new_neighborhood(recipe(None, None)).unwrap()
                } else {
                    FamilySimulation::new_funding(recipe(None, None), budget).unwrap()
                };
                s.run().unwrap();
                (
                    serde_json::to_value(&s.journal).unwrap(),
                    serde_json::to_value(&s.rows).unwrap(),
                    serde_json::to_value(&s.lineage).unwrap(),
                    serde_json::to_value(&s.mutation).unwrap(),
                )
            })
    };
    assert_eq!(run(1, 0), run(1, 12));
    assert_eq!(run(1, 24), run(4, 24));
}
