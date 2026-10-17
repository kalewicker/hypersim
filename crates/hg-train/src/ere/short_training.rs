//! Prospectively registered 100-step complete-family training, with native receipts.
use super::{
    complete_admission::{self as a, ReceiptPaths, Res},
    runtime_signal as r, scaling,
};
use crate::optim::{standard_normal, OpenAiEs};
use hg_sim_core::ere::{
    signal,
    valid_grammar::{self, Grammar},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub const PROFILE: &str = "E2-complete-short-training-v1";
pub const CONTINUATION: &str = "E2-complete-short-training-continuation-v2";
const PREFIX_SEAL: &str = "813245dfa5ca432714a27e41a911fac75aef2951848867cc811a178b32d5ea5c";
pub const SEED: u64 = 0xE654000001;
const SIGNAL: &str = "crates/hg-sim-core/src/ere/signal.rs";
const PRE: [u64; 4] = [1601, 1699, 1801, 1901];
const TRAIN: [u64; 4] = [2003, 2111, 2203, 2309];
const HOLD: [u64; 4] = [2411, 2503, 2609, 2707];
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    pub profile: String,
    pub master_seed: u64,
    pub horizon: usize,
    pub parameters: usize,
    pub initial_parameter_value: f32,
    pub precheck_generations: usize,
    pub precheck_pairs: usize,
    pub generations: usize,
    pub candidates: usize,
    pub held_out_draws: usize,
    pub precheck_physics: Vec<u64>,
    pub training_physics: Vec<u64>,
    pub held_out_physics: Vec<u64>,
    pub sigma: f32,
    pub alpha: f32,
    pub minimum_valid_training_pairs: usize,
    pub work: usize,
    pub results: usize,
    pub wall_seconds: u64,
    pub cell_wall_seconds: u64,
    pub output_cap_bytes: u64,
}
impl Design {
    pub fn registered() -> Self {
        Self {
            profile: PROFILE.into(),
            master_seed: SEED,
            horizon: 100,
            parameters: 183,
            initial_parameter_value: 0.0,
            precheck_generations: 5,
            precheck_pairs: 4,
            generations: 20,
            candidates: 16,
            held_out_draws: 64,
            precheck_physics: PRE.to_vec(),
            training_physics: TRAIN.to_vec(),
            held_out_physics: HOLD.to_vec(),
            sigma: 1.0,
            alpha: 0.05,
            minimum_valid_training_pairs: 4,
            work: 50000,
            results: 50000,
            wall_seconds: 3600,
            cell_wall_seconds: 30,
            output_cap_bytes: 512 * 1024 * 1024,
        }
    }
}
pub fn repaired_source(old: &str) -> Res<String> {
    let source = scaling::bridged_source(old)?;
    let before = "        let before = s.step();\n        let stats = engine.step(&mut s);";
    a::check(
        source.matches(before).count() == 1,
        "event-capture bridge entry",
    )?;
    Ok(source.replacen(before, "        let before = s.step();\n        capture.lock().unwrap().clear();\n        let stats = engine.step(&mut s);", 1))
}
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn write(path: &Path, value: &Value) -> Res<()> {
    let tmp = path.with_extension("pending.json");
    fs::write(&tmp, serde_json::to_vec(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}
fn seal(out: &Path) -> Res<()> {
    let mut names = a::files(out)?;
    names.remove("output_sha256.json");
    let paths: Vec<_> = names.iter().map(|p| out.join(p)).collect();
    let hashes: BTreeMap<_, _> = names.into_iter().zip(a::hashes(&paths)?).collect();
    write(&out.join("output_sha256.json"), &json!(hashes))
}
fn snapshot_names(pins: &BTreeMap<String, String>) -> Res<BTreeSet<String>> {
    let mut names: BTreeSet<_> = pins.keys().cloned().collect();
    // Include the entire learning crate, not just the optimizer or new runner.
    for p in a::files(&repo().join("crates/hg-train"))? {
        if Path::new(&p)
            .extension()
            .is_some_and(|e| e == "rs" || e == "toml")
        {
            names.insert(format!("crates/hg-train/{p}"));
        }
    }
    for p in [
        "docs/ere_complete_short_training_v1.md",
        "configs/ere_complete_short_training_v1.json",
        "docs/ere_complete_short_training_continuation_v2.md",
        "configs/ere_complete_short_training_continuation_v2.json",
        "configs/ere_complete_receipts_v1.json",
        "scripts/audit_ere_complete_short_training.py",
        "scripts/test_audit_ere_complete_short_training.py",
        "scripts/audit_ere_complete_admission.py",
        "scripts/audit_ere_complete_scaling.py",
    ] {
        names.insert(p.into());
    }
    Ok(names)
}
fn prepare(paths: &ReceiptPaths) -> Res<(Value, BTreeMap<String, String>)> {
    let historical = a::prepare_frozen_evidence(paths)?;
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
        for p in a::files(&repo().join("crates").join(c))? {
            if Path::new(&p)
                .extension()
                .is_some_and(|e| e == "rs" || e == "toml")
            {
                coverage.insert(format!("crates/{c}/{p}"));
            }
        }
    }
    a::check(
        coverage == pins.keys().cloned().collect(),
        "native source coverage changed",
    )?;
    let old = fs::read_to_string(paths.runtime.join("source_snapshots").join(SIGNAL))
        .map_err(|e| e.to_string())?;
    a::check(
        fs::read_to_string(repo().join(SIGNAL)).map_err(|e| e.to_string())?
            == repaired_source(&old)?,
        "native change beyond registered horizon bridge and capture reset",
    )?;
    let current_paths: Vec<_> = pins.keys().map(|p| repo().join(p)).collect();
    for ((name, expected), actual) in pins.iter_mut().zip(a::hashes(&current_paths)?) {
        if name == SIGNAL {
            *expected = actual;
        } else {
            a::check(*expected == actual, &format!("native drift {name}"))?;
        }
    }
    // Check the already sealed control catalogue before the expensive rebuild.
    let catalogue = a::load(&paths.grammar.join("catalogue.json"))?;
    for (p, op) in [(4, 11), (5, 12)] {
        a::check(
            catalogue["parents"][p]["groups"][op]["roles"][1]["productions"]
                .as_array()
                .is_some_and(|x| !x.is_empty()),
            "missing registered control",
        )?;
    }
    Ok((
        json!({"profile":PROFILE,"historical_evidence":historical.evidence(),"implementation_sources_verified":pins.len(),
        "native_source_bridge":{"source":SIGNAL,"historical_sha256":historical.pins()[SIGNAL],"current_sha256":pins[SIGNAL],"exact_registered_transform":true},
        "user_authorized_training":true,"admitted_horizon":100,"admitted_generations":20,"admitted_candidates":16}),
        pins,
    ))
}
fn freeze(out: &Path, pins: &BTreeMap<String, String>) -> Res<Value> {
    let names = snapshot_names(pins)?;
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
fn recheck(snapshot: &Value) -> Res<()> {
    let sources: BTreeMap<String, String> =
        serde_json::from_value(snapshot["sources"].clone()).map_err(|e| e.to_string())?;
    let paths: Vec<_> = sources.keys().map(|p| repo().join(p)).collect();
    for ((p, expected), actual) in sources.iter().zip(a::hashes(&paths)?) {
        a::check(*expected == actual, &format!("live source changed {p}"))?;
    }
    a::check(
        a::digest(&std::env::current_exe().map_err(|e| e.to_string())?)?
            == snapshot["binary_sha256"].as_str().ok_or("binary hash")?,
        "executed binary changed",
    )
}
pub fn schedule() -> Vec<Value> {
    let mut rows = vec![];
    let mut add = |id: String, stage: &str, seed: u64, physics: u64| {
        rows.push(json!({"id":id,"stage":stage,"emission_seed":seed,"physics_seed":physics,"horizon":100}))
    };
    for i in 0..8 {
        for s in PRE {
            add(format!("baseline-{i}-{s}"), "precheck", SEED + 2000 + i, s);
        }
    }
    for active in [true, false] {
        for s in PRE {
            add(format!("control-{active}-{s}"), "precheck", SEED + 3000, s);
        }
    }
    for g in 0..5 {
        for p in 0..4 {
            for sign in [1, -1] {
                for s in &PRE[..2] {
                    add(
                        format!("pre-es-{g}-{p}-{sign}-{s}"),
                        "precheck",
                        SEED + 4000 + 4 * g + p,
                        *s,
                    );
                }
            }
        }
    }
    for g in 0..20 {
        for p in 0..8 {
            for sign in [1, -1] {
                let s = TRAIN[((g + p) % 4) as usize];
                add(
                    format!("train-es-{g}-{p}-{sign}-{s}"),
                    "training",
                    SEED + 5000 + 8 * g + p,
                    s,
                );
            }
        }
    }
    for i in 0..64 {
        for arm in ["champion", "untrained", "random"] {
            for s in HOLD {
                add(
                    format!("hold-{i}-{arm}-{s}"),
                    "held_out",
                    SEED + 6000 + i,
                    s,
                );
            }
        }
    }
    rows
}
pub fn terminal<T>(f: impl FnOnce() -> Res<T>) -> Res<T> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => Err(format!(
            "native panic: {}",
            payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-string panic")
        )),
    }
}
struct Campaign<'a> {
    out: &'a Path,
    manifest: Value,
    design: Design,
    deadline: Instant,
    rows: Vec<Value>,
    plan: Vec<Value>,
}
impl Campaign<'_> {
    fn save(&self) -> Res<()> {
        write(&self.out.join("manifest.json"), &self.manifest)
    }
    fn evaluate(&mut self, id: &str, e: &signal::Emission, theta: &[f32]) -> Res<Option<f64>> {
        let q = self.plan[self.rows.len()].clone();
        a::check(
            q["id"] == id && q["emission_seed"] == e.record["emission_seed"],
            "evaluation schedule divergence",
        )?;
        a::check(Instant::now() < self.deadline, "campaign wall ceiling")?;
        // Each evaluation is one durable file; all NotRun slots already exist.
        self.manifest["current_cell"] = q.clone();
        self.manifest["inflight_native_work_unknown"] = json!(true);
        self.save()?;
        let start = Instant::now();
        let limit = self
            .deadline
            .min(start + Duration::from_secs(self.design.cell_wall_seconds));
        let result = terminal(|| {
            signal::evaluate_with_horizon(
                e,
                q["physics_seed"].as_u64().unwrap(),
                limit,
                50000,
                50000,
                100,
            )
        });
        let mut row = json!({"schedule":q,"emission":e.record,"parameters":theta,"elapsed_seconds":start.elapsed().as_secs_f64()});
        let failure;
        match result {
            Ok(mut v) => {
                v["runtime_match_budget"] = json!({"policy":"Pooled","work":50000,"results":50000});
                row["status"] = json!(if v["score"].is_number() {
                    "Complete"
                } else {
                    "Null"
                });
                row["evaluation"] = v;
                failure = None;
            }
            Err(error) => {
                row["status"] = json!("Incomplete");
                row["reason"] = json!(error);
                row["evaluation"] = Value::Null;
                row["partial_steps_unknown"] = json!(true);
                failure = Some(error);
                self.manifest["unknown_partial_work_cells"] = json!(
                    self.manifest["unknown_partial_work_cells"]
                        .as_u64()
                        .unwrap()
                        + 1
                );
            }
        }
        let size = serde_json::to_vec(&row).map_err(|e| e.to_string())?.len() as u64;
        a::check(
            self.manifest["receipt_bytes"].as_u64().unwrap() + size
                < self.design.output_cap_bytes - 64 * 1024 * 1024,
            "receipt output ceiling",
        )?;
        write(&self.out.join("cells").join(format!("{id}.json")), &row)?;
        if !row["evaluation"].is_null() {
            let stage = q["stage"].as_str().unwrap();
            for field in ["attempted_steps", "advancing_steps"] {
                self.manifest["counts"][stage][field] = json!(
                    self.manifest["counts"][stage][field].as_u64().unwrap()
                        + row["evaluation"][field].as_u64().ok_or("native counts")?
                );
            }
        }
        self.manifest["receipt_bytes"] =
            json!(self.manifest["receipt_bytes"].as_u64().unwrap() + size);
        self.manifest["recorded_cells"] = json!(self.rows.len() + 1);
        self.manifest["inflight_native_work_unknown"] = json!(false);
        self.save()?;
        let score = row["evaluation"]["score"].as_f64();
        self.rows.push(row);
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(score)
    }
}
fn distribution(grammar: &Grammar, theta: &[f32]) -> Res<Vec<Value>> {
    (0..64).map(|i|grammar.emit(theta,SEED+1000+i).map(|e|json!({"seed":SEED+1000+i,"signature":e.record["signature"],"selection":e.record}))).collect()
}
fn es(c: &mut Campaign<'_>, grammar: &Grammar, pre: bool) -> Res<(Vec<f32>, usize, f64)> {
    let mut theta = vec![0.0; 183];
    let mut optimizer = OpenAiEs::new(
        1.0,
        0.05,
        if pre { 4 } else { 8 },
        ChaCha8Rng::seed_from_u64(if pre { SEED } else { SEED + 10000 }),
    );
    let initial = distribution(grammar, &theta)?;
    let mut max_tv = 0.0f64;
    let mut valid_total = 0;
    for g in 0..if pre { 5 } else { 20 } {
        recheck(&c.manifest["snapshot"])?;
        let before = theta.clone();
        let eps = optimizer.perturbations(183);
        let mut pairs = vec![];
        let mut eval_ids = vec![];
        for (p, noise) in eps.iter().enumerate() {
            let seed = SEED
                + if pre {
                    4000 + 4 * g + p as u64
                } else {
                    5000 + 8 * g + p as u64
                };
            let physics = if pre {
                PRE[..2].to_vec()
            } else {
                vec![TRAIN[((g + p as u64) % 4) as usize]]
            };
            let mut arms = vec![];
            for sign in [1, -1] {
                let candidate = optimizer.candidate(&theta, noise, sign as f32);
                let e = grammar.emit(&candidate, seed)?;
                let mut scores = vec![];
                for &s in &physics {
                    let id = format!(
                        "{}-es-{g}-{p}-{sign}-{s}",
                        if pre { "pre" } else { "train" }
                    );
                    scores.push(c.evaluate(&id, &e, &candidate)?);
                    eval_ids.push(id);
                }
                arms.push(scores);
            }
            pairs.push(r::pair(&arms[0], &arms[1]));
        }
        let valid = pairs.iter().filter(|p| p.plus.is_some()).count();
        valid_total += valid;
        let admitted = pre || valid >= 4;
        if admitted {
            optimizer.update(&mut theta, &eps, &pairs);
        }
        let panel = distribution(grammar, &theta)?;
        let tv = r::tv(&initial, &panel);
        max_tv = max_tv.max(tv);
        write(
            &c.out
                .join("generations")
                .join(format!("{}-{g}.json", if pre { "pre" } else { "train" })),
            &json!({"generation":g,"theta_before":before,"eps":eps,"evaluation_ids":eval_ids,
                "pair_scores":pairs.iter().map(|p|json!([p.plus,p.minus])).collect::<Vec<_>>(),"valid_pairs":valid,
                "update_applied":admitted,"theta_after":theta,"distribution":panel,"tv_from_initial":tv}),
        )?;
        a::check(
            admitted,
            "fewer than four valid training pairs; campaign stopped",
        )?;
        println!(
            "{} generation {}: {valid} valid pairs, TV {tv:.6}",
            if pre { "precheck" } else { "training" },
            g + 1
        );
    }
    Ok((theta, valid_total, max_tv))
}
fn prechecks(c: &mut Campaign<'_>, grammar: &Grammar) -> Res<bool> {
    let zero = vec![0.0; 183];
    let initial = distribution(grammar, &zero)?;
    write(&c.out.join("initial_distribution.json"), &json!(initial))?;
    let mut baseline = vec![];
    for i in 0..8 {
        let e = grammar.emit(&zero, SEED + 2000 + i)?;
        for s in PRE {
            baseline.push(c.evaluate(&format!("baseline-{i}-{s}"), &e, &zero)?);
        }
    }
    let active = grammar.emit_selected(4, 11, 1, 0, SEED + 3000)?;
    let dormant = grammar.emit_selected(5, 12, 1, 0, SEED + 3000)?;
    let p = active.program.as_ref().ok_or("active missing")?;
    let q = dormant.program.as_ref().ok_or("dormant missing")?;
    a::check(
        p.active
            && !q.active
            && p.core == q.core
            && p.proposal == q.proposal
            && active.record["initial_world_hash"] == dormant.record["initial_world_hash"],
        "controls not matched",
    )?;
    let mut controls = vec![];
    for (on, e) in [(true, &active), (false, &dormant)] {
        for s in PRE {
            controls.push(c.evaluate(&format!("control-{on}-{s}"), e, &zero)?);
        }
    }
    let (_, valid, tv) = es(c, grammar, true)?;
    let b: Option<Vec<_>> = baseline.into_iter().collect();
    let control: Option<Vec<_>> = controls.into_iter().collect();
    let mean = b.as_ref().map(|x| r::mean(x));
    let sd = b.as_ref().map(|x| r::sd(x));
    let gap = control
        .as_ref()
        .map(|x| r::mean(&x[..4]) - r::mean(&x[4..]));
    let p1 = !initial
        .iter()
        .any(|x| x["signature"] == active.record["signature"]);
    let p2 = tv >= 0.05 && valid == 20;
    let p3 = gap.zip(sd).is_some_and(|(g, s)| g > 2.0 * s && g > 1e-12);
    let p4 = mean.is_some_and(|m| m > 0.02 && m < 0.98);
    let envelope = c.rows.len() == 120
        && c.rows
            .iter()
            .all(|x| x["status"] == "Complete" && x["evaluation"]["advancing_steps"] == 100);
    let pass = p1 && p2 && p3 && p4 && envelope;
    write(
        &c.out.join("prechecks.json"),
        &json!({"checks":{"P1":p1,"P2":p2,"P3":p3,"P4":p4},
        "runtime_envelope_pass":envelope,"baseline_mean":mean,"baseline_population_sd":sd,"forced_gap":gap,"valid_pairs":valid,"max_distribution_tv":tv,
        "precheck_pass":pass,"rollouts":120,"training_admitted":false}),
    )?;
    Ok(pass)
}
pub fn sign_p(wins: usize, losses: usize) -> f64 {
    let n = wins + losses;
    if n == 0 {
        return 1.0;
    }
    let mut term = 2.0f64.powi(-(n as i32));
    let mut sum = 0.0;
    for k in 0..=n {
        if k >= wins {
            sum += term;
        }
        if k < n {
            term *= (n - k) as f64 / (k + 1) as f64;
        }
    }
    sum
}
fn held_out(c: &mut Campaign<'_>, grammar: &Grammar, theta: &[f32]) -> Res<Value> {
    let zero = vec![0.0; 183];
    let mut draws = vec![];
    let mut random_parameters = vec![];
    for i in 0..64 {
        let mut rng = ChaCha8Rng::seed_from_u64(SEED + 7000 + i);
        let random: Vec<f32> = (0..183).map(|_| standard_normal(&mut rng)).collect();
        random_parameters.push(random.clone());
        let mut scores = BTreeMap::new();
        for (arm, t) in [
            ("champion", theta),
            ("untrained", zero.as_slice()),
            ("random", random.as_slice()),
        ] {
            let e = grammar.emit(t, SEED + 6000 + i)?;
            let mut sample = vec![];
            for s in HOLD {
                sample.push(c.evaluate(&format!("hold-{i}-{arm}-{s}"), &e, t)?);
            }
            scores.insert(
                arm,
                sample
                    .iter()
                    .copied()
                    .collect::<Option<Vec<_>>>()
                    .map(|x| r::mean(&x)),
            );
        }
        draws.push(scores);
    }
    write(
        &c.out.join("random_parameters.json"),
        &json!(random_parameters),
    )?;
    let all = draws.iter().all(|d| d.values().all(Option::is_some));
    let mut wins = 0;
    let mut losses = 0;
    let mut differences = vec![];
    let mut random_differences = vec![];
    for d in &draws {
        if let (Some(a), Some(b)) = (d["champion"], d["untrained"]) {
            let delta = a - b;
            wins += usize::from(delta > 0.0);
            losses += usize::from(delta < 0.0);
            differences.push(delta);
        }
        if let (Some(a), Some(b)) = (d["champion"], d["random"]) {
            random_differences.push(a - b);
        }
    }
    let gap = if all {
        Some(r::mean(&differences))
    } else {
        None
    };
    let random_gap = if all {
        Some(r::mean(&random_differences))
    } else {
        None
    };
    let p = if all {
        Some(sign_p(wins, losses))
    } else {
        None
    };
    Ok(
        json!({"draws":draws,"fixed_denominator":64,"all_held_out_valid":all,"complete_primary_pairs":differences.len(),
        "champion_minus_untrained":gap,"champion_minus_random":random_gap,"primary_wins":wins,"primary_losses":losses,
        "primary_ties":differences.len()-wins-losses,"one_sided_paired_sign_p":p,
        "held_out_improvement_supported":gap.zip(p).is_some_and(|(g,p)|g>0.0 && p<0.05),
        "champion_selection":"final_generation_parameters","independent_physics_replay":false}),
    )
}
pub fn run(paths: &ReceiptPaths, out: &Path) -> Res<()> {
    run_inner(paths, out, None)
}
pub fn run_continuation(paths: &ReceiptPaths, prefix: &Path, out: &Path) -> Res<()> {
    a::check(
        a::digest(&prefix.join("output_sha256.json"))? == PREFIX_SEAL,
        "registered continuation prefix identity",
    )?;
    let target = out
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("output name")?);
    a::check(
        !target.starts_with(prefix.canonicalize().map_err(|e| e.to_string())?),
        "continuation output inside prefix",
    )?;
    run_inner(paths, out, Some(prefix))
}
fn run_inner(paths: &ReceiptPaths, out: &Path, prefix: Option<&Path>) -> Res<()> {
    a::check(!out.exists(), "fresh training output required")?;
    let absolute = out
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("output name")?);
    for p in paths.directories() {
        a::check(
            !absolute.starts_with(p.canonicalize().map_err(|e| e.to_string())?),
            "output inside prerequisite",
        )?;
    }
    let mut design = Design::registered();
    if prefix.is_some() {
        design.profile = CONTINUATION.into();
    }
    let plan = schedule();
    fs::create_dir(out).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("cells")).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("generations")).map_err(|e| e.to_string())?;
    for q in &plan {
        write(
            &out.join("cells")
                .join(format!("{}.json", q["id"].as_str().unwrap())),
            &json!({"schedule":q,"status":"NotRun","reason":"not_started","evaluation":null}),
        )?;
    }
    let start = Instant::now();
    let mut c = Campaign {
        out,
        design: design.clone(),
        deadline: start + Duration::from_secs(design.wall_seconds),
        rows: vec![],
        plan,
        manifest: json!({"profile":design.profile,"design":design,"schedule":schedule(),"status":"validating","reason":null,
            "receipt_paths":paths,"training_admitted":false,"full_training_authorized":false,"grammar_compilations":0,
            "scheduled_cells":1208,"recorded_cells":0,"unknown_partial_work_cells":0,"inflight_native_work_unknown":false,"receipt_bytes":0,
            "counts":{"precheck":{"attempted_steps":0,"advancing_steps":0},"training":{"attempted_steps":0,"advancing_steps":0},"held_out":{"attempted_steps":0,"advancing_steps":0}}}),
    };
    c.save()?;
    let result = terminal(|| {
        let config: Design = serde_json::from_value(a::load(&repo().join(if prefix.is_some() {
            "configs/ere_complete_short_training_continuation_v2.json"
        } else {
            "configs/ere_complete_short_training_v1.json"
        }))?)
        .map_err(|e| e.to_string())?;
        a::check(
            config == design,
            "training config differs from registered design",
        )?;
        let (evidence, pins) = prepare(paths)?;
        c.manifest["receipt_paths"] = json!(evidence["historical_evidence"]["receipts"]
            .as_object()
            .ok_or("historical receipt paths")?
            .iter()
            .map(|(name, proof)| (name.clone(), proof["archive"].clone()))
            .collect::<BTreeMap<_, _>>());
        c.manifest["admission"] = evidence;
        c.manifest["snapshot"] = freeze(out, &pins)?;
        c.save()?;
        if let Some(prefix) = prefix {
            let checked = std::process::Command::new("python3")
                .arg("-B")
                .arg(out.join("source_snapshots/scripts/audit_ere_complete_short_training.py"))
                .arg(prefix)
                .arg("--review-prefix")
                .output()
                .map_err(|e| e.to_string())?;
            a::check(
                checked.status.success(),
                &format!(
                    "prefix review failed: {}",
                    String::from_utf8_lossy(&checked.stderr)
                ),
            )?;
            let review: Value =
                serde_json::from_slice(&checked.stdout).map_err(|e| e.to_string())?;
            a::check(
                review["audit_pass"] == true
                    && review["precheck_pass"] == true
                    && review["original_frozen_checker_pass"] == false
                    && review["prefix_seal"] == PREFIX_SEAL,
                "prefix accounting review refusal",
            )?;
            let old = a::load(&prefix.join("manifest.json"))?;
            for (name, hash) in &pins {
                a::check(
                    old["snapshot"]["sources"][name] == *hash,
                    &format!("prefix native drift {name}"),
                )?;
            }
            write(&out.join("prefix_accounting_review.json"), &review)?;
            c.manifest["precheck_prefix"] =
                json!(prefix.canonicalize().map_err(|e| e.to_string())?);
            c.manifest["precheck_steps_reused"] = json!(12000);
            c.manifest["new_precheck_native_steps"] = json!(0);
            for q in c.plan.iter().take(120) {
                let name = format!("cells/{}.json", q["id"].as_str().unwrap());
                fs::copy(prefix.join(&name), out.join(&name)).map_err(|e| e.to_string())?;
                c.rows.push(a::load(&out.join(&name))?);
            }
            for name in [
                "prechecks.json".to_string(),
                "initial_distribution.json".to_string(),
            ]
            .into_iter()
            .chain((0..5).map(|g| format!("generations/pre-{g}.json")))
            {
                fs::copy(prefix.join(&name), out.join(&name)).map_err(|e| e.to_string())?;
            }
            c.manifest["counts"]["precheck"] = old["counts"]["precheck"].clone();
            c.manifest["recorded_cells"] = json!(120);
            c.manifest["receipt_bytes"] = json!(c
                .rows
                .iter()
                .map(|r| serde_json::to_vec(r).unwrap().len() as u64)
                .sum::<u64>());
            c.save()?;
        }
        c.manifest["status"] = json!("compiling");
        c.manifest["grammar_compilations"] = json!(1);
        c.save()?;
        let compile_start = Instant::now();
        let grammar = valid_grammar::compile()?;
        c.manifest["grammar_compile_seconds"] = json!(compile_start.elapsed().as_secs_f64());
        fs::write(
            out.join("catalogue.json"),
            serde_json::to_vec(&grammar.catalogue()).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        a::check(
            a::digest(&out.join("catalogue.json"))? == a::CATALOGUE_SHA,
            "catalogue drift",
        )?;
        recheck(&c.manifest["snapshot"])?;
        c.manifest["status"] = json!("prechecks");
        c.save()?;
        let pass = if prefix.is_some() {
            a::load(&out.join("prechecks.json"))?["precheck_pass"] == true
        } else {
            prechecks(&mut c, &grammar)?
        };
        c.manifest["prechecks_pass"] = json!(pass);
        c.save()?;
        a::check(
            pass,
            "fresh stateability or runtime precheck failed; training did not start",
        )?;
        let checked = std::process::Command::new("python3")
            .arg("-B")
            .arg(out.join("source_snapshots/scripts/audit_ere_complete_short_training.py"))
            .arg(out)
            .arg("--admission-check")
            .output()
            .map_err(|e| e.to_string())?;
        a::check(
            checked.status.success(),
            &format!(
                "frozen precheck audit failed: {}",
                String::from_utf8_lossy(&checked.stderr)
            ),
        )?;
        let audit: Value = serde_json::from_slice(&checked.stdout).map_err(|e| e.to_string())?;
        a::check(
            audit["audit_pass"] == true && audit["precheck_pass"] == true,
            "frozen precheck audit refusal",
        )?;
        write(&out.join("precheck_audit.json"), &audit)?;
        c.manifest["precheck_audit_pass"] = json!(true);
        c.manifest["training_admitted"] = json!(true);
        c.manifest["status"] = json!("training");
        c.save()?;
        let (theta, pairs, tv) = es(&mut c, &grammar, false)?;
        write(
            &out.join("champion.json"),
            &json!({"parameters":theta,"selection":"final_generation_parameters","generation":20,"valid_pairs":pairs,"max_distribution_tv":tv}),
        )?;
        recheck(&c.manifest["snapshot"])?;
        c.manifest["status"] = json!("held_out");
        c.save()?;
        let mut summary = held_out(&mut c, &grammar, &theta)?;
        summary["profile"] = json!(design.profile);
        summary["counts"] = c.manifest["counts"].clone();
        summary["training_complete"] = json!(true);
        summary["training_admitted"] = json!(true);
        summary["full_training_authorized"] = json!(false);
        write(&out.join("summary.json"), &summary)?;
        recheck(&c.manifest["snapshot"])?;
        prepare(paths)?;
        let total: u64 = a::files(out)?
            .iter()
            .map(|p| {
                fs::metadata(out.join(p))
                    .map(|m| m.len())
                    .map_err(|e| e.to_string())
            })
            .collect::<Res<Vec<_>>>()?
            .iter()
            .sum();
        a::check(
            total < design.output_cap_bytes - 16 * 1024 * 1024,
            "total output ceiling",
        )?;
        a::check(Instant::now() < c.deadline, "campaign wall ceiling")?;
        c.manifest["status"] = json!("complete");
        c.manifest["measurement_complete"] = json!(c.rows.len() == 1208);
        c.manifest["live_sources_rechecked"] = json!(true);
        Ok(())
    });
    if let Err(error) = &result {
        c.manifest["status"] = json!(if c.rows.is_empty() {
            "blocked_before_evaluation"
        } else {
            "incomplete"
        });
        c.manifest["reason"] = json!(error);
    }
    c.manifest["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
    c.save()?;
    seal(out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delayed_cap_has_no_stale_event_or_fitness() {
        let e = signal::forced(true, 17).unwrap();
        let v = signal::evaluate_with_horizon(
            &e,
            1597,
            Instant::now() + Duration::from_secs(30),
            3000,
            50000,
            100,
        )
        .unwrap();
        assert!(v["attempted_steps"].as_u64().unwrap() > 1);
        assert_eq!(v["reason"], "resource_cap");
        assert!(v["score"].is_null());
        let steps = v["steps"].as_array().unwrap();
        assert!(!steps[0]["rule_ids"].as_array().unwrap().is_empty());
        assert_eq!(steps.last().unwrap()["rule_ids"], json!([]));
        assert_eq!(steps.last().unwrap()["advanced"], 0);
        assert_eq!(
            v["proposal_fires"].as_u64().unwrap(),
            steps.len() as u64 - 1
        );
    }
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "full-budget delayed cap is verified separately in release mode"
    )]
    fn delayed_cap_at_registered_long_budget_returns_null() {
        let v = signal::evaluate_with_horizon(
            &signal::forced(true, 17).unwrap(),
            1361,
            Instant::now() + Duration::from_secs(30),
            50000,
            50000,
            10000,
        )
        .unwrap();
        assert_eq!(v["reason"], "resource_cap");
        assert!(v["score"].is_null());
        assert!(v["attempted_steps"].as_u64().unwrap() > 1000);
        assert_eq!(
            v["steps"].as_array().unwrap().last().unwrap()["rule_ids"],
            json!([])
        );
    }
    #[test]
    fn failures_are_caught_and_written_before_campaign_stops() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join("cells")).unwrap();
        let plan = schedule();
        let q = plan[0].clone();
        let e = signal::forced(true, q["emission_seed"].as_u64().unwrap()).unwrap();
        let mut c = Campaign {
            out: tmp.path(),
            manifest: json!({"counts":{"precheck":{"attempted_steps":0,"advancing_steps":0}},"receipt_bytes":0,"unknown_partial_work_cells":0}),
            design: Design::registered(),
            deadline: Instant::now() + Duration::from_secs(30),
            rows: vec![],
            plan,
        };
        // Panic capture itself is tested without consuming any native work.
        assert!(terminal::<()>(|| panic!("injected native failure"))
            .unwrap_err()
            .contains("injected native failure"));
        c.design.cell_wall_seconds = 0;
        assert!(c
            .evaluate(q["id"].as_str().unwrap(), &e, &vec![0.0; 183])
            .is_err());
        let row = a::load(
            &tmp.path()
                .join("cells")
                .join(format!("{}.json", q["id"].as_str().unwrap())),
        )
        .unwrap();
        assert_eq!(row["status"], "Incomplete");
        assert_eq!(row["partial_steps_unknown"], true);
        assert_eq!(c.manifest["recorded_cells"], 1);
        assert_eq!(c.manifest["inflight_native_work_unknown"], false);
    }
    #[test]
    fn registered_schedule_and_exact_reliability_bridge() {
        let plan = schedule();
        assert_eq!(plan.len(), 1208);
        for (stage, count) in [("precheck", 120), ("training", 320), ("held_out", 768)] {
            assert_eq!(plan.iter().filter(|q| q["stage"] == stage).count(), count);
        }
        let runtime = repo()
            .join("output/ere_runtime50k_signal_panel_20260930/source_snapshots")
            .join(SIGNAL);
        assert_eq!(
            repaired_source(&fs::read_to_string(runtime).unwrap()).unwrap(),
            fs::read_to_string(repo().join(SIGNAL)).unwrap()
        );
        let config: Design = serde_json::from_value(
            a::load(&repo().join("configs/ere_complete_short_training_v1.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(config, Design::registered());
        assert_eq!(sign_p(0, 0), 1.0);
        assert_eq!(sign_p(6, 0), 1.0 / 64.0);
        assert_eq!(sign_p(0, 6), 1.0);
    }
    #[test]
    fn registered_native_admission_checks_real_evidence_readonly() {
        let bundle = repo().join("configs/ere_complete_receipts_v1.json");
        let mut paths: ReceiptPaths = serde_json::from_value(a::load(&bundle).unwrap()).unwrap();
        paths.resolve_relative(bundle.parent().unwrap());
        let (proof, pins) = prepare(&paths).unwrap();
        assert_eq!(pins.len(), 230);
        assert_eq!(proof["admitted_horizon"], 100);
        assert_eq!(proof["user_authorized_training"], true);
        paths.runtime = repo().join("output/missing-short-training-receipt");
        assert!(prepare(&paths).is_err());
    }
}
