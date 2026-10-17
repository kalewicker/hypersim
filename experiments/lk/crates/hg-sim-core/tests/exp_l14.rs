//! EXP-L14 (v4 F3): the removal experiment — the first well-posed
//! instance of the original EXP-L2 question. With subdivide (production)
//! and fuse (removal) both present: does the emergent-wormhole
//! population decay, hold steady, or grow?
//!
//! Arms: {pump, swap} × {clean, planted} × 3 seeds, 32², E₀ = 4e5,
//! fuse enabled, run well past quench. (First pass ran at 16²/6e4
//! because the pre-optimization matcher scanned all edge pairs for the
//! fuse wedge — O(E²)/step; adjacency-guided matching restored the
//! original scale. The 16² results are recorded in the findings doc
//! alongside these.) Planted handles are untouchable
//! by design (§1.4, generation guard) and serve as the inert reference
//! population. The conserved object is the HANDLE census
//! (tracked_components): planted edges may legitimately be refined by
//! subdivision and reabsorbed by fuse (lineage closure follows both),
//! but a component disappearing would mean fuse removed a handle — the
//! generation-guard bug, asserted here.
//!
//! Pre-registered classification (frozen before the runs), on the
//! post-peak wormhole_count_est trajectory:
//!   decay  = final < 0.05 × peak
//!   growth = final ≥ 0.95 × peak AND still rising over the last quarter
//!   steady = anything else with final > 0 (a production/removal balance)
//! The steady row authorizes EXP-L7R on its stratum.

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

fn l14_config(dir: &Path, seed: u64, priority: &str, planted: bool) -> SimConfig {
    // Pump era = E₀/(q+c) ≈ 6666 steps; the swap regime needs its
    // quiescent diffusion era (O(E₀/τ)) before growth starts.
    let max_steps: u64 = if priority == "transport_first" { 30000 } else { 12000 };
    let rules = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        "property = 6\nop       = \"lt\"\nvalue    = 1.0e18",
    );
    let toml = format!(
        r#"
experiment_name = "exp_l14"
seed = {seed}
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
defect_density = {}
[engine]
step_mode = "MaxParallel"
max_steps = {}
metrics_window_interval = 500
thread_count = 1
[lk_rules]
enabled = true
e_total = 400000.0
g_max = 1.0e18
priority = "{priority}"
[lk_rules.fuse]
enabled = true
[leakage]
enabled = true
cadence_steps = 100
[leakage.tracked]
enabled = {}
recompute_cadence = 100
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
        if planted { 0.005 } else { 0.0 },
        max_steps,
        planted,
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules);
    cfg
}

#[test]
#[ignore]
fn exp_l14_removal_experiment() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!("cell | n_sub | audit | e_fused | churn | wh peak->final | verdict | sparse_w | chains(final) n/max | tracked_const");
    for priority in ["subdivide_first", "transport_first"] {
        for planted in [false, true] {
            for seed in 42u64..45 {
                let cell = format!(
                    "{}_{}_s{seed}",
                    if priority == "subdivide_first" { "pump" } else { "swap" },
                    if planted { "planted" } else { "clean" },
                );
                let dir = root
                    .as_ref()
                    .map(|r| {
                        let p = Path::new(r).join(&cell);
                        std::fs::create_dir_all(&p).unwrap();
                        p
                    })
                    .unwrap_or_else(|| tmp.path().join(&cell));
                let cfg = l14_config(&dir, seed, priority, planted);
                let artifacts = run_headless(&cfg, None, None, None).expect("run");
                let leakage =
                    std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
                let rows: Vec<Vec<&str>> =
                    leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
                let last = rows.last().unwrap();
                let n_sub: f64 = last[15].parse().unwrap();
                let e_fused: f64 = last[16].parse().unwrap();
                let churn: u32 = last[17].parse().unwrap_or(0);
                let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
                let sparse_w = rows.iter().filter(|r| r[12] == "sparse").count();
                let counts: Vec<f64> = rows
                    .iter()
                    .filter_map(|r| r[10].parse().ok())
                    .collect();
                let peak = counts.iter().copied().fold(0.0f64, f64::max);
                let final_c = *counts.last().unwrap_or(&0.0);
                let last_quarter = &counts[counts.len() - counts.len() / 4..];
                let rising = last_quarter.first().is_some_and(|&a| final_c > a);
                let verdict = if peak > 0.0 && final_c < 0.05 * peak {
                    "DECAY"
                } else if final_c >= 0.95 * peak && rising {
                    "GROWTH"
                } else if final_c > 0.0 {
                    "STEADY"
                } else {
                    "EMPTY"
                };
                // Chain census on the final snapshot.
                let snaps =
                    hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
                let census = snaps
                    .last()
                    .map(|(_, s)| hg_eval::chains::chain_census(s))
                    .unwrap_or_default();
                // Planted arms: the handle census (components) must be
                // conserved — refinement/reabsorption may change edge
                // ids, removal may not happen.
                let tracked_const = if planted {
                    let tracked =
                        std::fs::read_to_string(artifacts.run_dir.join("tracked.csv")).unwrap();
                    let comps: Vec<u32> = tracked
                        .lines()
                        .skip(2)
                        .map(|l| l.split(',').nth(3).unwrap().parse().unwrap())
                        .collect();
                    let constant = comps.windows(2).all(|w| w[0] == w[1]);
                    assert!(
                        constant && comps.first().is_some_and(|&c| c > 0),
                        "{cell}: tracked HANDLE census changed under fuse —                          generation-guard bug: {comps:?}"
                    );
                    "yes"
                } else {
                    "-"
                };
                println!(
                    "{cell} | {n_sub} | {} | {e_fused:.0} | {churn} | {peak:.0}->{final_c:.0} | {verdict} | {sparse_w} | {}/{} | {tracked_const}",
                    if audit_ok { "exact" } else { "DRIFT" },
                    census.chains_count,
                    census.chain_length_max,
                );
            }
        }
    }
}
