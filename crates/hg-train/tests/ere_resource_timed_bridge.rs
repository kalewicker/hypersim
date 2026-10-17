#![cfg(feature = "ere")]
use hg_sim_core::ere::{defense_allocation as old, resource_timed_bridge as n};
use hg_train::ere::{resource_adapter as a, resource_timed_policy as p};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn save(path: &Path, v: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(v).unwrap()).unwrap();
}
#[test]
fn catalogue_and_request_guards() {
    for action in [None, Some(old::Action::Defend), Some(old::Action::Harvest)] {
        let (raw, compiled) = n::catalogue(action).unwrap();
        let (mut expected, mut expected_compiled) =
            old::catalogue(action.unwrap_or(old::Action::Defend)).unwrap();
        if action.is_none() {
            for r in &mut expected {
                if r.name.starts_with("allocation_") {
                    r.enabled = false;
                }
            }
            for r in &mut expected_compiled {
                if r.name.starts_with("allocation_") {
                    r.enabled = false;
                }
            }
        }
        assert_eq!(
            serde_json::to_value(raw).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(compiled).unwrap(),
            serde_json::to_value(expected_compiled).unwrap()
        );
        assert!(n::validate_decision(
            13,
            if action.is_some() { 1. } else { 0. },
            n::Decision {
                step: 13,
                request: action
            }
        )
        .is_ok());
    }
    for (step, wallet, d) in [
        (
            0,
            1.,
            n::Decision {
                step: 13,
                request: Some(old::Action::Defend),
            },
        ),
        (
            13,
            1.,
            n::Decision {
                step: 14,
                request: Some(old::Action::Defend),
            },
        ),
        (
            13,
            1.,
            n::Decision {
                step: 13,
                request: None,
            },
        ),
        (
            13,
            0.,
            n::Decision {
                step: 13,
                request: Some(old::Action::Defend),
            },
        ),
        (
            13,
            f64::NAN,
            n::Decision {
                step: 13,
                request: None,
            },
        ),
        (
            13,
            2.,
            n::Decision {
                step: 13,
                request: None,
            },
        ),
    ] {
        assert!(n::validate_decision(step, wallet, d).is_err());
    }
    let fail = n::run(
        old::Scenario::HazardLow,
        old::Resource::Paid,
        Instant::now(),
        |_| panic!("expired callback"),
    )
    .unwrap_err();
    assert_eq!(
        (
            fail.native_advances,
            fail.seal_applications,
            fail.decision_calls
        ),
        (0, 0, 0)
    );
}
#[test]
fn pure_policy_refusals() {
    assert!(p::choose(&a::PROBE, a::Mode::Full, &json!({"step":0}), 0.25).is_err());
}
#[test]
#[ignore = "requires explicit fresh output; bounded native engineering, not training"]
fn fixed_timed_bridge_fixture() {
    let root = std::path::PathBuf::from(
        std::env::var("ERE_RESOURCE_TIMED_BRIDGE_ARCHIVE").expect("fresh archive required"),
    );
    let native = root.join("native");
    assert!(!native.exists());
    std::fs::create_dir(&native).unwrap();
    let mut schedule = vec![];
    for (mi, mode) in a::MODES.into_iter().enumerate() {
        for (si, scenario) in old::SCENARIOS.into_iter().enumerate() {
            for (ri, resource) in old::RESOURCES.into_iter().enumerate() {
                schedule.push(json!({"id":schedule.len(),"mode":mode,"state":si*2+ri,"scenario":scenario,"resource":resource,"draw":([0.25,0.75,0.25,0.75][mi])}));
            }
        }
    }
    for name in [
        "expired",
        "callback_error",
        "wrong_step",
        "unfunded_request",
    ] {
        schedule.push(json!({"id":schedule.len(),"refusal":name}));
    }
    assert_eq!(schedule.len(), 36);
    save(&native.join("schedule.json"), &json!(schedule));
    let mut status: Vec<_> = schedule
        .iter()
        .map(|s| json!({"id":s["id"],"status":"pending"}))
        .collect();
    save(&native.join("status.json"), &json!(status));
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut halted = false;
    for slot in &schedule {
        let id = slot["id"].as_u64().unwrap() as usize;
        if halted {
            status[id]["status"] = json!("not_run_after_failure");
            continue;
        }
        status[id]["status"] = json!("running");
        save(&native.join("status.json"), &json!(status));
        if id < 32 {
            let mode = serde_json::from_value(slot["mode"].clone()).unwrap();
            let draw = slot["draw"].as_f64().unwrap();
            let mut policy = Value::Null;
            let result = n::run(
                serde_json::from_value(slot["scenario"].clone()).unwrap(),
                serde_json::from_value(slot["resource"].clone()).unwrap(),
                deadline,
                |inventory| {
                    let (decision, record) = p::choose(&a::PROBE, mode, inventory, draw)?;
                    policy = record;
                    Ok(decision)
                },
            );
            match result {
                Ok(receipt) => {
                    save(
                        &native.join(format!("cell-{id:04}.json")),
                        &json!({"slot":slot,"policy":policy,"receipt":receipt}),
                    );
                    status[id]["status"] = json!("complete");
                    status[id]["native_advances"] = json!(32);
                    status[id]["seal_applications"] = json!(1);
                }
                Err(fail) => {
                    save(
                        &native.join(format!("cell-{id:04}.json")),
                        &json!({"slot":slot,"policy":policy,"failure":fail}),
                    );
                    status[id]["status"] = json!("failed");
                    status[id]["failure"] = json!(fail);
                    halted = true;
                }
            }
        } else {
            let name = slot["refusal"].as_str().unwrap();
            let resource = if name == "unfunded_request" {
                old::Resource::Unpaid
            } else {
                old::Resource::Paid
            };
            let result = n::run(
                old::Scenario::HazardLow,
                resource,
                if name == "expired" {
                    Instant::now()
                } else {
                    deadline
                },
                |_| match name {
                    "callback_error" => Err("registered callback error".into()),
                    "wrong_step" => Ok(n::Decision {
                        step: 14,
                        request: Some(old::Action::Defend),
                    }),
                    "unfunded_request" => Ok(n::Decision {
                        step: 13,
                        request: Some(old::Action::Defend),
                    }),
                    _ => panic!("expired callback called"),
                },
            );
            match result {
                Err(fail) => {
                    let expected = if name == "expired" { 0 } else { 13 };
                    let pass = fail.native_advances == expected
                        && fail.seal_applications == u64::from(expected > 0)
                        && fail.decision_calls == u64::from(expected > 0);
                    save(
                        &native.join(format!("cell-{id:04}.json")),
                        &json!({"slot":slot,"failure":fail}),
                    );
                    status[id] = json!({"id":id,"status":if pass{"expected_refusal"}else{"failed"},"failure":fail});
                    halted = !pass;
                }
                Ok(receipt) => {
                    save(
                        &native.join(format!("cell-{id:04}.json")),
                        &json!({"slot":slot,"unexpected_receipt":receipt}),
                    );
                    status[id]["status"] = json!("failed");
                    halted = true;
                }
            }
        }
        save(&native.join("status.json"), &json!(status));
    }
    save(&native.join("status.json"), &json!(status));
    assert!(!halted, "fixed bridge failure retained; no retry");
}
