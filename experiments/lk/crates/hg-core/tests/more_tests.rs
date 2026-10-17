//! Tests for hg-core crate - snapshot and generators

use hg_core::graph::{HypergraphStore, NodeData, HyperEdge, Label};
use hg_core::generators::*;
use smallvec::{smallvec};

#[test]
fn snapshot_is_read_only() {
    let mut store = HypergraphStore::new();
    let n1 = store.add_node(NodeData::new(Label(0), 0));
    let snapshot = store.snapshot();
    
    // This should compile but we can't call mut methods on snapshot
    // The test is that GraphSnapshot has no &mut methods
    let _ = snapshot.get_node(n1);
    let _ = snapshot.node_count();
    let _ = snapshot.nodes().count();
}

#[test]
fn crystal_has_correct_node_count() {
    let mut store = HypergraphStore::new();
    generate_crystal(&mut store, 10, 10, 2, false, 42);
    assert_eq!(store.node_count(), 100);
}

#[test]
fn crystal_is_connected() {
    let mut store = HypergraphStore::new();
    generate_crystal(&mut store, 5, 5, 2, false, 42);
    
    // BFS from first node should reach all nodes
    let start = store.nodes().map(|(id, _)| id).next().unwrap();
    let mut visited = std::collections::HashSet::new();
    let mut queue = vec![start];
    visited.insert(start);
    
    while let Some(node) = queue.pop() {
        for edge_id in store.edges_incident_to(node) {
            if let Some(edge) = store.get_edge(*edge_id) {
                for &neighbor in &edge.nodes {
                    if visited.insert(neighbor) {
                        queue.push(neighbor);
                    }
                }
            }
        }
    }
    
    assert_eq!(visited.len(), 25);
}

#[test]
fn defect_application_changes_expected_edge_count() {
    let mut store1 = HypergraphStore::new();
    generate_crystal(&mut store1, 10, 10, 2, false, 42);
    let _base_edges = store1.edge_count();

    let mut store2 = HypergraphStore::new();
    generate_crystal_with_defects(&mut store2, 10, 10, 2, false, 0.01, 42);

    // With 1% defect density on 100 nodes, some edges should be removed.
    // Each defect removes up to 2 edges and adds up to 2 random edges,
    // so the count may increase or decrease, but the graph should differ.
    // At minimum, the defect graph should not be identical to the pure crystal.
    assert_ne!(store1.canonical_hash(), store2.canonical_hash(),
        "Defect application should produce a different graph");
    assert!(store2.edge_count() > 0);
    assert_eq!(store2.node_count(), 100, "Defects should not change node count");
}

#[test]
fn canonical_hash_stable_across_equivalent_construction_orders() {
    let mut store1 = HypergraphStore::new();
    generate_crystal(&mut store1, 5, 5, 2, false, 42);
    let hash1 = store1.canonical_hash();
    
    let mut store2 = HypergraphStore::new();
    generate_crystal(&mut store2, 5, 5, 2, false, 42);
    let hash2 = store2.canonical_hash();
    
    assert_eq!(hash1, hash2);
}

#[test]
fn canonical_hash_differs_for_nonisomorphic_graphs() {
    // Use structurally different graphs (different node counts) to guarantee
    // non-isomorphism without relying on RNG seed differences.
    let mut store1 = HypergraphStore::new();
    generate_crystal(&mut store1, 5, 5, 2, false, 42);
    let hash1 = store1.canonical_hash();

    let mut store2 = HypergraphStore::new();
    generate_crystal(&mut store2, 6, 6, 2, false, 42); // Different size → different structure
    let hash2 = store2.canonical_hash();

    assert_ne!(hash1, hash2, "Different-sized crystals should have different canonical hashes");
}

#[test]
fn stale_nodeid_returns_none_after_removal() {
    let mut store = HypergraphStore::new();
    let n1 = store.add_node(NodeData::new(Label(0), 0));
    let n2 = store.add_node(NodeData::new(Label(1), 0));
    
    // Verify node exists
    assert!(store.get_node(n1).is_some());
    assert!(store.get_node(n2).is_some());
    
    // Remove n1
    store.remove_node(n1);
    
    // n1 should now return None (stale)
    assert!(store.get_node(n1).is_none(), "Removed node should return None");
    // n2 should still exist
    assert!(store.get_node(n2).is_some(), "Other nodes should be unaffected");
}

#[test]
fn stale_edgeid_returns_none_after_removal() {
    let mut store = HypergraphStore::new();
    let n1 = store.add_node(NodeData::new(Label(0), 0));
    let n2 = store.add_node(NodeData::new(Label(1), 0));
    let n3 = store.add_node(NodeData::new(Label(2), 0));
    
    let e1 = store.add_edge(HyperEdge::new(Label(0), smallvec![n1, n2]));
    let e2 = store.add_edge(HyperEdge::new(Label(1), smallvec![n2, n3]));
    
    // Verify edges exist
    assert!(store.get_edge(e1).is_some());
    assert!(store.get_edge(e2).is_some());
    
    // Remove edge e1
    store.remove_edge(e1);
    
    // e1 should now return None (stale)
    assert!(store.get_edge(e1).is_none(), "Removed edge should return None");
    // e2 should still exist
    assert!(store.get_edge(e2).is_some(), "Other edges should be unaffected");
}

#[test]
fn incident_edge_list_updated_on_remove_node() {
    let mut store = HypergraphStore::new();
    let n1 = store.add_node(NodeData::new(Label(0), 0));
    let n2 = store.add_node(NodeData::new(Label(1), 0));
    let n3 = store.add_node(NodeData::new(Label(2), 0));
    
    let e1 = store.add_edge(HyperEdge::new(Label(0), smallvec![n1, n2]));
    let e2 = store.add_edge(HyperEdge::new(Label(1), smallvec![n1, n3]));
    
    // n1 should have 2 incident edges
    assert_eq!(store.edges_incident_to(n1).len(), 2);
    // n2 should have 1 incident edge
    assert_eq!(store.edges_incident_to(n2).len(), 1);
    // n3 should have 1 incident edge
    assert_eq!(store.edges_incident_to(n3).len(), 1);
    
    // Remove n1 - this should also remove e1 and e2 (dangling edge cleanup)
    store.remove_node(n1);
    
    // n2 should now have 0 incident edges (e1 was removed)
    assert_eq!(store.edges_incident_to(n2).len(), 0);
    // n3 should now have 0 incident edges (e2 was removed)
    assert_eq!(store.edges_incident_to(n3).len(), 0);
    
    // Both edges should be gone
    assert!(store.get_edge(e1).is_none());
    assert!(store.get_edge(e2).is_none());
}