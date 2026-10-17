//! The pre-registered radius selection rule (A1.2).
//!
//! Frozen in `docs/atrack_a1_preregistration.md` §2, committed at `a43ca3d`
//! before any alphabet cardinality existed.
//!
//! > **R\* = the smallest R ∈ {1, 2, 3} with g(R) < ε, ε = 0.05**, where
//! > `g(R) = (|A(R+1)| − |A(R)|) / |A(R)|`.
//! > If no such R exists the alphabet **does not saturate within the swept
//! > range**, no R\* is reported, and the rule has REJECTED.
//!
//! The rejecting case is the point. Six documented instances in this project of
//! a guard passing for the wrong reason are why every criterion now ships with
//! a case where it rejects; `tests/a2_1_synthetic_firing.rs` supplies this
//! one's.

/// The committed refinement threshold. Not a tunable.
pub const EPSILON: f64 = 0.05;

/// Radii eligible to be selected. R = 0 is the no-context degenerate baseline;
/// R = 4 is unreachable because verifying saturation at R needs R+1 measured.
pub const ELIGIBLE: std::ops::RangeInclusive<usize> = 1..=3;

#[derive(Clone, Debug)]
pub struct Saturation {
    /// |A(R)| for R = 0..=R_MAX.
    pub sizes: Vec<usize>,
    /// g(R) for R = 0..R_MAX.
    pub gains: Vec<f64>,
    /// The selected radius, or `None` = "does not saturate within the swept
    /// range". `None` is a REPORTABLE OUTCOME, not an error.
    pub r_star: Option<usize>,
    /// Refinement monotonicity |A(R+1)| >= |A(R)|. A violation is an INSTRUMENT
    /// FAULT, not a finding: WL colours at round R+1 determine those at round R.
    pub monotone: bool,
    /// **Diagnostic, not a rule change.** The largest gain at any radius ABOVE
    /// the selected R\*. The committed rule takes the *smallest* passing R, so a
    /// run whose gain dips below ε at one radius and rises again above it is
    /// reported as saturating at the dip. The plan's A3.2 asks for "the point at
    /// which refinement stops adding symbols — or that it does not", and a bare
    /// R\* cannot express the second. This field can.
    pub max_gain_above_r_star: Option<f64>,
}

impl Saturation {
    pub fn saturates(&self) -> bool {
        self.r_star.is_some()
    }

    /// True when refinement RESUMES above R\*: the alphabet was not finished
    /// growing, and R\* marks a dip rather than a plateau. Must be reported
    /// wherever R\* is reported.
    pub fn refines_above_r_star(&self) -> bool {
        self.max_gain_above_r_star.is_some_and(|g| g >= EPSILON)
    }
}

/// Apply the committed rule to one alphabet-size sweep.
pub fn select_radius(sizes: &[usize]) -> Saturation {
    let monotone = sizes.windows(2).all(|w| w[1] >= w[0]);

    let gains: Vec<f64> = sizes
        .windows(2)
        .map(|w| {
            if w[0] == 0 {
                // An empty alphabet cannot be refined; report an infinite gain
                // only if the next radius is nonempty, else zero.
                if w[1] > 0 {
                    f64::INFINITY
                } else {
                    0.0
                }
            } else {
                (w[1] as f64 - w[0] as f64) / w[0] as f64
            }
        })
        .collect();

    let r_star = ELIGIBLE
        .clone()
        .find(|&r| gains.get(r).is_some_and(|&g| g < EPSILON));

    let max_gain_above_r_star = r_star.map(|r| {
        gains
            .iter()
            .skip(r + 1)
            .copied()
            .fold(0.0f64, |a, b| if b > a { b } else { a })
    });

    Saturation {
        sizes: sizes.to_vec(),
        gains,
        r_star,
        monotone,
        max_gain_above_r_star,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_selects_the_smallest_eligible_radius() {
        // R = 0's large gain must not fool it, and R = 1 is eligible.
        let s = select_radius(&[4, 40, 41, 41, 41]);
        assert_eq!(s.r_star, Some(1));
        assert!(s.monotone);
    }

    #[test]
    fn rule_rejects_a_still_refining_alphabet() {
        let s = select_radius(&[3, 6, 12, 24, 48]);
        assert_eq!(s.r_star, None, "a doubling alphabet must not be called saturated");
    }

    #[test]
    fn zero_radius_is_never_selected() {
        // g(0) = 0 (flat from the start) but R = 0 is not eligible; the rule
        // must select R = 1, the smallest ELIGIBLE radius that also passes.
        let s = select_radius(&[6, 6, 6, 6, 6]);
        assert_eq!(s.r_star, Some(1));
    }

    #[test]
    fn non_monotone_input_is_flagged_as_an_instrument_fault() {
        let s = select_radius(&[10, 9, 9, 9, 9]);
        assert!(!s.monotone);
    }
}
