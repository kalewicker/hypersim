//! Tests for hg-core crate

use hg_core::graph::{HypergraphStore, NodeData, HyperEdge, Label};
use smallvec::SmallVec;

#[test]
fn stale_nodeid_returns_none_after_removal() {
    let mut store = HypergraphStore::new();
    let id = store.add_node(NodeData::new(Label(0), 0));
    assert!(store.get_node(id).is_some());
    store.remove_node(id);
    assert!(store.get_node(id).is_none());
}

#[test]
fn stale_edgeid_returns_none_after_removal() {
    let mut store = HypergraphStore::new();
    let n1 = store.add_node(NodeData::new(Label(0), 0));
    let n2 = store.add_node(NodeData::new(Label(0), 0));
    let edge = HyperEdge::new(Label(0), SmallVec::from_slice(&[n1, n2]));
    let id = store.add_edge(edge);
    assert!(store.get_edge(id).is_some());
    store.remove_edge(id);
    assert!(store.get_edge(id).is_none());
}

#[test]
fn incident_edge_list_updated_on_remove_node() {
    let mut store = HypergraphStore::new();
    let n1 = store.add_node(NodeData::new(Label(0), 0));
    let n2 = store.add_node(NodeData::new(Label(0), 0));
    let n3 = store.add_node(NodeData::new(Label(0), 0));
    
    let e1 = store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[n1, n2])));
    let e2 = store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[n1, n3])));
    
    assert_eq!(store.edges_incident_to(n1).len(), 2);
    assert_eq!(store.edges_incident_to(n2).len(), 1);
    assert_eq!(store.edges_incident_to(n3).len(), 1);
    
    store.remove_node(n1);
    
    // n1 is gone
    assert!(store.get_node(n1).is_none());
    // n2 and n3 should have no incident edges now
    assert_eq!(store.edges_incident_to(n2).len(), 0);
    assert_eq!(store.edges_incident_to(n3).len(), 0);
    // edges should be removed
    assert!(store.get_edge(e1).is_none());
    assert!(store.get_edge(e2).is_none());
}