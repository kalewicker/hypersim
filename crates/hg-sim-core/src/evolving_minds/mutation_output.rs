use super::{choice::CueMode, family::*, mutation::*, source_inventory::SOURCES};
use hg_output::RunManifest;
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
fn put(path: &Path, bytes: &[u8]) -> Result<(), String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?
        .write_all(bytes)
        .map_err(|e| e.to_string())
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(v).map_err(|e| e.to_string())
}
fn flags(ok: bool, reason: Option<&String>) -> serde_json::Value {
    serde_json::json!({"profile":MUTATION_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"mutation_mechanism_enabled":true,"stop_reason":reason})
}
fn metadata(name: &str, steps: u64) -> RunManifest {
    let mut m = RunManifest::default();
    m.run_id = name.into();
    m.experiment_name = MUTATION_PROFILE.into();
    m.seed = 17;
    m.max_steps = steps;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "45 native passes".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m
}
fn evidence(dir: &Path, s: &FamilySimulation) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("journal.jsonl"))
        .map_err(|e| e.to_string())?;
    for event in &s.journal {
        serde_json::to_writer(&mut file, event).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    put(&dir.join("final.json"), &json(&s.final_frame())?)?;
    put(&dir.join("rows.json"), &json(&s.rows)?)?;
    put(&dir.join("lineage.json"), &json(&s.lineage)?)?;
    put(
        &dir.join("mutations.json"),
        &json(
            &s.mutation
                .as_ref()
                .ok_or("mutation profile required")?
                .records,
        )?,
    )?;
    put(&dir.join("mutation_state.json"), &json(&s.mutation)?)
}
fn size(dir: &Path) -> Result<u64, String> {
    let mut total = 0;
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let e = entry.map_err(|e| e.to_string())?;
        total += if e.file_type().map_err(|e| e.to_string())?.is_dir() {
            size(&e.path())?
        } else {
            e.metadata().map_err(|e| e.to_string())?.len()
        };
    }
    Ok(total)
}
fn cell(
    dir: &Path,
    name: &str,
    mode: CueMode,
    kernel: MutationKernel,
) -> Result<serde_json::Value, String> {
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut m = metadata(name, FAMILY_TICKS * MUTATION_STEPS)
        .create(dir)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let mut sim = match FamilySimulation::new_mutation(family_program(name)?, 17, mode, kernel) {
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
        put(&dir.join("schedule.json"), &json(&sim.schedule)?)?;
        put(
            &dir.join("rules.toml"),
            mutation_law_text(&sim.schedule).as_bytes(),
        )?;
        put(&dir.join("initial.json"), &json(&sim.initial)?)?;
        put(&dir.join("initial.checkpoint"), &sim.checkpoint()?)?;
        let start = Instant::now();
        for tick in 1..=FAMILY_TICKS {
            if start.elapsed().as_secs() >= 120 {
                return Err("family advancing wall ceiling".into());
            }
            sim.advance_tick()?;
            m.final_step = tick * MUTATION_STEPS;
            m.persist().map_err(|e| e.to_string())?;
            if tick == 3 {
                put(&dir.join("mutation.checkpoint"), &sim.checkpoint()?)?;
            }
            if tick == 16 {
                put(&dir.join("midpoint.checkpoint"), &sim.checkpoint()?)?;
            }
        }
        put(&dir.join("final.checkpoint"), &sim.checkpoint()?)?;
        Ok::<_, String>(())
    })();
    let mut result = result.and_then(|_| evidence(dir, &sim));
    if result.is_err() && !dir.join("journal.jsonl").exists() {
        if let Err(e) = evidence(dir, &sim) {
            result = Err(format!("{}; prefix output: {e}", result.unwrap_err()))
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
    status["kernel"] = serde_json::to_value(kernel).unwrap();
    status["cue_mode"] = serde_json::to_value(mode).unwrap();
    status["final_row"] = serde_json::to_value(sim.rows.last()).unwrap();
    put(&dir.join("run_status.json"), &json(&status)?)?;
    result?;
    Ok(status)
}
pub fn write_mutation_panel(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir).map_err(|e| format!("exclusive mutation directory: {e}"))?;
    let mut m = metadata("one_shot_mutation_panel", 6 * FAMILY_TICKS * MUTATION_STEPS)
        .create(dir)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let mut done = 0;
    let result = (|| {
        for (name, bytes) in [
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_mutation_v1.md"
                )
                .as_slice(),
            ),
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_mutation.py").as_slice(),
            ),
            (
                "replay.py",
                include_bytes!("../../../../scripts/verify_evolving_minds.py").as_slice(),
            ),
            (
                "d0_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_d0.py").as_slice(),
            ),
            (
                "schedule_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_choice.py").as_slice(),
            ),
            ("base_rules.toml", MUTATION_LAWS.as_bytes()),
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
        let mut statuses = vec![];
        for (name, kernel) in [
            ("cue", MutationKernel::Faithful),
            ("cue", MutationKernel::SourceKernel),
            ("constant1", MutationKernel::SourceKernel),
            ("cue", MutationKernel::Coin),
            ("cue", MutationKernel::Invalid),
            ("cue", MutationKernel::MissingPort),
        ] {
            statuses.push(cell(
                &dir.join(format!("{}_{}", kernel.name(), name)),
                name,
                CueMode::Informative,
                kernel,
            )?);
            done += FAMILY_TICKS * MUTATION_STEPS;
            m.final_step = done;
            m.persist().map_err(|e| e.to_string())?;
            if size(dir)? > 128 * 1024 * 1024 - 4 * 1024 * 1024 {
                return Err("family panel output ceiling".into());
            }
        }
        put(&dir.join("summary.json"), &json(&statuses)?)?;
        put(
            &dir.join("neighborhoods.json"),
            &json(&vec![
                source_neighborhood(&family_program("cue")?),
                source_neighborhood(&family_program("constant1")?),
            ])?,
        )?;
        let mut report=String::from("# One-shot source mutation and escrow engineering panel\n\nDevelopment only; no scientific evolution or training.\n\n| Kernel | Program | Paid births | Deaths | Ancestry depth | Living at 64 | Harvest |\n| --- | --- | ---: | ---: | ---: | ---: | ---: |\n");
        for s in &statuses {
            let r = &s["final_row"];
            report += &format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                s["kernel"],
                s["program"],
                r["births"],
                r["deaths"],
                r["max_ancestry_depth"],
                r["live_organisms"],
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
pub fn resume_mutation(dir: &Path, checkpoint: &Path) -> Result<(), String> {
    let mut sim = FamilySimulation::resume(&fs::read(checkpoint).map_err(|e| e.to_string())?)?;
    if sim.admission.is_some() || sim.mutation.is_none() || sim.lifecycle.is_some() {
        return Err("mutation checkpoint required".into());
    }
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut m = metadata(
        "mutation_diagnostic_continuation",
        FAMILY_TICKS * MUTATION_STEPS,
    )
    .create(dir)
    .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result = (|| {
        let start = Instant::now();
        while sim.rows.len() < 64 {
            if start.elapsed().as_secs() >= 120 {
                return Err("family continuation wall ceiling".into());
            }
            sim.advance_tick()?;
            m.final_step = sim.store().step();
            m.persist().map_err(|e| e.to_string())?;
        }
        evidence(dir, &sim)?;
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
