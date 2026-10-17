#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;

fn sim(name: &str, mode: CueMode, reserve: f64) -> D0Simulation {
    D0Simulation::new(d0_program(name).unwrap(), 17, mode, reserve).unwrap()
}

#[test]
fn graph_program_builds_distinct_paid_circuits_and_actions() {
    let mut direct = sim("cue", CueMode::Informative, 512.0);
    let mut constant = sim("constant0", CueMode::Informative, 512.0);
    let mut delayed = sim("delayed_cue", CueMode::Informative, 512.0);
    for s in [&mut direct, &mut constant, &mut delayed] {
        assert_eq!(s.graph_program().unwrap(), s.program);
        s.run().unwrap();
        assert!(s.rows.iter().all(|r| r.residual == 0.0));
        assert_eq!(s.rows.last().unwrap().status, 2);
        assert_eq!(
            s.rows.last().unwrap().development_paid,
            s.program.instructions.len() as u64
        );
    }
    let d = direct.rows.last().unwrap();
    assert_eq!((d.attempts, d.successes, d.failures), (22, 22, 0));
    assert!(constant.rows.last().unwrap().failures > 0);
    assert!(delayed.rows.last().unwrap().failures > 0);
    assert_ne!(
        d.material_nodes,
        constant.rows.last().unwrap().material_nodes
    );
    assert_ne!(
        constant.rows.last().unwrap().material_nodes,
        delayed.rows.last().unwrap().material_nodes
    );
    // BUFFER preparation reads only old SIGNAL: the extra state delays the cue one tick.
    for f in delayed
        .frames
        .iter()
        .filter(|f| f.events.iter().any(|e| e.rule == "attempt_commit"))
    {
        let (_, root) = f
            .nodes
            .iter()
            .find(|(_, n)| n.label.0 == 0xE1000000)
            .unwrap();
        assert_eq!(
            root.get_property(241008),
            Some(hg_core::PropertyValue::Bool(
                delayed.schedule.cues[f.tick as usize - 2]
            ))
        );
    }
    let mut withheld = sim("cue", CueMode::Withheld, 512.0);
    withheld.run().unwrap();
    assert!(withheld.rows.last().unwrap().successes < d.successes);
}

#[test]
fn bad_references_slots_writers_and_actuators_fail_once_and_pay() {
    for name in [
        "missing_port",
        "duplicate_slot",
        "duplicate_writer",
        "second_actuator",
    ] {
        let mut s = sim(name, CueMode::Informative, 512.0);
        s.run().unwrap();
        let last = s.rows.last().unwrap();
        assert_eq!(last.status, 3);
        assert_eq!(last.development_failures, 1);
        assert_eq!(last.attempts, 0);
        assert_eq!(last.development_paid, last.cursor as u64 + 1);
        assert_eq!(
            s.frames
                .iter()
                .flat_map(|f| &f.events)
                .filter(|e| e.rule == "development_fail")
                .count(),
            1
        );
    }
    let mut waiting = sim("cue", CueMode::Informative, 0.0);
    waiting.run().unwrap();
    assert_eq!(
        (
            waiting.rows[23].status,
            waiting.rows[23].cursor,
            waiting.rows[23].development_paid
        ),
        (1, 0, 0)
    );
    assert_eq!(
        waiting
            .frames
            .iter()
            .flat_map(|f| &f.events)
            .filter(|e| e.rule == "development_wait")
            .count(),
        1
    );
}

#[test]
fn grammar_bounds_and_instruction_identity_are_explicit() {
    let mut p = d0_program("cue").unwrap();
    p.instructions[1].id = p.instructions[0].id;
    assert!(p.validate().is_err());
    let mut p = d0_program("cue").unwrap();
    p.instructions.pop();
    assert!(p.validate().is_err());
    let mut p = d0_program("cue").unwrap();
    p.instructions[1].source = 8;
    assert!(p.validate().is_err());
    let mut p = d0_program("cue").unwrap();
    let before = p.canonical_bytes().unwrap();
    p.instructions[1].source = 1;
    assert_ne!(before, p.canonical_bytes().unwrap());
    assert_eq!(p.instructions[1].id, 101);
    let mut s = D0Simulation::new(p, 17, CueMode::Informative, 512.0).unwrap();
    s.run().unwrap();
    assert!(s.rows[23].failures > 0);
}

#[test]
fn maximum_length_recurrence_is_bounded_and_self_wires_fail() {
    use D0Op::*;
    let make = |ops: Vec<(D0Op, u8, u8, bool)>| D0Program {
        instructions: ops
            .into_iter()
            .enumerate()
            .map(|(i, (op, slot, source, initial))| D0Instruction {
                id: 100 + i as u32,
                op,
                slot,
                source,
                initial,
            })
            .collect(),
    };
    let p = make(vec![
        (State, 2, 0, false),
        (State, 3, 0, true),
        (State, 4, 0, true),
        (Actuator, 5, 0, false),
        (Wire, 2, 3, false),
        (Wire, 3, 2, false),
        (Wire, 5, 2, false),
        (Halt, 0, 0, false),
    ]);
    let mut s = D0Simulation::new(p, 17, CueMode::Informative, 512.0).unwrap();
    s.run().unwrap();
    assert_eq!(s.rows[23].development_paid, 8);
    assert_eq!(s.rows[23].status, 2);
    assert_eq!(s.rows[23].development_failures, 0);
    assert!(s.rows.iter().all(|r| r.residual == 0.0));
    let p = make(vec![
        (State, 2, 0, false),
        (Wire, 2, 2, false),
        (Halt, 0, 0, false),
    ]);
    let mut s = D0Simulation::new(p, 17, CueMode::Informative, 512.0).unwrap();
    s.run().unwrap();
    assert_eq!(s.rows[23].status, 3);
    let mut p = d0_program("cue").unwrap();
    let halt = p.instructions.pop().unwrap();
    for i in 0..6 {
        let mut g = p.instructions[0].clone();
        g.id = 200 + i;
        p.instructions.push(g)
    }
    p.instructions.push(halt);
    assert!(p.validate().is_err());
}

#[test]
fn checkpoints_preserve_partial_development_failure_waiting_and_full_history() {
    for (name, reserve) in [
        ("delayed_cue", 512.0),
        ("duplicate_writer", 512.0),
        ("cue", 0.0),
    ] {
        let mut full = sim(name, CueMode::Informative, reserve);
        full.run().unwrap();
        for stop in [0, 1, 2, 3, 4, 12, 24] {
            let mut prefix = sim(name, CueMode::Informative, reserve);
            for _ in 0..stop {
                prefix.advance_tick().unwrap()
            }
            let bytes = prefix.checkpoint().unwrap();
            let mut resumed = D0Simulation::resume(&bytes).unwrap();
            resumed.run().unwrap();
            assert_eq!(resumed.rows, full.rows);
            assert!(
                serde_json::to_value(&resumed.frames).unwrap()
                    == serde_json::to_value(&full.frames).unwrap(),
                "D0 continuation {name} stop {stop}"
            );
            let mut broken = bytes;
            let end = broken.len() - 1;
            broken[end] ^= 1;
            assert!(D0Simulation::resume(&broken).is_err());
        }
    }
}

#[test]
fn thread_count_preserves_native_events_and_allocator_continuation() {
    let run = |n| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .unwrap()
            .install(|| {
                let mut s = sim("delayed_cue", CueMode::Informative, 512.0);
                s.run().unwrap();
                serde_json::to_value(&s.frames).unwrap()
            })
    };
    assert!(
        run(1) == run(4),
        "D0 thread-dependent graph or event trajectory"
    );
}
