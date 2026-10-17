use hg_cumulative_readiness::{
    native_work,
    population::regulation::world::{self, World},
};
use serde_json::{json, Value};
struct Work;
impl Drop for Work {
    fn drop(&mut self) {
        println!("native_test_rewrites={}", native_work());
    }
}
fn contract() -> Value {
    serde_json::from_str(include_str!("../integration_contract.json")).unwrap()
}

#[test]
fn every_declared_world_retains_its_horizon_and_version_witness() {
    let _work = Work;
    let f = world::fixture().unwrap();
    assert_eq!(f["native_rewrites"], 205);
    assert_eq!(f["cases"].as_array().unwrap().len(), 12);
    for k in [
        "readiness_gate_complete",
        "scientific_acceptance",
        "evolution_demonstrated",
        "campaign_launch_authorized",
    ] {
        assert_eq!(f[k], false);
    }
    let witness = f["cases"].as_array().unwrap().last().unwrap();
    let births = witness["final"]["births"].as_array().unwrap();
    assert_eq!(births.len(), 2);
    assert_eq!(
        births[0]["transmitted"]["genes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| g[0].as_i64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 2, 3]
    );
    assert_eq!(births[0]["transmitted"]["mask"], 17);
    assert_eq!(births[1]["transmitted"]["mask"], 0);
    assert_eq!(f["continuation"]["original"], f["continuation"]["restored"]);
}
#[test]
fn shared_births_reset_complete_nonheritable_policy_without_new_resources() {
    let _work = Work;
    let spec = &contract()["cases"][2];
    let mut p = World::from_case(spec, 2026100905, 0).unwrap();
    p.step(false).unwrap();
    let s = p.state().unwrap();
    assert_eq!(s["births"].as_array().unwrap().len(), 1);
    assert_eq!(s["cells"][0]["state"]["program"]["enabled"], 0);
    let child = &s["cells"][1]["state"];
    assert_eq!(child["program"], child["baseline"]);
    assert_eq!(child["endowment"], 8);
    assert_eq!(child["reservoir"], 0);
    assert_eq!(child["grants"], 0);
    assert_eq!(child["core"]["environment"], 0);
    assert_eq!(child["attempts"], 0);
}
#[test]
fn full_competition_continues_after_checkpoint_with_reversed_inputs() {
    let _work = Work;
    let spec = &contract()["cases"][0];
    let mut a = World::from_case(spec, 2026100905, 0).unwrap();
    a.step(false).unwrap();
    a.step(false).unwrap();
    let mut b = World::restore(a.checkpoint().unwrap()).unwrap();
    a.rows.clear();
    for _ in 0..3 {
        a.step(false).unwrap();
        b.step(true).unwrap();
    }
    assert_eq!(a.state().unwrap(), b.state().unwrap());
    assert_eq!(a.rows, b.rows);
}
#[test]
fn malformed_world_and_member_checkpoints_reject_without_native_work() {
    let _work = Work;
    let before = native_work();
    let cp = World::from_case(&contract()["cases"][0], 2026100905, 0)
        .unwrap()
        .checkpoint()
        .unwrap();
    let mut bad = cp.clone();
    bad["state"]["reservoir"] = json!(144);
    assert!(World::restore(bad).is_err());
    let mut bad = cp.clone();
    bad["cells"][1]["site"] = json!(0);
    assert!(World::restore(bad).is_err());
    let mut bad = cp.clone();
    bad["cells"][0]["checkpoint"]["graph"]["label_index"]["1001"] = json!([]);
    assert!(World::restore(bad).is_err());
    let mut bad = cp.clone();
    bad["cells"][0]["checkpoint"]["state"]["program"]["mask"] = json!(0);
    assert!(World::restore(bad).is_err());
    let mut bad = cp;
    let duplicate = bad["cells"][0].clone();
    bad["cells"].as_array_mut().unwrap().push(duplicate);
    assert!(World::restore(bad).is_err());
    assert_eq!(native_work(), before);
}
#[test]
fn invalid_founder_denominator_rejects_before_native_execution() {
    let _work = Work;
    let before = native_work();
    for (key, value) in [
        ("sites", json!(0)),
        ("reservoir", json!(15)),
        ("capital", json!(65)),
        ("programs", json!([])),
        ("parcels_per_tick", json!(9)),
    ] {
        let mut s = contract()["cases"][0].clone();
        s[key] = value;
        assert!(World::from_case(&s, 9, 0).is_err());
    }
    let mut s = contract()["cases"][0].clone();
    s["programs"][1]["mask"] = json!(32);
    assert!(World::from_case(&s, 9, 0).is_err());
    assert_eq!(native_work(), before);
}
#[test]
fn dead_cells_stay_in_the_world_and_tick_cap_is_unchanged_refusal() {
    let _work = Work;
    let mut p = World::from_case(&contract()["cases"][8], 2026100905, 0).unwrap();
    for _ in 0..16 {
        p.step(false).unwrap();
    }
    let before = p.state().unwrap();
    assert_eq!(before["cells"].as_array().unwrap().len(), 1);
    assert_eq!(before["cells"][0]["site"], Value::Null);
    assert_eq!(before["cells"][0]["state"]["core"]["tick"], 16);
    assert!(p.step(false).is_err());
    assert_eq!(p.state().unwrap(), before);
    assert_eq!(
        World::restore(p.checkpoint().unwrap())
            .unwrap()
            .state()
            .unwrap(),
        before
    );
}
