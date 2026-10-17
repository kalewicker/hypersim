//! `SubstrateRow` — one row of `substrate.csv`, the substrate-ONLY metric
//! emission for ALife runs (hg-alife S0.2, scope §9 Q3).
//!
//! Carriers are hypergraph nodes, so on the full graph they inflate `n_nodes`,
//! `d_eff`, and the regime classifier — a carrier bloom reads as the *substrate*
//! `Exploding`. These columns recompute the structural metrics + regime over the
//! substrate subgraph only (carriers and their occupancy edges removed), so
//! substrate physics stays attributable to substrate physics. Emitted as a
//! SEPARATE file (like `leakage.csv`/`ecology.csv`) so `metrics.csv` is
//! byte-identical without the `alife` feature/flag.
//!
//! `full_*` columns carry the SAME-window full-graph values so the pollution is
//! visible at a glance and the paired bare-substrate control is checkable.

use crate::Regime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SubstrateRow {
    pub window_id: u64,
    pub step_end: u64,
    /// Substrate subgraph (carrier nodes + occupancy edges removed).
    pub substrate_n_nodes: usize,
    pub substrate_n_edges: usize,
    pub substrate_mean_degree: Option<f64>,
    pub substrate_d_eff: Option<f64>,
    pub substrate_regime: String,
    /// Full-graph values this same window, for pollution comparison.
    pub full_n_nodes: usize,
    pub full_regime: String,
}

impl SubstrateRow {
    pub fn regime_str(r: Regime) -> String {
        format!("{r:?}")
    }
}
