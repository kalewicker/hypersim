//! Owned checked staging. No fallible edit ever touches the supplied graph.
use hg_core::{EdgeId, HyperEdge, HypergraphStore, Label, NodeData, NodeId, PropertyValue};
use slotmap::Key;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub enum NodeRef {
    Existing(NodeId),
    Local(u32),
}
#[derive(Clone, Debug)]
pub struct NodeObservation {
    pub node: NodeId,
    pub label: Label,
    pub properties: Vec<(u32, PropertyValue)>,
    /// Exact observation, including absence; compared without edge order.
    pub incident: Vec<EdgeId>,
}
#[derive(Clone, Debug)]
pub enum CheckedEdit {
    ReplaceProperty {
        node: NodeId,
        key: u32,
        expected: PropertyValue,
        value: PropertyValue,
    },
    AddNode {
        local: u32,
        data: NodeData,
    },
    AddEdge {
        label: Label,
        nodes: Vec<NodeRef>,
    },
    RemoveEdge(EdgeId),
    /// All incident edges must have been explicitly removed before this edit.
    RemoveNode(NodeId),
}
#[derive(Clone, Debug)]
pub struct CheckedGraphPatch {
    pub observations: Vec<NodeObservation>,
    pub edits: Vec<CheckedEdit>,
    pub max_nodes: usize,
    pub max_edges: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedStageError {
    MissingHandle,
    ObservationMismatch,
    PropertyType,
    DuplicateWrite,
    UnexpectedCleanup,
    UnresolvedReference,
    Capacity,
    GenerationExhausted,
}
pub struct StagedGraphTransition {
    graph: HypergraphStore,
}
impl StagedGraphTransition {
    pub fn graph(&self) -> &HypergraphStore {
        &self.graph
    }
    pub fn into_graph(self) -> HypergraphStore {
        self.graph
    }
}
fn same_variant(a: PropertyValue, b: PropertyValue) -> bool {
    std::mem::discriminant(&a) == std::mem::discriminant(&b)
}
pub fn stage_checked(
    before: &HypergraphStore,
    patch: &CheckedGraphPatch,
) -> Result<StagedGraphTransition, CheckedStageError> {
    use CheckedStageError::*;
    for o in &patch.observations {
        let n = before.get_node(o.node).ok_or(MissingHandle)?;
        let actual: HashSet<_> = before.edges_incident_to(o.node).iter().copied().collect();
        let expected: HashSet<_> = o.incident.iter().copied().collect();
        if n.label != o.label
            || actual != expected
            || expected.len() != o.incident.len()
            || o.properties
                .iter()
                .any(|(k, v)| n.get_property(*k) != Some(*v))
        {
            return Err(ObservationMismatch);
        }
    }
    let mut graph = before.clone();
    let mut locals = HashMap::new();
    let mut writes = HashSet::new();
    let mut removals = HashSet::new();
    for edit in &patch.edits {
        match edit {
            CheckedEdit::ReplaceProperty {
                node,
                key,
                expected,
                value,
            } => {
                if !writes.insert((*node, *key)) {
                    return Err(DuplicateWrite);
                }
                let n = graph.get_node_mut(*node).ok_or(MissingHandle)?;
                let actual = n.get_property(*key).ok_or(ObservationMismatch)?;
                if !same_variant(actual, *expected) || !same_variant(actual, *value) {
                    return Err(PropertyType);
                }
                if actual != *expected {
                    return Err(ObservationMismatch);
                }
                if n.properties.iter().filter(|(k, _)| k == key).count() != 1 {
                    return Err(ObservationMismatch);
                }
                n.set_property(*key, *value);
            }
            CheckedEdit::AddNode { local, data } => {
                if locals.contains_key(local) {
                    return Err(DuplicateWrite);
                }
                if graph.node_count() >= patch.max_nodes {
                    return Err(Capacity);
                }
                let keys: HashSet<_> = data.properties.iter().map(|(k, _)| *k).collect();
                if keys.len() != data.properties.len() {
                    return Err(DuplicateWrite);
                }
                locals.insert(*local, graph.add_node(data.clone()));
            }
            CheckedEdit::AddEdge { label, nodes } => {
                let resolved: Result<Vec<_>, _> = nodes
                    .iter()
                    .map(|r| match r {
                        NodeRef::Existing(id) => {
                            if graph.get_node(*id).is_some() {
                                Ok(*id)
                            } else {
                                Err(MissingHandle)
                            }
                        }
                        NodeRef::Local(id) => locals
                            .get(id)
                            .copied()
                            .filter(|n| graph.get_node(*n).is_some())
                            .ok_or(UnresolvedReference),
                    })
                    .collect();
                let resolved = resolved?;
                if graph.edge_count() >= patch.max_edges {
                    return Err(Capacity);
                }
                graph.add_edge(HyperEdge::new(*label, resolved.into_iter().collect()));
            }
            CheckedEdit::RemoveEdge(id) => {
                if !removals.insert(*id) {
                    return Err(DuplicateWrite);
                }
                if graph.get_edge(*id).is_none() {
                    return Err(MissingHandle);
                }
                if (id.data().as_ffi() >> 32) == u32::MAX as u64 {
                    return Err(GenerationExhausted);
                }
                graph.remove_edge(*id);
            }
            CheckedEdit::RemoveNode(id) => {
                if graph.get_node(*id).is_none() {
                    return Err(MissingHandle);
                }
                if graph.edges().any(|(_, e)| e.nodes.contains(id))
                    || !graph.edges_incident_to(*id).is_empty()
                {
                    return Err(UnexpectedCleanup);
                }
                if (id.data().as_ffi() >> 32) == u32::MAX as u64 {
                    return Err(GenerationExhausted);
                }
                graph.remove_node(*id);
            }
        }
    }
    Ok(StagedGraphTransition { graph })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn patch(edits: Vec<CheckedEdit>) -> CheckedGraphPatch {
        CheckedGraphPatch {
            observations: vec![],
            edits,
            max_nodes: 32,
            max_edges: 64,
        }
    }
    #[test]
    fn undeclared_cleanup_and_typed_writes_fail_without_live_mutation() {
        let mut g = HypergraphStore::new();
        let mut n = NodeData::new(Label(0), 0);
        n.set_property(1, PropertyValue::Bool(true));
        let a = g.add_node(n);
        let b = g.add_node(NodeData::new(Label(0), 0));
        let e = g.add_edge(HyperEdge::new(Label(0), [a, b].into_iter().collect()));
        assert_eq!(
            stage_checked(&g, &patch(vec![CheckedEdit::RemoveNode(a)])).err(),
            Some(CheckedStageError::UnexpectedCleanup)
        );
        assert_eq!(
            stage_checked(
                &g,
                &patch(vec![CheckedEdit::ReplaceProperty {
                    node: a,
                    key: 1,
                    expected: PropertyValue::Bool(true),
                    value: PropertyValue::Int(1)
                }])
            )
            .err(),
            Some(CheckedStageError::PropertyType)
        );
        assert_eq!(g.node_count(), 2);
        assert!(g.get_edge(e).is_some());
        assert_eq!(
            g.get_node(a).unwrap().get_property(1),
            Some(PropertyValue::Bool(true))
        );
        let staged = stage_checked(
            &g,
            &patch(vec![CheckedEdit::RemoveEdge(e), CheckedEdit::RemoveNode(a)]),
        )
        .unwrap();
        assert!(staged.graph().get_node(a).is_none());
        assert!(g.get_node(a).is_some());
    }
    #[test]
    fn duplicate_writes_unresolved_locals_and_stale_absence_observations_reject() {
        let mut g = HypergraphStore::new();
        let mut n = NodeData::new(Label(0), 0);
        n.set_property(1, PropertyValue::Int(0));
        let a = g.add_node(n);
        let edit = CheckedEdit::ReplaceProperty {
            node: a,
            key: 1,
            expected: PropertyValue::Int(0),
            value: PropertyValue::Int(1),
        };
        assert_eq!(
            stage_checked(&g, &patch(vec![edit.clone(), edit])).err(),
            Some(CheckedStageError::DuplicateWrite)
        );
        assert_eq!(
            stage_checked(
                &g,
                &patch(vec![CheckedEdit::AddEdge {
                    label: Label(0),
                    nodes: vec![NodeRef::Local(9)]
                }])
            )
            .err(),
            Some(CheckedStageError::UnresolvedReference)
        );
        let mut p = patch(vec![]);
        p.observations.push(NodeObservation {
            node: a,
            label: Label(0),
            properties: vec![],
            incident: vec![],
        });
        g.add_edge(HyperEdge::new(Label(0), [a].into_iter().collect()));
        assert_eq!(
            stage_checked(&g, &p).err(),
            Some(CheckedStageError::ObservationMismatch)
        );
    }
    #[test]
    fn stale_generational_endpoint_and_caps_fail_closed() {
        let mut g = HypergraphStore::new();
        let old = g.add_node(NodeData::new(Label(0), 0));
        g.remove_node(old);
        let new = g.add_node(NodeData::new(Label(0), 0));
        assert_ne!(old, new);
        assert_eq!(
            stage_checked(
                &g,
                &patch(vec![CheckedEdit::AddEdge {
                    label: Label(0),
                    nodes: vec![NodeRef::Existing(old)]
                }])
            )
            .err(),
            Some(CheckedStageError::MissingHandle)
        );
        let mut p = patch(vec![CheckedEdit::AddNode {
            local: 0,
            data: NodeData::new(Label(0), 0),
        }]);
        p.max_nodes = 1;
        assert_eq!(
            stage_checked(&g, &p).err(),
            Some(CheckedStageError::Capacity)
        );
        assert_eq!(g.node_count(), 1);
    }
}
