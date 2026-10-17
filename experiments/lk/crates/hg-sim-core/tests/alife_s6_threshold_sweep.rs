//! S6 / §8 — the replication_threshold × influx sweep. Unlike child_node_cost (a
//! symmetric SCALE that moves birth and death thresholds in lockstep, leaving the
//! ratio 1.47 invariant — §8), `replication_threshold` is ASYMMETRIC: it appears
//! only in the birth condition (`energy ≥ threshold`), never in death (`energy ≤ 0`).
//! Lowering it lowers the birth/startup floor toward the sustain floor (~0.333)
//! WITHOUT touching the mortality ceiling — the degree of freedom child_node_cost
//! lacked. This can, in principle, reopen a turnover band on the static substrate.
//!
//! CAVEAT (reported, not asserted): `replication_threshold` is a genome trait under
//! jitter + selection, not a physics constant. Sweeping the BOOTSTRAP value sweeps
//! an INITIAL CONDITION; with node-funded replication nothing opposes it drifting
//! to its floor, so a band found here is a band in initial conditions. The mean-
//! threshold trajectory is reported so that drift is visible.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, EffectOpDecl, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);

/// Rules with influx add-value = `influx` and the BOOTSTRAP replication_threshold
/// (genome key 11) = `thr`. Descendants inherit via copy_from+jitter.
fn rules_with(influx: f64, thr: f64) -> Vec<CompiledRule> {
    let mut raw = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    let mut hits = 0;
    for r in raw.iter_mut() {
        match r.name.as_str() {
            "influx" => {
                for eff in &mut r.rhs_property_effects {
                    if eff.key == 1 {
                        if let EffectOpDecl::Add { value } = &mut eff.op {
                            *value = PropertyValue::Float(influx);
                            hits += 1;
                        }
                    }
                }
            }
            "bootstrap_seed" => {
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "carrier" && eff.key == 11 {
                        if let EffectOpDecl::Set { value } = &mut eff.op {
                            *value = PropertyValue::Float(thr);
                            hits += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(hits, 2, "expected to patch influx add + bootstrap threshold");
    compile_rules(&raw).unwrap()
}

#[derive(Default, Debug)]
struct Obs {
    births: Vec<u32>,
    deaths: Vec<u32>,
    carriers: Vec<usize>,
}
struct Handle {
    inner: Arc<Mutex<Obs>>,
    alife: Arc<Mutex<AlifeObserver>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, events: &[RewriteEvent]) {
        {
            let mut o = self.inner.lock().unwrap();
            o.births.push(events.iter().filter(|e| e.rule_name == "replicate").count() as u32);
            o.deaths.push(events.iter().filter(|e| e.rule_name == "die_and_recycle").count() as u32);
            o.carriers.push(snap.nodes().filter(|(_, d)| d.label == CARRIER).count());
        }
        self.alife.lock().unwrap().on_step_end(snap, s, events);
    }
}

fn cfg(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "thr_sweep"
run_id = "thr_sweep"
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

struct Cell {
    regime: &'static str,
    final_c: usize,
    peak: usize,
    births: u64,
    deaths: u64,
    b_last: u64,
    d_last: u64,
    thr_first: f64,
    thr_last: f64,
}

fn run_cell(influx: f64, thr: f64, steps: u64) -> Cell {
    let config = cfg(steps);
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules_with(influx, thr), StepMode::MaxParallel, config.seed, 100000);
    let obs = Arc::new(Mutex::new(Obs::default()));
    let alife = Arc::new(Mutex::new(AlifeObserver::new(steps / 5)));
    engine.add_observer(Box::new(Handle { inner: obs.clone(), alife: alife.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let o = Arc::try_unwrap(obs).unwrap().into_inner().unwrap();
    let a = alife.lock().unwrap();
    let n = o.carriers.len();
    let w = (steps as usize / 5).max(1);
    let births: u64 = o.births.iter().map(|&x| x as u64).sum();
    let deaths: u64 = o.deaths.iter().map(|&x| x as u64).sum();
    let b_last: u64 = o.births[n - w..].iter().map(|&x| x as u64).sum();
    let d_last: u64 = o.deaths[n - w..].iter().map(|&x| x as u64).sum();
    let final_c = *o.carriers.last().unwrap();
    let peak = *o.carriers.iter().max().unwrap();
    // Mean replication_threshold (trait index 1) first vs last window.
    let win = a.windows();
    let thr_first = win.iter().find(|w| w.carriers > 0).map(|w| w.trait_mean[1]).unwrap_or(f64::NAN);
    let thr_last = win.iter().rev().find(|w| w.carriers > 0).map(|w| w.trait_mean[1]).unwrap_or(f64::NAN);
    let regime = if final_c == 0 {
        "EXTINCT"
    } else if b_last > 0 && d_last > 0 {
        "TURNOVER"
    } else if b_last == 0 && d_last == 0 {
        "FROZEN"
    } else {
        "TRANSIENT"
    };
    Cell { regime, final_c, peak, births, deaths, b_last, d_last, thr_first, thr_last }
}

#[test]
fn s6_threshold_influx_sweep() {
    let steps = 12_000u64;
    let influxes = [0.28, 0.34, 0.40, 0.48];
    let thresholds = [3.0, 6.0, 15.0];

    eprintln!("\n============ S6 replication_threshold × influx sweep ============");
    eprintln!("sustain r*≈0.333; asymmetric lever (birth-only). {steps} steps, 36-site static crystal.\n");
    eprint!("    thr\\r  |");
    for r in influxes {
        eprint!("  {r:>5.2}  |");
    }
    eprintln!();
    eprintln!("    -------+{}", "---------+".repeat(influxes.len()));

    let mut grid = Vec::new();
    let mut any_turnover = None;
    for &thr in &thresholds {
        eprint!("    {thr:>5.1}  |", thr = thr);
        let mut row = Vec::new();
        for &r in &influxes {
            let cell = run_cell(r, thr, steps);
            eprint!(" {:<8}|", cell.regime);
            if cell.regime == "TURNOVER" && any_turnover.is_none() {
                any_turnover = Some((thr, r));
            }
            row.push(cell);
        }
        eprintln!();
        grid.push((thr, row));
    }

    eprintln!("\n  detail (non-frozen, non-extinct cells; thr drift = selection signal):");
    for (thr, row) in &grid {
        for (i, cell) in row.iter().enumerate() {
            if cell.regime == "TURNOVER" || cell.regime == "TRANSIENT" {
                eprintln!(
                    "    thr0={thr:.1} r={:.2}: {} final={} peak={} births={} deaths={} b_last={} d_last={} | thr {:.2}→{:.2}",
                    influxes[i], cell.regime, cell.final_c, cell.peak, cell.births, cell.deaths,
                    cell.b_last, cell.d_last, cell.thr_first, cell.thr_last
                );
            }
        }
    }

    match any_turnover {
        Some((thr, r)) => {
            eprintln!("\n  ⟹ TURNOVER at bootstrap thr={thr:.1}, influx={r:.2} (INITIAL-CONDITION-dependent:");
            eprintln!("     threshold is a jittered genome trait; watch the thr drift column for collapse).");
        }
        None => {
            eprintln!("\n  ⟹ NO turnover cell. Even the asymmetric lever leaves frozen↔extinct on a");
            eprintln!("     homogeneous substrate — synchronized death survives; route A genuinely earned.");
        }
    }
    eprintln!("================================================================\n");

    let regimes: Vec<&str> = grid.iter().flat_map(|(_, row)| row.iter().map(|c| c.regime)).collect();
    assert!(regimes.iter().any(|&x| x == "EXTINCT"), "some cell must starve");
    assert!(regimes.iter().any(|&x| x != "EXTINCT"), "some cell must sustain a population");
}
