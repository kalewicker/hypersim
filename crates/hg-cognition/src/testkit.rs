//! Small graph/binding builders shared by this crate's unit tests, its
//! integration tests, and the criterion bench.
//!
//! Public so `tests/` and `benches/` can use it; `#[doc(hidden)]` because
//! it is not part of the crate's interface. Nothing in the policy zoo
//! depends on it.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
use hg_observe::Binding;
use smallvec::SmallVec;

/// A ring of `n` nodes joined by binary edges, plus `hubs` extra edges
/// attached to node 0 so degrees are not uniform (feature 4 needs spread
/// to be worth extracting).
pub fn ring_store(n: usize, hubs: usize) -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let ids: Vec<NodeId> = (0..n)
        .map(|_| store.add_node(NodeData::new(Label(0), 0)))
        .collect();
    for i in 0..n {
        let mut ends: SmallVec<[NodeId; 8]> = SmallVec::new();
        ends.push(ids[i]);
        ends.push(ids[(i + 1) % n]);
        store.add_edge(HyperEdge::new(Label(0), ends));
    }
    for j in 0..hubs {
        let mut ends: SmallVec<[NodeId; 8]> = SmallVec::new();
        ends.push(ids[0]);
        ends.push(ids[(j + 2) % n]);
        store.add_edge(HyperEdge::new(Label(1), ends));
    }
    store
}

pub fn node_ids(store: &HypergraphStore) -> Vec<NodeId> {
    store.nodes().map(|(id, _)| id).collect()
}

/// A creation binding: zero bound nodes, zero matched edges.
pub fn creation(rule_id: usize, priority: i32) -> Binding {
    Binding::new(rule_id, priority, SmallVec::new())
}

/// An ordinary pattern-match binding over `nodes`.
pub fn matched(rule_id: usize, priority: i32, nodes: &[NodeId]) -> Binding {
    let mut bound: SmallVec<[NodeId; 8]> = SmallVec::new();
    bound.extend_from_slice(nodes);
    Binding::new(rule_id, priority, bound)
}

/// `n_match` pattern-match bindings spread over `n_rules` rules and the
/// store's nodes, preceded by `n_creation` creation bindings.
pub fn candidate_pool(
    store: &HypergraphStore,
    n_creation: usize,
    n_match: usize,
    n_rules: usize,
    arity: usize,
) -> Vec<Binding> {
    let ids = node_ids(store);
    let mut out: Vec<Binding> = (0..n_creation).map(|i| creation(i % n_rules, 0)).collect();
    for i in 0..n_match {
        let nodes: Vec<NodeId> = (0..arity).map(|a| ids[(i * arity + a) % ids.len()]).collect();
        out.push(matched(i % n_rules, (i % 3) as i32, &nodes));
    }
    out
}
