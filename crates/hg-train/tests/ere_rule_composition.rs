#![cfg(feature = "ere")]
use hg_sim_core::ere::{
    lifecycle_fixture,
    rule_composition::{self as native, Arm},
};
use hg_train::ere::rule_composition as adapter;
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn save(path: PathBuf, value: &Value) {
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

#[test]
fn shared_policy_joint_gradients_and_refusals() {
    let theta = [
        0.2, -0.3, 0.8, -1.1, 1.0, 0.0, -0.5, 0.7, -0.2, 0.3, 0.1, 0.6,
    ];
    for context in native::CONTEXTS {
        let p = adapter::probabilities(&theta, context).unwrap();
        for a in 0..4 {
            for b in 0..4 {
                let actions = [a, b];
                let gradient = adapter::score_gradient(&theta, context, actions).unwrap();
                for i in 0..12 {
                    let mut hi = theta;
                    let mut lo = theta;
                    hi[i] += 1e-6;
                    lo[i] -= 1e-6;
                    let derivative = (adapter::log_probability(&hi, context, actions).unwrap()
                        - adapter::log_probability(&lo, context, actions).unwrap())
                        / 2e-6;
                    assert!((gradient[i] - derivative).abs() < 2e-9);
                }
                let draws = [
                    p[0][..a].iter().sum::<f64>() + p[0][a] / 2.,
                    p[1][..b].iter().sum::<f64>() + p[1][b] / 2.,
                ];
                assert_eq!(adapter::sample(&theta, context, draws).unwrap(), actions);
            }
        }
    }
    let diagonal = adapter::probabilities(&theta, [20, 21]).unwrap();
    assert_eq!(diagonal[0], diagonal[1]);
    let first = adapter::probabilities(&theta, [20, 11]).unwrap();
    let second = adapter::probabilities(&theta, [10, 21]).unwrap();
    assert_eq!(first[0], second[1]);
    assert!(adapter::probabilities(&theta, [99, 11]).is_err());
    assert!(adapter::probabilities(&theta[..11], [10, 11]).is_err());
    assert!(adapter::sample(&theta, [10, 11], [0., 1.]).is_err());
    assert!(adapter::sample(&theta, [10, 11], [f64::NAN, 0.]).is_err());
    assert!(adapter::log_probability(&theta, [10, 11], [1, 4]).is_err());
    let mut extreme = [0.; 12];
    extreme[0] = -1e308;
    assert!(adapter::log_probability(&extreme, [10, 11], [0, 0]).is_err());
    assert!(adapter::score_gradient(&extreme, [10, 11], [0, 0]).is_err());
    assert!(adapter::reward(&json!({})).is_err());
    assert!(native::run(
        Arm::LivePaid,
        [99, 11],
        [1, 1],
        native::SEEDS[0],
        Instant::now()
    )
    .is_err());
    assert!(native::run(
        Arm::LivePaid,
        [10, 11],
        [1, 4],
        native::SEEDS[0],
        Instant::now()
    )
    .is_err());
    assert!(native::run(Arm::LivePaid, [10, 11], [1, 1], 0, Instant::now()).is_err());
}

#[test]
fn registered_native_composition_panel() {
    let tmp = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_RULE_COMPOSITION_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| tmp.path().join("panel"));
    std::fs::create_dir(&out).expect("fresh composition output required");
    let design: Value = serde_json::from_str(include_str!(
        "../../../configs/ere_rule_composition_v1.json"
    ))
    .unwrap();
    assert_eq!(design["contexts"], json!(native::CONTEXTS));
    assert_eq!(design["choices"], json!(native::CHOICES));
    assert_eq!(design["seeds"], json!(native::SEEDS));
    assert_eq!(design["arms"], json!(native::ARMS));
    assert_eq!(design["maximum_native_attempts"], 9504);
    save(
        out.join("manifest.json"),
        &json!({"status":"running","design":design,"training_admitted":false}),
    );
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../docs/ere_rule_composition_v1.md"),
    )
    .unwrap();
    if std::env::var_os("ERE_RULE_COMPOSITION_OUTPUT").is_some() {
        std::fs::copy(
            std::env::current_exe().unwrap(),
            out.join("executed_test_binary"),
        )
        .unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut failures = vec![];
    let mut observations = vec![];
    for seed in native::SEEDS {
        for context in native::CONTEXTS {
            let correct = std::array::from_fn(|i| {
                native::CHOICES[i]
                    .iter()
                    .position(|a| *a == Some(context[i]))
                    .unwrap()
            });
            let mut cells: Vec<_> = (0..4)
                .flat_map(|a| (0..4).map(move |b| (Arm::LivePaid, [a, b])))
                .collect();
            cells.extend(native::ARMS[1..].iter().map(|arm| (*arm, correct)));
            for (arm, actions) in cells {
                let mut row = match native::run(arm, context, actions, seed, deadline) {
                    Ok(r) => r,
                    Err(error) => {
                        failures.push(error.clone());
                        json!({"seed":seed,"context":context,"actions":actions,
                    "arm":arm,"error":error,"partial_native_work_unknown":true})
                    }
                };
                if row.get("error").is_none() {
                    match adapter::reward(&row) {
                        Ok(r) => row["adapter_reward"] = json!(r),
                        Err(error) => {
                            failures.push(error.clone());
                            row["adapter_error"] = json!(error);
                        }
                    }
                }
                let arm_name = serde_json::to_value(arm).unwrap();
                save(
                    out.join(format!(
                        "{seed}-{}-{}-{}-{}-{}.json",
                        context[0],
                        context[1],
                        arm_name.as_str().unwrap(),
                        actions[0],
                        actions[1]
                    )),
                    &row,
                );
                // Keep only compact observations, not hundreds of full graph journals.
                observations.push(json!({"arm":arm,"context":context,"actions":actions,"correct":correct,
                "completed_steps":row["completed_steps"],"summary":row["summary"],"reward":row["adapter_reward"]}));
                if row.get("error").is_none()
                    && seed == native::SEEDS[0]
                    && context == [10, 11]
                    && actions == [1, 1]
                {
                    for field in [
                        "timeline",
                        "summary",
                        "initial_all_candidates",
                        "ordinary_nodes",
                    ] {
                        let mut bad = row.clone();
                        bad[field] = Value::Null;
                        assert!(adapter::reward(&bad).is_err());
                    }
                    let mut bad = row.clone();
                    bad["timeline"][7]["usage"]["truncated"] = json!(true);
                    assert!(adapter::reward(&bad).is_err());
                    bad = row.clone();
                    bad["timeline"][24]["installed"] = json!([]);
                    assert!(adapter::reward(&bad).is_err());
                }
            }
        }
    }
    // All scheduled outcomes are retained before asserting the predicted causal matrix.
    save(
        out.join("manifest.json"),
        &json!({"status":if failures.is_empty(){"native_complete"}else{"failed"},
        "design":design,"recorded_cells":observations.len(),"failures":failures,"training_admitted":false}),
    );
    assert!(failures.is_empty(), "composition errors: {failures:?}");
    assert_eq!(observations.len(), 396);
    let mut credited = 0.;
    for r in &observations {
        assert_eq!(r["completed_steps"], 24);
        let productive = matches!(r["arm"].as_str(), Some("live_paid" | "reverse_paid"))
            && r["actions"] == r["correct"];
        let expected = if productive { 9. / 24. } else { 0. };
        assert_eq!(r["reward"].as_f64().unwrap(), expected);
        credited += expected * 24.;
        if productive {
            let classes = r["summary"]["novel_classes"].as_array().unwrap();
            assert_eq!(classes.len(), 2);
            let partial = classes.iter().find(|c| c["first_birth"] == 1).unwrap();
            let complete = classes.iter().find(|c| c["first_birth"] == 5).unwrap();
            assert_eq!(partial["genotype_spells"], json!([{"start":1,"end":5}]));
            assert_eq!(partial["installed_spells"], json!([{"start":2,"end":6}]));
            assert_eq!(partial["non_meta_fire_steps"], json!([]));
            assert_eq!(complete["first_installation"], 6);
            assert_eq!(complete["first_non_meta_fire"], 7);
            assert_eq!(
                complete["non_meta_fire_steps"],
                json!([7, 9, 11, 13, 15, 17, 19, 21, 23])
            );
            assert_eq!(r["summary"]["mean_edit_interarrival"], 4.);
            assert_eq!(complete["genotype_spells"], json!([{"start":5,"end":null}]));
        }
    }
    assert_eq!(credited, 324.);
    let theta = vec![
        0.2, -0.3, 0.8, -1.1, 1.0, 0.0, -0.5, 0.7, -0.2, 0.3, 0.1, 0.6,
    ];
    let contexts:Vec<_>=native::CONTEXTS.iter().map(|&context| {
        let p=adapter::probabilities(&theta,context).unwrap();
        let actions:Vec<_>=(0..4).flat_map(|a|(0..4).map(move |b|[a,b])).map(|actions| {
            let draws=std::array::from_fn(|i|p[i][..actions[i]].iter().sum::<f64>()+p[i][actions[i]]/2.);
            json!({"actions":actions,"sample_midpoints":adapter::sample(&theta,context,draws).unwrap(),
                "log_probability":adapter::log_probability(&theta,context,actions).unwrap(),
                "score_gradient":adapter::score_gradient(&theta,context,actions).unwrap().to_vec()})
        }).collect();json!({"context":context,"probabilities":p,"actions":actions})
    }).collect();
    save(
        out.join("policy.json"),
        &json!({"theta":theta,"contexts":contexts,"training_updates":0,"training_admitted":false}),
    );
    save(
        out.join("native_assertions.json"),
        &json!({"passed":true,"cells":396,"native_attempts":9504,
        "advancing_steps":9504,"training_updates":0,"scientific_acceptance":false}),
    );
    // Keep structural-only initial exclusion checked via the unchanged helper.
    assert_eq!(
        lifecycle_fixture::summarize(&[], &[])["scientific_acceptance"],
        false
    );
}
