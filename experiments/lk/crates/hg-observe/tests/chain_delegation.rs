//! Phase 3 exit: ObserverChain delegates to N members in push order,
//! verified for all three lifecycle methods together, not each in
//! isolation.

use hg_core::HypergraphStore;
use hg_observe::{Binding, Observer, ObserverChain, RewriteEvent, StepStats};
use smallvec::SmallVec;
use std::sync::{Arc, Mutex};

struct Recorder {
    id: usize,
    log: Arc<Mutex<Vec<(usize, &'static str)>>>,
}

impl Observer for Recorder {
    fn on_step_end(
        &mut self,
        _s: &hg_core::GraphSnapshot<'_>,
        _st: &StepStats,
        _e: &[RewriteEvent],
    ) {
        self.log.lock().unwrap().push((self.id, "step_end"));
    }
    fn on_window_complete(&mut self, _s: &hg_core::GraphSnapshot<'_>, _w: u64) {
        self.log.lock().unwrap().push((self.id, "window"));
    }
    fn on_run_end(&mut self, _s: &hg_core::GraphSnapshot<'_>) {
        self.log.lock().unwrap().push((self.id, "run_end"));
    }
}

#[test]
fn chain_delegates_in_push_order_across_all_three_methods() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut chain = ObserverChain::new();
    for id in 0..3 {
        chain.add(Box::new(Recorder {
            id,
            log: log.clone(),
        }));
    }

    let store = HypergraphStore::new();
    let snapshot = store.snapshot();
    let stats = StepStats::new();
    let events = vec![RewriteEvent {
        rule_id: 0,
        rule_name: "r".into(),
        binding: Binding::new(0, 0, SmallVec::new()),
        nodes_created: vec![],
        nodes_deleted: vec![],
        edges_created: vec![],
        edges_deleted: vec![],
        cleanup_neighbors: vec![],
        vanished_energy: 0.0,
    }];

    // Interleave all three lifecycle methods, as the engine would.
    chain.on_step_end(&snapshot, &stats, &events);
    chain.on_window_complete(&snapshot, 0);
    chain.on_step_end(&snapshot, &stats, &[]);
    chain.on_run_end(&snapshot);

    let got = log.lock().unwrap().clone();
    let expected: Vec<(usize, &str)> = vec![
        (0, "step_end"), (1, "step_end"), (2, "step_end"),
        (0, "window"), (1, "window"), (2, "window"),
        (0, "step_end"), (1, "step_end"), (2, "step_end"),
        (0, "run_end"), (1, "run_end"), (2, "run_end"),
    ];
    assert_eq!(got, expected);
}

#[test]
fn creation_binding_is_trivial_and_conflict_free_shaped() {
    let b = Binding::new(3, 7, SmallVec::new());
    assert!(b.is_creation());
    assert!(b.node_ids_sorted().is_empty());
    let key = b.canonical_key();
    assert_eq!(key.0, std::cmp::Reverse(7));
    assert_eq!(key.1, 3);
}
