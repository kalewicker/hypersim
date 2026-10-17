#![cfg(feature = "ere")]
use hg_train::ere::{graph_encoder as e, rule_maintenance as m};
use serde_json::{json, Value};
#[test]
fn schedule_and_import_refusals() {
    assert_eq!(m::design()["cells"], 63);
    assert_eq!(m::design()["maximum_native_advances"], 6048);
    assert!(m::decisions(&json!([])).is_err());
    let import = json!([{"checkpoint":12},{"checkpoint":100,"final_graph":{"theta":e::initialize()},"final_blind":{"theta":e::initialize()}}]);
    let d = m::decisions(&import).unwrap();
    assert_eq!(d.as_array().unwrap().len(), 9);
    assert_eq!(d, m::decisions(&import).unwrap());
}
#[test]
fn fixed_maintenance_panel() {
    let Some(path) = std::env::var_os("ERE_RULE_MAINTENANCE_OUTPUT") else {
        return;
    };
    let root = std::path::PathBuf::from(path);
    let import: Value =
        serde_json::from_slice(&std::fs::read(root.join("checkpoint_import.json")).unwrap())
            .unwrap();
    let result =
        m::run(&root.join("native"), &import).expect("native failure retained without retry");
    assert_eq!(result["completed_cells"], 63);
}
