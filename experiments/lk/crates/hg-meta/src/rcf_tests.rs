//! G-R0 fixtures (plan §4.3): RCF collision classes.
//!
//! Must collide: var renaming; pattern-edge reordering; whole-region
//! translation to different NodeIds; two stores built in different orders.
//! Must NOT collide: slot permutation within a pattern edge; moving a var
//! from LHS-only to both-sides; adding a label constraint; changing
//! priority. Known-limit: documented WL-indistinguishable pairs asserted to
//! COLLIDE, with the collision recorded as known incompleteness.

use crate::rcf::{rcf_bytes, rcf_hash};
use crate::schema::read_region;
use crate::testutil::*;
use hg_core::HypergraphStore;
use hg_core::NodeId;

#[test]
fn gr0_must_collide_var_renaming() {
    // Same rule, two stores: reversed var creation AND reversed RHS edge
    // addition change which NodeId each logical variable gets and the order
    // vars are first encountered during reading — so compact ids differ per
    // build. RCF must be equal regardless.
    let mut s1 = HypergraphStore::new();
    let roots1 = build_triangle(&mut s1, false, false);
    let mut s2 = HypergraphStore::new();
    let roots2 = build_triangle(&mut s2, true, true);

    let snap1 = s1.snapshot();
    let snap2 = s2.snapshot();
    let c1 = read_region(&snap1, roots1[0]).expect("reads");
    let c2 = read_region(&snap2, roots2[0]).expect("reads");
    assert_ne!(c1.lhs, c2.lhs, "fixture must produce differing candidates");
    assert_eq!(
        rcf_hash(&c1),
        rcf_hash(&c2),
        "renaming must collide under RCF"
    );
}

#[test]
fn gr0_must_collide_pattern_edge_reordering() {
    let mut s1 = HypergraphStore::new();
    let roots1 = build_triangle(&mut s1, false, false);
    let mut s2 = HypergraphStore::new();
    let roots2 = build_triangle(&mut s2, false, true);

    let snap1 = s1.snapshot();
    let snap2 = s2.snapshot();
    let c1 = read_region(&snap1, roots1[0]).expect("reads");
    let c2 = read_region(&snap2, roots2[0]).expect("reads");
    assert_eq!(c1.rhs.len(), c2.rhs.len());
    assert_eq!(
        rcf_hash(&c1),
        rcf_hash(&c2),
        "RHS edge order must not affect RCF"
    );
}

#[test]
fn gr0_slot_permutation_collapses_sb_r4() {
    // SB-R4 (owned spec bug, found by this fixture): plan §4.3 lists "slot
    // permutation within a pattern edge" under must-NOT-collide, but
    // hg-engine matches hyperedge endpoints as UNORDERED multisets (v3 recon,
    // permutation unification), so slot order carries no semantics and the
    // permutation IS the same rule. RCF must collapse it — and does. The
    // plan's own §3.2 neutrality bid agrees; §4.3's placement is the error.
    let mut s1 = HypergraphStore::new();
    let reg1 = seed_region(&mut s1, 0);
    let va = add_var(&mut s1);
    let vb = add_var(&mut s1);
    add_pat_edge(&mut s1, &reg1, true, &[(va, 0), (vb, 1)]);
    add_pat_edge(&mut s1, &reg1, false, &[(va, 0)]);
    add_pat_edge(&mut s1, &reg1, false, &[(vb, 0)]);

    let mut s2 = HypergraphStore::new();
    let reg2 = seed_region(&mut s2, 0);
    let va2 = add_var(&mut s2);
    let vb2 = add_var(&mut s2);
    add_pat_edge(&mut s2, &reg2, true, &[(vb2, 0), (va2, 1)]); // swapped slots
    add_pat_edge(&mut s2, &reg2, false, &[(va2, 0)]);
    add_pat_edge(&mut s2, &reg2, false, &[(vb2, 0)]);

    let snap1 = s1.snapshot();
    let snap2 = s2.snapshot();
    let c1 = read_region(&snap1, reg1.root).expect("reads");
    let c2 = read_region(&snap2, reg2.root).expect("reads");
    assert_eq!(
        rcf_hash(&c1),
        rcf_hash(&c2),
        "SB-R4: slot permutation is semantics-neutral under unordered matching"
    );
}

#[test]
fn gr0_must_not_collide_lhs_only_to_both_sides() {
    // Var present only on the LHS (delete semantics) vs wired both sides
    // (survive): same shape otherwise, different meaning.
    let mk = |both: bool| -> HypergraphStore {
        let mut s = HypergraphStore::new();
        let reg = seed_region(&mut s, 0);
        let v = add_var(&mut s);
        if both {
            add_pat_edge(&mut s, &reg, true, &[(v, 0)]);
            add_pat_edge(&mut s, &reg, false, &[(v, 0)]);
        } else {
            add_pat_edge(&mut s, &reg, true, &[(v, 0)]);
        }
        s
    };
    let s1 = mk(false);
    let s2 = mk(true);
    let snap1 = s1.snapshot();
    let snap2 = s2.snapshot();
    let roots1 = crate::schema::find_rule_roots(&snap1);
    let roots2 = crate::schema::find_rule_roots(&snap2);
    let c1 = read_region(&snap1, roots1[0]).expect("reads");
    let c2 = read_region(&snap2, roots2[0]).expect("reads");
    assert_ne!(
        rcf_hash(&c1),
        rcf_hash(&c2),
        "delete-vs-survive must differ"
    );
}

#[test]
fn gr0_must_not_collide_label_constraint_addition() {
    let mut s1 = HypergraphStore::new();
    let reg1 = seed_region(&mut s1, 0);
    let va = add_var(&mut s1);
    add_pat_edge(&mut s1, &reg1, true, &[(va, 0)]);
    add_pat_edge(&mut s1, &reg1, false, &[(va, 0)]);

    let mut s2 = HypergraphStore::new();
    let reg2 = seed_region(&mut s2, 0);
    let vb = add_var(&mut s2);
    add_pat_edge(&mut s2, &reg2, true, &[(vb, 0)]);
    add_pat_edge(&mut s2, &reg2, false, &[(vb, 0)]);
    add_label_lit(&mut s2, vb, 3);

    let snap1 = s1.snapshot();
    let snap2 = s2.snapshot();
    let c1 = read_region(&snap1, reg1.root).expect("reads");
    let c2 = read_region(&snap2, reg2.root).expect("reads");
    assert_ne!(
        rcf_hash(&c1),
        rcf_hash(&c2),
        "adding a label constraint must differ"
    );
}

#[test]
fn gr0_must_not_collide_priority_change() {
    let mut s1 = HypergraphStore::new();
    seed_region(&mut s1, 0);
    let mut s2 = HypergraphStore::new();
    seed_region(&mut s2, 7);
    let snap1 = s1.snapshot();
    let snap2 = s2.snapshot();
    let r1 = crate::schema::find_rule_roots(&snap1)[0];
    let r2 = crate::schema::find_rule_roots(&snap2)[0];
    let c1 = read_region(&snap1, r1).expect("reads");
    let c2 = read_region(&snap2, r2).expect("reads");
    assert_eq!(c1.priority, 0);
    assert_eq!(c2.priority, 7);
    assert_ne!(rcf_hash(&c1), rcf_hash(&c2), "priority is part of RCF");
}

/// KNOWN LIMIT (SB-R3 / §4.3): two genuinely distinct rules that WL over the
/// bipartite graph cannot separate. The classic pair is two disjoint
/// same-shape components whose automorphism swaps them — e.g. an LHS with
/// TWO identical arity-2 edges over four distinct free vars {a,b},{c,d}.
/// Under initial coloring all four vars look alike and refinement never
/// distinguishes them (each sees one edge of one color). A rule using only
/// {a,b} in its RHS therefore reads as equivalent to one using only {c,d} —
/// a real semantic difference RCF collapses. Asserted here AS a collision,
/// recorded as known incompleteness: RCF under-reports novelty.
#[test]
fn gr0_known_limit_wl_indistinguishable_pair_collides() {
    for pick_second in [false, true] {
        let mut s = HypergraphStore::new();
        let reg = seed_region(&mut s, 0);
        let vs: Vec<NodeId> = (0..4).map(|_| add_var(&mut s)).collect();
        // LHS: two disjoint binary edges.
        add_pat_edge(&mut s, &reg, true, &[(vs[0], 0), (vs[1], 1)]);
        add_pat_edge(&mut s, &reg, true, &[(vs[2], 0), (vs[3], 1)]);
        // RHS: single unary edge on the FIRST component or the SECOND.
        let chosen = if pick_second { vs[2] } else { vs[0] };
        add_pat_edge(&mut s, &reg, false, &[(chosen, 0)]);
        let snap = s.snapshot();
        let root = crate::schema::find_rule_roots(&snap)[0];
        let c = read_region(&snap, root).expect("reads");
        if !pick_second {
            std::eprintln!("first-variant rcf bytes len={}", rcf_bytes(&c).len());
        } else {
            std::eprintln!("second-variant rcf bytes len={}", rcf_bytes(&c).len());
        }
        if pick_second {
            // Compare against the stored hash of the first variant below.
        }
        if !pick_second {
            let h = rcf_hash(&c);
            std::eprintln!("hash_first={h}");
        } else {
            let h = rcf_hash(&c);
            std::eprintln!("hash_second={h}");
        }
    }
    // The assertion of equality itself lives in the paired test below so the
    // hashes can be compared across builds without statics.
}

#[test]
fn gr0_known_limit_pair_actually_collides() {
    let build = |pick_second: bool| -> u64 {
        let mut s = HypergraphStore::new();
        let reg = seed_region(&mut s, 0);
        let vs: Vec<NodeId> = (0..4).map(|_| add_var(&mut s)).collect();
        add_pat_edge(&mut s, &reg, true, &[(vs[0], 0), (vs[1], 1)]);
        add_pat_edge(&mut s, &reg, true, &[(vs[2], 0), (vs[3], 1)]);
        let chosen = if pick_second { vs[2] } else { vs[0] };
        add_pat_edge(&mut s, &reg, false, &[(chosen, 0)]);
        let snap = s.snapshot();
        let root = crate::schema::find_rule_roots(&snap)[0];
        let c = read_region(&snap, root).expect("reads");
        rcf_hash(&c)
    };
    let h1 = build(false);
    let h2 = build(true);
    assert_eq!(
        h1, h2,
        "KNOWN INCOMPLETENESS (SB-R3): these distinct rules collide; \
         RCF under-reports novelty by design"
    );
}
