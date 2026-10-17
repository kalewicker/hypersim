//! Read-only GUI-facing DTOs (v4 §2.1).
//!
//! `CompactGraphSnapshot` uses the real engine identifiers (`NodeId`/
//! `EdgeId`) — not a decoupled GUI-only id scheme — so a canvas click can
//! look up the exact same node in the causal inspector with no translation
//! layer. `KnowledgeGraphView` is a flattened, display-only projection of
//! hg-causal's much richer internal types.

use hg_core::{EdgeId, Label, NodeId};
use hg_schema::{GateName, GateStatus, MetricsRow, Regime};

#[derive(Clone, Debug, Default)]
pub struct CompactGraphSnapshot {
    pub step: u64,
    pub nodes: Vec<CompactNode>,
    pub edges: Vec<CompactEdge>,
}

#[derive(Clone, Debug)]
pub struct CompactNode {
    pub id: NodeId,
    pub label: Label,
    pub generation: u64,
    /// Energy proxy for GUI color mode `E` (blue→red).
    pub energy: f32,
}

#[derive(Clone, Debug)]
pub struct CompactEdge {
    pub id: EdgeId,
    pub label: Label,
    pub nodes: Vec<NodeId>,
}

#[derive(Clone, Debug, Default)]
pub struct MetricsHistory {
    pub rows: Vec<MetricsRow>,
}

#[derive(Clone, Debug, Default)]
pub struct EvalState {
    pub current_regime: Regime,
    pub regime_history: Vec<Regime>,
    pub pi_history: Vec<f64>,
    pub interestingness_history: Vec<f64>,
    pub gate_statuses: Vec<(GateName, GateStatus)>,
    pub science_unlocked: bool,
}

/// Flattened, display-only knowledge-graph projection.
#[derive(Clone, Debug, Default)]
pub struct KnowledgeGraphView {
    pub tasks: Vec<TaskLatticeRow>,
    pub constructors: Vec<ConstructorRow>,
    pub claims: Vec<ClaimRow>,
    /// Latest per-strategy EI/CE results (empty until the emission
    /// interval and minimum-transition gate are crossed).
    pub ei_results: Vec<EiResultRow>,
}

/// Display projection of one EI/CE measurement.
#[derive(Clone, Debug)]
pub struct EiResultRow {
    pub strategy: String,
    pub ei_micro: f64,
    pub ei_macro: f64,
    pub ce_coefficient: f64,
    pub transition_count: u64,
    pub cadence_steps: u64,
    /// M(t): WL-class registry size at emission.
    pub class_registry_size: u32,
}

#[derive(Clone, Debug)]
pub struct TaskLatticeRow {
    pub id: u64,
    pub input_hash: u64,
    pub output_hash: u64,
    pub status: String,
    pub certainty: String,
    pub witness_count: u32,
}

#[derive(Clone, Debug)]
pub struct ConstructorRow {
    pub wl_hash: u64,
    pub fidelity: f32,
    pub confirmed_cycles: u32,
    pub is_replicator: bool,
    /// "no" / "trivial" (structural, rule-mechanical) / "non-trivial"
    /// (bounded self-copying individual — ALife layer only).
    pub replicator_kind: String,
    pub certainty: String,
}

#[derive(Clone, Debug)]
pub struct ClaimRow {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub certainty: String,
    pub step: u64,
}
