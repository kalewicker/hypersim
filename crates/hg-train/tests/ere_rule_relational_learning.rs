#![cfg(feature = "ere")]
use hg_sim_core::ere::rule_relational as native;
use hg_train::ere::{
    rule_relational as old,
    rule_relational_learning::{self as l, Mode, Sample},
};
use std::path::PathBuf;

#[test]
fn matched_policy_update_refusals() {
    let theta = [
        0.2, -0.3, 0.1, -0.4, 0.7, -0.1, 0.2, 0.8, 0.4, -0.2, -0.3, 0.9, 0.1, 0.2, -0.4, 0.6,
    ];
    let mut blind_reference = None;
    for context in native::CONTEXTS {
        let o = old::observe(&native::initial_observation(context).unwrap()).unwrap();
        let graph = l::input(Mode::Graph, &o);
        let blind = l::input(Mode::Blind, &o);
        assert_eq!(
            l::probabilities(&theta, &graph).unwrap(),
            old::probabilities(&theta, &o).unwrap()
        );
        let blind_p = l::probabilities(&theta, &blind).unwrap();
        if let Some(p) = blind_reference {
            assert_eq!(p, blind_p);
        } else {
            blind_reference = Some(blind_p);
        }
        for a in 0..4 {
            for b in 0..4 {
                let actions = [a, b];
                assert_eq!(
                    l::score_gradient(&theta, &graph, actions).unwrap(),
                    old::score_gradient(&theta, &o, actions).unwrap()
                );
                for input in [&graph, &blind] {
                    let g = l::score_gradient(&theta, input, actions).unwrap();
                    let p = l::probabilities(&theta, input).unwrap();
                    for i in 0..16 {
                        let mut hi = theta;
                        let mut lo = theta;
                        hi[i] += 1e-6;
                        lo[i] -= 1e-6;
                        let fd = (l::log_probability(&hi, input, actions).unwrap()
                            - l::log_probability(&lo, input, actions).unwrap())
                            / 2e-6;
                        assert!((fd - g[i]).abs() < 3e-9);
                    }
                    let draws = std::array::from_fn(|s| {
                        p[s][..actions[s]].iter().sum::<f64>() + p[s][actions[s]] / 2.
                    });
                    assert_eq!(l::sample(&theta, input, draws).unwrap(), actions);
                }
            }
        }
    }
    let zero = vec![0.; 16];
    let mut samples: Vec<_> = l::DEVELOPMENT
        .into_iter()
        .flat_map(|context| {
            let o = old::observe(&native::initial_observation(context).unwrap()).unwrap();
            let input = l::input(Mode::Graph, &o);
            (0..2).map(move |_| Sample {
                context,
                input: input.clone(),
                actions: [1, 2],
                reward: 0.,
            })
        })
        .collect();
    assert_eq!(l::update(Mode::Graph, &zero, &samples).unwrap(), zero);
    samples[0].reward = 0.375;
    samples[1].reward = 0.375;
    let changed = l::update(Mode::Graph, &zero, &samples).unwrap();
    assert_eq!(&changed[..4], &[-0.125, 0.125, 0.125, -0.125]);
    assert_eq!(changed[7], 0.1875);
    assert_eq!(changed[11], 0.1875);
    assert!(l::update(Mode::Graph, &zero, &samples[..11]).is_err());
    samples[0].context = l::RESERVED[0];
    assert!(l::update(Mode::Graph, &zero, &samples).is_err());
    samples[0].context = l::DEVELOPMENT[0];
    assert!(l::update(Mode::Blind, &zero, &samples).is_err());
    samples[0].reward = f64::NAN;
    assert!(l::update(Mode::Graph, &zero, &samples).is_err());
    samples[0].reward = -0.1;
    assert!(l::update(Mode::Graph, &zero, &samples).is_err());
    samples[0].reward = 0.;
    samples[0].actions = [4, 1];
    assert!(l::update(Mode::Graph, &zero, &samples).is_err());
    assert!(l::endpoint(Mode::Graph, &zero, &[0.; 143]).is_err());
    assert!(l::endpoint(Mode::Graph, &zero, &[f64::NAN; 144]).is_err());
    let input = &samples[1].input;
    assert!(l::probabilities(&theta[..15], input).is_err());
    assert!(l::probabilities(&[f64::NAN; 16], input).is_err());
    assert!(l::sample(&theta, input, [1., 0.]).is_err());
    let mut extreme = [0.; 16];
    extreme[0] = -1e308;
    assert!(l::log_probability(&extreme, input, [0, 0]).is_err());
    let rewards: Vec<_> = native::CONTEXTS
        .into_iter()
        .flat_map(|c| {
            let correct = [
                native::CHOICES[0]
                    .iter()
                    .position(|a| *a == Some(c[0]))
                    .unwrap(),
                native::CHOICES[1]
                    .iter()
                    .position(|a| *a == Some(c[1]))
                    .unwrap(),
            ];
            (0..4)
                .flat_map(move |a| (0..4).map(move |b| if [a, b] == correct { 0.375 } else { 0. }))
        })
        .collect();
    let e = l::endpoint(Mode::Graph, &zero, &rewards).unwrap();
    assert!((e["balanced_expected_reward"].as_f64().unwrap() - 9. / 384.).abs() < 2e-12);
    assert!((e["reserved_expected_reward"].as_f64().unwrap() - 9. / 384.).abs() < 2e-12);
}

#[test]
fn fixed_native_relational_learning_trial() {
    let tmp = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_RULE_RELATIONAL_LEARNING_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| tmp.path().join("trial"));
    let r = l::run(&out).expect("invalid trial retained; no automatic retry");
    assert_eq!(r["recorded_cells"], 1782);
    assert_eq!(r["completed_updates"], 72);
    assert_eq!(r["native_attempts"], 42768);
    assert_eq!(r["scientific_acceptance"], false);
    // Negative learning remains a complete trial, not a test failure.
}
