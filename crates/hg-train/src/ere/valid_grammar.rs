//! Build/export a no-physics grammar. This does not launch optimizer prechecks.
pub use hg_sim_core::ere::valid_grammar::{compile, Grammar, OPERATORS, PARAMS, PROFILE};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
type Res<T> = Result<T, String>;
fn digest(p: &Path) -> Res<String> {
    let r = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(p)
        .output()
        .map_err(|e| e.to_string())?;
    if !r.status.success() {
        return Err(format!("hash failed {}", p.display()));
    }
    Ok(String::from_utf8(r.stdout)
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
        .ok_or("empty digest")?
        .into())
}
fn write(p: &Path, v: &Value) -> Res<()> {
    fs::write(p, serde_json::to_vec(v).unwrap()).map_err(|e| e.to_string())
}
fn sources(dir: &Path, paths: &mut Vec<PathBuf>) -> Res<()> {
    for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.is_dir() {
            sources(&p, paths)?;
        } else if p.extension().is_some_and(|x| x == "rs" || x == "toml") {
            paths.push(p);
        }
    }
    Ok(())
}
pub fn export(out: &Path) -> Res<()> {
    fs::create_dir(out).map_err(|e| format!("fresh grammar output required: {e}"))?;
    let result = (|| {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let mut paths = vec![
            repo.join("Cargo.toml"),
            repo.join("Cargo.lock"),
            repo.join("docs/ere_valid_production_grammar_v1.md"),
            repo.join("scripts/audit_ere_valid_grammar.py"),
        ];
        for c in [
            "hg-core",
            "hg-engine",
            "hg-observe",
            "hg-rules",
            "hg-meta",
            "hg-sim-core",
            "hg-train",
        ] {
            sources(&repo.join("crates").join(c), &mut paths)?;
        }
        paths.sort();
        let mut hashes = serde_json::Map::new();
        for p in paths {
            let rel = p.strip_prefix(&repo).map_err(|e| e.to_string())?;
            let dst = out.join("source_snapshots").join(rel);
            fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::copy(&p, &dst).map_err(|e| e.to_string())?;
            hashes.insert(rel.to_string_lossy().into(), json!(digest(&dst)?));
        }
        let binary = out.join("executed_binary");
        fs::copy(std::env::current_exe().map_err(|e| e.to_string())?, &binary)
            .map_err(|e| e.to_string())?;
        let mut manifest = json!({"profile":PROFILE,"status":"compiling","sources":hashes,"binary_sha256":digest(&binary)?,"frozen_before_compilation":true,"engine_steps":0,"optimizer_rollouts":0,"training_admitted":false,"evidence_scope":"engineering grammar compilation only"});
        write(&out.join("manifest.json"), &manifest)?;
        let start = Instant::now();
        let grammar = compile()?;
        let catalogue = grammar.catalogue();
        write(&out.join("catalogue.json"), &catalogue)?;
        let mut rows = vec![];
        for seed in 0..64 {
            rows.push(grammar.emit(&vec![0.; PARAMS], seed)?.record);
        }
        write(&out.join("emission_checks.json"), &json!(rows))?;
        manifest["status"] = json!("complete");
        manifest["compile_and_emission_check_seconds"] = json!(start.elapsed().as_secs_f64());
        manifest["catalogue_sha256"] = json!(digest(&out.join("catalogue.json"))?);
        manifest["emission_checks_sha256"] = json!(digest(&out.join("emission_checks.json"))?);
        write(&out.join("manifest.json"), &manifest)?;
        println!("grammar exported to {} (no Engine steps)", out.display());
        Ok(())
    })();
    if let Err(e) = &result {
        write(
            &out.join("failure.json"),
            &json!({"error":e,"status":"incomplete","training_admitted":false}),
        )?;
    }
    result
}
