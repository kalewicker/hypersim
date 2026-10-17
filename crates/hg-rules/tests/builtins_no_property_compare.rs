//! S1.3 precondition: the bit-identical claim for PropertyCompare rests on the
//! fact that no existing builtin USES the new guard. This asserts exactly that,
//! unambiguously and fast — every builtin compiles to a pattern with an empty
//! `property_compares`, so the new (short-circuiting) guard arm never runs for
//! them and cannot alter their match set. Combined with the two genuine
//! pre-change full-run baselines (property_effects_regression.rs:
//! triangle_subdivide + glider, recorded 2026-07-09), this covers the leak
//! vectors the review named: an enum-discriminant/serialization shift or a
//! match-order perturbation would show up either here (non-empty compares) or
//! in those hashes (drift). No RNG is involved in guard evaluation.

use hg_rules::{builtin_rules, compile_rules};

const BUILTINS: &[&str] = &[
    "triangle_subdivide",
    "binary_edge",
    "tetrahedral",
    "glider",
    "growth_treadmill",
];

#[test]
fn no_builtin_uses_property_compare() {
    for &name in BUILTINS {
        let rules = builtin_rules(name).unwrap_or_else(|e| panic!("builtin '{name}': {e}"));
        // Raw parse form: no rule carries a property-compare guard.
        for r in &rules {
            assert!(
                r.lhs_node_property_compares.is_empty(),
                "builtin '{name}' rule '{}' declares a node_property_compare — the \
bit-identical precondition assumes none do",
                r.name
            );
        }
        // Compiled form: every pattern's property_compares vec is empty, so the
        // S1 guard arm short-circuits and the match set is unchanged.
        let compiled = compile_rules(&rules).unwrap_or_else(|e| panic!("builtin '{name}': {e:?}"));
        for cr in &compiled {
            assert!(
                cr.pattern.property_compares.is_empty(),
                "builtin '{name}' rule '{}' compiled to a non-empty property_compares",
                cr.name
            );
        }
    }
}
