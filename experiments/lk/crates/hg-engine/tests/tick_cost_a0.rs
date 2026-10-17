//! A0 question 4 (v6 §2): measure the per-step cost of a single-node
//! LHS decrement rule (`lk_tick`) at 1e4–1e5 nodes. This is the one real
//! performance risk (hazard 1): the isolated-var match path scans ALL
//! node_ids each step. If the tick dominates step time, the fallback is
//! §4.3 (piggyback on transport's edge LHS). Run with --ignored --nocapture.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_rules::{compile_rules, parse_rules_toml};
use smallvec::smallvec;
use std::time::Instant;

const COUNTDOWN: u32 = 7;

/// lk_tick: single-node LHS, guard countdown > 0, RHS decrements it.
fn tick_rule() -> Vec<hg_rules::CompiledRule> {
    let toml = r#"
[[rules]]
name = "lk_tick"
priority = 2
[rules.lhs]
nodes = ["w"]
[[rules.lhs.node_property_guards]]
var = "w"
property = 7
op = "gt"
value = 0
[rules.rhs]
nodes = ["w"]
[[rules.rhs.property_effects]]
var = "w"
key = 7
op = "subtract"
value = 1
"#;
    compile_rules(&parse_rules_toml(toml).unwrap()).unwrap()
}

/// A ring of `n` nodes, a fraction `live_frac` carrying countdown = 50
/// (the rest countdown-free, like gen-0 lattice). Ring edges give a
/// realistic incidence index without exploding memory.
fn ring_with_countdowns(n: usize, live_frac: f64) -> HypergraphStore {
    let mut s = HypergraphStore::new();
    let ids: Vec<_> = (0..n)
        .map(|i| {
            let mut d = NodeData::new(Label(0), 0);
            if (i as f64) < n as f64 * live_frac {
                d.set_property(COUNTDOWN, PropertyValue::Int(50));
            }
            s.add_node(d)
        })
        .collect();
    for i in 0..n {
        s.add_edge(HyperEdge::new(Label(0), smallvec![ids[i], ids[(i + 1) % n]]));
    }
    s
}

#[test]
#[ignore]
fn a0_tick_cost() {
    let rules = tick_rule();
    println!("n | live_frac | bindings | µs/step (find_bindings) | ns/node");
    for &n in &[10_000usize, 50_000, 100_000] {
        for &frac in &[0.1f64, 0.5, 1.0] {
            let store = ring_with_countdowns(n, frac);
            let snap = store.snapshot();
            // Warm + timed passes over find_bindings (the scan-all cost).
            let _ = hg_engine::matcher::find_bindings(&snap, &rules[0], 0);
            let iters = 20;
            let t = Instant::now();
            let mut nb = 0;
            for _ in 0..iters {
                nb = hg_engine::matcher::find_bindings(&snap, &rules[0], 0).len();
            }
            let us = t.elapsed().as_micros() as f64 / iters as f64;
            println!(
                "{n} | {frac} | {nb} | {us:.1} | {:.2}",
                us * 1000.0 / n as f64
            );
        }
    }
    // Reference: a step of the whole LK pump family at ~5000 nodes runs
    // in well under a millisecond (the exp_l14 timing). A tick scan that
    // costs >~500µs/step at 1e5 nodes would dominate; the numbers above
    // decide whether §4.3 fallback is needed.
}

/// A0 question 5: an Int countdown property (key 7) survives the
/// snapshot bincode round-trip (the relaxation-IC path serializes the
/// whole store), and lk_tick decrements it exactly once per apply.
#[test]
fn a0_countdown_roundtrip_and_decrement() {
    let mut store = ring_with_countdowns(4, 1.0);
    // Round-trip through the snapshot codec used by SnapshotWriter.
    let bytes = bincode::serialize(&store).unwrap();
    let restored: HypergraphStore = bincode::deserialize(&bytes).unwrap();
    for (id, d) in restored.nodes() {
        assert_eq!(
            d.get_property(COUNTDOWN),
            Some(PropertyValue::Int(50)),
            "countdown must survive round-trip on {id:?}"
        );
    }
    store = restored;
    // One engine step decrements every live countdown by exactly 1
    // (all single-node bindings node-disjoint ⇒ all fire).
    let rules = tick_rule();
    let mut engine = hg_engine::Engine::new(rules, hg_rules::StepMode::MaxParallel, 42, 100000);
    engine.step(&mut store);
    for (_, d) in store.nodes() {
        assert_eq!(d.get_property(COUNTDOWN), Some(PropertyValue::Int(49)));
    }
}
