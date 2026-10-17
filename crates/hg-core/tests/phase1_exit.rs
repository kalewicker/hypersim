//! Phase 1 exit criteria (v4 §13):
//! - canonical_hash stable across construction order
//! - WL hash isomorphism-invariant on hand-crafted pairs
//! - all generators reproducible under a fixed seed

use hg_core::{
    compute_wl_hash, generate_crystal, generate_minimal_seed, generate_white_noise,
    HyperEdge, HypergraphStore, Label, NodeData, PropertyOp, PropertyValue,
};
use proptest::prelude::*;
use smallvec::SmallVec;

fn node(label: u32) -> NodeData {
    NodeData::new(Label(label), 0)
}

#[test]
fn canonical_hash_stable_across_construction_order() {
    // Graph: labels 1,2,3 with edges (1-2), (2-3).
    let mut a = HypergraphStore::new();
    let a1 = a.add_node(node(1));
    let a2 = a.add_node(node(2));
    let a3 = a.add_node(node(3));
    a.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a1, a2])));
    a.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a2, a3])));

    // Same logical graph, reversed insertion order.
    let mut b = HypergraphStore::new();
    let b3 = b.add_node(node(3));
    let b2 = b.add_node(node(2));
    let b1 = b.add_node(node(1));
    b.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[b2, b3])));
    b.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[b1, b2])));

    assert_eq!(a.canonical_hash(), b.canonical_hash());
}

#[test]
fn canonical_hash_stable_after_removal_churn() {
    // Insert junk, remove it — hash must equal the clean construction.
    let mut a = HypergraphStore::new();
    let junk = a.add_node(node(99));
    let a1 = a.add_node(node(1));
    let a2 = a.add_node(node(2));
    a.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a1, a2])));
    a.remove_node(junk);

    let mut b = HypergraphStore::new();
    let b1 = b.add_node(node(1));
    let b2 = b.add_node(node(2));
    b.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[b1, b2])));

    assert_eq!(a.canonical_hash(), b.canonical_hash());
}

#[test]
fn canonical_hash_distinguishes_different_graphs() {
    let mut a = HypergraphStore::new();
    let a1 = a.add_node(node(1));
    let a2 = a.add_node(node(2));
    a.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a1, a2])));

    let mut b = HypergraphStore::new();
    let b1 = b.add_node(node(1));
    let b2 = b.add_node(node(2));
    b.add_edge(HyperEdge::new(Label(7), SmallVec::from_slice(&[b1, b2])));

    assert_ne!(a.canonical_hash(), b.canonical_hash());
}

#[test]
fn canonical_hash_sees_properties() {
    let mut a = HypergraphStore::new();
    let id = a.add_node(node(1));
    a.get_node_mut(id)
        .unwrap()
        .set_property(1, PropertyValue::Float(0.5));

    let mut b = HypergraphStore::new();
    b.add_node(node(1));

    assert_ne!(a.canonical_hash(), b.canonical_hash());
}

#[test]
fn wl_hash_isomorphism_invariant_on_handcrafted_pair() {
    // Two triangles built in different orders: center nodes must agree.
    fn triangle(order: [u32; 3]) -> (HypergraphStore, hg_core::NodeId) {
        let mut s = HypergraphStore::new();
        let mut ids = Vec::new();
        for lbl in order {
            ids.push(s.add_node(node(lbl)));
        }
        // find the node labelled 1 (our probe)
        let probe = *ids
            .iter()
            .find(|id| s.get_node(**id).unwrap().label == Label(1))
            .unwrap();
        for i in 0..3 {
            for j in (i + 1)..3 {
                s.add_edge(HyperEdge::new(
                    Label(0),
                    SmallVec::from_slice(&[ids[i], ids[j]]),
                ));
            }
        }
        (s, probe)
    }

    let (s1, p1) = triangle([1, 2, 3]);
    let (s2, p2) = triangle([3, 2, 1]);
    for r in 0..=3u8 {
        assert_eq!(
            compute_wl_hash(&s1, p1, r),
            compute_wl_hash(&s2, p2, r),
            "radius {r}"
        );
    }
}

#[test]
fn wl_hash_distinguishes_non_isomorphic_neighbourhoods() {
    // Path a-b-c: endpoint vs middle differ at radius 1.
    let mut s = HypergraphStore::new();
    let a = s.add_node(node(0));
    let b = s.add_node(node(0));
    let c = s.add_node(node(0));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[b, c])));

    assert_ne!(compute_wl_hash(&s, a, 1), compute_wl_hash(&s, b, 1));
    // The two endpoints are symmetric — must agree.
    assert_eq!(compute_wl_hash(&s, a, 1), compute_wl_hash(&s, c, 1));
}

#[test]
fn wl_hash_survives_glider_rebuild_at_new_node_ids() {
    // The same 3-node chain built twice in one store (fresh NodeIds the
    // second time) must give the middle node the same WL hash — this is
    // what lets the evaluator recognise a translated glider.
    let mut s = HypergraphStore::new();
    let a = s.add_node(node(0));
    let b = s.add_node(node(0));
    let c = s.add_node(node(0));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[b, c])));
    let h_before = compute_wl_hash(&s, b, 2);

    s.remove_node(a);
    s.remove_node(b);
    s.remove_node(c);
    let mut s2 = s; // same store, continue after deletion
    let a2 = s2.add_node(NodeData::new(Label(0), 7)); // later generation
    let b2 = s2.add_node(NodeData::new(Label(0), 7));
    let c2 = s2.add_node(NodeData::new(Label(0), 7));
    s2.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a2, b2])));
    s2.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[b2, c2])));

    assert_eq!(h_before, compute_wl_hash(&s2, b2, 2));
}

#[test]
fn generators_reproducible_under_fixed_seed() {
    for seed in [0u64, 42, 12345] {
        let mut a = HypergraphStore::new();
        let mut b = HypergraphStore::new();
        generate_white_noise(&mut a, 30, 40, 3, 5, seed);
        generate_white_noise(&mut b, 30, 40, 3, 5, seed);
        assert_eq!(a.canonical_hash(), b.canonical_hash(), "white_noise seed {seed}");

        let mut c = HypergraphStore::new();
        let mut d = HypergraphStore::new();
        generate_crystal(&mut c, 6, 6, 2, false, seed);
        generate_crystal(&mut d, 6, 6, 2, false, seed);
        assert_eq!(c.canonical_hash(), d.canonical_hash(), "crystal seed {seed}");

        let mut e = HypergraphStore::new();
        let mut f = HypergraphStore::new();
        generate_minimal_seed(&mut e, 3, 2, seed);
        generate_minimal_seed(&mut f, 3, 2, seed);
        assert_eq!(e.canonical_hash(), f.canonical_hash(), "minimal_seed seed {seed}");
    }
}

#[test]
fn property_value_comparison_semantics() {
    use PropertyValue as PV;
    // Same variant: normal comparison.
    assert!(PV::Int(5).compare(PropertyOp::Gt, &PV::Int(3)));
    assert!(PV::Float(0.5).compare(PropertyOp::Lte, &PV::Float(0.5)));
    assert!(PV::Bool(true).compare(PropertyOp::Ne, &PV::Bool(false)));
    // Variant mismatch: false, never a panic or coercion (v4 §3).
    assert!(!PV::Int(1).compare(PropertyOp::Eq, &PV::Float(1.0)));
    assert!(!PV::Float(1.0).compare(PropertyOp::Gt, &PV::Int(0)));
    assert!(!PV::Bool(true).compare(PropertyOp::Eq, &PV::Int(1)));
    // NaN: all comparisons false.
    assert!(!PV::Float(f64::NAN).compare(PropertyOp::Eq, &PV::Float(f64::NAN)));
}

#[test]
fn property_value_toml_untagged_selection() {
    #[derive(serde::Deserialize)]
    struct Probe {
        a: PropertyValue,
        b: PropertyValue,
        c: PropertyValue,
    }
    let p: Probe = toml::from_str("a = 100\nb = 0.8\nc = true").unwrap();
    assert_eq!(p.a, PropertyValue::Int(100));
    assert_eq!(p.b, PropertyValue::Float(0.8));
    assert_eq!(p.c, PropertyValue::Bool(true));
}

proptest! {
    /// Property test: canonical_hash is invariant under a random node
    /// insertion permutation of the same logical graph.
    #[test]
    fn canonical_hash_invariant_under_permutation(perm_seed in 0u64..1000) {
        use rand::seq::SliceRandom;
        use rand::SeedableRng;

        let labels: Vec<u32> = (0..8).collect();
        // Logical edges by label pairs.
        let edges: Vec<(u32, u32)> = vec![(0,1),(1,2),(2,3),(3,4),(4,5),(5,6),(6,7),(7,0),(0,4)];

        let build = |order: &[u32]| {
            let mut s = HypergraphStore::new();
            let mut id_of = std::collections::HashMap::new();
            for &lbl in order {
                id_of.insert(lbl, s.add_node(node(lbl)));
            }
            for &(x, y) in &edges {
                s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[id_of[&x], id_of[&y]])));
            }
            s
        };

        let base = build(&labels);
        let mut shuffled = labels.clone();
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(perm_seed);
        shuffled.shuffle(&mut rng);
        let permuted = build(&shuffled);

        prop_assert_eq!(base.canonical_hash(), permuted.canonical_hash());
    }

    /// WL hash never panics on arbitrary radii/stale lookups.
    #[test]
    fn wl_hash_panic_free(radius in 0u8..5, n in 1usize..20) {
        let mut s = HypergraphStore::new();
        let mut ids = Vec::new();
        for i in 0..n {
            ids.push(s.add_node(node(i as u32 % 3)));
        }
        for w in ids.windows(2) {
            s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[w[0], w[1]])));
        }
        let probe = ids[0];
        let _ = compute_wl_hash(&s, probe, radius);
        // Stale id after removal:
        s.remove_node(probe);
        prop_assert_eq!(compute_wl_hash(&s, probe, radius), hg_core::WlHash(0));
    }
}

/// Regression (review finding): content-identical nodes with different
/// connectivity must not let insertion order leak into the hash. Nodes
/// u,v share identical content; only u carries an edge. Before the
/// WL-refinement tie-break, insertion order (u,v) vs (v,u) produced
/// different canonical indices for the tied pair and thus different
/// hashes for the same logical graph.
#[test]
fn canonical_hash_breaks_content_ties_by_structure() {
    let build = |tied_first: bool| {
        let mut s = HypergraphStore::new();
        let (u, _v) = if tied_first {
            let u = s.add_node(node(0));
            let v = s.add_node(node(0));
            (u, v)
        } else {
            let v = s.add_node(node(0));
            let u = s.add_node(node(0));
            (u, v)
        };
        let w = s.add_node(node(1));
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[u, w])));
        s
    };
    assert_eq!(build(true).canonical_hash(), build(false).canonical_hash());
}

proptest! {
    /// The shared class map must be bit-identical to the per-node WL hash
    /// at every node — the evaluator and causal layer's notion of "type"
    /// hangs on this equivalence.
    #[test]
    fn class_map_equals_per_node_wl(seed in 0u64..200, radius in 0u8..3) {
        let mut s = HypergraphStore::new();
        generate_white_noise(&mut s, 25, 35, 3, 4, seed);
        let map = hg_core::compute_wl_class_map(&s, radius);
        for (id, _) in s.nodes() {
            prop_assert_eq!(map[&id], compute_wl_hash(&s, id, radius));
        }
    }
}
