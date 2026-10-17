//! Genome trait keys, encoding to/from `PropertyValue`, and trait bounds.
//!
//! The genome is **continuous, one Float property per trait** (scope §2.1) —
//! NOT bit-packed — because guards cannot decode bits. Mutation is `jitter`
//! (the existing property-effect op); inheritance is `copy_from` then `jitter`.
//! Keys are registered in `docs/property_registry.md` (the ALife key block).

use hg_core::{NodeData, PropertyValue};

/// Property keys shared with `alife_base.toml`. Energy is the pre-existing
/// key-1 convention; the genome traits get a dedicated block.
pub mod keys {
    /// Energy (Float) — the node-funded replication economy reads/writes this.
    pub const ENERGY: u32 = 1;
    /// Genome: harvest efficiency (deferred as an *active* trait in Exp. 5).
    pub const HARVEST_RATE: u32 = 10;
    /// Genome: energy at/above which a carrier may replicate.
    pub const REPLICATION_THRESHOLD: u32 = 11;
    /// Genome: metabolic drain (deferred as an *active* trait in Exp. 5).
    pub const MAINTENANCE_COST: u32 = 12;
    /// Genome: energy at/above which a carrier stops harvesting (§6.2 crowding).
    pub const SATIATION: u32 = 13;
    /// Bootstrap seed marker (Bool) — set by the generator, cleared once.
    pub const CARRIER_MARKER: u32 = 20;
    /// Carrier age counter (Float; reserved, inactive in Exp. 5).
    pub const LIFETIME: u32 = 21;
    /// Site occupancy flag (Bool) — a rule-maintained NAC surrogate. `true` iff a
    /// carrier occupies this substrate node. The matcher has no negative
    /// application condition (`VarGuardType` = `VarsDistinct` only), so
    /// "disperse a child onto an *unoccupied* neighbour" cannot be expressed
    /// structurally; instead `birth-dispersal` guards `target.occupied == false`
    /// and sets it `true`, `die_and_recycle` clears it `false`, and
    /// `bootstrap_seed` sets the founding site `true`. Because this flag is
    /// rule-maintained state that parallels the graph, it MUST be checked against
    /// the graph every step: `count(occupied sites) == count(carriers)` (see
    /// `alife_dispersal.rs`). Initialised `false` on every substrate node at run
    /// start (analogue of `initial_substrate_energy`).
    pub const OCCUPIED: u32 = 22;
}

/// The minimal active trait set for Experiment 5 (scope §11 Q1):
/// {harvest_rate, replication_threshold, maintenance_cost, satiation}.
pub const GENOME_TRAIT_KEYS: [u32; 4] = [
    keys::HARVEST_RATE,
    keys::REPLICATION_THRESHOLD,
    keys::MAINTENANCE_COST,
    keys::SATIATION,
];

/// Nominal range of a trait, used for sampling initial genomes and for
/// documenting where jitter drift is "in range".
///
/// **Jitter does NOT clamp against these** (scope §S2.1, option (b)): the
/// engine stays ALife-agnostic and the biology must be robust to out-of-range
/// traits (a negative `harvest_rate` simply never harvests). These bounds are
/// genome *semantics* — sampling + human reasoning — not an enforced invariant.
#[derive(Clone, Copy, Debug)]
pub struct TraitBounds {
    pub key: u32,
    pub min: f64,
    pub max: f64,
}

const TRAIT_BOUNDS: [TraitBounds; 4] = [
    TraitBounds { key: keys::HARVEST_RATE, min: 0.0, max: 2.0 },
    TraitBounds { key: keys::REPLICATION_THRESHOLD, min: 1.0, max: 50.0 },
    TraitBounds { key: keys::MAINTENANCE_COST, min: 0.0, max: 5.0 },
    TraitBounds { key: keys::SATIATION, min: 1.0, max: 50.0 },
];

/// The four active traits' nominal ranges (see [`TraitBounds`]).
pub fn trait_bounds() -> &'static [TraitBounds] {
    &TRAIT_BOUNDS
}

/// Read a Float property as `f64`; `None` if absent or a non-Float variant.
pub fn read_float(node: &NodeData, key: u32) -> Option<f64> {
    match node.get_property(key) {
        Some(PropertyValue::Float(v)) => Some(v),
        _ => None,
    }
}

/// A carrier's genome snapshotted from its node properties. Used by the
/// lineage tree (S5) to record traits at birth. A missing/non-Float trait
/// reads as `None` — a diagnosable defect, never a silent 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Genome {
    pub harvest_rate: Option<f64>,
    pub replication_threshold: Option<f64>,
    pub maintenance_cost: Option<f64>,
    pub satiation: Option<f64>,
}

impl Genome {
    /// Read the four active traits from a node's properties.
    pub fn from_node(node: &NodeData) -> Self {
        Self {
            harvest_rate: read_float(node, keys::HARVEST_RATE),
            replication_threshold: read_float(node, keys::REPLICATION_THRESHOLD),
            maintenance_cost: read_float(node, keys::MAINTENANCE_COST),
            satiation: read_float(node, keys::SATIATION),
        }
    }
}
