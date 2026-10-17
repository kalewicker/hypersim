#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
fn recipe(program: &str, id: Option<u32>, source: Option<u8>) -> NeighborhoodRecipe {
    NeighborhoodRecipe {
        program: program.into(),
        target_id: id,
        new_source: source,
    }
}
#[test]
fn enumeration_is_complete_semantic_and_invalid_requests_have_no_fallback() {
    let recipes = neighborhood_recipes();
    assert_eq!(recipes.len(), 58);
    assert_eq!(recipes.iter().filter(|r| r.program == "cue").count(), 22);
    assert_eq!(
        recipes
            .iter()
            .filter(|r| r.program == "delayed_cue")
            .count(),
        36
    );
    let mut s = FamilySimulation::new_neighborhood(recipe("cue", Some(102), Some(1))).unwrap();
    s.run().unwrap();
    let m = s.mutation.as_ref().unwrap();
    assert_eq!(m.records.len(), 1);
    let r = &m.records[0];
    assert!(!r.syntax_valid);
    assert_eq!(r.operator, "enumerated_source");
    assert!(r.raw_draws.is_empty());
    assert_eq!(r.refunded, 16.5);
    assert_eq!(s.lineage.len(), 1);
    assert!(s.checkpoint().is_err());
}
#[test]
fn paid_child_failure_is_not_grammar_reachability_and_founder_is_not_reseeded() {
    for r in [
        recipe("cue", Some(101), Some(7)),
        recipe("delayed_cue", None, None),
    ] {
        let mut s = FamilySimulation::new_neighborhood(r.clone()).unwrap();
        s.run().unwrap();
        let o = neighborhood_outcome(&s, r.clone()).unwrap();
        assert!(o.syntax_valid);
        assert!(!o.first_child.as_ref().unwrap().paid_construction_completed);
        assert!(!o.first_child_reproduced);
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
        if r.program == "delayed_cue" {
            assert!(o.material_reachable);
            assert_eq!(
                o.first_child.unwrap().death_cause.as_deref(),
                Some("death_empty")
            );
        }
    }
}
#[test]
fn faithful_offspring_turnover_and_workers_have_exact_events() {
    let run = |n| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .unwrap()
            .install(|| {
                let r = recipe("cue", None, None);
                let mut s = FamilySimulation::new_neighborhood(r.clone()).unwrap();
                s.run().unwrap();
                let o = neighborhood_outcome(&s, r).unwrap();
                assert!(o.first_child_reproduced && o.sustained_turnover);
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
                (
                    serde_json::to_value(&s.journal).unwrap(),
                    serde_json::to_value(o).unwrap(),
                )
            })
    };
    assert_eq!(run(1), run(4));
}
