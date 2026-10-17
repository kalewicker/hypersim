//! v4 F1 required tests: priority-as-axis (§1.2), guard-engagement
//! counters (§1.1), degree-guard pinned semantics (§6.2), and the
//! fused-energy ledger (§6.3).

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use hg_sim_core::{run_headless, SimConfig};
use smallvec::smallvec;

const ENERGY: u32 = 1;
const GENERATION: u32 = 6;

fn lkr_config_toml(dir: &std::path::Path, priority: &str, q: f64, tau: f64) -> String {
    format!(
        r#"
experiment_name = "v4_f1"
seed = 42
out_dir = "{}"
[rules]
source = "File"
file_path = "{}/../../rules/lk_pair.toml"
[initial]
condition_type = "TriLattice"
width = 16
[engine]
step_mode = "MaxParallel"
max_steps = 300
metrics_window_interval = 100
thread_count = 1
[lk_rules]
enabled = true
e_total = 20000.0
priority = "{priority}"
q = {q}
tau = {tau}
[leakage]
enabled = true
cadence_steps = 100
[output]
fixed_provenance = true
"#,
        dir.display(),
        env!("CARGO_MANIFEST_DIR"),
    )
}

/// §1.2: the priority axis is dynamically load-bearing (different
/// canonical hash), recorded in the manifest along with the §1.3
/// dead-zone flag, and validated (unknown value = config error).
#[test]
fn priority_axis_swaps_dynamics_and_is_recorded() {
    let run = |priority: &str, q: f64, tau: f64| -> (u64, serde_json::Value) {
        let dir = tempfile::tempdir().unwrap();
        let cfg = SimConfig::from_toml(&lkr_config_toml(dir.path(), priority, q, tau)).unwrap();
        let artifacts = run_headless(&cfg, None, None, None).expect("run");
        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(artifacts.run_dir.join("manifest.json")).unwrap(),
        )
        .unwrap();
        (artifacts.canonical_hash, manifest)
    };
    let (h_pump, m_pump) = run("subdivide_first", 50.0, 25.0);
    let (h_swap, m_swap) = run("transport_first", 50.0, 25.0);
    assert_ne!(h_pump, h_swap, "the priority axis must change the trajectory");
    assert_eq!(m_pump["priority"], "subdivide_first");
    assert_eq!(m_swap["priority"], "transport_first");
    // §1.3: q = 2τ at reference ⇒ children born in the dead zone.
    assert_eq!(m_pump["children_born_in_dead_zone"], true);
    let (_, m_unfrozen) = run("subdivide_first", 50.0, 15.0);
    assert_eq!(m_unfrozen["children_born_in_dead_zone"], false, "q > 2τ clears the flag");

    let dir = tempfile::tempdir().unwrap();
    let cfg = SimConfig::from_toml(&lkr_config_toml(dir.path(), "fuse_first", 50.0, 25.0)).unwrap();
    assert!(run_headless(&cfg, None, None, None).is_err(), "unknown priority is a config error");
}

/// §1.1 required test: scripted fixture with a KNOWN rejection count.
/// One edge x(E=200, gen=5)—y(E=0, gen=0) against the capped family
/// (g_max = 4): the matcher enumerates both orientations, so the energy
/// guard evaluates twice and rejects once (the y-as-payer orientation),
/// and the generation guard evaluates once (only after energy passes)
/// and rejects once (gen 5 ≥ 4). Exact, deterministic.
#[test]
fn guard_counters_scripted_rejection_count() {
    let rules_toml = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_pair_capped.toml"
    ))
    .unwrap();
    let mut store = HypergraphStore::new();
    let mut x = NodeData::new(Label(0), 0);
    x.set_property(ENERGY, PropertyValue::Float(200.0));
    x.set_property(GENERATION, PropertyValue::Float(5.0));
    let x = store.add_node(x);
    let mut y = NodeData::new(Label(0), 0);
    y.set_property(ENERGY, PropertyValue::Float(0.0));
    y.set_property(GENERATION, PropertyValue::Float(0.0));
    let y = store.add_node(y);
    store.add_edge(HyperEdge::new(Label(0), smallvec![x, y]));

    let rules = compile_rules(&parse_rules_toml(&rules_toml).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 100000);
    engine.step(&mut store);
    let drained = engine.drain_guard_stats();
    let subdivide = drained
        .iter()
        .find(|(name, _)| name == "lk_metered_subdivide")
        .expect("subdivide rule carries guards");
    // prop0 = energy guard, prop1 = generation guard (declaration order).
    assert_eq!(subdivide.1.property_guards[0], (2, 1), "energy guard: 2 evals, 1 reject");
    assert_eq!(subdivide.1.property_guards[1], (1, 1), "generation guard: 1 eval, 1 reject");
    // Drain resets: a second drain with no steps in between is empty.
    assert!(engine.drain_guard_stats().is_empty());
}

/// §1.1: guards.csv is emitted per window for leakage-enabled runs and
/// carries measured eval/reject counts for every declared guard.
#[test]
fn guards_csv_written_with_measured_counts() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = SimConfig::from_toml(&lkr_config_toml(dir.path(), "subdivide_first", 50.0, 25.0)).unwrap();
    cfg.rules.file_path = Some(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../rules/lk_pair_capped.toml").into(),
    );
    cfg.lk_rules.g_max = Some(4.0);
    let artifacts = run_headless(&cfg, None, None, None).expect("run");
    let content = std::fs::read_to_string(artifacts.run_dir.join("guards.csv")).expect("guards.csv");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(
        lines[1], "window,rule,guard_id,evals,rejects,claim_type",
        "guards.csv header is a frozen schema surface"
    );
    let rows: Vec<Vec<&str>> = lines[2..].iter().map(|l| l.split(',').collect()).collect();
    assert!(
        rows.iter().any(|r| r[1] == "lk_metered_subdivide" && r[2] == "prop0"
            && r[3].parse::<u64>().unwrap() > 0),
        "energy guard engaged: {rows:?}"
    );
    assert!(rows.iter().all(|r| r[5] == "measured"));
    // The transport compare guard must also be counted.
    assert!(rows.iter().any(|r| r[1] == "lk_transport" && r[2] == "cmp0"));
}

/// §6.2 pinned semantics: degree = distinct incident hyperedges; a node
/// appearing twice in one edge counts that edge ONCE. Plus determinism
/// and counters wired from birth.
#[test]
fn degree_guard_pinned_on_multi_incidence() {
    let rule = r#"
[[rules]]
name = "needs_degree_2"
[rules.lhs]
nodes = ["w"]
edges = []
[[rules.lhs.node_degree_compares]]
var = "w"
op = "eq"
value = 2
[rules.rhs]
nodes = ["w"]
edges = []
"#;
    let rules = compile_rules(&parse_rules_toml(rule).unwrap()).unwrap();
    let bindings_and_stats = |store: &HypergraphStore| {
        hg_engine::find_bindings_counted(&store.snapshot(), &rules[0], 0)
    };

    // w in a multi-incidence edge (w,w) + one ordinary edge (w,x):
    // 2 distinct edges → degree 2 → binds, despite 3 index slots.
    let mut store = HypergraphStore::new();
    let w = store.add_node(NodeData::new(Label(0), 0));
    let x = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), smallvec![w, w]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![w, x]));
    let (bindings, stats) = bindings_and_stats(&store);
    assert!(
        bindings.iter().any(|b| b.bound_nodes.first() == Some(&w)),
        "multi-incidence counts once: degree(w) = 2"
    );
    // x has degree 1 → rejected; counters saw both candidates.
    assert!(!bindings.iter().any(|b| b.bound_nodes.first() == Some(&x)));
    assert_eq!(stats.degree_compares[0], (2, 1), "2 evals (w, x), 1 reject (x)");

    // Determinism: identical stores give identical bindings and counts.
    let (b2, s2) = bindings_and_stats(&store);
    assert_eq!(b2.len(), bindings.len());
    assert_eq!(s2, stats);

    // Offset machinery: eq 1 + offset 1 ≡ eq 2.
    let rule_off = rule.replace("value = 2", "value = 1\noffset = 1");
    let rules_off = compile_rules(&parse_rules_toml(&rule_off).unwrap()).unwrap();
    let (b_off, _) =
        hg_engine::find_bindings_counted(&store.snapshot(), &rules_off[0], 0);
    assert!(b_off.iter().any(|b| b.bound_nodes.first() == Some(&w)));
}

/// §6.3 ledger, engine level: fusing a degree-2 gen-1 midpoint removes
/// the node, rejoins the endpoints, and records EXACTLY its energy as
/// vanished on the event.
#[test]
fn fuse_vanishes_node_with_exact_ledger_entry() {
    let fuse_toml = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../rules/lk_fuse.toml"
    ))
    .unwrap();
    let mut store = HypergraphStore::new();
    let node = |store: &mut HypergraphStore, e: f64, g: f64| -> NodeId {
        let mut d = NodeData::new(Label(0), 0);
        d.set_property(ENERGY, PropertyValue::Float(e));
        d.set_property(GENERATION, PropertyValue::Float(g));
        store.add_node(d)
    };
    let x = node(&mut store, 10.0, 0.0);
    let w = node(&mut store, 50.0, 1.0);
    let y = node(&mut store, 20.0, 0.0);
    store.add_edge(HyperEdge::new(Label(0), smallvec![x, w]));
    store.add_edge(HyperEdge::new(Label(0), smallvec![w, y]));

    let rules = compile_rules(&parse_rules_toml(&fuse_toml).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 100000);
    let mut vanished_total = 0.0f64;
    struct Probe(std::sync::Arc<std::sync::Mutex<f64>>);
    impl hg_observe::Observer for Probe {
        fn on_step_end(
            &mut self,
            _s: &hg_core::GraphSnapshot<'_>,
            _st: &hg_observe::StepStats,
            events: &[hg_observe::RewriteEvent],
        ) {
            *self.0.lock().unwrap() += events.iter().map(|e| e.vanished_energy).sum::<f64>();
        }
    }
    let sum = std::sync::Arc::new(std::sync::Mutex::new(0.0));
    engine.add_observer(Box::new(Probe(std::sync::Arc::clone(&sum))));
    engine.step(&mut store);
    vanished_total += *sum.lock().unwrap();

    assert!(store.get_node(w).is_none(), "w removed");
    assert_eq!(vanished_total, 50.0, "exactly w's energy on the ledger");
    assert_eq!(store.node_count(), 2);
    assert_eq!(store.edge_count(), 1, "endpoints rejoined by one edge");
    let rejoined = store.edges().next().unwrap().1;
    let mut ends: Vec<NodeId> = rejoined.nodes.to_vec();
    ends.sort_unstable();
    let mut expect = vec![x, y];
    expect.sort_unstable();
    assert_eq!(ends, expect);
}
