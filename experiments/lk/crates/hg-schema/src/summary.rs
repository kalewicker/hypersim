//! `RunSummary` — the report content shape (v4 §11 report is rendered from
//! this). Uses `MetricsRow` directly; provenance is a plain data mirror of
//! `hg-output`'s `RunManifest` so this crate stays serde-only with no
//! dependency on `hg-output` (§2.2).

use crate::gate::{GateName, GateStatus};
use crate::metrics::MetricsRow;
use crate::regime::Regime;
use serde::{Deserialize, Serialize};

/// Provenance bookkeeping mirrored into the report's Manifest section.
/// `hg-sim-core` fills both this and `hg-output::RunManifest` from one config.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RunProvenance {
    pub run_id: String,
    pub experiment_name: String,
    pub seed: u64,
    pub thread_count: usize,
    pub step_mode: String,
    pub hypersim_version: String,
    pub state: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub max_steps: u64,
    pub final_step: u64,
    pub config_hash: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RunSummary {
    pub provenance: RunProvenance,
    /// Rendered verbatim into the Run Parameters section (already-formatted
    /// config lines) so the schema crate needs no knowledge of `SimConfig`.
    pub run_parameters: Vec<(String, String)>,
    pub final_metrics: Option<MetricsRow>,
    pub regime_summary: Vec<(Regime, u64)>,
    pub gate_statuses: Vec<(GateName, GateStatus)>,
    pub fixture_truth_table: Vec<(String, String, bool)>,
    pub science_unlocked: bool,
    pub science_locked_reason: Option<String>,
    pub evaluator_details: Vec<(String, String)>,
    pub causal_summary: Vec<(String, String)>,
    pub constructor_ecology: Vec<(String, String)>,
    pub suspicious_flags: Vec<String>,
    pub observations: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub campaign_summary: Option<String>,
    /// Locality & leakage rows emitted this run (LK5; empty = disabled).
    #[serde(default)]
    pub leakage_rows: Vec<crate::leakage::LeakageRow>,
    /// Pre-formatted leakage summary lines (closure defect, cone/twin
    /// pointers), keeping this crate serde-only.
    #[serde(default)]
    pub leakage_summary: Vec<(String, String)>,
}

impl RunSummary {
    pub fn new() -> Self {
        Self::default()
    }
}
