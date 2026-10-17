use super::{choice::CueMode, d0::*, source_inventory::SOURCES};
use hg_output::RunManifest;
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};

fn put(path: &Path, data: &[u8]) -> Result<(), String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?
        .write_all(data)
        .map_err(|e| e.to_string())
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(v).map_err(|e| e.to_string())
}
fn output_size(dir: &Path) -> Result<u64, String> {
    let mut size = 0;
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        size += if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            output_size(&entry.path())?
        } else {
            entry.metadata().map_err(|e| e.to_string())?.len()
        };
    }
    Ok(size)
}
fn manifest(name: &str, steps: u64) -> RunManifest {
    let mut m = RunManifest::default();
    m.run_id = name.into();
    m.experiment_name = D0_PROFILE.into();
    m.seed = 17;
    m.max_steps = steps;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "15 native passes".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m
}
fn flags(complete: bool, reason: Option<&String>) -> serde_json::Value {
    serde_json::json!({"profile":D0_PROFILE,"engineering_execution_complete":complete,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"stop_reason":reason})
}
fn trace(dir: &Path, sim: &D0Simulation) -> Result<(), String> {
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("trace.jsonl"))
        .map_err(|e| e.to_string())?;
    let mut size = 0;
    for f in &sim.frames {
        let bytes = serde_json::to_vec(f).map_err(|e| e.to_string())?;
        size += bytes.len() + 1;
        if size > 8 * 1024 * 1024 {
            return Err("D0 trace output ceiling".into());
        }
        out.write_all(&bytes)
            .and_then(|_| out.write_all(b"\n"))
            .map_err(|e| e.to_string())?;
    }
    put(&dir.join("rows.json"), &json(&sim.rows)?)
}
fn cell(dir: &Path, name: &str, mode: CueMode, reserve: f64) -> Result<serde_json::Value, String> {
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut m = manifest(name, D0_TICKS * D0_STEPS)
        .create(dir)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let program = d0_program(name)?;
    let mut sim = match D0Simulation::new(program, 17, mode, reserve) {
        Ok(s) => s,
        Err(reason) => {
            m.seal_interrupted(0).map_err(|e| e.to_string())?;
            put(
                &dir.join("run_status.json"),
                &json(&flags(false, Some(&reason)))?,
            )?;
            return Err(reason);
        }
    };
    let result = (|| {
        put(&dir.join("program.json"), &json(&sim.program)?)?;
        put(
            &dir.join("genotype.bin"),
            &sim.graph_program()?.canonical_bytes()?,
        )?;
        put(&dir.join("schedule.json"), &json(&sim.schedule)?)?;
        put(
            &dir.join("rules.toml"),
            d0_law_text(&sim.schedule).as_bytes(),
        )?;
        put(&dir.join("initial.checkpoint"), &sim.checkpoint()?)?;
        let start = Instant::now();
        for tick in 1..=D0_TICKS {
            if start.elapsed().as_secs() >= 60 {
                return Err("D0 advancing wall ceiling".into());
            }
            sim.advance_tick()?;
            m.final_step = tick * D0_STEPS;
            m.persist().map_err(|e| e.to_string())?;
            if tick == 2 {
                put(&dir.join("partial.checkpoint"), &sim.checkpoint()?)?;
            }
        }
        put(&dir.join("final.checkpoint"), &sim.checkpoint()?)?;
        Ok::<_, String>(())
    })();
    // Preserve completed prefix evidence even when an advancing operation fails.
    let mut result = result.and_then(|_| trace(dir, &sim));
    if result.is_err() && !dir.join("trace.jsonl").exists() {
        if let Err(reason) = trace(dir, &sim) {
            result = Err(format!("{}; prefix output: {reason}", result.unwrap_err()));
        }
    }
    if result.is_ok() {
        m.seal_completed(sim.store().step())
    } else {
        m.seal_interrupted(sim.store().step())
    }
    .map_err(|e| e.to_string())?;
    let mut status = flags(result.is_ok(), result.as_ref().err());
    status["program"] = name.into();
    status["cue_mode"] = serde_json::to_value(mode).unwrap();
    status["initial_reserve"] = reserve.into();
    status["final_row"] = serde_json::to_value(sim.rows.last()).unwrap();
    put(&dir.join("run_status.json"), &json(&status)?)?;
    result?;
    Ok(status)
}
pub fn write_d0_panel(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir).map_err(|e| format!("exclusive D0 directory: {e}"))?;
    let mut m = manifest("d0_development_panel", 20 * D0_TICKS * D0_STEPS)
        .create(dir)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let mut done = 0;
    let result = (|| {
        for (name, bytes) in [
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_development_v1.md"
                )
                .as_slice(),
            ),
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_d0.py").as_slice(),
            ),
            (
                "replay.py",
                include_bytes!("../../../../scripts/verify_evolving_minds.py").as_slice(),
            ),
            (
                "schedule_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_choice.py").as_slice(),
            ),
            ("base_rules.toml", D0_LAWS.as_bytes()),
        ] {
            put(&dir.join(name), bytes)?;
        }
        put(
            &dir.join("sources.json"),
            &json(
                &SOURCES
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeMap<_, _>>(),
            )?,
        )?;
        let mut cases = vec![];
        for mode in [CueMode::Informative, CueMode::Permuted, CueMode::Withheld] {
            for name in ["cue", "coin", "constant0", "constant1", "delayed_cue"] {
                cases.push((format!("{}_{}", mode.name(), name), name, mode, 512.0));
            }
        }
        for name in [
            "missing_port",
            "duplicate_writer",
            "duplicate_slot",
            "second_actuator",
        ] {
            cases.push((
                format!("informative_{name}"),
                name,
                CueMode::Informative,
                512.0,
            ));
        }
        cases.push((
            "informative_cue_waiting".into(),
            "cue",
            CueMode::Informative,
            0.0,
        ));
        let mut results = vec![];
        for (folder, name, mode, reserve) in cases {
            results.push(cell(&dir.join(folder), name, mode, reserve)?);
            done += D0_TICKS * D0_STEPS;
            m.final_step = done;
            m.persist().map_err(|e| e.to_string())?;
            let size = output_size(dir)?;
            if size > 192 * 1024 * 1024 - 4 * 1024 * 1024 {
                return Err("D0 panel output ceiling".into());
            }
        }
        put(&dir.join("summary.json"), &json(&results)?)?;
        let mut report=String::from("# D0 development engineering panel\n\nDevelopment only. One funded founder; reproduction and mutation remain pending.\n\n| Cue | Program | Status | Material | Paid instructions | Attempts | Successes | Harvest |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
        for s in &results {
            let r = &s["final_row"];
            report += &format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                s["cue_mode"],
                s["program"],
                r["status"],
                r["material_nodes"],
                r["development_paid"],
                r["attempts"],
                r["successes"],
                r["harvested"]
            );
        }
        put(&dir.join("report.md"), report.as_bytes())?;
        Ok::<_, String>(())
    })();
    if result.is_ok() {
        m.seal_completed(done)
    } else {
        m.seal_interrupted(done)
    }
    .map_err(|e| e.to_string())?;
    put(
        &dir.join("run_status.json"),
        &json(&flags(result.is_ok(), result.as_ref().err()))?,
    )?;
    result
}
pub fn resume_d0(dir: &Path, checkpoint: &Path) -> Result<(), String> {
    // Validate the source completely before reserving a fresh output directory.
    let mut sim = D0Simulation::resume(&fs::read(checkpoint).map_err(|e| e.to_string())?)?;
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut m = manifest("d0_diagnostic_continuation", D0_TICKS * D0_STEPS)
        .create(dir)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result = (|| {
        let start = Instant::now();
        while sim.rows.len() < D0_TICKS as usize {
            if start.elapsed().as_secs() >= 60 {
                return Err("D0 continuation wall ceiling".into());
            }
            sim.advance_tick()?;
            m.final_step = sim.store().step();
            m.persist().map_err(|e| e.to_string())?;
        }
        trace(dir, &sim)?;
        put(&dir.join("final.checkpoint"), &sim.checkpoint()?)?;
        Ok::<_, String>(())
    })();
    if result.is_ok() {
        m.seal_completed(sim.store().step())
    } else {
        m.seal_interrupted(sim.store().step())
    }
    .map_err(|e| e.to_string())?;
    put(
        &dir.join("run_status.json"),
        &json(&flags(result.is_ok(), result.as_ref().err()))?,
    )?;
    result
}
