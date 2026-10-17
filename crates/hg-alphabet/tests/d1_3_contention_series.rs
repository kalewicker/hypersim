//! **D1.3 — the re-run.** Is B1's LINEAR verdict measuring node-budget contention?
//!
//! Three arms, same instrument, same S1 at R = 2, same horizons, same void rule:
//!
//! * **A — baseline**: B1 verbatim (n = 60). The anchor; if it does not
//!   reproduce B1's published numbers nothing else here is readable.
//! * **B — scaled**: n = 960, a 16× scale-up. The plan's specified arm, run
//!   **despite failing its own `demand ≤ 10% of n` requirement** (D1.2), so no
//!   row from it may be called low-contention.
//! * **C — reversed `rule_id`**: n = 60, the same rule *sets*, the conflict
//!   resolver's tie-break inverted. The substituted control — it varies
//!   contention alone and confounds nothing with size.
//!
//! Headline is **`recurrent`** (D-track binding rule 2); `A_eff` is the
//! companion, reported with its `2^H(w)` share factor separated per C0.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test d1_3_contention_series -- --ignored --nocapture
//! ```

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::scaling::classify_scaling;
use hg_alphabet::signature::rule_only_symbol;
use hg_alphabet::{btrack, dtrack, measure, select_radius, RunAlphabets, RunSpec};

const R: usize = 2;

fn r_star_or_budget_limited(a: &RunAlphabets) -> String {
    let ceiling = a.context[hg_alphabet::N_RADII - 1].ceiling_fraction();
    if ceiling >= SymbolStats::CEILING_WARN {
        return format!("BUDGET-LIMITED ({ceiling:.3})");
    }
    match select_radius(&a.context_sizes()).r_star {
        Some(r) => format!("R*={r}"),
        None => "R*=NONE".into(),
    }
}

struct Row {
    k: usize,
    events: u64,
    raw: usize,
    a_eff: f64,
    share: f64,
    within: f64,
    singletons: usize,
    recurrent: usize,
    ceiling: f64,
    shares: String,
    r_star: String,
    ok: bool,
}

fn run_row(spec: &RunSpec, k: usize, names: &[&str]) -> Row {
    let a = measure(spec).full;
    let s = &a.context[R];
    let d = a.share_decomposition(R);
    let total = a.rule_only.total_events().max(1);
    let shares = names
        .iter()
        .enumerate()
        .map(|(id, n)| {
            format!(
                "{n} {:.1}%",
                100.0 * a.rule_only.count_of(rule_only_symbol(id)) as f64 / total as f64
            )
        })
        .collect::<Vec<_>>()
        .join("  ");
    Row {
        k,
        events: a.n_events(),
        raw: s.cardinality(),
        a_eff: s.effective_size(),
        share: d.share_perplexity,
        within: d.within_sum,
        singletons: s.singletons(),
        recurrent: s.cardinality() - s.singletons(),
        ceiling: s.ceiling_fraction(),
        shares,
        r_star: r_star_or_budget_limited(&a),
        // B1.2's rule: `A_rule_only` must equal the rule count or the row is VOID.
        ok: a.rule_only.cardinality() == k,
    }
}

fn arm(label: &str, by_events: bool, mk: impl Fn(usize) -> RunSpec, reversed: bool) -> Vec<Row> {
    eprintln!("\n  ======== {label} ========");
    eprintln!("   k | events  |  |A|  | A_eff  | 2^H(w) | within | singl | recur | |A|/n_ev | R*");
    eprintln!("  ---+---------+-------+--------+--------+--------+-------+-------+----------+--------");
    let mut rows = Vec::new();
    for k in btrack::COUNTS {
        let names = if reversed {
            dtrack::reversed_rule_names(k)
        } else {
            btrack::rule_names(k)
        };
        let r = run_row(&mk(k), k, &names);
        eprintln!(
            "   {k} | {:>7} | {:>5} | {:>6.2} | {:>6.3} | {:>6.2} | {:>5} | {:>5} | {:>8.4} | {}{}",
            r.events, r.raw, r.a_eff, r.share, r.within, r.singletons, r.recurrent, r.ceiling,
            r.r_star,
            if r.ok { "" } else { "   << ROW VOID" }
        );
        eprintln!("     shares: {}", r.shares);
        rows.push(r);
    }
    let _ = by_events;
    rows
}

fn fits(label: &str, rows: &[Row]) {
    let pick = |f: &dyn Fn(&Row) -> f64| -> Vec<(f64, f64)> {
        rows.iter().filter(|r| r.ok).map(|r| (r.k as f64, f(r))).collect()
    };
    let recur = classify_scaling(&pick(&|r| r.recurrent as f64));
    let aeff = classify_scaling(&pick(&|r| r.a_eff));
    let within = classify_scaling(&pick(&|r| r.within));
    let raw = classify_scaling(&pick(&|r| r.raw as f64));
    eprintln!("\n    {label}");
    eprintln!("      HEADLINE recurrent : {}", recur.line());
    eprintln!("      companion  A_eff   : {}", aeff.line());
    eprintln!("      companion  within  : {}", within.line());
    eprintln!("      diagnostic raw |A| : {}", raw.line());
}

#[test]
#[ignore = "long run: D1.3 contention series, three arms x two horizons"]
fn d1_3_does_the_verdict_survive_relieved_contention() {
    eprintln!("\n################################################################################");
    eprintln!("#  D-TRACK / D1.3 — IS B1'S VERDICT MEASURING CONTENTION?");
    eprintln!("#  Pre-registration: docs/dtrack_preregistration.md @ 2901f56");
    eprintln!("#  Arm B's substrate FAILS D1.2's demand target and is labelled so on every row.");
    eprintln!("################################################################################");

    for (hlabel, by_events) in [("PRIMARY: equal events", true), ("SECONDARY: equal steps", false)] {
        eprintln!("\n\n################ {hlabel} ################");

        let a = arm(
            "ARM A — BASELINE (B1 verbatim, n = 60)",
            by_events,
            |k| if by_events { dtrack::baseline_by_events(k) } else { dtrack::baseline_by_steps(k) },
            false,
        );
        let b = arm(
            "ARM B — SCALED (n = 960; STILL HIGH-CONTENTION, influx demand 99.9% of n)",
            by_events,
            |k| if by_events { dtrack::scaled_by_events(k) } else { dtrack::scaled_by_steps(k) },
            false,
        );
        let c = arm(
            "ARM C — REVERSED rule_id (n = 60; contention varied, size held)",
            by_events,
            |k| if by_events { dtrack::reversed_by_events(k) } else { dtrack::reversed_by_steps(k) },
            true,
        );

        eprintln!("\n  ---- scaling fits, {hlabel} ----");
        fits("ARM A — BASELINE", &a);
        fits("ARM B — SCALED", &b);
        fits("ARM C — REVERSED", &c);

        // The anchor claim. B1 published the 1-rule row as glider's 3-symbol
        // alphabet; if the baseline no longer reproduces it, the arms are not
        // being read against what they are supposed to be read against.
        assert_eq!(
            a[0].raw, 3,
            "ARM A must reproduce B1's published 3-symbol glider row; got {}",
            a[0].raw
        );
    }

    eprintln!("\n################################################################################\n");
}
