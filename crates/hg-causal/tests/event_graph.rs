//! Event-graph builder mechanism tests (LK2): parent attribution,
//! cleanup-channel tagging, dynamics-beats-cleanup, and JSONL round-trip.

use hg_causal::{CausalEventObserver, EventGraphLog};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
use hg_observe::{Binding, Observer, RewriteEvent, StepStats};
use smallvec::SmallVec;

fn event(
    step_unused: u64,
    bound: &[hg_core::NodeId],
    created: &[hg_core::NodeId],
    cleanup: &[hg_core::NodeId],
) -> RewriteEvent {
    let _ = step_unused;
    RewriteEvent {
        rule_id: 0,
        rule_name: "t".into(),
        binding: Binding::new(0, 0, SmallVec::from_slice(bound)),
        nodes_created: created.to_vec(),
        nodes_deleted: Vec::new(),
        edges_created: Vec::new(),
        edges_deleted: Vec::new(),
        cleanup_neighbors: cleanup.to_vec(),
        vanished_energy: 0.0,
    }
}

fn stats(step: u64) -> StepStats {
    StepStats {
        step,
        ..Default::default()
    }
}

#[test]
fn cleanup_channel_is_tagged_separately() {
    let mut store = HypergraphStore::new();
    let x = store.add_node(NodeData::new(Label(0), 0));
    let y = store.add_node(NodeData::new(Label(0), 0));
    let z = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[x, y])));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[y, z])));

    let (mut obs, log) = CausalEventObserver::new();
    let snapshot = store.snapshot();

    // Step 1, event 0: binds y, cleanup-affects x.
    obs.on_step_end(&snapshot, &stats(1), &[event(1, &[y], &[], &[x])]);
    // Step 2: event 1 binds x (cleanup parentage), event 2 binds y
    // (dynamics parentage).
    obs.on_step_end(
        &snapshot,
        &stats(2),
        &[event(2, &[x], &[], &[]), event(2, &[y], &[], &[])],
    );

    let log = log.lock().unwrap();
    assert_eq!(log.records.len(), 3);
    assert!(log.records[0].parents.is_empty());
    assert_eq!(log.records[1].parents, vec![(0, true)], "via cleanup");
    assert_eq!(log.records[2].parents, vec![(0, false)], "via dynamics");
}

#[test]
fn dynamics_beats_cleanup_on_the_same_node() {
    // Event 0 BINDS x and also cleanup-affects x (a bound node that lost
    // an unrelated incident edge): the dynamics affect must win, so a
    // reader of x is a dynamics child.
    let mut store = HypergraphStore::new();
    let x = store.add_node(NodeData::new(Label(0), 0));
    let (mut obs, log) = CausalEventObserver::new();
    let snapshot = store.snapshot();
    obs.on_step_end(&snapshot, &stats(1), &[event(1, &[x], &[], &[x])]);
    obs.on_step_end(&snapshot, &stats(2), &[event(2, &[x], &[], &[])]);
    let log = log.lock().unwrap();
    assert_eq!(log.records[1].parents, vec![(0, false)]);
}

#[test]
fn last_writer_wins_and_jsonl_round_trips() {
    let mut store = HypergraphStore::new();
    let x = store.add_node(NodeData::new(Label(0), 0));
    let (mut obs, log) = CausalEventObserver::new();
    let snapshot = store.snapshot();
    obs.on_step_end(&snapshot, &stats(1), &[event(1, &[x], &[], &[])]); // 0
    obs.on_step_end(&snapshot, &stats(2), &[event(2, &[x], &[], &[])]); // 1: parent 0
    obs.on_step_end(&snapshot, &stats(3), &[event(3, &[x], &[], &[])]); // 2: parent 1, NOT 0
    let jsonl = log.lock().unwrap().to_jsonl();
    let back = EventGraphLog::from_jsonl(&jsonl).unwrap();
    assert_eq!(back.records.len(), 3);
    assert_eq!(back.records[2].parents, vec![(1, false)], "most recent affector only");
    assert_eq!(back.records[2].step, 3);
}
