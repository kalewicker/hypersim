//! Label-invariant damage metric for twin runs (LK3, plan §6).
//!
//! NodeId allocation order diverges globally downstream of a perturbation
//! even where structure is identical, so raw edge-set diffs are invalid
//! BY DEFINITION (plan §11.1 — same failure class as the
//! replicator-fidelity NodeId bug). The metric compares radius-r_a ball
//! WL-profiles instead: the sorted multiset of radius-1 WL hashes of
//! every node within graph distance r_a of the anchor. WL hashes are
//! label/structure/energy only — never NodeIds — so the profile is
//! label-invariant by construction (pinned by the ID-permutation test).
//!
//! This module is engine-free; the run harness that drives the two
//! branches lives in `hg_sim_core::twin` (v4 §2.3: hg-sim-core is the
//! only crate that imports hg-engine — a documented placement deviation
//! from the LK plan's hg-eval/src/twin/mod.rs).

use ahash::AHashMap;
use hg_core::{compute_wl_hash_snapshot, GraphSnapshot, HypergraphStore, NodeId};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::VecDeque;

/// Nodes within graph distance `radius` of `anchor` (hyperedges as
/// endpoint cliques), or None if the anchor is not alive.
fn ball_members(
    snapshot: &GraphSnapshot<'_>,
    anchor: NodeId,
    radius: usize,
) -> Option<Vec<NodeId>> {
    snapshot.get_node(anchor)?;
    let mut dist: AHashMap<NodeId, usize> = AHashMap::new();
    dist.insert(anchor, 0);
    let mut queue: VecDeque<NodeId> = VecDeque::new();
    queue.push_back(anchor);
    while let Some(v) = queue.pop_front() {
        let d = dist[&v];
        if d >= radius {
            continue;
        }
        for edge_id in snapshot.edges_incident_to(v) {
            if let Some(edge) = snapshot.get_edge(*edge_id) {
                for &nb in &edge.nodes {
                    if !dist.contains_key(&nb) && snapshot.get_node(nb).is_some() {
                        dist.insert(nb, d + 1);
                        queue.push_back(nb);
                    }
                }
            }
        }
    }
    Some(dist.into_keys().collect())
}

/// The anchor's radius-`radius` ball WL-profile: sorted multiset of
/// radius-1 WL hashes over the ball's members. `None` = anchor dead.
pub fn anchor_profile(
    snapshot: &GraphSnapshot<'_>,
    anchor: NodeId,
    radius: usize,
) -> Option<Vec<u64>> {
    let members = ball_members(snapshot, anchor, radius)?;
    let mut profile: Vec<u64> = members
        .iter()
        .map(|&n| compute_wl_hash_snapshot(snapshot, n, 1).0)
        .collect();
    profile.sort_unstable();
    Some(profile)
}

/// Seeded anchor sample over the snapshot's stable node order (the
/// d_eff sampling protocol).
pub fn sample_anchors(snapshot: &GraphSnapshot<'_>, n: usize, seed: u64) -> Vec<NodeId> {
    let ids: Vec<NodeId> = snapshot.node_ids().collect();
    let k = n.min(ids.len());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    rand::seq::index::sample(&mut rng, ids.len(), k)
        .iter()
        .map(|ix| ids[ix])
        .collect()
}

/// Graph distances from a set of site nodes (multi-source BFS over the
/// whole component; used once, at branch time, for pre-perturbation
/// distances).
pub fn distances_from(
    snapshot: &GraphSnapshot<'_>,
    sites: &[NodeId],
) -> AHashMap<NodeId, u64> {
    let mut dist: AHashMap<NodeId, u64> = AHashMap::new();
    let mut queue: VecDeque<NodeId> = VecDeque::new();
    for &s in sites {
        if snapshot.get_node(s).is_some() {
            dist.insert(s, 0);
            queue.push_back(s);
        }
    }
    while let Some(v) = queue.pop_front() {
        let d = dist[&v];
        for edge_id in snapshot.edges_incident_to(v) {
            if let Some(edge) = snapshot.get_edge(*edge_id) {
                for &nb in &edge.nodes {
                    if !dist.contains_key(&nb) && snapshot.get_node(nb).is_some() {
                        dist.insert(nb, d + 1);
                        queue.push_back(nb);
                    }
                }
            }
        }
    }
    dist
}

/// Rebuild a store with freshly allocated (hence permuted) NodeIds,
/// returning the old→new mapping. Structure, labels, energies,
/// properties, and edge order are preserved; only identities change.
/// The ID-permutation validity control: any damage metric worth trusting
/// must see ZERO damage between a store and its permutation.
pub fn permute_node_ids(
    store: &HypergraphStore,
    seed: u64,
) -> (HypergraphStore, AHashMap<NodeId, NodeId>) {
    use rand::seq::SliceRandom;
    let mut out = HypergraphStore::with_step(store.step());
    let mut order: Vec<(NodeId, hg_core::NodeData)> =
        store.nodes().map(|(id, d)| (id, d.clone())).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    order.shuffle(&mut rng);
    let mut map: AHashMap<NodeId, NodeId> = AHashMap::new();
    for (old_id, data) in order {
        let new_id = out.add_node(data);
        map.insert(old_id, new_id);
    }
    for (_, edge) in store.edges() {
        let nodes: smallvec::SmallVec<[NodeId; 8]> =
            edge.nodes.iter().map(|n| map[n]).collect();
        out.add_edge(hg_core::HyperEdge::new(edge.label, nodes));
    }
    (out, map)
}
