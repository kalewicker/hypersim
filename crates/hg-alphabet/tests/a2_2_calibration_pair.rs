//! **A2.2 — the calibration gate.** The instrument must separate a known-poor
//! rule from a known-rich one before any A3 number is interpretable.
//!
//! | rule | expectation | why |
//! |---|---|---|
//! | `glider` | tiny, saturates immediately | flat node count, mints no fresh nodes, reuses a, b, c |
//! | `triangle_subdivide` | large, plausibly growing | explodes 36 → 396+, minting fresh label-0 structure |
//!
//! The bar was committed in `docs/atrack_a1_preregistration.md` §5.2 before any
//! cardinality existed: `|A_S1(R)|` for `triangle_subdivide` must be at least
//! **4×** `glider`'s at **every** R ∈ {1,2,3,4}, and the two must not receive the
//! same growth-regime label. If they do not separate, the instrument is not
//! measuring context diversity and the track stops to diagnose.

use hg_alphabet::report;
use hg_alphabet::{measure, zoo};

/// The committed minimum separation. A mechanism-derived floor (one substrate
/// mints fresh structure at every firing, the other permutes a fixed ring), not
/// a ratio fitted to observed values.
const MIN_SEPARATION: f64 = 4.0;

#[test]
fn a2_2_calibration_pair_separates() {
    let poor = measure(&zoo::glider()).full;
    let rich = measure(&zoo::triangle_subdivide()).full;

    eprintln!("\n================ A2.2 — CALIBRATION GATE (known-poor vs known-rich) ================");
    eprint!("{}", report::run_block("glider (known-poor)", &poor));
    eprint!("{}", report::run_block("triangle_subdivide (known-rich)", &rich));

    // Instrument-registry Rule 1: an instrument that never engaged cannot
    // support a measurement, and must not be read as "small alphabet".
    assert!(poor.engaged(), "glider produced no events — {}", report::NOT_ENGAGED);
    assert!(
        rich.engaged(),
        "triangle_subdivide produced no events — {}",
        report::NOT_ENGAGED
    );

    let ps = poor.context_sizes();
    let rs = rich.context_sizes();

    eprintln!("\n  R  | glider |A| | triangle |A| | ratio | bar");
    eprintln!("  ---+------------+--------------+-------+-----");
    for r in 1..ps.len() {
        let ratio = rs[r] as f64 / ps[r].max(1) as f64;
        eprintln!(
            "  {r}  | {:>10} | {:>12} | {:>5.1} | {}",
            ps[r],
            rs[r],
            ratio,
            if ratio >= MIN_SEPARATION { "pass" } else { "FAIL" }
        );
    }

    for r in 1..ps.len() {
        let ratio = rs[r] as f64 / ps[r].max(1) as f64;
        assert!(
            ratio >= MIN_SEPARATION,
            "A2.2 GATE FAILED at R={r}: triangle_subdivide |A|={} vs glider |A|={} is a {ratio:.2}× \
             separation, below the committed {MIN_SEPARATION}×. The instrument is not measuring \
             context diversity; nothing downstream is interpretable. STOP AND DIAGNOSE.",
            rs[r],
            ps[r]
        );
    }

    // Second clause of the committed bar: the two must not land in the same
    // growth regime. Equal cardinalities with different curves would be a
    // different failure from equal cardinalities alone, so both are checked.
    let r_star_poor = hg_alphabet::select_radius(&ps).r_star.unwrap_or(2);
    let r_star_rich = hg_alphabet::select_radius(&rs).r_star.unwrap_or(2);
    let poor_fit = hg_alphabet::fit_growth(&poor.context[r_star_poor].curve_by_step(200));
    let rich_fit = hg_alphabet::fit_growth(&rich.context[r_star_rich].curve_by_step(200));
    eprintln!("\n  glider   @R*={r_star_poor}: {}", poor_fit.line());
    eprintln!("  triangle @R*={r_star_rich}: {}", rich_fit.line());
    assert_ne!(
        poor_fit.regime, rich_fit.regime,
        "A2.2 GATE FAILED: the known-poor and known-rich rules received the same growth regime \
         label ({}). STOP AND DIAGNOSE.",
        poor_fit.regime
    );

    eprintln!("\n  A2.2 GATE: PASS — the instrument separates the calibration pair.");
    eprintln!("====================================================================================\n");
}

/// The trap, asserted rather than described: the vacuous definition ("which
/// rule fired") is bounded by the rules file and cannot come back large, while
/// the realised event space can. If the two ever coincide, the measurement has
/// silently degenerated into the definition the plan forbids.
#[test]
fn a2_2_realised_event_space_is_not_the_vacuous_definition() {
    let rich = measure(&zoo::triangle_subdivide()).full;
    assert!(rich.engaged());
    let vacuous = rich.rule_only.cardinality();
    let realised = rich.context_sizes();
    eprintln!(
        "\n  triangle_subdivide: A_rule_only = {vacuous} (bounded by the rules file) vs \
         realised S1 |A(R)| = {realised:?}"
    );
    assert_eq!(vacuous, 1, "the rule file has one rule, so the vacuous count is 1 by construction");
    for (r, &n) in realised.iter().enumerate().skip(1) {
        assert!(
            n > vacuous,
            "S1 at R={r} collapsed to the vacuous definition ({n} vs {vacuous}) — the measurement \
             would be unable to return `large` and is therefore not evidence"
        );
    }
}
