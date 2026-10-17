//! The apply-path relabel fix: an RHS node_label on a SURVIVING var must
//! be applied to the bound node (this path was missing entirely — RHS
//! labels on survivors were silently ignored for every rule). Absence of
//! an explicit RHS label must keep the survivor's label, never reset it.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use smallvec::SmallVec;

fn two_node_store() -> (HypergraphStore, hg_core::NodeId, hg_core::NodeId) {
    let mut store = HypergraphStore::new();
    let a = store.add_node(NodeData::new(Label(1), 0));
    let b = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    (store, a, b)
}

#[test]
fn explicit_rhs_label_relabels_surviving_node() {
    let rules = compile_rules(
        &parse_rules_toml(
            r#"
[[rules]]
name = "swap"
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
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs.node_labels]
a = 0
b = 1
"#,
        )
        .unwrap(),
    )
    .unwrap();

    let (mut store, a, b) = two_node_store();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 4);
    let stats = engine.step(&mut store);
    assert_eq!(stats.accepted_rewrites, 1);
    // Same nodes, swapped labels, nothing created or destroyed.
    assert_eq!(store.get_node(a).unwrap().label, Label(0), "a must be relabelled");
    assert_eq!(store.get_node(b).unwrap().label, Label(1), "b must be relabelled");
    assert_eq!(store.node_count(), 2);
    assert_eq!(store.edge_count(), 1);
}

#[test]
fn survivor_without_explicit_rhs_label_keeps_its_label() {
    // RHS labels only the created node; survivors a (1) and b (0) keep
    // their labels — no silent reset to 0.
    let rules = compile_rules(
        &parse_rules_toml(
            r#"
[[rules]]
name = "grow"
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
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["a", "c"]]
[rules.rhs.node_labels]
c = 2
"#,
        )
        .unwrap(),
    )
    .unwrap();

    let (mut store, a, b) = two_node_store();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 4);
    assert_eq!(engine.step(&mut store).accepted_rewrites, 1);
    assert_eq!(store.get_node(a).unwrap().label, Label(1), "a keeps its label");
    assert_eq!(store.get_node(b).unwrap().label, Label(0), "b keeps its label");
    let created: Vec<_> = store
        .nodes()
        .filter(|(id, _)| *id != a && *id != b)
        .collect();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].1.label, Label(2));
}
