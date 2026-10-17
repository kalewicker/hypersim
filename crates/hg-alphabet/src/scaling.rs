//! The two B-track verdict classifiers.
//!
//! Thresholds frozen in `docs/btrack_preregistration.md` §1.6 and §2.3,
//! committed at `6bb672d` before any B-track cardinality existed. Both ship
//! with firing tests (`tests/b1_firing.rs`) — a scaling verdict that cannot
//! return SUPER-LINEAR cannot support "the alphabet does not scale", and a
//! ceiling criterion that cannot return BUDGET ARTIFACT cannot earn
//! SCALE-INVARIANT.

// ----------------------------------------------------- B1.3 scaling verdict

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scaling {
    /// Fewer than 3 points, or a degenerate series.
    Insufficient,
    SubLinear,
    Linear,
    SuperLinear,
}

impl std::fmt::Display for Scaling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Scaling::Insufficient => "INSUFFICIENT",
            Scaling::SubLinear => "SUB-LINEAR / SATURATING",
            Scaling::Linear => "LINEAR",
            Scaling::SuperLinear => "SUPER-LINEAR",
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScalingFit {
    pub verdict: Scaling,
    /// log–log slope of `A_eff(k)`.
    pub exponent: f64,
    pub r2: f64,
    /// `A_eff(k_max) / A_eff(k_min)`.
    pub ratio: f64,
    pub n_points: usize,
}

impl ScalingFit {
    /// The verdict is never quoted without its fit quality, and never without
    /// the four-point caveat the pre-registration requires.
    pub fn line(&self) -> String {
        if self.verdict == Scaling::Insufficient {
            return format!("{} (n_points={})", self.verdict, self.n_points);
        }
        format!(
            "{:<24} b={:+.3} r2={:.3} ratio={:.2}x n={} — INDICATIVE, NOT DECISIVE ({} points)",
            self.verdict.to_string(),
            self.exponent,
            self.r2,
            self.ratio,
            self.n_points,
            self.n_points
        )
    }
}

/// Fit `A_eff` against rule count. `points` are `(rule_count, A_eff)`.
pub fn classify_scaling(points: &[(f64, f64)]) -> ScalingFit {
    let usable: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|&(k, a)| k >= 1.0 && a > 0.0)
        .collect();
    if usable.len() < 3 {
        return ScalingFit {
            verdict: Scaling::Insufficient,
            exponent: f64::NAN,
            r2: f64::NAN,
            ratio: f64::NAN,
            n_points: usable.len(),
        };
    }
    let logs: Vec<(f64, f64)> = usable.iter().map(|&(k, a)| (k.ln(), a.ln())).collect();
    let (b, r2) = ols(&logs);
    let ratio = usable.last().unwrap().1 / usable.first().unwrap().1;

    // Committed order of evaluation (pre-registration §1.6).
    let verdict = if b < 0.5 && ratio < 2.0 {
        Scaling::SubLinear
    } else if b <= 1.5 {
        Scaling::Linear
    } else {
        Scaling::SuperLinear
    };
    ScalingFit {
        verdict,
        exponent: b,
        r2,
        ratio,
        n_points: usable.len(),
    }
}

/// Slope and r² by OLS. A constant `y` is a perfect fit to a constant — detected
/// from the DATA (`y_max == y_min`), never from `Σ(y − ȳ)² == 0`, which leaks a
/// ~1e−28 denominator in log space. That defect was live in the A-track's growth
/// classifier and is not being reintroduced here.
fn ols(pts: &[(f64, f64)]) -> (f64, f64) {
    let n = pts.len() as f64;
    let y_min = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let y_max = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    if y_max == y_min {
        return (0.0, 1.0);
    }
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let syy: f64 = pts.iter().map(|p| (p.1 - my).powi(2)).sum();
    if sxx == 0.0 {
        return (0.0, f64::NAN);
    }
    (sxy / sxx, (sxy * sxy) / (sxx * syy))
}

// ------------------------------------------- B2.4 ceiling scale-invariance

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ceiling {
    /// The ladder does not span the required range.
    Insufficient,
    /// `|A|/n_events` fell materially as the budget grew: a longer horizon
    /// would resolve it, so the recurrence verdict stays suspended.
    BudgetArtifact,
    /// `|A|/n_events` did not move across a ≥16× budget increase: not a budget
    /// artifact but a property of the rule.
    ScaleInvariant,
    Indeterminate,
}

impl std::fmt::Display for Ceiling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Ceiling::Insufficient => "INSUFFICIENT LADDER",
            Ceiling::BudgetArtifact => "BUDGET ARTIFACT",
            Ceiling::ScaleInvariant => "SCALE-INVARIANT",
            Ceiling::Indeterminate => "INDETERMINATE",
        })
    }
}

/// Required span of the ladder in `n_events`, committed as ≥ 16×.
pub const MIN_SPAN: f64 = 16.0;
/// A fall of more than this in `|A|/n_events` is dilution.
pub const DILUTION: f64 = 0.05;
/// At or above this, the ceiling has not moved at all.
pub const PINNED: f64 = 0.99;

#[derive(Clone, Copy, Debug)]
pub struct CeilingFit {
    pub verdict: Ceiling,
    pub span: f64,
    pub first_ratio: f64,
    pub last_ratio: f64,
    pub drop: f64,
}

impl CeilingFit {
    pub fn line(&self) -> String {
        format!(
            "{:<20} span={:.1}x  |A|/n_events {:.4} -> {:.4}  fall={:+.4}",
            self.verdict.to_string(),
            self.span,
            self.first_ratio,
            self.last_ratio,
            -self.drop
        )
    }
}

// ---------------------------------------------------- C3.2 the gap criterion

/// Which class a C-track rule lands in. Thresholds and the evaluation order are
/// frozen in `docs/ctrack_preregistration.md` §3.2, committed before any
/// C3 number existed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gap {
    /// `|A|/n_events` well below 1 **and** recurrent count grows with horizon.
    Occupied,
    /// Below the ceiling but recurrence does not accumulate — bounded
    /// structure. A third class, not the target.
    Partial,
    /// `|A|/n_events → 1`: B2.1 generalises further than the zoo.
    GrowingInDisguise,
    /// Plateaus at a handful of symbols: bimodality holds.
    FlatInDisguise,
    Indeterminate,
}

impl std::fmt::Display for Gap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Gap::Occupied => "GAP OCCUPIED",
            Gap::Partial => "PARTIAL (bounded, no accumulation)",
            Gap::GrowingInDisguise => "GROWING RULE IN DISGUISE",
            Gap::FlatInDisguise => "FLAT RULE IN DISGUISE",
            Gap::Indeterminate => "INDETERMINATE",
        })
    }
}

/// `|A|/n_events` at or above this reads as riding the ceiling.
pub const GAP_CEILING: f64 = 0.99;
/// "Well below 1" — amendment 1's bar, reused.
pub const GAP_WELL_BELOW: f64 = 0.5;
/// Recurrent-count growth factor across the horizon ladder that counts as
/// accumulating rather than flat.
pub const GAP_GROWTH: f64 = 1.5;
/// `A_eff` at or below this is a handful of symbols.
pub const GAP_FLAT_AEFF: f64 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct GapFit {
    pub verdict: Gap,
    pub ceiling_hi: f64,
    pub a_eff_hi: f64,
    pub recurrent_lo: f64,
    pub recurrent_hi: f64,
    pub recurrence_growth: f64,
}

impl GapFit {
    pub fn line(&self) -> String {
        format!(
            "{:<34} |A|/n_ev={:.4}  A_eff={:.2}  recurrent {:.0}->{:.0} (x{:.2})",
            self.verdict.to_string(),
            self.ceiling_hi,
            self.a_eff_hi,
            self.recurrent_lo,
            self.recurrent_hi,
            self.recurrence_growth
        )
    }
}

/// Classify one rule from its two horizon rungs.
///
/// **Evaluation order, committed:** rule out the two "in disguise" readings
/// first — they say the rule belongs to an existing class — then decide between
/// gap and partial. Written down because an unstated precedence is a free
/// parameter.
pub fn classify_gap(
    ceiling_hi: f64,
    a_eff_hi: f64,
    recurrent_lo: f64,
    recurrent_hi: f64,
) -> GapFit {
    let recurrence_growth = if recurrent_lo > 0.0 {
        recurrent_hi / recurrent_lo
    } else if recurrent_hi > 0.0 {
        f64::INFINITY
    } else {
        1.0
    };
    let verdict = if ceiling_hi >= GAP_CEILING {
        Gap::GrowingInDisguise
    } else if a_eff_hi <= GAP_FLAT_AEFF {
        Gap::FlatInDisguise
    } else if ceiling_hi < GAP_WELL_BELOW && recurrence_growth >= GAP_GROWTH {
        Gap::Occupied
    } else if ceiling_hi < GAP_WELL_BELOW {
        Gap::Partial
    } else {
        Gap::Indeterminate
    };
    GapFit {
        verdict,
        ceiling_hi,
        a_eff_hi,
        recurrent_lo,
        recurrent_hi,
        recurrence_growth,
    }
}

/// `rungs` are `(n_events, |A|/n_events)`, ascending in `n_events`.
///
/// The reasoning the criterion encodes: **a budget artifact must dilute as the
/// budget grows; a structural property must not.**
pub fn classify_ceiling(rungs: &[(f64, f64)]) -> CeilingFit {
    let (Some(&(e0, r0)), Some(&(e1, r1))) = (rungs.first(), rungs.last()) else {
        return CeilingFit {
            verdict: Ceiling::Insufficient,
            span: 0.0,
            first_ratio: f64::NAN,
            last_ratio: f64::NAN,
            drop: f64::NAN,
        };
    };
    let span = if e0 > 0.0 { e1 / e0 } else { 0.0 };
    let drop = r1 - r0;
    let verdict = if rungs.len() < 2 || span < MIN_SPAN {
        Ceiling::Insufficient
    } else if drop < -DILUTION {
        Ceiling::BudgetArtifact
    } else if rungs.iter().all(|&(_, r)| r >= PINNED) {
        Ceiling::ScaleInvariant
    } else {
        Ceiling::Indeterminate
    };
    CeilingFit {
        verdict,
        span,
        first_ratio: r0,
        last_ratio: r1,
        drop,
    }
}
