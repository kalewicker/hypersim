//! Causal-language types: attributes, tasks, claims, constructors.

use hg_core::WlHash;
use serde::{Deserialize, Serialize};

/// A structural attribute: a WL-class identity observed in the graph.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    pub wl_hash: WlHash,
    pub radius: u8,
    pub motif_class: u32,
    pub node_count: u32,
    pub certainty: CertaintyClass,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub input: Attribute,
    pub output: Attribute,
    pub status: TaskStatus,
    pub certainty: CertaintyClass,
    pub witness_count: u32,
    pub first_seen: u64,
    pub last_seen: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Candidate,
    Confirmed,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CertaintyClass {
    Unknown,
    Intuition,
    Model,
    Empirical,
    Axiom,
    Law,
}

/// Closed, exhaustively-matched predicate vocabulary (v4 §6.4) — no
/// `Custom` escape hatch, deliberately: adding a real variant costs one
/// small edit and keeps exhaustiveness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Predicate {
    /// Subject class persisted across a window boundary.
    PersistsAcross,
    /// Subject class transformed into object class.
    TransformsInto,
    /// Subject class catalyzed a task while itself persisting.
    Catalyzes,
    /// Subject class's population grew while persisting (replication).
    Replicates,
    /// Subject class appeared for the first time.
    EmergesAt,
    /// Subject class disappeared.
    VanishesAt,
    /// Subject (a coarse-graining strategy) exhibits the object CE
    /// coefficient — EI_macro / EI_micro at the recorded step.
    ExhibitsCe,
}

impl Predicate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Predicate::PersistsAcross => "persists_across",
            Predicate::TransformsInto => "transforms_into",
            Predicate::Catalyzes => "catalyzes",
            Predicate::Replicates => "replicates",
            Predicate::EmergesAt => "emerges_at",
            Predicate::VanishesAt => "vanishes_at",
            Predicate::ExhibitsCe => "exhibits_ce",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub id: u64,
    /// Subject: a WL class (hex-formatted hash) or task id.
    pub subject: String,
    pub predicate: Predicate,
    pub object: String,
    pub certainty: CertaintyClass,
    pub step: u64,
}

/// What kind of replication a constructor exhibits.
///
/// `Trivial`: the rule's own mechanical action yields more instances of
/// the pattern (a self-similar rewrite copying structure for free) — the
/// expected kind on a bare substrate. `NonTrivial`: a bounded, persistent
/// individual producing a copy of its whole self — requires the ALife
/// layer's mobile/persistent individual tracking and is NEVER assigned by
/// the substrate-only causal layer. Experiment 6's "replication onset"
/// baseline is the non-trivial count, which on bare substrate must be 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReplicatorKind {
    #[default]
    None,
    Trivial,
    NonTrivial,
}

impl ReplicatorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReplicatorKind::None => "no",
            ReplicatorKind::Trivial => "trivial",
            ReplicatorKind::NonTrivial => "non-trivial",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstructorRecord {
    pub pattern: Attribute,
    pub task_catalyzed: u64,
    pub confirmed_cycles: u32,
    pub degraded_cycles: u32,
    /// Convenience: `replicator_kind != None`.
    pub is_replicator: bool,
    pub replicator_kind: ReplicatorKind,
    pub certainty: CertaintyClass,
    pub fidelity: f32,
}
