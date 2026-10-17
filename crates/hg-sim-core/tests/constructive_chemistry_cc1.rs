#![cfg(feature = "constructive-chemistry")]
use hg_sim_core::constructive_chemistry::fixture::{run_case, run_contract};
use serde_json::{json, Value};
fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json"
    ))
    .unwrap()
}
#[test]
fn all_41_frozen_cases_and_82_worker_variants() {
    let c = contract();
    let result = run_contract(&c);
    let variants = result["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 82);
    for (case, pair) in c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(variants.chunks_exact(2))
    {
        for r in pair {
            assert_eq!(r["id"], case["id"]);
            if let Some(n) = case["expected"].get("accepted") {
                assert_eq!(r["accepted"], *n, "{}", r["id"]);
            }
            if let Some(e) = case["expected"].get("rejected") {
                assert_eq!(r["rejected"], *e, "{}", r["id"]);
            }
            if let Some(e) = case["expected"].get("rejection") {
                assert_eq!(r["rejected"], json!([e]), "{}", r["id"]);
            }
            for a in r["attempts"].as_array().unwrap() {
                if a["decision"] != "accepted" {
                    assert_eq!(
                        a["before"]["exact"], a["after"]["exact"],
                        "rejection/preparation mutated {}",
                        r["id"]
                    );
                }
            }
        }
        let mut a = pair[0].clone();
        let mut b = pair[1].clone();
        a.as_object_mut().unwrap().remove("workers");
        b.as_object_mut().unwrap().remove("workers");
        assert_eq!(a, b, "same-order worker identity {}", case["id"]);
    }
}
#[test]
fn independent_world_authority_even_with_identical_display_identity() {
    let c = contract();
    let case = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "cross_world_commit")
        .unwrap();
    let result = run_case(&c, case, 4);
    assert_eq!(result["rejected"], json!(["WorldIdentity"]));
    assert_eq!(
        result["diagnostic"]["other_before"],
        result["diagnostic"]["other_after"]
    );
}
#[test]
fn finite_boundary_roundtrip_reuses_slot_with_new_generation() {
    let c = contract();
    let case = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "deleted_and_reused_fuel_handle")
        .unwrap();
    let r = run_case(&c, case, 4);
    let old = &r["diagnostic"]["old_handle"]["node"];
    let new = &r["diagnostic"]["new_handle"]["node"];
    assert_ne!(old, new);
    assert_eq!(r["rejected"], json!(["StaleState", "MissingHandle"]));
}
#[test]
fn no_scientific_admission_or_reaction_clock() {
    let r = run_contract(&contract());
    for f in [
        "training_admitted",
        "scientific_acceptance",
        "sustained_claim",
        "evolution_enabled",
        "scientific_run_launched",
        "ready_for_scientific_launch",
        "reproductive_fairness_accepted",
        "power_adequacy_established",
    ] {
        assert_eq!(r[f], false);
    }
    assert_eq!(r["rng"], Value::Null);
    assert_eq!(r["physical_time"], Value::Null);
    assert_eq!(r["seeds"], json!([]));
}
