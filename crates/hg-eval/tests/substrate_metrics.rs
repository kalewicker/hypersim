//! S0.2 gate (scope §9 Q3, review item #5): substrate-only metrics and the
//! substrate regime are INVARIANT to the number of carrier nodes, and a carrier
//! bloom that reads as `Exploding` on the full graph does NOT on the substrate.

use hg_eval::classify;
use hg_eval::substrate::{substrate_regime, substrate_structural};
use hg_eval::window::WindowComputer;
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
use hg_schema::Regime;
use smallvec::SmallVec;

/// Matches `hg_alife::CARRIER_LABEL` (hg-eval must not depend on hg-alife).
const CARRIER: Label = Label(100);
const DEFF_SEED: u64 = 0xA11FE;

/// A ring of `n` substrate nodes (label 0). Returns the store and the ids.
fn build_ring(n: usize) -> (HypergraphStore, Vec<NodeId>) {
    let mut s = HypergraphStore::new();
    let ids: Vec<NodeId> = (0..n).map(|_| s.add_node(NodeData::new(Label(0), 0))).collect();
    for i in 0..n {
        let a = ids[i];
        let b = ids[(i + 1) % n];
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    }
    (s, ids)
}

/// Attach `k` carrier nodes (label 100), each occupying a substrate node via an
/// occupancy edge. Added AFTER all substrate nodes so substrate ordering (and
/// thus the seeded d_eff sample) is stable across carrier counts.
fn attach_carriers(s: &mut HypergraphStore, substrate: &[NodeId], k: usize) {
    for i in 0..k {
        let c = s.add_node(NodeData::new(CARRIER, 0));
        let site = substrate[i % substrate.len()];
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[site, c])));
    }
}

#[test]
fn substrate_structural_invariant_to_carrier_count() {
    let (baseline_store, _) = build_ring(40);
    let baseline = substrate_structural(&baseline_store.snapshot(), CARRIER, DEFF_SEED);
    assert_eq!(baseline.n_nodes, 40);
    assert_eq!(baseline.n_edges, 40);
    assert!(baseline.d_eff.is_some(), "ring of 40 should have a defined d_eff");

    for k in [1usize, 5, 20, 100, 400] {
        let (mut store, ids) = build_ring(40);
        attach_carriers(&mut store, &ids, k);
        let m = substrate_structural(&store.snapshot(), CARRIER, DEFF_SEED);
        assert_eq!(
            m, baseline,
            "substrate structural metrics changed with {k} carriers attached — \
the label filter is not isolating the substrate"
        );
    }
}

#[test]
fn carrier_bloom_explodes_full_graph_but_not_substrate() {
    // Window 1: ring(40) + 5 carriers = 45 full nodes.
    let (mut w1, ids1) = build_ring(40);
    attach_carriers(&mut w1, &ids1, 5);
    // Window 2: ring(40) + 35 carriers = 75 full nodes. Full growth 75/45 = 1.67
    // ≥ 1.5 ⟹ Exploding on the full graph; substrate stays at 40 ⟹ not.
    let (mut w2, ids2) = build_ring(40);
    attach_carriers(&mut w2, &ids2, 35);

    let mut full = WindowComputer::new(1);
    let mut sub = WindowComputer::new(1);

    // Prime window 1 (sets prev for growth_ratio).
    let _ = full.observe(&w1.snapshot());
    let _ = substrate_regime(&mut sub, &w1.snapshot(), CARRIER);

    // Window 2: the bloom.
    let full_regime = classify(&full.observe(&w2.snapshot()));
    let (_f, sub_regime) = substrate_regime(&mut sub, &w2.snapshot(), CARRIER);

    assert_eq!(
        full_regime,
        Regime::Exploding,
        "the carrier bloom should read as Exploding on the full (unfiltered) graph"
    );
    assert_ne!(
        sub_regime,
        Regime::Exploding,
        "the substrate regime must NOT read Exploding — the bloom is biology, not substrate growth"
    );
}

#[test]
fn substrate_regime_invariant_to_bloom() {
    // Same two-window substrate history, once with a carrier bloom in window 2
    // and once with none. The substrate regime must be identical.
    let regime_with = {
        let (mut a, ia) = build_ring(40);
        attach_carriers(&mut a, &ia, 5);
        let (mut b, ib) = build_ring(40);
        attach_carriers(&mut b, &ib, 200);
        let mut sub = WindowComputer::new(1);
        let _ = substrate_regime(&mut sub, &a.snapshot(), CARRIER);
        substrate_regime(&mut sub, &b.snapshot(), CARRIER).1
    };
    let regime_without = {
        let (a, _) = build_ring(40);
        let (b, _) = build_ring(40);
        let mut sub = WindowComputer::new(1);
        let _ = substrate_regime(&mut sub, &a.snapshot(), CARRIER);
        substrate_regime(&mut sub, &b.snapshot(), CARRIER).1
    };
    assert_eq!(
        regime_with, regime_without,
        "substrate regime differed between a bloom run and a bare run"
    );
}
