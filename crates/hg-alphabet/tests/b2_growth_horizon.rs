//! **B2 — de-confound non-recurrence from run budget.**
//!
//! The A-track rejected its own "growing alphabet" branch because
//! `tetrahedral`, `growth_treadmill` and `@noise` emit unboundedly many symbols
//! with **zero recurrent** ones. But those rows sat at `|A|/n_events = 1.000`
//! on runs of 120, 400 and 3200 events over a substrate growing from 36 nodes.
//! Nothing can recur when there are barely more events than nodes, so zero
//! recurrence at that budget is what would be measured whether or not
//! recurrence exists at longer horizons. The rejection may well be correct; it
//! was not earned by that data.
//!
//! Ladder, readings and the B2.4 sub-test: `docs/btrack_preregistration.md` §2
//! (`6bb672d`), committed before any of these numbers existed.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test b2_growth_horizon -- --ignored --nocapture
//! ```

use hg_alphabet::scaling::classify_ceiling;
use hg_alphabet::{btrack, measure, report, zoo};

/// §2.1's structural fact, asserted arithmetically rather than hoped for: every
/// growing rule in the zoo mints exactly ONE node per firing, so
/// `n_nodes = n_0 + n_events` identically and `n_events / n_nodes → 1`.
///
/// The plan's stated target — a horizon where `n_events ≫ n_nodes` — is
/// therefore unreachable **by construction, not by compute**, and the decision
/// quantity has to be `|A|/n_events` directly rather than that proxy.
#[test]
fn b2_every_growing_zoo_rule_mints_one_node_per_event() {
    let cases = [
        ("growth_treadmill", zoo::growth_treadmill().steps(60), 36usize),
        ("triangle_subdivide", zoo::triangle_subdivide().steps(12), 36),
        ("binary_edge", zoo::binary_edge().steps(8), 36),
        ("tetrahedral", zoo::tetrahedral().steps(40), 4),
    ];
    eprintln!("\n  B2.1 — nodes minted per event, across every growing rule in the zoo:");
    for (name, spec, n0) in cases {
        let a = measure(&spec).full;
        assert!(a.engaged(), "{name} produced no events");
        let minted = a.final_nodes as i64 - n0 as i64;
        let events = a.n_events() as i64;
        eprintln!(
            "    {name:<20} n {n0} -> {:<7} events={:<7} minted/event = {:.4}",
            a.final_nodes,
            events,
            minted as f64 / events as f64
        );
        assert_eq!(
            minted, events,
            "{name}: nodes minted ({minted}) != events ({events}); the structural identity \
             n_nodes = n_0 + n_events does not hold and B2's framing must be revisited"
        );
    }
    eprintln!(
        "\n    => n_events / n_nodes -> 1 for EVERY growing rule in the zoo. The plan's\n\
         \x20      'n_events >> n_nodes' target is unreachable by CONSTRUCTION, not by compute.\n"
    );
}

#[test]
#[ignore = "long run: the B2 horizon ladder"]
fn b2_growth_treadmill_horizon_ladder() {
    eprintln!("\n################################################################################");
    eprintln!("#  B-TRACK / B2 — DOES NON-RECURRENCE SURVIVE A LONGER HORIZON?");
    eprintln!("#  Pre-registration: docs/btrack_preregistration.md @ 6bb672d");
    eprintln!("#  growth_treadmill, the A-track spec exactly, varying only the step cap.");
    eprintln!("################################################################################");

    let mut rungs: Vec<(f64, f64)> = Vec::new();
    eprintln!("\n  steps | n_nodes | events  |  |A|   | A_eff   | singl. | recur | |A|/n_events | wall");
    eprintln!("  ------+---------+---------+--------+---------+--------+-------+--------------+------");
    for steps in btrack::B2_LADDER {
        let t0 = std::time::Instant::now();
        let a = measure(&btrack::growth_treadmill_at(steps)).full;
        let secs = t0.elapsed().as_secs_f64();
        assert!(a.engaged(), "rung {steps} produced no events");
        // R = 2 throughout, the A-track's stated fallback, fixed across rungs so
        // the ladder varies only the horizon.
        let s = &a.context[2];
        eprintln!(
            "  {:>5} | {:>7} | {:>7} | {:>6} | {:>7.2} | {:>6} | {:>5} | {:>12.4} | {:>5.1}s",
            steps,
            a.final_nodes,
            a.n_events(),
            s.cardinality(),
            s.effective_size(),
            s.singletons(),
            s.cardinality() - s.singletons(),
            s.ceiling_fraction(),
            secs
        );
        rungs.push((a.n_events() as f64, s.ceiling_fraction()));

        if steps == *btrack::B2_LADDER.last().unwrap() {
            eprintln!("\n  --- top rung detail ---");
            eprint!("{}", report::run_block(&format!("growth_treadmill @ {steps} steps"), &a));
            eprint!("{}", report::novelty(s));
        }
    }

    let fit = classify_ceiling(&rungs);
    eprintln!("\n  B2.4 SCALE-INVARIANCE SUB-TEST: {}", fit.line());

    let first = rungs.first().expect("ladder is non-empty");
    let last = rungs.last().expect("ladder is non-empty");
    eprintln!(
        "\n  B2.3 pre-committed reading, applied:\n\
         \x20   |A|/n_events at the top rung = {:.4} (bar for 'well below the ceiling' is < 0.5)",
        last.1
    );
    let reading = if last.1 < 0.5 && last.1 < first.1 {
        "RECURRENCE APPEARS / ceiling escaped — the A-track's serial-number reading is a budget \
         artifact and A5's branch rejection is WITHDRAWN"
    } else if last.1 < 0.5 {
        "NON-RECURRENCE PERSISTS below the ceiling — the serial-number reading is EARNED"
    } else {
        "THE CEILING CANNOT BE ESCAPED — BUDGET-LIMITED; the branch stays OPEN rather than \
         rejected, and the A-track's A5 must be amended to say so"
    };
    eprintln!("    => {reading}");
    eprintln!("\n################################################################################\n");
}
