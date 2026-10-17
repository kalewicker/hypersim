//! WL hash computation for attribute identity

use crate::graph::{HypergraphStore, NodeId};
use crate::snapshot::GraphSnapshot;
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
use smallvec::SmallVec;
use ahash::AHashMap;

/// WL hash wrapper (u64)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize)]
pub struct WlHash(pub u64);

impl WlHash {
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }
}

/// Deterministic nonzero seed folded into every WL hasher (see base_hash).
const WL_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// Compute WL hash for a node at given radius.
/// Uses FxHasher (deterministic) and sorts neighbor hashes before combining.
/// Never panics on missing/stale nodes (returns WlHash(0)).
pub fn compute_wl_hash(store: &HypergraphStore, node: NodeId, radius: u8) -> WlHash {
    // Base hash for a node: label + energy + structure. The properties map is
    // DELIBERATELY EXCLUDED — WL identity is structural/state, never per-key
    // property content. This is the ALife Q2 invariant (docs/property_registry.md,
    // scope §9 Q2): continuous, per-individual genome traits stored as properties
    // must NOT mint a fresh WL class per carrier, or the class registry explodes
    // and EI_micro = log2(M). Enforced by `wl_hash_invariant_under_property_change`.
    // Generation is likewise excluded — a glider rebuilt from new nodes keeps its
    // hash. Energy is the `f32` state field (also in canonical_hash and the EI
    // buckets); ALife energy lives in property key 1, which — being a property —
    // is excluded here, so carriers keep a label-stable WL class.
    fn base_hash(store: &HypergraphStore, id: NodeId) -> Option<u64> {
        let node_data = store.get_node(id)?;
        let mut hasher = FxHasher::default();
        // Nonzero seed: FxHasher maps an all-zero input stream to 0, and 0 is
        // its fixed point — without this, every label-0/energy-0 node would
        // collapse to WlHash(0) and refinement could never separate them.
        WL_SEED.hash(&mut hasher);
        node_data.label.0.hash(&mut hasher);
        node_data.energy.to_bits().hash(&mut hasher);
        Some(hasher.finish())
    }

    if store.get_node(node).is_none() {
        return WlHash(0);
    }

    // Collect the radius-r ball around `node` so each refinement round has
    // the neighbor hashes it needs.
    let mut ball: Vec<NodeId> = vec![node];
    let mut seen: AHashMap<NodeId, ()> = AHashMap::default();
    seen.insert(node, ());
    let mut frontier = vec![node];
    for _ in 0..radius {
        let mut next_frontier = Vec::new();
        for &nid in &frontier {
            for edge_id in store.edges_incident_to(nid) {
                if let Some(edge) = store.get_edge(*edge_id) {
                    for &neighbor in &edge.nodes {
                        if !seen.contains_key(&neighbor) && store.get_node(neighbor).is_some() {
                            seen.insert(neighbor, ());
                            ball.push(neighbor);
                            next_frontier.push(neighbor);
                        }
                    }
                }
            }
        }
        frontier = next_frontier;
    }

    let mut current_hashes: AHashMap<NodeId, u64> = AHashMap::default();
    for &nid in &ball {
        if let Some(h) = base_hash(store, nid) {
            current_hashes.insert(nid, h);
        }
    }

    // Iterate radius times, refining every node in the ball.
    for _ in 0..radius {
        let mut next_hashes: AHashMap<NodeId, u64> = AHashMap::default();

        for &nid in &ball {
            let mut neighbor_hashes: SmallVec<[u64; 8]> = SmallVec::new();
            for edge_id in store.edges_incident_to(nid) {
                if let Some(edge) = store.get_edge(*edge_id) {
                    for &neighbor in &edge.nodes {
                        if neighbor != nid {
                            if let Some(&h) = current_hashes.get(&neighbor) {
                                neighbor_hashes.push(h);
                            }
                        }
                    }
                }
            }
            // CRITICAL: sort neighbor hashes before combining for determinism
            neighbor_hashes.sort_unstable();

            let mut hasher = FxHasher::default();
            WL_SEED.hash(&mut hasher);
            if let Some(&prev_hash) = current_hashes.get(&nid) {
                prev_hash.hash(&mut hasher);
            }
            for h in neighbor_hashes {
                h.hash(&mut hasher);
            }
            next_hashes.insert(nid, hasher.finish());
        }

        current_hashes = next_hashes;
    }

    WlHash(current_hashes.get(&node).copied().unwrap_or(0))
}

/// Compute WL hash from a GraphSnapshot (read-only)
pub fn compute_wl_hash_snapshot(snapshot: &GraphSnapshot<'_>, node: NodeId, radius: u8) -> WlHash {
    compute_wl_hash(snapshot.store, node, radius)
}

/// Compute the WL hash of EVERY live node in one synchronized refinement
/// pass — bit-identical to calling `compute_wl_hash(store, v, radius)` per
/// node (property-tested), but O(radius · E) total instead of one ball
/// walk per node, and computed once for all consumers. The evaluator and
/// the causal layer both need the full per-node class map every window;
/// this is the shared computation (review finding: they each recomputed
/// it independently).
pub fn compute_wl_class_map_snapshot(
    snapshot: &GraphSnapshot<'_>,
    radius: u8,
) -> AHashMap<NodeId, WlHash> {
    compute_wl_class_map(snapshot.store, radius)
}

pub fn compute_wl_class_map(
    store: &HypergraphStore,
    radius: u8,
) -> AHashMap<NodeId, WlHash> {
    // Round 0: base hashes (same recipe as compute_wl_hash::base_hash —
    // label + energy + structure, properties map excluded; see that fn).
    let mut current: AHashMap<NodeId, u64> = AHashMap::default();
    for (id, node_data) in store.nodes() {
        let mut hasher = FxHasher::default();
        WL_SEED.hash(&mut hasher);
        node_data.label.0.hash(&mut hasher);
        node_data.energy.to_bits().hash(&mut hasher);
        current.insert(id, hasher.finish());
    }

    // Refinement rounds: identical combine step to compute_wl_hash.
    for _ in 0..radius {
        let mut next: AHashMap<NodeId, u64> = AHashMap::default();
        for (id, _) in store.nodes() {
            let mut neighbor_hashes: SmallVec<[u64; 8]> = SmallVec::new();
            for edge_id in store.edges_incident_to(id) {
                if let Some(edge) = store.get_edge(*edge_id) {
                    for &neighbor in &edge.nodes {
                        if neighbor != id {
                            if let Some(&h) = current.get(&neighbor) {
                                neighbor_hashes.push(h);
                            }
                        }
                    }
                }
            }
            neighbor_hashes.sort_unstable();
            let mut hasher = FxHasher::default();
            WL_SEED.hash(&mut hasher);
            if let Some(&prev) = current.get(&id) {
                prev.hash(&mut hasher);
            }
            for h in neighbor_hashes {
                h.hash(&mut hasher);
            }
            next.insert(id, hasher.finish());
        }
        current = next;
    }

    current.into_iter().map(|(id, h)| (id, WlHash(h))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{HyperEdge, HypergraphStore, Label, NodeData};
    use crate::property::PropertyValue;
    use smallvec::SmallVec;

    /// Build a two-node, one-edge fixture and return (store, [n0, n1]).
    fn fixture() -> (HypergraphStore, [NodeId; 2]) {
        let mut store = HypergraphStore::new();
        let n0 = store.add_node(NodeData::new(Label(1), 0));
        let n1 = store.add_node(NodeData::new(Label(2), 0));
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[n0, n1])));
        (store, [n0, n1])
    }

    /// The ALife Q2 invariant (scope §9 Q2, docs/property_registry.md): a
    /// node's WL hash depends only on label + energy + structure, NEVER on the
    /// properties map. Setting ANY property — energy (key 1), a genome trait
    /// (key 11), a marker (key 20) — must leave every node's WL hash and the
    /// full class map bit-identical. This is a positive invariant: it holds for
    /// every key, present and future, so it cannot rot as new property keys are
    /// added. If it ever fails, continuous per-individual genomes mint a fresh
    /// WL class per carrier and the class registry explodes (EI_micro = log2 M).
    #[test]
    fn wl_hash_invariant_under_property_change() {
        for radius in [1u8, 2, 3] {
            let (mut store, [n0, n1]) = fixture();

            let before_single: Vec<WlHash> =
                [n0, n1].iter().map(|&n| compute_wl_hash(&store, n, radius)).collect();
            let before_map = compute_wl_class_map(&store, radius);

            // Mutate properties on both nodes: energy, a genome trait, a marker.
            for &n in &[n0, n1] {
                let d = store.get_node_mut(n).unwrap();
                d.set_property(1, PropertyValue::Float(42.5));
                d.set_property(11, PropertyValue::Float(0.137));
                d.set_property(20, PropertyValue::Bool(true));
            }

            let after_single: Vec<WlHash> =
                [n0, n1].iter().map(|&n| compute_wl_hash(&store, n, radius)).collect();
            let after_map = compute_wl_class_map(&store, radius);

            assert_eq!(
                before_single, after_single,
                "compute_wl_hash changed after a property write (radius {radius})"
            );
            assert_eq!(
                before_map, after_map,
                "compute_wl_class_map changed after a property write (radius {radius})"
            );
        }
    }

    /// The energy `f32` field, unlike the properties map, DOES participate in WL
    /// identity (it is also in canonical_hash and the EI energy buckets). This
    /// pins the intended asymmetry so a future "just exclude everything" edit
    /// trips a test rather than silently moving substrate baselines.
    #[test]
    fn wl_hash_still_sees_energy_field() {
        let (mut store, [n0, _n1]) = fixture();
        let before = compute_wl_hash(&store, n0, 2);
        store.get_node_mut(n0).unwrap().energy = 7.0;
        let after = compute_wl_hash(&store, n0, 2);
        assert_ne!(before, after, "energy field must remain part of WL identity");
    }

    /// The two WL entry points (per-node ball walk vs. one synchronized pass)
    /// must agree after the properties-excluded change, just as before it.
    #[test]
    fn wl_single_and_map_agree() {
        let (store, [n0, n1]) = fixture();
        let map = compute_wl_class_map(&store, 2);
        for &n in &[n0, n1] {
            assert_eq!(compute_wl_hash(&store, n, 2), map[&n]);
        }
    }
}
