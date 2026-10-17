//! Restoration check (plan v5 §2, Appendix B): is the terminal state of
//! a relaxation run STRUCTURALLY the clean lattice?
//!
//! The comparison is a whole-graph WL hash, label-invariant by
//! construction: NodeIds never enter (signatures are structural),
//! properties/generation/energy are DELIBERATELY excluded — the
//! restored lattice is energy-DEAD (every joule ledgered or parked as a
//! sub-θ remainder on the seed), and the claim is about the geometry,
//! not the bookkeeping. Per the v5 §1.1 handle-identity principle this
//! is an assertion against the topological object.
//!
//! Not full isomorphism (WL-hard cases exist), but for
//! lattice-vs-lattice-with-leftovers any surviving chain node changes
//! the node count and the refined signature multiset.

use hg_core::{HypergraphStore, NodeId};
use rustc_hash::FxHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const WL_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// Whole-graph structural WL hash: per-node signatures seeded from the
/// node LABEL only, refined through (edge label + member signatures)
/// until stabilization (≤ 16 rounds, same discipline as
/// `canonical_hash`), then the sorted signature multiset + node/edge
/// counts are hashed. Equal graphs-up-to-NodeId-permutation hash equal;
/// property content never participates.
pub fn structural_wl_hash(store: &HypergraphStore) -> u64 {
    let mut sig: HashMap<NodeId, u64> = store
        .nodes()
        .map(|(id, n)| {
            let mut h = FxHasher::default();
            WL_SEED.hash(&mut h);
            n.label.0.hash(&mut h);
            (id, h.finish())
        })
        .collect();
    let mut distinct = {
        let mut v: Vec<u64> = sig.values().copied().collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    };
    for _ in 0..16 {
        if distinct == store.node_count() {
            break;
        }
        let mut next: HashMap<NodeId, u64> = HashMap::default();
        for (id, _) in store.nodes() {
            let mut neigh: Vec<u64> = Vec::new();
            for edge_id in store.edges_incident_to(id) {
                if let Some(edge) = store.get_edge(*edge_id) {
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
            break;
        }
        distinct = new_distinct;
    }
    let mut all: Vec<u64> = sig.values().copied().collect();
    all.sort_unstable();
    let mut h = FxHasher::default();
    WL_SEED.hash(&mut h);
    store.node_count().hash(&mut h);
    store.edge_count().hash(&mut h);
    for s in all {
        s.hash(&mut h);
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_core::{HyperEdge, Label, NodeData, PropertyValue};
    use smallvec::smallvec;

    /// NodeId-permutation invariance + property blindness + structural
    /// sensitivity, on small hand graphs.
    #[test]
    fn structural_hash_invariants() {
        // Triangle built in two insertion orders, one with junk props.
        let tri = |order: [usize; 3], props: bool| {
            let mut s = HypergraphStore::new();
            let mut ids = vec![];
            for i in 0..3 {
                let mut d = NodeData::new(Label(0), 0);
                if props {
                    d.set_property(1, PropertyValue::Float(42.0 + i as f64));
                    d.set_property(6, PropertyValue::Float(i as f64));
                }
                ids.push(s.add_node(d));
            }
            for w in [[0usize, 1], [1, 2], [2, 0]] {
                s.add_edge(HyperEdge::new(
                    Label(0),
                    smallvec![ids[order[w[0]]], ids[order[w[1]]]],
                ));
            }
            s
        };
        let a = structural_wl_hash(&tri([0, 1, 2], false));
        let b = structural_wl_hash(&tri([2, 0, 1], true));
        assert_eq!(a, b, "permutation + property invariance");

        // Path of 3 is structurally different from a triangle.
        let mut path = HypergraphStore::new();
        let ids: Vec<_> = (0..3)
            .map(|_| path.add_node(NodeData::new(Label(0), 0)))
            .collect();
        path.add_edge(HyperEdge::new(Label(0), smallvec![ids[0], ids[1]]));
        path.add_edge(HyperEdge::new(Label(0), smallvec![ids[1], ids[2]]));
        assert_ne!(a, structural_wl_hash(&path), "structure sensitivity");
    }

    /// The R1 use case in miniature: fixture equality at two widths.
    #[test]
    fn clean_lattice_fixture_self_equality() {
        let a = crate::fixtures::locality::small_world_with_shortcuts(8, 0.0, 42).0;
        let b = crate::fixtures::locality::small_world_with_shortcuts(8, 0.0, 99).0;
        let c = crate::fixtures::locality::small_world_with_shortcuts(10, 0.0, 42).0;
        assert_eq!(
            structural_wl_hash(&a),
            structural_wl_hash(&b),
            "clean lattice is seed-independent"
        );
        assert_ne!(structural_wl_hash(&a), structural_wl_hash(&c));
    }
}
