//! Pattern matcher over compiled rules.
//!
//! **Non-injective by default** (v4 §13 Phase 5): two pattern variables may
//! bind the same graph node unless a `vars_distinct` guard says otherwise.
//! Pattern *edges* are injective — each LHS edge consumes a distinct graph
//! edge, because apply deletes every matched edge exactly once.

use hg_core::{EdgeId, GraphSnapshot, NodeId, PropertyValue};
use hg_observe::Binding;
use hg_rules::{CompiledRule, MatchPattern, NodeQuantity};
use smallvec::SmallVec;

/// Work model used by public matcher/Engine dispatch in this exact build.
/// Explicit legacy/candidate control APIs can deliberately use the other one.
pub const MATCHER_WORK_MODEL: &str = if cfg!(feature = "repeated-edge-feasibility") {
    "repeated-self-pair-feasibility-v1"
} else {
    "legacy-dfs-v1"
};

/// Per-rule guard-engagement counters (plan v4 §1.1): one (evals,
/// rejects) pair per declared guard, in declaration order. Guards
/// short-circuit, so a guard after a rejecting one is not evaluated
/// (and not counted) for that candidate — evals differ down the list
/// by design. `vars_distinct` is a binding constraint (it prunes the
/// enumeration, it is not evaluated against completed candidates) and
/// is deliberately not counted.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GuardStats {
    pub enumeration_work: usize,
    pub enumeration_truncated: bool,
    /// node_property_guards, by declaration index.
    pub property_guards: Vec<(u64, u64)>,
    /// node_property_compares, by declaration index.
    pub property_compares: Vec<(u64, u64)>,
    /// node_degree_compares, by declaration index.
    pub degree_compares: Vec<(u64, u64)>,
    /// node_quantity_compares, by declaration index (E-track).
    pub quantity_compares: Vec<(u64, u64)>,
}

impl GuardStats {
    pub fn sized_for(pattern: &MatchPattern) -> Self {
        Self {
            enumeration_work: 0,
            enumeration_truncated: false,
            property_guards: vec![(0, 0); pattern.property_guards.len()],
            property_compares: vec![(0, 0); pattern.property_compares.len()],
            degree_compares: vec![(0, 0); pattern.degree_compares.len()],
            quantity_compares: vec![(0, 0); pattern.quantity_compares.len()],
        }
    }

    pub fn merge(&mut self, other: &GuardStats) {
        self.enumeration_work += other.enumeration_work;
        self.enumeration_truncated |= other.enumeration_truncated;
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
        acc(&mut self.quantity_compares, &other.quantity_compares);
    }

    pub fn is_empty(&self) -> bool {
        self.property_guards.is_empty()
            && self.property_compares.is_empty()
            && self.degree_compares.is_empty()
            && self.quantity_compares.is_empty()
    }
}

fn enter(pattern: &MatchPattern, out: &[Binding], stats: &mut GuardStats) -> bool {
    if pattern.match_work_limit.is_none() && pattern.match_result_limit.is_none() { return true; }
    if stats.enumeration_truncated { return false; }
    if pattern.match_work_limit.is_some_and(|n| stats.enumeration_work >= n)
        || pattern.match_result_limit.is_some_and(|n| out.len() >= n) {
        stats.enumeration_truncated = true; return false;
    }
    stats.enumeration_work += 1;
    true
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
    if cfg!(feature = "repeated-edge-feasibility") {
        find_bindings_counted_feasible(snapshot, rule, rule_id)
    } else {
        find_bindings_counted_legacy(snapshot, rule, rule_id)
    }
}

/// Explicit original counted-work model, including its finite-budget prefix.
/// Differential engineering control; default builds continue to use this.
pub fn find_bindings_counted_legacy(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
) -> (Vec<Binding>, GuardStats) {
    find_bindings_internal(snapshot, rule, rule_id, false)
}

/// Opt-in repeated-self-pair-feasibility-v1 counted-work model.
///
/// Charge necessary-condition probes to the same allowance as DFS. Only
/// binary [v,v] edges and explicit work budgets participate. A successful
/// probe does not reorder/filter the original DFS or evaluate its guards.
/// Complete bindings/guard counts agree with legacy; finite-budget work and
/// completion intentionally need not. This is not scientific admission.
pub fn find_bindings_counted_feasible(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
) -> (Vec<Binding>, GuardStats) {
    find_bindings_internal(snapshot, rule, rule_id, true)
}

fn find_bindings_internal(
    snapshot: &GraphSnapshot<'_>,
    rule: &CompiledRule,
    rule_id: usize,
    feasibility: bool,
) -> (Vec<Binding>, GuardStats) {
    let pattern = &rule.pattern;
    let mut stats = GuardStats::sized_for(pattern);
    if pattern.n_vars == 0 && pattern.edges.is_empty() {
        return (Vec::new(), stats);
    }

    let mut out = Vec::new();
    let mut bound: Vec<Option<NodeId>> = vec![None; pattern.n_vars];
    if feasibility && pattern.match_work_limit.is_some()
        && !self_pairs_feasible(snapshot, pattern, &bound, &out, &mut stats) {
        return (out, stats);
    }
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

/// Global necessary condition, not an assignment, edge-injectivity test or
/// validation cache. No graph candidates/metadata are excluded from charging.
fn self_pairs_feasible(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    out: &[Binding],
    stats: &mut GuardStats,
) -> bool {
    for pe in pattern.edges.iter().filter(|e| e.var_ixs.len() == 2 && e.var_ixs[0] == e.var_ixs[1]) {
        if !enter(pattern, out, stats) { return false; } // probe
        let mut possible = false;
        for id in snapshot.edge_ids() {
            if !enter(pattern, out, stats) { return false; } // edge inspection
            let Some(edge) = snapshot.get_edge(id) else { continue };
            if edge.nodes.len() != 2 || pe.label.is_some_and(|l| l != edge.label) {
                continue;
            }
            if !enter(pattern, out, stats) { return false; } // endpoint equality
            if edge.nodes[0] != edge.nodes[1] { continue; }
            if !enter(pattern, out, stats) { return false; } // bind eligibility
            if bind_ok(snapshot, pattern, bound, pe.var_ixs[0], edge.nodes[0]) {
                possible = true;
                break;
            }
        }
        if !possible {
            // Conservative boundary: never declare a complete negative if
            // the allowance ran out before charging the proof decision.
            if !enter(pattern, out, stats) { return false; }
            return false;
        }
    }
    true
}

#[cfg(test)]
#[path = "feasibility_tests.rs"]
mod feasibility_tests;

/// Can `var` be bound to `node` given constraints and current assignment?
fn bind_ok(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    var: usize,
    node: NodeId,
) -> bool {
    if let Some(ceiling) = pattern.node_label_ceiling {
        if snapshot.get_node(node).is_none_or(|data| data.label.0 >= ceiling) {
            return false;
        }
    }
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
    if !enter(pattern, out, stats) { return; }
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
        if stats.enumeration_truncated { break; }
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
    if !enter(pattern, out, stats) { return; }
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
        if stats.enumeration_truncated { break; }
        if pe.ordered && pos != var_pos {
            continue;
        }
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
    if !enter(pattern, out, stats) { return; }
    if var == pattern.n_vars {
        if check_property_guards(snapshot, pattern, bound, stats)
            && check_property_compares(snapshot, pattern, bound, stats)
            && check_degree_compares(snapshot, pattern, bound, stats)
            // E-track, appended LAST so no existing guard's short-circuit
            // position moves — guard counters are position-sensitive by design.
            && check_quantity_compares(snapshot, pattern, bound, stats)
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
        if stats.enumeration_truncated { break; }
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
        let degree = distinct_degree(snapshot, *node);
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

/// Degree of a bound node: count of DISTINCT incident hyperedges. Factored out
/// so `check_degree_compares` and `check_quantity_compares` cannot drift apart
/// on what a degree is — the store's incidence index records one entry PER
/// SLOT, so multi-incidence appears twice and must be deduplicated here.
fn distinct_degree(snapshot: &GraphSnapshot<'_>, node: NodeId) -> i64 {
    let mut distinct: SmallVec<[EdgeId; 8]> = SmallVec::new();
    for e in snapshot.edges_incident_to(node) {
        if !distinct.contains(e) {
            distinct.push(*e);
        }
    }
    distinct.len() as i64
}

/// Var-to-var node-quantity comparison guards (E-track): evaluate
/// `quantity(var_a) <op> (quantity(var_b) + offset)`.
///
/// **A post-binding filter.** Unlike `check_degree_compares`, which compares
/// against a constant, this cannot be evaluated until BOTH vars are bound, so
/// it runs only on completed candidates and never prunes enumeration. Cost is
/// `O(deg(a) + deg(b))` per completed candidate.
///
/// Failure semantics mirror the other guards exactly: an unbound var or a node
/// missing from the snapshot fails the guard, never panics. Int arithmetic,
/// saturating; no RNG drawn; no float surface.
fn check_quantity_compares(
    snapshot: &GraphSnapshot<'_>,
    pattern: &MatchPattern,
    bound: &[Option<NodeId>],
    stats: &mut GuardStats,
) -> bool {
    for (i, q) in pattern.quantity_compares.iter().enumerate() {
        stats.quantity_compares[i].0 += 1;
        let reject = |stats: &mut GuardStats| {
            stats.quantity_compares[i].1 += 1;
            false
        };
        let Some(Some(node_a)) = bound.get(q.var_a_ix) else { return reject(stats) };
        let Some(Some(node_b)) = bound.get(q.var_b_ix) else { return reject(stats) };
        if snapshot.get_node(*node_a).is_none() || snapshot.get_node(*node_b).is_none() {
            return reject(stats);
        }
        let (a, b) = match q.quantity {
            NodeQuantity::Degree => {
                (distinct_degree(snapshot, *node_a), distinct_degree(snapshot, *node_b))
            }
        };
        let rhs = match q.offset {
            None => b,
            Some(off) => b.saturating_add(off),
        };
        if !PropertyValue::Int(a).compare(q.op, &PropertyValue::Int(rhs)) {
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
