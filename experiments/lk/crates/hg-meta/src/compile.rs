//! E1 → Rule compilation (plan §2.4 compile pipeline, candidate stage).
//!
//! Turns a validated `CandidateRule` into an `hg_rules::Rule` with
//! auto-generated variable names (`v0`, `v1`, …) in compacted slot order.
//! This is the round-trip half of G-R2: TOML-parseable rules must compile
//! to behaviorally identical rules when expressed as E1 regions and read
//! back.
//!
//! E1's expressivity envelope (plan §3.6) is respected exactly:
//! - node label constraints (E_HASLABEL) → `lhs.node_labels`;
//! - priority → `rule.priority`;
//! - NO property effects (E2), NO schedules (SB-R2), NO edge labels in
//!   patterns, NO var-distinctness encoding. The absence is stated here,
//!   measured by G-R2's coverage number, and repeated in every novelty
//!   claim downstream.

use crate::schema::{
    CandidateEffect, CandidateGuard, CandidatePatternEdge, CandidateRule, CandidateVarGuard,
};
use ahash::AHashMap;
use hg_core::{Label, PropertyValue};
use hg_rules::rule::Rule;
use hg_rules::PatternGraph;

/// Variable naming convention for compiled candidates. Deterministic and
/// collision-free with hand-written TOML names only by prefix discipline;
/// G-R3 (interpreter identity) pins that A0/A1 rule sets compare equal via
/// this exact function.
pub fn var_name(ix: usize) -> String {
    format!("v{ix}")
}

/// Interned var names for a candidate, shared by the compiler and the
/// protection gate (`protect::candidate_to_rule_gated`).
///
/// Walk order is LHS edges then RHS edges, each side in slot-tuple
/// lexicographic order (the reader's canonical discipline modulo node
/// relabeling); vars are named `v0`, `v1`, … on first encounter. Entry `i`
/// is the compiled name of original var `i`; unwired vars keep the empty
/// string (the compiler discards them — no side wiring means no
/// semantics — and the gate must skip them for the same reason).
///
/// SB-GE6 (opened by EXP-R04's `exp_r04_a2` failure): the gate used to
/// name vars by raw index (`var_name(v)`), which coincides with this
/// interning ONLY when the reader happens to return vars in identity
/// order. The reader interns by encounter over content-sorted pattern
/// edges (RHS sorts before LHS on the `is_lhs` tie-break), so a read-back
/// rule like `lhs [[0,2]]` interns original var 2 as `v1` while the gate
/// injected a guard on `v2` — a name absent from `lhs.nodes`, rejected by
/// `compile_rules` as `InvalidGuard("guard var 'v2' not in LHS")`. The
/// gate now speaks these interned names; raw-index naming must never
/// recur at any compile-adjacent site.
///
/// Returns `(name_of, intern)`: `name_of[i]` is the compiled name of
/// original var `i` (empty = unwired = discarded); `intern` is the
/// position → original-var order used for `lhs.nodes` / `rhs.nodes`.
pub fn interned_names(cand: &CandidateRule) -> (Vec<String>, Vec<usize>) {
    let mut intern: Vec<usize> = Vec::new(); // position -> original var idx
    let mut seen = vec![false; cand.n_vars()];
    let visit = |slots: &[usize], intern: &mut Vec<usize>, seen: &mut [bool]| {
        for s in slots {
            if !seen[*s] {
                seen[*s] = true;
                intern.push(*s);
            }
        }
    };
    let lhs_sorted: Vec<&CandidatePatternEdge> = {
        let mut v: Vec<&CandidatePatternEdge> = cand.lhs.iter().collect();
        v.sort_by(|a, b| a.slots.cmp(&b.slots));
        v
    };
    let rhs_sorted: Vec<&CandidatePatternEdge> = {
        let mut v: Vec<&CandidatePatternEdge> = cand.rhs.iter().collect();
        v.sort_by(|a, b| a.slots.cmp(&b.slots));
        v
    };
    for e in &lhs_sorted {
        visit(&e.slots, &mut intern, &mut seen);
    }
    for e in &rhs_sorted {
        visit(&e.slots, &mut intern, &mut seen);
    }
    let mut m = vec![String::new(); cand.n_vars()];
    for (pos, orig) in intern.iter().enumerate() {
        m[*orig] = var_name(pos);
    }
    (m, intern)
}
/// Compile a candidate to a declarative Rule. Assumes validation passed
/// (V1..V9); compiling a quarantined candidate may panic or produce a dead
/// rule — callers gate on [`crate::validate::validate`] first.
///
/// Label semantics by side wiring (the homomorphic reading of §3.3):
/// - var wired into any LHS edge  → `lhs.node_labels` (match constraint);
/// - var ONLY in RHS (created)    → `rhs_node_labels` (created-node label);
/// - unwired var                  → DISCARDED (no side wiring = no
///   semantics: it can neither match nor be created; the reader cannot see
///   it either, so both paths agree. A label on an unwired var is inert).
///
/// Var naming follows the reader's canonical discipline (first-encounter
/// order over canonically sorted pattern edges, LHS before RHS), so a rule
/// built from a table and the same rule re-read from its E1 region produce
/// BYTE-IDENTICAL Rules — that is what makes G-R3's identity checkable at
/// the Rule level, not merely behaviorally.
pub fn candidate_to_rule(cand: &CandidateRule) -> Rule {
    // Canonical var interning shared with the protection gate (SB-GE6).
    let (name_of, intern) = interned_names(cand);
    let mut seen = vec![false; cand.n_vars()];
    for o in &intern {
        seen[*o] = true;
    }
    let lhs_sorted: Vec<&CandidatePatternEdge> = {
        let mut v: Vec<&CandidatePatternEdge> = cand.lhs.iter().collect();
        v.sort_by(|a, b| a.slots.cmp(&b.slots));
        v
    };
    let rhs_sorted: Vec<&CandidatePatternEdge> = {
        let mut v: Vec<&CandidatePatternEdge> = cand.rhs.iter().collect();
        v.sort_by(|a, b| a.slots.cmp(&b.slots));
        v
    };

    // Per-var LHS membership decides which side a label constraint binds.
    let mut lhs_membership = vec![false; cand.n_vars()];
    for e in &cand.lhs {
        for s in &e.slots {
            lhs_membership[*s] = true;
        }
    }

    // Per-var side wiring: a var referenced ONLY by RHS edges is created,
    // not matched — it must NOT appear in lhs.nodes (an extra LHS var is a
    // match requirement for the engine). LHS-wired vars appear on both
    // sides iff they survive; LHS-only vars (deleted) appear only in LHS.
    let mut rhs_membership = vec![false; cand.n_vars()];
    for e in &cand.rhs {
        for s in &e.slots {
            rhs_membership[*s] = true;
        }
    }

    let mut lhs = PatternGraph::new();
    lhs.nodes = intern
        .iter()
        .filter(|o| lhs_membership[**o])
        .map(|o| name_of[*o].clone())
        .collect();
    for e in &lhs_sorted {
        lhs.edges
            .push(e.slots.iter().map(|s| name_of[*s].clone()).collect());
    }
    let mut node_labels = AHashMap::default();
    let mut rhs_node_labels = AHashMap::default();
    for (v, l) in cand.var_labels.iter().enumerate() {
        if let Some(l) = l {
            if !seen[v] {
                continue; // unwired var: discarded by convention
            }
            if lhs_membership[v] {
                node_labels.insert(name_of[v].clone(), Label(*l));
            } else {
                // RHS-only var: created with this label.
                rhs_node_labels.insert(name_of[v].clone(), Label(*l));
            }
        }
    }
    lhs.node_labels = node_labels;

    let mut rhs = PatternGraph::new();
    // RHS carries every var that survives (LHS-wired) or is created
    // (RHS-only). LHS-only vars are deleted and must NOT appear here.
    rhs.nodes = intern
        .iter()
        .filter(|o| rhs_membership[**o])
        .map(|o| name_of[*o].clone())
        .collect();
    for e in &rhs_sorted {
        rhs.edges
            .push(e.slots.iter().map(|s| name_of[*s].clone()).collect());
    }

    // E1-G guards: op code (selfmod v2 convention) → PropertyOp. The
    // threshold rides verbatim; variant mismatch fails the guard at match
    // time per hg-core's no-coercion rule.
    let lhs_node_property_guards: Vec<hg_rules::NodePropertyGuard> = cand
        .guards
        .iter()
        .map(|g| hg_rules::NodePropertyGuard {
            var: name_of[g.var].clone(),
            property: g.key,
            op: match g.op {
                0 => hg_core::PropertyOp::Gt,
                1 => hg_core::PropertyOp::Lt,
                2 => hg_core::PropertyOp::Eq,
                // R5 extension: full six-op vocabulary.
                3 => hg_core::PropertyOp::Gte,
                4 => hg_core::PropertyOp::Lte,
                _ => hg_core::PropertyOp::Ne,
            },
            value: g.val,
        })
        .collect();

    // E1-V var guards: vars_distinct rides through as the engine's own
    // VarGuard (the engine expands to pairwise distinctness at compile).
    let lhs_var_guards: Vec<hg_rules::VarGuard> = cand
        .var_guards
        .iter()
        .map(|vg| hg_rules::VarGuard {
            guard_type: hg_rules::VarGuardType::VarsDistinct,
            vars: vg.vars.iter().map(|v| name_of[*v].clone()).collect(),
        })
        .collect();

    // E2-lite effects: op codes mirror EffectOpDecl. Constant ops carry
    // `val`; source ops carry source var + its key.
    const OP_SET: u8 = 0;
    const OP_COPYFROM: u8 = 1;
    const OP_SCALEFROM: u8 = 2;
    const OP_ADD: u8 = 3;
    const OP_SUBTRACT: u8 = 4;
    const OP_ADDFROMPROPERTY: u8 = 5;
    const OP_SUBTRACTFROMPROPERTY: u8 = 6;
    const OP_JITTER: u8 = 7;
    let rhs_property_effects: Vec<hg_rules::PropertyEffectDecl> = cand
        .effects
        .iter()
        .map(|e| {
            let op = match (e.op, &e.val, e.source, e.source_key) {
                (OP_SET, Some(v), _, _) => hg_rules::EffectOpDecl::Set { value: *v },
                (OP_COPYFROM, _, Some(s), Some(sk)) => hg_rules::EffectOpDecl::CopyFrom {
                    source_var: name_of[s].clone(),
                    source_key: sk,
                },
                (OP_SCALEFROM, Some(v), Some(s), Some(sk)) => {
                    let factor = match v {
                        PropertyValue::Float(f) => *f,
                        _ => 1.0,
                    };
                    hg_rules::EffectOpDecl::ScaleFrom {
                        source_var: name_of[s].clone(),
                        source_key: sk,
                        factor,
                    }
                }
                (OP_ADD, Some(v), _, _) => hg_rules::EffectOpDecl::Add { value: *v },
                (OP_SUBTRACT, Some(v), _, _) => hg_rules::EffectOpDecl::Subtract { value: *v },
                (OP_ADDFROMPROPERTY, _, Some(s), Some(sk)) => {
                    hg_rules::EffectOpDecl::AddFromProperty {
                        source_var: name_of[s].clone(),
                        source_key: sk,
                    }
                }
                (OP_SUBTRACTFROMPROPERTY, _, Some(s), Some(sk)) => {
                    hg_rules::EffectOpDecl::SubtractFromProperty {
                        source_var: name_of[s].clone(),
                        source_key: sk,
                    }
                }
                (OP_JITTER, Some(v), _, _) => {
                    let magnitude = match v {
                        PropertyValue::Float(f) => *f,
                        _ => 0.0,
                    };
                    hg_rules::EffectOpDecl::Jitter { magnitude }
                }
                _ => hg_rules::EffectOpDecl::Set {
                    value: PropertyValue::Int(0),
                }, // unreachable for well-formed candidates
            };
            hg_rules::PropertyEffectDecl {
                var: name_of[e.target].clone(),
                key: e.key,
                op,
            }
        })
        .collect();

    Rule {
        name: String::new(),
        priority: cand.priority,
        weight: 1.0,
        enabled: true,
        lhs,
        rhs,
        lhs_node_property_guards,
        lhs_node_property_compares: Vec::new(),
        lhs_node_degree_compares: Vec::new(),
        lhs_node_age_compares: Vec::new(),
        lhs_var_guards,
        rhs_node_labels,
        rhs_property_effects,
        schedule: None,
    }
}

/// Inverse extraction (G-R2 sweep): a parsed `Rule` → the CandidateRule
/// E1 would encode. Returns None when the rule uses structure E1 cannot
/// carry even structurally (edge labels on patterns). This does NOT
/// pre-check envelope exclusions — the sweep classifies those itself.
pub fn rule_to_candidate(rule: &Rule) -> Option<CandidateRule> {
    if !rule.lhs.edge_labels.is_empty() || !rule.rhs.edge_labels.is_empty() {
        return None; // edge labels in patterns: not in E1's envelope
    }

    // Intern vars across LHS then RHS (creation order is fine here; RCF
    // and the reader both normalize).
    let mut order: Vec<String> = Vec::new();
    for v in rule.lhs.nodes.iter().chain(rule.rhs.nodes.iter()) {
        if !order.contains(v) {
            order.push(v.clone());
        }
    }
    let idx = |name: &str| -> Option<usize> { order.iter().position(|v| v == name) };

    let mut var_labels = vec![None; order.len()];
    // LHS constraints land on their var.
    for (name, l) in &rule.lhs.node_labels {
        let ix = idx(name)?;
        var_labels[ix] = Some(l.0);
    }
    // RHS-only label = created-node label; survivor relabels are NOT
    // translated (SB-R5 exclusion — caller rejects them earlier). Checked
    // against BOTH spellings: `rhs.node_labels` (graph field) and legacy
    // top-level `rhs_node_labels` (compiled.rs merges them via or_else).
    for source in [&rule.rhs_node_labels, &rule.rhs.node_labels] {
        for (name, l) in source {
            let ix = idx(name)?;
            let in_lhs = rule.lhs.nodes.contains(name);
            if !in_lhs {
                var_labels[ix] = Some(l.0);
            }
        }
    }

    let conv_edges = |edges: &Vec<Vec<String>>| -> Option<Vec<CandidatePatternEdge>> {
        edges
            .iter()
            .map(|e| {
                let mut slots: Vec<usize> = Vec::with_capacity(e.len());
                for v in e {
                    slots.push(idx(v)?);
                }
                Some(CandidatePatternEdge { slots })
            })
            .collect()
    };

    let guards = rule
        .lhs_node_property_guards
        .iter()
        .map(|g| {
            Some(CandidateGuard {
                var: idx(&g.var)?,
                key: g.property,
                op: match g.op {
                    hg_core::PropertyOp::Gt => 0,
                    hg_core::PropertyOp::Lt => 1,
                    hg_core::PropertyOp::Eq => 2,
                    // E1-G op-set extension (R5): the full six-op vocabulary.
                    hg_core::PropertyOp::Gte => 3,
                    hg_core::PropertyOp::Lte => 4,
                    hg_core::PropertyOp::Ne => 5,
                },
                val: g.value,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let mut guards = guards;
    guards.sort_by(|a, b| {
        (a.var, a.key, a.op, prop_repr_pub(&a.val)).cmp(&(
            b.var,
            b.key,
            b.op,
            prop_repr_pub(&b.val),
        ))
    });

    // E1-V: vars_distinct guards → var-index lists.
    let mut var_guards: Vec<CandidateVarGuard> = Vec::new();
    for vg in &rule.lhs_var_guards {
        match vg.guard_type {
            hg_rules::VarGuardType::VarsDistinct => {
                let mut vars: Vec<usize> =
                    vg.vars.iter().map(|v| idx(v)).collect::<Option<Vec<_>>>()?;
                vars.sort();
                var_guards.push(CandidateVarGuard { vars });
            }
        }
    }
    var_guards.sort();

    // E2-lite: effects → op codes. Ops outside E2-lite's set (none today)
    // would return None.
    const OP_SET: u8 = 0;
    const OP_COPYFROM: u8 = 1;
    const OP_SCALEFROM: u8 = 2;
    const OP_ADD: u8 = 3;
    const OP_SUBTRACT: u8 = 4;
    const OP_ADDFROMPROPERTY: u8 = 5;
    const OP_SUBTRACTFROMPROPERTY: u8 = 6;
    const OP_JITTER: u8 = 7;
    let mut effects: Vec<CandidateEffect> = Vec::new();
    for e in &rule.rhs_property_effects {
        let target = idx(&e.var)?;
        let (op, val, source, source_key): (u8, Option<PropertyValue>, Option<usize>, Option<u32>) =
            match &e.op {
                hg_rules::EffectOpDecl::Set { value } => (OP_SET, Some(*value), None, None),
                hg_rules::EffectOpDecl::CopyFrom {
                    source_var,
                    source_key,
                } => (OP_COPYFROM, None, Some(idx(source_var)?), Some(*source_key)),
                hg_rules::EffectOpDecl::ScaleFrom {
                    source_var,
                    source_key,
                    factor,
                } => (
                    OP_SCALEFROM,
                    Some(PropertyValue::Float(*factor)),
                    Some(idx(source_var)?),
                    Some(*source_key),
                ),
                hg_rules::EffectOpDecl::Add { value } => (OP_ADD, Some(*value), None, None),
                hg_rules::EffectOpDecl::Subtract { value } => {
                    (OP_SUBTRACT, Some(*value), None, None)
                }
                hg_rules::EffectOpDecl::AddFromProperty {
                    source_var,
                    source_key,
                } => (
                    OP_ADDFROMPROPERTY,
                    None,
                    Some(idx(source_var)?),
                    Some(*source_key),
                ),
                hg_rules::EffectOpDecl::SubtractFromProperty {
                    source_var,
                    source_key,
                } => (
                    OP_SUBTRACTFROMPROPERTY,
                    None,
                    Some(idx(source_var)?),
                    Some(*source_key),
                ),
                hg_rules::EffectOpDecl::Jitter { magnitude } => (
                    OP_JITTER,
                    Some(PropertyValue::Float(*magnitude)),
                    None,
                    None,
                ),
            };
        effects.push(CandidateEffect {
            target,
            key: e.key,
            op,
            val,
            source,
            source_key,
        });
    }
    effects.sort_by(|a, b| (a.target, a.key, a.op).cmp(&(b.target, b.key, b.op)));

    Some(CandidateRule {
        priority: rule.priority,
        lhs: conv_edges(&rule.lhs.edges)?,
        rhs: conv_edges(&rule.rhs.edges)?,
        var_labels,
        guards,
        var_guards,
        effects,
    })
}

fn prop_repr_pub(v: &PropertyValue) -> (u8, i64, u64) {
    match v {
        PropertyValue::Int(i) => (0, *i, 0),
        PropertyValue::Float(f) => (1, 0, f.to_bits()),
        PropertyValue::Bool(b) => (2, *b as i64, 0),
    }
}
