#![cfg(feature = "ere")]
use hg_train::ere::resource_adapter as e;
use serde_json::{json, Value};

fn raw(wallet: f64, forecast: i64, bounty: i64) -> Value {
    json!({"step":13,"nodes":[
        ["NodeId(1v1)",{"data":{"label":4293949189u64,"properties":[[1,{"Float":wallet}]]},"incident":[]}],
        ["NodeId(2v1)",{"data":{"label":4293949188u64,"properties":[[30468,{"Int":0}],[30469,{"Int":forecast}]]},"incident":[]}],
        ["NodeId(3v1)",{"data":{"label":4293949190u64,"properties":[[30470,{"Int":bounty}],[30471,{"Int":0}]]},"incident":[]}]
    ],"edges":[]})
}
#[test]
fn observation_and_masks() {
    let mut observations = vec![];
    for w in [0., 1.] {
        for f in [0, 1] {
            for b in [2, 12] {
                let mut v = raw(w, f, b);
                let o = e::observe(&v).unwrap();
                observations.push(o);
                v["nodes"].as_array_mut().unwrap().reverse();
                for (i, r) in v["nodes"].as_array_mut().unwrap().iter_mut().enumerate() {
                    r[0] = json!(format!("NodeId({}v9)", 100 + i));
                    r[1]["data"]["properties"].as_array_mut().unwrap().reverse();
                }
                v["scenario"] = json!("bogus");
                v["action"] = json!("harvest");
                v["wallet_units"] = json!(999);
                v["nodes"].as_array_mut().unwrap().push(
                    json!(["NodeId(999v1)",{"data":{"label":12,"properties":[]},"incident":[]}]),
                );
                assert_eq!(e::observe(&v).unwrap(), o);
                assert_eq!(
                    e::input(e::Mode::Full, &o).unwrap(),
                    [1., w, f as f64, b as f64 / 12.]
                );
                assert_eq!(e::input(e::Mode::CuesMasked, &o).unwrap(), [1., w, 0., 0.]);
                assert_eq!(
                    e::input(e::Mode::ForecastMasked, &o).unwrap(),
                    e::input(
                        e::Mode::ForecastMasked,
                        &e::observe(&raw(w, 1 - f, b)).unwrap()
                    )
                    .unwrap()
                );
                assert_eq!(
                    e::input(e::Mode::BountyMasked, &o).unwrap(),
                    e::input(
                        e::Mode::BountyMasked,
                        &e::observe(&raw(w, f, 14 - b)).unwrap()
                    )
                    .unwrap()
                );
            }
        }
    }
    for i in 0..8 {
        for j in i + 1..8 {
            assert_ne!(observations[i], observations[j]);
        }
    }
}
#[test]
fn observation_refusals() {
    let original = raw(1., 1, 2);
    let mut cases = vec![];
    let mut v = original.clone();
    v["step"] = json!(0);
    cases.push(v);
    let mut v = original.clone();
    v["nodes"].as_array_mut().unwrap().pop();
    cases.push(v);
    let mut v = original.clone();
    v["nodes"]
        .as_array_mut()
        .unwrap()
        .push(original["nodes"][0].clone());
    cases.push(v);
    let mut v = original.clone();
    let mut r = v["nodes"][0].clone();
    r[0] = json!("NodeId(99v1)");
    v["nodes"].as_array_mut().unwrap().push(r);
    cases.push(v);
    let mut v = original.clone();
    v["nodes"][0][1]["data"]["properties"][0][1] = json!({"Int":1});
    cases.push(v);
    let mut v = original.clone();
    v["nodes"][1][1]["data"]["properties"][1][1] = json!({"Float":1.});
    cases.push(v);
    let mut v = original.clone();
    v["nodes"][0][1]["data"]["properties"] = json!([[1,{"Float":1.}],[1,{"Float":1.}]]);
    cases.push(v);
    let mut v = original.clone();
    v["nodes"][0][1]["incident"] = json!(["EdgeId(1v1)"]);
    cases.push(v);
    let mut v = original.clone();
    v["edges"] = json!([["EdgeId(1v1)",{"nodes":[{"idx":1,"version":1}]}]]);
    cases.push(v);
    let mut v = original.clone();
    v["edges"] = json!([["EdgeId(1v1)",{"nodes":[{"idx":99,"version":1}]}]]);
    cases.push(v);
    let mut v = original.clone();
    v["nodes"][0][0] = json!("NodeId(1v0)");
    cases.push(v);
    let mut v = original.clone();
    v["nodes"][1][1]["data"]["properties"][0][1] = json!({"Int":1});
    cases.push(v);
    let mut v = original.clone();
    v["nodes"][2][1]["data"]["properties"][1][1] = json!({"Int":1});
    cases.push(v);
    cases.extend([raw(2., 1, 2), raw(1., 2, 2), raw(1., 1, 3)]);
    assert_eq!(cases.len(), 16);
    for v in cases {
        assert!(e::observe(&v).is_err(), "{v}");
    }
    let invalid: e::Observation =
        serde_json::from_value(json!({"wallet":1,"forecast":1,"bounty":3})).unwrap();
    assert!(e::input(e::Mode::Full, &invalid).is_err());
    assert!(e::request(&invalid, 0).is_err());
}
#[test]
fn head_gradients_and_bridge() {
    for w in [0., 1.] {
        for f in [0, 1] {
            for b in [2, 12] {
                let o = e::observe(&raw(w, f, b)).unwrap();
                for mode in e::MODES {
                    let x = e::input(mode, &o).unwrap();
                    let p = e::probabilities(&e::PROBE, &x).unwrap();
                    assert_eq!(e::probabilities(&e::initialize(), &x).unwrap(), [0.5, 0.5]);
                    assert_eq!(e::sample(&e::PROBE, &x, 0.).unwrap(), 0);
                    assert_eq!(e::sample(&e::PROBE, &x, p[0]).unwrap(), 1);
                    for action in 0..2 {
                        let g = e::score_gradient(&e::PROBE, &x, action).unwrap();
                        for i in 0..4 {
                            let mut hi = e::PROBE;
                            let mut lo = e::PROBE;
                            hi[i] += 1e-6;
                            lo[i] -= 1e-6;
                            let fd = (e::probabilities(&hi, &x).unwrap()[action].ln()
                                - e::probabilities(&lo, &x).unwrap()[action].ln())
                                / 2e-6;
                            assert!((g[i] - fd).abs() < 2e-8);
                        }
                        if w == 0. {
                            assert!(e::request(&o, action).is_err());
                        } else {
                            assert_eq!(
                                serde_json::to_value(e::request(&o, action).unwrap()).unwrap(),
                                json!(if action == 0 { "defend" } else { "harvest" })
                            );
                        }
                    }
                    let (_, g) = e::expected_utility(&e::PROBE, &x, [8., 12.]).unwrap();
                    for i in 0..4 {
                        let mut hi = e::PROBE;
                        let mut lo = e::PROBE;
                        hi[i] += 1e-6;
                        lo[i] -= 1e-6;
                        let fd = (e::expected_utility(&hi, &x, [8., 12.]).unwrap().0
                            - e::expected_utility(&lo, &x, [8., 12.]).unwrap().0)
                            / 2e-6;
                        assert!((g[i] - fd).abs() < 2e-8);
                    }
                }
            }
        }
    }
    let x = [1., 1., 1., 1.];
    assert!(e::probabilities(&[f64::NAN; 4], &x).is_err());
    assert!(e::probabilities(&[1e308; 4], &x).is_err());
    assert!(e::probabilities(&[100.; 4], &x).is_err());
    assert!(e::probabilities(&e::PROBE, &[0.; 4]).is_err());
    for d in [-1., 1., f64::NAN] {
        assert!(e::sample(&e::PROBE, &x, d).is_err());
    }
    assert!(e::score_gradient(&e::PROBE, &x, 2).is_err());
    assert!(e::expected_utility(&e::PROBE, &x, [f64::NAN, 8.]).is_err());
    assert!(e::request(&e::observe(&raw(1., 1, 2)).unwrap(), 2).is_err());
}
#[test]
#[ignore = "requires explicit archived receipt directory; zero native work"]
fn fixed_receipt_fixture() {
    let root = std::path::PathBuf::from(
        std::env::var("ERE_RESOURCE_ADAPTER_ARCHIVE").expect("archive required"),
    );
    let load = |p: std::path::PathBuf| -> Value {
        serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
    };
    let audit = load(root.join("inputs/allocation_audit.json"));
    let mut records = vec![];
    for state in 0..8 {
        let receipts: Vec<_> = (0..2)
            .map(|a| load(root.join(format!("inputs/cell-{:04}.json", state * 2 + a))))
            .collect();
        assert_eq!(
            receipts[0]["allocation_observation"],
            receipts[1]["allocation_observation"]
        );
        let o = e::observe(&receipts[0]["allocation_observation"]["inventory"]).unwrap();
        let u = std::array::from_fn(|a| audit["cells"][state * 2 + a]["utility"].as_f64().unwrap());
        for mode in e::MODES {
            let x = e::input(mode, &o).unwrap();
            let p = e::probabilities(&e::PROBE, &x).unwrap();
            let scores:Vec<_>=(0..2).map(|a|json!({"action":a,"gradient":e::score_gradient(&e::PROBE,&x,a).unwrap(),"request":e::request(&o,a).ok(),"sample":e::sample(&e::PROBE,&x,if a==0 {p[0]/2.} else {p[0]+p[1]/2.}).unwrap()})).collect();
            let (expected, gradient) = e::expected_utility(&e::PROBE, &x, u).unwrap();
            records.push(json!({"state":state,"mode":mode,"observation":o,"input":x,"probabilities":p,"neutral":e::probabilities(&e::initialize(),&x).unwrap(),"utility":u,"expected_utility":expected,"utility_gradient":gradient,"actions":scores,"boundary_sample":e::sample(&e::PROBE,&x,p[0]).unwrap()}));
        }
    }
    assert_eq!(records.len(), 32);
    std::fs::write(root.join("adapter_records.json"),serde_json::to_vec_pretty(&json!({"profile":e::PROFILE,"theta":e::PROBE,"records":records,"native_advances":0,"optimizer_updates":0})).unwrap()).unwrap();
}
