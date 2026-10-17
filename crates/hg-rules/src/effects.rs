//! `property_effects`: property WRITES in the rewrite system (v4 §5.2 /
//! §6.3 plus `jitter`) — the substrate primitive the ALife node-funded
//! replication mechanic requires.
//!
//! Raw (parsed, string var names) and compiled (RHS-index-resolved)
//! forms. Semantics live in hg-engine's apply step 5.5: effects run in
//! declaration order, and each effect's reads see earlier effects'
//! writes (docs/rule_grammar.md).

use hg_core::PropertyValue;
use serde::{Deserialize, Serialize};

/// Parsed effect operation, variables still by name.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectOpDecl {
    /// target[key] = value (any PropertyValue variant).
    Set { value: PropertyValue },
    /// target[key] = source[source_key] (skip if source key missing).
    CopyFrom { source_var: String, source_key: u32 },
    /// target[key] = Float(source Float × factor); source must be Float.
    ScaleFrom {
        source_var: String,
        source_key: u32,
        factor: f64,
    },
    /// target[key] += value; Int+Int (saturating) or Float+Float.
    Add { value: PropertyValue },
    /// target[key] −= value; Int−Int (saturating) or Float−Float.
    Subtract { value: PropertyValue },
    /// target[key] += source[source_key] (Float only); the amount is a PROPERTY,
    /// not a constant — the genome-scaled effect op deviation #4 needs (harvest a
    /// carrier's `harvest_rate` from a site). Skips if either property is missing
    /// or non-Float.
    AddFromProperty { source_var: String, source_key: u32 },
    /// target[key] −= source[source_key] (Float only). Genome-scaled sink
    /// (metabolise a carrier's `maintenance_cost`). Skips if missing/non-Float.
    SubtractFromProperty { source_var: String, source_key: u32 },
    /// target[key] += uniform(−m, +m); Float continuous, Int integer
    /// delta in [−⌊m⌋, +⌊m⌋]. The stochastic mutation primitive.
    Jitter { magnitude: f64 },
}

/// One parsed property effect on the RHS.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyEffectDecl {
    /// Target: an RHS variable name.
    pub var: String,
    /// Property key on the target node.
    pub key: u32,
    pub op: EffectOpDecl,
}

/// Compiled effect operation: variables resolved to RHS indices.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum CompiledEffectOp {
    Set { #[serde(with = "hg_core::wire::value")] value: PropertyValue },
    CopyFrom { source_ix: usize, source_key: u32 },
    ScaleFrom {
        source_ix: usize,
        source_key: u32,
        factor: f64,
    },
    Add { #[serde(with = "hg_core::wire::value")] value: PropertyValue },
    Subtract { #[serde(with = "hg_core::wire::value")] value: PropertyValue },
    AddFromProperty { source_ix: usize, source_key: u32 },
    SubtractFromProperty { source_ix: usize, source_key: u32 },
    Jitter { magnitude: f64 },
}

/// One compiled property effect (target resolved to an RHS index).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PropertyEffect {
    pub target_ix: usize,
    pub key: u32,
    pub op: CompiledEffectOp,
}
