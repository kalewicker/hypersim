//! V-track V4.2 — paired bare-substrate control for THE experiment run.
//!
//! The run that produces the reported CE (Experiment 6's config: seed 12345,
//! influx 0.34, bootstrap threshold 3.0) gets a same-seed bare-substrate control.
//! The substrate-only `d_eff`, node/edge counts, mean degree, and the substrate
//! SUBGRAPH regime classification must match the control.
//!
//! Why this is load-bearing (scope Q3): carriers are hypergraph NODES, so on the
//! full graph a carrier bloom inflates `n_nodes`/`d_eff` and would read as the
//! substrate `Exploding`. Recomputing over the substrate subgraph is what licenses
//! attributing anything at all to biology rather than to substrate physics. The
//! sweep-wide version of this check is `alife_criterion7_paired_control_sweep`;
//! this pins it to the exact configuration the CE headline was computed on.
#![cfg(feature = "alife")]

use hg_alife::CARRIER_LABEL;
use hg_engine::Engine;
use hg_eval::{classify, substrate_regime, substrate_structural, WindowComputer};
use hg_rules::StepMode;
use hg_sim_core::SimConfig;
use hg_schema::Regime;

const STEPS: u64 = 6_000;
const WINDOW: u64 = 500;
const DEFF_SEED: u64 = 12345;

fn config(alife: bool) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "v4_2"
run_id = "v4_2"
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
max_steps = {STEPS}
metrics_window_interval = 128
max_pairings_per_step = 100000
[evaluator]
enabled = false
[causal]
enabled = false
[alife]
enabled = {alife}
seed_carrier_marker = {alife}
initial_substrate_energy = 5.0
influx_rate = 0.34
bootstrap_replication_threshold = 3.0
"#
    );
    SimConfig::from_toml(&toml).unwrap()
}

struct Sample {
    step: u64,
    full_nodes: usize,
    sub_nodes: usize,
    sub_edges: usize,
    mean_degree: Option<f64>,
    d_eff: Option<f64>,
    regime: Regime,
}

fn run_arm(alife: bool) -> Vec<Sample> {
    let cfg = config(alife);
    let rules = hg_sim_core::runner::load_rules(&cfg).unwrap();
    let mut store = hg_sim_core::runner::build_initial_store(&cfg);
    let mut engine = Engine::new(rules, StepMode::MaxParallel, cfg.seed, 100_000);
    // The substrate computer is threaded across windows so growth_ratio/novelty are
    // measured on the substrate's OWN history, never contaminated by carrier births.
    let mut computer = WindowComputer::new(WINDOW);
    let mut out = Vec::new();

    for step in 1..=STEPS {
        engine.step(&mut store);
        if step % WINDOW != 0 {
            continue;
        }
        let snap = store.snapshot();
        let st = substrate_structural(&snap, CARRIER_LABEL, DEFF_SEED);
        let (features, regime) = substrate_regime(&mut computer, &snap, CARRIER_LABEL);
        debug_assert_eq!(regime, classify(&features));
        out.push(Sample {
            step,
            full_nodes: snap.node_count(),
            sub_nodes: st.n_nodes,
            sub_edges: st.n_edges,
            mean_degree: st.mean_degree,
            d_eff: st.d_eff,
            regime,
        });
    }
    out
}

#[test]
fn v4_2_experiment_run_matches_bare_control() {
    let live = run_arm(true);
    let bare = run_arm(false);
    assert_eq!(live.len(), bare.len(), "arms must sample the same windows");

    eprintln!("\n========== V4.2 — paired bare-substrate control for the CE experiment run ==========");
    eprintln!("seed 12345, influx 0.34, bootstrap thr 3.0, {STEPS} steps, window {WINDOW}\n");
    eprintln!("  step | full n_nodes (live/bare) | substrate (n,e) live | substrate (n,e) bare | d_eff live/bare | regime live/bare");
    eprintln!("  -----+--------------------------+----------------------+----------------------+-----------------+-----------------");

    let mut carrier_peak = 0usize;
    for (l, b) in live.iter().zip(bare.iter()) {
        let carriers = l.full_nodes.saturating_sub(l.sub_nodes);
        carrier_peak = carrier_peak.max(carriers);
        let f = |d: Option<f64>| d.map(|v| format!("{v:.3}")).unwrap_or_else(|| "None".into());
        eprintln!(
            "  {:>4} | {:>9} / {:<12} | {:>8?} | {:>8?} | {} / {} | {:?} / {:?}",
            l.step,
            l.full_nodes,
            b.full_nodes,
            (l.sub_nodes, l.sub_edges),
            (b.sub_nodes, b.sub_edges),
            f(l.d_eff),
            f(b.d_eff),
            l.regime,
            b.regime
        );

        // The load-bearing assertions: the substrate subgraph of the live run is
        // IDENTICAL to the same-seed bare control, in every structural metric.
        assert_eq!(l.sub_nodes, b.sub_nodes, "V4.2 at step {}: substrate node count diverged from control", l.step);
        assert_eq!(l.sub_edges, b.sub_edges, "V4.2 at step {}: substrate edge count diverged from control", l.step);
        assert_eq!(l.mean_degree, b.mean_degree, "V4.2 at step {}: substrate mean degree diverged from control", l.step);
        assert_eq!(l.d_eff, b.d_eff, "V4.2 at step {}: substrate d_eff diverged from control", l.step);
        assert_eq!(l.regime, b.regime, "V4.2 at step {}: substrate regime diverged from control", l.step);
        // Q3: a carrier bloom must NOT read as substrate Exploding.
        assert_ne!(l.regime, Regime::Exploding, "V4.2 at step {}: carrier bloom read as substrate Exploding — the partition is contaminated", l.step);
    }

    eprintln!("\n  peak carriers on the full graph: {carrier_peak} (the control is doing real work:");
    eprintln!("  the live full graph carries up to {carrier_peak} extra nodes, yet its SUBSTRATE subgraph");
    eprintln!("  is identical to the bare control in nodes, edges, mean degree, d_eff, and regime)");
    assert!(carrier_peak > 0, "V4.2 vacuous: the live arm never grew any carriers");
    eprintln!("  ⟹ substrate metrics stay attributable; anything measured beyond them is biology.");
    eprintln!("====================================================================================\n");
}
