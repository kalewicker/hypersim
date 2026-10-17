//! G-R1 unit tests: kernel totality, exclusive classification, determinism,
//! and the seed-set smoke battery (small-scale; the 10^4-sample gate run
//! lives in the campaign harness, not `cargo test`).

use crate::gr1::{classify_one, Class};
use crate::mutate::{mutate, MutationRng};
use crate::schema::{CandidatePatternEdge, CandidateRule};

/// Deterministic xorshift RNG for tests — the kernel is RNG-agnostic.
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

fn triangle() -> CandidateRule {
    CandidateRule {
        priority: 0,
        lhs: vec![CandidatePatternEdge {
            slots: vec![0, 1, 2],
        }],
        rhs: vec![
            CandidatePatternEdge {
                slots: vec![0, 1, 3],
            },
            CandidatePatternEdge {
                slots: vec![1, 2, 3],
            },
            CandidatePatternEdge {
                slots: vec![0, 2, 3],
            },
        ],
        var_labels: vec![None; 4],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

#[test]
fn kernel_mutations_are_total_and_deterministic() {
    let parent = triangle();
    for seed in 1..50u64 {
        let mut r1 = XorShift(seed);
        let mut r2 = XorShift(seed);
        let (a, ap1) = mutate(&parent, &mut r1);
        let (b, ap2) = mutate(&parent, &mut r2);
        assert_eq!(ap1, ap2);
        assert_eq!(a, b, "same seed must reproduce the same child");
    }
}

#[test]
fn remove_var_keeps_candidate_consistent() {
    let parent = triangle(); // vars 0..3, d=3 appears only on RHS
                             // Force a remove-var by retrying seeds until one lands (deterministic).
    let mut found = false;
    for seed in 1..500u64 {
        let mut rng = XorShift(seed ^ 0xDEAD);
        let (child, applied) = mutate(&parent, &mut rng);
        if applied != crate::mutate::Applied::Edited {
            continue;
        }
        let n0 = parent.n_vars();
        let n1 = child.n_vars();
        if n1 == n0 - 1 {
            found = true;
            // Every remaining slot index must be < n1.
            for e in child.lhs.iter().chain(child.rhs.iter()) {
                for s in &e.slots {
                    assert!(*s < n1, "dangling slot after remove_var");
                }
            }
            // Label vector length tracks var count.
            assert_eq!(child.var_labels.len(), n1);
            break;
        }
    }
    assert!(found, "remove_var sampled within 500 seeds");
}

#[test]
fn classification_is_exclusive_and_totals_match() {
    let parent = triangle();
    let mut rng = XorShift(0xC0FFEE);
    let mut c = crate::gr1::ClassCounts::default();
    for _ in 0..200 {
        let (_, counts) = classify_one(&parent, &mut rng);
        // classify_one returns per-sample counts; accumulate manually.
        c.nothing += counts.nothing;
        c.quarantined += counts.quarantined;
        c.neutral += counts.neutral;
        c.valid_distinct += counts.valid_distinct;
    }
    assert_eq!(c.total(), 200);
    // Rates are in [0,1] and consistent with buckets.
    assert!(c.valid_rate() >= 0.0 && c.valid_rate() <= 1.0);
    assert!((c.valid_rate() - (c.neutral + c.valid_distinct) as f64 / 200.0).abs() < 1e-12);
}

#[test]
fn seed_set_smoke_battery() {
    // Small-scale G-R1: 5 seed rules x 200 mutations. Not the gate run —
    // this pins that the harness RUNS, classifies exclusively, and produces
    // nonzero installable children on healthy seeds.
    let seeds = [triangle(), labeled_pair(), chain_rule()];
    let mut any_distinct = false;
    for (si, parent) in seeds.iter().enumerate() {
        let mut rng = XorShift(1000 + si as u64);
        let mut c = crate::gr1::ClassCounts::default();
        for _ in 0..200 {
            let (_, counts) = classify_one(parent, &mut rng);
            c.nothing += counts.nothing;
            c.quarantined += counts.quarantined;
            c.neutral += counts.neutral;
            c.valid_distinct += counts.valid_distinct;
        }
        assert!(
            c.valid_rate() > 0.0,
            "seed {si} produced zero valid children"
        );
        any_distinct |= c.valid_distinct > 0;
    }
    assert!(
        any_distinct,
        "no seed produced any RCF-distinct valid child"
    );
}

fn labeled_pair() -> CandidateRule {
    CandidateRule {
        priority: 2,
        lhs: vec![CandidatePatternEdge { slots: vec![0, 1] }],
        rhs: vec![CandidatePatternEdge { slots: vec![0] }],
        var_labels: vec![Some(3), None],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

fn chain_rule() -> CandidateRule {
    CandidateRule {
        priority: -1,
        lhs: vec![
            CandidatePatternEdge { slots: vec![0, 1] },
            CandidatePatternEdge { slots: vec![1, 2] },
        ],
        rhs: vec![CandidatePatternEdge { slots: vec![0, 2] }],
        var_labels: vec![None, None, None],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

#[test]
fn nothing_class_fires_on_degenerate_parents() {
    // A parent with a single unary LHS edge and empty RHS: remove_pattern_edge
    // can refuse (would empty a side), and several ops have narrow ranges.
    let parent = CandidateRule {
        priority: 0,
        lhs: vec![CandidatePatternEdge { slots: vec![0] }],
        rhs: vec![],
        var_labels: vec![None],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    };
    let mut rng = XorShift(7);
    let mut nothings = 0;
    for _ in 0..50 {
        let (class, _) = classify_one(&parent, &mut rng);
        if class == Class::Nothing {
            nothings += 1;
        }
    }
    assert!(
        nothings > 0,
        "degenerate parent should produce some Nothings"
    );
}
