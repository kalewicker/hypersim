//! EXP-L10 (plan v3 §4): the designed encounter — V3 (conduit
//! amplification) by construction. F-LK7 recorded a NON-encounter (the
//! front never reached the planted set in 260 windows); here the seed
//! is placed ON a tracked mouth, so transport reaches the handle at
//! step 1 by construction.
//!
//! Arms: {at_tracked_mouth, matched_offset} × {uncapped,
//! depth-cap g=1} × seeds {42..46}, EXP-L2 reference geometry
//! (48² TriLattice, p = 0.005, E₀ = 1.5e6, 26k steps). The capped arm
//! uses the EXP-L9b depth-cap instrument (g_max = 1, the clean-
//! background arm) because the pre-registered x-cap is inert — see
//! leakage_experiment_findings.md; only-interpretable-if-partial
//! condition is met (EXP-L9b landed partial).
//!
//! Any arm where far_nucleus fires is rerun with swapped rule
//! priorities (hazard 4): a far-nucleus claim that does not survive the
//! swap is a tie-break artifact.
//!
//! Run with --ignored; set HYPERSIM_SWEEP_OUT to persist run dirs.

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn rules_toml(capped: bool, swap_priorities: bool) -> String {
    let path = if capped {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../rules/lk_pair_depth_capped.toml"
        )
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../rules/lk_pair.toml")
    };
    let mut rules = std::fs::read_to_string(path).unwrap();
    if capped {
        rules = rules.replace(
            "property = 6\nop       = \"lt\"\nvalue    = 4.0",
            "property = 6\nop       = \"lt\"\nvalue    = 1.0",
        );
    }
    if swap_priorities {
        rules = rules
            .replace("priority = 1", "priority = TMP")
            .replace("priority = 0", "priority = 1")
            .replace("priority = TMP", "priority = 0");
    }
    rules
}

fn l10_config(
    dir: &Path,
    seed: u64,
    placement: &str,
    capped: bool,
    swap: bool,
) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp_l10"
seed = {seed}
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 48
defect_density = 0.005
[engine]
step_mode = "MaxParallel"
max_steps = 26000
metrics_window_interval = 100
thread_count = 3
[lk_rules]
enabled = true
e_total = 1500000.0
seed_placement = "{placement}"
{}
[leakage]
enabled = true
cadence_steps = 100
detour_sample_edges = 1024
[leakage.tracked]
enabled = true
recompute_cadence = 100
[output]
fixed_provenance = true
"#,
        dir.display(),
        if capped { "g_max = 1.0" } else { "" },
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules_toml(capped, swap));
    cfg
}

struct ArmResult {
    mouth_energy_max: f32,
    t_far_nucleus: Option<u64>,
    t_ambient_front_at_far: Option<u64>,
    audit_ok: bool,
}

fn run_arm(dir: &Path, seed: u64, placement: &str, capped: bool, swap: bool) -> ArmResult {
    let cfg = l10_config(dir, seed, placement, capped, swap);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let tracked = std::fs::read_to_string(artifacts.run_dir.join("tracked.csv")).unwrap();
    let last: Vec<&str> = tracked.lines().last().unwrap().split(',').collect();
    // Columns: 9 = mouth_energy_max, 10 = t_far_nucleus, 11 = t_ambient_front_at_far.
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let audit_ok = leakage
        .lines()
        .skip(2)
        .all(|l| l.split(',').nth(18).unwrap().parse::<f64>().unwrap() == 0.0);
    ArmResult {
        mouth_energy_max: last[9].parse().unwrap_or(f32::NAN),
        t_far_nucleus: last[10].parse().ok(),
        t_ambient_front_at_far: last[11].parse().ok(),
        audit_ok,
    }
}

#[test]
#[ignore]
fn exp_l10_designed_encounter() {
    let persist_root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let dir_for = |cell: &str| -> std::path::PathBuf {
        persist_root
            .as_ref()
            .map(|r| {
                let p = std::path::Path::new(r).join(cell);
                std::fs::create_dir_all(&p).unwrap();
                p
            })
            .unwrap_or_else(|| tmp.path().join(cell))
    };
    println!("cell | seed | audit | mouth_E_max (θ=100) | t_far_nucleus | t_ambient_at_far | swap_verdict");
    for &capped in &[false, true] {
        for placement in ["at_tracked_mouth", "matched_offset"] {
            for seed in 42u64..47 {
                let cell = format!(
                    "{}_{}_s{seed}",
                    if capped { "g1" } else { "uncapped" },
                    if placement == "at_tracked_mouth" { "mouth" } else { "offset" },
                );
                let r = run_arm(&dir_for(&cell), seed, placement, capped, false);
                // Hazard-4 exclusion: rerun with swapped priorities
                // wherever far_nucleus fired.
                let swap_verdict = if r.t_far_nucleus.is_some() {
                    let r2 = run_arm(
                        &dir_for(&format!("{cell}_swap")),
                        seed,
                        placement,
                        capped,
                        true,
                    );
                    if r2.t_far_nucleus.is_some() {
                        "SURVIVES_SWAP"
                    } else {
                        "TIEBREAK_ARTIFACT"
                    }
                } else {
                    "-"
                };
                println!(
                    "{cell} | {seed} | {} | {:.1} | {:?} | {:?} | {swap_verdict}",
                    if r.audit_ok { "exact" } else { "DRIFT" },
                    r.mouth_energy_max,
                    r.t_far_nucleus,
                    r.t_ambient_front_at_far,
                );
            }
        }
    }
}
