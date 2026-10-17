//! Q7-5 (v7 §5): EXP-L7R — the closure defect's FIRST AUTHORIZED
//! attempt, on the coexistence stratum.
//!
//! Why this is not another premature attempt: every prior CD failure
//! traced to NONSTATIONARITY (the trajectory crossed phases mid-run, so
//! blocked CV admits the clock confound that killed the v2 number). The
//! coexistence cells are the first stationary dynamics the track has
//! produced — a steady state does not cross phases. A3 showed
//! re-windowing cannot manufacture effective samples, so the remaining
//! shortfall was purely era length, which transfer mode's E₀/c scaling
//! turns into a budget question.
//!
//! Configuration: swap regime, transfer ledger, k = 300 (Q7-1: the only
//! cells simultaneously IN RANGE for the wormhole census and long-lived;
//! A4: the longest Sparse stratum at 127 windows), E₀ scaled up, 5 seeds.
//!
//! PRE-REGISTERED PRECONDITIONS, all required before the estimator runs:
//!   P1 ≥ 200 consecutive windows that are Sparse-classified AND carry a
//!      standing population > 0, inside the PRE-QUENCH segment. The
//!      segment is production-rate-defined (n_subdivisions rising), never
//!      run-fraction-defined — the A4 driver's bug, explicitly excluded
//!      (hazard §8.3). The count is the **effective** sample
//!      N·(1−ρ)/(1+ρ), not the raw window count: A3 restated the floor
//!      that way precisely because adjacent windows are correlated, and
//!      applying the raw count here would re-import the error A3 closed.
//!      (The first version of this driver checked the raw count and
//!      declared a measurement that the restated floor rejects by 5×.)
//!   P2 stationarity: over that segment, |mean₁ − mean₂| / mean < 0.25
//!      and variance ratio ∈ [0.5, 2.0] across the segment's halves.
//!      A test, not an eyeball.
//!   P3 the countdown key is absent from the CD micro features — asserted
//!      by `micro_features_ignore_countdown_and_all_properties`
//!      (hg-causal), re-asserted structurally here.
//!
//! Interpretations are carried unchanged from the standing protocol and
//! are NOT decided here: CD_clock above seed spread = the first valid
//! leakage-in-bits measurement; CD ≈ 0 with population > 0 = the macro
//! screens off the standing spectrum (one macro refinement maximum);
//! signal < spread = record and stop.
//!
//! Run with --ignored; HYPERSIM_SWEEP_OUT persists.

use hg_causal::closure::{estimate_stratum, ClosureSettings, WindowSample};
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
    /// Longest run of consecutive windows satisfying P1.
    stratum: Vec<usize>,
    samples: Vec<WindowSample>,
    census: Vec<f64>,
    audit_ok: bool,
}

/// v8 §5: which clock the cell runs under. `"countdown"` is the v7
/// mechanism (kept so the archived attempt is reproducible bit for bit);
/// `"age"` is the passive clock, which is what a COMMISSIONED EXP-L7R
/// must use — CD on ticked dynamics measures a graph the instrument was
/// helping to shape.
fn gate() -> String {
    std::env::var("Q7_GATE").unwrap_or_else(|_| "countdown".into())
}

fn run(dir: &Path, seed: u64, k: i64, e_total: f64, steps: u64, window: u64) -> Cell {
    let gate = gate();
    let toml = format!(
        r#"
experiment_name = "q7_5_l7r"
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
e_total = {e_total}
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
    let a = run_headless(&cfg, None, None, None).expect("run");

    let leak = std::fs::read_to_string(a.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<String>> = leak
        .lines()
        .skip(2)
        .map(|l| l.split(',').map(str::to_string).collect())
        .collect();
    let audit_ok = rows.iter().all(|r| r[18].parse::<f64>().unwrap_or(1.0) == 0.0);
    let nsub: Vec<f64> = rows.iter().map(|r| r[15].parse().unwrap_or(0.0)).collect();
    let census: Vec<f64> = rows.iter().map(|r| r[10].parse().unwrap_or(0.0)).collect();

    // P1 mask: pre-quench (production rising) ∧ Sparse ∧ population > 0.
    let ok: Vec<bool> = (0..rows.len())
        .map(|i| {
            let producing = i > 0 && nsub[i] > nsub[i - 1];
            producing && rows[i][12] == "sparse" && census[i] > 0.0
        })
        .collect();
    // Longest consecutive run.
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
    // The stratum mask indexes leakage.csv rows and is applied to the
    // closure samples, so the two MUST be emitted 1:1 (they are: both
    // are pushed inside the same `lk.due(step)` block). A silent
    // misalignment would mask the wrong windows and quietly corrupt the
    // estimate, so it is asserted rather than assumed.
    assert_eq!(
        samples.len(),
        rows.len(),
        "closure samples must align 1:1 with leakage rows"
    );
    Cell { stratum: best, samples, census, audit_ok }
}

/// Mean lag-1 autocorrelation of the micro features over a stratum, and
/// the resulting EFFECTIVE sample size N·(1−ρ)/(1+ρ) — the A3-restated
/// window floor. Constant-valued feature dimensions are skipped (they
/// carry no information and have undefined autocorrelation).
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

/// P2, pre-registered: mean drift < 25% and variance ratio within 2×.
fn stationary(v: &[f64]) -> (bool, f64, f64) {
    let h = v.len() / 2;
    let (a, b) = (&v[..h], &v[h..]);
    let m = |x: &[f64]| x.iter().sum::<f64>() / x.len().max(1) as f64;
    let var = |x: &[f64], mu: f64| {
        x.iter().map(|y| (y - mu).powi(2)).sum::<f64>() / x.len().max(1) as f64
    };
    let (ma, mb) = (m(a), m(b));
    let overall = m(v);
    let (va, vb) = (var(a, ma), var(b, mb));
    let drift = if overall > 0.0 { (ma - mb).abs() / overall } else { f64::INFINITY };
    let ratio = if vb > 0.0 { va / vb } else { f64::INFINITY };
    (drift < 0.25 && (0.5..=2.0).contains(&ratio), drift, ratio)
}

#[test]
#[ignore]
fn q7_5_exp_l7r() {
    let root = std::env::var("HYPERSIM_SWEEP_OUT").ok();
    let tmp = tempfile::tempdir().unwrap();
    let k: i64 = std::env::var("Q7_K").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let e0: f64 = std::env::var("Q7_E0").ok().and_then(|s| s.parse().ok()).unwrap_or(4.0e5);
    let steps: u64 = std::env::var("Q7_STEPS").ok().and_then(|s| s.parse().ok()).unwrap_or(60000);
    // Cadence stays at A4's 100. A3 established that refining cadence
    // multiplies raw windows without buying effective samples
    // (autocorrelation 0.895 → 0.997), so the ≥200 windows must come
    // from ERA LENGTH — which transfer mode's E₀/c scaling supplies —
    // and never from finer slicing.
    let window: u64 = std::env::var("Q7_WINDOW").ok().and_then(|s| s.parse().ok()).unwrap_or(100);
    let seeds: Vec<u64> = std::env::var("Q7_SEEDS")
        .ok()
        .map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_else(|| vec![42, 43, 44, 45, 46]);

    // P3, asserted structurally before any run: no CD micro feature is a
    // property channel, so the monotone countdown clock cannot enter the
    // estimator (v6 §1.3; the invariance itself is pinned by
    // `micro_features_ignore_countdown_and_all_properties` in hg-causal).
    for f in hg_causal::closure::features::MICRO_FIELDS {
        assert!(
            !f.to_lowercase().contains("countdown") && !f.to_lowercase().contains("propert"),
            "P3: micro feature '{f}' must not carry a property/clock channel"
        );
    }

    // v8 §3.3: the same exclusion for the PASSIVE clock's channel. The
    // birth step is a FIELD, not a property, so the loop above does not
    // cover it; the guarantee is pinned by
    // `micro_features_ignore_birth_step_and_age` (hg-causal) and named
    // here so a passive-clock run carries the same structural assertion
    // the ticked one did.
    for f in hg_causal::closure::features::MICRO_FIELDS {
        assert!(
            !f.to_lowercase().contains("age") && !f.to_lowercase().contains("birth"),
            "P3: micro feature '{f}' must not carry an age/birth-step channel"
        );
    }

    println!(
        "EXP-L7R: swap × transfer × k={k} × E₀={e0:.0e} × window={window} × gate={}",
        gate()
    );
    println!("cell | audit | P1 stratum (raw/eff) | P2 drift/varratio | P1∧P2 | CD_raw | CD_clock | n_win | certainty");
    let mut clocks: Vec<f64> = Vec::new();
    let mut authorized = 0usize;
    for seed in seeds {
        let cell = format!("l7r_k{k}_s{seed}");
        let d = root
            .as_ref()
            .map(|r| {
                let p = Path::new(r).join(&cell);
                std::fs::create_dir_all(&p).unwrap();
                p
            })
            .unwrap_or_else(|| tmp.path().join(&cell));
        let c = run(&d, seed, k, e0, steps, window);
        assert!(c.audit_ok, "{cell}: audit drift halts interpretation");
        let strat_census: Vec<f64> = c.stratum.iter().map(|&i| c.census[i]).collect();
        let (stat_ok, drift, ratio) = if strat_census.len() >= 4 {
            stationary(&strat_census)
        } else {
            (false, f64::NAN, f64::NAN)
        };
        let (rho, eff) = effective_n(&c.samples, &c.stratum);
        // P1 on the RESTATED floor: effective, not raw, sample size.
        let p1 = eff >= 200.0;
        let go = p1 && stat_ok;
        let (mut raw, mut clk, mut nwin, mut cert) =
            (f64::NAN, f64::NAN, 0usize, "not-run".to_string());
        if go && !c.samples.is_empty() {
            let mask: Vec<bool> = (0..c.samples.len())
                .map(|i| c.stratum.contains(&i))
                .collect();
            let rep = estimate_stratum(
                &c.samples,
                &mask,
                "coexistence_steady",
                &ClosureSettings::default(),
            );
            raw = rep.cd_raw_bits;
            clk = rep.cd_clock_bits.unwrap_or(f64::NAN);
            nwin = rep.n_windows;
            cert = rep.certainty.clone();
            if clk.is_finite() {
                clocks.push(clk);
            }
            authorized += 1;
        }
        println!(
            "{cell} | exact | {} raw / {eff:.0} eff (ρ={rho:.3}) | {drift:.3}/{ratio:.2} | {} | \
             {raw:.4} | {clk:.4} | {nwin} | {cert}",
            c.stratum.len(),
            if go { "GO" } else if !p1 { "NO (P1 effective)" } else { "NO (P2)" },
        );
    }
    if clocks.len() >= 2 {
        let mean = clocks.iter().sum::<f64>() / clocks.len() as f64;
        let spread = clocks.iter().copied().fold(f64::MIN, f64::max)
            - clocks.iter().copied().fold(f64::MAX, f64::min);
        println!(
            "TUPLE: CD_clock mean {mean:.4}, sampler_spread {spread:.4} \
             (ENSEMBLE = sampler noise; the dynamical component is zero on a \
             zero-draw substrate, v9 §1.2), n_runs {} \
             — VERDICT: {}",
            clocks.len(),
            if mean.abs() > spread && mean > 0.0 {
                "SIGNAL ABOVE SPREAD (first valid leakage-in-bits measurement)"
            } else if mean <= 0.0 {
                "NEGATIVE (estimator pathology — never a finding)"
            } else {
                "SIGNAL < SPREAD (record and stop; no relaxed standards)"
            }
        );
    } else {
        println!("TUPLE: not computed — {authorized} seeds authorized (need ≥ 2)");
    }
}
