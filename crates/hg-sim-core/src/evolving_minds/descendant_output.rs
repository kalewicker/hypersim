//! Fixed paid supplied-edit descendant diagnostic; no scientific admission.
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
pub const DESCENDANT_PANEL_PROFILE: &str =
    "evolving-minds-d0-paid-descendant-neighborhood-development-v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DescendantCase {
    pub name: String,
    pub arm: String,
    pub config: FundedAdmissionConfig,
    pub descendant: DescendantConfig,
    pub fault: StreamFault,
}
pub fn descendant_cases() -> Vec<DescendantCase> {
    let mut rows = vec![];
    let mut push = |seed, target_id, new_source, fault| {
        let arm = match (target_id, new_source) {
            (Some(id), Some(source)) => format!("g{id}_s{source}"),
            _ => "unchanged".into(),
        };
        rows.push(DescendantCase {
            name: format!("world{seed}_{arm}"),
            arm,
            config: FundedAdmissionConfig {
                seed,
                endowment: 24,
                founders: [FundedProgram::Delayed; 2],
                kernels: [MutationKernel::Faithful; 2],
                home_offsets: [0, 4],
                rotation: 0,
                reversed: false,
                horizon: 64,
                scheduler: "semantic-lottery-v1".into(),
            },
            descendant: DescendantConfig {
                policy: DESCENDANT_POLICY.into(),
                target_id,
                new_source,
            },
            fault,
        });
    };
    for seed in [601, 607] {
        push(seed, None, None, StreamFault::Normal);
        for id in 100..=104 {
            for source in 1..=7 {
                push(seed, Some(id), Some(source), StreamFault::Normal);
            }
        }
    }
    push(17, Some(103), Some(1), StreamFault::CleanStop);
    rows.last_mut().unwrap().name = "world17_recovery".into();
    rows.last_mut().unwrap().arm = "recovery".into();
    rows
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
    serde_json::json!({"profile":DESCENDANT_PANEL_PROFILE,"engineering_execution_complete":ok,"development_only":true,"scientific_run_launched":false,"gate_pass":false,"training_admitted":false,"evolution_enabled":false,"reproductive_fairness_accepted":false,"scientific_acceptance":false,"sustained_claim":false,"power_adequacy_established":false,"ready_for_scientific_launch":false})
}
pub fn write_descendant_panel(p: &Path) -> Result<(), String> {
    fs::create_dir(p).map_err(|e| format!("exclusive descendant development panel: {e}"))?;
    let mut m = RunManifest::default();
    m.run_id = "paid_descendant_development_v1".into();
    m.experiment_name = DESCENDANT_PANEL_PROFILE.into();
    m.seed = 601;
    m.max_steps = 73 * 2944;
    m.thread_count = rayon::current_num_threads();
    m.step_mode = "73 fixed diagnostic cells; one fresh-process recovery;64ticks each".into();
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
            return Err("descendant pilot requires four workers".into());
        }
        for (name, b) in [
            (
                "checker.py",
                include_bytes!("../../../../scripts/verify_evolving_minds_descendant.py").as_slice(),
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
                    "../../../../experiments/evolving_minds/contracts/d0_paid_descendant_neighborhood_v1.md"
                )
                .as_slice(),
            ),
            (
                "shared_replay.py",
                include_bytes!("../../../../scripts/evolving_minds_shared_replay.py").as_slice(),
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
        let cases = descendant_cases();
        put(&p.join("registered_cases.json"), &json(&cases)?)?;
        let started = Instant::now();
        let mut reports = vec![];
        let check = || -> Result<(), String> {
            if started.elapsed().as_secs() >= 3600
                || size(p)? > 512 * 1024 * 1024 - 16 * 1024 * 1024
            {
                Err("descendant development panel cap".into())
            } else {
                Ok(())
            }
        };
        for case in cases {
            check()?;
            let r = write_descendant_stream_segment(
                &p.join(&case.name),
                case.config,
                case.descendant,
                case.fault,
            )?;
            if case.arm == "recovery" {
                if r.state != "Interrupted" || r.committed_final_tick != 24 {
                    return Err("descendant recovery cut".into());
                }
                let exit =
                    std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                        .arg("descendant-resume")
                        .arg(p.join(&case.name))
                        .arg(p.join("world17_recovery_resumed"))
                        .status()
                        .map_err(|e| e.to_string())?;
                if !exit.success() {
                    return Err("fresh-process descendant recovery".into());
                }
                let resumed: SegmentReport = serde_json::from_slice(
                    &fs::read(p.join("world17_recovery_resumed/segment_report.json"))
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                if resumed.state != "Completed" || resumed.committed_final_tick != 64 {
                    return Err("descendant resumed horizon".into());
                }
                reports.push(resumed);
            } else if r.state != "Completed" || r.native_final_tick != 64 || r.start_tick != 0 {
                return Err("descendant full horizon".into());
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
            return Err("descendant summed advancement cap".into());
        }
        put(&p.join("summary.json"), &json(&reports)?)?;
        put(
            &p.join("execution_counts.json"),
            &json(
                &serde_json::json!({"fresh_worlds":2,"main_arms_per_world":36,"initial_cells":73,"segments":74,"ticks":4672,"native_advances_executed":214912}),
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
