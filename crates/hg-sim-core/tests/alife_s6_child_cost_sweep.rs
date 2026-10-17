//! S6 / §8 — child_node_cost × influx re-sweep. The influx sweep proved an empty
//! turnover window between the SUSTAIN threshold (r*≈0.333) and the STARTUP
//! threshold (≈0.49). The startup threshold is set by `child_node_cost` (a target
//! must fill to it before a birth). Lowering `child_node_cost` lowers startup
//! toward sustain and may reopen a births≈deaths turnover band on the SAME static
//! homogeneous substrate — the diagnosis-named parameter adjustment (docs §8),
//! not a §8.1 forbidden move.
//!
//! `child_node_cost = c` scales four coupled values to keep the economy self-
//! consistent: the fundability guard (target.energy ≥ c), the target payment (−c),
//! child birth energy (0.8c), and the death recycle (0.8c, matched to birth
//! energy). Stillbirth floor: 0.8c must clear the maintenance quantum (0.5) ⟹ c≳0.7.
#![cfg(feature = "alife")]

use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, EffectOpDecl, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);

/// Rules with influx add-value = `influx` and child_node_cost = `c` (all four
/// coupled values scaled). Panics if any expected site fails to patch.
fn rules_with(influx: f64, c: f64) -> Vec<CompiledRule> {
    let birth = 0.8 * c;
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
            "replicate" => {
                for gd in &mut r.lhs_node_property_guards {
                    if gd.var == "target" && gd.property == 1 {
                        gd.value = PropertyValue::Float(c);
                        hits += 1;
                    }
                }
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "target" && eff.key == 1 {
                        if let EffectOpDecl::Subtract { value } = &mut eff.op {
                            *value = PropertyValue::Float(c);
                            hits += 1;
                        }
                    }
                    if eff.var == "child" && eff.key == 1 {
                        if let EffectOpDecl::Set { value } = &mut eff.op {
                            *value = PropertyValue::Float(birth);
                            hits += 1;
                        }
                    }
                }
            }
            "die_and_recycle" => {
                for eff in &mut r.rhs_property_effects {
                    if eff.var == "site" && eff.key == 1 {
                        if let EffectOpDecl::Add { value } = &mut eff.op {
                            *value = PropertyValue::Float(birth);
                            hits += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(hits, 5, "expected to patch 5 coupled values (influx, guard, pay, birth, recycle)");
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
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, events: &[RewriteEvent]) {
        let mut o = self.inner.lock().unwrap();
        o.births.push(events.iter().filter(|e| e.rule_name == "replicate").count() as u32);
        o.deaths.push(events.iter().filter(|e| e.rule_name == "die_and_recycle").count() as u32);
        o.carriers.push(snap.nodes().filter(|(_, d)| d.label == CARRIER).count());
    }
}

fn cfg(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "cc_sweep"
run_id = "cc_sweep"
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
}

fn run_cell(influx: f64, c: f64, steps: u64) -> Cell {
    let config = cfg(steps);
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules_with(influx, c), StepMode::MaxParallel, config.seed, 100000);
    let obs = Arc::new(Mutex::new(Obs::default()));
    engine.add_observer(Box::new(Handle { inner: obs.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let o = Arc::try_unwrap(obs).unwrap().into_inner().unwrap();
    let n = o.carriers.len();
    let w = (steps as usize / 5).max(1);
    let births: u64 = o.births.iter().map(|&x| x as u64).sum();
    let deaths: u64 = o.deaths.iter().map(|&x| x as u64).sum();
    let b_last: u64 = o.births[n - w..].iter().map(|&x| x as u64).sum();
    let d_last: u64 = o.deaths[n - w..].iter().map(|&x| x as u64).sum();
    let final_c = *o.carriers.last().unwrap();
    let peak = *o.carriers.iter().max().unwrap();
    let regime = if final_c == 0 {
        "EXTINCT"
    } else if b_last > 0 && d_last > 0 {
        "TURNOVER"
    } else if b_last == 0 && d_last == 0 {
        "FROZEN"
    } else {
        "TRANSIENT"
    };
    Cell { regime, final_c, peak, births, deaths, b_last, d_last }
}

#[test]
fn s6_child_cost_influx_resweep() {
    let steps = 12_000u64;
    // Influx spans below/around the sustain floor r*≈0.333 up to the old startup
    // threshold; child_node_cost spans from near the stillbirth floor up to 10.
    let influxes = [0.35, 0.45, 0.55, 0.70, 1.0];
    let costs = [2.0, 5.0, 10.0];

    eprintln!("\n================ S6 child_node_cost × influx re-sweep ================");
    eprintln!("sustain r*≈0.333; old startup≈0.49 (at cost 10). {steps} steps, 36-site static crystal.");
    eprintln!("stillbirth floor c≳0.7 (birth energy 0.8c > maintenance 0.5).\n");
    eprintln!("  regime grid (rows = child_node_cost, cols = influx rate):\n");
    eprint!("     cost\\r |");
    for r in influxes {
        eprint!("  {r:>5.2}  |");
    }
    eprintln!();
    eprintln!("    -------+{}", "---------+".repeat(influxes.len()));

    let mut any_turnover = None;
    let mut grid = Vec::new();
    for &c in &costs {
        eprint!("     {c:>5.1} |", c = c);
        let mut row = Vec::new();
        for &r in &influxes {
            let cell = run_cell(r, c, steps);
            eprint!(" {:<8}|", cell.regime);
            if cell.regime == "TURNOVER" && any_turnover.is_none() {
                any_turnover = Some((c, r));
            }
            row.push(cell);
        }
        eprintln!();
        grid.push((c, row));
    }

    // Detail on any turnover / transient cells (the interesting ones).
    eprintln!("\n  detail (non-frozen, non-extinct cells):");
    for (c, row) in &grid {
        for (i, cell) in row.iter().enumerate() {
            if cell.regime == "TURNOVER" || cell.regime == "TRANSIENT" {
                eprintln!(
                    "    cost={c:.1} r={:.2}: {} final={} peak={} births={} deaths={} b_last={} d_last={}",
                    influxes[i], cell.regime, cell.final_c, cell.peak, cell.births, cell.deaths, cell.b_last, cell.d_last
                );
            }
        }
    }

    match any_turnover {
        Some((c, r)) => {
            eprintln!("\n  ⟹ TURNOVER BAND FOUND at child_node_cost={c:.1}, influx={r:.2}.");
            eprintln!("     Lowering the diagnosis-named parameter reopened turnover on a STATIC");
            eprintln!("     homogeneous substrate (docs §8 outcome 1 — the cleanest result).");
        }
        None => {
            eprintln!("\n  ⟹ NO turnover cell at any tested (cost, influx). Frozen↔extinct persists;");
            eprintln!("     homogeneity is the binding obstacle (docs §8 outcome 3 → route A next).");
        }
    }
    eprintln!("======================================================================\n");

    // The grid must span regimes (some extinct, some non-extinct) so it is
    // interpretable; the turnover result itself is REPORTED, not asserted.
    let regimes: Vec<&str> = grid.iter().flat_map(|(_, row)| row.iter().map(|c| c.regime)).collect();
    assert!(regimes.iter().any(|&x| x == "EXTINCT"), "some cell must starve");
    assert!(regimes.iter().any(|&x| x != "EXTINCT"), "some cell must sustain a population");
}
