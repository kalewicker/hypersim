//! S5.3 — `EcologyMetrics`: the per-window population quantities Experiment 5's
//! exit criteria are read from (scope §S5.3). Computed from a snapshot (live
//! population + trait distribution) plus event-derived counters the observer
//! accumulates (births/deaths/origins).

use crate::genome::GENOME_TRAIT_KEYS;
use crate::motif::CARRIER_LABEL;
use hg_core::{GraphSnapshot, PropertyValue};
use serde::{Deserialize, Serialize};

/// One window's ecology snapshot. Trait statistics are over the LIVE carrier
/// population; `NaN` when there are no carriers (a diagnosable absence, not 0).
#[derive(Clone, Debug, PartialEq)]
pub struct EcologyWindow {
    pub step: u64,
    pub carriers: usize,
    pub substrate_nodes: usize,
    /// carriers / substrate_nodes (0.283 is a comparison datum, not a threshold).
    pub carrier_ratio: f64,
    pub births_this_window: u64,
    pub deaths_this_window: u64,
    /// Cumulative counts by origin, for "spontaneous replication" evidence.
    pub bootstrap_origin_total: u64,
    pub replication_origin_total: u64,
    /// Per-trait mean and variance over live carriers, in `GENOME_TRAIT_KEYS`
    /// order (harvest_rate, replication_threshold, maintenance_cost, satiation).
    pub trait_mean: [f64; 4],
    pub trait_var: [f64; 4],
    /// Lineage-derived metrics at this window (filled by the observer, which
    /// holds the tree). `non_trivial_replicators` is the criterion-6 quantity.
    pub max_generation: u64,
    pub clade_count: usize,
    pub non_trivial_replicators: usize,
    pub total_deaths: u64,
}

/// Flat, CSV-serialisable form of [`EcologyWindow`] — one row of `ecology.csv`.
/// A SEPARATE file from `metrics.csv` (like `leakage.csv`), created only for
/// ALife runs, so the main metrics CSV stays byte-identical without the feature.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct EcologyRow {
    pub window_id: u64,
    pub step_end: u64,
    pub carriers: usize,
    pub substrate_nodes: usize,
    pub carrier_ratio: f64,
    pub births: u64,
    pub deaths: u64,
    pub bootstrap_total: u64,
    pub replication_total: u64,
    pub max_generation: u64,
    pub clade_count: usize,
    pub non_trivial_replicators: usize,
    pub cumulative_deaths: u64,
    pub harvest_rate_mean: f64,
    pub harvest_rate_var: f64,
    pub repl_threshold_mean: f64,
    pub repl_threshold_var: f64,
    pub maint_cost_mean: f64,
    pub maint_cost_var: f64,
    pub satiation_mean: f64,
    pub satiation_var: f64,
}

impl EcologyWindow {
    /// Flatten to a CSV row. `window_id` is the run-relative window index.
    pub fn to_row(&self, window_id: u64) -> EcologyRow {
        EcologyRow {
            window_id,
            step_end: self.step,
            carriers: self.carriers,
            substrate_nodes: self.substrate_nodes,
            carrier_ratio: self.carrier_ratio,
            births: self.births_this_window,
            deaths: self.deaths_this_window,
            bootstrap_total: self.bootstrap_origin_total,
            replication_total: self.replication_origin_total,
            max_generation: self.max_generation,
            clade_count: self.clade_count,
            non_trivial_replicators: self.non_trivial_replicators,
            cumulative_deaths: self.total_deaths,
            harvest_rate_mean: self.trait_mean[0],
            harvest_rate_var: self.trait_var[0],
            repl_threshold_mean: self.trait_mean[1],
            repl_threshold_var: self.trait_var[1],
            maint_cost_mean: self.trait_mean[2],
            maint_cost_var: self.trait_var[2],
            satiation_mean: self.trait_mean[3],
            satiation_var: self.trait_var[3],
        }
    }
}

/// Mean and (population) variance of one trait key over the live carriers.
fn trait_mean_var(snap: &GraphSnapshot<'_>, key: u32) -> (f64, f64) {
    let vals: Vec<f64> = snap
        .nodes()
        .filter(|(_, d)| d.label == CARRIER_LABEL)
        .filter_map(|(_, d)| match d.get_property(key) {
            Some(PropertyValue::Float(v)) => Some(v),
            _ => None,
        })
        .collect();
    if vals.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let n = vals.len() as f64;
    let mean = vals.iter().sum::<f64>() / n;
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    (mean, var)
}

/// Count live carriers and substrate (non-carrier) nodes in a snapshot.
pub fn population(snap: &GraphSnapshot<'_>) -> (usize, usize) {
    let mut carriers = 0;
    let mut substrate = 0;
    for (_, d) in snap.nodes() {
        if d.label == CARRIER_LABEL {
            carriers += 1;
        } else {
            substrate += 1;
        }
    }
    (carriers, substrate)
}

impl EcologyWindow {
    /// Build a window record from a snapshot and the observer's accumulated
    /// counters (window-scoped births/deaths, cumulative origin totals).
    #[allow(clippy::too_many_arguments)]
    pub fn from_snapshot(
        snap: &GraphSnapshot<'_>,
        step: u64,
        births_this_window: u64,
        deaths_this_window: u64,
        bootstrap_origin_total: u64,
        replication_origin_total: u64,
    ) -> Self {
        let (carriers, substrate_nodes) = population(snap);
        let carrier_ratio = if substrate_nodes == 0 {
            0.0
        } else {
            carriers as f64 / substrate_nodes as f64
        };
        let mut trait_mean = [0.0; 4];
        let mut trait_var = [0.0; 4];
        for (i, &k) in GENOME_TRAIT_KEYS.iter().enumerate() {
            let (m, v) = trait_mean_var(snap, k);
            trait_mean[i] = m;
            trait_var[i] = v;
        }
        Self {
            step,
            carriers,
            substrate_nodes,
            carrier_ratio,
            births_this_window,
            deaths_this_window,
            bootstrap_origin_total,
            replication_origin_total,
            trait_mean,
            trait_var,
            // Lineage fields are filled by the observer (which holds the tree).
            max_generation: 0,
            clade_count: 0,
            non_trivial_replicators: 0,
            total_deaths: 0,
        }
    }
}
