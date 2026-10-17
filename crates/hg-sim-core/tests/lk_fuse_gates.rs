//! F2 gates (plan v4 §7): FG1 audit-exact with the fused ledger, FG2
//! no-churn at reference params, FG3 lattice-inert with an ENGAGED
//! generation guard, FG4 removal-capable on a scripted throat.
//!
//! v5 §3 amendments: FG4's throat interiors sit BELOW f_cap (the v4
//! flat-q fixture would make a throttled FG4 vacuously fail), and FG5
//! pins f_cap boundary strictness (v5 §9.4: any off-by-one silently
//! shifts every t_age in the R3 sweep). Both build their rules through
//! the PRODUCTION path (`load_rules` with `[lk_rules.fuse] f_cap`), so
//! the config plumbing is what gets gated.
//!
//! FG2 pre-registration (the F2 notes the plan requires BEFORE the
//! first fuse run): churn threshold = 0 at reference params in the
//! pump regime — interior fuses rejoin sub-θ endpoints, which cannot
//! re-subdivide, so any nonzero churn is signal.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::StepMode;
use hg_sim_core::{run_headless, SimConfig};
use smallvec::smallvec;

const ENERGY: u32 = 1;
const GENERATION: u32 = 6;

fn fuse_run_config(dir: &std::path::Path, e_total: f64, max_steps: u64) -> SimConfig {
    let toml = format!(
        r#"
experiment_name = "fg_gates"
seed = 42
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/lk_pair_capped.toml"
[initial]
condition_type = "TriLattice"
width = 16
[engine]
step_mode = "MaxParallel"
max_steps = {max_steps}
metrics_window_interval = 100
thread_count = 1
[lk_rules]
enabled = true
e_total = {e_total}
g_max = 1.0e18
[lk_rules.fuse]
enabled = true
[leakage]
enabled = true
cadence_steps = 100
[output]
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR"),
    );
    let mut cfg = SimConfig::from_toml(&toml).unwrap();
    // Cap ∞ via the same string patch the sweeps use.
    let rules = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap()
    .replace(
        "property = 6\nop       = \"lt\"\nvalue    = 4.0",
        "property = 6\nop       = \"lt\"\nvalue    = 1.0e18",
    );
    cfg.rules.source = hg_sim_core::RuleSource::Inline;
    cfg.rules.file_path = None;
    cfg.rules.inline_toml = Some(rules);
    cfg
}

/// FG1 + FG2: with fuse active through a full era and past quench, the
/// extended invariant E + c·N_sub + Σ E_fused = E₀ holds EXACTLY at
/// every window, the ledger actually engages (fuse fires), and the
/// churn counter stays at the pre-registered threshold (0).
#[test]
fn fg1_audit_exact_and_fg2_no_churn() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = fuse_run_config(dir.path(), 60_000.0, 1400);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let leakage = std::fs::read_to_string(artifacts.run_dir.join("leakage.csv")).unwrap();
    let rows: Vec<Vec<&str>> = leakage.lines().skip(2).map(|l| l.split(',').collect()).collect();
    assert!(rows.len() >= 10);
    for row in &rows {
        let e_now: f64 = row[14].parse().unwrap();
        let n_sub: f64 = row[15].parse().unwrap();
        let e_fused: f64 = row[16].parse().unwrap();
        let residual: f64 = row[18].parse().unwrap();
        assert_eq!(residual, 0.0, "FG1 audit drift at step {}", row[0]);
        assert_eq!(
            e_now + 10.0 * n_sub + e_fused,
            60_000.0,
            "FG1 recomputed invariant at step {}",
            row[0]
        );
    }
    let last = rows.last().unwrap();
    let e_fused_final: f64 = last[16].parse().unwrap();
    assert!(e_fused_final > 0.0, "the fuse rule must actually engage the ledger");
    let churn_final: u32 = last[17].parse().unwrap();
    assert_eq!(churn_final, 0, "FG2: churn at reference params must be 0 (pre-registered)");
}

/// FG3 lattice-inert: on a clean lattice with no gen > 0 nodes (and no
/// energy above θ so nothing subdivides, no gradient ≥ 2τ so nothing
/// transports), fuse never fires — AND the generation guard is ENGAGED
/// (nonzero rejections), not merely never-evaluated (§1.1: the EXP-L9
/// lesson, promoted to a gate).
#[test]
fn fg3_lattice_inert_with_engaged_guard() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = fuse_run_config(dir.path(), 20.0, 300);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let metrics = std::fs::read_to_string(&artifacts.csv_path).unwrap();
    let header: Vec<&str> = metrics.lines().nth(1).unwrap().split(',').collect();
    let n_ix = header.iter().position(|h| *h == "n_nodes").unwrap();
    let counts: Vec<u64> = metrics
        .lines()
        .skip(2)
        .filter_map(|l| l.split(',').nth(n_ix)?.parse().ok())
        .collect();
    assert!(
        counts.iter().all(|&n| n == 256),
        "FG3: the clean lattice must be untouched (no fuse, no subdivide): {counts:?}"
    );
    let guards = std::fs::read_to_string(artifacts.run_dir.join("guards.csv")).expect("guards.csv");
    let fuse_gen_rejects: u64 = guards
        .lines()
        .skip(2)
        .filter_map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            (f[1] == "lk_fuse" && f[2] == "prop0").then(|| f[4].parse::<u64>().unwrap_or(0))
        })
        .sum();
    assert!(
        fuse_gen_rejects > 0,
        "FG3: the generation guard must be ENGAGED (nonzero rejections), not never-evaluated"
    );
}

/// The PRODUCTION fuse rule set for a given throttle: `load_rules` on a
/// minimal config with an empty base rule set, so what comes back is
/// exactly the appended lk_fuse rule with (or without) the f_cap guard —
/// the same object every sweep executes.
fn fuse_rules_via_config(f_cap: Option<f64>) -> Vec<hg_rules::CompiledRule> {
    let toml = format!(
        r#"
experiment_name = "fg_fuse_rules"
seed = 42
[rules]
source = "Inline"
[lk_rules]
enabled = true
[lk_rules.fuse]
enabled = true
{}
"#,
        f_cap.map(|f| format!("f_cap = {f:.1}")).unwrap_or_default(),
    );
    let mut cfg = SimConfig::from_toml(&toml).unwrap();
    cfg.rules.inline_toml = Some("rules = []".into());
    hg_sim_core::load_rules(&cfg).unwrap()
}

/// FG4 removal-capable (v5-amended): on a scripted throat S–w₁–…–w₅–N
/// (interior generations ≥ 1, interior energies BELOW f_cap, everything
/// sub-θ so only fuse can fire), the THROTTLED fuse reduces the chain
/// monotonically to zero and the census follows; the endpoints end
/// rejoined by a single edge.
#[test]
fn fg4_removal_capable_on_scripted_throat() {
    const F_CAP: f64 = 40.0;
    let mut store = HypergraphStore::new();
    let node = |store: &mut HypergraphStore, g: f64| -> NodeId {
        let mut d = NodeData::new(Label(0), 0);
        // v5 §3: interiors below f_cap — an aged/drained throat, the
        // state the guard is designed to admit.
        d.set_property(ENERGY, PropertyValue::Float(30.0));
        d.set_property(GENERATION, PropertyValue::Float(g));
        store.add_node(d)
    };
    let s = node(&mut store, 0.0);
    let n = node(&mut store, 0.0);
    let mut prev = s;
    let mut chain = Vec::new();
    for i in 1..=5 {
        let w = node(&mut store, i as f64);
        store.add_edge(HyperEdge::new(Label(0), smallvec![prev, w]));
        chain.push(w);
        prev = w;
    }
    store.add_edge(HyperEdge::new(Label(0), smallvec![prev, n]));
    // Give S and N lattice-like degree so they are not fusable even in
    // principle (degree > 2) — mirrors a throat anchored in the bulk.
    let a = node(&mut store, 0.0);
    let b = node(&mut store, 0.0);
    store.add_edge(HyperEdge::new(Label(0), smallvec![s, a]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![n, b]));

    let rules = fuse_rules_via_config(Some(F_CAP));
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 100000);
    let mut census_trace = vec![hg_eval::chains::chain_census(&store).chains_count];
    let mut node_trace = vec![store.node_count()];
    for _ in 0..12 {
        engine.step(&mut store);
        census_trace.push(hg_eval::chains::chain_census(&store).chains_count);
        node_trace.push(store.node_count());
        if hg_eval::chains::chain_census(&store).chains_count == 0 {
            break;
        }
    }
    assert!(
        node_trace.windows(2).all(|w| w[1] <= w[0]),
        "FG4: node count must decrease monotonically: {node_trace:?}"
    );
    assert_eq!(
        *census_trace.last().unwrap(),
        0,
        "FG4: the chain census must reach zero: {census_trace:?}"
    );
    for w in chain {
        assert!(store.get_node(w).is_none(), "every interior node removed");
    }
    // S and N rejoined directly.
    let joined = store.edges().any(|(_, e)| {
        let mut v: Vec<NodeId> = e.nodes.to_vec();
        v.sort_unstable();
        let mut want = vec![s, n];
        want.sort_unstable();
        v == want
    });
    assert!(joined, "FG4: throat endpoints rejoined by a single edge");
}

/// FG5 aging (v5 §3): on a scripted drain sequence, a node becomes
/// fusable EXACTLY when its energy first drops below f_cap, and not one
/// step earlier. The boundary is STRICT — a node at exactly f_cap is
/// not fusable (v5 §9.4: an off-by-one here silently shifts every t_age
/// in the R3 sweep). Also asserts the guard counters are ENGAGED
/// (§1.1: rejections observed, not merely never-evaluated).
#[test]
fn fg5_aging_boundary_strictness() {
    const F_CAP: f64 = 40.0;
    let mut store = HypergraphStore::new();
    let node = |store: &mut HypergraphStore, g: f64, e: f64| -> NodeId {
        let mut d = NodeData::new(Label(0), 0);
        d.set_property(ENERGY, PropertyValue::Float(e));
        d.set_property(GENERATION, PropertyValue::Float(g));
        store.add_node(d)
    };
    let s = node(&mut store, 0.0, 0.0);
    let n = node(&mut store, 0.0, 0.0);
    let w = node(&mut store, 1.0, F_CAP + 20.0);
    store.add_edge(HyperEdge::new(Label(0), smallvec![s, w]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![w, n]));
    // Anchor the endpoints in a bulk (mirrors FG4).
    let a = node(&mut store, 0.0, 0.0);
    let b = node(&mut store, 0.0, 0.0);
    store.add_edge(HyperEdge::new(Label(0), smallvec![s, a]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![n, b]));

    let rules = fuse_rules_via_config(Some(F_CAP));
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 100000);

    // Scripted drain: above the cap, AT the cap (the boundary step —
    // must NOT fuse), below the cap (must fuse this step, not later).
    for (energy, alive_after) in [(F_CAP + 20.0, true), (F_CAP, true), (F_CAP - 25.0, false)] {
        if let Some(d) = store.get_node_mut(w) {
            d.set_property(ENERGY, PropertyValue::Float(energy));
        }
        engine.step(&mut store);
        assert_eq!(
            store.get_node(w).is_some(),
            alive_after,
            "FG5: at w.energy = {energy} the node must be {}",
            if alive_after { "NOT fusable (strict <)" } else { "fused" },
        );
    }

    // Guard engagement: the f_cap guard (prop1; prop0 is generation) was
    // evaluated every step and rejected exactly the two ≥-f_cap steps.
    let stats = engine.drain_guard_stats();
    let (_, gs) = stats
        .iter()
        .find(|(name, _)| name == "lk_fuse")
        .expect("lk_fuse guard stats");
    let (evals, rejects) = gs.property_guards[1];
    assert!(
        evals >= 3 && rejects >= 2 && evals > rejects,
        "FG5: f_cap guard must be engaged (evals {evals}, rejects {rejects})"
    );

    // Endpoints rejoined by the accepted terminal fuse.
    let joined = store.edges().any(|(_, e)| {
        let mut v: Vec<NodeId> = e.nodes.to_vec();
        v.sort_unstable();
        let mut want = vec![s, n];
        want.sort_unstable();
        v == want
    });
    assert!(joined, "FG5: endpoints rejoined once the drain crossed f_cap");
}
