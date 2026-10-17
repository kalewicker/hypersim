#![cfg(feature = "ere")]
use hg_train::ere::{graph_encoder as e, graph_encoder_confirmation as c};
use serde_json::{json, Value};
use std::path::PathBuf;
#[test]
fn frozen_import_refusals() {
    let initial = e::initialize();
    let import = json!([{"checkpoint":12},{"checkpoint":100,"final_graph":{"theta":initial},"final_blind":{"theta":initial}}]);
    let p = c::policies(&import).unwrap();
    assert_eq!(p.len(), 4);
    assert_eq!(p[0].1, p[3].1);
    assert_eq!(p[2].1, e::initialize());
    assert_eq!(p[0].0, e::Mode::Graph);
    assert_eq!(p[3].0, e::Mode::Blind);
    for bad in [
        json!([]),
        json!([{"checkpoint":12},{"checkpoint":99}]),
        json!([{"checkpoint":12},{"checkpoint":100,"final_graph":{"theta":[0.]},"final_blind":{"theta":initial}}]),
    ] {
        assert!(c::policies(&bad).is_err());
    }
    assert_eq!(c::design()["total_cells"], 396);
    assert_eq!(c::design()["maximum_native_attempts"], 9504);
}
#[test]
fn fixed_native_confirmation() {
    let Some(path) = std::env::var_os("ERE_GRAPH_ENCODER_CONFIRMATION_OUTPUT") else {
        return;
    };
    let root = PathBuf::from(path);
    let load = |name: &str| -> Value {
        serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap()
    };
    let import = load("checkpoint_import.json");
    let data = load("matrix_import.json");
    let rewards: Vec<f64> = data["worlds"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|w| {
            w["rewards"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_f64().unwrap())
        })
        .collect();
    let result =
        c::run(&root.join("native"), &import, &rewards).expect("native failure retained; no retry");
    assert_eq!(result["recorded_cells"], 396);
    assert_eq!(result["native_attempts"], 9504);
    assert_eq!(result["optimizer_updates"], 0);
    assert!(result["confirmation_pass"].is_boolean());
}
