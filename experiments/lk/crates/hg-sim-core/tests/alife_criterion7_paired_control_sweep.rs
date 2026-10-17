//! Criterion 7 (hard gate) — a paired bare-substrate control at EVERY sweep point,
//! with the substrate-only structural metrics matching their control.
//!
//! Experiment 6 coarse-grains over the substrate; if biology perturbs the substrate
//! metrics beyond adding carrier nodes, the "substrate" S7 partitions over is not
//! clean. The single-config version of this check lives in alife_substrate_control.rs;
//! here we confirm it holds ACROSS the influx sweep (the sweep varies energy flow
//! through the substrate — harvest drains sites, influx fills them, replicate/die
//! move energy — so a naive worry is that heavier flow at some influx rate perturbs
//! substrate structure). The static substrate has NO rules, so its TOPOLOGY is inert
//! at every influx; this test measures that rather than asserting it.
#![cfg(feature = "alife")]

use hg_core::GraphSnapshot;
use hg_engine::Engine;
use hg_eval::substrate_structural;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use std::sync::{Arc, Mutex};

const CARRIER: hg_core::Label = hg_core::Label(100);

fn config(alife: bool, influx: f64, steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "pairctl"
run_id = "pairctl"
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
enabled = {alife}
seed_carrier_marker = {alife}
initial_substrate_energy = 5.0
influx_rate = {influx}
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

/// Sample substrate-only (n_nodes, n_edges) each window over the run; also record the
/// full-graph node count so pollution (carriers on the full graph) is visible.
struct Sampler {
    interval: u64,
    step: u64,
    sub: Arc<Mutex<Vec<(usize, usize)>>>,
    full_nodes: Arc<Mutex<Vec<usize>>>,
}
impl Observer for Sampler {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _s: &StepStats, _e: &[RewriteEvent]) {
        self.step += 1;
        if self.step % self.interval == 0 {
            let s = substrate_structural(snap, CARRIER, 0xABCD);
            self.sub.lock().unwrap().push((s.n_nodes, s.n_edges));
            self.full_nodes.lock().unwrap().push(snap.nodes().count());
        }
    }
}

fn run_substrate_trace(alife: bool, influx: f64, steps: u64) -> (Vec<(usize, usize)>, Vec<usize>) {
    let cfg = config(alife, influx, steps);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100000);
    let sub = Arc::new(Mutex::new(Vec::new()));
    let full = Arc::new(Mutex::new(Vec::new()));
    engine.add_observer(Box::new(Sampler { interval: 500, step: 0, sub: sub.clone(), full_nodes: full.clone() }));
    for _ in 0..steps {
        engine.step(&mut store);
    }
    drop(engine);
    (
        Arc::try_unwrap(sub).unwrap().into_inner().unwrap(),
        Arc::try_unwrap(full).unwrap().into_inner().unwrap(),
    )
}

#[test]
fn substrate_matches_bare_control_at_every_sweep_point() {
    let steps = 4_000u64;
    let influxes = [0.28, 0.34, 0.40, 0.50, 1.0];

    // Bare-substrate control: alife OFF ⟹ no biology at all; the substrate subgraph is
    // the whole graph. Its (n_nodes, n_edges) trace is influx-independent (influx only
    // fires under alife), so one control suffices as the reference topology.
    let (control_sub, _control_full) = run_substrate_trace(false, 0.0, steps);
    let control_ref = control_sub[0];

    eprintln!("\n===== CRITERION 7 — paired substrate control across the influx sweep =====");
    eprintln!("  bare control substrate: {} nodes, {} edges (static)\n", control_ref.0, control_ref.1);
    eprintln!("  influx | substrate (n_nodes,n_edges) | full n_nodes (with carriers) | matches control?");
    eprintln!("  -------+----------------------------+------------------------------+-----------------");
    for &r in &influxes {
        let (sub, full) = run_substrate_trace(true, r, steps);
        // Every window's substrate subgraph must equal the bare control's topology.
        let all_match = sub.iter().all(|&s| s == control_ref);
        let full_max = *full.iter().max().unwrap();
        eprintln!(
            "  {r:>5.2}  | {:?} (all windows)        | up to {full_max} (36 substrate + carriers)   | {}",
            sub.last().unwrap(),
            if all_match { "YES" } else { "NO — back-reaction!" }
        );
        assert!(
            all_match,
            "at influx {r}, the substrate subgraph deviates from the bare control — biology perturbed substrate topology"
        );
        // Sanity: carriers DO inflate the full graph (so the substrate filter is doing real work).
        assert!(full_max > control_ref.0, "carriers should inflate the full-graph node count");
    }
    eprintln!("\n  ⟹ substrate topology == bare control at ALL sweep points; carriers only ADD nodes on");
    eprintln!("     the full graph. The S7 substrate partition is clean (static substrate, no back-reaction).");
    eprintln!("========================================================================\n");
}
