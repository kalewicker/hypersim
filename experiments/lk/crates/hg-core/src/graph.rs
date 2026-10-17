//! Core hypergraph store with slotmap-backed generational IDs

use crate::snapshot::GraphSnapshot;
use crate::property::PropertyValue;
use ahash::AHashMap;
use smallvec::SmallVec;
use slotmap::{SlotMap, new_key_type, SecondaryMap};
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

new_key_type! {
    /// Generational node ID - survives node removal/reuse
    pub struct NodeId;

    /// Generational edge ID
    pub struct EdgeId;
}

/// Node label (u32)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize, Default)]
pub struct Label(pub u32);

impl From<u32> for Label {
    fn from(v: u32) -> Self { Label(v) }
}

/// Inline property storage: `SmallVec<[(u32, PropertyValue); 4]>` (v4 §3).
pub type Properties = SmallVec<[(u32, PropertyValue); 4]>;

/// Node data stored in the hypergraph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeData {
    pub label: Label,
    /// The step at which this node was created — the substrate's own
    /// birth stamp, written once by every creation site and never
    /// rewritten. Read by the v8 age guard (`node_age_compares`) to give
    /// the fuse a lifetime with no per-step rule.
    ///
    /// RENAMED from `generation` in v9 C2.1, because that word already
    /// named two other things: the SlotMap slot generation counter, and
    /// property key 6 (LK filament depth). A guard written against the
    /// wrong one is the failure this rename exists to prevent — see the
    /// disambiguation section of `docs/property_registry.md`.
    ///
    /// `serde(rename = "generation")` keeps the persisted name so any
    /// name-based codec still reads archived snapshots; bincode is
    /// positional and was never affected. Pinned by
    /// `archived_snapshot_round_trips_after_birth_step_rename`.
    #[serde(rename = "generation")]
    pub birth_step: u64,
    /// Tagged at the persistence boundary: `PropertyValue` is
    /// `#[serde(untagged)]` for TOML, which bincode serializes losslessly
    /// AND cannot deserialize — the wire module writes a per-value
    /// discriminant so snapshots round-trip. See `crate::wire`.
    #[serde(with = "crate::wire::properties")]
    pub properties: Properties,
    pub energy: f32,
}

impl NodeData {
    pub fn new(label: Label, birth_step: u64) -> Self {
        Self {
            label,
            birth_step,
            properties: SmallVec::new(),
            energy: 0.0,
        }
    }

    /// Get a property value by key, if present.
    pub fn get_property(&self, key: u32) -> Option<PropertyValue> {
        self.properties.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    /// Insert or overwrite a property.
    pub fn set_property(&mut self, key: u32, value: PropertyValue) {
        if let Some(slot) = self.properties.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value;
        } else {
            self.properties.push((key, value));
        }
    }

    /// Properties sorted by key — the canonical order for hashing/compare.
    pub fn sorted_properties(&self) -> Properties {
        let mut p = self.properties.clone();
        p.sort_by_key(|(k, _)| *k);
        p
    }
}

/// Hyperedge connecting N nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HyperEdge {
    pub label: Label,
    pub nodes: SmallVec<[NodeId; 8]>,
}

impl HyperEdge {
    pub fn new(label: Label, nodes: SmallVec<[NodeId; 8]>) -> Self {
        Self { label, nodes }
    }

    pub fn arity(&self) -> usize {
        self.nodes.len()
    }
}

/// Main hypergraph store
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HypergraphStore {
    nodes: SlotMap<NodeId, NodeData>,
    edges: SlotMap<EdgeId, HyperEdge>,
    /// Incident edges for each node (secondary index)
    node_to_edges: SecondaryMap<NodeId, SmallVec<[EdgeId; 8]>>,
    /// Edge index by label
    label_index: AHashMap<Label, Vec<EdgeId>>,
    /// Current simulation step
    step: u64,
}

impl Default for HypergraphStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HypergraphStore {
    /// Opt-in allocator sidecar used by exact long-running checkpoints.
    pub fn allocator_order(&self) -> (Vec<u32>, Vec<u32>) {
        (self.nodes.vacant_slot_order(), self.edges.vacant_slot_order())
    }

    /// Restore both free lists atomically, without touching keys, versions or data.
    pub fn restore_allocator_order(&mut self, order: &(Vec<u32>, Vec<u32>)) -> Result<(), &'static str> {
        let mut nodes = self.nodes.clone();
        let mut edges = self.edges.clone();
        nodes.restore_vacant_slot_order(&order.0)?;
        edges.restore_vacant_slot_order(&order.1)?;
        self.nodes = nodes;
        self.edges = edges;
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            node_to_edges: SecondaryMap::new(),
            label_index: AHashMap::default(),
            step: 0,
        }
    }

    pub fn with_step(step: u64) -> Self {
        let mut s = Self::new();
        s.step = step;
        s
    }

    /// Current simulation step
    pub fn step(&self) -> u64 {
        self.step
    }

    /// Set simulation step
    pub fn set_step(&mut self, step: u64) {
        self.step = step;
    }

    /// Add a node, returns its ID
    pub fn add_node(&mut self, data: NodeData) -> NodeId {
        let id = self.nodes.insert(data);
        self.node_to_edges.insert(id, SmallVec::new());
        id
    }

    /// Add an edge, returns its ID
    pub fn add_edge(&mut self, edge: HyperEdge) -> EdgeId {
        let id = self.edges.insert(edge.clone());

        // Update incident edge lists
        for &node_id in &edge.nodes {
            if let Some(vec) = self.node_to_edges.get_mut(node_id) {
                vec.push(id);
            }
        }

        // Update label index
        self.label_index.entry(edge.label).or_default().push(id);

        id
    }

    /// Remove a node and all its incident edges (dangling-edge cleanup)
    pub fn remove_node(&mut self, id: NodeId) {
        let incident = self.node_to_edges.remove(id).unwrap_or_default();
        for edge_id in incident {
            self.remove_edge_internal(edge_id);
        }
        self.nodes.remove(id);
    }

    /// Internal edge removal (without touching node_to_edges again)
    fn remove_edge_internal(&mut self, id: EdgeId) {
        if let Some(edge) = self.edges.remove(id) {
            if let Some(vec) = self.label_index.get_mut(&edge.label) {
                vec.retain(|&e| e != id);
                if vec.is_empty() {
                    self.label_index.remove(&edge.label);
                }
            }
            for &node_id in &edge.nodes {
                if let Some(vec) = self.node_to_edges.get_mut(node_id) {
                    vec.retain(|e| *e != id);
                }
            }
        }
    }

    /// Remove an edge
    pub fn remove_edge(&mut self, id: EdgeId) {
        self.remove_edge_internal(id);
    }

    /// Get node data (None if stale/removed)
    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(id)
    }

    /// Get edge data (None if stale/removed)
    pub fn get_edge(&self, id: EdgeId) -> Option<&HyperEdge> {
        self.edges.get(id)
    }

    /// Get mutable node data
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(id)
    }

    /// Get mutable edge data
    pub fn get_edge_mut(&mut self, id: EdgeId) -> Option<&mut HyperEdge> {
        self.edges.get_mut(id)
    }

    /// Get edges with a specific label
    pub fn edges_with_label(&self, label: Label) -> &[EdgeId] {
        self.label_index.get(&label).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get edges incident to a node
    pub fn edges_incident_to(&self, node: NodeId) -> &[EdgeId] {
        self.node_to_edges.get(node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Iterate all nodes
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &NodeData)> {
        self.nodes.iter()
    }

    /// Iterate all edges
    pub fn edges(&self) -> impl Iterator<Item = (EdgeId, &HyperEdge)> {
        self.edges.iter()
    }

    /// Create a read-only snapshot
    pub fn snapshot(&self) -> GraphSnapshot<'_> {
        GraphSnapshot { store: self }
    }

    /// Compute the canonical hash for reproducibility testing (v4 §10).
    ///
    /// Nodes are sorted by `(label, birth_step, sorted-properties, energy)`
    /// with a WL-style refinement signature as tie-break, then assigned
    /// canonical indices `0, 1, 2, …`. Every edge's node list is remapped
    /// through this canonical index table (never through raw
    /// `NodeId::as_ffi()`, which encodes allocation order), each remapped
    /// edge's index list is sorted ascending, and the edge list is sorted
    /// lexicographically before hashing.
    ///
    /// The refinement tie-break makes the hash construction-order-
    /// independent whenever content-identical nodes are structurally
    /// distinguishable (the common case). Known residual limitation: nodes
    /// that no refinement can separate (true automorphism orbits, e.g. an
    /// unlabeled cycle) may still hash differently across insertion orders
    /// — full canonicalization is graph-isomorphism-hard and out of scope
    /// (see docs/reproducibility_policy.md). Same-construction-sequence
    /// determinism (what the engine tests rely on) is unaffected.
    pub fn canonical_hash(&self) -> u64 {
        let mut hasher = FxHasher::default();

        // Per-node content hash (label, birth_step, sorted props, energy).
        let content_hash = |node: &NodeData| -> u64 {
            let mut h = FxHasher::default();
            0x9E37_79B9_7F4A_7C15u64.hash(&mut h); // avoid the all-zeros fixed point
            node.label.0.hash(&mut h);
            node.birth_step.hash(&mut h);
            for (k, v) in node.sorted_properties() {
                k.hash(&mut h);
                v.hash_into(&mut h);
            }
            node.energy.to_bits().hash(&mut h);
            h.finish()
        };

        // WL-style refinement signatures over the whole graph: separates
        // content-tied nodes by structure so sort ties don't fall back to
        // allocation order. Rounds capped: each round can only split
        // classes, and log2-ish rounds settle all practically occurring
        // graphs.
        let mut sig: AHashMap<NodeId, u64> = self
            .nodes
            .iter()
            .map(|(id, n)| (id, content_hash(n)))
            .collect();
        let mut distinct = {
            let mut v: Vec<u64> = sig.values().copied().collect();
            v.sort_unstable();
            v.dedup();
            v.len()
        };
        for _round in 0..16 {
            if distinct == self.nodes.len() {
                break; // fully discriminated
            }
            let mut next: AHashMap<NodeId, u64> = AHashMap::default();
            for (id, _) in self.nodes.iter() {
                let mut neigh: Vec<u64> = Vec::new();
                for edge_id in self.edges_incident_to(id) {
                    if let Some(edge) = self.edges.get(*edge_id) {
                        let mut member_sigs: Vec<u64> = edge
                            .nodes
                            .iter()
                            .filter(|n| **n != id)
                            .filter_map(|n| sig.get(n).copied())
                            .collect();
                        member_sigs.sort_unstable();
                        let mut eh = FxHasher::default();
                        edge.label.0.hash(&mut eh);
                        for m in member_sigs {
                            m.hash(&mut eh);
                        }
                        neigh.push(eh.finish());
                    }
                }
                neigh.sort_unstable();
                let mut h = FxHasher::default();
                sig[&id].hash(&mut h);
                for n2 in neigh {
                    n2.hash(&mut h);
                }
                next.insert(id, h.finish());
            }
            let new_distinct = {
                let mut v: Vec<u64> = next.values().copied().collect();
                v.sort_unstable();
                v.dedup();
                v.len()
            };
            sig = next;
            if new_distinct == distinct {
                break; // refinement stabilized
            }
            distinct = new_distinct;
        }

        // 1. Canonical node ordering — precomputed sort keys (one
        //    allocation per node, not one per comparison).
        struct Key<'a> {
            label: Label,
            birth_step: u64,
            props: Properties,
            energy_bits: u32,
            sig: u64,
            id: NodeId,
            node: &'a NodeData,
        }
        let mut keys: Vec<Key> = self
            .nodes
            .iter()
            .map(|(id, node)| Key {
                label: node.label,
                birth_step: node.birth_step,
                props: node.sorted_properties(),
                energy_bits: node.energy.to_bits(),
                sig: sig.get(&id).copied().unwrap_or(0),
                id,
                node,
            })
            .collect();
        keys.sort_by(|a, b| {
            a.label
                .cmp(&b.label)
                .then_with(|| a.birth_step.cmp(&b.birth_step))
                .then_with(|| prop_list_cmp(&a.props, &b.props))
                .then_with(|| a.energy_bits.cmp(&b.energy_bits))
                .then_with(|| a.sig.cmp(&b.sig))
        });

        // 2. Assign canonical indices and hash the node sequence.
        let mut index_of: AHashMap<NodeId, u64> = AHashMap::default();
        for (canon, key) in keys.iter().enumerate() {
            index_of.insert(key.id, canon as u64);
            key.node.label.0.hash(&mut hasher);
            key.node.birth_step.hash(&mut hasher);
            for (k, v) in &key.props {
                k.hash(&mut hasher);
                v.hash_into(&mut hasher);
            }
            key.energy_bits.hash(&mut hasher);
        }

        // 3. Remap edges through canonical indices; sort endpoints; sort
        //    the whole edge list lexicographically.
        let mut remapped: Vec<(Vec<u64>, u32)> = Vec::with_capacity(self.edges.len());
        for (_id, edge) in self.edges.iter() {
            let mut idxs: Vec<u64> = edge
                .nodes
                .iter()
                .filter_map(|n| index_of.get(n).copied())
                .collect();
            idxs.sort_unstable();
            remapped.push((idxs, edge.label.0));
        }
        remapped.sort();

        // 4. Hash the canonical edge sequence.
        for (idxs, label) in &remapped {
            label.hash(&mut hasher);
            idxs.len().hash(&mut hasher);
            for i in idxs {
                i.hash(&mut hasher);
            }
        }

        hasher.finish()
    }
}

/// Total order on property lists (PropertyValue has no Ord; compare by
/// variant tag then bit pattern — deterministic, allocation-independent).
fn prop_list_cmp(a: &Properties, b: &Properties) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    for (pa, pb) in a.iter().zip(b.iter()) {
        let o = pa.0.cmp(&pb.0).then_with(|| prop_val_cmp(&pa.1, &pb.1));
        if o != Ordering::Equal {
            return o;
        }
    }
    a.len().cmp(&b.len())
}

fn prop_val_cmp(a: &PropertyValue, b: &PropertyValue) -> std::cmp::Ordering {
    fn tag(v: &PropertyValue) -> u8 {
        match v {
            PropertyValue::Int(_) => 0,
            PropertyValue::Float(_) => 1,
            PropertyValue::Bool(_) => 2,
        }
    }
    tag(a).cmp(&tag(b)).then_with(|| match (a, b) {
        (PropertyValue::Int(x), PropertyValue::Int(y)) => x.cmp(y),
        (PropertyValue::Float(x), PropertyValue::Float(y)) => x.to_bits().cmp(&y.to_bits()),
        (PropertyValue::Bool(x), PropertyValue::Bool(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    })
}
