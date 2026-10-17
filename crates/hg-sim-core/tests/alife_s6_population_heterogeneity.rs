//! S6 / §8 cheap checks (before the replication_threshold sweep): is the
//! endogenous, genome-sourced heterogeneity already enough to desynchronise
//! starvation, and is selection already acting on the "frozen" saturated run?
//!
//! The substrate is homogeneous, but the POPULATION is not: carriers inherit
//! satiation ± jitter, and satiation sets the energy each carrier idles at, so a
//! uniform drain drives carriers pinned at different levels to zero at different
//! times. If the live satiation spread is ≥ ~1.47× (the influx-window ratio from
//! §7), the desync budget route A would manufacture is already present.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::{GraphSnapshot, HypergraphStore, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);
const REPL_THR: u32 = 11;
const SATIATION: u32 = 13;
const TRAIT_KEYS: [u32; 4] = [10, 11, 12, 13];
const TRAIT_NAMES: [&str; 4] = ["harvest_rate", "repl_threshold", "maint_cost", "satiation"];

struct Handle {
    inner: Arc<Mutex<AlifeObserver>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, events: &[RewriteEvent]) {
        self.inner.lock().unwrap().on_step_end(snap, s, events);
    }
}

fn cfg(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "het"
run_id = "het"
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
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

fn all_rules() -> Vec<hg_rules::CompiledRule> {
    compile_rules(&parse_rules_toml(hg_alife::alife_base_toml()).unwrap()).unwrap()
}

/// (mean, var, min, max) of a trait over the live carriers.
fn dist(store: &HypergraphStore, key: u32) -> (f64, f64, f64, f64) {
    let vals: Vec<f64> = store
        .nodes()
        .filter(|(_, d)| d.label == CARRIER)
        .filter_map(|(_, d)| match d.get_property(key) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        })
        .collect();
    let n = vals.len() as f64;
    let mean = vals.iter().sum::<f64>() / n;
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (mean, var, min, max)
}

#[test]
fn population_heterogeneity_and_selection_signal() {
    let steps = 8_000u64;
    let config = cfg(steps);
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(all_rules(), StepMode::MaxParallel, config.seed, 100000);
    let obs = Arc::new(Mutex::new(AlifeObserver::new(1_000)));
    engine.add_observer(Box::new(Handle { inner: obs.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);

    eprintln!("\n============ population heterogeneity @ frozen state ============");
    eprintln!("  trait            mean      var       min       max     max/min");
    eprintln!("  ---------------+---------+---------+---------+---------+--------");
    let mut satiation_ratio = 1.0;
    for (i, &k) in TRAIT_KEYS.iter().enumerate() {
        let (m, v, lo, hi) = dist(&store, k);
        let ratio = if lo.abs() > 1e-9 { hi / lo } else { f64::INFINITY };
        eprintln!(
            "  {:<14} | {:>7.3} | {:>7.4} | {:>7.3} | {:>7.3} | {:>6.3}",
            TRAIT_NAMES[i], m, v, lo, hi, ratio
        );
        if k == SATIATION {
            satiation_ratio = ratio;
        }
    }

    // Sterility check: carriers whose jittered threshold exceeds their OWN
    // satiation can never climb to reproduce (satiation caps harvest).
    let sterile = store
        .nodes()
        .filter(|(_, d)| d.label == CARRIER)
        .filter(|(_, d)| {
            let thr = match d.get_property(REPL_THR) {
                Some(PropertyValue::Float(v)) => v,
                _ => return false,
            };
            let sat = match d.get_property(SATIATION) {
                Some(PropertyValue::Float(v)) => v,
                _ => return false,
            };
            thr > sat
        })
        .count();
    let total = store.nodes().filter(|(_, d)| d.label == CARRIER).count();
    eprintln!("\n  carriers with repl_threshold > own satiation (structurally sterile): {sterile}/{total}");

    // Selection signal: trajectory of each trait mean across windows. A directional
    // shift (or variance collapse) would be criterion 5 in a supposedly inert run.
    let o = obs.lock().unwrap();
    let w = o.windows();
    if w.len() >= 2 {
        eprintln!("\n  trait-mean trajectory (first vs last window):");
        for (i, name) in TRAIT_NAMES.iter().enumerate() {
            let f = w.first().unwrap().trait_mean[i];
            let l = w.last().unwrap().trait_mean[i];
            let fv = w.first().unwrap().trait_var[i];
            let lv = w.last().unwrap().trait_var[i];
            eprintln!(
                "    {name:<15}: mean {f:>7.3} → {l:>7.3} ({:+.3});  var {fv:>6.4} → {lv:>6.4}",
                l - f
            );
        }
    }

    eprintln!("\n  satiation max/min = {satiation_ratio:.3}  vs desync target ~1.47");
    if satiation_ratio >= 1.47 {
        eprintln!("  ⟹ endogenous satiation spread ALREADY spans the desync budget — route A's");
        eprintln!("     motivation (manufacturing heterogeneity) is undercut; test the threshold sweep.");
    } else {
        eprintln!("  ⟹ satiation spread is BELOW the desync budget: endogenous genome heterogeneity");
        eprintln!("     is insufficient to stagger deaths; supports route A / a real threshold barrier.");
    }
    eprintln!("================================================================\n");

    assert!(total > 0, "a live population must exist to measure");
}
