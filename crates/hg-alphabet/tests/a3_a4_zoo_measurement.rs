//! **A3 + A4 — the measurement.** Runs the whole rule zoo, reports alphabet
//! cardinality at the pre-registered radius, the refinement sweep and its
//! saturation point, the composition beside the raw count, and the novelty
//! curve with its fitted growth regime.
//!
//! `#[ignore]` because it is the long run, not because it is optional:
//!
//! ```text
//! cargo test --release -p hg-alphabet --test a3_a4_zoo_measurement -- --ignored --nocapture
//! ```
//!
//! **This test refuses to report a number until the A2.2 calibration gate has
//! passed inside the same process.** The plan forbids reporting an alphabet
//! result before the calibration pair separates cleanly, and a gate in a
//! different test file is a gate somebody can forget to run.

use hg_alphabet::report;
use hg_alphabet::{fit_growth, measure, select_radius, zoo};

const MIN_SEPARATION: f64 = 4.0;

/// The A2.2 gate, re-run here as a precondition rather than trusted.
fn calibration_gate_passes() -> (bool, String) {
    let poor = measure(&zoo::glider()).full;
    let rich = measure(&zoo::triangle_subdivide()).full;
    if !poor.engaged() || !rich.engaged() {
        return (false, "a calibration arm produced no events".into());
    }
    let ps = poor.context_sizes();
    let rs = rich.context_sizes();
    for r in 1..ps.len() {
        let ratio = rs[r] as f64 / ps[r].max(1) as f64;
        if ratio < MIN_SEPARATION {
            return (
                false,
                format!("separation at R={r} is {ratio:.2}x, below the committed {MIN_SEPARATION}x"),
            );
        }
    }
    (true, format!("glider |A| = {ps:?} vs triangle_subdivide |A| = {rs:?}"))
}

#[test]
#[ignore = "long run: the full zoo measurement"]
fn a3_a4_rule_zoo_alphabet_measurement() {
    let (passed, detail) = calibration_gate_passes();
    eprintln!("\n######## A2.2 PRECONDITION ########\n  {detail}\n  gate: {}\n", if passed { "PASS" } else { "FAIL" });
    assert!(
        passed,
        "A2.2 calibration gate did not pass; the plan forbids reporting an alphabet result before \
         the pair separates cleanly. STOP AND DIAGNOSE."
    );

    eprintln!("\n################################################################################");
    eprintln!("#  A-TRACK / A3 + A4 — THE SUBSTRATE'S REALISED EVENT ALPHABET");
    eprintln!("#  Pre-registration: docs/atrack_a1_preregistration.md @ a43ca3d");
    eprintln!("#  Bare substrate, zero biology (firewall asserted in tests/a0_firewall.rs)");
    eprintln!("################################################################################");

    let mut summary: Vec<String> = Vec::new();

    for spec in zoo::all() {
        let outcome = measure(&spec);
        let a = &outcome.full;
        eprint!("{}", report::run_block(&spec.name, a));

        if !a.engaged() {
            summary.push(format!("{:<24} {}", spec.name, report::NOT_ENGAGED));
            continue;
        }

        // A0.1 byproduct — the NODE-state class count over time. A different
        // object from the event alphabet, and labelled as such so the two are
        // never conflated in the report.
        let ncc = &a.node_class_count;
        if let (Some(first), Some(last)) = (ncc.first(), ncc.last()) {
            let peak = ncc.iter().map(|(_, c)| *c).max().unwrap_or(0);
            eprintln!(
                "  [A0.1 byproduct] node-WL-class count M(t) at R=2: t={} -> {}   t={} -> {}   peak={}",
                first.0, first.1, last.0, last.1, peak
            );
        }

        // A3.1 — cardinality at the PRE-REGISTERED radius.
        let sat = select_radius(&a.context_sizes());
        let r_report = sat.r_star.unwrap_or(2);
        eprintln!(
            "  [A3.1] S1 at the pre-registered radius: {}",
            match sat.r_star {
                Some(r) => format!("R* = {r}"),
                None => "R* = NONE (does not saturate); reporting R = 2 as the stated fallback".into(),
            }
        );
        eprintln!("         {}", report::composition(&a.context[r_report]));

        // A4.1 / A4.2 — the novelty curve, both abscissae, with fit quality.
        eprintln!("  [A4.1/A4.2] novelty curve, S1 @ R={r_report}:");
        eprint!("{}", report::novelty(&a.context[r_report]));

        // A4.3 — the bounded-churn substrate reported separately, after burn-in.
        if let Some(b) = outcome.post_burn_in.as_ref() {
            eprintln!(
                "\n  [A4.3] POST-BURN-IN (step >= {}, the M1 correction: churn must be scored after \n         burn-in, since its startup transients dominate the class count)",
                zoo::CHURN_BURN_IN
            );
            if b.engaged() {
                let bsat = select_radius(&b.context_sizes());
                let br = bsat.r_star.unwrap_or(2);
                let (sweep, _) = report::radius_sweep("S1 context signature, post-burn-in", &b.context);
                eprint!("{sweep}");
                eprintln!("         {}", report::composition(&b.context[br]));
                eprint!("{}", report::novelty(&b.context[br]));
            } else {
                eprintln!("         {} — no events after burn-in", report::NOT_ENGAGED);
            }
        }

        let fit_step = fit_growth(&a.context[r_report].curve_by_step(200));
        let fit_evt = fit_growth(&a.context[r_report].curve_by_event(200));
        summary.push(format!(
            "{:<24} R*={:<5} |A|={:<6} A_eff={:<9.1} single={:<5} events={:<7} vs-step={:<13} vs-event={:<13} {}",
            spec.name,
            sat.r_star.map(|r| r.to_string()).unwrap_or_else(|| "none".into()),
            a.context[r_report].cardinality(),
            a.context[r_report].effective_size(),
            a.context[r_report].singletons(),
            a.n_events(),
            fit_step.regime.to_string(),
            fit_evt.regime.to_string(),
            if sat.refines_above_r_star() { "[refines above R*]" } else { "" }
        ));
    }

    eprintln!("\n################################################################################");
    eprintln!("#  SUMMARY — raw count NEVER without effective size, label NEVER without its fit");
    eprintln!("################################################################################");
    for line in &summary {
        eprintln!("  {line}");
    }
    eprintln!();
}
