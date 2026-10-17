#![cfg(feature = "hg-meta")]
use hg_sim_core::ere::defense_allocation as n;
use serde_json::json;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
fn save(path: &std::path::Path, value: &serde_json::Value) {
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}
#[test]
fn catalogue_and_deadline_refusals() {
    let rows = n::ACTIONS
        .into_iter()
        .map(|action| {
            let (raw, compiled) = n::catalogue(action).unwrap();
            assert_eq!(raw.len(), 9);
            assert!(compiled
                .iter()
                .all(|r| r.meta_visible && r.pattern.node_label_ceiling.is_none()));
            json!({"action":action,"raw":raw,"compiled":compiled})
        })
        .collect::<Vec<_>>();
    assert!(n::run(
        n::Scenario::HazardLow,
        n::Resource::Paid,
        n::Action::Defend,
        Instant::now()
    )
    .is_err());
    if let Some(path) = std::env::var_os("ERE_DEFENSE_ALLOCATION_PREFLIGHT_OUTPUT") {
        save(&PathBuf::from(path).join("catalogues.json"), &json!(rows));
    }
}
#[test]
fn fixed_allocation_fixture() {
    let Some(path) = std::env::var_os("ERE_DEFENSE_ALLOCATION_OUTPUT") else {
        return;
    };
    let root = PathBuf::from(path);
    let out = root.join("native");
    assert!(!out.exists());
    std::fs::create_dir(&out).unwrap();
    let mut slots = vec![];
    for scenario in n::SCENARIOS {
        for resource in n::RESOURCES {
            for action in n::ACTIONS {
                slots.push(json!({"id":slots.len(),"scenario":scenario,"resource":resource,"action":action,"horizon":32,"seed":n::SEED}));
            }
        }
    }
    let mut status = slots
        .iter()
        .map(|s| {
            let mut s = s.clone();
            s["status"] = json!("pending");
            s
        })
        .collect::<Vec<_>>();
    save(&out.join("schedule.json"), &json!(slots));
    save(&out.join("status.json"), &json!(status));
    save(
        &out.join("manifest.json"),
        &json!({"profile":n::PROFILE,"status":"running","optimizer_updates":0,"training_admitted":false,"scientific_acceptance":false,"sustained_claim":false}),
    );
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut halted = false;
    for s in &slots {
        let id = s["id"].as_u64().unwrap() as usize;
        if halted {
            status[id]["status"] = json!("not_run_after_failure");
        } else {
            match n::run(
                serde_json::from_value(s["scenario"].clone()).unwrap(),
                serde_json::from_value(s["resource"].clone()).unwrap(),
                serde_json::from_value(s["action"].clone()).unwrap(),
                deadline,
            ) {
                Ok(r) => {
                    save(&out.join(format!("cell-{id:04}.json")), &r);
                    status[id]["status"] = json!("complete");
                }
                Err(e) => {
                    status[id]["status"] = json!("failed");
                    status[id]["error"] = json!(e);
                    halted = true;
                }
            }
        }
        save(&out.join("status.json"), &json!(status));
    }
    save(
        &out.join("manifest.json"),
        &json!({"profile":n::PROFILE,"status":if halted {"failed"} else {"complete"},"scheduled_cells":16,
        "completed_cells":status.iter().filter(|s|s["status"]=="complete").count(),"requested_advances":512,"native_work_unknown":halted,
        "optimizer_updates":0,"training_admitted":false,"scientific_acceptance":false,"sustained_claim":false}),
    );
    assert!(!halted, "native failure retained without retry");
}
