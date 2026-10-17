#![cfg(feature = "hg-meta")]
use hg_sim_core::ere::lifecycle_fixture::{self as f, Arm, Spell};
use serde_json::{json, Value};
use std::{path::PathBuf, time::{Duration, Instant}};

fn save(path: PathBuf, value: &Value) {
    std::fs::write(path,serde_json::to_vec(value).unwrap()).unwrap();
}

#[test]
fn class_spells_exclude_anchor_survival_and_preserve_censoring() {
    let timeline = vec![vec![],vec!["new".into()],vec!["new".into()],vec![],vec!["new".into()]];
    assert_eq!(f::spells(&timeline,"new"),vec![Spell {start:1,end:Some(3)},Spell {start:4,end:None}]);
    assert_eq!(f::spells(&timeline,"absent"),vec![]);
}

#[test]
fn priority_only_and_dormant_initial_classes_are_not_structural_births() {
    let original = f::candidate(false,99);
    let mut priority = original.clone(); priority.priority += 1;
    let baseline = vec![f::candidate_record(&original)];
    let mut row = json!({"step":0,"genotype":baseline,"installed":[],"events":[],"paid_edits":0});
    let mut after = row.clone(); after["step"] = json!(1);
    after["genotype"] = json!([f::candidate_record(&priority)]);
    let summary = f::summarize(&baseline,&[row.clone(),after]);
    assert_eq!(summary["novel_class_count"],0);
    row["step"] = json!(1);
    assert_eq!(f::summarize(&baseline,&[row.clone(),row])["novel_class_count"],0);
}

#[test]
fn registered_native_birth_lifecycle_panel() {
    let temporary = tempfile::tempdir().unwrap();
    let out = std::env::var_os("ERE_LIFECYCLE_OUTPUT").map(PathBuf::from)
        .unwrap_or_else(||temporary.path().join("panel"));
    std::fs::create_dir(&out).expect("refusing to overwrite lifecycle fixture receipts");
    let design: Value = serde_json::from_str(include_str!("../../../configs/ere_rule_lifecycle_fixture_v1.json")).unwrap();
    assert_eq!(design["seeds"],json!(f::SEEDS)); assert_eq!(design["arms"],json!(f::ARMS));
    assert_eq!(design["horizon"],f::HORIZON); assert_eq!(design["maximum_native_attempts"],432);
    save(out.join("manifest.json"),&json!({"status":"running","design":design,"training_admitted":false}));
    std::fs::write(out.join("preregistration.md"),include_str!("../../../docs/ere_rule_lifecycle_fixture_v1.md")).unwrap();
    if std::env::var_os("ERE_LIFECYCLE_OUTPUT").is_some() {
        std::fs::copy(std::env::current_exe().unwrap(),out.join("executed_test_binary")).unwrap();
    }
    let deadline = Instant::now()+Duration::from_secs(60); let mut receipts = vec![]; let mut failures = vec![];
    // Save all eighteen outcomes BEFORE asserting the predicted causal pattern.
    for seed in f::SEEDS {
        for arm in f::ARMS {
            let result = match f::run(arm,seed,deadline) {
                Ok(row)=>row,
                Err(error)=>{ failures.push(error.clone()); json!({"seed":seed,"arm":arm,"error":error,
                    "partial_native_work_unknown":true,"training_admitted":false}) }
            };
            let name = format!("{seed}-{}.json",serde_json::to_value(arm).unwrap().as_str().unwrap());
            save(out.join(name),&result); receipts.push(result);
        }
    }
    save(out.join("manifest.json"),&json!({"status":if failures.is_empty(){"native_complete"}else{"failed"},
        "design":design,"recorded_cells":receipts.len(),"failures":failures,"training_admitted":false}));
    assert!(failures.is_empty(),"native fixture failures: {failures:?}");
    for r in &receipts {
        let arm: Arm = serde_json::from_value(r["arm"].clone()).unwrap();
        assert_eq!(r["completed_steps"],24);
        assert!(r["timeline"].as_array().unwrap().iter().all(|row|row["state"]["root_alive"]==true));
        let summary = &r["summary"];
        assert_eq!(summary["training_admitted"],false); assert_eq!(summary["sustained_claim"],false);
        if matches!(arm,Arm::LivePaid|Arm::LiveReplacement|Arm::FrozenPaid) {
            assert_eq!(summary["novel_class_count"],1);
            let class = &summary["novel_classes"][0];
            assert_eq!(class["first_birth"],1);
            match arm {
                Arm::LivePaid=>{
                    assert_eq!(class["first_installation"],2); assert_eq!(class["first_non_meta_fire"],3);
                    assert_eq!(class["non_meta_fire_steps"].as_array().unwrap().len(),11);
                    assert_eq!(class["genotype_spells"],json!([{"start":1,"end":null}]));
                    assert_eq!(summary["mean_edit_interarrival"],Value::Null);
                },
                Arm::LiveReplacement=>{
                    assert_eq!(class["genotype_spells"],json!([{"start":1,"end":9},{"start":17,"end":null}]));
                    assert_eq!(class["installed_spells"],json!([{"start":2,"end":10},{"start":18,"end":null}]));
                    assert_eq!(class["non_meta_fire_steps"],json!([3,5,7,9,19,21,23]));
                    assert_eq!(summary["mean_edit_interarrival"],8.);
                },
                Arm::FrozenPaid=>{
                    assert_eq!(class["installed_spells"],json!([]));
                    assert_eq!(class["first_non_meta_fire"],Value::Null);
                }, _=>unreachable!()
            }
        } else { assert_eq!(summary["novel_class_count"],0); }
    }
    save(out.join("native_assertions.json"),&json!({"passed":true,"cells":18,"native_attempts":432,
        "advancing_steps":432,"training_updates":0,"scientific_acceptance":false}));
}
