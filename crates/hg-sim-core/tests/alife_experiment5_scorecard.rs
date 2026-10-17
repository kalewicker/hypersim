//! Experiment-5 SCORECARD on the turnover regime discovered by the
//! replication_threshold sweep (bootstrap thr=3, sub-sustain influx). Runs that
//! regime with the S5 lineage/ecology instrumentation and reports all seven exit
//! criteria (scope §8 / plan S6.2), including the criterion-5 degenerate-selection
//! caveat (§1) and criterion 6 via `non_trivial_replicator_count`.
//!
//! This is a REPORT (the scorecard is printed and lightly asserted where a
//! criterion is unambiguous); it does not tune anything.
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, EffectOpDecl, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);

fn rules_with(influx: f64, thr: f64) -> Vec<CompiledRule> {
    let mut raw = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    for r in raw.iter_mut() {
        if r.name == "influx" {
            for eff in &mut r.rhs_property_effects {
                if eff.key == 1 {
                    if let EffectOpDecl::Add { value } = &mut eff.op {
                        *value = PropertyValue::Float(influx);
                    }
                }
            }
        }
        if r.name == "bootstrap_seed" {
            for eff in &mut r.rhs_property_effects {
                if eff.var == "carrier" && eff.key == 11 {
                    if let EffectOpDecl::Set { value } = &mut eff.op {
                        *value = PropertyValue::Float(thr);
                    }
                }
            }
        }
    }
    compile_rules(&raw).unwrap()
}

#[derive(Default)]
struct Counts {
    births: u64,
    deaths: u64,
    b_last: u64,
    d_last: u64,
    carriers: Vec<usize>,
}
struct Handle {
    counts: Arc<Mutex<Counts>>,
    alife: Arc<Mutex<AlifeObserver>>,
    last_window_start: u64,
    step: u64,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, events: &[RewriteEvent]) {
        self.step += 1;
        let b = events.iter().filter(|e| e.rule_name == "replicate").count() as u64;
        let d = events.iter().filter(|e| e.rule_name == "die_and_recycle").count() as u64;
        {
            let mut c = self.counts.lock().unwrap();
            c.births += b;
            c.deaths += d;
            if self.step >= self.last_window_start {
                c.b_last += b;
                c.d_last += d;
            }
            c.carriers.push(snap.nodes().filter(|(_, dd)| dd.label == CARRIER).count());
        }
        self.alife.lock().unwrap().on_step_end(snap, s, events);
    }
}

fn cfg(steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp5"
run_id = "exp5"
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

#[test]
fn experiment5_scorecard_on_turnover_regime() {
    let steps = 15_000u64;
    let (influx, thr) = (0.34, 3.0); // a fully-populated turnover cell
    let config = cfg(steps);
    let mut store = hg_sim_core::runner::build_initial_store(&config);
    let mut engine = Engine::new(rules_with(influx, thr), StepMode::MaxParallel, config.seed, 100000);
    let counts = Arc::new(Mutex::new(Counts::default()));
    let alife = Arc::new(Mutex::new(AlifeObserver::new(1_000)));
    engine.add_observer(Box::new(Handle {
        counts: counts.clone(),
        alife: alife.clone(),
        last_window_start: steps - steps / 5,
        step: 0,
    }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);

    let c = counts.lock().unwrap();
    let a = alife.lock().unwrap();
    let lin = a.lineage();
    let win = a.windows();

    // Late-window (last 20%) carrier ratio mean + variance (criterion 3).
    let tail = &c.carriers[c.carriers.len() - c.carriers.len() / 5..];
    let ratios: Vec<f64> = tail.iter().map(|&n| n as f64 / 36.0).collect();
    let rmean = ratios.iter().sum::<f64>() / ratios.len() as f64;
    let rvar = ratios.iter().map(|r| (r - rmean).powi(2)).sum::<f64>() / ratios.len() as f64;

    let thr_first = win.iter().find(|w| w.carriers > 0).map(|w| w.trait_mean[1]).unwrap();
    let thr_last = win.iter().rev().find(|w| w.carriers > 0).map(|w| w.trait_mean[1]).unwrap();
    let ntr = lin.non_trivial_replicator_count(3);

    eprintln!("\n============ EXPERIMENT 5 SCORECARD ============");
    eprintln!("regime: static 36-site crystal, influx={influx}, bootstrap repl_threshold={thr}, {steps} steps\n");
    eprintln!("  1 spontaneous replication : {} births from 1 bootstrap founder  ✓", c.births);
    eprintln!(
        "  2 bounded (no explode/extinct): final carriers {}, peak {}  {}",
        c.carriers.last().unwrap(),
        c.carriers.iter().max().unwrap(),
        if *c.carriers.last().unwrap() > 0 { "✓" } else { "✗ EXTINCT" }
    );
    eprintln!(
        "  3 stable ratio, low late variance: ratio {:.3} ± {:.5} (var)  {}",
        rmean, rvar, if rvar < 0.01 { "✓" } else { "(report value)" }
    );
    eprintln!(
        "  4 multi-generation lineage : max depth {}, total births {} (deep persistent clade)  ✓",
        lin.max_generation(),
        lin.total_births()
    );
    eprintln!(
        "  5 selection acting (shift) : mean repl_threshold {:.2} → {:.2}  ({:+.2})  ✓ via §1 DEGENERATE route",
        thr_first, thr_last, thr_last - thr_first
    );
    eprintln!("      (node-funded ⟹ nothing opposes a lower threshold; the OTHER traits are then");
    eprintln!("       under weak/no selection — criterion 5 is met trivially, as §1 pre-committed.)");
    eprintln!("  6 non_trivial_replicator_count 0→>0 : {ntr}  ✓ (S5 machinery; 0 on bare substrate)");
    eprintln!(
        "  turnover: births {} ≈ deaths {} (last window {} ≈ {})",
        c.births, c.deaths, c.b_last, c.d_last
    );
    eprintln!("================================================\n");

    // Unambiguous assertions (the degenerate-selection interpretation is REPORTED).
    assert!(*c.carriers.last().unwrap() > 0, "criterion 2: not extinct in-window");
    assert!(c.b_last > 0 && c.d_last > 0, "turnover: births and deaths both ongoing at the end");
    assert!(
        (c.b_last as f64 - c.d_last as f64).abs() / (c.b_last as f64) < 0.2,
        "turnover is balanced (births ≈ deaths in the last window)"
    );
    assert!(thr_last < thr_first - 1.0, "criterion 5: replication_threshold shifted DOWN (directional)");
    assert!(ntr > 0, "criterion 6: non-trivial replicators present");
    assert!(lin.max_generation() >= 10, "criterion 4: deep multi-generation lineage");
}
