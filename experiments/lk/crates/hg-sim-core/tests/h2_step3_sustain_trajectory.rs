//! Step 3 / H2.1 diagnostic — what actually SUSTAINS a non-replicating population?
//!
//! The G-track sterile token died at step 120 on static (pinned to one depleting
//! site, influx<maintenance, can't move). The V4/V5 hope was that a churn substrate
//! sustains a non-replicating population via "fresh nodes recolonised". But churn
//! mints fresh FREE sites, and a non-replicating, NON-MOTILE carrier has no way to
//! reach them (no birth, no movement). So this measures the four arms to locate the
//! actual sustaining mechanism:
//!   - replicating (F8)                : sustains via birth-dispersal (baseline)
//!   - sterile, non-motile             : the G-track token (dies ~120)
//!   - sterile, MOTILE                 : can a non-replicating carrier that RELOCATES
//!                                        to fresh sites sustain? on static? on churn?
//! Motility = the SAME individual relocates its occupancy to an adjacent unoccupied
//! higher-energy site when hungry. No child, no heredity — movement, not replication.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_core::{NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::runner::apply_alife_overrides;
use hg_sim_core::SimConfig;

const STEPS: u64 = 6_000;
const ENERGY: u32 = 1;

fn config(seed: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "s3"
run_id = "s3"
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
max_steps = {STEPS}
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = true
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

// churn rule only (no influx; biology supplies it).
fn churn_toml() -> &'static str {
    r#"
[[rules]]
name = "churn"
priority = 2
[rules.lhs]
nodes = ["a", "old", "b"]
edges = [["a", "old"], ["old", "b"]]
[rules.lhs.node_labels]
a = 0
old = 0
b = 0
[[rules.lhs.node_property_guards]]
var = "old"
property = 1
op = "gte"
value = 20.0
[[rules.lhs.node_property_guards]]
var = "old"
property = 22
op = "eq"
value = false
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "old", "b"]
[rules.rhs]
nodes = ["a", "fresh", "b"]
edges = [["a", "fresh"], ["fresh", "b"]]
[rules.rhs.node_labels]
fresh = 0
[[rules.rhs.property_effects]]
var = "fresh"
key = 1
op = "set"
value = 0.0
[[rules.rhs.property_effects]]
var = "fresh"
key = 22
op = "set"
value = false
"#
}

// MOTILITY: a hungry carrier on `site` relocates to an adjacent unoccupied `target`
// with harvestable energy. The occupancy edge moves site->target; no child created,
// no genome copied — the same individual moves. site freed, target occupied.
fn motility_toml() -> &'static str {
    r#"
[[rules]]
name = "relocate"
priority = 2
[rules.lhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "carrier"], ["site", "target"]]
[rules.lhs.node_labels]
carrier = 100
site = 0
target = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["site", "carrier", "target"]
[[rules.lhs.node_property_compares]]
var_a = "carrier"
key_a = 1
op = "lt"
var_b = "carrier"
key_b = 13
[[rules.lhs.node_property_guards]]
var = "target"
property = 1
op = "gte"
value = 1.0
[[rules.lhs.node_property_guards]]
var = "target"
property = 22
op = "eq"
value = false
[[rules.lhs.node_property_guards]]
var = "site"
property = 1
op = "lt"
value = 1.0
[rules.rhs]
nodes = ["site", "carrier", "target"]
edges = [["site", "target"], ["target", "carrier"]]
[[rules.rhs.property_effects]]
var = "site"
key = 22
op = "set"
value = false
[[rules.rhs.property_effects]]
var = "target"
key = 22
op = "set"
value = true
"#
}

fn build_rules(cfg: &SimConfig, sterile: bool, motile: bool, churn: bool) -> Vec<hg_rules::CompiledRule> {
    let mut biology = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    apply_alife_overrides(&mut biology, &cfg.alife);
    if sterile {
        biology.retain(|r| r.name != "replicate");
    }
    if motile {
        biology.extend(parse_rules_toml(motility_toml()).unwrap());
    }
    if churn {
        biology.extend(parse_rules_toml(churn_toml()).unwrap());
    }
    compile_rules(&biology).unwrap()
}

struct Traj { peak: usize, final_pop: usize, last_alive: u64, fresh_churn: u64 }

fn run(seed: u64, sterile: bool, motile: bool, churn: bool) -> Traj {
    let cfg = config(seed);
    let rules = build_rules(&cfg, sterile, motile, churn);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    let mut peak = 0usize;
    let mut final_pop = 0usize;
    let mut last_alive = 0u64;
    let mut fresh_churn = 0u64;
    let mut prev_ids: std::collections::HashSet<NodeId> = store.nodes().map(|(id, _)| id).collect();
    for step in 1..=STEPS {
        engine.step(&mut store);
        let snap = store.snapshot();
        let mut live = 0usize;
        let now_ids: std::collections::HashSet<NodeId> = snap.nodes().map(|(id, _)| id).collect();
        for (_, d) in snap.nodes() {
            if d.label == CARRIER_LABEL { live += 1; }
        }
        fresh_churn += now_ids.difference(&prev_ids).count() as u64;
        prev_ids = now_ids;
        peak = peak.max(live);
        final_pop = live;
        if live > 0 { last_alive = step; }
    }
    Traj { peak, final_pop, last_alive, fresh_churn }
}

#[test]
fn step3_sustain_trajectory() {
    let seed = 12345;
    eprintln!("\n================ STEP 3 / H2.1 — WHAT SUSTAINS A NON-REPLICATING POPULATION? (seed {seed}) ================");
    eprintln!("  arm                              | peak | final | last-alive | fresh-churn total");
    eprintln!("  ---------------------------------+------+-------+------------+------------------");
    let arms: [(&str, bool, bool, bool); 6] = [
        ("replicating (F8)              static", false, false, false),
        ("sterile non-motile           static", true,  false, false),
        ("sterile non-motile           churn ", true,  false, true),
        ("sterile MOTILE               static", true,  true,  false),
        ("sterile MOTILE               churn ", true,  true,  true),
        ("replicating MOTILE           churn ", false, true,  true),
    ];
    let mut motile_static_sustains = false;
    let mut motile_churn_sustains = false;
    for (name, s, m, c) in arms {
        let t = run(seed, s, m, c);
        eprintln!("  {:<32} | {:>4} | {:>5} | {:>10} | {:>16}", name, t.peak, t.final_pop, t.last_alive, t.fresh_churn);
        if name.contains("sterile MOTILE") && name.contains("static") { motile_static_sustains = t.last_alive > 3000; }
        if name.contains("sterile MOTILE") && name.contains("churn") { motile_churn_sustains = t.last_alive > 3000; }
    }
    eprintln!("=================================================================================================\n");
    eprintln!("Reading: the G-track token (sterile non-motile) dies ~120 on static AND on churn (churn mints fresh");
    eprintln!("FREE sites but a stuck non-motile carrier cannot reach them). If sterile MOTILE sustains, MOVEMENT to");
    eprintln!("fresh energy -- not replication -- is what provides duration; that is the sustained non-replicating");
    eprintln!("specimen H2 needs. Whether it needs CHURN or grazing on STATIC alone suffices is read from the two rows.");
    eprintln!("  sterile-motile sustains: static={motile_static_sustains}, churn={motile_churn_sustains}");
}
