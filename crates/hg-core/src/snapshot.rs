//! Read-only graph snapshot for observers

use crate::graph::{HypergraphStore, HyperEdge, NodeData, NodeId, EdgeId, Label};

/// Read-only view of the hypergraph for observers
/// Enforces that observers cannot mutate the store (compile-time via lifetime)
pub struct GraphSnapshot<'a> {
    pub(crate) store: &'a HypergraphStore,
}

impl<'a> GraphSnapshot<'a> {
    /// Get node data
    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.store.get_node(id)
    }
    
    /// Get edge data
    pub fn get_edge(&self, id: EdgeId) -> Option<&HyperEdge> {
        self.store.get_edge(id)
    }
    
    /// Get edges with label
    pub fn edges_with_label(&self, label: Label) -> &[EdgeId] {
        self.store.edges_with_label(label)
    }
    
    /// Get edges incident to node
    pub fn edges_incident_to(&self, node: NodeId) -> &[EdgeId] {
        self.store.edges_incident_to(node)
    }
    
    /// Node count
    pub fn node_count(&self) -> usize {
        self.store.node_count()
    }
    
    /// Edge count
    pub fn edge_count(&self) -> usize {
        self.store.edge_count()
    }
    
    /// Iterate all nodes
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &NodeData)> + '_ {
        self.store.nodes()
    }
    
    /// Iterate all edges
    pub fn edges(&self) -> impl Iterator<Item = (EdgeId, &HyperEdge)> + '_ {
        self.store.edges()
    }
    
    /// Current step
    pub fn step(&self) -> u64 {
        self.store.step()
    }
    
    /// Get all node IDs
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.store.nodes().map(|(id, _)| id)
    }
    
    /// Get all edge IDs
    pub fn edge_ids(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.store.edges().map(|(id, _)| id)
    }
}