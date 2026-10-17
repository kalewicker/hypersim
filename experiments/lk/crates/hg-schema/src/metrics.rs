//! `MetricsRow` — the CSV row shape (one row per evaluation window).
//!
//! `Option` fields serialize to empty/`NaN` when absent; see
//! `docs/metric_formulas.md` for which fields are [computed] vs [reserved].

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetricsRow {
    pub window_id: u64,
    pub step_start: u64,
    pub step_end: u64,
    pub n_nodes: usize,
    pub n_edges: usize,
    pub accepted_rewrites: usize,
    pub attempted_matches: usize,
    pub accepted_per_step: f64,
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
    /// Max CE across enabled coarse-graining strategies this interval.
    pub ce_best: Option<f64>,
    pub ce_best_strategy: Option<String>,
    /// M(t): WL-class registry size, populated at EI emission cadence.
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
    pub gate_fixture: Option<String>,
    pub gate_white_noise: Option<String>,
    pub gate_pi: Option<String>,
    pub gate_generative: Option<String>,
    pub gate_treadmill: Option<String>,
    pub gate_mobile: Option<String>,
    pub gate_controller: Option<String>,
    pub gate_completeness: Option<String>,
    pub science_unlocked: Option<bool>,
}
