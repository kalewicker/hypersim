#![cfg(feature = "ere")]
use hg_sim_core::ere::rule_relational::{self as native, Arm};
use hg_train::ere::{rule_composition as blind, rule_relational as policy};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn save(path: PathBuf, v: &Value) {
    std::fs::write(path, serde_json::to_vec(v).unwrap()).unwrap();
}
fn remap(mut v: Value) -> Value {
    for n in v["nodes"].as_array_mut().unwrap() {
        let id = n[0].as_str().unwrap();
        let idx = id
            .strip_prefix("NodeId(")
            .unwrap()
            .split('v')
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        n[0] = json!(format!("NodeId({}v9)", idx + 500));
    }
    for e in v["edges"].as_array_mut().unwrap() {
        for n in e[1]["nodes"].as_array_mut().unwrap() {
            n["idx"] = json!(n["idx"].as_u64().unwrap() + 500);
            n["version"] = json!(9);
        }
    }
    v
}

#[test]
fn graph_observation_gradients_and_refusals() {
    let theta = [
        0.2, -0.3, 0.1, -0.4, 0.7, -0.1, 0.2, 0.8, 0.4, -0.2, -0.3, 0.9, 0.1, 0.2, -0.4, 0.6,
    ];
    let fixed_blind = blind::probabilities(&[0.; 12], [10, 11]).unwrap();
    let mut observed = vec![];
    for context in native::CONTEXTS {
        let raw = native::initial_observation(context).unwrap();
        let o = policy::observe(&raw).unwrap();
        assert_eq!(policy::observe(&remap(raw.clone())).unwrap(), o);
        let mut reordered = raw.clone();
        reordered["nodes"].as_array_mut().unwrap().reverse();
        reordered["edges"].as_array_mut().unwrap().reverse();
        for e in reordered["edges"].as_array_mut().unwrap() {
            e[1]["nodes"].as_array_mut().unwrap().reverse();
        }
        assert_eq!(policy::observe(&reordered).unwrap(), o);
        let mut metadata = raw.clone();
        metadata["nodes"].as_array_mut().unwrap().push(json!(["NodeId(999v1)",{"data":{"label":native::ORDINARY_CEILING+9,"answer":context},"incident":[]} ]));
        assert_eq!(policy::observe(&metadata).unwrap(), o);
        let p = policy::probabilities(&theta, &o).unwrap();
        observed.push(o.clone());
        assert_eq!(
            blind::probabilities(&[0.; 12], [10, 11]).unwrap(),
            fixed_blind
        );
        for a in 0..4 {
            for b in 0..4 {
                let actions = [a, b];
                let gradient = policy::score_gradient(&theta, &o, actions).unwrap();
                for i in 0..16 {
                    let mut hi = theta;
                    let mut lo = theta;
                    hi[i] += 1e-6;
                    lo[i] -= 1e-6;
                    let derivative = (policy::log_probability(&hi, &o, actions).unwrap()
                        - policy::log_probability(&lo, &o, actions).unwrap())
                        / 2e-6;
                    assert!((gradient[i] - derivative).abs() < 3e-9);
                }
                let draws = std::array::from_fn(|slot| {
                    p[slot][..actions[slot]].iter().sum::<f64>() + p[slot][actions[slot]] / 2.
                });
                assert_eq!(policy::sample(&theta, &o, draws).unwrap(), actions);
            }
        }
    }
    for i in 0..observed.len() {
        for j in i + 1..observed.len() {
            assert_ne!(observed[i], observed[j]);
        }
    }
    let o = &observed[0];
    assert!(policy::probabilities(&theta[..15], o).is_err());
    assert!(policy::probabilities(&[f64::NAN; 16], o).is_err());
    assert!(policy::sample(&theta, o, [1., 0.]).is_err());
    assert!(policy::sample(&theta, o, [0., f64::NAN]).is_err());
    assert!(policy::log_probability(&theta, o, [4, 0]).is_err());
    let mut extreme = [0.; 16];
    extreme[0] = -1e308;
    assert!(policy::log_probability(&extreme, o, [0, 0]).is_err());
    assert!(policy::score_gradient(&extreme, o, [0, 0]).is_err());
    let raw = native::initial_observation([10, 11]).unwrap();
    let mut bad = raw.clone();
    bad["step"] = json!(1);
    assert!(policy::observe(&bad).is_err());
    bad = raw.clone();
    bad["edges"].as_array_mut().unwrap().pop();
    assert!(policy::observe(&bad).is_err());
    bad = raw.clone();
    bad["edges"][0][1]["label"] = json!(1);
    assert!(policy::observe(&bad).is_err());
    bad = raw.clone();
    bad["edges"][0][1]["nodes"][1] = bad["edges"][0][1]["nodes"][0].clone();
    assert!(policy::observe(&bad).is_err());
    bad = raw.clone();
    bad["nodes"][0][1]["data"]["label"] = json!(99);
    assert!(policy::observe(&bad).is_err());
    bad = raw.clone();
    let duplicate = bad["nodes"][0].clone();
    bad["nodes"].as_array_mut().unwrap().push(duplicate);
    assert!(policy::observe(&bad).is_err());
    assert!(policy::observe(&json!({})).is_err());
}

#[test]
fn registered_native_relational_panel() {
    let tmp = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_RULE_RELATIONAL_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| tmp.path().join("panel"));
    std::fs::create_dir(&out).expect("fresh output required");
    let design: Value =
        serde_json::from_str(include_str!("../../../configs/ere_rule_relational_v1.json")).unwrap();
    assert_eq!(design["contexts"], json!(native::CONTEXTS));
    assert_eq!(design["seeds"], json!(native::SEEDS));
    assert_eq!(design["ordinary_labels"], json!(native::ORDINARY_LABELS));
    assert_eq!(design["maximum_native_attempts"], 9504);
    save(
        out.join("manifest.json"),
        &json!({"status":"running","design":design,"training_updates":0,"training_admitted":false}),
    );
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../docs/ere_rule_relational_v1.md"),
    )
    .unwrap();
    if std::env::var_os("ERE_RULE_RELATIONAL_OUTPUT").is_some() {
        std::fs::copy(
            std::env::current_exe().unwrap(),
            out.join("executed_test_binary"),
        )
        .unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut failures = vec![];
    let mut compact = vec![];
    let mut worlds = vec![];
    let theta = [
        0.2, -0.3, 0.1, -0.4, 0.7, -0.1, 0.2, 0.8, 0.4, -0.2, -0.3, 0.9, 0.1, 0.2, -0.4, 0.6,
    ];
    let blind_theta = [
        0.2, -0.3, 0.8, -1.1, 1.0, 0.0, -0.5, 0.7, -0.2, 0.3, 0.1, 0.6,
    ];
    for seed in native::SEEDS {
        for context in native::CONTEXTS {
            let correct = std::array::from_fn(|i| {
                native::CHOICES[i]
                    .iter()
                    .position(|x| *x == Some(context[i]))
                    .unwrap()
            });
            let mut schedule: Vec<_> = (0..4)
                .flat_map(|a| (0..4).map(move |b| (Arm::LivePaid, [a, b])))
                .collect();
            schedule.extend(native::ARMS[1..].iter().map(|a| (*a, correct)));
            let mut rewards = vec![];
            let mut observation = None;
            for (arm, actions) in schedule {
                let mut row = match native::run(arm, context, actions, seed, deadline) {
                    Ok(r) => r,
                    Err(e) => {
                        failures.push(e.clone());
                        json!({"error":e,"partial_native_work_unknown":true,"seed":seed,"context":context,"actions":actions,"arm":arm})
                    }
                };
                if row.get("error").is_none() {
                    match policy::observe(&row["initial_inventory"]) {
                        Ok(o) => {
                            row["encoded_observation"] = json!(o);
                            observation = Some(o);
                        }
                        Err(e) => {
                            failures.push(e.clone());
                            row["observation_error"] = json!(e);
                        }
                    }
                    match policy::reward(&row) {
                        Ok(r) => row["adapter_reward"] = json!(r),
                        Err(e) => {
                            failures.push(e.clone());
                            row["adapter_error"] = json!(e);
                        }
                    }
                }
                let name = serde_json::to_value(arm).unwrap();
                save(
                    out.join(format!(
                        "{seed}-{}-{}-{}-{}-{}.json",
                        context[0],
                        context[1],
                        name.as_str().unwrap(),
                        actions[0],
                        actions[1]
                    )),
                    &row,
                );
                compact.push(json!({"arm":arm,"actions":actions,"correct":correct,"reward":row["adapter_reward"]}));
                if arm == Arm::LivePaid {
                    rewards.push(row["adapter_reward"].clone());
                }
            }
            if let Some(o) = observation {
                let p = policy::probabilities(&theta, &o).unwrap();
                let actions:Vec<_>=(0..4).flat_map(|a|(0..4).map(move |b|[a,b])).map(|actions|{
                let draws=std::array::from_fn(|i|p[i][..actions[i]].iter().sum::<f64>()+p[i][actions[i]]/2.);
                json!({"actions":actions,"sample_midpoints":policy::sample(&theta,&o,draws).unwrap(),"log_probability":policy::log_probability(&theta,&o,actions).unwrap(),"score_gradient":policy::score_gradient(&theta,&o,actions).unwrap().to_vec()})
            }).collect();
                let graph_probe = policy::probabilities(&policy::probe_theta(), &o).unwrap();
                let uniform = policy::probabilities(&[0.; 16], &o).unwrap();
                let category = blind::probabilities(&blind_theta, [10, 11]).unwrap();
                let expected = |p: [[f64; 4]; 2]| -> f64 {
                    (0..4)
                        .flat_map(|a| (0..4).map(move |b| (a, b)))
                        .map(|(a, b)| p[0][a] * p[1][b] * rewards[a * 4 + b].as_f64().unwrap())
                        .sum()
                };
                worlds.push(json!({"seed":seed,"context":context,"observation":o,"probabilities":p,"actions":actions,
                "graph_probe_probabilities":graph_probe,"uniform_probabilities":uniform,"category_only_probabilities":category,
                "graph_probe_expected_reward":expected(graph_probe),"uniform_expected_reward":expected(uniform),"category_only_expected_reward":expected(category)}));
            }
        }
    }
    save(
        out.join("manifest.json"),
        &json!({"status":if failures.is_empty(){"native_complete"}else{"failed"},"design":design,"recorded_cells":compact.len(),"failures":failures,"training_updates":0,"training_admitted":false}),
    );
    assert!(
        failures.is_empty(),
        "native failures retained: {failures:?}"
    );
    assert_eq!(compact.len(), 396);
    for r in &compact {
        let productive = matches!(r["arm"].as_str(), Some("live_paid" | "reverse_paid"))
            && r["actions"] == r["correct"];
        assert_eq!(
            r["reward"].as_f64().unwrap(),
            if productive { 9. / 24. } else { 0. }
        );
    }
    let graph_mean = worlds
        .iter()
        .map(|r| r["graph_probe_expected_reward"].as_f64().unwrap() / 18.)
        .sum::<f64>();
    let blind_mean = worlds
        .iter()
        .map(|r| r["category_only_expected_reward"].as_f64().unwrap() / 18.)
        .sum::<f64>();
    assert!(graph_mean > 0.30);
    assert!(blind_mean <= 1. / 24.);
    save(
        out.join("policy.json"),
        &json!({"theta":theta,"probe_theta":policy::probe_theta().to_vec(),"category_only_theta":blind_theta,
        "category_only_cue":[10,11],"worlds":worlds,"graph_probe_mean_reward":graph_mean,"category_only_mean_reward":blind_mean,
        "category_only_upper_bound":1./24.,"training_updates":0,"training_admitted":false,"scientific_acceptance":false}),
    );
    save(
        out.join("native_assertions.json"),
        &json!({"passed":true,"cells":396,"native_attempts":9504,"advancing_steps":9504,"training_updates":0,"scientific_acceptance":false}),
    );
}
