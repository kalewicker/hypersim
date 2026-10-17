//! EXP-L16 (v5 §4, Phase R3): the balance sweep — does a steady-turnover
//! band exist between v4's unthrottled suppression and pure growth?
//!
//! Grid: transport_first × (q,τ) = (75,25) × f_cap ∈ {10,25,40,60,74} ×
//! E₀ ∈ {1e4, 1e5} × 5 seeds, plus control columns: fuse-disabled (pure
//! growth) and f_cap-disabled (v4 unthrottled suppression), plus the R2
//! §3 pre-registered pump-regime cells (flat-at-q never drains → fuse
//! throttled dead: evals > 0, rejects = evals, e_fused = 0 — DESIGNED
//! behavior, not failure).
//!
//! Pre-registered scaling law: standing population ∝ t_age × production
//! rate ≈ (q − f_cap)/τ × events-per-step (proportionality, not just
//! monotonicity — Pearson r across the f_cap column reported per E₀).
//!
//! Band criterion (§1.4 authorization, restated): ≥ 200 consecutive
//! Sparse-classified windows (leakage cadence) with standing
//! emergent-wormhole population > 0, in a single cell. Nothing else
//! authorizes a CD claim (EXP-L7R).
//!
//! Pre-committed rows: band exists → balance demonstrated (first
//! persistent leakage spectrum; EXP-L7R authorized on those cells).
//! Bistability (suppression↔growth, no band) → energy gate too coarse;
//! recorded; annealed f_cap is a v6 question. Monotone growth wherever
//! throttled → the attractor claim gains its boundary (locality only in
//! the unthrottled limit).
//!
//! Run with --ignored; HYPERSIM_SWEEP_OUT persists (R4's lifetime
//! instrument consumes the persisted snapshots offline).

use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;

/// exp_l12/l13 patcher: x-inheritance capped family at cap ∞, (q,τ).
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
    peak: f64,
    final_wh: f64,
    verdict: &'static str,
    /// Longest run of consecutive Sparse windows with census > 0, and
    /// the mean census over that segment.
    band_len: usize,
    band_mean: f64,
    /// Production rate: n_sub_final / steps-to-quench.
    rate: f64,
    /// f_cap guard (prop1) totals, if the guard exists.
    fcap_evals: u64,
    fcap_rejects: u64,
}

fn run_cell(
    dir: &Path,
    seed: u64,
    e_total: f64,
    max_steps: u64,
    priority: &str,
    f_cap: Option<f64>,
    fuse_enabled: bool,
) -> Cell {
    run_cell_qt(dir, seed, e_total, max_steps, priority, f_cap, fuse_enabled, 75.0, 25.0)
}

#[allow(clippy::too_many_arguments)]
fn run_cell_qt(
    dir: &Path,
    seed: u64,
    e_total: f64,
    max_steps: u64,
    priority: &str,
    f_cap: Option<f64>,
    fuse_enabled: bool,
    q: f64,
    tau: f64,
) -> Cell {
    let window = if max_steps > 20000 { 500 } else { 100 };
    let toml = format!(
        r#"
experiment_name = "exp_l16"
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
priority = "{priority}"
q = {q}
tau = {tau}
[lk_rules.fuse]
enabled = {fuse_enabled}
{}
[leakage]
enabled = true
cadence_steps = 100
[output]
snapshots_enabled = true
fixed_provenance = true
"#,
        dir.display(),
        f_cap.map(|f| format!("f_cap = {f:.1}")).unwrap_or_default(),
    );
    let mut cfg = SimConfig::from_toml(&toml).expect("config parses");
    cfg.rules.inline_toml = Some(rules_toml(q, tau));
    let artifacts = run_headless(&cfg, None, None, None).expect("run");

    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> =
        leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap() == 0.0);
    let last = rows.last().unwrap();
    let n_sub: f64 = last[15].parse().unwrap();
    let e_fused: f64 = last[16].parse().unwrap();
    let churn: u32 = last[17].parse().unwrap_or(0);

    let wh: Vec<f64> = rows.iter().map(|r| r[10].parse().unwrap_or(0.0)).collect();
    let peak = wh.iter().copied().fold(0.0f64, f64::max);
    let final_wh = *wh.last().unwrap_or(&0.0);
    let last_quarter = &wh[wh.len() - wh.len() / 4..];
    let rising = last_quarter.first().is_some_and(|&a| final_wh > a);
    let verdict = if peak > 0.0 && final_wh < 0.05 * peak {
        "DECAY"
    } else if final_wh >= 0.95 * peak && rising {
        "GROWTH"
    } else if final_wh > 0.0 {
        "STEADY"
    } else {
        "EMPTY"
    };

    // §1.4 stratum: longest consecutive Sparse ∧ census>0 run.
    let (mut band_len, mut band_mean) = (0usize, 0.0f64);
    let (mut cur, mut cur_sum) = (0usize, 0.0f64);
    for r in &rows {
        let sparse_pop = r[12] == "sparse" && r[10].parse::<f64>().unwrap_or(0.0) > 0.0;
        if sparse_pop {
            cur += 1;
            cur_sum += r[10].parse::<f64>().unwrap();
        } else {
            cur = 0;
            cur_sum = 0.0;
        }
        if cur > band_len {
            band_len = cur;
            band_mean = cur_sum / cur as f64;
        }
    }

    // Production rate: n_sub / steps at quench (first window where
    // n_subdivisions stops changing; full run if it never quenches).
    let quench_step = rows
        .windows(2)
        .find(|w| w[0][15] == w[1][15])
        .map(|w| w[0][0].parse::<f64>().unwrap())
        .unwrap_or(max_steps as f64);
    let rate = if quench_step > 0.0 { n_sub / quench_step } else { 0.0 };

    // f_cap guard totals (prop1 of lk_fuse).
    let (mut fcap_evals, mut fcap_rejects) = (0u64, 0u64);
    if let Ok(guards) = std::fs::read_to_string(artifacts.run_dir.join("guards.csv")) {
        for l in guards.lines().skip(2) {
            let f: Vec<&str> = l.split(',').collect();
            if f[1] == "lk_fuse" && f[2] == "prop1" {
                fcap_evals += f[3].parse::<u64>().unwrap_or(0);
                fcap_rejects += f[4].parse::<u64>().unwrap_or(0);
            }
        }
    }

    Cell {
        audit_ok,
        n_sub,
        e_fused,
        churn,
        peak,
        final_wh,
        verdict,
        band_len,
        band_mean,
        rate,
        fcap_evals,
        fcap_rejects,
    }
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

#[test]
#[ignore]
fn exp_l16_balance_sweep() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let f_caps = [10.0f64, 25.0, 40.0, 60.0, 74.0];
    let e_totals = [(1.0e4f64, 12000u64), (1.0e5, 45000)];
    println!(
        "cell | audit | n_sub | e_fused | churn | wh peak->final | verdict | band len/mean | \
         rate | fcap evals/rejects"
    );
    // Scaling-law accumulators: per E₀, per f_cap → mean band census and
    // mean t_age × rate over seeds.
    let mut scaling: Vec<((usize, usize), (f64, f64))> = Vec::new();
    for (ei, &(e_total, max_steps)) in e_totals.iter().enumerate() {
        for (fi, &f_cap) in f_caps.iter().enumerate() {
            let (mut sum_census, mut sum_pred) = (0.0, 0.0);
            for seed in 42u64..47 {
                let cell = format!("f{f_cap:.0}_E{e_total:.0e}_s{seed}");
                let d = dir_for(&root, tmp.path(), &cell);
                let c = run_cell(&d, seed, e_total, max_steps, "transport_first", Some(f_cap), true);
                assert!(c.audit_ok, "{cell}: audit drift");
                println!(
                    "{cell} | exact | {} | {:.0} | {} | {:.0}->{:.0} | {} | {}/{:.1} | {:.3} | {}/{}",
                    c.n_sub, c.e_fused, c.churn, c.peak, c.final_wh, c.verdict,
                    c.band_len, c.band_mean, c.rate, c.fcap_evals, c.fcap_rejects,
                );
                sum_census += c.band_mean;
                sum_pred += (75.0 - f_cap) / 25.0 * c.rate;
            }
            scaling.push(((ei, fi), (sum_census / 5.0, sum_pred / 5.0)));
        }
        // Controls: fuse-disabled (pure growth), f_cap-disabled (v4
        // unthrottled suppression).
        for seed in 42u64..47 {
            let cell = format!("nofuse_E{e_total:.0e}_s{seed}");
            let d = dir_for(&root, tmp.path(), &cell);
            let c = run_cell(&d, seed, e_total, max_steps, "transport_first", None, false);
            assert!(c.audit_ok, "{cell}: audit drift");
            assert_eq!(c.e_fused, 0.0, "{cell}: fuse-disabled control must not ledger");
            println!(
                "{cell} | exact | {} | 0 | {} | {:.0}->{:.0} | {} | {}/{:.1} | {:.3} | -/-",
                c.n_sub, c.churn, c.peak, c.final_wh, c.verdict, c.band_len, c.band_mean, c.rate,
            );
        }
        for seed in 42u64..47 {
            let cell = format!("uncapped_E{e_total:.0e}_s{seed}");
            let d = dir_for(&root, tmp.path(), &cell);
            let c = run_cell(&d, seed, e_total, max_steps, "transport_first", None, true);
            assert!(c.audit_ok, "{cell}: audit drift");
            println!(
                "{cell} | exact | {} | {:.0} | {} | {:.0}->{:.0} | {} | {}/{:.1} | {:.3} | -/-",
                c.n_sub, c.e_fused, c.churn, c.peak, c.final_wh, c.verdict,
                c.band_len, c.band_mean, c.rate,
            );
        }
    }
    // R2 §3 pre-registered pump-regime prediction: flat-at-q profiles
    // never drain → fuse goes dead → pure growth returns. Guard must
    // show evals > 0 with rejects = evals; ledger empty. DESIGNED.
    for seed in 42u64..45 {
        let cell = format!("pump_f40_E1e4_s{seed}");
        let d = dir_for(&root, tmp.path(), &cell);
        let c = run_cell(&d, seed, 1.0e4, 2000, "subdivide_first", Some(40.0), true);
        assert!(c.audit_ok, "{cell}: audit drift");
        assert!(
            c.fcap_evals > 0 && c.fcap_rejects == c.fcap_evals,
            "{cell}: pump-regime f_cap guard must be engaged and reject ALL \
             (evals {}, rejects {})",
            c.fcap_evals,
            c.fcap_rejects,
        );
        assert_eq!(c.e_fused, 0.0, "{cell}: pump-regime fuse must ledger nothing");
        println!(
            "{cell} | exact | {} | 0 | {} | {:.0}->{:.0} | {} | {}/{:.1} | {:.3} | {}/{}",
            c.n_sub, c.churn, c.peak, c.final_wh, c.verdict, c.band_len, c.band_mean,
            c.rate, c.fcap_evals, c.fcap_rejects,
        );
    }
    // Scaling law (pre-registered): standing population ∝ t_age × rate.
    scaling_report(&e_totals, &scaling);
}

/// DOCUMENTED EXTENSION (not in the v5 §4 grid): the pinned (75,25)
/// cell is Q2's row (c) — ZERO interior conduction (profile flatness) —
/// so if the main sweep shows the f_cap guard rejecting every eval, the
/// aging timer never matures there and the sweep records an
/// instrument-never-engaged verdict (the EXP-L9 lesson: an inert
/// instrument is NOT a balance finding). §3's own selection criterion
/// ("the live corner … Q1/Q2 demonstrated drainage and interleaving")
/// actually names Q2's row (b), (q,τ) = (50,15), the fully-conducting
/// cell. This extension runs the SAME sweep design there: f_cap ∈
/// {10, 25, 40, 49} (< q = 50) × E₀ = 1e5 × 3 seeds + both controls.
#[test]
#[ignore]
fn exp_l16_live_corner_extension_q50t15() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let (q, tau, e_total, max_steps) = (50.0f64, 15.0f64, 1.0e5f64, 45000u64);
    println!(
        "cell | audit | n_sub | e_fused | churn | wh peak->final | verdict | band len/mean | \
         rate | fcap evals/rejects"
    );
    let mut scaling: Vec<((usize, usize), (f64, f64))> = Vec::new();
    for (fi, &f_cap) in [10.0f64, 25.0, 40.0, 49.0].iter().enumerate() {
        let (mut sum_census, mut sum_pred) = (0.0, 0.0);
        for seed in 42u64..45 {
            let cell = format!("q50t15_f{f_cap:.0}_E1e5_s{seed}");
            let d = dir_for(&root, tmp.path(), &cell);
            let c = run_cell_qt(
                &d, seed, e_total, max_steps, "transport_first", Some(f_cap), true, q, tau,
            );
            assert!(c.audit_ok, "{cell}: audit drift");
            println!(
                "{cell} | exact | {} | {:.0} | {} | {:.0}->{:.0} | {} | {}/{:.1} | {:.3} | {}/{}",
                c.n_sub, c.e_fused, c.churn, c.peak, c.final_wh, c.verdict,
                c.band_len, c.band_mean, c.rate, c.fcap_evals, c.fcap_rejects,
            );
            sum_census += c.band_mean;
            sum_pred += (q - f_cap) / tau * c.rate;
        }
        scaling.push(((0, fi), (sum_census / 3.0, sum_pred / 3.0)));
    }
    for (name, f_cap, fuse) in
        [("nofuse", None, false), ("uncapped", None::<f64>, true)]
    {
        for seed in 42u64..45 {
            let cell = format!("q50t15_{name}_E1e5_s{seed}");
            let d = dir_for(&root, tmp.path(), &cell);
            let c = run_cell_qt(
                &d, seed, e_total, max_steps, "transport_first", f_cap, fuse, q, tau,
            );
            assert!(c.audit_ok, "{cell}: audit drift");
            println!(
                "{cell} | exact | {} | {:.0} | {} | {:.0}->{:.0} | {} | {}/{:.1} | {:.3} | -/-",
                c.n_sub, c.e_fused, c.churn, c.peak, c.final_wh, c.verdict,
                c.band_len, c.band_mean, c.rate,
            );
        }
    }
    scaling_report(&[(e_total, max_steps)], &scaling);
}

fn scaling_report(e_totals: &[(f64, u64)], scaling: &[((usize, usize), (f64, f64))]) {
    for (ei, &(e_total, _)) in e_totals.iter().enumerate() {
        let pts: Vec<(f64, f64)> = scaling
            .iter()
            .filter(|((e, _), _)| *e == ei)
            .map(|(_, p)| *p)
            .collect();
        let n = pts.len() as f64;
        let (mx, my) = (
            pts.iter().map(|p| p.1).sum::<f64>() / n,
            pts.iter().map(|p| p.0).sum::<f64>() / n,
        );
        let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
        for &(census, pred) in &pts {
            sxy += (pred - mx) * (census - my);
            sxx += (pred - mx) * (pred - mx);
            syy += (census - my) * (census - my);
        }
        let r = if sxx > 0.0 && syy > 0.0 { sxy / (sxx * syy).sqrt() } else { f64::NAN };
        println!(
            "scaling E{e_total:.0e}: (t_age×rate, census) = {:?} | pearson r = {r:.3}",
            pts.iter().map(|p| (p.1, p.0)).collect::<Vec<_>>(),
        );
    }
}

/// BISTABILITY straddle (v5 §4 pre-committed row, made concrete): the
/// main sweep + extension established that interior energy in the LK-R
/// family settles at a PLATEAU above q (≈ 65 at (50,15); the profile
/// is a near-spike, not a spread — energy_probe.rs), so the aging-timer
/// premise ("a fresh child carries exactly q, untouchable until it
/// drains") is inverted: downhill-only metered transport with a 2τ dead
/// zone conducts a fresh child UP to the plateau, never below it. This
/// test straddles the plateau at (50,15) with one seed to show the
/// transition is a JUMP with no band between: f_cap below the plateau =
/// pure growth (fuse never fires), f_cap above it = v4 unthrottled
/// suppression. An energy gate is too coarse a rate control here; a
/// finer mechanism (annealed f_cap) is a v6 question, not improvised.
#[test]
#[ignore]
fn exp_l16_bistability_straddle_q50t15() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let (q, tau, e_total, max_steps) = (50.0f64, 15.0f64, 1.0e5f64, 45000u64);
    println!("f_cap | verdict | e_fused | churn | census(band) | side-of-plateau(≈65)");
    // Below plateau (60): inert. Straddling/above (66, 80, 100):
    // unthrottled suppression. No f_cap yields a nonzero steady census
    // band that the below-plateau column doesn't also (frozen) have.
    let mut jumped = false;
    for &f_cap in &[60.0f64, 66.0, 80.0, 100.0] {
        let cell = format!("q50t15_straddle_f{f_cap:.0}_s42");
        let d = dir_for(&root, tmp.path(), &cell);
        let c = run_cell_qt(
            &d, 42, e_total, max_steps, "transport_first", Some(f_cap), true, q, tau,
        );
        assert!(c.audit_ok, "{cell}: audit drift");
        let side = if f_cap < 65.0 { "below→growth" } else { "above→suppression" };
        // The jump: below-plateau ledgers nothing, above-plateau ledgers
        // the whole endowment (suppression).
        if f_cap < 65.0 {
            assert_eq!(c.e_fused, 0.0, "{cell}: below plateau, fuse must not fire");
        } else {
            assert!(c.e_fused > 0.0, "{cell}: above plateau, fuse must engage");
            jumped = true;
        }
        println!(
            "{f_cap:.0} | {} | {:.0} | {} | {:.1} | {side}",
            c.verdict, c.e_fused, c.churn, c.band_mean,
        );
    }
    assert!(jumped, "bistability: an above-plateau f_cap must flip to suppression");
}
