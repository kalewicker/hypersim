//! Fixed witness sealing/repair diagnostic. No optimizer or gate admission.
use super::{e2, maintenance, persistence};
use hg_schema::{ere::EreStepObservation, EreMetricsRow};
use hg_sim_core::{
    ere::{EreArm, MetaInstructionSeed, MetaOperation, MetaVocabularyConfig},
    SimConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};
pub const PROTOCOL: &str = hg_sim_core::ere::REPAIR_PROTOCOL;
pub const SEED: u64 = 0xE5_523A_0001;
pub const STEPS: u64 = 100;
pub const PREREG_SHA: &str = "bde3d47d95ce408153519148bbee44f8a5c86fcaea1931ee0994258e9339fe51";
pub const CONDITIONS: [(&str, EreArm, bool, bool); 6] = [
    ("A3_neither", EreArm::A3, false, false),
    ("A3_seal", EreArm::A3, true, false),
    ("A3_repair", EreArm::A3, false, true),
    ("A3_both", EreArm::A3, true, true),
    ("A2_neither", EreArm::A2, false, false),
    ("A2_both", EreArm::A2, true, true),
];
fn save<T: Serialize>(out: &Path, name: &str, v: &T) -> Result<(), String> {
    std::fs::write(
        out.join(name),
        serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn configure(
    base: &SimConfig,
    seed: u64,
    index: usize,
) -> Result<(SimConfig, e2::Emission), String> {
    let (name, arm, seal, repair) = *CONDITIONS.get(index).ok_or("unknown condition")?;
    let (mut cfg, emission) = persistence::configure(base, seed, arm, true)?;
    cfg.experiment_name = format!("ERE_witness_repair_{name}");
    cfg.ere.as_mut().unwrap().vocabulary = Some(MetaVocabularyConfig {
        protocol: PROTOCOL.into(),
        instructions: vec![
            MetaInstructionSeed {
                operation: MetaOperation::WitnessActivate,
                fuel: 1.0,
            },
            MetaInstructionSeed {
                operation: MetaOperation::Seal,
                fuel: if seal { 16.0 } else { 0.0 },
            },
            MetaInstructionSeed {
                operation: MetaOperation::Seal,
                fuel: if seal { 16.0 } else { 0.0 },
            },
            MetaInstructionSeed {
                operation: MetaOperation::Repair,
                fuel: if repair { 32.0 } else { 0.0 },
            },
            MetaInstructionSeed {
                operation: MetaOperation::Repair,
                fuel: if repair { 32.0 } else { 0.0 },
            },
        ],
    });
    Ok((cfg, emission))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub evaluation: usize,
    pub seed: u64,
    pub condition: String,
    pub effective_config: SimConfig,
    pub emission: e2::Emission,
    pub requested_steps: u64,
    pub completed_steps: Option<u64>,
    pub error: Option<String>,
    pub observation: Option<maintenance::Observation>,
    pub ere_windows: Vec<EreMetricsRow>,
    pub step_observations: Vec<EreStepObservation>,
    pub accounting_ok: bool,
}
pub fn verify_accounting(r: &Receipt) -> bool {
    let Some(config) = r
        .effective_config
        .ere
        .as_ref()
        .and_then(|e| e.vocabulary.as_ref())
    else {
        return false;
    };
    if config.protocol != PROTOCOL {
        return false;
    }
    let Some(count) = r.completed_steps else {
        return false;
    };
    let traces: Vec<_> = r
        .ere_windows
        .iter()
        .flat_map(|w| w.encoded_meta_operation_traces.as_deref().unwrap_or(&[]))
        .collect();
    let paid: usize = r
        .ere_windows
        .iter()
        .filter_map(|w| w.encoded_meta_rewrite_events)
        .sum();
    if paid != traces.len()
        || config.instructions.len() != 5
        || r.step_observations.len() as u64 != count
    {
        return false;
    }
    let mut ledger: BTreeMap<String, (String, f64)> = BTreeMap::new();
    for (i, w) in r.ere_windows.iter().enumerate() {
        let Some(states) = w.encoded_meta_instruction_states.as_ref() else {
            return false;
        };
        if states.len() != 5 {
            return false;
        }
        for (state, initial) in states.iter().zip(&config.instructions) {
            ledger
                .entry(state.instruction.clone())
                .or_insert((initial.operation.name().into(), initial.fuel));
            if !state.valid
                || state.checks != w.step_end
                || state.valid_steps != state.checks
                || state.operation.as_deref() != Some(initial.operation.name())
            {
                return false;
            }
        }
        let Some(events) = w.encoded_meta_operation_traces.as_ref() else {
            return false;
        };
        for t in events {
            let Some((op, fuel)) = ledger.get_mut(&t.instruction) else {
                return false;
            };
            if *op != t.operation
                || t.step != w.step_end
                || t.fuel_before != Some(*fuel)
                || *fuel < 1.0
                || t.fuel_after != Some(*fuel - 1.0)
            {
                return false;
            }
            *fuel -= 1.0;
        }
        for state in states {
            if state.fuel != ledger.get(&state.instruction).map(|(_, fuel)| *fuel) {
                return false;
            }
        }
        if w.seeded_editor_rewrite_events != Some(0)
            || w.encoded_meta_rewrite_events != Some(events.len())
            || w.meta_rewrite_events != w.ordinary_meta_rewrite_events
            || w.match_ceiling_binding
            || w.rule_region_ceiling_binding
            || w.step_start != i as u64
            || w.step_end != i as u64 + 1
        {
            return false;
        }
    }
    r.step_observations.iter().enumerate().all(|(i, s)| {
        s.step == i as u64 + 1
            && s.families.len() == 4
            && s.rng_before_compile == s.rng_after_compile
            && !s.match_ceiling_binding
            && s.match_work <= 50000
            && s.match_results <= 50000
            && s.world_rule_table.iter().all(Option::is_some)
            && (i == 0
                || (r.step_observations[i - 1].rng_after_step == s.rng_before_compile
                    && r.step_observations[i - 1].world_hash_after == s.world_hash_before))
    })
}
pub fn evaluate(cfg: SimConfig, emission: e2::Emission, index: usize, condition: &str) -> Receipt {
    let (result, rows, steps) = hg_sim_core::runner::run_ere_instrumented_diagnostic(&cfg);
    let (completed, hash, error) = match result {
        Ok(a) => (
            Some(a.final_step),
            Some(format!("{:016x}", a.canonical_hash)),
            None,
        ),
        Err(e) => {
            let done = match &e {
                hg_sim_core::runner::SimError::EreHalt {
                    completed_steps, ..
                } => Some(*completed_steps),
                _ => rows.last().map(|w| w.step_end),
            };
            (done, None, Some(e.to_string()))
        }
    };
    let observation = completed.zip(hash).and_then(|(done, hash)| {
        maintenance::summarize(&rows, done, hash, cfg.engine.max_steps, 16).ok()
    });
    let mut r = Receipt {
        evaluation: index,
        seed: cfg.seed,
        condition: condition.into(),
        requested_steps: cfg.engine.max_steps,
        effective_config: cfg,
        emission,
        completed_steps: completed,
        error,
        observation,
        ere_windows: rows,
        step_observations: steps,
        accounting_ok: false,
    };
    r.accounting_ok = verify_accounting(&r);
    if !r.accounting_ok || r.observation.is_none() {
        r.error
            .get_or_insert("incomplete or inconsistent witness repair observations".into());
    }
    r
}
fn complete(r: &Receipt) -> bool {
    r.error.is_none()
        && r.accounting_ok
        && r.completed_steps == Some(STEPS)
        && r.step_observations.len() == STEPS as usize
        && r.observation.is_some()
}
pub fn contrasts(group: &[Receipt]) -> Result<serde_json::Value, String> {
    if group.len() != 6
        || group
            .iter()
            .enumerate()
            .any(|(i, r)| r.condition != CONDITIONS[i].0 || r.seed != group[0].seed)
    {
        return Err("all six ordered matched conditions required".into());
    }
    let o = |i: usize| group[i].observation.as_ref();
    let recovery = o(3).is_some_and(|a| {
        a.families
            .iter()
            .map(|f| f.recovery_transitions)
            .sum::<u64>()
            > 0
    });
    let repair = group[3]
        .ere_windows
        .iter()
        .flat_map(|w| w.encoded_meta_operation_traces.as_deref().unwrap_or(&[]))
        .any(|t| t.operation == "Repair");
    let checks = BTreeMap::from([
        ("all_six_complete", group.iter().all(complete)),
        (
            "combined_all_final_valid",
            o(3).is_some_and(|a| a.final_valid_fraction == 1.0),
        ),
        ("combined_measured_recovery", recovery),
        ("combined_paid_repair", repair),
        (
            "combined_greater_valid_time",
            o(3).zip(o(0))
                .is_some_and(|(a, b)| a.time_valid_fraction > b.time_valid_fraction),
        ),
        (
            "combined_proposal_world_fires",
            o(3).is_some_and(|a| a.structural_world_fires > 0),
        ),
        (
            "protected_references_intact",
            [4, 5].iter().all(|&i| {
                o(i).is_some_and(|a| a.time_valid_fraction == 1.0 && a.final_valid_fraction == 1.0)
            }),
        ),
    ]);
    Ok(
        json!({"seed":group[0].seed,"checks":checks,"combined_control_pass":checks.values().all(|&b|b),
        "conditions":group.iter().map(|r|json!({"condition":r.condition,"observation":r.observation,"accounting_ok":r.accounting_ok,"error":r.error})).collect::<Vec<_>>() }),
    )
}
pub fn run_assay(base: &SimConfig, out: &Path) -> Result<(), String> {
    if out.exists() {
        return Err("refusing to overwrite witness repair receipt directory".into());
    }
    let mut planned = Vec::new();
    for seed in SEED..SEED + 4 {
        for i in 0..6 {
            let (cfg, emission) = configure(base, seed, i)?;
            planned.push((cfg, emission, CONDITIONS[i].0));
        }
    }
    std::fs::create_dir_all(out.join("rollouts")).map_err(|e| e.to_string())?;
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../../docs/ere_witness_repair_v2.md"),
    )
    .map_err(|e| e.to_string())?;
    std::fs::copy(
        std::env::current_exe().map_err(|e| e.to_string())?,
        out.join("executed_binary"),
    )
    .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(out.join("source_snapshots")).map_err(|e| e.to_string())?;
    for (name, body) in [
        ("driver.rs", include_str!("repair.rs")),
        ("repair.rs", include_str!("../../../hg-meta/src/repair.rs")),
        (
            "vocabulary.rs",
            include_str!("../../../hg-meta/src/vocabulary.rs"),
        ),
        ("source.rs", include_str!("../../../hg-meta/src/source.rs")),
        (
            "runtime.rs",
            include_str!("../../../hg-sim-core/src/ere.rs"),
        ),
        (
            "runner.rs",
            include_str!("../../../hg-sim-core/src/runner.rs"),
        ),
        (
            "program.rs",
            include_str!("../../../hg-meta/src/program.rs"),
        ),
    ] {
        std::fs::write(out.join("source_snapshots").join(name), body).map_err(|e| e.to_string())?;
    }
    save(
        out,
        "manifest.json",
        &json!({"protocol":PROTOCOL,"seed":SEED,"conditions":CONDITIONS.iter().map(|c|c.0).collect::<Vec<_>>(),
        "attempts":24,"steps_per_attempt":STEPS,"requested_step_ceiling":2400,"preregistration_sha256":PREREG_SHA,
        "planned_effective_configs":planned.iter().map(|p|&p.0).collect::<Vec<_>>(),"optimizer_evaluations":0,"held_out_evaluations":0,"old_gate_admission_reused":false}),
    )?;
    let mut receipts = Vec::new();
    let mut log = Vec::new();
    for (index, (cfg, emission, name)) in planned.into_iter().enumerate() {
        let r = evaluate(cfg, emission, index, name);
        let path = format!("rollouts/evaluation_{index:03}.json");
        save(out, &path, &r)?;
        log.push(json!({"evaluation":index,"seed":r.seed,"condition":name,"completed_steps":r.completed_steps,"error":r.error,"accounting_ok":r.accounting_ok,"receipt":path}));
        save(out, "evaluations.json", &log)?;
        eprintln!(
            "witness repair {}/24 {}: {:?} steps {}",
            index + 1,
            name,
            r.completed_steps,
            if r.error.is_none() {
                "complete"
            } else {
                "failed"
            }
        );
        receipts.push(r);
    }
    let decisions = receipts
        .chunks(6)
        .map(contrasts)
        .collect::<Result<Vec<_>, _>>()?;
    let passed = decisions.iter().all(|v| v["combined_control_pass"] == true);
    save(out, "contrasts.json", &decisions)?;
    save(
        out,
        "status.json",
        &json!({"protocol":PROTOCOL,"attempts":24,"complete":receipts.iter().filter(|r|complete(r)).count(),
        "errors":receipts.iter().filter(|r|!complete(r)).count(),"actual_steps":receipts.iter().filter_map(|r|r.completed_steps).sum::<u64>(),
        "requested_steps":2400,"combined_control_pass":passed,"status":if passed{"bounded_combined_control_passed"}else{"incomplete_or_failed"},
        "optimizer_evaluation_steps":0,"held_out_evaluation_steps":0,"full_training_admitted":false}),
    )
}
