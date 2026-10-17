//! Explicit gate-specific resource profile. Never relabels historical receipts.
use super::{experiments, pooled, semantic, POOLED_PROTOCOL};
use serde_json::{json, Value};
use std::path::Path;

pub const PROFILE: &str = "E2-pooled-classification400k50k-runtime50k-v1";
fn load(dir: &Path, name: &str) -> Result<Value, String> {
    serde_json::from_slice(
        &std::fs::read(dir.join(name)).map_err(|e| format!("missing {name}: {e}"))?,
    )
    .map_err(|e| e.to_string())
}
fn n(v: &Value, key: &str) -> u64 {
    v[key].as_u64().unwrap_or(0)
}
fn check(ok: bool, message: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("{PROFILE}: {message}"))
    }
}
fn downstream(v: &Value) -> bool {
    v["protocol"] == POOLED_PROTOCOL && v["match_budget_policy"] == "Pooled"
}

/// Admission for the newly authorized E2 grammar pilot and future explicit
/// consumers. Legacy campaign admission/defaults deliberately remain unchanged.
pub fn require_pooled_400k50k_gates(classification: &Path, gates: &Path) -> Result<Value, String> {
    let one = load(classification, "exp_r01.json")?;
    let samples = n(&one, "samples");
    let valid = n(&one, "valid");
    check(
        one["protocol"] == pooled::TRIAL_PROTOCOL
            && one["match_budget_policy"] == "Pooled"
            && n(&one, "global_work") == pooled::TRIAL_WORK as u64
            && n(&one, "global_results") == pooled::TRIAL_RESULTS as u64
            && n(&one, "rng_seed") == experiments::MUTATION_SEED,
        "classification protocol/resource identity mismatch",
    )?;
    check(
        one["pass"] == true
            && samples == 10_000
            && n(&one, "seeds") == 168
            && n(&one, "trajectory_steps") == 100
            && one["unmeasured"].as_u64() == Some(0)
            && valid + n(&one, "quarantined") == samples
            && n(&one, "neutral")
                + n(&one, "near_neutral")
                + n(&one, "divergent")
                + n(&one, "catastrophic")
                == valid
            && valid as f64 / samples as f64 >= 0.40
            && n(&one, "near_neutral") as f64 / samples as f64 >= 0.10
            && n(&one, "catastrophic") as f64 / samples as f64 <= 0.25
            && n(&one, "neutral_soundness_checked") >= 50
            && one["neutral_soundness_failures"].as_u64() == Some(0)
            && n(&one, "at_cap_samples") > 0,
        "full G-R1 classification failed",
    )?;
    semantic::validate_receipt(&one)?;
    let usage = &one["resource_usage"];
    check(
        [
            "budget_violations",
            "match_ceiling_steps",
            "graph_ceiling_steps",
        ]
        .iter()
        .all(|k| usage[k].as_u64() == Some(0))
            && n(usage, "attempted_steps") > 0
            && n(usage, "peak_step_work") <= pooled::TRIAL_WORK as u64
            && n(usage, "peak_step_results") <= pooled::TRIAL_RESULTS as u64,
        "classification resource accounting failed",
    )?;
    let two = load(gates, "exp_r02.json")?;
    let header = load(gates, "exp_r02_encoding.json")?;
    check(
        downstream(&header)
            && header["encoding"] == "AlternativeFamiliesV3"
            && n(&header, "global_work") == 2_000_000
            && n(&header, "global_results") == 100_000
            && n(&header, "trajectory_steps") == 1000
            && n(&header, "library_rules") == 45,
        "G-R2 profile mismatch",
    )?;
    let rows = two.as_array().ok_or("G-R2 rows missing")?;
    let identities: std::collections::BTreeSet<_> = rows
        .iter()
        .map(|r| {
            (
                r["file"].as_str().unwrap_or(""),
                r["name"].as_str().unwrap_or(""),
            )
        })
        .collect();
    check(
        rows.len() == 45
            && identities.len() == 45
            && identities
                .iter()
                .all(|(f, name)| !f.is_empty() && !name.is_empty())
            && rows.iter().any(|r| r["status"] == "identical")
            && rows.iter().all(|r| {
                if r["status"] == "identical" {
                    n(r, "steps") == 1000
                } else {
                    r["status"]
                        .as_str()
                        .is_some_and(|s| s.starts_with("outside-envelope:"))
                        && r["steps"].as_u64() == Some(0)
                }
            }),
        "full G-R2 failed",
    )?;
    let three = load(gates, "exp_r03.json")?;
    check(
        downstream(&three)
            && [
                "pass",
                "graph_hashes_equal",
                "canonical_csv_equal",
                "rng_states_equal",
                "rng_consumed",
                "every_step_advanced",
            ]
            .iter()
            .all(|k| three[k] == true)
            && ["steps", "completed_steps", "a0_world_step", "a1_world_step"]
                .iter()
                .all(|k| n(&three, k) == 100_000)
            && three["match_ceiling_bound"] == false
            && n(&three, "accepted_rewrites") > 0
            && n(&three, "attempted_matches") > 0
            && n(&three, "canonical_csv_bytes") > 0
            && n(&three, "global_work") == 50_000
            && n(&three, "global_results") == 50_000
            && n(&three, "peak_step_work") <= 50_000
            && n(&three, "peak_step_results") <= 50_000,
        "full non-vacuous G-R3 failed",
    )?;
    let f = load(gates, "fixture_gates.json")?;
    check(
        downstream(&f)
            && ["pass", "gr0", "gr4", "gr5", "gr6"]
                .iter()
                .all(|k| f[k] == true)
            && n(&f, "rcf_pairs") == 40
            && f["quarantine_by_validator"] == json!([1, 1, 1, 1, 1, 1, 1, 1, 1])
            && [
                "protection_steps",
                "protection_completed_steps",
                "protection_world_step",
            ]
            .iter()
            .all(|k| n(&f, k) == 10_000)
            && n(&f, "protection_accepted_rewrites") > 0
            && n(&f, "meta_editor_events") > 0
            && n(&f, "global_work") == 50_000
            && n(&f, "global_results") == 50_000
            && n(&f, "peak_step_work") <= 50_000
            && n(&f, "peak_step_results") <= 50_000
            && f["changed_rule_set_stable"] == false
            && ["changed_meta_included_ei", "changed_meta_excluded_ei"]
                .iter()
                .all(|k| f.get(*k).is_some_and(Value::is_null)),
        "G-R0/4/5/6 failed",
    )?;
    Ok(
        json!({"profile":PROFILE,"pass":true,"classification_dir":classification,
        "downstream_dir":gates,"classification_protocol":pooled::TRIAL_PROTOCOL,
        "downstream_protocol":POOLED_PROTOCOL,"match_budget_policy":"Pooled",
        "runtime_work":50_000,"runtime_results":50_000,
        "supported_rules":rows.iter().filter(|r|r["status"] == "identical").count(),
        "library_rules":45,"legacy_campaign_admission_changed":false}),
    )
}
