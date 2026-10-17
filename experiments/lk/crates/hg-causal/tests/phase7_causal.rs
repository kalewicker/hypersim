//! Phase 7 (hg-causal) exit criteria: a sustained run produces a non-empty
//! task lattice and at least one constructor candidate; knowledge graph is
//! certainty-tagged; EI computes on the class-transition matrix.

use hg_causal::{CausalObserver, CertaintyClass, Predicate, TaskLattice, Attribute};
use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId, WlHash};
use smallvec::SmallVec;

/// A world with a persistent catalyst: hub(1) — buffers(2) — transformers
/// (3<->4 alternating each window). The hub's radius-1 class is stable
/// (its direct neighbors keep label 2) while transformers two hops away
/// keep changing class — the constructor signature.
fn evolving_world(windows: usize) -> (Vec<HypergraphStore>, ()) {
    let mut store = HypergraphStore::new();
    let hub = store.add_node(NodeData::new(Label(1), 0));
    let mut transformers: Vec<NodeId> = Vec::new();
    for _ in 0..3 {
        let buffer = store.add_node(NodeData::new(Label(2), 0));
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[hub, buffer])));
        let t = store.add_node(NodeData::new(Label(3), 0));
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[buffer, t])));
        transformers.push(t);
    }
    let mut stores = vec![store.clone()];
    for w in 1..windows {
        let new_label = if w % 2 == 0 { Label(3) } else { Label(4) };
        for &t in &transformers {
            store.get_node_mut(t).unwrap().label = new_label;
        }
        stores.push(store.clone());
    }
    (stores, ())
}

#[test]
fn task_lattice_and_constructor_emerge() {
    let (stores, _) = evolving_world(12);
    let mut obs = CausalObserver::new().with_ei_settings(hg_causal::EiSettings {
        interval_steps: 1,
        min_transitions: 1,
        cadence_steps: 1,
        ..hg_causal::EiSettings::default()
    });
    for (w, s) in stores.iter().enumerate() {
        obs.process_window(&s.snapshot(), w as u64);
    }
    assert!(obs.lattice.task_count() >= 1, "non-empty task lattice");
    assert!(obs.lattice.depth() >= 1);
    assert!(obs.constructors.count() >= 1, "at least one constructor candidate");

    // The hub persisted while satellites transformed every window: after
    // >= 3 confirmed cycles it must be Empirical or better.
    let best = obs
        .constructors
        .all_sorted()
        .iter()
        .map(|c| c.certainty)
        .max()
        .unwrap();
    assert!(best >= CertaintyClass::Empirical);

    // Knowledge graph is certainty-tagged and non-empty.
    assert!(obs.knowledge.claim_count() > 0);
    assert!(obs
        .knowledge
        .claims()
        .any(|c| c.predicate == Predicate::TransformsInto));
    assert!(obs
        .knowledge
        .claims()
        .any(|c| c.certainty == CertaintyClass::Empirical));

    // EI over a 2-class alternation is defined and positive once enough
    // transitions accumulate (deterministic system => high EI).
    assert!(!obs.latest_ei.is_empty());
    assert!(obs.latest_ei[0].ei_micro > 0.0);
}

#[test]
fn task_witnessing_confirms_at_three() {
    let mut lattice = TaskLattice::new();
    let a = Attribute {
        wl_hash: WlHash(1),
        radius: 1,
        motif_class: 0,
        node_count: 1,
        certainty: CertaintyClass::Empirical,
    };
    let b = Attribute {
        wl_hash: WlHash(2),
        ..a.clone()
    };
    lattice.witness(&a, &b, 0);
    lattice.witness(&a, &b, 1);
    assert!(lattice.tasks().all(|t| t.witness_count == 2));
    lattice.witness(&a, &b, 2);
    let t = lattice.tasks().next().unwrap();
    assert_eq!(t.witness_count, 3);
    assert_eq!(t.status, hg_causal::TaskStatus::Confirmed);
    assert_eq!(t.certainty, CertaintyClass::Empirical);
    assert_eq!(lattice.task_count(), 1);
}

#[test]
fn lattice_depth_follows_chained_tasks() {
    let mut lattice = TaskLattice::new();
    let attr = |h: u64| Attribute {
        wl_hash: WlHash(h),
        radius: 1,
        motif_class: 0,
        node_count: 1,
        certainty: CertaintyClass::Empirical,
    };
    lattice.witness(&attr(1), &attr(2), 0);
    lattice.witness(&attr(2), &attr(3), 1);
    lattice.witness(&attr(3), &attr(4), 2);
    assert_eq!(lattice.task_count(), 3);
    assert_eq!(lattice.depth(), 3);
}

#[test]
fn knowledge_graph_detects_functional_conflicts() {
    let mut kg = hg_causal::KnowledgeGraph::new();
    kg.add_claim("x".into(), Predicate::TransformsInto, "y".into(), CertaintyClass::Empirical, 0);
    kg.add_claim("x".into(), Predicate::TransformsInto, "z".into(), CertaintyClass::Empirical, 1);
    assert_eq!(kg.conflicts().len(), 1);
    // Multi-valued predicates never conflict.
    kg.add_claim("x".into(), Predicate::PersistsAcross, "w1".into(), CertaintyClass::Empirical, 1);
    kg.add_claim("x".into(), Predicate::PersistsAcross, "w2".into(), CertaintyClass::Empirical, 2);
    assert_eq!(kg.conflicts().len(), 1);
}
