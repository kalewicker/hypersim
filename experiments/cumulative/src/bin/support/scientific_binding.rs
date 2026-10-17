//! Scientific authorization and exact request projection; no native operations.
use super::admission;
use serde_json::{json, Value};
pub const CONTRACT: &str = include_str!("../../../scientific_launch_contract.json");
fn fields(v: &Value, keys: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|m| m.len() == keys.len() && keys.iter().all(|k| m.contains_key(*k)))
}
fn hash(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
pub fn validate(e: &Value, binary: &str, execute: bool) -> Result<Value, String> {
    if !fields(e, &["profile", "request", "authorization", "controller"])
        || e["profile"] != "cumulative-scientific-execution-binding-v1"
    {
        return Err("binding fields/profile".into());
    }
    let r = &e["request"];
    let admitted = admission::validate(r)?;
    let c = admission::parse(CONTRACT.as_bytes())?;
    if c["panel_sha256"] != admission::sha(admission::PANEL.as_bytes())
        || c["admission_sha256"] != admission::sha(admission::CONTRACT.as_bytes())
        || c["planning_sha256"] != admission::sha(admission::PLANNING.as_bytes())
    {
        return Err("compiled launch provenance".into());
    }
    let a = &e["authorization"];
    if !fields(
        a,
        &[
            "profile",
            "mode",
            "approved",
            "approval_reference",
            "campaign_id",
            "launch_sha256",
            "panel_sha256",
            "binary_sha256",
            "source_sha256",
            "limits",
        ],
    ) || a["profile"] != "cumulative-scientific-authorization-v1"
        || a["launch_sha256"] != admission::sha(CONTRACT.as_bytes())
        || a["panel_sha256"] != c["panel_sha256"]
        || a["binary_sha256"] != binary
        || a["limits"] != c["limits"]
        || !hash(&a["source_sha256"])
        || !hash(&a["campaign_id"])
    {
        return Err("authorization identity/domain".into());
    }
    let preview =
        a["mode"] == "preview" && a["approved"] == false && a["approval_reference"].is_null();
    let launch = a["mode"] == "launch"
        && a["approved"] == true
        && a["approval_reference"]
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 1024);
    if !(preview || launch) {
        return Err("explicit approval or inert preview required".into());
    }
    if execute && !launch {
        return Err("launch authorization required; preview cannot execute".into());
    }
    let t = &e["controller"];
    if !fields(
        t,
        &[
            "owner_sha256",
            "intent_sha256",
            "request_sha256",
            "work_bound",
        ],
    ) || !hash(&t["owner_sha256"])
        || !hash(&t["intent_sha256"])
        || t["request_sha256"] != admission::sha(&serde_json::to_vec(r).map_err(|e| e.to_string())?)
        || t["work_bound"].as_u64() != admitted["admitted"]["work_bound"].as_u64()
    {
        return Err("exact controller request/reservation binding".into());
    }
    if r["unit"] == "world" {
        let w = &r["world"];
        let mut spec = w.as_object().unwrap().clone();
        for k in ["block", "seed", "world_id"] {
            spec.remove(k);
        }
        spec.insert("name".into(), w["world_id"].clone());
        Ok(
            json!({"unit":"world","spec":spec,"seed":w["seed"],"items":[],"start":0,"end":w["ticks"],"execution_allowance":r["execution_allowance"]}),
        )
    } else {
        Ok(
            json!({"unit":"assays","spec":null,"seed":null,"items":r["registry"]["assays"],"start":r["range"]["start"],"end":r["range"]["end"],"execution_allowance":r["execution_allowance"]}),
        )
    }
}
