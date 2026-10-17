//! **B0 — the instrument pin.** B-track binding rule 2: reuse the A-track
//! instrument unchanged, because any change voids comparison against A-track
//! numbers.
//!
//! "Unchanged" is enforced here rather than asserted in prose: this file
//! re-runs A-track specs and checks their **published** numbers reproduce
//! exactly. The B-track added an `Init` variant, an `event_cap` budget, a
//! `rule_only_symbol` accessor and report formatting; if any of those has
//! reached into the measurement, this test fails before a single B-track number
//! is reported.
//!
//! Sources for every pinned value: `docs/atrack_findings.md` A3 table and
//! `runs/atrack/a3_a4_zoo_measurement.txt`.

use hg_alphabet::alphabet::StopReason;
use hg_alphabet::{measure, zoo};

#[test]
fn b0_glider_reproduces_the_atrack_numbers() {
    let a = measure(&zoo::glider()).full;
    assert_eq!(a.n_events(), 2000, "glider event count moved");
    assert_eq!(a.steps_run, 2000);
    assert_eq!(a.stop_reason, StopReason::StepCap);
    assert_eq!(a.final_nodes, 24);
    assert_eq!(
        a.context_sizes(),
        vec![1, 3, 3, 3, 3],
        "glider S1 radius sweep moved from the published [1, 3, 3, 3, 3]"
    );
    assert_eq!(a.delta_sizes(), vec![1, 3, 3, 3, 3]);
    assert_eq!(a.degree.cardinality(), 1);
    assert_eq!(a.rule_only.cardinality(), 1);

    let sat = hg_alphabet::select_radius(&a.context_sizes());
    assert_eq!(sat.r_star, Some(1), "glider R* moved from the published 1");
    let s = &a.context[1];
    assert_eq!(s.cardinality(), 3);
    assert_eq!(s.singletons(), 0);
    // Pinned to the PUBLISHED precision (the A-track table prints A_eff at 2 dp).
    // The exact value is 2.99696, not 3.0: glider's three symbols are near-
    // uniform, not exactly uniform (top-1 mass 35.4% against 33.3% for a perfect
    // three-way split). The A-track prose called this "the theoretical maximum
    // evenness for its size", which the display rounding made look exact and
    // which the B-track corrects.
    assert!(
        (s.effective_size() - 3.00).abs() < 0.005,
        "glider A_eff = {} , published 3.00",
        s.effective_size()
    );
    assert!(
        s.effective_size() < 3.0,
        "glider's alphabet is not perfectly uniform; if it becomes so, the distribution moved"
    );
}

#[test]
fn b0_bounded_churn_reproduces_the_atrack_numbers() {
    let out = measure(&zoo::bounded_churn());
    let a = &out.full;
    assert_eq!(a.n_events(), 39_320, "bounded_churn whole-run event count moved");
    assert_eq!(a.final_nodes, 36, "n(t) is flat at 36 by construction");
    assert_eq!(a.final_edges, 24, "the published connectivity-thinning endpoint moved");
    assert_eq!(
        a.context_sizes(),
        vec![2, 28, 61, 95, 106],
        "bounded_churn S1 sweep moved from the published [2, 28, 61, 95, 106]"
    );
    assert_eq!(a.delta_sizes(), vec![2, 35, 76, 100, 109]);
    assert_eq!(a.rule_only.cardinality(), 2);
    assert_eq!(
        hg_alphabet::select_radius(&a.context_sizes()).r_star,
        None,
        "bounded_churn published R* = NONE (does not saturate)"
    );

    // The load-bearing A-track datum: post-burn-in.
    let b = out.post_burn_in.as_ref().expect("burn-in variant requested");
    assert_eq!(b.n_events(), 24_164, "post-burn-in event count moved");
    assert_eq!(
        b.context_sizes(),
        vec![2, 5, 12, 19, 23],
        "post-burn-in sweep moved from the published [2, 5, 12, 19, 23]"
    );
    let s = &b.context[2];
    assert_eq!(s.cardinality(), 12, "the headline 12 raw symbols moved");
    assert_eq!(s.singletons(), 0, "the headline zero singletons moved");
    assert!(
        (s.effective_size() - 3.57).abs() < 0.005,
        "the headline A_eff = {} , published 3.57",
        s.effective_size()
    );
    assert!(
        (s.top_mass(3) - 0.952).abs() < 0.001,
        "published top-3 mass 95.2%, got {:.3}",
        s.top_mass(3)
    );
}

/// The B-track's `event_cap` must be inert when unset — otherwise every A-track
/// spec silently changed meaning.
#[test]
fn b0_event_cap_is_inert_by_default_and_exact_when_set() {
    let base = zoo::glider().steps(300);
    assert_eq!(base.event_cap, u64::MAX, "event_cap must default to no cap");
    let uncapped = measure(&base).full;
    assert_eq!(uncapped.n_events(), 300);

    let capped = measure(&zoo::glider().steps(300).event_cap(117)).full;
    assert_eq!(
        capped.n_events(),
        117,
        "the event cap must truncate EXACTLY, not at the end of the crossing step"
    );
    assert_eq!(capped.stop_reason, StopReason::EventCap);

    // Truncation must be a prefix of the uncapped run, not a different run.
    for r in 0..hg_alphabet::N_RADII {
        assert!(
            capped.context[r].cardinality() <= uncapped.context[r].cardinality(),
            "a truncated run cannot have MORE symbols than the run it is a prefix of"
        );
    }
}
