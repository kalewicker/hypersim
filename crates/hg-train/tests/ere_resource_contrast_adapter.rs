#![cfg(feature = "ere")]
use hg_train::ere::resource_contrast_adapter as e;
use serde_json::{json, Value};

fn raw(w: f64, f: i64, b: i64) -> Value {
    json!({"step":13,"nodes":[
        ["NodeId(1v1)",{"data":{"label":4293949189u64,"properties":[[1,{"Float":w}]]},"incident":[]}],
        ["NodeId(2v1)",{"data":{"label":4293949188u64,"properties":[[30468,{"Int":0}],[30469,{"Int":f}]]},"incident":[]}],
        ["NodeId(3v1)",{"data":{"label":4293949190u64,"properties":[[30470,{"Int":b}],[30471,{"Int":0}]]},"incident":[]}]
    ],"edges":[]})
}
#[test]
fn features_masks_and_invariance() {
    for w in [0., 1.] {
        for f in [0, 1] {
            for b in [2, 12] {
                let v = raw(w, f, b);
                let o = e::observe(&v).unwrap();
                let sf = 2. * f as f64 - 1.;
                let sb = 2. * (b - 2) as f64 / 10. - 1.;
                assert_eq!(e::input(e::Mode::Full, &o).unwrap(), [1., sf, sb, sf * sb]);
                assert_eq!(
                    e::input(e::Mode::ForecastMasked, &o).unwrap(),
                    [1., 0., sb, 0.]
                );
                assert_eq!(
                    e::input(e::Mode::BountyMasked, &o).unwrap(),
                    [1., sf, 0., 0.]
                );
                assert_eq!(e::input(e::Mode::CuesMasked, &o).unwrap(), [1., 0., 0., 0.]);
                let opposite_wallet = e::observe(&raw(1. - w, f, b)).unwrap();
                for mode in e::MODES {
                    assert_eq!(
                        e::input(mode, &o).unwrap(),
                        e::input(mode, &opposite_wallet).unwrap()
                    );
                    let original = e::choose(&e::PROBES[1], mode, &v, 0.25).unwrap().1;
                    let mut changed = v.clone();
                    changed["nodes"].as_array_mut().unwrap().reverse();
                    for (i, node) in changed["nodes"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .enumerate()
                    {
                        node[0] = json!(format!("NodeId({}v7)", 100 + i));
                        node[1]["data"]["properties"]
                            .as_array_mut()
                            .unwrap()
                            .reverse();
                    }
                    changed["scenario"] = json!("wrong");
                    changed["utility"] = json!(999);
                    changed["action"] = json!("harvest");
                    assert_eq!(
                        e::choose(&e::PROBES[1], mode, &changed, 0.25).unwrap().1,
                        original
                    );
                }
            }
        }
    }
}
#[test]
fn funding_is_not_actor_coordinate() {
    for f in [0, 1] {
        for b in [2, 12] {
            for mode in e::MODES {
                for w in [0., 1.] {
                    let v = raw(w, f, b);
                    let o = e::observe(&v).unwrap();
                    assert_eq!(e::funded(&o).unwrap(), w == 1.);
                    let g = e::contribution(&e::initialize(), mode, &o, 0, 20.).unwrap();
                    assert_eq!(g == [0.; 4], w == 0.);
                    for draw in [0.25, 0.75] {
                        let (decision, _) = e::choose(&e::initialize(), mode, &v, draw).unwrap();
                        assert_eq!(decision.step, 13);
                        assert_eq!(decision.request.is_some(), w == 1.);
                    }
                }
            }
        }
    }
    let paid = e::observe(&raw(1., 0, 2)).unwrap();
    let x = e::input(e::Mode::CuesMasked, &paid).unwrap();
    assert_eq!(x[1], 0.);
    assert_ne!(
        e::contribution(&e::initialize(), e::Mode::CuesMasked, &paid, 0, 8.).unwrap(),
        [0.; 4]
    );
}
#[test]
fn domains_and_refusals() {
    let x = [1., 1., -1., -1.];
    for t in [[8.1, 0., 0., 0.], [f64::NAN, 0., 0., 0.]] {
        assert!(e::probabilities(&t, &x).is_err());
    }
    for t in [[8.; 4], [-8.; 4]] {
        assert!(e::probabilities(&t, &[1.; 4])
            .unwrap()
            .iter()
            .all(|v| *v > 0. && *v < 1.));
    }
    for x in [
        [0., 1., 1., 1.],
        [1., 0.5, 1., 0.5],
        [1., 1., -1., 1.],
        [1., f64::NAN, 0., 0.],
    ] {
        assert!(e::probabilities(&e::initialize(), &x).is_err());
    }
    for draw in [-0.1, 1., f64::NAN] {
        assert!(e::sample(&e::initialize(), &x, draw).is_err());
    }
    assert!(e::score_gradient(&e::initialize(), &x, 2).is_err());
    let o = e::observe(&raw(0., 1, 2)).unwrap();
    for u in [-1., 21., f64::INFINITY] {
        assert!(e::contribution(&e::initialize(), e::Mode::Full, &o, 0, u).is_err());
    }
    let mut cases = vec![raw(2., 1, 2), raw(1., 2, 2), raw(1., 1, 3)];
    let mut v = raw(1., 1, 2);
    v["step"] = json!(14);
    cases.push(v);
    let mut v = raw(1., 1, 2);
    v["nodes"].as_array_mut().unwrap().pop();
    cases.push(v);
    let mut v = raw(1., 1, 2);
    let row = v["nodes"][0].clone();
    v["nodes"].as_array_mut().unwrap().push(row);
    cases.push(v);
    let mut v = raw(1., 1, 2);
    v["nodes"][0][1]["incident"] = json!(["EdgeId(1v1)"]);
    cases.push(v);
    let mut v = raw(1., 1, 2);
    v["nodes"][1][1]["data"]["properties"][0][1] = json!({"Int":1});
    cases.push(v);
    let mut v = raw(1., 1, 2);
    v["nodes"][2][1]["data"]["properties"][1][1] = json!({"Int":2});
    cases.push(v);
    for v in cases {
        assert!(e::choose(&e::initialize(), e::Mode::CuesMasked, &v, 0.25).is_err());
    }
}
#[test]
fn orthogonal_full_inputs() {
    let xs: Vec<_> = [0, 1]
        .into_iter()
        .flat_map(|f| {
            [2, 12].into_iter().map(move |b| {
                e::input(e::Mode::Full, &e::observe(&raw(1., f, b)).unwrap()).unwrap()
            })
        })
        .collect();
    for (i, x) in xs.iter().enumerate() {
        for (j, y) in xs.iter().enumerate() {
            assert_eq!(
                x.iter().zip(y).map(|(a, b)| a * b).sum::<f64>(),
                if i == j { 4. } else { 0. }
            );
        }
    }
}
#[test]
#[ignore = "explicit saved-input arithmetic; zero native work and no parameter updates"]
fn saved_contrast_fixture() {
    let root = std::path::PathBuf::from(std::env::var("ERE_RESOURCE_CONTRAST_PREFLIGHT").unwrap());
    let load = |name: &str| -> Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let history = load("inputs/allocation_audit.json");
    let mut states = vec![];
    let mut records = vec![];
    for state in 0..8 {
        let pair = [
            load(&format!("inputs/cell-{:04}.json", state * 2)),
            load(&format!("inputs/cell-{:04}.json", state * 2 + 1)),
        ];
        assert_eq!(
            pair[0]["allocation_observation"],
            pair[1]["allocation_observation"]
        );
        let raw = pair[0]["allocation_observation"]["inventory"].clone();
        let o = e::observe(&raw).unwrap();
        let mut utility = [0.; 2];
        for action in 0..2 {
            let r = &pair[action];
            let h = &history["cells"][state * 2 + action];
            let fires = r["timeline"]
                .as_array()
                .unwrap()
                .iter()
                .skip(17)
                .flat_map(|row| row["events"].as_array().unwrap())
                .filter(|event| {
                    event["kind"] == "world" && event["rcf"] == h["measured"]["identity"]
                })
                .count();
            let payoff: Vec<_> = r["final_inventory"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row[1]["data"]["label"] == 4293949190u64)
                .collect();
            assert_eq!(payoff.len(), 1);
            let credits: Vec<_> = payoff[0][1]["data"]["properties"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row[0] == 30471)
                .collect();
            assert_eq!(credits.len(), 1);
            utility[action] = fires as f64 + credits[0][1]["Int"].as_i64().unwrap() as f64;
            assert_eq!(utility[action], h["utility"].as_f64().unwrap());
        }
        states.push((o, utility));
        for (probe, theta) in e::PROBES.iter().enumerate() {
            for mode in e::MODES {
                let x = e::input(mode, &o).unwrap();
                let probs = e::probabilities(theta, &x).unwrap();
                let scores: Vec<_> = (0..2)
                    .map(|a| e::score_gradient(theta, &x, a).unwrap())
                    .collect();
                let credits: Vec<_> = (0..2)
                    .map(|a| e::contribution(theta, mode, &o, a, utility[a]).unwrap())
                    .collect();
                let requests: Vec<_> = [0.25, 0.75]
                    .into_iter()
                    .map(|draw| e::choose(theta, mode, &raw, draw).unwrap().1)
                    .collect();
                let (expected_utility, utility_gradient) =
                    e::expected_utility(theta, &x, utility).unwrap();
                records.push(json!({"state":state,"probe":probe,"theta":theta,"mode":mode,"observation":o,
                    "input":x,"funded":e::funded(&o).unwrap(),"utility":utility,"probabilities":probs,
                    "scores":scores,"contributions":credits,"requests":requests,
                    "boundary_sample":e::sample(theta,&x,probs[0]).unwrap(),
                    "expected_utility":expected_utility,"utility_gradient":utility_gradient}));
            }
        }
    }
    let mut batches = vec![];
    for (probe, theta) in e::PROBES.iter().enumerate() {
        for mode in e::MODES {
            let mut g = [0.; 4];
            let mut mean = 0.;
            for (o, u) in &states {
                let x = e::input(mode, o).unwrap();
                let p = e::probabilities(theta, &x).unwrap();
                for (action, probability) in p.iter().enumerate() {
                    let c = e::contribution(theta, mode, o, action, u[action]).unwrap();
                    for i in 0..4 {
                        g[i] += probability * c[i] / 8.;
                    }
                }
                if e::funded(o).unwrap() {
                    mean += e::expected_utility(theta, &x, *u).unwrap().0 / 4.;
                }
            }
            let directions: Vec<_> = states
                .iter()
                .filter(|(o, _)| e::funded(o).unwrap())
                .map(|(o, u)| {
                    let x = e::input(mode, o).unwrap();
                    let p = e::probabilities(theta, &x).unwrap();
                    json!({"input":x,"actual":x.iter().zip(g).map(|(a,b)| a*b).sum::<f64>(),
                    "orthogonal_prediction":p[0]*p[1]*(u[0]-u[1])/40.})
                })
                .collect();
            batches.push(
                json!({"probe":probe,"theta":theta,"mode":mode,"expected_batch_gradient":g,
                "funded_expected_mean":mean,"directions":directions}),
            );
        }
    }
    let data = json!({"profile":e::PROFILE,"classification":"saved-input-contrast-arithmetic-only",
        "records":records,"batches":batches,"native_advances":0,"optimizer_updates":0,
        "controller_fitting":false,"scientific_acceptance":false,"training_admitted":false,
        "sustained_claim":false,"independent_physics_replay":false});
    std::fs::write(
        root.join("contrast_records.json"),
        serde_json::to_vec_pretty(&data).unwrap(),
    )
    .unwrap();
}
