use super::{admission::ADMISSION_LAWS, retained::*, source_inventory::SOURCES, stream_output::*};
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
fn flags(ok: bool) -> serde_json::Value {
    serde_json::json!({"profile":RETAINED_STREAM_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false})
}
pub fn write_retained_panel(p: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive retained panel: {e}"))?;
    let mut m = RunManifest::default();
    m.run_id = "retained_first_proposal_integration".into();
    m.experiment_name = RETAINED_STREAM_PROFILE.into();
    m.seed = 17;
    m.max_steps = 7 * 2944;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "7 complete committed chains; actual native tails counted separately".into();
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
        for (name, b) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_retained.py").as_slice(),
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
                "opportunity_reference.py",
                include_bytes!("../../../../scripts/evolving_minds_opportunity.py").as_slice(),
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
            (
                "contract.md",
                include_bytes!(
                    "../../../../experiments/evolving_minds/contracts/d0_retained_proposal_v1.md"
                )
                .as_slice(),
            ),
            ("base_rules.toml", ADMISSION_LAWS.as_bytes()),
        ] {
            put(&p.join(name), b)?;
        }
        put(
            &p.join("sources.json"),
            &json(&SOURCES.iter().copied().collect::<BTreeMap<_, _>>())?,
        )?;
        put(
            &p.join("b24_base_rules.toml"),
            super::funding::funded_text(ADMISSION_LAWS, 24)?.as_bytes(),
        )?;
        let cases = retained_cases();
        put(&p.join("registered_cases.json"), &json(&cases)?)?;
        let started = Instant::now();
        let check = || -> Result<(), String> {
            if started.elapsed().as_secs() >= 1800 || size(p)? > 256 * 1024 * 1024 - 1024 * 1024 {
                Err("retained panel cap".into())
            } else {
                Ok(())
            }
        };
        let mut reports = vec![];
        for case in &cases {
            check()?;
            let r = write_retained_stream_segment(
                &p.join(format!("baseline_{}", case.name)),
                case.config.clone(),
                StreamFault::Normal,
            )?;
            if r.state != "Completed" {
                return Err("unexpected retained baseline interruption".into());
            }
            reports.push(r);
            m.final_step += 2944;
            m.persist().map_err(|e| e.to_string())?;
        }
        let cfg = cases[3].config.clone();
        let prefix = p.join("fault_before_commit");
        check()?;
        let r = write_retained_stream_segment(&prefix, cfg, StreamFault::BeforeCommit)?;
        if r.state != "Interrupted" || r.committed_final_tick != 8 || r.native_final_tick != 16 {
            return Err("retained fixed cut".into());
        }
        reports.push(r);
        let r = resume_stream_segment(&prefix, &p.join("recovered_before_commit"))?;
        if r.state != "Completed" || r.start_tick != 8 || r.committed_final_tick != 64 {
            return Err("retained recovery boundary".into());
        }
        reports.push(r);
        m.final_step += 2944;
        check()?;
        let advances: u64 = reports
            .iter()
            .map(|r| (r.native_final_tick - r.start_tick) * 46)
            .sum();
        put(
            &p.join("execution_counts.json"),
            &json(
                &serde_json::json!({"segments":8,"complete_chains":7,"complete_committed_chain_advances":20608,"native_advances_executed":advances,"unpublished_or_repeated_tail_advances":advances-20608}),
            )?,
        )?;
        put(&p.join("summary.json"), &json(&reports)?)?;
        put(&p.join("run_status.json"), &json(&flags(true))?)?;
        m.seal_completed(m.final_step).map_err(|e| e.to_string())
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
