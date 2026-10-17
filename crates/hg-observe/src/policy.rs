//! `RuleSelectionPolicy` — the hypergraph-neural-net / ALife seam (v4 §6.1).
//!
//! The seam sits between candidate-collection and conflict-resolution: a
//! policy returns indices into the candidate slice, and the engine's
//! conflict resolver still runs on the selection, so a policy can pick a
//! suboptimal or overlapping candidate set but can never violate the
//! rewrite contract's node-disjointness guarantee. Implementors need only
//! `hg-observe` and `hg-core`, never `hg-engine`.

use crate::binding::Binding;
use hg_core::GraphSnapshot;

pub trait RuleSelectionPolicy: Send + 'static {
    /// Return the indices (into `candidates`) to forward to conflict
    /// resolution.
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize>;
}

/// Default policy: forward every candidate.
pub struct SelectAllPolicy;

impl RuleSelectionPolicy for SelectAllPolicy {
    fn select(&mut self, _snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        (0..candidates.len()).collect()
    }
}
