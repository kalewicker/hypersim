//! EXP-L15 (v5 §2, Phase R1): the relaxation experiment — the missing
//! half of the attractor demonstration. IC = the archived post-quench
//! pump state (six long throats, interiors flat at q; transport and
//! subdivide dead by energetics), full family enabled.
//!
//! Arms: (a) quiescent relaxation, seeds {42,43,44}; (b) concurrent
//! relaxation (same IC + fresh E₀ at a NEW site, seed 42).
//!
//! Pre-registered predictions (arm a, frozen before the runs):
//! - HALVING LAW: fuse under MaxParallel selects a maximal independent
//!   set of interior degree-2 triples → chain length ≈ L₀·2^(−t).
//!   Acceptance: within a factor-2 band of exact halving (deterministic
//!   tie-breaking makes each seed exact but MIS-boundary-dependent;
//!   per-seed trajectories archived — v5 §9.3).
//! - RESTORATION ENDPOINT: terminal state = the clean lattice,
//!   structurally bit-restored (WL-hash equality vs the clean-lattice
//!   fixture; label-invariant by construction). Energy-dead, ledgered.
//! - CENSUS FOLLOWS: wormhole count → 0 monotonically after the last
//!   crossing-length chain drops below R_max/2 (the 2L+2 threshold per
//!   F-LK14 — corrected arithmetic, v5 §9.2).
//!
//! Hazard §9.1: the IC is the archived snapshot; this run executes at
//! the ARCHIVED params (q=75, τ=25, cap ∞, subdivide_first) — copied
//! from the source manifest, and the source manifest params land in
//! this run's manifest as ic_provenance.
//!
//! Run with --ignored; HYPERSIM_SWEEP_OUT persists.

use hg_sim_core::{run_headless, SimConfig};
use std::path::{Path, PathBuf};

/// Archived-params rule patch (identical to the exp_l12/l13 patcher):
/// x-inheritance capped family at cap ∞, q=75, τ=25, c=10.
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
    .replace("value = 50.0\n", &format!("value = {q:.1}\n"))
    .replace("value = 60.0", &format!("value = {:.1}", q + 10.0))
    .replace("offset = 50.0", &format!("offset = {:.1}", 2.0 * tau))
    .replace("value = 25.0", &format!("value = {tau:.1}"))
}

fn ic_source(seed: u64) -> PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join(format!("runs/exp_l13/q75t25_pump_s{seed}/exp_l12_{seed}_TEST"))
}

fn l15_config(
    dir: &Path,
    seed: u64,
    concurrent: bool,
    max_steps: u64,
    window: u64,
) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp_l15"
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
max_pairings_per_step = 100000
thread_count = 1
[lk_rules]
enabled = true
e_total = 400000.0
g_max = 1.0e18
priority = "subdivide_first"
q = 75.0
tau = 25.0
seed_placement = "corner"
[lk_rules.fuse]
enabled = true
[leakage]
enabled = true
cadence_steps = {window}
[relaxation]
ic_source = "{}"
concurrent_seed = {concurrent}
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
        ic_source(seed).display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules_toml(75.0, 25.0));
    cfg
}

fn dir_for(root: &Option<String>, tmp: &Path, cell: &str) -> PathBuf {
    root.as_ref()
        .map(|r| {
            let p = Path::new(r).join(cell);
            std::fs::create_dir_all(&p).unwrap();
            p
        })
        .unwrap_or_else(|| tmp.join(cell))
}

/// Arm (a): quiescent relaxation, per-seed. Window/cadence 1 so the
/// halving trajectory is per-step exact.
#[test]
#[ignore]
fn exp_l15_arm_a_quiescent_relaxation() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!("cell | L0 | audit | e_fused | halving trace L(t) | halving | restoration | wh mono->0");
    for seed in 42u64..45 {
        let cell = format!("quiesce_s{seed}");
        let dir = dir_for(&root, tmp.path(), &cell);
        // L₀ from the archived IC itself (the state under measurement).
        let ic = hg_eval::cone::load_snapshots(&ic_source(seed).join("snapshots"))
            .unwrap()
            .pop()
            .expect("archived snapshot")
            .1;
        let ic_census = hg_eval::chains::chain_census(&ic);
        let l0 = ic_census.chain_length_max as f64;
        assert!(l0 > 100.0, "{cell}: archived IC must hold long throats (L0 = {l0})");

        let cfg = l15_config(&dir, seed, false, 40, 1);
        let artifacts = run_headless(&cfg, None, None, None).expect("run");

        // Audit exact at every emission, against the LOADED baseline.
        let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
        let rows: Vec<Vec<&str>> =
            leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
        let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
        assert!(audit_ok, "{cell}: audit drift on relaxation baseline");
        let e_fused_final: f64 = rows.last().unwrap()[16].parse().unwrap();

        // Halving law on the per-step snapshot census.
        let snaps =
            hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
        let mut trace: Vec<(u64, u32, u32)> = Vec::new(); // (t, chains, Lmax)
        for (t, (_, s)) in snaps.iter().enumerate() {
            let c = hg_eval::chains::chain_census(s);
            trace.push((t as u64 + 1, c.chains_count, c.chain_length_max));
        }
        // Pre-registered acceptance (v5 §2): FITTED decay within a
        // factor-2 band of exact halving — λ from least squares on
        // ln L(t) over the decaying segment (L ≥ 2), accepted iff
        // λ ∈ [ln2/2, 2·ln2].
        //
        // Mechanism note (first run, diagnosed): the plan's "roughly
        // alternate nodes fuse per step" assumed EDGE-disjoint bindings,
        // but MaxParallel conflict resolution is NODE-disjoint
        // (hg-engine conflict.rs) — a fuse binding claims {x, w, y}, so
        // the chain MIS takes every THIRD interior node and the exact
        // per-seed ratio is 2/3, λ = ln(3/2) ≈ 0.405: inside the
        // pre-registered band. Same error class as F-LK13 (§1.1 item 2:
        // concurrency-semantics assumptions must be justified against
        // MaxParallel at design time). The shared seed anchor also
        // serializes the six terminal length-1 fuses (node-disjointness
        // again), giving the -1/step tail after the exponential leg.
        let pts: Vec<(f64, f64)> = std::iter::once((0.0, l0.ln()))
            .chain(
                trace
                    .iter()
                    .filter(|&&(_, _, l)| l >= 2)
                    .map(|&(t, _, l)| (t as f64, (l as f64).ln())),
            )
            .collect();
        let n = pts.len() as f64;
        let (sx, sy) = pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
        let (sxx, sxy) = pts
            .iter()
            .fold((0.0, 0.0), |a, p| (a.0 + p.0 * p.0, a.1 + p.0 * p.1));
        let lambda = -((n * sxy - sx * sy) / (n * sxx - sx * sx));
        let ln2 = std::f64::consts::LN_2;
        let halving_ok = lambda >= 0.5 * ln2 && lambda <= 2.0 * ln2;
        let final_chains = trace.last().unwrap().1;
        assert_eq!(final_chains, 0, "{cell}: chains must fully reabsorb: {trace:?}");

        // Restoration endpoint: the runner's own restoration_check.
        let restoration = artifacts
            .summary
            .leakage_summary
            .iter()
            .find(|(k, _)| k == "restoration_check")
            .map(|(_, v)| v.clone())
            .expect("restoration_check emitted");
        let restored = restoration.contains("RESTORED") && !restoration.contains("NOT_RESTORED");

        // Census follows: wormhole_count_est non-increasing to 0 once the
        // longest chain < R_max/2 = 6 (2L+2 crossing threshold, F-LK14).
        let wh: Vec<f64> = rows.iter().filter_map(|r| r[10].parse().ok()).collect();
        let below_at = trace.iter().position(|&(_, _, lmax)| lmax < 6).unwrap_or(0);
        let tail = &wh[below_at.min(wh.len().saturating_sub(1))..];
        let wh_mono =
            tail.windows(2).all(|w| w[1] <= w[0]) && tail.last().copied() == Some(0.0);

        let short: Vec<u32> = trace.iter().map(|&(_, _, l)| l).take(14).collect();
        println!(
            "{cell} | {l0} | exact | {e_fused_final:.0} | {short:?} | λ={lambda:.4} ratio={:.4} {} | {} | {}",
            (-lambda).exp(),
            if halving_ok { "PASS" } else { "FAIL" },
            &restoration[..restoration.find(" — ").unwrap_or(restoration.len())],
            if wh_mono { "PASS" } else { "FAIL" },
        );
        assert!(
            halving_ok,
            "{cell}: fitted decay λ = {lambda:.4} outside the factor-2 halving band: {trace:?}"
        );
        assert!(restored, "{cell}: {restoration}");
        assert!(wh_mono, "{cell}: wormhole census not monotone→0 after threshold: {wh:?}");
    }
}

/// Arm (b): concurrent relaxation — same IC plus a fresh E₀ at a new
/// site (corner; the archived seed was center). Measures the
/// interaction of standing-structure removal with active production.
/// No pre-registered rows beyond audit exactness; readouts recorded.
#[test]
#[ignore]
fn exp_l15_arm_b_concurrent_relaxation() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let cell = "concurrent_s42";
    let dir = dir_for(&root, tmp.path(), cell);
    // Era of the fresh budget: E₀/(q+c) ≈ 4706 steps; slack past quench.
    let cfg = l15_config(&dir, 42, true, 5600, 100);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> =
        leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
    assert!(audit_ok, "{cell}: audit drift on relaxation+seed baseline");
    let last = rows.last().unwrap();
    let n_sub: f64 = last[15].parse().unwrap();
    let e_fused: f64 = last[16].parse().unwrap();
    let churn: u32 = last[17].parse().unwrap_or(0);
    let wh: Vec<f64> = rows.iter().filter_map(|r| r[10].parse().ok()).collect();
    let peak = wh.iter().copied().fold(0.0f64, f64::max);
    let final_wh = wh.last().copied().unwrap_or(0.0);
    assert!(n_sub > 0.0, "{cell}: the fresh seed must actually produce (n_sub = 0)");

    // Old-throat decay under concurrent production: census at the first
    // vs last snapshot.
    let snaps = hg_eval::cone::load_snapshots(&artifacts.run_dir.join("snapshots")).unwrap();
    let first_census = hg_eval::chains::chain_census(&snaps.first().unwrap().1);
    let last_census = hg_eval::chains::chain_census(&snaps.last().unwrap().1);
    println!(
        "{cell} | n_sub {n_sub} | audit exact | e_fused {e_fused:.0} | churn {churn} | \
         wh {peak:.0}->{final_wh:.0} | chains first {}/{} last {}/{}",
        first_census.chains_count,
        first_census.chain_length_max,
        last_census.chains_count,
        last_census.chain_length_max,
    );
}
