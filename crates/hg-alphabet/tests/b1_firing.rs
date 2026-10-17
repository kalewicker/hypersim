//! **Firing tests for the two new B-track criteria** (binding rule 5).
//!
//! Seven documented instances of guards passing for the wrong reason. A scaling
//! verdict that cannot return SUPER-LINEAR cannot support "adding rules does not
//! buy alphabet", and a ceiling criterion that cannot return BUDGET ARTIFACT
//! cannot earn SCALE-INVARIANT — each would be measuring its own definition.

use hg_alphabet::scaling::{classify_ceiling, classify_scaling, Ceiling, Scaling};

// -------------------------------------------------- B1.3 scaling verdict

#[test]
fn b1_scaling_reports_sub_linear_on_a_flat_series() {
    let f = classify_scaling(&[(1.0, 3.0), (2.0, 3.0), (3.0, 3.1), (4.0, 3.2)]);
    assert_eq!(f.verdict, Scaling::SubLinear, "{}", f.line());
    assert!(f.ratio < 2.0);
}

#[test]
fn b1_scaling_reports_linear_on_a_proportional_series() {
    let f = classify_scaling(&[(1.0, 3.0), (2.0, 6.0), (3.0, 9.0), (4.0, 12.0)]);
    assert_eq!(f.verdict, Scaling::Linear, "{}", f.line());
    assert!((f.exponent - 1.0).abs() < 1e-9, "{}", f.line());
    assert!(f.r2 > 0.999);
}

/// **THE FIRING CASE.** If a cubic series is not called SUPER-LINEAR, the
/// classifier cannot falsify the fixed-alphabet reading, and a sub-linear result
/// from it would be worthless.
#[test]
fn b1_scaling_reports_super_linear_on_a_cubic_series() {
    let f = classify_scaling(&[(1.0, 3.0), (2.0, 24.0), (3.0, 81.0), (4.0, 192.0)]);
    assert_eq!(
        f.verdict,
        Scaling::SuperLinear,
        "FIRING TEST FAILED: A_eff = 3k^3 was not called super-linear: {}",
        f.line()
    );
    eprintln!("  B1.3 firing case: A_eff = 3k^3 -> {} ✓", f.line());
}

/// The sub-linear branch has TWO clauses (`b < 0.5` AND `ratio < 2.0`), so a
/// series with a shallow slope but a large jump must NOT be called saturating.
#[test]
fn b1_scaling_sub_linear_needs_both_clauses() {
    // b ≈ 0.46 but the alphabet nearly doubled-and-a-half: not saturating.
    let f = classify_scaling(&[(1.0, 3.0), (2.0, 4.5), (3.0, 5.6), (4.0, 7.5)]);
    assert_ne!(
        f.verdict,
        Scaling::SubLinear,
        "a 2.5x rise must not pass as saturating on slope alone: {}",
        f.line()
    );
}

#[test]
fn b1_scaling_refuses_to_guess_on_two_points() {
    let f = classify_scaling(&[(1.0, 3.0), (4.0, 12.0)]);
    assert_eq!(f.verdict, Scaling::Insufficient, "{}", f.line());
}

// ------------------------------------------ B2.4 ceiling scale-invariance

/// **THE FIRING CASE.** A ratio that dilutes as the budget grows must be called
/// a BUDGET ARTIFACT. Without this the SCALE-INVARIANT verdict is unearnable,
/// and the whole point of B2.4 is to distinguish the two.
#[test]
fn b2_ceiling_reports_budget_artifact_when_the_ratio_dilutes() {
    let f = classify_ceiling(&[(400.0, 1.000), (1600.0, 0.720), (6400.0, 0.310)]);
    assert_eq!(
        f.verdict,
        Ceiling::BudgetArtifact,
        "FIRING TEST FAILED: a ratio falling 1.000 -> 0.310 over a 16x budget was not called a \
         budget artifact: {}",
        f.line()
    );
    eprintln!("  B2.4 firing case: 1.000 -> 0.310 over 16x -> {} ✓", f.line());
}

#[test]
fn b2_ceiling_reports_scale_invariant_when_pinned() {
    let f = classify_ceiling(&[(400.0, 1.000), (1600.0, 1.000), (6400.0, 1.000)]);
    assert_eq!(f.verdict, Ceiling::ScaleInvariant, "{}", f.line());
    assert!(f.span >= 16.0);
}

/// A ladder that does not actually span 16x cannot support either verdict, no
/// matter how flat it looks — otherwise "scale-invariant" would be claimable
/// from a budget that never changed.
#[test]
fn b2_ceiling_refuses_a_short_ladder() {
    let f = classify_ceiling(&[(400.0, 1.000), (1600.0, 1.000)]);
    assert_eq!(
        f.verdict,
        Ceiling::Insufficient,
        "a 4x ladder is pinned at 1.000 but spans far less than the committed 16x, so it cannot \
         support SCALE-INVARIANT: {}",
        f.line()
    );
}

#[test]
fn b2_ceiling_middle_ground_is_indeterminate() {
    // Fell a little, but not to dilution, and not pinned at 0.99 either.
    let f = classify_ceiling(&[(400.0, 0.980), (1600.0, 0.960), (6400.0, 0.950)]);
    assert_eq!(f.verdict, Ceiling::Indeterminate, "{}", f.line());
}
