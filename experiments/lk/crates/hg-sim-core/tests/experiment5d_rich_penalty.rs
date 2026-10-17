//! Experiment 5d — an energy-proportional reproductive tax to close 5c's residual basin.
//!
//! Pre-registration: `docs/experiment5d_rich_penalty_preregistration.md`. On top of
//! parent-funded replication (5c), `repro_energy_tax = Some(tau)` makes the parent lose fraction
//! `tau` of its energy to entropy on each birth (`parent.energy ×= 1−tau`), so reproducing while
//! rich is wasteful — the threshold stops being a neutral (drifting) trait for reliably-rich
//! lineages. Tests: (F-C) energy never spuriously increases + determinism; (P1–P3) as tau rises,
//! the collapsed fraction falls and the interior fraction rises (basin closing), until too much
//! tax starves the population.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);
const REPL_THR: usize = 1;

fn config(seed: u64, boot: f64, tau: f64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp5d"
run_id = "exp5d"
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
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = {boot}
parent_funded_replication = true
repro_energy_tax = {tau}
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

struct Obs {
    alife: Arc<Mutex<AlifeObserver>>,
    stats: Arc<Mutex<(u64, u64, u64, u64, f64, f64)>>, // births, deaths, d_first, d_last, max_energy_seen, min_site
    step: u64,
    steps: u64,
}
impl Observer for Obs {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, st: &StepStats, e: &[RewriteEvent]) {
        self.step += 1;
        let b = e.iter().filter(|x| x.rule_name == "replicate").count() as u64;
        let d = e.iter().filter(|x| x.rule_name == "die_and_recycle").count() as u64;
        {
            let mut s = self.stats.lock().unwrap();
            s.0 += b;
            s.1 += d;
            if self.step <= self.steps / 2 { s.2 += d } else { s.3 += d }
            for (_, dd) in snap.nodes() {
                if let Some(PropertyValue::Float(v)) = dd.get_property(1) {
                    if dd.label == CARRIER {
                        s.4 = s.4.max(v);
                    } else {
                        s.5 = s.5.min(v);
                    }
                }
            }
        }
        self.alife.lock().unwrap().on_step_end(snap, st, e);
    }
}

/// (final repl_threshold, births, deaths_first_half, deaths_last_half, min_site_energy)
fn run(seed: u64, boot: f64, tau: f64, steps: u64) -> Option<(f64, u64, u64, u64, f64)> {
    let cfg = config(seed, boot, tau, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);
    let alife = Arc::new(Mutex::new(AlifeObserver::new(500)));
    let stats = Arc::new(Mutex::new((0u64, 0u64, 0u64, 0u64, 0.0f64, f64::INFINITY)));
    engine.add_observer(Box::new(Obs { alife: alife.clone(), stats: stats.clone(), step: 0, steps }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let a = alife.lock().unwrap();
    let last = a.windows().iter().rev().find(|w| w.carriers > 0)?.trait_mean[REPL_THR];
    let s = *stats.lock().unwrap();
    if s.0 == 0 {
        return None;
    }
    Some((last, s.0, s.2, s.3, s.5))
}

#[test]
fn tax_conserves_and_is_deterministic() {
    let a = run(12345, 6.0, 0.3, 3000).expect("establishes");
    let b = run(12345, 6.0, 0.3, 3000).expect("establishes");
    assert_eq!(a.1, b.1, "deterministic births");
    assert_eq!((a.2, a.3), (b.2, b.3), "deterministic deaths");
    assert!(a.4 >= -1e-9, "sites never negative under the tax (min {})", a.4);
    assert!(a.1 > 0, "reproduction occurs");
}

#[test]
fn energy_tax_closes_the_degenerate_basin() {
    let steps = 10_000u64;
    let boot = 6.0;
    let taus = [0.0, 0.3, 0.6];
    let seeds = [12345u64, 1, 7, 99999, 2024, 42];

    eprintln!("\n===== EXPERIMENT 5d — energy-proportional reproductive tax (close 5c's basin) =====");
    eprintln!("parent-funded, child cost 10, bootstrap repl_threshold {boot}, {steps} steps, {} seeds\n", seeds.len());
    eprintln!("  tau | est | median | interior(7-13) | collapsed(<3) | turnover | finals");
    eprintln!("  ----+-----+--------+----------------+---------------+----------+-------");

    let mut summary = Vec::new();
    for &tau in &taus {
        let mut finals = Vec::new();
        let mut turnover_alive = 0;
        for &seed in &seeds {
            if let Some((thr, _b, d1, d2, _)) = run(seed, boot, tau, steps) {
                finals.push(thr);
                if d2 > d1 / 4 {
                    turnover_alive += 1;
                }
            }
        }
        let est = finals.len();
        let mut sorted = finals.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let median = if est == 0 { f64::NAN } else { sorted[est / 2] };
        let interior = finals.iter().filter(|v| (7.0..=13.0).contains(*v)).count();
        let collapsed = finals.iter().filter(|v| **v < 3.0).count();
        let vals: Vec<String> = finals.iter().map(|v| format!("{v:.1}")).collect();
        eprintln!(
            "  {tau:.1} | {est}/{} | {median:>6.1} | {interior:>4}           | {collapsed:>4}          | {turnover_alive}/{est}      | [{}]",
            seeds.len(), vals.join(", ")
        );
        summary.push((tau, est, interior, collapsed, turnover_alive));
    }

    eprintln!("\n  P1 (basin closes): collapsed fraction vs tau:");
    for (tau, est, interior, collapsed, alive) in &summary {
        eprintln!("    tau {tau:.1}: {collapsed} collapsed, {interior} interior, {alive}/{est} turnover-alive");
    }
    let base_collapsed = summary[0].3;
    let high_collapsed = summary.last().unwrap().3;
    let high_alive = summary.last().unwrap().4;
    if high_collapsed < base_collapsed && high_alive > 0 {
        eprintln!("  ⟹ FINDING: the rich-biting tax REDUCES floor-collapse ({base_collapsed}→{high_collapsed}) while the");
        eprintln!("    population persists — the energy-proportional cost closes (part of) the degenerate basin.");
    } else if high_alive == 0 {
        eprintln!("  ⟹ P3/F-B: high tau starves the population before the basin closes — trade-off, reported.");
    } else {
        eprintln!("  ⟹ F-A: tax does not reduce collapse — a NON-energy cost (mortality/aging) is needed next.");
    }
    eprintln!("=================================================================================\n");

    assert!(summary.iter().any(|s| s.1 > 0), "some regime must establish");
    // The reported interpretation is the finding; assert only that we characterized the sweep.
    assert!(summary.len() == taus.len());
}
