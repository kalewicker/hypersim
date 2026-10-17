//! **D1.1 — quantify the node budget, before any re-run.**
//!
//! C3.4 established `swap`'s demand by hand: 4 nodes per firing × 9 firings per
//! step = 36 = the whole crystal, so `swap` and every crystal-resident rule are
//! mutually exclusive under `MaxParallel` node-disjointness. That was an
//! argument about one rule. This makes it a **measured quantity for all five**,
//! so "B1's shares are contention" stops being an inference from the shares
//! themselves.
//!
//! Each rule is run **alone** on the B1 substrate. Alone is the right condition:
//! demand is what a rule *asks for*, and once other rules are present the node
//! budget has already been spent, which is the thing being tested for.
//!
//! Reported per rule: LHS arity (read off the compiled rule, not the TOML),
//! mean and PEAK firings/step, node-demand/step = arity × firings, and demand as
//! a fraction of `n`. **Peak is the number that decides contention** — the
//! budget binds at the moment of simultaneous demand, not on average.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test d1_1_node_demand -- --ignored --nocapture
//! ```

use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_alphabet::btrack;
use hg_alphabet::run::Init;
use hg_engine::Engine;
use std::sync::{Arc, Mutex};

/// Firings per step, in order. Nothing else — this is a fire-count probe, the
/// same discipline `tests/b1_liveness_probe.rs` and `docs/policy_config.md` §2.5
/// use to fix a composition without reading any alphabet.
struct PerStep {
    counts: Arc<Mutex<Vec<u64>>>,
}

impl Observer for PerStep {
    fn on_step_end(&mut self, _s: &hg_core::GraphSnapshot<'_>, _st: &StepStats, events: &[RewriteEvent]) {
        self.counts.lock().expect("probe poisoned").push(events.len() as u64);
    }
}

struct Demand {
    name: &'static str,
    arity: usize,
    n: usize,
    total: u64,
    mean_per_step: f64,
    /// Mean over the first `EARLY` steps — a rule with a finite per-node budget
    /// (`influx` adds 1.0 per firing against a `< 20` guard from a fill of 5.0,
    /// so it has exactly 15 firings per node available) decays to nothing, and a
    /// whole-run mean would report that decay as low demand.
    early_per_step: f64,
    peak: u64,
}

impl Demand {
    fn peak_demand(&self) -> usize {
        self.arity * self.peak as usize
    }
    fn early_demand(&self) -> f64 {
        self.arity as f64 * self.early_per_step
    }
}

const EARLY: usize = 50;
const PROBE_STEPS: u64 = 500;

fn demand(name: &'static str, toml: &str, init: &Init) -> Demand {
    let mut store = init.build();
    let n = store.node_count();
    let parsed = parse_rules_toml(toml).expect("parse");
    let rules = compile_rules(&parsed).expect("compile");
    let arity = rules
        .iter()
        .map(|r| r.lhs_var_names.len())
        .max()
        .expect("at least one rule");

    let counts = Arc::new(Mutex::new(Vec::new()));
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    engine.add_observer(Box::new(PerStep { counts: Arc::clone(&counts) }));
    for _ in 0..PROBE_STEPS {
        engine.step(&mut store);
    }

    let c = counts.lock().expect("probe poisoned").clone();
    let total: u64 = c.iter().sum();
    let early: u64 = c.iter().take(EARLY).sum();
    Demand {
        name,
        arity,
        n,
        total,
        mean_per_step: total as f64 / PROBE_STEPS as f64,
        early_per_step: early as f64 / EARLY.min(c.len()).max(1) as f64,
        peak: c.iter().copied().max().unwrap_or(0),
    }
}

/// The four B1 rules, each isolated. `btrack::ruleset` is nested, so rule `k`
/// alone is not `ruleset(k)`; the singles are assembled from the same text by
/// difference, which keeps one source of truth for the rule bodies.
fn single(k: usize) -> String {
    let all = btrack::ruleset(4);
    // Each rule body starts at its own `[[rules]]` header. `parts[0]` is whatever
    // precedes the first header — `rules/glider.toml` opens with a comment block —
    // so rule `k` is `parts[k + 1]`, not `parts[k]`.
    let parts: Vec<&str> = all.split("[[rules]]").collect();
    assert_eq!(parts.len(), 5, "the k=4 file must contain exactly four rule headers");
    format!("[[rules]]{}", parts[k + 1])
}

/// Firings of ONE named rule inside a two-rule file, plus that rule's arity.
/// Returns `(total_firings, peak_firings_per_step, arity)`.
fn paired_demand(toml: &str, init: &Init, want: &str) -> (u64, u64, usize) {
    let mut store = init.build();
    let parsed = parse_rules_toml(toml).expect("parse");
    let rules = compile_rules(&parsed).expect("compile");
    let arity = rules
        .iter()
        .find(|r| r.name == want)
        .map(|r| r.lhs_var_names.len())
        .expect("named rule present");

    let named = Arc::new(Mutex::new(Vec::<u64>::new()));
    let sink = Arc::clone(&named);
    let target = want.to_string();
    struct One {
        out: Arc<Mutex<Vec<u64>>>,
        target: String,
    }
    impl Observer for One {
        fn on_step_end(&mut self, _s: &hg_core::GraphSnapshot<'_>, _st: &StepStats, events: &[RewriteEvent]) {
            let c = events.iter().filter(|e| e.rule_name == self.target).count() as u64;
            self.out.lock().expect("probe poisoned").push(c);
        }
    }
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    engine.add_observer(Box::new(One { out: sink, target }));
    for _ in 0..PROBE_STEPS {
        engine.step(&mut store);
    }
    let c = named.lock().expect("probe poisoned").clone();
    (c.iter().sum(), c.iter().copied().max().unwrap_or(0), arity)
}

#[test]
#[ignore = "long run: D1.1 node-demand table"]
fn d1_1_node_demand_on_the_b1_substrate() {
    eprintln!("\n################################################################################");
    eprintln!("#  D-TRACK / D1.1 — NODE-BUDGET DEMAND ON `CompositeChurnRing` (B1's substrate)");
    eprintln!("#  Each rule ALONE. Fire counts only; no alphabet is read here.");
    eprintln!("################################################################################");

    let sub = btrack::substrate();
    let names = btrack::rule_names(4);
    let mut rows: Vec<Demand> = Vec::new();
    for (k, name) in names.iter().enumerate() {
        rows.push(demand(name, &single(k), &sub));
    }
    // `swap` is not in B1, but C3.4 established its demand is the whole crystal
    // and it is the clearest case; it is also the D1.5 candidate.
    rows.push(demand("swap", btrack::SWAP, &sub));

    let n = rows[0].n;
    eprintln!("\n  substrate n = {n} nodes ({} steps probed, MaxParallel)", PROBE_STEPS);
    eprintln!("\n  rule       | LHS arity | firings/step        | node-demand/step    | demand / n");
    eprintln!("             |           |  early    mean  peak |  early        peak  |  early    peak");
    eprintln!("  -----------+-----------+---------------------+---------------------+----------------");
    for r in &rows {
        eprintln!(
            "  {:<10} | {:>9} | {:>6.2} {:>7.2} {:>5} | {:>7.1} {:>11} | {:>6.1}% {:>6.1}%",
            r.name,
            r.arity,
            r.early_per_step,
            r.mean_per_step,
            r.peak,
            r.early_demand(),
            r.peak_demand(),
            100.0 * r.early_demand() / n as f64,
            100.0 * r.peak_demand() as f64 / n as f64,
        );
    }

    // The k = 4 set only — `swap` is excluded from the sum because it is not in
    // B1 and including it would answer a question B1 never asked.
    let k4: Vec<&Demand> = rows.iter().filter(|r| r.name != "swap").collect();
    let summed_early: f64 = k4.iter().map(|r| r.early_demand()).sum();
    let summed_peak: usize = k4.iter().map(|r| r.peak_demand()).sum();
    eprintln!(
        "\n  SUMMED over the k = 4 set:  early {:.1} nodes/step ({:.1}% of n)   peak {} nodes/step ({:.1}% of n)",
        summed_early,
        100.0 * summed_early / n as f64,
        summed_peak,
        100.0 * summed_peak as f64 / n as f64
    );
    eprintln!(
        "  `swap` alone:               early {:.1} nodes/step ({:.1}% of n)   peak {} nodes/step ({:.1}% of n)",
        rows[4].early_demand(),
        100.0 * rows[4].early_demand() / n as f64,
        rows[4].peak_demand(),
        100.0 * rows[4].peak_demand() as f64 / n as f64
    );

    eprintln!(
        "\n  READING: contention is BINDING if the summed demand approaches or exceeds n, since\n\
         \x20  `MaxParallel` admits only node-DISJOINT bindings — demand above n cannot all be\n\
         \x20  granted and the `(Reverse(priority), rule_id, sorted nodes)` tie-break decides who\n\
         \x20  is cut. It is NOT binding if the sum sits well under n, in which case B1's shares\n\
         \x20  need a different explanation and the contention hypothesis is already in trouble\n\
         \x20  before the large substrate is built.\n"
    );

    // ------------------------------------------------------------------
    // The zeros. C3.4's lesson applies verbatim: a zero does not explain
    // itself, and two of the four rows above are zero. Both are GUARD zeros,
    // not starvation zeros, and the mechanism is checkable without any
    // alphabet: the substrate seeds property 1 (fill) at 5.0, `churn` needs
    // `old` at fill >= 20 and `transfer` needs `src` at fill > 10, and the
    // ONLY rule that raises fill is `influx`. So `churn` and `transfer` have
    // no candidate sites at all when run alone — they are DOWNSTREAM of
    // `influx`, and their demand is not a property of the rule but of the
    // composition it sits in.
    //
    // Their demand is therefore measured in the MINIMAL LIVE CONFIGURATION
    // (`influx` + the rule). That configuration is itself contended, so these
    // rows are a LOWER BOUND on intrinsic demand and are labelled as such.
    // ------------------------------------------------------------------
    let dead: Vec<&Demand> = rows.iter().filter(|r| r.total == 0).collect();
    if !dead.is_empty() {
        eprintln!(
            "  ---- {} rule(s) fired 0 times ALONE; enabled demand (`influx` + rule) ----",
            dead.len()
        );
        eprintln!("  These are GUARD zeros, not starvation zeros: fill is seeded at 5.0 and only");
        eprintln!("  `influx` raises it, so these rules have NO candidate sites without it.");
    }
    let dead_names: Vec<&'static str> = dead.iter().map(|r| r.name).collect();
    let mut enabled: Vec<(&'static str, u64, u64, usize)> = Vec::new();
    for (k, name) in names.iter().enumerate() {
        if !dead_names.contains(name) {
            continue;
        }
        let pair = format!("{}\n{}", single(1), single(k));
        let (fires, peak, arity) = paired_demand(&pair, &sub, name);
        eprintln!(
            "    influx + {:<9} -> {:<9} fired {:>6} in {} steps, peak {:>3}/step, \
             peak demand {:>3} nodes ({:.1}% of n)  [LOWER BOUND — the pair is contended]",
            name,
            name,
            fires,
            PROBE_STEPS,
            peak,
            arity * peak as usize,
            100.0 * (arity * peak as usize) as f64 / n as f64
        );
        enabled.push((name, fires, peak, arity));
    }

    // Claims, not decoration. Every rule is live SOMEWHERE, or the table is a
    // measurement of nothing; and a zero that survives even its enabler would be
    // a different finding entirely.
    for r in &rows {
        if r.total > 0 {
            continue;
        }
        let e = enabled.iter().find(|(n, ..)| *n == r.name).expect("dead rule must be probed enabled");
        assert!(
            e.1 > 0,
            "{} fired 0 times ALONE **and** 0 times with `influx` present — the guard-zero \
             diagnosis is then wrong and the demand table cannot be read",
            r.name
        );
    }
    // C3.4's hand computation for `swap`, now asserted rather than argued.
    let swap = rows.iter().find(|r| r.name == "swap").expect("swap row");
    assert_eq!(swap.arity, 4, "swap binds 4 nodes per firing (C3.4)");
    assert_eq!(swap.peak, 9, "swap fires 9x per step on the 36-node crystal (C3.4)");
    // The headline D1.1 claim, asserted so it cannot silently stop being true.
    assert!(
        summed_peak as f64 / n as f64 >= 1.0,
        "D1.1 published summed peak demand at or above n ({summed_peak} vs n = {n}); if this \
         stops holding, the contention premise of D1 has changed and D1.2's substrate must be \
         re-derived"
    );

    eprintln!("################################################################################\n");
}
