//! Phase 7 exit criteria (docs/fixture_definitions.md): the fixture truth
//! table passes exactly as specified, and white noise never ranks highest
//! in interestingness.

use hg_eval::{run_truth_table, fixture_gate_status};
use hg_schema::GateStatus;

#[test]
fn fixture_truth_table() {
    let outcomes = run_truth_table();
    for o in &outcomes {
        assert!(
            o.passed,
            "{} classified {:?} (windows: {:?})",
            o.name, o.regime, o.per_window
        );
    }
    assert_eq!(fixture_gate_status(), GateStatus::Pass);
}

#[test]
fn white_noise_never_wins_interestingness() {
    let outcomes = run_truth_table();
    let noise = outcomes
        .iter()
        .find(|o| o.name == "WhiteNoiseToy")
        .unwrap()
        .mean_interestingness;
    for o in &outcomes {
        if o.name != "WhiteNoiseToy" {
            assert!(
                noise < o.mean_interestingness || noise < 0.5,
                "white noise ({noise:.3}) must not outrank {} ({:.3})",
                o.name,
                o.mean_interestingness
            );
        }
    }
}

/// Regression: above the PI sampling cap (4096 nodes), growth_ratio must
/// use the FULL node count on both sides. The bug: the previous window's
/// stored total was the sampled count, so a static 6400-node crystal
/// reported growth 6400/3200 = 2.0 and classified Exploding.
#[test]
fn static_graph_above_sampling_cap_is_not_exploding() {
    use hg_eval::{classify, WindowComputer};
    use hg_schema::Regime;

    let mut store = hg_core::HypergraphStore::new();
    hg_core::generate_crystal(&mut store, 80, 80, 2, false, 0); // 6400 nodes
    assert!(store.node_count() > hg_eval::thresholds::PI_SAMPLE_CAP);

    let mut computer = WindowComputer::new(1);
    let _w0 = computer.observe(&store.snapshot());
    let w1 = computer.observe(&store.snapshot()); // identical snapshot

    assert_eq!(w1.pi_sample_strategy, "stride");
    assert_eq!(w1.growth_ratio, 1.0, "static graph must have growth 1.0");
    assert_eq!(classify(&w1), Regime::Frozen, "static crystal is Frozen, not Exploding");
}
