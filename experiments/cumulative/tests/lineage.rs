use hg_cumulative_readiness::{lineage, native_work};
use serde_json::{json, Value};
struct Work;
impl Drop for Work {
    fn drop(&mut self) {
        println!("native_test_rewrites={}", native_work());
    }
}
fn requests(programs: &[(&str, Vec<i64>)]) -> Value {
    json!({"profile":lineage::PROFILE,"assays":programs.iter().enumerate().map(|(id,(role,stages))|
        json!({"id":id,"role":role,"stages":stages})).collect::<Vec<_>>()})
}
#[test]
fn authored_descent_preserves_paid_origins_and_explicit_diagnostic_credit() {
    let _work = Work;
    let w = lineage::witness().unwrap();
    assert_eq!(w["controls"].as_array().unwrap().len(), 3);
    for c in w["controls"].as_array().unwrap() {
        assert_eq!(c["generations"].as_array().unwrap().len(), 4);
        for (i, g) in c["generations"].as_array().unwrap().iter().enumerate() {
            assert_eq!(g["initial"]["generation"], i);
            assert_eq!(g["final"]["tick"], 4);
            assert_eq!(g["initial"]["environment"], 512);
        }
    }
    assert_eq!(w["discovery_evidence"], false);
    assert_eq!(w["shared_economy_claim"], false);
}
#[test]
fn standardized_benefits_pay_storage_execution_and_copying() {
    let _work = Work;
    let a = lineage::assays(&requests(&[
        ("intact", vec![0]),
        ("intact", vec![0, 1]),
        ("intact", vec![0, 1, 2]),
    ]))
    .unwrap();
    let scores: Vec<_> = a["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["score"].as_i64().unwrap())
        .collect();
    assert_eq!(scores, vec![0, 7, 22]);
    for r in a["results"].as_array().unwrap() {
        assert_eq!(r["initial"]["arm"], "frozen");
        assert_eq!(r["final"]["transferred"], 8);
        assert!(r["receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["kind"] != "endogenous_edit"));
    }
}
#[test]
fn redundant_group_removal_and_padding_recover_dependency() {
    let _work = Work;
    let a = lineage::assays(&requests(&[
        ("intact", vec![0, 1, 1, 2]),
        ("single_remove_earlier", vec![0, 1, 2]),
        ("remove_earlier", vec![0, 2]),
        ("padded_remove_earlier", vec![0, -1, -1, 2]),
        ("sham", vec![0, 1, 1, 2]),
        ("restored", vec![0, 1, 1, 2]),
    ]))
    .unwrap();
    let v: Vec<_> = a["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["score"].as_i64().unwrap())
        .collect();
    assert_eq!(v, vec![17, 22, -5, -15, 17, 17]);
}
#[test]
fn whole_invalid_request_denominator_rejects_before_native_work() {
    let _work = Work;
    let before = native_work();
    for stages in [vec![], vec![5], vec![0; 9]] {
        assert!(lineage::assays(&requests(&[("intact", vec![0]), ("intact", stages)])).is_err());
    }
    let mut bad = requests(&[("intact", vec![0])]);
    bad["assays"][0]["role"] = json!("unknown");
    assert!(lineage::assays(&bad).is_err());
    assert_eq!(native_work(), before);
}
#[test]
fn unfunded_copy_remains_a_full_horizon_assay_outcome() {
    let _work = Work;
    let a = lineage::assays(&requests(&[("intact", vec![-1; 8])])).unwrap();
    assert_eq!(a["results"][0]["status"], "unfunded_copy");
    assert_eq!(a["results"][0]["final"]["tick"], 4);
    assert_eq!(a["results"][0]["final"]["transferred"], 0);
    assert_eq!(a["results"][0]["score"], -36);
}
