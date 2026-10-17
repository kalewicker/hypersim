//! Sealed historical evidence plus an explicitly bounded current execution scope.
//! This capability cannot be deserialized or used to authorize full training.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub const PROFILE: &str = "E2-complete-receipt-bound-runner-v1";
pub const CATALOGUE_SHA: &str = "ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6";
pub const WORK: usize = 50_000;
pub const RESULTS: usize = 50_000;
pub const HORIZON: usize = 100;
pub const OUTPUT_CAP: u64 = 512 * 1024 * 1024;
pub type Res<T> = Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptPaths {
    pub deterministic: PathBuf,
    pub long_gates: PathBuf,
    pub closure: PathBuf,
    pub grammar: PathBuf,
    pub runtime: PathBuf,
}
impl ReceiptPaths {
    pub fn resolve_relative(&mut self, base: &Path) {
        for p in [
            &mut self.deterministic,
            &mut self.long_gates,
            &mut self.closure,
            &mut self.grammar,
            &mut self.runtime,
        ] {
            if p.is_relative() {
                *p = base.join(&*p);
            }
        }
    }
    pub(crate) fn directories(&self) -> [&Path; 5] {
        [
            &self.deterministic,
            &self.long_gates,
            &self.closure,
            &self.grammar,
            &self.runtime,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub profile: String,
    pub purpose: String,
    pub protocol: String,
    pub grammar_profile: String,
    pub matcher_work_model: String,
    pub allocation_mode: String,
    pub ordinary_world: String,
    pub step_mode: String,
    pub max_pairings: usize,
    pub match_budget_policy: String,
    pub work: usize,
    pub results: usize,
    pub horizon: usize,
    pub evaluations: usize,
    pub optimizer_generations: usize,
    pub held_out_evaluations: usize,
    pub wall_seconds: u64,
    pub output_cap_bytes: u64,
    pub parameters: Vec<f32>,
    pub emission_seed: u64,
    pub physics_seed: u64,
}
impl Request {
    pub fn engineering(emission_seed: u64, physics_seed: u64) -> Self {
        Self {
            profile: PROFILE.into(),
            purpose: "bounded_engineering".into(),
            protocol: "E2-witness-complete-v1".into(),
            grammar_profile: "E2-complete-valid-production-grammar-v1".into(),
            matcher_work_model: "repeated-self-pair-feasibility-v1".into(),
            allocation_mode: "metadata-isolated-v1".into(),
            ordinary_world: "signal-fixed-world-v1".into(),
            step_mode: "MaxParallel".into(),
            max_pairings: 1,
            match_budget_policy: "Pooled".into(),
            work: WORK,
            results: RESULTS,
            horizon: HORIZON,
            evaluations: 1,
            optimizer_generations: 0,
            held_out_evaluations: 0,
            wall_seconds: 900,
            output_cap_bytes: OUTPUT_CAP,
            parameters: vec![0.; 183],
            emission_seed,
            physics_seed,
        }
    }
    pub fn validate(&self) -> Res<()> {
        check(
            self.profile == PROFILE && self.purpose == "bounded_engineering",
            "only bounded engineering execution is admitted; full training is closed",
        )?;
        check(
            self.protocol == "E2-witness-complete-v1"
                && self.grammar_profile == "E2-complete-valid-production-grammar-v1",
            "protocol/grammar mismatch",
        )?;
        check(
            self.matcher_work_model == "repeated-self-pair-feasibility-v1"
                && self.allocation_mode == "metadata-isolated-v1"
                && self.ordinary_world == "signal-fixed-world-v1",
            "matcher/allocator/world mismatch",
        )?;
        check(
            self.step_mode == "MaxParallel"
                && self.max_pairings == 1
                && self.match_budget_policy == "Pooled",
            "execution mode mismatch",
        )?;
        check(
            self.work == WORK && self.results == RESULTS && self.horizon == HORIZON,
            "only measured 100-step 50k/50k envelope is admitted",
        )?;
        check(
            self.evaluations == 1
                && self.optimizer_generations == 0
                && self.held_out_evaluations == 0,
            "campaign/held-out execution is not authorized by this profile",
        )?;
        check(
            self.wall_seconds == 900 && self.output_cap_bytes == OUTPUT_CAP,
            "registered resource limits mismatch",
        )?;
        check(
            self.parameters.len() == 183 && self.parameters.iter().all(|x| x.is_finite()),
            "expected 183 finite logits",
        )
    }
}

struct Spec {
    name: &'static str,
    seal: &'static str,
    pin: &'static str,
}
const SPECS: [Spec; 5] = [
    Spec {
        name: "deterministic",
        seal: "SHA256SUMS",
        pin: "b992ca9ab6df4b642b40320377d87a20f331a5bb8cf0b29dcc68262c8c82c5f3",
    },
    Spec {
        name: "long_gates",
        seal: "output_sha256.json",
        pin: "32bc8eb713b67d71569a94e8dc6ed6000784c3c89afd00ad67a583c6b9edf5c5",
    },
    Spec {
        name: "closure",
        seal: "output_sha256.json",
        pin: "fa3725f7cf5a03b207635d4c121d47280d38b4a05ae018ec3606e00086a4b60a",
    },
    Spec {
        name: "grammar",
        seal: "output_sha256.json",
        pin: "ccc22798899e426f93cd2801020fef0e93391e8790a23a214d22b625caa58891",
    },
    Spec {
        name: "runtime",
        seal: "output_sha256.json",
        pin: "0099ecae4b7487e9ac975f73d813e7cbde0de430470dc66198bbafd7df16488c",
    },
];

pub(crate) fn check(ok: bool, detail: &str) -> Res<()> {
    if ok {
        Ok(())
    } else {
        Err(format!("{PROFILE}: {detail}"))
    }
}
pub(crate) fn load(path: &Path) -> Res<Value> {
    check(
        fs::metadata(path)
            .map_err(|e| format!("missing {}: {e}", path.display()))?
            .len()
            <= 256 * 1024 * 1024,
        "JSON input exceeds 256 MiB",
    )?;
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\n', '\r', '\\'])
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
pub(crate) fn hashes(paths: &[PathBuf]) -> Res<Vec<String>> {
    let mut values = vec![];
    for batch in paths.chunks(64) {
        let result = Command::new("shasum")
            .args(["-a", "256", "--"])
            .args(batch)
            .output()
            .map_err(|e| e.to_string())?;
        check(result.status.success(), "SHA256 command failed")?;
        let output = String::from_utf8(result.stdout).map_err(|e| e.to_string())?;
        let parsed: Vec<_> = output
            .lines()
            .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
            .collect();
        check(
            parsed.len() == batch.len()
                && parsed
                    .iter()
                    .all(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())),
            "malformed SHA256 output",
        )?;
        values.extend(parsed);
    }
    Ok(values)
}
pub(crate) fn digest(path: &Path) -> Res<String> {
    Ok(hashes(&[path.to_path_buf()])?.remove(0))
}
pub(crate) fn files(root: &Path) -> Res<BTreeSet<String>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeSet<String>) -> Res<()> {
        for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let p = e.map_err(|e| e.to_string())?.path();
            let m = fs::symlink_metadata(&p).map_err(|e| e.to_string())?;
            check(
                !m.file_type().is_symlink(),
                "symlinks are not admissible evidence",
            )?;
            if m.is_dir() {
                walk(root, &p, out)?;
            } else {
                check(m.is_file(), "non-regular evidence file")?;
                let rel = p
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("non-UTF8 evidence path")?;
                check(safe_relative(rel), "unsafe evidence path")?;
                out.insert(rel.into());
            }
        }
        Ok(())
    }
    check(
        !fs::symlink_metadata(root)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink(),
        "symlink archive root",
    )?;
    let mut out = BTreeSet::new();
    walk(root, root, &mut out)?;
    Ok(out)
}
fn parse_seal(dir: &Path, seal: &str) -> Res<BTreeMap<String, String>> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    if seal == "SHA256SUMS" {
        for line in fs::read_to_string(dir.join(seal))
            .map_err(|e| e.to_string())?
            .lines()
        {
            let (hash, name) = line.split_once("  ").ok_or("malformed SHA256SUMS row")?;
            check(
                out.insert(name.into(), hash.into()).is_none(),
                "duplicate sealed path",
            )?;
        }
    } else {
        out = serde_json::from_value(load(&dir.join(seal))?).map_err(|e| e.to_string())?;
    }
    check(!out.is_empty(), "empty seal")?;
    for (p, h) in &out {
        check(
            safe_relative(p)
                && p != seal
                && h.len() == 64
                && h.bytes().all(|b| b.is_ascii_hexdigit()),
            "unsafe/self-referential sealed entry",
        )?;
    }
    Ok(out)
}
fn verify_archive(dir: &Path, spec: &Spec) -> Res<Value> {
    check(
        digest(&dir.join(spec.seal))? == spec.pin,
        &format!("{} root seal is missing, stale or unrecognized", spec.name),
    )?;
    let seal = parse_seal(dir, spec.seal)?;
    let mut actual = files(dir)?;
    actual.remove(spec.seal);
    check(
        actual == seal.keys().cloned().collect(),
        &format!("{} exact archive file coverage failed", spec.name),
    )?;
    let paths: Vec<_> = seal.keys().map(|rel| dir.join(rel)).collect();
    for ((rel, expected), actual) in seal.iter().zip(hashes(&paths)?) {
        check(
            expected == &actual,
            &format!("{} sealed content drift: {rel}", spec.name),
        )?;
    }
    Ok(
        json!({"archive":dir.canonicalize().map_err(|e|e.to_string())?,"seal_file":spec.seal,"seal_sha256":spec.pin,"sealed_files_verified":seal.len(),"exact_file_coverage":true}),
    )
}
fn n(v: &Value, key: &str) -> Res<u64> {
    v[key]
        .as_u64()
        .ok_or_else(|| format!("missing/incompatible integer {key}"))
}
fn receipt(dir: &Path, name: &str) -> Res<Value> {
    load(&dir.join(name))
}

fn predicates(paths: &ReceiptPaths) -> Res<()> {
    let m = receipt(&paths.deterministic, "manifest.json")?;
    let status = receipt(&paths.deterministic, "run_status.json")?;
    let audit = receipt(&paths.deterministic, "independent_audit.json")?;
    check(
        m["profile"] == "E2-complete-deterministic-gates-v1"
            && m["protocol"] == "E2-witness-complete-v1"
            && m["family_version"] == 10
            && m["instruction_version"] == 11
            && m["frozen_before_measurement"] == true
            && m["training_admitted"] == false
            && status["clean_bounded_receipt"] == true
            && status["semantic_audit_pass"] == true
            && status["native_bounded_gates_pass"] == true
            && audit["audit_pass"] == true,
        "deterministic complete gate predicates failed",
    )?;
    for name in ["G-R0.json", "G-R2.json", "G-R4.json", "G-R6.json"] {
        check(
            receipt(&paths.deterministic, name)?["pass"] == true,
            "required deterministic gate failed",
        )?;
    }
    let r2 = receipt(&paths.deterministic, "G-R2.json")?;
    check(
        n(&r2, "horizon")? == 1000
            && n(&r2, "supported")? == 2
            && r2["rows"].as_array().is_some_and(|r| r.len() == 45),
        "G-R2 must retain its declared exclusions",
    )?;
    let m = receipt(&paths.long_gates, "manifest.json")?;
    let s = receipt(&paths.long_gates, "run_status.json")?;
    let a = receipt(&paths.long_gates, "audit_full.json")?;
    check(
        m["profile"] == "E2-complete-long-horizon-gates-v1"
            && m["protocol"] == "E2-witness-complete-v1"
            && m["development_only"] == false
            && s["status"] == "complete"
            && s["gate_pass"] == true
            && s["training_admitted"] == false
            && n(&s, "advancing_engine_steps")? == 540000
            && a["scope"] == "full"
            && a["audit_pass"] == true
            && a["gate_pass"] == true
            && n(&a, "steps")? == 540000
            && n(&a, "tamper_rejections")? == 396,
        "full G-R3/G-R5 evidence failed",
    )?;
    let cells = s["cells"].as_array().ok_or("missing long-gate cells")?;
    check(
        cells.len() == 18
            && cells
                .iter()
                .all(|c| c["native_complete"] == true && c["native_checks_pass"] == true),
        "long-gate cell denominator/completion failed",
    )?;
    let m = receipt(&paths.closure, "manifest.json")?;
    let s = receipt(&paths.closure, "run_status.json")?;
    let a = receipt(&paths.closure, "audit_full.json")?;
    let c = receipt(&paths.closure, "classification_summary.json")?;
    check(
        m["profile"] == "E2-complete-coherent-closure-v1"
            && m["protocol"] == "E2-witness-complete-v1"
            && m["allocation_mode"] == "metadata-isolated-v1"
            && m["matcher_work_model"] == "repeated-self-pair-feasibility-v1"
            && n(&m, "draws")? == 10000
            && n(&m, "parents")? == 168
            && n(&m, "horizon")? == 100
            && m["development_only"] == false
            && m["resources"]["classification_work_limit_per_step"] == 400000
            && m["resources"]["classification_result_limit_per_step"] == 50000
            && s["status"] == "complete"
            && s["gate_pass"] == true
            && s["measurement_complete"] == true
            && s["training_admitted"] == false
            && a["audit_pass"] == true
            && a["gate_pass"] == true
            && a["scope"] == "full"
            && n(&a, "tamper_rejections")? == 27
            && c["numerical_pass"] == true
            && c["parent_baselines_complete"] == true
            && n(&c, "neutral_soundness_checked")? == 50
            && n(&c, "neutral_soundness_failures")? == 0,
        "accepted coherent G-R1 predicates failed",
    )?;
    for key in ["overall", "structural", "active_parent_structural"] {
        let row = &c["strata"][key];
        let total = n(row, "samples")?;
        let valid = n(row, "valid")?;
        check(
            total > 0
                && valid + n(row, "rejected")? == total
                && valid
                    == n(row, "neutral")?
                        + n(row, "near_neutral")?
                        + n(row, "divergent")?
                        + n(row, "catastrophic")?
                && valid as f64 / total as f64 >= 0.4
                && n(row, "near_neutral")? as f64 / total as f64 >= 0.1
                && n(row, "catastrophic")? as f64 / total as f64 <= 0.25,
            "mandatory coherent G-R1 stratum failed",
        )?;
    }
    let m = receipt(&paths.grammar, "manifest.json")?;
    let a = receipt(&paths.grammar, "audit.json")?;
    check(
        m["profile"] == "E2-complete-valid-production-grammar-v1"
            && m["status"] == "complete"
            && m["catalogue_sha256"] == CATALOGUE_SHA
            && a["audit_pass"] == true
            && n(&a, "parents")? == 168
            && n(&a, "legal_productions")? == 18956
            && n(&a, "engine_steps")? == 0
            && n(&a, "optimizer_rollouts")? == 0
            && m["training_admitted"] == false,
        "grammar prerequisite failed",
    )?;
    check(
        digest(&paths.grammar.join("catalogue.json"))? == CATALOGUE_SHA,
        "catalogue identity failed",
    )?;
    let m = receipt(&paths.runtime, "manifest.json")?;
    let s = receipt(&paths.runtime, "summary.json")?;
    let a = receipt(&paths.runtime, "audit.json")?;
    let budget = json!({"policy":"Pooled","work":WORK,"results":RESULTS});
    check(
        m["profile"] == "E2-complete-runtime50k-signal-v1"
            && s["profile"] == m["profile"]
            && m["status"] == "complete"
            && m["runtime_match_budget"] == budget
            && s["runtime_match_budget"] == budget
            && n(&m, "horizon")? == 100
            && m["matcher_work_model"] == "repeated-self-pair-feasibility-v1"
            && m["catalogue_rebuilt"] == true
            && m["frozen_before_measurement"] == true
            && s["measurement_complete"] == true
            && s["runtime_envelope_pass"] == true
            && s["precheck_pass"] == true
            && n(&s, "rollouts")? == 120
            && n(&s, "valid_rollouts")? == 120
            && n(&s, "valid_pairs")? == 20
            && n(&s, "attempted_steps")? == 12000
            && n(&s, "advancing_steps")? == 12000
            && a["audit_pass"] == true
            && a["runtime_envelope_pass"] == true
            && a["reference_preflight"] == false
            && n(&a, "tamper_rejections")? == 23
            && m["training_admitted"] == false
            && s["training_admitted"] == false
            && a["training_admitted"] == false,
        "runtime precheck/profile predicates failed",
    )?;
    for key in ["P1", "P2", "P3", "P4"] {
        check(
            s["checks"][key] == true && a["checks"][key] == true,
            "stateability precheck failed",
        )?;
    }
    check(
        m["previous_signal"]["original_frozen_checker_pass"] == false
            && m["previous_signal"]["corrected_accounting_pass"] == true,
        "historical checker correction must remain disclosed",
    )?;
    Ok(())
}

fn pinned_source(rel: &str) -> bool {
    rel.starts_with("crates/hg-core/")
        || rel.starts_with("crates/hg-engine/")
        || rel.starts_with("crates/hg-meta/")
        || rel.starts_with("crates/hg-rules/")
        || rel.starts_with("crates/hg-observe/")
        || rel.starts_with("crates/hg-sim-core/")
        || [
            "Cargo.toml",
            "Cargo.lock",
            "crates/hg-train/Cargo.toml",
            "crates/hg-train/src/optim.rs",
        ]
        .contains(&rel)
}
fn implementation_pins(runtime: &Path, repo: &Path) -> Res<BTreeMap<String, String>> {
    let m = receipt(runtime, "manifest.json")?;
    let sources = m["snapshot"]["sources"]
        .as_object()
        .ok_or("missing runtime source pins")?;
    let mut pins = BTreeMap::new();
    for (rel, h) in sources {
        if pinned_source(rel) {
            check(safe_relative(rel), "unsafe source pin")?;
            pins.insert(
                rel.clone(),
                h.as_str().ok_or("malformed source hash")?.to_string(),
            );
        }
    }
    check(
        pins.len() == 230,
        "registered implementation pin denominator changed",
    )?;
    let mut current: BTreeSet<String> = [
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
        for rel in files(&repo.join("crates").join(c))? {
            if Path::new(&rel)
                .extension()
                .is_some_and(|x| x == "rs" || x == "toml")
            {
                current.insert(format!("crates/{c}/{rel}"));
            }
        }
    }
    check(
        current == pins.keys().cloned().collect(),
        "current native source file coverage is stale",
    )?;
    let paths: Vec<_> = pins.keys().map(|r| repo.join(r)).collect();
    for ((rel, expected), actual) in pins.iter().zip(hashes(&paths)?) {
        check(
            expected == &actual,
            &format!("stale current implementation: {rel}"),
        )?;
    }
    Ok(pins)
}

/// Opaque, process-local bounded execution capability; no Deserialize or public fields.
pub struct Prepared {
    paths: ReceiptPaths,
    request: Request,
    repo: PathBuf,
    evidence: Value,
    pins: BTreeMap<String, String>,
}
impl Prepared {
    pub fn evidence(&self) -> &Value {
        &self.evidence
    }
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub(crate) fn repo(&self) -> &Path {
        &self.repo
    }
    pub(crate) fn pins(&self) -> &BTreeMap<String, String> {
        &self.pins
    }
    pub(crate) fn recheck(&self) -> Res<()> {
        prepare_at(&self.paths, &self.request, &self.repo).map(|_| ())
    }
}
fn prepare_at(paths: &ReceiptPaths, request: &Request, repo: &Path) -> Res<Prepared> {
    request.validate()?;
    let mut proofs = BTreeMap::new();
    for (spec, dir) in SPECS.iter().zip(paths.directories()) {
        proofs.insert(spec.name, verify_archive(dir, spec)?);
    }
    predicates(paths)?;
    let pins = implementation_pins(&paths.runtime, repo)?;
    let evidence = json!({"profile":PROFILE,"historical_gate_evidence_verified":true,"bounded_execution_admitted":true,
        "training_admitted":false,"full_training_authorized":false,"admitted_horizon":HORIZON,"admitted_evaluations":1,
        "runtime_match_budget":{"policy":"Pooled","work":WORK,"results":RESULTS},"receipts":proofs,"implementation_sources_verified":pins.len(),
        "historical_scopes_preserved":true,"prior_original_frozen_checker_pass":false,"prior_corrected_accounting_pass":true,
        "limitations":["Historical semantic gates retain their original fixtures and execution profiles","G-R2 supports 2 of 45 rules","Runtime evidence is a fixed-world 100-step fixture","No 10000-step scaling or learned maintenance admission"]});
    Ok(Prepared {
        paths: paths.clone(),
        request: request.clone(),
        repo: repo.to_path_buf(),
        evidence,
        pins,
    })
}
pub fn prepare(paths: &ReceiptPaths, request: &Request) -> Res<Prepared> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    prepare_at(paths, request, &repo)
}
/// Historical evidence only. A new profile must separately verify current native identity.
pub(crate) fn prepare_frozen_evidence(paths: &ReceiptPaths) -> Res<Prepared> {
    prepare_at(
        paths,
        &Request::engineering(7, 101),
        &paths.runtime.join("source_snapshots"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
    }
    fn real_paths() -> ReceiptPaths {
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
    fn valid_request_is_bounded_and_rejects_unsupported_scopes() {
        let valid = Request::engineering(7, 101);
        valid.validate().unwrap();
        let mut cases = vec![];
        macro_rules! bad {
            ($field:ident,$value:expr) => {{
                let mut x = valid.clone();
                x.$field = $value;
                cases.push(x);
            }};
        }
        bad!(purpose, "training".into());
        bad!(horizon, 10000);
        bad!(work, 400000);
        bad!(results, 100000);
        bad!(optimizer_generations, 20);
        bad!(evaluations, 320);
        bad!(held_out_evaluations, 24);
        bad!(max_pairings, 2);
        bad!(match_budget_policy, "Split".into());
        bad!(matcher_work_model, "default".into());
        bad!(allocation_mode, "legacy".into());
        bad!(ordinary_world, "other".into());
        bad!(step_mode, "SequentialRandom".into());
        bad!(protocol, "E2-alternatives-pooled-v1".into());
        bad!(grammar_profile, "raw".into());
        bad!(wall_seconds, 901);
        bad!(output_cap_bytes, OUTPUT_CAP + 1);
        bad!(parameters, vec![0.; 64]);
        bad!(parameters, vec![f32::NAN; 183]);
        bad!(parameters, vec![f32::INFINITY; 183]);
        for x in cases {
            assert!(x.validate().is_err());
        }
    }
    #[test]
    fn request_json_rejects_unknown_fields() {
        let mut v = serde_json::to_value(Request::engineering(7, 101)).unwrap();
        v["training_authorized"] = json!(true);
        assert!(serde_json::from_value::<Request>(v).is_err());
    }
    #[test]
    fn unsafe_paths_and_duplicate_checksum_rows_are_rejected() {
        for p in [
            "",
            "/absolute",
            "../escape",
            "nested/../escape",
            "a\\b",
            "a\nb",
        ] {
            assert!(!safe_relative(p));
        }
        assert!(safe_relative("nested/output_sha256.json"));
        let d = tempfile::tempdir().unwrap();
        let h = "0".repeat(64);
        fs::write(d.path().join("SHA256SUMS"), format!("{h}  a\n{h}  a\n")).unwrap();
        assert!(parse_seal(d.path(), "SHA256SUMS").is_err());
    }
    #[test]
    fn seal_rejects_tampered_missing_and_extra_files_with_nested_coverage() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir(d.path().join("nested")).unwrap();
        fs::write(d.path().join("a"), "one").unwrap();
        fs::write(d.path().join("nested/output_sha256.json"), "nested").unwrap();
        let v = json!({"a":digest(&d.path().join("a")).unwrap(),"nested/output_sha256.json":digest(&d.path().join("nested/output_sha256.json")).unwrap()});
        fs::write(
            d.path().join("output_sha256.json"),
            serde_json::to_vec(&v).unwrap(),
        )
        .unwrap();
        let hash = digest(&d.path().join("output_sha256.json")).unwrap();
        let spec = Spec {
            name: "unit_fixture",
            seal: "output_sha256.json",
            pin: Box::leak(hash.into_boxed_str()),
        };
        verify_archive(d.path(), &spec).unwrap();
        fs::write(d.path().join("extra"), "x").unwrap();
        assert!(verify_archive(d.path(), &spec).is_err());
        fs::remove_file(d.path().join("extra")).unwrap();
        fs::write(d.path().join("nested/output_sha256.json"), "changed").unwrap();
        assert!(verify_archive(d.path(), &spec).is_err());
        fs::remove_file(d.path().join("a")).unwrap();
        assert!(verify_archive(d.path(), &spec).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_evidence_is_rejected() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a"), "x").unwrap();
        std::os::unix::fs::symlink(d.path().join("a"), d.path().join("alias")).unwrap();
        assert!(files(d.path()).is_err());
    }
    #[test]
    fn real_frozen_receipts_prepare_without_physics_and_do_not_admit_training() {
        let p = prepare_frozen_evidence(&real_paths()).unwrap();
        assert_eq!(p.evidence()["implementation_sources_verified"], 230);
        assert_eq!(p.evidence()["training_admitted"], false);
        assert_eq!(p.evidence()["bounded_execution_admitted"], true);
    }
    #[test]
    fn legacy_or_missing_receipt_cannot_create_capability() {
        let mut p = real_paths();
        p.deterministic = repo().join("output/ere_e2p_gates_20260926");
        assert!(prepare(&p, &Request::engineering(7, 101)).is_err());
        p.deterministic = repo().join("output/missing_complete_gate");
        assert!(prepare(&p, &Request::engineering(7, 101)).is_err());
    }
    #[test]
    fn stale_current_implementation_is_rejected_read_only() {
        let p = real_paths();
        let prepared = prepare_frozen_evidence(&p).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        for rel in prepared.pins().keys() {
            let target = tmp.path().join(rel);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::copy(prepared.repo().join(rel), target).unwrap();
        }
        implementation_pins(&p.runtime, tmp.path()).unwrap();
        fs::write(tmp.path().join("crates/hg-train/src/optim.rs"), "stale").unwrap();
        assert!(implementation_pins(&p.runtime, tmp.path()).is_err());
    }
    #[test]
    fn changed_gate_predicates_and_classification_runtime_confusion_refuse() {
        let mut p = real_paths();
        let tmp = tempfile::tempdir().unwrap();
        for rel in files(&p.deterministic).unwrap() {
            let dst = tmp.path().join(&rel);
            fs::create_dir_all(dst.parent().unwrap()).unwrap();
            fs::copy(p.deterministic.join(rel), dst).unwrap();
        }
        p.deterministic = tmp.path().to_path_buf();
        let gate = p.deterministic.join("G-R2.json");
        let original = fs::read(&gate).unwrap();
        let mut changed: Value = serde_json::from_slice(&original).unwrap();
        changed["supported"] = json!(45);
        fs::write(&gate, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(predicates(&p).is_err());
        assert!(prepare(&p, &Request::engineering(7, 101)).is_err());
        fs::write(gate, original).unwrap();
        predicates(&p).unwrap();
        p.runtime = repo().join("output/ere_valid_optimizer_signal_panel_20260929");
        assert!(prepare(&p, &Request::engineering(7, 101)).is_err());
    }
    #[test]
    fn execution_boundary_rechecks_evidence_after_readiness() {
        let mut prepared = prepare_frozen_evidence(&real_paths()).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        for rel in files(&prepared.paths.runtime).unwrap() {
            let dst = tmp.path().join(&rel);
            fs::create_dir_all(dst.parent().unwrap()).unwrap();
            fs::copy(prepared.paths.runtime.join(rel), dst).unwrap();
        }
        prepared.paths.runtime = tmp.path().to_path_buf();
        prepared.recheck().unwrap();
        fs::write(tmp.path().join("summary.json"), "{}").unwrap();
        assert!(prepared.recheck().is_err());
    }
    #[test]
    fn historical_live_guard_refuses_versioned_scaling_native_identity() {
        assert!(prepare(&real_paths(), &Request::engineering(7, 101))
            .err()
            .unwrap()
            .contains("stale current implementation"));
    }
}
