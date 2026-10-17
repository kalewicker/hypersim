//! Constructor fidelity is REAL (hypersim_investigate_replicator.md):
//! survival is instance identity (NodeIds), degraded cycles are recorded
//! (not skipped), certainty caps at Empirical, and substrate replication
//! is classified Trivial — never NonTrivial.

use hg_causal::{CausalObserver, CertaintyClass, ConstructorRegistry, ReplicatorKind};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
use smallvec::SmallVec;

/// hub(1) — buffer(2) — transformer(3): the hub's class is 2 hops from
/// the transformer, so it registers as the positioned catalyst.
fn catalyst_world() -> (HypergraphStore, NodeId, NodeId, NodeId) {
    let mut store = HypergraphStore::new();
    let hub = store.add_node(NodeData::new(Label(1), 0));
    let buffer = store.add_node(NodeData::new(Label(2), 0));
    let t = store.add_node(NodeData::new(Label(3), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[hub, buffer])));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[buffer, t])));
    (store, hub, buffer, t)
}

/// The hub's WL class hash (stable across the transformer's relabels:
/// radius-1 uses neighbor BASE hashes, and the buffer's base is its
/// unchanged label). Computed from the store so tests target the hub's
/// record specifically — the buffer's class also registers cycles.
fn hub_class(store: &HypergraphStore, hub: NodeId) -> u64 {
    hg_core::compute_wl_class_map(store, 1)[&hub].0
}

/// The plan-required `constructor_survival_uses_node_ids` test, positive
/// direction: the SAME hub node survives the cycle → confirmed.
#[test]
fn surviving_hub_confirms_cycle_by_node_id() {
    let (mut store, hub, _buffer, t) = catalyst_world();
    let cls = hub_class(&store, hub);
    let mut obs = CausalObserver::new();
    obs.process_window(&store.snapshot(), 0);
    store.get_node_mut(t).unwrap().label = Label(4); // the task fires
    obs.process_window(&store.snapshot(), 1);

    let rec = obs.constructors.get(cls).expect("hub cycle recorded");
    assert!(rec.confirmed_cycles >= 1, "surviving hub must confirm");
    assert_eq!(rec.degraded_cycles, 0);
    assert_eq!(rec.fidelity, 1.0);
}

/// The plan-required test, negative direction: the hub's node is DELETED
/// and a structurally identical node (same WL class, DIFFERENT NodeId)
/// takes its place. Same class hash present — but that is NOT the same
/// constructor surviving. Must count as DEGRADED, fidelity < 1.
#[test]
fn constructor_survival_uses_node_ids() {
    let (mut store, hub, buffer, t) = catalyst_world();
    let mut obs = CausalObserver::new();
    obs.process_window(&store.snapshot(), 0);

    // Replace the hub with a structurally identical impostor.
    let cls = hub_class(&store, hub);
    store.remove_node(hub);
    let hub2 = store.add_node(NodeData::new(Label(1), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[hub2, buffer])));
    store.get_node_mut(t).unwrap().label = Label(4); // the task fires
    obs.process_window(&store.snapshot(), 1);

    let rec = obs.constructors.get(cls).expect("hub cycle recorded");
    assert!(
        rec.degraded_cycles >= 1,
        "identical-hash impostor must be degraded, not survival"
    );
    assert_eq!(rec.confirmed_cycles, 0);
    assert_eq!(rec.fidelity, 0.0);
    assert!(!rec.is_replicator);
}

/// Measured constructors cap at Empirical — never Law, no matter how
/// many perfect cycles are observed.
#[test]
fn constructor_certainty_caps_at_empirical() {
    let (mut store, hub, _buffer, t) = catalyst_world();
    let cls = hub_class(&store, hub);
    let mut obs = CausalObserver::new();
    obs.process_window(&store.snapshot(), 0);
    for w in 1..=8u64 {
        // Alternate the transformer's label: a fresh task every window,
        // hub surviving (same NodeId) every time.
        let label = if w % 2 == 1 { Label(4) } else { Label(3) };
        store.get_node_mut(t).unwrap().label = label;
        obs.process_window(&store.snapshot(), w);
    }
    let rec = obs.constructors.get(cls).expect("hub cycle recorded");
    assert!(rec.confirmed_cycles >= 5);
    assert_eq!(rec.fidelity, 1.0, "genuine survival: fidelity legitimately 1");
    assert_eq!(
        rec.certainty,
        CertaintyClass::Empirical,
        "observed-and-measured caps at Empirical, not Law"
    );
}

/// Substrate-level replication is at most TRIVIAL; the non-trivial count
/// (the Experiment-6 baseline) is zero by construction.
#[test]
fn substrate_replication_is_trivial_never_nontrivial() {
    let mut registry = ConstructorRegistry::new();
    let pattern = hg_causal::Attribute {
        wl_hash: hg_core::WlHash(0xABCD),
        radius: 1,
        motif_class: 0,
        node_count: 3,
        certainty: CertaintyClass::Empirical,
    };
    registry.record_cycle(&pattern, 1, true, true);
    registry.record_cycle(&pattern, 1, true, true);

    let rec = registry.get(0xABCD).unwrap();
    assert!(rec.is_replicator);
    assert_eq!(rec.replicator_kind, ReplicatorKind::Trivial);
    assert_eq!(registry.replicator_count(), 1);
    assert_eq!(
        registry.non_trivial_replicator_count(),
        0,
        "bare substrate: zero non-trivial replicators, always"
    );
}
