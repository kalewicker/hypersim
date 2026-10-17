#![cfg(feature = "evolving-minds")]
use hg_sim_core::evolving_minds::*;

fn checkpoint() -> Vec<u8> {
    let mut sim = Simulation::new(
        Config {
            ticks: 2,
            ..Config::default()
        },
        Instruction::BuildRelay,
    )
    .unwrap();
    sim.run().unwrap();
    sim.checkpoint().unwrap()
}

#[test]
fn native_causal_forks_are_matched_and_owner_rescue_restores_exact_history() {
    let bytes = checkpoint();
    let probes: Vec<_> = ["sham", "signal_clamp", "foreign_owner", "owner_rescue"]
        .iter()
        .map(|arm| probe_checkpoint(&bytes, arm).unwrap())
        .collect();
    assert_eq!(
        probes.iter().map(|p| p.acquired).collect::<Vec<_>>(),
        vec![8.0, 0.0, 0.0, 8.0]
    );
    assert!(!probes[2].lifecycle_valid);
    let expected = &probes[0].frames[0];
    for p in &probes {
        let start = &p.frames[0];
        assert_eq!(start.edges.len(), expected.edges.len());
        assert_eq!(start.nodes.len(), expected.nodes.len());
        assert_eq!(
            serde_json::to_value(&start.edges).unwrap(),
            serde_json::to_value(&expected.edges).unwrap()
        );
        for ((id, n), (other_id, other)) in start.nodes.iter().zip(&expected.nodes) {
            assert_eq!(id, other_id);
            assert_eq!(n.get_property(ENERGY), other.get_property(ENERGY));
            assert_eq!(n.get_property(MAINTENANCE), other.get_property(MAINTENANCE));
        }
    }
    assert_eq!(
        serde_json::to_value(&probes[0].frames).unwrap(),
        serde_json::to_value(&probes[3].frames).unwrap()
    );
    assert_eq!(probes[2].frames[1].events.len(), 0);
    assert_eq!(probes[1].frames[1].events.len(), 1);
    assert_eq!(probes[0].frames[1].events.len(), 1);
    // Repeated forks do not mutate their source or consume another arm's RNG.
    assert_eq!(
        serde_json::to_value(probe_checkpoint(&bytes, "sham").unwrap()).unwrap(),
        serde_json::to_value(&probes[0]).unwrap()
    );
}

#[test]
fn unsupported_checkpoint_and_unknown_arm_are_rejected() {
    let seed = Simulation::new(Config::default(), Instruction::BuildRelay)
        .unwrap()
        .checkpoint()
        .unwrap();
    assert!(probe_checkpoint(&seed, "sham").is_err());
    assert!(probe_checkpoint(&checkpoint(), "invented").is_err());
    let mut corrupt = checkpoint();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(probe_checkpoint(&corrupt, "sham").is_err());
}
