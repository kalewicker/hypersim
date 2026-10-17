#![cfg(feature = "hg-meta")]
use hg_sim_core::ere::rule_prevention as p;
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn save(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}
#[test]
fn catalogue_and_deadline_refusals() {
    let mut catalogues = vec![];
    for arm in p::ARMS {
        let (raw, compiled) = p::catalogue(arm).unwrap();
        assert_eq!(raw.len(), 9);
        assert_eq!(compiled.len(), 9);
        assert!(compiled
            .iter()
            .all(|r| r.meta_visible && r.pattern.node_label_ceiling.is_none()));
        assert_eq!(raw[8].priority, 400);
        assert_eq!(raw[7].priority, 300);
        assert_eq!(raw[8].enabled, arm != p::Arm::Disabled);
        assert!(p::run(arm, Instant::now() - Duration::from_secs(1)).is_err());
        catalogues.push(json!({"arm":arm,"raw":raw,"compiled":compiled}));
    }
    if let Some(path) = std::env::var_os("ERE_RULE_PREVENTION_PREFLIGHT_OUTPUT") {
        save(
            &std::path::PathBuf::from(path).join("catalogues.json"),
            &json!(catalogues),
        );
    }
}
#[test]
fn fixed_prevention_fixture() {
    let Some(path) = std::env::var_os("ERE_RULE_PREVENTION_OUTPUT") else {
        return;
    };
    let root = std::path::PathBuf::from(path);
    let out = root.join("native");
    assert!(!out.exists());
    std::fs::create_dir(&out).unwrap();
    let slots:Vec<_>=p::ARMS.into_iter().enumerate().map(|(id,arm)|json!({"id":id,"arm":arm,"context":[10,11],"actions":[1,1],"seed":p::SEED,"horizon":96})).collect();
    save(&out.join("schedule.json"), &json!(slots));
    std::fs::copy(
        std::env::current_exe().unwrap(),
        out.join("executed_test_binary"),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut status = vec![];
    let mut halted = false;
    for (id, arm) in p::ARMS.into_iter().enumerate() {
        let mut row = slots[id].clone();
        if halted {
            row["status"] = json!("not_run_after_failure");
        } else {
            match p::run(arm, deadline) {
                Ok(r) => {
                    save(&out.join(format!("cell-{id:04}.json")), &r);
                    row["status"] = json!("complete");
                }
                Err(error) => {
                    row["status"] = json!("failed");
                    row["error"] = json!(error);
                    halted = true;
                }
            }
        }
        status.push(row);
        save(&out.join("status.json"), &json!(status));
    }
    save(
        &out.join("manifest.json"),
        &json!({"profile":p::PROFILE,"status":if halted {"failed"} else {"complete"},"scheduled_cells":5,
        "completed_cells":status.iter().filter(|r|r["status"]=="complete").count(),"native_work_unknown":halted,
        "requested_advances":480,"optimizer_updates":0,"training_admitted":false,"scientific_acceptance":false,"sustained_claim":false}),
    );
    assert!(!halted, "failure retained without retry");
}
