//! Closure-defect estimator (LK4, plan §1.4 — the headline metric).
//!
//! CD = I( macro(w+1) ; micro(w) | macro(w) )  [bits / window]
//!
//! Estimated with a predictor pair: ridge regression (hand-rolled normal
//! equations, no new deps) predicting macro(w+1) from macro(w) alone vs
//! macro(w) ⊕ micro(w), Gaussian residual model, under BLOCKED
//! time-series cross-validation (contiguous folds, never shuffled).
//! CD = mean excess test log-likelihood per window / ln 2.
//!
//! CD ≈ 0 → the geometric layer is informationally closed at this
//! coarse-graining; CD > 0 → the substrate leaks, in bits. Negative CD is
//! an ESTIMATOR PATHOLOGY (finite-sample overfit of the augmented
//! model), flagged and never reported as a result. Below
//! `min_windows` the certainty is `Unknown` — same discipline as the EI
//! 1000-transition floor. Everything here is `[Inferred]` (analysis
//! layer only).

pub mod features;

use serde::{Deserialize, Serialize};

/// One window-aligned observation. `macro_v`/`micro_v` layouts are frozen
/// by `features::MACRO_FIELDS` / `features::MICRO_FIELDS`; `clock_v` by
/// [`CLOCK_FIELDS`] (v3 §1.2 — window index + phase one-hot, the
/// "what time is it" information a nonstationary trajectory leaks into
/// every trending micro feature). Empty `clock_v` disables the clock
/// baseline (`cd_clock_bits` = None).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WindowSample {
    pub macro_v: Vec<f64>,
    pub micro_v: Vec<f64>,
    pub clock_v: Vec<f64>,
}

/// Frozen clock layout (v3 §1.2): the macro+clock baseline's extra
/// features. Changing it is a spec revision (leakage_closure_spec.md).
pub const CLOCK_FIELDS: [&str; 2] = ["window_index", "phase_onehot[3]"];

/// Build the frozen clock vector for one window.
pub fn clock_features(window_index: usize, phase: hg_schema::LeakagePhase) -> Vec<f64> {
    use hg_schema::LeakagePhase;
    let mut v = vec![window_index as f64];
    for p in [LeakagePhase::Sparse, LeakagePhase::Crossover, LeakagePhase::Dense] {
        v.push(if p == phase { 1.0 } else { 0.0 });
    }
    v
}

#[derive(Clone, Copy, Debug)]
pub struct ClosureSettings {
    /// Below this many windows: certainty Unknown (default 200).
    pub min_windows: usize,
    pub cv_folds: usize,
    /// Ridge penalty applied on standardized features.
    pub ridge_lambda: f64,
}

impl Default for ClosureSettings {
    fn default() -> Self {
        Self {
            min_windows: 200,
            cv_folds: 5,
            ridge_lambda: 1e-2,
        }
    }
}

/// v9 §1.2: what a reported spread is a spread OVER. A `±` without one
/// of these is a draft, not a result — the rule exists because this
/// track reported `seed_spread` for three rounds on a substrate where
/// the seed does not vary the dynamics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpreadEnsemble {
    /// Across cross-validation folds of one fit.
    Folds,
    /// Across runs that differ only in the instrument's sampling draws —
    /// the dynamical component is zero on a zero-draw substrate.
    Sampler,
    /// Resampling of one series (e.g. the C0 moving-block bootstrap).
    Bootstrap,
    /// Across genuinely different initial conditions. Requires breaking
    /// the substrate's symmetry, so on a vertex-transitive torus it
    /// changes the system rather than the realization (C0 memo).
    Ic,
}

/// closure.json shape — v3 §1.2: CD is a TUPLE, not a scalar.
/// Interpretation requires `cd_clock_bits`; `cd_raw_bits` alone may not
/// be quoted in findings (frozen in docs/leakage_closure_spec.md).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClosureReport {
    /// Excess of macro+micro over macro-only (the v1 scalar).
    pub cd_raw_bits: f64,
    /// Excess of macro+micro over macro+clock (window index + phase
    /// label). A monotone-trend micro feature beats the macro baseline
    /// by telling time; it cannot beat the clock. `None` when no clock
    /// features were supplied.
    pub cd_clock_bits: Option<f64>,
    pub n_windows: usize,
    /// Spread of `cd_clock_bits` (max − min) over whatever ensemble
    /// `spread_ensemble` names; campaign-level, `None` in per-run
    /// reports. A CD claim requires signal > spread.
    ///
    /// RENAMED from `seed_spread_bits` in v9 C2.3. `serde(alias)` keeps
    /// archived `closure.json` readable. The old name asserted a
    /// replication that does not exist on a zero-draw substrate: varying
    /// the seed varies the sampled instruments and nothing else, so the
    /// dynamical component of this spread is identically zero (v8 B2,
    /// `exp_l18_seed_does_not_vary_the_dynamics`).
    #[serde(alias = "seed_spread_bits")]
    pub sampler_spread_bits: Option<f64>,
    /// v9 §1.2: **every spread states its ensemble.** What
    /// `sampler_spread_bits` is a spread OVER — one of `folds`,
    /// `sampler`, `bootstrap`, `ic`. `None` alongside a `None` spread;
    /// required whenever a spread is present.
    #[serde(default)]
    pub spread_ensemble: Option<SpreadEnsemble>,
    /// "all" | "sparse_only" (v3 §5.3 phase-stratified protocol).
    pub stratum: String,
    pub cv_folds: usize,
    pub macro_fields: Vec<String>,
    pub micro_fields: Vec<String>,
    pub clock_fields: Vec<String>,
    /// "Inferred" normally; "Unknown" below the window floor.
    pub certainty: String,
    /// closure_defect_negative suspicious flag: estimator pathology, not
    /// a finding. Keyed on cd_raw (the overfit-prone comparison).
    pub pathology_negative_cd: bool,
}

/// Contiguous blocked-CV fold boundaries over n transitions: k ranges,
/// in order, never shuffled. Public so the no-shuffle property is a
/// testable invariant, not a comment.
pub fn fold_ranges(n: usize, k: usize) -> Vec<std::ops::Range<usize>> {
    let k = k.max(1).min(n.max(1));
    let base = n / k;
    let rem = n % k;
    let mut out = Vec::with_capacity(k);
    let mut start = 0;
    for i in 0..k {
        let len = base + usize::from(i < rem);
        out.push(start..start + len);
        start += len;
    }
    out
}

/// Estimate the closure-defect tuple over all window-aligned samples
/// (stratum "all").
pub fn estimate(samples: &[WindowSample], settings: &ClosureSettings) -> ClosureReport {
    let mask = vec![true; samples.len()];
    estimate_stratum(samples, &mask, "all", settings)
}

/// Estimate over a window-mask stratum (v3 §5.3): only transitions
/// w → w+1 with BOTH windows in the stratum enter the fit, so a
/// phase-stratified CD never regresses across a phase boundary. Fold
/// blocking is over the retained transitions in time order (a fold may
/// straddle a masked-out gap; it never shuffles).
pub fn estimate_stratum(
    samples: &[WindowSample],
    mask: &[bool],
    stratum: &str,
    settings: &ClosureSettings,
) -> ClosureReport {
    assert_eq!(samples.len(), mask.len(), "one mask entry per window");
    // Transition dataset: predict macro(w+1) for retained transitions.
    let transitions: Vec<usize> = (0..samples.len().saturating_sub(1))
        .filter(|&i| mask[i] && mask[i + 1])
        .collect();
    let n = transitions.len();
    // The certainty floor counts what the fit actually sees: usable
    // transitions + 1 (identical to the v1 window count when unmasked).
    let n_windows = if n == 0 { 0 } else { n + 1 };
    let certainty = if n_windows < settings.min_windows {
        "Unknown"
    } else {
        "Inferred"
    };

    let have_clock = samples.iter().any(|s| !s.clock_v.is_empty());
    let mut cd_raw = 0.0;
    let mut cd_clock = None;
    if n >= 4 {
        let x_base: Vec<Vec<f64>> = transitions
            .iter()
            .map(|&i| samples[i].macro_v.clone())
            .collect();
        let x_clock: Vec<Vec<f64>> = transitions
            .iter()
            .map(|&i| {
                let mut v = samples[i].macro_v.clone();
                v.extend_from_slice(&samples[i].clock_v);
                v
            })
            .collect();
        let x_aug: Vec<Vec<f64>> = transitions
            .iter()
            .map(|&i| {
                let mut v = samples[i].macro_v.clone();
                v.extend_from_slice(&samples[i].micro_v);
                v
            })
            .collect();
        let y: Vec<Vec<f64>> = transitions
            .iter()
            .map(|&i| samples[i + 1].macro_v.clone())
            .collect();

        let mut excess_raw = 0.0; // nats
        let mut excess_clock = 0.0;
        let mut n_test_total = 0usize;
        for fold in fold_ranges(n, settings.cv_folds) {
            if fold.is_empty() {
                continue;
            }
            let train: Vec<usize> = (0..n).filter(|i| !fold.contains(i)).collect();
            if train.len() < 3 {
                continue;
            }
            let test: Vec<usize> = fold.collect();
            let ll_base = ridge_test_ll(&x_base, &y, &train, &test, settings.ridge_lambda);
            let ll_aug = ridge_test_ll(&x_aug, &y, &train, &test, settings.ridge_lambda);
            excess_raw += ll_aug - ll_base;
            if have_clock {
                let ll_clock =
                    ridge_test_ll(&x_clock, &y, &train, &test, settings.ridge_lambda);
                excess_clock += ll_aug - ll_clock;
            }
            n_test_total += test.len();
        }
        if n_test_total > 0 {
            cd_raw = excess_raw / n_test_total as f64 / std::f64::consts::LN_2;
            if have_clock {
                cd_clock = Some(excess_clock / n_test_total as f64 / std::f64::consts::LN_2);
            }
        }
    }

    ClosureReport {
        cd_raw_bits: cd_raw,
        cd_clock_bits: cd_clock,
        n_windows,
        sampler_spread_bits: None,
        spread_ensemble: None,
        stratum: stratum.to_string(),
        cv_folds: settings.cv_folds,
        macro_fields: features::MACRO_FIELDS.iter().map(|s| s.to_string()).collect(),
        micro_fields: features::MICRO_FIELDS.iter().map(|s| s.to_string()).collect(),
        clock_fields: if have_clock {
            CLOCK_FIELDS.iter().map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        },
        certainty: certainty.to_string(),
        pathology_negative_cd: cd_raw < 0.0,
    }
}

/// Fit ridge on the train rows, evaluate Gaussian test log-likelihood
/// (nats) on the test rows. Residual variances are per-output-dim, from
/// TRAIN residuals (floored to keep the density proper).
// Index-based loops are the clearest form for the normal-equations
// matrix assembly here; the "loop variable used to index" lints would
// only obscure it.
#[allow(clippy::needless_range_loop)]
fn ridge_test_ll(
    x: &[Vec<f64>],
    y: &[Vec<f64>],
    train: &[usize],
    test: &[usize],
    lambda: f64,
) -> f64 {
    let d = x[0].len();
    let k = y[0].len();

    // Standardize features on train stats (ridge is scale-sensitive);
    // constant features collapse to 0 and are carried by the intercept.
    let mut mean = vec![0.0; d];
    let mut std = vec![0.0; d];
    for &i in train {
        for j in 0..d {
            mean[j] += x[i][j];
        }
    }
    for m in mean.iter_mut() {
        *m /= train.len() as f64;
    }
    for &i in train {
        for j in 0..d {
            std[j] += (x[i][j] - mean[j]).powi(2);
        }
    }
    for s in std.iter_mut() {
        *s = (*s / train.len() as f64).sqrt();
        if *s < 1e-12 {
            *s = 1.0;
        }
    }
    let phi = |i: usize| -> Vec<f64> {
        // Standardized features + intercept.
        let mut v: Vec<f64> = (0..d).map(|j| (x[i][j] - mean[j]) / std[j]).collect();
        v.push(1.0);
        v
    };
    let dd = d + 1;

    // Normal equations: (Φ'Φ + λI)β = Φ'y  (no penalty on the intercept).
    let mut a = vec![vec![0.0; dd]; dd];
    let mut b = vec![vec![0.0; k]; dd];
    for &i in train {
        let f = phi(i);
        for r in 0..dd {
            for c in 0..dd {
                a[r][c] += f[r] * f[c];
            }
            for o in 0..k {
                b[r][o] += f[r] * y[i][o];
            }
        }
    }
    for r in 0..d {
        a[r][r] += lambda * train.len() as f64;
    }
    let beta = solve_multi(a, b); // dd x k

    // Train residual variances per output dim.
    let mut var = vec![0.0; k];
    for &i in train {
        let f = phi(i);
        for o in 0..k {
            let pred: f64 = (0..dd).map(|r| f[r] * beta[r][o]).sum();
            var[o] += (y[i][o] - pred).powi(2);
        }
    }
    for v in var.iter_mut() {
        *v = (*v / train.len() as f64).max(1e-9);
    }

    // Test log-likelihood (nats).
    let mut ll = 0.0;
    for &i in test {
        let f = phi(i);
        for o in 0..k {
            let pred: f64 = (0..dd).map(|r| f[r] * beta[r][o]).sum();
            let r2 = (y[i][o] - pred).powi(2);
            ll += -0.5 * (2.0 * std::f64::consts::PI * var[o]).ln() - 0.5 * r2 / var[o];
        }
    }
    ll
}

/// Gaussian elimination with partial pivoting, multiple right-hand
/// sides. Hand-rolled per the plan's "no new heavy deps" mandate.
#[allow(clippy::needless_range_loop)]
fn solve_multi(mut a: Vec<Vec<f64>>, mut b: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let n = a.len();
    let k = b[0].len();
    for col in 0..n {
        // Pivot.
        let pivot = (col..n)
            .max_by(|&r1, &r2| a[r1][col].abs().total_cmp(&a[r2][col].abs()))
            .unwrap();
        a.swap(col, pivot);
        b.swap(col, pivot);
        let p = a[col][col];
        if p.abs() < 1e-12 {
            continue; // singular direction: leave zeros (ridge prevents this)
        }
        for r in 0..n {
            if r == col {
                continue;
            }
            let factor = a[r][col] / p;
            for c in col..n {
                a[r][c] -= factor * a[col][c];
            }
            for o in 0..k {
                b[r][o] -= factor * b[col][o];
            }
        }
    }
    (0..n)
        .map(|r| {
            let p = a[r][r];
            (0..k)
                .map(|o| if p.abs() < 1e-12 { 0.0 } else { b[r][o] / p })
                .collect()
        })
        .collect()
}
