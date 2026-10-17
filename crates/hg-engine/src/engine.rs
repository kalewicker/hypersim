//! The step loop: creation-rule firing, matching, policy seam, conflict
//! resolution, apply.

use crate::conflict::resolve_conflicts;
use crate::matcher::{find_bindings_counted, GuardStats};
use hg_core::{HyperEdge, HypergraphStore, NodeData, NodeId, PropertyValue};
use hg_observe::{Binding, ObserverChain, RewriteEvent, RuleSelectionPolicy, StepStats};
use hg_rules::{CompiledRule, MatchBudgetPolicy, StepMode};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use smallvec::SmallVec;

#[cfg(test)]
#[path = "pooled_tests.rs"]
mod pooled_tests;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuleMatchUsage {
    pub rule_id: usize,
    pub work: usize,
    pub results: usize,
    pub truncated: bool,
    pub work_allowance: Option<usize>,
    pub result_allowance: Option<usize>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MatchBudgetUsage {
    pub work: usize,
    pub results: usize,
    pub truncated: bool,
    pub rules: Vec<RuleMatchUsage>,
}
impl MatchBudgetUsage {
    fn record(&mut self, id: usize, g: &GuardStats, results: usize, rule: &CompiledRule) {
        self.work += g.enumeration_work;
        self.results += results;
        self.truncated |= g.enumeration_truncated;
        self.rules.push(RuleMatchUsage {
            rule_id: id,
            work: g.enumeration_work,
            results,
            truncated: g.enumeration_truncated,
            work_allowance: rule.pattern.match_work_limit,
            result_allowance: rule.pattern.match_result_limit,
        });
    }
}

/// Same-build diagnostic execution state. Observers are explicitly reattached
/// by the owner; policies and unsupported rule sources fail closed.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EngineCheckpoint {
    rules: Vec<CompiledRule>,
    step_mode: StepMode,
    rng: ChaCha8Rng,
    max_pairings_per_step: usize,
    stats: StepStats,
    guard_stats: Vec<GuardStats>,
    pub source: Option<hg_rules::RuleSourceCheckpoint>,
    compile_report: Option<hg_rules::RuleCompileReport>,
    rule_table_revision: u64,
    last_compile_step: Option<u64>,
    match_budget: Option<(usize, usize)>,
    match_budget_policy: MatchBudgetPolicy,
    match_budget_usage: MatchBudgetUsage,
    match_ceiling_binding: bool,
}

pub struct Engine {
    rules: Vec<CompiledRule>,
    step_mode: StepMode,
    rng: ChaCha8Rng,
    max_pairings_per_step: usize,
    stats: StepStats,
    observers: ObserverChain,
    policy: Option<Box<dyn RuleSelectionPolicy>>,
    /// v4 §1.1 guard-engagement counters, cumulative since the last
    /// [`Engine::drain_guard_stats`] call, indexed by rule id.
    guard_stats: Vec<GuardStats>,
    rule_source: Option<Box<dyn hg_rules::RuleSource>>,
    compile_report: Option<hg_rules::RuleCompileReport>,
    rule_table_revision: u64,
    last_compile_step: Option<u64>,
    match_budget: Option<(usize, usize)>,
    match_budget_policy: MatchBudgetPolicy,
    match_budget_usage: MatchBudgetUsage,
    match_ceiling_binding: bool,
}

impl Engine {
    pub fn diagnostic_checkpoint(&self) -> Result<EngineCheckpoint, String> {
        if self.policy.is_some() {
            return Err("checkpoint does not support selection policies".into());
        }
        Ok(EngineCheckpoint {
            rules: self.rules.clone(), step_mode: self.step_mode, rng: self.rng.clone(),
            max_pairings_per_step: self.max_pairings_per_step, stats: self.stats.clone(),
            guard_stats: self.guard_stats.clone(),
            source: self.rule_source.as_ref().map(|s|s.checkpoint()).transpose()?,
            compile_report: self.compile_report.clone(), rule_table_revision: self.rule_table_revision,
            last_compile_step: self.last_compile_step, match_budget: self.match_budget,
            match_budget_policy: self.match_budget_policy, match_budget_usage: self.match_budget_usage.clone(),
            match_ceiling_binding: self.match_ceiling_binding,
        })
    }
    /// Does not compile, seed, draw RNG, or install arbitrary observers.
    pub fn from_diagnostic_checkpoint(c: EngineCheckpoint,
        source: Option<Box<dyn hg_rules::RuleSource>>) -> Result<Self, String> {
        if c.source.is_some() != source.is_some() {
            return Err("checkpoint rule-source presence mismatch".into());
        }
        Ok(Self {rules:c.rules, step_mode:c.step_mode, rng:c.rng,
            max_pairings_per_step:c.max_pairings_per_step, stats:c.stats,
            observers:ObserverChain::new(), policy:None, guard_stats:c.guard_stats,
            rule_source:source, compile_report:c.compile_report, rule_table_revision:c.rule_table_revision,
            last_compile_step:c.last_compile_step, match_budget:c.match_budget,
            match_budget_policy:c.match_budget_policy, match_budget_usage:c.match_budget_usage,
            match_ceiling_binding:c.match_ceiling_binding})
    }
    /// Rules keep their given order: `rule_id` = index. Priority is honored
    /// by conflict resolution (`Reverse<i32>` in the sort key), not by
    /// reordering the rules table.
    pub fn new(
        rules: Vec<CompiledRule>,
        step_mode: StepMode,
        seed: u64,
        max_pairings_per_step: usize,
    ) -> Self {
        Self {
            rules,
            step_mode,
            rng: ChaCha8Rng::seed_from_u64(seed),
            max_pairings_per_step: max_pairings_per_step.max(1),
            stats: StepStats::new(),
            observers: ObserverChain::new(),
            policy: None,
            guard_stats: Vec::new(),
            rule_source: None,
            compile_report: None,
            rule_table_revision: 0,
            last_compile_step: None,
            match_budget: None,
            match_budget_policy: MatchBudgetPolicy::Split,
            match_budget_usage: MatchBudgetUsage::default(),
            match_ceiling_binding: false,
        }
    }

    /// Take and reset the accumulated guard-engagement counters
    /// (v4 §1.1): one (rule name, stats) entry per rule with any
    /// declared guards. The runner drains this per window into
    /// guards.csv.
    pub fn drain_guard_stats(&mut self) -> Vec<(String, GuardStats)> {
        let mut out = Vec::new();
        for (rule_id, stats) in self.guard_stats.iter_mut().enumerate() {
            if !stats.is_empty() {
                out.push((self.rules[rule_id].name.clone(), std::mem::take(stats)));
            }
        }
        out
    }

    pub fn add_observer(&mut self, observer: Box<dyn hg_observe::Observer>) {
        self.observers.add(observer);
    }

    pub fn set_policy(&mut self, policy: Box<dyn RuleSelectionPolicy>) {
        self.policy = Some(policy);
    }

    pub fn set_rule_source(&mut self, source: Box<dyn hg_rules::RuleSource>) {
        self.rule_source = Some(source);
        self.last_compile_step = None;
    }

    pub fn take_compile_report(&mut self) -> Option<hg_rules::RuleCompileReport> {
        self.compile_report.take()
    }

    /// Read-only diagnostic view. Unlike `take_compile_report`, this does not
    /// consume the lifecycle receipt needed by the ERE runtime.
    pub fn compile_report(&self) -> Option<&hg_rules::RuleCompileReport> {
        self.compile_report.as_ref()
    }

    /// Revision changes only when executable rule content changes, not just
    /// because an epoch boundary was visited. TPM windows must pin this.
    pub fn rule_table_revision(&self) -> u64 {
        self.rule_table_revision
    }

    /// Read-only reproducibility probe; cloning cannot advance engine RNG.
    pub fn rng_state_probe(&self) -> u64 {
        use rand::RngCore;
        self.rng.clone().next_u64()
    }

    pub fn compile_at_boundary(&mut self, store: &HypergraphStore) {
        let Some(source) = self.rule_source.as_mut() else {
            return;
        };
        if self.last_compile_step == Some(store.step()) || !source.epoch_len().due(store.step()) {
            return;
        }
        self.last_compile_step = Some(store.step());
        if let hg_rules::RuleSetDelta::Replace { rules, report } =
            source.rules(&store.snapshot(), store.step())
        {
            // Do not attribute old guard counters to newly assigned rule ids.
            if report.changed {
                self.guard_stats.clear();
                self.rule_table_revision += 1;
                self.rules = rules;
                self.apply_match_budget();
            }
            self.compile_report = Some(report);
        }
    }

    pub fn rules(&self) -> &[CompiledRule] {
        &self.rules
    }
    pub fn replace_static_rules(&mut self, rules: Vec<CompiledRule>) {
        self.rules = rules;
        self.guard_stats.clear();
        self.apply_match_budget();
    }
    pub fn set_match_budget(&mut self, work: usize, results: usize) {
        self.set_match_budget_policy(work, results, MatchBudgetPolicy::Split);
    }
    pub fn set_match_budget_policy(
        &mut self,
        work: usize,
        results: usize,
        policy: MatchBudgetPolicy,
    ) {
        self.match_budget = Some((work, results));
        self.match_budget_policy = policy;
        self.apply_match_budget();
    }
    fn apply_match_budget(&mut self) {
        if let Some((work, results)) = self.match_budget {
            let n = self.rules.len().max(1);
            for r in &mut self.rules {
                let (w, b) = match self.match_budget_policy {
                    MatchBudgetPolicy::Split => ((work / n).max(1), (results / n).max(1)),
                    MatchBudgetPolicy::Pooled => (work, results),
                };
                r.pattern.match_work_limit = Some(w);
                r.pattern.match_result_limit = Some(b);
            }
        }
    }
    pub fn match_ceiling_binding(&self) -> bool {
        self.match_ceiling_binding
    }
    pub fn match_budget_usage(&self) -> &MatchBudgetUsage {
        &self.match_budget_usage
    }

    pub fn observers_mut(&mut self) -> &mut ObserverChain {
        &mut self.observers
    }

    /// Collect the full candidate pool for this step: due creation-rule
    /// bindings first (v4 §5.3), then ordinary matches, then canonical sort
    /// so collection (thread) order is irrelevant.
    fn collect_candidates(&mut self, store: &HypergraphStore) -> Vec<Binding> {
        self.match_budget_usage = MatchBudgetUsage::default();
        let snapshot = store.snapshot();
        let step = store.step();

        let mut candidates: Vec<Binding> = Vec::new();

        for (rule_id, rule) in self.rules.iter().enumerate() {
            if !rule.enabled {
                continue;
            }
            if let Some(schedule) = rule.schedule {
                if schedule.is_due(step) {
                    // Trivial binding, zero bound nodes: can never conflict.
                    candidates.push(Binding::new(rule_id, rule.priority, SmallVec::new()));
                }
            }
        }

        if self.guard_stats.len() < self.rules.len() {
            self.guard_stats
                .resize(self.rules.len(), GuardStats::default());
        }
        if let Some((work, results)) = self
            .match_budget
            .filter(|_| self.match_budget_policy == MatchBudgetPolicy::Pooled)
        {
            // Canonical rule-id order, deliberately serial in parallel builds.
            // No priority/fitness shortcut may change the completed pool.
            for (rule_id, rule) in self.rules.iter().enumerate() {
                if !rule.enabled || rule.schedule.is_some() {
                    continue;
                }
                let mut bounded = rule.clone();
                bounded.pattern.match_work_limit = Some(work - self.match_budget_usage.work);
                bounded.pattern.match_result_limit =
                    Some(results - self.match_budget_usage.results);
                let (b, g) = find_bindings_counted(&snapshot, &bounded, rule_id);
                self.match_budget_usage
                    .record(rule_id, &g, b.len(), &bounded);
                let truncated = g.enumeration_truncated;
                candidates.extend(b);
                self.guard_stats[rule_id].merge(&g);
                if truncated {
                    break;
                }
            }
        } else {
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                let matched: Vec<(usize, Vec<Binding>, GuardStats)> = self
                    .rules
                    .par_iter()
                    .enumerate()
                    .filter(|(_, rule)| rule.enabled && rule.schedule.is_none())
                    .map(|(rule_id, rule)| {
                        let (b, g) = find_bindings_counted(&snapshot, rule, rule_id);
                        (rule_id, b, g)
                    })
                    .collect();
                for (rule_id, bindings, g) in matched {
                    self.match_budget_usage.record(
                        rule_id,
                        &g,
                        bindings.len(),
                        &self.rules[rule_id],
                    );
                    candidates.extend(bindings);
                    self.guard_stats[rule_id].merge(&g);
                }
            }
            #[cfg(not(feature = "parallel"))]
            {
                for (rule_id, rule) in self.rules.iter().enumerate() {
                    if rule.enabled && rule.schedule.is_none() {
                        let (b, g) = find_bindings_counted(&snapshot, rule, rule_id);
                        self.match_budget_usage.record(rule_id, &g, b.len(), rule);
                        candidates.extend(b);
                        self.guard_stats[rule_id].merge(&g);
                    }
                }
            }
        }

        // Canonical order: determinism regardless of thread count.
        candidates.sort_by_key(|b| b.canonical_key());
        candidates
    }

    /// Execute one step; returns the step's stats. Rewrite events go to the
    /// observer chain's `on_step_end`.
    pub fn step(&mut self, store: &mut HypergraphStore) -> StepStats {
        self.compile_at_boundary(store);
        self.stats = StepStats::new();

        let mut candidates = self.collect_candidates(store);
        self.stats.attempted_matches = candidates.len();
        self.match_ceiling_binding = if self.match_budget_policy == MatchBudgetPolicy::Pooled {
            self.match_budget_usage.truncated
        } else {
            self.guard_stats.iter().any(|g| g.enumeration_truncated)
        };
        if self.match_ceiling_binding {
            // Partial enumeration must never silently become a biased
            // rewrite-selection distribution. Halt without applying it.
            return self.stats.clone();
        }

        // Policy seam (v4 §6.1): between candidate collection and conflict
        // resolution — a policy can narrow the pool but the resolver still
        // guarantees node-disjointness.
        if let Some(policy) = self.policy.as_mut() {
            let snapshot = store.snapshot();
            let mut keep = policy.select(&snapshot, &candidates);
            keep.sort_unstable();
            keep.dedup();
            candidates = keep
                .into_iter()
                .filter_map(|i| candidates.get(i).cloned())
                .collect();
        }

        // Due creation firings are always accepted (v4 §5.3) — they can
        // never conflict, and neither the MaxParallel pairing cap nor the
        // sequential modes' single-candidate sampling may starve a
        // schedule. The cap/sampling budgets ordinary rewrites only.
        // (A RuleSelectionPolicy ran before this split, so a policy may
        // still deliberately drop a creation candidate.)
        let (creations, ordinary): (Vec<Binding>, Vec<Binding>) =
            candidates.into_iter().partition(|b| b.is_creation());

        let chosen_ordinary: Vec<Binding> = match self.step_mode {
            StepMode::MaxParallel => resolve_conflicts(&ordinary)
                .into_iter()
                .take(self.max_pairings_per_step)
                .collect(),
            StepMode::SequentialRandom => ordinary
                .choose(&mut self.rng)
                .cloned()
                .into_iter()
                .collect(),
            StepMode::SequentialWeighted => ordinary
                .choose_weighted(&mut self.rng, |b| self.rules[b.rule_id].weight)
                .ok()
                .cloned()
                .into_iter()
                .collect(),
        };
        let chosen: Vec<Binding> = creations.into_iter().chain(chosen_ordinary).collect();

        let mut events: Vec<RewriteEvent> = Vec::with_capacity(chosen.len());
        for binding in &chosen {
            // Disjoint field borrows: `rules` immutable, `rng` mutable.
            // Selection draws happened above, so the streams never
            // interleave within a step.
            let rule = &self.rules[binding.rule_id];
            let (event, applied, skipped) = apply_binding(store, rule, binding, &mut self.rng);
            self.stats.property_effects_applied += applied;
            self.stats.property_effects_skipped += skipped;
            self.stats.accepted_rewrites += 1;
            self.stats.nodes_created += event.nodes_created.len();
            self.stats.nodes_deleted += event.nodes_deleted.len();
            self.stats.edges_created += event.edges_created.len();
            self.stats.edges_deleted += event.edges_deleted.len();
            events.push(event);
        }

        store.set_step(store.step() + 1);
        self.stats.step = store.step();

        let snapshot = store.snapshot();
        self.observers.on_step_end(&snapshot, &self.stats, &events);

        self.stats.clone()
    }

    pub fn stats(&self) -> &StepStats {
        &self.stats
    }
}

/// Apply one accepted binding. Order per docs/rewrite_contract.md:
/// delete matched edges, delete LHS-only nodes (dangling cleanup), create
/// RHS nodes, apply property effects (step 5.5), create RHS edges.
///
/// Returns `(event, effects_applied, effects_skipped)`. The RNG is drawn
/// from ONLY when a `jitter` effect actually executes — rulesets without
/// jitter are stream-identical to pre-feature behavior
/// (docs/reproducibility_policy.md).
pub fn apply_binding(
    store: &mut HypergraphStore,
    rule: &CompiledRule,
    binding: &Binding,
    rng: &mut ChaCha8Rng,
) -> (RewriteEvent, u64, u64) {
    let template = &rule.template;
    let mut nodes_created = Vec::new();
    let mut nodes_deleted = Vec::new();
    let mut edges_created = Vec::new();
    let mut edges_deleted = Vec::new();

    // 1. Delete matched edges.
    for &edge_id in &binding.matched_edges {
        if store.get_edge(edge_id).is_some() {
            store.remove_edge(edge_id);
            edges_deleted.push(edge_id);
        }
    }

    // 2. Delete LHS-only nodes (incident edges removed as dangling cleanup).
    // Surviving endpoints of cleanup-deleted edges are recorded on the
    // event: they are the nodes affected through the engine's documented
    // locality-of-effect exception, and downstream causal analyses report
    // that channel separately (edge ids alone would be useless post-hoc —
    // the edges no longer exist to look up).
    let mut cleanup_neighbors: Vec<NodeId> = Vec::new();
    let mut vanished_energy = 0.0f64;
    for &lhs_ix in &template.lhs_deleted {
        if let Some(&node) = binding.bound_nodes.get(lhs_ix) {
            if store.get_node(node).is_some() {
                // v4 §6.3 ledger: energy (key 1) leaves the system with
                // the node; the amount is recorded on the event, read
                // BEFORE deletion, exact (no rounding source: reads are
                // stored f64 values).
                if let Some(PropertyValue::Float(e)) =
                    store.get_node(node).and_then(|d| d.get_property(1))
                {
                    vanished_energy += e;
                }
                let dangling: Vec<_> = store.edges_incident_to(node).to_vec();
                for e in dangling {
                    if let Some(edge) = store.get_edge(e) {
                        for &endpoint in &edge.nodes {
                            if endpoint != node && !cleanup_neighbors.contains(&endpoint) {
                                cleanup_neighbors.push(endpoint);
                            }
                        }
                    }
                    edges_deleted.push(e);
                }
                store.remove_node(node);
                nodes_deleted.push(node);
            }
        }
    }

    // 3. Create RHS nodes (survivors resolve to their bound ids; a
    // survivor with an explicit RHS label is RELABELLED — the conservative
    // mark-transfer mechanism. This path was missing entirely, which
    // silently ignored RHS node_labels on surviving vars for every rule).
    let mut rhs_nodes: Vec<Option<NodeId>> = Vec::with_capacity(template.n_rhs_vars);
    for rhs_ix in 0..template.n_rhs_vars {
        match template.rhs_from_lhs[rhs_ix] {
            Some(lhs_ix) => {
                let id = binding.bound_nodes.get(lhs_ix).copied();
                if let (Some(node), Some(Some(new_label))) = (id, template.rhs_relabels.get(rhs_ix))
                {
                    if let Some(data) = store.get_node_mut(node) {
                        data.label = *new_label;
                    }
                }
                rhs_nodes.push(id);
            }
            None => {
                let label = template.rhs_new_labels[rhs_ix];
                let id = store.add_node(NodeData::new(label, store.step()));
                nodes_created.push(id);
                rhs_nodes.push(Some(id));
            }
        }
    }

    // 4 (a.k.a. step 5.5). Property effects: between RHS node creation
    // and RHS edge creation, since an effect's source_var may be freshly
    // created. Sequential, declaration order, reads see earlier writes.
    let (fx_applied, fx_skipped) = apply_property_effects(store, rule, &rhs_nodes, rng);

    // 5. Create RHS edges (labels from the sparse map; absent = 0).
    for (var_ixs, label) in &template.rhs_edges {
        let mut endpoints: SmallVec<[NodeId; 8]> = SmallVec::new();
        for &ix in var_ixs {
            match rhs_nodes.get(ix).copied().flatten() {
                Some(id) if store.get_node(id).is_some() => endpoints.push(id),
                _ => break,
            }
        }
        if endpoints.len() == var_ixs.len() {
            let edge_id = store.add_edge(HyperEdge::new(*label, endpoints));
            edges_created.push(edge_id);
        }
    }

    (
        RewriteEvent {
            rule_id: binding.rule_id,
            rule_name: rule.name.clone(),
            binding: binding.clone(),
            nodes_created,
            nodes_deleted,
            edges_created,
            edges_deleted,
            cleanup_neighbors,
            vanished_energy,
        },
        fx_applied,
        fx_skipped,
    )
}

/// Rate-limited skip warning: the counter in `StepStats` is the durable
/// signal; the log is a courtesy that must not flood.
fn warn_effect_skipped(rule: &str, reason: &str) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static WARNED: AtomicU32 = AtomicU32::new(0);
    let n = WARNED.fetch_add(1, Ordering::Relaxed);
    if n < 16 {
        tracing::warn!(rule, reason, "property effect skipped");
    } else if n == 16 {
        tracing::warn!("further property-effect skip warnings suppressed (see StepStats.property_effects_skipped)");
    }
}

/// `property_effects` execution (v4 §5.2/§6.3 + jitter): sequential in
/// declaration order; each effect's reads see earlier effects' writes in
/// the same rewrite (chaining is under rule-author control — see
/// docs/rule_grammar.md's 160-vs-80 example). A runtime mismatch skips
/// THAT effect only: the rewrite always completes (Contract Rule 6
/// atomicity), the skip is counted, never silent. Int arithmetic
/// saturates; values are NOT clamped — guards are the author's tool for
/// preconditions, the engine does arithmetic as written.
fn apply_property_effects(
    store: &mut HypergraphStore,
    rule: &CompiledRule,
    rhs_nodes: &[Option<NodeId>],
    rng: &mut ChaCha8Rng,
) -> (u64, u64) {
    use hg_rules::CompiledEffectOp;
    use rand::Rng;

    let mut applied = 0u64;
    let mut skipped = 0u64;
    for effect in &rule.template.property_effects {
        let skip = |reason: &str, n: &mut u64| {
            warn_effect_skipped(&rule.name, reason);
            *n += 1;
        };
        let Some(target) = rhs_nodes.get(effect.target_ix).copied().flatten() else {
            skip("target node unresolved", &mut skipped);
            continue;
        };
        // Read phase (immutable): compute the value to write. Reads go
        // through the store NOW, so earlier effects' writes are visible.
        let new_value = match &effect.op {
            CompiledEffectOp::Set { value } => Some(*value),
            CompiledEffectOp::CopyFrom {
                source_ix,
                source_key,
            } => {
                let Some(source) = rhs_nodes.get(*source_ix).copied().flatten() else {
                    skip("source node unresolved", &mut skipped);
                    continue;
                };
                match store
                    .get_node(source)
                    .and_then(|d| d.get_property(*source_key))
                {
                    Some(v) => Some(v),
                    None => {
                        skip("copy_from: source property missing", &mut skipped);
                        continue;
                    }
                }
            }
            CompiledEffectOp::ScaleFrom {
                source_ix,
                source_key,
                factor,
            } => {
                let Some(source) = rhs_nodes.get(*source_ix).copied().flatten() else {
                    skip("source node unresolved", &mut skipped);
                    continue;
                };
                match store
                    .get_node(source)
                    .and_then(|d| d.get_property(*source_key))
                {
                    Some(PropertyValue::Float(f)) => Some(PropertyValue::Float(f * factor)),
                    Some(_) => {
                        skip("scale_from: source is not Float", &mut skipped);
                        continue;
                    }
                    None => {
                        skip("scale_from: source property missing", &mut skipped);
                        continue;
                    }
                }
            }
            CompiledEffectOp::Add { value } | CompiledEffectOp::Subtract { value } => {
                let sub = matches!(effect.op, CompiledEffectOp::Subtract { .. });
                let current = store
                    .get_node(target)
                    .and_then(|d| d.get_property(effect.key));
                match (current, value) {
                    (Some(PropertyValue::Int(a)), PropertyValue::Int(b)) => {
                        Some(PropertyValue::Int(if sub {
                            a.saturating_sub(*b)
                        } else {
                            a.saturating_add(*b)
                        }))
                    }
                    (Some(PropertyValue::Float(a)), PropertyValue::Float(b)) => {
                        Some(PropertyValue::Float(if sub { a - b } else { a + b }))
                    }
                    (None, _) => {
                        skip("add/subtract: target property missing", &mut skipped);
                        continue;
                    }
                    _ => {
                        skip("add/subtract: variant mismatch", &mut skipped);
                        continue;
                    }
                }
            }
            CompiledEffectOp::AddFromProperty {
                source_ix,
                source_key,
            }
            | CompiledEffectOp::SubtractFromProperty {
                source_ix,
                source_key,
            } => {
                let sub = matches!(effect.op, CompiledEffectOp::SubtractFromProperty { .. });
                let Some(source) = rhs_nodes.get(*source_ix).copied().flatten() else {
                    skip(
                        "add/subtract_from_property: source node unresolved",
                        &mut skipped,
                    );
                    continue;
                };
                let amount = match store
                    .get_node(source)
                    .and_then(|d| d.get_property(*source_key))
                {
                    Some(PropertyValue::Float(f)) => f,
                    Some(_) => {
                        skip(
                            "add/subtract_from_property: source is not Float",
                            &mut skipped,
                        );
                        continue;
                    }
                    None => {
                        skip(
                            "add/subtract_from_property: source property missing",
                            &mut skipped,
                        );
                        continue;
                    }
                };
                match store
                    .get_node(target)
                    .and_then(|d| d.get_property(effect.key))
                {
                    Some(PropertyValue::Float(a)) => Some(PropertyValue::Float(if sub {
                        a - amount
                    } else {
                        a + amount
                    })),
                    Some(_) => {
                        skip(
                            "add/subtract_from_property: target is not Float",
                            &mut skipped,
                        );
                        continue;
                    }
                    None => {
                        skip(
                            "add/subtract_from_property: target property missing",
                            &mut skipped,
                        );
                        continue;
                    }
                }
            }
            CompiledEffectOp::Jitter { magnitude } => {
                // Validate BEFORE drawing: a skipped jitter consumes no
                // randomness, so mismatches cannot shift the stream.
                let current = store
                    .get_node(target)
                    .and_then(|d| d.get_property(effect.key));
                match current {
                    Some(PropertyValue::Float(f)) => {
                        let delta: f64 = rng.gen_range(-magnitude..=*magnitude);
                        Some(PropertyValue::Float(f + delta))
                    }
                    Some(PropertyValue::Int(i)) => {
                        let m = magnitude.floor() as i64;
                        let delta = if m > 0 { rng.gen_range(-m..=m) } else { 0 };
                        Some(PropertyValue::Int(i.saturating_add(delta)))
                    }
                    Some(_) => {
                        skip("jitter: target is not Int or Float", &mut skipped);
                        continue;
                    }
                    None => {
                        skip("jitter: target property missing", &mut skipped);
                        continue;
                    }
                }
            }
        };
        // Write phase.
        match (store.get_node_mut(target), new_value) {
            (Some(data), Some(v)) => {
                data.set_property(effect.key, v);
                applied += 1;
            }
            _ => skip("target node missing at write", &mut skipped),
        }
    }
    (applied, skipped)
}
