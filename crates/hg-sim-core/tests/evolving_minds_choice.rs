#![cfg(feature = "evolving-minds")]
use hg_core::PropertyValue;
use hg_sim_core::evolving_minds::*;

#[test]
fn paid_information_control_has_multigeneration_descent_and_inactive_extinction() {
    for seed in [17, 29] {
        let mut reactive =
            ChoiceSimulation::new(Controller::Reactive, seed, CueMode::Informative).unwrap();
        reactive.run().unwrap();
        let row = reactive.rows.last().unwrap();
        assert!(row.births >= 3 && row.live_organisms > 0);
        assert_eq!(row.successes, row.attempts);
        assert_eq!(row.failures, 0);
        let mut depths = std::collections::BTreeMap::from([(1, 0)]);
        for (&id, b) in &reactive.lineage {
            if let Some(parent) = b.parent_id {
                depths.insert(id, depths[&parent] + 1);
            }
        }
        assert!(*depths.values().max().unwrap() >= 3);
        for controller in [
            Controller::Constant0,
            Controller::Constant1,
            Controller::Random,
            Controller::Inactive,
        ] {
            let mut control =
                ChoiceSimulation::new(controller, seed, CueMode::Informative).unwrap();
            control.run().unwrap();
            assert!(row.harvested >= control.rows.last().unwrap().harvested + 16.0);
            assert!(control.rows.iter().all(|r| r.residual == 0.0));
        }
    }
}

#[test]
fn failure_spends_attempt_before_outcome_and_prevents_a_second_action() {
    let mut sim = ChoiceSimulation::new(Controller::Constant0, 17, CueMode::Informative).unwrap();
    sim.run().unwrap();
    let failed = sim
        .frames
        .iter()
        .position(|f| f.events.iter().any(|e| e.rule == "settle_failure"))
        .unwrap();
    let event = &sim.frames[failed].events[0];
    let root = event
        .bound
        .iter()
        .find(|id| {
            sim.frames[failed - 1]
                .nodes
                .iter()
                .any(|(n, v)| n == *id && v.label.0 == ROOT)
        })
        .unwrap();
    let energy = |f: usize| {
        sim.frames[f]
            .nodes
            .iter()
            .find(|(id, _)| id == root)
            .unwrap()
            .1
            .get_property(ENERGY)
            .unwrap()
    };
    assert_eq!(
        match (energy(failed - 2), energy(failed - 1)) {
            (PropertyValue::Float(a), PropertyValue::Float(b)) => a - b,
            _ => panic!("wrong energy type"),
        },
        1.0
    );
    assert_eq!(energy(failed - 1), energy(failed));
    assert_eq!(
        sim.frames[failed - 1]
            .events
            .iter()
            .filter(|e| e.rule == "attempt_commit")
            .count(),
        1
    );
    assert_eq!(
        sim.rows.last().unwrap().attempts,
        sim.rows.last().unwrap().successes + sim.rows.last().unwrap().failures
    );
    let mut withheld = ChoiceSimulation::new(Controller::Reactive, 17, CueMode::Withheld).unwrap();
    withheld.run().unwrap();
    let mut constant = ChoiceSimulation::new(Controller::Constant0, 17, CueMode::Withheld).unwrap();
    constant.run().unwrap();
    assert_eq!(withheld.rows, constant.rows);
}

#[test]
fn checkpoint_replays_graph_events_lineage_and_rejects_corruption() {
    let mut full = ChoiceSimulation::new(Controller::Reactive, 29, CueMode::Informative).unwrap();
    full.run().unwrap();
    for stop in [0, 1, 16, 32, 64] {
        let mut prefix =
            ChoiceSimulation::new(Controller::Reactive, 29, CueMode::Informative).unwrap();
        for _ in 0..stop {
            prefix.advance_tick().unwrap();
        }
        let bytes = prefix.checkpoint().unwrap();
        let mut resumed = ChoiceSimulation::resume(&bytes).unwrap();
        resumed.run().unwrap();
        assert_eq!(resumed.rows, full.rows);
        assert_eq!(resumed.lineage, full.lineage);
        for (i, (a, b)) in resumed.frames.iter().zip(&full.frames).enumerate() {
            assert!(
                serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
                "frame mismatch checkpoint {stop} frame {i}"
            );
        }
        let mut bad = bytes;
        *bad.last_mut().unwrap() ^= 1;
        assert!(ChoiceSimulation::resume(&bad).is_err());
    }
}

#[test]
fn choice_trajectory_is_equal_at_one_and_four_threads() {
    let run = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                let mut sim =
                    ChoiceSimulation::new(Controller::Reactive, 17, CueMode::Informative).unwrap();
                sim.run().unwrap();
                serde_json::to_value(sim.frames).unwrap()
            })
    };
    assert_eq!(run(1), run(4));
}
