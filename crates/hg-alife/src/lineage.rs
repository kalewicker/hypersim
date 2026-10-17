//! S5.2 — `LineageTree`: the carrier genealogy, reconstructed from the rewrite
//! event stream alone (scope §S5.2). A birth is a `replicate` event (parent =
//! the carrier-labelled bound node; child = the created carrier); a root is a
//! `bootstrap_seed` carrier (no parent); a death is a `die_and_recycle` event's
//! deleted carrier. Carriers are keyed by `NodeId`; a stable sequential display
//! id is assigned in birth order for Newick output (slotmap keys don't render
//! cleanly).
//!
//! This type is snapshot-agnostic and engine-free: it takes `NodeId`s, a
//! `Genome`, and step numbers, so it unit-tests without an engine. The observer
//! (`observer.rs`) is what reads those out of events + snapshot.

use crate::genome::Genome;
use ahash::AHashMap;
use hg_core::NodeId;

/// A carrier genealogy built incrementally from births and deaths.
#[derive(Default, Debug, Clone)]
pub struct LineageTree {
    /// child carrier -> parent carrier (roots absent).
    parent: AHashMap<NodeId, NodeId>,
    /// parent carrier -> children, in birth order.
    children: AHashMap<NodeId, Vec<NodeId>>,
    /// Traits at birth (read from the snapshot when the birth was recorded).
    genome_at: AHashMap<NodeId, Genome>,
    birth_step: AHashMap<NodeId, u64>,
    /// Death step, present only for carriers observed to die.
    death_step: AHashMap<NodeId, u64>,
    /// Carriers with no parent — i.e. `bootstrap_seed` founders.
    roots: Vec<NodeId>,
    /// Every carrier in birth/record order (stable display-id order).
    order: Vec<NodeId>,
    /// NodeId -> sequential display id (1-based), assigned in `order`.
    display_id: AHashMap<NodeId, u64>,
}

impl LineageTree {
    pub fn new() -> Self {
        Self::default()
    }

    fn register(&mut self, c: NodeId) {
        if !self.display_id.contains_key(&c) {
            self.order.push(c);
            self.display_id.insert(c, self.order.len() as u64);
        }
    }

    /// Record a carrier birth. `parent = None` ⟹ a bootstrap founder (a root).
    /// Idempotent-safe against duplicate ids (a NodeId is never reused live).
    pub fn record_birth(&mut self, child: NodeId, parent: Option<NodeId>, genome: Genome, step: u64) {
        self.register(child);
        self.genome_at.insert(child, genome);
        self.birth_step.insert(child, step);
        match parent {
            Some(p) => {
                self.register(p);
                self.parent.insert(child, p);
                self.children.entry(p).or_default().push(child);
            }
            None => self.roots.push(child),
        }
    }

    /// Record a carrier death.
    pub fn record_death(&mut self, carrier: NodeId, step: u64) {
        self.death_step.insert(carrier, step);
    }

    // --- queries ---

    pub fn total_carriers(&self) -> usize {
        self.order.len()
    }

    pub fn total_births(&self) -> usize {
        // Every non-root recorded carrier is a birth via replication.
        self.parent.len()
    }

    pub fn total_deaths(&self) -> usize {
        self.death_step.len()
    }

    pub fn is_alive(&self, c: NodeId) -> bool {
        self.display_id.contains_key(&c) && !self.death_step.contains_key(&c)
    }

    /// The set of carriers recorded as born and not yet recorded as dead —
    /// i.e. the live population reconstructed FROM EVENTS ALONE. Compared against
    /// the graph's actual carrier-labelled nodes, this is the ground-truth check
    /// that births/deaths are attributed to the right node ids (S5 gate).
    pub fn live_carriers(&self) -> std::collections::HashSet<NodeId> {
        self.order
            .iter()
            .copied()
            .filter(|c| !self.death_step.contains_key(c))
            .collect()
    }

    pub fn genome_at_birth(&self, c: NodeId) -> Option<&Genome> {
        self.genome_at.get(&c)
    }

    pub fn parent_of(&self, c: NodeId) -> Option<NodeId> {
        self.parent.get(&c).copied()
    }

    pub fn children_of(&self, c: NodeId) -> &[NodeId] {
        self.children.get(&c).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Number of distinct founding ancestors (bootstrap origins). With the
    /// current single-marker seeding this is 1; kept general for multi-seed runs.
    /// NOTE: Experiment 5 criterion 4 ("clade_count > 1") is about *sub-lineage*
    /// diversity under one ancestor, not this — see [`surviving_subclades`].
    pub fn clade_count(&self) -> usize {
        self.roots.len()
    }

    /// Generation depth of a carrier: 0 for a root, parent's depth + 1 otherwise.
    pub fn generation(&self, c: NodeId) -> u64 {
        let mut g = 0;
        let mut cur = c;
        while let Some(&p) = self.parent.get(&cur) {
            g += 1;
            cur = p;
        }
        g
    }

    /// The deepest generation reached by any recorded carrier.
    pub fn max_generation(&self) -> u64 {
        self.order.iter().map(|&c| self.generation(c)).max().unwrap_or(0)
    }

    /// A sub-lineage diversity measure suited to a single-ancestor run: the
    /// number of the founder's *immediate children* whose sub-tree still has at
    /// least one living carrier. This is the honest "co-existing lineages"
    /// count Experiment-5 criterion 4 wants when there is one bootstrap root.
    pub fn surviving_subclades(&self) -> usize {
        let mut count = 0;
        for &root in &self.roots {
            for &child in self.children_of(root) {
                if self.subtree_has_living(child) {
                    count += 1;
                }
            }
            // A living root with no children is itself a (degenerate) subclade.
        }
        count
    }

    fn subtree_has_living(&self, root: NodeId) -> bool {
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if self.is_alive(n) {
                return true;
            }
            stack.extend_from_slice(self.children_of(n));
        }
        false
    }

    /// clade survival curve: for each generation g (0..=max), how many recorded
    /// carriers reached generation g. A lineage "persisting > K generations"
    /// (criterion 4) means `survival_curve()[K+1] > 0`.
    pub fn survival_curve(&self) -> Vec<usize> {
        let maxg = self.max_generation();
        let mut curve = vec![0usize; (maxg + 1) as usize];
        for &c in &self.order {
            curve[self.generation(c) as usize] += 1;
        }
        curve
    }

    /// `non_trivial_replicator_count` (scope §S5 / criterion 6): a bounded,
    /// persistent individual that produced a copy of its whole self. Operationally:
    /// a carrier that produced >= 1 offspring (is a parent) AND whose own lineage
    /// reached >= `min_generations` depth below it (so it is *sustained* self-
    /// copying, not a single stochastic birth). On a bare substrate this is 0 (no
    /// carriers, no `replicate` events), which is the Experiment-6 baseline.
    pub fn non_trivial_replicator_count(&self, min_generations: u64) -> usize {
        self.children
            .keys()
            .filter(|&&p| self.deepest_descendant_depth(p) >= min_generations)
            .count()
    }

    /// Generations of descendants below `c` (0 if it never reproduced).
    ///
    /// The lineage is a tree by construction — every child has exactly one parent,
    /// recorded once, and `NodeId`s never alias within a run (S5 soundness proved
    /// SlotMap versioned keys are distinct even under heavy churn), so no cycle can
    /// form. The `visited` guard is defensive belt-and-braces: on malformed input
    /// (e.g. a hand-built tree with an accidental cycle) it keeps this DFS bounded
    /// and terminating instead of looping forever and exhausting memory.
    fn deepest_descendant_depth(&self, c: NodeId) -> u64 {
        let mut best = 0;
        let mut visited = ahash::AHashSet::new();
        let mut stack = vec![(c, 0u64)];
        while let Some((n, d)) = stack.pop() {
            if !visited.insert(n) {
                continue; // already expanded n — skip (breaks any cycle)
            }
            best = best.max(d);
            for &child in self.children_of(n) {
                stack.push((child, d + 1));
            }
        }
        best
    }

    /// Export the genealogy as a Newick forest string. Multiple roots are wrapped
    /// under an unnamed implicit root so the result is a single parseable tree
    /// (`(rootA,rootB);` when there are several). Leaf/interior labels are the
    /// stable sequential display ids.
    pub fn export_newick(&self) -> String {
        if self.roots.is_empty() {
            return ";".to_string();
        }
        let mut parts = Vec::new();
        for &r in &self.roots {
            parts.push(self.newick_subtree(r));
        }
        if parts.len() == 1 {
            format!("{};", parts[0])
        } else {
            format!("({});", parts.join(","))
        }
    }

    fn newick_subtree(&self, node: NodeId) -> String {
        let id = self.display_id.get(&node).copied().unwrap_or(0);
        let kids = self.children_of(node);
        if kids.is_empty() {
            format!("{id}")
        } else {
            let inner: Vec<String> = kids.iter().map(|&k| self.newick_subtree(k)).collect();
            format!("({}){id}", inner.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hg_core::{HypergraphStore, Label, NodeData};

    fn g() -> Genome {
        Genome { harvest_rate: Some(1.0), replication_threshold: Some(3.0), maintenance_cost: Some(1.0), satiation: Some(20.0) }
    }

    /// Malformed input: a cyclic parent/child map (impossible in a real run — the
    /// lineage is a tree, NodeIds never alias) must still TERMINATE. Without the
    /// visited guard the DFS in `deepest_descendant_depth` loops forever and OOMs.
    #[test]
    fn deepest_descendant_depth_terminates_on_a_cycle() {
        let mut store = HypergraphStore::new();
        let a = store.add_node(NodeData::new(Label(100), 0));
        let b = store.add_node(NodeData::new(Label(100), 0));

        let mut t = LineageTree::new();
        t.record_birth(a, None, g(), 1); // a is a root
        t.record_birth(b, Some(a), g(), 2); // a -> b
        t.record_birth(a, Some(b), g(), 3); // b -> a  ⟹ cycle a -> b -> a

        // The guard bounds the walk: a and b are each expanded once, so the deepest
        // reachable depth is finite (2: a->b->a). The point is that it RETURNS.
        assert!(t.deepest_descendant_depth(a) <= 2, "cyclic walk must be bounded, not infinite");
        // And the public entry point terminates too (this is what would hang).
        let _ = t.non_trivial_replicator_count(2);
    }
}
