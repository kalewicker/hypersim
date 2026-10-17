//! LK1R exit criteria (plan v2 §4): on a hand-built lattice+1-shortcut
//! fixture with scripted events, every tracked.csv column matches
//! hand-computed values. Plus lineage closure, component invariance
//! under refinement of a single handle, purity, and determinism.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_eval::fixtures::locality::lattice_2d_tri;
use hg_observe::tracked::TrackedEdgeObserver;
use hg_observe::{Binding, RewriteEvent};
use smallvec::SmallVec;

const R_MAX: usize = 12;

/// 32×32 torus (diameter 16 > cap: registrable) + one shortcut between
/// (0,0) and (16,16) — lattice distance 16, so the shortcut's exact
/// detour is 16 > 12: one wormhole, one handle.
fn fixture() -> (HypergraphStore, hg_core::EdgeId, hg_core::NodeId, hg_core::NodeId) {
    let mut store = lattice_2d_tri(32);
    let ids: Vec<hg_core::NodeId> = store.nodes().map(|(id, _)| id).collect();
    let at = |x: usize, y: usize| ids[y * 32 + x];
    let (a, b) = (at(0, 0), at(16, 16));
    let shortcut = store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    (store, shortcut, a, b)
}

fn subdivision_event(
    store: &mut HypergraphStore,
    edge: hg_core::EdgeId,
) -> (RewriteEvent, [hg_core::EdgeId; 2], hg_core::NodeId) {
    // Perform a real subdivision on the store and fabricate the matching
    // RewriteEvent (scripted events per the exit criteria).
    let endpoints = store.get_edge(edge).expect("edge exists").nodes.clone();
    let (a, b) = (endpoints[0], endpoints[1]);
    store.remove_edge(edge);
    let w = store.add_node(NodeData::new(Label(0), store.step()));
    let e1 = store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, w])));
    let e2 = store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[w, b])));
    let event = RewriteEvent {
        rule_id: 0,
        rule_name: "lk_metered_subdivide".into(),
        binding: Binding::with_edges(
            0,
            0,
            SmallVec::from_slice(&[a, b]),
            SmallVec::from_slice(&[edge]),
        ),
        nodes_created: vec![w],
        nodes_deleted: vec![],
        edges_created: vec![e1, e2],
        edges_deleted: vec![edge],
        cleanup_neighbors: vec![],
        vanished_energy: 0.0,
    };
    (event, [e1, e2], w)
}

#[test]
fn hand_fixture_columns_match_hand_computed_values() {
    let (mut store, shortcut, _a, _b) = fixture();
    let mut obs = TrackedEdgeObserver::new(vec![shortcut], R_MAX, vec![]);

    // Window 0: the un-subdivided planted shortcut.
    let row = obs.measure(&store.snapshot(), 0, 0);
    assert_eq!(row.tracked_edge_count, 1, "one planted shortcut");
    assert_eq!(row.tracked_components, 1, "one handle");
    assert_eq!(row.tracked_wormhole_count, 1, "detour 16 > 12: wormhole");
    assert!(row.tracked_detour_p50.is_infinite());
    assert!(row.tracked_detour_p95.is_infinite());
    assert_eq!(row.throat_length_max, 1, "single tracked edge: path length 1");
    assert!(!row.far_nucleus_flag, "no seed sites: probe disabled");

    // Scripted subdivision of the tracked edge (lineage closure).
    let (event, [e1, e2], _w) = subdivision_event(&mut store, shortcut);
    obs.ingest_event(&event);
    assert_eq!(obs.tracked_count(), 2, "both replacement edges join the lineage");

    // Window 1: one handle refined into a 2-path.
    let row = obs.measure(&store.snapshot(), 1, 10);
    assert_eq!(row.tracked_edge_count, 2);
    assert_eq!(
        row.tracked_components, 1,
        "refinement of a single handle must NOT mint a second component"
    );
    // Both half-edges are wormholes: removing {a,w} leaves a↔w only via
    // b (lattice distance 16 + 1 hop), > cap.
    assert_eq!(row.tracked_wormhole_count, 2);
    assert_eq!(row.throat_length_max, 2, "a–w–b in tracked hops");

    // Subdivide one half again: still one component, throat grows.
    let (event2, _, _) = subdivision_event(&mut store, e1);
    obs.ingest_event(&event2);
    let row = obs.measure(&store.snapshot(), 2, 20);
    assert_eq!(row.tracked_edge_count, 3);
    assert_eq!(row.tracked_components, 1, "still one handle after re-refinement");
    assert_eq!(row.throat_length_max, 3);
    let _ = e2;
}

#[test]
fn untracked_events_do_not_join_lineage() {
    let (mut store, shortcut, _, _) = fixture();
    let mut obs = TrackedEdgeObserver::new(vec![shortcut], R_MAX, vec![]);
    // Subdivide a LATTICE edge (untracked): lineage must not grow.
    let lattice_edge = store
        .edges()
        .map(|(id, _)| id)
        .find(|&id| id != shortcut)
        .unwrap();
    let (event, _, _) = subdivision_event(&mut store, lattice_edge);
    obs.ingest_event(&event);
    assert_eq!(obs.tracked_count(), 1, "untracked consumption must not leak in");
}

#[test]
fn tracked_edge_deleted_without_replacement_leaves_lineage() {
    let (store, shortcut, _, _) = fixture();
    let mut obs = TrackedEdgeObserver::new(vec![shortcut], R_MAX, vec![]);
    let event = RewriteEvent {
        rule_id: 0,
        rule_name: "delete".into(),
        binding: Binding::new(0, 0, SmallVec::new()),
        nodes_created: vec![],
        nodes_deleted: vec![],
        edges_created: vec![],
        edges_deleted: vec![shortcut],
        cleanup_neighbors: vec![],
        vanished_energy: 0.0,
    };
    obs.ingest_event(&event);
    assert_eq!(obs.tracked_count(), 0);
    let row = obs.measure(&store.snapshot(), 0, 0);
    assert_eq!(row.tracked_edge_count, 0);
    assert_eq!(row.tracked_components, 0);
    assert!(row.tracked_detour_p50.is_nan(), "empty lineage: NaN percentiles");
}

#[test]
fn observer_purity_and_determinism() {
    let (mut store, shortcut, _, _) = fixture();
    let (event, _, _) = subdivision_event(&mut store, shortcut);

    let hash_before = store.canonical_hash();
    let mut a = TrackedEdgeObserver::new(vec![shortcut], R_MAX, vec![]);
    let mut b = TrackedEdgeObserver::new(vec![shortcut], R_MAX, vec![]);
    for obs in [&mut a, &mut b] {
        obs.ingest_event(&event);
        obs.measure(&store.snapshot(), 0, 5);
        obs.measure(&store.snapshot(), 1, 10);
    }
    assert_eq!(store.canonical_hash(), hash_before, "observer mutated the store");
    assert_eq!(a.rows, b.rows, "identical inputs must yield identical rows");
}
