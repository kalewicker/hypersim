//! ERE's additive metrics. Ordinary MetricsRow serialization is deliberately
//! unchanged; the ERE runner emits this structured record alongside it.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleReplicatorKind {
    None,
    Trivial,
    NonTrivial,
}

/// Measurements of one substrate projection. Unlike legacy MetricsRow, every
/// value can be absent: an unobserved match pool must never look like zero.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct EreSubstrateMetrics {
    pub n_nodes: Option<usize>,
    pub n_edges: Option<usize>,
    pub accepted_rewrites: Option<usize>,
    pub attempted_matches: Option<usize>,
    pub accepted_per_step: Option<f64>,
    pub d_eff: Option<f64>,
    pub spectral_dim: Option<f64>,
    pub diameter: Option<f64>,
    pub mean_degree: Option<f64>,
    pub activity: Option<f64>,
    pub activity_cv: Option<f64>,
    pub entropy_rate: Option<f64>,
    pub pi_normalized: Option<f64>,
    pub pi_sample_size: Option<usize>,
    pub pi_sample_strategy: Option<String>,
    pub ce_structural: Option<f64>,
    pub ce_energetic: Option<f64>,
    pub ce_functional: Option<f64>,
    pub ei_micro: Option<f64>,
    pub ei_macro: Option<f64>,
    pub ce_best: Option<f64>,
    pub ce_best_strategy: Option<String>,
    pub wl_class_count: Option<usize>,
    pub task_lattice_depth: Option<usize>,
    pub constructor_count: Option<usize>,
    pub replicator_count: Option<usize>,
    pub regime: Option<String>,
    pub new_type_bin_rate: Option<f64>,
    pub type_histogram_entropy: Option<f64>,
    pub dominant_type_fraction: Option<f64>,
    pub type_histogram_jsd: Option<f64>,
    pub active_persistent_structures: Option<usize>,
    pub static_repeated_motifs: Option<usize>,
    pub static_persistent_motifs: Option<usize>,
    pub archive_size: Option<usize>,
    pub archive_evictions: Option<usize>,
    pub new_signatures: Option<usize>,
    pub archive_capacity: Option<usize>,
    pub interestingness_score: Option<f64>,
    pub unmeasured_reasons: BTreeMap<String, String>,
}

impl From<&crate::MetricsRow> for EreSubstrateMetrics {
    fn from(row: &crate::MetricsRow) -> Self {
        Self {
            n_nodes: Some(row.n_nodes),
            n_edges: Some(row.n_edges),
            accepted_rewrites: Some(row.accepted_rewrites),
            attempted_matches: Some(row.attempted_matches),
            accepted_per_step: Some(row.accepted_per_step),
            d_eff: row.d_eff,
            spectral_dim: row.spectral_dim,
            diameter: row.diameter,
            mean_degree: row.mean_degree,
            activity: row.activity,
            activity_cv: row.activity_cv,
            entropy_rate: row.entropy_rate,
            pi_normalized: row.pi_normalized,
            pi_sample_size: row.pi_sample_size,
            pi_sample_strategy: row.pi_sample_strategy.clone(),
            ce_structural: row.ce_structural,
            ce_energetic: row.ce_energetic,
            ce_functional: row.ce_functional,
            ei_micro: row.ei_micro,
            ei_macro: row.ei_macro,
            ce_best: row.ce_best,
            ce_best_strategy: row.ce_best_strategy.clone(),
            wl_class_count: row.wl_class_count,
            task_lattice_depth: row.task_lattice_depth,
            constructor_count: row.constructor_count,
            replicator_count: row.replicator_count,
            regime: row.regime.clone(),
            new_type_bin_rate: row.new_type_bin_rate,
            type_histogram_entropy: row.type_histogram_entropy,
            dominant_type_fraction: row.dominant_type_fraction,
            type_histogram_jsd: row.type_histogram_jsd,
            active_persistent_structures: row.active_persistent_structures,
            static_repeated_motifs: row.static_repeated_motifs,
            static_persistent_motifs: row.static_persistent_motifs,
            archive_size: row.archive_size,
            archive_evictions: row.archive_evictions,
            new_signatures: row.new_signatures,
            archive_capacity: row.archive_capacity,
            interestingness_score: row.interestingness_score,
            unmeasured_reasons: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct EreFamilyMaintenance {
    pub family_index: usize,
    pub anchor: String,
    pub checks: u64,
    pub valid_steps: u64,
    pub final_valid: bool,
    pub first_invalid_step: Option<u64>,
    pub invalid_transitions: u64,
    pub recovery_transitions: u64,
    pub final_failure: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EreMetaOperationTrace {
    pub step: u64,
    pub operation: String,
    pub instruction: String,
    pub target_family: String,
    pub fuel_before: Option<f64>,
    pub fuel_after: Option<f64>,
    pub family_valid_before: bool,
    pub family_valid_after: bool,
    pub executable_before: Option<String>,
    pub executable_after: Option<String>,
    pub proposal_before: Option<String>,
    pub proposal_after: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EreInstructionState {
    pub instruction: String,
    pub checks: u64,
    pub valid_steps: u64,
    pub first_invalid_step: Option<u64>,
    pub valid: bool,
    pub operation: Option<String>,
    pub fuel: Option<f64>,
}

/// Read-only, opt-in completed-step diagnostic; not an ordinary metrics field.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EreFamilyStepState {
    pub anchor: String,
    pub valid: bool,
    pub active: Option<bool>,
    pub execution_bytes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EreStepObservation {
    pub step: u64,
    pub world_hash_before: String,
    pub world_hash_after: String,
    pub world_nodes: usize,
    pub world_edges: usize,
    /// Full candidate bytes in installed rule-ID order; null means decode failed.
    pub world_rule_table: Vec<Option<String>>,
    pub rule_table_revision: u64,
    /// Hex strings preserve all 64 bits in JSON/JQ consumers.
    pub rng_before_compile: String,
    pub rng_after_compile: String,
    pub rng_after_step: String,
    pub match_work: usize,
    pub match_results: usize,
    pub match_ceiling_binding: bool,
    pub families: Vec<EreFamilyStepState>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct EreMetricsRow {
    pub step_start: u64,
    pub step_end: u64,
    pub rule_region_nodes: Option<usize>,
    pub rule_region_edges: Option<usize>,
    pub rules_installed: Option<usize>,
    pub rules_retired: Option<usize>,
    pub rules_quarantined: Option<usize>,
    pub quarantine_by_validator: Option<[usize; 9]>,
    pub rule_rcf_alive: Option<usize>,
    pub rule_rcf_novel_cumulative: Option<usize>,
    pub rule_rcf_novel_rate: Option<f64>,
    pub rule_survival_half_life: Option<f64>,
    pub rule_reuse_count_by_class: Option<BTreeMap<u64, usize>>,
    pub nontrivial_rule_reuse_count_by_class: Option<BTreeMap<u64, usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_rule_reuse_count_by_class: Option<BTreeMap<u64, usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structural_proposal_reuse_count_by_class: Option<BTreeMap<u64, usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambiguous_role_reuse_count_by_class: Option<BTreeMap<u64, usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_activation_count: Option<usize>,
    /// Cumulative post-step checks of original seeded anchors, not clones.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeded_family_maintenance: Option<Vec<EreFamilyMaintenance>>,
    /// Per-window events from the explicit static editor rule-ID suffix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeded_editor_rewrite_events: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinary_meta_rewrite_events: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoded_meta_vocabulary_protocol: Option<String>,
    /// Subset of non-static-editor (ordinary_meta) events, not an extra summand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoded_meta_rewrite_events: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoded_meta_operation_traces: Option<Vec<EreMetaOperationTrace>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoded_meta_instruction_states: Option<Vec<EreInstructionState>>,
    pub novel_nontrivial_classes: Option<usize>,
    pub survival_half_life_right_censored: bool,
    pub meta_rewrite_events: Option<usize>,
    pub meta_rewrite_site_entropy: Option<f64>,
    pub meta_edges_lost_to_cleanup: Option<usize>,
    pub rule_replicator_kind: Option<RuleReplicatorKind>,
    pub epoch_len_effective: Option<u64>,
    pub tpm_rule_set_stable: Option<bool>,
    pub rule_region_ceiling_binding: bool,
    pub match_ceiling_binding: bool,
    pub quarantine_capture_alarm: bool,
    pub anchor_survival_fraction: Option<f64>,
    /// Zero is a real measurement; None means no stationary TPM yet.
    pub meta_excluded: Option<Box<EreSubstrateMetrics>>,
    pub meta_included: Option<Box<EreSubstrateMetrics>>,
    pub ei_unmeasured_reason: Option<String>,
    pub ei_micro_ci_width_meta_included: Option<f64>,
    pub ei_micro_ci_width_meta_excluded: Option<f64>,
    pub rule_change_rate: Option<f64>,
    pub ei_ci_unmeasured_reason: Option<String>,
}
