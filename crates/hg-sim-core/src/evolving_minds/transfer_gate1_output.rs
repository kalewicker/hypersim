//! TA-1 Gate1 fixed development assay on the unchanged handoff lifecycle.
use super::{admission::ADMISSION_LAWS, source_inventory::SOURCES, *};
use hg_output::RunManifest;
use std::{
    collections::BTreeMap,
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
fn json<T: serde::Serialize>(x: &T) -> Result<Vec<u8>, String> {
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
        };
    }
    Ok(n)
}
fn seal_files(root: &Path, p: &Path, hashes: &mut BTreeMap<String, String>) -> Result<(), String> {
    for entry in fs::read_dir(p).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            seal_files(root, &path, hashes)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned();
            hashes.insert(
                relative,
                stream::sha256(&fs::read(path).map_err(|e| e.to_string())?),
            );
        }
    }
    Ok(())
}
fn flags(ok: bool) -> serde_json::Value {
    serde_json::json!({"profile":TRANSFER_GATE1_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false,"transfer_established":false,"discovery_established":false,"cumulative_reuse_established":false})
}
pub fn write_transfer_gate1(p: &Path, prior: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive TA Gate1 panel: {e}"))?;
    let mut m = RunManifest::default();
    m.run_id = "ta1_gate1_development_v2".into();
    m.experiment_name = TRANSFER_GATE1_PROFILE.into();
    m.seed = 17;
    m.max_steps = 13 * 2944;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "13 development cells; one fresh-process recovery;64ticks each".into();
    m.build_features = vec!["evolving-minds".into()];
    m.target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    m.rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|x| x.status.success())
        .map(|x| String::from_utf8_lossy(&x.stdout).trim().into())
        .unwrap_or_default();
    let mut m = m.create(p).map_err(|e| e.to_string())?;
    m.start().map_err(|e| e.to_string())?;
    let result: Result<(), String> = (|| {
        if rayon::current_num_threads() != 4 {
            return Err("TA Gate1 requires four workers".into());
        }
        for (name, b) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_transfer_gate1.py")
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
                "opportunity_reference.py",
                include_bytes!("../../../../scripts/evolving_minds_opportunity.py").as_slice(),
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
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/ta1_gate1_development_v2.md"
                )
                .as_slice(),
            ),
            (
                "shared_replay.py",
                include_bytes!("../../../../scripts/evolving_minds_shared_replay.py").as_slice(),
            ),
            (
                "gate0_reference.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_transfer.py").as_slice(),
            ),
            ("base_rules.toml", ADMISSION_LAWS.as_bytes()),
        ] {
            put(&p.join(name), b)?;
        }
        put(
            &p.join("b24_base_rules.toml"),
            super::funding::funded_text(ADMISSION_LAWS, 24)?.as_bytes(),
        )?;
        put(
            &p.join("sources.json"),
            &json(&SOURCES.iter().copied().collect::<BTreeMap<_, _>>())?,
        )?;
        let original: serde_json::Value = serde_json::from_slice(
            &fs::read(prior.join("manifest.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if original["state"] != "Interrupted"
            || original["experiment_name"] != "evolving-minds-ta1-gate1-development-v1"
            || original["final_step"] != 35328
        {
            return Err("TA Gate1 v2 requires preserved declared v1 interruption".into());
        }
        let mut prior_files = BTreeMap::new();
        seal_files(prior, prior, &mut prior_files)?;
        put(
            &p.join("prior_attempt.json"),
            &json(
                &serde_json::json!({"directory":prior.canonicalize().map_err(|e|e.to_string())?,"files_sha256":prior_files,"native_advances":35696,"combined_advances_with_v2":73968,"independent_replicates_added":0}),
            )?,
        )?;
        let registration = transfer_gate1_registration()?;
        put(&p.join("registration.json"), &json(&registration)?)?;
        let binary = fs::read(std::env::current_exe().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        put(
            &p.join("binary_sha256.txt"),
            stream::sha256(&binary).as_bytes(),
        )?;
        let cases = transfer_gate1_cases()?;
        put(
            &p.join("evaluation_registration.json"),
            &json(&transfer_gate1_evaluation_registration()?)?,
        )?;
        put(&p.join("registered_cases.json"), &json(&cases)?)?;
        let started = Instant::now();
        let mut reports = vec![];
        let check = || -> Result<(), String> {
            if started.elapsed().as_secs() >= 3600
                || size(p)? > 512 * 1024 * 1024 - 16 * 1024 * 1024
            {
                Err("handoff development panel cap".into())
            } else {
                Ok(())
            }
        };
        for case in cases {
            check()?;
            let r = write_transfer_stream_segment(
                &p.join(&case.name),
                case.config,
                case.environment,
                case.fault,
            )?;
            if case.arm == "recovery" {
                if r.state != "Interrupted" || r.committed_final_tick != 24 {
                    return Err("handoff recovery cut".into());
                }
                let exit =
                    std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                        .arg("stream-resume")
                        .arg(p.join(&case.name))
                        .arg(p.join("world17_recovery_resumed"))
                        .status()
                        .map_err(|e| e.to_string())?;
                if !exit.success() {
                    return Err("fresh-process handoff recovery".into());
                }
                let resumed: SegmentReport = serde_json::from_slice(
                    &fs::read(p.join("world17_recovery_resumed/segment_report.json"))
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                if resumed.state != "Completed" || resumed.committed_final_tick != 64 {
                    return Err("handoff resumed horizon".into());
                }
                reports.push(resumed);
            } else if r.state != "Completed" || r.native_final_tick != 64 || r.start_tick != 0 {
                return Err("handoff full horizon".into());
            }
            reports.push(r);
            m.final_step += 2944;
            m.persist().map_err(|e| e.to_string())?;
        }
        check()?;
        if reports
            .iter()
            .map(|r| r.resources.advance_seconds)
            .sum::<f64>()
            > 2400.0
        {
            return Err("handoff summed advancement cap".into());
        }
        put(&p.join("summary.json"), &json(&reports)?)?;
        put(
            &p.join("execution_counts.json"),
            &json(&registration["counts"])?,
        )?;
        put(&p.join("run_status.json"), &json(&flags(true))?)?;
        m.seal_completed(m.final_step).map_err(|e| e.to_string())?;
        let mut hashes = BTreeMap::new();
        seal_files(p, p, &mut hashes)?;
        put(
            &p.join("output_sha256.json"),
            &json(&serde_json::json!({
                "profile":TRANSFER_GATE1_PROFILE,"files_sha256":hashes,
                "development_only":true,"scientific_run_launched":false
            }))?,
        )?;
        if size(p)? > 512 * 1024 * 1024 {
            return Err("TA Gate1 final archive cap".into());
        }
        Ok(())
    })();
    if let Err(error) = &result {
        m.seal_interrupted(m.final_step)
            .map_err(|e| e.to_string())?;
        if !p.join("run_status.json").exists() {
            put(&p.join("run_status.json"), &json(&flags(false))?)?;
        }
        put(&p.join("failure.txt"), error.as_bytes())?;
    }
    result
}
