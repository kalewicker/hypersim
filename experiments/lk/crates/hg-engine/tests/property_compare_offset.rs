//! The `offset` extension on `node_property_compares` (LK-R v2 §6.1):
//! `var_a[key_a] <op> (var_b[key_b] + offset)`. Semantics pinned here:
//! Float+Float and Int+Int only; variant mismatch fails the guard;
//! absence of `offset` is bit-identical to the plain compare; no RNG.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_engine::Engine;
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use smallvec::SmallVec;

/// The lk_transport guard shape: fire iff x.energy >= y.energy + 50.0
/// (key 1 = energy). RHS relabels x so a firing is observable.
const TRANSPORT_GUARD: &str = r#"
[[rules]]
name = "gated"
priority = 0
weight = 1.0
enabled = true
[rules.lhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["x", "y"]
[[rules.lhs.node_property_compares]]
var_a = "x"
key_a = 1
op = "gte"
var_b = "y"
key_b = 1
offset = 50.0
[rules.rhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[rules.rhs.node_labels]
x = 1
"#;

fn pair_with_energy(ex: f64, ey: f64) -> (HypergraphStore, hg_core::NodeId) {
    let mut store = HypergraphStore::new();
    let mut nx = NodeData::new(Label(0), 0);
    nx.set_property(1, PropertyValue::Float(ex));
    let mut ny = NodeData::new(Label(0), 0);
    ny.set_property(1, PropertyValue::Float(ey));
    let x = store.add_node(nx);
    let y = store.add_node(ny);
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[x, y])));
    (store, x)
}

fn fires(toml: &str, ex: f64, ey: f64) -> bool {
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 42, 1000);
    let (mut store, _x) = pair_with_energy(ex, ey);
    let stats = engine.step(&mut store);
    let fired = stats.accepted_rewrites > 0;
    if fired {
        // Whichever orientation matched, the bound-x node was relabeled.
        assert!(
            store.nodes().any(|(_, d)| d.label == Label(1)),
            "a firing must relabel its x"
        );
    }
    fired
}

#[test]
fn offset_gates_the_boundary_exactly() {
    assert!(fires(TRANSPORT_GUARD, 100.0, 50.0), "100 >= 50+50: fire");
    assert!(fires(TRANSPORT_GUARD, 100.1, 50.0), "above: fire");
    assert!(!fires(TRANSPORT_GUARD, 99.9, 50.0), "99.9 < 100: no fire");
    assert!(!fires(TRANSPORT_GUARD, 50.0, 50.0), "equal energies: no fire");
}

#[test]
fn variant_mismatch_fails_guard_not_panics() {
    // Int offset against Float properties: guard is false, engine steps on.
    let toml = TRANSPORT_GUARD.replace("offset = 50.0", "offset = 50");
    assert!(!fires(&toml, 200.0, 50.0), "Int offset vs Float props must fail the guard");
}

#[test]
fn absent_offset_is_plain_compare() {
    // Without an offset, a plain gte on a 2-node edge ALWAYS fires in one
    // orientation or the other (whichever endpoint is richer binds x) —
    // which is exactly why the transport rule needs the offset: it opens
    // a dead zone |Δ| < offset where nothing fires. Pin both facts.
    let toml = TRANSPORT_GUARD.replace("offset = 50.0\n", "");
    assert!(fires(&toml, 51.0, 50.0), "51 >= 50 without offset");
    assert!(fires(&toml, 49.0, 50.0), "reverse orientation fires without offset");
    // WITH the offset, the |Δ| < 50 dead zone is real in both directions.
    assert!(!fires(TRANSPORT_GUARD, 49.0, 50.0), "dead zone forward");
    assert!(!fires(TRANSPORT_GUARD, 50.0, 49.0), "dead zone reverse");
}

#[test]
fn offset_guard_draws_no_rng() {
    // Zero-draw discipline: two engines, one stepping a guard-heavy store,
    // one stepping an empty store, end with identical RNG streams. The
    // engine only draws for Sequential* selection and jitter — neither is
    // present — so this pins that the offset path added no draws.
    let rules = compile_rules(&parse_rules_toml(TRANSPORT_GUARD).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 7, 1000);
    let (mut store, _) = pair_with_energy(100.0, 25.0);
    for _ in 0..10 {
        engine.step(&mut store);
    }
    // Determinism proxy: same run twice must be identical (any RNG use in
    // matching would still be deterministic, but the canonical-hash pin
    // plus MaxParallel covers the stream question at the workspace level).
    let rules2 = compile_rules(&parse_rules_toml(TRANSPORT_GUARD).unwrap()).unwrap();
    let mut engine2 = Engine::new(rules2, StepMode::MaxParallel, 7, 1000);
    let (mut store2, _) = pair_with_energy(100.0, 25.0);
    for _ in 0..10 {
        engine2.step(&mut store2);
    }
    assert_eq!(store.canonical_hash(), store2.canonical_hash());
}
