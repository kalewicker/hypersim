//! Compile-stage tests: CandidateRule → Rule → CompiledRule round-trip,
//! the behavioral core of G-R2 (expressivity measured against the shipped
//! grammar, one fixture at a time).

use crate::compile::{candidate_to_rule, var_name};
use crate::schema::{CandidatePatternEdge, CandidateRule};
use hg_core::Label;
use hg_rules::compiled::compile_rule;
use hg_rules::rule::Rule;
use hg_rules::PatternGraph;
use std::collections::BTreeMap;

fn triangle_candidate() -> CandidateRule {
    // §3.4 worked example as read back: LHS [a,b,c]; RHS [a,b,d],[b,c,d],[a,c,d].
    CandidateRule {
        priority: 0,
        lhs: vec![CandidatePatternEdge {
            slots: vec![0, 1, 2],
        }],
        rhs: vec![
            CandidatePatternEdge {
                slots: vec![0, 1, 3],
            },
            CandidatePatternEdge {
                slots: vec![1, 2, 3],
            },
            CandidatePatternEdge {
                slots: vec![0, 2, 3],
            },
        ],
        var_labels: vec![None; 4],
        guards: Vec::new(),
        var_guards: Vec::new(),
        effects: Vec::new(),
    }
}

fn triangle_reference() -> Rule {
    // The same rule as it would be written directly (A0/TOML path), with
    // the exact variable names candidate_to_rule generates. All four vars
    // are LHS-wired (a,b,c match; d is created via RHS-only wiring — but
    // d appears only on the RHS, so lhs.nodes carries just a,b,c).
    let names = |ixs: &[usize]| -> Vec<String> { ixs.iter().map(|i| var_name(*i)).collect() };
    let mut lhs = PatternGraph::new();
    lhs.nodes = (0..3).map(var_name).collect();
    lhs.edges = vec![names(&[0, 1, 2])];
    let mut rhs = PatternGraph::new();
    rhs.nodes = (0..4).map(var_name).collect();
    rhs.edges = vec![names(&[0, 1, 3]), names(&[1, 2, 3]), names(&[0, 2, 3])];
    Rule {
        name: String::new(),
        priority: 0,
        weight: 1.0,
        enabled: true,
        lhs,
        rhs,
        ..Rule::default()
    }
}

#[test]
fn gr2_triangle_compiles_and_matches_reference() {
    let compiled_from_region = compile_rule(&candidate_to_rule(&triangle_candidate()))
        .expect("region-derived rule compiles");
    let direct = compile_rule(&triangle_reference()).expect("reference rule compiles");

    // Behavioral identity proxy at the compiled level: identical match
    // pattern (var count + edge structure) on both sides. G-R3 upgrades
    // this to full-trajectory identity once the interpreter exists.
    assert_eq!(compiled_from_region.pattern.n_vars, direct.pattern.n_vars);
    assert_eq!(
        compiled_from_region.pattern.edges.len(),
        direct.pattern.edges.len()
    );
    for (a, b) in compiled_from_region
        .pattern
        .edges
        .iter()
        .zip(direct.pattern.edges.iter())
    {
        assert_eq!(
            a.var_ixs, b.var_ixs,
            "LHS/RHS edge var structure must match"
        );
        assert_eq!(a.label, b.label);
    }
}

#[test]
fn gr2_label_constraints_follow_side_wiring() {
    let mut cand = triangle_candidate();
    // v0: wired both sides → LHS constraint. v3: RHS-only (created) →
    // created-node label. v1: unconstrained.
    cand.var_labels = vec![Some(7), None, None, Some(9)];
    let rule = candidate_to_rule(&cand);
    assert_eq!(rule.lhs.node_labels.get("v0"), Some(&Label(7)));
    assert!(!rule.lhs.node_labels.contains_key("v3"));
    // Created node carries its label on the RHS (E1's route to
    // create-labeled-node rules, e.g. churn3-R1).
    assert_eq!(rule.rhs_node_labels.get("v3"), Some(&Label(9)));
    assert!(rule.rhs_node_labels.get("v0").is_none());
}

#[test]
fn gr2_priority_round_trips_negative() {
    let mut cand = triangle_candidate();
    cand.priority = -5;
    let rule = candidate_to_rule(&cand);
    assert_eq!(rule.priority, -5);
    assert!(compile_rule(&rule).is_ok());
}

#[test]
fn gr2_edge_labels_absent_by_envelope() {
    // E1 cannot express pattern-edge labels (plan §3.6); compiled candidates
    // must therefore carry NO edge labels rather than defaulting something.
    let mut cand = triangle_candidate();
    cand.lhs.push(CandidatePatternEdge { slots: vec![0] });
    let rule = candidate_to_rule(&cand);
    let expected: BTreeMap<usize, Label> = BTreeMap::new();
    assert_eq!(rule.lhs.edge_labels, expected);
}

#[test]
fn gr2_no_effects_no_schedule_by_envelope() {
    let rule = candidate_to_rule(&triangle_candidate());
    assert!(rule.rhs_property_effects.is_empty(), "E2 deferred");
    assert!(rule.schedule.is_none(), "SB-R2: schedules unencodable");
}

/// E1-G (§3.5), landed in R4: a guarded rule survives the full
/// install → read → compile path with its guard intact and correctly
/// translated (op 0=Gt, 1=Lt, 2=Eq).
#[test]
fn e1g_guard_survives_region_round_trip() {
    use crate::graph_source::GraphRuleSource;
    use crate::schema::{CandidateGuard, CandidatePatternEdge, CandidateRule};
    use hg_core::{HypergraphStore, PropertyValue};
    use hg_rules::source::RuleSource;

    let cand = CandidateRule {
        priority: 0,
        lhs: vec![CandidatePatternEdge { slots: vec![0] }],
        rhs: vec![],
        var_labels: vec![None],
        guards: vec![CandidateGuard {
            var: 0,
            key: 1, // energy mirror key used by selfmod rules
            op: 1,  // Lt
            val: PropertyValue::Float(0.5),
        }],
        var_guards: Vec::new(),
        effects: Vec::new(),
    };

    // Compile directly: one guard, Lt semantics.
    let rule = candidate_to_rule(&cand);
    assert_eq!(rule.lhs_node_property_guards.len(), 1);
    let g = &rule.lhs_node_property_guards[0];
    assert_eq!(g.property, 1);
    assert!(matches!(g.op, hg_core::PropertyOp::Lt));
    assert_eq!(g.value, PropertyValue::Float(0.5));

    // Round-trip through a store region.
    let mut s = HypergraphStore::new();
    GraphRuleSource::install_candidates(&mut s, std::slice::from_ref(&cand));
    let mut src = GraphRuleSource::new();
    let snap = s.snapshot();
    let d = src.rules(&snap, 0);
    assert_eq!(d.rules.len(), 1, "guarded rule installs cleanly");
    assert_eq!(d.rules[0].lhs_node_property_guards.len(), 1);
    assert_eq!(
        d.rules[0].lhs_node_property_guards[0].op,
        hg_core::PropertyOp::Lt,
        "op code must survive the graph round-trip"
    );
    // And it compiles.
    assert!(compile_rule(&d.rules[0]).is_ok());
}
