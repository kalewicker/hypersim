//! New preregistered joint-objective pilot; unchanged grammar and physics.
use super::{e2, OpenAiEs, PairResult};
use hg_schema::{ere::EreFamilyMaintenance, EreMetricsRow};
use hg_sim_core::{
    ere::{self, EreArm},
    runner::SimError,
    SimConfig,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub const PROTOCOL: &str = "E2-maintenance-reuse-pilot-v1";
pub const SEED: u64 = 0xE3_5239_0001;
pub const STEPS: u64 = 1000;
pub const MAX_EVALUATIONS: usize = 58;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub final_step: u64,
    pub final_hash: String,
    pub reuse_score: f64,
    pub time_valid_fraction: f64,
    pub final_valid_fraction: f64,
    pub joint_score: Option<f64>,
    pub families: Vec<EreFamilyMaintenance>,
    pub structural_reuse_by_class: BTreeMap<u64, usize>,
    pub structural_world_fires: usize,
    pub seeded_editor_events: usize,
    pub ordinary_meta_events: usize,
    pub meta_rewrite_events: usize,
    pub quarantine_decisions: usize,
    pub cleanup_edges_lost: usize,
    pub valid: bool,
}

/// Pure receipt reconstruction; no simulation and no inferred missing zeros.
pub fn summarize(
    rows: &[EreMetricsRow],
    step: u64,
    hash: String,
    requested: u64,
    max_pairings: usize,
) -> Result<Observation, String> {
    let last = rows.last().ok_or("missing ERE history")?;
    let families = last
        .seeded_family_maintenance
        .clone()
        .ok_or("missing family maintenance")?;
    if families.len() != e2::GROUPS
        || step == 0
        || max_pairings == 0
        || families.iter().enumerate().any(|(i, f)| {
            f.family_index != i
                || f.checks != step
                || f.valid_steps > f.checks
                || f.final_valid != f.final_failure.is_none()
        })
    {
        return Err("incomplete/inconsistent family observations".into());
    }
    let map = last
        .structural_proposal_reuse_count_by_class
        .clone()
        .ok_or("missing structural reuse map")?;
    let possible = requested as f64 * max_pairings as f64;
    let r = (map
        .values()
        .map(|n| (n.saturating_sub(1) as f64).ln_1p())
        .sum::<f64>()
        / (e2::GROUPS as f64 * (possible / e2::GROUPS as f64).ln_1p()))
    .clamp(0.0, 1.0);
    let t = families.iter().map(|f| f.valid_steps).sum::<u64>() as f64
        / (families.len() as f64 * step as f64);
    let f = families.iter().filter(|f| f.final_valid).count() as f64 / families.len() as f64;
    let sum = |get: fn(&EreMetricsRow) -> Option<usize>| -> Result<usize, String> {
        rows.iter()
            .map(|r| get(r).ok_or("missing event/counter measurement".into()))
            .sum()
    };
    let editors = sum(|r| r.seeded_editor_rewrite_events)?;
    let ordinary = sum(|r| r.ordinary_meta_rewrite_events)?;
    let meta = sum(|r| r.meta_rewrite_events)?;
    if editors + ordinary != meta {
        return Err("editor/ordinary meta event accounting mismatch".into());
    }
    let valid = step == requested
        && !rows
            .iter()
            .any(|r| r.match_ceiling_binding || r.rule_region_ceiling_binding)
        && [r, t, f].iter().all(|x| x.is_finite());
    Ok(Observation {
        final_step: step,
        final_hash: hash,
        reuse_score: r,
        time_valid_fraction: t,
        final_valid_fraction: f,
        joint_score: valid.then_some(r * t * f),
        structural_world_fires: map.values().sum(),
        structural_reuse_by_class: map,
        families,
        seeded_editor_events: editors,
        ordinary_meta_events: ordinary,
        meta_rewrite_events: meta,
        quarantine_decisions: sum(|r| r.rules_quarantined)?,
        cleanup_edges_lost: sum(|r| r.meta_edges_lost_to_cleanup)?,
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
    for seed in SEED..SEED + 64 {
        let emission = e2::emit(theta, seed)?;
        *counts
            .entry(ere::seed_family_signature(&emission.families, genotype)?)
            .or_default() += 1;
    }
    Ok(counts)
}
fn tv(a: &BTreeMap<String, usize>, b: &BTreeMap<String, usize>) -> f64 {
    let keys: BTreeSet<_> = a.keys().chain(b.keys()).collect();
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
            return Err("maintenance pilot hard budget exhausted".into());
        }
        let index = self.log.len();
        let emission = e2::emit(theta, seed)?;
        let mut cfg = e2::configure(self.base, &emission, seed, STEPS, arm)?;
        cfg.ere.as_mut().unwrap().measure_family_maintenance = true;
        let (observation, error, completed, rows) = match hg_sim_core::runner::run_ere_rollout(&cfg)
        {
            Ok(run) => {
                let rows: Vec<_> = run
                    .history
                    .into_iter()
                    .filter_map(|r| r.ere.map(|r| *r))
                    .collect();
                let observation = summarize(
                    &rows,
                    run.final_step,
                    format!("{:016x}", run.canonical_hash),
                    STEPS,
                    cfg.engine.max_pairings_per_step,
                );
                (
                    observation.as_ref().ok().cloned(),
                    observation.err(),
                    Some(run.final_step),
                    rows,
                )
            }
            Err(error) => {
                let completed = match &error {
                    SimError::EreHalt {
                        completed_steps, ..
                    } => Some(*completed_steps),
                    _ => None,
                };
                (None, Some(error.to_string()), completed, Vec::new())
            }
        };
        let receipt = format!("rollouts/evaluation_{index:03}.json");
        save(
            self.out,
            &receipt,
            &json!({"evaluation":index,"effective_config":cfg,"emission":emission,
            "completed_steps":completed,"observation":observation,"error":error,"ere_windows":rows,
            "partial_error_history_retained":false}),
        )?;
        self.log.push(
            json!({"evaluation":index,"label":label,"seed":seed,"arm":arm,
            "requested_steps":STEPS,"theta":theta,"observation":observation,"error":error,
            "completed_steps":completed,"receipt":receipt}),
        );
        save(self.out, "evaluations.json", &self.log)?;
        Ok(observation)
    }
}

pub fn run_pilot(
    base: &SimConfig,
    out: &Path,
    classification: &Path,
    gates: &Path,
) -> Result<(), String> {
    if out.exists() {
        return Err("refusing to overwrite maintenance pilot directory".into());
    }
    let mut cfg = base.clone();
    if cfg.engine.step_mode != hg_rules::StepMode::MaxParallel
        || cfg.initial.width != 4
        || cfg.initial.height != 4
        || !cfg.initial.periodic
        || cfg.initial.arity != 2
        || serde_json::to_value(&cfg.initial.condition_type).map_err(|e| e.to_string())?
            != "PureCrystal"
    {
        return Err("maintenance pilot requires the frozen 4x4 periodic binary PureCrystal MaxParallel profile".into());
    }
    let e = cfg.ere.as_mut().ok_or("maintenance pilot requires ERE")?;
    if e.vocabulary.is_some() {return Err("maintenance-v1 cannot reuse vocabulary semantics".into());}
    if e.match_ceiling != 50_000 || e.rule_region_ceiling != 500 {
        return Err("maintenance caps must remain 50k/500".into());
    }
    e.epoch = hg_rules::EpochPolicy::Fixed(10.try_into().unwrap());
    e.meta_seed_energy = 32.0;
    e.measure_ei = false;
    e.measure_family_maintenance = true;
    cfg.engine.metrics_window_interval = 100;
    cfg.engine.max_pairings_per_step = 16;
    cfg.engine.thread_count = 1;
    cfg.engine.max_steps = STEPS;
    cfg.evaluator.enabled = false;
    cfg.causal.enabled = false;
    std::fs::create_dir_all(out.join("rollouts")).map_err(|e| e.to_string())?;
    save(
        out,
        "manifest.json",
        &json!({"protocol":PROTOCOL,"seed":SEED,"parameters":e2::PARAMS,"families":e2::GROUPS,
        "max_evaluations":MAX_EVALUATIONS,"steps_per_eval":STEPS,"maximum_simulation_steps":58000,
        "base_config":cfg,"objective":"structural_reuse_x_time_valid_fraction_x_final_valid_fraction",
        "precheck_generations":5,"precheck_population":8,"sigma":1.0,"alpha":0.05,
        "classification_receipts":classification,"downstream_receipts":gates,"full_campaign":"not_launched"}),
    )?;
    let admission = match ere::admission::require_pooled_400k50k_gates(classification, gates) {
        Ok(v) => v,
        Err(error) => {
            save(
                out,
                "status.json",
                &json!({"status":"blocked_by_admission","error":error,
            "evaluations_attempted":0,"full_optimizer_evaluation_steps":0}),
            )?;
            return Err(error);
        }
    };
    save(out, "admission.json", &admission)?;
    let initial = vec![0.0; e2::PARAMS];
    let active = e2::forced(0, 1, true);
    let dormant = e2::forced(0, 0, false);
    let mut ev = Evaluations {
        base: &cfg,
        out,
        log: Vec::new(),
    };
    let mut controls = Vec::new();
    for seed in SEED..SEED + 4 {
        for arm in [EreArm::A1, EreArm::A2, EreArm::A3] {
            let o = ev.run("matched_control", &initial, seed, arm)?;
            controls.push(json!({"seed":seed,"arm":arm,"observation":o}));
        }
    }
    save(out, "controls.json", &controls)?;
    let before = distribution(&initial, true)?;
    let initial_exec = distribution(&initial, false)?;
    let forced_exec = distribution(&active, false)?;
    let distinct = forced_exec
        .keys()
        .filter(|k| !initial_exec.contains_key(*k))
        .count();
    let mut baseline = Vec::new();
    for seed in SEED..SEED + 4 {
        baseline.push(
            ev.run("stateability_baseline", &initial, seed, EreArm::A3)?
                .and_then(|o| o.joint_score),
        );
    }
    let a = ev
        .run("forced_structural_active", &active, SEED, EreArm::A3)?
        .and_then(|o| o.joint_score);
    let b = ev
        .run("forced_priority_dormant", &dormant, SEED, EreArm::A3)?
        .and_then(|o| o.joint_score);
    let scores: Option<Vec<_>> = baseline.into_iter().collect();
    let stats = scores.map(|s| {
        let mean = s.iter().sum::<f64>() / 4.0;
        let sd = (s.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / 4.0).sqrt();
        (mean, sd)
    });
    let mut moved = initial.clone();
    let mut optimizer = OpenAiEs::new(1.0, 0.05, 4, ChaCha8Rng::seed_from_u64(SEED ^ 0x5032));
    let mut valid_pairs = 0;
    for generation in 0..5 {
        let eps = optimizer.perturbations(e2::PARAMS);
        let mut pairs = Vec::new();
        for perturbation in &eps {
            let plus = optimizer.candidate(&moved, perturbation, 1.0);
            let minus = optimizer.candidate(&moved, perturbation, -1.0);
            let p = ev
                .run("optimizer_probe_plus", &plus, SEED + generation, EreArm::A3)?
                .and_then(|o| o.joint_score);
            let m = ev
                .run(
                    "optimizer_probe_minus",
                    &minus,
                    SEED + generation,
                    EreArm::A3,
                )?
                .and_then(|o| o.joint_score);
            if p.is_some() && m.is_some() {
                valid_pairs += 1;
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
            "maintenance pilot probe generation {}/5; {} attempts",
            generation + 1,
            ev.log.len()
        );
    }
    let after = distribution(&moved, true)?;
    let moved_tv = tv(&before, &after);
    let parameter_l2 = moved.iter().map(|x|(*x as f64).powi(2)).sum::<f64>().sqrt();
    save(
        out,
        "emission_distributions.json",
        &json!({"initial_genotype":before,"final_genotype":after,
        "initial_execution":initial_exec,"forced_execution":forced_exec}),
    )?;
    save(out, "probe_theta.json", &moved)?;
    let discrimination = a.zip(b).map(|(a, b)| (a - b).abs());
    let margin = stats.map(|(mean, _)| mean.min(1.0 - mean));
    let checks = json!([
        {"name":"P1_CLASS_EXPRESSIBLE","measured":distinct,"threshold":1,"passed":distinct>=1},
        {"name":"P2_OPTIMIZER_TRAVERSAL","measured":moved_tv,"threshold":0.05,"valid_pairs":valid_pairs,"parameter_l2":parameter_l2,"passed":moved_tv>=0.05 && valid_pairs==20},
        {"name":"P3_JOINT_DISCRIMINATION","measured":discrimination,"threshold":stats.map(|(_,sd)|2.0*sd),
            "passed":discrimination.zip(stats).is_some_and(|(d,(_,sd))|d>2.0*sd && d>1e-12)},
        {"name":"P4_NOT_SATURATED","measured":margin,"threshold":0.02,"passed":margin.is_some_and(|m|m>0.02)}]);
    save(out, "prechecks.json", &checks)?;
    let stateability = checks
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["passed"] == true);
    let controls_valid = controls.iter().all(|c| c["observation"]["valid"] == true);
    let intact = |arm: &str| {
        controls.iter().filter(|c| c["arm"] == arm).all(|c| {
            let o = &c["observation"];
            o["valid"] == true
                && o["time_valid_fraction"] == 1.0
                && o["final_valid_fraction"] == 1.0
                && o["quarantine_decisions"] == 0
                && o["cleanup_edges_lost"] == 0
        })
    };
    let reused = controls.iter().filter(|c| c["arm"] == "A3").any(|c| {
        c["observation"]["structural_world_fires"]
            .as_u64()
            .unwrap_or(0)
            > 0
    });
    let protected = intact("A2")
        && controls.iter().filter(|c| c["arm"] == "A2").all(|c| {
            c["observation"]["seeded_editor_events"]
                .as_u64()
                .unwrap_or(0)
                > 0
        });
    let maintained = intact("A3") && reused;
    let ready = stateability && controls_valid && maintained && protected;
    let known_steps: u64 = ev
        .log
        .iter()
        .filter_map(|r| r["completed_steps"].as_u64())
        .sum();
    save(
        out,
        "status.json",
        &json!({"status":if ready {"pilot_passed_full_training_not_launched"} else {"full_training_blocked_by_maintenance_pilot"},
        "stateability_pass":stateability,"controls_valid":controls_valid,"unprotected_maintenance_pass":maintained,
        "protected_instrument_control_pass":protected,"evaluations_attempted":ev.log.len(),
        "requested_step_upper_bound":ev.log.len() as u64*STEPS,"known_completed_steps":known_steps,
        "unmeasured_completed_step_attempts":ev.log.iter().filter(|r|r["completed_steps"].is_null()).count(),
        "probe_generations_completed":5,"probe_evaluations":40,"full_optimizer_generations_completed":0,
        "full_optimizer_evaluation_steps":0,"held_out_evaluation_steps":0,"ready_for_full_campaign_decision":ready}),
    )?;
    save(
        out,
        "report_summary.json",
        &json!({"stateability":checks,"controls":controls}),
    )?;
    println!("maintenance pilot finished: stateability={stateability}, A3_maintenance={maintained}, protected_control={protected}");
    Ok(())
}
