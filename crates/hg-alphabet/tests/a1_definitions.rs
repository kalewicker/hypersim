//! Invariants of the committed definitions (A1.1), asserted rather than assumed.
//!
//! These are the "predicted range" clauses of instrument-registry Rule 2 in
//! executable form: each one is a bound the MECHANISM says must hold, so a
//! violation is an instrument fault rather than a finding.

use hg_alphabet::signature::{class_maps, class_maps_reference, N_RADII};
use hg_alphabet::{measure, zoo};

/// The progressive class-map sweep must be BIT-IDENTICAL to calling the shipped
/// WL machinery once per radius. An optimisation that changes a published
/// number is not an optimisation.
#[test]
fn a1_progressive_class_maps_match_the_shipped_wl_machinery() {
    for init in [
        hg_alphabet::Init::PureCrystal { w: 6, h: 6, arity: 3, seed_label1: false },
        hg_alphabet::Init::PureCrystal { w: 5, h: 4, arity: 2, seed_label1: true },
        hg_alphabet::Init::GliderRing { n: 24 },
        hg_alphabet::Init::K4,
        hg_alphabet::Init::WhiteNoise {
            nodes: 40, edges: 90, max_arity: 3, label_range: 3, seed: 7, seed_label1: false,
        },
    ] {
        let store = init.build();
        let fast = class_maps(&store);
        let slow = class_maps_reference(&store);
        assert_eq!(fast.len(), N_RADII);
        for r in 0..N_RADII {
            assert_eq!(
                fast[r], slow[r],
                "progressive WL sweep diverged from the shipped machinery at radius {r} on {init:?}"
            );
        }
    }
}

/// Refinement monotonicity: WL colours at round R+1 determine those at round R,
/// so |A(R)| cannot fall as R rises. This is the pre-registration's §2 hard
/// bound; a violation is an INSTRUMENT FAULT.
#[test]
fn a1_alphabet_is_non_decreasing_in_radius() {
    for spec in [zoo::glider(), zoo::growth_treadmill().steps(40)] {
        let a = measure(&spec).full;
        assert!(a.engaged(), "{} produced no events", spec.name);
        for (variant, sizes) in [("S1", a.context_sizes()), ("S2", a.delta_sizes())] {
            assert!(
                sizes.windows(2).all(|w| w[1] >= w[0]),
                "{}/{variant}: |A(R)| fell as R rose: {sizes:?} — INSTRUMENT FAULT",
                spec.name
            );
        }
    }
}

/// S2 hashes the whole S1 input as a prefix, so S2 refines S1 and
/// |A_S2(R)| >= |A_S1(R)| always. Equality is a FINDING (the post-state adds
/// nothing beyond the matched context), not a bug — but the inequality is
/// structural and is asserted.
#[test]
fn a1_delta_signature_refines_the_context_signature() {
    for spec in [zoo::glider(), zoo::growth_treadmill().steps(40), zoo::triangle_subdivide().steps(20)] {
        let a = measure(&spec).full;
        assert!(a.engaged());
        let s1 = a.context_sizes();
        let s2 = a.delta_sizes();
        for r in 0..N_RADII {
            assert!(
                s2[r] >= s1[r],
                "{}: |A_S2({r})|={} < |A_S1({r})|={} — S2 hashes S1's input as a prefix, so this \
                 is impossible unless the signature construction is wrong",
                spec.name,
                s2[r],
                s1[r]
            );
        }
    }
}

/// Rule 2 hard bounds: 1 <= |A| <= n_events, 1 <= A_eff <= |A|, and S1 at any
/// radius refines the vacuous rule-only definition.
#[test]
fn a1_hard_bounds_hold() {
    for spec in [zoo::glider(), zoo::growth_treadmill().steps(40), zoo::bounded_churn().steps(60).burn_in(0)] {
        let a = measure(&spec).full;
        assert!(a.engaged(), "{} produced no events", spec.name);
        let n = a.n_events();
        for r in 0..N_RADII {
            let s = &a.context[r];
            assert!(s.cardinality() >= 1 && (s.cardinality() as u64) <= n, "{}: |A({r})| out of [1, n_events]", spec.name);
            let eff = s.effective_size();
            assert!(
                eff >= 1.0 - 1e-9 && eff <= s.cardinality() as f64 + 1e-9,
                "{}: A_eff({r}) = {eff} outside [1, |A|={}]",
                spec.name,
                s.cardinality()
            );
            assert!(
                s.cardinality() >= a.rule_only.cardinality(),
                "{}: S1 at R={r} is coarser than the rule-only definition it contains",
                spec.name
            );
        }
    }
}
