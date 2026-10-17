//! Isolated diagnostic forks, never admitted back into lifecycle execution.
use super::{
    config::*,
    runtime::{float, int, Capture, Frame},
    Simulation,
};
use hg_core::{NodeId, PropertyValue};
use hg_engine::Engine;
use hg_observe::StepStats;
use hg_output::RunManifest;
use hg_rules::{compile_rules, parse_rules_toml, MatchBudgetPolicy};
use serde::Serialize;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Serialize)]
pub struct Probe {
    pub arm: String,
    pub target_birth_id: i64,
    pub sensor: NodeId,
    pub connection: NodeId,
    pub actuator: NodeId,
    pub lifecycle_valid: bool,
    pub frames: Vec<Frame>,
    pub acquired: f64,
    pub edits: Vec<(NodeId, u32, PropertyValue)>,
}

/// Fixed calibration boundary: founder-only, developed relay at completed tick 2.
/// Every arm decodes the exact same checkpoint, including engine RNG/allocators.
pub fn probe_checkpoint(bytes: &[u8], arm: &str) -> Result<Probe, String> {
    if !["sham", "signal_clamp", "foreign_owner", "owner_rescue"].contains(&arm) {
        return Err("unknown diagnostic arm".into());
    }
    let mut sim = Simulation::resume(bytes)?;
    if sim.tick() != 2 || sim.lineage().len() != 1 || sim.instruction != Instruction::BuildRelay {
        return Err("probe requires founder-only relay checkpoint at tick 2".into());
    }
    let owned = |label| -> Result<NodeId, String> {
        let ids: Vec<_> = sim
            .store
            .nodes()
            .filter(|(_, n)| n.label.0 == label && int(n, OWNER).ok() == Some(1))
            .map(|(id, _)| id)
            .collect();
        if ids.len() != 1 {
            return Err("ambiguous semantic target".into());
        }
        Ok(ids[0])
    };
    let sensor = owned(SENSOR)?;
    let connection = owned(CONNECTION)?;
    let actuator = owned(ACTUATOR)?;
    let root = sim
        .store
        .nodes()
        .find(|(_, n)| n.label.0 == ROOT && int(n, BIRTH_ID).ok() == Some(1))
        .ok_or("founder absent")?
        .0;
    if sim.store.get_node(sensor).unwrap().get_property(SIGNAL) != Some(PropertyValue::Bool(true)) {
        return Err("positive sensor calibration absent".into());
    }
    // Common washout prevents a stale actuator from bypassing the tested signal.
    let mut edits = vec![(actuator, SIGNAL, PropertyValue::Bool(false))];
    sim.store
        .get_node_mut(actuator)
        .unwrap()
        .set_property(SIGNAL, PropertyValue::Bool(false));
    if arm == "signal_clamp" {
        edits.push((sensor, SIGNAL, PropertyValue::Bool(false)));
        sim.store
            .get_node_mut(sensor)
            .unwrap()
            .set_property(SIGNAL, PropertyValue::Bool(false));
    }
    if arm == "foreign_owner" || arm == "owner_rescue" {
        edits.push((connection, OWNER, PropertyValue::Int(999)));
        sim.store
            .get_node_mut(connection)
            .unwrap()
            .set_property(OWNER, PropertyValue::Int(999));
        if arm == "owner_rescue" {
            edits.push((connection, OWNER, PropertyValue::Int(1)));
            sim.store
                .get_node_mut(connection)
                .unwrap()
                .set_property(OWNER, PropertyValue::Int(1));
        }
    }
    let lifecycle_valid = sim.validate_material().is_ok();
    if lifecycle_valid != (arm != "foreign_owner") {
        return Err("unexpected lifecycle validity".into());
    }
    let mut engine = Engine::from_diagnostic_checkpoint(
        // Use the source checkpoint's engine state, not a new seeded scheduler.
        Simulation::resume(bytes)?.diagnostic_engine_checkpoint()?,
        None,
    )?;
    let capture = Arc::new(Mutex::new(Vec::new()));
    engine.add_observer(Box::new(Capture(capture.clone())));
    engine.set_match_budget_policy(
        sim.config.match_work,
        sim.config.match_results,
        MatchBudgetPolicy::Pooled,
    );
    let rules = compile_rules(&parse_rules_toml(RULES).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let initial = float(sim.store.get_node(root).unwrap(), ENERGY)?;
    let mut frames = vec![Frame::new(
        &sim.store,
        2,
        None,
        vec![],
        StepStats::default(),
        0,
    )];
    for phase in [Phase::Signal, Phase::Action] {
        engine.replace_static_rules(
            rules
                .iter()
                .filter(|r| r.name.starts_with(phase.prefix()))
                .cloned()
                .collect(),
        );
        let stats = engine.step(&mut sim.store);
        if engine.match_ceiling_binding() || stats.property_effects_skipped != 0 {
            return Err("probe matching/effect failure".into());
        }
        frames.push(Frame::new(
            &sim.store,
            2,
            Some(phase),
            std::mem::take(&mut *capture.lock().unwrap()),
            stats,
            engine.match_budget_usage().work,
        ));
    }
    let acquired = float(sim.store.get_node(root).unwrap(), ENERGY)? - initial;
    if acquired
        != if arm == "sham" || arm == "owner_rescue" {
            8.0
        } else {
            0.0
        }
    {
        return Err("probe calibration outcome".into());
    }
    Ok(Probe {
        arm: arm.into(),
        target_birth_id: 1,
        sensor,
        connection,
        actuator,
        lifecycle_valid,
        frames,
        acquired,
        edits,
    })
}

pub fn write_interventions(dir: &Path) -> Result<(), String> {
    std::fs::create_dir(dir).map_err(|e| format!("exclusive intervention directory: {e}"))?;
    let mut metadata = RunManifest::default();
    metadata.run_id = "evolving_minds_causal_controls_v1".into();
    metadata.experiment_name = metadata.run_id.clone();
    metadata.max_steps = 8;
    metadata.seed = 42;
    metadata.thread_count = rayon::current_num_threads();
    metadata.build_features = vec!["evolving-minds".into()];
    metadata.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    metadata.step_mode = "isolated Signal/Action checkpoint forks".into();
    let mut manifest = metadata.create(dir).map_err(|e| e.to_string())?;
    manifest.start().map_err(|e| e.to_string())?;
    let result = (|| {
        std::fs::write(
            dir.join("contract.md"),
            include_bytes!(
                "../../../../experiments/evolving_minds/contracts/causal_controls_v1.md"
            ),
        )
        .map_err(|e| e.to_string())?;
        let config = Config {
            ticks: 2,
            ..Config::default()
        };
        super::output::write_run(&dir.join("baseline"), config, Instruction::BuildRelay)?;
        let bytes =
            std::fs::read(dir.join("baseline/final.checkpoint")).map_err(|e| e.to_string())?;
        let probes: Vec<_> = ["sham", "signal_clamp", "foreign_owner", "owner_rescue"]
            .iter()
            .map(|arm| probe_checkpoint(&bytes, arm))
            .collect::<Result<_, _>>()?;
        std::fs::write(
            dir.join("probes.json"),
            serde_json::to_vec_pretty(&probes).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        std::fs::write(
            dir.join("checker.py"),
            include_bytes!("../../../../scripts/verify_evolving_minds_interventions.py"),
        )
        .map_err(|e| e.to_string())?;
        Ok::<_, String>(())
    })();
    if result.is_ok() {
        manifest.seal_completed(8)
    } else {
        manifest.seal_interrupted(0)
    }
    .map_err(|e| e.to_string())?;
    let status = serde_json::json!({"profile":"evolving-minds-causal-controls-v1","engineering_execution_complete":result.is_ok(),"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"stop_reason":result.as_ref().err()});
    std::fs::write(
        dir.join("run_status.json"),
        serde_json::to_vec_pretty(&status).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    result
}
