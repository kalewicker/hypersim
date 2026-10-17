//! Criterion 7 (hard gate, outcome-independent) — the energy books must balance to
//! the DECLARED entropy loss at EVERY point of the influx sweep, not just one.
//!
//! The sweep varies influx; if the accounting only closed at one rate the band would
//! be uninterpretable. The load-bearing term is the death RESIDUAL: a carrier dies at
//! energy r ≤ 0 and deleting it removes that negative energy, so death contributes
//! recycle − r ≥ recycle. That term scales with how deep carriers overshoot below
//! zero, which is a function of the economy (hence of influx). This test drives the
//! PRODUCTION path (`runner::load_rules` with the `influx_rate` override) at several
//! rates spanning turnover and frozen regimes, and asserts the per-step ledger
//! identity E(t) = E(0) + seed + influx − maintenance − birth_entropy
//!                 + (death_recycle − death_residual) holds to f64 tolerance at each.
#![cfg(feature = "alife")]

use hg_core::{GraphSnapshot, HypergraphStore, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
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

#[derive(Default)]
struct Ledger {
    ledger: f64,
    influx: f64,
    maintenance: f64,
    birth_entropy: f64,
    death_recycle: f64,
    death_residual: f64,
    births: u64,
    deaths: u64,
    max_divergence: f64,
    prev_carrier_energy: HashMap<NodeId, f64>,
    final_carriers: usize,
}

struct LedgerObs {
    influx_rate: f64,
    state: Arc<Mutex<Ledger>>,
}
impl Observer for LedgerObs {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, events: &[RewriteEvent]) {
        let mut s = self.state.lock().unwrap();
        let rate = self.influx_rate;
        for e in events {
            match e.rule_name.as_str() {
                "bootstrap_seed" => s.ledger += 10.0,
                "influx" => {
                    s.ledger += rate;
                    s.influx += rate;
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
        let measured = total_energy_snap(snap);
        let div = (s.ledger - measured).abs();
        if div > s.max_divergence {
            s.max_divergence = div;
        }
        s.prev_carrier_energy = snap
            .nodes()
            .filter(|(_, d)| d.label == CARRIER)
            .filter_map(|(id, d)| match d.get_property(ENERGY) {
                Some(PropertyValue::Float(v)) => Some((id, v)),
                _ => None,
            })
            .collect();
        s.final_carriers = snap.nodes().filter(|(_, d)| d.label == CARRIER).count();
    }
}

fn config(influx: f64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "crit7"
run_id = "crit7"
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
initial_substrate_energy = 5.0
influx_rate = {influx}
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn run_ledger(influx: f64, steps: u64) -> Ledger {
    let cfg = config(influx, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap(); // PRODUCTION path
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);
    let mut init = Ledger { ledger: total_energy(&store), ..Default::default() };
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
    engine.add_observer(Box::new(LedgerObs { influx_rate: influx, state: state.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    Arc::try_unwrap(state).map_err(|_| ()).unwrap().into_inner().unwrap()
}

#[test]
fn criterion7_energy_balances_at_every_sweep_point() {
    let steps = 8_000u64;
    let rates = [0.28, 0.34, 0.40, 0.50, 1.0];

    eprintln!("\n===== CRITERION 7 — energy books balance across the influx sweep =====");
    eprintln!("  influx | births deaths | Σinflux  Σmaint  Σbirth_ent  Σrecycle  Σresidual | maxDiv | finalC");
    eprintln!("  -------+---------------+-----------------------------------------------+--------+-------");
    for &r in &rates {
        let l = run_ledger(r, steps);
        eprintln!(
            "  {r:>5.2}  | {:>6} {:>6} | {:>7.1} {:>7.1} {:>10.1} {:>9.1} {:>9.3} | {:.1e} | {}",
            l.births, l.deaths, l.influx, l.maintenance, l.birth_entropy, l.death_recycle, l.death_residual,
            l.max_divergence, l.final_carriers
        );
        // The hard gate: books balance to f64 tolerance at THIS influx rate.
        assert!(
            l.max_divergence < 1e-6,
            "ledger diverged by {:.3e} at influx {r} — accounting does not close at this sweep point",
            l.max_divergence
        );
        // Death residual must be a NON-negative source that is actually exercised
        // wherever deaths occur (the term the plan flagged at S3).
        if l.deaths > 0 {
            assert!(l.death_residual >= 0.0, "residual is a source (Σ|r| ≥ 0)");
        }
    }
    eprintln!("  ⟹ books close at EVERY sweep point (turnover AND frozen); the band is interpretable.");
    eprintln!("=====================================================================\n");
}
