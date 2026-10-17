//! S5.1 — `AlifeObserver` (`impl hg_observe::Observer`, read-only). Consumes the
//! rewrite event stream and maintains the [`LineageTree`] and per-window
//! [`EcologyWindow`]s. It never mutates the store (scope §5 observer contract) and
//! does not depend on `hg-engine`.
//!
//! Birth/parent identification (scope §S5.2 subtlety): a `replicate` event's
//! child is the freshly CREATED carrier (`nodes_created`), and the parent is the
//! bound carrier-labelled node (`binding.bound_nodes`) — the two are distinguished
//! by label against the post-step snapshot, so this does not hard-code LHS
//! variable indices. `replicate` and `bootstrap_seed` being DISTINCT rule ids is
//! what makes births filterable at all.

use crate::genome::Genome;
use crate::lineage::LineageTree;
use crate::metrics::EcologyWindow;
use crate::motif::CARRIER_LABEL;
use hg_core::{GraphSnapshot, NodeId};
use hg_observe::{Observer, RewriteEvent, StepStats};

/// Minimum descendant-generation depth for a carrier to count as a NonTrivial
/// (bounded, sustained self-copying) replicator in the per-window CSV metric.
/// A single stochastic birth (depth 1) does not qualify; depth ≥ 2 = sustained.
pub const NON_TRIVIAL_MIN_GENERATIONS: u64 = 2;

/// Read-only lineage + ecology observer for the ALife layer.
#[derive(Debug)]
pub struct AlifeObserver {
    lineage: LineageTree,
    step: u64,
    window_size: u64,
    windows: Vec<EcologyWindow>,
    // Window-scoped counters (reset each window) and cumulative origin totals.
    births_this_window: u64,
    deaths_this_window: u64,
    bootstrap_origin_total: u64,
    replication_origin_total: u64,
}

impl AlifeObserver {
    /// `window_size` = steps between ecology samples (must be >= 1).
    pub fn new(window_size: u64) -> Self {
        Self {
            lineage: LineageTree::new(),
            step: 0,
            window_size: window_size.max(1),
            windows: Vec::new(),
            births_this_window: 0,
            deaths_this_window: 0,
            bootstrap_origin_total: 0,
            replication_origin_total: 0,
        }
    }

    pub fn lineage(&self) -> &LineageTree {
        &self.lineage
    }
    pub fn windows(&self) -> &[EcologyWindow] {
        &self.windows
    }
    /// The recorded windows flattened to CSV rows (one per window, in order).
    pub fn ecology_rows(&self) -> Vec<crate::metrics::EcologyRow> {
        self.windows
            .iter()
            .enumerate()
            .map(|(i, w)| w.to_row(i as u64))
            .collect()
    }
    pub fn bootstrap_origin_total(&self) -> u64 {
        self.bootstrap_origin_total
    }
    pub fn replication_origin_total(&self) -> u64 {
        self.replication_origin_total
    }

    /// The created carrier in an event (the child), if any.
    fn created_carrier(snap: &GraphSnapshot<'_>, e: &RewriteEvent) -> Option<NodeId> {
        e.nodes_created
            .iter()
            .copied()
            .find(|&n| snap.get_node(n).map(|d| d.label == CARRIER_LABEL).unwrap_or(false))
    }

    /// The bound carrier-labelled node in an event (the parent), if any.
    fn bound_carrier(snap: &GraphSnapshot<'_>, e: &RewriteEvent) -> Option<NodeId> {
        e.binding
            .bound_nodes
            .iter()
            .copied()
            .find(|&n| snap.get_node(n).map(|d| d.label == CARRIER_LABEL).unwrap_or(false))
    }
}

impl Observer for AlifeObserver {
    fn on_step_end(&mut self, snap: &GraphSnapshot<'_>, _stats: &StepStats, events: &[RewriteEvent]) {
        self.step += 1;
        for e in events {
            match e.rule_name.as_str() {
                "bootstrap_seed" => {
                    if let Some(child) = Self::created_carrier(snap, e) {
                        let g = snap.get_node(child).map(Genome::from_node).unwrap_or(Genome {
                            harvest_rate: None,
                            replication_threshold: None,
                            maintenance_cost: None,
                            satiation: None,
                        });
                        self.lineage.record_birth(child, None, g, self.step);
                        self.bootstrap_origin_total += 1;
                    }
                }
                "replicate" => {
                    if let Some(child) = Self::created_carrier(snap, e) {
                        let parent = Self::bound_carrier(snap, e);
                        let g = snap.get_node(child).map(Genome::from_node).unwrap_or(Genome {
                            harvest_rate: None,
                            replication_threshold: None,
                            maintenance_cost: None,
                            satiation: None,
                        });
                        self.lineage.record_birth(child, parent, g, self.step);
                        self.replication_origin_total += 1;
                        self.births_this_window += 1;
                    }
                }
                "die_and_recycle" => {
                    // The deleted carrier is the dying individual.
                    for &n in &e.nodes_deleted {
                        // A dead carrier is gone from the snapshot; record any id
                        // we know as a carrier lineage member.
                        if self.lineage.genome_at_birth(n).is_some() {
                            self.lineage.record_death(n, self.step);
                            self.deaths_this_window += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        if self.step % self.window_size == 0 {
            let mut w = EcologyWindow::from_snapshot(
                snap,
                self.step,
                self.births_this_window,
                self.deaths_this_window,
                self.bootstrap_origin_total,
                self.replication_origin_total,
            );
            // Fill lineage-derived metrics from the tree the observer holds.
            w.max_generation = self.lineage.max_generation();
            w.clade_count = self.lineage.clade_count();
            w.non_trivial_replicators =
                self.lineage.non_trivial_replicator_count(NON_TRIVIAL_MIN_GENERATIONS);
            w.total_deaths = self.lineage.total_deaths() as u64;
            self.windows.push(w);
            self.births_this_window = 0;
            self.deaths_this_window = 0;
        }
    }
}
