//! Phase 4 exit criteria (v4 §13): fixture rule sets parse and compile;
//! sparse edge_labels on both sides; creation-rule schedule validation in
//! both directions; auto-generated VarsDistinct pairs on symmetric and
//! asymmetric patterns; PropertyValue-typed guards.

use hg_core::{Label, PropertyOp, PropertyValue};
use hg_rules::{
    builtin_rules, compile_rule, compile_rules, parse_rules_toml, ParseError, Schedule,
    BUILTIN_NAMES,
};

#[test]
fn all_builtin_rule_sets_parse_and_compile() {
    for name in BUILTIN_NAMES {
        let rules = builtin_rules(name).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(!rules.is_empty(), "{name} empty");
        let compiled = compile_rules(&rules).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(compiled.len(), rules.len());
    }
}

#[test]
fn glider_var_guards_actually_load() {
    // Regression: the old parser silently ignored [rules.lhs.var_guards].
    let rules = builtin_rules("glider").unwrap();
    let compiled = compile_rule(&rules[0]).unwrap();
    // 3 distinct vars -> 3 auto-generated pairs.
    assert_eq!(compiled.pattern.distinct_pairs.len(), 3);
}

#[test]
fn sparse_edge_labels_lhs_and_rhs() {
    let toml = r#"
[[rules]]
name = "edge_labels"
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
[rules.lhs.edge_labels]
0 = 7
[rules.rhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"], ["a", "c"]]
[rules.rhs.edge_labels]
2 = 5
"#;
    let rules = parse_rules_toml(toml).unwrap();
    let c = compile_rule(&rules[0]).unwrap();
    // LHS: edge 0 constrained to 7, edge 1 unconstrained — the case a
    // parallel Vec<Option<u32>> could not express from TOML.
    assert_eq!(c.pattern.edges[0].label, Some(Label(7)));
    assert_eq!(c.pattern.edges[1].label, None);
    // RHS: edge 2 labelled 5, edges 0/1 default 0.
    assert_eq!(c.template.rhs_edges[0].1, Label(0));
    assert_eq!(c.template.rhs_edges[1].1, Label(0));
    assert_eq!(c.template.rhs_edges[2].1, Label(5));
}

#[test]
fn edge_label_index_out_of_range_rejected() {
    let toml = r#"
[[rules]]
name = "bad"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.lhs.edge_labels]
3 = 1
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
"#;
    assert!(matches!(
        parse_rules_toml(toml),
        Err(ParseError::EdgeLabelIndexOutOfRange(_, 3, 1))
    ));
}

#[test]
fn creation_rule_requires_schedule() {
    let toml = r#"
[[rules]]
name = "spawn"
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
"#;
    assert!(matches!(
        parse_rules_toml(toml),
        Err(ParseError::CreationRuleMissingSchedule(_))
    ));
}

#[test]
fn non_creation_rule_rejects_schedule() {
    let toml = r#"
[[rules]]
name = "notcreation"
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]
[rules.rhs]
nodes = ["a"]
edges = []
[rules.schedule]
mode = "once"
"#;
    assert!(matches!(
        parse_rules_toml(toml),
        Err(ParseError::NonCreationRuleWithSchedule(_))
    ));
}

#[test]
fn creation_rule_schedules_parse() {
    let toml = r#"
[[rules]]
name = "spawn_once"
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["a", "b", "c"]
edges = [["a", "b", "c"]]
[rules.schedule]
mode = "once"

[[rules]]
name = "spawn_periodic"
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["x"]
edges = []
[rules.schedule]
mode = "every_n_steps"
n = 1000
"#;
    let rules = parse_rules_toml(toml).unwrap();
    assert_eq!(rules[0].schedule, Some(Schedule::Once));
    assert_eq!(rules[1].schedule, Some(Schedule::EveryNSteps(1000)));
    assert!(rules[0].schedule.unwrap().is_due(0));
    assert!(!rules[0].schedule.unwrap().is_due(1));
    assert!(rules[1].schedule.unwrap().is_due(0));
    assert!(rules[1].schedule.unwrap().is_due(2000));
    assert!(!rules[1].schedule.unwrap().is_due(2001));
}

#[test]
fn property_guards_typed_by_toml_literal_syntax() {
    let toml = r#"
[[rules]]
name = "guarded"
[rules.lhs]
nodes = ["a"]
edges = []

[[rules.lhs.node_property_guards]]
var = "a"
property = 1
op = "gt"
value = 100.0

[[rules.lhs.node_property_guards]]
var = "a"
property = 3
op = "eq"
value = 100

[[rules.lhs.node_property_guards]]
var = "a"
property = 2
op = "eq"
value = true

[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let rules = parse_rules_toml(toml).unwrap();
    let g = &rules[0].lhs_node_property_guards;
    assert_eq!(g[0].value, PropertyValue::Float(100.0));
    assert_eq!(g[0].op, PropertyOp::Gt);
    assert_eq!(g[1].value, PropertyValue::Int(100));
    assert_eq!(g[2].value, PropertyValue::Bool(true));
}

#[test]
fn vars_distinct_symmetric_and_asymmetric() {
    // Symmetric: all three vars in one guard -> 3 pairs.
    let sym = r#"
[[rules]]
name = "sym"
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
    let c = compile_rule(&parse_rules_toml(sym).unwrap()[0]).unwrap();
    assert_eq!(c.pattern.distinct_pairs, vec![(0, 1), (0, 2), (1, 2)]);

    // Asymmetric: only a subset constrained -> 1 pair; b==c still allowed.
    let asym = r#"
[[rules]]
name = "asym"
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "c"]
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let c = compile_rule(&parse_rules_toml(asym).unwrap()[0]).unwrap();
    assert_eq!(c.pattern.distinct_pairs, vec![(0, 2)]);
}

#[test]
fn file_order_is_rule_id_order() {
    let toml = r#"
[[rules]]
name = "low"
priority = 0
[rules.lhs]
nodes = ["a"]
edges = []
[rules.rhs]
nodes = ["a"]
edges = []

[[rules]]
name = "high"
priority = 9
[rules.lhs]
nodes = ["a"]
edges = []
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    let rules = parse_rules_toml(toml).unwrap();
    // Priority must NOT reorder rules: rule_id = file index, and conflict
    // resolution honours priority via Reverse<i32> in its sort key instead.
    assert_eq!(rules[0].name, "low");
    assert_eq!(rules[1].name, "high");
}

#[test]
fn duplicate_and_undeclared_vars_rejected() {
    let dup = r#"
[[rules]]
name = "dup"
[rules.lhs]
nodes = ["a", "a"]
edges = []
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    assert!(matches!(
        parse_rules_toml(dup),
        Err(ParseError::DuplicateVariable(..))
    ));

    let undecl = r#"
[[rules]]
name = "undecl"
[rules.lhs]
nodes = ["a"]
edges = [["a", "zz"]]
[rules.rhs]
nodes = ["a"]
edges = []
"#;
    assert!(matches!(
        parse_rules_toml(undecl),
        Err(ParseError::UndeclaredVariable(..))
    ));
}

#[test]
fn template_maps_survivors_creations_deletions() {
    let toml = r#"
[[rules]]
name = "glider_like"
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b"], ["b", "c"]]
[rules.rhs]
nodes = ["b", "c", "d"]
edges = [["b", "c"], ["c", "d"]]
[rules.rhs.node_labels]
d = 1
"#;
    let c = compile_rule(&parse_rules_toml(toml).unwrap()[0]).unwrap();
    // RHS b,c survive from LHS ixs 1,2; d is fresh with label 1.
    assert_eq!(c.template.rhs_from_lhs, vec![Some(1), Some(2), None]);
    assert_eq!(c.template.rhs_new_labels[2], Label(1));
    // LHS a (ix 0) is deleted.
    assert_eq!(c.template.lhs_deleted, vec![0]);
}
