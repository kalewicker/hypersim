//! G-R3 identity harness (plan §6): A0 (static table) vs A1
//! (compiled-from-graph, frozen at step 0) must be byte-identical.
//!
//! The driver: a deterministic step loop over `hg_engine::Engine` that
//! consults a `Box<dyn RuleSource>` at epoch boundaries. A0 consults
//! `StaticRuleSource` (returns its table, Never due — consulted once at
//! step 0 by construction); A1 consults `GraphRuleSource` over a world
//! store pre-seeded with the SAME rules as E1 regions, then freezes
//! (consults once, ignores thereafter).
//!
//! Identity assertion: identical `canonical_hash` at every step AND
//! identical final StepStats. This is the strongest test in the track and
//! nearly free — if it fails, the interpreter is wrong and every result
//! downstream is noise.

use crate::compile::candidate_to_rule;
use crate::graph_source::GraphRuleSource;
use crate::schema::CandidateRule;
use hg_core::{HypergraphStore, Label, NodeData, PropertyValue};
use hg_engine::Engine;
use hg_rules::compiled::compile_rules;
use hg_rules::source::{RuleSource, StaticRuleSource};
use hg_rules::StepMode;
use smallvec::smallvec;

/// Deterministic world: 8 nodes, 6 binary edges, labels 0..3, energy prop.
/// No RNG — a fixed lattice so both arms seed identically by construction.
pub fn seed_world_fixed(n_nodes: usize) -> HypergraphStore {
    let mut s = HypergraphStore::new();
    for i in 0..n_nodes {
        let mut d = NodeData::new(Label((i % 4) as u32), 0);
        d.set_property(1, PropertyValue::Float((i as f64) * 0.125));
        s.add_node(d);
    }
    let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
    if ids.is_empty() {
        return s;
    }
    for w in 0..6usize {
        let a = ids[w % ids.len()];
        let b = ids[(w * 3 + 1) % ids.len()];
        if a != b {
            s.add_edge(hg_core::HyperEdge::new(
                Label((w % 3) as u32),
                smallvec![a, b],
            ));
        }
    }
    s
}

/// The reference rule set: two rules over the fixed world, BOTH fully
/// inside E1's envelope.
/// R1: match (label0,label1) pair, create label2 node, wire a-c-b.
/// R2: match (label2,label1) pair, delete the label2 side. PURE DELETE —
/// the original draft relabeled the survivor, which is OUTSIDE E1
/// (SB-R5: no survivor-relabels; a both-sides var's label is a match
/// constraint, not a write) and its stranded label-lit vanished in the
/// graph round-trip because unwired/unreferenced vars are invisible to
/// the reader (vars exist iff referenced by ≥1 SLOT edge).
pub fn reference_candidates() -> Vec<CandidateRule> {
    vec![
        CandidateRule {
            priority: 0,
            lhs: vec![crate::schema::CandidatePatternEdge { slots: vec![0, 1] }],
            rhs: vec![
                crate::schema::CandidatePatternEdge { slots: vec![0, 2] },
                crate::schema::CandidatePatternEdge { slots: vec![2, 1] },
            ],
            var_labels: vec![Some(0), Some(1), Some(2)],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        },
        CandidateRule {
            priority: 1,
            lhs: vec![crate::schema::CandidatePatternEdge { slots: vec![0, 1] }],
            rhs: vec![crate::schema::CandidatePatternEdge { slots: vec![1] }],
            var_labels: vec![Some(2), Some(1)],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        },
    ]
}

/// One arm's trajectory: (hashes per step, final stats).
struct Trajectory {
    hashes: Vec<u64>,
    final_attempted: usize,
    final_accepted: usize,
}

fn run_arm(boxed: &mut dyn RuleSource, steps: usize) -> Trajectory {
    // BOTH arms step the same world: substrate + rule regions co-resident
    // (the ERE thesis). Identity isolates ONLY the rule-table origin.
    let mut store = seed_world_fixed(8);
    GraphRuleSource::install_candidates(&mut store, &reference_candidates());

    // Consult once at step 0 (both arms' construction), compile, freeze.
    let snap = store.snapshot();
    let delta = boxed.rules(&snap, 0);
    let compiled = compile_rules(&delta.rules).expect("reference rules compile");
    assert_eq!(compiled.len(), 2, "arm must install both reference rules");
    let mut engine = Engine::new(compiled, StepMode::MaxParallel, 42, 64);

    let mut hashes = Vec::with_capacity(steps);
    for step in 0u64..steps as u64 {
        // Epoch boundary consultation (before the step): A0's source returns
        // Never-due so this is a no-op after construction for it; A1's is
        // frozen by the harness (consult only at step 0).
        if step > 0 && boxed.epoch_len().due(step) {
            let snap = store.snapshot();
            let d = boxed.rules(&snap, step);
            let _ = d; // A2/A3 will reinstall here; identity arms freeze.
        }
        engine.step(&mut store);
        hashes.push(store.canonical_hash());
    }
    let st = engine.stats().clone();
    Trajectory {
        hashes,
        final_attempted: st.attempted_matches,
        final_accepted: st.accepted_rewrites,
    }
}

#[test]
fn gr3_a0_static_vs_a1_graph_frozen_byte_identical() {
    let cands = reference_candidates();
    let table: Vec<hg_rules::rule::Rule> = cands.iter().map(candidate_to_rule).collect();

    // A0: static table.
    let mut a0 = StaticRuleSource(table.clone());
    // A1: graph-compiled. Seed a SEPARATE store with the regions (the
    // world-under-rewriting is a different store; the source reads the
    // rule-region store, the engine steps the world store — the R3
    // separation that A2/A3 will dissolve).
    let mut region_store = seed_world_fixed(0);
    GraphRuleSource::install_candidates(&mut region_store, &cands);
    let mut a1 = GraphRuleSource::new();
    {
        let snap = region_store.snapshot();
        let d = a1.rules(&snap, 0);
        assert_eq!(d.rules.len(), 2, "both reference rules install");
        assert_eq!(d.quarantined, 0);
        assert_eq!(d.read_rejected, 0);
    }

    const STEPS: usize = 2000;
    let t0 = run_arm(&mut a0, STEPS);
    let t1 = run_arm(&mut a1, STEPS);

    assert_eq!(
        t0.hashes, t1.hashes,
        "canonical_hash diverged between A0 and A1"
    );
    assert_eq!(
        t0.final_attempted, t1.final_attempted,
        "match counts diverged"
    );
    assert_eq!(
        t0.final_accepted, t1.final_accepted,
        "rewrite counts diverged"
    );
}

#[test]
fn gr3_a1_region_round_trip_is_the_table() {
    // The compiled-from-graph rule set must equal the static table rule set
    // BEHAVIORALLY. Raw vN names are NOT preserved by the graph round-trip:
    // the reader interns vars in first-encounter order over canonically
    // sorted pattern edges, which need not match the writer's creation
    // order. Name equality is therefore not the contract — content equality
    // under RCF (which is invariant to exactly this renaming) plus
    // per-rule structural equality modulo renaming is.
    let cands = reference_candidates();
    let table: Vec<hg_rules::rule::Rule> = cands.iter().map(candidate_to_rule).collect();

    let mut region_store = seed_world_fixed(0);
    GraphRuleSource::install_candidates(&mut region_store, &cands);
    let mut a1 = GraphRuleSource::new();
    let snap = region_store.snapshot();
    let d = a1.rules(&snap, 0);

    assert_eq!(d.rules.len(), table.len());

    // Canonical naming (same interning discipline on both paths) makes the
    // Rules directly comparable — no renaming quotient needed. The strongest
    // identity claim available at the Rule level:

    for (g, t) in d.rules.iter().zip(table.iter()) {
        // Canonical naming (same interning discipline on both paths) makes
        // the Rules directly comparable — the strongest identity claim:
        assert_eq!(g.priority, t.priority);
        assert_eq!(g.lhs.nodes, t.lhs.nodes, "var names must match exactly");
        assert_eq!(g.lhs.edges, t.lhs.edges);
        assert_eq!(g.rhs.nodes, t.rhs.nodes);
        assert_eq!(g.rhs.edges, t.rhs.edges);
        let mut gl: Vec<(String, u32)> = g
            .lhs
            .node_labels
            .iter()
            .map(|(k, v)| (k.clone(), v.0))
            .collect();
        let mut tl: Vec<(String, u32)> = t
            .lhs
            .node_labels
            .iter()
            .map(|(k, v)| (k.clone(), v.0))
            .collect();
        gl.sort();
        tl.sort();
        assert_eq!(gl, tl, "LHS label constraints must match exactly");
        let mut gr: Vec<(String, u32)> = g
            .rhs_node_labels
            .iter()
            .map(|(k, v)| (k.clone(), v.0))
            .collect();
        let mut tr: Vec<(String, u32)> = t
            .rhs_node_labels
            .iter()
            .map(|(k, v)| (k.clone(), v.0))
            .collect();
        gr.sort();
        tr.sort();
        assert_eq!(gr, tr, "created-node labels must match exactly");
    }
}
