//! Validators V1–V9 (plan §5.4) — semantic quarantine classes.
//!
//! PROVENANCE NOTE: the plan's §5.4 section defining V1..V9 is ABSENT from
//! the v1 document body (§5 jumps from the phase list onward). These nine
//! are instantiated here from E1's actual failure modes; each entry names
//! the predicate, what it protects downstream, and its engagement-counter
//! key (G-R4 requires every counter present in every run's output, zeros
//! included). Flagged to the plan owner for ratification or amendment.
//!
//! Distinction from [`crate::schema::ReadRejection`]: read rejections mean
//! "no rule could be formed"; validator hits mean "a candidate formed but
//! is quarantined" — the region stays in the graph, the rule never
//! installs.

use crate::schema::{CandidateRule, RegionFootprint, MAX_ARITY, META_BASE};

/// Budget ceilings (FM-R3 bloat guard). Binding/non-binding status is
/// logged by the compile layer every run — a result produced under a
/// binding ceiling is a result about the ceiling.
pub const REGION_NODE_BUDGET: usize = 256;
pub const REGION_EDGE_BUDGET: usize = 512;

/// One validator verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Passes all validators; installs.
    Valid,
    /// Fails exactly this validator (first hit wins, deterministic order
    /// V1..V9 so quarantine accounting is stable).
    Quarantined(ValidatorId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ValidatorId {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
}

impl ValidatorId {
    /// Engagement-counter key for metrics rows and G-R4 reporting.
    pub fn counter_key(self) -> &'static str {
        match self {
            ValidatorId::V1 => "v1_lhs_empty",
            ValidatorId::V2 => "v2_arity_zero",
            ValidatorId::V3 => "v3_arity_overflow",
            ValidatorId::V4 => "v4_meta_label",
            ValidatorId::V5 => "v5_priority_malformed",
            ValidatorId::V6 => "v6_repeated_var_in_edge",
            ValidatorId::V7 => "v7_node_budget",
            ValidatorId::V8 => "v8_orphan_fragment",
            ValidatorId::V9 => "v9_edge_budget",
        }
    }

    pub const ALL: [ValidatorId; 9] = [
        ValidatorId::V1,
        ValidatorId::V2,
        ValidatorId::V3,
        ValidatorId::V4,
        ValidatorId::V5,
        ValidatorId::V6,
        ValidatorId::V7,
        ValidatorId::V8,
        ValidatorId::V9,
    ];
}

/// Per-run engagement counters: one slot per validator, zeros included
/// (G-R4: "a zero here is a measurement, not an absence").
#[derive(Clone, Debug, Default)]
pub struct EngagementCounters(pub [usize; 9]);

impl EngagementCounters {
    pub fn record(&mut self, id: ValidatorId) {
        self.0[id as usize] += 1;
    }

    pub fn get(&self, id: ValidatorId) -> usize {
        self.0[id as usize]
    }
}

/// Priority property as observed by V5: `None` = absent (defaults to 0),
/// `Some(Err)` = present but malformed.
#[derive(Clone, Copy, Debug)]
pub struct PriorityObservation {
    pub malformed: bool,
}

/// Validate one candidate. `footprint` comes from the reader;
/// `priority_malformed` is observed by the compile layer where the raw
/// property is still available (the reader coerces silently to keep its
/// own contract total).
///
/// Deterministic order V1..V9; first failure wins.
pub fn validate(
    rule: &CandidateRule,
    footprint: &RegionFootprint,
    priority: &PriorityObservation,
) -> Verdict {
    // V1 — LHS empty: creation rules cannot be expressed endogenously
    // (SB-R2); an encoded rule must consume at least one pattern edge or it
    // is not a rewrite but an unguarded emission.
    if rule.lhs.is_empty() {
        return Verdict::Quarantined(ValidatorId::V1);
    }

    // V2 — arity-0 pattern edge: matches nothing, fires never; pure bloat
    // that would inflate match cost (FM-R3 channel).
    if rule
        .lhs
        .iter()
        .chain(rule.rhs.iter())
        .any(|e| e.slots.is_empty())
    {
        return Verdict::Quarantined(ValidatorId::V2);
    }

    // V3 — arity above MAX_ARITY after compaction: the encoding caps
    // expressible arity (SB-R1); over-cap edges cannot round-trip to Rule.
    if rule
        .lhs
        .iter()
        .chain(rule.rhs.iter())
        .any(|e| e.slots.len() > MAX_ARITY)
    {
        return Verdict::Quarantined(ValidatorId::V3);
    }

    // V4 — label constraint inside the meta band: an ordinary rule whose
    // pattern demands meta-band nodes would match the encoding itself
    // (live self-reference is out of scope for v1; §11 epoch boundary).
    if rule.var_labels.iter().flatten().any(|l| *l >= META_BASE) {
        return Verdict::Quarantined(ValidatorId::V4);
    }

    // V5 — priority property malformed (present, non-Int, or out of i32):
    // the reader would coerce to 0, silently changing semantics; quarantine
    // instead so the mutation that broke it is visible in accounting.
    if priority.malformed {
        return Verdict::Quarantined(ValidatorId::V5);
    }

    // V6 — same var twice within ONE pattern edge: hg-engine matches
    // endpoints as unordered multisets, so a repeated var is either always
    // false (strict distinctness assumed) or degenerate (same node bound
    // twice); both make the rule dead-on-arrival or FM-class.
    for side in [&rule.lhs, &rule.rhs] {
        for e in side.iter() {
            let mut seen: Vec<usize> = Vec::with_capacity(e.slots.len());
            for s in &e.slots {
                if seen.contains(s) {
                    return Verdict::Quarantined(ValidatorId::V6);
                }
                seen.push(*s);
            }
        }
    }

    // V7 / V9 — region budgets (FM-R3): node and edge ceilings on the
    // region footprint. Logged binding/non-binding by the caller.
    if footprint.nodes > REGION_NODE_BUDGET {
        return Verdict::Quarantined(ValidatorId::V7);
    }
    if footprint.edges > REGION_EDGE_BUDGET {
        return Verdict::Quarantined(ValidatorId::V9);
    }

    // V8 — orphan fragment: reserved for the compile layer's adjacency
    // audit (meta-band nodes adjacent to the region but unreachable from
    // its root within r_meta). The offline harness has no snapshot access
    // at this point, so it reports Valid here and the compile layer
    // records V8 separately. Documented as a two-stage validator.
    let _ = ();

    Verdict::Valid
}
