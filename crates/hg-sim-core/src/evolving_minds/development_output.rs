//! Fixed matched development pilot on existing funded and retained native profiles.
use super::{admission::ADMISSION_LAWS, source_inventory::SOURCES, *};
use hg_output::RunManifest;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
pub const DEVELOPMENT_PROFILE: &str = "evolving-minds-d0-matched-development-v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DevelopmentCase {
    pub name: String,
    pub arm: String,
    pub config: FundedAdmissionConfig,
}
pub fn development_cases() -> Vec<DevelopmentCase> {
    let mut cases = vec![];
    for seed in [101, 103, 107, 109] {
        for (arm, program, kernel) in [
            (
                "inherited",
                FundedProgram::Delayed,
                MutationKernel::SourceKernel,
            ),
            ("faithful", FundedProgram::Delayed, MutationKernel::Faithful),
            (
                "retained",
                FundedProgram::Delayed,
                MutationKernel::SourceKernel,
            ),
            (
                "immediate",
                FundedProgram::Immediate,
                MutationKernel::Faithful,
            ),
            ("coin", FundedProgram::Coin, MutationKernel::Faithful),
        ] {
            cases.push(DevelopmentCase {
                name: format!("world{seed}_{arm}"),
                arm: arm.into(),
                config: FundedAdmissionConfig {
                    seed,
                    endowment: 24,
                    founders: [program; 2],
                    kernels: [kernel; 2],
                    home_offsets: [0, 4],
                    rotation: 0,
                    reversed: false,
                    horizon: 64,
                    scheduler: "semantic-lottery-v1".into(),
                },
            });
        }
    }
    cases
}
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
    serde_json::json!({"profile":DEVELOPMENT_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false})
}
pub fn write_development_panel(p: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive matched development panel: {e}"))?;
    let mut m = RunManifest::default();
    m.run_id = "matched_five_arm_development_v1".into();
    m.experiment_name = DEVELOPMENT_PROFILE.into();
    m.seed = 101;
    m.max_steps = 20 * 2944;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "20 fixed development cells, each64ticks; no held-out worlds".into();
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
            return Err("matched pilot requires four workers".into());
        }
        for (name, b) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_development.py")
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
                    "../../../../experiments/evolving_minds/contracts/d0_matched_development_v1.md"
                )
                .as_slice(),
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
        let cases = development_cases();
        put(&p.join("registered_cases.json"), &json(&cases)?)?;
        let started = Instant::now();
        let mut reports = vec![];
        let check = || -> Result<(), String> {
            if started.elapsed().as_secs() >= 3600
                || size(p)? > 1024 * 1024 * 1024 - 16 * 1024 * 1024
            {
                Err("matched development panel cap".into())
            } else {
                Ok(())
            }
        };
        for case in cases {
            check()?;
            let r = if case.arm == "retained" {
                write_retained_stream_segment(
                    &p.join(&case.name),
                    case.config,
                    StreamFault::Normal,
                )?
            } else {
                write_funded_stream_segment(&p.join(&case.name), case.config, StreamFault::Normal)?
            };
            if r.state != "Completed" || r.native_final_tick != 64 || r.start_tick != 0 {
                return Err("unexpected matched cell interruption".into());
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
            return Err("matched summed advancement cap".into());
        }
        put(&p.join("summary.json"), &json(&reports)?)?;
        put(
            &p.join("execution_counts.json"),
            &json(
                &serde_json::json!({"worlds":4,"arms_per_world":5,"cells":20,"ticks":1280,"native_advances_executed":58880}),
            )?,
        )?;
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
