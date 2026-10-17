//! `RewriteEvent.cleanup_neighbors` (LK2 bookkeeping): when a deleted
//! LHS-only node drags incident edges down as dangling cleanup, the
//! SURVIVING endpoints of those edges are recorded on the event — they
//! are the nodes affected through the engine's documented
//! locality-of-effect exception (v4 §8.1), and downstream causal
//! analyses report that channel separately.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_engine::Engine;
use hg_observe::{Observer, RewriteEvent, StepStats};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use smallvec::SmallVec;
use std::sync::{Arc, Mutex};

const DELETE_MARKED: &str = r#"
[[rules]]
name = "delete_marked"
priority = 0
weight = 1.0
enabled = true
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.lhs.node_labels]
a = 1
b = 0
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b"]
[rules.rhs]
nodes = ["b"]
edges = []
"#;

struct Capture(Arc<Mutex<Vec<RewriteEvent>>>);
impl Observer for Capture {
    fn on_step_end(
        &mut self,
        _snapshot: &hg_core::GraphSnapshot<'_>,
        _stats: &StepStats,
        events: &[RewriteEvent],
    ) {
        self.0.lock().unwrap().extend_from_slice(events);
    }
}

#[test]
fn cleanup_deleted_edge_endpoints_are_recorded() {
    // Star: marked center with 4 spokes. One binding fires (all bindings
    // conflict on the center); the matched edge is deleted normally, the
    // other 3 spokes' edges go as dangling cleanup — their surviving
    // endpoints must land in cleanup_neighbors.
    let mut store = HypergraphStore::new();
    let center = store.add_node(NodeData::new(Label(1), 0));
    let spokes: Vec<_> = (0..4)
        .map(|_| store.add_node(NodeData::new(Label(0), 0)))
        .collect();
    for &s in &spokes {
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[center, s])));
    }

    let rules = compile_rules(&parse_rules_toml(DELETE_MARKED).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 1000);
    let captured = Arc::new(Mutex::new(Vec::new()));
    engine.add_observer(Box::new(Capture(Arc::clone(&captured))));

    engine.step(&mut store);

    let events = captured.lock().unwrap();
    assert_eq!(events.len(), 1, "exactly one non-conflicting binding fires");
    let e = &events[0];
    assert_eq!(e.nodes_deleted, vec![center]);
    assert_eq!(e.edges_deleted.len(), 4, "1 matched + 3 dangling");
    // The matched spoke survives and is a bound node, not a cleanup
    // neighbor; the other three spokes are cleanup neighbors.
    let matched_spoke = e.binding.bound_nodes[1];
    assert_eq!(e.cleanup_neighbors.len(), 3);
    assert!(!e.cleanup_neighbors.contains(&matched_spoke));
    assert!(!e.cleanup_neighbors.contains(&center));
    for s in &spokes {
        if *s != matched_spoke {
            assert!(e.cleanup_neighbors.contains(s), "spoke missing from cleanup set");
        }
    }
    // Store state sanity: center gone, all edges gone, spokes alive.
    assert_eq!(store.node_count(), 4);
    assert_eq!(store.edge_count(), 0);
}

#[test]
fn pure_relabel_has_no_cleanup_neighbors() {
    let mut store = HypergraphStore::new();
    let a = store.add_node(NodeData::new(Label(1), 0));
    let b = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));

    let rules = compile_rules(
        &parse_rules_toml(include_str!("../../../rules/excitation_front.toml")).unwrap(),
    )
    .unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 1000);
    let captured = Arc::new(Mutex::new(Vec::new()));
    engine.add_observer(Box::new(Capture(Arc::clone(&captured))));
    engine.step(&mut store);

    let events = captured.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].cleanup_neighbors.is_empty());
    assert!(events[0].nodes_deleted.is_empty());
}
