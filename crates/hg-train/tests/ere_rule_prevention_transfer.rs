#![cfg(feature = "ere")]
use hg_train::ere::{graph_encoder as e, rule_prevention_transfer as p};
use serde_json::{json, Value};
#[test]
fn schedule_and_import_refusals() {
    assert_eq!(p::design()["cells"], 72);
    assert!(p::decisions(&json!([])).is_err());
    let import = json!([{"checkpoint":12},{"checkpoint":100,"final_graph":{"theta":e::initialize()},"final_blind":{"theta":e::initialize()}}]);
    let d = p::decisions(&import).unwrap();
    assert_eq!(d, p::decisions(&import).unwrap());
    assert_eq!(p::schedule(&d).unwrap().as_array().unwrap().len(), 72);
    let mut bad = d.clone();
    bad[0]["context"] = json!([20, 21]);
    assert!(p::schedule(&bad).is_err());
    if let Some(path) = std::env::var_os("ERE_RULE_PREVENTION_TRANSFER_DECISION_PREFLIGHT") {
        let root = std::path::PathBuf::from(path);
        let import: Value =
            serde_json::from_slice(&std::fs::read(root.join("checkpoint_import.json")).unwrap())
                .unwrap();
        let choices = p::decisions(&import).unwrap();
        std::fs::write(
            root.join("decision_preflight.json"),
            serde_json::to_vec(&choices).unwrap(),
        )
        .unwrap();
    }
    let mut bad = d.clone();
    bad[0]["policies"][0]["actions"] = json!([4, 1]);
    assert!(p::schedule(&bad).is_err());
}
#[test]
fn fixed_prevention_transfer_panel() {
    let Some(path) = std::env::var_os("ERE_RULE_PREVENTION_TRANSFER_OUTPUT") else {
        return;
    };
    let root = std::path::PathBuf::from(path);
    let import: Value =
        serde_json::from_slice(&std::fs::read(root.join("checkpoint_import.json")).unwrap())
            .unwrap();
    let result = p::run(&root.join("native"), &import).expect("failure retained without retry");
    assert_eq!(result["completed_cells"], 72);
}
