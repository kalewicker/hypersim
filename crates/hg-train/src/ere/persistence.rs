//! Preregistered one-use activation control. No new physics or optimizer.
use super::{e2, maintenance, vocabulary};
use hg_schema::{ere::EreStepObservation, EreMetricsRow};
use hg_sim_core::{
    ere::{
        EreArm, EreSeedFamily, MetaInstructionSeed, MetaOperation, MetaVocabularyConfig,
        VOCABULARY_PROTOCOL,
    },
    SimConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

pub const PROTOCOL: &str = "E2-one-use-activation-persistence-v1";
pub const SEED: u64 = 0xE5_5239_0001;
pub const STEPS: u64 = 100;
pub const ATTEMPTS: usize = 24;
pub const PREREG_SHA: &str = "23d6254fd323d48b596189abc951c00111eb639fa957120292b9091457c5430a";
fn save<T: Serialize>(out: &Path, name: &str, value: &T) -> Result<(), String> {
    std::fs::write(
        out.join(name),
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn configure(
    base: &SimConfig,
    seed: u64,
    arm: EreArm,
    edit: bool,
) -> Result<(SimConfig, e2::Emission), String> {
    let (mut cfg, _) = vocabulary::configure(base, seed, arm)?;
    let template = |name: &str, proposal: bool| {
        format!("[[rules]]\nname = \"{name}\"\npriority = {}\n[rules.lhs]\nnodes = {}\nedges = {}\n[rules.rhs]\nnodes = {}\nedges = {}\n",
        if proposal{1}else{0},if proposal{"[\"a\",\"b\",\"c\"]"}else{"[\"a\",\"b\"]"},
        if proposal{"[[\"a\",\"b\"],[\"b\",\"c\"]]"}else{"[[\"a\",\"b\"]]"},
        if proposal{"[\"a\",\"b\",\"c\"]"}else{"[\"a\",\"b\"]"},
        if proposal{"[[\"a\",\"b\"],[\"a\",\"c\"]]"}else{"[[\"a\",\"b\"]]"})
    };
    let families: Vec<_> = (0..4)
        .map(|i| EreSeedFamily {
            core_toml: template(&format!("persistence_core_{i}"), false),
            proposal_toml: template(&format!("persistence_proposal_{i}"), true),
            active: false,
        })
        .collect();
    hg_sim_core::ere::seed_family_signature(&families, true)?;
    let emission = e2::Emission {
        core_toml: families
            .iter()
            .map(|f| f.core_toml.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        families,
    };
    cfg.rules.inline_toml = Some(emission.core_toml.clone());
    cfg.experiment_name = "ERE_one_use_activation_persistence".into();
    cfg.engine.max_steps = STEPS;
    cfg.engine.metrics_window_interval = 1;
    let e = cfg.ere.as_mut().unwrap();
    e.seed_families = Some(emission.families.clone());
    e.vocabulary = Some(MetaVocabularyConfig {
        protocol: VOCABULARY_PROTOCOL.into(),
        instructions: vec![MetaInstructionSeed {
            operation: MetaOperation::Activate,
            fuel: if edit { 1.0 } else { 0.0 },
        }],
    });
    Ok((cfg, emission))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub evaluation: usize,
    pub seed: u64,
    pub arm: EreArm,
    pub edit: bool,
    pub requested_steps: u64,
    pub effective_config: SimConfig,
    pub emission: e2::Emission,
    pub completed_steps: Option<u64>,
    pub error: Option<String>,
    pub observation: Option<maintenance::Observation>,
    pub ere_windows: Vec<EreMetricsRow>,
    pub step_observations: Vec<EreStepObservation>,
    pub accounting_ok: bool,
}
pub fn evaluate(cfg: SimConfig, emission: e2::Emission, edit: bool, index: usize) -> Receipt {
    let requested = cfg.engine.max_steps;
    let (result, rows, steps) = hg_sim_core::runner::run_ere_instrumented_diagnostic(&cfg);
    let (completed, hash, mut error) = match result {
        Ok(run) => (
            Some(run.final_step),
            Some(format!("{:016x}", run.canonical_hash)),
            None,
        ),
        Err(e) => {
            let completed = match &e {
                hg_sim_core::runner::SimError::EreHalt {
                    completed_steps, ..
                } => Some(*completed_steps),
                _ => rows.last().map(|r| r.step_end),
            };
            (completed, None, Some(e.to_string()))
        }
    };
    let observation = match (completed, hash) {
        (Some(step), Some(hash)) => {
            match maintenance::summarize(&rows, step, hash, requested, 16) {
                Ok(v) => Some(v),
                Err(e) => {
                    error.get_or_insert(e);
                    None
                }
            }
        }
        _ => None,
    };
    let paid: usize = rows
        .iter()
        .filter_map(|r| r.encoded_meta_rewrite_events)
        .sum();
    let traces: Vec<_> = rows
        .iter()
        .flat_map(|r| r.encoded_meta_operation_traces.as_deref().unwrap_or(&[]))
        .collect();
    let states = rows
        .last()
        .and_then(|r| r.encoded_meta_instruction_states.as_ref());
    let initial = if edit { 1.0 } else { 0.0 };
    let accounting_ok = paid == traces.len()
        && paid <= usize::from(edit)
        && states.is_some_and(|s| {
            s.len() == 1
                && s[0].valid
                && s[0].checks == completed.unwrap_or(0)
                && s[0].valid_steps == s[0].checks
                && s[0]
                    .fuel
                    .is_some_and(|fuel| fuel >= 0.0 && fuel + paid as f64 == initial)
        })
        && traces.iter().all(|t| {
            t.operation == "Activate"
                && t.step == 1
                && t.fuel_before == Some(1.0)
                && t.fuel_after == Some(0.0)
        })
        && rows.iter().all(|r| {
            r.seeded_editor_rewrite_events == Some(0)
                && r.meta_rewrite_events == r.ordinary_meta_rewrite_events
                && r.encoded_meta_rewrite_events
                    .zip(r.ordinary_meta_rewrite_events)
                    .is_some_and(|(a, b)| a <= b)
        })
        && steps.len() as u64 == completed.unwrap_or(u64::MAX)
        && steps.iter().enumerate().all(|(i, s)| {
            s.step == i as u64 + 1
                && s.rng_before_compile == s.rng_after_compile
                && s.world_rule_table.iter().all(Option::is_some)
                && s.match_work <= 50_000
                && s.match_results <= 50_000
                && !s.match_ceiling_binding
        });
    if !accounting_ok {
        error.get_or_insert("incomplete or inconsistent persistence measurements".into());
    }
    Receipt {
        evaluation: index,
        seed: cfg.seed,
        arm: cfg.ere.as_ref().unwrap().arm,
        edit,
        requested_steps: requested,
        effective_config: cfg,
        emission,
        completed_steps: completed,
        error,
        observation,
        ere_windows: rows,
        step_observations: steps,
        accounting_ok,
    }
}
fn complete(r: &Receipt) -> bool {
    r.error.is_none()
        && r.accounting_ok
        && r.completed_steps == Some(STEPS)
        && r.step_observations.len() == STEPS as usize
}
fn comparable(s: &EreStepObservation) -> (&str, &str, &Vec<Option<String>>, &str, &str, &str) {
    (
        &s.world_hash_before,
        &s.world_hash_after,
        &s.world_rule_table,
        &s.rng_before_compile,
        &s.rng_after_compile,
        &s.rng_after_step,
    )
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contrast {
    pub seed: u64,
    pub protected_checks: BTreeMap<String, bool>,
    pub protected_persistence_pass: bool,
    pub first_table_difference: Option<u64>,
    pub first_world_difference: Option<u64>,
    pub first_proposal_world_fire: Option<u64>,
    pub a3_edited_family_valid_at_boundary: Option<bool>,
    pub a3_final_valid_families: Option<usize>,
}
/// Reconstruct decisions purely from saved completed steps/windows.
pub fn contrast(group: &[Receipt]) -> Result<Contrast, String> {
    if group.len() != 6 {
        return Err("contrast requires all six predeclared attempts".into());
    }
    let get = |arm, edit| {
        group
            .iter()
            .find(|r| r.arm == arm && r.edit == edit)
            .ok_or("contrast arm missing")
    };
    let a1 = get(EreArm::A1, true)?;
    let a2 = get(EreArm::A2, true)?;
    let n1 = get(EreArm::A1, false)?;
    let n2 = get(EreArm::A2, false)?;
    let a3 = get(EreArm::A3, true)?;
    let table = a1
        .step_observations
        .iter()
        .zip(&a2.step_observations)
        .find(|(a, b)| a.world_rule_table != b.world_rule_table)
        .map(|(_, b)| b.step);
    let world = a1
        .step_observations
        .iter()
        .zip(&a2.step_observations)
        .find(|(a, b)| a.world_hash_after != b.world_hash_after)
        .map(|(_, b)| b.step);
    let fire = a2
        .ere_windows
        .iter()
        .find(|r| {
            r.structural_proposal_reuse_count_by_class
                .as_ref()
                .is_some_and(|m| m.values().sum::<usize>() > 0)
        })
        .map(|r| r.step_end);
    let intact = |r: &Receipt| {
        r.step_observations
            .iter()
            .all(|s| s.families.len() == 4 && s.families.iter().all(|f| f.valid))
            && r.ere_windows
                .last()
                .and_then(|r| r.encoded_meta_instruction_states.as_ref())
                .is_some_and(|s| s.len() == 1 && s[0].valid)
    };
    let persists = |r: &Receipt| {
        let target = r
            .ere_windows
            .iter()
            .flat_map(|r| r.encoded_meta_operation_traces.as_deref().unwrap_or(&[]))
            .next();
        target.is_some_and(|t| {
            r.step_observations
                .first()
                .and_then(|s| s.families.iter().find(|f| f.anchor == t.target_family))
                .is_some_and(|first| {
                    first.valid
                        && first.active == Some(true)
                        && first.execution_bytes.is_some()
                        && r.step_observations.iter().all(|s| {
                            s.families
                                .iter()
                                .find(|f| f.anchor == t.target_family)
                                .is_some_and(|f| {
                                    f.valid
                                        && f.active == Some(true)
                                        && f.execution_bytes == first.execution_bytes
                                })
                        })
                })
        })
    };
    let mut checks = BTreeMap::new();
    checks.insert(
        "complete_protected_and_frozen_controls".into(),
        [a1, a2, n1, n2].iter().all(|r| complete(r)),
    );
    checks.insert(
        "one_paid_edit_no_static_editor".into(),
        [a1, a2].iter().all(|r| {
            r.ere_windows
                .iter()
                .filter_map(|r| r.encoded_meta_rewrite_events)
                .sum::<usize>()
                == 1
        }),
    );
    checks.insert(
        "families_and_instructions_intact".into(),
        [a1, a2, n1, n2].iter().all(|r| intact(r)),
    );
    checks.insert("paid_edit_persists".into(), persists(a1) && persists(a2));
    checks.insert(
        "same_edit_world_table_rng_through_step10".into(),
        a1.step_observations.len() >= 10
            && a2.step_observations.len() >= 10
            && a1
                .step_observations
                .iter()
                .zip(&a2.step_observations)
                .take(10)
                .all(|(a, b)| comparable(a) == comparable(b)),
    );
    checks.insert(
        "table_installed_at_step11".into(),
        table == Some(11)
            && a2
                .step_observations
                .get(10)
                .zip(a2.step_observations.first())
                .is_some_and(|(later, first)| later.world_rule_table != first.world_rule_table),
    );
    checks.insert(
        "world_changes_after_compile".into(),
        world.is_some_and(|s| (11..=STEPS).contains(&s)),
    );
    checks.insert(
        "genuine_proposal_world_firing".into(),
        fire.is_some_and(|s| (11..=STEPS).contains(&s))
            && a1
                .observation
                .as_ref()
                .is_some_and(|o| o.structural_world_fires == 0),
    );
    checks.insert(
        "no_edit_negative_identity".into(),
        n1.step_observations.len() == STEPS as usize
            && n2.step_observations.len() == STEPS as usize
            && n1
                .step_observations
                .iter()
                .zip(&n2.step_observations)
                .all(|(a, b)| comparable(a) == comparable(b))
            && [n1, n2].iter().all(|r| {
                r.step_observations
                    .iter()
                    .all(|s| s.families.iter().all(|f| f.active == Some(false)))
                    && r.observation
                        .as_ref()
                        .is_some_and(|o| o.structural_world_fires == 0)
                    && r.ere_windows
                        .iter()
                        .filter_map(|r| r.encoded_meta_rewrite_events)
                        .sum::<usize>()
                        == 0
            }),
    );
    let target = a3
        .ere_windows
        .iter()
        .flat_map(|r| r.encoded_meta_operation_traces.as_deref().unwrap_or(&[]))
        .next()
        .map(|t| &t.target_family);
    let a3_boundary = target.and_then(|root| {
        a3.step_observations
            .get(9)
            .and_then(|s| s.families.iter().find(|f| &f.anchor == root))
            .map(|f| f.valid && f.active == Some(true))
    });
    Ok(Contrast {
        seed: a1.seed,
        protected_persistence_pass: checks.values().all(|b| *b),
        protected_checks: checks,
        first_table_difference: table,
        first_world_difference: world,
        first_proposal_world_fire: fire,
        a3_edited_family_valid_at_boundary: a3_boundary,
        a3_final_valid_families: a3
            .observation
            .as_ref()
            .map(|o| o.families.iter().filter(|f| f.final_valid).count()),
    })
}

pub fn run_assay(base: &SimConfig, out: &Path) -> Result<(), String> {
    if out.exists() {
        return Err("refusing to overwrite persistence assay directory".into());
    }
    let mut planned = Vec::new();
    for seed in SEED..SEED + 4 {
        for arm in [EreArm::A1, EreArm::A2, EreArm::A3] {
            for edit in [false, true] {
                let (cfg, emission) = configure(base, seed, arm, edit)?;
                planned.push((cfg, emission, edit));
            }
        }
    }
    std::fs::create_dir_all(out.join("rollouts")).map_err(|e| e.to_string())?;
    std::fs::write(
        out.join("preregistration.md"),
        include_str!("../../../../docs/ere_edit_persistence_v1.md"),
    )
    .map_err(|e| e.to_string())?;
    std::fs::copy(
        std::env::current_exe().map_err(|e| e.to_string())?,
        out.join("executed_binary"),
    )
    .map_err(|e| e.to_string())?;
    let sources = [
        ("persistence.rs", include_str!("persistence.rs")),
        (
            "runner.rs",
            include_str!("../../../hg-sim-core/src/runner.rs"),
        ),
        (
            "runtime.rs",
            include_str!("../../../hg-sim-core/src/ere.rs"),
        ),
        (
            "step_schema.rs",
            include_str!("../../../hg-schema/src/ere.rs"),
        ),
    ];
    std::fs::create_dir_all(out.join("source_snapshots")).map_err(|e| e.to_string())?;
    for (name, content) in sources {
        std::fs::write(out.join("source_snapshots").join(name), content)
            .map_err(|e| e.to_string())?;
    }
    save(
        out,
        "manifest.json",
        &json!({"protocol":PROTOCOL,"vocabulary_protocol":VOCABULARY_PROTOCOL,"seed":SEED,
        "attempts":ATTEMPTS,"steps_per_attempt":STEPS,"requested_step_ceiling":2400,"preregistration_sha256":PREREG_SHA,
        "planned_effective_configs":planned.iter().map(|(cfg,_,_)|cfg).collect::<Vec<_>>(),"old_gate_admission_reused":false,
        "optimizer_evaluations":0,"held_out_evaluations":0}),
    )?;
    let mut receipts = Vec::new();
    let mut log = Vec::new();
    for (index, (cfg, emission, edit)) in planned.into_iter().enumerate() {
        let receipt = evaluate(cfg, emission, edit, index);
        let name = format!("rollouts/evaluation_{index:03}.json");
        save(out, &name, &receipt)?;
        log.push(json!({"evaluation":index,"seed":receipt.seed,"arm":receipt.arm,"edit":edit,"requested_steps":STEPS,
            "completed_steps":receipt.completed_steps,"error":receipt.error,"accounting_ok":receipt.accounting_ok,"receipt":name}));
        save(out, "evaluations.json", &log)?;
        eprintln!(
            "persistence {}/{} {:?} edit={}: {} steps {}",
            index + 1,
            ATTEMPTS,
            receipt.arm,
            edit,
            receipt.completed_steps.unwrap_or(0),
            if receipt.error.is_none() {
                "complete"
            } else {
                "failed"
            }
        );
        receipts.push(receipt);
    }
    let contrasts = receipts
        .chunks(6)
        .map(contrast)
        .collect::<Result<Vec<_>, _>>()?;
    save(out, "contrasts.json", &contrasts)?;
    let complete = receipts.iter().filter(|r| complete(r)).count();
    let pass = complete == ATTEMPTS && contrasts.iter().all(|c| c.protected_persistence_pass);
    save(
        out,
        "status.json",
        &json!({"protocol":PROTOCOL,"status":if pass{"protected_persistence_control_passed"}else{"incomplete_or_failed"},
        "attempts":receipts.len(),"complete":complete,"errors":ATTEMPTS-complete,"protected_persistence_pass":pass,
        "actual_steps":receipts.iter().filter_map(|r|r.completed_steps).sum::<u64>(),"requested_steps":2400,
        "unmeasured_completed_step_attempts":receipts.iter().filter(|r|r.completed_steps.is_none()).count(),
        "optimizer_evaluation_steps":0,"held_out_evaluation_steps":0,"full_training_admitted":false}),
    )?;
    Ok(())
}
