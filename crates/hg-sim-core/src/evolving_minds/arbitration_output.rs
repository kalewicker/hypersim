use super::{arbitration::*, source_inventory::SOURCES};
use hg_output::RunManifest;
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
fn json<T: serde::Serialize>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(v).map_err(|e| e.to_string())
}
fn flags(ok: bool, reason: Option<&String>) -> serde_json::Value {
    serde_json::json!({"profile":ARBITRATION_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false,"stop_reason":reason})
}
fn meta(name: &str, steps: u64) -> RunManifest {
    let mut m = RunManifest::default();
    m.run_id = name.into();
    m.experiment_name = ARBITRATION_PROFILE.into();
    m.seed = 17;
    m.max_steps = steps;
    m.thread_count = rayon::current_num_threads();
    m.build_features = vec!["evolving-minds".into()];
    m.step_mode = "four native diagnostic passes".into();
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m
}
fn size(path: &Path) -> Result<u64, String> {
    let mut n = 0;
    for e in fs::read_dir(path).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        n += if e.file_type().map_err(|e| e.to_string())?.is_dir() {
            size(&e.path())?
        } else {
            e.metadata().map_err(|e| e.to_string())?.len()
        };
    }
    Ok(n)
}
pub fn write_arbitration_panel(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir).map_err(|e| format!("exclusive arbitration directory: {e}"))?;
    let mut m = meta("birth_arbitration_panel", 256)
        .create(dir)
        .map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let mut done = 0;
    let result = (|| {
        for (name, bytes) in [
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_lifecycle_fairness_v1.md"
                )
                .as_slice(),
            ),
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_arbitration.py")
                    .as_slice(),
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
            ("base_rules.toml", ARBITRATION_LAWS.as_bytes()),
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
        let mut summaries = vec![];
        let start = Instant::now();
        for policy in [BirthPolicy::Global, BirthPolicy::Local] {
            for topology in [OfferTopology::Disjoint, OfferTopology::Competing] {
                for rotation in 0..8 {
                    for reversed in [false, true] {
                        if start.elapsed().as_secs() >= 30 {
                            return Err("arbitration advancing wall ceiling".into());
                        }
                        let recipe = ArbitrationRecipe {
                            policy,
                            topology,
                            rotation,
                            reversed,
                        };
                        let sub = dir.join(recipe.name());
                        fs::create_dir(&sub).map_err(|e| e.to_string())?;
                        let mut cell = meta(&recipe.name(), 4)
                            .create(&sub)
                            .map_err(|e| e.to_string())?;
                        cell.start().map_err(|e| e.to_string())?;
                        let result = run_arbitration(recipe.clone());
                        let mut status = flags(result.is_ok(), result.as_ref().err());
                        status["recipe"] = serde_json::to_value(&recipe).unwrap();
                        if let Ok(r) = &result {
                            put(&sub.join("result.json"), &json(r)?)?;
                            status["accepted_parents"] =
                                serde_json::to_value(&r.accepted_parents).unwrap();
                            cell.seal_completed(4).map_err(|e| e.to_string())?;
                        } else {
                            cell.seal_interrupted(0).map_err(|e| e.to_string())?;
                        }
                        put(&sub.join("run_status.json"), &json(&status)?)?;
                        result?;
                        summaries.push(status);
                        done += 4;
                        m.final_step = done;
                        m.persist().map_err(|e| e.to_string())?;
                        if size(dir)? > 16 * 1024 * 1024 - 1024 * 1024 {
                            return Err("arbitration output ceiling".into());
                        }
                    }
                }
            }
        }
        put(&dir.join("summary.json"), &json(&summaries)?)?;
        put(&dir.join("report.md"),b"# Bounded birth arbitration diagnostic\n\nGlobal counter: one birth for disjoint offers; local prototype: two. Competing offers: one. Full fixed rotations/allocation orders remain in summary.json. Main ecology is unchanged; reproductive fairness is not accepted.\n")?;
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
