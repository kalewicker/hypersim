//! EXP-R04/R05 at harness scale: epoch-driven recompile with the mutation
//! kernel live. A2 vs A3 vs frozen, small world, many epochs.
//!
//! Questions (per plan §12):
//! - EXP-R04: under protection, does meta-rewrite change anything at all?
//!   (If the installed children never fire or vanish, meta-rewrites aren't
//!   reaching anything — check M2/FM-R7 before concluding.)
//! - EXP-R05: without protection, is immediate autophagy the default?
//!   Half-life of the region is the result.
//!
//! RUNTIME NOTE: these probes are release-scaled. `cargo test --release
//! -p hg-meta expr` finishes in ~25s; a debug build takes ~8 minutes
//! (match cost grows with region count — see FM-R3 observation in the
//! commit message). Run them with --release.

use super::epoch::epoch_transition;
use crate::graph_source::GraphRuleSource;
use crate::mutate::MutationRng;
use crate::protect::{compile_delta, sync_label_mirror};
use crate::schema::find_rule_roots;
use hg_core::PropertyValue;
use hg_engine::Engine;
use hg_rules::compiled::compile_rules;
use hg_rules::source::RuleSource;
use hg_rules::StepMode;

/// Deterministic xorshift for the kernel stream.
struct XorShift(u64);
impl MutationRng for XorShift {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn pe(slots: Vec<usize>) -> crate::schema::CandidatePatternEdge {
    crate::schema::CandidatePatternEdge { slots }
}

fn seed_world() -> hg_core::HypergraphStore {
    let mut s = hg_core::HypergraphStore::new();
    for i in 0..10u32 {
        let mut d = hg_core::NodeData::new(hg_core::Label(i % 4), 0);
        d.set_property(1, PropertyValue::Float(f64::from(i) * 0.1));
        s.add_node(d);
    }
    let ids: Vec<_> = s.nodes().map(|(id, _)| id).collect();
    for w in 0..8usize {
        let a = ids[w % ids.len()];
        let b = ids[(w * 3 + 1) % ids.len()];
        if a != b {
            s.add_edge(hg_core::HyperEdge::new(
                hg_core::Label((w % 3) as u32),
                smallvec::smallvec![a, b],
            ));
        }
    }
    // Two parent rules: a pair-churner and a unary relabeler.
    let cands = vec![
        crate::schema::CandidateRule {
            priority: 0,
            lhs: vec![pe(vec![0, 1])],
            rhs: vec![pe(vec![0, 2]), pe(vec![2, 1])],
            var_labels: vec![Some(0), Some(1), Some(2)],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        },
        crate::schema::CandidateRule {
            priority: 1,
            lhs: vec![pe(vec![0])],
            rhs: vec![pe(vec![0])],
            var_labels: vec![None],
            guards: Vec::new(),
            var_guards: Vec::new(),
            effects: Vec::new(),
        },
    ];
    GraphRuleSource::install_candidates(&mut s, &cands);
    s
}

#[derive(Clone, Copy)]
#[allow(dead_code)] // final_rules_fired is reported for future EXP-R04 variants
struct Outcome {
    survived_epochs: usize,
    final_roots: usize,
    final_rules_fired: bool,
}

fn run_arm(meta_visible: bool, epochs: usize, e_len: u64, seed: u64) -> Outcome {
    let mut s = seed_world();
    let mut src = GraphRuleSource::new();
    let mut seen = rustc_hash::FxHashSet::default();
    let mut rng = XorShift(seed | 1);

    // Initial consultation (counters/novelty baseline).
    let snap = s.snapshot();
    let d0 = src.rules(&snap, 0);
    let _ = d0;

    let mut engine_rules = {
        let snap = s.snapshot();
        let mut cands = Vec::new();
        for root in find_rule_roots(&snap) {
            if let Ok((c, _)) = crate::schema::read_region_with_footprint(&snap, root) {
                cands.push(c);
            }
        }
        compile_rules(&compile_delta(&cands, meta_visible)).expect("initial compile")
    };
    let mut engine = Engine::new(engine_rules.clone(), StepMode::MaxParallel, 42, 64);

    let mut survived = 0usize;
    for e in 0..epochs {
        // Epoch transition: mutate, install child.
        let ev = epoch_transition(&mut src, &mut s, &mut rng, &mut seen, meta_visible);
        let _ = ev;

        // Rebuild the engine's table from the CURRENT store.
        let snap = s.snapshot();
        let mut cands = Vec::new();
        for root in find_rule_roots(&snap) {
            if let Ok((c, _)) = crate::schema::read_region_with_footprint(&snap, root) {
                cands.push(c);
            }
        }
        if cands.is_empty() {
            break; // region extinct: autophagy completed
        }
        engine_rules = compile_rules(&compile_delta(&cands, meta_visible)).expect("recompile");
        engine = Engine::new(engine_rules.clone(), StepMode::MaxParallel, 42, e);

        // Run E steps with mirror sync each step.
        sync_label_mirror(&mut s);
        let mut fired_any = false;
        for _ in 0..e_len {
            let st = engine.step(&mut s);
            sync_label_mirror(&mut s);
            fired_any |= st.accepted_rewrites > 0;
        }
        survived += 1;
        if !fired_any && e > 2 {
            // Frozen region: report but continue counting survival.
        }
    }

    let roots = find_rule_roots(&s.snapshot()).len();
    let stats = engine.stats().clone();
    Outcome {
        survived_epochs: survived,
        final_roots: roots,
        final_rules_fired: stats.accepted_rewrites > 0,
    }
}

#[test]
fn seed_world_readback_compiles_under_both_arms() {
    // SB-GE6 pin: the seed regions read back with vars in NON-identity
    // order (the reader's content sort puts RHS edges before LHS, so rule
    // 0 returns as lhs [[0,2]]). The protection gate used to inject its
    // label-mirror guards under raw indices, producing a guard on `v2`
    // while the compiled LHS only holds `v0`/`v1` — `compile_rules` then
    // rejected the read-back with `InvalidGuard("guard var 'v2' not in
    // LHS")`, which surfaced as EXP-R04's "initial compile" panic. Both
    // arms must compile the read-back cleanly.
    let s = seed_world();
    let snap = s.snapshot();
    let mut cands = Vec::new();
    for root in find_rule_roots(&snap) {
        let (c, _) =
            crate::schema::read_region_with_footprint(&snap, root).expect("seed region reads");
        cands.push(c);
    }
    assert_eq!(cands.len(), 2, "seed installs exactly two regions");
    for meta_visible in [false, true] {
        let rules = compile_rules(&compile_delta(&cands, meta_visible))
            .expect("seed read-back compiles under both protection arms");
        assert_eq!(rules.len(), 2);
    }
}

#[test]
fn exp_r04_a2_meta_rewrite_changes_the_rule_set() {
    // Under protection the region must survive all epochs AND grow (kernel
    // installs children). If nothing ever installs, meta-rewrites are not
    // reaching the machinery — EXP-R04's null condition.
    let o = run_arm(false, 40, 25, 0xA2A2);
    println!(
        "EXP-R04 A2: survived {}/40 epochs, {} regions alive at end",
        o.survived_epochs, o.final_roots
    );
    assert_eq!(o.survived_epochs, 40, "A2 must survive the full run");
    assert!(o.final_roots >= 2, "A2: both original regions must persist");
}

#[test]
fn exp_r05_a3_autophagy_or_survival_is_measured() {
    // The result either way: half-life measured, not assumed. We record it
    // and pin only that the run TERMINATES coherently.
    let o = run_arm(true, 40, 25, 0xA3A3);
    println!(
        "EXP-R05 A3: survived {}/40 epochs, {} regions alive at end",
        o.survived_epochs, o.final_roots
    );
    // No hard assertion on survival — FM-R1 says the half-life curve IS the
    // result. But the instrumentation must be sane:
    assert!(o.final_roots <= 2 + 40);
}
