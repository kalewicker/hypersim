#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;
fn recipe(case: &str, reversed: bool) -> AdmissionRecipe {
    AdmissionRecipe {
        case: case.into(),
        rotation: 0,
        reversed,
    }
}
#[test]
fn lottery_is_presentation_independent_and_disjoint() {
    for seed in 0..32 {
        for competing in [false, true] {
            let a = AdmissionOffer {
                parent: 1,
                home: 0,
                target: 1,
                length: 3,
            };
            let b = AdmissionOffer {
                parent: 2,
                home: if competing { 2 } else { 4 },
                target: if competing { 1 } else { 5 },
                length: 5,
            };
            let x = lottery(seed, 1, vec![a.clone(), b.clone()]).unwrap();
            assert_eq!(x, lottery(seed, 1, vec![b, a]).unwrap());
            assert_eq!(x.selected.len(), if competing { 1 } else { 2 });
        }
    }
    for seed in 0..32 {
        let offers = vec![
            AdmissionOffer {
                parent: 1,
                home: 0,
                target: 1,
                length: 3,
            },
            AdmissionOffer {
                parent: 1,
                home: 0,
                target: 7,
                length: 3,
            },
            AdmissionOffer {
                parent: 2,
                home: 2,
                target: 3,
                length: 5,
            },
            AdmissionOffer {
                parent: 3,
                home: 4,
                target: 2,
                length: 1,
            },
        ];
        let r = lottery(seed, 3, offers).unwrap();
        let mut parents = std::collections::BTreeSet::new();
        let mut sites = std::collections::BTreeSet::new();
        for i in r.selected {
            let o = &r.offers[i];
            assert!(parents.insert(o.parent));
            assert!(sites.insert(o.home));
            assert!(sites.insert(o.target));
        }
    }
    assert!(lottery(
        17,
        1,
        vec![
            AdmissionOffer {
                parent: 1,
                home: 0,
                target: 1,
                length: 3
            };
            2
        ]
    )
    .is_err());
}
#[test]
fn full_ecology_parallel_settlement_and_replay() {
    for case in ["faithful_disjoint", "mixed_competing", "mixed_settlement"] {
        let mut s = FamilySimulation::new_admission(recipe(case, false)).unwrap();
        for _ in 0..8 {
            s.advance_tick().unwrap();
        }
        let cp = s.checkpoint().unwrap();
        let mut r = FamilySimulation::resume(&cp).unwrap();
        s.run().unwrap();
        r.run().unwrap();
        assert_eq!(s.rows, r.rows);
        assert_eq!(
            serde_json::to_value(&s.journal).unwrap(),
            serde_json::to_value(&r.journal).unwrap()
        );
        assert_eq!(s.admission, r.admission);
        assert_eq!(
            serde_json::to_value(s.final_frame()).unwrap(),
            serde_json::to_value(r.final_frame()).unwrap()
        );
        if case != "mixed_competing" {
            assert!(
                s.admission
                    .as_ref()
                    .unwrap()
                    .rounds
                    .iter()
                    .any(|r| r.selected.len() > 1),
                "{case}"
            );
        }
        assert!(s.rows.last().unwrap().deaths > 0);
        if case == "mixed_settlement" {
            let a = s.admission.as_ref().unwrap();
            assert!(a
                .records
                .iter()
                .any(|r| !r.syntax_valid && r.refunded > 0.0));
            assert!(a.records.iter().any(|r| r.child_id.is_some()));
        }
        let mut broken = cp;
        let last = broken.len() - 1;
        broken[last] ^= 1;
        assert!(FamilySimulation::resume(&broken).is_err());
    }
}
#[test]
fn allocation_and_thread_invariance() {
    for case in ["faithful_disjoint", "mixed_competing", "mixed_settlement"] {
        let run = |threads, reversed| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap()
                .install(|| {
                    let mut s = FamilySimulation::new_admission(recipe(case, reversed)).unwrap();
                    s.run().unwrap();
                    s
                })
        };
        let a = run(1, false);
        let b = run(4, true);
        let c = run(4, false);
        let rows = |s: &FamilySimulation| {
            s.rows
                .iter()
                .cloned()
                .map(|mut r| {
                    r.match_work = 0;
                    r
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(rows(&a), rows(&b));
        let aa = a.admission.as_ref().unwrap();
        let bb = b.admission.as_ref().unwrap();
        assert_eq!(aa.rounds, bb.rounds);
        assert_eq!(aa.records, bb.records);
        assert_eq!(
            serde_json::to_value(a.lineage.clone()).unwrap(),
            serde_json::to_value(b.lineage).unwrap()
        );
        assert_eq!(a.rows, c.rows);
        assert_eq!(
            serde_json::to_value(a.journal).unwrap(),
            serde_json::to_value(c.journal).unwrap()
        );
    }
}
