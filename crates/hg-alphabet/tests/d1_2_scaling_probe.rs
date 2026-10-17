//! **D1.2 derivation probe — can the node budget be relieved by scaling at all?**
//!
//! D1.2 requires a substrate on which summed node-demand at k = 4 is ≤ 10% of
//! `n`, "computed from D1.1's numbers, not guessed". D1.1's numbers say that
//! target may be **unreachable by scaling**, and the reason is structural rather
//! than a matter of picking a bigger number:
//!
//! `influx` has **LHS arity 1** and a guard (`fill < 20`) that every node on a
//! freshly built substrate satisfies. So it proposes one binding per node —
//! `n` bindings, all mutually node-disjoint, all accepted. Its demand is
//! **extensive**: it grows exactly as fast as the budget it is measured against,
//! so `demand / n` is invariant under scaling.
//!
//! If that holds, the D1.2 target cannot be met by any substrate in the family,
//! and the contention D1 is testing for is not node scarcity — it is
//! **universal coverage plus `rule_id` order**, which no amount of substrate is
//! going to relieve. That distinction decides what D1.3 is even able to test, so
//! it is measured before the substrate is committed, not assumed either way.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test d1_2_scaling_probe -- --ignored --nocapture
//! ```

use hg_alphabet::btrack;
use hg_alphabet::run::Init;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// The scaling family: `CompositeChurnRing` with the crystal and the ring scaled
/// together, so the two components keep their 36 : 24 = 3 : 2 node ratio and the
/// substrate stays a scaled version of B1's rather than a differently-shaped one.
fn scaled(mult: usize) -> Init {
    Init::CompositeChurnRing {
        w: 6 * mult,
        h: 6 * mult,
        arity: 3,
        fill: 5.0,
        // 24 · mult² keeps the ring in proportion to the crystal's w·h growth.
        ring: 24 * mult * mult,
    }
}

struct ByRule {
    peak: Arc<Mutex<BTreeMap<String, u64>>>,
    total: Arc<Mutex<BTreeMap<String, u64>>>,
}

impl Observer for ByRule {
    fn on_step_end(&mut self, _s: &hg_core::GraphSnapshot<'_>, _st: &StepStats, events: &[RewriteEvent]) {
        let mut this_step: BTreeMap<String, u64> = BTreeMap::new();
        for e in events {
            *this_step.entry(e.rule_name.clone()).or_insert(0) += 1;
        }
        let mut peak = self.peak.lock().expect("probe poisoned");
        let mut total = self.total.lock().expect("probe poisoned");
        for (k, v) in this_step {
            let p = peak.entry(k.clone()).or_insert(0);
            *p = (*p).max(v);
            *total.entry(k).or_insert(0) += v;
        }
    }
}

/// `(n, peak firings per rule, total firings per rule)` for one ruleset.
fn probe(toml: &str, init: &Init, steps: u64) -> (usize, BTreeMap<String, u64>, BTreeMap<String, u64>) {
    let mut store = init.build();
    let n = store.node_count();
    let rules = compile_rules(&parse_rules_toml(toml).expect("parse")).expect("compile");
    let peak = Arc::new(Mutex::new(BTreeMap::new()));
    let total = Arc::new(Mutex::new(BTreeMap::new()));
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    engine.add_observer(Box::new(ByRule {
        peak: Arc::clone(&peak),
        total: Arc::clone(&total),
    }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    let p = peak.lock().expect("probe poisoned").clone();
    let t = total.lock().expect("probe poisoned").clone();
    (n, p, t)
}

fn influx_only() -> String {
    let all = btrack::ruleset(4);
    let parts: Vec<&str> = all.split("[[rules]]").collect();
    format!("[[rules]]{}", parts[2])
}

#[test]
#[ignore = "long run: D1.2 scaling derivation probe"]
fn d1_2_is_the_node_budget_relievable_by_scaling() {
    eprintln!("\n################################################################################");
    eprintln!("#  D-TRACK / D1.2 — DERIVATION PROBE: does scaling relieve the node budget?");
    eprintln!("#  Fire counts only. No alphabet is read, and no substrate is committed here.");
    eprintln!("################################################################################");

    // ---- Part 1: is `influx`'s demand extensive? ----
    eprintln!("\n  ---- `influx` ALONE across the scaling family (arity 1) ----");
    eprintln!("  mult |    n   | peak firings/step | peak demand/n");
    eprintln!("  -----+--------+-------------------+---------------");
    let mut fractions: Vec<(usize, f64)> = Vec::new();
    for mult in [1usize, 2, 3, 4] {
        let (n, peak, _) = probe(&influx_only(), &scaled(mult), 60);
        let p = peak.get("influx").copied().unwrap_or(0);
        let frac = p as f64 / n as f64;
        eprintln!("  {mult:>4} | {n:>6} | {p:>17} | {:>12.1}%", 100.0 * frac);
        fractions.push((n, frac));
    }

    let lo = fractions.first().expect("probed").1;
    let hi = fractions.last().expect("probed").1;
    eprintln!(
        "\n  `influx` peak demand fraction: {:.1}% at n={} -> {:.1}% at n={}  (span {:.1}x in n)",
        100.0 * lo,
        fractions[0].0,
        100.0 * hi,
        fractions[3].0,
        fractions[3].0 as f64 / fractions[0].0 as f64
    );

    // ---- Part 2: do the k = 4 shares move under scaling? ----
    eprintln!("\n  ---- the k = 4 composition across the same family (500 steps) ----");
    eprintln!("  mult |    n   | glider   influx   churn   transfer   (event share)");
    eprintln!("  -----+--------+----------------------------------------------------");
    for mult in [1usize, 2, 3, 4] {
        let (n, _, total) = probe(&btrack::ruleset(4), &scaled(mult), 500);
        let sum: u64 = total.values().sum::<u64>().max(1);
        let pct = |k: &str| 100.0 * total.get(k).copied().unwrap_or(0) as f64 / sum as f64;
        eprintln!(
            "  {mult:>4} | {n:>6} | {:>6.1}%  {:>6.1}%  {:>6.1}%  {:>7.1}%",
            pct("glider"),
            pct("influx"),
            pct("churn"),
            pct("transfer")
        );
    }

    eprintln!(
        "\n  READING. `influx` binds ONE node and matches universally, so it proposes `n`\n\
         \x20  mutually disjoint bindings and its demand grows exactly as fast as the budget.\n\
         \x20  If the fraction above is flat, D1.2's `demand <= 10% of n` target is UNREACHABLE\n\
         \x20  by scaling — not because the multiplier is too small but because the quantity is\n\
         \x20  scale-invariant — and the contention B1 exhibits is universal coverage plus\n\
         \x20  `rule_id` order, which a larger substrate cannot relieve.\n"
    );

    // The load-bearing claim, asserted so the D1.2 derivation cannot rest on a
    // number that quietly stopped being true.
    assert!(
        (hi - lo).abs() < 0.05,
        "the D1.2 derivation assumes `influx`'s peak demand fraction is scale-INVARIANT; it \
         moved {:.1}% -> {:.1}% across a {:.0}x span in n, which would mean scaling DOES \
         relieve the budget and the committed substrate must be re-derived",
        100.0 * lo,
        100.0 * hi,
        fractions[3].0 as f64 / fractions[0].0 as f64
    );
    eprintln!("################################################################################\n");
}
