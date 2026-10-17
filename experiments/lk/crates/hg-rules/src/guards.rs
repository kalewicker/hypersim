//! Guards for rule matching (typed via PropertyValue, v4 §3)

use hg_core::{PropertyOp, PropertyValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodePropertyGuard {
    pub var: String,
    pub property: u32,
    pub op: PropertyOp,
    /// Threshold; TOML literal syntax selects the variant (untagged).
    pub value: PropertyValue,
}

/// Property-to-property comparison across (possibly the same) variables — the
/// hg-alife S1 extension (scope §5.3). Where `NodePropertyGuard` compares a
/// property to a TOML constant, this compares `var_a[key_a] <op> var_b[key_b]`,
/// enabling genome-dependent guards (e.g. `carrier.energy >= carrier.repl_thr`).
/// `var_a`/`var_b` may be the same variable (self-comparison across two keys)
/// or different (cross-node). Evaluation mirrors `NodePropertyGuard` exactly:
/// variant mismatch ⟹ false (no coercion), missing property on either side ⟹
/// false, no RNG drawn.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodePropertyCompare {
    pub var_a: String,
    pub key_a: u32,
    pub op: PropertyOp,
    pub var_b: String,
    pub key_b: u32,
    /// Optional constant added to side B before comparing:
    /// `var_a[key_a] <op> (var_b[key_b] + offset)` — the LK-R transport
    /// guard's `x.energy >= y.energy + 2τ` shape (plan v2 §6.1). `None`
    /// is bit-identical to the pre-offset semantics. Variant rules follow
    /// the guard discipline exactly: Int+Int and Float+Float only; any
    /// mismatch makes the guard false (no coercion), and no RNG is drawn.
    #[serde(default)]
    pub offset: Option<PropertyValue>,
}

/// Degree comparison guard (plan v4 §6.2, `node_degree_compares`):
/// `degree(var) <op> (value + offset)`, Int arithmetic, saturating.
/// Degree = count of DISTINCT incident hyperedges (a node appearing
/// twice in one edge counts that edge once) — pinned semantics, tested
/// on multi-incidence. Counters wired from birth (§1.1).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeDegreeCompare {
    pub var: String,
    pub op: PropertyOp,
    pub value: i64,
    #[serde(default)]
    pub offset: Option<i64>,
}

/// Age comparison guard (plan v8 §3.1, `node_age_compares`):
/// `(step − birth_step(var)) <op> value`, Int arithmetic, saturating.
///
/// The birth step is the node's creation step, already recorded on
/// `NodeData::birth_step` by every creation site — so this guard reads
/// state the store already holds and needs no property key, no stamping
/// effect, and no per-step rule (B0 question 1; v8 §1.1 design order a).
/// Evaluated in the same post-candidate filter as the property and degree
/// guards, so it touches neither matcher enumeration nor conflict
/// resolution (B0 question 2, the stop condition).
///
/// SEMANTIC NOTE (B0 question 4): every node has a birth step, including
/// IC nodes, which read as born at step 0 and are therefore maximally
/// OLD. Unlike the countdown gate — where a missing key rejected — this
/// guard supplies no protection for the original lattice. Rules that
/// relied on that incidental protection must keep their own guards.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeAgeCompare {
    pub var: String,
    pub op: PropertyOp,
    pub value: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VarGuard {
    pub guard_type: VarGuardType,
    pub vars: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarGuardType {
    VarsDistinct,
}
