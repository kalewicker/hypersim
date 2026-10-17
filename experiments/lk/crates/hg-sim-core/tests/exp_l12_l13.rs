//! EXP-L12 (v4 Q1, priority swap) and EXP-L13 (v4 Q2, unfreeze cells).
//!
//! Q1: `priority = transport_first` at reference params, E₀ ∈
//! {1e4, 1e5, 1e6} × seeds {42,43,44}. The x-inheritance capped family
//! at cap ∞ (dynamics-equivalent to lk_pair; generation tracked) so the
//! chain census and the "did any non-seed node ever pay" readout (max
//! generation > 1 under x-inheritance) work.
//!
//! Q2: q > 2τ cells (q,τ) ∈ {(50,15), (75,25)} × both priorities ×
//! seeds {42,43,44}, E₀ = 4e5. Question: frozen structure or dead-zone-
//! trapped energy? Readouts: interior conduction (gen ≥ 1 node energy ≠
//! its birth q) and interior re-concentration (max generation ≥ 2 =
//! secondary subdivision = branching morphology).
//!
//! Row classifications printed per cell; interpretation in
//! leakage_experiment_findings.md. Run with --ignored;
//! HYPERSIM_SWEEP_OUT persists.

use hg_core::{HypergraphStore, NodeId, PropertyValue};
use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

const ENERGY: u32 = 1;
const GENERATION: u32 = 6;

/// x-inheritance capped rules with the guard at cap ∞, with q/τ patched
/// (rule constants: set q, pay q+c, move τ, dead zone 2τ; c = 10).
fn rules_toml(q: f64, tau: f64) -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        "property = 6\nop       = \"lt\"\nvalue    = 1.0e18",
    )
    .replace("value = 50.0\n", &format!("value = {q:.1}\n")) // w.energy set q
    .replace("value = 60.0", &format!("value = {:.1}", q + 10.0)) // x pays q+c
    .replace("offset = 50.0", &format!("offset = {:.1}", 2.0 * tau)) // dead zone
    .replace("value = 25.0", &format!("value = {tau:.1}")) // τ moved
}

fn config(
    dir: &Path,
    seed: u64,
    e_total: f64,
    max_steps: u64,
    priority: &str,
    q: f64,
    tau: f64,
) -> SimConfig {
    // Snapshot volume control: snapshots write per metrics window, so
    // long cells use a coarser window (leakage cadence stays 100 — the
    // phase/stratum count is cadence-based, not window-based).
    let window = if max_steps > 20000 { 500 } else { 100 };
    let toml = format!(
        r#"
experiment_name = "exp_l12"
seed = {seed}
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = {max_steps}
metrics_window_interval = {window}
thread_count = 1
[lk_rules]
enabled = true
e_total = {e_total}
g_max = 1.0e18
priority = "{priority}"
q = {q}
tau = {tau}
[leakage]
enabled = true
cadence_steps = 100
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules_toml(q, tau));
    cfg
}

struct CellOut {
    n_sub: f64,
    audit_ok: bool,
    sparse_windows: usize,
    phases: String,
    chains: (u32, f32, u32),
    max_gen: f64,
    /// gen ≥ 1 nodes whose energy differs from birth q (conduction).
    conducting: usize,
    gen_pos_nodes: usize,
    transport_rejects_total: u64,
    seed_paid_only: bool,
}

fn dir_for(root: &Option<String>, tmp: &Path, cell: &str) -> std::path::PathBuf {
    root.as_ref()
        .map(|r| {
            let p = Path::new(r).join(cell);
            std::fs::create_dir_all(&p).unwrap();
            p
        })
        .unwrap_or_else(|| tmp.join(cell))
}

fn run_cell(dir: &Path, seed: u64, e_total: f64, max_steps: u64, priority: &str, q: f64, tau: f64) -> CellOut {
    let cfg = config(dir, seed, e_total, max_steps, priority, q, tau);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    let n_sub: f64 = rows.last().unwrap()[15].parse().unwrap();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
    let sparse_windows = rows.iter().filter(|r| r[12] == "sparse").count();
    let mut phases: std::collections::BTreeMap<&str, usize> = Default::default();
    for r in &rows {
        *phases.entry(r[12]).or_default() += 1;
    }
    // Guard counters: total transport rejects (cmp0) over the run.
    let transport_rejects_total = std::fs::read_to_string(artifacts.run_dir.join("guards.csv"))
        .map(|g| {
            g.lines()
                .skip(2)
                .filter_map(|l| {
                    let f: Vec<&str> = l.split(',').collect();
                    (f[1] == "lk_transport" && f[2] == "cmp0")
                        .then(|| f[4].parse::<u64>().unwrap_or(0))
                })
                .sum()
        })
        .unwrap_or(0);

    // Final snapshot: census + generation/energy morphology.
    let snaps = hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
    let (_, store) = snaps.last().expect("snapshots");
    let census = hg_eval::chains::chain_census(store);
    let fprop = |store: &HypergraphStore, id: NodeId, key: u32| -> Option<f64> {
        match store.get_node(id)?.get_property(key) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        }
    };
    let mut max_gen = 0.0f64;
    let mut conducting = 0usize;
    let mut gen_pos_nodes = 0usize;
    for (id, _) in store.nodes() {
        let g = fprop(store, id, GENERATION).unwrap_or(0.0);
        if g > 0.0 {
            gen_pos_nodes += 1;
            max_gen = max_gen.max(g);
            if fprop(store, id, ENERGY).is_some_and(|e| e != q) {
                conducting += 1;
            }
        }
    }
    CellOut {
        n_sub,
        audit_ok,
        sparse_windows,
        phases: format!("{phases:?}"),
        chains: (census.chains_count, census.chain_length_p50, census.chain_length_max),
        max_gen,
        conducting,
        gen_pos_nodes,
        transport_rejects_total,
        seed_paid_only: max_gen <= 1.0,
    }
}

#[test]
#[ignore]
fn exp_l12_priority_swap() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!("cell | n_sub | audit | sparse_w | chains n/p50/max | max_gen | conducting/genpos | transport_rejects | phases");
    for &(e_total, max_steps) in &[(1.0e4f64, 4000u64), (1.0e5, 12000), (1.0e6, 60000)] {
        for seed in 42u64..45 {
            let cell = format!("swap_E{e_total:.0e}_s{seed}");
            let d = dir_for(&root, tmp.path(), &cell);
            let r = run_cell(&d, seed, e_total, max_steps, "transport_first", 50.0, 25.0);
            println!(
                "{cell} | {} | {} | {} | {}/{}/{} | {} | {}/{} | {} | {}",
                r.n_sub,
                if r.audit_ok { "exact" } else { "DRIFT" },
                r.sparse_windows,
                r.chains.0, r.chains.1, r.chains.2,
                r.max_gen,
                r.conducting, r.gen_pos_nodes,
                r.transport_rejects_total,
                r.phases,
            );
        }
    }
}

#[test]
#[ignore]
fn exp_l13_unfreeze_cells() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!("cell | n_sub | audit | chains n/p50/max | max_gen | conducting/genpos | seed_only | phases");
    for &(q, tau) in &[(50.0f64, 15.0f64), (75.0, 25.0)] {
        for priority in ["subdivide_first", "transport_first"] {
            for seed in 42u64..45 {
                let cell = format!("q{q:.0}t{tau:.0}_{}_s{seed}",
                    if priority == "subdivide_first" { "pump" } else { "swap" });
                let d = dir_for(&root, tmp.path(), &cell);
                // era = E₀/(q+c); cap steps at era + slack, larger for swap.
                let era = (4.0e5 / (q + 10.0)) as u64;
                let steps = if priority == "transport_first" { era + 20000 } else { era + 400 };
                let r = run_cell(&d, seed, 4.0e5, steps, priority, q, tau);
                println!(
                    "{cell} | {} | {} | {}/{}/{} | {} | {}/{} | {} | {}",
                    r.n_sub,
                    if r.audit_ok { "exact" } else { "DRIFT" },
                    r.chains.0, r.chains.1, r.chains.2,
                    r.max_gen,
                    r.conducting, r.gen_pos_nodes,
                    r.seed_paid_only,
                    r.phases,
                );
            }
        }
    }
}
