//! EXP-L9b (EXPLORATORY, 2026-07-15): depth-capped subdivision — the
//! corrected mechanism discriminator after EXP-L9 found the
//! pre-registered x-cap structurally inert (guard never binds; all
//! cells identical to the ∞-control; see leakage_experiment_findings.md
//! before reading these numbers).
//!
//! Delta vs EXP-L9: rules/lk_pair_depth_capped.toml (guard on y,
//! inheritance from y — generation counts filament depth). Same
//! reference cell and seeds.
//!
//! Thresholds FROZEN here, before the runs: per (g_max, seed) cell vs
//! its same-seed ∞-control —
//!   collapse    = final wormhole_count_est < 5% of control AND
//!                 chain_length_max ≤ g_max
//!   persistence = final wormhole_count_est ≥ 50% of control
//!   partial     = anything else
//! The ∞-control itself is the EXP-L9 ginf cell (identical dynamics,
//! rerun here for same-binary hygiene).

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn l9b_config(dir: &Path, seed: u64, g_max: f64) -> SimConfig {
    let rules = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_depth_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        &format!("property = 6\nop       = \"lt\"\nvalue    = {g_max:.1}"),
    );
    let era_est = (400_000.0f64 / 60.0) as u64;
    let toml = format!(
        r#"
experiment_name = "exp_l9b"
seed = {seed}
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = {}
metrics_window_interval = 500
thread_count = 1
[lk_rules]
enabled = true
e_total = 400000.0
g_max = {g_max:.1}
[leakage]
enabled = true
cadence_steps = 200
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
        era_est + 400,
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules);
    cfg
}

fn run_cell(dir: &Path, seed: u64, g_max: f64) -> (f64, bool, f64, u32, u32, String) {
    let cfg = l9b_config(dir, seed, g_max);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<String>> = leakage
        .lines()
        .skip(2)
        .map(|l| l.split(',').map(str::to_string).collect())
        .collect();
    let last = rows.last().unwrap();
    let n_sub: f64 = last[15].parse().unwrap();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
    let final_count: f64 = last[10].parse().unwrap_or(0.0);
    let mut phases: std::collections::BTreeMap<String, usize> = Default::default();
    for r in &rows {
        *phases.entry(r[12].clone()).or_default() += 1;
    }
    let chain_rows = hg_eval::chains::census_run_dir(&artifacts.run_dir).expect("census");
    let lastc = chain_rows.last().cloned().unwrap_or_default();
    (
        n_sub,
        audit_ok,
        final_count,
        lastc.chains_count,
        lastc.chain_length_max,
        format!("{phases:?}"),
    )
}

#[test]
#[ignore]
fn exp_l9b_depth_cap_sweep() {
    let persist_root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!("cell | seed | era_subs | audit | final_wh_count | chains n / len_max | vs_control | phases");
    for seed in 42u64..47 {
        // Same-seed ∞-control first (denominator for the thresholds).
        let dirc = persist_root
            .as_ref()
            .map(|r| {
                let p = std::path::Path::new(r).join(format!("dinf_s{seed}"));
                std::fs::create_dir_all(&p).unwrap();
                p
            })
            .unwrap_or_else(|| tmp.path().join(format!("dinf_s{seed}")));
        let (c_subs, c_audit, c_count, c_chains, c_lenmax, c_phases) =
            run_cell(&dirc, seed, 1.0e18);
        println!(
            "dinf_s{seed} | {seed} | {c_subs} | {} | {c_count} | {c_chains} / {c_lenmax} | (control) | {c_phases}",
            if c_audit { "exact" } else { "DRIFT" },
        );
        for &g_max in &[1.0f64, 2.0, 4.0, 8.0] {
            let cell = format!("d{g_max:.0}_s{seed}");
            let dir = persist_root
                .as_ref()
                .map(|r| {
                    let p = std::path::Path::new(r).join(&cell);
                    std::fs::create_dir_all(&p).unwrap();
                    p
                })
                .unwrap_or_else(|| tmp.path().join(&cell));
            let (subs, audit, count, chains, lenmax, phases) = run_cell(&dir, seed, g_max);
            let ratio = count / c_count.max(1.0);
            let verdict = if ratio < 0.05 && (lenmax as f64) <= g_max {
                "COLLAPSE"
            } else if ratio >= 0.5 {
                "PERSISTENCE"
            } else {
                "PARTIAL"
            };
            println!(
                "{cell} | {seed} | {subs} | {} | {count} | {chains} / {lenmax} | {ratio:.4} {verdict} | {phases}",
                if audit { "exact" } else { "DRIFT" },
            );
        }
    }
}
