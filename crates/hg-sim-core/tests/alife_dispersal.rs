//! S4 birth-dispersal — the isolated conservation + placement + invariant fixture
//! for the rewritten `replicate` (child colonises an adjacent UNOCCUPIED substrate
//! node, target-pays). Written before trusting it in the composed run, per the S3
//! discipline. See docs/experiment5_preregistration.md §6.
//!
//! The load-bearing invariant: `occupied` (key 22) is rule-maintained state that
//! parallels the graph (the matcher has no NAC), so it can desync. Every test
//! that runs the biology asserts `count(occupied sites) == count(carriers)`.
#![cfg(feature = "alife")]

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, StepMode};
use hg_sim_core::SimConfig;
use smallvec::SmallVec;

const ENERGY: u32 = 1;
const REPL_THR: u32 = 11;
const SATIATION: u32 = 13;
const OCCUPIED: u32 = 22;
const CARRIER: Label = Label(100);

fn only_rule(name: &str) -> Vec<CompiledRule> {
    let raw: Vec<_> = parse_rules_toml(hg_alife::alife_base_toml())
        .unwrap()
        .into_iter()
        .filter(|r| r.name == name)
        .collect();
    compile_rules(&raw).unwrap()
}

fn all_rules() -> Vec<CompiledRule> {
    compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap()
}

fn fget(store: &HypergraphStore, id: NodeId, key: u32) -> Option<f64> {
    match store.get_node(id)?.get_property(key) {
        Some(PropertyValue::Float(v)) => Some(v),
        _ => None,
    }
}

fn occupied(store: &HypergraphStore, id: NodeId) -> Option<bool> {
    match store.get_node(id)?.get_property(OCCUPIED) {
        Some(PropertyValue::Bool(b)) => Some(b),
        _ => None,
    }
}

fn carrier_count(store: &HypergraphStore) -> usize {
    store.nodes().filter(|(_, d)| d.label == CARRIER).count()
}

fn occupied_count(store: &HypergraphStore) -> usize {
    store
        .nodes()
        .filter(|(_, d)| matches!(d.get_property(OCCUPIED), Some(PropertyValue::Bool(true))))
        .count()
}

fn total_energy(store: &HypergraphStore) -> f64 {
    store
        .nodes()
        .filter_map(|(_, d)| match d.get_property(ENERGY) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        })
        .sum()
}

/// Make a substrate node with energy + occupied=false.
fn site(store: &mut HypergraphStore, energy: f64) -> NodeId {
    let id = store.add_node(NodeData::new(Label(0), 0));
    let d = store.get_node_mut(id).unwrap();
    d.set_property(ENERGY, PropertyValue::Float(energy));
    d.set_property(OCCUPIED, PropertyValue::Bool(false));
    id
}

/// Put a fertile carrier on `s`, marking `s` occupied and wiring the edge.
fn carrier_on(store: &mut HypergraphStore, s: NodeId, energy: f64) -> NodeId {
    let c = store.add_node(NodeData::new(CARRIER, 0));
    {
        let d = store.get_node_mut(c).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(energy));
        d.set_property(10, PropertyValue::Float(1.0)); // harvest_rate
        d.set_property(REPL_THR, PropertyValue::Float(15.0));
        d.set_property(12, PropertyValue::Float(1.0)); // maintenance_cost
        d.set_property(SATIATION, PropertyValue::Float(20.0));
    }
    store.get_node_mut(s).unwrap().set_property(OCCUPIED, PropertyValue::Bool(true));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[s, c])));
    c
}

fn lattice_edge(store: &mut HypergraphStore, a: NodeId, b: NodeId) {
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
}

/// A fertile parent on A dispersing onto adjacent unoccupied B: the child lands
/// on B (not A), B pays the cost, B becomes occupied, the parent is untouched,
/// energy is conserved to the 2.0 birth entropy, and the invariant holds.
#[test]
fn birth_dispersal_places_child_on_adjacent_unoccupied_node() {
    let mut store = HypergraphStore::new();
    let a = site(&mut store, 20.0);
    let b = site(&mut store, 20.0);
    lattice_edge(&mut store, a, b);
    let parent = carrier_on(&mut store, a, 20.0); // energy 20 >= threshold 15

    assert_eq!(occupied_count(&store), carrier_count(&store), "invariant at t=0");
    let before = total_energy(&store);

    let mut engine = Engine::new(only_rule("replicate"), StepMode::MaxParallel, 7, 1000);
    engine.step(&mut store);

    // Exactly one child, and it is on B, not A.
    assert_eq!(carrier_count(&store), 2, "one child born");
    let children: Vec<NodeId> = store
        .nodes()
        .filter(|(id, d)| d.label == CARRIER && *id != parent)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(children.len(), 1);
    let child = children[0];
    // The child shares an edge with B (its new site), not A.
    let child_on = |s: NodeId| {
        store
            .edges()
            .any(|(_, e)| e.nodes.contains(&child) && e.nodes.contains(&s))
    };
    assert!(child_on(b), "child must be placed on the adjacent target B");
    assert!(!child_on(a), "child must NOT be on the parent's site A");

    // Target-pays: B funded the child (20 → 10), parent untouched (still 20).
    assert_eq!(fget(&store, b, ENERGY), Some(10.0), "target B paid child cost");
    assert_eq!(fget(&store, parent, ENERGY), Some(20.0), "parent energy untouched (node-funded)");
    assert_eq!(fget(&store, child, ENERGY), Some(8.0), "child birth energy");

    // Occupancy flags updated: A still occupied (parent), B now occupied (child).
    assert_eq!(occupied(&store, a), Some(true), "A stays occupied");
    assert_eq!(occupied(&store, b), Some(true), "B now occupied");

    // Energy conserved to the 2.0 birth entropy; invariant preserved.
    assert!((before - total_energy(&store) - 2.0).abs() < 1e-9, "birth entropy exactly 2.0");
    assert_eq!(occupied_count(&store), carrier_count(&store), "invariant after birth");
}

/// The `occupied == false` guard is a real NAC surrogate: a fertile parent will
/// NOT place a child on an already-occupied neighbour, but WILL colonise a free
/// one. Fixture: parent on A, neighbours B (occupied) and C (free).
#[test]
fn occupied_guard_blocks_taken_neighbour_but_allows_free_one() {
    let mut store = HypergraphStore::new();
    let a = site(&mut store, 20.0);
    let b = site(&mut store, 20.0);
    let c = site(&mut store, 20.0);
    lattice_edge(&mut store, a, b);
    lattice_edge(&mut store, a, c);
    let _parent = carrier_on(&mut store, a, 20.0);
    let _other = carrier_on(&mut store, b, 5.0); // B occupied, but infertile (energy < threshold)

    assert_eq!(occupied_count(&store), carrier_count(&store), "invariant at t=0 (2==2)");

    let mut engine = Engine::new(only_rule("replicate"), StepMode::MaxParallel, 7, 1000);
    engine.step(&mut store);

    // The only fertile parent (on A) must colonise C, never B.
    assert_eq!(occupied(&store, c), Some(true), "free neighbour C colonised");
    assert_eq!(fget(&store, c, ENERGY), Some(10.0), "C paid the child cost");
    assert_eq!(fget(&store, b, ENERGY), Some(20.0), "occupied B untouched — not re-colonised");
    assert_eq!(occupied_count(&store), carrier_count(&store), "invariant holds after birth");
}

/// THE desync guard: over a full composed soak, the rule-maintained `occupied`
/// flag must never drift from the graph. Asserted EVERY step.
#[test]
fn occupied_invariant_holds_every_step_in_composed_soak() {
    let toml = r#"
experiment_name = "disp"
run_id = "disp"
seed = 4242
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 6
height = 6
arity = 3
[engine]
step_mode = "MaxParallel"
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
"#;
    let config = SimConfig::from_toml(toml).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, config.seed, 100000);

    for step in 1..=5_000u64 {
        engine.step(&mut store);
        let carriers = carrier_count(&store);
        let occ = occupied_count(&store);
        assert_eq!(
            occ, carriers,
            "occupied/carrier desync at step {step}: {occ} occupied vs {carriers} carriers"
        );
    }
}
