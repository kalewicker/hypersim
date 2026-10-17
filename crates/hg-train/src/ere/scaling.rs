//! Registered fixed-panel scaling diagnostic, never an optimizer/training path.
use super::complete_admission::{self as a, ReceiptPaths, Res};
use hg_sim_core::ere::{signal, valid_grammar};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-complete-scaling-v2";
const SIGNAL: &str = "crates/hg-sim-core/src/ere/signal.rs";
const ENGINEERING_SEAL: &str = "a988f131bebbf6c2e14393bf6c2226737908c628ca7111a1ca3ff1c66b4657fd";
pub const EXTRA: [&str; 14] = [
    "crates/hg-train/src/ere.rs",
    "crates/hg-train/src/ere/complete_admission.rs",
    "crates/hg-train/src/ere/complete_runner.rs",
    "crates/hg-train/src/bin/ere-train.rs",
    "docs/ere_complete_admission_runner_v1.md",
    "configs/ere_complete_receipts_v1.json",
    "configs/ere_complete_bounded_request_v1.json",
    "scripts/audit_ere_complete_admission.py",
    "scripts/test_audit_ere_complete_admission.py",
    "crates/hg-train/src/ere/scaling.rs",
    "docs/ere_complete_scaling_trial_v2.md",
    "configs/ere_complete_scaling_v2.json",
    "scripts/audit_ere_complete_scaling.py",
    "scripts/test_audit_ere_complete_scaling.py",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    pub profile: String,
    pub purpose: String,
    pub horizons: Vec<usize>,
    pub physics_seeds: Vec<u64>,
    pub emission_seeds: Vec<u64>,
    pub control_parents: Vec<usize>,
    pub random_draws: usize,
    pub parameters: Vec<f32>,
    pub work: usize,
    pub results: usize,
    pub wall_seconds: u64,
    pub cell_wall_seconds: u64,
    pub output_cap_bytes: u64,
    pub optimizer_generations: usize,
    pub held_out_evaluations: usize,
}
impl Design {
    pub fn registered() -> Self {
        Self {
            profile: PROFILE.into(),
            purpose: "fixed_panel_scaling".into(),
            horizons: vec![100, 1000, 10000],
            physics_seeds: vec![1361, 1471],
            emission_seeds: (0xE653000101..=0xE653000106).collect(),
            control_parents: vec![4, 5],
            random_draws: 4,
            parameters: vec![0.0; 183],
            work: 50000,
            results: 50000,
            wall_seconds: 1800,
            cell_wall_seconds: 120,
            output_cap_bytes: 1024 * 1024 * 1024,
            optimizer_generations: 0,
            held_out_evaluations: 0,
        }
    }
    pub fn validate(&self) -> Res<()> {
        a::check(
            *self == Self::registered(),
            "only the exact registered scaling design is authorized; training remains closed",
        )
    }
}
pub fn schedule(d: &Design) -> Value {
    let mut cells = vec![];
    for (production, &emission_seed) in d.emission_seeds.iter().enumerate() {
        for &physics_seed in &d.physics_seeds {
            for &horizon in &d.horizons {
                cells.push(json!({"id":format!("p{production}-s{physics_seed}-h{horizon}"),"production":production,
                    "kind":if production<4{"draw"}else if production==4{"active"}else{"dormant"},
                    "emission_seed":emission_seed,"physics_seed":physics_seed,"horizon":horizon}));
            }
        }
    }
    json!(cells)
}

/// Exact prospective source bridge: one evaluator generalized, no hidden native drift.
pub fn bridged_source(old: &str) -> Res<String> {
    let marker = ") -> Result<Value, String> {\n    let Some(p) = &emission.program else {";
    a::check(
        old.matches(marker).count() == 1,
        "unrecognized frozen evaluator entry",
    )?;
    let entry = ") -> Result<Value, String> {\n    let mut result = evaluate_with_horizon(emission, seed, deadline, work, results, HORIZON)?;\n    result.as_object_mut().unwrap().remove(\"horizon\");\n    if let Some(steps) = result[\"steps\"].as_array_mut() {\n        for step in steps {\n            step.as_object_mut().unwrap().remove(\"nodes\");\n            step.as_object_mut().unwrap().remove(\"edges\");\n        }\n    }\n    Ok(result)\n}\n/// Versioned scaling API; legacy wrappers retain exact 100-step receipt shape.\npub fn evaluate_with_horizon(\n    emission: &Emission,\n    seed: u64,\n    deadline: Instant,\n    work: usize,\n    results: usize,\n    horizon: usize,\n) -> Result<Value, String> {\n    if !(2..=10_000).contains(&horizon) {\n        return Err(\"scaling horizon must be in 2..=10000\".into());\n    }\n    let Some(p) = &emission.program else {";
    let mut current = old.replacen(marker, entry, 1);
    for (before, after, count) in [
        ("for step in 1..=HORIZON", "for step in 1..=horizon", 1),
        ("steps.len() == HORIZON", "steps.len() == horizon", 1),
        ("/ (HORIZON as f64).ln()", "/ (horizon as f64).ln()", 1),
        (
            "json!({\"physics_seed\":seed,\"score\":",
            "json!({\"horizon\":horizon,\"physics_seed\":seed,\"score\":",
            2,
        ),
        (
            "steps.push(json!({\"step\":step",
            "steps.push(json!({\"nodes\":s.node_count(),\"edges\":s.edge_count(),\"step\":step",
            1,
        ),
    ] {
        a::check(
            current.matches(before).count() == count,
            "unrecognized evaluator transformation",
        )?;
        current = current.replace(before, after);
    }
    Ok(current)
}
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn write(path: &Path, v: &Value) -> Res<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
fn bytes(root: &Path) -> Res<u64> {
    a::files(root)?
        .iter()
        .map(|p| {
            fs::metadata(root.join(p))
                .map(|m| m.len())
                .map_err(|e| e.to_string())
        })
        .sum()
}
fn seal(root: &Path) -> Res<()> {
    let mut names = a::files(root)?;
    names.remove("output_sha256.json");
    let paths: Vec<_> = names.iter().map(|n| root.join(n)).collect();
    let hashes: BTreeMap<_, _> = names.into_iter().zip(a::hashes(&paths)?).collect();
    write(&root.join("output_sha256.json"), &json!(hashes))
}
fn verify_engineering(root: &Path) -> Res<Value> {
    a::check(
        a::digest(&root.join("output_sha256.json"))? == ENGINEERING_SEAL,
        "engineering receipt root identity",
    )?;
    let hashes: BTreeMap<String, String> =
        serde_json::from_value(a::load(&root.join("output_sha256.json"))?)
            .map_err(|e| e.to_string())?;
    let mut names = a::files(root)?;
    names.remove("output_sha256.json");
    a::check(
        hashes.len() == 247 && names == hashes.keys().cloned().collect(),
        "engineering exact coverage",
    )?;
    let paths: Vec<_> = hashes.keys().map(|n| root.join(n)).collect();
    for ((n, h), actual) in hashes.iter().zip(a::hashes(&paths)?) {
        a::check(h == &actual, &format!("engineering sealed drift: {n}"))?;
    }
    let m = a::load(&root.join("manifest.json"))?;
    let audit = a::load(&root.join("audit.json"))?;
    a::check(
        m["status"] == "complete"
            && m["counts"]["bounded_evaluation_steps"] == 100
            && audit["audit_pass"] == true
            && audit["tamper_rejections"] == 22
            && m["training_admitted"] == false,
        "engineering prerequisite predicates",
    )?;
    Ok(
        json!({"archive":root.canonicalize().map_err(|e|e.to_string())?,"seal_sha256":ENGINEERING_SEAL,"sealed_files_verified":247}),
    )
}
struct Ready {
    evidence: Value,
    pins: BTreeMap<String, String>,
}
fn control_preflight(catalogue: &Value, d: &Design) -> Res<()> {
    for (parent, op) in d.control_parents.iter().zip([11, 12]) {
        a::check(
            catalogue["parents"][*parent]["groups"][op]["roles"][1]["productions"]
                .as_array()
                .is_some_and(|p| !p.is_empty()),
            "registered control missing from legal catalogue",
        )?;
    }
    let bank = hg_sim_core::ere::semantic::parent_bank();
    let active = &bank[d.control_parents[0]].1;
    let dormant = &bank[d.control_parents[1]].1;
    a::check(
        !active.active
            && dormant.active
            && active.core == dormant.core
            && active.proposal == dormant.proposal,
        "registered control parents are not matched opposite-activation fixtures",
    )
}
fn prepare(paths: &ReceiptPaths, d: &Design) -> Res<Ready> {
    d.validate()?;
    let historical = a::prepare_frozen_evidence(paths)?;
    control_preflight(&a::load(&paths.grammar.join("catalogue.json"))?, d)?;
    let base = repo();
    let engineering =
        verify_engineering(&base.join("output/ere_complete_bounded_engineering_20260930"))?;
    let mut pins = historical.pins().clone();
    let mut coverage: BTreeSet<String> = [
        "Cargo.toml",
        "Cargo.lock",
        "crates/hg-train/Cargo.toml",
        "crates/hg-train/src/optim.rs",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    for c in [
        "hg-core",
        "hg-engine",
        "hg-meta",
        "hg-rules",
        "hg-observe",
        "hg-sim-core",
    ] {
        for n in a::files(&base.join("crates").join(c))? {
            if Path::new(&n)
                .extension()
                .is_some_and(|e| e == "rs" || e == "toml")
            {
                coverage.insert(format!("crates/{c}/{n}"));
            }
        }
    }
    a::check(
        coverage == pins.keys().cloned().collect(),
        "scaling native source coverage",
    )?;
    let old = fs::read_to_string(paths.runtime.join("source_snapshots").join(SIGNAL))
        .map_err(|e| e.to_string())?;
    let current = fs::read_to_string(base.join(SIGNAL)).map_err(|e| e.to_string())?;
    a::check(
        current == bridged_source(&old)?,
        "native drift beyond exact registered horizon bridge",
    )?;
    let paths_current: Vec<_> = pins.keys().map(|p| base.join(p)).collect();
    for ((name, expected), actual) in pins.iter_mut().zip(a::hashes(&paths_current)?) {
        if name == SIGNAL {
            *expected = actual;
        } else {
            a::check(expected == &actual, &format!("stale scaling source {name}"))?;
        }
    }
    Ok(Ready {
        evidence: json!({"profile":PROFILE,"historical_evidence":historical.evidence(),"engineering_prerequisite":engineering,
        "scaling_execution_authorized":true,"training_admitted":false,"full_training_authorized":false,
        "native_source_bridge":{"source":SIGNAL,"historical_sha256":historical.pins()[SIGNAL],"current_sha256":pins[SIGNAL],"exact_registered_transform":true},
        "implementation_sources_verified":230}),
        pins,
    })
}
fn freeze(ready: &Ready, out: &Path) -> Res<Value> {
    let mut names: BTreeSet<_> = ready.pins.keys().cloned().collect();
    names.extend(EXTRA.into_iter().map(String::from));
    let mut dests = vec![];
    for n in &names {
        let dst = out.join("source_snapshots").join(n);
        fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::copy(repo().join(n), &dst).map_err(|e| e.to_string())?;
        dests.push(dst);
    }
    let sources: BTreeMap<_, _> = names.into_iter().zip(a::hashes(&dests)?).collect();
    fs::copy(
        std::env::current_exe().map_err(|e| e.to_string())?,
        out.join("executed_binary"),
    )
    .map_err(|e| e.to_string())?;
    Ok(
        json!({"sources":sources,"binary_sha256":a::digest(&out.join("executed_binary"))?,"frozen_before_measurement":true}),
    )
}
fn checkpoint(out: &Path, m: &mut Value, state: &str, reason: Option<&str>) -> Res<()> {
    m["status"] = json!(state);
    m["reason"] = json!(reason);
    write(&out.join("manifest.json"), m)
}
pub fn run(paths: &ReceiptPaths, d: &Design, out: &Path) -> Res<()> {
    a::check(!out.exists(), "fresh scaling output required")?;
    let absolute = out
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("output name")?);
    for root in paths.directories().into_iter().chain([repo()
        .join("output/ere_complete_bounded_engineering_20260930")
        .as_path()])
    {
        if let Ok(p) = root.canonicalize() {
            a::check(
                !absolute.starts_with(p),
                "scaling output cannot be inside prerequisites",
            )?;
        }
    }
    fs::create_dir(out).map_err(|e| e.to_string())?;
    let plan = schedule(&Design::registered());
    let mut m = json!({"profile":PROFILE,"design":d,"schedule":plan,"status":"validating","training_admitted":false,
        "full_training_authorized":false,"optimizer_evaluation_steps":0,"precheck_evaluation_steps":0,"held_out_evaluation_steps":0,
        "grammar_compilations":0,"attempted_steps":0,"advancing_steps":0,"scheduled_cells":36,"terminal_native_cells":0});
    checkpoint(out, &mut m, "validating", None)?;
    let result: Res<()> = (|| {
        let ready = prepare(paths, d)?;
        m["admission"] = ready.evidence.clone();
        m["snapshot"] = freeze(&ready, out)?;
        let start = Instant::now();
        let deadline = start + Duration::from_secs(d.wall_seconds);
        m["grammar_compilations"] = json!(1);
        checkpoint(out, &mut m, "compiling", None)?;
        let grammar = valid_grammar::compile()?;
        m["grammar_compile_seconds"] = json!(start.elapsed().as_secs_f64());
        fs::write(
            out.join("catalogue.json"),
            serde_json::to_vec(&grammar.catalogue()).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        a::check(
            a::digest(&out.join("catalogue.json"))? == a::CATALOGUE_SHA,
            "scaling catalogue identity",
        )?;
        prepare(paths, d)?;
        let mut emissions = vec![];
        for (i, &seed) in d.emission_seeds.iter().enumerate() {
            emissions.push(if i < 4 {
                grammar.emit(&d.parameters, seed)?
            } else {
                grammar.emit_selected(
                    d.control_parents[i - 4],
                    if i == 4 { 11 } else { 12 },
                    1,
                    0,
                    seed,
                )?
            });
        }
        let active = emissions[4]
            .program
            .as_ref()
            .ok_or("active control absent")?;
        let dormant = emissions[5]
            .program
            .as_ref()
            .ok_or("dormant control absent")?;
        a::check(
            active.active
                && !dormant.active
                && active.core == dormant.core
                && active.proposal == dormant.proposal
                && emissions[4].record["initial_world_hash"]
                    == emissions[5].record["initial_world_hash"],
            "emitted controls must remain matched",
        )?;
        write(
            &out.join("emissions.json"),
            &json!(emissions.iter().map(|e| &e.record).collect::<Vec<_>>()),
        )?;
        fs::create_dir(out.join("cells")).map_err(|e| e.to_string())?;
        let mut attempts = 0u64;
        let mut advances = 0u64;
        let mut terminal = 0usize;
        let mut cells = vec![];
        for cell in plan.as_array().unwrap() {
            let id = cell["id"].as_str().unwrap();
            if Instant::now() >= deadline || bytes(out)? >= d.output_cap_bytes - 32 * 1024 * 1024 {
                cells.push(json!({"schedule":cell,"status":"NotRun","reason":"global_resource_ceiling","evaluation":null}));
                continue;
            }
            prepare(paths, d)?;
            m["current_cell"] = cell.clone();
            m["inflight_native_work_unknown"] = json!(true);
            checkpoint(out, &mut m, "evaluating", None)?;
            let cell_start = Instant::now();
            let cell_deadline = deadline.min(cell_start + Duration::from_secs(d.cell_wall_seconds));
            let e = &emissions[cell["production"].as_u64().unwrap() as usize];
            let evaluated = signal::evaluate_with_horizon(
                e,
                cell["physics_seed"].as_u64().unwrap(),
                cell_deadline,
                d.work,
                d.results,
                cell["horizon"].as_u64().unwrap() as usize,
            );
            let row = match evaluated {
                Ok(mut v) => {
                    v["runtime_match_budget"] =
                        json!({"policy":"Pooled","work":d.work,"results":d.results});
                    attempts += v["attempted_steps"].as_u64().ok_or("missing attempts")?;
                    advances += v["advancing_steps"].as_u64().ok_or("missing advances")?;
                    terminal += 1;
                    json!({"schedule":cell,"status":if v["score"].is_number(){"Complete"}else{"Null"},"elapsed_seconds":cell_start.elapsed().as_secs_f64(),"evaluation":v})
                }
                Err(error) => {
                    json!({"schedule":cell,"status":"Incomplete","reason":error,"elapsed_seconds":cell_start.elapsed().as_secs_f64(),"partial_steps_unknown":true,"evaluation":null})
                }
            };
            let wire = serde_json::to_vec(&row).map_err(|e| e.to_string())?;
            // Never replace a full native receipt with a made-up zero if output is too large.
            a::check(
                bytes(out)? + (wire.len() as u64) < d.output_cap_bytes - 16 * 1024 * 1024,
                "scaling output ceiling",
            )?;
            fs::write(out.join("cells").join(format!("{id}.json")), wire)
                .map_err(|e| e.to_string())?;
            cells.push(row);
            m["attempted_steps"] = json!(attempts);
            m["advancing_steps"] = json!(advances);
            m["terminal_native_cells"] = json!(terminal);
            m["inflight_native_work_unknown"] = json!(false);
            m["recorded_cells"] = json!(cells.len());
            checkpoint(out, &mut m, "evaluating", None)?;
        }
        // Persist NotRun rows too, preserving the complete scheduled denominator.
        for row in &cells {
            let p = out
                .join("cells")
                .join(format!("{}.json", row["schedule"]["id"].as_str().unwrap()));
            if !p.exists() {
                write(&p, row)?;
            }
        }
        let mut prefixes = true;
        let mut controls = true;
        for p in 0..6 {
            for seed in &d.physics_seeds {
                let trio: Vec<_> = cells
                    .iter()
                    .filter(|r| {
                        r["schedule"]["production"] == p && r["schedule"]["physics_seed"] == *seed
                    })
                    .collect();
                for pair in trio.windows(2) {
                    let short = pair[0]["evaluation"]["steps"].as_array();
                    let long = pair[1]["evaluation"]["steps"].as_array();
                    prefixes &= match (short, long) {
                        (Some(a), Some(b)) => a.len() <= b.len() && *a == b[..a.len()],
                        _ => false,
                    };
                }
                if p >= 4 {
                    for r in trio {
                        controls &= r["status"] == "Complete"
                            && r["evaluation"]["proposal_fires"]
                                == if p == 4 {
                                    r["schedule"]["horizon"].clone()
                                } else {
                                    json!(0)
                                };
                    }
                }
            }
        }
        let complete = cells.iter().filter(|r| r["status"] == "Complete").count();
        let mut by_horizon = BTreeMap::new();
        for h in &d.horizons {
            let rows: Vec<_> = cells
                .iter()
                .filter(|r| r["schedule"]["horizon"] == *h)
                .collect();
            by_horizon.insert(h.to_string(),json!({"scheduled":12,"complete":rows.iter().filter(|r|r["status"]=="Complete").count(),
                "null":rows.iter().filter(|r|r["status"]=="Null").count(),"incomplete":rows.iter().filter(|r|r["status"]=="Incomplete").count(),
                "not_run":rows.iter().filter(|r|r["status"]=="NotRun").count()}));
        }
        let pass = complete == 36
            && prefixes
            && controls
            && Instant::now() < deadline
            && cells.iter().all(|r| {
                r["elapsed_seconds"]
                    .as_f64()
                    .is_some_and(|t| t < d.cell_wall_seconds as f64)
            });
        let summary = json!({"profile":PROFILE,"scheduled_cells":36,"terminal_native_cells":terminal,"complete_cells":complete,
            "measurement_complete":terminal==36,"native_scaling_checks_pass":pass,"prefixes_equal":prefixes,"controls_pass":controls,
            "by_horizon":by_horizon,"attempted_steps":attempts,"advancing_steps":advances,"elapsed_seconds":start.elapsed().as_secs_f64(),
            "counts_cover_receipted_cells_only":true,"unknown_partial_work_cells":cells.iter().filter(|r|r["status"]=="Incomplete").count(),
            "grammar_compile_seconds":m["grammar_compile_seconds"],"training_admitted":false,"full_training_authorized":false});
        write(&out.join("summary.json"), &summary)?;
        m["elapsed_seconds"] = summary["elapsed_seconds"].clone();
        m["measurement_complete"] = json!(terminal == 36);
        m["native_scaling_checks_pass"] = json!(pass);
        checkpoint(
            out,
            &mut m,
            if terminal == 36 {
                "complete"
            } else {
                "incomplete"
            },
            None,
        )?;
        Ok(())
    })();
    if let Err(error) = &result {
        let state = if m["grammar_compilations"] == 0 {
            "blocked_before_evaluation"
        } else {
            "incomplete"
        };
        checkpoint(out, &mut m, state, Some(error))?;
    }
    seal(out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn paths() -> ReceiptPaths {
        let r = repo().join("output");
        ReceiptPaths {
            deterministic: r.join("ere_complete_gates_fresh_20260926"),
            long_gates: r.join("ere_complete_long_gates_full_20260927"),
            closure: r.join("ere_complete_coherent_closure_full_20260929"),
            grammar: r.join("ere_valid_production_grammar_20260929"),
            runtime: r.join("ere_runtime50k_signal_panel_20260930"),
        }
    }
    #[test]
    fn exact_source_bridge_and_real_evidence() {
        let p = paths();
        let old = fs::read_to_string(p.runtime.join("source_snapshots").join(SIGNAL)).unwrap();
        assert_eq!(
            bridged_source(&old).unwrap(),
            fs::read_to_string(
                repo()
                    .join("output/ere_complete_scaling_trial_v2_20260930/source_snapshots")
                    .join(SIGNAL)
            )
            .unwrap()
        );
        // The old scaling profile cannot admit the subsequently repaired evaluator.
        assert!(prepare(&p, &Design::registered()).is_err());
    }
    #[test]
    fn fixed_schedule_and_wrong_design_refuse_zero_work() {
        let d = Design::registered();
        let config: Design = serde_json::from_slice(
            &fs::read(repo().join("configs/ere_complete_scaling_v2.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(config, d);
        let s = schedule(&d);
        assert_eq!(s.as_array().unwrap().len(), 36);
        assert_eq!(
            s.as_array()
                .unwrap()
                .iter()
                .map(|c| c["horizon"].as_u64().unwrap())
                .sum::<u64>(),
            133200
        );
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("refusal");
        let mut wrong = d;
        wrong.work = 400000;
        assert!(run(&paths(), &wrong, &out).is_err());
        let m = a::load(&out.join("manifest.json")).unwrap();
        assert_eq!(m["grammar_compilations"], 0);
        assert_eq!(m["attempted_steps"], 0);
        assert!(!out.join("cells").exists());
    }
    #[test]
    fn legacy_hundred_step_receipt_and_scaling_prefix_parity() {
        let e = signal::forced(true, 17).unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let old = signal::evaluate_with_budget(&e, 1597, deadline, 50000, 50000).unwrap();
        let mut short =
            signal::evaluate_with_horizon(&e, 1597, deadline, 50000, 50000, 100).unwrap();
        let long = signal::evaluate_with_horizon(&e, 1597, deadline, 50000, 50000, 120).unwrap();
        assert_eq!(
            short["steps"],
            json!(&long["steps"].as_array().unwrap()[..100])
        );
        short.as_object_mut().unwrap().remove("horizon");
        for s in short["steps"].as_array_mut().unwrap() {
            s.as_object_mut().unwrap().remove("nodes");
            s.as_object_mut().unwrap().remove("edges");
        }
        assert_eq!(old, short);
        assert_eq!(long["proposal_fires"], 120);
        assert!(signal::evaluate_with_horizon(&e, 1, deadline, 50000, 50000, 1).is_err());
        assert!(signal::evaluate_with_horizon(&e, 1, deadline, 50000, 50000, 10001).is_err());
        let cap = signal::evaluate_with_horizon(&e, 1, deadline, 1, 1, 1000).unwrap();
        assert!(cap["score"].is_null());
        assert_eq!(cap["attempted_steps"], 1);
    }
    #[test]
    fn control_catalogue_preflight_catches_original_dormant_setup_defect_without_physics() {
        let catalogue = a::load(&paths().grammar.join("catalogue.json")).unwrap();
        let mut d = Design::registered();
        control_preflight(&catalogue, &d).unwrap();
        d.control_parents[1] = 4;
        assert!(control_preflight(&catalogue, &d).is_err());
        assert!(d.validate().is_err());
    }
    #[test]
    fn legacy_receipt_equals_historical_native_control_not_just_new_wrapper() {
        let rows = fs::read_to_string(paths().runtime.join("rollouts.jsonl")).unwrap();
        let row: Value = rows
            .lines()
            .map(|s| serde_json::from_str::<Value>(s).unwrap())
            .find(|v| v["emission"]["parent"] == 4 && v["emission"]["operator"] == 11)
            .unwrap();
        let e = signal::forced(true, row["emission"]["emission_seed"].as_u64().unwrap()).unwrap();
        let actual = signal::evaluate_with_budget(
            &e,
            row["evaluation"]["physics_seed"].as_u64().unwrap(),
            Instant::now() + Duration::from_secs(30),
            50000,
            50000,
        )
        .unwrap();
        let mut expected = row["evaluation"].clone();
        expected
            .as_object_mut()
            .unwrap()
            .remove("runtime_match_budget");
        assert_eq!(actual, expected);
    }
}
