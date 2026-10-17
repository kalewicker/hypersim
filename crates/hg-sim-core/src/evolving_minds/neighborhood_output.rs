use super::{family::*, mutation::*, neighborhood::*, source_inventory::SOURCES};
use hg_output::RunManifest;
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
fn put(p: &Path, b: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| e.to_string())?;
    f.write_all(b).map_err(|e| e.to_string())
}
fn json<T: Serialize>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(v).map_err(|e| e.to_string())
}
fn size(p: &Path) -> Result<u64, String> {
    let mut n = 0;
    for e in fs::read_dir(p).map_err(|e| e.to_string())? {
        let p = e.map_err(|e| e.to_string())?.path();
        n += if p.is_dir() {
            size(&p)?
        } else {
            p.metadata().map_err(|e| e.to_string())?.len()
        };
    }
    Ok(n)
}
fn flags(ok: bool, reason: Option<&str>) -> serde_json::Value {
    serde_json::json!({"profile":NEIGHBORHOOD_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false,"mutation_mechanism_enabled":true,"stop_reason":reason})
}
fn meta(name: &str, steps: u64) -> RunManifest {
    let mut m = RunManifest::default();
    m.run_id = name.into();
    m.experiment_name = NEIGHBORHOOD_PROFILE.into();
    m.seed = 17;
    m.max_steps = steps;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "45 native passes; exhaustive first request; faithful descendants".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m.rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    m
}
fn cell(p: &Path, recipe: NeighborhoodRecipe) -> Result<NeighborhoodOutcome, String> {
    fs::create_dir(p).map_err(|e| e.to_string())?;
    let mut m = meta(&recipe.name(), 2880)
        .create(p)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result: Result<NeighborhoodOutcome, String> = (|| {
        let mut s = FamilySimulation::new_neighborhood(recipe.clone())?;
        put(&p.join("recipe.json"), &json(&recipe)?)?;
        put(&p.join("program.json"), &json(&s.program)?)?;
        put(&p.join("schedule.json"), &json(&s.schedule)?)?;
        put(
            &p.join("rules.toml"),
            mutation_law_text(&s.schedule).as_bytes(),
        )?;
        put(
            &p.join("descendant_reserve.toml"),
            descendant_reserve_text()?.as_bytes(),
        )?;
        put(&p.join("initial.json"), &json(&s.initial)?)?;
        let start = Instant::now();
        let advancing: Result<(), String> = (|| {
            for _ in 0..64 {
                if start.elapsed().as_secs() >= 120 {
                    return Err("neighborhood cell wall ceiling".into());
                }
                s.advance_tick()?;
                m.final_step = s.completed_tick() * 45;
            }
            Ok(())
        })();
        let advance_seconds = start.elapsed().as_secs_f64();
        let mut log = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p.join("journal.jsonl"))
            .map_err(|e| e.to_string())?;
        for e in &s.journal {
            serde_json::to_writer(&mut log, e).map_err(|e| e.to_string())?;
            log.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        log.flush().map_err(|e| e.to_string())?;
        for (name, b) in [
            ("final.json", json(&s.final_frame())?),
            ("rows.json", json(&s.rows)?),
            ("lineage.json", json(&s.lineage)?),
            (
                "mutations.json",
                json(&s.mutation.as_ref().unwrap().records)?,
            ),
            ("mutation_state.json", json(s.mutation.as_ref().unwrap())?),
            ("lifetimes.json", json(&s.neighborhood_lifetimes)?),
        ] {
            if b.len() > 4 * 1024 * 1024 {
                return Err("neighborhood JSON ceiling".into());
            }
            put(&p.join(name), &b)?;
        }
        put(
            &p.join("resources.json"),
            &json(
                &serde_json::json!({"advance_seconds":advance_seconds,"native_advances":s.completed_tick()*45,"timer_scope":"native advancement"}),
            )?,
        )?;
        advancing?;
        let outcome = neighborhood_outcome(&s, recipe)?;
        put(&p.join("outcome.json"), &json(&outcome)?)?;
        Ok(outcome)
    })();
    match result {
        Ok(outcome) => {
            m.seal_completed(2880).map_err(|e| e.to_string())?;
            let mut status = flags(true, None);
            status["recipe"] = serde_json::to_value(&outcome.recipe).unwrap();
            status["final_row"] = serde_json::to_value(&outcome.final_row).unwrap();
            put(&p.join("run_status.json"), &json(&status)?)?;
            Ok(outcome)
        }
        Err(e) => {
            m.seal_interrupted(m.final_step)
                .map_err(|e| e.to_string())?;
            put(&p.join("run_status.json"), &json(&flags(false, Some(&e)))?)?;
            Err(e)
        }
    }
}
pub fn write_neighborhood_panel(p: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive neighborhood directory: {e}"))?;
    let mut m = meta("paid_neighborhood", 58 * 2880)
        .create(p)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result: Result<(), String> = (|| {
        for (name, b) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_neighborhood.py")
                    .as_slice(),
            ),
            (
                "mutation_reference.py",
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
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_neighborhood_v1.md"
                )
                .as_slice(),
            ),
        ] {
            put(&p.join(name), b)?;
        }
        let sources: std::collections::BTreeMap<_, _> = SOURCES.iter().copied().collect();
        put(&p.join("sources.json"), &json(&sources)?)?;
        let recipes = neighborhood_recipes();
        put(&p.join("registered_cells.json"), &json(&recipes)?)?;
        let mut results = vec![];
        let start = Instant::now();
        for recipe in recipes {
            if start.elapsed().as_secs() >= 1800 {
                return Err("neighborhood panel wall ceiling".into());
            }
            results.push(cell(&p.join(recipe.name()), recipe)?);
            m.final_step += 2880;
            m.persist().map_err(|e| e.to_string())?;
            if size(p)? > 256 * 1024 * 1024 - 1024 * 1024 {
                return Err("neighborhood panel output ceiling".into());
            }
        }
        put(&p.join("summary.json"), &json(&results)?)?;
        put(&p.join("run_status.json"), &json(&flags(true, None))?)?;
        m.seal_completed(m.final_step).map_err(|e| e.to_string())
    })();
    if let Err(e) = &result {
        m.seal_interrupted(m.final_step)
            .map_err(|e| e.to_string())?;
        if !p.join("run_status.json").exists() {
            put(&p.join("run_status.json"), &json(&flags(false, Some(e)))?)?
        }
    }
    result
}
