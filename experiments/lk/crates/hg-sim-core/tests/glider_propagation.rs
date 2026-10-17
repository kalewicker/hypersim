//! Glider regression (hypersim_diagnose_glider.md): the conservative
//! mark-transfer rule must walk a ring track indefinitely — exactly one
//! mark alive, a hop (nearly) every step, never Frozen, Mobile passing.
//! The pre-fix rule died after ONE firing even on a clean track (its RHS
//! deleted the tail and spawned a detached head instead of relabelling).

use hg_core::{HypergraphStore, Label};
use hg_engine::Engine;
use hg_rules::{builtin_rules, compile_rules, StepMode};
use hg_schema::{GateName, GateStatus};
use hg_sim_core::{run_headless, SimConfig};

fn marks(store: &HypergraphStore) -> Vec<hg_core::NodeId> {
    store
        .nodes()
        .filter(|(_, d)| d.label == Label(1))
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn glider_walks_the_ring_conserving_one_mark() {
    let mut store = HypergraphStore::new();
    hg_core::generate_glider_ring(&mut store, 60);
    assert_eq!(marks(&store).len(), 1, "ring seeds exactly one mark");
    let start_mark = marks(&store)[0];
    let (n0, e0) = (store.node_count(), store.edge_count());

    let rules = compile_rules(&builtin_rules("glider").unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::SequentialRandom, 42, 1);

    let mut hops = 0usize;
    for step in 0..400u64 {
        let stats = engine.step(&mut store);
        hops += stats.accepted_rewrites;
        let m = marks(&store);
        assert_eq!(m.len(), 1, "mark count broke conservation at step {step}");
    }
    assert!(hops >= 380, "glider stalled: only {hops} hops in 400 steps");
    assert_ne!(marks(&store)[0], start_mark, "mark never moved");
    // Pure relabel: the substrate itself is untouched.
    assert_eq!(store.node_count(), n0, "node count changed");
    assert_eq!(store.edge_count(), e0, "edge count changed");
}

#[test]
fn glider_run_is_mobile_not_frozen() {
    let dir = tempfile::tempdir().unwrap();
    let config = SimConfig::from_toml(&format!(
        r#"
experiment_name = "glider_e2e"
seed = 42
out_dir = "{}"
run_id = "glider_e2e"

[rules]
source = "Builtin"
builtin_name = "glider"

[initial]
condition_type = "GliderRing"
node_count = 60

[engine]
step_mode = "SequentialRandom"
max_steps = 2048
metrics_window_interval = 256
max_pairings_per_step = 1
thread_count = 1
"#,
        dir.path().display()
    ))
    .unwrap();

    let artifacts = run_headless(&config, None, None, None).unwrap();
    let final_row = artifacts.summary.final_metrics.as_ref().unwrap();
    assert_eq!(
        final_row.regime.as_deref(),
        Some("ActivePersistentStructure"),
        "glider on its track must classify as a mover, got {:?}",
        final_row.regime
    );
    let mobile = artifacts
        .summary
        .gate_statuses
        .iter()
        .find(|(g, _)| *g == GateName::Mobile)
        .map(|(_, s)| *s);
    assert_eq!(mobile, Some(GateStatus::Pass), "Mobile gate must pass");
}
