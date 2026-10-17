# selfmod v3 — Phase 0 reconnaissance

Mapping from the v2 prototype's concepts to the real engine, filled from source
(`hg-engine`, plus its dependency crates `hg-core`, `hg-rules`, `hg-observe`, which are
present in this workspace and were read directly — no API was guessed). Two of the plan's
assumptions are corrected below (store crate; effect-record coverage).

## Mapping table

| Prototype concept | Engine equivalent |
|---|---|
| Store (nodes, hyperedges, properties) | **`hg-core::HypergraphStore`** (NOT `hg-schema` — that crate holds experiment schema types: gate/leakage/metrics/regime). SlotMap generational `NodeId`/`EdgeId`; `NodeData { label: Label(u32), generation: u64, properties: SmallVec<(u32, PropertyValue)>, energy: f32 }`; `HyperEdge { label, nodes: SmallVec<[NodeId;8]> }`. Property API: `NodeData::get_property/set_property`. Incidence: `edges_incident_to`; label index: `edges_with_label`. |
| `Rule` struct | **`hg-rules::Rule`** (raw, all-pub, directly constructible in code — no TOML round-trip needed) compiled via **`hg_rules::compile_rule` → `CompiledRule`** (`MatchPattern` + `RewriteTemplate`). LHS: `PatternGraph { nodes: Vec<String> var names, edges: Vec<Vec<String>>, node_labels, edge_labels: BTreeMap<usize, Label> (sparse) }`. Deletion = LHS var absent from RHS. Relabel = survivor with explicit RHS label (`rhs_relabels`). |
| VF2 matcher entry point; injectivity | **`hg_engine::find_bindings(&GraphSnapshot, &CompiledRule, rule_id) -> Vec<Binding>`**. **Non-injective by default**; per-pair distinctness via `vars_distinct` guards (`VarGuard`/`distinct_pairs`). Pattern **edges** are injective (each LHS edge consumes a distinct graph edge). Hyperedge endpoints match as an **unordered** set (permutation unification); arity must equal exactly. Disconnected patterns supported (isolated vars enumerate all nodes — expensive). |
| MaxParallel conflict set + ordering | **`hg_engine::resolve_conflicts(&[Binding]) -> Vec<Binding>`**, greedy over `Binding::canonical_key()` = `(Reverse(priority), rule_id, sorted bound-node ffi keys)`; accepts iff no bound node already claimed. `hg_engine::conflict_set` gives the full documented set (checkable invariant). `Engine::step` additionally truncates with `max_pairings_per_step`; we run our own loop and apply `match_cap_per_rule` truncation in canonical order instead. |
| `RewriteEvent` / effect record | **`hg_observe::RewriteEvent { rule_id, rule_name, binding, nodes_created, nodes_deleted, edges_created, edges_deleted, cleanup_neighbors }`**, returned by `hg_engine::apply_binding(store, rule, binding, rng) -> (RewriteEvent, fx_applied, fx_skipped)`. **Gap: relabels and property effects are NOT in the event** (property-effect activity appears only as counters in `StepStats`). Workaround for effect-gated reward (§7): structural change = any of the four event vecs non-empty; for relabel-only rules, capture the bound survivors' labels before apply and compare after. Cheap (≤4 nodes per binding) and exact. |
| Canonical hash; WL classes | **`hg-core`** (re-exported): `HypergraphStore::canonical_hash()` (content-sorted node indices + WL-style refinement tie-break; note it includes `generation` and the `energy` f32 field). WL classes: **`compute_wl_class_map(&store, radius) -> AHashMap<NodeId, WlHash>`** — label + energy field + structure only; the **properties map is deliberately excluded** from WL identity (Q2 invariant). M_data = cumulative distinct `WlHash` values. |
| `RuleSelectionPolicy` seam | `hg_observe::RuleSelectionPolicy` (`select(&snapshot, &[Binding]) -> Vec<usize>`), sits between collection and conflict resolution in `Engine::step`. Reference only; not used by v3 (we run our own loop). |
| RNG plumbing | `ChaCha8Rng::seed_from_u64(seed)` (`rand_chacha`), one stream per Engine. `apply_binding` draws **only** when a `jitter` property effect executes (validate-before-draw discipline). We follow the same convention: one ChaCha8 stream per run, fixed draw order. |

## Semantics notes (plan §2 a–c)

**(a) Alias/deletion under non-injective matching.** Apply order (documented in
`apply_binding`): (1) delete matched edges → (2) delete LHS-deleted nodes (dangling-edge
cleanup, surviving endpoints recorded as `cleanup_neighbors`) → (3) create RHS nodes /
relabel survivors → (4) property effects → (5) create RHS edges. If vars i and j bind the
same node, i deleted, j relabelled: the node is deleted at (2); the relabel at (3) is a
silent no-op (`get_node_mut` → None); RHS edges referencing the dead node are skipped
(endpoint existence checked). **Defined, non-panicking — the decoder may emit such rules.**
Every mismatch path skips rather than aborts; the rewrite always completes (atomicity).

**(b) Guards.** `NodePropertyGuard { var, property key, op, value }` with
`PropertyOp ∈ {Eq, Ne, Lt, Gt, Lte, Gte}` over `PropertyValue::{Int, Float, Bool}` —
a superset of the prototype's {Gt, Lt, Eq}. Variant mismatch or missing property fails the
guard (no coercion, no panic, no RNG). Also available: property-to-property compares with
optional offset (`NodePropertyCompare`) — not needed for v3 parity but usable later.

**(c) Max hyperedge arity.** No hard cap; `SmallVec<[NodeId; 8]>` inline capacity 8.
Matcher requires exact arity equality between pattern edge and graph edge. v3 cap
`MAX_ARITY = 3` is comfortably inside the inline capacity.

## Additional recon findings

- **Ordered vs unordered edges.** The plan's §4 asks for *ordered* PEdge endpoint refs.
  The engine matches hyperedge endpoints as an **unordered multiset** (all permutations
  unified), while RHS edges store endpoints in declared order. So the encoding keeps
  ordered refs (faithful to RHS construction), but LHS matching semantics are unordered —
  recorded here so nobody expects directional matching.
- **Determinism caveat.** `canonical_key` sorts by slotmap ffi keys (allocation order).
  Deterministic for identical construction sequences (what our seeds guarantee), not
  content-canonical. G1 therefore also requires the direct and encoded paths to present
  rules in the same `rule_id` order — the decoder's canonical enumeration (class-hash
  order) must be applied to the direct path's rule Vec too.
- **Creation rules** (`schedule`, empty LHS) exist but are not used: abiogenesis and
  fission install rule-subgraphs via economy code, outside the object step.
- **World generators** exist in `hg-core::generators`; we seed our own world (labels 0–3,
  energy property) for continuity with v2, using the store API directly.
- **Engine feature note:** `hg-engine` default feature `parallel` (rayon) only affects
  candidate collection order pre-sort; we call `find_bindings` per rule ourselves in rule
  order, so our loop is deterministic regardless.
- **PropEffects (§4):** the engine's effect grammar is rich and cheap to expose, but it is
  **deferred to v3.1** as the plan permits — v3's mutation operators do not emit
  `property_effects`, keeping the operator set and G0 fixtures at parity with the plan.

## Exit criterion check

Every section's engine-native reference resolves: store/hash/WL (`hg-core`), rule
construction + compilation (`hg-rules`), matching/conflict/apply (`hg-engine`), events &
policy (`hg-observe`). No MISSING rows; the one true gap (relabels absent from
`RewriteEvent`) has the workaround above, chosen before implementation.
