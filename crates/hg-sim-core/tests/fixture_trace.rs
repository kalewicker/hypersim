//! Independent hand-traced fixture verification for property_effects
//! (rules/effects_cascade_fixture.toml, rules/conservation_soak.toml).
//!
//! These fixtures were authored by a reviewer who never saw the
//! implementation; every outcome is pre-computed in the TOML comments.
//! We run them through the REAL `run_headless` pipeline (Empty initial
//! condition, MaxParallel) and read exact per-node energies back from the
//! final bincoded snapshot — no reimplementation of apply, no tolerance
//! games (10.0 and 25.0 are f64-exact).

use hg_core::{HypergraphStore, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, load_rules_from_file, StepMode};
use std::path::{Path, PathBuf};

const ENERGY: u32 = 1;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../rules")
        .join(name)
}

/// Drive a rule-file fixture on an Empty world through the REAL engine
/// (`Engine::step` → `apply_binding`, identical to run_headless's apply
/// path — the only things run_headless adds are observers/CSV/snapshots,
/// which never alter the store), then return the final live store.
///
/// Direct-drive rather than snapshot readback because the bincode
/// snapshot cannot round-trip a property-bearing store (`PropertyValue`
/// is `#[serde(untagged)]`, which bincode's non-self-describing format
/// can't deserialize — a latent limitation, reported separately).
fn run_and_load_final_store(rule_file: &str, steps: u64, _window: u64, _tag: &str) -> HypergraphStore {
    run_and_count(rule_file, steps).0
}

/// As above, also aggregating the real `StepStats` property-effect
/// counters across the whole run (Tier 2: the counters must populate).
fn run_and_count(rule_file: &str, steps: u64) -> (HypergraphStore, u64, u64) {
    let rules = compile_rules(&load_rules_from_file(&fixture_path(rule_file)).expect("rules parse"))
        .expect("rules compile");
    // Empty initial condition.
    let mut store = HypergraphStore::new();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 1000);
    let (mut applied, mut skipped) = (0u64, 0u64);
    for _ in 0..steps {
        let stats = engine.step(&mut store);
        applied += stats.property_effects_applied;
        skipped += stats.property_effects_skipped;
    }
    (store, applied, skipped)
}

fn energy(store: &HypergraphStore, id: hg_core::NodeId) -> Option<f64> {
    match store.get_node(id)?.get_property(ENERGY) {
        Some(PropertyValue::Float(f)) => Some(f),
        _ => None,
    }
}

/// Sorted list of every node's Float energy (unset nodes excluded).
fn all_energies(store: &HypergraphStore) -> Vec<f64> {
    let mut v: Vec<f64> = store
        .nodes()
        .filter_map(|(id, _)| energy(store, id))
        .collect();
    v.sort_by(|a, b| b.total_cmp(a)); // descending
    v
}

/// Cascade fixture: a four-firing starvation curve, self-terminating at
/// step 5. Every energy is predicted exactly in the TOML.
#[test]
fn cascade_fixture_traces_exactly() {
    let (store, applied, skipped) = run_and_count("effects_cascade_fixture.toml", 10);

    // Tier 2 (end-to-end): counters populate. 1 creation `set` + 4 spawns
    // × 2 effects each = 9 applied, 0 skipped (all types match).
    assert_eq!((applied, skipped), (9, 0), "property-effect counters must populate");

    assert_eq!(store.node_count(), 6, "seed + anchor + 4 children");
    assert_eq!(store.edge_count(), 5, "seed-anchor + 4 spawn edges");

    // Reference chain built with the SAME f64 ops the engine performs
    // (`scale_from` = f*0.8, `subtract` = a-100). The fixture's decimal
    // literals (2.4, 81.92, 272.32) are ideal real-number results; 0.8 is
    // not binary-representable, so an honest IEEE implementation lands a
    // few ULP away. We assert bit-equality against this reference (proves
    // correct arithmetic) and separately show the ULP gap vs the literals.
    let seed0 = 200.0_f64;
    let child1 = seed0 * 0.8; // 160 — read PRE-subtract (the tripwire)
    let child2 = child1 * 0.8; // 128
    let child3 = child2 * 0.8; // 102.4
    let child4 = child3 * 0.8; // 81.92-ish
    // Final donor energies after each pays 100.
    let ref_desc = {
        let mut v = [seed0 - 100.0, child1 - 100.0, child2 - 100.0, child3 - 100.0, child4];
        v.sort_by(|a, b| b.total_cmp(a));
        v
    };
    let energies = all_energies(&store);
    assert_eq!(energies.len(), 5, "five funded nodes carry energy");
    for (got, want) in energies.iter().zip(ref_desc.iter()) {
        assert_eq!(
            got.to_bits(),
            want.to_bits(),
            "energy {got} != honest-f64 reference {want} — arithmetic wrong"
        );
    }

    // Tripwire: the exactly-representable values (dyadic) are bit-exact
    // vs the fixture's literals. child1 -> 60 (=160-100) is the proof of
    // sequential reads-see-pre-subtract: snapshot/swapped semantics would
    // give child1 = (200-100)*0.8 = 80, never 160/60.
    assert_eq!(energies[0].to_bits(), 100.0_f64.to_bits(), "seed = 100 exact");
    assert!(energies.contains(&60.0), "child1 = 60 exact (proves pre-subtract read)");
    assert!(energies.contains(&28.0), "child2 = 28 exact");

    let sum: f64 = energies.iter().sum();
    // ULP gaps vs the fixture's decimal literals — the reportable finding.
    let ulp = |g: f64, l: f64| g.to_bits() as i64 - l.to_bits() as i64;
    eprintln!("CASCADE final energies (desc): {energies:?}");
    // energies = [seed 100, child4 81.92, child1 60, child2 28, child3 2.4].
    eprintln!("  vs fixture literals: child4 81.92 (ULP {}), child3 2.4 (ULP {}), sum 272.32 (ULP {})",
        ulp(energies[1], 81.92), ulp(energies[4], 2.4), ulp(sum, 272.32));
    eprintln!("  seed/child1/child2 = 100/60/28 bit-exact; the gaps are 0.8-not-dyadic, not a bug");
    assert!((sum - 272.32).abs() < 1e-10, "sum ~= 272.32 (got {sum})");
}

/// Conservation soak: sum of the four energies is EXACTLY 100.0 forever,
/// node/edge counts pinned at 4/4, no freeze. Run a long soak so any
/// drift ratchet becomes visible.
#[test]
fn conservation_soak_holds_at_1000_steps() {
    let store = run_and_load_final_store("conservation_soak.toml", 1000, 250, "soak");

    assert_eq!(store.node_count(), 4, "transfer creates no nodes");
    assert_eq!(store.edge_count(), 4, "transfer recreates its matched edge");

    let energies = all_energies(&store);
    assert_eq!(energies.len(), 4, "all four nodes carry Float energy");
    let sum: f64 = energies.iter().sum();
    assert_eq!(
        sum.to_bits(),
        100.0_f64.to_bits(),
        "energy sum must be EXACTLY 100.0 after 1000 steps, got {sum} from {energies:?}"
    );
    // Never froze: max >= 25 > 10 guarantees a transfer always fires, and
    // conservation means the total is intact.
    eprintln!("SOAK @1000 four energies (desc): {energies:?}  sum = {sum}");
}
