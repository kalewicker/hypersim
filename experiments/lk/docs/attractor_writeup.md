# Locality as a Balance, Not a Geometry

*An attractor characterization of the LK-R rule family.*
*Companion to the substrate characterization report; LK track, v8 / Phase B4.*
*Draft 2026-07-17; revised 2026-07-18 (v6 coexistence), 2026-07-19 (v7 audit),
2026-07-30 (v8 — measurement back-action removed and quantified).*

---

## Abstract

Within the energy-metered LK-R rule family on a triangulated torus, spatial
locality — the absence of long-range shortcuts ("wormholes") in the graph
metric — is **not a fixed property of the geometry**. It is a property of the
balance between two dynamical processes: *extrusion*, the production of
1-D filament structure by metered subdivision, and *removal*, the
reabsorption of that filament by generation-guarded fusion. Locality is an
attractor exactly when a removal process exists and outpaces extrusion; the
geometry dissolves into a wormhole-dense phase otherwise. We exhibit both
halves — a dissolution phase (v2–v3) and a removal-dominated attractor with
a measured decay constant λ = ln(3/2) and a structurally bit-restored
endpoint (R1) — and we map the boundary between them. An *energy*-gated
removal rate turns out to be bistable (R3): it reads a variable that is
delta-distributed at a conduction plateau, so it can only be off or
saturated. A *lifetime*-gated rate, whose gate variable has dispersion by
construction, produces genuine **coexistence** (A4): a bounded standing
population held against sustained production, scaling with the lifetime k
according to **L\* = β·k·p**, verified for k ∈ [10, 300] across a 4× range
of energy budget.

That law is reported here from runs whose clock is *passive*. An earlier
version of this work timed the lifetime with a rule — one that, having a
single-node left-hand side at high priority, won every scheduling contest
it entered and became the dominant actor in the dynamics it was added to
observe. Replacing it with a clock derived from state the store already
holds changes the answer (B2): the blocking prefactor β drops from
1.24–2.00 to **1.03–1.18**, and a "hard structural cap" previously
reported for seed-monopolised production — a standing population pinned at
12, independent of k over a 30× range — **disappears entirely**, with
L\*/k returning to 0.99 at every k. Production serialization at a single
anchor is real and is measured here cleanly; what it pins is the
production rate, not the standing population. The size of that correction,
cell by cell, is this work's quantitative statement of measurement
back-action, and is offered as its most transferable result.

Two limits are stated here rather than buried. **The closure defect — the
leakage-in-bits metric this work is named for — remains unmeasured**,
after four attempts blocked for four distinct reasons, each identified
and quantified rather than argued around. That history is reported as a
result in its own right (§6): the bars were set in advance and never
moved, and each attempt converted a suspicion into a number. The
sample-size precondition, which blocked the third attempt, now **passes**
— effective N is 239–249 against a floor of 200, on dynamics no
instrument participates in. What blocks the fourth is stationarity, and
the obstacle is a mask rather than the regime: the pre-registered stratum
admits both the fill ramp and the quench of a metered run, because its
production clause tests a *rising counter* rather than a *steady rate*.
The plateau between them is the tightest steady state this work has
produced (standing population 420 ± 6.5 over ~10⁵ steps), and is
deliberately **not** adopted as a stratum here, since selecting a window
after seeing where the variance is small is the error this programme has
already paid for twice. Separately, for most of the parameter range
explored, the standing structure that coexists lies **below the threshold
at which this work calls anything non-local at all** — what balances is
sub-threshold refinement, with a genuine wormhole population appearing
only at the largest removal lifetimes. Every claim is conditional on the
scaffolding named in §1.

---

## 1. Scaffolding conditionality (stated once)

Everything below holds *within* a specific, imposed scaffolding, and nowhere
do we claim these are emergent. Stated once, precisely, and not re-hedged
per paragraph:

- **Dissipative metering.** Subdivision costs a fixed quantum `c` per event,
  drawn from a finite seeded budget `E₀` placed at one site. Boundedness of
  the structural phase is *imposed* by this dissipation, not discovered.
- **Seeded budget.** All energy enters at t=0 at a single site; there is no
  influx. Every process is ultimately budget-limited, and every run quenches.
- **Deterministic MaxParallel.** The engine fires a maximal node-disjoint set
  of rule bindings each step, with a deterministic tie-break. There are no
  temporal "eras" in the scheduler — production and removal are concurrent
  processes, ordered only where they contend for a shared node (§4 depends on
  this).
- **Degree-guarded fusion.** The removal rule reabsorbs only degree-2
  midpoints; the original lattice edges and any planted handles are
  non-removable by construction. The attractor is therefore an attractor of
  the emergent structure, over a fixed lattice skeleton.

  *The operative guard is degree, not generation, and this sentence said
  otherwise until v10.* `lk_fuse` carries both guards, but the skeleton's
  protection is entirely the degree guard's: no rule in the family can
  change any node's degree (subdivision preserves endpoint degree, fusion
  removes a midpoint and rejoins its endpoints, transport is topology-inert),
  so a gen-0 node sits at its birth degree — 6 on the torus — permanently,
  and can never meet `degree == 2`. Audited across 19,323 persisted
  snapshots: **0** gen-0 nodes at degree 2 in 19.2M observations, and **0**
  degree-guard rejections in 8.25×10⁸ evaluations across 182 runs. The
  generation guard is real but redundant; deleting it would admit nothing.
  Full record and its consequences for skeleton erosion:
  `docs/guard_shadow_audit_v10.md`.

Reference parameters: θ=100 (subdivision threshold), q (child energy), c=10
(dissipation), τ (transport quantum), on a 32² torus. The exact energy
audit `E_total + c·N_sub + Σ E_fused = E₀` holds to a residual of exactly
0.0 at every window of every run cited here; any nonzero residual halts
interpretation, and none occurred.

---

## 2. The dissolution side (v2–v3)

With production alone (metered subdivide + downhill transport, no removal),
the seed is the only site that ever re-reaches θ, and it extrudes **1-D
filaments** — chains of degree-2, generation>0 nodes — radially into the
lattice. Six filaments form, one per lattice direction off the seed, and
each subdivision event adds exactly one link to one filament: filament
production and wormhole production are the *same event*.

Each interior link of a filament is a graph-metric shortcut. Its detour —
the shortest return path when the link is cut — is **2L+2**, not L+2 (the
seed's own bulk edges are consumed into the filaments, so the return path
must descend a second inflated chain). This retrodiction — a zero plateau
of wormhole count until a filament crosses the R_max/2 detour threshold,
then a slope-1 rise through the origin — matches the measured census
(slope 1.02–1.04). The era is cumulative: the graph crosses from a sparse
to a wormhole-dense phase as the filaments lengthen, and **growth dissolves
its own locality**. This is the dissolution phase: production drives the
geometry away from locality, monotonically, until the budget quenches.

---

## 3. The attractor side (v4 + R1)

### 3.1 Suppression at birth (v4)

Introduce the removal rule (fuse: a degree-2, generation≥1 midpoint is
removed and its endpoints rejoined, its energy vanishing onto the ledger)
and run production + removal together. Under MaxParallel there are no eras:
a fuse binding disjoint from a growth binding fires in the *same* step.
The result is **suppression at birth** — the emergent-wormhole population is
zero at every window, in every clean arm, both priority regimes. Roughly
83% of the endowment exits through the fused ledger; only the sink (the
seed's sub-θ remainder) survives. This is the attractor in its degenerate
limit: no population ever accumulates, so there is nothing to decay. The
demonstration was half complete — an attractor with no observed basin flow.

### 3.2 The basin: exact exponential relaxation (R1 / EXP-L15)

To exhibit the missing half we start from a *populated* far-from-attractor
state — the archived post-quench pump state, six filaments of L₀ = 785
links, interiors flat at q so production is dead — and enable removal.

- **Decay constant.** Fusion under MaxParallel selects a maximal *node*-
  disjoint set of interior triples {x, w, y}; on a filament this is every
  third interior node, so length obeys L(t) = L₀·(2/3)ᵗ exactly. The fitted
  rate is **λ = ln(3/2) ≈ 0.405 per step** — the family's headline number.
  (The naive prediction of alternate-node halving, λ = ln 2, assumed
  *edge*-disjoint bindings; the node-disjoint scheduler is what makes it
  2/3. Same concurrency-semantics correction as §3.1.)
- **Restoration endpoint.** When a filament reaches length 1 its last node
  is a fusable degree-2 midpoint, and its removal restores the original
  lattice edge. The terminal state is **structurally bit-identical to the
  clean lattice** (whole-graph WL hash equality, label- and property-blind:
  1024 nodes, 3072 edges). Every joule of filament energy is reabsorbed;
  the restored lattice is energy-dead and ledgered exactly.
- **Census follows.** The wormhole count decreases monotonically to zero
  once the longest filament drops below the R_max/2 = 6 detour threshold.

Removal dominates from a populated start and drives the geometry back to
the clean lattice at a measured, mechanism-derived rate. With §3.1's
suppression limit and §3.2's basin flow, the attractor demonstration is
complete: **locality is an attractor of the LK-R family whenever removal
exists.** Concurrent production during relaxation (a fresh budget seeded at
a new site) does not interact — the old population decays at λ while the
new production is suppressed at birth, ledger exact throughout.

---

## 4. The balance, part I (R3 / EXP-L16): an energy gate is bistable

The remaining question is whether a *partial* removal rate holds a nonzero
population in steady turnover — a balance point between dissolution and the
attractor. We throttle fusion with an energy gate `f_cap`: a filament node
is fusable only once its energy drops below `f_cap`. The intent was an
aging timer — a fresh child carries q, and drains toward f_cap over
`t_age ≈ (q − f_cap)/τ` steps — with standing population ∝ t_age × rate.

**The knob is structurally inert, and the reason is a clean negative
result.** Downhill-only metered transport with a 2τ dead zone never
produces a filament node below q: a fresh child at energy q is conducted
*up* to a plateau (≈65 at (q,τ)=(50,15); flat at q for (75,25)), not drained
down. The interior energy distribution is a near-**spike** at the plateau,
not a spread. So the guard's admitting region (energy < f_cap < q) is
disjoint from the interior energy support (≥ q), and at every f_cap below
the plateau the throttled run is **bit-identical to the fuse-disabled
control** — pure growth, fuse never firing (guard rejects = evals, ledger
empty). Any f_cap above the plateau admits the entire spike at once,
recovering unthrottled suppression (§3.1). The transition is a **jump with
no band between** — bistability, with the threshold at the conduction
plateau, not at the naive q.

The straddle at (q,τ)=(50,15), plateau ≈ 65, is unambiguous:

| f_cap | side | verdict | Σ E_fused | churn | steady census |
|------:|------|---------|----------:|------:|--------------:|
| 60 | below plateau | STEADY (frozen) | 0 | 0 | 1206 |
| 66 | above plateau | EMPTY (suppressed) | 80,275 | 958 | 0 |
| 80 | above plateau | EMPTY (suppressed) | 80,275 | 958 | 0 |
| 100 | above plateau | EMPTY (suppressed) | 80,275 | 958 | 0 |

Below the plateau the ledger is empty (fuse never fires, identical to
pure growth); a single step above it the entire endowment routes through
the ledger and the population is suppressed. The three above-plateau
columns are bit-identical (Σ E_fused and churn to the unit), confirming
the gate saturates the moment it admits the spike. There is no f_cap that
yields an intermediate standing census.

This is the pre-committed "energy gate too coarse" row of the plan: an
energy gate cannot be a *rate* control in this family, because the state
variable it reads (interior energy) has no spread for the gate to select a
fraction of, and never enters the sub-q region the aging-timer premise
assumed.

**The scope of this negative is narrower than its language.** What is
demonstrated is that *a threshold on a delta-distributed variable is a
step function* — which is nearly a tautology once the distribution is
measured. It does not show that a fractional removal rate is impossible,
only that this knob cannot express one. That distinction is what §5
tests, and the answer changes the conclusion.

---

## 5. The balance, part II (A4 / EXP-L17, re-measured as EXP-L18): a lifetime gate coexists

Replace the energy gate with a **lifetime** gate whose variable has
dispersion by construction: a node becomes fusable exactly k steps after
its birth. Nodes born at different times therefore have different ages —
the standing population's age distribution is uniform on [0, k].
Elementary balance predicts a standing population L\* = k·p for production
rate p.

**Two mechanisms implement that gate, and the choice between them is not
neutral.** The first (v6) endowed each child with `countdown = k` and
added a single-node rule to decrement every live countdown once per step.
The second (v8) derives the age from the creation step the store already
records on every node, and adds no rule at all. They agree step-for-step
on a scripted throat — a link fuses on exactly the same step under both —
so they are the same gate in the only sense the physics should care
about. They do not agree on the measurement, because the first one
*competes for bindings* (§5.2).

**Everything numbered below is from the passive-clock runs.** The ticked
numbers are retained in the methods appendix as the back-action case, and
the difference between the two is the subject of §5.2.

Two measurement corrections were required before the rows could be read,
both recorded in the findings and both of the same family this document
is about (§7):

1. The first driver defined its steady segment as the second half of the
   run, which at E₀ = 1e5 lies entirely *post-quench* — p = 0 there, so
   L\* = k·p = 0 held trivially and every cell read "suppressed". A
   pre-registered statistic satisfied without the mechanism being tested.
2. The wormhole census reads **0 in most countdown cells** — but that is
   the instrument's detection domain, not the population. A link
   registers as a wormhole only once its detour exceeds R_max = 12, and
   the detour grows with chain length. The standing population is
   therefore read as the generation > 0 node count, which has no
   threshold.

   Two things about that domain were established only on audit, and both
   correct earlier statements. First, **the detour law is
   regime-dependent**: measuring detour directly against chain length
   gives ≈ 2L+2 in the pump — the F-LK14 formula, which holds there
   because the seed's own bulk edges are consumed into chains so a return
   path must descend a *second* chain — but ≈ **L+3** in the swap regime,
   where the ambient lattice around a filament is intact. Visibility at
   R_max = 12 is thus L ≥ 6 in the pump and **L ≥ 10 in swap**, and using
   the pump formula everywhere would misclassify a cell that reads zero
   as being in range. Second, the zeros are **threshold zeros, not
   sampling zeros**: an exact, unsampled count over every
   filament-incident edge agrees with the shipped 256-edge sampler in
   every cell, including reading exactly 0.0 wherever it reads 0.0.

   The morphology is also not what the population's size suggests. Under
   removal the swap population is **many short structures, not a few long
   ones**: the chain count grows with k (12.8 → 55.5 in transfer) while
   the **median chain length is 1 at every k**, with a heavy tail (max
   80.9 at k = 300). Only 7–14% of chains are in range even in the cells
   that register. So for k ≤ 30 — and in the pump at every k — the
   coexistence regime is **entirely local by this document's own
   definition**: what coexists is sub-threshold refinement. A genuine
   standing wormhole population (54–165 edges, exact) appears only at
   k = 100–300 in swap.

**Result: coexistence exists.** Every lifetime-gated cell holds a bounded
standing population under sustained production, while both no-removal
controls run away to the full era's output (1514 and 1490 links — every
link ever made). A lifetime gate does what the energy gate could not: it
holds a population instead of choosing between nothing and everything.

**The standing population grows with k in both regimes** (passive clock,
E₀ = 1e5):

| k | pump L\* | pump L\*/k | swap L\* (transfer) |
|---:|---:|---:|---:|
| 10 | 9.9 | 0.986 | 11.0 |
| 30 | 29.9 | 0.995 | 32.8 |
| 100 | 99.9 | 0.999 | 80.7 |
| 300 | 299.8 | 0.999 | 171.7 |

In the pump the law is **exact**: L\* = k to three significant figures at
every k, with production pinned at p = 0.993–0.994 events/step — the
one-event-per-step limit that node-disjoint scheduling imposes when only
one node ever exceeds θ. In swap, production is distributed and p varies
with k, so L\* follows β·k·p rather than k (§5.1).

**A prior version of this section reported the opposite for the pump** —
a standing population pinned at 12, flat across a 30× range of k, with a
mechanism (§5.3) built to explain the flatness. That result was an
artifact of the ticked clock and does not survive its removal. Under the
tick, p fell as 1/k (0.541 → 0.020) and the pump never spent its energy
budget: `n_subdivisions` reached 1666 — the full E₀/(q+c) — in every
passive cell, but only 1164 / 360 / 120 at k = 30 / 100 / 300 under the
tick. What was read as a structural fixed point was a clock throttling
the process it was timing. The retraction is §5.3.

### 5.1 The balance law, and why the obvious statement of it is wrong

The natural reading of the swap column is L\* ∝ k. It does not survive
an audit at a second budget. Measured with the exact standing census
under the passive clock:

| E₀ | k | L\* | p | L\*/k | L\*/(k·p) | 1/(1−f) from the expired share |
|---|---:|---:|---:|---:|---:|---:|
| 1e5 | 10 | 11.0 | 1.034 | 1.10 | **1.068** | 1.227 |
| 1e5 | 30 | 32.8 | 0.926 | 1.09 | **1.180** | 1.280 |
| 1e5 | 100 | 80.7 | 0.773 | 0.81 | **1.044** | 1.065 |
| 1e5 | 300 | 171.7 | 0.556 | 0.57 | **1.029** | 1.017 |
| 4e5 | 10 | 14.4 | 1.333 | 1.44 | **1.083** | 1.195 |
| 4e5 | 30 | 45.0 | 1.275 | 1.50 | **1.176** | 1.229 |
| 4e5 | 100 | 121.3 | 1.155 | 1.21 | **1.050** | 1.063 |
| 4e5 | 300 | 307.4 | 0.992 | 1.02 | **1.033** | 1.017 |

`L*/k` is not a constant: it is 0.81 at (k=100, E₀=1e5) and 1.21 at the
same k with four times the budget, because it silently carries the
production rate p, which is budget-sensitive. The budget-invariant
quantity is `L*/(k·p)`. Stated with its domain:

> **L\* = β(k) · k · p**, where β(k) is a **blocking factor** (§5.2),
> 1.03–1.18 and *falling* with k, budget-independent to 1.5%; and p is
> the realized production rate, measured per cell rather than predicted.
> Verified for k ∈ [10, 300] and E₀ ∈ [1e5, 4e5] in swap × transfer;
> **β = 1.00 in the pump**, where the law reduces to L\* = k·p exactly.

That exactness is **measured and unexplained**, and is reported that way.
A spacing mechanism was proposed for it — round-robin extrusion places
births on a chain exactly 6 steps apart, a deterministic lifetime places
expirations identically, so no two adjacent nodes are ever simultaneously
expired and fuse-fuse contention is structurally zero — and was checked
against the logged contention counters as a pre-registered ≈ 0. It came
back **1666 at every k**, which is exactly `n_subdivisions`, the full
E₀/(q+c) budget: one fuse-fuse contention per production event, invariant
in k. The proposed mechanism is therefore refuted, and no replacement is
offered here. The sharper open question is not why there is no contention
but **why Little's law is exact despite there being contention at
precisely one event per production event.**

The decomposition is exact: at E₀ = 4e5 the measured L\*(300)/L\*(100)
is 2.534, and 3·β(300)p(300)/β(100)p(100) predicts **2.535**; at E₀ = 1e5
the same test gives measured 2.128 against predicted 2.127. That
exactness was first established at the ticked β and survives at the clean
one, which is the strongest evidence that the decomposition is a real
factorization rather than a fit.

The final column is the check that makes the decomposition falsifiable:
1/(1−f) references neither L\* nor p, so its agreement with L\*/(k·p) is
independent. It agrees to **1.2–2.0% at k ≥ 100**, and to 4–13% at
k ≤ 30. That degradation is honest rather than alarming — at k ≤ 30,
β − 1 is itself only 0.07–0.18, so an absolute discrepancy of 0.1 is
comparable to the whole quantity being estimated. The cross-check's
useful domain is k ≥ 100, and is stated as such.

### 5.2 Why β > 1, and how much of it was the instrument

β is a **blocking factor**: a node that is removal-ready by age may still
not be removed, because a fuse binding claims three nodes {x, w, y} and
loses them to any higher-priority binding that overlaps. Residence
therefore exceeds k, and L\* exceeds k·p by the same ratio.

**Under the ticked clock, most of that blocking was the instrument.**
`lk_tick` had a single-node LHS, so it claimed the node it decremented,
at a priority above every other rule; under node-disjoint scheduling it
won every contest it entered. An expired node could not be removed while
either neighbour was still counting down. The signature was unambiguous —
roughly half the standing population expired but still present, 77–99% of
removal-ready nodes blocked by a ticking neighbour, the unblocked ones
almost entirely at chain ends where a gen-0 anchor is never tick-eligible
— and the direct counter gave the magnitude: in 3000 pump steps at
k = 100, `lk_fuse` lost to `lk_tick` 34,206 times.

**Removing the instrument removes most of β, and inverts its dependence
on k.** Same cells, same config, one changed field:

| k | β ticked | β passive | expired share f, ticked → passive |
|---:|---:|---:|---|
| 10 | 1.243 | 1.068 | 0.260 → 0.185 |
| 30 | 1.314 | 1.180 | 0.259 → 0.219 |
| 100 | 1.577 | 1.044 | 0.373 → 0.061 |
| 300 | 1.836 | 1.029 | 0.443 → 0.017 |

Instrument blocking **rose** with k; intrinsic blocking **falls** with k.
That is why the ticked β looked like a single smooth function of k and
was not one — it was a sum of two opposite dependences, and the larger
term did not belong to the physics.

**What remains is real, and differently attributed than expected.** At
k ≥ 100 the residual is 3–5%, close enough to 1 that L\* = k·p is
essentially exact; at k ≤ 30 it is 7–18%, genuinely above 1. The natural
guess is that fuse bindings block *each other* — two adjacent fuse
bindings share a node, the same node-disjointness that gives the 2/3 law
in §3.2. The directional counter says that is only part of it: `lk_fuse`
loses overwhelmingly to **`lk_transport`**, which claims two nodes at
higher priority.

| k (E₀=1e5) | fuse → transport | fuse → subdivide | fuse → fuse | fuse-fuse share |
|---:|---:|---:|---:|---:|
| 10 | 10,280 | 1,084 | 6,739 | 37% |
| 30 | 78,887 | 2,770 | 7,230 | 8.1% |
| 100 | 68,669 | 1,665 | 6,795 | 8.8% |
| 300 | 56,399 | 928 | 7,655 | 11.8% |

So the residual is contention among the *physics* rules — which is the
substantive claim — but not principally the fuse-fuse node-sharing the
prediction reasoned from. The corroboration that does hold is ordinal:
normalising total fuse losses by the standing population and the era
length, the per-node-step loss rate ranks the four cells in exactly the
same order as β − 1 (k=30 > k=10 > k=100 > k=300, on both). The ratio
between the two is not constant (0.26–0.97), so this is corroboration of
the attribution and **not a second estimator**; it is reported at that
strength.

The grammar audit behind the ticked clock had asked the *converse*
question — can a tick lose its node to an edge rule? — and answered
ρ = 1.000, never. That result was evidence **for** blocking, not against
it: the tick never yielded, which is exactly why everything else yielded
to the tick. Auditing one direction of a contention and inferring the
other is the error this document's appendix names as a practice to avoid.

### 5.3 The pump cap: retracted

Earlier versions of this document reported a **hard structural cap** in
the pump regime — a standing population pinned at 12, flat across a 30×
range of k, with a peak of exactly 13 in every cell — and gave it a
mechanism: the pump holds 6.0 chains × ~2.0 links, each chain carrying
one ticking node and one expired-but-blocked node, with the same clock
gating both the fuse that would shorten it and the subdivision that would
lengthen it. The supporting counter was real: `lk_metered_subdivide` lost
to `lk_tick` 16,962 times in 3000 steps, so the clock did throttle
production, and p did fall as 1/k (0.541 → 0.020) leaving k·p flat at
5.4–6.0.

**The cap does not exist.** Under the passive clock, with nothing else
changed, L\*/k is 0.986, 0.995, 0.999, 0.999 at k = 10, 30, 100, 300 —
the k-dependence returns in full, and β = 1.00. Every element of the
mechanism above was correctly measured and correctly reasoned; it was a
mechanism for an artifact. This was tested as a pre-registered prediction
with the opposite outcome named in advance (a surviving cap would have
falsified the attribution and blocked the round), and it came back
unambiguous: L\* rises 30.4× across a 30× range of k.

**What survives is the serialization, cleanly measured.** Only the seed
ever exceeds θ in the pump, so all production is serialized at a single
shared anchor — the same mechanism that serialized the terminal tail of
the relaxation in §3.2. Under the passive clock that shows up exactly
where it should: p = 0.993–0.994 events/step at every k, the
one-event-per-step limit, invariant in k. **Serialization pins the
production rate, not the standing population.** The prior claim
conflated the two, and the conflation was only possible because the
clock was suppressing p by a further factor of up to 50.

A second, blunter symptom of the same error: under the tick the pump
never spent its budget. `n_subdivisions` reaches 1666 — the full
E₀/(q+c) — in every passive cell, but only 1164 / 360 / 120 at
k = 30 / 100 / 300 under the tick. The earlier text observed that
"production stops with 93,760 of E₀ = 100,000 still in the graph" and
read it as evidence *against* budget exhaustion, which it was. It was
also evidence that something was stopping production, and that something
was the instrument.

**Consequence for the thesis.** The balance is real *and* continuously
tunable, in both production geometries. Its law carries a scheduler term
(β = 1.03–1.18) where production is distributed, and reduces to the
elementary L\* = k·p where production is serialized — the reverse of the
previous claim, which had the serialized regime as the pathological one.
Bistability belongs to the energy gate, not to the family: it was a
property of reading a degenerate variable. The attractor claim's boundary
is correspondingly restated: locality is removal-dominated in the
unthrottled limit, dissolves in the production-dominated limit, and
between them there is a genuine coexistence regime — one whose standing
structure is, for most of the parameter range explored, *below the
threshold at which this document calls anything non-local at all*.

---

## 6. Negative results as findings

**The stationarity milestone comes first here, because it is the reason
the closure defect became priceable at all.** Every earlier attempt at
the closure defect failed for the same structural reason: the trajectory
crossed phases mid-run, and blocked cross-validation on a nonstationary
series admits precisely the clock confound that retired the original
number. A steady state does not cross phases. On the coexistence stratum
the phase precondition passes outright — 600/600 windows in phase, and
CD_clock exceeding CD_raw for the first time in the track's history.

**A qualification the CD attempts forced, stated here rather than in the
appendix.** The drift-and-variance tests passed on the *short*
(3.3 × 10⁴-step) strata of the first attempt; on the 1.4 × 10⁵-step
strata the larger budget bought, the variance test fails. The exact
census locates why, and it is not a failure of the regime: a metered run
is a fill ramp, then a plateau, then a quench, and the pre-registered
stratum mask does not exclude the transients. **The plateau itself is the
tightest steady state in this work** — standing population 420 ± 6.5 over
roughly 10⁵ steps, a coefficient of variation of 1.6%. Until a
transient-excluding mask is pre-registered and run, the defensible scope
of the milestone is **phase stationarity at every scale tested, and
distributional stationarity demonstrated on a plateau that the current
gate cannot isolate**. Coexistence is therefore not only the
answer to the balance question; it is the first regime in this work in
which the headline metric has a well-posed estimation problem, and the
first whose remaining obstacle is a sample count rather than a
pathology.

- **Non-locality is not self-amplifying (V3, closed negative).** Under the
  fair test (swap regime), the mouths of a planted handle energize but no
  far-side nucleus amplifies: gradient-driven diffusion flattens the profile
  before far-side accumulation reaches θ. Handles are passive conductors.
  Tested once, under the pre-registered fair condition, negative.
- **Seed spread is an instrument-noise floor, not a replication (§7).**
  Because the scheduler is deterministic and the rules draw no RNG, a
  multi-seed sweep on this substrate re-observes one trajectory. The CD
  tuple's `seed_spread` therefore estimates the sampling noise of the
  detour instrument with the dynamical component identically zero. It is
  retained — clearing it is necessary — but the protocol's certainty
  language must not describe it as replication, and does not.
- **The closure-defect estimator does not get relaxed standards (CD
  retirement, protocol case study).** The original headline CD = +15.99
  bits/window did not survive a 5-seed replication (clock-baseline spread
  ≫ mean); it was retired, and CD is now reported only as a tuple on a
  gated stratum, never as a scalar.
- **The first authorized attempt at the closure defect failed its own
  preconditions — including once against this document's own tooling.**
  On the coexistence stratum the stationarity precondition passed
  (600/600 windows in phase, drift and variance tests clear), which no
  earlier attempt achieved. The sample-size precondition failed: lag-1
  autocorrelation of the micro features is ρ ≈ 0.78, so the effective
  sample N(1−ρ)/(1+ρ) is 31–41 against a floor of 200, despite raw
  strata of 225–351 windows. The measurement driver initially checked the
  *raw* count, authorized two seeds, and reported "first valid
  leakage-in-bits measurement" at CD_clock ≈ 2.9 bits/window. That
  verdict was **withdrawn**: the effective-sample restatement of the
  floor already existed and the driver had not implemented it. The
  numbers are recorded as unauthorized and are not quoted as a result.
  What the attempt does produce is a *quantitative* requirement — the
  measured correlation time τ ≈ 400 steps implies a steady segment of
  ≈1.4–1.7 × 10⁵ steps, about 5× longer than achieved, and cadence
  cannot substitute because the effective sample T/2τ is
  cadence-independent. An honest negative with a price tag attached is
  more useful than the number would have been.
- **The second attempt bought the sample and lost the stationarity — and
  the gate, not the regime, is what failed.** The price tag above was
  paid: E₀ = 2 × 10⁶, 3 × 10⁵ steps, five runs, under the passive clock,
  at 0.44 h/run. **The sample-size precondition passes for the first time
  in the track** — effective N = 239–249 against the floor of 200, with
  ρ ≈ 0.70 *not* rising with segment length, which was the assumption the
  whole extrapolation rested on. The stationarity precondition then fails
  on all five: mean drift stays small (5.8–8.6%) but the variance ratio
  is 0.31–0.36 against a [0.5, 2.0] band. The stratum's terminating
  condition named a suspect — the mask ends when `wormhole_count_est`
  reads zero, while `n_subdivisions` keeps climbing and the phase stays
  Sparse — so the sampled, threshold-limited census was crossing its own
  detection floor. That is the instrument §5 had already retired in
  favour of the gen>0 standing census; the EXP-L7R mask was never updated
  with it. The mask was **not** corrected and re-run in the same round;
  the corrected mask, its prediction, and the alternative it would
  exclude were pre-registered for the next one.
- **The third attempt executed that pre-registration, and refuted it —
  which is what pre-registering it was for.** Zero new compute: the exact
  census is recoverable from the persisted runs, because the gen-0
  lattice is non-removable under the age gate, so `census = n_nodes −
  1024` exactly. The prediction was that the variance inflation would
  disappear. It did not; the ratio got *worse* (0.31 → 0.17), and the
  quartile structure was not the one predicted — **two** quartiles are
  inflated, not one, and the middle two are far tighter than the sampled
  instrument had ever shown (420 ± 6.5, sd/mean ≈ 1.6%).

  The exact census then supplied the diagnosis the sampled one could not.
  A metered run has three phases and the mask does not distinguish them:
  a **fill ramp** (~30,000 steps, driven by the production rate itself
  climbing as energy spreads from the seed — not by the k = 300
  lifetime), a **plateau**, and a **quench** as the budget empties. The
  mask's `producing` clause tests whether a counter is rising, which
  stays true throughout the quench, so the stratum straddles a transient
  at each end and the variance test compares a transient against a steady
  state. The pre-registered fallback — restrict to quartiles 1–3 — clears
  the effective-N floor at 278–283 and fails stationarity far worse
  (ratio 112), because it drops the quench and keeps the ramp.

  So v8's correction was right, necessary, and insufficient: it is what
  made the fill-ramp/plateau/quench structure visible at all. The
  plateau is not adopted as a stratum here, for the same reason as
  before. A production-*rate* criterion is pre-registered instead, with
  its band fixed in advance and its own likely failure mode stated: on
  ~950 plateau windows at ρ ≈ 0.70 the effective sample would be ≈ 165,
  below the floor — in which case the honest reading is that this budget
  buys a plateau that is stationary but too short.
- **A green-suite claim that was not green.** The previous round was
  recorded as ending with the full workspace suite passing. One test was
  failing and had been since that round's own contention counters started
  emitting a third CSV; the assertion that the run directory holds only
  two was never updated. Caught by re-running the suite here. Recorded
  because it is the same failure mode this appendix catalogues — a check
  reported as passed without being read.
- **The window floor is statistics-limited, and re-windowing cannot fix
  it.** A dedicated characterization re-windowed a 26k-step run to
  effective cadences 25–500. CD_clock is pathologically negative at every
  cadence, and lag-1 feature autocorrelation rises from 0.895 to 0.997 as
  the cadence shrinks: refining from 52 to 1040 windows buys an effective
  sample of ≈ 1.6. The floor is therefore restated as an *effective-sample*
  requirement rather than a window count, and CD waits for longer
  **stationary** dynamics rather than finer slicing of nonstationary ones.
  This closes a route the plan had explicitly hoped was open.

**The four obstacles, as a protocol case study.** Each attempt at the
closure defect failed for a different reason, and each failure was
converted into a number before the next attempt was authorized:

| attempt | obstacle | what it cost to establish | what it bought |
|---|---|---|---|
| 1 (v6) | no stationary regime existed | the coexistence experiment | a regime that does not cross phases |
| 2 (v7) | effective sample 5× short | measuring ρ ≈ 0.78 ⇒ τ ≈ 400 steps | a required segment length and a budget |
| 3 (v8) | the gate read a retired instrument | one 5-run campaign at E₀ = 2e6 | the effective-N floor cleared, 239–249 |
| 4 (v9) | the mask admits fill ramp and quench | zero — re-analysis of persisted runs | the plateau located: 420 ± 6.5 over ~10⁵ steps |

The bars — 200 effective samples, a variance ratio in [0.5, 2.0], a
signal exceeding its stated uncertainty — were set before the first
attempt and have not moved. Nothing in this table is a result about
geometry; all of it is a result about what it takes to earn one.

---

## 7. Methods appendix: eight concurrency-semantics failures

The most transferable content in this document is not a result about
geometry. It is that **eight** distinct predictions in this work failed
for the same reason: a design assumption about *when things happen* that
was never checked against the scheduler. The engine fires a maximal
**node-disjoint** set of rule bindings per step. Almost every wrong
prediction here came from reasoning as though it fired something else.

1. **Two-era dynamics (F-LK13).** The design assumed growth would finish
   before removal began, because removal had lower priority. Priority
   orders *conflicts*; it does not schedule *processes*. Fuse bindings
   disjoint from the growth event fire in the same step, so there are no
   eras — and the predicted decay had no population to act on.
2. **The halving law (F-LK17).** Relaxation was predicted to halve chain
   length per step, assuming fuse bindings were edge-disjoint. A fuse
   binding claims three nodes {x, w, y}, so the independent set takes
   every *third* interior node: L(t) = L₀(2/3)ᵗ, λ = ln(3/2), confirmed
   to the integer. Note the diagnostic failure: the pre-registered
   factor-2 tolerance band around ln 2 **passed while the derivation was
   wrong** (ln(3/2)/ln 2 = 0.58, inside the band). A tolerance band on a
   rate cannot discriminate a mechanism; a prediction must name the
   discrete alternative it excludes.
3. **The terminal decay tail (§3.2).** All six throats anchor on the
   seed, so node-disjointness admits only one terminal fuse per step and
   the census walks down one chain at a time instead of collapsing
   together.
4. **The aging timer (F-LK19).** The energy gate assumed a child born at
   q would *drain* toward the threshold. Downhill transport conducts it
   *up* to a plateau above q, so the admitting region and the state
   distribution never intersect.
5. **The pump's standing-population cap (§5.3) — and then the cap
   itself (§5.3).** This entry failed twice. First the prediction: the
   shared-anchor serialization of (3) was expected to cap the steady
   state, and a flat L\* = 12 was duly measured and explained. Then the
   explanation: removing the instrument removed the cap. What
   serialization actually pins is the production *rate* (p ≈ 1 event per
   step, invariant in k, now measured cleanly); L\* then follows k. The
   correctly-reasoned mechanism was a mechanism for an artifact — the
   most expensive kind of error in this catalogue, because it looked like
   understanding.
6. **Stolen ticks — predicted, and refuted by measurement (§5.2).** The
   grammar audit flagged in advance that single-node tick bindings could
   lose their node to competing edge-rule bindings, which would have
   inflated the standing population by a factor 1/ρ. Direct measurement
   gave ρ = 1.000. This is the one case where the hazard was named
   *before* the run and then measured rather than assumed.
7. **Blocked removal — the converse of (6), and the one that got past
   the audit (§5.2).** Having established that a tick never loses its
   node, it was tacitly treated as harmless. The opposite follows: a
   rule that never yields is a rule that everything else yields *to*.
   The tick blocks the three-node fuse binding whenever a neighbour is
   counting down, and blocks production the same way — 77–99% of
   removal-ready nodes are held up, and the entire L\*/k prefactor is
   this effect. The audit that caught (6) asked only one direction of a
   two-directional question.
8. **Measurement back-action — the instrument is a scheduler actor
   (§5.2, §5.3).** Failures (6) and (7) are about *which* rule wins a
   contest. This one is about the rule being there at all. `lk_tick` was
   introduced as a passive measurement clock and had no physics content
   whatsoever, yet it inflated the swap standing population by up to 74%,
   suppressed pump production by up to 50×, and manufactured a structural
   result that survived two rounds of audit. The correction is not "give
   the instrument a lower priority" — that trades uniform blocking for a
   worse confound, since a demoted clock's tick rate would correlate with
   local activity. The correction is to **not add a rule**: the same
   lifetime gate, derived from the creation step the store already
   records, adds no bindings and has zero back-action by construction.

   Two details make this generalizable rather than a story about one
   rule. First, the instrument was **cheap** — removing it saves 0.7–2.1%
   of step time, measured on matched states, while it moved the dynamics
   by up to 50×. **Back-action is not a cost phenomenon: scheduling
   footprint and compute footprint are orthogonal axes, and profiling
   will never find a scheduling artifact.** Second, the two
   mechanisms are **step-for-step identical** on a scripted test: a link
   fuses on exactly the same step under both. Semantic equivalence of the
   gate did not imply equivalence of the measurement, because the
   difference was never in the gate — it was in what the gate's
   implementation claimed from the scheduler.

Five practices follow, and each caught a real error here:
**guard-engagement counters** (an instrument that never fires looks
exactly like a null result — three separate rules were silently inert
before counters were mandatory); **dispersion preconditions** (verify the
gate variable's live distribution is non-degenerate *before* the sweep,
or a threshold sweep measures nothing); **mechanism-derived predictions**
rather than tolerance bands; and **instrument-range preconditions** — the
wormhole census reads zero below a chain-length threshold that is itself
regime-dependent, so it would have reported a confident false negative
for the entire coexistence regime. An instrument's blind spot is as
load-bearing as its calibration, and it now ships as a per-window schema
field rather than as prose.

The seventh failure adds a fifth: **contention audits must be
directional**. For each pair of rules that can bind overlapping nodes,
ask which one loses — in *both* directions — and instrument the answer
rather than inferring it from the converse.

The eighth adds a sixth, and a design order to go with it:
**instrumentation is dynamics**. Any rule added for measurement ships
with a directional contention audit *before* it is used to produce a
result, and its binding-claim footprint is stated up front. Preferred
order: (a) derive the quantity from state already present; (b) compute it
engine-side with no rule; (c) add a rule, and only with the back-action
characterized. This work reached for (c) first and paid for it across two
rounds; (a) turned out to be available the whole time, and cheaper.

Underlying all of them is a single lesson that generalizes past this
substrate: **the countdown was introduced to measure a balance and became
the dominant participant in it.** The gate variable was not inert with
respect to the dynamics it gated. Any instrument that must be *written
into* the system it measures deserves the same suspicion applied here to
rules — and the same counters.

A closing observation on that theme, discovered while replicating and
reported because it constrains how every null in this document should be
read: **under this scheduler the seed does not vary the dynamics.**
Conflict resolution is a deterministic greedy pass over a canonical key
and no LK-R rule draws RNG, so a multi-seed sweep produces one trajectory
observed several times. The step, energy, subdivision, fusion and churn
accumulators are byte-identical across seeds; only the sampled
instruments — the 256-edge detour sampler and the spectral estimates —
differ. Multi-seed spread here is therefore an **instrument-noise floor**,
not a replication. It remains a valid guard, since a signal that cannot
clear the instrument's own sampling noise is certainly not a measurement,
but it is not the check its name implies, and §6 states the consequence
for the closure defect's certainty language.

That observation generalizes into a sixth practice's companion rule,
adopted here for everything this work reports: **every spread states its
ensemble.** A ±, a confidence interval, or a spread names what it is over
— cross-validation folds, sampler noise, bootstrap blocks, or initial
conditions — because a spread whose ensemble is unstated invites exactly
the reading this work gave `seed_spread` for three rounds. The field is
now called `sampler_spread` wherever the dynamics is zero-draw, and
carries an explicit ensemble tag.

---

## 8. [Speculative] closing

The measured content of this work reduces to one line: **leakage is
lifetime times production rate.** L\* = β·k·p, with β = 1.00 in the pump
and 1.03–1.18 in swap — a scheduler correction, not a regime. Production
geometry enters only through **p**: serialize production at one anchor
and p is pinned at one event per step however much energy is available;
distribute it and p rises. It does not enter the law anywhere else. The
cap-versus-tunable dichotomy that two earlier drafts built a framing on
does not exist; it was an instrument artifact (§5.3).

*The rest of this section is speculative, tagged, and nothing in the body
depends on it.* A physical geometry that stays local under
structure-changing dynamics would, on the evidence here, be one whose
microdynamics is **removal-dominated**. What this work adds is that the
removal rate is not the only parameter: if an effective layer looks
clean, the reason might be less that its substrate deletes shortcuts
efficiently than that its substrate cannot make them fast in a spatially
concentrated way.

An earlier version of this section said something stronger and wrong —
that serialized production yields a rigid structural fixed point, "a
configuration that gates its own growth and its own decay with the same
clock." The configuration was real; the clock in it was the
*instrument*. Remove the instrument and the serialized regime becomes the
*better-behaved* one, where the law is exact. So the uncomfortable
reading offered in §5.2 turns out to be the literal one: the cleanliness
did partly belong to the observer, and the part that did was the part
that looked most like a discovery.

---

## Appendix: provenance

- Dissolution (§2): `runs/expL_v2_archive/`, `runs/q0_retrodiction/`,
  `runs/exp_l9b/`; findings in `docs/leakage_experiment_findings.md` (v2–v3).
- Suppression (§3.1): `runs/exp_l14/`, `runs/exp_l14_16sq/`.
- Basin (§3.2): `runs/exp_l15/`; driver `crates/hg-sim-core/tests/exp_l15.rs`;
  restoration check `crates/hg-eval/src/restoration.rs`.
- Energy-gate bistability (§4): `runs/exp_l16/`; driver
  `crates/hg-sim-core/tests/exp_l16.rs`; interior-energy diagnosis
  `crates/hg-eval/tests/energy_probe.rs`.
- Coexistence, primary numbers (§5, passive clock): EXP-L18, driver
  `crates/hg-sim-core/tests/exp_l18.rs`; mechanism = the `gate = "age"`
  guard (`rules/lk_fuse.toml`, `node_age_compares`), gates
  `crates/hg-sim-core/tests/b1_age_gates.rs`; full back-action table and
  the P1–P4 verdicts in `docs/back_action_v8.md`.
- Coexistence, the ticked arm (§5.2, §5.3 — the back-action case):
  `runs/exp_l17/`; driver `crates/hg-sim-core/tests/exp_l17.rs`;
  countdown mechanism `rules/lk_tick.toml` + gates
  `crates/hg-sim-core/tests/a2_countdown_gates.rs`; standing-population
  and tick-rate instruments `crates/hg-eval/tests/l17_standing_census.rs`,
  `crates/hg-eval/tests/l17_tick_rate.rs`. The EXP-L18 control arm
  reproduces these numbers to three decimals from a differently written
  driver, which is what licenses reading the difference as back-action.
- Measurement back-action (§5.2, §5.3, §7 item 8):
  `docs/back_action_v8.md`; grammar audit and cost delta
  `docs/grammar_audit_v8.md` +
  `crates/hg-engine/tests/age_grammar_b0.rs`; the zero-back-action gate
  is BG6, read from the `contention:instrument_losses` row in
  `guards.csv`.
- Seed invariance of the dynamics (§6, §7):
  `exp_l18_seed_does_not_vary_the_dynamics`, asserted in both directions.
- Window-floor characterization (§6): `runs/a3_window_floor/`; driver
  `crates/hg-sim-core/tests/a3_window_floor.rs`; verdict in
  `docs/leakage_closure_spec.md`.
- Closure-defect attempts (§6): drivers
  `crates/hg-sim-core/tests/q7_5_exp_l7r.rs` (`Q7_GATE` selects the
  clock) and `c1_regate_attempt3.rs` (the corrected-mask re-gate and
  attempt 3, zero new compute over `runs/v8_l7r_passive/`); pricing pilot
  and the D3 decision `crates/hg-sim-core/tests/b3_cd_pricing.rs`. All
  four attempts, the ensemble memo, and the next pre-registration are in
  `docs/leakage_closure_spec.md`.
- Metronome check and the k-split status (§5.1, §5.2): the addendum in
  `docs/back_action_v8.md`, read from the persisted `runs/v8_l18/`
  contention counters.

**Spreads and their ensembles (§7).** Every ± in this document is over
one of: cross-validation folds, sampler noise, or bootstrap blocks. The
v7 tables reproduced in the appendix report `seed_spread`, which on this
substrate is **sampler noise** with a dynamical component of exactly
zero — the schema field is now named `sampler_spread_bits` and carries an
explicit `spread_ensemble` tag. No quantity in this work has an
initial-condition ensemble; producing one would require breaking the
torus's symmetry, which changes the system rather than the realization.
- Grammar audit behind the countdown mechanism: `docs/grammar_audit_v6.md`;
  cost measurement `crates/hg-engine/tests/tick_cost_a0.rs`.
- Lifetime instrument: `crates/hg-eval/src/lifetimes.rs` (offline;
  `lifetimes.csv` per run).
- Every cited run's energy audit residual is exactly 0.0 (both ledger
  modes); the fuse boundary is pinned by gate FG5 and the countdown
  lifetime by AG2 (`crates/hg-sim-core/tests/lk_fuse_gates.rs`,
  `a2_countdown_gates.rs`).
