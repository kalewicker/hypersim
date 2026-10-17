//! LK3 ID-permutation control: the damage metric must be label-invariant
//! by construction — a store and its NodeId-permuted rebuild must show
//! ZERO damage under the anchor mapping. Nonzero ⇒ the metric leaks
//! NodeIds and every twin result is invalid (pre-committed).

use hg_eval::fixtures::locality::{lattice_2d_tri, small_world};
use hg_eval::twin::{anchor_profile, permute_node_ids, sample_anchors};

#[test]
fn id_permutation_zero_damage() {
    for (name, store) in [
        ("lattice", lattice_2d_tri(16)),
        ("small_world", small_world(16, 0.02, 9)),
    ] {
        let (permuted, map) = permute_node_ids(&store, 1234);
        assert_eq!(store.node_count(), permuted.node_count());
        assert_eq!(store.edge_count(), permuted.edge_count());
        let snap_a = store.snapshot();
        let snap_b = permuted.snapshot();
        let anchors = sample_anchors(&snap_a, 64, 42);
        for &a in &anchors {
            let pa = anchor_profile(&snap_a, a, 3).expect("anchor alive");
            let pb = anchor_profile(&snap_b, map[&a], 3).expect("mapped anchor alive");
            assert_eq!(pa, pb, "{name}: damage metric is not label-invariant");
        }
    }
}

#[test]
fn metric_detects_real_structural_change() {
    // Sanity counterpart: an actual structural difference near an anchor
    // must register (the metric is not vacuously blind).
    let store = lattice_2d_tri(16);
    let mut changed = store.clone();
    let anchor = store.nodes().next().map(|(id, _)| id).unwrap();
    // Delete one edge incident to the anchor in the changed copy.
    let edge = changed.edges_incident_to(anchor)[0];
    changed.remove_edge(edge);
    let pa = anchor_profile(&store.snapshot(), anchor, 3).unwrap();
    let pb = anchor_profile(&changed.snapshot(), anchor, 3).unwrap();
    assert_ne!(pa, pb, "metric must see a deleted incident edge");
}
