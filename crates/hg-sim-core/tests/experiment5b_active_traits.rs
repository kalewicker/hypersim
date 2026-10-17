//! Experiment 5b — activating harvest_rate & maintenance_cost (deviation #4).
//!
//! Pre-registration: `docs/experiment5b_active_traits_preregistration.md`. With
//! `genome_scaled_metabolism = true`, metabolise drains `maintenance_cost` and harvest
//! transfers `harvest_rate`, coupling the two previously-inert (drift-null) traits to
//! fitness. Tests: (smoke) the economy is sane — sites never go negative, run is
//! deterministic, bootstrap economy matches the fixed quantum; (science) do the newly
//! active traits shift directionally, and does turnover survive `maintenance_cost` collapse?
#![cfg(feature = "alife")]

use hg_alife::AlifeObserver;
use hg_core::{GraphSnapshot, PropertyValue};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const ENERGY: u32 = 1;
// GENOME_TRAIT_KEYS order: harvest_rate, replication_threshold, maintenance_cost, satiation.
const HARVEST: usize = 0;
const MAINT: usize = 2;

fn config(seed: u64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "exp5b"
run_id = "exp5b"
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
bootstrap_replication_threshold = 3.0
genome_scaled_metabolism = true
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

#[derive(Default)]
struct Rec {
    min_site_energy: f64,
    births: Vec<u64>,
    deaths: Vec<u64>,
    carriers: Vec<usize>,
}
struct Handle {
    rec: Arc<Mutex<Rec>>,
    alife: Arc<Mutex<AlifeObserver>>,
}
impl Observer for Handle {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, s: &StepStats, e: &[RewriteEvent]) {
        {
            let mut r = self.rec.lock().unwrap();
            // Substrate sites are label 0; track the lowest energy any site reaches.
            for (_, d) in snap.nodes().filter(|(_, d)| d.label == hg_core::Label(0)) {
                if let Some(PropertyValue::Float(v)) = d.get_property(ENERGY) {
                    r.min_site_energy = r.min_site_energy.min(v);
                }
            }
            r.births.push(e.iter().filter(|x| x.rule_name == "replicate").count() as u64);
            r.deaths.push(e.iter().filter(|x| x.rule_name == "die_and_recycle").count() as u64);
            r.carriers.push(snap.nodes().filter(|(_, d)| d.label == hg_alife::CARRIER_LABEL).count());
        }
        self.alife.lock().unwrap().on_step_end(snap, s, e);
    }
}

struct Run {
    trait_first: [f64; 4],
    trait_last: [f64; 4],
    births: u64,
    deaths: u64,
    deaths_first_half: u64,
    deaths_last_half: u64,
    final_carriers: usize,
    min_site_energy: f64,
}

fn run(seed: u64, steps: u64) -> Option<Run> {
    let cfg = config(seed, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);
    let rec = Arc::new(Mutex::new(Rec { min_site_energy: f64::INFINITY, ..Default::default() }));
    let alife = Arc::new(Mutex::new(AlifeObserver::new(500)));
    engine.add_observer(Box::new(Handle { rec: rec.clone(), alife: alife.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let rec = Arc::try_unwrap(rec).map_err(|_| ()).unwrap().into_inner().unwrap();
    let a = alife.lock().unwrap();
    let win = a.windows();
    let first = win.iter().find(|w| w.carriers > 0)?;
    let last = win.iter().rev().find(|w| w.carriers > 0)?;
    if std::ptr::eq(first, last) {
        return None;
    }
    let half = rec.deaths.len() / 2;
    Some(Run {
        trait_first: first.trait_mean,
        trait_last: last.trait_mean,
        births: rec.births.iter().sum(),
        deaths: rec.deaths.iter().sum(),
        deaths_first_half: rec.deaths[..half].iter().sum(),
        deaths_last_half: rec.deaths[half..].iter().sum(),
        final_carriers: *rec.carriers.last().unwrap(),
        min_site_energy: rec.min_site_energy,
    })
}

/// Smoke: the genome-scaled economy is mechanically sound — sites never go negative
/// (the scaled site-floor guard holds), the run is deterministic, and it establishes.
#[test]
fn genome_scaled_economy_is_sound_and_deterministic() {
    let a = run(12345, 3000).expect("establishes");
    let b = run(12345, 3000).expect("establishes");
    assert!(a.min_site_energy >= -1e-9, "sites never go negative (got {})", a.min_site_energy);
    assert_eq!(a.births, b.births, "deterministic births");
    assert_eq!(a.deaths, b.deaths, "deterministic deaths");
    assert!(a.births > 0, "reproduction occurs");
}

/// The science: measure the two now-active traits across seeds and whether turnover
/// survives. Reported against the pre-registration (P1 maintenance collapse + turnover
/// death; P2 harvest_rate shift). Drift reference: fixed-economy |Δ| envelope ≈ 4.5,
/// sign-MIXED (criterion-5 validation).
#[test]
fn active_metabolic_traits_selection_report() {
    let steps = 10_000u64;
    let seeds = [12345u64, 1, 7, 99999, 2024, 42];

    eprintln!("\n===== EXPERIMENT 5b — active harvest_rate & maintenance_cost =====");
    eprintln!("genome_scaled_metabolism, turnover regime, {steps} steps\n");
    eprintln!("  seed   | Δharvest  Δmaint  | births deaths | deaths 1st→2nd half | finalC minSite");
    eprintln!("  -------+------------------+---------------+---------------------+---------------");

    let mut est = 0;
    let (mut h_disp, mut m_disp) = (Vec::new(), Vec::new());
    let mut turnover_dying = 0;
    for &seed in &seeds {
        match run(seed, steps) {
            Some(r) => {
                est += 1;
                let dh = r.trait_last[HARVEST] - r.trait_first[HARVEST];
                let dm = r.trait_last[MAINT] - r.trait_first[MAINT];
                h_disp.push(dh);
                m_disp.push(dm);
                let dying = r.deaths_last_half < r.deaths_first_half / 2;
                if dying {
                    turnover_dying += 1;
                }
                eprintln!(
                    "  {seed:>6} | {dh:+8.3} {dm:+8.3} | {:>6} {:>6} | {:>6} → {:<6} {}| {:>5}  {:+.2}",
                    r.births, r.deaths, r.deaths_first_half, r.deaths_last_half,
                    if dying { "↓collapsing " } else { "" }, r.final_carriers, r.min_site_energy
                );
            }
            None => eprintln!("  {seed:>6} | (never established)"),
        }
    }

    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let frac_neg = |v: &[f64]| v.iter().filter(|x| **x < 0.0).count() as f64 / v.len().max(1) as f64;
    eprintln!("\n  established: {est}/{}", seeds.len());
    eprintln!("  harvest_rate: mean Δ {:+.3}, {:.0}% down (drift null was sign-mixed ~50%)", mean(&h_disp), 100.0 * frac_neg(&h_disp));
    eprintln!("  maintenance : mean Δ {:+.3}, {:.0}% down", mean(&m_disp), 100.0 * frac_neg(&m_disp));
    eprintln!("  turnover collapsing (2nd-half deaths < ½ 1st-half): {turnover_dying}/{est}");
    eprintln!("=================================================================\n");

    // Structural assertions only (the interpretation is reported, per the pre-registration):
    assert!(est >= 2, "some seeds must establish to characterize");
    assert!(h_disp.iter().chain(&m_disp).all(|d| d.is_finite()), "trait displacements finite");
}
