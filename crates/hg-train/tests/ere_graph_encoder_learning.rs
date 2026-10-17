#![cfg(feature = "ere")]
use hg_sim_core::ere::rule_relational as native;
use hg_train::ere::{
    graph_encoder as e, graph_encoder_learning as l, rule_relational_learning as old,
};
use serde_json::{json, Value};
use std::path::PathBuf;

#[test]
fn endpoint_baseline_and_update_refusals() {
    let initial = e::initialize();
    let mut rewards = vec![];
    let mut controls = vec![];
    let mut samples = vec![];
    for c in native::CONTEXTS {
        let correct = std::array::from_fn::<_, 2, _>(|s| {
            native::CHOICES[s]
                .iter()
                .position(|v| *v == Some(c[s]))
                .unwrap()
        });
        for a in 0..4 {
            for b in 0..4 {
                rewards.push(if [a, b] == correct { 9. / 24. } else { 0. });
            }
        }
        if old::RESERVED.contains(&c) {
            for arm in native::ARMS[1..].iter() {
                controls.push(json!({"context":c,"arm":arm,"reward":if *arm==native::Arm::ReversePaid{9./24.}else{0.}}));
            }
        }
        if old::DEVELOPMENT.contains(&c) {
            let o = e::observe(&native::initial_observation(c).unwrap()).unwrap();
            let input = e::input(e::Mode::Graph, &o);
            for _ in 0..2 {
                samples.push(e::Sample {
                    context: c,
                    input: input.clone(),
                    actions: correct,
                    reward: 0.,
                });
            }
        }
    }
    let comparison = l::comparison(
        l::POLICY_SEEDS[0],
        0,
        &initial,
        &initial,
        &rewards,
        &rewards,
        json!(controls),
    )
    .unwrap();
    assert_eq!(comparison["ceiling_pass"], false);
    assert_eq!(comparison["contrast_pass"], false);
    assert_eq!(comparison["reserved_gain_pass"], false);
    // Nonzero initialization already favors edit actions: uniform 1/16 would
    // incorrectly call an unchanged model's reserved probabilities improved.
    assert!(comparison["baseline"]["contexts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["reserved"] == true)
        .all(|c| c["correct_joint_probability"].as_f64().unwrap() > 1. / 16.));
    assert_eq!(comparison["every_reserved_joint_probability_gain"], false);
    for key in [
        "frozen_unchanged",
        "uniform_unchanged",
        "native_matrix_unchanged",
        "positive_control_unchanged",
        "positive_control_pass",
        "controls_pass",
    ] {
        assert_eq!(comparison[key], true);
    }
    assert_eq!(
        e::update(e::Mode::Graph, &initial, &samples, 4.).unwrap(),
        initial
    );
    assert!(e::update(e::Mode::Graph, &initial, &samples[..11], 4.).is_err());
    samples[0].context = old::RESERVED[0];
    assert!(e::update(e::Mode::Graph, &initial, &samples, 4.).is_err());
    samples[0].context = old::DEVELOPMENT[0];
    assert!(e::update(e::Mode::Blind, &initial, &samples, 4.).is_err());
    assert!(l::endpoint(e::Mode::Graph, &initial, &rewards[..143]).is_err());
    assert!(l::endpoint(e::Mode::Graph, &initial, &[f64::NAN; 144]).is_err());
    let d = l::design();
    assert_eq!(d["parameters_per_policy"], 110);
    assert_eq!(d["maximum_native_attempts"], 42768);
}

#[test]
fn fixed_native_graph_encoder_learning_trial() {
    let tmp = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_GRAPH_ENCODER_LEARNING_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| tmp.path().join("trial"));
    let m = l::run(&out).expect("invalid native trial retained; no retry");
    assert_eq!(m["recorded_cells"], 1782);
    assert_eq!(m["completed_updates"], 72);
    assert_eq!(m["native_attempts"], 42768);
    assert_eq!(m["scientific_acceptance"], false);
    assert!(m["learning_criterion_pass"].is_boolean());
    // Negative learning is a complete result, not a failing test.
    let _: Value = m;
}
