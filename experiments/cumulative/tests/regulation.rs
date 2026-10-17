use hg_cumulative_readiness::{
    native_work,
    population::regulation::{self, RegulatedCell},
};
use serde_json::{json, Value};
struct Work;
impl Drop for Work {
    fn drop(&mut self) {
        println!("native_test_rewrites={}", native_work());
    }
}
fn spec() -> Value {
    json!({"stages":[0,1],"enabled":1,"mask":31,"arm":"endogenous","capital":40,"reservoir":0})
}

#[test]
fn all_registered_routes_execute_with_closed_admission() {
    let _work = Work;
    let f = regulation::fixture().unwrap();
    assert_eq!(f["native_rewrites"], 108);
    assert_eq!(f["cases"].as_array().unwrap().len(), 17);
    for k in [
        "readiness_gate_complete",
        "scientific_acceptance",
        "evolution_demonstrated",
        "campaign_launch_authorized",
    ] {
        assert_eq!(f[k], false);
    }
    assert_eq!(f["continuation"]["original"], f["continuation"]["restored"]);
}
#[test]
fn disabled_and_masked_refusals_do_not_spend_energy_or_rng() {
    let _work = Work;
    for (key, value) in [("enabled", 0), ("mask", 0)] {
        let mut s = spec();
        s[key] = json!(value);
        let mut c = RegulatedCell::from_case(&s, 9, 0).unwrap();
        let mut before = c.state().unwrap();
        assert!(!c.edit(false, None, false).unwrap());
        before["attempts"] = json!(1);
        assert_eq!(before, c.state().unwrap());
        assert_eq!(c.frames.len(), 1);
    }
}
#[test]
fn paid_native_policy_controls_permission_and_full_inheritance() {
    let _work = Work;
    for arm in ["endogenous", "nonheritable", "external", "frozen"] {
        let mut s = spec();
        s["arm"] = json!(arm);
        let mut c = RegulatedCell::from_case(&s, 9, 0).unwrap();
        if arm == "endogenous" || arm == "nonheritable" {
            assert!(c.edit(false, Some((3, 0)), false).unwrap());
        }
        let mut child = c.birth().unwrap().unwrap();
        let state = child.state().unwrap();
        assert_eq!(state["core"]["capital"], 8);
        assert_eq!(state["reservoir"], 0);
        assert_eq!(state["core"]["environment"], 0);
        if arm == "endogenous" {
            assert_eq!(state["program"]["enabled"], 0);
            assert!(!child.edit(false, None, false).unwrap());
        }
        if arm == "nonheritable" {
            assert_eq!(state["program"], state["baseline"]);
            assert!(child.edit(false, Some((3, 0)), false).unwrap());
        }
        if arm == "external" {
            assert!(!child.edit(false, None, false).unwrap());
            assert!(child.edit(true, Some((3, 0)), false).unwrap());
        }
        if arm == "frozen" {
            assert!(!child.edit(false, None, false).unwrap());
            assert!(!child.edit(true, None, false).unwrap());
        }
    }
}
#[test]
fn deletion_holes_survive_checkpoint_paid_copy_and_new_serials() {
    let _work = Work;
    let mut s = spec();
    s["stages"] = json!([0, 1, 2]);
    s["capital"] = json!(64);
    let mut c = RegulatedCell::from_case(&s, 9, 0).unwrap();
    assert!(c.edit(false, Some((2, 1)), false).unwrap());
    assert!(c.edit(false, Some((0, 0)), false).unwrap());
    assert_eq!(
        c.program()
            .unwrap()
            .genes
            .iter()
            .map(|g| g.0)
            .collect::<Vec<_>>(),
        vec![0, 2, 3]
    );
    let mut restored = RegulatedCell::restore(c.checkpoint().unwrap()).unwrap();
    let child = c.birth().unwrap().unwrap();
    let child_b = restored.birth().unwrap().unwrap();
    assert_eq!(child.state().unwrap(), child_b.state().unwrap());
    assert_eq!(child.program().unwrap().next_serial, 4);
    c.tick(false).unwrap();
    restored.tick(true).unwrap();
    assert_eq!(c.state().unwrap(), restored.state().unwrap());
    assert_eq!(
        c.frames[c.frames.len() - 3..],
        restored.frames[restored.frames.len() - 3..]
    );
}
#[test]
fn malformed_checkpoints_and_domains_reject_before_native_work() {
    let _work = Work;
    let before = native_work();
    for (key, value) in [
        ("enabled", json!(2)),
        ("mask", json!(32)),
        ("capital", json!(65)),
        ("reservoir", json!(17)),
        ("stages", json!([5])),
        ("next_serial", json!(1)),
    ] {
        let mut s = spec();
        s[key] = value;
        assert!(RegulatedCell::from_case(&s, 9, 0).is_err());
    }
    let cp = RegulatedCell::from_case(&spec(), 9, 0)
        .unwrap()
        .checkpoint()
        .unwrap();
    let mut bad = cp.clone();
    bad["state"]["program"]["mask"] = json!(0);
    assert!(RegulatedCell::restore(bad).is_err());
    let mut bad = cp.clone();
    bad["graph"]["label_index"]["1001"] = json!([]);
    assert!(RegulatedCell::restore(bad).is_err());
    let mut bad = cp.clone();
    bad["state"]["core"]["allocator"][0] = json!([1]);
    assert!(RegulatedCell::restore(bad).is_err());
    let mut bad = cp;
    bad["extra"] = json!(1);
    assert!(RegulatedCell::restore(bad).is_err());
    assert_eq!(native_work(), before);
}
#[test]
fn dead_ticks_and_caps_remain_explicit_bounded_routes() {
    let _work = Work;
    let mut s = spec();
    s["capital"] = json!(1);
    s["enabled"] = json!(0);
    let mut c = RegulatedCell::from_case(&s, 9, 0).unwrap();
    for _ in 0..16 {
        c.tick(false).unwrap();
    }
    let before = c.state().unwrap();
    assert_eq!(before["alive"], false);
    assert_eq!(before["core"]["tick"], 16);
    assert!(c.tick(false).is_err());
    assert_eq!(c.state().unwrap(), before);
    for _ in 16..32 {
        assert!(!c.edit(false, None, false).unwrap());
    }
    let before = c.state().unwrap();
    assert!(c.edit(false, None, false).is_err());
    assert_eq!(c.state().unwrap(), before);
    assert!(c.birth().unwrap().is_none());
    let mut early = RegulatedCell::from_case(&s, 9, 0).unwrap();
    for _ in 0..32 {
        assert!(!early.edit(false, None, false).unwrap());
    }
    let before = early.state().unwrap();
    assert!(early.tick(false).is_err());
    assert_eq!(early.state().unwrap(), before);
}
