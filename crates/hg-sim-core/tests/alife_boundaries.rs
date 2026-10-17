//! Pre-S4 test areas 2 (guard boundaries), 5 (null-biology controls), and the
//! pre-registered replication_threshold degeneracy (tests a & b).
#![cfg(feature = "alife")]

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, StepMode};
use hg_sim_core::SimConfig;
use smallvec::SmallVec;

const ENERGY: u32 = 1;
const HARVEST: u32 = 10;
const REPL_THR: u32 = 11;
const MAINT: u32 = 12;
const SATIATION: u32 = 13;
const CARRIER: Label = Label(100);

fn only_rule(name: &str) -> Vec<CompiledRule> {
    let raw: Vec<_> = parse_rules_toml(hg_alife::alife_base_toml())
        .unwrap()
        .into_iter()
        .filter(|r| r.name == name)
        .collect();
    compile_rules(&raw).unwrap()
}

fn rules_without(name: &str) -> Vec<CompiledRule> {
    let raw: Vec<_> = parse_rules_toml(hg_alife::alife_base_toml())
        .unwrap()
        .into_iter()
        .filter(|r| r.name != name)
        .collect();
    compile_rules(&raw).unwrap()
}

fn energy(store: &HypergraphStore, id: NodeId) -> f64 {
    match store.get_node(id).unwrap().get_property(ENERGY) {
        Some(PropertyValue::Float(v)) => v,
        _ => f64::NAN,
    }
}

fn carrier_count(store: &HypergraphStore) -> usize {
    store.nodes().filter(|(_, d)| d.label == CARRIER).count()
}

/// site —occupancy— carrier with a full genome; returns (store, site, carrier).
fn genome_carrier(
    site_e: f64,
    carrier_e: f64,
    repl_thr: f64,
    satiation: f64,
) -> (HypergraphStore, NodeId, NodeId) {
    let mut s = HypergraphStore::new();
    let site = s.add_node(NodeData::new(Label(0), 0));
    s.get_node_mut(site).unwrap().set_property(ENERGY, PropertyValue::Float(site_e));
    let c = s.add_node(NodeData::new(CARRIER, 0));
    {
        let d = s.get_node_mut(c).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(carrier_e));
        d.set_property(HARVEST, PropertyValue::Float(1.0));
        d.set_property(REPL_THR, PropertyValue::Float(repl_thr));
        d.set_property(MAINT, PropertyValue::Float(1.0));
        d.set_property(SATIATION, PropertyValue::Float(satiation));
    }
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, c])));
    (s, site, c)
}

// ---------------------------------------------------------------------------
// Area 2: guard boundaries
// ---------------------------------------------------------------------------

/// Harvest has a site-floor guard (site.energy >= quantum). A carrier on a site
/// with energy below the quantum must NOT harvest, and the site must not go
/// negative. (Regression against a missing site-floor guard = a hidden source.)
#[test]
fn harvest_has_a_site_floor_guard() {
    let (mut store, site, carrier) = genome_carrier(0.5, 5.0, 15.0, 20.0); // site 0.5 < quantum 1.0
    let mut engine = Engine::new(only_rule("harvest"), StepMode::MaxParallel, 1, 1000);
    engine.step(&mut store);
    assert_eq!(energy(&store, site), 0.5, "harvest must not fire; site energy unchanged");
    assert_eq!(energy(&store, carrier), 5.0, "carrier gained nothing");
    assert!(energy(&store, site) >= 0.0, "site energy never negative");
}

/// PropertyCompare exact-equality boundaries for every op used in the biology.
#[test]
fn propertycompare_boundaries_are_exact() {
    // harvest uses `lt` (energy < satiation): energy == satiation ⟹ NO harvest.
    let (mut s, _site, c) = genome_carrier(50.0, 12.0, 15.0, 12.0); // energy == satiation
    let mut e = Engine::new(only_rule("harvest"), StepMode::MaxParallel, 1, 1000);
    e.step(&mut s);
    assert_eq!(energy(&s, c), 12.0, "energy == satiation must NOT harvest (lt is strict)");

    // replicate uses `gte` (energy >= repl_threshold): energy == threshold ⟹ replicate.
    // Birth-dispersal needs an adjacent unoccupied target for the child.
    let (mut s, site, _c) = genome_carrier(50.0, 15.0, 15.0, 20.0); // parent energy == repl_threshold
    let target = s.add_node(NodeData::new(Label(0), 0));
    {
        let d = s.get_node_mut(target).unwrap();
        d.set_property(ENERGY, PropertyValue::Float(50.0)); // fundable
        d.set_property(22, PropertyValue::Bool(false)); // unoccupied
    }
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, target])));
    s.get_node_mut(site).unwrap().set_property(22, PropertyValue::Bool(true)); // parent's site occupied
    let mut e = Engine::new(only_rule("replicate"), StepMode::MaxParallel, 7, 1000);
    e.step(&mut s);
    assert_eq!(carrier_count(&s), 2, "energy == replication_threshold MUST replicate (gte)");

    // die uses `lte` (energy <= 0): energy == 0 ⟹ die.
    let (mut s, _site, _c) = genome_carrier(50.0, 0.0, 15.0, 20.0); // energy == 0
    let mut e = Engine::new(only_rule("die_and_recycle"), StepMode::MaxParallel, 1, 1000);
    e.step(&mut s);
    assert_eq!(carrier_count(&s), 0, "energy == 0 MUST die (lte)");
}

// ---------------------------------------------------------------------------
// Area 5: null-biology controls (falsify "the economy is driving this")
// ---------------------------------------------------------------------------

fn run_from_config(rules: Vec<CompiledRule>, toml_extra: &str, steps: u64) -> HypergraphStore {
    let toml = format!(
        r#"
experiment_name = "null"
run_id = "null"
seed = 99
[rules]
source = "Inline"
inline_toml = "rules = []"
[initial]
condition_type = "PureCrystal"
width = 5
height = 5
arity = 3
[engine]
step_mode = "MaxParallel"
max_pairings_per_step = 100000
[causal]
enabled = false
[evaluator]
enabled = false
[alife]
enabled = true
{toml_extra}
"#
    );
    let config = SimConfig::from_toml(&toml).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, config.seed, 100000);
    for _ in 0..steps {
        engine.step(&mut store);
    }
    store
}

#[test]
fn no_influx_leads_to_extinction() {
    // Finite starting energy, no source ⟹ the population must die out.
    let store = run_from_config(
        rules_without("influx"),
        "seed_carrier_marker = true\ninitial_substrate_energy = 5.0",
        400,
    );
    assert_eq!(carrier_count(&store), 0, "with no influx the carrier population must go extinct");
}

#[test]
fn no_harvest_leads_to_extinction() {
    // Carriers cannot gain energy ⟹ they metabolise from birth energy and die.
    let store = run_from_config(
        rules_without("harvest"),
        "seed_carrier_marker = true\ninitial_substrate_energy = 5.0",
        400,
    );
    assert_eq!(carrier_count(&store), 0, "with no harvest the carrier population must go extinct");
}

#[test]
fn marker_unset_yields_zero_carriers() {
    // No seed marker ⟹ bootstrap never fires ⟹ the alife rules load but the run
    // stays a pure substrate run (0 carriers).
    let all = compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap();
    let store = run_from_config(all, "seed_carrier_marker = false\ninitial_substrate_energy = 0.0", 200);
    assert_eq!(carrier_count(&store), 0, "no marker ⟹ no carrier ever appears");
}

// ---------------------------------------------------------------------------
// Pre-registered degeneracy under BIRTH-DISPERSAL: replication_threshold → floor
// now has a SPATIAL signature — a depleted halo around a low-threshold lineage
// (docs/experiment5_preregistration.md §6). Under node-funded target-pays, a
// carrier with replication_threshold < 0 is ALWAYS fertile and spams children
// onto its unoccupied neighbours (one per step) at zero cost to itself, draining
// each neighbour by the child cost — a ring of depleted, freshly-occupied sites.
// ---------------------------------------------------------------------------

/// A `replication_threshold = -1` founder at the centre of a star of fundable,
/// unoccupied neighbours colonises every neighbour (one per step, MaxParallel
/// conflicts on the shared parent node), pays the cost from each TARGET, and is
/// itself never drained — the degenerate attractor, spatially realised.
#[test]
fn negative_replication_threshold_spams_children_onto_the_halo() {
    // Centre site + founder (repl_threshold = -1 ⟹ always fertile).
    let (mut store, centre, founder) = genome_carrier(50.0, 5.0, -1.0, 20.0);
    store.get_node_mut(centre).unwrap().set_property(22, PropertyValue::Bool(true)); // occupied

    // A halo of N fundable, unoccupied neighbours, each joined to the centre.
    const N: usize = 6;
    let mut halo = Vec::new();
    for _ in 0..N {
        let t = store.add_node(NodeData::new(Label(0), 0));
        {
            let d = store.get_node_mut(t).unwrap();
            d.set_property(ENERGY, PropertyValue::Float(20.0)); // fundable (>= child cost 10)
            d.set_property(22, PropertyValue::Bool(false));
        }
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[centre, t])));
        halo.push(t);
    }

    let mut engine = Engine::new(only_rule("replicate"), StepMode::MaxParallel, 7, 100000);
    for step in 1..=(N as u32 + 3) {
        let before = carrier_count(&store);
        engine.step(&mut store);
        let after = carrier_count(&store);
        // A free fundable neighbour still exists ⟹ exactly one more child (the
        // shared founder node forces one birth per step under MaxParallel).
        let free = halo
            .iter()
            .filter(|&&t| matches!(store.get_node(t).unwrap().get_property(22), Some(PropertyValue::Bool(false))))
            .count();
        if before < 1 + N && free > 0 {
            assert_eq!(after, before + 1, "fundable step {step} produced no child");
        }
    }

    // The whole halo is colonised: N children, founder never drained (node-funded),
    // and every neighbour is now occupied and depleted by exactly the child cost.
    assert_eq!(carrier_count(&store), 1 + N, "founder colonised every halo site");
    assert_eq!(energy(&store, founder), 5.0, "founder energy UNTOUCHED — zero self-cost");
    for &t in &halo {
        assert_eq!(energy(&store, t), 10.0, "each halo site paid the child cost (20 → 10)");
        assert!(
            matches!(store.get_node(t).unwrap().get_property(22), Some(PropertyValue::Bool(true))),
            "each halo site is now occupied"
        );
    }
}
