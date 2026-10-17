//! **E3 — the measurement, and the hard stop.**
//!
//! Same instrument, same S1/S2/S3, same R = 2, **the D2.4 horizons
//! (2000 / 8000 / 32000)** so the comparison against `swap`, arm E′ and arm F
//! is direct rather than quoted.
//!
//! Headline is `recurrent`; `A_eff` is the companion. The alphabet is reported
//! at all three rungs so byte-identity across 16× is directly checkable.
//!
//! E2.2 already returned the **third pre-registered outcome**: `rebalance` is
//! live and perpetual (64 002 firings, 6 400 in the final 10%) but its degree
//! sequence changed on only **10 of 8 000 steps and 0 in the final 10%**,
//! against the `rewire_bounded` control's 7 998. So it is measured knowing that
//! it becomes a conservation rule after ~10 steps — which makes its alphabet
//! result mechanistic rather than mysterious either way.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test e3_measurement -- --ignored --nocapture
//! ```

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::scaling::{classify_ceiling, classify_gap};
use hg_alphabet::{btrack, dtrack, etrack, measure, report, select_radius, RunAlphabets, RunSpec};

const R: usize = 2;

fn r_star_or_budget_limited(a: &RunAlphabets) -> String {
    let ceiling = a.context[hg_alphabet::N_RADII - 1].ceiling_fraction();
    if ceiling >= SymbolStats::CEILING_WARN {
        return format!("BUDGET-LIMITED (|A(4)|/n_events = {ceiling:.3} >= 0.5; no saturation verdict)");
    }
    match select_radius(&a.context_sizes()).r_star {
        Some(r) => format!("R* = {r}"),
        None => "R* = NONE (does not saturate within the swept range)".into(),
    }
}

struct Rung {
    steps: u64,
    events: u64,
    raw: usize,
    a_eff: f64,
    singletons: usize,
    recurrent: usize,
    ceiling: f64,
    degree_raw: usize,
    /// The byte-level fingerprint. N1.3's lesson: scalars agreeing is not
    /// evidence the underlying structure agrees, so byte-identity across rungs
    /// is checked on the symbol set itself, not on `|A|`.
    symbols: Vec<u64>,
}

fn rung(spec: &RunSpec, verbose: bool) -> Rung {
    let a = measure(spec).full;
    let s = &a.context[R];
    if verbose {
        eprintln!("\n  --- {} ---", spec.name);
        eprintln!(
            "    steps={} stop={} n={} e={} events={}",
            a.steps_run, a.stop_reason, a.final_nodes, a.final_edges, a.n_events()
        );
        let (sweep, _) = report::radius_sweep("S1 context signature", &a.context);
        eprint!("{sweep}");
        eprintln!("    amendment 1: {}", r_star_or_budget_limited(&a));
        eprintln!("    [R=2] {}", report::composition(s));
        eprint!("{}", report::novelty(s));
    }
    Rung {
        steps: a.steps_run,
        events: a.n_events(),
        raw: s.cardinality(),
        a_eff: s.effective_size(),
        singletons: s.singletons(),
        recurrent: s.cardinality() - s.singletons(),
        ceiling: s.ceiling_fraction(),
        degree_raw: a.degree.cardinality(),
        symbols: s.sorted_symbols(),
    }
}

fn verdict_for(name: &str, r: &[Rung; 3]) -> bool {
    let g = classify_gap(r[1].ceiling, r[1].a_eff, r[0].recurrent as f64, r[1].recurrent as f64);
    let c = classify_ceiling(&[(r[0].events as f64, r[0].ceiling), (r[2].events as f64, r[2].ceiling)]);
    eprintln!("\n  ======== {name} ========");
    eprintln!("    steps | events  |  |A|  | A_eff  | singl | recur | |A|/n_ev | S3 |A|");
    eprintln!("    ------+---------+-------+--------+-------+-------+----------+-------");
    for x in r {
        eprintln!(
            "    {:>5} | {:>7} | {:>5} | {:>6.2} | {:>5} | {:>5} | {:>8.4} | {:>5}",
            x.steps, x.events, x.raw, x.a_eff, x.singletons, x.recurrent, x.ceiling, x.degree_raw
        );
    }
    // Byte-identity, checked on the symbol SET rather than on its size.
    let byte_identical = r[0].symbols == r[1].symbols && r[1].symbols == r[2].symbols;
    eprintln!(
        "\n    alphabet BYTE-IDENTICAL across 16x: {}   (symbol sets compared, not |A|)",
        if byte_identical { "YES" } else { "NO" }
    );
    eprintln!("    C3.2 GAP VERDICT (2000/8000 pair)  : {}", g.line());
    eprintln!("    C3.3 scale-invariance (16x span)   : {}", c.line());
    eprintln!(
        "    HEADLINE recurrent 1x/4x/16x: {} -> {} -> {}",
        r[0].recurrent, r[1].recurrent, r[2].recurrent
    );
    byte_identical
}

#[test]
#[ignore = "long run: E3 measurement"]
fn e3_measure_the_self_balancing_rule() {
    eprintln!("\n################################################################################");
    eprintln!("#  E-TRACK / E3 — THE MEASUREMENT, AND THE HARD STOP");
    eprintln!("#  Pre-registration: docs/etrack_preregistration.md @ 17b0c02");
    eprintln!("#  E1.2 inertness gate PASSED before any number below was produced.");
    eprintln!("################################################################################");

    let [lo, hi] = btrack::C_LADDER;
    let long = btrack::C_LADDER_LONG;

    let arm = [
        rung(&etrack::rebalance_at(lo), true),
        rung(&etrack::rebalance_at(hi), true),
        rung(&etrack::rebalance_at(long), true),
    ];
    let arm_frozen = verdict_for("ARM G — rebalance: self-balancing, ZERO constants", &arm);

    // The three D-track arms, re-measured at the same horizons so the
    // comparison is direct rather than quoted from `docs/dtrack_findings.md`.
    let swap = [
        rung(&btrack::swap_at(lo), false),
        rung(&btrack::swap_at(hi), false),
        rung(&btrack::swap_at(long), false),
    ];
    verdict_for("CONTROL — swap (conserves what WL reads)", &swap);
    let e_prime = [
        rung(&dtrack::rewire_bounded_at(lo), false),
        rung(&dtrack::rewire_bounded_at(hi), false),
        rung(&dtrack::rewire_bounded_at(long), false),
    ];
    verdict_for("ARM E′ — rewire_bounded (constants)", &e_prime);
    let labelled = [
        rung(&dtrack::swap_labelled_at(lo), false),
        rung(&dtrack::swap_labelled_at(hi), false),
        rung(&dtrack::swap_labelled_at(long), false),
    ];
    verdict_for("ARM F — swap @ labelled substrate", &labelled);

    eprintln!("\n  ================ THE FIVE-MECHANISM TABLE ================");
    eprintln!("    arm                       |A| 1x/4x/16x            recurrent 1x/4x/16x   S3 |A|");
    for (name, r) in [
        ("control swap", &swap),
        ("arm E′ rewire_bounded", &e_prime),
        ("arm F swap@labelled", &labelled),
        ("ARM G rebalance (new)", &arm),
    ] {
        eprintln!(
            "    {name:<25} {:>6} / {:>6} / {:>6}    {:>5} / {:>5} / {:>5}   {:>5}",
            r[0].raw, r[1].raw, r[2].raw, r[0].recurrent, r[1].recurrent, r[2].recurrent, r[0].degree_raw
        );
    }

    eprintln!("\n  ---- E3 pre-committed reading ----");
    let accumulates = arm[2].recurrent > arm[0].recurrent;
    if accumulates {
        eprintln!(
            "    ACCUMULATES => the D-track's architectural claim is FALSIFIED and rested on a\n\
             \x20   grammar limitation. Reported at full strength: recurrent {} -> {}.",
            arm[0].recurrent, arm[2].recurrent
        );
    } else {
        eprintln!(
            "    SATURATES => THE QUALIFIER COMES OFF. Five independent mechanisms, including the\n\
             \x20   one the grammar previously FORBADE, all saturate. The architectural claim\n\
             \x20   publishes without \"for rules expressible in the current grammar\".\n\
             \x20   HARD STOP: the arc closes, the write-up proceeds."
        );
    }

    // The claims, asserted so they cannot silently stop being true.
    assert_eq!(swap[0].raw, 17, "the control must reproduce the C-track's 17-symbol swap alphabet");
    assert!(
        !accumulates,
        "arm G's recurrent count GREW ({} -> {}); if this fires, the E3 falsification branch is \
         live and the D-track's architectural claim must be withdrawn",
        arm[0].recurrent, arm[2].recurrent
    );
    assert!(
        arm_frozen,
        "arm G saturated on |A| but its symbol SET differed across rungs — byte-identity is the \
         claim, and |A| agreeing is not evidence the structure agrees (N1.3)"
    );

    eprintln!("\n################################################################################\n");
}
