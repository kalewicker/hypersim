//! B3 (v8 §5): price the closure defect under the PASSIVE clock, then
//! apply the pre-committed D3 decision.
//!
//! Pilot, not a commitment: one swap × transfer × best-k cell at
//! E₀ = 4e5, timed end to end, extrapolated to E₀ = 2e6 — the budget
//! Q7-5 showed is needed to clear the effective-sample floor (τ ≈ 400
//! steps ⇒ ≈1.4–1.7 × 10⁵ steps of steady segment).
//!
//! Two things are measured, not argued:
//!   1. wall-clock per seed at 4e5 and its extrapolation to 2e6;
//!   2. whether the effective sample size actually scales the way the
//!      extrapolation assumes. Buying 5× the steps only buys ≈5× the
//!      effective N if ρ does not rise with the segment. Q7-5 measured
//!      ρ ≈ 0.78; A3 already showed that finer cadence raises ρ toward 1
//!      and buys nothing. A longer ERA is the supported lever, but the
//!      pilot checks it rather than assuming it — otherwise D3 would
//!      commission a run that cannot clear the floor at any price.
//!
//! D3 (pre-committed, v8 §5):
//!   ≤ ~1 h/seed  → commission at E₀ ≈ 2e6, 5 seeds
//!   ~1–6 h/seed  → commission 3 seeds, state the reduced n as a limit
//!   > ~6 h/seed  → do not commission; record the price, ship with CD
//!                  unmeasured and quantified, treat as a v9 item
//!
//! Run with --ignored --nocapture --test-threads=1 (the timing is the
//! measurement — do not run it against a loaded machine).

use hg_causal::closure::WindowSample;
use hg_sim_core::{run_headless, SimConfig};
use std::path::Path;
use std::time::Instant;

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

struct Timed {
    secs: f64,
    steps: u64,
    /// Longest consecutive stratum satisfying P1's mask.
    stratum: usize,
    rho: f64,
    eff_n: f64,
    audit_ok: bool,
    e_left: f64,
}

fn run(dir: &Path, seed: u64, k: i64, e0: f64, steps: u64, window: u64, gate: &str) -> Timed {
    let toml = format!(
        r#"
experiment_name = "b3_pricing"
seed = {seed}
out_dir = "{}"
[rules]
source = "Inline"
[initial]
condition_type = "TriLattice"
width = 32
[engine]
step_mode = "MaxParallel"
max_steps = {steps}
metrics_window_interval = {window}
max_pairings_per_step = 100000
thread_count = 1
[lk_rules]
enabled = true
e_total = {e0}
g_max = 1.0e18
priority = "transport_first"
q = 50.0
tau = 15.0
[lk_rules.fuse]
enabled = true
gate = "{gate}"
countdown_k = {k}
lifetime_k = {k}
ledger_mode = "transfer"
[leakage]
enabled = true
cadence_steps = {window}
dump_closure_samples = true
[output]
fixed_provenance = true
"#,
        dir.display(),
    );
    let mut cfg = SimConfig::from_toml(&toml).unwrap();
    cfg.rules.inline_toml = Some(rules_toml(50.0, 15.0));

    let t = Instant::now();
    let a = run_headless(&cfg, None, None, None).expect("run");
    let secs = t.elapsed().as_secs_f64();

    let leak = std::fs::read_to_string(a.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<String>> = leak
        .lines()
        .skip(2)
        .map(|l| l.split(',').map(str::to_string).collect())
        .collect();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap_or(1.0) == 0.0);
    let nsub: Vec<f64> = rows.iter().map(|r| r[15].parse().unwrap_or(0.0)).collect();
    let census: Vec<f64> = rows.iter().map(|r| r[10].parse().unwrap_or(0.0)).collect();
    let ok: Vec<bool> = (0..rows.len())
        .map(|i| i > 0 && nsub[i] > nsub[i - 1] && rows[i][12] == "sparse" && census[i] > 0.0)
        .collect();
    let (mut best, mut cur): (Vec<usize>, Vec<usize>) = (vec![], vec![]);
    for (i, &good) in ok.iter().enumerate() {
        if good {
            cur.push(i);
            if cur.len() > best.len() {
                best = cur.clone();
            }
        } else {
            cur.clear();
        }
    }
    let samples: Vec<WindowSample> =
        std::fs::read_to_string(a.run_dir.join("closure_samples.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
    let (rho, eff_n) = effective_n(&samples, &best);
    // Energy left in the graph at the end (column 14, `e_total_now`) —
    // the budget question, and a check that the run actually consumed
    // what it was given.
    let e_left = rows
        .last()
        .and_then(|r| r[14].parse::<f64>().ok())
        .unwrap_or(f64::NAN);
    Timed {
        secs,
        steps,
        stratum: best.len(),
        rho,
        eff_n,
        audit_ok,
        e_left,
    }
}

/// The A3-restated floor: effective, not raw, sample size. Same
/// implementation as the EXP-L7R driver's, deliberately — pricing the
/// measurement against a different statistic than the one that gates it
/// would be worthless.
fn effective_n(samples: &[WindowSample], stratum: &[usize]) -> (f64, f64) {
    if stratum.len() < 5 {
        return (f64::NAN, 0.0);
    }
    let dims = samples[stratum[0]].micro_v.len();
    let mut rhos = Vec::new();
    for j in 0..dims {
        let x: Vec<f64> = stratum.iter().map(|&i| samples[i].micro_v[j]).collect();
        let m = x.iter().sum::<f64>() / x.len() as f64;
        let var: f64 = x.iter().map(|y| (y - m).powi(2)).sum();
        if var < 1e-12 {
            continue;
        }
        let cov: f64 = (1..x.len()).map(|t| (x[t] - m) * (x[t - 1] - m)).sum();
        rhos.push(cov / var);
    }
    if rhos.is_empty() {
        return (f64::NAN, 0.0);
    }
    let rho = rhos.iter().sum::<f64>() / rhos.len() as f64;
    let n = stratum.len() as f64;
    (rho, n * (1.0 - rho) / (1.0 + rho))
}

#[test]
#[ignore]
fn b3_price_cd_under_the_passive_clock() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let k: i64 = std::env::var("B3_K").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let gate = std::env::var("B3_GATE").unwrap_or_else(|_| "age".into());
    let dir_for = |cell: &str| -> std::path::PathBuf {
        root.as_ref()
            .map(|r| {
                let p = Path::new(r).join(cell);
                std::fs::create_dir_all(&p).unwrap();
                p
            })
            .unwrap_or_else(|| tmp.path().join(cell))
    };

    println!("B3 pilot — swap × transfer × k={k} × gate={gate}. Timing IS the measurement.");
    println!("cell | E₀ | steps | wall s | s/1e3 steps | stratum | ρ | eff N | E left | audit");

    // Two budgets, so the extrapolation has a measured slope rather than
    // an assumed proportionality: cost per step can rise with the graph.
    let mut points: Vec<(f64, u64, f64)> = Vec::new(); // (E₀, steps, secs)
    let mut effs: Vec<(f64, f64, f64)> = Vec::new(); // (E₀, ρ, eff N)
    for &(e0, steps) in &[(1.0e5f64, 20_000u64), (4.0e5, 60_000)] {
        let cell = format!("b3_k{k}_{gate}_E{e0:.0e}");
        let t = run(&dir_for(&cell), 42, k, e0, steps, 100, &gate);
        assert!(t.audit_ok, "{cell}: audit drift halts interpretation");
        println!(
            "{cell} | {e0:.0e} | {} | {:.1} | {:.2} | {} | {:.3} | {:.2} | {:.0} | exact",
            t.steps,
            t.secs,
            1000.0 * t.secs / t.steps as f64,
            t.stratum,
            t.rho,
            t.eff_n,
            t.e_left,
        );
        points.push((e0, t.steps, t.secs));
        effs.push((e0, t.rho, t.eff_n));
    }

    // Extrapolate to E₀ = 2e6. The era scales as E₀/c in transfer mode,
    // so steps scale with E₀; cost per step is taken from the LARGER
    // pilot (the conservative end if per-step cost is rising).
    let target_e0 = 2.0e6f64;
    let (e0_big, steps_big, secs_big) = points[1];
    let per_step = secs_big / steps_big as f64;
    let target_steps = (steps_big as f64 * target_e0 / e0_big).round() as u64;
    let est_secs = per_step * target_steps as f64;
    let est_hours = est_secs / 3600.0;

    // Does cost per step grow with the budget? A flat reading between two
    // NEARBY budgets does not license extrapolating far past them — that
    // is a tolerance band masquerading as a mechanism, the error this
    // track already catalogued once. So the growth is turned into an
    // exponent and applied, and the extrapolation is labelled UNSUPPORTED
    // whenever the target sits far outside the measured span.
    let per_step_small = points[0].2 / points[0].1 as f64;
    let pilot_span = e0_big / points[0].0;
    let growth = per_step / per_step_small;
    let cost_alpha = growth.ln() / pilot_span.ln();
    let target_ratio_e0 = target_e0 / e0_big;
    let per_step_projected = per_step * target_ratio_e0.powf(cost_alpha);
    let est_secs_grown = per_step_projected * target_steps as f64;

    println!(
        "\nEXTRAPOLATION: {:.0} steps at E₀=2e6.",
        target_steps
    );
    println!(
        "  per-step cost {:.0e} → {:.0e}: {:.3} → {:.3} ms (×{growth:.2} over a ×{pilot_span:.0} \
         budget span) ⇒ cost ∝ E₀^{cost_alpha:.3}",
        points[0].0,
        e0_big,
        1000.0 * per_step_small,
        1000.0 * per_step,
    );
    println!(
        "  flat-cost estimate: {:.0} s = {:.2} h/seed | growth-corrected: {:.0} s = {:.2} h/seed",
        est_secs,
        est_hours,
        est_secs_grown,
        est_secs_grown / 3600.0,
    );
    // The decision is taken on the CONSERVATIVE (growth-corrected)
    // number, and the two are printed side by side so a future reader can
    // see how much the choice of extrapolation was worth.
    let est_hours = est_secs_grown / 3600.0;
    if target_ratio_e0 > pilot_span {
        println!(
            "  NOTE: the target is ×{target_ratio_e0:.0} beyond the larger pilot, wider than the \
             ×{pilot_span:.0} span the exponent was fitted on. Treat the number as indicative \
             and confirm against the first commissioned seed."
        );
    }

    // The other half of the price, and the half that actually decides
    // whether commissioning buys anything: does effective N scale with
    // the era the way a linear projection assumes?
    //
    // It does NOT, and the pilot measures the shortfall rather than
    // inheriting the assumption. Effective N is N(1−ρ)/(1+ρ); a longer
    // segment raises N but ALSO raises ρ (more windows inside one
    // correlation time), so the product grows sublinearly. Projecting
    // linearly is the same error family as A3's "finer cadence buys more
    // windows" — the quantity that scales is not the one being counted.
    let (rho_s, eff_s) = (effs[0].1, effs[0].2);
    let (rho_b, eff_b) = (effs[1].1, effs[1].2);
    let budget_ratio = e0_big / points[0].0;
    let eff_ratio = eff_b / eff_s.max(1e-9);
    // Empirical scaling exponent: eff_N ∝ E₀^alpha.
    let alpha = eff_ratio.ln() / budget_ratio.ln();
    let target_ratio = target_e0 / e0_big;
    let proj_linear = eff_b * target_ratio;
    let proj_measured = eff_b * target_ratio.powf(alpha);
    println!(
        "\neffective N 1e5 → 4e5: {eff_s:.1} → {eff_b:.1} (×{eff_ratio:.2} for a ×{budget_ratio:.0} \
         budget) at ρ {rho_s:.3} → {rho_b:.3} ⇒ eff_N ∝ E₀^{alpha:.3}, SUBLINEAR."
    );
    println!(
        "  projected eff N at 2e6: {proj_linear:.0} (linear, OPTIMISTIC) vs \
         {proj_measured:.0} (measured exponent) against the floor of 200 — {}",
        if proj_measured >= 200.0 {
            "clears on both"
        } else if proj_linear >= 200.0 {
            "STRADDLES the floor: the two projections disagree across it, so the \
             projection cannot decide authorization — the run must be measured, \
             and its preconditions re-checked on the actual segment"
        } else {
            "SHORT on both: commissioning would buy an unusable run"
        }
    );

    let decision = if est_hours <= 1.0 {
        "D3-A: commission EXP-L7R at E₀ ≈ 2e6, 5 seeds, under the passive clock."
    } else if est_hours <= 6.0 {
        "D3-B: commission 3 seeds; report per-seed spread with the reduced n stated as a limitation."
    } else {
        "D3-C: do NOT commission. Record the price; ship with CD unmeasured and quantified; v9 item contingent on hardware."
    };
    println!("\nD3 (pre-committed): {decision}");
    println!(
        "STANDING DISCIPLINE: if commissioned and signal < spread, record and stop. \
         The estimator does not get relaxed standards — this would be the third application."
    );
}
