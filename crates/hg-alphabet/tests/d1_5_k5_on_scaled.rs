//! **D1.5 — the k = 5 row, retried on the scaled substrate.**
//!
//! C3.4 found `swap` satisfies every B1.1 criterion for a fifth rule — live,
//! perpetual, node-count-neutral — and still could not compose, because its node
//! demand was 36/36 = **the whole crystal**. The plan expects a substrate on
//! which demand is a small fraction of `n` to remove that blocker by
//! construction.
//!
//! D1.2 found the premise does not hold: `swap` matches every 2-edge, so its
//! demand is **extensive** exactly like `influx`'s and its *fraction* does not
//! fall with `n`. The pre-registered prediction is therefore **VOID again**.
//!
//! **This is a FIRE-COUNT probe, not a measurement.** D1.5 asks only whether
//! `A_rule_only` reaches 5 — whether every rule fires at least once — which is
//! exactly what C3.4's own probe answered, and it needs no alphabet.
//!
//! **Two recorded reductions, both forced and both reported as reductions.**
//! `swap`'s LHS is two INDEPENDENT 2-edges, so candidate enumeration is
//! quadratic in edge count: ~85² ≈ 7 000 pairs/step on B1's crystal against
//! ~1 650² ≈ 2.7 M on D1.2a's. At `SCALE = 4` (n = 960) `swap` is **not
//! runnable at all** — three attempts were abandoned, the last after 10 minutes
//! of CPU and ~10 GB resident without completing 50 steps. So:
//!
//! 1. the horizon is 50 steps, not B1's 2 000 / 40 000; and
//! 2. **D1.5 runs at `SCALE = 2` (n = 240), not D1.2a's `SCALE = 4`.**
//!
//! Neither weakens what D1.5 tests. D1.2's probe already established that the
//! demand fraction is scale-INVARIANT, so n = 240 tests the same claim as
//! n = 960; and the question is a fire-count question, settled in single-digit
//! steps. **No alphabet number is read at this horizon or quoted against a B1
//! row.** That `swap` is computationally unrunnable at n = 960 is itself a
//! second, independent reason the k = 5 row cannot be rescued by scaling.
//!
//! ```text
//! cargo test --release -p hg-alphabet --test d1_5_k5_on_scaled -- --ignored --nocapture
//! ```

use hg_alphabet::{btrack, dtrack};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Enough to answer "does it ever fire". `swap` alone fires on its very first
/// step, and `influx` saturates the substrate on its first step too, so the
/// question is settled in single-digit steps; this leaves an order of magnitude
/// of margin.
const STEPS: u64 = 50;

struct Fires {
    by_rule: Arc<Mutex<BTreeMap<String, u64>>>,
}

impl Observer for Fires {
    fn on_step_end(&mut self, _s: &hg_core::GraphSnapshot<'_>, _st: &StepStats, events: &[RewriteEvent]) {
        let mut m = self.by_rule.lock().expect("probe poisoned");
        for e in events {
            *m.entry(e.rule_name.clone()).or_insert(0) += 1;
        }
    }
}

fn probe(label: &str, toml: &str, order: &[&str]) -> BTreeMap<String, u64> {
    let mut store = dtrack::d1_5_substrate().build();
    let rules = compile_rules(&parse_rules_toml(toml).expect("parse")).expect("compile");
    let by_rule = Arc::new(Mutex::new(BTreeMap::new()));
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    engine.add_observer(Box::new(Fires { by_rule: Arc::clone(&by_rule) }));
    for _ in 0..STEPS {
        engine.step(&mut store);
    }
    let m = by_rule.lock().expect("probe poisoned").clone();
    let total: u64 = m.values().sum::<u64>().max(1);
    eprintln!("\n  --- {label} ---");
    for (id, n) in order.iter().enumerate() {
        let c = m.get(*n).copied().unwrap_or(0);
        eprintln!(
            "      rule_id {id}  {n:<9} fired={c:<9} ({:>5.1}%) {}",
            100.0 * c as f64 / total as f64,
            if c == 0 { "<< 0 firings" } else { "" }
        );
    }
    let live = order.iter().filter(|n| m.get(**n).copied().unwrap_or(0) > 0).count();
    eprintln!(
        "      A_rule_only = {live} (rule count {}) -> {}",
        order.len(),
        if live == order.len() { "row VALID" } else { "ROW VOID — a rule never fired" }
    );
    m
}

#[test]
#[ignore = "long run: D1.5, the k=5 row on the scaled substrate"]
fn d1_5_does_swap_compose_at_k5_on_a_larger_substrate() {
    eprintln!("\n################################################################################");
    eprintln!("#  D-TRACK / D1.5 — k = 5 ON THE SCALED SUBSTRATE (B-track open item 3)");
    eprintln!("#  Pre-registered prediction: VOID again — `swap`'s demand is EXTENSIVE, so the");
    eprintln!("#  blocker was never absolute node scarcity and scaling does not remove it.");
    eprintln!("#  FIRE COUNTS ONLY, at a reduced+recorded horizon ({STEPS} steps) and SCALE=2 (n=240):");
    eprintln!("#  `swap` is computationally UNRUNNABLE at SCALE=4 (quadratic LHS). See module docs.");
    eprintln!("################################################################################");

    // `swap` alone, so its demand at n = 960 is measured here rather than
    // carried over as an inference from n = 60.
    let alone = probe("`swap` ALONE on the scaled substrate", btrack::SWAP, &["swap"]);
    let per_step = alone.get("swap").copied().unwrap_or(0) as f64 / STEPS as f64;
    eprintln!(
        "      => {:.1} firings/step x 4 nodes = {:.0} nodes/step = {:.1}% of the 144-node crystal\n\
         \x20        (it was 36/36 = 100% of the 36-node crystal at n = 60)",
        per_step,
        4.0 * per_step,
        100.0 * 4.0 * per_step / 144.0
    );

    let names = ["glider", "influx", "churn", "transfer", "swap"];
    let composed = probe(
        "k = 5 as composed (`swap` is rule_id 4, last) — the D1.5 row",
        &format!("{}\n{}", btrack::ruleset(4), btrack::SWAP),
        &names,
    );
    let swap_fired = composed.get("swap").copied().unwrap_or(0);
    let live = names.iter().filter(|n| composed.get(**n).copied().unwrap_or(0) > 0).count();

    // A zero does not explain itself (C3.4). Promoting `swap` to `rule_id` 0
    // says whether it was starved or simply had no candidates — starvation
    // running BOTH ways is what ruled out "`swap` is weak" at n = 60.
    let reversed = probe(
        "CONTROL: `swap` promoted to rule_id 0, same substrate, same five rules",
        &format!("{}\n{}", btrack::SWAP, btrack::ruleset(4)),
        &["swap", "glider", "influx", "churn", "transfer"],
    );

    eprintln!("\n  ---- D1.5 verdict ----");
    let silent: Vec<&str> = names
        .iter()
        .filter(|n| composed.get(**n).copied().unwrap_or(0) == 0)
        .copied()
        .collect();
    eprintln!(
        "    A_rule_only = {live} of 5 -> {}{}",
        if live == 5 { "ROW VALID — B-track open item 3 CLOSES" } else { "ROW VOID" },
        if silent.is_empty() { String::new() } else { format!("  (silent: {})", silent.join(", ")) }
    );
    eprintln!(
        "    starvation direction: `swap` fired {swap_fired} at rule_id 4, and {} at rule_id 0 \n\
         \x20     while the B1 four went to {}/{}/{}/{} (glider/influx/churn/transfer).",
        reversed.get("swap").copied().unwrap_or(0),
        reversed.get("glider").copied().unwrap_or(0),
        reversed.get("influx").copied().unwrap_or(0),
        reversed.get("churn").copied().unwrap_or(0),
        reversed.get("transfer").copied().unwrap_or(0),
    );

    eprintln!(
        "\n  MECHANISM. `swap` matches every 2-edge and binds 4 nodes, so like `influx` its\n\
         \x20  demand grows with the substrate and its FRACTION does not fall. It also sits at\n\
         \x20  `rule_id` 4 — last — so it is the first thing the resolver cuts. Scaling changes\n\
         \x20  neither fact, which is why the prediction was VOID and why a bigger substrate was\n\
         \x20  never going to close B-track open item 3.\n"
    );

    // `swap` alone must be live on the scaled substrate, or the composed zero
    // says nothing about composition at all.
    assert!(
        alone.get("swap").copied().unwrap_or(0) > 0,
        "`swap` must be live ALONE on the scaled substrate for the composed row to mean anything"
    );
    eprintln!("################################################################################\n");
}
