//! Small helpers over `HypergraphStore` used by the RuleStore side:
//! deterministic (sorted) enumeration, rule-subgraph traversal, duplication.
//!
//! Slotmap keys are allocation-ordered, not content-canonical; every
//! enumeration here sorts by ffi key, which is deterministic for identical
//! construction sequences (the workspace reproducibility policy).

use crate::schema::*;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};

pub fn key_u64(id: NodeId) -> u64 {
    use slotmap::Key;
    id.data().as_ffi()
}

pub fn get_int(s: &HypergraphStore, n: NodeId, key: u32) -> Option<i64> {
    match s.get_node(n)?.get_property(key)? {
        PropertyValue::Int(v) => Some(v),
        _ => None,
    }
}

pub fn set_int(s: &mut HypergraphStore, n: NodeId, key: u32, v: i64) {
    if let Some(d) = s.get_node_mut(n) {
        d.set_property(key, PropertyValue::Int(v));
    }
}

pub fn add_rule_node(
    s: &mut HypergraphStore,
    label: u32,
    props: Vec<(u32, PropertyValue)>,
) -> NodeId {
    let mut d = NodeData::new(Label(label), 0);
    for (k, v) in props {
        d.set_property(k, v);
    }
    s.add_node(d)
}

pub fn add_role_edge(s: &mut HypergraphStore, root: NodeId, child: NodeId, role: u32) {
    s.add_edge(HyperEdge::new(
        Label(role),
        SmallVec::from_slice(&[root, child]),
    ));
}

/// All RuleRoot ids, sorted by ffi key (deterministic enumeration base).
pub fn rule_roots(s: &HypergraphStore) -> Vec<NodeId> {
    let mut v: Vec<NodeId> = s
        .nodes()
        .filter(|(_, d)| d.label.0 == RULE_ROOT)
        .map(|(id, _)| id)
        .collect();
    v.sort_by_key(|&id| key_u64(id));
    v
}

/// Children of `root` reached over edges with the given role label, sorted.
pub fn children(s: &HypergraphStore, root: NodeId, role: u32) -> Vec<NodeId> {
    let mut out = Vec::new();
    for &e in s.edges_incident_to(root) {
        if let Some(edge) = s.get_edge(e) {
            if edge.label.0 == role {
                for &n in &edge.nodes {
                    if n != root {
                        out.push(n);
                    }
                }
            }
        }
    }
    out.sort_by_key(|&id| key_u64(id));
    out
}

/// Full rule subgraph reachable from `root` over role edges (labels >= ROLE_BASE).
pub fn rule_subgraph(s: &HypergraphStore, root: NodeId) -> (Vec<NodeId>, Vec<hg_core::EdgeId>) {
    let mut seen: HashSet<NodeId> = HashSet::new();
    let mut stack = vec![root];
    let mut nodes = Vec::new();
    let mut edge_ids: HashSet<hg_core::EdgeId> = HashSet::new();
    while let Some(cur) = stack.pop() {
        if !seen.insert(cur) {
            continue;
        }
        nodes.push(cur);
        for &e in s.edges_incident_to(cur) {
            if let Some(edge) = s.get_edge(e) {
                if edge.label.0 >= ROLE_BASE {
                    edge_ids.insert(e);
                    for &n in &edge.nodes {
                        if !seen.contains(&n) {
                            stack.push(n);
                        }
                    }
                }
            }
        }
    }
    nodes.sort_by_key(|&id| key_u64(id));
    let mut es: Vec<hg_core::EdgeId> = edge_ids.into_iter().collect();
    es.sort_by_key(|&id| {
        use slotmap::Key;
        id.data().as_ffi()
    });
    (nodes, es)
}

/// Duplicate a rule subgraph; returns the new root. Node insertion follows
/// the sorted source order, so the construction sequence is deterministic.
pub fn duplicate_rule(s: &mut HypergraphStore, root: NodeId) -> NodeId {
    let (nodes, edges) = rule_subgraph(s, root);
    let mut remap: HashMap<NodeId, NodeId> = HashMap::new();
    let mut new_root = root;
    for &n in &nodes {
        let data = s.get_node(n).expect("subgraph node live").clone();
        let id = s.add_node(data);
        if n == root {
            new_root = id;
        }
        remap.insert(n, id);
    }
    for &e in &edges {
        let edge = s.get_edge(e).expect("subgraph edge live").clone();
        let ends: SmallVec<[NodeId; 8]> = edge
            .nodes
            .iter()
            .map(|x| *remap.get(x).unwrap_or(x))
            .collect();
        s.add_edge(HyperEdge::new(edge.label, ends));
    }
    new_root
}

pub fn delete_rule(s: &mut HypergraphStore, root: NodeId) {
    let (nodes, _) = rule_subgraph(s, root);
    for n in nodes {
        s.remove_node(n);
    }
}
