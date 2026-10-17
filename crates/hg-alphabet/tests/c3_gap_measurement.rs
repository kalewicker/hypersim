//! **C3 — the measurement.** Same instrument, same S1/S2/S3, same radius rule,
//! same order gate. Criterion and precedence committed at `61fc0ae`, before
//! any of these numbers existed.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test c3_gap_measurement -- --ignored --nocapture
//! ```

use hg_alphabet::alphabet::SymbolStats;
use hg_alphabet::scaling::{classify_ceiling, classify_gap, classify_scaling};
use hg_alphabet::{btrack, fit_growth, measure, report, select_radius, RunAlphabets, RunSpec};

const R: usize = 2;

/// Amendment 1, applied prospectively to every C-track row.
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
    within: f64,
    singletons: usize,
    recurrent: usize,
    ceiling: f64,
}

fn rung(spec: &RunSpec) -> Rung {
    let a = measure(spec).full;
    let s = &a.context[R];
    let d = a.share_decomposition(R);
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
        "    recurrent = {} of {}   within-rule A_eff sum = {:.2}   share 2^H(w) = {:.3}",
        s.cardinality() - s.singletons(),
        s.cardinality(),
        d.within_sum,
        d.share_perplexity
    );
    eprintln!(
        "    S3 degree signature: |A|={} A_eff={:.2}",
        a.degree.cardinality(),
        a.degree.effective_size()
    );
    eprint!("{}", report::novelty(s));
    Rung {
        steps: a.steps_run,
        events: a.n_events(),
        raw: s.cardinality(),
        a_eff: s.effective_size(),
        within: d.within_sum,
        singletons: s.singletons(),
        recurrent: s.cardinality() - s.singletons(),
        ceiling: s.ceiling_fraction(),
    }
}

fn verdict_for(name: &str, lo: &Rung, hi: &Rung, long: &Rung) {
    // C3.2 is read from the PRE-REGISTERED pair only.
    let g = classify_gap(hi.ceiling, hi.a_eff, lo.recurrent as f64, hi.recurrent as f64);
    // C3.3 needs a 16x span to fire at all; the third rung supplies it.
    let c_pre = classify_ceiling(&[(lo.events as f64, lo.ceiling), (hi.events as f64, hi.ceiling)]);
    let c_long =
        classify_ceiling(&[(lo.events as f64, lo.ceiling), (long.events as f64, long.ceiling)]);
    eprintln!("\n  ======== {name} ========");
    eprintln!("    steps | events  |  |A|  | A_eff  | within | singl | recur | |A|/n_ev");
    eprintln!("    ------+---------+-------+--------+--------+-------+-------+---------");
    for (r, tag) in [(lo, "pre-reg"), (hi, "pre-reg"), (long, "power")] {
        eprintln!(
            "    {:>5} | {:>7} | {:>5} | {:>6.2} | {:>6.2} | {:>5} | {:>5} | {:>7.4}  [{tag}]",
            r.steps, r.events, r.raw, r.a_eff, r.within, r.singletons, r.recurrent, r.ceiling
        );
    }
    eprintln!("\n    C3.2 GAP VERDICT (pre-registered 2000/8000 pair) : {}", g.line());
    eprintln!("    C3.3 on the pre-registered pair  : {}", c_pre.line());
    eprintln!("    C3.3 on the 16x span (power rung): {}", c_long.line());
    eprintln!(
        "    raw |A| across all three rungs: {} -> {} -> {}",
        lo.raw, hi.raw, long.raw
    );
}

#[test]
#[ignore = "long run: the C3 gap measurement"]
fn c3_measure_the_gap_rules() {
    eprintln!("\n################################################################################");
    eprintln!("#  C-TRACK / C3 — THE MEASUREMENT");
    eprintln!("#  Pre-registration: docs/ctrack_preregistration.md; criterion at 61fc0ae");
    eprintln!("#  cyclic3 cycle length = 3 (reported beside every number, per C1.2)");
    eprintln!("################################################################################");

    let [lo_steps, hi_steps] = btrack::C_LADDER;

    let swap_lo = rung(&btrack::swap_at(lo_steps));
    let swap_hi = rung(&btrack::swap_at(hi_steps));
    let swap_long = rung(&btrack::swap_at(btrack::C_LADDER_LONG));
    verdict_for(
        "swap (topology churn, 0 parameters)",
        &swap_lo,
        &swap_hi,
        &swap_long,
    );

    let cyc_lo = rung(&btrack::cyclic3_torus_at(lo_steps));
    let cyc_hi = rung(&btrack::cyclic3_torus_at(hi_steps));
    let cyc_long = rung(&btrack::cyclic3_torus_at(btrack::C_LADDER_LONG));
    verdict_for(
        "cyclic3 on torus (state churn, cycle length 3) — POST-HOC SUBSTRATE DEVIATION",
        &cyc_lo,
        &cyc_hi,
        &cyc_long,
    );

    // The pre-registered cyclic3 substrate failed the C2 gate, so it gets no
    // C3 row: a rule that self-terminates has no horizon behaviour to measure.
    eprintln!(
        "\n  cyclic3 @ 6x6 open (PRE-REGISTERED): no C3 row — failed the C2 gate\n\
         \x20   (59 events in 8000 steps, self-terminated). Reported, not measured further."
    );

    // C1's committed structural prediction for `swap`, checked here because it
    // is an alphabet claim rather than a trajectory one.
    let swap_a = measure(&btrack::swap_at(lo_steps)).full;
    eprintln!(
        "\n  C1.1 prediction — S3 degree signature DEGENERATE at |A| = 1: measured |A| = {}",
        swap_a.degree.cardinality()
    );

    eprintln!("\n################################################################################\n");
}

#[test]
#[ignore = "long run: C3.4, the B1 series extended to k = 5"]
fn c3_4_extend_the_b1_series_to_k5() {
    eprintln!("\n################################################################################");
    eprintln!("#  C-TRACK / C3.4 — B1 EXTENDED TO k = 5 (B-track open item 3)");
    eprintln!("#  Same CompositeChurnRing substrate, same flattened priorities, both horizons.");
    eprintln!("################################################################################");

    for (label, by_events) in [("PRIMARY: equal events", true), ("SECONDARY: equal steps", false)] {
        eprintln!("\n================ {label} ================");
        let mut pts_aeff: Vec<(f64, f64)> = Vec::new();
        let mut pts_recur: Vec<(f64, f64)> = Vec::new();
        let mut pts_within: Vec<(f64, f64)> = Vec::new();
        eprintln!("   k | events  |  |A|  | A_eff  | within | recurrent | A_rule_only");
        eprintln!("  ---+---------+-------+--------+--------+-----------+------------");
        for k in 1..=5usize {
            let spec = match (k, by_events) {
                (5, true) => btrack::series_k5_by_events(),
                (5, false) => btrack::series_k5_by_steps(),
                (_, true) => btrack::series_by_events(k),
                (_, false) => btrack::series_by_steps(k),
            };
            let a = measure(&spec).full;
            let s = &a.context[R];
            let d = a.share_decomposition(R);
            let ar = a.rule_only.cardinality();
            eprintln!(
                "   {k} | {:>7} | {:>5} | {:>6.2} | {:>6.2} | {:>9} | {ar} {}",
                a.n_events(),
                s.cardinality(),
                s.effective_size(),
                d.within_sum,
                s.cardinality() - s.singletons(),
                if ar == k { "" } else { "<< ROW VOID — a rule never fired" }
            );
            // B1.2's own rule: a row whose `A_rule_only` misses the rule count
            // is VOID and is REPORTED as void, not read as a small alphabet and
            // not silently dropped. It is excluded from every fit.
            if ar != k {
                continue;
            }
            pts_aeff.push((k as f64, s.effective_size()));
            pts_within.push((k as f64, d.within_sum));
            pts_recur.push((k as f64, (s.cardinality() - s.singletons()) as f64));
        }
        eprintln!(
            "\n  fits over the {} VALID rows only:",
            pts_aeff.len()
        );
        eprintln!("  A_eff      : {}", classify_scaling(&pts_aeff).line());
        eprintln!("  within_sum : {}", classify_scaling(&pts_within).line());
        eprintln!("  recurrent  : {}", classify_scaling(&pts_recur).line());
        if pts_aeff.len() < 5 {
            eprintln!(
                "\n  k = 5 is VOID: `swap` binds 4 nodes per firing and the composite crystal has\n\
                 \x20  exactly 36, so under MaxParallel node-disjointness it and the crystal rules are\n\
                 \x20  MUTUALLY EXCLUSIVE — see tests/c3_4_composition_probe.rs. B-track open item 3\n\
                 \x20  is NOT closed: `swap` meets every B1.1 criterion for a fifth rule and still\n\
                 \x20  cannot be composed with the k = 4 set on this substrate."
            );
        }
    }
    let _ = fit_growth(&[(1.0, 1.0)]);
    eprintln!("\n################################################################################\n");
}
