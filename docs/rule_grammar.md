# Rule TOML Grammar Specification

This document defines the complete grammar for rule definitions in Hypersim. Rules are expressed in TOML format and loaded from `rules.toml` files.

## Top-Level Structure

A rules file contains one or more `[[rules]]` tables. Each table defines a single rewriting rule.

```toml
[[rules]]
name     = "string"       # required; unique within file
priority = 0              # integer; default 0; higher = preferred in conflict
weight   = 1.0            # float; default 1.0; used by SequentialWeighted
enabled  = true           # bool; default true

[rules.lhs]
nodes = ["a", "b", "c"]  # variable names; all must be unique within rule
edges = [                 # each inner array = one hyperedge, listed by variable
  ["a", "b", "c"],        # matches a 3-ary edge connecting nodes a, b, c
  ["a", "b"],             # matches a 2-ary edge connecting nodes a, b
]

# Optional: constrain node labels
[rules.lhs.node_labels]
a = 0    # node bound to var 'a' must have label 0
b = 1

# Optional: constrain node properties
[[rules.lhs.node_property_guards]]
var      = "a"
property = 1        # property key (u32)
op       = "gt"     # "eq" | "lt" | "gt" | "lte" | "gte" | "ne"
value    = 100

# Optional: compare one property against another (property-to-property).
# `var_a`/`var_b` may be the SAME variable (self-comparison across two keys,
# e.g. carrier.energy < carrier.satiation) or DIFFERENT (cross-node). Same op
# vocabulary as node_property_guards. Variant mismatch or a missing property on
# either side ⟹ guard false (no coercion, no panic); draws no RNG. (hg-alife S1)
[[rules.lhs.node_property_compares]]
var_a = "a"
key_a = 1           # property key on var_a (u32)
op    = "gte"
var_b = "b"
key_b = 11          # property key on var_b (u32)

# Optional: degree comparison guard (plan v4 §6.2). Degree = count of
# DISTINCT incident hyperedges (a node appearing twice in one edge counts
# that edge ONCE — pinned by test). `offset` (Int, saturating) reuses the
# compare-offset machinery: degree <op> (value + offset). Same op
# vocabulary. Draws no RNG. Engagement counters wired from birth (§1.1).
[[rules.lhs.node_degree_compares]]
var   = "w"
op    = "eq"
value = 2           # i64
# offset = 1        # optional

# Optional: variable inequality guards
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b"]   # a != b (prevents self-loop matches)

[rules.rhs]
nodes = ["a", "b", "c", "d"]  # may include LHS vars (surviving) + new vars
edges = [
  ["a", "b", "d"],
  ["b", "c", "d"],
  ["a", "c", "d"],
]

# Optional: set labels on new nodes
[rules.rhs.node_labels]
d = 0     # new node 'd' gets label 0; LHS node labels unchanged unless specified

# Optional: constrain LHS edge labels — SPARSE MAP keyed by edge index
# (index into lhs.edges). Absent key = that edge is unconstrained.
[rules.lhs.edge_labels]
0 = 0    # lhs.edges[0] must have label 0; edge 1 unconstrained

# Optional: set labels on new RHS edges — SPARSE MAP keyed by edge index
# (index into rhs.edges). Absent key = default label 0.
[rules.rhs.edge_labels]
0 = 5    # rhs.edges[0] gets label 5; edges 1, 2 default to 0
```

### Why edge_labels is a sparse map, not a parallel array (v4 §5.1)

TOML has no per-element null, so a `Vec<Option<u32>>` cannot express
"constrain edge 0 but leave edge 1 unconstrained" from a plain array —
there is nowhere to put a null placeholder in a homogeneously-typed TOML
array. `edge_labels` is therefore a TOML **table keyed by edge index**
(a sparse map), for both LHS and RHS, mirroring how `node_labels` was
already correctly designed. An absent key means "unconstrained" (LHS) or
"default label 0" (RHS) — no null sentinel needed, because absence of a
key already means exactly that.

## Grammar Constraints (Validated at Load Time)

1. **Variable uniqueness**: `lhs.nodes` and `rhs.nodes` are sets of variable names (strings, no duplicates per list)

2. **Edge variable coverage**: Every variable appearing in `lhs.edges` must appear in `lhs.nodes`. Every variable in `rhs.edges` must appear in `rhs.nodes`.

3. **Creation rules** (v4 §5.3): A rule with empty `lhs.edges` **and** empty
   `lhs.nodes` is a **creation rule** and **must** carry a `[rules.schedule]`
   section. A rule with a non-empty pattern **must not** carry one. Both
   directions are validated at parse time.

   ```toml
   [rules.schedule]
   mode = "once"            # fires once, at step 0
   # or:
   mode = "every_n_steps"
   n    = 1000              # fires every 1000 steps
   ```

   At the start of `Engine::step()`, before ordinary matching, every enabled
   creation rule whose schedule is due contributes a trivial binding with
   zero bound nodes into the same candidate pool as ordinary matches. An
   empty node set can never conflict (the shared-node check is vacuously
   false against an empty set), so creation firings are always accepted.

4. **Deletion rules**: A rule with an empty `rhs.nodes` is a **deletion rule** (removes matched subgraph entirely).

5. **Label types**: Node labels and edge labels are `u32` values. Default is `0`.

6. **Property guards**: The `property` key is `u32`. The `op` must be one of: `eq`, `lt`, `gt`, `lte`, `gte`, `ne`. The `value` is compared as `i64` (supports negative numbers).

7. **Variable guards**: Currently only `vars_distinct` is supported. It takes a `vars` array of variable names that must all bind to distinct NodeIds.

8. **Priority and weight**: `priority` is `i32` (default 0). Higher priority rules are preferred during conflict resolution in MaxParallel mode. `weight` is `f64` (default 1.0) used by SequentialWeighted mode.

9. **Name uniqueness**: Rule names must be unique within a single TOML file. Duplicate names are a load-time error.

## Example: Triangle Subdivision Rule

```toml
[[rules]]
name = "triangle_subdivide"
priority = 0
weight = 1.0
enabled = true

[rules.lhs]
nodes = ["a", "b", "c"]
edges = [
  ["a", "b", "c"]
]

[rules.rhs]
nodes = ["a", "b", "c", "d"]
edges = [
  ["a", "b", "d"],
  ["b", "c", "d"],
  ["a", "c", "d"]
]

[rules.rhs.node_labels]
d = 0
```

## Example: Binary Edge Rule with Guards

```toml
[[rules]]
name = "binary_edge_split"
priority = 5
weight = 2.0
enabled = true

[rules.lhs]
nodes = ["x", "y"]
edges = [
  ["x", "y"]
]

[rules.lhs.node_labels]
x = 1
y = 1

[[rules.lhs.node_property_guards]]
var = "x"
property = 42
op = "gt"
value = 10

[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["x", "y"]

[rules.rhs]
nodes = ["x", "y", "z"]
edges = [
  ["x", "z"],
  ["z", "y"]
]

[rules.rhs.node_labels]
z = 2
```

## Property Effects (`[[rules.rhs.property_effects]]`)

Property WRITES in the rewrite system (v4 §5.2/§6.3 plus `jitter`) — the
substrate primitive for the ALife node-funded replication mechanic.

```toml
[[rules.rhs.property_effects]]
var    = "child"        # target: MUST be an RHS variable name
key    = 1              # property key (u32)
op     = "scale_from"
source_var = "parent"   # MUST be an RHS variable name
source_key = 1
factor = 0.8            # f64

[[rules.rhs.property_effects]]
var   = "parent"
key   = 1
op    = "subtract"
value = 100.0           # TOML literal -> PropertyValue (untagged)

[[rules.rhs.property_effects]]
var       = "child"
key       = 2
op        = "jitter"
magnitude = 0.05        # f64, must be > 0
```

| op | fields | semantics |
|---|---|---|
| `set` | value | target[key] = value (any PropertyValue) |
| `copy_from` | source_var, source_key | target[key] = source[source_key] |
| `scale_from` | source_var, source_key, factor | target[key] = Float(source Float × factor); source must be Float |
| `add` / `subtract` | value | target[key] ± value; Int±Int (saturating) or Float±Float |
| `add_from_property` / `subtract_from_property` | source_var, source_key | target[key] ± source[source_key]; **Float only** — the amount is a PROPERTY, not a constant (genome-scaled effects, deviation #4). Skips if source/target missing or non-Float |
| `jitter` | magnitude | target[key] += uniform(−m, +m); Float continuous, Int integer delta in [−⌊m⌋, +⌊m⌋] |

**Validation (parse-time, hard errors, `deny_unknown_fields`):** `var` and
`source_var` must name RHS variables. An LHS-only (deleted) variable is
rejected as `source_var` with an explanatory error — its node is deleted at
apply step 4, before effects run, so there is nothing to read. Unknown op,
missing op-specific fields, and `magnitude <= 0` are all parse errors.
Creation rules may carry effects (schedule + `set` = seeded initial
properties).

**Sequential semantics — reads see earlier writes.** Effects apply in
declaration order, and each effect's reads observe the results of earlier
effects in the SAME rewrite (deliberate choice over snapshot semantics:
chaining is possible and ordering is under the rule author's control):

```
parent.energy = 200
1. child scale_from parent ×0.8   -> child = 160   (reads pre-subtract value)
2. parent subtract 100            -> parent = 100
```

Swapped declaration order gives child = 80 — different, and correctly so
(the scale reads the post-subtract 100).

**Failure semantics (Contract Rule 6 atomicity preserved):** a runtime
mismatch — missing source property, wrong variant (e.g. `scale_from` on an
Int), missing target key for add/subtract/jitter — skips THAT effect only,
increments `StepStats.property_effects_skipped`, and emits a rate-limited
warning. The rewrite itself always completes. Int arithmetic saturates. No
clamping: energy may go negative if the author under-guards — guards are
the author's precondition tool; the engine does arithmetic as written.

**Jitter determinism:** jitter draws from the engine's seeded stream (the
same one step-mode selection uses) in deterministic order — accepted
bindings in conflict-resolution order, effects in declaration order. Zero
draws occur unless a jitter effect actually executes, so rulesets without
jitter are stream-identical to pre-feature behavior
(docs/reproducibility_policy.md).

## Semantic Notes

- **Edge labels** may be constrained in LHS via the sparse `lhs.edge_labels`
  map (absent index = unconstrained). RHS edge labels come from the sparse
  `rhs.edge_labels` map (absent index = default 0).
- **Node properties** are `SmallVec<[(u32, PropertyValue); 4]>` on each node,
  where `PropertyValue` is `Int(i64)`/`Float(f64)`/`Bool(bool)` (v4 §3).
  Property guards compare a stored value against a threshold of the same
  variant; a variant mismatch makes the guard `false` (no coercion, no
  panic). See `docs/property_registry.md`.
- **Generation** is a `u64` field on nodes (birth step). Not directly constrainable in rules; available for observers.
- **Energy** is a `f32` field on nodes. Not directly constrainable in rules; available for observers.