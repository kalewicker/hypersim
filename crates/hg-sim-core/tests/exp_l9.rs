//! EXP-L9 (plan v3 §3): generation-capped subdivision sweep — the
//! mechanism discriminator for F-LK5's channelization hypothesis.
//!
//! Reference cell: TriLattice 32², θ=100/q=50/c=10/τ=25, E₀ = 4e5
//! (the P0 re-audit's G4-new-failing era length: the ∞-control must
//! reproduce the sparse→dense crossing for the sweep to discriminate).
//! Sweep: g_max ∈ {1, 2, 4, 8, ∞(=1e18)} × seeds {42..46}.
//!
//! Pre-registered outcome thresholds (frozen in the plan §3, BEFORE the
//! runs): collapse = wormhole rate < 0.05/event AND G4-new pass on ≥ 1
//! cell; persistence = rate > 0.5/event at g_max = 1; partial =
//! everything else.
//!
//! Run with --ignored; set HYPERSIM_SWEEP_OUT to persist run dirs
//! (chains.csv is written into each).

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn l9_config(dir: &Path, seed: u64, g_max: f64) -> SimConfig {
    let rules = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        &format!("property = 6\nop       = \"lt\"\nvalue    = {g_max:.1}"),
    );
    let era_est = (400_000.0f64 / 60.0) as u64;
    let toml = format!(
        r#"
experiment_name = "exp_l9"
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

struct CellResult {
    era_subs: f64,
    audit_ok: bool,
    exploding: bool,
    sparse_frac: f64,
    phases: std::collections::BTreeMap<String, usize>,
    /// Mean Δwormhole_count_est / Δn_subdivisions over era windows.
    wormhole_rate: f64,
    chains_final: (u32, f32, f32, u32),
    canonical_hash: u64,
    nodes: usize,
}

fn run_cell(dir: &Path, seed: u64, g_max: f64) -> CellResult {
    let cfg = l9_config(dir, seed, g_max);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<String>> = leakage
        .lines()
        .skip(2)
        .map(|l| l.split(',').map(str::to_string).collect())
        .collect();
    let n_sub_final: f64 = rows.last().unwrap()[15].parse().unwrap();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
    let era_rows: Vec<&Vec<String>> = rows
        .iter()
        .take_while(|r| r[15].parse::<f64>().unwrap() < n_sub_final)
        .collect();
    let mut phases: std::collections::BTreeMap<String, usize> = Default::default();
    for r in &era_rows {
        *phases.entry(r[12].clone()).or_default() += 1;
    }
    let sparse_frac =
        *phases.get("sparse").unwrap_or(&0) as f64 / era_rows.len().max(1) as f64;
    // Wormhole rate: per-window Δcount_est / Δevents, era windows only.
    let mut rates = Vec::new();
    for w in era_rows.windows(2) {
        let dc = w[1][10].parse::<f64>().unwrap_or(0.0) - w[0][10].parse::<f64>().unwrap_or(0.0);
        let de = w[1][15].parse::<f64>().unwrap() - w[0][15].parse::<f64>().unwrap();
        if de > 0.0 {
            rates.push(dc / de);
        }
    }
    let wormhole_rate = rates.iter().sum::<f64>() / rates.len().max(1) as f64;
    let metrics = std::fs::read_to_string(&artifacts.csv_path).unwrap();
    let regime_ix = metrics
        .lines()
        .nth(1)
        .unwrap()
        .split(',')
        .position(|h| h == "regime")
        .unwrap();
    let exploding = metrics
        .lines()
        .skip(2)
        .filter_map(|l| l.split(',').nth(regime_ix))
        .any(|r| r == "Exploding");
    let chain_rows = hg_eval::chains::census_run_dir(&artifacts.run_dir).expect("census");
    let last = chain_rows.last().cloned().unwrap_or_default();
    CellResult {
        era_subs: n_sub_final,
        audit_ok,
        exploding,
        sparse_frac,
        phases,
        wormhole_rate,
        chains_final: (
            last.chains_count,
            last.chain_length_p50,
            last.chain_length_p95,
            last.chain_length_max,
        ),
        canonical_hash: artifacts.canonical_hash,
        nodes: artifacts.summary.final_metrics.map(|m| m.n_nodes).unwrap_or(0),
    }
}

#[test]
#[ignore]
fn exp_l9_gmax_sweep() {
    let persist_root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!(
        "cell | seed | era_subs | audit | G1 | G4new(sparse) | phases | wh_rate/event | chains(final): n,p50,p95,max | nodes"
    );
    let mut g5_hashes: Vec<u64> = Vec::new();
    for &g_max in &[1.0f64, 2.0, 4.0, 8.0, 1.0e18] {
        for seed in 42u64..47 {
            let cell = if g_max > 1.0e17 {
                format!("ginf_s{seed}")
            } else {
                format!("g{g_max:.0}_s{seed}")
            };
            let dir = persist_root
                .as_ref()
                .map(|r| {
                    let p = std::path::Path::new(r).join(&cell);
                    std::fs::create_dir_all(&p).unwrap();
                    p
                })
                .unwrap_or_else(|| tmp.path().join(&cell));
            let r = run_cell(&dir, seed, g_max);
            println!(
                "{cell} | {seed} | {} | {} | {} | {:.2} {} | {:?} | {:.4} | {},{},{},{} | {}",
                r.era_subs,
                if r.audit_ok { "exact" } else { "DRIFT" },
                if r.exploding { "EXPLODING" } else { "ok" },
                r.sparse_frac,
                if r.sparse_frac >= 0.8 { "PASS" } else { "FAIL" },
                r.phases,
                r.wormhole_rate,
                r.chains_final.0,
                r.chains_final.1,
                r.chains_final.2,
                r.chains_final.3,
                r.nodes,
            );
            // G5 spot-check cell: g_max = 4, seed 42, run twice.
            if seed == 42 && g_max == 4.0 {
                g5_hashes.push(r.canonical_hash);
                let dir2 = tmp.path().join("g4_s42_repeat");
                std::fs::create_dir_all(&dir2).unwrap();
                let r2 = run_cell(&dir2, seed, g_max);
                g5_hashes.push(r2.canonical_hash);
            }
        }
    }
    assert_eq!(g5_hashes[0], g5_hashes[1], "G5: capped family deterministic across runs");
    println!("G5 spot-check (g4_s42 twice): identical canonical hash — pass");
}
