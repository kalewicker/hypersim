# hg-alife — Implementation Plan (Build Spec)

Build spec for the concrete ALife layer: genomes, carriers, node-funded
replication, lineage, ecology metrics. Target is **Experiment 5** (a stable,
spontaneously-replicating carrier population), which gates Experiment 6.
Companion to `hg_alife_scope_and_architecture.md` — read that for the *why*;
this is the *how*. Where they conflict, the scope doc's architecture governs
and this doc is wrong.

Instructions are staged (S0–S7); each stage gates the next. Fixture-first:
cheap gates before expensive commitments, energy-conservation assertions before
composition, hand-traced outcomes where feasible.

## Confirmed preconditions (Q1/Q2 resolved — do not re-litigate)

Verified against `compute_wl_hash` source:
- **WL hash incorporates node `Label`** (init and per-iteration hash
  `node.label.0`). Carriers with a distinct label ⟹ distinct WL classes. The
  `MotifClassifier` seam and `NonTrivial` detection are viable.
- **WL hash does NOT incorporate node properties.** Continuous genomes do NOT
  mint a class per individual; the registry does not explode. Finding F1 stays
  a caveat, not a catastrophe.

The required invariant (WL sees label, ignores properties) already holds. **The
one thing to confirm in S0** is a possible latent nondeterminism: the WL source
uses `DefaultHasher` (RandomState-seeded), not the `FxHasher` the
reproducibility policy standardized on. If `DefaultHasher` is still live, WL
hashes differ across processes and every cross-run comparison is invalid —
verify it was migrated to a fixed-seed hasher, and if not, that is S0's first
fix.

---

## S0 — Preconditions and metric-pollution decisions

No ALife code yet. Resolve these; each has a concrete deliverable.

### S0.1 WL hasher determinism (blocking if it fails)
Confirm WL hashing uses a fixed-seed hasher (`FxHasher`), not `DefaultHasher`.
Test: same graph, two processes, identical WL hashes. If it fails, migrate to
`FxHasher` and re-baseline — this is prerequisite to *any* cross-run claim,
ALife or not.

### S0.2 Carrier metric pollution (Q3)
Carriers become nodes, so `n_nodes`, `d_eff`, and the regime classifier will
include them. Decide and implement:
- **Substrate-only metric variants:** `d_eff` and node/edge counts computed over
  *substrate-labelled nodes only*, reported alongside the all-inclusive
  versions. The regime classifier should run on the substrate subgraph (a
  carrier bloom must not read as `Exploding` of the substrate).
- **Paired control discipline:** every ALife run is launched with a same-seed
  bare-substrate control, so any metric shift is attributable to biology, not to
  carriers-as-nodes inflating a count.

Deliverable: label-filtered metric functions + a run harness that always
produces the paired control.

### S0.3 Influx performance (Q4)
Confirm a guarded influx rule (matches only nodes below an energy cap) does not
blow up matcher cost at ~10⁴ nodes. This is the design anyway (§6.1 scope), so
the check is just: measure matcher time with the guard present. If unguarded
matching of all nodes was ever going to be the approach, it is rejected here.

---

## S1 — Substrate extension: `Guard::PropertyCompare`

The one blocking substrate change. Everything else composes verified machinery.

### S1.1 Grammar (`hg-rules`)
Guards today compare a property to a constant. Add property-to-property,
cross-variable comparison:

```toml
[[rules.lhs.node_property_compares]]
var_a = "carrier"
key_a = 10               # energy
op    = "gt"
var_b = "carrier"
key_b = 11               # replication_threshold (a genome trait)
```

### S1.2 Types + eval (`hg-rules`, `hg-engine`)
```rust
Guard::PropertyCompare { var_a: usize, key_a: u32, op: GuardOp,
                         var_b: usize, key_b: u32 }
```
Evaluation mirrors the existing `NodePropertyOp` exactly:
- resolve both variables to bound nodes, read both properties;
- **variant mismatch ⟹ guard false** (no coercion, no panic);
- **missing property on either side ⟹ false**;
- `var_a`/`var_b` may be the same variable (self-comparison across two keys) or
  different (cross-node, e.g. `carrier.energy >= substrate.child_node_cost` if a
  cost were stored per-substrate-node).

### S1.3 Determinism and regression
`PropertyCompare` reads state, draws no RNG, adds no non-determinism. **Gate:
canonical_hash regression** — every existing builtin (triangle_subdivide,
binary_edge, growth_treadmill, glider, white_noise) produces a bit-identical
final hash to pre-S1. A rule not using `PropertyCompare` must be entirely
unaffected. This is the same regression discipline `property_effects` passed;
record the pre-S1 hashes first, assert against them.

### S1.4 Tests
- Each op (`gt`/`lt`/`ge`/`le`/`eq`/`ne`) on Float-vs-Float, both directions.
- Variant mismatch (Int vs Float) ⟹ guard false, rewrite does not fire.
- Missing key ⟹ false.
- Self-comparison (same var, two keys) and cross-var comparison.
- The regression suite of S1.3.

---

## S2 — `hg-alife` skeleton (no dynamics)

Crate per the scope doc's §4 layout. Deps: `hg-core`, `hg-observe`,
`hg-causal`, `hg-schema`. **Never `hg-engine`** — enforce by the absence of the
dependency; a compile failure if anyone adds it is the desired behavior.

### S2.1 `genome.rs` — trait keys and encoding

Property keys are a shared namespace; register every one in
`docs/property_registry.md`. Proposed starting allocation (energy is the
existing key 1; genome traits get a dedicated block):

```
key  1  energy                 Float   (existing convention)
key 10  harvest_rate           Float   genome
key 11  replication_threshold  Float   genome
key 12  maintenance_cost       Float   genome
key 13  satiation              Float   genome   (the §6.2 crowding trait)
key 20  carrier_marker         Bool    bootstrap seed marker (§6.3)
key 21  lifetime               Float   carrier age counter (counts down)
```

Start with the **minimal trait set** {harvest_rate, replication_threshold,
maintenance_cost, satiation} (scope §11 Q1). `move_threshold`,
`exploration_rate`, `shell_affinity` are deferred until a mechanic demands them
— every trait is a search-space dimension and a degenerate-solution opportunity.

```rust
pub mod keys {
    pub const ENERGY: u32 = 1;
    pub const HARVEST_RATE: u32 = 10;
    pub const REPLICATION_THRESHOLD: u32 = 11;
    pub const MAINTENANCE_COST: u32 = 12;
    pub const SATIATION: u32 = 13;
    pub const CARRIER_MARKER: u32 = 20;
    pub const LIFETIME: u32 = 21;
}

/// Trait bounds for jitter clamping and initial-genome sampling. A jittered
/// trait is clamped to its [min, max] so mutation cannot drive a trait
/// out of its meaningful range. These bounds live here, in code, not in the
/// rule file — the rule file does jitter; the bounds are genome semantics.
pub struct TraitBounds { pub key: u32, pub min: f64, pub max: f64 }
pub fn trait_bounds() -> &'static [TraitBounds] { /* the four traits' ranges */ }
```

**Note on jitter clamping:** the `jitter` op adds uniform noise but does not
clamp. Either (a) clamp inside the effect application against `trait_bounds`
(requires the engine to know bounds — rejected, it couples the engine to ALife),
or (b) accept unclamped traits and guard against pathological values
behaviorally (a negative harvest_rate simply never harvests). **Choose (b):**
keep the engine ALife-agnostic; let biology be robust to out-of-range traits.
Document that traits can drift out of nominal range and that rules must behave
sanely when they do. Revisit only if unclamped drift destabilizes runs.

### S2.2 `motif.rs`
```rust
pub enum MotifKind { Carrier, Catalyst, Shell }  // define all; implement Carrier only
```
Carrier identification: a node whose `Label` is the carrier label. A helper
`is_carrier(node) -> bool` and `carrier_energy(node) -> Option<f64>`.

### S2.3 `classifier.rs` — `HypergraphMotifClassifier`
Implements `MotifClassifier`. Classifies on **label-bearing WL class only**
(scope §5.2): carrier-class ⟹ `MotifKind::Carrier`'s class id; everything else
⟹ substrate. Does *not* read energy/genome — motif kind is structural, energy
is state (scope §5.2). Injected into `CausalLanguageObserver` in place of
`NullMotifClassifier` by `hg-sim-core`.

Gate S2: a hand-built graph with one carrier node and several substrate nodes —
`HypergraphMotifClassifier` returns the carrier class for the carrier and the
substrate class for the rest.

---

## S3 — `alife_base.toml`: the biology, one rule at a time

**This file is the ecology.** Every rule is one biological process, named, with
a purpose comment stating which process it implements and why it is not
composable from existing rules (scope §4.1 — the god-object risk). Build and
test each rule *in isolation on a fixture substrate* before composing them.

Per-rule gate: an **energy-conservation assertion** (the `conservation_soak`
method — sum of energy over all nodes changes only by the declared amount) plus
a hand-traced fixture where feasible.

### S3.1 `bootstrap_seed` (creation of the first carrier)
The generator sets `carrier_marker = true` (Bool) on exactly one substrate node.

```toml
[[rules]]
name = "bootstrap_seed"   # one-shot: turn the marked substrate node's neighbor
                          # into a carrier. Cannot be a creation rule (those make
                          # isolated nodes); must match the marked node to attach.
[rules.lhs]
nodes = ["site"]
edges = []                # matches a lone marked node; see note
[[rules.lhs.node_property_guards]]
var = "site"; property = 20; op = "eq"; value = true    # carrier_marker
[rules.rhs]
nodes = ["site", "carrier"]
edges = [["site", "carrier"]]
[[rules.rhs.property_effects]]
var = "carrier"; key = 1; op = "set"; value = 10.0       # seed energy
# ... set the four genome traits to initial values via `set` ...
[[rules.rhs.property_effects]]
var = "site"; key = 20; op = "set"; value = false        # clear the marker
```

Note: an empty-LHS-edges rule matching a single node needs the matcher to
support single-node patterns (no edge). Confirm this is expressible; if the
grammar requires ≥1 edge in a pattern, the marker node must be pre-attached to a
partner in the generator and the rule matches that 1-edge. Resolve in S3.1
before proceeding.

Gate: exactly one carrier appears, attached to the seeded site, marker cleared;
running many steps creates no further carriers from this rule.

### S3.2 `harvest` (energy: substrate node → carrier)
```toml
# Carrier draws energy from its occupied substrate node, gated by satiation so a
# full carrier yields to a hungry cohabitant (scope §6.2).
[rules.lhs] nodes = ["site", "carrier"]; edges = [["site", "carrier"]]
# guards: carrier is a carrier (label handled structurally via the matched
#   pattern shape / a carrier-label constraint), site has energy to give,
#   carrier.energy < carrier.satiation  (PropertyCompare — the S1 extension)
[[rules.lhs.node_property_compares]]
var_a="carrier"; key_a=1; op="lt"; var_b="carrier"; key_b=13   # energy < satiation
# effect: move a fixed quantum (or harvest_rate-scaled) from site to carrier
[[rules.rhs.property_effects]]
var="site";    key=1; op="subtract"; value= <quantum>
[[rules.rhs.property_effects]]
var="carrier"; key=1; op="add";      value= <quantum>
```
Harvest quantum: to make it genome-dependent (`harvest_rate`) needs
`scale_from` reading `harvest_rate` — express as `carrier.energy +=
site.energy × harvest_rate`? That reads *two* sources. Check expressibility: a
single `scale_from` reads one source×factor. Harvesting a genome-scaled fraction
of *site* energy into *carrier* is `scale_from(source=site.energy,
factor=harvest_rate_as_constant)` — but factor is a constant, not a property.
**This is a real expressibility limit:** genome-scaled harvest needs a
property×property effect that does not exist. Options: (a) fixed harvest quantum
for Experiment 5 (harvest_rate deferred as an active trait), or (b) add a
`scale_from_property` effect (factor read from a property). **Recommend (a) for
Experiment 5** — a fixed quantum still gives a working energy economy; defer the
genome-scaled harvest (and its effect-op extension) until the economy is stable
and harvest_rate's selective effect is actually wanted. Record as a deviation.

Gate: energy conservation exact (site loses exactly what carrier gains); a
satiated carrier stops harvesting.

### S3.3 `metabolise` (carrier energy decay)
```toml
[[rules.rhs.property_effects]]
var="carrier"; key=1; op="subtract"; value= <maintenance quantum>
```
Same expressibility note: genome-dependent `maintenance_cost` needs
property-scaled subtraction. For Experiment 5, a fixed maintenance quantum;
`maintenance_cost` as an active trait deferred with harvest_rate. Gate: carrier
energy strictly decreases when not harvesting.

### S3.4 `die_and_recycle` (death with 80% recovery)
```toml
# A carrier with energy <= 0 (or lifetime expired) dies: delete the carrier,
# return 80% of a reference energy to the site, 20% entropy loss.
[rules.lhs] nodes = ["site","carrier"]; edges = [["site","carrier"]]
[[rules.lhs.node_property_guards]]
var="carrier"; property=1; op="le"; value=0.0
[rules.rhs]
nodes = ["site"]          # carrier absent from RHS ⟹ deleted; edge auto-cleaned
[[rules.rhs.property_effects]]
var="site"; key=1; op="add"; value= <recycle amount>
```
The recycle amount: 80% of *what*? If of the carrier's energy — but a dying
carrier has energy ≤ 0. The 80% must be of a **birth/reference energy**, not
current. Simplest for Experiment 5: recycle a fixed fraction of the seed/birth
energy (a constant), 20% lost. Record the exact energy-accounting choice; it
must be fixed across the influx sweep or the sweep is uninterpretable (scope §11
Q4). Gate: carrier node gone, occupancy edge gone (Contract Rule 5), site energy
increased by exactly the recycle amount.

### S3.5 `influx` (energy enters the closed economy)
```toml
# A depleted substrate node gains energy, up to a cap. This is the sole energy
# SOURCE; carrying capacity emerges from the balance of influx vs metabolic
# drain + entropy loss (scope §6.1). Guarded so only depleted nodes match
# (also the S0.3 performance fix).
[rules.lhs] nodes = ["site"]; edges = []   # or the 1-edge form per S3.1's resolution
[[rules.lhs.node_property_guards]]
var="site"; property=1; op="lt"; value= <influx cap>
[[rules.rhs.property_effects]]
var="site"; key=1; op="add"; value= <influx rate>
```
`<influx rate>` is **the** swept parameter for Experiment 5 (scope §7.2). Gate:
depleted nodes refill, capped nodes do not match, bookkeeping conserved (influx
is a deliberate source — it is the one place energy is *created*, and the
accounting must record how much, so the closed-economy check is "energy in via
influx = energy out via entropy loss + stored delta").

### S3.6 `replicate` (node-funded, the core mechanic)
The mechanic the whole `property_effects` seam was built for. Node-funded: the
*substrate node* pays, parent energy unchanged (the C++ fix).

```toml
# Carrier replicates: if carrier.energy >= replication_threshold AND the site has
# >= child cost, spawn a child carrier on the site with inherited+jittered genome.
# The SITE pays the child cost; the parent's energy is untouched.
[rules.lhs] nodes = ["site","parent"]; edges = [["site","parent"]]
[[rules.lhs.node_property_compares]]
var_a="parent"; key_a=1; op="ge"; var_b="parent"; key_b=11   # energy >= repl_threshold
[[rules.lhs.node_property_guards]]
var="site"; property=1; op="ge"; value= <child_node_cost>     # site can pay
[rules.rhs]
nodes = ["site","parent","child"]
edges = [["site","parent"], ["site","child"]]
[[rules.rhs.property_effects]]
var="site";  key=1; op="subtract"; value= <child_node_cost>   # SITE pays
[[rules.rhs.property_effects]]
var="child"; key=1; op="set";      value= <child_birth_energy> # e.g. cost×0.8
# inherit each trait: copy_from parent, then jitter
[[rules.rhs.property_effects]]
var="child"; key=11; op="copy_from"; source_var="parent"; source_key=11
[[rules.rhs.property_effects]]
var="child"; key=11; op="jitter";    magnitude= <mut_mag>
# ... repeat copy_from+jitter for keys 10,12,13 ...
```

This is the intrinsic realization of `try_replicate` from `not_nothing_v2_plan`
§7.3 — node-funded, mutation via jitter (the continuous-genome deviation, scope
§7.1), inheritance via copy_from. Parent energy genuinely untouched (it is not a
target of any effect). Gate: a hand-traced fixture — parent above threshold,
site funded ⟹ child appears with parent's traits ± jitter, site energy down by
exactly child_node_cost, parent energy unchanged; the sequential-semantics
tripwire (child reads parent's genome before any effect could alter it).

### S3.7 Composition order and conflict interaction
All rules are `MaxParallel`. Two carriers on one site conflict on the site node
⟹ only one acts per step (the free crowding mechanic, scope §6.2). Deterministic
conflict resolution means satiation gating (S3.2) is what rotates access —
without it, one carrier monopolizes. Confirm the composed rule set's priority
ordering does not create a pathology (e.g. `die_and_recycle` must be able to fire
on a carrier that also matches `metabolise`; order so death is not starved).

---

## S4 — First full ecology run (expect failure)

Compose all S3 rules, seed one carrier, run. **Expect explosion or extinction** —
that is the normal first outcome, and the failure table (scope §8.1) says how to
read it. Do not tune blindly; diagnose against the energy economy. The forbidden
responses (adding a global cap, hand-injecting fitness, lowering thresholds
without diagnosing the budget) are forbidden here too — re-read scope §8.1's
right column.

Deliverable: a characterization of *why* the first run failed (which term
dominates — influx > drain ⟹ explosion; drain > harvest ⟹ extinction) and the
parameter adjustment justified by that diagnosis.

---

## S5 — Observer: lineage and ecology metrics

### S5.1 `AlifeObserver` (`impl Observer`, read-only)
Consumes `RewriteEvent`s. A replication event is recognizable: an event of the
`replicate` rule id, whose RHS created a carrier node. From the event stream
alone, maintain:

### S5.2 `lineage.rs` — `LineageTree`
```rust
pub struct LineageTree {
    parent:     AHashMap<NodeId, NodeId>,    // child carrier -> parent carrier
    genome_at:  AHashMap<NodeId, Genome>,    // traits at birth (from the event)
    birth_step: AHashMap<NodeId, u64>,
    death_step: AHashMap<NodeId, Option<u64>>,
}
```
`export_newick`, `clade_count`, `clade_survival_curve` (per
`not_nothing_v2_plan` §7.5). The parent carrier of a birth: the `parent`
variable's node in the `replicate` event. Death: a `die_and_recycle` event's
deleted carrier.

**Subtlety:** carriers are identified by `NodeId`, but a `replicate` event's
child is a freshly created node — its id is in the event's created-nodes. The
parent is a surviving node in the same event. Map both from the event's node
lists; this is why replication must be a *distinct rule id* (so the observer can
filter for it) rather than one of several rules that happen to add a node.

Gate: lineage reconstructable from events alone; parent/child links sound on a
hand-traced 3-generation fixture; Newick parses in a standard tool.

### S5.3 `metrics.rs` — `EcologyMetrics`
Per window: carrier count, carriers/substrate-node ratio, births this window,
deaths this window, replication-origin vs bootstrap-origin counts, and per-trait
mean+variance across the live carrier population. These feed the CSV (new
columns) and the report's ecology section, and are the quantities Experiment 5's
exit criteria are read from.

---

## S6 — Experiment 5 proper

### S6.1 Influx sweep and stability characterization
Sweep `<influx rate>` (S3.5). For each, run to the run window and record: does
the population explode / go extinct / stabilize; the settled carriers/node ratio
and its late-window variance; the per-trait distribution trajectory.

### S6.2 Exit criteria (all must hold, fixed seed, reproducible) — scope §8
1. Spontaneous replication from one ancestor (no injection — structurally
   impossible).
2. Bounded (no `Exploding`; no extinction in-window).
3. Stable ratio, low late-window variance. **Report the value; 0.283 is a
   comparison datum, not a threshold** (scope §7.2).
4. Multi-generation lineage: clade_count > 1, ≥1 clade persisting > K
   generations.
5. Selection acting: trait distributions *shift* (directional mean change or
   variance collapse), not merely drift.
6. `non_trivial_replicator_count()` goes **0 → >0** — the bridge to Experiment 6
   and the validation of the replicator taxonomy.
7. No substrate regressions: determinism holds; energy books balance to the
   declared entropy loss; no-`alife`-feature build bit-identical to current
   substrate.

### S6.3 Movement decision
Deferred to here (scope §11 Q2). If a stationary carrier satisfies all seven
criteria, **movement is not needed for Experiment 5** — record that finding and
leave motility for a later phase. Only add a `move` rule (rewire the occupancy
edge) if stationarity provably prevents a criterion.

---

## S7 — Experiment 6 becomes runnable

With a stable population and `NonTrivial` replicators detectable, CE across the
replication-onset boundary is measurable. The functional coarse-graining now has
its real partition (carrier / substrate over the same micro state space). This
is a separate experiment doc; S7 is only the statement that S6 unblocks it.

**The pre-committed Experiment-6 caveat still stands (Finding F1):** CE is biased
*downward* on open registries, so a CE > 1 result clears a conservative bar
(strengthened), but a real effect could be masked. Do not tune to produce CE > 1.

---

## Deviations from `not_nothing_v2_plan.md` (consolidated, all pre-committed)

1. **Continuous genome (property-per-trait) + jitter mutation**, not bit-packed
   `Genome(u32)` (scope §2.1, §7.1). Forced — guards cannot decode bits.
2. **Emergent carrying capacity** from a closed energy economy, not
   `SOFT/HARD_CAPACITY` (scope §6.1, §7.2). 0.283 demoted to comparison datum.
3. **Satiation-gated harvest**, not `harvest/sqrt(cohabitants)` (scope §6.2,
   §7.3). Crowding via conflict resolution + a satiation trait.
4. **Fixed harvest/maintenance quanta for Experiment 5**, not genome-scaled
   (S3.2, S3.3) — genome-scaled versions need a property×property effect op that
   does not exist; deferred until the economy is stable and the traits'
   selective effect is wanted.
5. **Recycle/birth energies referenced to birth energy** (constants), not
   current carrier energy (S3.4, S3.6) — a dying carrier has no energy to
   recycle a fraction *of*.

---

## Scope discipline

- `hg-alife` **never** depends on `hg-engine`.
- Dynamics live in `alife_base.toml`; bookkeeping in the crate. A behavior is
  attributable to one or the other, never smeared.
- Every ALife run is paired with a same-seed bare-substrate control (S0.2).
- No global state, no per-generation injection — neither is expressible, and
  that is the point.
- The forbidden-response column of scope §8.1 is binding: do not reach for
  global regulation or hand-tuned fitness to hide a failure of the emergent
  economy.

## Report back at each gate

S0: WL hasher determinism result; the metric-pollution decisions made.
S1: the pre/post canonical hashes proving no builtin regressed.
S2: classifier distinguishes carrier from substrate on the fixture.
S3: per-rule energy-conservation results + the replicate hand-trace verbatim.
S4: the diagnosis of the first run's failure mode.
S5: lineage soundness on the 3-generation fixture; Newick validity.
S6: the influx sweep table + which exit criteria pass, with the settled ratio
    and its variance, and — the headline — whether
    `non_trivial_replicator_count()` crossed 0 → >0.
```
