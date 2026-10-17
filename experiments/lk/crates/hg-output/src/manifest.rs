//! Run manifest and run-state machine (docs/run_state_machine.md).
//!
//! Every transition is durably committed to `manifest.json` — the
//! authoritative, disk-persisted record of run status, independent of
//! whether any live event consumer exists (v4 §9.1).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RunState {
    #[default]
    Created,
    Running,
    Paused,
    Interrupted,
    Completed,
}

impl RunState {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunState::Created => "Created",
            RunState::Running => "Running",
            RunState::Paused => "Paused",
            RunState::Interrupted => "Interrupted",
            RunState::Completed => "Completed",
        }
    }

    /// Small integer discriminant for atomic storage (v4 §9.2).
    pub fn as_u8(&self) -> u8 {
        match self {
            RunState::Created => 0,
            RunState::Running => 1,
            RunState::Paused => 2,
            RunState::Interrupted => 3,
            RunState::Completed => 4,
        }
    }

    pub fn from_u8(v: u8) -> RunState {
        match v {
            1 => RunState::Running,
            2 => RunState::Paused,
            3 => RunState::Interrupted,
            4 => RunState::Completed,
            _ => RunState::Created,
        }
    }
}

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("invalid run-state transition: {0:?} -> {1:?}")]
    InvalidTransition(RunState, RunState),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RunManifest {
    pub run_id: String,
    pub experiment_name: String,
    pub seed: u64,
    pub thread_count: usize,
    pub step_mode: String,
    pub hypersim_version: String,
    pub rustc_version: String,
    pub target: String,
    pub build_features: Vec<String>,
    pub state: RunState,
    pub start_time: String,
    pub end_time: Option<String>,
    pub max_steps: u64,
    pub final_step: u64,
    pub config_hash: String,
    /// v3 §1.1 provenance: which fixture_calibration.md version the
    /// phase-classifier thresholds cite. Present iff leakage is enabled.
    #[serde(default)]
    pub classifier_calibration: Option<String>,
    /// v4 §1.2: the LK priority regime this run executed under
    /// ("subdivide_first" | "transport_first"). Present iff lk_rules on.
    #[serde(default)]
    pub priority: Option<String>,
    /// v4 §1.3 derived flag: q ≤ 2τ — every fresh child is born inside
    /// the transport dead zone. Findings about filament energetics must
    /// cite this. Present iff lk_rules on.
    #[serde(default)]
    pub children_born_in_dead_zone: Option<bool>,
    /// v5 Appendix B: fuse throttle in force ("null" when the fuse is
    /// unthrottled). Present iff the fuse is enabled.
    #[serde(default)]
    pub f_cap: Option<f64>,
    /// v5 §9.1: provenance of a relaxation IC — source path, the source
    /// manifest's identifying params, and the snapshot step loaded.
    /// Present iff `[relaxation] ic_source` is set.
    #[serde(default)]
    pub ic_provenance: Option<String>,
    /// v6 §4/§3 fuse configuration. Present iff the fuse is enabled.
    /// `gate` ∈ {none, f_cap, countdown, age}; `ledger_mode` ∈
    /// {vanish, transfer}.
    #[serde(default)]
    pub gate: Option<String>,
    #[serde(default)]
    pub countdown_k: Option<i64>,
    /// v8 Appendix B: the lifetime k actually in force under a lifetime
    /// gate, in whichever spelling the config used. Present iff
    /// `gate ∈ {countdown, age}` — so a ticked/passive comparison pair can
    /// be checked for k agreement from the manifests alone.
    #[serde(default)]
    pub lifetime_k: Option<i64>,
    /// v8 §3.1: which clock produced the lifetime — `"tick_rule"` (the
    /// v6 countdown, a scheduler actor) or `"birth_stamp"` (the passive
    /// age guard, no bindings claimed). Present iff a lifetime gate is in
    /// force. This is the field that distinguishes the two EXP-L18 arms.
    #[serde(default)]
    pub clock_mechanism: Option<String>,
    #[serde(default)]
    pub ledger_mode: Option<String>,
    /// v6 §1.2 dispersion precondition verdict for a countdown-gated run:
    /// the live countdown distribution's p10/p90 over the first windows,
    /// with a pass/fail. Present iff `gate = "countdown"`.
    #[serde(default)]
    pub dispersion_check: Option<String>,
    /// Directory this manifest persists into (not serialized).
    #[serde(skip)]
    dir: Option<PathBuf>,
}

impl RunManifest {
    /// Create the manifest and immediately persist it (state = Created).
    pub fn create(mut self, dir: &Path) -> Result<Self, ManifestError> {
        std::fs::create_dir_all(dir)?;
        self.dir = Some(dir.to_path_buf());
        self.state = RunState::Created;
        self.persist()?;
        Ok(self)
    }

    /// Load a manifest back from `dir/manifest.json` (post-hoc tools, GUI
    /// re-attach). The event stream is transient; this file is the record.
    pub fn load(dir: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(dir.join("manifest.json"))?;
        let mut m: RunManifest = serde_json::from_str(&content)?;
        m.dir = Some(dir.to_path_buf());
        Ok(m)
    }

    pub fn state(&self) -> RunState {
        self.state
    }

    fn transition(&mut self, allowed_from: &[RunState], to: RunState) -> Result<(), ManifestError> {
        if !allowed_from.contains(&self.state) {
            return Err(ManifestError::InvalidTransition(self.state, to));
        }
        self.state = to;
        self.persist()
    }

    pub fn start(&mut self) -> Result<(), ManifestError> {
        self.start_time = chrono::Utc::now().to_rfc3339();
        self.transition(&[RunState::Created], RunState::Running)
    }

    pub fn pause(&mut self) -> Result<(), ManifestError> {
        self.transition(&[RunState::Running], RunState::Paused)
    }

    pub fn resume(&mut self) -> Result<(), ManifestError> {
        self.transition(&[RunState::Paused], RunState::Running)
    }

    pub fn seal_completed(&mut self, final_step: u64) -> Result<(), ManifestError> {
        self.final_step = final_step;
        self.end_time = Some(chrono::Utc::now().to_rfc3339());
        self.transition(&[RunState::Running, RunState::Paused], RunState::Completed)
    }

    pub fn seal_interrupted(&mut self, final_step: u64) -> Result<(), ManifestError> {
        self.final_step = final_step;
        self.end_time = Some(chrono::Utc::now().to_rfc3339());
        self.transition(
            &[RunState::Created, RunState::Running, RunState::Paused],
            RunState::Interrupted,
        )
    }

    /// Write `manifest.json`. Called by every transition; may also be
    /// called directly after mutating bookkeeping fields.
    pub fn persist(&self) -> Result<(), ManifestError> {
        let Some(dir) = &self.dir else {
            return Ok(()); // not yet bound to a directory
        };
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join("manifest.json"), content)?;
        Ok(())
    }
}
