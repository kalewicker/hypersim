//! E1 encoding schema (ERE plan §3): label constants, region reader.
//!
//! Design principle (plan §3.1): the encoding is homomorphic to the
//! semantics. A pattern variable is a single node wired into LHS
//! pattern-edges, RHS pattern-edges, or both; survive/delete/create falls
//! out of which sides a var touches. No name table, no interpretive layer.
//!
//! Determinism:
//! - D1 (plan §2.4): rule roots enumerate in canonical content order —
//!   never HashMap iteration order, never slotmap slot order (slot reuse
//!   makes the latter history-dependent). Fixture-tested.
//! - D2: nothing in this crate draws from any RNG; every tie-break is on
//!   content.

use hg_core::{GraphSnapshot, NodeData, NodeId, PropertyValue};
use std::collections::HashMap;

// ---------- label bands ----------

/// Labels >= META_BASE are meta-band. Arm A2's protection is an implicit
/// `label < META_BASE` guard injected at compile time per LHS var.
pub const META_BASE: u32 = 0xFFF0_0000;
pub const L_RULE_ROOT: u32 = META_BASE + 0x01;
pub const L_SIDE_L: u32 = META_BASE + 0x02;
pub const L_SIDE_R: u32 = META_BASE + 0x03;
pub const L_PAT_EDGE: u32 = META_BASE + 0x04;
pub const L_VAR: u32 = META_BASE + 0x05;
pub const L_LABEL_LIT: u32 = META_BASE + 0x06;
pub const L_GUARD: u32 = META_BASE + 0x07; // E1-G, Phase R4
/// E2-lite (Phase R5): RHS property effect node.
pub const L_EFFECT: u32 = META_BASE + 0x08;

// ---------- edge labels ----------

pub const E_BECOMES: u32 = META_BASE + 0x40;
pub const E_MEMBER: u32 = META_BASE + 0x41;
/// Slot position lives in the EDGE LABEL (`E_SLOT_BASE + k`, k < MAX_ARITY),
/// not tuple position: deleting one of three slot edges leaves a valid
/// arity-2 pattern edge rather than an arity-3 edge with a hole. The reader
/// compacts surviving slots by sorted k. Mutational neutrality by
/// construction (the bid behind gate G-R1).
pub const E_SLOT_BASE: u32 = META_BASE + 0x50;
pub const E_HASLABEL: u32 = META_BASE + 0x70;
/// E1-G (§3.5): GUARD node wired to its var.
pub const E_GUARDS: u32 = META_BASE + 0x71;
/// E1-V: VAR_GUARD node (vars_distinct) wired to the RULE_ROOT.
pub const E_VAR_GUARD: u32 = META_BASE + 0x72;
/// E2-lite: EFFECT node wired to its target var.
pub const E_EFFECT: u32 = META_BASE + 0x73;

// ---------- property keys ----------

/// Property on RULE_ROOT carrying priority (`Int`).
pub const K_PRIORITY: u32 = 2;
/// Property on LABEL_LIT carrying the constrained label value (`Int`).
pub const K_LABEL_VALUE: u32 = 0;
/// E1-G properties on a GUARD node: which world property to test, the op
/// (selfmod v2 convention: 0=Gt, 1=Lt, 2=Eq), and the threshold value.
pub const K_GUARD_KEY: u32 = 10;
pub const K_GUARD_OP: u32 = 11;
pub const K_GUARD_VAL: u32 = 12;
/// E1-V properties on a VAR_GUARD node: the guarded var indices as Ints
/// under consecutive keys starting here (K_VGUARD_VARS + i for i-th var).
pub const K_VGUARD_VARS: u32 = 20;
/// E2-lite properties on an EFFECT node: target key, op code
/// (0=Set 1=CopyFrom 2=ScaleFrom 3=Add 4=Subtract 5=AddFromProperty
/// 6=SubtractFromProperty 7=Jitter), and the constant payload where the
/// op takes one (Set/Add/Subtract value; ScaleFrom/Jitter magnitude).
pub const K_EFFECT_KEY: u32 = 30;
pub const K_EFFECT_OP: u32 = 31;
pub const K_EFFECT_VAL: u32 = 32;

/// Maximum pattern-edge arity. Slot labels above `E_SLOT_BASE + MAX_ARITY`
/// are out of band and rejected by the reader.
pub const MAX_ARITY: usize = 8;

// ---------- extracted form ----------

/// One pattern edge: ordered var references (compacted slot order).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePatternEdge {
    pub slots: Vec<usize>,
}

/// E1-G (§3.5): a property guard on one var. `op` uses the selfmod v2
/// convention (0=Gt, 1=Lt, 2=Eq) — the encoding's own vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateGuard {
    pub var: usize,
    pub key: u32,
    pub op: u8,
    pub val: PropertyValue,
}

/// E1-V: a vars_distinct constraint over a var list.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CandidateVarGuard {
    pub vars: Vec<usize>,
}

/// E2-lite: an RHS property effect. Ops mirror `EffectOpDecl` (0=Set
/// 1=CopyFrom 2=ScaleFrom 3=Add 4=Subtract 5=AddFromProperty
/// 6=SubtractFromProperty 7=Jitter). Constant payload rides `val`;
/// source-var ops carry the source var index + its key.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateEffect {
    pub target: usize,
    pub key: u32,
    pub op: u8,
    pub val: Option<PropertyValue>,
    pub source: Option<usize>,
    pub source_key: Option<u32>,
}

/// A rule region extracted from a graph — the intermediate between the
/// region reader and everything downstream (RCF, validation, compilation).
/// Node identity survives only long enough for adjacency during extraction.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateRule {
    pub priority: i32,
    /// LHS pattern edges, in canonical pattern-edge order.
    pub lhs: Vec<CandidatePatternEdge>,
    /// RHS pattern edges, same ordering discipline.
    pub rhs: Vec<CandidatePatternEdge>,
    /// Per-var label constraint by compacted var id; `None` = free var.
    pub var_labels: Vec<Option<u32>>,
    /// E1-G guards, sorted canonically by (var, key, op, value repr).
    pub guards: Vec<CandidateGuard>,
    /// E1-V vars_distinct constraints, sorted canonically by var list.
    pub var_guards: Vec<CandidateVarGuard>,
    /// E2-lite effects, sorted canonically by (target, key, op).
    pub effects: Vec<CandidateEffect>,
}

impl CandidateRule {
    pub fn n_vars(&self) -> usize {
        self.var_labels.len()
    }
}

// ---------- content-based ordering (D1/D2 machinery) ----------

fn prop_repr(v: &PropertyValue) -> (u8, i64, u64) {
    match v {
        PropertyValue::Int(i) => (0, *i, 0),
        PropertyValue::Float(f) => (1, 0, f.to_bits()),
        PropertyValue::Bool(b) => (2, *b as i64, 0),
    }
}

/// Content key for a node: (label, birth step, sorted property reprs).
/// Content-based, so two stores built by different insertion orders produce
/// identical keys for corresponding nodes — the property D1's fixture pins.
type NodeSortKey = (u32, u64, Vec<(u32, u8, i64, u64)>);

fn node_sort_key(d: &NodeData) -> NodeSortKey {
    let mut props: Vec<(u32, u8, i64, u64)> = d
        .properties
        .iter()
        .map(|(k, v)| {
            let (t, i, f) = prop_repr(v);
            (*k, t, i, f)
        })
        .collect();
    props.sort();
    (d.label.0, d.birth_step, props)
}

fn node_ffi(id: NodeId) -> u64 {
    use slotmap::Key;
    id.data().as_ffi()
}

/// All RULE_ROOT nodes in canonical content order (D1). The trailing ffi-key
/// term only breaks ties between genuinely identical-content roots (e.g. two
/// freshly seeded empty rules); it is deterministic for a fixed construction
/// sequence and never overrides content ordering.
pub fn find_rule_roots(snap: &GraphSnapshot<'_>) -> Vec<NodeId> {
    let mut roots: Vec<(NodeId, &NodeData)> = snap
        .nodes()
        .filter(|(_, d)| d.label.0 == L_RULE_ROOT)
        .collect();
    roots.sort_by_key(|(id, d)| (node_sort_key(d), node_ffi(*id)));
    roots.into_iter().map(|(id, _)| id).collect()
}

// ---------- the region reader ----------

/// Why a region could not be read at all. Distinct from validator
/// quarantine (V-classes): these are walk-level structural failures that
/// prevent forming a `CandidateRule` in the first place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadRejection {
    NoBecomesEdge,
    MultipleBecomesEdges(usize),
    BecomesArity(usize),
    /// A MEMBER edge points at the same PAT_EDGE node from both sides.
    BothSidesMember,
    MemberEndpointNotPatEdge,
    SlotEndpointNotVar,
    SlotLabelOutOfRange(u32),
    /// A pattern edge has a slot at position >= MAX_ARITY. Quarantined
    /// (V3), never silently compacted.
    ArityOverflow,
    /// E_GUARDS endpoint is not a GUARD node.
    GuardEndpointNotGuard,
    /// GUARD node missing/malformed key, op, or value property.
    GuardMalformed,
    /// E_EFFECT endpoint is not an EFFECT node.
    EffectTargetNotEffect,
    /// EFFECT node missing/malformed key/op/value property.
    EffectMalformed,
    /// Source-var effect op has no source var wired.
    EffectMissingSource,
    /// Effect source edge points at a node that is not a var of this rule.
    EffectSourceNotAWireVar,
    DuplicateHasLabel,
    HasLabelEndpointNotLit,
    LitMissingLabelValue,
    LabelOutOfRange(i64),
}

impl std::fmt::Display for ReadRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadRejection::NoBecomesEdge => write!(f, "root has no E_BECOMES edge"),
            ReadRejection::MultipleBecomesEdges(n) => {
                write!(f, "{n} E_BECOMES edges on root")
            }
            ReadRejection::BecomesArity(n) => write!(f, "E_BECOMES arity {n} != 2"),
            ReadRejection::BothSidesMember => {
                write!(f, "pattern edge is MEMBER of both sides")
            }
            ReadRejection::MemberEndpointNotPatEdge => {
                write!(f, "MEMBER endpoint is not a PAT_EDGE")
            }
            ReadRejection::SlotEndpointNotVar => write!(f, "SLOT endpoint is not a VAR"),
            ReadRejection::SlotLabelOutOfRange(k) => {
                write!(f, "slot label {k} out of band")
            }
            ReadRejection::DuplicateHasLabel => write!(f, "duplicate HASLABEL on one VAR"),
            ReadRejection::HasLabelEndpointNotLit => {
                write!(f, "HASLABEL endpoint is not a LABEL_LIT")
            }
            ReadRejection::LitMissingLabelValue => {
                write!(f, "LABEL_LIT missing Int(K_LABEL_VALUE)")
            }
            ReadRejection::LabelOutOfRange(l) => write!(f, "label literal {l} out of u32 range"),
            ReadRejection::ArityOverflow => {
                write!(f, "pattern edge has a slot at position >= MAX_ARITY")
            }
            ReadRejection::GuardEndpointNotGuard => {
                write!(f, "E_GUARDS endpoint is not a GUARD node")
            }
            ReadRejection::GuardMalformed => {
                write!(f, "GUARD node missing or malformed key/op/value")
            }
            ReadRejection::EffectTargetNotEffect => {
                write!(f, "E_EFFECT endpoint is not an EFFECT node")
            }
            ReadRejection::EffectMalformed => {
                write!(f, "EFFECT node missing or malformed key/op/value")
            }
            ReadRejection::EffectMissingSource => {
                write!(f, "source-var effect op has no source var wired")
            }
            ReadRejection::EffectSourceNotAWireVar => {
                write!(f, "effect source edge points at a non-var node")
            }
        }
    }
}

/// Footprint of a region: node/edge counts for budget validators (V7/V9)
/// and the anchor set identity (plan §2.5 — gen-0 handles, never edge sets).
#[derive(Clone, Debug)]
pub struct RegionFootprint {
    pub nodes: usize,
    pub edges: usize,
}

/// Read one rule region by bounded walk from its root. Totality: the walk
/// visits root → sides → members/slots/haslabels, depth ≤ 3, and every
/// failure mode returns a typed [`ReadRejection`] rather than panicking or
/// looping. On success also reports the footprint (nodes/edges visited).
///
/// Semantic violations are the validator's job (V-classes); they land in
/// quarantine, not in this function's contract.
pub fn read_region_with_footprint(
    snap: &GraphSnapshot<'_>,
    root: NodeId,
) -> Result<(CandidateRule, RegionFootprint), ReadRejection> {
    read_region_impl(snap, root).map(|(r, n, e)| (r, RegionFootprint { nodes: n, edges: e }))
}

/// Convenience wrapper: just the candidate.
pub fn read_region(snap: &GraphSnapshot<'_>, root: NodeId) -> Result<CandidateRule, ReadRejection> {
    read_region_impl(snap, root).map(|(r, _, _)| r)
}

fn read_region_impl(
    snap: &GraphSnapshot<'_>,
    root: NodeId,
) -> Result<(CandidateRule, usize, usize), ReadRejection> {
    // --- hop 1: root → sides via the (single) E_BECOMES edge ---
    let mut becomess: Vec<(NodeId, NodeId)> = Vec::new();
    let mut footprint_edges: usize = 0;
    for e in snap.edges_incident_to(root) {
        let Some(edge) = snap.get_edge(*e) else {
            continue;
        };
        if edge.label.0 != E_BECOMES {
            continue;
        }
        let others: Vec<NodeId> = edge.nodes.iter().copied().filter(|n| *n != root).collect();
        if others.len() != 2 {
            return Err(ReadRejection::BecomesArity(others.len()));
        }
        becomess.push((others[0], others[1]));
    }
    match becomess.len() {
        0 => return Err(ReadRejection::NoBecomesEdge),
        1 => {}
        n => return Err(ReadRejection::MultipleBecomesEdges(n)),
    }
    let (side_l, side_r) = becomess[0];

    // --- hop 2: sides → MEMBER edges → pattern-edge nodes ---
    // Pattern edges are interned in canonical content order so var indexing
    // does not depend on edge enumeration order (D2 discipline).
    let mut members: Vec<(bool, EdgeRef)> = Vec::new(); // (is_lhs, pat_edge ref)
    for (side, is_lhs) in [(side_l, true), (side_r, false)] {
        for e in snap.edges_incident_to(side) {
            let Some(edge) = snap.get_edge(*e) else {
                continue;
            };
            if edge.label.0 != E_MEMBER {
                continue;
            }
            footprint_edges += 1;
            let Some(pe) = edge.nodes.iter().copied().find(|n| *n != side) else {
                continue;
            };
            members.push((is_lhs, EdgeRef::of(pe)));
        }
    }

    // Canonical pattern-edge order: content key of the PAT_EDGE node itself
    // (SortKey first), L-before-R stable tie-break, ffi key last for totality.
    let mut pe_refs: Vec<(SortKey, bool, EdgeRef)> = Vec::new();
    for (is_lhs, r) in members {
        let d = snap
            .get_node(r.node())
            .ok_or(ReadRejection::MemberEndpointNotPatEdge)?;
        if d.label.0 != L_PAT_EDGE {
            return Err(ReadRejection::MemberEndpointNotPatEdge);
        }
        pe_refs.push((node_sort_key(d), is_lhs, r));
    }
    pe_refs.sort();

    // A pattern edge reachable from BOTH sides would make survive/create/
    // delete classification ambiguous; the reader rejects it (validator
    // owns finer-grained structural checks).
    for w in pe_refs.windows(2) {
        if w[0].2.node() == w[1].2.node() && w[0].1 != w[1].1 {
            return Err(ReadRejection::BothSidesMember);
        }
    }

    // --- hop 3: pattern edges → SLOT_k edges → var nodes ---
    let mut var_ids: Vec<NodeId> = Vec::new();
    let mut var_index: HashMap<NodeId, usize> = HashMap::new();
    let intern = |v: NodeId, ids: &mut Vec<NodeId>, idx: &mut HashMap<NodeId, usize>| -> usize {
        *idx.entry(v).or_insert_with(|| {
            ids.push(v);
            ids.len() - 1
        })
    };

    let mut lhs: Vec<CandidatePatternEdge> = Vec::new();
    let mut rhs: Vec<CandidatePatternEdge> = Vec::new();
    for (_, is_lhs, pe_ref) in &pe_refs {
        let pe = pe_ref.node();
        let mut slots: Vec<(u32, usize)> = Vec::new();
        let mut slot_overflow = false;
        for e in snap.edges_incident_to(pe) {
            let Some(edge) = snap.get_edge(*e) else {
                continue;
            };
            let l = edge.label.0;
            // Accept ANY E_SLOT_k label; over-arity positions are recorded
            // and rejected by V3 below (quarantine, don't coerce — the old
            // `continue` silently produced a legal arity-8 edge from an
            // illegal arity-9 one).
            if !(E_SLOT_BASE..E_SLOT_BASE + u32::from(u8::MAX)).contains(&l) {
                continue;
            }
            let k = l - E_SLOT_BASE;
            if k >= MAX_ARITY as u32 {
                slot_overflow = true;
            }
            footprint_edges += 1;
            let Some(v) = edge.nodes.iter().copied().find(|n| *n != pe) else {
                continue;
            };
            let vd = snap.get_node(v).ok_or(ReadRejection::SlotEndpointNotVar)?;
            if vd.label.0 != L_VAR {
                return Err(ReadRejection::SlotEndpointNotVar);
            }
            let vi = intern(v, &mut var_ids, &mut var_index);
            slots.push((k, vi));
        }
        if slot_overflow {
            return Err(ReadRejection::ArityOverflow);
        }
        slots.sort_by_key(|(k, _)| *k);
        let edge = CandidatePatternEdge {
            slots: slots.into_iter().map(|(_, vi)| vi).collect(),
        };
        if *is_lhs {
            lhs.push(edge);
        } else {
            rhs.push(edge);
        }
    }

    // --- var label constraints via HASLABEL → LABEL_LIT(Int) ---
    let mut var_labels: Vec<Option<u32>> = vec![None; var_ids.len()];
    for (vi, v) in var_ids.iter().enumerate() {
        for e in snap.edges_incident_to(*v) {
            let Some(edge) = snap.get_edge(*e) else {
                continue;
            };
            if edge.label.0 != E_HASLABEL {
                continue;
            }
            footprint_edges += 1;
            let Some(lit) = edge.nodes.iter().copied().find(|n| n != v) else {
                continue;
            };
            let ld = snap
                .get_node(lit)
                .ok_or(ReadRejection::HasLabelEndpointNotLit)?;
            if ld.label.0 != L_LABEL_LIT {
                return Err(ReadRejection::HasLabelEndpointNotLit);
            }
            let value = ld.get_property(K_LABEL_VALUE);
            match value {
                Some(PropertyValue::Int(l)) => {
                    if !(0..=u32::MAX as i64).contains(&l) {
                        return Err(ReadRejection::LabelOutOfRange(l));
                    }
                    if var_labels[vi].is_some() {
                        return Err(ReadRejection::DuplicateHasLabel);
                    }
                    var_labels[vi] = Some(l as u32);
                }
                _ => return Err(ReadRejection::LitMissingLabelValue),
            }
        }
    }

    // --- E1-G guards: VAR → E_GUARDS → GUARD(key, op, val) ---
    // NOTE: E_GUARDS is also the generic reference edge for E2-lite source
    // wirings (EFFECT → source VAR). A var-side E_GUARDS edge whose other
    // endpoint is an EFFECT node belongs to that effect, not to a property
    // guard — skip it here; the effect walk below consumes it.
    let mut guards: Vec<CandidateGuard> = Vec::new();
    let mut guard_nodes = 0usize;
    for (vi, v) in var_ids.iter().enumerate() {
        for e in snap.edges_incident_to(*v) {
            let Some(edge) = snap.get_edge(*e) else {
                continue;
            };
            if edge.label.0 != E_GUARDS {
                continue;
            }
            let Some(g) = edge.nodes.iter().copied().find(|n| n != v) else {
                continue;
            };
            if snap.get_node(g).map(|d| d.label.0) == Some(L_EFFECT) {
                continue; // effect→source reference, not a property guard
            }
            footprint_edges += 1;
            // The other endpoint must be a GUARD node.
            let Some(g) = edge.nodes.iter().copied().find(|n| n != v) else {
                continue;
            };
            let gd = snap
                .get_node(g)
                .ok_or(ReadRejection::GuardEndpointNotGuard)?;
            if gd.label.0 != L_GUARD {
                return Err(ReadRejection::GuardEndpointNotGuard);
            }
            guard_nodes += 1;
            let key = match gd.get_property(K_GUARD_KEY) {
                Some(PropertyValue::Int(k)) if k >= 0 && k <= u32::MAX as i64 => k as u32,
                _ => return Err(ReadRejection::GuardMalformed),
            };
            // E1-G op codes: 0=Gt 1=Lt 2=Eq, extended (R5) by 3=Gte 4=Lte 5=Ne.
            let op = match gd.get_property(K_GUARD_OP) {
                Some(PropertyValue::Int(o)) if (0..=5).contains(&o) => o as u8,
                _ => return Err(ReadRejection::GuardMalformed),
            };
            let val = gd
                .get_property(K_GUARD_VAL)
                .ok_or(ReadRejection::GuardMalformed)?;
            guards.push(CandidateGuard {
                var: vi,
                key,
                op,
                val,
            });
        }
    }
    guards.sort_by(|a, b| {
        (a.var, a.key, a.op, prop_repr(&a.val)).cmp(&(b.var, b.key, b.op, prop_repr(&b.val)))
    });

    // --- E1-V var guards: ROOT → E_VAR_GUARD → VAR_GUARD{K_VGUARD_VARS+i} ---
    let mut var_guards: Vec<CandidateVarGuard> = Vec::new();
    let mut vguard_nodes = 0usize;
    for e in snap.edges_incident_to(root) {
        let Some(edge) = snap.get_edge(*e) else {
            continue;
        };
        if edge.label.0 != E_VAR_GUARD {
            continue;
        }
        footprint_edges += 1;
        let Some(g) = edge.nodes.iter().copied().find(|n| *n != root) else {
            continue;
        };
        let gd = snap.get_node(g).ok_or(ReadRejection::GuardMalformed)?;
        if gd.label.0 != L_GUARD {
            return Err(ReadRejection::GuardEndpointNotGuard);
        }
        vguard_nodes += 1;
        // Var list: consecutive Int keys from K_VGUARD_VARS.
        let mut vars: Vec<usize> = Vec::new();
        let mut k = K_VGUARD_VARS;
        while let Some(PropertyValue::Int(vi)) = gd.get_property(k) {
            if vi < 0 || vi as usize >= var_ids.len() {
                return Err(ReadRejection::GuardMalformed);
            }
            vars.push(vi as usize);
            k += 1;
        }
        if vars.len() < 2 {
            // vars_distinct over <2 vars is vacuous; treat as malformed so a
            // truncated property write cannot silently weaken the rule.
            return Err(ReadRejection::GuardMalformed);
        }
        vars.sort();
        var_guards.push(CandidateVarGuard { vars });
    }
    var_guards.sort();

    // --- E2-lite effects: VAR → E_EFFECT → EFFECT{key, op, val[, source]} ---
    // Source-var ops wire the SOURCE var by a second edge from the effect
    // node, labelled E_GUARDS (the generic "references" role in this band).
    // Op codes 1 (CopyFrom), 5 (AddFromProperty), 6 (SubtractFromProperty)
    // fall into the catch-all source-var arm below.
    #[allow(dead_code)]
    const OP_COPYFROM: u8 = 1;
    const OP_SET: u8 = 0;
    const OP_SCALEFROM: u8 = 2;
    const OP_ADD: u8 = 3;
    const OP_SUBTRACT: u8 = 4;
    #[allow(dead_code)]
    const OP_ADDFROMPROPERTY: u8 = 5;
    #[allow(dead_code)]
    const OP_SUBTRACTFROMPROPERTY: u8 = 6;
    const OP_JITTER: u8 = 7;
    let mut effects: Vec<CandidateEffect> = Vec::new();
    for (vi, v) in var_ids.iter().enumerate() {
        for e in snap.edges_incident_to(*v) {
            let Some(edge) = snap.get_edge(*e) else {
                continue;
            };
            if edge.label.0 != E_EFFECT {
                continue;
            }
            footprint_edges += 1;
            let Some(en) = edge.nodes.iter().copied().find(|n| n != v) else {
                continue;
            };
            let ed = snap.get_node(en).ok_or(ReadRejection::EffectMalformed)?;
            if ed.label.0 != L_EFFECT {
                return Err(ReadRejection::EffectTargetNotEffect);
            }
            let key = match ed.get_property(K_EFFECT_KEY) {
                Some(PropertyValue::Int(k)) if k >= 0 && k <= u32::MAX as i64 => k as u32,
                _ => return Err(ReadRejection::EffectMalformed),
            };
            let op = match ed.get_property(K_EFFECT_OP) {
                Some(PropertyValue::Int(o)) if (0..=7).contains(&o) => o as u8,
                _ => return Err(ReadRejection::EffectMalformed),
            };
            // Constant-payload ops: Set/Add/Subtract value, ScaleFrom/Jitter
            // magnitude. Source ops carry source var via an E_GUARDS edge.
            match op {
                OP_SET | OP_ADD | OP_SUBTRACT | OP_JITTER | OP_SCALEFROM => {
                    let val = ed
                        .get_property(K_EFFECT_VAL)
                        .ok_or(ReadRejection::EffectMalformed)?;
                    effects.push(CandidateEffect {
                        target: vi,
                        key,
                        op,
                        val: Some(val),
                        source: None,
                        source_key: None,
                    });
                }
                _ => {
                    // Source-var op: find the wired source var.
                    let mut src_vi = None;
                    for se in snap.edges_incident_to(en) {
                        let Some(sedge) = snap.get_edge(*se) else {
                            continue;
                        };
                        if sedge.label.0 != E_GUARDS
                            || !sedge.nodes.contains(&en)
                            || sedge.nodes.len() != 2
                        {
                            continue;
                        }
                        let Some(sv) = sedge.nodes.iter().copied().find(|n| *n != en) else {
                            continue;
                        };
                        if sv == *v {
                            continue; // self-reference is not a source wiring
                        }
                        let Some(pos) = var_ids.iter().position(|x| x == &sv) else {
                            return Err(ReadRejection::EffectSourceNotAWireVar);
                        };
                        src_vi = Some(pos);
                        break;
                    }
                    let source = src_vi.ok_or(ReadRejection::EffectMissingSource)?;
                    // Source key: stored on the effect node under
                    // K_EFFECT_VAL as Int when present, else malformed.
                    let src_key = match ed.get_property(K_EFFECT_VAL) {
                        Some(PropertyValue::Int(sk)) if sk >= 0 && sk <= u32::MAX as i64 => {
                            Some(sk as u32)
                        }
                        _ => return Err(ReadRejection::EffectMalformed),
                    };
                    effects.push(CandidateEffect {
                        target: vi,
                        key,
                        op,
                        val: None,
                        source: Some(source),
                        source_key: src_key,
                    });
                }
            }
        }
    }
    effects.sort_by(|a, b| (a.target, a.key, a.op).cmp(&(b.target, b.key, b.op)));

    // Footprint: every node and edge the walk touched. Nodes = root + sides
    // + pattern edges + vars (+ label lits, counted via constraints); edges
    // counted at each hop below.
    let label_lits = var_labels.iter().filter(|l| l.is_some()).count();
    Ok((
        CandidateRule {
            priority: read_priority(snap, root),
            lhs,
            rhs,
            var_labels,
            guards,
            var_guards,
            effects,
        },
        1 + 2 + pe_refs.len() + var_ids.len() + label_lits + guard_nodes + vguard_nodes,
        footprint_edges,
    ))
}

fn read_priority(snap: &GraphSnapshot<'_>, root: NodeId) -> i32 {
    match snap.get_node(root).and_then(|d| d.get_property(K_PRIORITY)) {
        Some(PropertyValue::Int(i)) => i as i32,
        _ => 0,
    }
}

// ---------- internal helpers ----------

/// Opaque copyable reference to a node, keeping sort keys beside identity so
/// canonical ordering needs no second lookup pass. Ordering is by raw ffi
/// key; the content SortKey that should dominate lives in the enclosing
/// tuple, so this Ord only makes the tuple's last slot total.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeRef(NodeId);

impl EdgeRef {
    fn of(n: NodeId) -> Self {
        Self(n)
    }
    fn node(&self) -> NodeId {
        self.0
    }
}

type SortKey = (u32, u64, Vec<(u32, u8, i64, u64)>);
