#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;

fn active() -> Simulation {
    Simulation::new(Config::default(), Instruction::BuildRelay).unwrap()
}
#[test]
fn seed_grows_paid_owned_relay_then_acquires_energy() {
    let mut sim = active();
    assert_eq!(
        sim.store()
            .nodes()
            .filter(|(_, n)| n.label.0 == CONNECTION)
            .count(),
        0
    );
    sim.advance_tick().unwrap();
    let first = &sim.rows()[0];
    assert_eq!(first.developed, 1);
    assert_eq!(first.harvested, 0.0);
    assert_eq!(first.dissipated, 7.0); // 2 development + 5 mandatory upkeep
    assert_eq!(
        sim.store()
            .nodes()
            .filter(|(_, n)| n.label.0 == CONNECTION)
            .count(),
        1
    );
    sim.advance_tick().unwrap();
    assert_eq!(sim.rows()[1].harvested, 8.0);
}
#[test]
fn paid_turnover_has_faithful_multigeneration_descent_and_idle_extinction() {
    let mut sim = active();
    sim.run().unwrap();
    let last = sim.rows().last().unwrap();
    assert!(last.births > 2 && last.deaths > 0 && last.live_organisms > 0);
    assert!(sim.rows().iter().all(|r| r.residual == 0.0));
    assert!(sim.lineage().values().any(|b| b
        .parent_id
        .is_some_and(|p| sim.lineage()[&p].parent_id.is_some())));
    assert!(sim.lineage().values().all(|b| b.opcode == 1));
    let mut idle = Simulation::new(Config::default(), Instruction::Idle).unwrap();
    idle.run().unwrap();
    let last = idle.rows().last().unwrap();
    assert_eq!(last.births, 0);
    assert_eq!(last.harvested, 0.0);
    assert_eq!(last.live_organisms, 0);
    assert_eq!(last.deaths, 1);
}
#[test]
fn full_state_roundtrip_at_birth_development_death_and_completed_boundaries() {
    let mut uninterrupted = active();
    uninterrupted.run().unwrap();
    for stop in [0, 1, 5, 8, 16, 32, 64] {
        let mut prefix = active();
        for _ in 0..stop {
            prefix.advance_tick().unwrap();
        }
        let mut resumed = Simulation::resume(&prefix.checkpoint().unwrap()).unwrap();
        resumed.run().unwrap();
        assert_eq!(resumed.rows(), uninterrupted.rows());
        assert_eq!(resumed.lineage(), uninterrupted.lineage());
        for (index, (a, b)) in resumed
            .frames()
            .iter()
            .zip(uninterrupted.frames())
            .enumerate()
        {
            let a = serde_json::to_value(a).unwrap();
            let b = serde_json::to_value(b).unwrap();
            let keys: Vec<_> = a
                .as_object()
                .unwrap()
                .keys()
                .filter(|k| a[*k] != b[*k])
                .collect();
            assert!(
                keys.is_empty(),
                "checkpoint at {stop}, frame {index}, differing fields {keys:?}"
            );
        }
    }
}
#[test]
fn checkpoint_corruption_and_trailing_data_are_rejected() {
    let mut bytes = active().checkpoint().unwrap();
    bytes[30] ^= 1;
    assert!(Simulation::resume(&bytes).is_err());
    let mut bytes = active().checkpoint().unwrap();
    bytes.push(0);
    assert!(Simulation::resume(&bytes).is_err());
}
#[test]
fn matching_ceiling_never_applies_partial_candidate_set_or_allows_checkpoint() {
    let config = Config {
        match_work: 1,
        ..Config::default()
    };
    let mut sim = Simulation::new(config, Instruction::BuildRelay).unwrap();
    let hash = sim.store().canonical_hash();
    assert!(sim.advance_tick().unwrap_err().contains("matching ceiling"));
    assert_eq!(sim.store().canonical_hash(), hash);
    assert_eq!(sim.store().step(), 0);
    assert!(sim.checkpoint().is_err());
}
#[test]
fn material_ceiling_stops_without_culling_and_bad_config_never_runs() {
    let config = Config {
        max_nodes: 20,
        ..Config::default()
    };
    let mut sim = Simulation::new(config, Instruction::BuildRelay).unwrap();
    assert_eq!(sim.advance_tick().unwrap_err(), "material ceiling");
    assert_eq!(sim.store().node_count(), 22); // native paid development, no pruning to force fit
    assert!(Simulation::new(
        Config {
            sites: 1,
            ..Config::default()
        },
        Instruction::Idle
    )
    .is_err());
}
#[test]
fn exclusive_output_does_not_overwrite_an_existing_run() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("smoke");
    write_smoke(
        &out,
        Config {
            ticks: 4,
            ..Config::default()
        },
    )
    .unwrap();
    let before = std::fs::read(out.join("active/manifest.json")).unwrap();
    assert!(write_smoke(&out, Config::default()).is_err());
    assert_eq!(
        before,
        std::fs::read(out.join("active/manifest.json")).unwrap()
    );
}

#[test]
fn exhausted_output_budget_is_durable_interruption_not_running_or_success() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("small");
    assert!(write_smoke(
        &out,
        Config {
            output_bytes: 1024 * 1024,
            ..Config::default()
        }
    )
    .is_err());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("active/manifest.json")).unwrap()).unwrap();
    let status: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("active/run_status.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["state"], "Interrupted");
    assert_eq!(status["engineering_execution_complete"], false);
    assert_eq!(status["training_admitted"], false);
    let bytes: u64 = std::fs::read_dir(out.join("active"))
        .unwrap()
        .map(|p| p.unwrap().metadata().unwrap().len())
        .sum();
    assert!(bytes <= 1024 * 1024);
}

#[test]
fn trajectories_match_at_one_and_four_candidate_collection_threads() {
    let run = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                let mut sim = Simulation::new(
                    Config {
                        ticks: 24,
                        ..Config::default()
                    },
                    Instruction::BuildRelay,
                )
                .unwrap();
                sim.run().unwrap();
                serde_json::to_value(sim.frames()).unwrap()
            })
    };
    assert!(
        run(1) == run(4),
        "thread count changed exact frame material"
    );
}
