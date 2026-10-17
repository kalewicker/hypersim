//! Explicit engineering runner; no optimizer campaign or persisted admission token.
use super::complete_admission::{self as admission, Prepared, ReceiptPaths, Request, Res, PROFILE};
use hg_sim_core::ere::{signal, valid_grammar};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Default, Serialize)]
struct Counts {
    grammar_compilations: u64,
    optimizer_evaluation_steps: u64,
    precheck_evaluation_steps: u64,
    held_out_evaluation_steps: u64,
    bounded_evaluation_steps: Option<u64>,
    bounded_attempted_steps: Option<u64>,
}
impl Counts {
    fn zero() -> Self {
        Self {
            bounded_evaluation_steps: Some(0),
            bounded_attempted_steps: Some(0),
            ..Self::default()
        }
    }
}
fn write(path: &Path, value: &Value) -> Res<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
fn output_bytes(out: &Path) -> Res<u64> {
    admission::files(out)?
        .iter()
        .map(|r| {
            fs::metadata(out.join(r))
                .map(|m| m.len())
                .map_err(|e| e.to_string())
        })
        .sum()
}
fn fresh_output(paths: &ReceiptPaths, out: &Path) -> Res<()> {
    admission::check(
        !out.exists(),
        "fresh output directory required; no overwrite",
    )?;
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let absolute = parent
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("missing output directory name")?);
    for archive in paths.directories() {
        if let Ok(root) = archive.canonicalize() {
            admission::check(
                !absolute.starts_with(&root),
                "output cannot be placed inside a prerequisite archive",
            )?;
        }
    }
    fs::create_dir(out).map_err(|e| e.to_string())
}
fn snapshot(p: &Prepared, out: &Path) -> Res<Value> {
    let mut names: Vec<_> = p.pins().keys().cloned().collect();
    names.extend(
        [
            "crates/hg-train/src/ere.rs",
            "crates/hg-train/src/ere/complete_admission.rs",
            "crates/hg-train/src/ere/complete_runner.rs",
            "crates/hg-train/src/bin/ere-train.rs",
            "docs/ere_complete_admission_runner_v1.md",
            "configs/ere_complete_receipts_v1.json",
            "configs/ere_complete_bounded_request_v1.json",
            "scripts/audit_ere_complete_admission.py",
            "scripts/test_audit_ere_complete_admission.py",
        ]
        .into_iter()
        .map(String::from),
    );
    names.sort();
    names.dedup();
    let mut files = vec![];
    for rel in &names {
        let dst = out.join("source_snapshots").join(rel);
        fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::copy(p.repo().join(rel), &dst).map_err(|e| e.to_string())?;
        files.push(dst);
    }
    let pins: BTreeMap<_, _> = names.into_iter().zip(admission::hashes(&files)?).collect();
    let binary = out.join("executed_binary");
    fs::copy(std::env::current_exe().map_err(|e| e.to_string())?, &binary)
        .map_err(|e| e.to_string())?;
    Ok(
        json!({"sources":pins,"binary_sha256":admission::digest(&binary)?,"frozen_before_evaluation":true}),
    )
}
fn seal(out: &Path) -> Res<()> {
    let mut files = admission::files(out)?;
    files.remove("output_sha256.json");
    let paths: Vec<_> = files.iter().map(|r| out.join(r)).collect();
    let hashes: BTreeMap<_, _> = files.into_iter().zip(admission::hashes(&paths)?).collect();
    write(&out.join("output_sha256.json"), &json!(hashes))
}
fn status(
    out: &Path,
    manifest: &mut Value,
    state: &str,
    counts: &Counts,
    detail: Option<&str>,
) -> Res<()> {
    manifest["status"] = json!(state);
    manifest["counts"] = serde_json::to_value(counts).map_err(|e| e.to_string())?;
    manifest["reason"] = json!(detail);
    write(&out.join("manifest.json"), manifest)?;
    write(
        &out.join("status.json"),
        &json!({"profile":PROFILE,"status":state,"reason":detail,"counts":counts,
        "training_admitted":false,"full_training_authorized":false,"engineering_only":true}),
    )
}

pub fn preflight(paths: &ReceiptPaths, request: &Request, out: &Path) -> Res<()> {
    run(paths, request, out, false)
}
pub fn run_bounded(paths: &ReceiptPaths, request: &Request, out: &Path) -> Res<()> {
    run(paths, request, out, true)
}

fn run(paths: &ReceiptPaths, request: &Request, out: &Path, execute: bool) -> Res<()> {
    fresh_output(paths, out)?;
    let mut counts = Counts::zero();
    let mut manifest = json!({"profile":PROFILE,"run_mode":if execute{"bounded_engineering"}else{"preflight"},
        "status":"validating","request":request,"receipt_paths":paths,"training_admitted":false,
        "full_training_authorized":false,"engineering_only":true,"counts":counts});
    status(out, &mut manifest, "validating", &counts, None)?;
    let result: Res<()> = (|| {
        let prepared = admission::prepare(paths, request)?;
        write(&out.join("admission.json"), prepared.evidence())?;
        manifest["receipt_paths"] = json!({"deterministic":prepared.evidence()["receipts"]["deterministic"]["archive"],
            "long_gates":prepared.evidence()["receipts"]["long_gates"]["archive"],"closure":prepared.evidence()["receipts"]["closure"]["archive"],
            "grammar":prepared.evidence()["receipts"]["grammar"]["archive"],"runtime":prepared.evidence()["receipts"]["runtime"]["archive"]});
        manifest["snapshot"] = snapshot(&prepared, out)?;
        if !execute {
            status(
                out,
                &mut manifest,
                "ready_for_bounded_execution",
                &counts,
                None,
            )?;
            return Ok(());
        }
        let start = Instant::now();
        let deadline = start + Duration::from_secs(request.wall_seconds);
        prepared.recheck()?;
        counts.grammar_compilations = 1;
        status(out, &mut manifest, "compiling", &counts, None)?;
        let grammar = valid_grammar::compile()?;
        manifest["grammar_compile_seconds"] = json!(start.elapsed().as_secs_f64());
        write(&out.join("grammar_catalogue.json"), &grammar.catalogue())?;
        // Catalogue identity is defined over compact canonical serde output, not display formatting.
        let compact = out.join("grammar_catalogue.compact.json");
        fs::write(
            &compact,
            serde_json::to_vec(&grammar.catalogue()).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        admission::check(
            admission::digest(&compact)? == admission::CATALOGUE_SHA,
            "recompiled catalogue identity changed",
        )?;
        admission::check(
            Instant::now() < deadline && output_bytes(out)? < request.output_cap_bytes,
            "bounded engineering resource ceiling before evaluation",
        )?;
        let emission = grammar.emit(&request.parameters, request.emission_seed)?;
        // Long setup cannot turn an earlier readiness check into a stale execution permit.
        prepared.recheck()?;
        manifest["execution_boundary_rechecked"] = json!(true);
        admission::check(
            Instant::now() < deadline,
            "bounded engineering wall ceiling before evaluation",
        )?;
        counts.bounded_attempted_steps = None;
        counts.bounded_evaluation_steps = None;
        status(out, &mut manifest, "evaluating", &counts, None)?;
        let mut evaluation = signal::evaluate_with_budget(
            &emission,
            request.physics_seed,
            deadline,
            admission::WORK,
            admission::RESULTS,
        )?;
        evaluation["runtime_match_budget"] =
            json!({"policy":"Pooled","work":admission::WORK,"results":admission::RESULTS});
        counts.bounded_evaluation_steps = evaluation["advancing_steps"].as_u64();
        counts.bounded_attempted_steps = evaluation["attempted_steps"].as_u64();
        let row =
            json!({"engineering_only":true,"emission":emission.record,"evaluation":evaluation});
        let wire = serde_json::to_vec_pretty(&row).map_err(|e| e.to_string())?;
        admission::check(
            output_bytes(out)? + (wire.len() as u64) < request.output_cap_bytes,
            "bounded engineering output ceiling",
        )?;
        fs::write(out.join("evaluation.json"), wire).map_err(|e| e.to_string())?;
        manifest["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
        manifest["bounded_evaluation_complete"] = json!(evaluation["score"].is_number());
        status(
            out,
            &mut manifest,
            if evaluation["score"].is_number() {
                "complete"
            } else {
                "complete_with_null"
            },
            &counts,
            None,
        )?;
        Ok(())
    })();
    if let Err(error) = &result {
        let state = if counts.grammar_compilations == 0 {
            "blocked_before_evaluation"
        } else {
            "incomplete"
        };
        status(out, &mut manifest, state, &counts, Some(error))?;
    }
    admission::check(
        output_bytes(out)? < admission::OUTPUT_CAP,
        "final engineering output ceiling",
    )?;
    seal(out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn paths(root: &Path) -> ReceiptPaths {
        let r = root.join("output");
        ReceiptPaths {
            deterministic: r.join("ere_complete_gates_fresh_20260926"),
            long_gates: r.join("ere_complete_long_gates_full_20260927"),
            closure: r.join("ere_complete_coherent_closure_full_20260929"),
            grammar: r.join("ere_valid_production_grammar_20260929"),
            runtime: r.join("ere_runtime50k_signal_panel_20260930"),
        }
    }
    fn root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
    }
    fn assert_zero(out: &Path) {
        let s = admission::load(&out.join("status.json")).unwrap();
        assert_eq!(s["status"], "blocked_before_evaluation");
        for name in [
            "grammar_compilations",
            "optimizer_evaluation_steps",
            "precheck_evaluation_steps",
            "held_out_evaluation_steps",
            "bounded_evaluation_steps",
            "bounded_attempted_steps",
        ] {
            assert_eq!(s["counts"][name], 0);
        }
        assert_eq!(s["training_admitted"], false);
        assert!(!out.join("evaluation.json").exists());
        assert!(!out.join("grammar_catalogue.json").exists());
    }
    #[test]
    fn full_training_and_wrong_budget_refuse_before_any_work() {
        let tmp = tempfile::tempdir().unwrap();
        let mut request = Request::engineering(7, 101);
        request.horizon = 10000;
        request.optimizer_generations = 20;
        let out = tmp.path().join("training");
        assert!(run_bounded(&paths(&root()), &request, &out).is_err());
        assert_zero(&out);
        request = Request::engineering(7, 101);
        request.work = 400000;
        let out = tmp.path().join("wrong_budget");
        assert!(run_bounded(&paths(&root()), &request, &out).is_err());
        assert_zero(&out);
    }
    #[test]
    fn missing_and_incompatible_evidence_refuse_with_truthful_zero_counts() {
        let tmp = tempfile::tempdir().unwrap();
        let mut p = paths(&root());
        p.deterministic = tmp.path().join("missing");
        let out = tmp.path().join("missing_out");
        assert!(run_bounded(&p, &Request::engineering(7, 101), &out).is_err());
        assert_zero(&out);
        p = paths(&root());
        p.deterministic = root().join("output/ere_complete_gates_20260926");
        let out = tmp.path().join("wrong_out");
        assert!(run_bounded(&p, &Request::engineering(7, 101), &out).is_err());
        assert_zero(&out);
    }
    #[test]
    fn existing_output_and_prerequisite_directory_are_never_written() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("existing");
        fs::create_dir(&out).unwrap();
        fs::write(out.join("sentinel"), "preserve").unwrap();
        let p = paths(&root());
        assert!(preflight(&p, &Request::engineering(7, 101), &out).is_err());
        assert_eq!(
            fs::read_to_string(out.join("sentinel")).unwrap(),
            "preserve"
        );
        let inside = p.grammar.join("must_not_create_admission_output");
        assert!(preflight(&p, &Request::engineering(7, 101), &inside).is_err());
        assert!(!inside.exists());
    }
}
