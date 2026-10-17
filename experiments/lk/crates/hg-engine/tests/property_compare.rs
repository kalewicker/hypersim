//! S1.4 — `Guard::PropertyCompare` evaluation (hg-alife scope §5.3).
//!
//! Exercises the full parse → compile → match path for property-to-property
//! comparison guards: every op in both directions, variant mismatch, missing
//! key, self- vs cross-variable comparison. Also pins that a single-node /
//! no-edge LHS matches every node (the S3.1 bootstrap/influx pattern shape).

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_engine::find_bindings;
use hg_rules::{compile_rules, parse_rules_toml};
use smallvec::SmallVec;

/// Self-comparison on one node: `a[10] <op> a[11]`, single-node no-edge LHS.
/// Returns the number of bindings (1 = guard passed, 0 = guard failed).
fn self_cmp_bindings(op: &str, a10: PropertyValue, a11: Option<PropertyValue>) -> usize {
    let toml = format!(
        r#"
[[rules]]
name = "cmp"
[rules.lhs]
nodes = ["a"]
edges = []
[[rules.lhs.node_property_compares]]
var_a = "a"
key_a = 10
op = "{op}"
var_b = "a"
key_b = 11
[rules.rhs]
nodes = ["a"]
edges = []
"#
    );
    let rules = compile_rules(&parse_rules_toml(&toml).unwrap()).unwrap();
    let mut s = HypergraphStore::new();
    let n = s.add_node(NodeData::new(Label(0), 0));
    s.get_node_mut(n).unwrap().set_property(10, a10);
    if let Some(v) = a11 {
        s.get_node_mut(n).unwrap().set_property(11, v);
    }
    find_bindings(&s.snapshot(), &rules[0], 0).len()
}

fn f(x: f64) -> PropertyValue {
    PropertyValue::Float(x)
}

#[test]
fn op_matrix_float_vs_float_both_directions() {
    // a > b
    assert_eq!(self_cmp_bindings("gt", f(2.0), Some(f(1.0))), 1);
    assert_eq!(self_cmp_bindings("gt", f(1.0), Some(f(2.0))), 0);
    // a < b
    assert_eq!(self_cmp_bindings("lt", f(1.0), Some(f(2.0))), 1);
    assert_eq!(self_cmp_bindings("lt", f(2.0), Some(f(1.0))), 0);
    // a >= b
    assert_eq!(self_cmp_bindings("gte", f(2.0), Some(f(1.0))), 1);
    assert_eq!(self_cmp_bindings("gte", f(1.0), Some(f(1.0))), 1);
    assert_eq!(self_cmp_bindings("gte", f(1.0), Some(f(2.0))), 0);
    // a <= b
    assert_eq!(self_cmp_bindings("lte", f(1.0), Some(f(2.0))), 1);
    assert_eq!(self_cmp_bindings("lte", f(1.0), Some(f(1.0))), 1);
    assert_eq!(self_cmp_bindings("lte", f(2.0), Some(f(1.0))), 0);
    // a == b
    assert_eq!(self_cmp_bindings("eq", f(1.0), Some(f(1.0))), 1);
    assert_eq!(self_cmp_bindings("eq", f(1.0), Some(f(2.0))), 0);
    // a != b
    assert_eq!(self_cmp_bindings("ne", f(1.0), Some(f(2.0))), 1);
    assert_eq!(self_cmp_bindings("ne", f(1.0), Some(f(1.0))), 0);
}

#[test]
fn variant_mismatch_fails_guard_no_fire() {
    // key_a = Int, key_b = Float ⟹ no coercion ⟹ guard false ⟹ no binding.
    assert_eq!(
        self_cmp_bindings("eq", PropertyValue::Int(1), Some(f(1.0))),
        0,
        "Int-vs-Float must not coerce; the guard must fail and the rewrite must not fire"
    );
    // Even 'ne' fails on a variant mismatch (compare returns false, not true).
    assert_eq!(self_cmp_bindings("ne", PropertyValue::Int(1), Some(f(2.0))), 0);
}

#[test]
fn missing_property_fails_guard() {
    // key_b absent on the node ⟹ guard false.
    assert_eq!(self_cmp_bindings("gt", f(5.0), None), 0);
}

#[test]
fn single_node_no_edge_pattern_matches_every_node() {
    // The S3.1 bootstrap/influx pattern shape: LHS with nodes and no edges
    // matches every live node. Guard true on both here ⟹ two bindings.
    let toml = r#"
[[rules]]
name = "cmp"
[rules.lhs]
nodes = ["a"]
edges = []
[[rules.lhs.node_property_compares]]
var_a = "a"
key_a = 10
op = "gt"
var_b = "a"
key_b = 11
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut s = HypergraphStore::new();
    for _ in 0..2 {
        let n = s.add_node(NodeData::new(Label(0), 0));
        s.get_node_mut(n).unwrap().set_property(10, f(9.0));
        s.get_node_mut(n).unwrap().set_property(11, f(1.0));
    }
    // A third node whose guard is FALSE must be excluded.
    let n3 = s.add_node(NodeData::new(Label(0), 0));
    s.get_node_mut(n3).unwrap().set_property(10, f(1.0));
    s.get_node_mut(n3).unwrap().set_property(11, f(9.0));

    assert_eq!(find_bindings(&s.snapshot(), &rules[0], 0).len(), 2);
}

#[test]
fn cross_variable_comparison_orientation_pinned() {
    // Cross-node compare `carrier.energy(1) >= carrier... ` analogue:
    // a[1] >= b[1], with labels pinning a=n0(energy 10), b=n1(energy 3).
    let toml = r#"
[[rules]]
name = "cross"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.lhs.node_labels]
a = 1
b = 2
[[rules.lhs.node_property_compares]]
var_a = "a"
key_a = 1
op = "gte"
var_b = "b"
key_b = 1
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();

    // a(10) >= b(3) ⟹ match.
    let mut s = HypergraphStore::new();
    let a = s.add_node(NodeData::new(Label(1), 0));
    let b = s.add_node(NodeData::new(Label(2), 0));
    s.get_node_mut(a).unwrap().set_property(1, f(10.0));
    s.get_node_mut(b).unwrap().set_property(1, f(3.0));
    s.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a, b])));
    assert_eq!(find_bindings(&s.snapshot(), &rules[0], 0).len(), 1);

    // a(3) >= b(10) is false ⟹ no match. Labels forbid the reverse binding.
    let mut s2 = HypergraphStore::new();
    let a2 = s2.add_node(NodeData::new(Label(1), 0));
    let b2 = s2.add_node(NodeData::new(Label(2), 0));
    s2.get_node_mut(a2).unwrap().set_property(1, f(3.0));
    s2.get_node_mut(b2).unwrap().set_property(1, f(10.0));
    s2.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[a2, b2])));
    assert_eq!(find_bindings(&s2.snapshot(), &rules[0], 0).len(), 0);
}
