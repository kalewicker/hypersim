use super::{
    config::*,
    runtime::{checksum, Simulation},
};
use hg_output::{CsvWriter, RunManifest};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

fn write_new(path: &Path, bytes: &[u8], used: &mut u64, cap: u64) -> Result<(), String> {
    if bytes.len() as u64 > cap.saturating_sub(*used) {
        return Err("output byte ceiling".into());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    *used += bytes.len() as u64;
    Ok(())
}
fn json<T: Serialize + ?Sized>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(v).map_err(|e| e.to_string())
}

pub(crate) fn write_run(
    dir: &Path,
    config: Config,
    instruction: Instruction,
) -> Result<(), String> {
    fs::create_dir(dir).map_err(|e| format!("exclusive run directory: {e}"))?;
    let result = write_owned_run(dir, config, instruction);
    if let Err(reason) = &result {
        if let Ok(mut manifest) = RunManifest::load(dir) {
            if matches!(
                manifest.state,
                hg_output::RunState::Created | hg_output::RunState::Running
            ) {
                manifest
                    .seal_interrupted(manifest.final_step)
                    .map_err(|e| e.to_string())?;
            }
        }
        if !dir.join("run_status.json").exists() {
            let status = serde_json::json!({"profile":PROFILE,"engineering_execution_complete":false,
                "stop_reason":reason,"development_only":true,"scientific_run_launched":false,
                "gate_pass":false,"training_admitted":false,"evolution_enabled":false});
            let mut used = 0;
            write_new(
                &dir.join("run_status.json"),
                &json(&status)?,
                &mut used,
                4096,
            )?;
        }
    }
    result
}
fn write_owned_run(dir: &Path, config: Config, instruction: Instruction) -> Result<(), String> {
    let run_id = match instruction {
        Instruction::BuildRelay => "authored_relay",
        Instruction::Idle => "authored_idle",
    };
    let mut metadata = RunManifest::default();
    metadata.run_id = run_id.into();
    metadata.experiment_name = PROFILE.into();
    metadata.seed = config.seed;
    metadata.thread_count = rayon::current_num_threads();
    metadata.step_mode = "MaxParallel/9-explicit-phases".into();
    metadata.hypersim_version = env!("CARGO_PKG_VERSION").into();
    metadata.build_features = vec!["evolving-minds".into()];
    if cfg!(feature = "alife") {
        metadata.build_features.push("alife".into());
    }
    if cfg!(feature = "cognition") {
        metadata.build_features.push("cognition".into());
    }
    if cfg!(feature = "hg-meta") {
        metadata.build_features.push("hg-meta".into());
    }
    metadata.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    metadata.rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unavailable at invocation".into())
        .trim()
        .to_string();
    metadata.max_steps = config.ticks * PHASES.len() as u64;
    metadata.config_hash = format!(
        "fnv64:{:016x}",
        checksum(&json(&(config.clone(), instruction))?)
    );
    let mut manifest = metadata.create(dir).map_err(|e| e.to_string())?;
    manifest.start().map_err(|e| e.to_string())?;
    let mut used = fs::metadata(dir.join("manifest.json"))
        .map_err(|e| e.to_string())?
        .len();
    let cap = config.output_bytes.saturating_sub(4096); // reserve terminal failure/status space
    write_new(&dir.join("config.json"), &json(&config)?, &mut used, cap)?;
    write_new(&dir.join("rules.toml"), RULES.as_bytes(), &mut used, cap)?;
    let sources: std::collections::BTreeMap<_, _> =
        super::source_inventory::SOURCES.iter().copied().collect();
    write_new(&dir.join("sources.json"), &json(&sources)?, &mut used, cap)?;
    write_new(
        &dir.join("contract.md"),
        include_bytes!("../../../../experiments/evolving_minds/contracts/engineering_v1.md"),
        &mut used,
        cap,
    )?;
    write_new(
        &dir.join("checker.py"),
        include_bytes!("../../../../scripts/verify_evolving_minds.py"),
        &mut used,
        cap,
    )?;
    let mut sim = Simulation::new(config.clone(), instruction)?;
    let midpoint = config.ticks / 2;
    let mut run_error = None;
    let started = std::time::Instant::now();
    while sim.tick() < config.ticks {
        if started.elapsed().as_secs() >= 60 {
            run_error = Some("60-second advancing-run wall ceiling".into());
            break;
        }
        if let Err(e) = sim.advance_tick() {
            run_error = Some(e);
            break;
        }
        manifest.final_step = sim.store().step();
        manifest.persist().map_err(|e| e.to_string())?;
        if sim.tick() == midpoint {
            write_new(
                &dir.join("midpoint.checkpoint"),
                &sim.checkpoint()?,
                &mut used,
                cap,
            )?;
        }
    }
    let mut trace = Vec::new();
    for frame in sim.frames() {
        serde_json::to_writer(&mut trace, frame).map_err(|e| e.to_string())?;
        trace.push(b'\n');
    }
    write_new(&dir.join("trace.jsonl"), &trace, &mut used, cap)?;
    write_new(
        &dir.join("organisms.json"),
        &json(sim.lineage())?,
        &mut used,
        cap,
    )?;
    if run_error.is_none() {
        write_new(
            &dir.join("final.checkpoint"),
            &sim.checkpoint()?,
            &mut used,
            cap,
        )?;
    }
    if json(sim.rows())?.len() as u64 + 2048 > cap.saturating_sub(used) {
        return Err("CSV output byte ceiling".into());
    }
    let mut csv = CsvWriter::create(&dir.join("metrics.csv"), 1, run_id, config.seed, "")
        .map_err(|e| e.to_string())?;
    for row in sim.rows() {
        csv.write_row(row).map_err(|e| e.to_string())?;
    }
    csv.flush().map_err(|e| e.to_string())?;
    used += fs::metadata(dir.join("metrics.csv"))
        .map_err(|e| e.to_string())?
        .len();
    let last = sim.rows().last();
    let status = serde_json::json!({"profile":PROFILE,"instruction":instruction,"development_only":true,
        "engineering_execution_complete":run_error.is_none(),"completed_ticks":sim.tick(),
        "scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,
        "stop_reason":run_error,"final_row":last,"artifact_bytes_before_status":used,
        "advancing_wall_seconds":started.elapsed().as_secs_f64()});
    write_new(
        &dir.join("run_status.json"),
        &json(&status)?,
        &mut used,
        cap,
    )?;
    let report=format!("# Evolving Minds engineering fixture\n\nProfile `{PROFILE}`; authored `{instruction:?}`.\n\nCompleted {} of {} environmental ticks ({} native steps).\n\n{}\n\nNo evolutionary, learning, cognitive, or training-admission claim.\nIndependent verification is a separate read-only checker step.\n",sim.tick(),config.ticks,sim.store().step(),serde_json::to_string_pretty(&last).map_err(|e|e.to_string())?);
    write_new(&dir.join("report.md"), report.as_bytes(), &mut used, cap)?;
    if status["engineering_execution_complete"] == true {
        manifest
            .seal_completed(sim.store().step())
            .map_err(|e| e.to_string())?;
    } else {
        manifest
            .seal_interrupted(sim.store().step())
            .map_err(|e| e.to_string())?;
    }
    if let Some(e) = status["stop_reason"].as_str() {
        return Err(e.into());
    }
    Ok(())
}

/// Two engineering control arms; neither evolves or trains.
pub fn write_smoke(dir: &Path, config: Config) -> Result<(), String> {
    config.validate()?;
    fs::create_dir(dir).map_err(|e| format!("exclusive smoke directory: {e}"))?;
    write_run(&dir.join("active"), config.clone(), Instruction::BuildRelay)?;
    write_run(&dir.join("idle"), config, Instruction::Idle)
}

/// Fresh-process continuation used to verify complete state, not only a hash.
pub fn resume_to(dir: &Path, checkpoint: &Path) -> Result<(), String> {
    let size = fs::metadata(checkpoint).map_err(|e| e.to_string())?.len();
    if size > 64 * 1024 * 1024 + 24 {
        return Err("checkpoint file ceiling".into());
    }
    let mut sim = Simulation::resume(&fs::read(checkpoint).map_err(|e| e.to_string())?)?;
    let started = std::time::Instant::now();
    while sim.tick() < sim.config.ticks {
        if started.elapsed().as_secs() >= 60 {
            return Err("60-second resume wall ceiling".into());
        }
        sim.advance_tick()?;
    }
    fs::create_dir(dir).map_err(|e| format!("exclusive resume directory: {e}"))?;
    let mut used = 0;
    let cap = sim.config.output_bytes;
    write_new(&dir.join("rows.json"), &json(sim.rows())?, &mut used, cap)?;
    let mut trace = Vec::new();
    for frame in sim.frames() {
        serde_json::to_writer(&mut trace, frame).map_err(|e| e.to_string())?;
        trace.push(b'\n');
    }
    write_new(&dir.join("trace.jsonl"), &trace, &mut used, cap)?;
    write_new(
        &dir.join("organisms.json"),
        &json(sim.lineage())?,
        &mut used,
        cap,
    )?;
    write_new(
        &dir.join("final.checkpoint"),
        &sim.checkpoint()?,
        &mut used,
        cap,
    )
}
