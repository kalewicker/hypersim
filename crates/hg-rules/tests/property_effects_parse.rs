//! property_effects grammar tests (hypersim_property_effects_and_ei_docs.md
//! B6 tests 1–3): round-trip each op, every rejection is a parse error
//! with an explanatory message, and the absent-array case is exactly
//! today's behavior.

use hg_core::PropertyValue;
use hg_rules::{
    builtin_rules, compile_rules, parse_rules_toml, CompiledEffectOp, ParseError, BUILTIN_NAMES,
};

fn base_rule(effects: &str) -> String {
    format!(
        r#"
[[rules]]
name = "fund_child"

[rules.lhs]
nodes = ["parent", "other"]
edges = [["parent", "other"]]

[rules.rhs]
nodes = ["parent", "other", "child"]
edges = [["parent", "child"], ["parent", "other"]]

{effects}
"#
    )
}

/// Test 1: every op round-trips from TOML with correct compiled indices.
#[test]
fn all_ops_round_trip_with_compiled_indices() {
    let toml = base_rule(
        r#"
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

[[rules.rhs.property_effects]]
var = "child"
key = 2
op = "set"
value = 7

[[rules.rhs.property_effects]]
var = "child"
key = 3
op = "copy_from"
source_var = "other"
source_key = 3

[[rules.rhs.property_effects]]
var = "other"
key = 1
op = "add"
value = 5.0

[[rules.rhs.property_effects]]
var = "child"
key = 2
op = "jitter"
magnitude = 0.05
"#,
    );
    let rules = parse_rules_toml(&toml).expect("parses");
    let compiled = compile_rules(&rules).expect("compiles");
    let fx = &compiled[0].template.property_effects;
    assert_eq!(fx.len(), 6, "declaration order preserved, none dropped");

    // RHS indices: parent=0, other=1, child=2 (declared order).
    assert_eq!(fx[0].target_ix, 2);
    assert_eq!(fx[0].key, 1);
    assert!(matches!(
        fx[0].op,
        CompiledEffectOp::ScaleFrom { source_ix: 0, source_key: 1, factor } if factor == 0.8
    ));
    assert_eq!(fx[1].target_ix, 0);
    assert!(matches!(
        fx[1].op,
        CompiledEffectOp::Subtract { value: PropertyValue::Float(v) } if v == 100.0
    ));
    assert!(matches!(
        fx[2].op,
        CompiledEffectOp::Set { value: PropertyValue::Int(7) }
    ));
    assert!(matches!(
        fx[3].op,
        CompiledEffectOp::CopyFrom { source_ix: 1, source_key: 3 }
    ));
    assert!(matches!(
        fx[4].op,
        CompiledEffectOp::Add { value: PropertyValue::Float(v) } if v == 5.0
    ));
    assert!(matches!(
        fx[5].op,
        CompiledEffectOp::Jitter { magnitude } if magnitude == 0.05
    ));
}

/// Test 2a: target var not declared in RHS at all.
#[test]
fn rejects_target_not_in_rhs() {
    let toml = base_rule(
        r#"
[[rules.rhs.property_effects]]
var = "ghost"
key = 1
op = "set"
value = 1
"#,
    );
    let err = parse_rules_toml(&toml).unwrap_err();
    assert!(matches!(err, ParseError::InvalidPropertyEffect(..)));
    assert!(err.to_string().contains("not declared in RHS"));
}

/// Test 2b: source_var naming an LHS-only (deleted) variable — the error
/// must EXPLAIN why (the node is gone before effects run).
#[test]
fn rejects_lhs_only_source_var_with_explanation() {
    let toml = r#"
[[rules]]
name = "deleting"

[rules.lhs]
nodes = ["doomed", "keeper"]
edges = [["doomed", "keeper"]]

[rules.rhs]
nodes = ["keeper", "child"]
edges = [["keeper", "child"]]

[[rules.rhs.property_effects]]
var = "child"
key = 1
op = "copy_from"
source_var = "doomed"
source_key = 1
"#;
    let err = parse_rules_toml(toml).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("LHS-only"), "names the category: {msg}");
    assert!(
        msg.contains("deletion happens before property effects"),
        "explains WHY it is invalid: {msg}"
    );
}

/// Test 2c–2f: unknown op, missing op-specific fields, magnitude <= 0,
/// unknown field (deny_unknown_fields), effects under lhs.
#[test]
fn rejects_malformed_effects() {
    let cases: Vec<(String, &str)> = vec![
        (
            base_rule("[[rules.rhs.property_effects]]\nvar = \"child\"\nkey = 1\nop = \"frobnicate\"\n"),
            "unknown op",
        ),
        (
            base_rule("[[rules.rhs.property_effects]]\nvar = \"child\"\nkey = 1\nop = \"scale_from\"\nsource_var = \"parent\"\nsource_key = 1\n"),
            "requires field 'factor'",
        ),
        (
            base_rule("[[rules.rhs.property_effects]]\nvar = \"child\"\nkey = 1\nop = \"set\"\n"),
            "requires field 'value'",
        ),
        (
            base_rule("[[rules.rhs.property_effects]]\nvar = \"child\"\nkey = 1\nop = \"jitter\"\nmagnitude = 0.0\n"),
            "magnitude must be > 0",
        ),
        (
            base_rule("[[rules.rhs.property_effects]]\nvar = \"child\"\nkey = 1\nop = \"jitter\"\nmagnitude = -0.5\n"),
            "magnitude must be > 0",
        ),
        (
            base_rule("[[rules.lhs.property_effects]]\nvar = \"parent\"\nkey = 1\nop = \"set\"\nvalue = 1\n"),
            "belong under [rules.rhs]",
        ),
    ];
    for (toml, expect) in cases {
        let err = parse_rules_toml(&toml).unwrap_err();
        assert!(
            err.to_string().contains(expect),
            "expected '{expect}' in: {err}"
        );
    }

    // deny_unknown_fields: a typo'd field is a TOML error, not a no-op.
    let toml = base_rule(
        "[[rules.rhs.property_effects]]\nvar = \"child\"\nkey = 1\nop = \"set\"\nvalue = 1\nfactr = 2.0\n",
    );
    assert!(parse_rules_toml(&toml).is_err(), "unknown field must error");
}

/// Test 3: absent array = empty effects; ALL builtins parse unchanged
/// with zero effects (the falsifiable "exactly today's behavior" claim
/// pairs with the canonical-hash regression in hg-sim-core).
#[test]
fn absent_array_is_empty_and_builtins_unchanged() {
    let toml = base_rule("");
    let rules = parse_rules_toml(&toml).expect("parses without effects");
    assert!(rules[0].rhs_property_effects.is_empty());
    let compiled = compile_rules(&rules).expect("compiles");
    assert!(compiled[0].template.property_effects.is_empty());

    for name in BUILTIN_NAMES {
        let rules = builtin_rules(name).unwrap_or_else(|e| panic!("builtin {name}: {e}"));
        for r in &rules {
            assert!(
                r.rhs_property_effects.is_empty(),
                "builtin '{name}' rule '{}' must carry no effects",
                r.name
            );
        }
        compile_rules(&rules).unwrap_or_else(|e| panic!("builtin {name} compiles: {e}"));
    }
}
