//! `RewriteEvent` — what one accepted rewrite did to the store.

use crate::binding::Binding;
use hg_core::{EdgeId, NodeId};

#[derive(Clone, Debug)]
pub struct RewriteEvent {
    pub rule_id: usize,
    pub rule_name: String,
    pub binding: Binding,
    pub nodes_created: Vec<NodeId>,
    pub nodes_deleted: Vec<NodeId>,
    pub edges_created: Vec<EdgeId>,
    pub edges_deleted: Vec<EdgeId>,
    /// Surviving endpoints of edges removed by DANGLING-EDGE CLEANUP
    /// (apply step 2), i.e. nodes whose neighborhood this rewrite changed
    /// through the engine's one documented locality-of-effect exception
    /// (v4 §8.1) rather than through the rule's own pattern. Causal
    /// analyses must attribute influence flowing through these nodes to a
    /// separate cleanup channel (LK plan §1.3).
    pub cleanup_neighbors: Vec<NodeId>,
    /// v4 §6.3 fused-energy ledger: sum of property-key-1 Float energy
    /// on nodes DELETED by this event, read at deletion time. 0.0 for
    /// events that delete no energy-carrying node. The audit invariant
    /// extends to E_total + c·N_sub + Σ vanished = E₀ (exact).
    pub vanished_energy: f64,
}
