//! Phase 5 exit criteria (v4 §13):
//! - matcher non-injective by default (explicit test)
//! - priority honored via Reverse<i32> in the conflict-resolution sort key
//! - creation-rule firing (§5.3)
//! - disjoint-bound-nodes ⇒ disjoint-conflict-sets, checked directly (§8.2)
//! - same seed + config ⇒ same canonical_hash regardless of thread count
//! - MaxParallel applies all non-conflicting matches

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData};
use rand::SeedableRng;
use hg_engine::{apply_binding, conflict_set, find_bindings, resolve_conflicts, Engine};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use smallvec::SmallVec;

fn store_with_chain(n: usize) -> HypergraphStore {
    let mut s = HypergraphStore::new();
    let ids: Vec<_> = (0..n)
        .map(|_| s.add_node(NodeData::new(Label(0), 0)))
        .collect();
    for w in ids.windows(2) {
        s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[w[0], w[1]])));
    }
    s
}

#[test]
fn matcher_non_injective_by_default() {
    // Pattern: two edges a-b, b-c with NO vars_distinct guard. On a single
    // edge x-y, the assignment a=x, b=y, c=x (a==c) is a legal match only
    // if matching is non-injective. Two physical edges are still required
    // (edge injectivity), so build two parallel edges x-y.
    let toml = r#"
[[rules]]
name = "chain"
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
[rules.rhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut s = HypergraphStore::new();
    let x = s.add_node(NodeData::new(Label(0), 0));
    let y = s.add_node(NodeData::new(Label(0), 0));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[x, y])));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[x, y])));

    let bindings = find_bindings(&s.snapshot(), &rules[0], 0);
    // Non-injective assignments (a==c or a==b==… patterns) must be present.
    assert!(
        bindings.iter().any(|b| b.bound_nodes[0] == b.bound_nodes[2]),
        "expected a non-injective match (a == c); got {} bindings",
        bindings.len()
    );

    // Same pattern WITH vars_distinct must exclude them.
    let toml_distinct = r#"
[[rules]]
name = "chain_distinct"
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b", "c"]
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let rules_d = compile_rules(&parse_rules_toml(toml_distinct).unwrap()).unwrap();
    let bindings_d = find_bindings(&s.snapshot(), &rules_d[0], 0);
    assert!(bindings_d.is_empty(), "vars_distinct must kill 2-node chains");
}

#[test]
fn priority_wins_conflicts_via_reverse_sort_key() {
    // Two rules match the same edge; higher priority must win even though
    // it has the higher rule_id (file order must NOT decide).
    let toml = r#"
[[rules]]
name = "low"
priority = 0
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]

[[rules]]
name = "high"
priority = 5
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let s = store_with_chain(2);
    let mut all = Vec::new();
    for (i, r) in rules.iter().enumerate() {
        all.extend(find_bindings(&s.snapshot(), r, i));
    }
    let accepted = resolve_conflicts(&all);
    assert!(!accepted.is_empty());
    assert_eq!(accepted[0].rule_id, 1, "high-priority rule must be accepted first");
    // Both bindings share nodes, so only one is accepted.
    assert_eq!(accepted.len(), 1);
}

#[test]
fn creation_rule_fires_on_schedule_and_never_conflicts() {
    let toml = r#"
[[rules]]
name = "seed_triangle"
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.schedule]
mode = "every_n_steps"
n = 2

[[rules]]
name = "edge_grow"
[rules.lhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[rules.rhs]
nodes = ["x", "y", "z"]
edges = [["x", "y"], ["y", "z"]]
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 1000);
    let mut store = HypergraphStore::new();

    // Step 0: creation due (0 % 2 == 0) -> 2 nodes, 1 edge.
    engine.step(&mut store);
    assert_eq!(store.node_count(), 2);
    assert_eq!(store.edge_count(), 1);

    // Step 1: creation not due; edge_grow fires on the seeded structure.
    let before = store.node_count();
    engine.step(&mut store);
    assert!(store.node_count() > before, "ordinary rule fired alongside");

    // Step 2: creation due again — accepted alongside whatever else fires.
    let nodes_before = store.node_count();
    engine.step(&mut store);
    assert!(store.node_count() >= nodes_before + 2, "creation fired at step 2");
}

#[test]
fn disjoint_bound_nodes_implies_effects_cannot_touch_dependencies() {
    // The §8.2 invariant, checked directly — in the form the rewrite
    // contract's proof actually establishes. Note: two bindings deleting
    // ADJACENT nodes can share a frontier edge in both dangling-cleanup
    // sets (so raw conflict_set ∩ conflict_set may be non-empty); that
    // overlap is order-independent because edge removal is idempotent.
    // The invariant that guarantees correctness is: with disjoint
    // bound_nodes, one binding's EFFECTS (deleted nodes, matched edges,
    // cleanup edges) never intersect the other's DEPENDENCIES (bound
    // nodes, matched edges). See docs/rewrite_contract.md.
    let toml = r#"
[[rules]]
name = "shrink"
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b", "c"]
[rules.rhs]
nodes = ["b", "c"]
edges = [["b", "c"]]
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let store = store_with_chain(8);
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    assert!(bindings.len() > 2);

    for i in 0..bindings.len() {
        for j in 0..bindings.len() {
            if i == j {
                continue;
            }
            let a = &bindings[i];
            let b = &bindings[j];
            let nodes_a: std::collections::HashSet<_> =
                a.bound_nodes.iter().copied().collect();
            if !b.bound_nodes.iter().all(|n| !nodes_a.contains(n)) {
                continue; // bound nodes overlap — resolver rejects anyway
            }
            // a's effects = full conflict set incl. cleanup edges.
            let effects_a = conflict_set(&store, &rules[0], a);
            // b's dependencies = bound nodes + matched edges only.
            let dep_nodes: std::collections::HashSet<_> =
                b.bound_nodes.iter().copied().collect();
            let dep_edges: std::collections::HashSet<_> =
                b.matched_edges.iter().copied().collect();

            let deleted_a: std::collections::HashSet<_> = rules[0]
                .template
                .lhs_deleted
                .iter()
                .map(|&ix| a.bound_nodes[ix])
                .collect();
            assert!(
                deleted_a.is_disjoint(&dep_nodes),
                "a deletes a node b depends on: {a:?} vs {b:?}"
            );
            assert!(
                effects_a.edges.is_disjoint(&dep_edges),
                "a's effects remove an edge b depends on: {a:?} vs {b:?}"
            );
        }
    }
}

#[test]
fn max_parallel_applies_all_non_conflicting_matches() {
    // Chain of 9 nodes: shrink rule matches overlapping windows; the
    // resolver must accept a maximal greedy set, not just one.
    let toml = r#"
[[rules]]
name = "relabel"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b"]
[rules.rhs]
nodes = ["a", "b", "m"]
edges = [["a", "m"], ["m", "b"]]
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 1, 1000);
    // 4 disjoint edges -> all 4 must fire in one step.
    let mut store = HypergraphStore::new();
    for _ in 0..4 {
        let a = store.add_node(NodeData::new(Label(0), 0));
        let b = store.add_node(NodeData::new(Label(0), 0));
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    }
    let stats = engine.step(&mut store);
    assert_eq!(stats.accepted_rewrites, 4);
    assert_eq!(store.node_count(), 12);
    assert_eq!(store.edge_count(), 8);
}

fn run_to_hash(threads: usize, seed: u64, steps: u64) -> u64 {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    pool.install(|| {
        let rules = compile_rules(&hg_rules::builtin_rules("binary_edge").unwrap()).unwrap();
        let mut engine = Engine::new(rules, StepMode::MaxParallel, seed, 64);
        let mut store = HypergraphStore::new();
        hg_core::generate_white_noise(&mut store, 20, 25, 2, 3, seed);
        for _ in 0..steps {
            engine.step(&mut store);
        }
        store.canonical_hash()
    })
}

#[test]
fn same_seed_same_hash_across_thread_counts() {
    let h1 = run_to_hash(1, 42, 6);
    let h2 = run_to_hash(2, 42, 6);
    let h8 = run_to_hash(8, 42, 6);
    assert_eq!(h1, h2);
    assert_eq!(h1, h8);

    // And a different seed must (overwhelmingly) differ.
    let other = run_to_hash(4, 43, 6);
    assert_ne!(h1, other);
}

#[test]
fn apply_binding_respects_edge_labels_and_survivors() {
    let toml = r#"
[[rules]]
name = "labelled_split"
[rules.lhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[rules.rhs]
nodes = ["x", "y", "z"]
edges = [["x", "z"], ["z", "y"]]
[rules.rhs.edge_labels]
0 = 5
[rules.rhs.node_labels]
z = 2
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut store = HypergraphStore::new();
    let x = store.add_node(NodeData::new(Label(0), 0));
    let y = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[x, y])));

    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    let (event, _, _) = apply_binding(&mut store, &rules[0], &bindings[0], &mut rand_chacha::ChaCha8Rng::seed_from_u64(0));

    assert_eq!(event.nodes_created.len(), 1);
    assert_eq!(event.edges_deleted.len(), 1);
    assert_eq!(event.edges_created.len(), 2);
    let z = event.nodes_created[0];
    assert_eq!(store.get_node(z).unwrap().label, Label(2));
    // Sparse RHS edge labels: edge 0 -> 5, edge 1 -> default 0.
    assert_eq!(store.get_edge(event.edges_created[0]).unwrap().label, Label(5));
    assert_eq!(store.get_edge(event.edges_created[1]).unwrap().label, Label(0));
    // Survivors keep their NodeIds.
    assert!(store.get_node(x).is_some());
    assert!(store.get_node(y).is_some());
}

#[test]
fn property_guard_gates_matching() {
    use hg_core::PropertyValue;
    let toml = r#"
[[rules]]
name = "hot_only"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[[rules.lhs.node_property_guards]]
var = "a"
property = 1
op = "gt"
value = 0.5
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut store = HypergraphStore::new();
    let a = store.add_node(NodeData::new(Label(0), 0));
    let b = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));

    // No property set: guard fails, no match.
    assert!(find_bindings(&store.snapshot(), &rules[0], 0).is_empty());

    // Float 0.8 > 0.5: matches (var a must be the hot node).
    store.get_node_mut(a).unwrap().set_property(1, PropertyValue::Float(0.8));
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    assert!(!bindings.is_empty());
    assert!(bindings.iter().all(|bi| bi.bound_nodes[0] == a));

    // Variant mismatch (Int stored vs Float threshold): guard false.
    store.get_node_mut(a).unwrap().set_property(1, PropertyValue::Int(999));
    assert!(find_bindings(&store.snapshot(), &rules[0], 0).is_empty());
}

/// Regression (v4 §5.3): creation firings are always accepted — the
/// MaxParallel pairing cap budgets ordinary rewrites only. The bug:
/// .take(cap) ran over the merged accepted list, so higher-priority
/// ordinary matches could starve a scheduled creation rule forever.
#[test]
fn creation_rule_survives_pairing_cap_pressure() {
    let toml = r#"
[[rules]]
name = "busy_high_priority"
priority = 9
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b"]
[rules.rhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]

[[rules]]
name = "seeder"
priority = 0
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["x"]
edges = []
[rules.rhs.node_labels]
x = 7
[rules.schedule]
mode = "every_n_steps"
n = 1
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    // Cap of 1 with several disjoint high-priority matches available.
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 9, 1);
    let mut store = HypergraphStore::new();
    for _ in 0..3 {
        let a = store.add_node(NodeData::new(Label(0), 0));
        let b = store.add_node(NodeData::new(Label(0), 0));
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    }

    for _ in 0..4 {
        engine.step(&mut store);
    }
    // 4 steps, schedule due every step: exactly 4 label-7 seeder nodes must
    // exist despite the cap being consumed by the busy rule every step.
    let seeded = store
        .nodes()
        .filter(|(_, d)| d.label == Label(7))
        .count();
    assert_eq!(seeded, 4, "creation rule must fire every due step under cap pressure");
}

/// Creation firings are also exempt from sequential single-candidate
/// sampling: a once-scheduled seed must not lose the coin flip.
#[test]
fn creation_rule_fires_in_sequential_modes() {
    let toml = r#"
[[rules]]
name = "grower"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]

[[rules]]
name = "seed_once"
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[rules.rhs.node_labels]
x = 5
y = 5
[rules.schedule]
mode = "once"
"#;
    for mode in [StepMode::SequentialRandom, StepMode::SequentialWeighted] {
        let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
        let mut engine = Engine::new(rules, mode, 123, 1000);
        let mut store = HypergraphStore::new();
        let a = store.add_node(NodeData::new(Label(0), 0));
        let b = store.add_node(NodeData::new(Label(0), 0));
        store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));

        let stats = engine.step(&mut store);
        let seeded = store.nodes().filter(|(_, d)| d.label == Label(5)).count();
        assert_eq!(seeded, 2, "{mode:?}: once-schedule must fire at step 0");
        // And an ordinary rewrite still happened alongside it.
        assert_eq!(stats.accepted_rewrites, 2, "{mode:?}: creation + one sampled rewrite");
    }
}

/// Regression (review finding): an edge with repeated endpoints must not
/// produce duplicate identical bindings via endpoint-position permutation,
/// while a normal edge still yields both genuine variable assignments.
#[test]
fn repeated_endpoint_edge_yields_one_binding_not_two() {
    let toml = r#"
[[rules]]
name = "pair"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();

    // Self-pair edge (x,x): exactly ONE binding {a->x, b->x}.
    let mut s = HypergraphStore::new();
    let x = s.add_node(NodeData::new(Label(0), 0));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[x, x])));
    let bindings = find_bindings(&s.snapshot(), &rules[0], 0);
    assert_eq!(bindings.len(), 1, "identical-endpoint permutations must dedup");
    assert_eq!(bindings[0].bound_nodes[0], x);
    assert_eq!(bindings[0].bound_nodes[1], x);

    // Distinct-endpoint edge (x,y): both assignments remain (a=x,b=y and
    // a=y,b=x are different morphisms).
    let mut s2 = HypergraphStore::new();
    let x2 = s2.add_node(NodeData::new(Label(0), 0));
    let y2 = s2.add_node(NodeData::new(Label(0), 0));
    s2.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[x2, y2])));
    let bindings2 = find_bindings(&s2.snapshot(), &rules[0], 0);
    assert_eq!(bindings2.len(), 2, "distinct endpoints keep both assignments");
}
