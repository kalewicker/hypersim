//! EXP-L17 (v6 §6, A4): the coexistence experiment. Does a countdown
//! removal rule with a genuinely fractional lifetime k produce a standing
//! leakage population — a balance between extrusion and removal?
//!
//! Mechanism-derived prediction (§4.2): standing population L* = k · p,
//! p = subdivision events per step. Pump regime serializes production at
//! the seed so p ≈ 1 and **L*(k) = k exactly** — a zero-free-parameter
//! linear law. Named discriminating alternatives:
//!   L* = k      — every link lives its full lifetime (removal keeps up);
//!   L* = 2k/3   — MIS-limited removal at the node-disjoint spacing;
//!   L* → ∞      — removal saturated (runaway).
//! The census slope over k discriminates them.
//!
//! Grid (full A4): regime{pump,swap} × ledger{vanish,transfer} ×
//! k{10,30,100,300} × E₀{1e5,4e5} × 3 seeds + controls {fuse-off,
//! countdown=0 (v5 suppression), f_cap (bistable reference)}. This driver
//! runs a SLICE selected by env HYPERSIM_L17_SLICE (default the decisive
//! pump×vanish×E1e5 k-sweep + controls); the full grid is the same
//! function over more axes. Run with --ignored; HYPERSIM_SWEEP_OUT
//! persists (snapshots feed the R4 lifetime cross-check offline).

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

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

struct Cell {
    audit_ok: bool,
    n_sub: f64,
    e_fused: f64,
    churn: u32,
    /// Standing chain census: mean and var over the steady segment.
    l_mean: f64,
    l_var: f64,
    /// Production rate p = subdivisions per step over the steady segment.
    p: f64,
    steady_windows: usize,
    longest_sparse_pop: usize,
    verdict: &'static str,
}

#[allow(clippy::too_many_arguments)]
fn run_cell(
    dir: &Path,
    seed: u64,
    regime: &str,
    gate: &str,
    ledger: &str,
    k: i64,
    e_total: f64,
    max_steps: u64,
    fuse_enabled: bool,
    f_cap: Option<f64>,
) -> Cell {
    let (q, tau) = (50.0f64, 15.0f64);
    // Window/leakage cadence must resolve the PRODUCTION era, which in
    // vanish mode is only E₀/(q+c) ≈ 1666 steps: at cadence 100 that is
    // ~16 windows, too few to read a standing level. Scale the cadence
    // to the expected era so every cell gets ≥ ~50 in-era windows.
    let window = if ledger == "transfer" { 100 } else { 25 };
    let fcap_line = f_cap.map(|f| format!("f_cap = {f:.1}")).unwrap_or_default();
    let toml = format!(
        r#"
experiment_name = "exp_l17"
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
e_total = {e_total}
g_max = 1.0e18
priority = "{regime}"
q = {q}
tau = {tau}
[lk_rules.fuse]
enabled = {fuse_enabled}
gate = "{gate}"
countdown_k = {k}
ledger_mode = "{ledger}"
{fcap_line}
[leakage]
enabled = true
cadence_steps = {window}
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules_toml(q, tau));
    let artifacts = run_headless(&cfg, None, None, None).expect("run");

    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
    let last = rows.last().unwrap();
    let n_sub: f64 = last[15].parse().unwrap();
    let e_fused: f64 = last[16].parse().unwrap();
    let churn: u32 = last[17].parse().unwrap_or(0);

    // STEADY SEGMENT = the PRODUCTION era, past the fill transient.
    //
    // Coexistence is a balance between extrusion and removal, so it is
    // only defined where production is live. The first cut of this
    // driver used "second half of the run", which at E₀=1e5 lands
    // entirely POST-QUENCH (era = E₀/(q+c) = 1666 events ≈ 1666 steps in
    // the serialized pump, vs a 30k-step run): p = 0 there, so L* = k·p
    // = 0 trivially and every cell read EMPTY. That is a vacuous
    // measurement, not a bistability result — recorded as a driver bug,
    // fixed here. (Same error family as the plan's own catalogue: the
    // statistic was satisfiable without the mechanism being tested.)
    //
    // Segment: windows where n_sub is still rising (production live) AND
    // step > k (the first cohort has had time to expire, so the standing
    // population has filled). Standing population = wormhole_count_est.
    let wh: Vec<f64> = rows.iter().map(|r| r[10].parse().unwrap_or(0.0)).collect();
    let steps: Vec<f64> = rows.iter().map(|r| r[0].parse().unwrap_or(0.0)).collect();
    let nsubs: Vec<f64> = rows.iter().map(|r| r[15].parse().unwrap_or(0.0)).collect();
    let producing: Vec<usize> = (1..rows.len())
        .filter(|&i| nsubs[i] > nsubs[i - 1] && steps[i] > k.max(0) as f64)
        .collect();
    let (l_mean, l_var, p) = if producing.len() >= 3 {
        let vals: Vec<f64> = producing.iter().map(|&i| wh[i]).collect();
        let m = vals.iter().sum::<f64>() / vals.len() as f64;
        let v = vals.iter().map(|x| (x - m).powi(2)).sum::<f64>() / vals.len() as f64;
        let (f, l) = (producing[0], *producing.last().unwrap());
        let p = if steps[l] > steps[f] {
            (nsubs[l] - nsubs[f]) / (steps[l] - steps[f])
        } else {
            0.0
        };
        (m, v, p)
    } else {
        (0.0, 0.0, 0.0)
    };
    let seg = producing.clone();
    // Longest consecutive Sparse ∧ census>0 stratum (EXP-L7R gate).
    let mut longest = 0usize;
    let mut cur = 0usize;
    for r in &rows {
        if r[12] == "sparse" && r[10].parse::<f64>().unwrap_or(0.0) > 0.0 {
            cur += 1;
            longest = longest.max(cur);
        } else {
            cur = 0;
        }
    }
    // Verdict over the PRODUCTION segment: a standing population that is
    // neither ~0 (removal wins: suppression) nor monotonically climbing
    // across the segment (production wins: runaway) is coexistence.
    let verdict = if seg.len() < 3 {
        "NO_ERA" // production segment too short to judge — not a result
    } else {
        let first = wh[seg[0]];
        let last = wh[*seg.last().unwrap()];
        if l_mean < 0.5 {
            "SUPPRESSED"
        } else if last > 1.5 * first.max(1.0) {
            "RUNAWAY"
        } else {
            "COEXIST"
        }
    };

    Cell { audit_ok, n_sub, e_fused, churn, l_mean, l_var, p, steady_windows: seg.len(), longest_sparse_pop: longest, verdict }
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

/// Q7-3 (v7 §4): the k = 300 departure (L\*/k = 0.81 against ≈1.27 at
/// k ≤ 100). Pre-registered test of the era-length explanation: rerun
/// k = 300 at 4× budget, where transfer mode's E₀/c scaling gives a
/// proportionally longer era. If L\*/k returns to ≈1.27 the linear law
/// holds for k ≪ era length and the domain is stated as such; if it
/// stays at 0.81 the departure is a real saturation needing its own
/// mechanism. k = 100 at the same budget is the control (it should be
/// unchanged, ruling out a budget effect on the law itself).
#[test]
#[ignore]
fn q7_3_scaling_domain_at_4x_budget() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    println!("cell | audit | n_sub | L*(mean±sd) | p | L*/(k·p) | era_w | verdict");
    for k in [100i64, 300] {
        let cell = format!("q73_swap_transfer_k{k}_E4e5_s42");
        let d = dir_for(&root, tmp.path(), &cell);
        let c = run_cell(
            &d, 42, "transport_first", "countdown", "transfer", k, 4.0e5, 60000, true, None,
        );
        assert!(c.audit_ok, "{cell}: audit drift");
        let pred = k as f64 * c.p;
        println!(
            "{cell} | exact | {} | {:.1}±{:.1} | {:.3} | {} | {} | {}",
            c.n_sub,
            c.l_mean,
            c.l_var.sqrt(),
            c.p,
            if pred > 0.0 { format!("{:.3}", c.l_mean / pred) } else { "n/a".into() },
            c.steady_windows,
            c.verdict,
        );
    }
}

#[test]
#[ignore]
fn exp_l17_coexistence() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let slice = std::env::var("HYPERSIM_L17_SLICE").unwrap_or_else(|_| "decisive".into());
    // Decisive slice: pump (subdivide_first) × {vanish, transfer} ×
    // k-sweep × E1e5 × seed 42, + controls. BOTH ledgers are in the
    // decisive slice because the coexistence question needs a live
    // production era: vanish quenches at E₀/(q+c) ≈ 1666 events, while
    // transfer runs to E₀/c ≈ 10000 (A1), giving a genuinely sustained
    // era — the regime where a balance can exist at all.
    let ks = [10i64, 30, 100, 300];
    let e0 = 1.0e5;
    // Steps sized to the era per ledger (+ margin), not a flat 30k.
    let steps_for = |ledger: &str| -> u64 {
        if ledger == "transfer" { 20000 } else { 6000 }
    };
    println!("cell | audit | n_sub | e_fused | churn | L*(mean±sd) | p | L*/(k·p) | era_w | sparse_pop | verdict");

    let seeds: &[u64] = if slice == "multiseed" { &[42, 43, 44] } else { &[42] };
    let regimes: &[&str] = match slice.as_str() {
        "full" => &["subdivide_first", "transport_first"],
        // The bottleneck probe: swap has ~14–15 chains and distributed
        // payers (EXP-L12) instead of the pump's 6 seed-anchored chains,
        // so if L* is set by a structural bottleneck rather than by k,
        // it must MOVE between regimes.
        "swap" => &["transport_first"],
        _ => &["subdivide_first"],
    };
    let ledgers: &[&str] = &["vanish", "transfer"];

    for &regime in regimes {
        for &ledger in ledgers {
            for &seed in seeds {
                for &k in &ks {
                    let cell = format!(
                        "{}_{ledger}_k{k}_E1e5_s{seed}",
                        if regime == "subdivide_first" { "pump" } else { "swap" }
                    );
                    let d = dir_for(&root, tmp.path(), &cell);
                    let c = run_cell(
                        &d, seed, regime, "countdown", ledger, k, e0, steps_for(ledger), true, None,
                    );
                    assert!(c.audit_ok, "{cell}: audit drift");
                    // The prediction is L* = k·p, so the diagnostic ratio
                    // is L*/(k·p): 1 = full-lifetime, 0.67 = MIS-limited.
                    let pred = k as f64 * c.p;
                    println!(
                        "{cell} | exact | {} | {:.0} | {} | {:.1}±{:.1} | {:.3} | {} | {} | {} | {}",
                        c.n_sub, c.e_fused, c.churn, c.l_mean, c.l_var.sqrt(), c.p,
                        if pred > 0.0 { format!("{:.3}", c.l_mean / pred) } else { "n/a".into() },
                        c.steady_windows, c.longest_sparse_pop, c.verdict,
                    );
                }
            }
        }
    }
    // Controls (once, pump/E1e5/seed42): fuse-off (pure growth),
    // countdown=0 (v5 unthrottled suppression), f_cap=40 (bistable ref).
    let ctrls: [(&str, &str, i64, bool, Option<f64>); 3] = [
        ("nofuse", "none", 0, false, None),
        ("cd0", "countdown", 0, true, None),
        ("fcap40", "f_cap", 0, true, Some(40.0)),
    ];
    for (name, gate, k, fuse, fcap) in ctrls {
        let cell = format!("ctrl_{name}_E1e5_s42");
        let d = dir_for(&root, tmp.path(), &cell);
        let c = run_cell(
            &d, 42, "subdivide_first", gate, "vanish", k, e0, steps_for("vanish"), fuse, fcap,
        );
        assert!(c.audit_ok, "{cell}: audit drift");
        println!(
            "{cell} | exact | {} | {:.0} | {} | {:.1}±{:.1} | {:.3} | - | {} | {} | {}",
            c.n_sub, c.e_fused, c.churn, c.l_mean, c.l_var.sqrt(), c.p,
            c.steady_windows, c.longest_sparse_pop, c.verdict,
        );
    }
    println!(
        "PREDICTION (§4.2): L*/(k·p) ≈ 1 ⇒ every link lives its full lifetime; \
         ≈ 0.67 ⇒ MIS-limited removal; L* climbing across the era ⇒ runaway; \
         L* ≈ 0 with a live era (p > 0) ⇒ still bistable on a DISPERSED gate \
         (the strong negative). A cell with p = 0 is NO_ERA and is not a result."
    );
}
