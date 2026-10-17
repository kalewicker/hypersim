use super::{
    admission::ADMISSION_LAWS, funded_admission::*, source_inventory::SOURCES, stream::sha256,
    stream_output::*,
};
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
fn json<T: Serialize>(x: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(x).map_err(|e| e.to_string())
}
fn size(p: &Path) -> Result<u64, String> {
    let mut n = 0;
    for e in fs::read_dir(p).map_err(|e| e.to_string())? {
        let p = e.map_err(|e| e.to_string())?.path();
        n += if p.is_dir() {
            size(&p)?
        } else {
            p.metadata().map_err(|e| e.to_string())?.len()
        }
    }
    Ok(n)
}
fn flags(ok: bool, reason: Option<&str>) -> serde_json::Value {
    serde_json::json!({"profile":FUNDED_STREAM_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false,"stop_reason":reason})
}
pub fn write_funded_admission_panel(p: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive funded admission directory: {e}"))?;
    let mut m = RunManifest::default();
    m.run_id = "funded_admission_integration".into();
    m.experiment_name = FUNDED_STREAM_PROFILE.into();
    m.seed = 17;
    m.max_steps = 16 * 2944;
    m.thread_count = rayon::current_num_threads();
    m.step_mode="16 complete committed chains; 46 native passes/tick; unpublished native tails counted separately".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m.rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let mut m = m.create(p).map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result: Result<(), String> = (|| {
        for (name, b) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_funded_admission.py")
                    .as_slice(),
            ),
            (
                "stream_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_streaming.py").as_slice(),
            ),
            (
                "admission_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_admission.py").as_slice(),
            ),
            (
                "funding_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_funding.py").as_slice(),
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
            ("base_rules.toml", ADMISSION_LAWS.as_bytes()),
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_funded_admission_v1.md"
                )
                .as_slice(),
            ),
        ] {
            put(&p.join(name), b)?;
        }
        let sources: std::collections::BTreeMap<_, _> = SOURCES.iter().copied().collect();
        put(&p.join("sources.json"), &json(&sources)?)?;
        put(
            &p.join("b24_base_rules.toml"),
            super::funding::funded_text(ADMISSION_LAWS, 24)?.as_bytes(),
        )?;
        let cases = funded_admission_cases();
        put(&p.join("registered_cases.json"), &json(&cases)?)?;
        let faults = [
            StreamFault::CleanStop,
            StreamFault::TornPayload,
            StreamFault::TornCheckpoint,
            StreamFault::BeforeCommit,
            StreamFault::AfterCommit,
            StreamFault::Quota,
        ];
        put(&p.join("registered_faults.json"), &json(&faults)?)?;
        let mut reports = vec![];
        let start = Instant::now();
        let check = || -> Result<(), String> {
            if start.elapsed().as_secs() >= 3600 || size(p)? > 512 * 1024 * 1024 - 1024 * 1024 {
                Err("funded panel resource ceiling".into())
            } else {
                Ok(())
            }
        };
        for case in &cases {
            check()?;
            let r = write_funded_stream_segment(
                &p.join(format!("baseline_{}", case.name)),
                case.config.clone(),
                StreamFault::Normal,
            )?;
            if r.state != "Completed" {
                return Err(format!(
                    "unexpected funded baseline interruption: {}",
                    case.name
                ));
            }
            reports.push(r);
            m.final_step += 2944;
            m.persist().map_err(|e| e.to_string())?;
        }
        let config = cases
            .iter()
            .find(|c| c.name == "mixed_reject_b24")
            .unwrap()
            .config
            .clone();
        for fault in faults {
            check()?;
            let source = p.join(format!("fault_{}", fault.name()));
            let r = write_funded_stream_segment(&source, config.clone(), fault.clone())?;
            if r.state != "Interrupted" {
                return Err("missing declared funded interruption".into());
            }
            let cut = r.committed_final_tick;
            if fault != StreamFault::Quota
                && cut
                    != if fault == StreamFault::AfterCommit {
                        16
                    } else {
                        8
                    }
            {
                return Err("funded fixed recovery cut".into());
            }
            let marker = source.join(format!("commit_{cut:04}.json"));
            let before = sha256(&fs::read(&marker).map_err(|e| e.to_string())?);
            reports.push(r);
            let r = resume_stream_segment(&source, &p.join(format!("recovered_{}", fault.name())))?;
            if r.state != "Completed"
                || r.start_tick != cut
                || before != sha256(&fs::read(&marker).map_err(|e| e.to_string())?)
            {
                return Err("funded recovery exact cut/immutable parent".into());
            }
            reports.push(r);
            m.final_step += 2944;
            m.persist().map_err(|e| e.to_string())?;
        }
        check()?;
        put(&p.join("summary.json"), &json(&reports)?)?;
        put(&p.join("run_status.json"), &json(&flags(true, None))?)?;
        let native: u64 = reports
            .iter()
            .map(|r| (r.native_final_tick - r.start_tick) * 46)
            .sum();
        put(
            &p.join("execution_counts.json"),
            &json(
                &serde_json::json!({"native_advances_executed":native,"complete_committed_chain_advances":16*2944,"complete_chains":16,"segments":22,"unpublished_or_repeated_tail_advances":native-16*2944}),
            )?,
        )?;
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
