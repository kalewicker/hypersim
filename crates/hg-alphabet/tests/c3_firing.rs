//! **Firing tests for the C3.2 gap criterion** (binding rule 5).
//!
//! The criterion has four readings and **every one of them needs a case where
//! it fires**. A criterion that cannot return GAP OCCUPIED cannot support "the
//! gap is not occupied", which is the C-track's most interesting possible
//! outcome; and one that cannot return FLAT RULE IN DISGUISE cannot support
//! "occupied". Seven documented instances in this project of a guard passing
//! for the wrong reason are why this file exists.

use hg_alphabet::scaling::{classify_gap, Gap};

/// **Firing case for the target reading.** Low ceiling, recurrence tripling
/// across the ladder, healthy effective size.
#[test]
fn c3_gap_criterion_returns_occupied() {
    let f = classify_gap(0.12, 48.0, 100.0, 300.0);
    assert_eq!(
        f.verdict,
        Gap::Occupied,
        "FIRING TEST FAILED: a rule far below the ceiling with tripling recurrence was not called \
         gap-occupied: {}",
        f.line()
    );
    eprintln!("  C3.2 firing case (occupied): {} ✓", f.line());
}

/// **Firing case for the outcome that would make the C-track's negative
/// interesting.** Same low ceiling, same big alphabet — but recurrence does not
/// accumulate.
#[test]
fn c3_gap_criterion_returns_partial_when_recurrence_is_flat() {
    let f = classify_gap(0.12, 48.0, 100.0, 105.0);
    assert_eq!(
        f.verdict,
        Gap::Partial,
        "FIRING TEST FAILED: flat recurrence must NOT read as gap-occupied: {}",
        f.line()
    );
    eprintln!("  C3.2 firing case (partial): {} ✓", f.line());
}

#[test]
fn c3_gap_criterion_returns_growing_in_disguise() {
    let f = classify_gap(1.000, 6400.0, 0.0, 0.0);
    assert_eq!(f.verdict, Gap::GrowingInDisguise, "{}", f.line());
}

#[test]
fn c3_gap_criterion_returns_flat_in_disguise() {
    let f = classify_gap(0.002, 3.57, 12.0, 12.0);
    assert_eq!(
        f.verdict,
        Gap::FlatInDisguise,
        "bounded_churn's published shape must read as a flat rule: {}",
        f.line()
    );
}

/// The precedence is load-bearing and is asserted, not left implicit: a rule
/// riding the ceiling is a growing rule **even if** its recurrence grew, and a
/// rule that plateaus at a handful of symbols is a flat rule **even if** it sits
/// far below the ceiling.
#[test]
fn c3_gap_criterion_precedence_is_pinned() {
    // Ceiling wins over growing recurrence.
    assert_eq!(classify_gap(0.995, 900.0, 10.0, 900.0).verdict, Gap::GrowingInDisguise);
    // A_eff <= 4 wins over a low ceiling with growing recurrence.
    assert_eq!(classify_gap(0.001, 3.9, 4.0, 12.0).verdict, Gap::FlatInDisguise);
    // Just above the flat bar, low ceiling, growing => occupied.
    assert_eq!(classify_gap(0.001, 4.1, 4.0, 12.0).verdict, Gap::Occupied);
}

/// Between the "well below" bar and the ceiling with nothing else decisive, the
/// criterion must decline rather than pick a side.
#[test]
fn c3_gap_criterion_declines_in_the_middle() {
    let f = classify_gap(0.75, 900.0, 100.0, 105.0);
    assert_eq!(f.verdict, Gap::Indeterminate, "{}", f.line());
}

/// Zero recurrence at both rungs is not "growth from nothing" — it is the
/// growing-rule signature, and the ceiling clause must catch it first.
#[test]
fn c3_zero_recurrence_does_not_manufacture_growth() {
    let f = classify_gap(0.30, 900.0, 0.0, 0.0);
    assert_eq!(f.recurrence_growth, 1.0);
    assert_eq!(f.verdict, Gap::Partial, "{}", f.line());
}
