#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;

#[test]
fn maximum_genome_is_copied_field_for_field_and_child_state_is_reset() {
    use D0Op::*;
    let ops = vec![
        (State, 2, 0, false),
        (State, 3, 0, true),
        (State, 4, 0, true),
        (Actuator, 5, 0, false),
        (Wire, 2, 3, false),
        (Wire, 3, 2, false),
        (Wire, 5, 2, false),
        (Halt, 0, 0, false),
    ];
    let program = D0Program {
        instructions: ops
            .into_iter()
            .enumerate()
            .map(|(i, (op, slot, source, initial))| D0Instruction {
                id: 700 + i as u32,
                op,
                slot,
                source,
                initial,
            })
            .collect(),
    };
    let mut s = FamilySimulation::new(program.clone(), 17, CueMode::Informative).unwrap();
    while s.lineage.len() == 1 {
        s.advance_tick().unwrap();
        assert!(s.rows.len() <= 16);
    }
    assert_eq!(s.lineage[&2].program, program);
    assert_eq!(s.graph_program(2).unwrap(), program);
    let (_, child) = s
        .store()
        .nodes()
        .find(|(_, n)| {
            n.label.0 == 0xE1000000
                && n.get_property(240007) == Some(hg_core::PropertyValue::Int(2))
        })
        .unwrap();
    for key in [240004, 242001, 242002] {
        assert_eq!(
            child.get_property(key),
            Some(hg_core::PropertyValue::Int(0))
        );
    }
    for key in [242011, 241007, 241008] {
        assert_eq!(
            child.get_property(key),
            Some(hg_core::PropertyValue::Bool(false))
        );
    }
    for (_, n) in s
        .store()
        .nodes()
        .filter(|(_, n)| n.get_property(240001) == Some(hg_core::PropertyValue::Int(2)))
    {
        assert!(![0xE1000004, 0xE1000005, 0xE1010000].contains(&n.label.0));
        if n.label.0 == 0xE1000002 {
            assert_eq!(
                n.get_property(240005),
                Some(hg_core::PropertyValue::Bool(false))
            );
        }
    }
}

#[test]
fn paid_faithful_copy_has_runtime_reset_and_turnover() {
    let mut s =
        FamilySimulation::new(family_program("cue").unwrap(), 17, CueMode::Informative).unwrap();
    s.run().unwrap();
    let r = &s.rows[63];
    assert!(
        r.births >= 3 && r.deaths >= 3 && r.max_ancestry_depth >= 3 && r.live_organisms > 0,
        "{r:?}"
    );
    assert_eq!(r.attempts, r.successes);
    assert_eq!(r.failures, 0);
    assert!(s.rows.iter().all(|r| r.residual == 0.0));
    for b in s.lineage.values() {
        if let Some(p) = b.parent {
            assert_eq!(b.program, s.lineage[&p].program);
        }
    }
    for entry in s.journal.iter().filter(|p| p.pass == 15) {
        for e in &entry.events {
            assert!(e.rule.starts_with("birth_faithful_"));
            assert_eq!(e.created.len(), 14);
            assert!(e.deleted.is_empty());
        }
    }
    assert_eq!(s.journal.len(), 2752);
}

#[test]
fn inactive_remains_in_fixed_horizon_and_no_tombstone_survives() {
    let mut s = FamilySimulation::new(
        family_program("inactive").unwrap(),
        17,
        CueMode::Informative,
    )
    .unwrap();
    s.run().unwrap();
    assert_eq!(s.rows.len(), 64);
    assert_eq!(s.rows[63].attempts, 0);
    assert_eq!(s.rows[63].live_organisms, 0);
    assert!(s.rows[63].births > 0 && s.rows[63].deaths == s.rows[63].births + 1);
    assert!(s.store().nodes().all(|(_, n)| n.label.0 != 0xE1000007));
    assert!(s.rows.iter().all(|r| r.residual == 0.0));
}

#[test]
fn checkpoints_continue_across_copy_and_cleanup_with_exact_events() {
    let mut full =
        FamilySimulation::new(family_program("cue").unwrap(), 17, CueMode::Informative).unwrap();
    full.run().unwrap();
    for stop in [0, 3, 16, 32, 64] {
        let mut prefix =
            FamilySimulation::new(family_program("cue").unwrap(), 17, CueMode::Informative)
                .unwrap();
        for _ in 0..stop {
            prefix.advance_tick().unwrap()
        }
        let bytes = prefix.checkpoint().unwrap();
        let mut resumed = FamilySimulation::resume(&bytes).unwrap();
        resumed.run().unwrap();
        assert_eq!(full.rows, resumed.rows);
        assert_eq!(full.lineage, resumed.lineage);
        assert!(
            serde_json::to_value(&full.journal).unwrap()
                == serde_json::to_value(&resumed.journal).unwrap(),
            "family journal {stop}"
        );
        assert!(
            serde_json::to_value(full.final_frame()).unwrap()
                == serde_json::to_value(resumed.final_frame()).unwrap(),
            "family final graph {stop}"
        );
        let mut bad = bytes;
        *bad.last_mut().unwrap() ^= 1;
        assert!(FamilySimulation::resume(&bad).is_err());
    }
}

#[test]
fn thread_count_keeps_paid_family_history_identical() {
    let run = |n| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .unwrap()
            .install(|| {
                let mut s =
                    FamilySimulation::new(family_program("cue").unwrap(), 17, CueMode::Informative)
                        .unwrap();
                s.run().unwrap();
                serde_json::to_value((&s.journal, &s.lineage, s.final_frame())).unwrap()
            })
    };
    assert!(run(1) == run(4));
}
