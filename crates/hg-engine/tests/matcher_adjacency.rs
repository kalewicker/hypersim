//! Adjacency-guided matching regression: the optimized candidate
//! enumeration (edges incident to an already-bound variable) must
//! produce EXACTLY the binding set of the naive all-edges scan, on a
//! fixture rich in wedges, multi-incidence, and label constraints.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, NodeId};
use hg_engine::find_bindings;
use hg_rules::{compile_rules, parse_rules_toml};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use smallvec::smallvec;
use std::collections::BTreeSet;

const WEDGE: &str = r#"
[[rules]]
name = "wedge"
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b", "c"]
[rules.rhs]
nodes = ["a", "c"]
edges = [["a", "c"]]
"#;

/// Deterministic scruffy graph: pair edges, a few duplicates of the
/// same endpoints, and one self-pair (multi-incidence).
fn fixture(seed: u64) -> (HypergraphStore, Vec<(hg_core::EdgeId, NodeId, NodeId)>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut store = HypergraphStore::new();
    let nodes: Vec<NodeId> = (0..20).map(|_| store.add_node(NodeData::new(Label(0), 0))).collect();
    let mut edges = Vec::new();
    for _ in 0..40 {
        let u = nodes[rng.gen_range(0..nodes.len())];
        let v = nodes[rng.gen_range(0..nodes.len())];
        let id = store.add_edge(HyperEdge::new(Label(0), smallvec![u, v]));
        edges.push((id, u, v));
    }
    (store, edges)
}

/// Naive reference: enumerate ordered pairs of distinct edges and all
/// orientation assignments, mirroring the pre-optimization semantics.
fn reference_bindings(
    edges: &[(hg_core::EdgeId, NodeId, NodeId)],
) -> BTreeSet<(Vec<NodeId>, Vec<hg_core::EdgeId>)> {
    let mut out = BTreeSet::new();
    for &(e1, u1, v1) in edges {
        // Interchangeable-position rule: a self-pair (u,u) yields one
        // orientation, not two.
        let orients1: Vec<(NodeId, NodeId)> =
            if u1 == v1 { vec![(u1, v1)] } else { vec![(u1, v1), (v1, u1)] };
        for (a, b) in orients1 {
            if a == b {
                continue; // vars_distinct(a,b)
            }
            for &(e2, u2, v2) in edges {
                if e2 == e1 {
                    continue;
                }
                let cs: Vec<NodeId> = if u2 == v2 {
                    if u2 == b { vec![u2] } else { vec![] }
                } else if u2 == b {
                    vec![v2]
                } else if v2 == b {
                    vec![u2]
                } else {
                    vec![]
                };
                for c in cs {
                    if c == a || c == b {
                        continue;
                    }
                    out.insert((vec![a, b, c], vec![e1, e2]));
                }
            }
        }
    }
    out
}

#[test]
fn adjacency_guided_matching_is_binding_set_identical() {
    let rules = compile_rules(&parse_rules_toml(WEDGE).unwrap()).unwrap();
    for seed in [1u64, 7, 42, 1234] {
        let (store, edges) = fixture(seed);
        let actual: BTreeSet<(Vec<NodeId>, Vec<hg_core::EdgeId>)> =
            find_bindings(&store.snapshot(), &rules[0], 0)
                .into_iter()
                .map(|b| (b.bound_nodes.to_vec(), b.matched_edges.to_vec()))
                .collect();
        let expected = reference_bindings(&edges);
        assert_eq!(
            actual, expected,
            "seed {seed}: adjacency-guided matching changed the binding set"
        );
        // Also assert no duplicates were emitted (set size == vec len).
        let raw = find_bindings(&store.snapshot(), &rules[0], 0);
        assert_eq!(raw.len(), actual.len(), "seed {seed}: duplicate bindings emitted");
    }
}
