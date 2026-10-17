//! Per-step statistics handed to observers.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StepStats {
    pub step: u64,
    pub attempted_matches: usize,
    pub accepted_rewrites: usize,
    pub nodes_created: usize,
    pub nodes_deleted: usize,
    pub edges_created: usize,
    pub edges_deleted: usize,
    /// Property effects executed this step (apply step 5.5).
    #[serde(default)]
    pub property_effects_applied: u64,
    /// Effects skipped on runtime mismatch (missing key / wrong variant).
    /// Nonzero here is visible without log spelunking.
    #[serde(default)]
    pub property_effects_skipped: u64,
}

impl StepStats {
    pub fn new() -> Self {
        Self::default()
    }
}
