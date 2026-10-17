//! Fixed diagnostic only. New vocabulary semantics are not admitted to training.
use super::{e2, maintenance};
use hg_sim_core::{
    ere::{EreArm, MetaInstructionSeed, MetaOperation, MetaVocabularyConfig, VOCABULARY_PROTOCOL},
    runner::SimError,
    SimConfig,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;

pub const SEED: u64 = 0xE4_5239_0001;
pub const STEPS: u64 = 1000;
pub const ATTEMPTS: usize = 12;
fn save<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    std::fs::write(
        dir.join(name),
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn configure(
    base: &SimConfig,
    seed: u64,
    arm: EreArm,
) -> Result<(SimConfig, e2::Emission), String> {
    let emission = e2::emit(&vec![0.0; e2::PARAMS], seed)?;
    let mut cfg = e2::configure(base, &emission, seed, STEPS, arm)?;
    cfg.experiment_name = "ERE_meta_vocabulary_v1".into();
    cfg.initial.condition_type = hg_sim_core::InitialConditionType::PureCrystal;
    cfg.initial.width = 4;
    cfg.initial.height = 4;
    cfg.initial.arity = 2;
    cfg.initial.periodic = true;
    cfg.engine.step_mode = hg_rules::StepMode::MaxParallel;
    cfg.engine.max_pairings_per_step = 16;
    cfg.engine.thread_count = 1;
    cfg.engine.metrics_window_interval = 100;
    cfg.evaluator.enabled = false;
    cfg.causal.enabled = false;
    let e = cfg.ere.as_mut().unwrap();
    e.epoch = hg_rules::EpochPolicy::Fixed(10.try_into().unwrap());
    e.rule_region_ceiling = 500;
    e.match_ceiling = 50_000;
    e.meta_seed_energy = 0.0;
    e.meta_rules_toml = None;
    e.measure_ei = false;
    e.measure_family_maintenance = true;
    e.vocabulary = Some(MetaVocabularyConfig {
        protocol: VOCABULARY_PROTOCOL.into(),
        instructions: [
            MetaOperation::Activate,
            MetaOperation::Deactivate,
            MetaOperation::Transpose,
            MetaOperation::Transpose,
        ]
        .into_iter()
        .map(|operation| MetaInstructionSeed {
            operation,
            fuel: 32.0,
        })
        .collect(),
    });
    Ok((cfg, emission))
}

pub fn run_assay(base: &SimConfig, out: &Path) -> Result<(), String> {
    if out.exists() {
        return Err("refusing to overwrite vocabulary assay directory".into());
    }
    let mut planned = Vec::new();
    for seed in SEED..SEED + 4 {
        for arm in [EreArm::A1, EreArm::A2, EreArm::A3] {
            let (cfg, emission) = configure(base, seed, arm)?;
            planned.push((cfg, emission));
        }
    }
    std::fs::create_dir_all(out.join("rollouts")).map_err(|e| e.to_string())?;
    save(
        out,
        "manifest.json",
        &json!({"protocol":VOCABULARY_PROTOCOL,"seed":SEED,
        "attempts":ATTEMPTS,"steps_per_attempt":STEPS,"requested_step_ceiling":12000,
        "optimizer_evaluations":0,"held_out_evaluations":0,"old_gate_admission_reused":false,
        "preregistration":"docs/ere_meta_vocabulary_v1.md",
        "preregistration_sha256":"569d3b170b6191d5e345c73c6ebbb56dc7e4f8bf1d93396cbee89a47f2e3bbba",
        "planned_effective_configs":planned.iter().map(|(cfg,_)|cfg).collect::<Vec<_>>() }),
    )?;
    let mut evaluations = Vec::<Value>::new();
    for (index, (cfg, emission)) in planned.into_iter().enumerate() {
        let (result, rows) = hg_sim_core::runner::run_ere_diagnostic(&cfg);
        let (completed, hash, mut error) = match result {
            Ok(run) => (
                Some(run.final_step),
                Some(format!("{:016x}", run.canonical_hash)),
                None,
            ),
            Err(e) => {
                let steps = if let SimError::EreHalt {
                    completed_steps, ..
                } = &e
                {
                    Some(*completed_steps)
                } else {
                    None
                };
                (steps, None, Some(e.to_string()))
            }
        };
        let observation = match (completed, hash) {
            (Some(steps), Some(hash)) => match maintenance::summarize(
                &rows,
                steps,
                hash,
                STEPS,
                cfg.engine.max_pairings_per_step,
            ) {
                Ok(v) => Some(v),
                Err(e) => {
                    error.get_or_insert(e);
                    None
                }
            },
            _ => None,
        };
        let encoded: usize = rows
            .iter()
            .filter_map(|r| r.encoded_meta_rewrite_events)
            .sum();
        let ordinary: usize = rows
            .iter()
            .filter_map(|r| r.ordinary_meta_rewrite_events)
            .sum();
        let traces: Vec<_> = rows
            .iter()
            .flat_map(|r| r.encoded_meta_operation_traces.as_deref().unwrap_or(&[]))
            .collect();
        let debit_ok = traces.len() == encoded
            && traces.iter().all(|t| match (t.fuel_before, t.fuel_after) {
                (Some(a), Some(b)) => a.is_finite() && b.is_finite() && a - b == 1.0 && b >= 0.0,
                _ => false,
            });
        let instructions = rows
            .last()
            .and_then(|r| r.encoded_meta_instruction_states.clone());
        let fuel_ok = instructions.as_ref().is_some_and(|states| {
            states.len() == 4
                && states.iter().all(|s| {
                    s.valid
                        && s.checks == completed.unwrap_or(0)
                        && s.valid_steps == s.checks
                        && s.fuel.is_some_and(|f| (0.0..=32.0).contains(&f))
                })
                && states.iter().filter_map(|s| s.fuel).sum::<f64>() + encoded as f64 == 128.0
        });
        let accounting_ok = ordinary >= encoded
            && debit_ok
            && fuel_ok
            && rows.iter().all(|r| {
                r.encoded_meta_vocabulary_protocol.as_deref() == Some(VOCABULARY_PROTOCOL)
                    && r.seeded_editor_rewrite_events == Some(0)
                    && r.meta_rewrite_events == r.ordinary_meta_rewrite_events
            });
        if !accounting_ok {
            error.get_or_insert("vocabulary accounting/energy measurement failed".into());
        }
        let receipt = format!("rollouts/evaluation_{index:03}.json");
        save(
            out,
            &receipt,
            &json!({"evaluation":index,"effective_config":cfg,"emission":emission,
            "completed_steps":completed,"observation":observation,"error":error,"ere_windows":rows,
            "partial_error_history_retained":true,"encoded_events":encoded,"world_rule_meta_events":ordinary.checked_sub(encoded),
            "instruction_states":instructions,"accounting_ok":accounting_ok}),
        )?;
        evaluations.push(json!({"evaluation":index,"seed":cfg.seed,"arm":cfg.ere.as_ref().unwrap().arm,
            "requested_steps":STEPS,"completed_steps":completed,"observation":observation,"error":error,
            "encoded_events":encoded,"world_rule_meta_events":ordinary.checked_sub(encoded),
            "accounting_ok":accounting_ok,"receipt":receipt}));
        save(out, "evaluations.json", &evaluations)?;
        eprintln!(
            "vocabulary diagnostic {}/{}: {} steps, {} paid edits, {}",
            index + 1,
            ATTEMPTS,
            completed.unwrap_or(0),
            encoded,
            if error.is_none() {
                "complete"
            } else {
                "failed"
            }
        );
    }
    let complete = evaluations
        .iter()
        .filter(|v| v["completed_steps"] == STEPS && v["error"].is_null())
        .count();
    save(
        out,
        "status.json",
        &json!({"protocol":VOCABULARY_PROTOCOL,
        "status":if complete==ATTEMPTS{"diagnostic_complete"}else{"diagnostic_incomplete_or_failed"},
        "attempts":evaluations.len(),"complete":complete,"errors":ATTEMPTS-complete,
        "actual_steps":evaluations.iter().filter_map(|v|v["completed_steps"].as_u64()).sum::<u64>(),
        "requested_steps":12000,"optimizer_evaluation_steps":0,"held_out_evaluation_steps":0,
        "full_training_admitted":false,"next_required":"new vocabulary full G-R and stateability admission"}),
    )?;
    Ok(())
}
