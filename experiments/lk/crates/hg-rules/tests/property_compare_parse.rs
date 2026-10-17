//! S1.4 — parsing of `node_property_compares` (hg-alife scope §5.3):
//! nested and legacy spellings, LHS-variable validation, typo rejection.

use hg_rules::parse_rules_toml;

#[test]
fn nested_spelling_parses() {
    let toml = r#"
[[rules]]
name = "r"
[rules.lhs]
nodes = ["carrier"]
edges = []
[[rules.lhs.node_property_compares]]
var_a = "carrier"
key_a = 1
op = "lt"
var_b = "carrier"
key_b = 13
[rules.rhs]
nodes = ["carrier"]
edges = []
"#;
    let rules = parse_rules_toml(toml).unwrap();
    assert_eq!(rules[0].lhs_node_property_compares.len(), 1);
    let c = &rules[0].lhs_node_property_compares[0];
    assert_eq!(c.var_a, "carrier");
    assert_eq!(c.key_a, 1);
    assert_eq!(c.var_b, "carrier");
    assert_eq!(c.key_b, 13);
}

#[test]
fn legacy_top_level_spelling_parses() {
    let toml = r#"
[[rules]]
name = "r"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[[rules.lhs_node_property_compares]]
var_a = "a"
key_a = 10
op = "ge"
var_b = "b"
key_b = 10
"#;
    // 'ge' is not a valid op token (only lt/gt/lte/gte/eq/ne); confirm the
    // op vocabulary matches the existing guards.
    let err = parse_rules_toml(toml).unwrap_err();
    assert!(
        err.to_string().contains("Unknown op"),
        "expected unknown-op error, got: {err}"
    );
}

#[test]
fn legacy_top_level_valid_op_parses() {
    let toml = r#"
[[rules]]
name = "r"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[[rules.lhs_node_property_compares]]
var_a = "a"
key_a = 10
op = "gte"
var_b = "b"
key_b = 10
"#;
    let rules = parse_rules_toml(toml).unwrap();
    assert_eq!(rules[0].lhs_node_property_compares.len(), 1);
}

#[test]
fn undeclared_var_a_is_rejected() {
    let toml = r#"
[[rules]]
name = "r"
[rules.lhs]
nodes = ["a"]
edges = []
[[rules.lhs.node_property_compares]]
var_a = "ghost"
key_a = 1
op = "gt"
var_b = "a"
key_b = 2
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let err = parse_rules_toml(toml).unwrap_err();
    assert!(
        err.to_string().contains("var_a 'ghost'"),
        "expected undeclared var_a error, got: {err}"
    );
}

#[test]
fn undeclared_var_b_is_rejected() {
    let toml = r#"
[[rules]]
name = "r"
[rules.lhs]
nodes = ["a"]
edges = []
[[rules.lhs.node_property_compares]]
var_a = "a"
key_a = 1
op = "gt"
var_b = "ghost"
key_b = 2
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let err = parse_rules_toml(toml).unwrap_err();
    assert!(
        err.to_string().contains("var_b 'ghost'"),
        "expected undeclared var_b error, got: {err}"
    );
}

#[test]
fn typoed_field_is_rejected() {
    // deny_unknown_fields: a misspelled field is a parse error, not a no-op.
    let toml = r#"
[[rules]]
name = "r"
[rules.lhs]
nodes = ["a"]
edges = []
[[rules.lhs.node_property_compares]]
var_a = "a"
keyA = 1
op = "gt"
var_b = "a"
key_b = 2
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    assert!(parse_rules_toml(toml).is_err());
}
