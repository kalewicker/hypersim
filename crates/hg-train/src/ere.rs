//! G_theta emits homomorphic seed graphs through the shared ERE rollout. The
//! optimizer never mutates an inner graph and imports no engine crate.
use crate::optim::{standard_normal, OpenAiEs, PairResult};
use hg_sim_core::{ere::experiments::seed_bank, SimConfig};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
pub const PARAMS: usize = 64;
pub const GROUPS: usize = 4;
pub const PRODUCTIONS: usize = 16;
pub mod e2;
pub mod maintenance;
pub mod vocabulary;
pub mod persistence;
pub mod repair;
pub mod rule_choice;
pub mod rule_choice_learning;
pub mod rule_composition;
pub mod rule_composition_learning;
pub mod rule_relational;
pub mod rule_relational_learning;
pub mod graph_encoder;
pub mod graph_encoder_learning;
pub mod graph_encoder_exact;
pub mod graph_encoder_confirmation;
pub mod rule_maintenance;
pub mod rule_prevention_transfer;
pub mod resource_adapter;
pub mod resource_timed_policy;
pub mod resource_learning;
pub mod resource_contrast_adapter;
#[cfg(feature = "ere-complete-signal")]
pub mod signal;
#[cfg(feature = "ere-complete-signal")]
pub mod valid_grammar;
#[cfg(feature = "ere-complete-signal")]
pub mod valid_signal;
#[cfg(feature = "ere-complete-signal")]
pub mod runtime_signal;
#[cfg(feature = "ere-complete-signal")]
pub mod complete_admission;
#[cfg(feature = "ere-complete-signal")]
pub mod complete_runner;
#[cfg(feature = "ere-complete-signal")]
pub mod scaling;
#[cfg(feature = "ere-complete-signal")]
pub mod short_training;
#[cfg(feature = "ere-complete-signal")]
pub mod confirmation;
#[cfg(feature = "ere-complete-signal")]
pub mod feasibility_training;
#[cfg(feature = "ere-complete-signal")]
pub mod minibatch;
#[cfg(feature = "ere-complete-signal")]
pub mod direction;
#[cfg(feature = "ere-complete-signal")]
pub mod categorical;
#[cfg(feature = "ere-complete-signal")]
pub mod categorical_comparison;
#[cfg(feature = "ere-complete-signal")]
pub mod categorical_comparison_bridge;
#[cfg(feature = "ere-complete-signal")]
pub mod categorical_feasible;
#[cfg(feature = "ere-complete-signal")]
pub mod categorical_feasible_development;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Budget {
    pub generations: usize,
    pub population: usize,
    pub steps_per_eval: u64,
    pub sigma: f32,
    pub alpha: f32,
    pub seed: u64,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            generations: 20,
            population: 16,
            steps_per_eval: 10_000,
            sigma: 1.0,
            alpha: 0.05,
            seed: 0xE1_5239_0001,
        }
    }
}
#[derive(Serialize)]
struct RawRuleSet {
    rules: Vec<hg_rules::Rule>,
}

/// Four grammar productions, each a softmax over 16 structural motifs. All
/// 64 logits participate, unlike a direct graph genome masquerading as R^d.
pub fn emit(theta: &[f32], seed: u64) -> Result<String, String> {
    if theta.len() != PARAMS || theta.iter().any(|x| !x.is_finite()) {
        return Err("G_theta requires 64 finite logits".into());
    }
    let bank = seed_bank();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut rules = Vec::new();
    for group in 0..GROUPS {
        let logits = &theta[group * PRODUCTIONS..(group + 1) * PRODUCTIONS];
        let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let weights: Vec<_> = logits.iter().map(|x| (*x - max).exp() as f64).collect();
        let mut draw = rng.gen::<f64>() * weights.iter().sum::<f64>();
        let mut index = PRODUCTIONS - 1;
        for (i, &w) in weights.iter().enumerate() {
            draw -= w;
            if draw <= 0.0 {
                index = i;
                break;
            }
        }
        rules.push(bank[index].to_rule(format!("grammar_{group}")));
    }
    // Rule's serde enum spelling is Rust-style (Gte); the parser's TOML
    // grammar requires lowercase op strings. Do not change the shared parser
    // or silently omit guards to make the grammar round trip.
    let mut value = toml::Value::try_from(RawRuleSet { rules }).map_err(|e| e.to_string())?;
    fn normalize(value: &mut toml::Value) {
        match value {
            toml::Value::Table(table) => {
                if let Some(toml::Value::String(op)) = table.get_mut("op") {
                    *op = op.to_lowercase();
                }
                for (_, v) in table.iter_mut() {
                    normalize(v);
                }
            }
            toml::Value::Array(values) => {
                for v in values {
                    normalize(v);
                }
            }
            _ => (),
        }
    }
    normalize(&mut value);
    toml::to_string(&value).map_err(|e| e.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Score {
    pub f5_reuse: f64,
    pub f4_pi: Option<f64>,
    pub ceiling_bound: bool,
}
fn evaluate(base: &SimConfig, theta: &[f32], seed: u64, steps: u64) -> Result<Score, String> {
    let mut config = base.clone();
    config.seed = seed;
    config.engine.max_steps = steps;
    config.rules.source = hg_sim_core::RuleSource::Inline;
    config.rules.inline_toml = Some(emit(theta, seed)?);
    config.rules.file_path = None;
    config.rules.builtin_name = None;
    config.causal.enabled = false;
    if let Some(ere) = config.ere.as_mut() {
        ere.measure_ei = false;
    }
    let ere = config
        .ere
        .as_ref()
        .ok_or("ERE training requires [ere] config")?;
    if ere.arm != hg_sim_core::ere::EreArm::A3 {
        return Err("ERE training requires A3, not a protected/frozen surrogate".into());
    }
    let run = hg_sim_core::runner::run_ere_rollout(&config).map_err(|e| e.to_string())?;
    let rows: Vec<_> = run
        .history
        .iter()
        .filter_map(|r| r.ere.as_deref())
        .collect();
    let last = rows
        .last()
        .ok_or("ERE rollout produced no measurement window")?;
    let reuse = last
        .nontrivial_rule_reuse_count_by_class
        .as_ref()
        .ok_or("unmeasured constructor reuse")?;
    let possible = (steps as f64 * config.engine.max_pairings_per_step as f64).max(1.0);
    let n = GROUPS as f64;
    let f5 = reuse
        .values()
        .map(|&fires| (fires.saturating_sub(1) as f64).ln_1p())
        .sum::<f64>()
        / (n * (possible / n).ln_1p());
    let pi: Vec<_> = rows
        .iter()
        .filter_map(|r| r.meta_excluded.as_ref()?.pi_normalized)
        .collect();
    Ok(Score {
        f5_reuse: f5.min(1.0),
        f4_pi: (!pi.is_empty()).then(|| pi.iter().sum::<f64>() / pi.len() as f64),
        ceiling_bound: rows
            .iter()
            .any(|r| r.rule_region_ceiling_binding || r.match_ceiling_binding),
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct Precheck {
    pub name: String,
    pub measured: f64,
    pub threshold: f64,
    pub passed: bool,
    pub detail: String,
}
pub fn prechecks(
    base: &SimConfig,
    theta: &[f32],
    budget: &Budget,
) -> Result<Vec<Precheck>, String> {
    let steps = budget.steps_per_eval.min(1000);
    let mut extreme = vec![-12.0; PARAMS];
    for g in 0..GROUPS {
        extreme[g * PRODUCTIONS] = 12.0;
    }
    let mut opposite = vec![-12.0; PARAMS];
    for g in 0..GROUPS {
        opposite[g * PRODUCTIONS + 9] = 12.0;
    }
    let distribution = |params: &[f32]| -> Result<BTreeMap<String, usize>, String> {
        let mut counts = BTreeMap::new();
        for seed in 0..64 {
            let signature = hg_sim_core::ere::experiments::canonical_seed_signature(&emit(
                params,
                budget.seed + seed,
            )?)?;
            *counts.entry(signature).or_default() += 1;
        }
        Ok(counts)
    };
    let initial = distribution(theta)?;
    let delta = distribution(&extreme)?;
    let different = delta.keys().filter(|s| !initial.contains_key(*s)).count() as f64;
    let keys: std::collections::BTreeSet<_> = initial.keys().chain(delta.keys()).collect();
    let tv = keys
        .iter()
        .map(|k| {
            (initial.get(*k).copied().unwrap_or(0) as f64
                - delta.get(*k).copied().unwrap_or(0) as f64)
                .abs()
        })
        .sum::<f64>()
        / 128.0;
    let mut baseline = Vec::new();
    for seed in 0..4 {
        baseline.push(evaluate(base, theta, budget.seed + seed, steps)?.f5_reuse);
    }
    let mean = baseline.iter().sum::<f64>() / baseline.len() as f64;
    let sd =
        (baseline.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / baseline.len() as f64).sqrt();
    let a = evaluate(base, &extreme, budget.seed, steps)?.f5_reuse;
    let b = evaluate(base, &opposite, budget.seed, steps)?.f5_reuse;
    let discrimination = (a - b).abs();
    let margin = mean.min(1.0 - mean);
    // P2 traverses with the REAL optimizer and objective, then measures
    // emitted-graph movement. A hand-picked 3-sigma displacement would only
    // show what the optimizer could do, not whether it actually does it.
    let mut moved = theta.to_vec();
    let mut probe_optimizer = OpenAiEs::new(
        budget.sigma,
        budget.alpha,
        4,
        ChaCha8Rng::seed_from_u64(budget.seed ^ 0x5032),
    );
    for generation in 0..5 {
        let eps = probe_optimizer.perturbations(PARAMS);
        let mut results = Vec::new();
        for e in &eps {
            let plus = probe_optimizer.candidate(&moved, e, 1.0);
            let minus = probe_optimizer.candidate(&moved, e, -1.0);
            let seed = budget.seed + generation;
            results.push(PairResult {
                plus: Some(evaluate(base, &plus, seed, steps)?.f5_reuse as f32),
                minus: Some(evaluate(base, &minus, seed, steps)?.f5_reuse as f32),
            });
        }
        probe_optimizer.update(&mut moved, &eps, &results);
    }
    let moved_distribution = distribution(&moved)?;
    let keys: std::collections::BTreeSet<_> =
        initial.keys().chain(moved_distribution.keys()).collect();
    let actual_tv = keys
        .iter()
        .map(|k| {
            (initial.get(*k).copied().unwrap_or(0) as f64
                - moved_distribution.get(*k).copied().unwrap_or(0) as f64)
                .abs()
        })
        .sum::<f64>()
        / 128.0;
    let l2 = theta
        .iter()
        .zip(&moved)
        .map(|(a, b)| ((*b - *a) as f64).powi(2))
        .sum::<f64>()
        .sqrt();
    Ok(vec![
        Precheck { name: "P1_CLASS_EXPRESSIBLE".into(), measured: different, threshold: 1.0, passed: different >= 1.0,
            detail: format!("extreme versus initialization induces seed-graph TV={tv:.6}") },
        Precheck { name: "P2_OPTIMIZER_TRAVERSAL".into(), measured: actual_tv, threshold: 0.05, passed: actual_tv >= 0.05,
            detail: format!("five real OpenAI-ES probe generations, eight candidates each; L2={l2:.6}; TV measured over 64 common seeds. As in cognition v2, TV is decisive and L2 diagnostic.") },
        Precheck { name: "P3_OBJECTIVE_DISCRIMINATION".into(), measured: discrimination, threshold: 2.0 * sd,
            passed: discrimination > 2.0 * sd && discrimination > 1e-12, detail: format!("baseline seed sd={sd:.6}") },
        Precheck { name: "P4_NOT_SATURATED".into(), measured: margin, threshold: 0.02, passed: margin > 0.02,
            detail: format!("initial F5'={mean:.6}; range [0,1]") },
    ])
}

/// Receipts are checked before probes or optimization. A short CI smoke
/// cannot be substituted for a full gate just by setting pass=true.
fn require_gates(dir: &Path, base: &SimConfig) -> Result<(), String> {
    if base.ere.as_ref().is_some_and(|e|e.vocabulary.is_some()) {
        return Err("vocabulary profile requires new full gates/stateability; legacy receipts cannot admit training".into());
    }
    hg_sim_core::ere::experiments::require_research_gates_with_policy(
        dir,
        base.ere
            .as_ref()
            .map(|c| c.encoding_version)
            .unwrap_or_default(),
        base.ere.as_ref().map(|c|c.match_budget_policy).unwrap_or_default(),
    )?;
    let load = |name| -> Result<serde_json::Value, String> {
        serde_json::from_slice(
            &std::fs::read(dir.join(name))
                .map_err(|e| format!("missing prerequisite {name}: {e}"))?,
        )
        .map_err(|e| e.to_string())
    };
    let gr1 = load("exp_r01.json")?;
    if gr1["pass"] != true
        || gr1["samples"].as_u64().unwrap_or(0) < 10_000
        || gr1["trajectory_steps"].as_u64().unwrap_or(0) < 100
    {
        return Err("G-R1 has not passed its full protocol; training halted".into());
    }
    let gr2 = load("exp_r02.json")?;
    if gr2.as_array().is_none_or(|rows| {
        rows.is_empty()
            || rows.iter().any(|r| {
                r["status"]
                    .as_str()
                    .is_none_or(|s| s != "identical" && !s.starts_with("outside-envelope:"))
                    || r["status"] == "identical" && r["steps"].as_u64().unwrap_or(0) < 1000
            })
    }) {
        return Err("G-R2 has not passed; training halted".into());
    }
    let gr3 = load("exp_r03.json")?;
    if gr3["pass"] != true || gr3["steps"].as_u64().unwrap_or(0) < 100_000 {
        return Err("G-R3 has not passed; training halted".into());
    }
    Ok(())
}

pub fn run_campaign(
    base: &SimConfig,
    out: &Path,
    receipts: &Path,
    budget: &Budget,
) -> Result<(), String> {
    if budget.population < 2
        || budget.population % 2 != 0
        || budget.generations == 0
        || budget.steps_per_eval == 0
        || budget.sigma <= 0.0
    {
        return Err("invalid precommitted budget".into());
    }
    if out.exists() {
        return Err(format!(
            "refusing to overwrite campaign directory {}",
            out.display()
        ));
    }
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    // Commit budget and base configuration BEFORE any probe; gating failures
    // still leave a truthful campaign record, not an empty successful report.
    let manifest = serde_json::json!({ "budget": budget, "base_config": base, "parameters": PARAMS,
        "precheck_steps_per_eval": budget.steps_per_eval.min(1000),
        "precheck_optimizer_generations": 5, "precheck_optimizer_population": 8,
        "held_out_control_evaluations": 24, "held_out_steps_per_eval": budget.steps_per_eval,
        "objective_primary": "F5_prime_nontrivial_reuse", "objective_secondary": "F4_prime_meta_excluded_PI",
        "controls": ["champion", "untrained", "random_resample"], "gate_receipts": receipts });
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    if let Err(error) = require_gates(receipts, base) {
        std::fs::write(
            out.join("status.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "status":"blocked_by_prerequisite", "reason":error,
                "optimizer_generations_completed":0, "optimizer_evaluation_steps":0,
                "precheck_evaluation_steps":0, "held_out_evaluation_steps":0
            }))
            .unwrap(),
        )
        .map_err(|e| e.to_string())?;
        std::fs::write(
            out.join("report.md"),
            format!("# EXP-R09\n\nNot launched: {error}\n"),
        )
        .map_err(|e| e.to_string())?;
        return Err(error);
    }
    let untrained = vec![0.0; PARAMS];
    let mut theta = untrained.clone();
    let checks = match prechecks(base, &theta, budget) {
        Ok(checks) => checks,
        Err(error) => {
            std::fs::write(
                out.join("report.md"),
                format!("# EXP-R09\n\nNot launched: precheck rollout failed: {error}\n"),
            )
            .map_err(|e| e.to_string())?;
            return Err(error);
        }
    };
    std::fs::write(
        out.join("prechecks.json"),
        serde_json::to_vec_pretty(&checks).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    if checks.iter().any(|c| !c.passed) {
        std::fs::write(
            out.join("report.md"),
            "# EXP-R09\n\nNot launched: stateability precheck failed. See prechecks.json.\n",
        )
        .map_err(|e| e.to_string())?;
        return Err("stateability precheck failed; no optimizer generations executed".into());
    }
    let mut optimizer = OpenAiEs::new(
        budget.sigma,
        budget.alpha,
        budget.population / 2,
        ChaCha8Rng::seed_from_u64(budget.seed),
    );
    let mut champion = theta.clone();
    let mut best = f64::NEG_INFINITY;
    let mut csv = String::from("generation,candidate,seed,f5_prime,f4_prime,status\n");
    for generation in 0..budget.generations {
        let eps = optimizer.perturbations(PARAMS);
        let mut pairs = Vec::new();
        let seed = budget.seed + generation as u64;
        for (pair, e) in eps.iter().enumerate() {
            let mut fitness = [None; 2];
            for (j, sign) in [1.0, -1.0].into_iter().enumerate() {
                let params = optimizer.candidate(&theta, e, sign);
                match evaluate(base, &params, seed, budget.steps_per_eval) {
                    Ok(score) if !score.ceiling_bound => {
                        fitness[j] = Some(score.f5_reuse as f32);
                        csv.push_str(&format!(
                            "{generation},{},{seed},{},{},ok\n",
                            pair * 2 + j,
                            score.f5_reuse,
                            score.f4_pi.map(|x| x.to_string()).unwrap_or_default()
                        ));
                        if score.f5_reuse > best {
                            best = score.f5_reuse;
                            champion = params;
                        }
                    }
                    result => {
                        let detail = format!("failed:{:?}", result.err()).replace('"', "\"\"");
                        csv.push_str(&format!(
                            "{generation},{},{seed},,,\"{detail}\"\n",
                            pair * 2 + j
                        ));
                    }
                }
            }
            pairs.push(PairResult {
                plus: fitness[0],
                minus: fitness[1],
            });
        }
        if pairs.iter().all(|p| p.plus.is_none() && p.minus.is_none()) {
            return Err("all evaluations failed; optimizer update withheld".into());
        }
        optimizer.update(&mut theta, &eps, &pairs);
        std::fs::write(out.join("campaign.csv"), &csv).map_err(|e| e.to_string())?;
    }
    let mut rng = ChaCha8Rng::seed_from_u64(budget.seed + 1_000_000);
    let mut controls = Vec::new();
    for seed in 0..8u64 {
        let seed = budget.seed + 10_000 + seed;
        let random: Vec<_> = (0..PARAMS).map(|_| standard_normal(&mut rng)).collect();
        controls.push(serde_json::json!({ "seed": seed,
            "champion": evaluate(base, &champion, seed, budget.steps_per_eval)?,
            "untrained": evaluate(base, &untrained, seed, budget.steps_per_eval)?,
            "random_resample": evaluate(base, &random, seed, budget.steps_per_eval)? }));
    }
    std::fs::write(
        out.join("champion.json"),
        serde_json::to_vec_pretty(&champion).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        out.join("controls.json"),
        serde_json::to_vec_pretty(&controls).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    let deltas: Vec<_> = controls
        .iter()
        .map(|c| {
            c["champion"]["f5_reuse"].as_f64().unwrap()
                - c["untrained"]["f5_reuse"].as_f64().unwrap()
        })
        .collect();
    let mean = deltas.iter().sum::<f64>() / deltas.len() as f64;
    let sd =
        (deltas.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (deltas.len() - 1) as f64).sqrt();
    let half_ci = 2.3646 * sd / (deltas.len() as f64).sqrt();
    std::fs::write(out.join("report.md"), format!("# EXP-R09\n\nCompleted precommitted OpenAI-ES budget. Primary paired comparison, champion minus untrained: {mean:.6} F5' (95% t interval [{:.6}, {:.6}], eight held-out seeds, df=7). See controls.json; F4' is secondary and F1-F3 are excluded.\n", mean - half_ci, mean + half_ci)).map_err(|e| e.to_string())?;
    Ok(())
}
