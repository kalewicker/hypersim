//! Pre-S4 test area 1 + 4: the closed-economy ledger identity, asserted every
//! step with ALL rules composed, plus determinism of the energy trace.
//!
//! The identity (per-event ΔE on total energy E = Σ property key 1):
//!   bootstrap_seed : +10.0        (carrier born with seed energy; site untouched)
//!   influx         : + rate       (the intended source)
//!   harvest        :   0          (site→carrier transfer)
//!   metabolise     : − maint      (maintenance sink)
//!   replicate      : − 2.0        (site −cost 10, child +birth 8 ⟹ −2 birth entropy)
//!   die_and_recycle: +recycle − r (r = the dying carrier's energy, ≤ 0; recycle 8)
//!
//! The death term is the load-bearing one: a carrier dies at energy r ≤ 0, and
//! deleting it REMOVES that (negative) energy, so death contributes recycle−r
//! ≥ recycle. If r were assumed 0, the books drift by Σ|r|. This test tracks r
//! exactly (from the carrier's energy at the previous step-end) and asserts the
//! ledger equals the measured total every step, reporting each term's REALIZED
//! magnitude.
#![cfg(feature = "alife")]

use hg_core::{GraphSnapshot, HypergraphStore, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::SimConfig;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const ENERGY: u32 = 1;
const CARRIER: hg_core::Label = hg_core::Label(100);

fn total_energy_snap(snap: &GraphSnapshot<'_>) -> f64 {
    snap.nodes()
        .filter_map(|(_, d)| match d.get_property(ENERGY) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        })
        .sum()
}

fn total_energy(store: &HypergraphStore) -> f64 {
    total_energy_snap(&store.snapshot())
}

#[derive(Default, Debug)]
struct Ledger {
    ledger: f64,
    seed: f64,
    influx: f64,
    maintenance: f64,
    birth_entropy: f64,
    death_recycle: f64,
    death_residual: f64, // Σ (−r), r ≤ 0, so ≥ 0
    births: u64,
    deaths: u64,
    influx_firings: Vec<usize>,
    carrier_counts: Vec<usize>,
    prev_carrier_energy: HashMap<NodeId, f64>,
    max_divergence: f64,
    energy_trace: Vec<u64>, // total-energy bits per step
}

struct LedgerObserver {
    state: Arc<Mutex<Ledger>>,
}

impl Observer for LedgerObserver {
    fn on_step_end(&mut self, snapshot: &GraphSnapshot<'_>, _stats: &StepStats, events: &[RewriteEvent]) {
        let mut s = self.state.lock().unwrap();
        for e in events {
            match e.rule_name.as_str() {
                "bootstrap_seed" => {
                    s.ledger += 10.0;
                    s.seed += 10.0;
                }
                "influx" => {
                    s.ledger += 2.0;
                    s.influx += 2.0;
                }
                "metabolise" => {
                    s.ledger -= 0.5;
                    s.maintenance += 0.5;
                }
                "replicate" => {
                    s.ledger -= 2.0;
                    s.birth_entropy += 2.0;
                    s.births += 1;
                }
                "die_and_recycle" => {
                    // Residual r = the dying carrier's energy at the previous
                    // step-end (die matched it at ≤ 0 at this step's start).
                    let mut r = 0.0;
                    for n in &e.nodes_deleted {
                        if let Some(&pe) = s.prev_carrier_energy.get(n) {
                            r += pe;
                        }
                    }
                    s.ledger += 8.0 - r;
                    s.death_recycle += 8.0;
                    s.death_residual += -r;
                    s.deaths += 1;
                }
                _ => {} // harvest: transfer, ΔE = 0
            }
        }
        s.influx_firings.push(events.iter().filter(|e| e.rule_name == "influx").count());

        let measured = total_energy_snap(snapshot);
        let div = (s.ledger - measured).abs();
        if div > s.max_divergence {
            s.max_divergence = div;
        }
        s.energy_trace.push(measured.to_bits());

        // Refresh the carrier-energy map for the NEXT step's death lookups.
        s.prev_carrier_energy = snapshot
            .nodes()
            .filter(|(_, d)| d.label == CARRIER)
            .filter_map(|(id, d)| match d.get_property(ENERGY) {
                Some(PropertyValue::Float(v)) => Some((id, v)),
                _ => None,
            })
            .collect();
        s.carrier_counts
            .push(snapshot.nodes().filter(|(_, d)| d.label == CARRIER).count());
    }
}

fn soak_config(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "ledger"
run_id = "ledger"
seed = 12345
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
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 0.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn all_rules() -> Vec<hg_rules::CompiledRule> {
    compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap()
}

/// Compile a named subset of alife_base, preserving relative order/priority.
fn rules_subset(names: &[&str]) -> Vec<hg_rules::CompiledRule> {
    let raw: Vec<_> = parse_rules_toml(hg_alife::alife_base_toml())
        .unwrap()
        .into_iter()
        .filter(|r| names.contains(&r.name.as_str()))
        .collect();
    compile_rules(&raw).unwrap()
}

/// Drive `rules` on `store` for `steps` with the ledger observer attached.
fn run_ledger_on(
    mut store: HypergraphStore,
    rules: Vec<hg_rules::CompiledRule>,
    seed: u64,
    steps: u64,
) -> Ledger {
    let mut engine = Engine::new(rules, StepMode::MaxParallel, seed, 100000);
    let mut init = Ledger {
        ledger: total_energy(&store), // E(0)
        ..Default::default()
    };
    // Seed the death-lookup map with any carriers already present at t=0.
    init.prev_carrier_energy = store
        .snapshot()
        .nodes()
        .filter(|(_, d)| d.label == CARRIER)
        .filter_map(|(id, d)| match d.get_property(ENERGY) {
            Some(PropertyValue::Float(v)) => Some((id, v)),
            _ => None,
        })
        .collect();
    let state = Arc::new(Mutex::new(init));
    engine.add_observer(Box::new(LedgerObserver { state: state.clone() }));

    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine); // releases the observer's Arc clone so we hold the sole ref
    Arc::try_unwrap(state).unwrap().into_inner().unwrap()
}

/// Run the composed ecology from a config-built substrate.
fn run_soak(steps: u64) -> Ledger {
    let config = soak_config(steps);
    let store = hg_sim_core::runner::build_initial_store(&config);
    run_ledger_on(store, all_rules(), config.seed, steps)
}

#[test]
fn closed_economy_ledger_balances_every_step() {
    let steps = 40_000u64;
    let l = run_soak(steps);

    eprintln!("--- ledger soak: {steps} steps ---");
    eprintln!("realized seed          (bootstrap): {:.1}", l.seed);
    eprintln!("realized influx  (SOURCE):          {:.1}  ({} firings)", l.influx, (l.influx / 2.0) as u64);
    eprintln!("realized maintenance (sink):        {:.1}", l.maintenance);
    eprintln!("realized birth entropy (sink):      {:.1}  ({} births)", l.birth_entropy, l.births);
    eprintln!("realized death recycle (source):    {:.1}  ({} deaths)", l.death_recycle, l.deaths);
    eprintln!("realized death residual (source):   {:.4}  (Σ|r| removed at death)", l.death_residual);
    eprintln!("final carriers: {:?}", l.carrier_counts.last());
    eprintln!(
        "realized influx per step: first10={:?} last10={:?}",
        &l.influx_firings[..10.min(l.influx_firings.len())],
        &l.influx_firings[l.influx_firings.len().saturating_sub(10)..]
    );
    eprintln!("MAX LEDGER DIVERGENCE: {:.3e}", l.max_divergence);

    assert!(
        l.max_divergence < 1e-6,
        "closed-economy ledger drifted by {:.3e} — an energy term is unaccounted",
        l.max_divergence
    );
    // The soak must at least be a live economy (replication happening). Deaths
    // require scarcity the abundant-substrate soak may not reach; the death
    // term is proven exactly by `death_residual_is_accounted` below.
    assert!(l.births > 0, "soak exercised no replication — economy is not live");
}

/// The death term, isolated and deterministic. A carrier that cannot harvest
/// (only metabolise + die active) drains below 0 and dies; the ledger must
/// account for the negative residual removed at deletion, still balancing to 0.
#[test]
fn death_residual_is_accounted() {
    let mut store = HypergraphStore::new();
    let site = store.add_node(hg_core::NodeData::new(hg_core::Label(0), 0));
    store.get_node_mut(site).unwrap().set_property(ENERGY, PropertyValue::Float(50.0));
    let carrier = store.add_node(hg_core::NodeData::new(CARRIER, 0));
    store.get_node_mut(carrier).unwrap().set_property(ENERGY, PropertyValue::Float(0.25));
    store.add_edge(hg_core::HyperEdge::new(
        hg_core::Label(0),
        smallvec::SmallVec::from_slice(&[site, carrier]),
    ));

    // metabolise: 0.25 → -0.25 (step 1). die fires step 2 (residual r = -0.25).
    let l = run_ledger_on(store, rules_subset(&["metabolise", "die_and_recycle"]), 7, 4);

    assert_eq!(l.deaths, 1, "the starving carrier must die");
    assert!(
        (l.death_residual - 0.25).abs() < 1e-9,
        "death residual should be exactly 0.25 (energy removed at death), got {}",
        l.death_residual
    );
    assert!(
        l.max_divergence < 1e-9,
        "ledger must still balance exactly with the residual accounted, drift {:.3e}",
        l.max_divergence
    );
}

#[test]
fn energy_trace_and_hash_are_deterministic() {
    // Two same-seed composed runs: bit-identical energy trace AND canonical hash.
    let a = run_soak(2_000);
    let b = run_soak(2_000);
    assert_eq!(a.energy_trace, b.energy_trace, "per-step total-energy trace is not deterministic");

    // Canonical hash via the full run pipeline, same seed, twice.
    let hash = |tag: &str| {
        let mut c = soak_config(2_000);
        c.out_dir = std::env::temp_dir().join(format!("hypersim_ledger_det_{tag}"));
        let _ = std::fs::remove_dir_all(&c.out_dir);
        hg_sim_core::run_headless(&c, None, None, None).unwrap().canonical_hash
    };
    assert_eq!(hash("a"), hash("b"), "canonical hash is not deterministic across same-seed runs");
}
