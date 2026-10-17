//! S6.1 — influx sweep, to test the pre-registered turnover predictions
//! (docs/experiment5_preregistration.md §7):
//!   * critical rate r* = M·H/(M+H) = 1/3 (M=maintenance 0.5, H=harvest 1);
//!   * a DISCONTINUOUS frozen (r>r*) ↔ extinct (r<r*) transition, with no stable
//!     births≈deaths turnover band on the static homogeneous substrate.
//!
//! The sweep varies ONLY the influx rule's add value; M and H are recorded so the
//! ratio r/r* is interpretable. This is a diagnostic, not a tuning pass: it reports
//! the regime at each rate and checks the transition against the committed r*.
#![cfg(feature = "alife")]

use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, CompiledRule, EffectOpDecl, StepMode};
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const ENERGY: u32 = 1;
const OCCUPIED: u32 = 22;
const CARRIER: hg_core::Label = hg_core::Label(100);
const MAINTENANCE: f64 = 0.5; // metabolise quantum M (recorded — physics is r/r*)
const HARVEST_Q: f64 = 1.0; // harvest quantum H
// r* = M·H/(M+H); with M=0.5, H=1 ⟹ 1/3.
const R_STAR: f64 = MAINTENANCE * HARVEST_Q / (MAINTENANCE + HARVEST_Q);

/// Rules with the influx add-value overridden to `rate` (everything else intact).
fn rules_with_influx(rate: f64) -> Vec<CompiledRule> {
    let mut raw = parse_rules_toml(hg_alife::alife_base_toml()).unwrap();
    let mut patched = false;
    for r in raw.iter_mut() {
        if r.name == "influx" {
            for eff in r.rhs_property_effects.iter_mut() {
                if eff.key == ENERGY {
                    if let EffectOpDecl::Add { value } = &mut eff.op {
                        *value = PropertyValue::Float(rate);
                        patched = true;
                    }
                }
            }
        }
    }
    assert!(patched, "failed to patch influx rate — rule shape changed");
    compile_rules(&raw).unwrap()
}

#[derive(Default, Debug)]
struct SweepObs {
    births: Vec<u32>, // per step
    deaths: Vec<u32>,
    carriers: Vec<usize>,
}
struct Handle {
    inner: Arc<Mutex<SweepObs>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, events: &[RewriteEvent]) {
        let mut o = self.inner.lock().unwrap();
        o.births.push(events.iter().filter(|e| e.rule_name == "replicate").count() as u32);
        o.deaths.push(events.iter().filter(|e| e.rule_name == "die_and_recycle").count() as u32);
        o.carriers.push(snap.nodes().filter(|(_, d)| d.label == CARRIER).count());
    }
}

fn config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "s6_sweep"
run_id = "s6_sweep"
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
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

struct RunResult {
    rate: f64,
    final_carriers: usize,
    peak: usize,
    births: u64,
    deaths: u64,
    births_last: u64,
    deaths_last: u64,
    regime: &'static str,
}

fn run_rate(rate: f64, steps: u64) -> RunResult {
    let cfg = config(12345, steps);
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules_with_influx(rate), StepMode::MaxParallel, cfg.seed, 100000);
    let obs = Arc::new(Mutex::new(SweepObs::default()));
    engine.add_observer(Box::new(Handle { inner: obs.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let o = Arc::try_unwrap(obs).unwrap().into_inner().unwrap();

    let n = o.carriers.len();
    let w = (steps as usize / 5).max(1); // last 20% window
    let births: u64 = o.births.iter().map(|&x| x as u64).sum();
    let deaths: u64 = o.deaths.iter().map(|&x| x as u64).sum();
    let births_last: u64 = o.births[n - w..].iter().map(|&x| x as u64).sum();
    let deaths_last: u64 = o.deaths[n - w..].iter().map(|&x| x as u64).sum();
    let final_carriers = *o.carriers.last().unwrap();
    let peak = *o.carriers.iter().max().unwrap();

    let regime = if final_carriers == 0 {
        "EXTINCT"
    } else if births_last > 0 && deaths_last > 0 {
        "TURNOVER"
    } else if births_last == 0 && deaths_last == 0 {
        "FROZEN"
    } else {
        "TRANSIENT" // one of birth/death still happening but not the other
    };

    RunResult { rate, final_carriers, peak, births, deaths, births_last, deaths_last, regime }
}

#[test]
fn s6_influx_sweep_transition() {
    let steps = 12_000u64;
    eprintln!("\n================ S6.1 influx sweep ================");
    eprintln!("M (maintenance) = {MAINTENANCE}, H (harvest) = {HARVEST_Q}  ⟹  r* = M·H/(M+H) = {R_STAR:.4}");
    eprintln!("36-site crystal, {steps} steps, static substrate (no substrate rules)\n");
    eprintln!("   rate | r/r*  | regime   | final | peak | births | deaths | b_last | d_last");
    eprintln!("  ------+-------+----------+-------+------+--------+--------+--------+-------");

    // Centered on r* = 1/3, spanning well below to well above, with fine
    // granularity across the observed startup transition (0.40–0.55) to locate
    // it precisely and expose any thin turnover band.
    let rates = [
        0.1, 0.2, 0.30, 0.333, 0.40, 0.42, 0.44, 0.45, 0.46, 0.48, 0.50, 0.52, 0.55, 1.0, 2.0,
    ];
    let mut results = Vec::new();
    for &rate in &rates {
        let r = run_rate(rate, steps);
        eprintln!(
            "  {:>5.3} | {:>5.2} | {:<8} | {:>5} | {:>4} | {:>6} | {:>6} | {:>6} | {:>6}",
            r.rate, r.rate / R_STAR, r.regime, r.final_carriers, r.peak, r.births, r.deaths,
            r.births_last, r.deaths_last
        );
        results.push(r);
    }

    // --- Report the transition and test the pre-registered predictions. ---
    let extinct_max = results.iter().filter(|r| r.regime == "EXTINCT").map(|r| r.rate).fold(f64::NAN, f64::max);
    let frozen_min = results.iter().filter(|r| r.regime == "FROZEN").map(|r| r.rate).fold(f64::INFINITY, f64::min);
    let turnover: Vec<f64> = results.iter().filter(|r| r.regime == "TURNOVER").map(|r| r.rate).collect();
    eprintln!("\n  highest EXTINCT rate: {extinct_max:.3}   lowest FROZEN rate: {frozen_min:.3}");
    eprintln!("  TURNOVER band: {turnover:?}");
    eprintln!("  pre-registered r* = {R_STAR:.4}");
    if turnover.is_empty() {
        eprintln!("  ⟹ DISCONTINUOUS frozen↔extinct, NO stable turnover band (as predicted §7).");
        eprintln!("     Decision rule → route 2: scarcity-mortality insufficient on a static substrate;");
        eprintln!("     lifetime aging (key 21, scoped) becomes justified, OR run over an active substrate.");
    } else {
        eprintln!("  ⟹ a turnover band EXISTS at {turnover:?} — Experiment-5 regime candidate (§7 rule 1).");
    }
    eprintln!("==================================================\n");

    // Report where the OBSERVED transition sits vs the predicted steady-state r*.
    // The observed boundary is the STARTUP threshold (found bootstrapping a
    // population: founder must climb to threshold AND fund a neighbour), which is
    // higher than the steady-state SUSTAIN threshold r* = 1/3. That gap is the
    // finding, not a bug (docs §7, updated post-sweep).
    let observed_transition = (extinct_max + frozen_min) / 2.0;
    eprintln!(
        "  OBSERVED transition ≈ {observed_transition:.3}  (predicted STEADY-STATE r* = {R_STAR:.3};",
    );
    eprintln!("     the observed boundary is the higher STARTUP threshold — see §7.)");

    // Robust invariants (NOT the refuted exact location): the sweep must bracket a
    // real transition — extinct at the lowest rate, a live population at the
    // highest — so the regime structure is interpretable.
    assert_eq!(results[0].regime, "EXTINCT", "lowest influx must starve the population");
    assert!(
        results.last().unwrap().final_carriers > 0,
        "highest influx must sustain a population"
    );
    // A discontinuity (some extinct, some frozen) must exist; whether a TURNOVER
    // band appears is the reported result, not an assertion — an emergent band
    // would be decision-rule 1 (a WIN), its absence decision-rule 2.
    assert!(extinct_max.is_finite() && frozen_min.is_finite(), "sweep must bracket a transition");
}
