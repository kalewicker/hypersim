use hg_cumulative_readiness::{
    native_work,
    population::regulation::world::journal::{self, PROFILE},
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
struct Work;
impl Drop for Work {
    fn drop(&mut self) {
        println!("native_test_rewrites={}", native_work());
    }
}
static NEXT: AtomicUsize = AtomicUsize::new(0);
fn root() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "cumulative-journal-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    p
}
fn request() -> Value {
    json!({"profile":PROFILE,"mode":"worlds","world_contract":serde_json::from_str::<Value>(include_str!("../integration_contract.json")).unwrap(),"assay_requests":{"profile":"cumulative-lineage-intervention-readiness-v1","assays":[{"id":0,"role":"intact","stages":[0,1,2]},{"id":1,"role":"intact","stages":[0,1]}]},"source_identity":{"binary_sha256":"0".repeat(64),"source_sha256":"0".repeat(64)},"native_budget":6000,"execution_allowance":6000,"journal_byte_allowance":33554432,"fault":null,"source_dir":null,"source_head_sha256":null,"replay":null})
}
fn run(p: &Path, name: &str, r: &Value) -> Result<Value, String> {
    let input = p.join(format!("{name}-input.json"));
    fs::write(&input, serde_json::to_vec(r).unwrap()).unwrap();
    journal::run(&input, &p.join(name))
}
fn head(p: &Path) -> String {
    let mut files = fs::read_dir(p)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| !p.file_name().unwrap().to_str().unwrap().starts_with('.'))
        .collect::<Vec<_>>();
    files.sort();
    journal::sha(&fs::read(files.last().unwrap()).unwrap())
}
fn continuation(r: &Value, p: &Path, mode: &str) -> Value {
    let mut v = r.clone();
    v["mode"] = json!(mode);
    v["fault"] = Value::Null;
    v["source_dir"] = json!(p);
    v["source_head_sha256"] = json!(head(p));
    v
}
#[test]
fn immutable_fresh_publication_and_hash_vectors() {
    let _work = Work;
    assert_eq!(
        journal::sha(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let p = root();
    let mut r = request();
    r["native_budget"] = json!(1);
    assert_eq!(run(&p, "a", &r).unwrap()["status"], "budget_stop");
    let before = fs::read(p.join("a/00000000.json")).unwrap();
    assert!(run(&p, "a", &r).is_err());
    assert_eq!(before, fs::read(p.join("a/00000000.json")).unwrap());
    assert_eq!(native_work(), 0);
}
#[test]
fn committed_world_boundary_continues_same_denominator() {
    let _work = Work;
    let p = root();
    let mut r = request();
    r["fault"] = json!({"case":0,"tick":1,"point":"after_result"});
    assert!(run(&p, "a", &r).is_err());
    let v = continuation(&r, &p.join("a"), "world_continue");
    let out = run(&p, "b", &v).unwrap();
    assert_eq!(out["status"], "complete");
    assert_eq!(out["work_total"], 151);
    assert_eq!(native_work(), 151);
    let values = journal::read(&p.join("b"), &head(&p.join("b"))).unwrap();
    assert_eq!(values[1]["cases"][0]["rows"].as_array().unwrap().len(), 2);
}
#[test]
fn pending_and_torn_sources_refuse_before_additional_work() {
    let _work = Work;
    let p = root();
    for (i, point) in ["before_result", "torn_result"].iter().enumerate() {
        let name = format!("a{i}");
        let mut r = request();
        r["fault"] = json!({"case":0,"tick":1,"point":point});
        assert!(run(&p, &name, &r).is_err());
        let n = native_work();
        let v = continuation(&r, &p.join(&name), "world_continue");
        assert!(run(&p, &format!("b{i}"), &v).is_err());
        assert_eq!(native_work(), n);
        assert!(!p.join(format!("b{i}")).exists());
    }
}
#[test]
fn continuation_cannot_change_identity_budget_or_input() {
    let _work = Work;
    let p = root();
    let mut r = request();
    r["native_budget"] = json!(1);
    run(&p, "a", &r).unwrap();
    let v = continuation(&r, &p.join("a"), "world_continue");
    for (i, k) in [
        "native_budget",
        "source_identity",
        "assay_requests",
        "world_contract",
    ]
    .iter()
    .enumerate()
    {
        let mut bad = v.clone();
        bad[*k] = if *k == "native_budget" {
            json!(6000)
        } else {
            Value::Null
        };
        assert!(run(&p, &format!("b{i}"), &bad).is_err());
        assert!(!p.join(format!("b{i}")).exists());
    }
    assert_eq!(native_work(), 0);
}
#[test]
fn invalid_late_assay_rejects_entire_request_before_work() {
    let _work = Work;
    let p = root();
    let mut r = request();
    r["mode"] = json!("assays");
    r["assay_requests"]["assays"][1]["stages"] = json!([9]);
    assert!(run(&p, "a", &r).is_err());
    assert!(!p.join("a").exists());
    assert_eq!(native_work(), 0);
}
#[test]
fn assay_boundary_preserves_global_request_order() {
    let _work = Work;
    let p = root();
    let mut r = request();
    r["mode"] = json!("assays");
    r["fault"] = json!({"case":0,"tick":0,"point":"after_result"});
    assert!(run(&p, "a", &r).is_err());
    let v = continuation(&r, &p.join("a"), "assay_continue");
    let out = run(&p, "b", &v).unwrap();
    assert_eq!(out["status"], "complete");
    let values = journal::read(&p.join("b"), &head(&p.join("b"))).unwrap();
    assert_eq!(values[1]["assays"].as_array().unwrap().len(), 1);
    let results = values
        .iter()
        .filter(|v| v["kind"] == "result")
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["index"], 1);
    assert_eq!(results[0]["native"]["results"][0]["request"]["id"], 0);
    assert_eq!(
        results[0]["native"]["results"][0]["request"]["stages"],
        json!([0, 1])
    );
}
#[test]
fn explicit_replay_preserves_source_prefix_and_charges_extra() {
    let _work = Work;
    let p = root();
    let r = request();
    assert_eq!(run(&p, "a", &r).unwrap()["native_rewrites"], 151);
    let mut v = continuation(&r, &p.join("a"), "world_replay");
    v["replay"] = json!({"case":11,"start_tick":2});
    let out = run(&p, "b", &v).unwrap();
    assert_eq!(out["status"], "complete");
    assert!(out["native_rewrites"].as_u64().unwrap() > 0);
    let source = journal::read(&p.join("a"), &head(&p.join("a"))).unwrap();
    let dest = journal::read(&p.join("b"), &head(&p.join("b"))).unwrap();
    let original = source
        .iter()
        .filter(|v| v["kind"] == "result" && v["case"] == 11 && v["tick"].as_u64().unwrap() >= 2)
        .map(|v| v["row"].clone())
        .collect::<Vec<_>>();
    let replay = dest
        .iter()
        .filter(|v| v["kind"] == "result")
        .map(|v| v["row"].clone())
        .collect::<Vec<_>>();
    assert_eq!(original, replay);
    assert_eq!(dest[1]["cases"].as_array().unwrap().len(), 12);
}
#[test]
fn changed_chain_and_extra_files_are_refused_without_work() {
    let _work = Work;
    let p = root();
    let mut r = request();
    r["native_budget"] = json!(1);
    run(&p, "a", &r).unwrap();
    let v = continuation(&r, &p.join("a"), "world_continue");
    fs::write(p.join("a/extra"), b"x").unwrap();
    assert!(run(&p, "b", &v).is_err());
    assert!(!p.join("b").exists());
    assert_eq!(native_work(), 0);
}
