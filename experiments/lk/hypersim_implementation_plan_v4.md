# Hypersim Implementation Plan v4

## The canonical, single-source-of-truth specification

This document is complete and self-contained. It supersedes
`hypersim_implementation_plan_v2.md`, `hypersim_extensibility_addendum.md`,
and `hypersim_implementation_plan_v3.md` in full — nothing further needs to
be cross-referenced from those documents to build Hypersim. Where prior
work exists against an earlier draft of this design, that is addressed once,
plainly, in §14 — not threaded through every section as ongoing correction
narrative.

---

## 1. Vision

Hypersim is a research tool for defining, running, observing, and
understanding arbitrary hypergraph rewriting systems, integrating a
rewriting engine, a scientific evaluator (regime classification, gate
enforcement), and a causal language layer (attributes, tasks, constructors,
a certainty-tagged knowledge graph) into one GUI workbench with a headless
CLI counterpart. Every run produces exactly one CSV and one Markdown
report. Same configuration and seed always produce the same result,
regardless of thread count. No ALife layer, GPU acceleration, or automatic
controller nudging exists until the substrate-only pipeline is proven
clean end to end.

---

## 2. Crate architecture

### 2.1 The four foundational, pure-data-or-vocabulary crates

**`hg-core`** — the hypergraph itself. `NodeId`/`EdgeId` (generational,
via `slotmap`), `HypergraphStore`, `GraphSnapshot` (read-only borrowed
view), graph generators (crystal, crystal-with-defects, white noise,
minimal seed), the deterministic WL neighbourhood hash, and `PropertyValue`
(§3). No internal dependencies.

**`hg-schema`** — pure, serializable data only, no logic, no traits with
open-ended future shape. Owns `Regime` (11 variants, §7), `GateStatus`
(`Pass`/`Fail`/`Pending`), `MetricsRow` (the CSV row shape), and
`RunSummary` (the report content shape — using `Regime`/`GateStatus`
directly, since both already live here). Depends on `serde` only.

**`hg-observe`** — the engine's extension-point vocabulary: `Observer`
trait, `RewriteEvent`, `ObserverChain`, `Binding` (§8), and
`RuleSelectionPolicy` (§3 of the extension seams, below). Depends on
`hg-core` (for `GraphSnapshot`, `NodeId`) and `hg-schema`. Has **no**
dependency on `hg-engine`'s matcher, conflict, or apply logic — those stay
in `hg-engine`, which depends on `hg-observe`, never the reverse.

**`hg-app-api`** — GUI/CLI-facing plumbing: `SimCommand`, the lossy
`SimEvent` and durable `SimStatusEvent` (§9), the `SimController` trait
(shape finalized once `hg-sim-core`/`hg-gui` are actually built — this
crate exists precisely so that trait's inevitable churn never touches
`hg-schema`'s stable data types), and every read-only GUI-facing DTO:
`CompactGraphSnapshot`, `MetricsHistory`, `EvalState`, `KnowledgeGraphView`
(a flattened, display-only projection of `hg-causal`'s much richer internal
`Task`/`ConstructorRecord`/`Claim` types — `TaskLatticeRow`,
`ConstructorRow`, `ClaimRow`). Depends on `hg-core` (for `NodeId`/`EdgeId`
directly in `CompactGraphSnapshot` — using the real engine identifiers,
not a decoupled GUI-only id scheme, so a canvas click can look up the exact
same node in the causal inspector with no translation layer) and
`hg-schema`. This is the crate `hg-gui`'s Phase 8 mock-data shell depends
on — nothing more.

Why three crates (`hg-schema`, `hg-observe`, `hg-app-api`) rather than one:
each has a genuinely different stability profile and a genuinely different
set of consumers. `hg-schema`'s types are essentially frozen once defined.
`hg-observe`'s traits are engine-facing and will only change alongside the
engine itself. `hg-app-api`'s controller trait and DTOs will iterate
throughout `hg-sim-core`/`hg-gui` development. Merging any two of these
would mean unrelated churn propagating to crates that don't need it.

### 2.2 Why `hg-output` has no internal dependencies — stated directly

`hg-output` (CSV writing, the manifest, run-state transitions, snapshot and
append-log binary formats) depends on **nothing** internal — not
`hg-schema`, not `hg-core`. This is deliberate, not an oversight the
dependency graph failed to capture:

- `CsvWriter<T>` is generic over any `T: Serialize`. It never needs to
  know `MetricsRow` exists — the caller (`hg-sim-core`) supplies whatever
  row type it wants written, and the `csv` crate's own serde integration
  derives the header from that type's field names. `hg-output` writes
  files correctly; it has no opinion on what's inside them.
- `RunState` and `RunManifest` are defined *inside* `hg-output` itself —
  they are not imported from anywhere, and nothing above `hg-output` needs
  to inject a type for them to use.
- `SnapshotWriter`/`AppendLogWriter` accept pre-serialized `&[u8]`. They
  never touch `HypergraphStore`, `NodeId`, or any `hg-core` type — the
  caller serializes whatever a "snapshot" conceptually is before handing
  over the bytes.

This is the same pattern applied consistently: a low-level infrastructure
crate stays generic; the orchestrating crate (`hg-sim-core`) supplies
concrete, typed values in the infrastructure crate's plain vocabulary.

### 2.3 Full dependency graph

```
hg-core        (no deps)
hg-schema      (serde only)
hg-rules       (hg-core)
hg-observe     (hg-core, hg-schema)
hg-parallel    (hg-core, hg-rules)
hg-engine      (hg-core, hg-rules, hg-observe, hg-parallel[feature])
hg-output      (no internal deps — §2.2)
hg-eval        (hg-core, hg-observe, hg-schema)
hg-causal      (hg-core, hg-observe, hg-schema)
hg-report      (hg-schema only)
hg-app-api     (hg-core, hg-schema)
hg-sim-core    (everything above — the ONLY crate that imports hg-engine
                directly)
hg-gui         (hg-app-api always; hg-sim-core only behind a "live" Cargo
                feature — Phase 8's mock shell needs nothing else)

Reserved, not yet built:
hg-alife       (hg-core, hg-observe, hg-causal)
hg-cognition   (hg-core, hg-observe, hg-causal)
```

"Only `hg-sim-core` imports `hg-engine` directly" is now actually true —
every crate that needs to plug into the step loop does so through
`hg-observe`'s traits.

---

## 3. Node properties: `PropertyValue`

Adopted directly, not deferred. `NodeData.properties` becomes
`SmallVec<[(u32, PropertyValue); 4]>`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    Bool(bool),
}
```

Three variants, not four — a single signed-integer type covers every
realistic count or flag this project needs (up to ~9.2 quintillion),
removing the ambiguous "should a bare `100` be `UInt` or `Int`" dispatch
question a fourth variant would introduce.

**Why now, not deferred:** energy is explicitly a first-class GUI color
dimension (color mode `E`, blue→red by energy proxy) — the renderer needs
to know a property is genuinely a float to gradient it correctly, not
infer that from convention. This is a concrete, already-designed
requirement, not a speculative future one.

**TOML ergonomics:** both a guard's threshold and a property-write's value
are written as an ordinary TOML literal — `value = 100` (int syntax),
`value = 0.8` (float syntax), `value = true` (bool) — and deserialize into
`PropertyValue` via `#[serde(untagged)]`, trying each variant in turn.
TOML's own lexer already distinguishes int/float/bool syntactically, so no
explicit type-tagging wrapper is needed in rule files.

**Comparison semantics:** `Guard::NodePropertyOp` compares a stored
`PropertyValue` against a threshold `PropertyValue`. If the variants match,
compare normally. If they don't, the guard fails (returns `false`) —
simple, safe, no coercion, no panic. `docs/property_registry.md` (§4)
remains a plain documentation table of which key conventionally holds
which variant, for human authors; it is not loaded or enforced by the
parser as a formal schema — that would be meaningful additional machinery
for a single-developer project that doesn't need it yet.

---

## 4. Phase 0 specifications (nine documents)

```
docs/rule_grammar.md              (§5, §6 — includes creation-rule schedule)
docs/rewrite_contract.md          (§8 — explicit conflict-set definition)
docs/reproducibility_policy.md    (§10 — includes the canonical_hash algorithm)
docs/run_state_machine.md         (RunState transitions, unchanged in substance)
docs/schema_versions.md           (CSV/binary format headers, unchanged)
docs/fixture_definitions.md       (§7 — Regime terms corrected)
docs/property_registry.md         (§3 — plain documentation table)
docs/metric_formulas.md           (PI proxy, EI formula, interestingness
                                    weighted sum and penalties, type
                                    histogram entropy/JSD, treadmill vs.
                                    generative threshold conditions —
                                    every numeric constant a fixture
                                    references must trace to a formula here)
docs/gate_formulas.md             (one entry per gate: exact pass/fail
                                    predicate, fields read, fixtures that
                                    exercise it)
```

---

## 5. Rule grammar

### 5.1 Node and edge patterns

```toml
[rules.lhs]
nodes = ["a", "b", "c"]
edges = [["a", "b", "c"]]

[rules.lhs.node_labels]
a = 1                    # sparse map keyed by variable name

[rules.lhs.edge_labels]
0 = 0                    # sparse map keyed by EDGE INDEX (into lhs.edges)
```

**Edge labels are sparse maps keyed by index, not parallel arrays.** TOML
has no per-element null — a `Vec<Option<u32>>` deserialized from a plain
array cannot express "constrain edge 0 but leave edge 1 unconstrained,"
since there's no way to place a null placeholder inside a homogeneously-
typed TOML array. This bug existed already in the RHS grammar as originally
specified (`rhs.edge_labels: Vec<Option<u32>>`) — it happened to go
unexercised because the one test written for it only ever populated a
single-edge case. The fix, applied to *both* LHS and RHS, mirrors how
`node_labels` was already correctly designed: a sparse map (TOML table)
keyed by index (LHS: edge index; the existing `node_labels` convention:
variable name), where an absent key means "unconstrained" — no null
sentinel needed, because absence of a key already means exactly that.

```toml
[rules.rhs]
nodes = ["a", "b", "c", "d"]
edges = [["a","b","d"], ["b","c","d"], ["a","c","d"]]

[rules.rhs.edge_labels]
0 = 5                     # edge index 0 gets label 5; edges 1, 2 default
```

### 5.2 Property guards and effects (typed via `PropertyValue`, §3)

```toml
[[rules.lhs.node_property_guards]]
var = "a"
property = 1
op = "gt"
value = 100.0             # Float — TOML's own float syntax selects the variant

[[rules.rhs.property_effects]]
var = "child"
key = 1
op = "scale_from"
source_var = "parent"
source_key = 1
factor = 0.8              # plain f64 — no PropertyValue wrapping; requires
                          # the source property to be Float, producing Float

[[rules.rhs.property_effects]]
var = "parent"
key = 1
op = "subtract"
value = 100
```

`property_effects` is a deferred, sketched design (§ Extension Seams) — not
implemented until `hg-alife` needs it — but its TOML shape and typing story
are fully specified here so no future redesign is needed.

### 5.3 Creation rules — fully specified, not deferred

A rule with empty `lhs.edges` (and empty `lhs.nodes`) is a creation rule
and **must** carry a `[rules.schedule]` section; a rule with a non-empty
pattern **must not**. Both directions are validated at parse time.

```toml
[rules.schedule]
mode = "once"             # fires once, at step 0
# or:
mode = "every_n_steps"
n = 1000
```

At the start of `Engine::step()`, before ordinary pattern matching, every
enabled creation rule's schedule is checked against the current step
number; a due rule contributes a trivial binding with zero bound nodes
(`Binding::new(rule.id, rule.priority, SmallVec::new())`) into the same
candidate pool ordinary matches populate. An empty node set can never
conflict with anything (the shared-node check is vacuously false against
an empty set), so creation-rule firings are always accepted alongside
whatever else fires that step.

---

## 6. Extension seams (ALife, hypergraph-neural-net policies)

Carried forward from the addendum with dependency edges corrected per §2.

### 6.1 `RuleSelectionPolicy` — in `hg-observe`

```rust
pub trait RuleSelectionPolicy: Send + 'static {
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize>;
}
```

A policy-guided step still runs through `hg-engine`'s existing conflict
resolver — the seam sits between candidate-collection and conflict-
resolution, not in place of either, so a policy can select a suboptimal or
overlapping candidate set but can never violate the rewrite contract's
node-disjointness guarantee. `hg-cognition` implementing this trait never
needs `hg-engine` as a dependency, only `hg-observe` and `hg-core`.

### 6.2 `MotifClassifier` — in `hg-causal`

Unchanged from the addendum: defined in `hg-causal`, implemented by
`hg-alife`. `NullMotifClassifier` is the default for substrate-only runs.

### 6.3 Property writes — `property_effects` (§5.2)

Operations: `set` (literal `PropertyValue`), `copy_from` (var, key),
`scale_from` (var, key, `factor: f64` — source must be `Float`), `add`/
`subtract` (literal delta — source must be `Int` or `Float`). Applied by
`hg-engine::apply` between RHS node creation and RHS edge creation, since
an effect's `source_var` may itself be freshly created.

### 6.4 General conventions (unchanged from the addendum)

Config schema stays additive (`Option<T>`/`#[serde(default)]` for every new
section). Optional layers (`hg-alife`, `hg-cognition`) are Cargo features on
`hg-sim-core`/`hg-gui`, mirroring `hg-engine`'s existing `parallel` feature.
`Predicate` in `hg-causal` stays a closed, exhaustively-matched enum — no
`Custom` escape hatch — because this is a single-developer codebase where
adding a real variant costs one small edit, and trading away exhaustiveness
for flexibility nobody needs would be the wrong trade. GUI dashboard tabs
are driven by a small internal registry, not hardcoded panel wiring.

---

## 7. Regime: 11 variants

```rust
pub enum Regime {
    Frozen, StaticOrdered, GrowthTreadmill, WhiteNoiseSoup, SterileChaos,
    Exploding, FragmentedIsolation, Monoculture,
    ActivePersistentStructure,   // new — see below
    GenerativeCandidate, Unknown,
}
```

Two terms from earlier fixture wording (`ActivePersistentStructure`,
`FrozenStructured`) were evaluated on their merits, not mechanically
reconciled. `FrozenStructured` is dropped — a static, ordered, unchanging
structure is already fully captured by the existing `StaticOrdered`/
`Frozen` pair, and the term added no distinct classification.
`ActivePersistentStructure` is kept as a genuine 11th category: a lone
moving glider does not necessarily satisfy `GenerativeCandidate`'s own
requirement (`new_type_bin_rate > new_type_bin_rate_min`) — it moves, but
doesn't necessarily invent new structure types, which is a meaningfully
weaker, distinct claim worth its own regime. Classifier priority places it
immediately before `GenerativeCandidate`.

`MovingGliderToy`'s fixture expectation: `ActivePersistentStructure` or
`GenerativeCandidate` (either satisfies it — a glider that also introduces
novelty clears the higher bar too). `StaticBlockToy`'s expectation:
`StaticOrdered`.

---

## 8. Rewrite contract: explicit conflict sets

### 8.1 Locality, precisely

Locality of **matching** is absolute — a rule's LHS can only inspect nodes
and edges within its own pattern. Locality of **effect** is bounded but not
absolute: deleting a matched node can have a well-defined, non-local
*consequence* (removing that node's other incident edges, per dangling-
edge cleanup) — a deliberate, documented exception to locality-of-effect,
not a violation of locality-of-matching.

### 8.2 Conflict sets, defined explicitly

Two bindings conflict if the union of their **conflict sets** intersects,
where a binding's conflict set is:

```text
conflict_set(binding) =
    bound_nodes(binding)
  ∪ matched_edges(binding)
  ∪ { edges incident to any node in bound_nodes(binding) that is
      LHS-only (deleted by this binding), reachable via dangling-edge
      cleanup }
```

The current implementation checks only `bound_nodes` overlap. This is
*provably sufficient* given the rule system's present constraints — proven
below — but the full definition is stated explicitly here, as a documented,
testable invariant, precisely so it gets re-checked if a future extension
(e.g. `property_effects` referencing state beyond a rule's own bound
variables) ever changes those constraints.

**Proof of current sufficiency:** any edge a binding's pattern depends on
necessarily has *all* its endpoints as bound variables of that binding —
edges are defined purely in terms of bound-variable endpoints in the
compiled `MatchPattern`, with no mechanism for a pattern to reference an
edge without binding both its ends. Therefore any node whose deletion (via
dangling cleanup) could remove an edge a *different* binding depends on
must already be a bound node of that other binding too — which the
existing shared-bound-node check already catches. A concrete adversarial
scenario (binding A deletes node X, incidentally removing edge X–Z via
cleanup, while binding B independently relabels node Z without X ever
being one of B's bound variables) was traced through explicitly: the final
graph state is identical regardless of application order, because B's
effect (touching only Z's own data) and A's effect (touching X and X's
incident edges) operate on disjoint parts of the graph that never interact.
No counterexample was found against the current rule system's actual
capabilities.

**Required test, added to the Phase 5 (`hg-engine`) test suite:** for any
two bindings with disjoint `bound_nodes` sets, assert their full
`conflict_set`s (as defined above) are *also* disjoint — an invariant
that's currently always true, checked directly rather than left implicit,
so a future rule-system extension that could break it is caught
immediately rather than discovered as a live bug.

---

## 9. Concurrency

### 9.1 `SimEvent` (lossy) vs. `SimStatusEvent` (durable)

```rust
// hg-app-api — lossy, bounded(256), try_send/drop-on-full. Shared
// Arc<RwLock<>> state is authoritative; dropping any of these is harmless.
pub enum SimEvent {
    WindowComplete { window: u64, metrics: Box<MetricsRow> },
    RegimeChanged { window: u64, old: Regime, new: Regime },
    GateUpdated { gate: GateName, status: GateStatus },
    ConstructorConfirmed { hash: WlHash, fidelity: f32, is_replicator: bool },
}

// hg-app-api — durable, unbounded, at-most-a-handful-per-run by construction.
pub enum SimStatusEvent {
    ErrorOccurred { message: String },
    RunCompleted { summary: Box<RunSummary> },
    ScienceUnlocked,
    ScienceLocked { reason: String },
}
```

**Send failures must not panic, and must not assume a listener exists.**
An unbounded channel's `send()` only fails when the receiver has been
dropped or was never created — exactly the headless-CLI case, where there
may be no GUI consuming `SimStatusEvent` at all. Every send site uses
`let _ = status_tx.send(...)`, never `.unwrap()` — a disconnected receiver
is a normal, expected condition, not a bug.

**Status must also be durably recorded independent of channel delivery.**
Relying solely on an event that might have no listener is insufficient for
information as important as "did this run complete." Every status
transition is *also* committed through `RunManifest`'s own transition
methods (`start`/`pause`/`resume`/`seal_completed`/`seal_interrupted` —
already built in `hg-output`) and saved to `manifest.json`, which is
therefore the authoritative, disk-persisted record of run status,
independent of whether any live consumer ever received the corresponding
event. A re-attached GUI, or any post-hoc tool, reads the manifest, not the
(necessarily transient) event stream.

### 9.2 `SimProgress` — atomics, not `RwLock`

```rust
pub struct SimProgress {
    step: AtomicU64,
    max_steps: u64,
    run_state: AtomicU8,     // RunState as a small integer discriminant
    sim_hz_bits: AtomicU32,  // f32 via to_bits()/from_bits()
}
```

`Ordering::Relaxed` throughout, deliberately, and only here — `SimProgress`
is purely advisory display data; a one-store-stale read is fine. This is
explicitly *not* extended to `MetricsHistory`/`EvalState`/the knowledge
graph's shared state, which keep their stronger `RwLock`-based consistency
because that data is scientifically load-bearing, not advisory. Two
different consistency requirements, two different mechanisms — not
unified for consistency's own sake.

---

## 10. Reproducibility policy

**Corrected wording:** same `SimConfig` + same seed ⟹ identical final
graph `canonical_hash` (exact, always); CSV data rows **cell-wise
identical with numeric tolerance**, after excluding the schema-version
comment row's `run_id`/`created=` fields (which vary by design, not by
flaw); report content identical after excluding the Manifest section's
`run_id`/timestamps. "Byte-for-byte within tolerance" was a contradiction
in terms — byte-for-byte means exact, "within tolerance" means
approximate; they cannot both hold at once and the wording above resolves
which applies where.

**The canonical hash algorithm, specified here (not only in code):**

```text
1. Sort all live nodes by (label, generation, sorted-properties) to
   obtain a canonical node ordering, independent of NodeId/slotmap
   allocation order. Assign each node a canonical index 0, 1, 2, ...
   under this ordering.
2. Remap every edge's node list through this canonical index table,
   then sort each remapped edge's index list ascending (edges are
   unordered sets of endpoints for hashing purposes).
3. Sort the full list of remapped, sorted edges lexicographically.
4. Hash the canonical node sequence, then the canonical edge sequence,
   using FxHasher (deterministic, no per-process random seed).
```

Two stores built from the same logical graph via different insertion
orders produce the same canonical node→index mapping and therefore the
same hash — this is what makes the hash meaningful across independently
constructed but logically identical graphs, not merely stable for a single
construction path.

**Testing implication:** Integration Test E (cross-thread-count
determinism) compares CSV files by skipping the metadata comment line
before diffing, or runs in a dedicated test mode with an injected fixed
`run_id` and timestamp, making whole-file byte comparison valid again
specifically for test tooling.

---

## 11. Report structure — 11 canonical sections

```
1.  Manifest                    (provenance: run_id, timestamps, version,
                                  thread count, state — from RunManifest)
2.  Run Parameters               (the experimental configuration itself:
                                  rule set, initial conditions, evaluator
                                  thresholds, causal-language settings —
                                  distinct from Manifest's bookkeeping;
                                  reproducing a run needs both)
3.  Campaign Summary             (if applicable)
4.  Gate Status                  (subsection: Fixture Truth Table)
5.  Regime Summary
6.  Evaluator Details            (subsections: PI Normalization, Type
                                  Histogram Dynamics, Static vs Active
                                  Persistence, Novelty Archive Stats)
7.  Causal Language Summary      (subsection: Constructor Ecology)
8.  Final Metrics Table
9.  Suspicious Metric Flags
10. Observations
11. Recommended Next Actions
```

`Run Parameters` was dropped without justification in an earlier
regrouping pass and is restored here as its own section — config
visibility is a reproducibility requirement, not an optional nicety.

---

## 12. Technology stack

`tracing` (not `log`+`env_logger`) — a genuinely better fit for a
multi-threaded architecture (sim thread, layout thread, GUI thread), since
span-based context lets a log line be attributed to the thread/phase it
came from. `tempfile`, `proptest`, and `trybuild` are workspace-shared dev-
dependencies. Everything else (`slotmap`+serde, `smallvec`+serde, `serde`,
`serde_json`, `toml`, `bincode`, `rayon`, `crossbeam-channel`, `rand`+
`rand_chacha`, `petgraph`, `nalgebra`, `ascent`, `clap`, `csv`, `chrono`,
`eframe`/`egui`/`egui_plot`/`rfd`/`syntect`, `thiserror`, `criterion`) as
established. `proptest` is well-justified given how much of this project's
testing rests on hand-verified properties (canonical-hash stability across
construction order, WL-hash panic-freedom) that generalize naturally to
property-based testing across randomly generated graphs, not just the
handful of hand-written examples.

---

## 13. Build sequence

```
Phase 0.  Write all nine Phase 0 documents (§4). No code.

Phase 1.  hg-core: NodeId/EdgeId/Label, HypergraphStore, GraphSnapshot,
          PropertyValue (§3), WL hash, generators.
          Exit: canonical_hash stable across construction order; WL hash
          isomorphism-invariant on hand-crafted pairs; all generators
          reproducible under a fixed seed.

Phase 2.  hg-schema: Regime (11 variants), GateStatus, MetricsRow,
          RunSummary. Pure data, no logic.
          Exit: every type round-trips through serde.

Phase 3.  hg-observe: Observer, RewriteEvent, ObserverChain, Binding
          (with priority: i32, §5), RuleSelectionPolicy.
          Exit: ObserverChain delegates to N members in push order,
          verified for all three lifecycle methods together, not each in
          isolation.

Phase 4.  hg-rules: MatchPattern, RewriteTemplate, Guard/PropertyValue-typed
          guards, the TOML parser (edge_labels as sparse maps — §5.1;
          creation-rule schedule validation — §5.3), builtins.
          Exit: fixture rule sets (triangle_subdivide, binary_edge,
          tetrahedral, glider, growth_treadmill) parse and compile
          correctly; auto-generated VarsDistinct guards verified against
          both symmetric and asymmetric patterns.

Phase 5.  hg-engine: matcher (non-injective-by-default — explicit test for
          this), conflict resolution (priority via Reverse<i32> in the
          sort key — §5; explicit conflict-set test — §8.2), apply
          (property_effects hook point present even if unimplemented),
          creation-rule firing (§5.3), the step loop.
          Exit: same seed + same config -> same canonical_hash regardless
          of thread count; MaxParallel applies all non-conflicting
          matches; disjoint-bound-nodes implies disjoint-conflict-sets,
          checked directly.

Phase 6.  hg-output: CsvWriter<T>, RunManifest (with transition methods
          durably recording status — §9.1), RunState, AppendLogWriter/
          Reader, SnapshotWriter/Reader.
          Exit: exactly one CSV per run, always; manifest.json reflects
          current status even with zero event-channel listeners.

Phase 7.  hg-eval and hg-causal (parallel — independent of each other):
          fixture truth table (all six fixtures, formulas locked per
          docs/metric_formulas.md and docs/gate_formulas.md), the full
          gate engine; WL-hash-based attributes, task lattice, constructor
          registry, EI calculator, the certainty-tagged knowledge graph.
          Exit: fixture truth table passes exactly as specified in
          docs/fixture_definitions.md; a 50k-step run produces a non-empty
          task lattice and at least one constructor candidate.

Phase 8.  hg-report (depends on hg-schema only): the 11-section report
          (§11), epistemic tagging ([Measured]/[Estimated]).
          Exit: every section present and non-empty (or explicitly says
          why not) on a real run's output.

Phase 9.  hg-app-api: SimCommand, SimEvent/SimStatusEvent (§9.1),
          SimController trait, CompactGraphSnapshot and the other GUI DTOs.
          Exit: MockSimController (built here or in hg-gui, implementing
          this trait) serves a complete fixture without any dependency
          beyond hg-app-api itself.

Phase 10. hg-sim-core: the real SimController implementation, orchestrating
          hg-engine + hg-eval + hg-causal + hg-output + hg-report behind
          the hg-app-api trait boundary.
          Exit: a live headless run produces one CSV, one report, a
          manifest with correct final state, regardless of whether
          anything was listening on the event channels.

Phase 11. hg-gui: Phase 8 (mock-data shell, hg-app-api only) before Phase
          9 (live wiring, hg-sim-core behind the "live" feature).
```

---

## 14. Relationship to prior work

Real code exists from an earlier pass at this design, covering `hg-core`
in full and partial progress on `hg-rules`, `hg-engine`, and `hg-output`.
Against this v4 specification: `hg-core`'s store, snapshot, generators, and
WL hash require no structural changes beyond adding `PropertyValue` (§3).
`hg-rules` and `hg-engine`'s already-written modules require the specific,
concrete corrections detailed in §5 (edge-label sparse maps, creation-rule
scheduling), §6.1 (moving `Binding`/`Observer` out of `hg-engine` into
`hg-observe`), and §8.2/§5 (priority in the conflict-resolution sort key).
`hg-output`'s existing modules require no changes at all — nothing in this
revision touches that crate. Where this section and the rest of the
document could ever be read as being in tension, this document's technical
content governs; this section exists only to state plainly what carries
forward unchanged versus what needs concrete correction, in one place,
rather than as a qualifier on every paragraph above.

---

*This is the authoritative specification. Build against this document.*
