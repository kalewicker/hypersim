//! Offline arithmetic/engineering entry point. No simulator or campaign launch.
use hg_train::ere::{categorical as c, complete_admission as a};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Batch {
    id: String,
    theta: Vec<f32>,
    alpha: f32,
    expected_draws: usize,
    samples: Vec<c::Sample>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScoreCase {
    id: String,
    theta: Vec<f32>,
    emission: Value,
    native_score: Option<f64>,
    reason: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleLeaf {
    selection: c::Selection,
    active: f64,
    eligible: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    profile: String,
    purpose: String,
    catalogue_sha256: String,
    batches: Vec<Batch>,
    score_cases: Vec<ScoreCase>,
    oracle_contexts: Vec<Vec<f32>>,
    oracle_leaves: Vec<OracleLeaf>,
}
// Keep the legacy admission helpers crate-private; this offline binary owns its
// file hashing and does not widen the admission API.
fn digest(path: &Path) -> Result<String, String> {
    let output = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("file hashing failed".into());
    }
    String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
        .map(String::from)
        .ok_or("empty file hash".into())
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: ere-categorical-adapter CATALOGUE.json REQUEST.json FRESH_RESULT.json (offline only)".into());
    }
    let output = Path::new(&args[3]);
    if output.exists() {
        return Err("fresh result file required".into());
    }
    let request: Request = serde_json::from_slice(&fs::read(&args[2]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if request.profile != c::PROFILE || request.purpose != "engineering_receipt_reconstruction" {
        return Err("engineering-only adapter request required".into());
    }
    let catalogue_sha256 = digest(Path::new(&args[1]))?;
    if catalogue_sha256 != request.catalogue_sha256 || catalogue_sha256 != a::CATALOGUE_SHA {
        return Err("pinned complete catalogue hash required".into());
    }
    let cat = serde_json::from_slice(&fs::read(&args[1]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let model = c::Model::from_catalogue(&cat)?;
    let batches=request.batches.iter().map(|b|Ok(json!({"id":b.id,"estimate":c::estimate(&model,&b.theta,&b.samples,b.expected_draws,b.alpha)?}))).collect::<Result<Vec<_>,String>>()?;
    let score_cases=request.score_cases.iter().map(|s| {
        let selection=model.selection_from_emission(&s.emission)?;
        let score=model.score(&s.theta,selection)?;
        let utility=hg_train::ere::feasibility_training::utility(s.native_score,s.reason.as_deref())?;
        Ok(json!({"id":s.id,"score":score,"native_score":s.native_score,"reason":s.reason,"utility":utility}))
    }).collect::<Result<Vec<_>,String>>()?;
    let active: Vec<_> = request
        .oracle_leaves
        .iter()
        .map(|v| (v.selection, v.active))
        .collect();
    let eligible: Vec<_> = request
        .oracle_leaves
        .iter()
        .map(|v| (v.selection, v.eligible))
        .collect();
    let oracles=request.oracle_contexts.iter().map(|theta|Ok(json!({"theta":theta,"active":model.expectation(theta,&active)?,"eligible":model.expectation(theta,&eligible)?}))).collect::<Result<Vec<_>,String>>()?;
    let result = json!({"profile":c::PROFILE,"classification":"engineering_arithmetic_only","native_steps_run":0,"grammar_compilations":0,
        "training_updates":0,"training_admitted":false,"catalogue_sha256":catalogue_sha256,"request_sha256":digest(Path::new(&args[2]))?,
        "batches":batches,"score_cases":score_cases,"oracles":oracles});
    // create_new protects a racing existing target too; never overwrite evidence.
    let bytes = serde_json::to_vec(&result).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    println!(
        "offline adapter result: {} (zero native work)",
        output.display()
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
