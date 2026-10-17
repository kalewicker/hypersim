//! d_eff estimator sanity (fix doc §2.4): a 1D ring fits ≈1, a 2D grid
//! fits ≈2, tiny graphs legitimately return None, and the seeded
//! sampling is reproducible.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_eval::effective_dimension;
use smallvec::SmallVec;

fn ring(n: usize) -> HypergraphStore {
    let mut store = HypergraphStore::new();
    let ids: Vec<_> = (0..n)
        .map(|_| store.add_node(NodeData::new(Label(0), 0)))
        .collect();
    for i in 0..n {
        store.add_edge(HyperEdge::new(
            Label(0),
            SmallVec::from_slice(&[ids[i], ids[(i + 1) % n]]),
        ));
    }
    store
}

#[test]
fn ring_is_one_dimensional() {
    let store = ring(400);
    let d = effective_dimension(&store.snapshot(), 42).expect("ring should fit");
    assert!((0.7..=1.3).contains(&d), "ring d_eff = {d}, expected ≈ 1");
}

#[test]
fn grid_is_two_dimensional() {
    let mut store = HypergraphStore::new();
    hg_core::generate_crystal(&mut store, 40, 40, 2, false, 42);
    let d = effective_dimension(&store.snapshot(), 42).expect("grid should fit");
    assert!((1.5..=2.5).contains(&d), "grid d_eff = {d}, expected ≈ 2");
}

#[test]
fn ring_and_grid_discriminate() {
    let ring_d = effective_dimension(&ring(400).snapshot(), 42).unwrap();
    let mut store = HypergraphStore::new();
    hg_core::generate_crystal(&mut store, 40, 40, 2, false, 42);
    let grid_d = effective_dimension(&store.snapshot(), 42).unwrap();
    assert!(
        grid_d > ring_d + 0.5,
        "grid ({grid_d}) should be clearly higher-dimensional than ring ({ring_d})"
    );
}

#[test]
fn tiny_graph_returns_none() {
    let store = ring(4); // below the minimum node floor
    assert_eq!(effective_dimension(&store.snapshot(), 42), None);
}

#[test]
fn same_seed_same_estimate() {
    let store = ring(300);
    let a = effective_dimension(&store.snapshot(), 7);
    let b = effective_dimension(&store.snapshot(), 7);
    assert_eq!(a, b, "same seed must reproduce bit-identically");
}

#[test]
fn disconnected_fragments_still_estimate() {
    // Two separate rings: sources estimate from their reachable ball;
    // global connectivity must NOT be required.
    let mut store = ring(200);
    let ids: Vec<_> = (0..200)
        .map(|_| store.add_node(NodeData::new(Label(1), 0)))
        .collect();
    for i in 0..200 {
        store.add_edge(HyperEdge::new(
            Label(0),
            SmallVec::from_slice(&[ids[i], ids[(i + 1) % 200]]),
        ));
    }
    let d = effective_dimension(&store.snapshot(), 42).expect("disconnected should still fit");
    assert!((0.7..=1.3).contains(&d), "two rings d_eff = {d}, expected ≈ 1");
}
