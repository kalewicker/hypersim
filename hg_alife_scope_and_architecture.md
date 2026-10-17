# hg-alife — Scope & Architecture

**Decision document.** Not a build spec. Its job is to fix the architecture,
name the seams, expose the gaps honestly, and pre-commit the exit criteria, so
that the build spec that follows is mechanical rather than exploratory.

Target: **Experiment 5** — a stable, spontaneously-replicating carrier
population on the validated substrate. Experiment 5 gates Experiment 6 (the
CE-vs-replication-onset result), because *CE measurements are meaningless on
an unstable population*.

---

## 0. Scope

**In:** genomes, carriers, node-funded replication, harvest, metabolism,
death-with-recycling, lineage/phylogeny, ecological metrics, the
`MotifClassifier` implementation that lets the causal layer see carriers.

**Out, explicitly:** the tri-ecology (`self_modifying_cognitive_ecology_
architecture.md`) — evaluator cells, meta-judges, world ecology, protected
kernel, LLM specialists, RSI detection. Also out: `hg-cognition`,
`RuleSelectionPolicy`, learned policies.

**Why out.** The tri-ecology's §18 lists ten *unsolved* research problems, of
which **evaluator grounding** is the acknowledged core ("how can evaluators
remain adaptive without becoming easy prey?"). This layer sidesteps that
problem entirely by having **no evaluator**: fitness is survival and
replication, full stop. There is nothing to Goodhart because there is no proxy
— only whether a lineage persists. That is not a limitation of this phase; it
is the reason this phase must precede the tri-ecology. Get spontaneous
replication and stable population dynamics working on a substrate you trust,
*then* introduce the machinery that creates the grounding problem.

---

## 1. The architectural decision: intrinsic, not imposed

### 1.1 The two candidate designs

**Imposed (the C++ design).** `hg-alife` owns a `MotifRegistry` of
`Motif { node_id, energy, genome, lifetime }` records. Carriers live in the
side-table; ALife dynamics mutate the side-table and reach into the graph to
adjust node energy. The hypergraph does not know carriers exist.

**Intrinsic.** Carriers **are hypergraph nodes**. Energy and genome are node
properties. Harvest, metabolism, replication, aging, and death are **rewrite
rules with property effects**. The hypergraph does not merely host the
biology; the biology *is* hypergraph physics.

### 1.2 The architecture already decided this — the imposed design is illegal

This is not a preference. Two invariants already in force make the imposed
design unbuildable:

1. **`Observer` is read-only.** It receives `&RewriteEvent` and
   `&GraphSnapshot`. It cannot mutate the store. An Observer-based motif
   registry therefore cannot make carriers *do* anything to the world — the
   carriers would be epiphenomenal spectators, which is not an ALife.
2. **`hg-alife` may not depend on `hg-engine`** (v4 dependency graph:
   `hg-alife → hg-core, hg-observe, hg-causal`; only `hg-sim-core` imports
   `hg-engine`). And the five-layer plan states it directly: *"No layer may
   directly mutate a lower layer at runtime."*

The only remaining seam through which anything above the engine can change the
graph is the rewrite system itself. **Therefore ALife dynamics must be rewrite
rules.** The crate boundaries and the borrow checker enforce what the C++
project had only as an intention — and failed to keep.

### 1.3 …and it is also the design the principles demand

The C++ post-mortem named two failures. Both are structurally prevented here:

- *"Motifs were imposed on top of emergent geometry rather than arising
  organically"* — carriers are now made of the same stuff as the world.
  Nothing is imposed. (Perturbation-seeding of an initial carrier remains
  acceptable; per-generation motif injection is not, and is now not even
  expressible.)
- *"Abstraction layer blurring made bugs unattributable"* — a wrong behavior
  now lives either **in the rule file** (biology) or **in the crate**
  (bookkeeping), never smeared across both. The split is enforced by the type
  system, not by discipline.

### 1.4 The consequence worth sitting with

**`hg-alife` contains almost no dynamics.** The biology is a TOML file. The
crate is genome encoding, a classifier, an observer, a lineage tree, and
metrics — *bookkeeping around physics that runs in the engine*. "Life as
physics" stops being a slogan and becomes the file layout.

---

## 2. Representation

| Concept | Encoding | Rationale |
|---|---|---|
| **Carrier** | a hypergraph **node** with a distinguishing **`Label`**, attached by an edge to the substrate node it occupies | first-class structure; visible to WL, d_eff, the constructor registry, and the EI micro-states *without special-casing* |
| **Occupancy / movement** | the carrier↔substrate edge; movement = rewire it | expressible as a rewrite rule |
| **Cohabitation** | N carrier-nodes attached to one substrate node | competition falls out of conflict resolution (§6.2) |
| **Energy** | `PropertyValue::Float`, key 1 (existing convention) | already the GUI colour dimension |
| **Genome traits** | **one `Float` property per trait** (replication_threshold, harvest_rate, maintenance_cost, max_lifetime, …), keys registered in `docs/property_registry.md` | see §2.1 — bit-packing is incompatible with the intrinsic design |
| **Death** | delete the carrier node | dangling-edge cleanup removes the occupancy edge automatically (Contract Rule 5) |
| **Lineage** | Observer side-table (`LineageTree`) | *observation*, not dynamics — correctly outside the substrate |

### 2.1 Why the genome cannot be a bit-packed `Genome(u32)`

`not_nothing_v2_plan.md` specifies `Genome(pub u32)` with traits decoded from
bit-fields. **A guard cannot decode bits.** Guards compare a property to a
value; there is no bit-extraction operator, and adding one would be a
substantial and ugly grammar extension. If genome-dependent behaviour must be
expressible as a rule (and under §1.2 it must), then each trait has to be
independently readable — i.e. **one property per trait**.

Consequences, stated plainly:

- **Mutation becomes continuous (`jitter`), not discrete bit-flip.** Deviation
  from the plan (§7.1). `jitter` already exists and is verified.
- **Inheritance is `copy_from` parent → child, then `jitter`** — both ops exist
  and are verified. The cascade fixture already demonstrated `scale_from`
  inheritance end-to-end.
- The genotype→phenotype map becomes identity-with-noise rather than a
  quantised decoding. Discrete quantisation creates neutral networks and
  punctuated dynamics; continuous does not. This is a **real change to the
  evolutionary dynamics**, not a cosmetic one, and it is pre-committed in §7.

---

## 3. What the intrinsic design gives away for free

This is the payoff, and it is large.

1. **The causal layer sees carriers with zero new machinery.** Carriers have
   distinct WL classes (via label), so they appear in the task lattice, the
   constructor registry, and the EI transition matrix automatically.
2. **`ReplicatorKind::NonTrivial` becomes detectable rather than hypothetical.**
   A replicating carrier is a hypergraph pattern whose output region contains
   two copies of itself — precisely what the *existing* replicator detector
   looks for. The only thing needed is for the `MotifClassifier` to tell the
   causal layer "this WL class is a carrier," so that a replicator of a
   carrier-class is classified `NonTrivial` rather than `Trivial`. The
   taxonomy you built as a placeholder was, it turns out, correctly shaped.
3. **The functional coarse-graining acquires the partition Experiment 6 needs.**
   `carrier / substrate` becomes a genuine partition of the *same* micro state
   space — a real coarse-graining, not a bolted-on second system. Under the
   imposed design, CE would have been computed over a state space in which the
   carriers *did not appear at all*, and Experiment 6 would have been
   meaningless.
4. **Energy conservation is checkable end-to-end**, by the same method as the
   `conservation_soak` fixture: the ecology's energy books must balance to the
   bit, modulo the declared entropy loss.

---

## 4. Crate layout

```
crates/hg-alife/
  Cargo.toml           deps: hg-core, hg-observe, hg-causal, hg-schema
                       NEVER hg-engine
  src/
    lib.rs             AlifeError, re-exports
    genome.rs          trait keys, encode/decode to PropertyValue, trait bounds
    motif.rs           MotifKind {Carrier, Catalyst, Shell}; carrier
                       identification from Label + properties
    classifier.rs      HypergraphMotifClassifier: impl MotifClassifier
    observer.rs        AlifeObserver: impl Observer — lineage + ecology metrics
    lineage.rs         LineageTree, Newick export, clade counts, survival curves
    metrics.rs         EcologyMetrics: carrier count, carriers/substrate-node
                       ratio, mean/variance per trait, births, deaths,
                       replication vs spontaneous origin counts
  rules/
    alife_base.toml    THE BIOLOGY. One named rule per biological process.
```

Wiring: `hg-sim-core` (the only crate that may touch `hg-engine`) loads
`alife_base.toml` into the engine's rule set, registers `AlifeObserver` on the
observer chain, and injects `HypergraphMotifClassifier` into
`CausalLanguageObserver` in place of `NullMotifClassifier`. Gated behind an
`alife` Cargo feature on `hg-sim-core`/`hg-gui`, per the v4 optional-layer
convention — a build without the feature must remain bit-identical to today's
substrate-only behaviour (assert via `canonical_hash`).

### 4.1 The rule file is the ecology, and it is a review artifact

Every rule in `alife_base.toml` is one biological process, named, with a
purpose comment: `harvest`, `metabolise`, `age`, `die_and_recycle`,
`replicate`, `influx`, `bootstrap_seed`. **Risk (new, and real): the rule file
becomes an unauditable god-object of special-case rules.** Mitigation: it is
versioned alongside the crate, kept minimal, and every added rule must state
which biological process it implements and why it cannot be composed from
existing ones.

---

## 5. Seams

### 5.1 Existing, no change needed

- **`Observer`** — `AlifeObserver` consumes `RewriteEvent`s to maintain lineage
  and ecology metrics. Read-only, which is correct: observation must not
  perturb the ecology it measures.
- **`property_effects`** — the ALife gateway, built and verified. `set`,
  `copy_from`, `scale_from`, `add`/`subtract`, `jitter` cover inheritance,
  metabolism, harvest, and mutation.
- **Creation-rule scheduling** — available for periodic global processes if
  needed.

### 5.2 `MotifClassifier` — exists, but the signature may not suffice

`fn classify(&self, attr: &Attribute) -> u32`. If `Attribute` is only a WL
hash, the classifier can distinguish carrier-class from substrate-class
**provided the WL hash incorporates node labels** (§9, Q1 — must verify
first). It cannot see energy or genome. The plan's sketch wants
"energy relative to local mean" in the classification, which this signature
cannot supply.

**Recommendation:** classify on label-bearing WL class only, for now. Motif
kind is a *structural* category (carrier / catalyst / shell / substrate);
energy is a *state* variable and does not belong in a type. If a later need for
state-conditioned classification appears, extend `Attribute` — do not widen the
trait speculatively.

### 5.3 Required substrate extension: `Guard::PropertyCompare` (the one real cost)

Guards today compare a property to a **constant**. Genome-dependent behaviour
needs property-to-property comparison, **including across variables**:

```
carrier.energy       >  carrier.replication_threshold   # genome-dependent
carrier.energy       <  carrier.satiation               # satiation gating (§6.2)
substrate.energy     >= child_node_cost                 # (constant — works today)
```

Proposed:

```rust
Guard::PropertyCompare { var_a: usize, key_a: u32, op: GuardOp,
                         var_b: usize, key_b: u32 }
```

Semantics mirror the existing `NodePropertyOp`: variant mismatch ⟹ guard
false, no coercion, no panic. Missing property on either side ⟹ false. This is
a contained extension to `hg-rules` (grammar + types + parser) and
`hg-engine`'s guard evaluation; it touches no other layer, adds no
non-determinism, and does not alter behaviour for any rule that does not use it
(assert with the `canonical_hash` regression, as `property_effects` did).

**This is the single blocking substrate change.** Everything else in this
document composes existing, verified machinery.

---

## 6. Gap analysis — what is *not* expressible, and what to do about it

Locality of matching is absolute in this substrate (a rule may inspect only its
own pattern). Three ALife mechanics in `not_nothing_v2_plan.md` are non-local.
Each is addressed honestly rather than smuggled in.

### 6.1 Global carrying capacity (`SOFT_CAPACITY = 600` / `HARD_CAPACITY = 1200`)

**Not expressible, and should not be.** Density-dependent maintenance keyed to
*global population* requires a global aggregate; no local rule can read it, and
adding a global-state channel would puncture the locality invariant that the
whole substrate rests on.

**Proposal: make carrying capacity emergent from a closed energy economy.**
Energy enters via a local, self-limiting `influx` rule (guarded: a node takes
influx only below an energy cap). Energy leaves via the declared entropy loss
on death (80% recycled, 20% lost) and metabolic drain. Population is then
bounded by the energy budget rather than by a hard-coded ceiling.

This is *better than the original*, on this project's own terms: the C++ design
imposed the capacity; here it **emerges**, and becomes a measurable dependent
variable rather than a constant. It yields an immediate falsifiable prediction:
**carrying capacity should scale with influx rate.** That is a real experiment
the C++ design could not run.

Cost: the plan's 0.283 carrying-capacity target was measured under the imposed
regulator. Under an emergent one it is **not** a pass/fail criterion — it is a
*comparison*. See §7.2.

### 6.2 Crowding-dependent harvest (`harvest / sqrt(cohabitants)`)

**A cohabitant count is not readable by a guard** (guards read properties, not
topology/degree).

But notice: **MaxParallel conflict resolution is already a crowding mechanism.**
Two carriers on the same substrate node both bind the substrate node ⟹ they
conflict ⟹ only one harvests this step. Competition for a shared resource is
enforced by the substrate's own disjointness rule, for free.

**The catch, and it is a real one:** conflict resolution is *deterministic*
(sorted by priority, rule id, node ffi). The same carrier wins **every** time,
and its cohabitants starve permanently. That is a fixed monopoly, not
competition.

**Proposal: satiation-gated harvest.** Add a guard `carrier.energy <
carrier.satiation` (a genome trait). A well-fed carrier stops matching the
harvest rule and drops out of contention, letting a hungrier cohabitant eat.
Access rotates. This is fully local, needs no topology-reading guard, is
biologically honest (organisms do stop eating), and makes satiation an
**evolvable trait** rather than a constant — greedy genomes monopolise but pay
metabolic costs; restrained ones share. That is a genuinely interesting
selective dimension the C++ design did not have.

Fallback if satiation gating proves insufficient: a `Guard::Degree` extension
(read a node's degree, optionally filtered by neighbour label). Larger
extension — topology-reading guards are a new *kind* of guard. Do not build it
until satiation gating is measured and found wanting.

### 6.3 Bootstrap: seeding the first carrier

A creation rule (empty LHS) creates *isolated* nodes; it cannot attach a
carrier to an existing substrate node. A rule matching "any substrate node"
would spawn a carrier on **every** node.

**Solution (reuses the glider's mark trick, no new machinery):** the generator
sets a one-off marker property on exactly one node; a `bootstrap_seed` rule
matches `substrate_node WITH marker`, creates the carrier + occupancy edge, and
clears the marker. Fires exactly once, on exactly one node. Perturbation-
seeding — permitted. Motif-seeding per generation — neither permitted nor
expressible.

---

## 7. Pre-committed deviations from `not_nothing_v2_plan.md`

Recorded now so they cannot be rationalised later.

### 7.1 Continuous genome, not bit-packed (§2.1)

Traits are per-key Float properties; mutation is `jitter`. **Forced** by the
intrinsic architecture (guards cannot decode bits). Expected consequence: no
neutral networks from bit-quantisation; smoother, less punctuated evolutionary
dynamics. If punctuated dynamics turn out to be *load-bearing* for
open-endedness, the fix is a `bitflip` property-effect op plus bit-extraction
guards — a substantial extension, not to be undertaken speculatively.

### 7.2 Emergent, not imposed, carrying capacity (§6.1)

**The 0.283 carriers/node ratio is demoted from pass/fail criterion to
comparison datum.** The plan's own open question #4 already asks whether 0.283
is universal or parameter-dependent; under an emergent regulator that question
becomes directly measurable (sweep the influx rate). Pre-committed
interpretations:

- Ratio lands near 0.283 → striking, and worth reporting: the constant survives
  a change of regulator, weak evidence it reflects something about the
  substrate rather than the C++ tuning.
- Ratio lands elsewhere but is **stable** → Experiment 5 still passes. Report
  the value and its influx-dependence. The requirement was ever only *a stable
  population*, not *that specific number*.
- Ratio is unstable at every influx rate → the energy economy does not
  self-regulate. That is a real finding and blocks Experiment 6 until resolved.
  Do **not** reach for a global cap to force stability; that would be re-
  importing the imposed design to hide a failure of the emergent one.

### 7.3 Satiation replaces sqrt-crowding (§6.2)

Adds an evolvable trait; changes the competitive dynamics. Pre-committed: if
carriers monopolise nodes and cohabitants starve *despite* satiation gating,
that is evidence satiation gating is insufficient and the degree-guard
extension is warranted — a finding, not a nuisance.

---

## 8. Exit criteria — Experiment 5

Experiment 5 passes when **all** hold, on a run with a fixed seed, reproducible:

1. **Spontaneous replication.** Carriers replicate from a single seeded
   ancestor. No per-generation injection (structurally impossible — good).
2. **Bounded population.** No `Exploding` regime; no extinction within the run
   window.
3. **Stable population.** The carriers/substrate-node ratio settles to a steady
   value with low variance across a long late window. *Report the value; do not
   require 0.283* (§7.2).
4. **Multi-generation lineage.** The `LineageTree` shows descent beyond the
   first generation: clade count > 1, at least one clade persisting > K
   generations. First-generation offspring that all die immediately is a
   failure, not a pass.
5. **Selection is acting.** Trait distributions *shift* over time — not merely
   drift. Measurable as directional change in a trait mean, or variance
   collapse around a favoured value. If all genomes are equally fit, there is
   no evolution, only replication.
6. **Non-trivial replicators detected.** `non_trivial_replicator_count()` goes
   **0 → >0**. This is the bridge to Experiment 6 and the validation of the
   whole replicator taxonomy: the baseline was zero *by construction*, and it
   must now become nonzero *by biology*.
7. **No substrate regressions.** Determinism holds (same seed ⟹ same
   `canonical_hash`); the energy books balance to the declared entropy loss;
   a no-`alife`-feature build is bit-identical to the current substrate.

### 8.1 Pre-committed failure interpretations

| Failure | Interpretation | Forbidden response |
|---|---|---|
| Population explodes | influx exceeds metabolic drain; the economy is not closed | adding a global cap |
| Population extinct | metabolic cost > harvest, or replication threshold unreachable | lowering the threshold until it works, without diagnosing the energy budget |
| Replication but no evolution | mutation absent (check `jitter` fires) or selection non-differential (traits don't affect fitness) | injecting fitness by hand |
| Cohabitants starve | satiation gating insufficient (§6.2) | reaching for global regulation |
| NonTrivial replicators not detected | classifier isn't marking carrier classes, or whole-pattern containment doesn't fire on carrier structures — **a detector issue, not an ALife issue** | weakening the replicator criterion to make it fire |

---

## 9. Must-verify before the build spec

These are cheap checks that could invalidate parts of the design. Do them
first.

- **Q1 — Does the WL hash incorporate node `Label`?** If **no**, carriers are
  structurally indistinguishable from substrate nodes of the same topology, and
  §3's entire "the causal layer sees carriers for free" payoff collapses, along
  with the `MotifClassifier` seam. Fix would be to include label in the WL
  hash — small, but it changes every existing WL class and therefore every
  recorded baseline.
- **Q2 — Does the WL hash incorporate node *properties*?** It must **not**. If
  it does, then continuous genomes mint a **fresh WL class per individual**,
  the class registry explodes, and Finding F1's open-registry EI pathology goes
  from a caveat to a catastrophe (EI_micro = log₂M with M growing as the
  population). **The required combination is: WL sees Label, WL ignores
  properties.** Verify; if properties are hashed, they must be excluded or
  coarsely bucketed before any ALife run.
- **Q3 — Do carriers pollute substrate metrics?** `n_nodes`, `d_eff`, and the
  regime classifier will now include carrier nodes. Decide: label-aware
  substrate-only variants of these metrics, and always pair an ALife run with a
  same-seed bare-substrate control.
- **Q4 — Rule-count/performance.** An `influx` rule matching every substrate
  node produces one binding per node per step (~10⁴ bindings). Confirm the
  matcher's cost is acceptable, or make influx conditional (guard: `energy <
  cap`) so that only depleted nodes match — which is the design anyway (§6.1).

---

## 10. Build sequence

Each stage gates the next. Fixture-first, cheap gates before expensive
commitments.

```
S0.  Answer Q1–Q4. If Q2 fails (WL hashes properties), fix it FIRST — every
     ALife run before that fix would be measuring a corrupted class registry.

S1.  Substrate: Guard::PropertyCompare (§5.3). Grammar + parser + eval + tests.
     Gate: canonical_hash regression — all existing builtins bit-identical.

S2.  hg-alife skeleton: genome.rs (trait keys + property registry entries),
     motif.rs, classifier.rs. No dynamics yet.
     Gate: HypergraphMotifClassifier distinguishes a hand-built carrier node
     from a substrate node.

S3.  alife_base.toml, incrementally, each rule tested in isolation on a fixture
     substrate before composition:
       bootstrap_seed  -> exactly one carrier appears, attached, marker cleared
       harvest         -> energy moves node -> carrier; conservation exact
       metabolise      -> carrier energy decays
       die_and_recycle -> carrier deleted; 80% returns to node; edge cleaned up
       influx          -> depleted nodes refill; conserved bookkeeping
       replicate       -> node-funded child with inherited+jittered genome
     Gate per rule: an energy-conservation assertion (the conservation_soak
     method), and a hand-traced fixture where feasible.

S4.  Compose. First full ecology run. Expect it to fail (explode or die out) —
     that is normal, and the failure table (§8.1) says how to read it.

S5.  AlifeObserver + LineageTree + EcologyMetrics + Newick export.
     Gate: lineage reconstructable from events alone; parent/child links sound.

S6.  Experiment 5 proper: influx sweep, stability characterisation, exit
     criteria §8.

S7.  Experiment 6 becomes runnable: CE across the replication-onset boundary.
```

---

## 11. Open design questions (resolve during the build spec, not before)

1. **Trait set.** Which traits are genome-encoded? The plan's list
   (move_threshold, harvest_rate, maintenance_cost, max_lifetime,
   replication_threshold, exploration_rate, shell_affinity) plus the new
   `satiation` (§6.2). Start minimal — every trait is a dimension of the search
   space and an opportunity for degenerate solutions. Recommend starting with
   {harvest_rate, maintenance_cost, replication_threshold, satiation} and adding
   only when a mechanic demands one.
2. **Movement.** Is it needed for Experiment 5? A stationary carrier that
   harvests its own node and replicates onto it can, in principle, satisfy all
   seven exit criteria. Movement adds a large behavioural surface. **Recommend
   deferring movement to S6+** — get the energy economy stable first, then ask
   whether motility changes the carrying capacity.
3. **Catalyst and Shell motif kinds.** `MotifKind` has three variants but
   Experiment 5 needs only `Carrier`. Define the enum; implement only Carrier.
4. **Entropy-loss placement.** 20% loss on death (per plan). Should replication
   also lose energy (the cascade fixture's ×0.8)? Two loss channels compound;
   decide deliberately, and hold it fixed across the influx sweep or the sweep
   is uninterpretable.
5. **Where does `NonTrivial` get set?** Cleanest: the causal layer already
   detects replicators; the `MotifClassifier` tells it which classes are
   carriers; a replicator whose pattern is a carrier-class is `NonTrivial`.
   Confirm this composes without modifying the replicator detector itself.

---

## 12. Risk register

| Risk | Guard |
|---|---|
| C++ failure #1: abstraction blurring, unattributable bugs | dynamics in the rule file, bookkeeping in the crate; enforced by crate boundaries and the read-only Observer |
| C++ failure #2: motifs imposed on emergent geometry | carriers *are* hypergraph nodes; per-generation injection is not expressible |
| **New:** the rule file becomes an unauditable god-object | one named rule per biological process, purpose-commented, versioned with the crate; new rules must justify why they cannot be composed |
| **New:** carriers pollute substrate metrics | Q3 — label-aware metrics + same-seed bare-substrate control run |
| **New:** continuous genomes explode the WL class registry | Q2 — WL must ignore properties; verify before any ALife run |
| Population tuned into stability by hand | §8.1's forbidden-response column; the emergent-capacity commitment (§7.2) |
| Goodharting | none possible — there is no evaluator, only survival. This is *why* this layer precedes the tri-ecology. |

---

*Decision required before the build spec: approve the intrinsic architecture
(§1), the property-per-trait genome (§2.1), the emergent-capacity deviation
(§7.2), and the `Guard::PropertyCompare` extension (§5.3). Then answer Q1–Q4
(§9) — Q2 in particular, since a WL hash that includes properties would have to
be fixed before any ALife run is meaningful.*
