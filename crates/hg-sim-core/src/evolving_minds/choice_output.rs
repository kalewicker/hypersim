use super::{choice::*, source_inventory::SOURCES};
use hg_output::{CsvWriter, RunManifest};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};

fn write_new(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(data).map_err(|e| e.to_string())
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(v).map_err(|e| e.to_string())
}
fn metadata(id: &str, seed: u64, steps: u64) -> RunManifest {
    let mut m = RunManifest::default();
    m.run_id = id.into();
    m.experiment_name = CHOICE_PROFILE.into();
    m.seed = seed;
    m.max_steps = steps;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "11 native phases".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m
}
fn flags(complete: bool, reason: Option<&String>) -> serde_json::Value {
    serde_json::json!({"profile":CHOICE_PROFILE,"engineering_execution_complete":complete,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"stop_reason":reason})
}
fn arm(
    dir: &Path,
    controller: Controller,
    seed: u64,
    mode: CueMode,
) -> Result<serde_json::Value, String> {
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut manifest = metadata(controller.name(), seed, 704)
        .create(dir)
        .map_err(|e| e.to_string())?;
    manifest.start().map_err(|e| e.to_string())?;
    let mut sim = match ChoiceSimulation::new(controller, seed, mode) {
        Ok(sim) => sim,
        Err(reason) => {
            manifest.seal_interrupted(0).map_err(|e| e.to_string())?;
            write_new(
                &dir.join("run_status.json"),
                &json(&flags(false, Some(&reason)))?,
            )?;
            return Err(reason);
        }
    };
    let mut advancing_seconds = 0.0;
    let result = (|| {
        write_new(&dir.join("schedule.json"), &json(&sim.schedule)?)?;
        write_new(&dir.join("rules.toml"), sim.schedule.law_text().as_bytes())?;
        write_new(&dir.join("initial.checkpoint"), &sim.checkpoint()?)?;
        let start = Instant::now();
        for t in 1..=64 {
            if start.elapsed().as_secs() >= 60 {
                return Err("choice advancing wall ceiling".into());
            }
            sim.advance_tick()?;
            manifest.final_step = t * 11;
            manifest.persist().map_err(|e| e.to_string())?;
            if t == 32 {
                write_new(&dir.join("midpoint.checkpoint"), &sim.checkpoint()?)?;
            }
        }
        advancing_seconds = start.elapsed().as_secs_f64();
        write_new(&dir.join("final.checkpoint"), &sim.checkpoint()?)?;
        let mut trace = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("trace.jsonl"))
            .map_err(|e| e.to_string())?;
        for f in &sim.frames {
            serde_json::to_writer(&mut trace, f).map_err(|e| e.to_string())?;
            trace.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        write_new(&dir.join("organisms.json"), &json(&sim.lineage)?)?;
        let mut csv = CsvWriter::create(&dir.join("metrics.csv"), 1, controller.name(), seed, "")
            .map_err(|e| e.to_string())?;
        for row in &sim.rows {
            csv.write_row(row).map_err(|e| e.to_string())?;
        }
        csv.flush().map_err(|e| e.to_string())?;
        let bytes: u64 = fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .map(|p| p.unwrap().metadata().unwrap().len())
            .sum();
        if bytes > 64 * 1024 * 1024 - 4096 {
            return Err("choice arm output ceiling".into());
        }
        Ok::<_, String>(())
    })();
    if result.is_ok() {
        manifest.seal_completed(704)
    } else {
        manifest.seal_interrupted(sim.store.step())
    }
    .map_err(|e| e.to_string())?;
    let mut status = flags(result.is_ok(), result.as_ref().err());
    status["controller"] = serde_json::json!(controller);
    status["seed"] = seed.into();
    status["cue_mode"] = serde_json::json!(mode);
    status["advancing_wall_seconds"] = advancing_seconds.into();
    status["final_row"] = serde_json::to_value(sim.rows.last()).map_err(|e| e.to_string())?;
    write_new(&dir.join("run_status.json"), &json(&status)?)?;
    result?;
    Ok(status)
}

pub fn write_choice_panel(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir).map_err(|e| format!("exclusive choice panel: {e}"))?;
    let mut manifest = metadata("authored_choice_panel", 0, 30 * 704)
        .create(dir)
        .map_err(|e| e.to_string())?;
    manifest.start().map_err(|e| e.to_string())?;
    let mut done = 0;
    let result = (|| {
        write_new(
            &dir.join("contract.md"),
            include_bytes!("../../../../experiments/evolving_minds/contracts/choice_v1.md"),
        )?;
        write_new(&dir.join("base_rules.toml"), CHOICE_LAWS.as_bytes())?;
        write_new(
            &dir.join("checker.py"),
            include_bytes!("../../../../scripts/verify_evolving_minds_choice.py"),
        )?;
        write_new(
            &dir.join("replay.py"),
            include_bytes!("../../../../scripts/verify_evolving_minds.py"),
        )?;
        write_new(
            &dir.join("sources.json"),
            &json(
                &SOURCES
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeMap<_, _>>(),
            )?,
        )?;
        let mut outcomes = Vec::new();
        let mut panel_bytes = 0u64;
        for seed in [17, 29] {
            for mode in [CueMode::Informative, CueMode::Permuted, CueMode::Withheld] {
                for controller in [
                    Controller::Inactive,
                    Controller::Constant0,
                    Controller::Constant1,
                    Controller::Random,
                    Controller::Reactive,
                ] {
                    let name = format!("s{seed}_{}_{}", mode.name(), controller.name());
                    let arm_dir = dir.join(name);
                    outcomes.push(arm(&arm_dir, controller, seed, mode)?);
                    for entry in fs::read_dir(&arm_dir).map_err(|e| e.to_string())? {
                        panel_bytes += entry
                            .map_err(|e| e.to_string())?
                            .metadata()
                            .map_err(|e| e.to_string())?
                            .len();
                    }
                    if panel_bytes > 256 * 1024 * 1024 - 8 * 1024 * 1024 {
                        return Err("choice panel output ceiling".into());
                    }
                    done += 704;
                    manifest.final_step = done;
                    manifest.persist().map_err(|e| e.to_string())?;
                }
            }
        }
        write_new(&dir.join("summary.json"), &json(&outcomes)?)?;
        let mut report=String::from("# Authored resource-choice engineering panel\n\nDevelopment only; no evolution or training. Independent verification is separate.\n\n| Seed | Cue | Controller | Harvested | Paid births | Living at tick 64 |\n| --- | --- | --- | ---: | ---: | ---: |\n");
        for s in &outcomes {
            let r = &s["final_row"];
            report += &format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                s["seed"],
                s["cue_mode"],
                s["controller"],
                r["harvested"],
                r["births"],
                r["live_organisms"]
            );
        }
        write_new(&dir.join("report.md"), report.as_bytes())?;
        Ok::<_, String>(())
    })();
    if result.is_ok() {
        manifest.seal_completed(done)
    } else {
        manifest.seal_interrupted(done)
    }
    .map_err(|e| e.to_string())?;
    write_new(
        &dir.join("run_status.json"),
        &json(&flags(result.is_ok(), result.as_ref().err()))?,
    )?;
    result
}
