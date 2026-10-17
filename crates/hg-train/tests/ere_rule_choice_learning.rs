#![cfg(feature = "ere")]
use hg_train::ere::rule_choice_learning::{self as learning, DrawStream, Sample};
use std::path::PathBuf;

#[test]
fn fixed_update_and_stream_refusals() {
    let theta = vec![0.; 12];
    let mut samples: Vec<_> = [10, 20, 30]
        .into_iter()
        .flat_map(|context| {
            (0..2).map(move |_| Sample {
                context,
                action: 1,
                reward: 0.,
            })
        })
        .collect();
    assert_eq!(learning::update(&theta, &samples).unwrap(), theta);
    samples[0].reward = 1.;
    samples[1].reward = 1.;
    assert_eq!(
        &learning::update(&theta, &samples).unwrap()[..4],
        &[-0.25, 0.75, -0.25, -0.25]
    );
    assert!(learning::update(&theta, &samples[..5]).is_err());
    samples[0].context = 20;
    assert!(learning::update(&theta, &samples).is_err());
    samples[0].context = 10;
    samples[0].reward = f64::NAN;
    assert!(learning::update(&theta, &samples).is_err());
    samples[0].reward = -0.1;
    assert!(learning::update(&theta, &samples).is_err());
    samples[0].reward = 1.1;
    assert!(learning::update(&theta, &samples).is_err());
    samples[0].reward = 0.;
    samples[0].action = 4;
    assert!(learning::update(&theta, &samples).is_err());
    let mut a = DrawStream(learning::POLICY_SEEDS[0]);
    let mut b = DrawStream(learning::POLICY_SEEDS[0]);
    for _ in 0..72 {
        let x = a.next_draw();
        assert_eq!(x, b.next_draw());
        assert!((0.0..1.0).contains(&x));
    }
}

#[test]
fn fixed_native_learning_trial() {
    let tmp = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_RULE_CHOICE_LEARNING_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| tmp.path().join("trial"));
    let result = learning::run(&out).expect("failed trial preserved; no outcome-dependent retry");
    assert_eq!(result["recorded_cells"], 540);
    assert_eq!(result["completed_updates"], 72);
    assert_eq!(result["native_attempts"], 12960);
    assert_eq!(result["scientific_acceptance"], false);
    // A negative learning endpoint remains a complete trial, not a harness failure.
}
