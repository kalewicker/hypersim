#![cfg(feature = "ere")]
use hg_sim_core::ere::rule_relational as native;
use hg_train::ere::{
    graph_encoder as e, graph_encoder_exact as x, rule_relational_learning as old,
};
fn worlds() -> Vec<x::World> {
    old::DEVELOPMENT
        .iter()
        .map(|c| x::World {
            context: *c,
            observation: e::observe(&native::initial_observation(*c).unwrap()).unwrap(),
            // Arbitrary dense synthetic rewards, not context-selected target labels.
            rewards: (0..16).map(|a| ((a * 7 + 3) % 17) as f64 / 17.).collect(),
        })
        .collect()
}
#[test]
fn exact_objective_finite_difference_and_refusals() {
    let w = worlds();
    let theta = e::initialize();
    for mode in [e::Mode::Graph, e::Mode::Blind] {
        let (_, gradient) = x::objective(mode, &theta, &w).unwrap();
        for j in 0..110 {
            let mut plus = theta.clone();
            let mut minus = theta.clone();
            plus[j] += 1e-6;
            minus[j] -= 1e-6;
            let diff = (x::objective(mode, &plus, &w).unwrap().0
                - x::objective(mode, &minus, &w).unwrap().0)
                / 2e-6;
            assert!((diff - gradient[j]).abs() < 2e-8, "parameter {j}");
        }
        let mut zero = w.clone();
        for world in &mut zero {
            world.rewards.fill(0.);
        }
        let (v, g, after) = x::update(mode, &theta, &zero, 4.).unwrap();
        assert_eq!(v, 0.);
        assert_eq!(g, vec![0.; 110]);
        assert_eq!(after, theta);
    }
    assert!(x::objective(e::Mode::Graph, &theta, &w[..5]).is_err());
    let mut bad = w.clone();
    bad[0].context = old::RESERVED[0];
    assert!(x::objective(e::Mode::Graph, &theta, &bad).is_err());
    bad = w.clone();
    bad.swap(0, 1);
    assert!(x::objective(e::Mode::Graph, &theta, &bad).is_err());
    for rewards in [
        vec![0.; 15],
        vec![0.; 17],
        vec![f64::NAN; 16],
        vec![-0.1; 16],
        vec![1.1; 16],
    ] {
        bad = w.clone();
        bad[0].rewards = rewards;
        assert!(x::objective(e::Mode::Graph, &theta, &bad).is_err());
    }
    for rate in [0., -1., 4.01, f64::NAN] {
        assert!(x::update(e::Mode::Graph, &theta, &w, rate).is_err());
    }
    assert!(x::objective(e::Mode::Graph, &theta[..109], &w).is_err());
    assert!(x::objective(e::Mode::Graph, &vec![f64::NAN; 110], &w).is_err());
    assert_eq!(x::design()["maximum_native_attempts"], 0);
}
#[test]
fn fixed_archive_exact_gradient_diagnostic() {
    // Ordinary cargo test never launches measured optimization accidentally.
    let Some(path) = std::env::var_os("ERE_GRAPH_ENCODER_EXACT_OUTPUT") else {
        return;
    };
    let result = x::run(std::path::Path::new(&path))
        .expect("offline diagnostic failed; retain without retry");
    assert_eq!(result["completed_updates"], 200);
    assert_eq!(result["native_attempts"], 0);
}
