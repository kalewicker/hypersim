//! Pattern matcher over compiled rules.
//!
//! **Non-injective by default** (v4 §13 Phase 5): two pattern variables may
//! bind the same graph node unless a `vars_distinct` guard says otherwise.
//! Pattern *edges* are injective — each LHS edge consumes a distinct graph
//! edge, because apply deletes every matched edge exactly once.

use hg_core::{EdgeId, GraphSnapshot, NodeId, PropertyValue};
use hg_observe::Binding;
use hg_rules::{CompiledRule, MatchPattern};
use smallvec::SmallVec;

/// Per-rule guard-engagement counters (plan v4 §1.1): one (evals,
/// rejects) pair per declared guard, in declaration order. Guards
/// short-circuit, so a guard after a rejecting one is not evaluated
/// (and not counted) for that candidate — evals differ down the list
/// by design. `vars_distinct` is a binding constraint (it prunes the
/// enumeration, it is not evaluated against completed candidates) and
/// is deliberately not counted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuardStats {
    /// node_property_guards, by declaration index.
    pub property_guards: Vec<(u64, u64)>,
    /// node_property_compares, by declaration index.
    pub property_compares: Vec<(u64, u64)>,
    /// node_degree_compares, by declaration index.
    pub degree_compares: Vec<(u64, u64)>,
    /// node_age_compares, by declaration index (v8 §3.1).
    pub age_compares: Vec<(u64, u64)>,
}

impl GuardStats {
    pub fn sized_for(pattern: &MatchPattern) -> Self {
        Self {
            property_guards: vec![(0, 0); pattern.property_guards.len()],
            property_compares: vec![(0, 0); pattern.property_compares.len()],
            degree_compares: vec![(0, 0); pattern.degree_compares.len()],
            age_compares: vec![(0, 0); pattern.age_compares.len()],
        }
    }

    pub fn merge(&mut self, other: &GuardStats) {
        let acc = |a: &mut Vec<(u64, u64)>, b: &[(u64, u64)]| {
            if a.len() < b.len() {
                a.resize(b.len(), (0, 0));
            }
            for (x, y) in a.iter_mut().zip(b) {
                x.0 += y.0;
                x.1 += y.1;
            }
        };
        acc(&mut self.property_guards, &other.property_guards);
        acc(&mut self.property_compares, &other.property_compares);
        acc(&mut self.degree_compares, &other.degree_compares);
        acc(&mut self.age_compares, &other.age_compares);
    }

    pub fn is_empty(&self) -> bool {
        self.property_guards.is_empty()
            && self.property_compares.is_empty()
            && self.degree_compares.is_empty()
            && self.age_compares.is_empty()
    }
}

/// Find all bindings of `rule`'s LHS in the snapshot. Creation rules
/// (empty pattern) are handled by the engine's schedule check, not here.
pub fn find_bindings(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
) -> Vec<Binding> {
    find_bindings_counted(snapshot, rule, rule_id).0
}

/// As [`find_bindings`], also returning the guard-engagement counters
/// accumulated over every completed candidate binding (v4 §1.1).
pub fn find_bindings_counted(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
) -> (Vec<Binding>, GuardStats) {
    let pattern = &rule.pattern;
    let mut stats = GuardStats::sized_for(pattern);
    if pattern.n_vars == 0 && pattern.edges.is_empty() {
        return (Vec::new(), stats);
    }

    let mut out = Vec::new();
    let mut bound: Vec<Option<NodeId>> = vec![None; pattern.n_vars];
    let mut used_edges: SmallVec<[EdgeId; 8]> = SmallVec::new();
    let mut matched_edges: Vec<Option<EdgeId>> = vec![None; pattern.edges.len()];

    match_edge(
        snapshot,
        rule,
        rule_id,
        pattern,
        0,
        &mut bound,
        &mut used_edges,
        &mut matched_edges,
        &mut out,
        &mut stats,
    );
    (out, stats)
}

/// Can `var` be bound to `node` given constraints and current assignment?
fn bind_ok(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    var: usize,
    node: NodeId,
) -> bool {
    if let Some(existing) = bound[var] {
        return existing == node;
    }
    if let Some(required) = pattern.node_label_constraints[var] {
        match snapshot.get_node(node) {
            Some(data) if data.label == required => {}
            _ => return false,
        }
    }
    // Distinctness only where guards demand it (non-injective by default).
    for &(a, b) in &pattern.distinct_pairs {
        let other = if a == var { b } else if b == var { a } else { continue };
        if bound[other] == Some(node) {
            return false;
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn match_edge(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
    pattern: &MatchPattern,
    edge_ix: usize,
    bound: &mut Vec<Option<NodeId>>,
    used_edges: &mut SmallVec<[EdgeId; 8]>,
    matched_edges: &mut Vec<Option<EdgeId>>,
    out: &mut Vec<Binding>,
    stats: &mut GuardStats,
) {
    if edge_ix == pattern.edges.len() {
        bind_free_vars(snapshot, rule, rule_id, pattern, 0, bound, matched_edges, out, stats);
        return;
    }

    let pe = &pattern.edges[edge_ix];

    // Candidate graph edges. Adjacency-guided when any of this pattern
    // edge's variables is already bound: only edges incident to that
    // node can match, and the incidence list is O(deg) instead of O(E)
    // — without this, multi-edge patterns (lk_fuse's wedge,
    // triangle_subdivide) cost O(E^k) per step. The incidence index is
    // per-slot (multi-incidence edges appear once per occurrence), so
    // candidates are deduplicated — a duplicate would emit duplicate
    // bindings. Candidate ORDER differs from the global-scan path, but
    // the engine canonically sorts all bindings before selection, so
    // trajectories are bit-identical (pinned by the hash regression
    // test). Falls back to the label index / full scan when no variable
    // is bound yet.
    let bound_anchor: Option<NodeId> = pe
        .var_ixs
        .iter()
        .filter_map(|&v| bound[v])
        .min_by_key(|&n| snapshot.edges_incident_to(n).len());
    let candidate_ids: Vec<EdgeId> = match (bound_anchor, pe.label) {
        (Some(anchor), label) => {
            let mut out: Vec<EdgeId> = Vec::new();
            for &e in snapshot.edges_incident_to(anchor) {
                if out.contains(&e) {
                    continue;
                }
                if let Some(l) = label {
                    match snapshot.get_edge(e) {
                        Some(edge) if edge.label == l => {}
                        _ => continue,
                    }
                }
                out.push(e);
            }
            out
        }
        (None, Some(label)) => snapshot.edges_with_label(label).to_vec(),
        (None, None) => snapshot.edge_ids().collect(),
    };

    for edge_id in candidate_ids {
        if used_edges.contains(&edge_id) {
            continue;
        }
        let Some(edge) = snapshot.get_edge(edge_id) else { continue };
        if edge.nodes.len() != pe.var_ixs.len() {
            continue;
        }

        // Unify pattern vars with the edge's endpoints over permutations
        // (a hyperedge is an unordered endpoint set for matching).
        let mut pos_used = vec![false; edge.nodes.len()];
        unify_positions(
            snapshot, rule, rule_id, pattern, edge_ix, edge_id,
            &edge.nodes, 0, &mut pos_used, bound, used_edges, matched_edges, out, stats,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn unify_positions(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
    pattern: &MatchPattern,
    edge_ix: usize,
    edge_id: EdgeId,
    edge_nodes: &[NodeId],
    var_pos: usize,
    pos_used: &mut Vec<bool>,
    bound: &mut Vec<Option<NodeId>>,
    used_edges: &mut SmallVec<[EdgeId; 8]>,
    matched_edges: &mut Vec<Option<EdgeId>>,
    out: &mut Vec<Binding>,
    stats: &mut GuardStats,
) {
    let pe = &pattern.edges[edge_ix];
    if var_pos == pe.var_ixs.len() {
        used_edges.push(edge_id);
        matched_edges[edge_ix] = Some(edge_id);
        match_edge(
            snapshot, rule, rule_id, pattern, edge_ix + 1,
            bound, used_edges, matched_edges, out, stats,
        );
        matched_edges[edge_ix] = None;
        used_edges.pop();
        return;
    }

    let var = pe.var_ixs[var_pos];
    // Positions holding the same node are interchangeable: trying both
    // yields byte-identical bindings (an edge (x,x) would otherwise match
    // pattern [a,b] twice), inflating attempted_matches and double-
    // weighting such matches in the sequential sampling modes.
    let mut tried: SmallVec<[NodeId; 8]> = SmallVec::new();
    for (pos, &node) in edge_nodes.iter().enumerate() {
        if pos_used[pos] || tried.contains(&node) {
            continue;
        }
        if !bind_ok(snapshot, pattern, bound, var, node) {
            continue;
        }
        tried.push(node);
        let was_bound = bound[var].is_some();
        bound[var] = Some(node);
        pos_used[pos] = true;

        unify_positions(
            snapshot, rule, rule_id, pattern, edge_ix, edge_id, edge_nodes,
            var_pos + 1, pos_used, bound, used_edges, matched_edges, out, stats,
        );

        pos_used[pos] = false;
        if !was_bound {
            bound[var] = None;
        }
    }
}

/// Bind variables that appear in no edge (isolated pattern vars), then
/// run guards and emit the binding.
#[allow(clippy::too_many_arguments)]
fn bind_free_vars(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
    pattern: &MatchPattern,
    var: usize,
    bound: &mut Vec<Option<NodeId>>,
    matched_edges: &[Option<EdgeId>],
    out: &mut Vec<Binding>,
    stats: &mut GuardStats,
) {
    if var == pattern.n_vars {
        if check_property_guards(snapshot, pattern, bound, stats)
            && check_property_compares(snapshot, pattern, bound, stats)
            && check_degree_compares(snapshot, pattern, bound, stats)
            && check_age_compares(snapshot, pattern, bound, stats)
        {
            let bound_nodes: SmallVec<[NodeId; 8]> =
                bound.iter().map(|b| b.expect("all vars bound")).collect();
            let matched: SmallVec<[EdgeId; 8]> =
                matched_edges.iter().map(|e| e.expect("all edges matched")).collect();
            out.push(Binding::with_edges(rule_id, rule.priority, bound_nodes, matched));
        }
        return;
    }
    if bound[var].is_some() {
        bind_free_vars(snapshot, rule, rule_id, pattern, var + 1, bound, matched_edges, out, stats);
        return;
    }
    // Isolated var: try every live node under the constraints.
    let ids: Vec<NodeId> = snapshot.node_ids().collect();
    for node in ids {
        if !bind_ok(snapshot, pattern, bound, var, node) {
            continue;
        }
        bound[var] = Some(node);
        bind_free_vars(snapshot, rule, rule_id, pattern, var + 1, bound, matched_edges, out, stats);
        bound[var] = None;
    }
}

fn check_property_guards(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    stats: &mut GuardStats,
) -> bool {
    for (i, g) in pattern.property_guards.iter().enumerate() {
        stats.property_guards[i].0 += 1;
        let reject = |stats: &mut GuardStats| {
            stats.property_guards[i].1 += 1;
            false
        };
        let Some(Some(node)) = bound.get(g.var_ix) else { return reject(stats) };
        let Some(data) = snapshot.get_node(*node) else { return reject(stats) };
        match data.get_property(g.key) {
            // Missing property fails the guard (no value to compare).
            None => return reject(stats),
            Some(v) => {
                if !v.compare(g.op, &g.value) {
                    return reject(stats);
                }
            }
        }
    }
    true
}

/// Degree comparison guards (plan v4 §6.2, `node_degree_compares`):
/// degree of a bound node vs an Int constant (+ optional Int offset).
/// Pinned semantics: degree = count of DISTINCT incident hyperedges — a
/// node appearing twice in one edge counts that edge once. The store's
/// incidence index records one entry PER SLOT (multi-incidence appears
/// twice), so the guard deduplicates here: O(deg), not O(1) — the §6.2
/// precondition check found the index is per-slot, and this is the
/// in-scope adjacency accommodation. Counters wired from birth (§1.1).
fn check_degree_compares(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    stats: &mut GuardStats,
) -> bool {
    for (i, d) in pattern.degree_compares.iter().enumerate() {
        stats.degree_compares[i].0 += 1;
        let reject = |stats: &mut GuardStats| {
            stats.degree_compares[i].1 += 1;
            false
        };
        let Some(Some(node)) = bound.get(d.var_ix) else { return reject(stats) };
        if snapshot.get_node(*node).is_none() {
            return reject(stats);
        }
        let mut distinct: SmallVec<[EdgeId; 8]> = SmallVec::new();
        for e in snapshot.edges_incident_to(*node) {
            if !distinct.contains(e) {
                distinct.push(*e);
            }
        }
        let degree = distinct.len() as i64;
        let rhs = match d.offset {
            None => d.value,
            Some(off) => d.value.saturating_add(off),
        };
        if !PropertyValue::Int(degree).compare(d.op, &PropertyValue::Int(rhs)) {
            return reject(stats);
        }
    }
    true
}

/// Age comparison guards (plan v8 §3.1, `node_age_compares`): evaluate
/// `(step − birth_step(var)) <op> value`, where the birth step is
/// `NodeData::birth_step` — written once, at creation, by every creation
/// site — and the step is the snapshot's own counter. This is v8's
/// PASSIVE CLOCK: it reads state the store already holds, so it claims no
/// bindings and can lose nothing to, and steal nothing from, the physics.
///
/// Saturating: a node whose birth step somehow exceeds the current step
/// (only reachable by hand-constructing a store) reads age 0 rather than
/// underflowing. Missing nodes reject, exactly as the other guards do.
///
/// NOT a lattice protection: every node has a birth step, and IC nodes
/// read as born at step 0, hence maximally old. The countdown gate got
/// lattice rejection for free from a missing key; this one does not, and
/// rules must carry their own (see `docs/grammar_audit_v8.md`).
fn check_age_compares(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    stats: &mut GuardStats,
) -> bool {
    if pattern.age_compares.is_empty() {
        return true;
    }
    let step = snapshot.step();
    for (i, a) in pattern.age_compares.iter().enumerate() {
        stats.age_compares[i].0 += 1;
        let reject = |stats: &mut GuardStats| {
            stats.age_compares[i].1 += 1;
            false
        };
        let Some(Some(node)) = bound.get(a.var_ix) else { return reject(stats) };
        let Some(data) = snapshot.get_node(*node) else { return reject(stats) };
        let age = step.saturating_sub(data.birth_step) as i64;
        if !PropertyValue::Int(age).compare(a.op, &PropertyValue::Int(a.value)) {
            return reject(stats);
        }
    }
    true
}

/// Property-to-property comparison guards (hg-alife S1, scope §5.3): evaluate
/// `var_a[key_a] <op> var_b[key_b]`. Mirrors `check_property_guards` exactly —
/// a missing property on either side fails the guard, a variant mismatch fails
/// the guard (via `PropertyValue::compare`), and no RNG is drawn. `var_a` and
/// `var_b` may resolve to the same bound node (self-comparison across keys).
fn check_property_compares(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    stats: &mut GuardStats,
) -> bool {
    for (i, c) in pattern.property_compares.iter().enumerate() {
        stats.property_compares[i].0 += 1;
        let reject = |stats: &mut GuardStats| {
            stats.property_compares[i].1 += 1;
            false
        };
        let Some(Some(node_a)) = bound.get(c.var_a_ix) else { return reject(stats) };
        let Some(Some(node_b)) = bound.get(c.var_b_ix) else { return reject(stats) };
        let Some(data_a) = snapshot.get_node(*node_a) else { return reject(stats) };
        let Some(data_b) = snapshot.get_node(*node_b) else { return reject(stats) };
        let (Some(a), Some(b)) = (data_a.get_property(c.key_a), data_b.get_property(c.key_b))
        else {
            // Missing property on either side fails the guard.
            return reject(stats);
        };
        // Optional offset on side B (LK-R v2): Int+Int / Float+Float only;
        // a variant mismatch fails the guard, never coerces. Int addition
        // saturates (same discipline as property effects).
        let b = match c.offset {
            None => b,
            Some(off) => match (b, off) {
                (PropertyValue::Int(x), PropertyValue::Int(y)) => {
                    PropertyValue::Int(x.saturating_add(y))
                }
                (PropertyValue::Float(x), PropertyValue::Float(y)) => {
                    PropertyValue::Float(x + y)
                }
                _ => return reject(stats),
            },
        };
        if !a.compare(c.op, &b) {
            return reject(stats);
        }
    }
    true
}
