//! E2 family grammar and a bounded, separately accounted pilot. No full campaign.
use super::{OpenAiEs, PairResult, RawRuleSet};
use hg_sim_core::{
    ere::{self, EreArm, EreSeedFamily},
    SimConfig,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};

pub const PARAMS: usize = 40;
pub const GROUPS: usize = 4;
pub const PILOT_SEED: u64 = 0xE2_5239_0001;
pub const PILOT_STEPS: u64 = 1000;
pub const MAX_EVALUATIONS: usize = 58;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emission {
    pub core_toml: String,
    pub families: Vec<EreSeedFamily>,
}
fn choose(logits: &[f32], rng: &mut ChaCha8Rng) -> usize {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let weights: Vec<_> = logits.iter().map(|x| ((*x - max) as f64).exp()).collect();
    let mut draw = rng.gen::<f64>() * weights.iter().sum::<f64>();
    for (i, w) in weights.iter().enumerate() {
        draw -= w;
        if draw <= 0.0 {
            return i;
        }
    }
    logits.len() - 1
}
pub fn forced(core: usize, proposal: usize, active: bool) -> Vec<f32> {
    assert!(core < 4 && proposal < 4);
    let mut theta = vec![-12.0; PARAMS];
    for group in 0..GROUPS {
        theta[group * 10 + core] = 12.0;
        theta[group * 10 + 4 + proposal] = 12.0;
        theta[group * 10 + 8 + usize::from(active)] = 12.0;
    }
    theta
}
pub fn emit(theta: &[f32], seed: u64) -> Result<Emission, String> {
    if theta.len() != PARAMS || theta.iter().any(|x| !x.is_finite()) {
        return Err("E2 grammar requires 40 finite logits".into());
    }
    let serialize = |r| toml::to_string(&RawRuleSet { rules: vec![r] }).map_err(|e| e.to_string());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut families = Vec::new();
    let mut core_toml = String::new();
    for group in 0..GROUPS {
        let i = group * 10;
        let core_choice = choose(&theta[i..i + 4], &mut rng);
        let proposal_choice = choose(&theta[i + 4..i + 8], &mut rng);
        let active = choose(&theta[i + 8..i + 10], &mut rng) == 1;
        let mut core = ere::experiments::seed_bank()[0].clone();
        core.lhs = match core_choice {
            0 => vec![vec![0, 1]],
            1 => vec![vec![0, 1], vec![1, 2]],
            2 => vec![vec![0, 1, 2]],
            _ => vec![vec![0, 1], vec![0, 2]],
        };
        core.rhs = core.lhs.clone();
        core.labels = vec![None; if core_choice == 0 { 2 } else { 3 }];
        let mut proposal = core.clone();
        proposal.priority = 1;
        if proposal_choice != 0 {
            proposal.labels = vec![None; 3];
            proposal.lhs = vec![vec![0, 1], vec![1, 2]];
            proposal.rhs = if proposal_choice == 2 {
                vec![vec![1, 1], vec![0, 2]]
            } else {
                vec![vec![0, 1], vec![0, 2]]
            };
            if proposal_choice == 3 {
                proposal.labels[0] = Some(1);
            }
        }
        let core_text = serialize(core.to_rule(format!("e2_core_{group}")))?;
        let proposal_text = serialize(proposal.to_rule(format!("e2_proposal_{group}")))?;
        core_toml.push_str(&core_text);
        core_toml.push('\n');
        families.push(EreSeedFamily {
            core_toml: core_text,
            proposal_toml: proposal_text,
            active,
        });
    }
    ere::seed_family_signature(&families, true)?; // Whole-family validation; never repair.
    Ok(Emission {
        core_toml,
        families,
    })
}

pub fn configure(
    base: &SimConfig,
    emission: &Emission,
    seed: u64,
    steps: u64,
    arm: EreArm,
) -> Result<SimConfig, String> {
    let mut cfg = base.clone();
    cfg.seed = seed;
    cfg.engine.max_steps = steps;
    cfg.rules.source = hg_sim_core::RuleSource::Inline;
    cfg.rules.inline_toml = Some(emission.core_toml.clone());
    cfg.rules.file_path = None;
    cfg.rules.builtin_name = None;
    cfg.causal.enabled = false;
    let e = cfg
        .ere
        .as_mut()
        .ok_or("E2 grammar requires explicit ERE config")?;
    if e.encoding_version.protocol() != "E2-alternatives-v1"
        || e.match_budget_policy != hg_rules::MatchBudgetPolicy::Pooled
        || e.vocabulary.is_some()
    {
        return Err("E2 pilot requires explicit E2 pooled configuration".into());
    }
    e.seed_families = Some(emission.families.clone());
    e.arm = arm;
    e.measure_ei = false;
    e.meta_rules_toml = if matches!(arm, EreArm::A2 | EreArm::A3) {
        Some(ere::experiments::seeded_editor_versioned(
            e.encoding_version,
        ))
    } else {
        None
    };
    Ok(cfg)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub final_step: u64,
    pub final_hash: String,
    pub structural_proposal_score: Option<f64>,
    pub proposal_world_fires: usize,
    pub structural_proposal_world_fires: usize,
    pub activation_count: usize,
    pub anchor_survival_fraction: Option<f64>,
    pub cleanup_edges_lost: usize,
    pub quarantine_decisions: usize,
    /// All meta-touching/creating rewrites, not only the seeded paid editor.
    #[serde(alias = "meta_editor_events")]
    pub meta_rewrite_events: usize,
    pub novel_classes: usize,
    pub ceiling_bound: bool,
    pub valid: bool,
}
pub fn observe(
    base: &SimConfig,
    theta: &[f32],
    seed: u64,
    steps: u64,
    arm: EreArm,
) -> Result<Observation, String> {
    let emission = emit(theta, seed)?;
    let cfg = configure(base, &emission, seed, steps, arm)?;
    let run = hg_sim_core::runner::run_ere_rollout(&cfg).map_err(|e| e.to_string())?;
    let rows: Vec<_> = run
        .history
        .iter()
        .filter_map(|r| r.ere.as_deref())
        .collect();
    let last = rows.last().ok_or("unmeasured E2 rollout: no ERE window")?;
    let map = last
        .structural_proposal_reuse_count_by_class
        .as_ref()
        .ok_or("unmeasured structural proposal reuse")?;
    let possible = (steps * cfg.engine.max_pairings_per_step as u64) as f64;
    let score = map
        .values()
        .map(|fires| (fires.saturating_sub(1) as f64).ln_1p())
        .sum::<f64>()
        / (GROUPS as f64 * (possible / GROUPS as f64).ln_1p());
    let ceiling = rows
        .iter()
        .any(|r| r.match_ceiling_binding || r.rule_region_ceiling_binding);
    let valid = !ceiling && run.final_step == steps && score.is_finite();
    Ok(Observation {
        final_step: run.final_step,
        final_hash: format!("{:016x}", run.canonical_hash),
        structural_proposal_score: valid.then_some(score.clamp(0.0, 1.0)),
        proposal_world_fires: last
            .proposal_rule_reuse_count_by_class
            .as_ref()
            .unwrap()
            .values()
            .sum(),
        structural_proposal_world_fires: map.values().sum(),
        activation_count: last.runtime_activation_count.unwrap_or(0),
        anchor_survival_fraction: last.anchor_survival_fraction,
        cleanup_edges_lost: rows
            .iter()
            .map(|r| r.meta_edges_lost_to_cleanup.unwrap_or(0))
            .sum(),
        quarantine_decisions: rows.iter().map(|r| r.rules_quarantined.unwrap_or(0)).sum(),
        meta_rewrite_events: rows
            .iter()
            .map(|r| r.meta_rewrite_events.unwrap_or(0))
            .sum(),
        novel_classes: last.rule_rcf_novel_cumulative.unwrap_or(0),
        ceiling_bound: ceiling,
        valid,
    })
}

fn save<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    std::fs::write(
        dir.join(name),
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
fn distribution(theta: &[f32], genotype: bool) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for seed in 0..64 {
        let emitted = emit(theta, PILOT_SEED + seed)?;
        *counts
            .entry(ere::seed_family_signature(&emitted.families, genotype)?)
            .or_default() += 1;
    }
    Ok(counts)
}
fn tv(a: &BTreeMap<String, usize>, b: &BTreeMap<String, usize>) -> f64 {
    let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
    keys.iter()
        .map(|k| {
            (a.get(*k).copied().unwrap_or(0) as f64 - b.get(*k).copied().unwrap_or(0) as f64).abs()
        })
        .sum::<f64>()
        / 128.0
}
struct Evaluations<'a> {
    base: &'a SimConfig,
    out: &'a Path,
    log: Vec<Value>,
}
impl Evaluations<'_> {
    fn run(
        &mut self,
        label: &str,
        theta: &[f32],
        seed: u64,
        arm: EreArm,
    ) -> Result<Option<Observation>, String> {
        if self.log.len() >= MAX_EVALUATIONS {
            return Err("preregistered pilot budget exhausted".into());
        }
        let result = observe(self.base, theta, seed, PILOT_STEPS, arm);
        let observation = result.as_ref().ok().cloned();
        self.log.push(
            json!({"evaluation":self.log.len(),"label":label,"seed":seed,
            "arm":arm,"requested_steps":PILOT_STEPS,"theta":theta,
            "observation":observation,"error":result.err()}),
        );
        save(self.out, "evaluations.json", &self.log)?;
        Ok(observation)
    }
}

/// Fixed pilot only. Never calls the legacy campaign or reserves its optimizer.
pub fn run_pilot(
    base: &SimConfig,
    out: &Path,
    classification: &Path,
    gates: &Path,
) -> Result<(), String> {
    if out.exists() {
        return Err("refusing to overwrite E2 pilot directory".into());
    }
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut cfg = base.clone();
    cfg.engine.metrics_window_interval = 100;
    cfg.engine.max_pairings_per_step = 16;
    cfg.engine.thread_count = 1;
    cfg.evaluator.enabled = false;
    let e = cfg.ere.as_mut().ok_or("pilot requires ERE")?;
    if e.match_ceiling != 50_000 || e.rule_region_ceiling != 500 {
        return Err("pilot requires unchanged runtime 50k/500 region caps".into());
    }
    e.epoch = hg_rules::EpochPolicy::Fixed(10.try_into().unwrap());
    e.meta_seed_energy = 32.0;
    save(
        out,
        "manifest.json",
        &json!({"protocol":"E2-family-grammar-pilot-v1",
        "parameters":PARAMS,"families":GROUPS,"seed":PILOT_SEED,"max_evaluations":MAX_EVALUATIONS,
        "steps_per_eval":PILOT_STEPS,"maximum_simulation_steps":58_000,"base_config":cfg,
        "classification_receipts":classification,"downstream_receipts":gates,
        "precheck_generations":5,"precheck_population":8,"sigma":1.0,"alpha":0.05,
        "objective":"structurally_distinct_proposal_world_reuse_excluding_priority_only",
        "full_campaign":"not_launched","full_training_budget_reserved":super::Budget::default()}),
    )?;
    let admission = match ere::admission::require_pooled_400k50k_gates(classification, gates) {
        Ok(v) => v,
        Err(error) => {
            save(
                out,
                "status.json",
                &json!({"status":"blocked_by_admission",
            "error":error,"pilot_evaluations":0,"full_optimizer_evaluation_steps":0}),
            )?;
            return Err(error);
        }
    };
    save(out, "admission.json", &admission)?;
    let initial = vec![0.0; PARAMS];
    let active = forced(0, 1, true);
    let dormant = forced(0, 0, false);
    save(
        out,
        "emitted_examples.json",
        &json!({"initial":emit(&initial,PILOT_SEED)?,
        "forced_structural_active":emit(&active,PILOT_SEED)?,"forced_priority_dormant":emit(&dormant,PILOT_SEED)?}),
    )?;
    let mut ev = Evaluations {
        base: &cfg,
        out,
        log: Vec::new(),
    };
    let mut controls = Vec::new();
    for seed in PILOT_SEED..PILOT_SEED + 4 {
        for arm in [EreArm::A1, EreArm::A2, EreArm::A3] {
            let r = ev.run("matched_control", &initial, seed, arm)?;
            controls.push(json!({"seed":seed,"arm":arm,"observation":r}));
        }
    }
    save(out, "controls.json", &controls)?;
    let before_genotypes = distribution(&initial, true)?;
    let initial_execution = distribution(&initial, false)?;
    let forced_execution = distribution(&active, false)?;
    let distinct = forced_execution
        .keys()
        .filter(|k| !initial_execution.contains_key(*k))
        .count();
    let mut baseline = Vec::new();
    for seed in PILOT_SEED..PILOT_SEED + 4 {
        baseline.push(
            ev.run("stateability_baseline", &initial, seed, EreArm::A3)?
                .and_then(|o| o.structural_proposal_score),
        );
    }
    let a = ev
        .run("forced_structural_active", &active, PILOT_SEED, EreArm::A3)?
        .and_then(|o| o.structural_proposal_score);
    let b = ev
        .run("forced_priority_dormant", &dormant, PILOT_SEED, EreArm::A3)?
        .and_then(|o| o.structural_proposal_score);
    let scores: Option<Vec<_>> = baseline.into_iter().collect();
    let stats = scores.map(|s| {
        let mean = s.iter().sum::<f64>() / 4.0;
        let sd = (s.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / 4.0).sqrt();
        (mean, sd)
    });
    let mut moved = initial.clone();
    let mut optimizer = OpenAiEs::new(1.0, 0.05, 4, ChaCha8Rng::seed_from_u64(PILOT_SEED ^ 0x5032));
    let mut valid_probe_pairs = 0;
    for generation in 0..5 {
        let eps = optimizer.perturbations(PARAMS);
        let mut pairs = Vec::new();
        for perturbation in &eps {
            let plus = optimizer.candidate(&moved, perturbation, 1.0);
            let minus = optimizer.candidate(&moved, perturbation, -1.0);
            let seed = PILOT_SEED + generation;
            let p = ev
                .run("optimizer_probe_plus", &plus, seed, EreArm::A3)?
                .and_then(|o| o.structural_proposal_score);
            let m = ev
                .run("optimizer_probe_minus", &minus, seed, EreArm::A3)?
                .and_then(|o| o.structural_proposal_score);
            // Whole invalid antithetic pairs are withheld, never ranked as zeros.
            if p.is_some() && m.is_some() {
                valid_probe_pairs += 1;
                pairs.push(PairResult {
                    plus: p.map(|v| v as f32),
                    minus: m.map(|v| v as f32),
                });
            } else {
                pairs.push(PairResult {
                    plus: None,
                    minus: None,
                });
            }
        }
        if pairs.iter().any(|p| p.plus.is_some()) {
            optimizer.update(&mut moved, &eps, &pairs);
        }
        println!(
            "E2 pilot probe generation {}/5; {} attempted evaluations",
            generation + 1,
            ev.log.len()
        );
    }
    let moved_tv = tv(&before_genotypes, &distribution(&moved, true)?);
    let l2 = initial
        .iter()
        .zip(&moved)
        .map(|(a, b)| ((*a - *b) as f64).powi(2))
        .sum::<f64>()
        .sqrt();
    let discrimination = a.zip(b).map(|(a, b)| (a - b).abs());
    let p3 = discrimination
        .zip(stats)
        .is_some_and(|(d, (_, sd))| d > 2.0 * sd && d > 1e-12);
    let margin = stats.map(|(mean, _)| mean.min(1.0 - mean));
    let checks = json!([
        {"name":"P1_CLASS_EXPRESSIBLE","measured":distinct,"threshold":1,"passed":distinct>=1},
        {"name":"P2_OPTIMIZER_TRAVERSAL","measured":moved_tv,"threshold":0.05,
            "passed":moved_tv>=0.05 && valid_probe_pairs==20,"parameter_l2":l2,"valid_pairs":valid_probe_pairs},
        {"name":"P3_OBJECTIVE_DISCRIMINATION","measured":discrimination,"threshold":stats.map(|(_,sd)|2.0*sd),"passed":p3},
        {"name":"P4_NOT_SATURATED","measured":margin,"threshold":0.02,"passed":margin.is_some_and(|m|m>0.02)}]);
    save(out, "prechecks.json", &checks)?;
    save(out, "probe_theta.json", &moved)?;
    let stateability = checks
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["passed"] == true);
    let controls_valid = controls.iter().all(|c| c["observation"]["valid"] == true);
    let unprotected_intact = controls.iter().filter(|c| c["arm"] == "A3").all(|c| {
        c["observation"]["anchor_survival_fraction"] == 1.0
            && c["observation"]["cleanup_edges_lost"] == 0
            && c["observation"]["quarantine_decisions"] == 0
    });
    let ready = stateability && controls_valid && unprotected_intact;
    let status = json!({"status":if ready {"pilot_checks_passed_full_training_not_launched"} else {"full_training_blocked_by_pilot"},
        "stateability_pass":stateability,"controls_valid":controls_valid,"unprotected_intact":unprotected_intact,
        "pilot_evaluations_attempted":ev.log.len(),"maximum_steps_requested":ev.log.len() as u64 * PILOT_STEPS,
        "pilot_probe_generations_completed":5,"pilot_probe_evaluations":40,"full_optimizer_generations_completed":0,
        "full_optimizer_evaluation_steps":0,"held_out_evaluation_steps":0,"ready_for_full_campaign_decision":ready});
    save(out, "status.json", &status)?;
    std::fs::write(out.join("report.md"),format!("# E2-aware grammar pilot\n\nStateability: {stateability}. Controls complete/cap-free: {controls_valid}. Unprotected anchors/cleanup/quarantine intact: {unprotected_intact}.\n\n{} attempted evaluations, at most {} requested simulation steps. Five small optimizer probe generations were run; **no full training campaign or held-out evaluations were launched**. Seeded proposal reuse is not endogenous discovery. See controls.json, evaluations.json, prechecks.json and status.json.\n\n```json\n{}\n```\n",ev.log.len(),ev.log.len() as u64*PILOT_STEPS,serde_json::to_string_pretty(&checks).unwrap())).map_err(|e|e.to_string())?;
    println!("E2 pilot finished: stateability={stateability}, controls_valid={controls_valid}, unprotected_intact={unprotected_intact}");
    Ok(())
}
