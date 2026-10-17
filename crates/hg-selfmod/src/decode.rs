//! Rule-subgraph -> `RuleSpec` -> engine-native `CompiledRule`.
//!
//! Well-formedness lives HERE, not in the mutators (junk-DNA discipline): a
//! subgraph that fails any validity check decodes to `None`, stays in the
//! store, still pays rent, and is counted in the `QuarantineReport`
//! (per-validator V1..V9 counters — see `docs/ere_validators.md`).
//! Never panics.

use crate::schema::*;
use crate::store_ext::*;
use hg_core::{HypergraphStore, Label, NodeId, PropertyValue};
use hg_rules::{
    compile_rule, CompiledRule, EffectOpDecl, PatternGraph, Rule, VarGuard, VarGuardType,
};

/// Validator ids V1..V9 (see `docs/ere_validators.md`). Order = check order
/// in `decode_spec_counted`: first failure wins, so adversarial fixtures must
/// isolate single violations.
pub const V1: usize = 0; // var-count well-formedness
pub const V2: usize = 1; // LHS pattern-edge well-formedness
pub const V3: usize = 2; // LHS edge-count cap
pub const V4: usize = 3; // guard well-formedness
pub const V5: usize = 4; // delete-list well-formedness
pub const V6: usize = 5; // relabel well-formedness
pub const V7: usize = 6; // create-list well-formedness
pub const V8: usize = 7; // RHS edge well-formedness
pub const V9: usize = 8; // compilability + non-vacuity
/// Number of validators. `quarantine_by_validator` has exactly this many
/// counters, present on every run including zeros.
pub const N_VALIDATORS: usize = 9;

/// Per-decode quarantine accounting. `total` always equals
/// `by_validator`'s sum (asserted in G-R4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuarantineReport {
    pub total: usize,
    pub by_validator: [usize; N_VALIDATORS],
}

fn get_prop(s: &HypergraphStore, n: NodeId, key: u32) -> Option<PropertyValue> {
    s.get_node(n)?.get_property(key)
}

/// Decode one rule-subgraph. `None` = malformed (inert junk DNA), counted
/// against exactly one validator in `counters` (first failure wins).
/// See `docs/ere_validators.md` for the V1..V9 predicates.
pub fn decode_spec_counted(
    s: &HypergraphStore,
    root: NodeId,
    caps: &Caps,
    counters: &mut [usize; N_VALIDATORS],
) -> Option<RuleSpec> {
    // Stale handle: caller bug, not quarantine (unreachable via decode_all,
    // which enumerates live roots only).
    if s.get_node(root).is_none() {
        return None;
    }
    let Some(n_vars_i) = get_int(s, root, K_NVARS) else {
        counters[V1] += 1;
        return None;
    };
    if n_vars_i < 1 || n_vars_i > caps.lhs_max_nodes as i64 {
        counters[V1] += 1;
        return None;
    }
    let n_vars = n_vars_i as u8;
    let priority = get_int(s, root, K_PRIORITY)? as i32;

    // Var label constraints. Out-of-range PNodes are ignored (v2 behavior:
    // a stale constraint left behind by drop_lhs_var is inert, not fatal).
    let mut var_labels = vec![None; n_vars as usize];
    for pn in children(s, root, HAS_PNODE) {
        let vid = get_int(s, pn, K_VAR_ID).unwrap_or(-1);
        if vid >= 0 && (vid as u8) < n_vars {
            if let Some(l) = get_int(s, pn, K_LABEL_CONSTRAINT) {
                var_labels[vid as usize] = Some(l as u32);
            }
        }
    }

    // Ordered endpoint list of a PEdge/REdge node; None if malformed.
    let read_refs = |en: NodeId, max_ref: i64| -> Option<Vec<u8>> {
        let arity = get_int(s, en, K_ARITY)?;
        if arity < 2 || arity > caps.max_arity as i64 {
            return None;
        }
        let mut refs = Vec::with_capacity(arity as usize);
        for i in 0..arity {
            let r = get_int(s, en, K_EP0 + i as u32)?;
            if r < 0 || r >= max_ref {
                return None;
            }
            refs.push(r as u8);
        }
        Some(refs)
    };

    let mut pedges = Vec::new();
    for pe in children(s, root, HAS_PEDGE) {
        match read_refs(pe, n_vars as i64) {
            Some(refs) => pedges.push(refs),
            None => {
                counters[V2] += 1;
                return None;
            }
        }
    }
    if pedges.len() > caps.lhs_max_edges {
        counters[V3] += 1;
        return None;
    }

    let mut guards = Vec::new();
    for gn in children(s, root, HAS_GUARD) {
        let Some(g) = (|| {
            Some(GuardSpec {
                var: get_int(s, gn, K_GVAR).filter(|v| *v >= 0 && *v < n_vars as i64)? as u8,
                key: get_int(s, gn, K_GKEY)? as u32,
                op: get_int(s, gn, K_GOP).filter(|o| (0..=5).contains(o))? as u8,
                val: get_prop(s, gn, K_GVAL)?,
            })
        })() else {
            counters[V4] += 1;
            return None;
        };
        if g.property_op().is_none() {
            counters[V4] += 1;
            return None;
        }
        guards.push(g);
    }

    let mut deletes = Vec::new();
    for dn in children(s, root, HAS_DELETE) {
        let Some(v) = get_int(s, dn, K_VAR_ID) else {
            counters[V5] += 1;
            return None;
        };
        if v < 0 || v >= n_vars as i64 {
            counters[V5] += 1;
            return None;
        }
        deletes.push(v as u8);
    }
    deletes.sort_unstable();
    deletes.dedup();

    let mut relabels = Vec::new();
    for rn in children(s, root, HAS_RELABEL) {
        let (Some(v), Some(l)) = (get_int(s, rn, K_VAR_ID), get_int(s, rn, K_NEW_LABEL)) else {
            counters[V6] += 1;
            return None;
        };
        if v < 0 || v >= n_vars as i64 {
            counters[V6] += 1;
            return None;
        }
        // Relabel of a deleted var is inert, not fatal (deletion wins —
        // recon note (a)); drop it here so the compiled rule is clean.
        if !deletes.contains(&(v as u8)) {
            relabels.push((v as u8, l as u32));
        }
    }
    relabels.sort_unstable();
    relabels.dedup_by_key(|(v, _)| *v); // one relabel per var (first in sorted order wins)

    // Distinct pairs (E1d, validated under V6): both vars in range; self-pairs
    // are dropped as inert (mirroring compile_rule's own skip), not quarantine.
    let mut distinct = Vec::new();
    for dn in children(s, root, HAS_DISTINCT) {
        let (Some(a), Some(b)) = (get_int(s, dn, K_VAR_ID), get_int(s, dn, K_VAR_ID2)) else {
            counters[V6] += 1;
            return None;
        };
        if a < 0 || a >= n_vars as i64 || b < 0 || b >= n_vars as i64 {
            counters[V6] += 1;
            return None;
        }
        if a == b {
            continue;
        }
        distinct.push((a.min(b) as u8, a.max(b) as u8));
    }
    distinct.sort_unstable();
    distinct.dedup();

    // Creates, ordered by index; indices must be exactly 0..len (junk otherwise).
    let mut cvec: Vec<(i64, u32)> = Vec::new();
    for cn in children(s, root, HAS_CREATE) {
        let (Some(k), Some(l)) = (
            get_int(s, cn, K_CREATE_INDEX),
            get_int(s, cn, K_CREATE_LABEL),
        ) else {
            counters[V7] += 1;
            return None;
        };
        cvec.push((k, l as u32));
    }
    cvec.sort();
    if cvec.len() > caps.rhs_max_create {
        counters[V7] += 1;
        return None;
    }
    for (i, (k, _)) in cvec.iter().enumerate() {
        if *k != i as i64 {
            counters[V7] += 1;
            return None;
        }
    }
    let creates: Vec<u32> = cvec.into_iter().map(|x| x.1).collect();

    let n_refs = n_vars as i64 + creates.len() as i64;
    let mut redges = Vec::new();
    for en in children(s, root, HAS_REDGE) {
        let Some(refs) = read_refs(en, n_refs) else {
            counters[V8] += 1;
            return None;
        };
        // Refs to deleted vars make the rule malformed (junk DNA).
        if refs.iter().any(|r| deletes.contains(r)) {
            counters[V8] += 1;
            return None;
        }
        redges.push(refs);
    }

    // Effects (E2, validated under V9): op known, required params present,
    // refs in range. Declaration order preserved (sequential semantics).
    // Targets on deleted vars pass through here and fail in compile_rule
    // (undeclared RHS name) — still V9, via the engine-reject arm below.
    // Every failure path counts exactly one validator (no bare `?`).
    let mut effects = Vec::new();
    for en in children(s, root, HAS_EFFECT) {
        let Some(target) = get_int(s, en, K_ETARGET) else {
            counters[V9] += 1;
            return None;
        };
        let Some(key) = get_int(s, en, K_EKEY) else {
            counters[V9] += 1;
            return None;
        };
        let Some(op) = get_int(s, en, K_EOP) else {
            counters[V9] += 1;
            return None;
        };
        if !(0..=7).contains(&op) || target < 0 || target >= n_refs {
            counters[V9] += 1;
            return None;
        }
        // Source pair, required by the from-variants; ignored (not an
        // error) when the op takes none — stray props are junk, not malformation.
        let src_pair = match (get_int(s, en, K_ESRC), get_int(s, en, K_ESRCKEY)) {
            (Some(src), Some(skey)) => {
                if src < 0 || src >= n_refs {
                    counters[V9] += 1;
                    return None;
                }
                Some((src as u8, skey as u32))
            }
            _ => None,
        };
        let val = |counters: &mut [usize; N_VALIDATORS]| -> Option<PropertyValue> {
            match get_prop(s, en, K_EVAL) {
                Some(v) => Some(v),
                None => {
                    counters[V9] += 1;
                    None
                }
            }
        };
        let fval = |counters: &mut [usize; N_VALIDATORS]| -> Option<f64> {
            match get_prop(s, en, K_EVAL) {
                Some(PropertyValue::Float(f)) => Some(f),
                _ => {
                    counters[V9] += 1;
                    None
                }
            }
        };
        let need_src = |counters: &mut [usize; N_VALIDATORS]| -> Option<(u8, u32)> {
            match src_pair {
                Some(p) => Some(p),
                None => {
                    counters[V9] += 1;
                    None
                }
            }
        };
        let eop = match op {
            0 => EffectOp::Set {
                value: val(counters)?,
            },
            1 => {
                let (source, source_key) = need_src(counters)?;
                EffectOp::CopyFrom { source, source_key }
            }
            2 => {
                let (source, source_key) = need_src(counters)?;
                EffectOp::ScaleFrom {
                    source,
                    source_key,
                    factor: fval(counters)?,
                }
            }
            3 => EffectOp::Add {
                value: val(counters)?,
            },
            4 => EffectOp::Subtract {
                value: val(counters)?,
            },
            5 => {
                let (source, source_key) = need_src(counters)?;
                EffectOp::AddFromProperty { source, source_key }
            }
            6 => {
                let (source, source_key) = need_src(counters)?;
                EffectOp::SubtractFromProperty { source, source_key }
            }
            7 => EffectOp::Jitter {
                magnitude: fval(counters)?,
            },
            _ => {
                counters[V9] += 1;
                return None;
            }
        };
        effects.push(EffectSpec {
            target: target as u8,
            key: key as u32,
            op: eop,
        });
    }

    let spec = RuleSpec {
        priority,
        n_vars,
        var_labels,
        pedges,
        guards,
        deletes,
        relabels,
        creates,
        redges,
        distinct,
        effects,
    };
    // V9: vacuous rules (no possible store effect) quarantine here. A rule
    // with no consumable pattern, no delete/relabel/create/edge effect, and
    // no property effect can never be productive — pure match-volume load.
    // Guards and label constraints alone do not act on the store.
    if spec.pedges.is_empty()
        && spec.deletes.is_empty()
        && spec.relabels.is_empty()
        && spec.creates.is_empty()
        && spec.redges.is_empty()
        && spec.effects.is_empty()
    {
        counters[V9] += 1;
        return None;
    }
    Some(spec)
}

/// Compat wrapper: uncounted decode (fixtures, one-off checks). Production
/// paths must use `decode_spec_counted` / `decode_all` so quarantine is
/// attributed.
pub fn decode_spec(s: &HypergraphStore, root: NodeId, caps: &Caps) -> Option<RuleSpec> {
    let mut counters = [0usize; N_VALIDATORS];
    decode_spec_counted(s, root, caps, &mut counters)
}

/// `RuleSpec` -> engine-native rule, via `hg_rules::Rule` + `compile_rule`.
/// LHS vars are named `v0..`, creates `c0..`. LHS pattern edges are consumed
/// by the engine on apply; survivors are every var not in `deletes`.
pub fn spec_to_compiled(spec: &RuleSpec, name: &str) -> Option<CompiledRule> {
    let mut lhs = PatternGraph::new();
    for i in 0..spec.n_vars {
        let vn = format!("v{i}");
        if let Some(l) = spec.var_labels[i as usize] {
            lhs.node_labels.insert(vn.clone(), Label(l));
        }
        lhs.nodes.push(vn);
    }
    for e in &spec.pedges {
        lhs.edges.push(e.iter().map(|r| format!("v{r}")).collect());
    }

    let mut rhs = PatternGraph::new();
    let mut rhs_node_labels = ahash_map();
    for i in 0..spec.n_vars {
        if spec.deletes.contains(&i) {
            continue;
        }
        let vn = format!("v{i}");
        if let Some((_, l)) = spec.relabels.iter().find(|(v, _)| *v == i) {
            rhs_node_labels.insert(vn.clone(), Label(*l));
        }
        rhs.nodes.push(vn);
    }
    for (k, l) in spec.creates.iter().enumerate() {
        let cn = format!("c{k}");
        rhs.node_labels.insert(cn.clone(), Label(*l));
        rhs.nodes.push(cn);
    }
    let ref_name = |r: u8| -> String {
        if r < spec.n_vars {
            format!("v{r}")
        } else {
            format!("c{}", r - spec.n_vars)
        }
    };
    for e in &spec.redges {
        rhs.edges.push(e.iter().map(|r| ref_name(*r)).collect());
    }

    let mut rule = Rule {
        name: name.to_string(),
        priority: spec.priority,
        ..Rule::default()
    };
    rule.lhs = lhs;
    rule.rhs = rhs;
    rule.rhs_node_labels = rhs_node_labels;
    for g in &spec.guards {
        rule.lhs_node_property_guards
            .push(hg_rules::NodePropertyGuard {
                var: format!("v{}", g.var),
                property: g.key,
                op: g.property_op()?,
                value: g.val,
            });
    }
    // Distinct pairs (E1d): one 2-var VarsDistinct guard per pair expands to
    // exactly that pair in compile_rule — no engine changes needed.
    for (a, b) in &spec.distinct {
        rule.lhs_var_guards.push(VarGuard {
            guard_type: VarGuardType::VarsDistinct,
            vars: vec![format!("v{a}"), format!("v{b}")],
        });
    }
    // Property effects (E2): numeric refs back to names via the existing
    // `ref_name` (survivors `v{i}`, creates `c{k}`), declaration order
    // preserved; compile_rule resolves and validates (a target on a deleted
    // var fails there → None → V9 engine-reject arm).
    for e in &spec.effects {
        let decl_op = match &e.op {
            EffectOp::Set { value } => EffectOpDecl::Set { value: *value },
            EffectOp::CopyFrom { source, source_key } => EffectOpDecl::CopyFrom {
                source_var: ref_name(*source),
                source_key: *source_key,
            },
            EffectOp::ScaleFrom {
                source,
                source_key,
                factor,
            } => EffectOpDecl::ScaleFrom {
                source_var: ref_name(*source),
                source_key: *source_key,
                factor: *factor,
            },
            EffectOp::Add { value } => EffectOpDecl::Add { value: *value },
            EffectOp::Subtract { value } => EffectOpDecl::Subtract { value: *value },
            EffectOp::AddFromProperty { source, source_key } => EffectOpDecl::AddFromProperty {
                source_var: ref_name(*source),
                source_key: *source_key,
            },
            EffectOp::SubtractFromProperty { source, source_key } => {
                EffectOpDecl::SubtractFromProperty {
                    source_var: ref_name(*source),
                    source_key: *source_key,
                }
            }
            EffectOp::Jitter { magnitude } => EffectOpDecl::Jitter {
                magnitude: *magnitude,
            },
        };
        rule.rhs_property_effects
            .push(hg_rules::PropertyEffectDecl {
                var: ref_name(e.target),
                key: e.key,
                op: decl_op,
            });
    }
    compile_rule(&rule).ok()
}

fn ahash_map() -> ahash::AHashMap<String, Label> {
    ahash::AHashMap::default()
}

/// One decoded, live rule.
pub struct DecodedRule {
    pub root: NodeId,
    pub spec: RuleSpec,
    pub class_hash: u64,
    pub compiled: CompiledRule,
}

/// Decode every RuleRoot; canonical order = (class_hash, K_SEQ). Returns the
/// decoded set plus the quarantine report (per-validator V1..V9 counters,
/// present including zeros; `total` = their sum).
pub fn decode_all(s: &HypergraphStore, caps: &Caps) -> (Vec<DecodedRule>, QuarantineReport) {
    let mut out = Vec::new();
    let mut report = QuarantineReport::default();
    for root in rule_roots(s) {
        let Some(spec) = decode_spec_counted(s, root, caps, &mut report.by_validator) else {
            report.total += 1;
            continue;
        };
        let ch = spec.class_hash();
        let Some(compiled) = spec_to_compiled(&spec, &format!("r{ch:016x}")) else {
            // Engine rejected a V1..V8-passing spec: V9 (regression hook).
            report.by_validator[V9] += 1;
            report.total += 1;
            continue;
        };
        out.push(DecodedRule {
            root,
            spec,
            class_hash: ch,
            compiled,
        });
    }
    debug_assert_eq!(report.total, report.by_validator.iter().sum::<usize>());
    out.sort_by_key(|d| (d.class_hash, get_int(s, d.root, K_SEQ).unwrap_or(i64::MAX)));
    (out, report)
}
