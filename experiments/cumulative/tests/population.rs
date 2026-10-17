use hg_cumulative_readiness::{
    native_work,
    population::{self, Population},
};
use serde_json::{json, Value};

struct Work;
impl Drop for Work {
    fn drop(&mut self) {
        println!("native_test_rewrites={}", native_work());
    }
}
fn cases() -> Value {
    serde_json::from_str(include_str!("../population_contract.json")).unwrap()
}

#[test]
fn one_reservoir_funds_children_without_new_environment() {
    let _work = Work;
    let c = cases();
    let mut p = Population::from_case(&c["cases"][0], 2026100902, 0).unwrap();
    for _ in 0..8 {
        p.step(false).unwrap();
        p.validate().unwrap();
    }
    let s = p.state().unwrap();
    assert_eq!(s["reservoir"], 0);
    assert!(s["births"].as_array().unwrap().len() >= 1);
    for m in s["cells"].as_array().unwrap() {
        let c = &m["state"];
        assert_eq!(c["environment"], 0);
        assert_eq!(
            c["initial"].as_i64().unwrap(),
            m["endowment"].as_i64().unwrap() + 16 * m["grants"].as_i64().unwrap()
        );
        if !c["parent"].is_null() {
            assert_eq!(m["endowment"], 8);
        }
    }
}
#[test]
fn starvation_and_site_refusals_remain_in_full_horizon() {
    let _work = Work;
    let c = cases();
    for index in [5, 6, 7, 8] {
        let spec = &c["cases"][index];
        let mut p = Population::from_case(spec, 2026100902, 0).unwrap();
        for _ in 0..spec["ticks"].as_u64().unwrap() {
            p.step(false).unwrap();
        }
        assert_eq!(p.rows.len() as u64, spec["ticks"].as_u64().unwrap());
        if index == 5 {
            let s = p.state().unwrap();
            assert!(s["cells"][0]["site"].is_null());
            assert_eq!(s["cells"][0]["state"]["capital"], 0);
        }
        if index == 7 {
            assert!(p.state().unwrap()["births"].as_array().unwrap().is_empty());
            assert!(p.rows.iter().any(|r| r["birth_refusals"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x["reason"] == "no_site")));
        }
    }
}
#[test]
fn ancestry_and_all_four_arms_have_distinct_paid_semantics() {
    let _work = Work;
    let c = cases();
    for index in [0, 2, 3, 4] {
        let mut p = Population::from_case(&c["cases"][index], 2026100902, 0).unwrap();
        p.step(false).unwrap();
        let s = p.state().unwrap();
        assert!(!s["births"].as_array().unwrap().is_empty());
        let b = &s["births"][0];
        assert!(b["parent"].as_u64().unwrap() < 2);
        assert_eq!(b["child"], 2);
        if index == 4 {
            assert_eq!(b["transmitted"]["stages"], json!([0]));
            assert_eq!(b["inherited"]["stages"].as_array().unwrap().len(), 2);
        }
        if index == 3 {
            assert!(p.rows[0]["operations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o["receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["kind"] == "external_edit")));
        }
    }
}
#[test]
fn population_checkpoint_and_order_resume_exactly_after_birth_and_edit() {
    let _work = Work;
    let c = cases();
    let mut a = Population::from_case(&c["cases"][2], 2026100902, 0).unwrap();
    for _ in 0..4 {
        a.step(false).unwrap();
    }
    let checkpoint = a.checkpoint().unwrap();
    let mut b = Population::restore(
        serde_json::from_slice(&serde_json::to_vec(&checkpoint).unwrap()).unwrap(),
    )
    .unwrap();
    a.rows.clear();
    for _ in 0..3 {
        a.step(false).unwrap();
        b.step(true).unwrap();
    }
    assert_eq!(a.state().unwrap(), b.state().unwrap());
    assert_eq!(a.rows, b.rows);
    for field in ["reservoir", "initial_energy", "next_id"] {
        let mut corrupt = checkpoint.clone();
        corrupt["state"][field] = json!(-1);
        assert!(Population::restore(corrupt).is_err());
    }
    let mut corrupt = checkpoint.clone();
    corrupt["cells"][0]["metadata"]["grants"] = json!(99);
    assert!(Population::restore(corrupt).is_err());
    let mut corrupt = checkpoint.clone();
    corrupt["cells"][0]["graph"]["label_index"] = json!({});
    assert!(Population::restore(corrupt).is_err());
    let mut corrupt = checkpoint.clone();
    corrupt["state"]["births"][0]["copy_cost"] = json!(0);
    assert!(Population::restore(corrupt).is_err());
    let mut corrupt = checkpoint;
    corrupt["unexpected"] = json!(true);
    assert!(Population::restore(corrupt).is_err());
}
#[test]
fn invalid_initial_world_rejects_before_native_work() {
    let _work = Work;
    let c = cases();
    let before = native_work();
    for field in ["capital", "reservoir", "sites", "parcels_per_tick"] {
        let mut bad = c["cases"][0].clone();
        bad[field] = json!(9999);
        assert!(Population::from_case(&bad, 0, 0).is_err());
    }
    assert_eq!(native_work(), before);
}
#[test]
fn complete_population_fixture_never_admits_evolution() {
    let _work = Work;
    let f = population::fixture().unwrap();
    assert_eq!(f["cases"].as_array().unwrap().len(), 9);
    for field in [
        "readiness_gate_complete",
        "scientific_acceptance",
        "evolution_demonstrated",
        "campaign_launch_authorized",
    ] {
        assert_eq!(f[field], false);
    }
}
