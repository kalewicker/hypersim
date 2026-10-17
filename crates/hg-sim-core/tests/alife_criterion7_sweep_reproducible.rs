//! Criterion 7 (hard gate) — the influx sweep is a CROSS-RUN comparison by
//! construction, so its band boundaries must be reproducible across PROCESSES, not
//! just within one run.
//!
//! What the sweep's determinism actually rests on: the regime classification is
//! computed from ENGINE rewrite events (birth/death/carrier counts), NOT from WL
//! hashing — the causal observer (the WL consumer) is disabled in these runs. So the
//! spine here is deterministic rewriting: same seed ⟹ identical rewrite sequence ⟹
//! identical counts ⟹ identical regime, in ANY process. (WL cross-process
//! determinism, proven in hg-core's wl_cross_process, is the spine for Experiment
//! 6's CE, which coarse-grains over WL classes — a different claim.)
//!
//! This test pins a SIGNATURE of a small sweep (regime label + birth/death/final
//! counts per cell, hashed) recorded in a prior process. A fresh `cargo test`
//! process recomputes and compares: if engine determinism were ever process-
//! dependent (e.g. a `RandomState`-seeded iteration order leaking into match
//! ordering), the band boundary would move and this constant would break — exactly
//! where a soft determinism corrupts the scientific claim rather than one datum.
#![cfg(feature = "alife")]

use hg_core::GraphSnapshot;
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);

// Recorded 2026-07-15 (this machine, first run). The point of a pinned constant is
// that it was captured in a DIFFERENT process than the one now checking it.
const SWEEP_SIGNATURE: u64 = 0x9845f145815ae1ad;

fn config(seed: u64, influx: f64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "sweepsig"
run_id = "sweepsig"
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
influx_rate = {influx}
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

#[derive(Default, Debug)]
struct Counts {
    births: u64,
    deaths: u64,
    b_last: u64,
    d_last: u64,
    final_c: usize,
}
struct Obs {
    c: Arc<Mutex<Counts>>,
    step: u64,
    tail_start: u64,
}
impl Observer for Obs {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, e: &[RewriteEvent]) {
        self.step += 1;
        let b = e.iter().filter(|x| x.rule_name == "replicate").count() as u64;
        let d = e.iter().filter(|x| x.rule_name == "die_and_recycle").count() as u64;
        let mut c = self.c.lock().unwrap();
        c.births += b;
        c.deaths += d;
        if self.step >= self.tail_start {
            c.b_last += b;
            c.d_last += d;
        }
        c.final_c = snap.nodes().filter(|(_, dd)| dd.label == CARRIER).count();
    }
}

fn cell(seed: u64, influx: f64, steps: u64) -> (&'static str, u64, u64, usize) {
    let cfg = config(seed, influx, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);
    let c = Arc::new(Mutex::new(Counts::default()));
    engine.add_observer(Box::new(Obs { c: c.clone(), step: 0, tail_start: steps - steps / 5 }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    let c = Arc::try_unwrap(c).unwrap().into_inner().unwrap();
    let regime = if c.final_c == 0 {
        "EXTINCT"
    } else if c.b_last > 0 && c.d_last > 0 {
        "TURNOVER"
    } else if c.b_last == 0 && c.d_last == 0 {
        "FROZEN"
    } else {
        "TRANSIENT"
    };
    (regime, c.births, c.deaths, c.final_c)
}

fn sweep_signature() -> (u64, Vec<(f64, &'static str, u64, u64, usize)>) {
    let steps = 4_000u64;
    let influxes = [0.28, 0.34, 0.40, 0.50];
    let mut h = rustc_hash::FxHasher::default();
    let mut rows = Vec::new();
    for &r in &influxes {
        let (regime, births, deaths, final_c) = cell(12345, r, steps);
        regime.hash(&mut h);
        births.hash(&mut h);
        deaths.hash(&mut h);
        final_c.hash(&mut h);
        r.to_bits().hash(&mut h);
        rows.push((r, regime, births, deaths, final_c));
    }
    (h.finish(), rows)
}

#[test]
fn sweep_boundary_signature_is_cross_process_stable() {
    let (sig, rows) = sweep_signature();
    eprintln!("\n===== CRITERION 7 — cross-process sweep boundary signature =====");
    for (r, regime, b, d, fc) in &rows {
        eprintln!("  influx {r:.2}: {regime:<9} births={b} deaths={d} final={fc}");
    }
    eprintln!("  sweep signature: {sig:#018x}");
    eprintln!("  pinned          : {SWEEP_SIGNATURE:#018x}");
    eprintln!("================================================================\n");

    if SWEEP_SIGNATURE == 0x0 {
        panic!("RECORD the printed signature into SWEEP_SIGNATURE, then this asserts cross-process stability");
    }
    assert_eq!(
        sig, SWEEP_SIGNATURE,
        "sweep band boundaries changed vs the pinned (prior-process) signature — determinism is soft"
    );
}
