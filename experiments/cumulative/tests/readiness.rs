use hg_core::PropertyValue;
use hg_cumulative_readiness::{Arm, Cell, CAPITAL, EDIT_MODE, MAX_GENES};
use serde_json::json;

struct Work;
impl Drop for Work {
    fn drop(&mut self) {
        println!(
            "native_test_rewrites={}",
            hg_cumulative_readiness::native_work()
        );
    }
}

#[test]
fn native_pathways_have_net_headroom_and_dependency() {
    let _work = Work;
    let mut gains = Vec::new();
    for stages in [
        vec![0],
        vec![0, 1],
        vec![0, 1, 2],
        vec![0, 2],
        vec![0, 1, 2, 0],
    ] {
        let mut c = Cell::new(&stages, Arm::Frozen, 31).unwrap();
        let before = c.capital().unwrap();
        c.cycle(false).unwrap();
        gains.push(c.capital().unwrap() - before);
        c.validate().unwrap();
    }
    assert_eq!(gains, vec![1, 3, 7, 0, 6]);
}

#[test]
fn edits_are_native_paid_and_new_genes_execute_after_decode() {
    let _work = Work;
    let mut c = Cell::new(&[0], Arm::Endogenous, 31).unwrap();
    c.graph
        .get_node_mut(c.editor)
        .unwrap()
        .set_property(EDIT_MODE, PropertyValue::Int(1));
    c.edit(false, Some(0)).unwrap();
    c.edit(false, Some(1)).unwrap();
    assert_eq!(c.genome().unwrap().stages, vec![0, 1, 2]);
    assert_eq!(c.capital().unwrap(), 36);
    c.cycle(false).unwrap();
    assert_eq!(c.capital().unwrap(), 43);
    assert_eq!(
        c.receipts
            .iter()
            .filter(|r| r["kind"] == "endogenous_edit")
            .count(),
        2
    );
}

#[test]
fn unfunded_edit_copy_and_capacity_rejections_preserve_exact_state() {
    let _work = Work;
    let mut c = Cell::new(&[0], Arm::Endogenous, 31).unwrap();
    c.graph
        .get_node_mut(c.wallet)
        .unwrap()
        .set_property(CAPITAL, PropertyValue::Int(1));
    c.dissipated = 39;
    c.validate().unwrap();
    let before = c.state().unwrap();
    assert!(c.edit(false, None).is_err());
    assert!(c.reproduce(1).is_err());
    assert_eq!(c.state().unwrap(), before);
    assert!(c.receipts.is_empty());
    let mut capped = Cell::new(&vec![0; MAX_GENES], Arm::Endogenous, 31).unwrap();
    let before = capped.state().unwrap();
    assert!(capped.edit(false, None).is_err());
    assert_eq!(capped.state().unwrap(), before);
}

#[test]
fn inheritance_controls_retain_costs_and_ancestry() {
    let _work = Work;
    for arm in [Arm::Endogenous, Arm::Nonheritable] {
        let mut c = Cell::new(&[0], arm, 31).unwrap();
        c.graph
            .get_node_mut(c.editor)
            .unwrap()
            .set_property(EDIT_MODE, PropertyValue::Int(1));
        c.edit(false, Some(0)).unwrap();
        let before = c.capital().unwrap();
        let child = c.reproduce(11).unwrap();
        assert_eq!(before - c.capital().unwrap(), 13);
        assert_eq!(child.capital().unwrap(), 8);
        assert_eq!(child.parent, Some(0));
        assert_eq!(
            child.genome().unwrap().stages,
            if arm == Arm::Nonheritable {
                vec![0]
            } else {
                vec![0, 1]
            }
        );
        c.validate().unwrap();
        child.validate().unwrap();
    }
    for arm in [Arm::Frozen, Arm::External] {
        let mut c = Cell::new(&[0], arm, 31).unwrap();
        let before = c.state().unwrap();
        assert!(c.edit(false, None).is_err());
        assert_eq!(c.state().unwrap(), before);
    }
}

#[test]
fn checkpoint_rejects_schema_energy_and_allocator_corruption() {
    let _work = Work;
    let c = Cell::new(&[0, 1], Arm::Endogenous, 31).unwrap();
    let saved = c.checkpoint().unwrap();
    for key in ["profile", "initial", "next_serial"] {
        let mut corrupt = saved.clone();
        corrupt[key] = if key == "profile" {
            json!("foreign")
        } else {
            json!(-1)
        };
        assert!(Cell::restore(corrupt).is_err(), "{key}");
    }
    let mut corrupt = saved.clone();
    corrupt["unknown"] = json!(true);
    assert!(Cell::restore(corrupt).is_err());
    let mut corrupt = saved;
    corrupt["allocator"] = json!([[99999], []]);
    assert!(Cell::restore(corrupt).is_err());
}

#[test]
fn snapshot_resume_preserves_rng_binding_and_allocation_after_recycling() {
    let _work = Work;
    let mut a = Cell::new(&[0, 1], Arm::Endogenous, 71).unwrap();
    a.cycle(false).unwrap();
    a.edit(false, None).unwrap();
    let bytes = serde_json::to_vec(&a.checkpoint().unwrap()).unwrap();
    let mut b = Cell::restore(serde_json::from_slice(&bytes).unwrap()).unwrap();
    a.receipts.clear();
    for _ in 0..2 {
        a.edit(false, None).unwrap();
        b.edit(false, None).unwrap();
        a.cycle(false).unwrap();
        b.cycle(false).unwrap();
    }
    assert_eq!(a.state().unwrap(), b.state().unwrap());
    assert_eq!(a.receipts, b.receipts);
}

#[test]
fn full_fixture_retains_negative_and_control_cases_without_admission() {
    let _work = Work;
    let f = hg_cumulative_readiness::fixture().unwrap();
    assert_eq!(f["cases"].as_array().unwrap().len(), 13);
    for flag in [
        "readiness_gate_complete",
        "scientific_acceptance",
        "evolution_demonstrated",
        "campaign_launch_authorized",
    ] {
        assert_eq!(f[flag], false);
    }
}
