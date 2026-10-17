#![cfg(feature = "ere")]
use hg_train::ere::{rule_choice_learning::DrawStream, rule_composition_learning as l};
use std::path::PathBuf;

#[test]
fn fixed_composition_update_refusals() {
    let theta = vec![0.; 12];
    let mut samples: Vec<_> = l::DEVELOPMENT
        .into_iter()
        .flat_map(|context| {
            (0..2).map(move |_| l::Sample {
                context,
                actions: [1, 1],
                reward: 0.,
            })
        })
        .collect();
    assert_eq!(l::update(&theta, &samples).unwrap(), theta);
    samples[0].reward = 0.375;
    samples[1].reward = 0.375;
    let changed = l::update(&theta, &samples).unwrap();
    assert_eq!(&changed[..4], &[-0.0625, 0.1875, -0.0625, -0.0625]);
    assert_eq!(&changed[4..8], &[-0.0625, 0.1875, -0.0625, -0.0625]);
    assert!(l::update(&theta, &samples[..11]).is_err());
    samples[0].context = l::RESERVED[0];
    assert!(l::update(&theta, &samples).is_err());
    samples[0].context = l::DEVELOPMENT[0];
    samples[0].reward = f64::NAN;
    assert!(l::update(&theta, &samples).is_err());
    samples[0].reward = 1.1;
    assert!(l::update(&theta, &samples).is_err());
    samples[0].reward = 0.;
    samples[0].actions = [4, 0];
    assert!(l::update(&theta, &samples).is_err());
    assert!(l::update(&[f64::NAN; 12], &samples).is_err());
    assert!(l::endpoint(&theta, &[0.; 47]).is_err());
    assert!(l::endpoint(&theta, &[f64::NAN; 48]).is_err());
    let mut rewards = vec![0.; 48];
    rewards[5] = 0.375;
    rewards[26] = 0.375;
    rewards[47] = 0.375;
    assert_eq!(
        l::endpoint(&theta, &rewards).unwrap()["mean_expected_reward"],
        9. / 384.
    );
    let mut a = DrawStream(l::POLICY_SEEDS[0]);
    let mut b = DrawStream(l::POLICY_SEEDS[0]);
    for _ in 0..288 {
        assert_eq!(a.next_draw(), b.next_draw());
    }
}

#[test]
fn fixed_native_composition_learning_trial() {
    let tmp = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_RULE_COMPOSITION_LEARNING_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| tmp.path().join("trial"));
    let result = l::run(&out).expect("failed trial retained; no automatic retry");
    assert_eq!(result["recorded_cells"], 1206);
    assert_eq!(result["completed_updates"], 72);
    assert_eq!(result["native_attempts"], 28944);
    assert_eq!(result["scientific_acceptance"], false);
    // A negative learning endpoint is not an execution failure.
}
