# Rewrite Contract Specification

This document defines the precise semantics of applying a rewrite rule to a hypergraph. Given a match binding variables to concrete nodes and edges, applying a rule performs the following operations in order.

## Contract Rule 1: Identity Preservation

Any variable that appears in both `LHS.nodes` and `RHS.nodes` maps to the **same** `NodeId`. That node survives the rewrite. Its label and properties are unchanged unless the RHS explicitly specifies new values for that variable.

```toml
# Example: variable 'a' survives
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]

[rules.rhs]
nodes = ["a", "c"]        # 'a' appears in both → same NodeId
edges = [["a", "c"]]
```

## Contract Rule 2: Node Deletion

Any variable in `LHS.nodes` that does **not** appear in `RHS.nodes` is deleted from the store. All incident edges (whether in the match or not) are also deleted — see Contract Rule 5.

```toml
# Example: variable 'b' is deleted
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]

[rules.rhs]
nodes = ["a"]             # 'b' not in RHS → deleted
edges = []
```

## Contract Rule 3: Node Creation

Any variable in `RHS.nodes` that does **not** appear in `LHS.nodes` creates a fresh node with a new `NodeId`. The new node receives:
- Label from `rhs.node_labels` (default `0`)
- Generation = current simulation step
- Empty properties map
- Energy = `0.0`

**Surviving-node relabel** (2026-07-08 correction): a variable that
appears in BOTH `LHS.nodes` and `RHS.nodes` keeps its `NodeId`, and if
`rhs.node_labels` names it explicitly, the node is **relabelled** to
that value when the rule fires — the conservative mark-transfer
mechanism the glider uses (`a = 1 -> 0`, `c = 0 -> 1`, no create/delete).
A surviving variable with NO explicit RHS label keeps its current label;
the `default 0` above applies only to created nodes. (Before this
correction the relabel path was silently missing: RHS labels on
surviving vars were ignored for every rule.)

```toml
# Example: variable 'c' is newly created
[rules.lhs]
nodes = ["a"]
edges = []

[rules.rhs]
nodes = ["a", "c"]        # 'c' is new
edges = [["a", "c"]]

[rules.rhs.node_labels]
c = 5                     # new node gets label 5
```

## Contract Rule 4: Edge Semantics

**All** edges in `LHS.edges` are deleted (even if they would appear unchanged in the RHS). **All** edges in `RHS.edges` are created fresh. There is no "edge identity preservation" — edges are always recreated.

```toml
# Even though the edge ["a","b"] appears structurally similar,
# it is deleted and recreated.
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]

[rules.rhs]
nodes = ["a", "b"]
edges = [["a", "b"]]      # deleted then recreated (new EdgeId)
```

**Edge labels**: RHS edges receive label from `rhs.edge_labels` (if specified) or default `0`. LHS edge labels are not matched against.

## Contract Rule 5: Dangling Edge Cleanup

If a deleted node (Contract Rule 2) has incident edges that were **not** in `LHS.edges` (i.e., the match did not include them), those edges are also deleted. This is automatic and silent. It is **NOT an error**.

```toml
# Graph has edge ["a", "x"] where "x" is not in the match
# Rule deletes "b" which has edge ["b", "y"] not in LHS.edges
[rules.lhs]
nodes = ["a", "b"]
edges = [["a", "b"]]      # match only includes edge a-b

[rules.rhs]
nodes = ["a"]             # b is deleted
edges = []

# Result: edge ["b", "y"] is also deleted (dangling cleanup)
# Edge ["a", "x"] is NOT deleted (a survives)
```

This is the "dangling edge" policy: deletion cascades to edges that become incident to non-existent nodes.

## Contract Rule 5.5: Property Effects

Between RHS-node resolution and RHS-edge creation, the rule's
`property_effects` run (sequential, declaration order, reads see earlier
writes in the same rewrite — docs/rule_grammar.md). All RHS variables are
resolvable at this point, including freshly created nodes. Effects touch
ONLY the properties of RHS-bound nodes, so the conflict-set definition
(v4 §8.2) is unaffected: effect targets are already bound nodes of the
binding. A runtime mismatch skips that one effect (counted in
`StepStats.property_effects_skipped`) — the rewrite always completes.

## Contract Rule 6: Atomicity

A single rule application is atomic — the entire deletion + creation happens as one unit before any observer is notified and before any subsequent match sees the intermediate state.

- All deletions (nodes + edges) are computed first
- All creations (nodes + edges) are computed second
- The store transitions from old state to new state in one step
- Observers receive the new state only

## Rewrite Semantics Are Purely Local

A rule can only delete what it matched. It cannot reach outside its LHS region to inspect or modify the rest of the graph. Guards filter on node properties but cannot inspect non-local structure at match time.

## Conflict Resolution (Engine Level)

When multiple matches (bindings) are found in a single step:
1. Bindings are sorted by canonical key: `(Reverse(priority), rule_id,
   sorted_node_ids)` — higher `priority` wins ties first (v4 §5, §8.2),
   then the canonical node ordering makes the rest deterministic.
2. Greedy acceptance: iterate in canonical order, accept a binding if its
   conflict set is disjoint from every already-accepted binding's.
3. This makes MaxParallel deterministic given the canonical sort.

### Conflict sets, defined explicitly (v4 §8.2)

Two bindings conflict if the union of their **conflict sets** intersects,
where a binding's conflict set is:

```text
conflict_set(binding) =
    bound_nodes(binding)
  ∪ matched_edges(binding)
  ∪ { edges incident to any LHS-only (deleted) node in bound_nodes(binding),
      reachable via dangling-edge cleanup }
```

The implementation checks only `bound_nodes` overlap. This is **provably
sufficient** under the current rule system: any edge a binding's pattern
depends on has all its endpoints as bound variables of that binding (a
compiled `MatchPattern` cannot reference an edge without binding both ends),
so any node whose deletion could remove an edge another binding depends on
must already be a bound node of that other binding — caught by the
shared-bound-node check. The full definition is stated here as a
documented, testable invariant so it is re-checked if a future extension
(e.g. `property_effects` reaching beyond a rule's bound variables) changes
those constraints.

**Required test (Phase 5)** — with one precision found during
implementation: two bindings deleting *adjacent* nodes can share the
connecting edge in both dangling-cleanup sets, so raw
`conflict_set ∩ conflict_set` is not always empty even with disjoint bound
nodes. That overlap is order-independent (edge removal is idempotent) and
harmless. The invariant the proof actually establishes, and the one the
test checks directly, is: for any two bindings with disjoint `bound_nodes`,
one binding's **effects** (deleted nodes, matched edges, cleanup edges)
never intersect the other's **dependencies** (bound nodes, matched edges).
A future extension that could break this is caught by the test
(`disjoint_bound_nodes_implies_effects_cannot_touch_dependencies`).

Creation-rule bindings have an empty `bound_nodes` set (v4 §5.3); an empty
set intersects nothing, so creation firings never conflict. They are also
exempt from `max_pairings_per_step` and from the sequential modes'
single-candidate sampling — those budgets apply to ordinary rewrites only,
so a due schedule can never be starved by a busy graph.

## Summary Table

| LHS Variable | In RHS? | Action |
|---|---|---|
| Yes | Survives (same NodeId) |
| No | Deleted (with dangling edges) |
| RHS Variable | In LHS? | Action |
| Yes | Survives (same NodeId) |
| No | Created (new NodeId) |
| LHS Edge | — | Always deleted |
| RHS Edge | — | Always created fresh |