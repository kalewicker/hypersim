//! S5.2 gate — `LineageTree` reconstructed correctly on a hand-traced
//! 3-generation fixture: parent/child links sound, generations correct, Newick
//! well-formed, and the criterion-6 non-trivial count behaves.
//!
//! `NodeId`s are real (minted from a throwaway store); the tree itself is
//! engine-free.

use hg_alife::genome::Genome;
use hg_alife::LineageTree;
use hg_core::{HypergraphStore, Label, NodeData, NodeId};

fn g(thr: f64) -> Genome {
    Genome {
        harvest_rate: Some(1.0),
        replication_threshold: Some(thr),
        maintenance_cost: Some(1.0),
        satiation: Some(20.0),
    }
}

/// Mint `n` distinct NodeIds.
fn ids(n: usize) -> Vec<NodeId> {
    let mut s = HypergraphStore::new();
    (0..n).map(|_| s.add_node(NodeData::new(Label(100), 0))).collect()
}

#[test]
fn three_generation_lineage_is_sound() {
    // founder(0) -> a(1), b(2);  a -> c(3);  c -> d(4).  (a chain a→c→d plus sib b)
    let v = ids(5);
    let (founder, a, b, c, d) = (v[0], v[1], v[2], v[3], v[4]);
    let mut t = LineageTree::new();
    t.record_birth(founder, None, g(15.0), 1); // bootstrap root
    t.record_birth(a, Some(founder), g(14.8), 5);
    t.record_birth(b, Some(founder), g(15.2), 6);
    t.record_birth(c, Some(a), g(14.6), 10);
    t.record_birth(d, Some(c), g(14.4), 15);

    // Structure.
    assert_eq!(t.clade_count(), 1, "one bootstrap founder ⟹ one root clade");
    assert_eq!(t.parent_of(a), Some(founder));
    assert_eq!(t.parent_of(d), Some(c));
    assert_eq!(t.parent_of(founder), None, "founder is a root");
    assert_eq!(t.children_of(founder), &[a, b]);
    assert_eq!(t.children_of(a), &[c]);

    // Generations: founder 0, a/b 1, c 2, d 3.
    assert_eq!(t.generation(founder), 0);
    assert_eq!(t.generation(a), 1);
    assert_eq!(t.generation(c), 2);
    assert_eq!(t.generation(d), 3);
    assert_eq!(t.max_generation(), 3);

    // survival_curve[g] = carriers at generation g: [1 founder, 2 (a,b), 1 (c), 1 (d)].
    assert_eq!(t.survival_curve(), vec![1, 2, 1, 1]);

    // Genome at birth is retained.
    assert_eq!(t.genome_at_birth(d).unwrap().replication_threshold, Some(14.4));

    // Newick: founder has two children, one of which (a) has a chain c→d.
    // display ids (birth order): founder=1, a=2, b=3, c=4, d=5.
    // chain a→c→d = ((5)4)2 ; founder's children = that + sib b(3), wrapped under 1.
    let nwk = t.export_newick();
    assert!(nwk.ends_with(';'), "Newick terminates with ';': {nwk}");
    assert_eq!(nwk, "(((5)4)2,3)1;", "hand-traced 3-generation Newick");
}

#[test]
fn deaths_and_liveness_track() {
    let v = ids(3);
    let (founder, a, bb) = (v[0], v[1], v[2]);
    let mut t = LineageTree::new();
    t.record_birth(founder, None, g(15.0), 1);
    t.record_birth(a, Some(founder), g(15.0), 5);
    t.record_birth(bb, Some(founder), g(15.0), 6);
    assert!(t.is_alive(a));
    t.record_death(a, 20);
    assert!(!t.is_alive(a), "a died at step 20");
    assert!(t.is_alive(bb), "bb still alive");
    assert_eq!(t.total_deaths(), 1);
    assert_eq!(t.total_births(), 2, "a and bb are replication births; founder is bootstrap");

    // Founder's surviving subclades: a's subtree is dead, bb's is alive ⟹ 1.
    assert_eq!(t.surviving_subclades(), 1);
}

#[test]
fn non_trivial_replicator_count_requires_sustained_reproduction() {
    // A one-off birth (founder→a, a never reproduces) is NOT a non-trivial
    // replicator at min_generations=2; a sustained chain founder→a→c→d IS.
    let v = ids(4);
    let (founder, a, c, d) = (v[0], v[1], v[2], v[3]);
    let mut t = LineageTree::new();
    t.record_birth(founder, None, g(15.0), 1);
    t.record_birth(a, Some(founder), g(15.0), 5);
    // Only founder is a parent so far; its deepest descendant depth = 1 (just a).
    assert_eq!(t.non_trivial_replicator_count(2), 0, "single birth is not sustained self-copying");

    t.record_birth(c, Some(a), g(15.0), 10);
    t.record_birth(d, Some(c), g(15.0), 15);
    // founder now has descendants a→c→d (depth 3); a has c→d (depth 2). Both ≥ 2.
    assert_eq!(
        t.non_trivial_replicator_count(2),
        2,
        "founder and a each sustain a lineage ≥ 2 generations deep"
    );
    // On a bare substrate (no births at all) the count is 0 by construction.
    assert_eq!(LineageTree::new().non_trivial_replicator_count(2), 0);
}
