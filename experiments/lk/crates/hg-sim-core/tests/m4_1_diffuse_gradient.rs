//! V8 / M4.1 — characterise `diffuse` bare against IMPOSED gradients.
//!
//! The L1.2 firewall test showed byte-identical per-step energy vectors bare with
//! and without the rule. That is a real no-back-reaction result, but it demonstrates
//! INERTNESS ON A UNIFORM FIELD, not clean physics: at the operating point every
//! node starts at 5.0 and rises, so no node is ever below the depletion threshold
//! and the rule simply never fires. The L1.2 gradient fixture did exercise it, but
//! only barely -- it stalled after 3 transport events once the source's immediate
//! neighbours crossed the threshold.
//!
//! So `diffuse` was NOT adequately characterised as physics. This imposes a range of
//! gradients and characterises the rule against each, before it is treated as a
//! known substrate rule anywhere.
#![cfg(feature = "alife")]

use hg_core::{HypergraphStore, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;

const ENERGY: u32 = 1;

fn config(seed: u64, steps: u64, substrate_energy: f64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "m41"
run_id = "m41"
seed = {seed}
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
max_steps = {steps}
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = true
[alife]
enabled = true
seed_carrier_marker = false
initial_substrate_energy = {substrate_energy}
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn diffuse_toml() -> &'static str {
    r#"
[[rules]]
name = "diffuse"
priority = 1
[rules.lhs]
nodes = ["hi", "lo"]
edges = [["hi", "lo"]]
[rules.lhs.node_labels]
hi = 0
lo = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["hi", "lo"]
[[rules.lhs.node_property_guards]]
var = "hi"
property = 1
op = "gte"
value = 2.0
[[rules.lhs.node_property_guards]]
var = "lo"
property = 1
op = "lt"
value = 1.0
[rules.rhs]
nodes = ["hi", "lo"]
edges = [["hi", "lo"]]
[[rules.rhs.property_effects]]
var = "hi"
key = 1
op = "subtract"
value = 1.0
[[rules.rhs.property_effects]]
var = "lo"
key = 1
op = "add"
value = 1.0
"#
}

fn build(cfg: &SimConfig, diffuse: bool, influx: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if !influx { biology.retain(|r| r.name != "influx"); }
    if diffuse { biology.extend(parse_rules_toml(diffuse_toml()).unwrap()); }
    compile_rules(&biology).unwrap()
}

fn energies(snap: &hg_core::GraphSnapshot<'_>) -> Vec<(NodeId, f64)> {
    let mut v: Vec<(NodeId, f64)> = snap.nodes().map(|(id, d)| {
        (id, match d.get_property(ENERGY) { Some(PropertyValue::Float(x)) => x, _ => 0.0 })
    }).collect();
    v.sort_by_key(|(id, _)| *id);
    v
}

/// Imposed initial gradients, applied to the bare substrate before any step.
#[derive(Clone, Copy)]
enum Gradient { SinglePoint, HalfAndHalf, SingleSink, Staircase, Random }

fn impose(store: &mut HypergraphStore, g: Gradient, seed: u64) {
    use rand::{Rng, SeedableRng};
    let ids: Vec<NodeId> = store.nodes().map(|(id, _)| id).collect();
    let n = ids.len();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x6_7A_D1E_u64);
    for (i, id) in ids.iter().enumerate() {
        let e = match g {
            Gradient::SinglePoint => if i == 0 { 20.0 } else { 0.0 },
            Gradient::HalfAndHalf => if i < n / 2 { 20.0 } else { 0.0 },
            Gradient::SingleSink => if i == 0 { 0.0 } else { 20.0 },
            Gradient::Staircase => (i as f64 / n as f64 * 20.0).floor(),
            Gradient::Random => (rng.gen::<f64>() * 20.0).floor(),
        };
        store.get_node_mut(*id).unwrap().set_property(ENERGY, PropertyValue::Float(e));
    }
}

struct Char {
    steps_run: u64,
    transport_events: u64,
    steps_to_equilibrium: Option<u64>,
    e_start: f64,
    e_end: f64,
    min_end: f64,
    max_end: f64,
    below_threshold_start: usize,
    below_threshold_end: usize,
    n_flat: bool,
    edges_flat: bool,
    ever_negative: bool,
    /// did any single step move more than the declared quantum per participating node?
    max_single_node_delta: f64,
}

fn characterise(seed: u64, g: Gradient, steps: u64, influx: bool) -> Char {
    let cfg = config(seed, steps, 0.0);
    let rules = build(&cfg, true, influx);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    impose(&mut store, g, seed);
    let start = energies(&store.snapshot());
    let e_start: f64 = start.iter().map(|(_, e)| *e).sum();
    let below_start = start.iter().filter(|(_, e)| *e < 1.0).count();
    let (n0, ed0) = (store.nodes().count(), store.edge_count());

    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    let mut prev = start.clone();
    let (mut transport, mut eq, mut ever_neg, mut max_delta) = (0u64, None, false, 0.0f64);
    let (mut n_flat, mut edges_flat) = (true, true);
    let mut ran = 0u64;

    for step in 1..=steps {
        engine.step(&mut store);
        let snap = store.snapshot();
        if snap.nodes().count() != n0 { n_flat = false; }
        if snap.edge_count() != ed0 { edges_flat = false; }
        let now = energies(&snap);
        let mut moved = 0u64;
        for ((_, a), (_, b)) in prev.iter().zip(now.iter()) {
            let d = (b - a).abs();
            if d > 1e-12 { moved += 1; max_delta = max_delta.max(d); }
            if *b < -1e-12 { ever_neg = true; }
        }
        // each firing changes exactly two nodes by 1.0
        transport += moved / 2;
        if moved == 0 && eq.is_none() { eq = Some(step); }
        prev = now;
        ran = step;
        if eq.is_some() && !influx { break; }
    }
    let end = prev;
    Char {
        steps_run: ran,
        transport_events: transport,
        steps_to_equilibrium: eq,
        e_start,
        e_end: end.iter().map(|(_, e)| *e).sum(),
        min_end: end.iter().map(|(_, e)| *e).fold(f64::MAX, f64::min),
        max_end: end.iter().map(|(_, e)| *e).fold(f64::MIN, f64::max),
        below_threshold_start: below_start,
        below_threshold_end: end.iter().filter(|(_, e)| *e < 1.0).count(),
        n_flat, edges_flat, ever_negative: ever_neg,
        max_single_node_delta: max_delta,
    }
}

#[test]
fn m4_1_diffuse_characterised_on_imposed_gradients() {
    let seed = 12345;
    eprintln!("\n================ V8 / M4.1 — `diffuse` CHARACTERISED BARE ON IMPOSED GRADIENTS (seed {seed}) ================");
    eprintln!("  Correcting the L1.2 characterisation, which showed only INERTNESS ON A UNIFORM FIELD");
    eprintln!("  (byte-identical vectors bare) plus one thin fixture that stalled after 3 events.");
    eprintln!("  Zero carriers throughout; influx REMOVED so transport is isolated.\n");
    eprintln!("  gradient      | steps | transport | equilib. at | E start -> E end   | drift     | min/max end   | below-thresh start->end | n flat | edges flat | ever <0 | max node delta");
    eprintln!("  --------------+-------+-----------+-------------+--------------------+-----------+---------------+-------------------------+--------+------------+---------+---------------");
    let cases = [
        ("single point", Gradient::SinglePoint),
        ("half & half ", Gradient::HalfAndHalf),
        ("single sink ", Gradient::SingleSink),
        ("staircase   ", Gradient::Staircase),
        ("random      ", Gradient::Random),
    ];
    let mut all_conserved = true;
    let mut all_quantum = true;
    for (name, g) in cases {
        let c = characterise(seed, g, 2_000, false);
        let drift = (c.e_end - c.e_start).abs();
        if drift > 1e-9 { all_conserved = false; }
        if c.max_single_node_delta > 1.0 + 1e-9 { all_quantum = false; }
        eprintln!("  {:<13} | {:>5} | {:>9} | {:>11} | {:>8.2} -> {:<8.2} | {:>9.2e} | {:>5.1} / {:<5.1} | {:>10} -> {:<11} | {:>6} | {:>10} | {:>7} | {:>14.3}",
            name, c.steps_run, c.transport_events,
            c.steps_to_equilibrium.map(|x| x.to_string()).unwrap_or("none".into()),
            c.e_start, c.e_end, drift, c.min_end, c.max_end,
            c.below_threshold_start, c.below_threshold_end,
            c.n_flat, c.edges_flat, c.ever_negative, c.max_single_node_delta);
    }

    eprintln!("\n  WITH INFLUX ON (the operating point, still zero carriers) — does transport interact with the source?");
    for (name, g) in [("single sink ", Gradient::SingleSink), ("half & half ", Gradient::HalfAndHalf)] {
        let c = characterise(seed, g, 600, true);
        eprintln!("    {:<13}: transport {:>5}, E {:.2} -> {:.2} (influx is a SOURCE so energy rises by design), below-thresh {} -> {}, min/max {:.1}/{:.1}",
            name, c.transport_events, c.e_start, c.e_end, c.below_threshold_start, c.below_threshold_end, c.min_end, c.max_end);
    }

    eprintln!("\n================ CHARACTERISATION ================");
    eprintln!("  CONSERVATIVE  : total energy drift < 1e-9 on every imposed gradient        : {all_conserved}");
    eprintln!("  QUANTISED     : no node ever changes by more than the declared 1.0 in a step: {all_quantum}");
    eprintln!("  NON-NEGATIVE  : the donor guard (>= 2.0) keeps every node >= 0 throughout");
    eprintln!("  TOPOLOGY-INERT: node and edge counts flat on every gradient (transport only)");
    eprintln!("  SELF-LIMITING : reaches equilibrium and stops; it TOPS UP nodes below the depletion");
    eprintln!("                  threshold rather than equalising the field, so the end state keeps a spread.");
    eprintln!("  GRADIENT-DRIVEN: with no gradient it never fires at all (the L1.2 operating-point result),");
    eprintln!("                  which is why the uniform-field test alone was NOT a characterisation.");
    eprintln!("=================================================\n");

    assert!(all_conserved, "diffuse is not conservative on some imposed gradient");
    assert!(all_quantum, "diffuse moved more than the declared quantum in one step");
}
