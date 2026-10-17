//! property_effects apply semantics (B6 tests 4–10): the canonical
//! node-funded mechanic, order sensitivity, skip-on-mismatch, saturating
//! arithmetic, creation-rule effects, jitter determinism, and the
//! zero-draw guarantee.

use hg_core::{HyperEdge, HypergraphStore, Label, NodeData, PropertyValue};
use hg_engine::{apply_binding, find_bindings, Engine};
use hg_rules::{compile_rules, parse_rules_toml, StepMode};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use smallvec::SmallVec;

const ENERGY: u32 = 1;

/// The canonical funding rule, with the two effects in `order`.
fn funding_rule_toml(effects: &str) -> String {
    format!(
        r#"
[[rules]]
name = "fund_child"

[rules.lhs]
nodes = ["parent", "other"]
edges = [["parent", "other"]]

[rules.lhs.node_labels]
parent = 1

[[rules.lhs.node_property_guards]]
var = "parent"
property = 1
op = "gt"
value = 100.0

[rules.rhs]
nodes = ["parent", "other", "child"]
edges = [["parent", "child"], ["parent", "other"]]

{effects}
"#
    )
}

const SCALE_THEN_SUBTRACT: &str = r#"
[[rules.rhs.property_effects]]
var = "child"
key = 1
op = "scale_from"
source_var = "parent"
source_key = 1
factor = 0.8

[[rules.rhs.property_effects]]
var = "parent"
key = 1
op = "subtract"
value = 100.0
"#;

const SUBTRACT_THEN_SCALE: &str = r#"
[[rules.rhs.property_effects]]
var = "parent"
key = 1
op = "subtract"
value = 100.0

[[rules.rhs.property_effects]]
var = "child"
key = 1
op = "scale_from"
source_var = "parent"
source_key = 1
factor = 0.8
"#;

/// World: parent(label 1, energy) — other(label 0), plus an edgeless
/// bystander that can never match.
fn world(parent_energy: PropertyValue) -> (HypergraphStore, hg_core::NodeId, hg_core::NodeId) {
    let mut store = HypergraphStore::new();
    let mut parent_data = NodeData::new(Label(1), 0);
    parent_data.set_property(ENERGY, parent_energy);
    let parent = store.add_node(parent_data);
    let other = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[parent, other])));
    let mut bystander_data = NodeData::new(Label(9), 0);
    bystander_data.set_property(ENERGY, PropertyValue::Float(50.0));
    let bystander = store.add_node(bystander_data);
    (store, parent, bystander)
}

fn apply_funding(
    store: &mut HypergraphStore,
    effects: &str,
    seed: u64,
) -> (hg_core::NodeId, u64, u64) {
    let rules = compile_rules(&parse_rules_toml(&funding_rule_toml(effects)).unwrap()).unwrap();
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    assert_eq!(bindings.len(), 1, "guard energy>100 matched exactly once");
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let (event, applied, skipped) = apply_binding(store, &rules[0], &bindings[0], &mut rng);
    assert_eq!(event.nodes_created.len(), 1, "child created");
    (event.nodes_created[0], applied, skipped)
}

/// Test 4 — the canonical node-funded mechanic, exact numbers:
/// parent 200.0 → child = 160.0 (scale reads PRE-subtract), parent 100.0.
#[test]
fn canonical_node_funded_mechanic() {
    let (mut store, parent, bystander) = world(PropertyValue::Float(200.0));
    let (child, applied, skipped) = apply_funding(&mut store, SCALE_THEN_SUBTRACT, 0);

    let energy = |id| store.get_node(id).unwrap().get_property(ENERGY);
    println!(
        "node-funded mechanic: parent {:?} -> child {:?}, parent {:?}, bystander {:?} ({applied} applied / {skipped} skipped)",
        PropertyValue::Float(200.0),
        energy(child),
        energy(parent),
        energy(bystander),
    );
    assert_eq!(energy(child), Some(PropertyValue::Float(160.0)), "child funded at 0.8x pre-subtract");
    assert_eq!(energy(parent), Some(PropertyValue::Float(100.0)), "parent paid 100");
    assert_eq!(energy(bystander), Some(PropertyValue::Float(50.0)), "bystander untouched");
    assert_eq!((applied, skipped), (2, 0));
}

/// Test 5 — order-sensitivity pin: swapped declaration order → child 80.
/// Sequential reads-see-earlier-writes is load-bearing, not incidental.
#[test]
fn effect_order_is_load_bearing() {
    let (mut store, parent, _) = world(PropertyValue::Float(200.0));
    let (child, applied, skipped) = apply_funding(&mut store, SUBTRACT_THEN_SCALE, 0);

    let energy = |id| store.get_node(id).unwrap().get_property(ENERGY);
    assert_eq!(energy(parent), Some(PropertyValue::Float(100.0)));
    assert_eq!(
        energy(child),
        Some(PropertyValue::Float(80.0)),
        "scale AFTER subtract reads the post-subtract 100"
    );
    assert_eq!((applied, skipped), (2, 0));
}

/// Test 6 — mismatch skip: scale_from on an Int source is skipped and
/// counted; the rewrite completes; the other effect still applies.
#[test]
fn mismatch_skips_one_effect_not_the_rewrite() {
    let (mut store, parent, _) = world(PropertyValue::Int(200));
    // Same shape as the funding rule but guardless: the Float guard
    // would (correctly) refuse the Int parent before effects ever ran,
    // and this test targets the runtime-mismatch path.
    let toml = r#"
[[rules]]
name = "fund_child_int"

[rules.lhs]
nodes = ["parent", "other"]
edges = [["parent", "other"]]

[rules.lhs.node_labels]
parent = 1

[rules.rhs]
nodes = ["parent", "other", "child"]
edges = [["parent", "child"], ["parent", "other"]]

[[rules.rhs.property_effects]]
var = "child"
key = 1
op = "scale_from"
source_var = "parent"
source_key = 1
factor = 0.8

[[rules.rhs.property_effects]]
var = "parent"
key = 1
op = "subtract"
value = 100
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    assert_eq!(bindings.len(), 1);
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let (event, applied, skipped) = apply_binding(&mut store, &rules[0], &bindings[0], &mut rng);
    let child = event.nodes_created[0];
    assert_eq!(skipped, 1, "scale_from on Int source skipped");
    assert_eq!(applied, 1, "subtract still applied");
    let energy = |id: hg_core::NodeId| store.get_node(id).unwrap().get_property(ENERGY);
    assert_eq!(energy(parent), Some(PropertyValue::Int(100)));
    assert_eq!(store.get_node(child).unwrap().get_property(ENERGY), None);
    // Rewrite completed: child exists with its RHS edge.
    assert!(store.edges_incident_to(child).len() == 1);
}

/// Test 7 — Int arithmetic saturates instead of panicking.
#[test]
fn int_subtract_saturates() {
    let (mut store, parent, _) = world(PropertyValue::Int(i64::MIN + 5));
    // Int(i64::MIN+5) fails the Float guard — use a guardless variant.
    let toml = r#"
[[rules]]
name = "drain"

[rules.lhs]
nodes = ["parent", "other"]
edges = [["parent", "other"]]

[rules.lhs.node_labels]
parent = 1

[rules.rhs]
nodes = ["parent", "other"]
edges = [["parent", "other"]]

[[rules.rhs.property_effects]]
var = "parent"
key = 1
op = "subtract"
value = 100
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let (_, applied, skipped) = apply_binding(&mut store, &rules[0], &bindings[0], &mut rng);
    assert_eq!((applied, skipped), (1, 0));
    assert_eq!(
        store.get_node(parent).unwrap().get_property(ENERGY),
        Some(PropertyValue::Int(i64::MIN)),
        "saturated, no overflow panic"
    );
}

/// Test 8 — creation rule with a set effect: the scheduled firing
/// creates the node with the property already set.
#[test]
fn creation_rule_seeds_initial_properties() {
    let toml = r#"
[[rules]]
name = "seeder"

[rules.schedule]
mode = "once"

[rules.rhs]
nodes = ["seed"]
edges = []

[rules.rhs.node_labels]
seed = 3

[[rules.rhs.property_effects]]
var = "seed"
key = 1
op = "set"
value = 42.5
"#;
    let rules = compile_rules(&parse_rules_toml(toml).unwrap()).unwrap();
    let mut engine = Engine::new(rules, StepMode::MaxParallel, 7, 100);
    let mut store = HypergraphStore::new();
    let stats = engine.step(&mut store);
    assert_eq!(stats.property_effects_applied, 1);
    let seeded: Vec<_> = store
        .nodes()
        .filter(|(_, d)| d.label == Label(3))
        .collect();
    assert_eq!(seeded.len(), 1);
    assert_eq!(
        seeded[0].1.get_property(ENERGY),
        Some(PropertyValue::Float(42.5)),
        "property present at creation"
    );
}

const JITTER_EFFECT: &str = r#"
[[rules.rhs.property_effects]]
var = "parent"
key = 1
op = "jitter"
magnitude = 0.05
"#;

/// Test 9 — jitter determinism: same seed → bit-identical; different
/// seed → different; |delta| <= magnitude always.
#[test]
fn jitter_is_seeded_and_bounded() {
    let run = |seed: u64| -> f64 {
        let (mut store, parent, _) = world(PropertyValue::Float(200.0));
        apply_funding(&mut store, JITTER_EFFECT, seed);
        match store.get_node(parent).unwrap().get_property(ENERGY) {
            Some(PropertyValue::Float(f)) => f,
            other => panic!("expected Float, got {other:?}"),
        }
    };
    let a1 = run(42);
    let a2 = run(42);
    let b = run(43);
    assert_eq!(a1.to_bits(), a2.to_bits(), "same seed => bit-identical");
    assert_ne!(a1.to_bits(), b.to_bits(), "different seed => different");
    for seed in 0..50 {
        let v = run(seed);
        assert!(
            (v - 200.0).abs() <= 0.05 + f64::EPSILON,
            "|delta| bounded by magnitude, got {v}"
        );
    }
}

/// Test 10 — the zero-draw guarantee: applying NON-jitter effects
/// consumes no RNG. If this ever regresses, every existing seeded run's
/// stream shifts the moment a ruleset carries effects.
#[test]
fn non_jitter_effects_consume_no_rng() {
    let (mut store, _, _) = world(PropertyValue::Float(200.0));
    let rules =
        compile_rules(&parse_rules_toml(&funding_rule_toml(SCALE_THEN_SUBTRACT)).unwrap())
            .unwrap();
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut control = ChaCha8Rng::seed_from_u64(42);
    let (_, applied, _) = apply_binding(&mut store, &rules[0], &bindings[0], &mut rng);
    assert_eq!(applied, 2, "effects really ran");
    assert_eq!(
        rng.next_u64(),
        control.next_u64(),
        "apply with non-jitter effects must not advance the stream"
    );

    // And a jitter effect DOES draw (the counter-case that proves the
    // comparison above can fail).
    let (mut store2, _, _) = world(PropertyValue::Float(200.0));
    let rules2 =
        compile_rules(&parse_rules_toml(&funding_rule_toml(JITTER_EFFECT)).unwrap()).unwrap();
    let bindings2 = find_bindings(&store2.snapshot(), &rules2[0], 0);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    let mut control2 = ChaCha8Rng::seed_from_u64(42);
    apply_binding(&mut store2, &rules2[0], &bindings2[0], &mut rng2);
    assert_ne!(
        rng2.next_u64(),
        control2.next_u64(),
        "jitter must consume from the stream"
    );
}

/// New ops (genome-scaled effect, deviation #4): `add_from_property` /
/// `subtract_from_property` move a PROPERTY-valued amount, and are exactly
/// conservative when the same source drives a paired add + subtract (the harvest
/// transfer). A non-Float or missing source skips that one effect, not the rewrite.
const RATE: u32 = 2;
fn world_with_rate(energy: f64, rate: PropertyValue) -> (HypergraphStore, hg_core::NodeId) {
    let mut store = HypergraphStore::new();
    let mut p = NodeData::new(Label(1), 0);
    p.set_property(ENERGY, PropertyValue::Float(energy));
    p.set_property(RATE, rate);
    let parent = store.add_node(p);
    let other = store.add_node(NodeData::new(Label(0), 0));
    store.add_edge(HyperEdge::new(Label(0), SmallVec::from_slice(&[parent, other])));
    (store, parent)
}

const TRANSFER_RATE: &str = r#"
[[rules.rhs.property_effects]]
var = "child"
key = 1
op = "set"
value = 0.0

[[rules.rhs.property_effects]]
var = "child"
key = 1
op = "add_from_property"
source_var = "parent"
source_key = 2

[[rules.rhs.property_effects]]
var = "parent"
key = 1
op = "subtract_from_property"
source_var = "parent"
source_key = 2
"#;

#[test]
fn add_and_subtract_from_property_are_conservative() {
    let (mut store, parent) = world_with_rate(200.0, PropertyValue::Float(7.5));
    let rules = compile_rules(&parse_rules_toml(&funding_rule_toml(TRANSFER_RATE)).unwrap()).unwrap();
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    assert_eq!(bindings.len(), 1);
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let (event, applied, skipped) = apply_binding(&mut store, &rules[0], &bindings[0], &mut rng);
    let child = event.nodes_created[0];
    let e = |id| store.get_node(id).unwrap().get_property(ENERGY);
    // child gained exactly `rate`; parent lost exactly `rate` — conservative.
    assert_eq!(e(child), Some(PropertyValue::Float(7.5)), "child += parent.rate");
    assert_eq!(e(parent), Some(PropertyValue::Float(192.5)), "parent -= parent.rate");
    assert_eq!((applied, skipped), (3, 0));
}

#[test]
fn from_property_skips_non_float_source_not_the_rewrite() {
    // rate is an Int ⟹ the two _from_property effects skip; the `set 0.0` still runs.
    let (mut store, parent) = world_with_rate(200.0, PropertyValue::Int(7));
    let rules = compile_rules(&parse_rules_toml(&funding_rule_toml(TRANSFER_RATE)).unwrap()).unwrap();
    let bindings = find_bindings(&store.snapshot(), &rules[0], 0);
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let (event, applied, skipped) = apply_binding(&mut store, &rules[0], &bindings[0], &mut rng);
    let child = event.nodes_created[0];
    let e = |id| store.get_node(id).unwrap().get_property(ENERGY);
    assert_eq!(e(child), Some(PropertyValue::Float(0.0)), "only the set ran on child");
    assert_eq!(e(parent), Some(PropertyValue::Float(200.0)), "parent untouched (subtract skipped)");
    assert_eq!((applied, skipped), (1, 2), "1 set applied, 2 from_property skipped");
}
