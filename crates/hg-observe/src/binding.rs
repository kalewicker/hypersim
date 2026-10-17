//! `Binding` — a candidate rule application (v4 §5.3, §8.2).
//!
//! Bound nodes are indexed by LHS variable position (the compiled pattern
//! maps variable names to indices), matched edges by LHS edge index. A
//! creation rule contributes a trivial binding with zero bound nodes:
//! `Binding::new(rule_id, priority, SmallVec::new())`.

use hg_core::{EdgeId, NodeId};
use smallvec::SmallVec;

#[derive(Clone, Debug)]
pub struct Binding {
    pub rule_id: usize,
    /// Rule priority, carried so conflict resolution can sort by
    /// `Reverse(priority)` without a rules-table lookup (v4 §5).
    pub priority: i32,
    /// NodeId bound to each LHS variable, by variable index.
    pub bound_nodes: SmallVec<[NodeId; 8]>,
    /// EdgeId matched by each LHS edge, by edge index.
    pub matched_edges: SmallVec<[EdgeId; 8]>,
}

impl Binding {
    pub fn new(rule_id: usize, priority: i32, bound_nodes: SmallVec<[NodeId; 8]>) -> Self {
        Self {
            rule_id,
            priority,
            bound_nodes,
            matched_edges: SmallVec::new(),
        }
    }

    pub fn with_edges(
        rule_id: usize,
        priority: i32,
        bound_nodes: SmallVec<[NodeId; 8]>,
        matched_edges: SmallVec<[EdgeId; 8]>,
    ) -> Self {
        Self {
            rule_id,
            priority,
            bound_nodes,
            matched_edges,
        }
    }

    /// True for schedule-driven creation-rule bindings (empty pattern).
    pub fn is_creation(&self) -> bool {
        self.bound_nodes.is_empty() && self.matched_edges.is_empty()
    }

    /// Sorted bound-node keys — the deterministic part of the canonical
    /// conflict-resolution sort key (see docs/reproducibility_policy.md).
    pub fn node_ids_sorted(&self) -> SmallVec<[u64; 8]> {
        use slotmap::Key;
        let mut ids: SmallVec<[u64; 8]> = self
            .bound_nodes
            .iter()
            .map(|id| id.data().as_ffi())
            .collect();
        ids.sort_unstable();
        ids
    }

    /// Canonical key: `(Reverse(priority), rule_id, sorted bound nodes)`.
    /// Sorting bindings by this key makes conflict resolution deterministic
    /// regardless of collection (thread) order, with higher priority first.
    pub fn canonical_key(&self) -> (std::cmp::Reverse<i32>, usize, SmallVec<[u64; 8]>) {
        (
            std::cmp::Reverse(self.priority),
            self.rule_id,
            self.node_ids_sorted(),
        )
    }
}
