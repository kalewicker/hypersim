use super::{admission::*, family::*, source_inventory::SOURCES};
use hg_output::RunManifest;
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
fn put(p: &Path, b: &[u8]) -> Result<(), String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| e.to_string())?
        .write_all(b)
        .map_err(|e| e.to_string())
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(v).map_err(|e| e.to_string())
}
fn flags(ok: bool, reason: Option<&str>) -> serde_json::Value {
    serde_json::json!({"profile":ADMISSION_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false,"stop_reason":reason})
}
fn meta(name: &str, steps: u64) -> RunManifest {
    let mut m = RunManifest::default();
    m.run_id = name.into();
    m.experiment_name = ADMISSION_PROFILE.into();
    m.seed = 17;
    m.max_steps = steps;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "46 native passes; supplied semantic admission lottery".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m
}
fn evidence(p: &Path, s: &FamilySimulation) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p.join("journal.jsonl"))
        .map_err(|e| e.to_string())?;
    for e in &s.journal {
        serde_json::to_writer(&mut file, e).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    for (name, bytes) in [
        ("rows.json", json(&s.rows)?),
        ("lineage.json", json(&s.lineage)?),
        ("final.json", json(&s.final_frame())?),
        (
            "admission_state.json",
            json(&s.admission.as_ref().ok_or("admission profile")?)?,
        ),
    ] {
        put(&p.join(name), &bytes)?
    }
    Ok(())
}
fn size(p: &Path) -> Result<u64, String> {
    let mut n = 0;
    for e in fs::read_dir(p).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        n += if e.file_type().map_err(|e| e.to_string())?.is_dir() {
            size(&e.path())?
        } else {
            e.metadata().map_err(|e| e.to_string())?.len()
        };
    }
    Ok(n)
}
fn cell(p: &Path, recipe: AdmissionRecipe) -> Result<serde_json::Value, String> {
    fs::create_dir(p).map_err(|e| e.to_string())?;
    let mut m = meta(&recipe.name(), 64 * ADMISSION_STEPS)
        .create(p)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let mut sim = match FamilySimulation::new_admission(recipe.clone()) {
        Ok(s) => s,
        Err(reason) => {
            m.seal_interrupted(0).map_err(|e| e.to_string())?;
            put(
                &p.join("run_status.json"),
                &json(&flags(false, Some(&reason)))?,
            )?;
            return Err(reason);
        }
    };
    let result: Result<serde_json::Value, String> = (|| {
        put(&p.join("recipe.json"), &json(&recipe)?)?;
        put(&p.join("schedule.json"), &json(&sim.schedule)?)?;
        put(
            &p.join("rules.toml"),
            admission_law_text(&sim.schedule).as_bytes(),
        )?;
        put(&p.join("initial.json"), &json(&sim.initial)?)?;
        put(&p.join("initial.checkpoint"), &sim.checkpoint()?)?;
        let start = Instant::now();
        for tick in 1..=64 {
            if start.elapsed().as_secs() >= 120 {
                return Err("admission advancing wall ceiling".into());
            }
            sim.advance_tick()?;
            m.final_step = tick * ADMISSION_STEPS;
            m.persist().map_err(|e| e.to_string())?;
            if [8, 16].contains(&tick) {
                put(
                    &p.join(if tick == 8 {
                        "admission.checkpoint"
                    } else {
                        "midpoint.checkpoint"
                    }),
                    &sim.checkpoint()?,
                )?;
            }
        }
        evidence(p, &sim)?;
        put(&p.join("final.checkpoint"), &sim.checkpoint()?)?;
        let mut status = flags(true, None);
        status["recipe"] = serde_json::to_value(&recipe).unwrap();
        status["final_row"] = serde_json::to_value(sim.rows.last()).unwrap();
        status["requests"] = sim.admission.as_ref().unwrap().records.len().into();
        put(&p.join("run_status.json"), &json(&status)?)?;
        m.final_step = 64 * ADMISSION_STEPS;
        m.seal_completed(m.final_step).map_err(|e| e.to_string())?;
        Ok(status)
    })();
    if let Err(reason) = &result {
        m.seal_interrupted(m.final_step)
            .map_err(|e| e.to_string())?;
        let status = p.join("run_status.json");
        if !status.exists() {
            put(&status, &json(&flags(false, Some(reason)))?)?
        }
    }
    result
}
fn calibration() -> Result<Vec<serde_json::Value>, String> {
    let mut out = vec![];
    for seed in 0..32 {
        for rotation in 0..8 {
            for reversed in [false, true] {
                for competing in [false, true] {
                    let homes = [rotation, (rotation + if competing { 2 } else { 4 }) % 8];
                    let targets = [
                        (rotation + 1) % 8,
                        (rotation + if competing { 1 } else { 5 }) % 8,
                    ];
                    let mut physical = vec![
                        serde_json::json!({"parent":1,"home":homes[0],"target":targets[0],"length":3}),
                        serde_json::json!({"parent":2,"home":homes[1],"target":targets[1],"length":3}),
                    ];
                    if reversed {
                        physical.reverse()
                    }
                    let offers = physical
                        .iter()
                        .map(|o| AdmissionOffer {
                            parent: o["parent"].as_i64().unwrap(),
                            home: (o["home"].as_i64().unwrap() - rotation + 8) % 8,
                            target: (o["target"].as_i64().unwrap() - rotation + 8) % 8,
                            length: 3,
                        })
                        .collect();
                    out.push(serde_json::json!({"seed":seed,"rotation":rotation,"reversed":reversed,"competing":competing,"physical_offers":physical,"round":lottery(seed,1,offers)?}));
                }
            }
        }
    }
    Ok(out)
}
pub fn write_admission_panel(p: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive admission directory: {e}"))?;
    let mut m = meta("parallel_admission_panel", 12 * 64 * ADMISSION_STEPS)
        .create(p)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result: Result<(), String> = (|| {
        for (name, bytes) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_admission.py").as_slice(),
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
            (
                "mutation_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_mutation.py").as_slice(),
            ),
            ("base_rules.toml", ADMISSION_LAWS.as_bytes()),
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_admission_v1.md"
                )
                .as_slice(),
            ),
        ] {
            put(&p.join(name), bytes)?
        }
        let sources: std::collections::BTreeMap<_, _> = SOURCES.iter().copied().collect();
        put(&p.join("sources.json"), &json(&sources)?)?;
        put(&p.join("lottery_calibration.json"), &json(&calibration()?)?)?;
        let mut statuses = vec![];
        for case in ["faithful_disjoint", "mixed_competing", "mixed_settlement"] {
            for rotation in [0, 3] {
                for reversed in [false, true] {
                    let recipe = AdmissionRecipe {
                        case: case.into(),
                        rotation,
                        reversed,
                    };
                    let status = cell(&p.join(recipe.name()), recipe)?;
                    statuses.push(status);
                    m.final_step += 64 * ADMISSION_STEPS;
                    m.persist().map_err(|e| e.to_string())?;
                    if size(p)? > 256 * 1024 * 1024 - 1024 * 1024 {
                        return Err("admission output ceiling".into());
                    }
                }
            }
        }
        put(&p.join("summary.json"), &json(&statuses)?)?;
        put(&p.join("run_status.json"), &json(&flags(true, None))?)?;
        put(&p.join("report.md"),b"# Parallel admission engineering panel\n\nTwelve full lifecycle cells and 1024 read-only lottery cases. Native paid parallel settlement; external semantic scheduling. Broad reproductive fairness and scientific evolution remain unaccepted.\n")?;
        m.seal_completed(m.final_step).map_err(|e| e.to_string())
    })();
    if let Err(reason) = &result {
        m.seal_interrupted(m.final_step)
            .map_err(|e| e.to_string())?;
        if !p.join("run_status.json").exists() {
            put(
                &p.join("run_status.json"),
                &json(&flags(false, Some(reason)))?,
            )?
        }
    }
    result
}
pub fn resume_admission(p: &Path, cp: &Path) -> Result<(), String> {
    let mut s = FamilySimulation::resume(&fs::read(cp).map_err(|e| e.to_string())?)?;
    if s.admission.is_none() {
        return Err("admission checkpoint required".into());
    }
    fs::create_dir(p).map_err(|e| format!("exclusive admission directory: {e}"))?;
    let mut m = meta("parallel_admission_continuation", 64 * ADMISSION_STEPS)
        .create(p)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result: Result<(), String> = (|| {
        let start = Instant::now();
        while s.rows.len() < 64 {
            if start.elapsed().as_secs() >= 120 {
                return Err("admission continuation wall ceiling".into());
            }
            s.advance_tick()?;
            m.final_step = s.rows.len() as u64 * ADMISSION_STEPS;
            m.persist().map_err(|e| e.to_string())?;
        }
        evidence(p, &s)?;
        put(&p.join("final.checkpoint"), &s.checkpoint()?)?;
        put(&p.join("run_status.json"), &json(&flags(true, None))?)?;
        m.final_step = 64 * ADMISSION_STEPS;
        m.seal_completed(m.final_step).map_err(|e| e.to_string())
    })();
    if let Err(reason) = &result {
        m.seal_interrupted(m.final_step)
            .map_err(|e| e.to_string())?;
        if !p.join("run_status.json").exists() {
            put(
                &p.join("run_status.json"),
                &json(&flags(false, Some(reason)))?,
            )?
        }
    }
    result
}
