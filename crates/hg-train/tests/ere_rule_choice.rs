#![cfg(feature = "ere")]
use hg_sim_core::ere::rule_choice::{self as native, Arm};
use hg_train::ere::rule_choice as adapter;
use serde_json::{json, Value};
use std::{path::PathBuf, time::{Duration, Instant}};

fn save(path: PathBuf, value: &Value) {
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

#[test]
fn stable_policy_boundaries_gradients_and_refusals() {
    let theta = [0.2,-0.3,0.8,-1.1, 1.0,0.0,-0.5,0.7, -0.2,0.3,0.1,0.6];
    for context in native::CONTEXTS {
        let p = adapter::probabilities(&theta, context).unwrap();
        assert!((p.iter().sum::<f64>()-1.).abs()<1e-14);
        let mut lower = 0.;
        for action in 0..4 {
            assert_eq!(adapter::sample(&theta,context,lower+p[action]/2.).unwrap(),action);
            lower += p[action];
            let gradient = adapter::score_gradient(&theta,context,action).unwrap();
            for i in 0..12 {
                let mut hi = theta; let mut lo = theta; hi[i] += 1e-6; lo[i] -= 1e-6;
                let difference = (adapter::log_probability(&hi,context,action).unwrap()
                    - adapter::log_probability(&lo,context,action).unwrap())/2e-6;
                assert!((gradient[i]-difference).abs()<1e-9);
            }
        }
    }
    let flat = [0.;12];
    for (draw,action) in [(0.,0),(0.25,1),(0.5,2),(0.75,3),(1.-f64::EPSILON,3)] {
        assert_eq!(adapter::sample(&flat,10,draw).unwrap(),action);
    }
    let extreme = [-1000.,1000.,-1000.,-1000.,0.,0.,0.,0.,0.,0.,0.,0.];
    assert_eq!(adapter::sample(&extreme,10,0.).unwrap(),1);
    assert_eq!(adapter::log_probability(&extreme,10,0).unwrap(),-2000.);
    for draw in [-1.,1.,f64::NAN,f64::INFINITY] { assert!(adapter::sample(&flat,10,draw).is_err()); }
    assert!(adapter::probabilities(&flat,99).is_err());
    assert!(adapter::probabilities(&flat[..11],10).is_err());
    assert!(adapter::log_probability(&flat,10,4).is_err());
    let mut bad = flat; bad[0] = f64::NAN; assert!(adapter::probabilities(&bad,10).is_err());
    bad[0] = f64::MAX; bad[1] = -f64::MAX; assert!(adapter::probabilities(&bad,10).is_err());
    assert!(adapter::reward(&json!({})).is_err());
    assert!(native::run(Arm::LivePaid,99,1,native::SEEDS[0],Instant::now()).is_err());
    assert!(native::run(Arm::LivePaid,10,4,native::SEEDS[0],Instant::now()).is_err());
    assert!(native::run(Arm::LivePaid,10,1,0,Instant::now()).is_err());
}

#[test]
fn registered_native_paid_choice_panel() {
    let temporary = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_RULE_CHOICE_OUTPUT").map(PathBuf::from)
        .unwrap_or_else(||temporary.path().join("panel"));
    std::fs::create_dir(&out).expect("refusing to overwrite choice receipts");
    let design: Value = serde_json::from_str(include_str!("../../../configs/ere_rule_choice_adapter_v1.json")).unwrap();
    assert_eq!(design["seeds"],json!(native::SEEDS));
    assert_eq!(design["contexts"],json!(native::CONTEXTS));
    assert_eq!(design["actions"],json!(native::ACTIONS));
    assert_eq!(design["arms"],json!(native::ARMS));
    assert_eq!(design["maximum_native_attempts"],1152);
    save(out.join("manifest.json"),&json!({"status":"running","design":design,"training_admitted":false}));
    std::fs::write(out.join("preregistration.md"),include_str!("../../../docs/ere_rule_choice_adapter_v1.md")).unwrap();
    if std::env::var_os("ERE_RULE_CHOICE_OUTPUT").is_some() {
        std::fs::copy(std::env::current_exe().unwrap(),out.join("executed_test_binary")).unwrap();
    }
    let deadline = Instant::now()+Duration::from_secs(60);
    let mut receipts = vec![]; let mut failures = vec![];
    for seed in native::SEEDS {
        for context in native::CONTEXTS {
            let correct = native::ACTIONS.iter().position(|x| *x == Some(context)).unwrap();
            let cells: Vec<_> = (0..4).map(|a| (Arm::LivePaid,a))
                .chain(native::ARMS[1..].iter().map(|arm| (*arm,correct))).collect();
            for (arm,action) in cells {
                let mut r = match native::run(arm,context,action,seed,deadline) {
                    Ok(r) => r,
                    Err(error) => { failures.push(error.clone()); json!({"seed":seed,"context":context,
                        "action":action,"arm":arm,"error":error,"partial_native_work_unknown":true}) }
                };
                if r.get("error").is_none() {
                    match adapter::reward(&r) {
                        Ok(reward) => r["adapter_reward"] = json!(reward),
                        Err(error) => { failures.push(error.clone()); r["adapter_error"] = json!(error); }
                    }
                }
                let arm_name = serde_json::to_value(arm).unwrap();
                save(out.join(format!("{seed}-{context}-{}-{action}.json",arm_name.as_str().unwrap())),&r);
                receipts.push(r);
            }
        }
    }
    save(out.join("manifest.json"),&json!({"status":if failures.is_empty(){"native_complete"}else{"failed"},
        "design":design,"recorded_cells":receipts.len(),"failures":failures,"training_admitted":false}));
    assert!(failures.is_empty(),"choice failures: {failures:?}");
    let mut credited = 0.;
    for r in &receipts {
        let correct = native::ACTIONS[r["action"].as_u64().unwrap() as usize] == Some(r["context"].as_u64().unwrap() as u32);
        let expected = if r["arm"] == "live_paid" && correct { 11./24. } else { 0. };
        assert_eq!(r["adapter_reward"].as_f64().unwrap(),expected);
        credited += expected * 24.;
        // Detached interface failures refuse rather than quietly assigning zero.
        for field in ["summary","timeline","initial_all_candidates","ordinary_nodes"] {
            let mut bad = r.clone(); bad[field] = Value::Null; assert!(adapter::reward(&bad).is_err());
        }
        let mut bad = r.clone(); bad["timeline"][3]["usage"]["truncated"] = json!(true);
        assert!(adapter::reward(&bad).is_err());
        bad = r.clone(); bad["timeline"][24]["installed"] = json!([]);
        assert!(adapter::reward(&bad).is_err());
    }
    assert_eq!(credited,66.);
    let theta = vec![0.2,-0.3,0.8,-1.1,1.0,0.0,-0.5,0.7,-0.2,0.3,0.1,0.6];
    let contexts: Vec<_> = native::CONTEXTS.iter().map(|&context| {
        let p = adapter::probabilities(&theta,context).unwrap(); let mut lower = 0.;
        let actions: Vec<_> = (0..4).map(|action| {
            let sampled = adapter::sample(&theta,context,lower+p[action]/2.).unwrap(); lower += p[action];
            json!({"action":action,"log_probability":adapter::log_probability(&theta,context,action).unwrap(),
                "score_gradient":adapter::score_gradient(&theta,context,action).unwrap().to_vec(),"sample_midpoint":sampled})
        }).collect();
        json!({"context":context,"probabilities":p,"actions":actions})
    }).collect();
    save(out.join("policy.json"),&json!({"theta":theta,"contexts":contexts,"training_updates":0,"training_admitted":false}));
    save(out.join("native_assertions.json"),&json!({"passed":true,"cells":48,"native_attempts":1152,
        "advancing_steps":1152,"training_updates":0,"scientific_acceptance":false}));
}
