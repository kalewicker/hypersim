//! **B1 — does the alphabet scale with rule count?**
//!
//! The confound the B-track exists for: every A-track row had `A_rule_only` of
//! 1 or 2, so "the substrate's alphabet is small" was measured entirely as
//! context diversity, with the rule factor contributing nothing. Concluding
//! from it that the alphabet is architecturally bounded is close to concluding
//! a language has a small vocabulary after testing sentences built from one verb.
//!
//! Composition scheme, horizons and verdict thresholds: `docs/btrack_preregistration.md`
//! (`6bb672d`), committed before any of these numbers existed.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test b1_rule_count_series -- --ignored --nocapture
//! ```

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::scaling::classify_scaling;
use hg_alphabet::signature::rule_only_symbol;
use hg_alphabet::{btrack, fit_growth, measure, report, select_radius, RunAlphabets};

/// Amendment 1, adopted prospectively: `R*` is reported only when the run is
/// not riding its own `|A| ≤ n_events` ceiling.
fn r_star_or_budget_limited(a: &RunAlphabets) -> String {
    let sizes = a.context_sizes();
    let ceiling = a.context[hg_alphabet::N_RADII - 1].ceiling_fraction();
    if ceiling >= SymbolStats::CEILING_WARN {
        return format!("BUDGET-LIMITED (|A(4)|/n_events = {ceiling:.3} >= 0.5; amendment 1: no saturation verdict)");
    }
    match select_radius(&sizes).r_star {
        Some(r) => format!("R* = {r}"),
        None => "R* = NONE (does not saturate within the swept range)".into(),
    }
}

fn per_rule_share(a: &RunAlphabets, names: &[&str]) -> String {
    let total = a.rule_only.total_events().max(1);
    names
        .iter()
        .enumerate()
        .map(|(id, n)| {
            let c = a.rule_only.count_of(rule_only_symbol(id));
            format!("{n} {:.1}%", 100.0 * c as f64 / total as f64)
        })
        .collect::<Vec<_>>()
        .join("  ")
}

struct Row {
    k: usize,
    a_eff: f64,
    raw: usize,
    singletons: usize,
    events: u64,
    ok: bool,
}

fn run_series(label: &str, by_events: bool) -> Vec<Row> {
    eprintln!("\n================ B1 — RULE-COUNT SERIES, {label} ================");
    let mut rows = Vec::new();
    for k in btrack::COUNTS {
        let spec = if by_events {
            btrack::series_by_events(k)
        } else {
            btrack::series_by_steps(k)
        };
        let a = measure(&spec).full;
        let names = btrack::rule_names(k);

        eprintln!(
            "\n--- k = {k} rules [{}] ---\n  steps={} stop={} n={} e={} events={}",
            names.join(", "),
            a.steps_run,
            a.stop_reason,
            a.final_nodes,
            a.final_edges,
            a.n_events()
        );

        // B1.2's void check: A_rule_only must equal the rule count.
        let ar = a.rule_only.cardinality();
        let ok = ar == k;
        eprintln!(
            "  A_rule_only = {ar} (rule count {k}) -> {}",
            if ok { "row VALID" } else { "ROW VOID — a rule never fired" }
        );
        eprintln!("  per-rule event share: {}", per_rule_share(&a, &names));

        if !ok {
            rows.push(Row { k, a_eff: f64::NAN, raw: 0, singletons: 0, events: a.n_events(), ok });
            continue;
        }

        let (sweep, _) = report::radius_sweep("S1 context signature", &a.context);
        eprint!("{sweep}");
        eprintln!("  amendment 1: {}", r_star_or_budget_limited(&a));

        // Reported at R = 2 throughout the series: comparing rows at DIFFERENT
        // radii would confound rule count with resolution, and R* is not even
        // defined on every row. R = 2 is the A-track's stated fallback and is
        // fixed for all rows before any of them was read.
        let s = &a.context[2];
        eprintln!("  [R=2, fixed across the series] {}", report::composition(s));
        eprintln!(
            "  recurrent = {} of {}",
            s.cardinality() - s.singletons(),
            s.cardinality()
        );
        eprint!("{}", report::novelty(s));

        rows.push(Row {
            k,
            a_eff: s.effective_size(),
            raw: s.cardinality(),
            singletons: s.singletons(),
            events: a.n_events(),
            ok,
        });
    }
    rows
}

fn verdict(rows: &[Row], label: &str) {
    eprintln!("\n  ---- {label}: A_eff vs rule count ----");
    eprintln!("   k | events  |  |A|  |  A_eff  | singletons | recurrent");
    eprintln!("  ---+---------+-------+---------+------------+----------");
    for r in rows {
        if !r.ok {
            eprintln!("   {} | {:<7} |  VOID (a rule never fired)", r.k, r.events);
            continue;
        }
        eprintln!(
            "   {} | {:<7} | {:>5} | {:>7.2} | {:>10} | {:>8}",
            r.k,
            r.events,
            r.raw,
            r.a_eff,
            r.singletons,
            r.raw - r.singletons
        );
    }
    let pts: Vec<(f64, f64)> = rows
        .iter()
        .filter(|r| r.ok)
        .map(|r| (r.k as f64, r.a_eff))
        .collect();
    let fit = classify_scaling(&pts);
    eprintln!("\n  VERDICT ({label}): {}", fit.line());

    // DIAGNOSTIC, not the verdict. The pre-registration fits A_eff because the
    // A-track showed the raw count is the wrong number in five of eight rows.
    // The raw series is reported beside it because the two can disagree, and a
    // disagreement is informative: raw growing faster than effective means the
    // extra symbols carry little mass.
    let raw_pts: Vec<(f64, f64)> = rows
        .iter()
        .filter(|r| r.ok)
        .map(|r| (r.k as f64, r.raw as f64))
        .collect();
    let raw_fit = classify_scaling(&raw_pts);
    eprintln!("  diagnostic, raw |A| (NOT the verdict): {}", raw_fit.line());
}

#[test]
#[ignore = "long run: the B1 rule-count series"]
fn b1_alphabet_scaling_in_rule_count() {
    eprintln!("\n################################################################################");
    eprintln!("#  B-TRACK / B1 — ALPHABET SCALING IN RULE COUNT");
    eprintln!("#  Pre-registration: docs/btrack_preregistration.md @ 6bb672d");
    eprintln!("#  Substrate FIXED: CompositeChurnRing(6x6 arity3 fill 5.0) + GliderRing(24), n=60");
    eprintln!("#  Series NESTED: glider < +influx < +churn < +transfer");
    eprintln!("################################################################################");

    let primary = run_series(
        &format!("PRIMARY: equal event budget = {}", btrack::EVENT_BUDGET),
        true,
    );
    let secondary = run_series(
        &format!("SECONDARY: equal step budget = {}", btrack::STEP_BUDGET),
        false,
    );

    eprintln!("\n################################################################################");
    verdict(&primary, "PRIMARY, equal events");
    verdict(&secondary, "SECONDARY, equal steps");
    eprintln!("\n################################################################################\n");

    // Every row must be valid, or the series does not answer the question.
    for r in primary.iter().chain(secondary.iter()) {
        assert!(
            r.ok,
            "k={} row is VOID: A_rule_only != rule count, so the composition did not fire all its \
             rules and no scaling verdict may be read from this series",
            r.k
        );
    }

    // A no-op sanity floor: the 1-rule row must reproduce glider's alphabet,
    // which the A-track published as 3 symbols. If the composite substrate
    // changed what glider does, the series is not measuring rule count.
    let one = &primary[0];
    assert_eq!(
        one.raw, 3,
        "the 1-rule row must still be glider's published 3-symbol alphabet; got {}",
        one.raw
    );

    // Growth curves are quoted, so record that the abscissae exist.
    let _ = fit_growth(&[(1.0, 1.0)]);
}
