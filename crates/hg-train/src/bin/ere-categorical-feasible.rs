//! Synthetic offline engineering only: no Engine, grammar compilation or training.
use hg_train::ere::{categorical as c, categorical_feasible as f, complete_admission as a};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    theta: Vec<f32>,
    utility: f::Panel,
    risk: f::Panel,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    profile: String,
    purpose: String,
    cases: Vec<Case>,
}
fn digest(path: &Path) -> Result<String, String> {
    let done = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !done.status.success() {
        return Err("hashing failed".into());
    }
    String::from_utf8(done.stdout)
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
        .map(String::from)
        .ok_or("missing digest".into())
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: ere-categorical-feasible CATALOGUE REQUEST FRESH_RESULT (synthetic offline only)".into());
    }
    if Path::new(&args[3]).exists() {
        return Err("fresh result required".into());
    }
    if digest(Path::new(&args[1]))? != a::CATALOGUE_SHA {
        return Err("pinned complete catalogue required".into());
    }
    let request: Request = serde_json::from_slice(&fs::read(&args[2]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if request.profile != f::PROFILE || request.purpose != "engineering_synthetic_only" {
        return Err("synthetic engineering-only request required".into());
    }
    let catalogue: Value = serde_json::from_slice(&fs::read(&args[1]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let model = c::Model::from_catalogue(&catalogue)?;
    let cases = request
        .cases
        .iter()
        .map(|q| Ok(json!({"id":q.id,"proposal":f::propose(&model,&q.theta,&q.utility,&q.risk)?})))
        .collect::<Result<Vec<_>, String>>()?;
    let result = json!({"profile":f::PROFILE,"classification":"engineering_synthetic_only","cases":cases,
        "catalogue_sha256":digest(Path::new(&args[1]))?,"request_sha256":digest(Path::new(&args[2]))?,
        "native_steps_run":0,"grammar_compilations":0,"training_updates":0,"training_admitted":false});
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
