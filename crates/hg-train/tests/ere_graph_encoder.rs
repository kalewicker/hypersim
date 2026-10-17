#![cfg(feature = "ere")]
use e::{Mode, Sample};
use hg_sim_core::ere::rule_relational as native;
use hg_train::ere::{graph_encoder as e, rule_relational_learning as previous};
use serde_json::{json, Value};
use std::path::Path;

fn raw() -> Value {
    native::initial_observation([10, 11]).unwrap()
}
fn reordered(mut v: Value) -> Value {
    v["nodes"].as_array_mut().unwrap().reverse();
    v["edges"].as_array_mut().unwrap().reverse();
    for row in v["edges"].as_array_mut().unwrap() {
        row[1]["nodes"].as_array_mut().unwrap().reverse();
    }
    v
}
fn remap(mut v: Value) -> Value {
    for row in v["nodes"].as_array_mut().unwrap() {
        let s = row[0].as_str().unwrap();
        let i = s[7..s.find('v').unwrap()].parse::<u64>().unwrap();
        row[0] = json!(format!("NodeId({}v9)", i + 500));
    }
    for row in v["edges"].as_array_mut().unwrap() {
        for n in row[1]["nodes"].as_array_mut().unwrap() {
            n["idx"] = json!(n["idx"].as_u64().unwrap() + 500);
            n["version"] = json!(9);
        }
    }
    v
}
fn correct(c: [u32; 2]) -> [usize; 2] {
    std::array::from_fn(|s| {
        native::CHOICES[s]
            .iter()
            .position(|v| *v == Some(c[s]))
            .unwrap()
    })
}
fn batch(mode: Mode) -> Vec<Sample> {
    previous::DEVELOPMENT
        .into_iter()
        .flat_map(|c| {
            let o = e::observe(&native::initial_observation(c).unwrap()).unwrap();
            let input = e::input(mode, &o);
            (0..2).map(move |_| Sample {
                context: c,
                input: input.clone(),
                actions: correct(c),
                reward: 0.375,
            })
        })
        .collect()
}
fn variant(mut raw: Value, c: [u32; 2], name: &str) -> Value {
    let labels: Vec<_> = native::CHOICES[0][1..]
        .iter()
        .chain(native::CHOICES[1][1..].iter())
        .filter_map(|l| *l)
        .filter(|l| !c.contains(l))
        .collect();
    let ids:Vec<_>=labels.iter().map(|l|{
        let row=raw["nodes"].as_array().unwrap().iter().find(|r|r[1]["data"]["label"]==*l).unwrap();
        let s=row[0].as_str().unwrap();json!({"idx":s[7..s.find('v').unwrap()].parse::<u64>().unwrap(),"version":s[s.find('v').unwrap()+1..s.len()-1].parse::<u64>().unwrap()})
    }).collect();
    let edges = match name {
        "base" => vec![],
        "lure_edge" => vec![(0, 1)],
        "lure_chain" => vec![(0, 1), (1, 2)],
        _ => panic!("unknown variant"),
    };
    for (i, (a, b)) in edges.into_iter().enumerate() {
        raw["edges"]
            .as_array_mut()
            .unwrap()
            .push(json!([format!("EdgeId({}v1)",999001+i),{"label":0,"nodes":[ids[a],ids[b]]}]));
    }
    raw
}
fn save(path: &Path, v: &Value) {
    std::fs::write(path, serde_json::to_vec(v).unwrap()).unwrap();
}

#[test]
fn arithmetic_invariance_and_refusals() {
    let theta = e::initialize();
    assert_eq!(theta.len(), 110);
    assert_eq!(theta, e::initialize());
    let mut blind_reference = None;
    let mut observed = vec![];
    for c in native::CONTEXTS {
        let raw = native::initial_observation(c).unwrap();
        let o = e::observe(&raw).unwrap();
        assert_eq!(e::observe(&reordered(raw.clone())).unwrap(), o);
        assert_eq!(e::observe(&remap(raw.clone())).unwrap(), o);
        let mut metadata = raw.clone();
        metadata["nodes"].as_array_mut().unwrap().push(
            json!(["NodeId(999999v1)",{"data":{"label":native::ORDINARY_CEILING+9,"answer":c}}]),
        );
        metadata["context"] = json!([30, 31]);
        metadata["features"] = json!([[0, 0, 1], [0, 0, 1]]);
        assert_eq!(e::observe(&metadata).unwrap(), o);
        observed.push(o.clone());
        for mode in [Mode::Graph, Mode::Blind] {
            let inp = e::input(mode, &o);
            let p = e::probabilities(&theta, &inp).unwrap();
            if mode == Mode::Blind {
                if let Some(reference) = blind_reference {
                    assert_eq!(reference, p);
                } else {
                    blind_reference = Some(p);
                }
            }
            assert_eq!(e::probabilities(&[0.; 110], &inp).unwrap(), [[0.25; 4]; 2]);
            for a in 0..4 {
                for b in 0..4 {
                    let actions = [a, b];
                    let g = e::score_gradient(&theta, &inp, actions).unwrap();
                    for i in 0..110 {
                        let mut hi = theta.clone();
                        let mut lo = theta.clone();
                        hi[i] += 1e-6;
                        lo[i] -= 1e-6;
                        let fd = (e::log_probability(&hi, &inp, actions).unwrap()
                            - e::log_probability(&lo, &inp, actions).unwrap())
                            / 2e-6;
                        assert!(
                            (g[i] - fd).abs() < 2e-8,
                            "{c:?} {mode:?} {actions:?} parameter{i}"
                        );
                    }
                    let draws = std::array::from_fn(|s| {
                        p[s][..actions[s]].iter().sum::<f64>() + p[s][actions[s]] / 2.
                    });
                    assert_eq!(e::sample(&theta, &inp, draws).unwrap(), actions);
                    let zero = e::score_gradient(&[0.; 110], &inp, actions).unwrap();
                    assert!(zero[..108].iter().all(|g| *g == 0.));
                }
            }
        }
    }
    for i in 0..9 {
        for j in i + 1..9 {
            assert_ne!(observed[i], observed[j]);
        }
    }
    let mut empty = raw();
    empty["edges"] = json!([]);
    assert!(e::observe(&empty).is_ok());
    let inp = e::input(Mode::Graph, &observed[0]);
    assert!(e::probabilities(&theta[..109], &inp).is_err());
    assert!(e::probabilities(&[f64::NAN; 110], &inp).is_err());
    assert!(e::sample(&theta, &inp, [1., 0.]).is_err());
    assert!(e::log_probability(&theta, &inp, [4, 0]).is_err());
    let mut overflow = vec![1e308; 110];
    overflow[28] = 1e308;
    assert!(e::probabilities(&overflow, &inp).is_err());
    let original = raw();
    let mut bad = original.clone();
    bad["step"] = json!(1);
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    bad["nodes"].as_array_mut().unwrap().pop();
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    let row = bad["nodes"][0].clone();
    bad["nodes"].as_array_mut().unwrap().push(row);
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    bad["nodes"][0][1]["data"]["label"] = json!(99);
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    bad["edges"][0][1]["label"] = json!(1);
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    bad["edges"][0][1]["nodes"][1] = bad["edges"][0][1]["nodes"][0].clone();
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    let row = bad["edges"][0].clone();
    bad["edges"].as_array_mut().unwrap().push(row);
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    bad["edges"][0][1]["nodes"][0]["idx"] = json!(999999);
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    bad["edges"][0][1]["nodes"][0]["version"] = json!(0);
    assert!(e::observe(&bad).is_err());
    bad = original.clone();
    bad["nodes"][0][0] = json!("bad");
    assert!(e::observe(&bad).is_err());
    assert!(e::observe(&json!({})).is_err());
}

#[test]
fn balanced_update_diagnostic_and_refusals() {
    let theta = e::initialize();
    for mode in [Mode::Graph, Mode::Blind] {
        let mut samples = batch(mode);
        let next = e::update(mode, &theta, &samples, 0.001).unwrap();
        let before: f64 = samples
            .iter()
            .map(|s| e::log_probability(&theta, &s.input, s.actions).unwrap())
            .sum();
        let after: f64 = samples
            .iter()
            .map(|s| e::log_probability(&next, &s.input, s.actions).unwrap())
            .sum();
        assert!(after > before);
        if mode == Mode::Graph {
            assert!(theta[..104].iter().zip(&next[..104]).any(|(a, b)| a != b));
        }
        for s in &mut samples {
            s.reward = 0.;
        }
        assert_eq!(e::update(mode, &theta, &samples, 0.001).unwrap(), theta);
        assert!(e::update(mode, &theta, &samples[..11], 0.001).is_err());
        assert!(e::update(mode, &theta, &samples, 0.).is_err());
        samples[0].context = previous::RESERVED[0];
        assert!(e::update(mode, &theta, &samples, 0.001).is_err());
        samples[0].context = previous::DEVELOPMENT[0];
        samples[0].reward = f64::NAN;
        assert!(e::update(mode, &theta, &samples, 0.001).is_err());
        samples[0].reward = 0.;
        samples[0].input = e::input(
            if mode == Mode::Graph {
                Mode::Blind
            } else {
                Mode::Graph
            },
            &e::observe(&raw()).unwrap(),
        );
        assert!(e::update(mode, &theta, &samples, 0.001).is_err());
    }
}

#[test]
fn saved_graph_engineering_panel() {
    let Some(out) = std::env::var_os("ERE_GRAPH_ENCODER_OUTPUT") else {
        return;
    };
    let out = Path::new(&out);
    let theta = e::initialize();
    let mut records = vec![];
    let mut diagnostics = vec![];
    for (world, c) in native::CONTEXTS.into_iter().enumerate() {
        let raw: Value = serde_json::from_slice(
            &std::fs::read(out.join(format!("inputs/world-{world}.json"))).unwrap(),
        )
        .unwrap();
        for name in ["base", "lure_edge", "lure_chain"] {
            let raw = variant(raw.clone(), c, name);
            let observation = e::observe(&raw).unwrap();
            assert_eq!(e::observe(&reordered(raw.clone())).unwrap(), observation);
            assert_eq!(e::observe(&remap(raw.clone())).unwrap(), observation);
            for mode in [Mode::Graph, Mode::Blind] {
                let inp = e::input(mode, &observation);
                let p = e::probabilities(&theta, &inp).unwrap();
                let mut actions = vec![];
                for a in 0..4 {
                    for b in 0..4 {
                        let action = [a, b];
                        let draws = std::array::from_fn(|s| {
                            p[s][..action[s]].iter().sum::<f64>() + p[s][action[s]] / 2.
                        });
                        actions.push(json!({"actions":action,"draws":draws,"sampled_actions":e::sample(&theta,&inp,draws).unwrap(),"log_probability":e::log_probability(&theta,&inp,action).unwrap(),"score_gradient":e::score_gradient(&theta,&inp,action).unwrap()}));
                    }
                }
                records.push(json!({"world":world,"context":c,"variant":name,"mode":mode,"raw":raw,"observation":observation,"input":inp,"probabilities":p,"actions":actions}));
            }
        }
    }
    for mode in [Mode::Graph, Mode::Blind] {
        let samples: Vec<_> = records
            .iter()
            .filter(|r| {
                r["variant"] == "base"
                    && r["mode"] == json!(mode)
                    && previous::DEVELOPMENT
                        .iter()
                        .any(|c| r["context"] == json!(c))
            })
            .flat_map(|r| {
                let c: [u32; 2] = serde_json::from_value(r["context"].clone()).unwrap();
                let observation = e::observe(&r["raw"]).unwrap();
                let inp = e::input(mode, &observation);
                (0..2).map(move |_| Sample {
                    context: c,
                    input: inp.clone(),
                    actions: correct(c),
                    reward: 0.375,
                })
            })
            .collect();
        let next = e::update(mode, &theta, &samples, 0.001).unwrap();
        let before: f64 = samples
            .iter()
            .map(|s| e::log_probability(&theta, &s.input, s.actions).unwrap() / 12.)
            .sum();
        let after: f64 = samples
            .iter()
            .map(|s| e::log_probability(&next, &s.input, s.actions).unwrap() / 12.)
            .sum();
        diagnostics.push(json!({"mode":mode,"before":theta,"after":next,"mean_log_probability_before":before,"mean_log_probability_after":after,"synthetic_teacher_selected":true,"native_steps":0}));
    }
    save(
        &out.join("encoder.json"),
        &json!({"profile":e::PROFILE,"theta":theta,"records":records,"diagnostics":diagnostics,"native_steps":0,"training_campaign_launched":false,"scientific_acceptance":false}),
    );
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .unwrap();
}
