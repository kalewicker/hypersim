//! **A2.1 — the synthetic firing test.**
//!
//! Binding rule 6: every new criterion ships with a case where it *rejects*.
//! Six documented instances in this project of a guard passing for the wrong
//! reason are the reason. A saturation criterion that cannot reject cannot
//! support a verdict, and a growth classifier that only ever says PLATEAU is
//! measuring its own definition rather than the substrate.
//!
//! Streams here are constructed so their answers are known *by construction*,
//! and are fed to the same accounting code the real measurement uses.

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::growth::{self, Regime};
use hg_alphabet::saturation::{select_radius, EPSILON};

/// Build a `SymbolStats` whose cumulative-distinct curve is prescribed:
/// `new_at(e)` says how many brand-new symbols appear at cumulative event `e`.
fn stream(n_events: u64, mut new_at: impl FnMut(u64) -> u64) -> SymbolStats {
    let mut s = SymbolStats::new();
    let mut next: u64 = 0;
    let mut known: Vec<u64> = Vec::new();
    for e in 1..=n_events {
        let fresh = new_at(e);
        for _ in 0..fresh {
            next += 1;
            known.push(next);
        }
        // One occurrence per event: emit a fresh symbol if one is due, else
        // recycle a known one so the alphabet stops growing but events do not.
        // Before the first symbol exists there is nothing to record — the run
        // has events but no alphabet yet, which is a state the real harness can
        // also be in.
        if fresh > 0 {
            s.record(next, e, e);
        } else if !known.is_empty() {
            s.record(known[(e as usize) % known.len()], e, e);
        }
    }
    s
}

// ---------------------------------------------------------------- saturation

#[test]
fn a2_1_saturation_criterion_accepts_a_fixed_alphabet() {
    let s = select_radius(&[6, 6, 6, 6, 6]);
    assert_eq!(s.r_star, Some(1), "a fixed alphabet must saturate at the smallest eligible R");
    assert!(s.monotone);
}

/// **THE FIRING CASE.** A deliberately growing alphabet — `|A(R)| = 3·2^R`, so
/// every refinement step doubles the symbol count — must NOT be reported as
/// saturating. If this passes as "saturated", every A3 number is worthless.
#[test]
fn a2_1_saturation_criterion_rejects_a_growing_alphabet() {
    let sizes = [3usize, 6, 12, 24, 48];
    let s = select_radius(&sizes);
    assert_eq!(
        s.r_star, None,
        "FIRING TEST FAILED: the saturation criterion accepted |A(R)| = 3·2^R, which doubles at \
         every radius. A criterion that cannot reject cannot support a verdict."
    );
    for (r, g) in s.gains.iter().enumerate() {
        assert!(
            *g >= EPSILON,
            "gain at R={r} is {g}, below the threshold — the stream was not the one intended"
        );
    }
    eprintln!("  A2.1 firing case: |A(R)| = {sizes:?}  ->  R* = NONE (criterion rejected) ✓");
}

/// R = 0's large gain must not be mistaken for the run refining: R = 0 is the
/// no-context baseline and jumping off it is expected, not evidence.
#[test]
fn a2_1_baseline_radius_gain_does_not_fool_the_rule() {
    let s = select_radius(&[4, 40, 41, 41, 41]);
    assert_eq!(s.r_star, Some(1));
    assert!(s.gains[0] > 1.0, "the baseline gain in this fixture should be large");
}

/// The eligible window is closed at both ends for a reason, and both ends are
/// asserted: R = 0 can never be selected, and R = 4 can never be selected
/// because verifying saturation at R requires R+1 to have been measured.
#[test]
fn a2_1_eligible_window_is_closed_at_both_ends() {
    // Flat everywhere: g(0) = 0 is the smallest passing gain, yet R* must be 1.
    assert_eq!(select_radius(&[9, 9, 9, 9, 9]).r_star, Some(1));
    // Still refining all the way to the edge of the sweep: there is no g(4) to
    // check, so this must read as "does not saturate", never as R* = 4.
    let s = select_radius(&[2, 8, 32, 128, 512]);
    assert_eq!(s.r_star, None);
    assert_eq!(s.gains.len(), 4, "only g(0..=3) are measurable at R_MAX = 4");
    assert_eq!(*hg_alphabet::saturation::ELIGIBLE.end(), 3, "R* = 4 must be unreachable");
    // R = 3 IS eligible when its gain is measurable and small — that is a
    // measured saturation at the last verifiable radius, not the edge case.
    assert_eq!(select_radius(&[2, 8, 32, 128, 129]).r_star, Some(3));
}

// ------------------------------------------------------------- growth regime

#[test]
fn a2_1_growth_classifier_reports_plateau() {
    // 20 symbols in the first 200 events, nothing after.
    let s = stream(20_000, |e| u64::from(e <= 200 && e % 10 == 0));
    let f = growth::fit(&s.curve_by_event(200));
    assert_eq!(f.regime, Regime::Plateau, "{}", f.line());
    assert_eq!(s.cardinality(), 20);
}

#[test]
fn a2_1_growth_classifier_reports_linear() {
    // One new symbol every 10 events, forever.
    let s = stream(20_000, |e| u64::from(e % 10 == 0));
    let f = growth::fit(&s.curve_by_event(200));
    assert_eq!(f.regime, Regime::Linear, "{}", f.line());
    assert!(f.r2_loglog > 0.99, "a clean power law should fit well: {}", f.line());
}

/// **The second firing case.** A curve that accelerates must not be reported as
/// linear or plateauing — the whole point of A4 is to be able to distinguish
/// them, and the standing challenge is stated in exactly those terms.
#[test]
fn a2_1_growth_classifier_reports_accelerating() {
    // N(e) ∝ e², produced by letting the per-event novelty rate rise with e.
    let mut emitted: u64 = 0;
    let s = stream(20_000, |e| {
        let target = (e * e) / 40_000; // reaches 10_000 at e = 20_000
        let fresh = target.saturating_sub(emitted);
        emitted += fresh;
        fresh
    });
    let f = growth::fit(&s.curve_by_event(200));
    assert_eq!(
        f.regime,
        Regime::Accelerating,
        "FIRING TEST FAILED: a quadratic novelty curve was not called accelerating: {}",
        f.line()
    );
    eprintln!("  A2.1 firing case: N(e) ∝ e²  ->  {} ✓", f.line());
}

#[test]
fn a2_1_growth_classifier_refuses_to_guess_on_too_few_points() {
    let s = stream(6, |_| 1);
    let f = growth::fit(&s.curve_by_event(200));
    assert_eq!(f.regime, Regime::Insufficient, "{}", f.line());
}

// ------------------------------------------------------ accounting mechanics

#[test]
fn a2_1_effective_size_collapses_a_mass_concentrated_alphabet() {
    // 500 symbols, 99% of the mass on three of them: "a three-symbol alphabet
    // in practice", which is exactly what A_eff exists to say out loud.
    let mut s = SymbolStats::new();
    let mut e = 0u64;
    for i in 0..497u64 {
        e += 1;
        s.record(1000 + i, e, e);
    }
    for _ in 0..16_500 {
        for k in 0..3u64 {
            e += 1;
            s.record(k, e, e);
        }
    }
    assert_eq!(s.cardinality(), 500);
    assert!(
        s.effective_size() < 10.0,
        "A_eff = {:.2} should be near 3 on a distribution with 99% of its mass on three symbols",
        s.effective_size()
    );
    assert!(s.top_mass(3) > 0.98);
    assert_eq!(s.singletons(), 497);
}

#[test]
fn a2_1_novelty_curve_is_exact() {
    let s = stream(1000, |e| u64::from(e % 100 == 0));
    assert_eq!(s.distinct_by_event(0), 0);
    assert_eq!(s.distinct_by_event(99), 0);
    assert_eq!(s.distinct_by_event(100), 1);
    assert_eq!(s.distinct_by_event(550), 5);
    assert_eq!(s.distinct_by_event(1000), 10);
    assert_eq!(s.cardinality(), 10);
}
