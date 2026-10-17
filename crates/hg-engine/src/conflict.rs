//! Conflict detection and resolution (v4 §8.2, docs/rewrite_contract.md).

use hg_core::{EdgeId, HypergraphStore, NodeId};
use hg_observe::Binding;
use hg_rules::CompiledRule;
use std::collections::HashSet;

/// A binding's full conflict set (v4 §8.2):
/// bound nodes ∪ matched edges ∪ edges reachable via dangling-edge cleanup
/// from this binding's deleted nodes.
#[derive(Debug, Default)]
pub struct ConflictSet {
    pub nodes: HashSet<NodeId>,
    pub edges: HashSet<EdgeId>,
}

impl ConflictSet {
    pub fn is_disjoint(&self, other: &ConflictSet) -> bool {
        self.nodes.is_disjoint(&other.nodes) && self.edges.is_disjoint(&other.edges)
    }
}

/// Compute the full conflict set of a binding — the documented, testable
/// definition. The fast path in `resolve_conflicts` checks only bound-node
/// overlap, which is provably sufficient under the current rule system;
/// this function exists so the invariant is re-checkable if the rule
/// system's constraints ever change.
pub fn conflict_set(store: &HypergraphStore, rule: &CompiledRule, binding: &Binding) -> ConflictSet {
    let mut cs = ConflictSet::default();
    cs.nodes.extend(binding.bound_nodes.iter().copied());
    cs.edges.extend(binding.matched_edges.iter().copied());
    // Dangling-edge cleanup: every edge incident to a deleted bound node.
    for &lhs_ix in &rule.template.lhs_deleted {
        if let Some(&node) = binding.bound_nodes.get(lhs_ix) {
            for &edge_id in store.edges_incident_to(node) {
                cs.edges.insert(edge_id);
            }
        }
    }
    cs
}

/// Greedy deterministic conflict resolution over the canonical order:
/// `(Reverse(priority), rule_id, sorted bound nodes)` — higher priority
/// first (v4 §5). Accepts a binding iff none of its bound nodes were
/// claimed by an earlier accepted binding. Creation bindings (empty bound
/// set) can never conflict and are always accepted.
pub fn resolve_conflicts(bindings: &[Binding]) -> Vec<Binding> {
    let mut sorted: Vec<Binding> = bindings.to_vec();
    sorted.sort_by_key(|b| b.canonical_key());

    let mut accepted = Vec::new();
    let mut used_nodes: HashSet<NodeId> = HashSet::new();

    for binding in sorted {
        let conflicts = binding
            .bound_nodes
            .iter()
            .any(|n| used_nodes.contains(n));
        if !conflicts {
            used_nodes.extend(binding.bound_nodes.iter().copied());
            accepted.push(binding);
        }
    }
    accepted
}
