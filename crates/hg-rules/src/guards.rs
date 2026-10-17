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

/// A **derived** node quantity — one computed from the node's place in the
/// graph rather than read out of its property map.
///
/// This is an enum rather than a bare "degree" field so that
/// [`NodeQuantityCompare`] is a general var-to-var comparison form: a second
/// quantity is a new VARIANT, not a new guard. Only `Degree` is implemented,
/// because only `Degree` has a use — adding unused variants to look general
/// would be its own kind of overfitting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeQuantity {
    /// Count of DISTINCT incident hyperedges. Semantics are **inherited
    /// verbatim** from [`NodeDegreeCompare`] (a node appearing twice in one
    /// edge counts that edge once); the two guards must never disagree about
    /// what a degree is.
    Degree,
}

/// Var-to-var comparison of a derived node quantity (E-track):
/// `quantity(var_a) <op> (quantity(var_b) + offset)`, Int arithmetic,
/// saturating.
///
/// **Why this exists.** [`NodeDegreeCompare`] compares a node quantity to a
/// **constant**, and [`NodePropertyCompare`] compares two vars but only through
/// the property map. Neither can express "the higher-degree of two matched
/// nodes", so a self-balancing degree-changing rewrite was **inexpressible** —
/// the D-track hit that wall directly: an unbounded rewire has an absorbing
/// state (one node joins every edge) and the corrected version needed
/// substrate-derived constant bounds to be perpetual at all.
///
/// **Evaluation is a POST-BINDING FILTER.** Unlike the var-to-constant form,
/// this cannot be checked until both vars are bound, so it runs once every
/// variable has a node — at the same point as `node_property_compares`, and
/// appended after the existing guards so no existing guard's short-circuit
/// position moves (guard counters are position-sensitive by design). It costs
/// `O(deg(a) + deg(b))` per COMPLETED candidate and does **not** shrink the
/// candidate set enumeration builds: a rule with a quadratic LHS stays
/// quadratic and merely filters more of what it enumerated.
///
/// Failure semantics mirror the other guards exactly: an unbound var or a node
/// missing from the snapshot makes the guard **false**, never a panic. No RNG
/// is drawn, and the comparison is `i64`, so there is no float-reproducibility
/// surface.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeQuantityCompare {
    pub var_a: String,
    pub quantity: NodeQuantity,
    pub op: PropertyOp,
    pub var_b: String,
    /// Constant added to **side B** before comparing, saturating — the same
    /// offset machinery as the other two compare guards. It is what makes
    /// strict and non-strict forms both expressible:
    /// `degree(a) > degree(b)` versus `degree(a) >= degree(b) + 1`.
    #[serde(default)]
    pub offset: Option<i64>,
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
