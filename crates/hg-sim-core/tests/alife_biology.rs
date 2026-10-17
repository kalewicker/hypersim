//! S3.2/S3.3 — the energy in/out of a carrier, each rule tested in ISOLATION on
//! a fixture with an energy-conservation assertion (scope §10, the
//! conservation_soak discipline) before any composition.
#![cfg(feature = "alife")]

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, StepMode};
use smallvec::SmallVec;

const ENERGY: u32 = 1;
const SATIATION: u32 = 13;

/// The full composed ruleset (all of alife_base.toml).
fn all_rules() -> Vec<CompiledRule> {
    compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap()
}

/// Compile a SINGLE named rule from alife_base.toml, in isolation.
fn only_rule(name: &str) -> Vec<CompiledRule> {
    let raw = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    let one: Vec<_> = raw.into_iter().filter(|r| r.name == name).collect();
    assert_eq!(one.len(), 1, "expected exactly one rule named {name}");
    compile_rules(&one).unwrap()
}

/// Total ALife energy = sum of property key 1 (Float) over all nodes.
fn total_energy(store: &HypergraphStore) -> f64 {
    store
        .nodes()
        .filter_map(|(_, d)| match d.get_property(ENERGY) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        })
        .sum()
}

fn energy(store: &HypergraphStore, id: NodeId) -> f64 {
    match store.get_node(id).unwrap().get_property(ENERGY) {
        Some(PropertyValue::Float(v)) => v,
        _ => f64::NAN,
    }
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// site (substrate) —occupancy— carrier, with the given energies.
fn occupied_site(site_energy: f64, carrier_energy: f64, satiation: f64) -> (HypergraphStore, NodeId, NodeId) {
    let mut s = HypergraphStore::new();
    let site = s.add_node(NodeData::new(Label(0), 0));
    s.get_node_mut(site).unwrap().set_property(ENERGY, PropertyValue::Float(site_energy));
    let carrier = s.add_node(NodeData::new(hg_alife::CARRIER_LABEL, 0));
    let cd = s.get_node_mut(carrier).unwrap();
    cd.set_property(ENERGY, PropertyValue::Float(carrier_energy));
    cd.set_property(SATIATION, PropertyValue::Float(satiation));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, carrier])));
    (s, site, carrier)
}

#[test]
fn harvest_conserves_energy_and_gates_on_satiation() {
    let (mut store, site, carrier) = occupied_site(5.0, 8.0, 12.0);
    let before = total_energy(&store);
    assert!(approx(before, 13.0));

    let mut engine = Engine::new(only_rule("harvest"), StepMode::MaxParallel, 1, 1000);

    // One step: a quantum moves site → carrier, total unchanged.
    engine.step(&mut store);
    assert!(approx(energy(&store, site), 4.0), "site lost one quantum");
    assert!(approx(energy(&store, carrier), 9.0), "carrier gained one quantum");
    assert!(approx(total_energy(&store), 13.0), "harvest conserves total energy");

    // Run to steady state: carrier fills to satiation (12) then stops harvesting.
    for _ in 0..20 {
        engine.step(&mut store);
    }
    assert!(approx(energy(&store, carrier), 12.0), "carrier caps at satiation");
    assert!(approx(energy(&store, site), 1.0), "site keeps the rest");
    assert!(approx(total_energy(&store), 13.0), "still conserved at steady state");
}

#[test]
fn harvest_never_drives_site_negative() {
    // Carrier never satiates (satiation 100); the site guard is what stops it.
    let (mut store, site, carrier) = occupied_site(2.0, 0.0, 100.0);
    let mut engine = Engine::new(only_rule("harvest"), StepMode::MaxParallel, 1, 1000);
    for _ in 0..10 {
        engine.step(&mut store);
    }
    assert!(approx(energy(&store, site), 0.0), "site depletes to exactly 0, never negative");
    assert!(approx(energy(&store, carrier), 2.0), "carrier took all the site had");
    assert!(approx(total_energy(&store), 2.0), "conserved");
}

#[test]
fn metabolise_is_a_declared_sink_and_only_targets_carriers() {
    let mut store = HypergraphStore::new();
    let carrier = store.add_node(NodeData::new(hg_alife::CARRIER_LABEL, 0));
    store.get_node_mut(carrier).unwrap().set_property(ENERGY, PropertyValue::Float(5.0));
    // A substrate node with energy that metabolise must NOT touch.
    let substrate = store.add_node(NodeData::new(Label(0), 0));
    store.get_node_mut(substrate).unwrap().set_property(ENERGY, PropertyValue::Float(3.0));

    let mut engine = Engine::new(only_rule("metabolise"), StepMode::MaxParallel, 1, 1000);

    let mut expected_total = 8.0;
    for _ in 0..3 {
        engine.step(&mut store);
        expected_total -= 0.5; // one carrier drains exactly one maintenance quantum
        assert!(
            approx(total_energy(&store), expected_total),
            "each step is a declared 0.5 sink"
        );
        assert!(approx(energy(&store, substrate), 3.0), "substrate energy untouched");
    }
    assert!(approx(energy(&store, carrier), 3.5), "carrier drained 3 × 0.5");
}

#[test]
fn die_and_recycle_deletes_carrier_cleans_edge_and_recycles() {
    let (mut store, site, _carrier) = occupied_site(5.0, 0.0, 12.0); // carrier at 0 ⟹ dies
    assert_eq!(store.edge_count(), 1);

    let mut engine = Engine::new(only_rule("die_and_recycle"), StepMode::MaxParallel, 1, 1000);
    engine.step(&mut store);

    let carriers = store
        .nodes()
        .filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL)
        .count();
    assert_eq!(carriers, 0, "carrier deleted (absent from RHS)");
    assert_eq!(store.edge_count(), 0, "occupancy edge cleaned up (Contract Rule 5)");
    assert!(
        approx(energy(&store, site), 13.0),
        "site += 8.0 recycle (0.8 × birth energy 10.0)"
    );
}

#[test]
fn die_and_recycle_spares_a_living_carrier() {
    let (mut store, site, carrier) = occupied_site(5.0, 0.5, 12.0); // energy 0.5 > 0
    let mut engine = Engine::new(only_rule("die_and_recycle"), StepMode::MaxParallel, 1, 1000);
    engine.step(&mut store);
    assert_eq!(
        store.nodes().filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL).count(),
        1,
        "a carrier above 0 energy does not die"
    );
    assert!(approx(energy(&store, site), 5.0), "site unchanged");
    assert!(approx(energy(&store, carrier), 0.5));
}

#[test]
fn influx_refills_depleted_substrate_only_and_is_a_source() {
    let mut store = HypergraphStore::new();
    let depleted = store.add_node(NodeData::new(Label(0), 0));
    store.get_node_mut(depleted).unwrap().set_property(ENERGY, PropertyValue::Float(5.0));
    let capped = store.add_node(NodeData::new(Label(0), 0));
    store.get_node_mut(capped).unwrap().set_property(ENERGY, PropertyValue::Float(25.0));
    let carrier = store.add_node(NodeData::new(hg_alife::CARRIER_LABEL, 0));
    store.get_node_mut(carrier).unwrap().set_property(ENERGY, PropertyValue::Float(5.0));

    let before = total_energy(&store); // 35.0
    let mut engine = Engine::new(only_rule("influx"), StepMode::MaxParallel, 1, 1000);
    engine.step(&mut store);

    assert!(approx(energy(&store, depleted), 7.0), "depleted substrate +influx_rate");
    assert!(approx(energy(&store, capped), 25.0), "capped site (>= cap) does not match");
    assert!(approx(energy(&store, carrier), 5.0), "a carrier never receives influx (label-gated)");
    assert!(
        approx(total_energy(&store), before + 2.0),
        "influx is the sole source: total rises by exactly the rate per firing"
    );
}

#[test]
fn influx_stops_at_cap() {
    let mut store = HypergraphStore::new();
    let site = store.add_node(NodeData::new(Label(0), 0));
    store.get_node_mut(site).unwrap().set_property(ENERGY, PropertyValue::Float(5.0));
    let mut engine = Engine::new(only_rule("influx"), StepMode::MaxParallel, 1, 1000);
    for _ in 0..30 {
        engine.step(&mut store);
    }
    // 5 + 2k: last fire at 19 (< 20) → 21, then 21 ≥ 20 stops.
    assert!(approx(energy(&store, site), 21.0), "site settles just above the cap and stops");
}

/// S3.6 + S4 hand-trace: node-funded BIRTH-DISPERSAL. Parent above threshold on
/// site A with an adjacent unoccupied target B ⟹ a child appears on B with the
/// parent's genome ± jitter, the TARGET B pays the child cost, and the parent's
/// energy (and its own site A) are UNCHANGED — reproduction is node-funded and
/// the child colonises the neighbour, not the parent's site (docs §6).
#[test]
fn replicate_is_node_funded_and_inherits_genome() {
    let mut store = HypergraphStore::new();
    let site = store.add_node(NodeData::new(Label(0), 0)); // parent's site A
    let target = store.add_node(NodeData::new(Label(0), 0)); // adjacent target B
    for s in [site, target] {
        let d = store.get_node_mut(s).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(20.0));
        d.set_property(22, PropertyValue::Bool(false)); // occupied
    }
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, target]))); // adjacency
    let parent = store.add_node(NodeData::new(hg_alife::CARRIER_LABEL, 0));
    {
        let p = store.get_node_mut(parent).unwrap();
        p.set_property(ENERGY, PropertyValue::Float(20.0)); // >= repl_threshold 15
        p.set_property(10, PropertyValue::Float(1.0)); // harvest_rate
        p.set_property(11, PropertyValue::Float(15.0)); // replication_threshold
        p.set_property(12, PropertyValue::Float(1.0)); // maintenance_cost
        p.set_property(13, PropertyValue::Float(12.0)); // satiation
    }
    store.get_node_mut(site).unwrap().set_property(22, PropertyValue::Bool(true)); // A occupied
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, parent])));

    let mut engine = Engine::new(only_rule("replicate"), StepMode::MaxParallel, 42, 1000);
    engine.step(&mut store);

    let carriers: Vec<NodeId> = store
        .nodes()
        .filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(carriers.len(), 2, "parent + exactly one child");
    let child = *carriers.iter().find(|&&c| c != parent).unwrap();

    assert!(approx(energy(&store, target), 10.0), "the TARGET B paid child_cost 10");
    assert!(approx(energy(&store, site), 20.0), "the parent's own site A is UNTOUCHED");
    assert!(
        approx(energy(&store, parent), 20.0),
        "parent energy UNTOUCHED — replication is node-funded, not parent-funded"
    );
    assert!(approx(energy(&store, child), 8.0), "child born with birth energy 8");

    let get = |id: NodeId, k: u32| match store.get_node(id).unwrap().get_property(k) {
        Some(PropertyValue::Float(v)) => v,
        _ => f64::NAN,
    };
    for (k, parent_val) in [(10u32, 1.0), (11, 15.0), (12, 1.0), (13, 12.0)] {
        assert!(approx(get(parent, k), parent_val), "parent trait {k} unchanged");
        let cv = get(child, k);
        assert!(
            (cv - parent_val).abs() <= 0.5 + 1e-9,
            "child trait {k} = {cv} is parent {parent_val} ± jitter (copy_from + jitter)"
        );
    }

    let attached_to = |s: NodeId, c: NodeId| {
        store.edges().any(|(_, e)| e.nodes.contains(&s) && e.nodes.contains(&c))
    };
    assert!(attached_to(site, parent), "parent still occupies its site A");
    assert!(attached_to(target, child), "child occupies the neighbour B");
    assert!(!attached_to(site, child), "child is NOT on the parent's site A");
}

/// Two carriers occupying one site (for the crowding mechanic).
fn two_carriers_on_site(site_e: f64, a_sat: f64, b_sat: f64) -> (HypergraphStore, NodeId, NodeId) {
    let mut s = HypergraphStore::new();
    let site = s.add_node(NodeData::new(Label(0), 0));
    s.get_node_mut(site).unwrap().set_property(ENERGY, PropertyValue::Float(site_e));
    let a = s.add_node(NodeData::new(hg_alife::CARRIER_LABEL, 0));
    {
        let d = s.get_node_mut(a).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(8.0));
        d.set_property(SATIATION, PropertyValue::Float(a_sat));
    }
    let b = s.add_node(NodeData::new(hg_alife::CARRIER_LABEL, 0));
    {
        let d = s.get_node_mut(b).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(8.0));
        d.set_property(SATIATION, PropertyValue::Float(b_sat));
    }
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, a])));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, b])));
    (s, a, b)
}

/// Crowding (scope §6.2): two carriers on one site conflict on the site each
/// step (only one harvests), but satiation gating ROTATES access — the first
/// fills, yields, and the second fills too. No permanent monopoly.
#[test]
fn crowding_satiation_rotates_access() {
    let (mut store, a, b) = two_carriers_on_site(200.0, 20.0, 20.0);
    let mut engine = Engine::new(only_rule("harvest"), StepMode::MaxParallel, 1, 1000);
    for _ in 0..40 {
        engine.step(&mut store);
    }
    assert!(approx(energy(&store, a), 20.0), "carrier A reached satiation");
    assert!(approx(energy(&store, b), 20.0), "carrier B also fed — access rotated, not starved");
}

/// Positive control that the gate is REAL: give one carrier an absurd satiation
/// so it never yields. It wins the deterministic conflict every step and
/// monopolises; the cohabitant starves. Removing satiation's effect ⟹ monopoly.
#[test]
fn crowding_without_satiation_gate_is_a_monopoly() {
    let (mut store, a, b) = two_carriers_on_site(200.0, 1000.0, 20.0);
    let mut engine = Engine::new(only_rule("harvest"), StepMode::MaxParallel, 1, 1000);
    for _ in 0..30 {
        engine.step(&mut store);
    }
    assert!(energy(&store, a) > 30.0, "the never-satiated carrier monopolised the site");
    assert!(approx(energy(&store, b), 8.0), "the cohabitant starved (never got access)");
}

// --- S3.7: composed-ruleset conflict ordering (no starvation pathologies) ---

/// A carrier at energy 0 also matches metabolise and harvest, but die_and_recycle
/// has the highest priority ⟹ it dies rather than being drained/fed forever.
#[test]
fn composed_death_is_not_starved() {
    let (mut store, site, _carrier) = occupied_site(20.0, 0.0, 12.0); // energy 0 ⟹ eligible to die
    let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, 1, 1000);
    engine.step(&mut store);
    let carriers = store
        .nodes()
        .filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL)
        .count();
    assert_eq!(carriers, 0, "die_and_recycle (priority 3) wins; the carrier dies");
    assert!(approx(energy(&store, site), 28.0), "site received the 8.0 recycle (20 + 8)");
}

/// A satiated, replication-eligible carrier also matches metabolise. replicate
/// has higher priority ⟹ it reproduces instead of draining the opportunity away.
/// Proven by the parent's energy staying at 16 (metabolise would give 15.5).
#[test]
fn composed_replication_is_not_starved_by_metabolise() {
    let mut store = HypergraphStore::new();
    let site = store.add_node(NodeData::new(Label(0), 0)); // parent's site A
    let target = store.add_node(NodeData::new(Label(0), 0)); // adjacent target B
    for s in [site, target] {
        let d = store.get_node_mut(s).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(20.0));
        d.set_property(22, PropertyValue::Bool(false));
    }
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, target]))); // adjacency
    let parent = store.add_node(NodeData::new(hg_alife::CARRIER_LABEL, 0));
    {
        let p = store.get_node_mut(parent).unwrap();
        p.set_property(ENERGY, PropertyValue::Float(16.0)); // ≥ repl_thr 15 and ≥ satiation 12 (no harvest)
        p.set_property(10, PropertyValue::Float(1.0));
        p.set_property(11, PropertyValue::Float(15.0));
        p.set_property(12, PropertyValue::Float(1.0));
        p.set_property(13, PropertyValue::Float(12.0));
    }
    store.get_node_mut(site).unwrap().set_property(22, PropertyValue::Bool(true)); // A occupied
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, parent])));

    let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, 42, 1000);
    engine.step(&mut store);

    let carriers = store
        .nodes()
        .filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL)
        .count();
    assert_eq!(carriers, 2, "replicate fired: parent + child");
    assert!(
        approx(energy(&store, parent), 16.0),
        "parent energy still 16 — replicate (priority 2) beat metabolise (which would leave 15.5)"
    );
    assert!(approx(energy(&store, target), 10.0), "the TARGET paid the child cost (replicate beat influx too)");
}
