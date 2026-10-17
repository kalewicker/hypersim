# Hypersim Substrate Characterization Report

**Internal characterization report — lab-notebook rigor.** This document
consolidates what the Hypersim substrate *is*, what was *measured*, and —
because the results are only as trustworthy as the instruments — how the
measurement machinery was *validated*. Its audience is the author in six
months and any collaborator who must trust the substrate before building the
ALife layer on top of it. It is written to be falsifiable: every quantitative
claim is either exact-by-construction (stated as such) or carries a slot
pointing at the run artifact it must be verified against.

Per the research program's sequencing, substrate characterization is a
standalone result, gated *before* ALife experiments. This report is that
result in internal form; it is structured so it can become an external draft
without re-architecting.

---

## 0. How to read this document

**Slot convention.** Every number sourced from a run appears as a slot:

> `{{FILL: <quantity> | session≈<provisional> | src: <artifact>}}`

The `session≈` value is the figure established during the build/verification
sessions — it is a *provisional anchor for sanity-checking*, NOT a verified
result. Before this report is trustworthy, resolve every slot: `grep '{{FILL'`
this file, pull the cited artifact, and replace the slot with the source
value. A source value that disagrees with its `session≈` anchor is itself a
finding — investigate the discrepancy before overwriting.

Slots marked `prediction-only` were never measured to a recorded value; they
carry the design prediction and need a first measurement, not a lookup.

**Exact-by-construction** claims (e.g. "non-trivial replicator count = 0")
are not slots — they follow from code structure, not from any particular run,
and are stated with their derivation.

**Prediction / Result / Resolution** blocks mark places where a pre-committed
prediction met reality and was revised. These are the program's core
epistemic events and are surfaced, not buried.

---

## 1. The substrate (spine)

### 1.1 Object of study

Hypersim is a hypergraph-rewriting system: a hypergraph evolves by repeatedly
matching local patterns (subhypergraphs) and replacing them per a rule set.
The substrate is the physics layer — engine, rules, graph — with no ALife,
GPU, or controller machinery. It is a multi-crate Rust workspace; the physics
layer is `hg-core` (graph, ids, WL hash, generators, `PropertyValue`),
`hg-rules` (pattern/template grammar, parser, builtins), and `hg-engine`
(matcher, conflict resolution, apply, step loop), with the analytical layers
(`hg-observe`, `hg-eval`, `hg-causal`) reading engine events without
influencing them.

### 1.2 Data model

- **Nodes and edges** carry generational identifiers (`slotmap`) — an id
  encodes both slot index and generation, so a stale id is detectably
  distinct from a live reuse of the same slot. This instance-identity
  discipline is load-bearing downstream (§3.3, §4).
- **Hyperedges are N-ary.** Graph distance treats an N-ary edge as making all
  its endpoints mutually distance-1.
- **Node properties** are `SmallVec<[(u32, PropertyValue); 4]>`, where
  `PropertyValue ∈ {Int(i64), Float(f64), Bool(bool)}`. Energy is the
  first-class property (key 1, Float) and a GUI color dimension.
- **Structural class identity** is the Weisfeiler–Lehman neighborhood hash of
  a node — a fingerprint of local structure invariant to node identity and
  absolute position, sensitive only to local topology. Every higher-level
  notion of "type" (regimes, constructors, the transition matrix) is built on
  the WL class.

### 1.3 Rewrite semantics

A rule is an LHS match pattern and an RHS template. Nodes present in both LHS
and RHS survive (same id); LHS-only nodes are deleted; RHS-only nodes are
created. Edges are always recreated, never preserved across a rewrite (there
is no edge identity through a rewrite). Deleting a node deletes its incident
edges automatically (dangling-edge cleanup) — a documented exception to
locality-of-*effect* that does not violate locality-of-*matching* (§1.6).

- **Guards** read node properties to gate matching (e.g. `energy > 100`).
- **Property effects** (RHS) write node properties, in declaration order,
  each effect's reads seeing earlier effects' writes (sequential, not
  snapshot, semantics — §1.5). Ops: `set`, `copy_from`, `scale_from`,
  `add`/`subtract`, `jitter`.
- **Step modes:** `MaxParallel` (apply all non-conflicting matches this step),
  `SequentialRandom` (one random match), `SequentialWeighted` (one match,
  weighted). MaxParallel is the deterministic default.
- **Creation rules** (empty LHS) carry a `[rules.schedule]` (`once` /
  `every_n_steps`) and contribute empty-node-set bindings that never conflict.

### 1.4 Determinism guarantee (verified)

Same config + same seed ⟹ identical final `canonical_hash`, regardless of
thread count. The canonical hash is order-independent by construction: nodes
are sorted into a canonical index by (label, generation, sorted-properties),
edges remapped through that index and internally sorted, the whole hashed with
a fixed-seed `FxHasher`. Two stores built by different insertion orders from
the same logical graph hash identically.

> **Verified.** Seeds `{{FILL: seed A | session=42 | src: run config}}` vs
> `{{FILL: seed B | session=777 | src: run config}}` under MaxParallel produce
> bit-identical EI columns and, after canonical dump ordering, byte-identical
> `ei_dump.json`. The RNG stream is untouched by property effects for
> jitter-free rulesets (proven by positive control, §4.2).

### 1.5 Sequential effect semantics (verified, pinned)

Effects apply in declaration order; reads see earlier writes. The canonical
node-funded mechanic depends on it: with `parent.energy = 200`, `child
scale_from parent ×0.8` (declared first) then `parent subtract 100` yields
`child = 160`, `parent = 100`. Snapshot semantics or swapped order would give
`child = 80`. This is pinned by an order-sensitivity test asserting exactly
160 vs 80 (§4.3) — the semantics are load-bearing, not incidental.

### 1.6 Conflict sets (defined, tested sufficient)

Two bindings conflict if the union of their conflict sets intersects, where
the conflict set is bound nodes ∪ matched edges ∪ dangling-cleanup edges. The
current implementation checks bound-node overlap only; this is *provably
sufficient* under the present rule system (any edge a binding depends on has
all endpoints as bound variables, so any cleanup-deletable node another
binding needs is already a shared bound node), and the full definition is
recorded so a future extension that breaks the precondition is caught by the
disjoint-bound-nodes ⟹ disjoint-conflict-sets invariant test.

---

## 2. Measurement machinery (methods spine)

The instruments. Each is stated with its method because §4 validates the
instruments, and instrument validation is itself a contribution of this work.

### 2.1 Regime classifier

Classifies each evaluation window into one of 11 regimes by a fixed priority
order (checked first-match-wins): `Exploding, Frozen, WhiteNoiseSoup,
SterileChaos, GrowthTreadmill, FragmentedIsolation, Monoculture,
ActivePersistentStructure, GenerativeCandidate, StaticOrdered, Unknown`.
Priority order is load-bearing: `GrowthTreadmill` (5) is inherently
monocultural and must be checked before `Monoculture` (7), or a treadmill
misclassifies (§6, Finding M1). `ActivePersistentStructure` (bounded, active,
mobile, non-generative) sits one rung below `GenerativeCandidate` (which
additionally requires `new_type_bin_rate` above threshold).

### 2.2 Effective dimension `d_eff`

Ball-growth estimator: from sampled source nodes (seeded RNG, 20–50 sources),
BFS outward recording N(r) = nodes within graph distance r; `d_eff` = slope of
log N(r) vs log r, fit over the non-degenerate range (exclude r=0 and the
saturation tail where N(r) approaches total node count). Returns `Option` —
`None` only for graphs too small to fit ≥3 points. Estimated quantity, tagged
`[Estimated]` in reports.

### 2.3 Constructor / replicator detection with fidelity

A constructor is a WL class that catalyzes a task (an input-class →
output-class transformation) while persisting. **Fidelity =
confirmed_cycles / (confirmed_cycles + degraded_cycles)**, where a cycle is
*confirmed* only if a specific focal node instance survives the catalytic
event with the **same NodeId** (instance identity, not class identity —
§4.1), *degraded* otherwise. Confirmed constructor requires fidelity ≥ 0.9 and
cycles ≥ 10; fidelity < 0.5 with cycles marks a *reactant* (consumed by its
own transformation). A constructor is a *replicator* when its output region
contains ≥2 copies of its own pattern, classified `ReplicatorKind ∈ {None,
Trivial, NonTrivial}` (§3.3). Certainty caps at `Empirical` — no volume of
observation earns `Law` (§4.1).

### 2.4 EI / CE causal-emergence calculator

Accumulates a motif-class transition matrix (TPM): for each node existing at
both observation points with the same NodeId, increment counts[from_class]
[to_class] (self-transitions included). Effective Information
EI = H(interventional) − mean-row-entropy, where interventional = uniform
mean of the normalized rows, all in bits. Determinism = 1 − H_noise/log₂M,
degeneracy = 1 − H_int/log₂M, satisfying EI = (det − deg)·log₂M. Coarse-grain
the TPM by a partition (structural / energetic / functional / optimal); the
CE coefficient = EI_macro / EI_micro, with CE > 1 indicating the macro
description has more causal power than the micro. Minimum 1000 transitions
before a claim is emitted; below that it is not yet knowledge.
**Measurement-cadence caveat: §6, Finding F1.**

### 2.5 Class-registry growth `M(t)`

The WL-class registry is append-only; its size over time is a
novelty-accumulation curve, surfaced as a first-class metric (byproduct of
2.4). Saturation vs unbounded growth discriminates substrate regimes (§6,
Finding F3).

---

## 3. What the substrate produces (spine — core results)

Per-builtin characterization on the canonical rule set. **All d_eff, EI, and
CE figures below are slots** pending verification against the cited run's
`metrics.csv` / `ei_dump.json`.

### 3.1 The four canonical rules + control

| Rule | Structure | `d_eff` | Edge growth | Regime | Notes |
|---|---|---|---|---|---|
| `triangle_subdivide` | filled 2D sheet | {{FILL: d_eff | prediction-only ≈2.5–3.5 | src: never measured to a value}} | +2 edges/firing | {{FILL: regime | session≈generative-leaning | src: regime history}} | self-similar; hosts the trivial replicator |
| `binary_edge` | tree / filament | {{FILL: d_eff | prediction-only ≈1–2 | src: run csv}} | +1 edge/firing (linear) | {{FILL: regime | session≈—  | src: regime history}} | the null model; must render + measure *distinct* from triangle |
| `growth_treadmill` | steady, monotypic | {{FILL: d_eff | src: run csv}} | steady | `GrowthTreadmill` | correct only after classifier-ordering fix (M1) |
| `glider` | mark on a ring | n/a (small) | conservative | `ActivePersistentStructure` + `Mobile:Pass` | requires a zero-defect ring track (§6, M2) |
| `white_noise` (control) | random | {{FILL: d_eff | session≈2.85 | src: white_noise run csv}} | — | {{FILL: regime | src: regime history}} | negative control; EI finding F1 |

**The discrimination claim (load-bearing):** the substrate's evaluator
distinguishes crystal, tree, treadmill, and glider — the stated bar for
trusting the evaluator on real experiments. `binary_edge` rendering and
measuring as a *tree distinct from* `triangle_subdivide`'s *sheet* is the
specific null-model check; it was masked twice (once by layout collapse, once
by node-only rendering — §6, M3) before the d_eff and edge-growth metrics
confirmed genuine structural difference independent of visualization.

### 3.2 Fidelity distribution (the measurement that had to be real first)

On bare `triangle_subdivide`, after the fidelity fix (§4.1):

> {{FILL: reactant count | session=12 | src: constructor ecology}} of
> {{FILL: candidate count | session=15 | src: constructor ecology}} constructor
> candidates fall to fidelity 0.00 (reactants — focal instances do not survive
> subdivision churn; certainty `Intuition`), {{FILL: confirmed count |
> session=3 | src: constructor ecology}} are fidelity 1.00 over
> {{FILL: cycles | session=12 | src: constructor ecology}} confirmed cycles,
> and the replicator population is exactly
> {{FILL: replicator count | session=1 | src: constructor ecology}}.

This bimodal 0.00/1.00 distribution is *expected* on a deterministic
MaxParallel substrate — a class either structurally survives its events or it
does not, uniformly. It is the direct evidence that fidelity became a
measurement rather than the tautological constant it was before (§4.1).

### 3.3 Replicator taxonomy and the Experiment-3 correction

> **Prediction.** Experiment 3 (Null Baseline) predicted constructor/
> replicator count 0 on bare substrate ("no replicators in bare substrate").
>
> **Result.** The detector flags a replicator on bare `triangle_subdivide`
> ({{FILL: replicator hash | session=f9dab61e722f99d4 (pre-fidelity-fix) /
> e7658220a6315811 (defects run) | src: constructor ecology}}).
>
> **Resolution.** The detector is correct to spec: a *replicator* is a
> confirmed constructor whose output contains ≥2 copies of its own pattern,
> and a self-similar rule (triangle → triangles) satisfies this by
> construction. The prediction assumed a *biological* replicator (a bounded
> carrier copying its whole self), which is a different and narrower thing.
> Resolved by the `ReplicatorKind` distinction: **Trivial** (the rule
> mechanically yields copies on a static substrate) vs **NonTrivial** (a
> bounded, persistent, mobile individual copying itself — requires the ALife
> layer). On bare substrate, `non_trivial_replicator_count() ≡ 0` **by
> construction** (nothing sets `NonTrivial` today), which is the clean
> baseline Experiment 6's onset marker measures against. Trivial replicators
> present, non-trivial zero — this both matches the detector and satisfies the
> *spirit* of Experiment 3's prediction.

---

## 4. Verification methodology (methods spine — how the instruments earned trust)

The distinctive methodological contribution: instruments were validated along
axes *independent* of their own implementation. "The implementer's tests pass"
was treated as the floor, never the ceiling.

### 4.1 Fidelity: from tautology to measurement

The fidelity instrument shipped broken in a specific, instructive way.

> **Finding V1.** `record_cycle` had a single call site with `confirmed: true`
> hardcoded; non-surviving cycles were `continue`d (skipped) rather than
> recorded as degraded. `degraded_cycles` was **unreachable code across the
> entire workspace**, so fidelity ≡ 1.00 identically — the uniform-1.00
> signature. Certainty then escalated to `Law` at fidelity ≥ 0.9, so every
> long-lived class became `Law`. Both were fixed: survival is judged
> per-focal-instance by NodeId (a structurally identical impostor with new ids
> is *degraded*, verified in both directions by
> `constructor_survival_uses_node_ids`); certainty caps at `Empirical`, pinned
> by a regression test asserting no observation earns `Law`. The fix changed
> the registry from "everything is a perfect Law constructor" to the real
> distribution of §3.2.

The general lesson, recorded for reuse: *a check existing is not a check being
load-bearing* — the node-id signal existed but was wired as a filter, not a
verdict.

### 4.2 EI/CE: independent recomputation + the positive control

**Independent recomputation.** A standalone Python verifier (`verify_ei.py`,
numpy, zero shared code/language/author-context with the Rust) recomputes EI,
determinism, degeneracy, and per-partition CE from a dumped TPM. It has a
self-test that first re-derives a hand fixture (so the verifier is trusted
before it judges), and an invariant suite (EI ≥ 0 by Jensen; EI ≤ log₂M; the
det/deg identity; identity-partition ⟹ CE = 1; total-partition ⟹ CE = 0;
coarsened rows stochastic). Result: agreement to
{{FILL: agreement digits | session=6 decimals | src: verifier output}} on
`triangle_subdivide` and `white_noise` dumps.

> **Finding V2 (the verifier caught its own author).** On first real-data run,
> the structural partition mismatched *and the verifier's own invariant suite
> failed on its own arithmetic* (identity-partition CE = 0.9549 ≠ 1.0). Because
> those invariants involve no Rust numbers, the defect was localized to the
> verifier's `coarsen()` — it kept an all-zero macro row for a partition group
> whose members were all row-less (destination-only) classes, which the
> dump contained (dead classes isolated in one group). The Rust side applied
> the zero-row-exclusion convention consistently; the verifier did not. One-line
> fix + a regression case (a destination-only class isolated in its own group,
> the exact shape the original self-test lacked). After the fix, both dumps:
> all quantities MATCH, all invariants PASS. **The disagreement was provably on
> the verifier's side** — which is exactly what a well-constructed independent
> check should be able to establish about itself.

**Positive control for the RNG zero-draw guarantee.** Rather than trust that a
jitter-free ruleset draws no RNG (which would silently shift every seeded
run's stream), a deliberate unconditional `rng.next_u64()` was injected into
the effects path: the glider canonical hash moved
{{FILL: perturbed hash | session=caa02b792c679980 → ff922d17015d9bd7 |
src: positive-control run}}. Reverted, it holds. This is strictly stronger
than a pre/post commit diff: it proves the hash *detects the exact fault
class*, so its being unchanged on real code *proves* zero draws — independent
of any recorded constant. (No git repo exists in the build environment, so the
positive control was also the only available provenance method; it is the
better one regardless.)

### 4.3 Hand-traced fixtures with pre-committed exact outcomes

Two end-to-end fixtures with every outcome computed by hand before running,
driven through the real Engine:

- **Cascade** (`effects_cascade_fixture.toml`): a self-seeding energy-funded
  spawn chain, 200 → 160 → 128 → 102.4 → 81.92, self-terminating at step 5
  when 0.8⁴·200 falls below the guard. Confirms sequential semantics (child1 =
  160 not 80) and the full guard→effect→extinction loop. This is the
  node-funded replication mechanic running as a real starvation curve.
- **Conservation soak** (`conservation_soak.toml`): a closed 4-node transfer
  system; node-energy sum must remain **exactly 100.0 forever** (dyadic
  values, bit-exact). Entangles effects with conflict resolution — any
  MaxParallel disjointness failure or double-apply drifts the sum and never
  recovers. Result: sum exactly 100.0 after
  {{FILL: soak steps | session=1000 | src: soak run}} steps, node/edge counts
  pinned, no drift.

> **Finding V3 (fixture-vs-IEEE-754, not a bug).** The cascade's decimal
> literals (102.4, 81.92, 2.4) are real-number-exact but not f64-representable;
> honest IEEE-754 lands 1–13 ULP off. The 13-ULP gap on 2.4 vs 1-ULP on 81.92
> is catastrophic cancellation: ~½ ULP of representation error at exponent 6
> (102.4), re-expressed at 2.4's exponent, inflates by 2⁵ ≈ 32×. The books
> balance to the bit. The test asserts bit-equality against the step-by-step
> f64 reference chain, not the decimal literals. (The conservation soak's
> dyadic 10.0/25.0 were chosen precisely to avoid this and got bit-exact
> results — the fixtures were designed to isolate representation error from
> logic error.)

### 4.4 Durable-output correctness

> **Finding V4 (write-side corruption, fixed).** `PropertyValue` is
> `#[serde(untagged)]` (required for TOML bare literals). Untagged + bincode
> (non-self-describing) means property-bearing snapshots serialized with **no
> discriminant** — `Int(5)` and `Float(5.0)` indistinguishable on disk, and
> deserialization failing outright (`DeserializeAnyNotSupported`). This was a
> live corruption of durable output on every property-bearing run, fixed at
> discovery (not deferred — a wrong write side is not subject to fixture-first
> deferral). Fix: a tagged wire type (`PropertyValueWire`, externally tagged)
> applied via `#[serde(with)]` at the persistence boundary only; domain type
> keeps untagged for TOML. Snapshot version bumped 1→2; v1 property-bearing
> snapshots recorded as unrecoverable-in-principle. Round-trip test:
> `Int(5)`/`Float(5.0)`/`Bool(true)` survive exactly, `Int(5) ≠ Float(5.0)`
> after restore. Bincode-site audit: two sites total, one (the store) fixed,
> one (`MetricsRow`) property-free; everything else is self-describing JSON.

---

## 5. Null baselines (spine — what does *not* happen, and why it matters)

The load-bearing negatives. Later work (Experiment 6: "does CE rise when
non-trivial replication establishes?") measures *against* these; a baseline
that silently drifts would invalidate the headline result.

1. **No causal emergence on bare substrate.** CE < 1 for *all* enabled
   strategies on both `triangle_subdivide` and `white_noise`:
   - triangle_subdivide: {{FILL: per-strategy CE | session≈—  | src: ei_dump}}
   - white_noise: structural {{FILL:|session≈0.337|src: ei_dump}}, energetic
     {{FILL:|session=0.000|src}}, functional {{FILL:|session≈0.178|src}}
   The calculator does not trivially emit CE > 1 for any input — the
   validation that it *can* detect the absence of emergence.
2. **Energetic strategy degenerate on uniform energy.** Bare substrate energy
   is uniform ⟹ K=1 partition ⟹ EI_macro = 0 ⟹ CE = 0. Correct, not a
   failure.
3. **Zero non-trivial replicators**, by construction (§3.3).
4. **The interesting result (CE > 1) is NOT present today**, and was not
   tuned for — reserved for the ALife condition. Absence of emergence on bare
   substrate is the correct null, exactly as zero non-trivial replicators is.

---

## 6. Methodological findings (supporting-findings spine — first-class results)

Findings about the *methods*, promoted from footnotes because several are more
novel than the substrate characterization itself.

### Finding F1 — EI on open-ended class registries

> **Prediction.** "EI ≈ 0 for white noise."
>
> **Result.** White noise gave EI_micro =
> {{FILL: white noise EI_micro | session=8.25 bits | src: ei_dump}} =
> log₂({{FILL: white noise class count | session=304 | src: ei_dump}}), with
> **determinism exactly 1.0**.
>
> **Resolution.** On a growing WL registry, a churned node's changed
> neighborhood mints a *fresh* class each window, so nearly every class is
> observed as a source exactly once ⟹ every TPM row is one-hot ⟹ H_noise = 0,
> EI_micro = log₂M *regardless of order vs noise*. This is the sample-starved
> pathology of empirical Markov estimation: the every-state-visited-once
> regime yields a maximally deterministic estimated TPM. The prediction
> silently assumed a *fixed* state space (the unit test pins that case
> correctly). **Consequence: EI_micro alone does not discriminate noise from
> order at window cadence on growing substrates — CE is the discriminating
> quantity, and it stayed < 1 on both, keeping the null intact.**

**CE bias direction (recorded for Experiment 6).** Because macro groups recur
(real multiplicity) while micro classes are one-shot (inflated to ceiling), CE
is biased **downward** on open registries. Therefore: (a) the null baselines
are extra-trustworthy; (b) any future CE > 1 clears a *conservative* bar —
strengthened, not weakened; (c) a real emergence effect could be *masked*
(false-negative risk). Revisit the measurement definition (e.g.
recurrent-class-restricted micro TPM, or comparing EI_macro across conditions
directly) **only if Experiment 6 nulls out and this bias is the suspect** —
not preemptively.

### Finding F2 — Fidelity requires stochasticity to populate the degrading band

> **Prediction.** Ensemble heterogeneity (many instances of a class in
> different defect environments) would populate the 0.5–0.9 "degrading
> constructor" band.
>
> **Result.** Temporal stochasticity produces it; spatial ensemble
> heterogeneity does not. The glider (`SequentialRandom`) yielded an
> intermediate fidelity of {{FILL: glider intermediate fidelity |
> session=0.14 over 6 cycles | src: glider constructor ecology}} — the mark's
> random walk survives some cycles, churns others. The defect-heavy crystal
> (deterministic `MaxParallel`) stayed **bimodal** — three classes at 1.00,
> the rest at 0.00, no middle band.
>
> **Resolution.** Under determinism, a class either survives its events or
> does not, uniformly — heterogeneity across instances did not smear the
> distribution. The middle band is reachable via *time* (stochastic step
> modes), not via *spatial ensemble* on this substrate. Practical: the
> glider/stochastic path is the generator for the degrading-constructor
> regime, should robustness-evolution studies need it. Caveat: the 0.14 rests
> on only {{FILL: glider cycles | session=6 | src}} cycles — thin; a longer
> glider run would tighten it. Its trustworthiness comes from the *company it
> keeps* (a 44-cycle 1.00 and clean 0.00s in the same run), not the lone value.

### Finding F3 — `M(t)` is a novelty-accumulation curve (free gift)

EI_micro ≈ log₂M means the EI machinery incidentally measures registry growth.
Surfaced as a first-class metric: crystal saturates at
{{FILL: crystal saturated class count | session≈15 | src: run csv}} classes
(hence the *stable EI_micro* the exit criterion checks — that criterion is now
understood as a registry-saturation detector), while noise climbs unbounded to
{{FILL: noise class count | session=304 | src}}. This is the Hash Chemistry
linear-vs-accelerating novelty-growth quantity, now computed as a byproduct —
directly relevant to the standing discreteness question (§8).

**Cross-check (independent-pipeline consistency).** The registry's per-interval
new-class rate is conceptually the evaluator's `new_type_bin_rate`
(treadmill/generative discriminator) computed from the causal side — two
independent pipelines, same quantity. Reported agreement:
{{FILL: cross-check | session=4/window triangle_subdivide, 8/window white_noise
| src: both runs}}. Agreement = free two-implementation consistency
confirmation; the pattern echoes V2 (the verifier) — cross-checks that can
catch either side.

### Finding M1–M3 — substrate/evaluator bugs found by discrimination runs

- **M1 (classifier ordering):** `growth_treadmill` misclassified as
  `Monoculture` because the classifier checked regimes out of priority order
  (`Monoculture` before `GrowthTreadmill`, which are near-synonyms separated
  only by priority). The over-sensitive term was `new_type_bin_rate` /
  structure-signature; fixing the ordering also protected the
  treadmill-vs-generative boundary for *every* rule — the same metric gates
  the not-nothing result.
- **M2 (glider substrate incompatibility, a finding not a bug):** the glider
  died at ~step 200 on generic crystal-with-defects because a conservative
  mark random-walking a finite defect-ridden graph reaches a configuration
  with no valid match — the substrate provides no propagation track. Correct
  behavior; the fix was a purpose-built zero-defect ring, after which it
  classifies `ActivePersistentStructure` + `Mobile:Pass` with the mark hopping
  the full run (only the layout froze). Embodies the program's principle:
  seeded structure requires a compatible substrate; dying on an incompatible
  one is honest reporting.
- **M3 (visualization masking structure):** `binary_edge` and
  `triangle_subdivide` rendered as identical blobs — twice-masked (force-
  directed layout collapse; node-only rendering with edges invisible except at
  extreme zoom). Resolved by metrics-not-pixels diagnosis (edge/node growth,
  d_eff) plus edge rendering + zoom-scaled node/edge sizing. The graphs were
  always distinct; the render lied.

---

## 7. Reproducibility

Every number in this report is regenerable:

- **Runs:** builtin via the GUI dropdown or CLI; custom via TOML. Fixed seed +
  config ⟹ fixed `canonical_hash` (§1.4). Record seed and config hash per run.
- **Artifacts per run:** `metrics.csv` (one row per window; d_eff, EI columns,
  `wl_class_count`), `report.md`, `manifest.json` (status, seed, thread count),
  `ei_dump.json` (TPM, classes, cadence, partitions — canonically ordered,
  byte-stable), snapshots (v2 wire format).
- **EI verification:** `verify_ei.py --self-test` first (validates the
  verifier), then `verify_ei.py <run>/ei_dump.json --expect-ei-micro X
  --expect-ce name=X` against the Rust-reported values.
- **Fixtures:** `effects_cascade_fixture.toml`, `conservation_soak.toml` —
  hand-traced outcomes in their headers.
- **Test suite:** {{FILL: workspace test count | session=137 | src: cargo test}}
  workspace tests, 0 clippy warnings, at last run.

**Verification punch-list:** `grep '{{FILL' this file` — every hit is a number
to resolve against its cited artifact before this report is trustworthy.

---

## 8. Known limitations & open questions (lab-notebook honesty)

1. **CE downward bias on open registries (F1).** The most consequential
   caveat for the headline experiment. Documented, not yet addressed; the
   decision to revisit is explicitly conditioned on Experiment 6 nulling out.
2. **Measurement cadence is window-level, not step-level.** WL classes are
   computed per window (per-step at 200k+ steps/s is prohibitive), so the TPM
   describes window-timescale dynamics. Legitimate, but the cadence is part of
   the measurement's definition and is recorded in every EI result. Step-level
   dynamics are unmeasured.
3. **d_eff is spatial-ensemble-blind to the degrading band (F2)** — a property
   of the fidelity instrument under determinism, relevant if robustness
   studies need intermediate fidelities without stochastic step modes.
4. **The substrate is deterministic-MaxParallel by default.** Stochastic modes
   have materially different measurement properties (F2). Characterization here
   is primarily of the deterministic regime; the stochastic regime is
   characterized only where the glider exercised it.
5. **triangle_subdivide's d_eff was never measured to a recorded value** —
   only predicted (§3.1). A first measurement is owed.
6. **The discreteness question (standing, foundational).** Hash Chemistry's
   *linear* (not accelerating) novelty growth is the most concrete evidence
   against the discrete-substrate assumption underlying the program. Hypersim
   now computes the analogous M(t) curve (F3) as a byproduct — so the substrate
   can, in principle, be turned on this very question. Whether Hypersim's
   novelty growth is linear or accelerating, and on which rules, is an open
   measurement this characterization *enables* but does not yet answer.

---

## 9. Status

The Phase-10 gate — *clean substrate pipeline as a prerequisite before any
ALife work* — is passed, with receipts: four rules discriminate correctly;
d_eff, fidelity, and EI/CE are validated instruments (fidelity per-instance
and honest; EI/CE agreeing with an independent implementation to
{{FILL: agreement digits | session=6 decimals | src}} and respecting all
invariants); null baselines hold (crystal and noise both CE < 1); property
effects conserve exactly under parallel conflict resolution with sequential
semantics pinned and the RNG zero-draw guarantee proven by positive control;
durable output is no longer lossy. The substrate is trustworthy enough to
build the tri-ecology ALife layer on top of — which is the next phase, and a
separate document.

*Resolve all `{{FILL}}` slots against source artifacts before circulating this
report as a result.*
