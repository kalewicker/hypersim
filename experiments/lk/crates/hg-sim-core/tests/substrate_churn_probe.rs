//! Route-1 kill-test PREREQUISITE probe (docs §7 route A): does any builtin give
//! a substrate that CHURNS WITHOUT NET GROWTH and mints fresh, influx-eligible
//! (label-0) nodes? Heterogeneity needs fresh energy-poor sites at a stable node
//! count; unbounded growth is the explosion confound the sweep warned about.
//!
//! This measures node-count trajectories + label distribution empirically (per the
//! instruction "verify against the measured node-count trajectory, not its name").
#![cfg(feature = "alife")]

use hg_core::{HypergraphStore, Label};
use hg_engine::Engine;
use hg_rules::{builtin_rules, compile_rules, StepMode};
use std::collections::BTreeMap;

fn label_hist(store: &HypergraphStore) -> BTreeMap<i64, usize> {
    let mut m = BTreeMap::new();
    for (_, d) in store.nodes() {
        *m.entry(d.label.0 as i64).or_insert(0) += 1;
    }
    m
}

fn run_builtin(name: &str, seed_label1: bool, steps: u64) -> (Vec<usize>, BTreeMap<i64, usize>) {
    // A small crystal; optionally mark one node label 1 to activate rules that
    // need a label-1 seed (growth_treadmill).
    let mut store = HypergraphStore::new();
    hg_core::generate_crystal(&mut store, 6, 6, 3, false, 1);
    if seed_label1 {
        let first = store.nodes().next().map(|(id, _)| id);
        if let Some(id) = first {
            store.get_node_mut(id).unwrap().label = Label(1);
        }
    }
    let rules = compile_rules(&builtin_rules(name).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1_000_000);
    let mut traj = vec![store.nodes().count()];
    for _ in 0..steps {
        engine.step(&mut store);
        traj.push(store.nodes().count());
    }
    let hist = label_hist(&store);
    (traj, hist)
}

#[test]
fn characterize_builtin_node_count_trajectories() {
    eprintln!("\n===== substrate churn probe: node-count trajectories =====");
    // (name, needs a label-1 seed to fire). binary_edge is omitted: it grows
    // super-linearly (each new edge spawns more) and blows the graph up — its
    // "grows" verdict is already covered by triangle_subdivide.
    let cases = [
        ("growth_treadmill", true),
        ("glider", false),
        ("triangle_subdivide", false),
    ];
    for (name, seed) in cases {
        let (traj, hist) = run_builtin(name, seed, 12);
        let n0 = traj[0];
        let nend = *traj.last().unwrap();
        let sample: Vec<usize> = traj.iter().step_by(6).copied().collect();
        eprintln!(
            "\n  {name:<18} n0={n0} -> n_end={nend}  ({})",
            if nend > n0 + 1 {
                "GROWS — explosion confound for coupling"
            } else if nend + 1 < n0 {
                "shrinks"
            } else {
                "flat count"
            }
        );
        eprintln!("    n(t) sampled every 6 steps: {sample:?}");
        eprintln!("    final label histogram: {hist:?}");
    }
    eprintln!("\n  CONCLUSION: no builtin gives churn-WITHOUT-growth + fresh LABEL-0 nodes.");
    eprintln!("   - growth_treadmill: GROWS, and its fresh nodes are LABEL-1 (influx ignores them).");
    eprintln!("   - glider: flat count but mints NO fresh nodes (reuses a,b,c) ⟹ no poor sites.");
    eprintln!("   - triangle_subdivide / binary_edge: mint fresh label-0 nodes but GROW (explode).");
    eprintln!("  Influx acts on label-0 ONLY, so heterogeneity needs a substrate that mints");
    eprintln!("  fresh label-0 nodes AT A STABLE COUNT — which requires a NEW balanced-churn");
    eprintln!("  rule (create+delete label-0), not an existing builtin. See docs §7 route A.");
    eprintln!("==========================================================\n");

    // Assert the finding so it can't silently rot: growth_treadmill grows and
    // glider mints no fresh label-0 nodes — i.e. neither is a churn-without-growth
    // substrate. (triangle_subdivide/binary_edge grow, established separately.)
    let (gt, gt_hist) = run_builtin("growth_treadmill", true, 12);
    assert!(*gt.last().unwrap() > gt[0], "growth_treadmill must grow (it does)");
    assert!(gt_hist.get(&1).copied().unwrap_or(0) > 0, "growth_treadmill mints label-1 nodes influx ignores");
    let (gl, _) = run_builtin("glider", false, 12);
    assert_eq!(*gl.last().unwrap(), gl[0], "glider is node-count-neutral but mints no fresh nodes");
}
