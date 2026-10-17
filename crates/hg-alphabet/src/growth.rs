//! Novelty-curve growth-regime fit (A4.2).
//!
//! Frozen in `docs/atrack_a1_preregistration.md` §4. A label is never returned
//! without its exponent, both r² values, and the final-decade new fraction —
//! the plan's wording is "report it with the fit quality, not just a label",
//! and a bare label is what makes a growth claim unfalsifiable.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    /// Too few usable points in the fit window. NOT a growth claim.
    Insufficient,
    Plateau,
    Sublinear,
    Linear,
    Accelerating,
}

impl std::fmt::Display for Regime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Regime::Insufficient => "INSUFFICIENT",
            Regime::Plateau => "PLATEAU",
            Regime::Sublinear => "SUBLINEAR",
            Regime::Linear => "LINEAR",
            Regime::Accelerating => "ACCELERATING",
        };
        f.write_str(s)
    }
}

/// Minimum usable points in the final-decade window before a regime may be
/// claimed at all.
pub const MIN_POINTS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct GrowthFit {
    pub regime: Regime,
    /// b in N ≈ a·x^b, fitted on the final decade in log–log.
    pub exponent: f64,
    pub r2_loglog: f64,
    pub slope_linear: f64,
    pub r2_linear: f64,
    /// (N(x_max) − N(x_max/10)) / N(x_max).
    pub final_decade_new_frac: f64,
    pub n_points: usize,
}

impl GrowthFit {
    pub fn insufficient(n_points: usize) -> Self {
        Self {
            regime: Regime::Insufficient,
            exponent: f64::NAN,
            r2_loglog: f64::NAN,
            slope_linear: f64::NAN,
            r2_linear: f64::NAN,
            final_decade_new_frac: f64::NAN,
            n_points,
        }
    }

    /// One line carrying the label AND the fit quality, so the two cannot be
    /// separated in a report.
    pub fn line(&self) -> String {
        if self.regime == Regime::Insufficient {
            return format!("{} (n_points={})", self.regime, self.n_points);
        }
        format!(
            "{:<13} b={:+.3} r2_loglog={:.3} lin_slope={:+.4e} r2_lin={:.3} last_decade_new={:.1}% n={}",
            self.regime.to_string(),
            self.exponent,
            self.r2_loglog,
            self.slope_linear,
            self.r2_linear,
            100.0 * self.final_decade_new_frac,
            self.n_points
        )
    }
}

/// Ordinary least squares on `(x, y)`; returns `(slope, intercept, r2)`.
///
/// A constant `y` is a perfect fit to a constant (slope 0, r² = 1), which is the
/// honest reading of "the curve did not move". Constancy is detected from the
/// DATA (`y_max == y_min`), not from `Σ(y − ȳ)² == 0`: the mean of n identical
/// values is not bit-identical to that value in general (`Σ ln 3 / 61 ≠ ln 3`),
/// so the variance test leaks a denominator of ~1e−28 and reports r² ≈ 0 for a
/// flawless plateau. That defect was live and is what this comment exists for.
fn ols(pts: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = pts.len() as f64;
    if n < 2.0 {
        return (0.0, 0.0, f64::NAN);
    }
    let y_min = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let y_max = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    if y_max == y_min {
        return (0.0, y_min, 1.0);
    }
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx) * (p.0 - mx)).sum();
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let syy: f64 = pts.iter().map(|p| (p.1 - my) * (p.1 - my)).sum();
    if sxx == 0.0 {
        return (0.0, my, f64::NAN);
    }
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let r2 = if syy == 0.0 { 1.0 } else { (sxy * sxy) / (sxx * syy) };
    (slope, intercept, r2)
}

/// Fit the committed regime classifier to a cumulative-novelty curve.
///
/// `points` are `(x, N(x))` samples, ascending in x. The fit window is the
/// final decade, `x >= x_max / 10`.
pub fn fit(points: &[(f64, f64)]) -> GrowthFit {
    let Some(&(x_max, n_max)) = points.last() else {
        return GrowthFit::insufficient(0);
    };
    let window: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|&(x, y)| x >= x_max / 10.0 && x >= 1.0 && y >= 1.0)
        .collect();
    if window.len() < MIN_POINTS {
        return GrowthFit::insufficient(window.len());
    }

    let logs: Vec<(f64, f64)> = window.iter().map(|&(x, y)| (x.ln(), y.ln())).collect();
    let (b, _, r2_loglog) = ols(&logs);
    let (slope_linear, _, r2_linear) = ols(&window);

    let n_at_start = window.first().map(|p| p.1).unwrap_or(0.0);
    let final_decade_new_frac = if n_max > 0.0 {
        (n_max - n_at_start) / n_max
    } else {
        0.0
    };

    // Committed order of evaluation.
    let regime = if final_decade_new_frac < 0.05 && b < 0.15 {
        Regime::Plateau
    } else if b < 0.85 {
        Regime::Sublinear
    } else if b <= 1.15 {
        Regime::Linear
    } else {
        Regime::Accelerating
    };

    GrowthFit {
        regime,
        exponent: b,
        r2_loglog,
        slope_linear,
        r2_linear,
        final_decade_new_frac,
        n_points: window.len(),
    }
}
