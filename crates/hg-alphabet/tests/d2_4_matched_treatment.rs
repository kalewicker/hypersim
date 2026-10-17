//! **D2.4 — the matched treatment, measured against its control.**
//!
//! Same instrument, same S1/S2/S3, same R = 2, same C-track horizons
//! (`C_LADDER` 2000/8000 plus the 32000 power rung) so the saturation comparison
//! against `swap` is direct rather than quoted.
//!
//! Arms, each differing from the `swap` control in **exactly one** property:
//!
//! * **control** `swap` on the bare crystal — conserves the degree sequence,
//!   which on an unlabelled substrate is exactly what WL reads.
//! * **arm E′** `rewire_bounded` on the same bare crystal — the *rule* varies.
//!   (Arm E itself failed the D2.3 gate and gets no row, as `cyclic3` got none.)
//! * **arm F** `swap`, untouched, on a labelled crystal — the *substrate* varies.
//!
//! Headline is **`recurrent`** (D-track binding rule 2); `A_eff` is the
//! companion, with its `2^H(w)` share factor separated per C0.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test d2_4_matched_treatment -- --ignored --nocapture
//! ```

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::scaling::{classify_ceiling, classify_gap};
use hg_alphabet::{btrack, dtrack, measure, report, select_radius, RunAlphabets, RunSpec};

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
}

fn rung(spec: &RunSpec) -> Rung {
    let a = measure(spec).full;
    let s = &a.context[R];
    eprintln!("\n  --- {} ---", spec.name);
    eprintln!(
        "    steps={} stop={} n={} e={} events={}",
        a.steps_run, a.stop_reason, a.final_nodes, a.final_edges, a.n_events()
    );
    let (sweep, _) = report::radius_sweep("S1 context signature", &a.context);
    eprint!("{sweep}");
    eprintln!("    amendment 1: {}", r_star_or_budget_limited(&a));
    eprintln!("    [R=2] {}", report::composition(s));
    eprintln!(
        "    recurrent = {} of {}   S3 degree |A|={} A_eff={:.2}",
        s.cardinality() - s.singletons(),
        s.cardinality(),
        a.degree.cardinality(),
        a.degree.effective_size()
    );
    Rung {
        steps: a.steps_run,
        events: a.n_events(),
        raw: s.cardinality(),
        a_eff: s.effective_size(),
        singletons: s.singletons(),
        recurrent: s.cardinality() - s.singletons(),
        ceiling: s.ceiling_fraction(),
        degree_raw: a.degree.cardinality(),
    }
}

fn verdict_for(name: &str, lo: &Rung, hi: &Rung, long: &Rung) {
    let g = classify_gap(hi.ceiling, hi.a_eff, lo.recurrent as f64, hi.recurrent as f64);
    let c_long =
        classify_ceiling(&[(lo.events as f64, lo.ceiling), (long.events as f64, long.ceiling)]);
    eprintln!("\n  ======== {name} ========");
    eprintln!("    steps | events  |  |A|  | A_eff  | singl | recur | |A|/n_ev | S3 |A|");
    eprintln!("    ------+---------+-------+--------+-------+-------+----------+-------");
    for r in [lo, hi, long] {
        eprintln!(
            "    {:>5} | {:>7} | {:>5} | {:>6.2} | {:>5} | {:>5} | {:>8.4} | {:>5}",
            r.steps, r.events, r.raw, r.a_eff, r.singletons, r.recurrent, r.ceiling, r.degree_raw
        );
    }
    eprintln!("\n    C3.2 GAP VERDICT (2000/8000 pair)  : {}", g.line());
    eprintln!("    C3.3 scale-invariance (16x span)   : {}", c_long.line());
    eprintln!(
        "    HEADLINE recurrent across 1x/4x/16x: {} -> {} -> {}   (raw |A|: {} -> {} -> {})",
        lo.recurrent, hi.recurrent, long.recurrent, lo.raw, hi.raw, long.raw
    );
}

#[test]
#[ignore = "long run: D2.4 matched treatment"]
fn d2_4_measure_the_matched_treatment() {
    eprintln!("\n################################################################################");
    eprintln!("#  D-TRACK / D2.4 — THE MATCHED TREATMENT vs THE CONSERVATION CONTROL");
    eprintln!("#  Pre-registration: docs/dtrack_preregistration.md @ 2901f56 (+ D2.3a deviation)");
    eprintln!("################################################################################");

    let [lo, hi] = btrack::C_LADDER;
    let long = btrack::C_LADDER_LONG;

    let s = [rung(&btrack::swap_at(lo)), rung(&btrack::swap_at(hi)), rung(&btrack::swap_at(long))];
    verdict_for("CONTROL — swap on the bare crystal (conserves what WL reads)", &s[0], &s[1], &s[2]);

    let e = [
        rung(&dtrack::rewire_bounded_at(lo)),
        rung(&dtrack::rewire_bounded_at(hi)),
        rung(&dtrack::rewire_bounded_at(long)),
    ];
    verdict_for(
        "ARM E′ — rewire_bounded: the RULE varies (degree-changing). RECORDED DEVIATION",
        &e[0], &e[1], &e[2],
    );

    let f = [
        rung(&dtrack::swap_labelled_at(lo)),
        rung(&dtrack::swap_labelled_at(hi)),
        rung(&dtrack::swap_labelled_at(long)),
    ];
    verdict_for("ARM F — swap on a LABELLED crystal: the SUBSTRATE varies", &f[0], &f[1], &f[2]);

    eprintln!(
        "\n  ---- D2.4 pre-committed readings ----\n\
         \x20   saturates (|A| identical across rungs, recurrent flat) => the architectural claim\n\
         \x20     gets real support: the named defect in `swap`'s design was removed and the\n\
         \x20     result did not change.\n\
         \x20   accumulates (recurrent grows, |A|/n_events well below 1) => the gap IS occupied\n\
         \x20     and `swap`'s failure was the conservation defect and nothing more.\n\
         \x20   one arm accumulates and the other does not => the diagnosis's two halves separate;\n\
         \x20     report which of rule-invariant and substrate-labelling was binding.\n"
    );
    eprintln!("    arm            recurrent 1x -> 4x -> 16x     raw |A| 1x -> 4x -> 16x");
    for (name, r) in [("control swap", &s), ("arm E′ rewire", &e), ("arm F labelled", &f)] {
        eprintln!(
            "    {name:<14} {:>4} -> {:>4} -> {:>4}            {:>5} -> {:>5} -> {:>5}",
            r[0].recurrent, r[1].recurrent, r[2].recurrent, r[0].raw, r[1].raw, r[2].raw
        );
    }

    // The control's published C-track numbers, pinned. If `swap` stops being 17
    // symbols flat across 16x, the arms are being compared against something
    // other than what the C-track published.
    assert_eq!(s[0].raw, 17, "the control must reproduce the C-track's 17-symbol swap alphabet");
    assert_eq!(s[2].raw, 17, "the control's alphabet must still be byte-identical at 16x");

    eprintln!("\n################################################################################\n");
}
