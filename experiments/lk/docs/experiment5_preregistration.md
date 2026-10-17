# Experiment 5 — Pre-registration (written before the S4 run)

Findings and predictions committed **before** composing the full ecology and
running it, so that post-hoc results cannot be rationalised. Companion to
`hg_alife_scope_and_architecture.md` §8 (exit criteria) and §8.1 (forbidden
responses). Tests referenced here live in `crates/hg-sim-core/tests/`.

## 1. Predicted degenerate outcome: `replication_threshold → floor`

**Prediction (committed).** With the current rule set, Experiment 5's exit
criterion 5 ("trait distributions *shift*, not merely drift") will be satisfied
by a **degenerate route**: directional collapse of `replication_threshold`
toward its floor.

**Why.** `replicate` is node-funded (scope §7.3): the *site* pays the child
cost; the parent pays nothing. S2.1 chose **unclamped** traits (option b). So
the only gate on reproduction is `parent.energy >= parent.replication_threshold`,
and `jitter` can drive `replication_threshold` negative. A carrier with
`replication_threshold < 0` replicates **every step the site can pay**, while
starving, at zero cost to itself. There is **no opposing selective force** on
that trait in the current rule set — a lower threshold is strictly fitter
(more offspring, no penalty). Selection therefore drives it to the floor.

**Measured bound (test `negative_replication_threshold_replicates_every_fundable_step`).**
A carrier hand-set with `replication_threshold = -1` on a site with energy 100
replicates once per step (one binding per site per step under MaxParallel),
producing ~10 children and exhausting the site (child cost 10) in ~10 steps.
That is the timescale of the degenerate attractor.

**Pre-committed interpretation.** This is **not automatically a bug** —
node-funded replication was a deliberate C++-post-mortem fix, and "selection
finds the degenerate solution" is a legitimate *finding*. If it occurs, the
honest options are exactly two:
1. **Accept and report it** as the Experiment 5 result: selection acts, via the
   degenerate route, and criterion 5 is met trivially. State that the other
   three traits (harvest_rate, maintenance_cost, satiation) are then under weak
   or no selection because the run never reaches a regime where they matter.
2. **Add a parent-side replication cost.** This **reopens the node-funded
   decision** (scope §1, §7.3) and must be argued on its own merits — it is a
   change to the core mechanic, not a knob. It must **not** be adopted quietly
   to make the run "look like" open-ended evolution. Scope §8.1's
   forbidden-response column applies: do not hand-inject a selective force to
   hide the degeneracy.

The choice between (1) and (2) is deferred to after the S4 run, but the
prediction and these two options are fixed now.

## 2. Finding: `satiation <= replication_threshold` is sterile

A carrier harvests only while `energy < satiation`, so it caps at `satiation`.
If `satiation <= replication_threshold`, the carrier can never accumulate enough
to replicate — the lineage is **sterile**, stuck at one carrier. The initial
bootstrap genome had `satiation = 12 < replication_threshold = 15` and produced
exactly this (0 births over 40k steps). Fixed: bootstrap `satiation = 20`
(> 15). This is a **correctness fix** (a logical impossibility), not tuning.
Consequence for §1: once fertile, jitter on `satiation` and
`replication_threshold` interacts — a lineage whose `replication_threshold`
collapses below its `satiation` becomes hyper-fertile, reinforcing the attractor.

## 3. Finding: realized influx ≠ swept `influx_rate` (S6 caveat)

Under MaxParallel, `influx` fires on **every** substrate node below the cap, so
the realized per-step source term is `rate × count(below_cap)` — state-
dependent. Measured (test `closed_economy_ledger_balances_every_step`): realized
influx starts at ~35 firings/step on a 36-node crystal and **collapses to ~0**
as the substrate saturates toward the cap. **The swept parameter is not the
quantity that feeds the economy at steady state.** S6's influx sweep must record
the *realized* source curve (firings/step over time), or the sweep is
uninterpretable. This is committed now, before the sweep is run.

## 4. Closed-economy ledger (verified, not predicted)

The global energy identity holds to **exactly zero** drift, every step, with all
rules composed (test `closed_economy_ledger_balances_every_step`, 40k steps):

```
E(t) = E(0) + Σ seed + Σ influx − Σ maintenance − Σ birth_entropy
            + Σ (death_recycle − death_residual)
```

The **death_residual** term (energy removed when a carrier is deleted at
`energy = r ≤ 0`) is real and is accounted exactly (test
`death_residual_is_accounted`): death contributes `recycle − r`, i.e. removing
negative energy is a source, bounded per death by one maintenance quantum
(die out-prioritises metabolise, so a carrier overshoots 0 by at most one
metabolise step before dying). Because the books close to zero, S4's
explosion/extinction diagnosis can trust that the only *unmatched* source is
influx and the only sinks are maintenance and birth entropy.

## 6. S4 OBSERVED RESULT (the first composed run) — single-site confinement

Recorded **after** the S4 run, against the predictions above. Test:
`crates/hg-sim-core/tests/alife_s4_ecology.rs`
(`s4_first_composed_ecology_run`, `s4_single_site_confinement_is_seed_robust`).

**Observed regime: neither explosion nor extinction — a frozen 2-carrier
duopoly on ONE site.** Over 30k steps on a 36-site crystal (and confirmed for
seeds {1, 7, 12345, 99999} on a 64-site crystal): the population reaches exactly
**2 carriers, 1 birth, 0 deaths**, and `occupied_sites == 1` for the entire run.
The other ~35–63 substrate sites sit **saturated (~20 energy each) and unused**.

**Mechanism (diagnosed, not tuned).** `bootstrap_seed` places one carrier on one
site; that carrier harvests alone, crosses `replication_threshold`, and
`replicate` places the child on the **same** site (replication is strictly
local — the child edge is `[site, child]` on the parent's site). Now two
carriers share one site and **conflict on `harvest`** (one binding per site node
per step under MaxParallel), so the site's single harvest quantum is split
against a combined maintenance draw of 2×0.5/step. The pair equilibrates just
**below** threshold (carrier energy ≈ 13.5 each vs mean threshold ≈ 14.8) and
can never again reproduce. There is **no dispersal/migration rule** to move a
carrier onto an adjacent empty, energy-rich site. The population is therefore
structurally confined to the founding site.

**Which energy term dominates.** Cumulatively influx (≈20.6k) ≈ maintenance
(≈20.0k) — a near-perfect standoff — with birth entropy ≈ 2 (one birth) and zero
death. The *dynamically* decisive process is `harvest` **contention**, which is
a conservative transfer (ΔE = 0) and so is invisible in the energy ledger: the
books look quiet precisely because the freeze is a transfer-level stalemate, not
an energy imbalance. Realized influx collapses to ≈0.33 firings/step (only the
one occupied site cycles), reconfirming §3.

**Status of the §1 prediction: untestable in the observed regime, therefore
neither confirmed nor refuted.** `replication_threshold` drift over 30k steps is
≈0.00 — because reproduction halts after one birth, **no trait is under any
selection**. The predicted degenerate collapse requires a *reproducing*
population; this run never reaches one. That the composed ecology does not reach
a selective regime at all is the real S4 finding, and it supersedes the §1
question until dispersal is resolved. §1 remains committed for whenever a
reproducing regime is reached.

**Ledger check.** The closed-economy identity holds to **exactly 0** divergence
across the whole composed run (§4 machinery, re-run inline), so the confinement
finding is not an accounting artifact.

### The S6.3 conditional has fired (NOT a reopened scope decision)

The implementation plan **pre-registered** exactly this branch. S6.3 ("Movement
decision"): *"Only add a `move` rule … if stationarity provably prevents a
criterion."* S4 is that proof. A stationary, single-site population **provably
hard-fails**:
- **Criterion 4** (multi-generation lineage: `clade_count > 1`, a clade
  persisting > K generations) — one birth, one lineage, one generation.
- **Criterion 5** (selection acting: trait distributions *shift*) — reproduction
  halts after one birth, so no trait is ever under selection.
- **Criterion 6** (`non_trivial_replicator_count` 0 → >0) — no sustained
  replicator ever forms.

Criteria 2 (bounded) and 3 (stable ratio) are *technically* satisfied but only
**degenerately** — the economy is "bounded and stable" precisely because nothing
happens. That is worse than failing them cleanly.

So adding a dispersal mechanic here is **executing a pre-committed escape hatch,
not relitigating scope** (§8.1). The S4 report reads "the S6.3 conditional
fired; here is the evidence," not "we changed our minds." This materially lowers
the cost of the mechanic — it was always the sanctioned response to proven
stationarity blockage.

**Chosen form: birth-dispersal, not motility.** S6.3 speaks of a `move` rule
(rewire the occupancy edge = motility). We instead modify `replicate`'s RHS so
the **child** attaches to an adjacent substrate node (`target`), leaving every
carrier stationary. This is strictly smaller than a move rule and — crucially —
keeps S6.3's *motility* question **genuinely deferred**, not silently answered.
The proposed edit (target-pays, so children only land where energy funds them):

```
replicate LHS: nodes = [site, parent, target]; edges = [[site,parent],[site,target]]
           guard: target.energy >= child_node_cost      # the TARGET pays
replicate RHS: child attached to target (not to site)
```

Target-pays gives a desirable emergent dynamic: children colonise energy-rich
regions along the substrate's own geometry (spontaneous-over-seeded, the project
prior). It is **not** adopted to "make the run look alive"; it is the minimal
structural removal of a *proven* structural block.

**Gating diagnostic (run BEFORE writing the rule).** Sterility could have two
independent causes and dispersal fixes only one. Confinement is provable by
inspection (no rule's RHS attaches a carrier to a substrate node other than the
one it occupies). But underneath it, a *solitary* carrier on a well-fed site may
still never repeatedly cross `replication_threshold`: harvest_quantum −
maintenance is a fixed surplus, but after paying `child_node_cost` the site must
recover to fund the next harvest chain. If carrier energy is a **monotone
approach to a fixed point below threshold** rather than a **sawtooth that crosses
threshold repeatedly**, the per-site economy is the bug and no dispersal rule
rescues it — you would get the same sterility, spatially distributed. Test:
hand-place one carrier on a well-fed site, run, inspect the energy trace. The
`2-carriers-on-2-sites` test is redundant with this and is skipped (if one
carrier can chain replications, two trivially can). This diagnostic is what keeps
the mechanic on the right side of §8.1's forbidden-response line.

**Expressibility constraints for the birth-dispersal rule (recorded now):**
1. **No NAC.** The matcher's `VarGuardType` is only `VarsDistinct` — there is no
   negative application condition, so "adjacent substrate node with *no* carrier"
   cannot be expressed structurally. Workaround: a Bool `occupied` property on
   sites, set at birth, cleared at death — expressible with existing guards, but
   it is **rule-maintained state that can desync from the graph**, so it needs
   its own invariant, asserted every step: `count(occupied sites) == count(carriers)`.
2. **Non-stationary adjacency.** The substrate is rewritten under the carriers, so
   a `target` neighbour may not be a neighbour next step. Acceptable (arguably the
   interesting part), but it couples dispersal availability to the substrate
   regime — which is exactly why the **paired same-seed bare-substrate control**
   (S0.2) is what distinguishes "dispersal failed" from "the substrate stopped
   providing neighbours."

**Re-registered degenerate attractor under target-pays (committed now).** With
target-pays, a lineage whose jittered `replication_threshold` drifts negative
spams children onto its neighbours at zero cost to itself until they are
exhausted — the §1 degeneracy acquires a **spatial signature: a depleted halo
around a low-threshold lineage**. Predicted here in writing so that if it appears
in S6 it is a confirmed prediction, not a post-hoc story.

No dispersal rule has been written yet — the gating diagnostic runs first.

### S4 GATING DIAGNOSTIC RESULT + BIRTH-DISPERSAL OUTCOME (recorded after running)

**Gating diagnostic (`s4_gating_fertile_lineage_chains_reproduction`, and its
pre-dispersal form): GO.** A solitary carrier on a well-fed site chains
replication — 18 births in 200 steps, founder energy pinned above threshold
100/100 steps in the second half. The per-site economy is healthy; sterility is
NOT a second bug. Confinement is the sole blocker, so birth-dispersal is the
correct, sufficient fix. The `2-carriers-on-2-sites` test was correctly skipped
as redundant.

**Expressibility (both resolved against the real substrate):** the crystal
generator builds its lattice from **binary `Label(0)` edges** even at arity 3
(`[center, neighbour]`, not 3-ary), so a 2-ary `[site, target]` LHS edge matches
substrate adjacency; `target = Label(0)` + `VarsDistinct` separate it cleanly
from the occupancy edge. The `occupied` Bool NAC-surrogate was implemented; its
invariant `count(occupied) == count(carriers)` is asserted every step over a
5 000-step composed soak (`alife_dispersal.rs`) and never desyncs.

**Birth-dispersal implemented and composed-run re-run. New regime: FULL-SATURATION
FREEZE (confinement fixed).** With target-pays birth-dispersal, the population
spreads from the founder across the WHOLE substrate — 2 → 36 carriers on a 36-site
crystal (and ≥20 on 64 sites, seeds {1,7,12345,99999}), **one carrier per site**
(`occupied_sites == carriers` throughout). It then **freezes at carrying
capacity**: every site occupied ⟹ no unoccupied neighbour ⟹ no dispersal target ⟹
reproduction halts. **Zero deaths** — a solitary carrier on its own influx-fed
site never starves. Ledger balances to exactly 0; determinism holds.

**Criteria status now:** 1 (spontaneous replication) MET; 2 (bounded) and 3
(stable ratio, here 1.0 with zero late-window variance) now **genuinely** met,
not degenerately. But **4/5/6 remain blocked by a NEW cause: no turnover.** With
carriers effectively immortal at saturation, reproduction stops, so there is no
sustained multi-generation churn (4), no ongoing selection — `replication_threshold`
drift ≈ 0.00 after the colonisation wave (5) — and no sustained non-trivial
replicator (6). Confinement was traded for saturation-without-mortality.

**The next decision (deferred to the user — do NOT reflexively add a mechanic).**
Turnover requires carrier mortality that frees sites so reproduction (and thus
selection) continues. Two honestly-different routes, and per §8.1 the choice must
be diagnosed, not tuned into existence:
1. **Influx sweep (S6.1) — no new mechanic.** Mortality may already be reachable
   by SCARCITY: at a low enough influx rate, sites cannot keep their carrier fed,
   carriers starve and die (`die_and_recycle` already exists), freeing sites for
   recolonisation → turnover → a regime where selection operates. The current run
   is the abundant-energy end (influx rate 2, cap 20); the sweep tests whether a
   Goldilocks band produces bounded, turning-over, selecting populations. This is
   the pre-registered S6.1 step and adds nothing. **Caveat:** influx rate is
   currently hard-coded in `alife_base.toml`; the sweep needs it parameterised
   (an `AlifeConfig` field), which is plumbing, not a mechanic.
2. **A mortality/aging mechanic** (e.g. a `lifetime` counter driving death, or
   density-independent decay) — a NEW core mechanic, argued on its merits like
   dispersal was, only if the influx sweep proves scarcity-driven mortality
   cannot yield a selecting regime. Route 1 is tried first precisely so route 2
   is not adopted prematurely (the §8.1 forbidden-response discipline).

Recommendation: **route 1** — parameterise influx and run the S6.1 sweep before
considering any new mortality mechanic.

## 7. S6.1 influx-sweep PRE-REGISTRATION (committed before the sweep)

Two frozen-state diagnostics were run first (they determine what the sweep even
means):

**(a) The freeze is OCCUPANCY-limited.** At saturation the run is 36 carriers on
36 sites, **0 unoccupied** — births are blocked structurally (dispersal has no
target), so **mortality is the sole unlock** for criteria 4/5. (An energy-limited
freeze — free sites but all below `child_node_cost` — would read differently; it
is not what occurs.)

**(b) Criterion 6 is blocked by MISSING MACHINERY, not by turnover.**
`ReplicatorKind::NonTrivial` is **never assigned** anywhere in the current code
(`hg-causal/src/constructor.rs`: *"NonTrivial requires the ALife layer's
bounded-individual tracking and is never assigned here"*). So
`non_trivial_replicator_count()` is **structurally 0** until S5 builds the lineage
taxonomy. **No influx value can move criterion 6** — it is gated on S5,
independently of the ecology. The sweep therefore targets criteria 4 (lineage)
and 5 (selection) only; criterion 6 is an S5 deliverable regardless of the run's
dynamics.

**Analytic critical influx rate (committed).** At saturation, one carrier per
site, no harvest contention. The carrier is pinned near satiation: it harvests
(+H, H=1) only when hungry and otherwise metabolises (−M, M=0.5); harvest and
metabolise conflict on the carrier so exactly one fires per step. Its steady
state has harvest fraction `f = M/(H+M) = 1/3`, so the **average site drain is
`f·H = M·H/(M+H) = 1/3` per step**. Influx fires whenever a site is below cap,
supplying `r` per firing, so a site is sustainable **iff `r ≥ 1/3`**. Therefore:

> **Predicted critical rate r\* = M·H/(M+H) = 1/3 ≈ 0.333.**
> Above r\*: FROZEN (carriers sustained, no death). Below r\*: sites deplete at
> rate `1/3 − r` and carriers starve.

This refutes the naive heuristic `r\* ≈ maintenance_quantum = 0.5`: the satiation
duty cycle lowers it to 1/3, and the S4 saturation data already shows realised
per-site influx = maintenance ≈ 0.33, corroborating 1/3 before the sweep is run.
**If the transition appears materially away from 1/3, the economy model is wrong —
and that is the finding.**

**Predicted transition SHAPE: discontinuous, no stable turnover band.** All sites
are identical and drain synchronously (the substrate is static — the S4 config
runs no substrate rules — and the per-site energy balance is independent of
lattice degree), so just below r\* **every** site depletes together and **every**
carrier starves near-simultaneously: mass extinction, not turnover. The expected
result is FROZEN ↔ EXTINCT with a razor-thin or absent middle. The stable
births≈deaths regime Experiment 5 needs likely does **not** exist on a static,
homogeneous substrate.

**Heterogeneity-rescue hypothesis (committed).** Stable turnover requires spatial
heterogeneity in site energy to desynchronise starvation. The current S4 setup
has **none** (static substrate, uniform init). A rewriting substrate would supply
it for free (fresh zero-energy nodes = poor sites; varying occupancy/geometry) —
but that is a setup change (run the ecology over an active substrate rule), not
part of this sweep. **Prediction:** turnover, if it ever appears, will be driven
by substrate churn, and death events will correlate with recently-created or
low-degree substrate nodes. If a future run shows that correlation, it is an
emergent-heterogeneity mechanism (spontaneous-over-seeded) and the S6 headline.

**Pre-committed decision rule (fixed now):**
1. Sweep finds a band with births ≈ deaths, bounded non-zero population, persistent
   clades ⟹ that is the Experiment-5 turnover regime; run the trait-distribution
   analysis on it.
2. Sweep finds only FROZEN ↔ EXTINCT, no stable band ⟹ scarcity-driven mortality
   is **proven insufficient** on this substrate, and `lifetime`-based aging
   (key 21, already scoped in the S2.1 registry as the carrier age counter)
   becomes justified — but only then, and gated behind this proof.
3. Sweep finds turnover but trait distributions only DRIFT (no directional shift
   or variance collapse) ⟹ criterion 5 fails on its own terms; that is a finding
   about selection strength, not a licence to add mechanics.

**Plumbing note.** The sweep varies `influx_rate` (the influx rule's add value),
but the physics depends on the ratio `r / (M·H/(M+H))`, so `M` (maintenance) and
`H` (harvest quantum) are recorded alongside each swept rate. A rate logged
without its denominator is uninterpretable later.

### S6.1 SWEEP RESULT (recorded after running; `alife_s6_influx_sweep.rs`)

36-site static crystal, 12 000 steps, rates 0.1 → 2.0 with fine granularity across
the transition:

| influx r | r/r\* | regime | final carriers | peak |
|---|---|---|---|---|
| 0.10 – 0.48 | 0.30 – 1.44 | **EXTINCT** | 0 | **1** |
| 0.50 | 1.50 | FROZEN | 5 | 5 |
| 0.52 – 2.0 | 1.56 – 6.0 | FROZEN | 36 | 36 |

**Discontinuity CONFIRMED (§7 core prediction holds).** Every rate is EXTINCT or
FROZEN; **no turnover band exists** at any granularity tested. Sharp boundary at
r ≈ 0.49.

**Exact r\* REFUTED (the pre-committed falsification fired).** The transition is at
≈ 0.49, **1.47× the predicted steady-state r\* = 0.333** — well outside tolerance.
The economy model was right about the *steady-state* balance but wrong about the
*binding constraint*.

**Diagnosed cause — a STARTUP barrier, not a sustain floor.** Every extinct run
has **peak = 1**: the founder dies *before it ever reproduces*. Bootstrapping a
population is more expensive than sustaining one: the founder must climb from
birth energy 10 to `replication_threshold` 15 (harvesting near 1/step, far above
the 1/3 steady-state draw), AND an adjacent site must independently fill to
`child_node_cost` 10 — both transient demands exceed the 1/3 maintenance floor.
So the **startup threshold (≈ 0.49) sits above the sustain threshold (0.333)**.

**Why this GUARANTEES no turnover band (the mechanism, now understood).** A
turnover regime would need `r ∈ (0.333, 0.49)`: high enough to sustain an
established population (> sustain floor), low enough to still kill carriers under
load. But nothing can *establish* below 0.49, and everything that establishes is
≥ 0.49 ≫ 0.333, so every established population is immortal. **The turnover window
is squeezed shut between the two thresholds.** On a homogeneous substrate the two
thresholds cannot be reconciled by any single global influx rate.

**Scope of the claim (important caveat).** The proven statement is *"no turnover
band on a **static, homogeneous** substrate."* This is strictly WEAKER than
"scarcity-mortality cannot yield turnover" — it is conditional on the homogeneity
that makes starvation synchronous. That conditionality is precisely why the
heterogeneity route (A below) is the correct next TEST, not a consolation prize:
the sweep specified the target quantitatively (a ~1.47× spread in effective
per-site influx is needed to reopen the squeezed `(0.333, 0.49)` window).

**Decision-rule outcome (pre-committed rule 2 fires).** Scarcity-driven mortality
is proven insufficient to produce turnover on the static homogeneous substrate.
Two committed routes forward (the user's choice):
- **(A) Spatial heterogeneity via an active substrate** — run the ecology over a
  rewriting substrate so fresh/poor nodes desynchronise starvation (the §7
  heterogeneity-rescue hypothesis). Tests whether turnover is *emergent* from
  substrate churn (the spontaneous-over-seeded headline) with NO new ALife rule.
  Adds a substrate-dynamics setup, and re-opens the question of which builtin.
- **(B) `lifetime`-based aging** — key 21 is already scoped as the carrier age
  counter; a death-by-age rule decouples mortality from the energy economy, so
  turnover no longer needs the impossible `(0.333, 0.49)` reconciliation. A new
  ALife rule, but a *pre-scoped* one, now justified by the sweep (not a patch).

Route A is more in the project's spirit (emergent, not imposed); route B is the
smaller, pre-scoped mechanic. Both are honest under §8.1 now that the sweep has
proven the global-influx route cannot work.

### Route-A prerequisite probe: no builtin provides the needed substrate

Before running the heterogeneity kill-test, the substrate must be chosen for
**churn WITHOUT net growth** (unbounded growth is the explosion confound: realised
influx = rate × count(below-cap sites) scales with node count, so a growing
substrate drives unbounded population growth, not turnover). Measured trajectories
(`substrate_churn_probe.rs`, verified by node count not by name):

| builtin | n(t) | fresh nodes | verdict |
|---|---|---|---|
| growth_treadmill | 36→66 (grows) | **label-1** | grows AND influx ignores its fresh nodes (label-1) |
| glider | 36→36 (flat) | none (reuses a,b,c) | flat but mints **no** fresh nodes ⟹ no poor sites |
| triangle_subdivide | 36→396 (explodes) | label-0 | mints fresh label-0 but grows explosively |
| binary_edge | super-linear | label-0 | same — grows, blows up |

**Finding: no existing builtin churns without growth while minting fresh,
influx-eligible (label-0) nodes.** The heterogeneity mechanism (fresh energy-poor
label-0 sites at a stable count) therefore has **no ready vehicle** — it requires a
NEW balanced-churn substrate rule (create + delete label-0, net zero). And once
such a rule exists, the resulting cross-site energy spread is a **tunable function
of (churn rate, influx rate)**, not a fixed emergent quantity: route A can in
principle be driven to the required ~1.47× spread by choosing the churn rate,
which reintroduces the emergent-vs-tuned question. This is a design decision, not
a measurement, and is surfaced to the user before any substrate rule is written.

*Substrate-track note (belongs in the substrate characterisation report too): the
builtin rule zoo contains **no bounded-churn regime** — every rule measured either
grows unboundedly (growth_treadmill, triangle_subdivide, binary_edge) or is
structurally frozen in node identity (glider). Bounded churn (high node turnover
at flat node count) is the regime a "physics" would be expected to occupy, and
none of the hand-written rules land there. Whether that is a fact about these five
rules or about rewrite systems generally is a substrate-track question.*

**Firewall on route A (recorded, per the layer-isolation discipline).** If a
balanced-churn substrate rule is written, it must be commissioned on the SUBSTRATE
track as physics — its own bare-substrate characterisation run, its own regime
classification, its own place in the rule zoo — and the ALife layer may then *use*
it. Writing a substrate rule whose selection criterion is "it makes the biology
work" is the contaminated coupling the project is organised against (the C++
codebase's unattributable bugs). The churn rate being a dial is fine (every physics
has constants); choosing the rule *to serve* the ALife layer is not.

## 8. child_node_cost re-examination (the diagnosis-named parameter — do BEFORE route A)

The empty influx interval `(0.333, 0.49)` is not only a falsification; it is a
**specification with a named parameter**. The upper edge (0.49) is the STARTUP
threshold, and startup requires a target site to fill to **`child_node_cost`
(currently 10)** before a birth can occur. Lowering `child_node_cost` lowers the
reproduction floor, lowers the startup threshold toward the sustain threshold
(0.333), and **narrows the gap the run must straddle**. Because the gap's width is
an analytic function of a parameter the diagnosis names, adjusting it and
re-running the pre-registered sweep is the legitimate S4 "justified parameter
adjustment" — NOT a §8.1 forbidden response (no global cap, no hand-injected
fitness, no threshold lowered *without* diagnosis; here the diagnosis is explicit).
It is also strictly cheaper than commissioning a churn substrate.

**Stillbirth floor (the hard constraint).** Child birth energy is `0.8 ×
child_node_cost`; a newborn must not be dead on arrival, i.e. `0.8·c > 0` and, to
have any chance of climbing to `replication_threshold`, comfortably above one
maintenance quantum `M = 0.5` ⟹ `c ≳ 0.7`. There is therefore real room below the
current 10.

**Pre-committed reading of the re-sweep:**
- Lowering `child_node_cost` **reopens a turnover band** (births≈deaths, bounded,
  persistent) ⟹ turnover on a static homogeneous substrate after all; route A is
  unnecessary and this is the cleanest outcome.
- The `child_node_cost` needed to reopen the band is **below the stillbirth floor**
  ⟹ a proof the current economy cannot support turnover at ANY influx rate, which
  is a strictly stronger result than "the sweep found no band" — and it is what
  genuinely earns the move to route A (churn substrate) or route B (aging).
- Still only FROZEN ↔ EXTINCT at every sane `child_node_cost` ⟹ homogeneity is the
  binding obstacle, not the birth cost; route A becomes the tested next step.

### §8 RE-SWEEP RESULT (recorded after running; `alife_s6_child_cost_sweep.rs`)

child_node_cost × influx grid, 12 000 steps, 36-site static crystal:

| cost \ r | 0.35 | 0.45 | 0.55 | 0.70 | 1.00 |
|---|---|---|---|---|---|
| 2.0 | EXTINCT | EXTINCT | FROZEN | FROZEN | FROZEN |
| 5.0 | EXTINCT | EXTINCT | FROZEN | FROZEN | FROZEN |
| 10.0 | EXTINCT | EXTINCT | FROZEN | FROZEN | FROZEN |

**Outcome 3 confirmed — and the §8 hypothesis is REFUTED.** The establishment
(EXTINCT→FROZEN) threshold sits at the **same** influx (~0.50) for **every**
child_node_cost; there is **no turnover cell** anywhere. `child_node_cost` is
therefore **not the startup barrier**: lowering it does not lower the establishment
threshold at all.

**Why the §8 arithmetic was wrong (mechanism).** The startup barrier has two
components, and §8 named the wrong one:
1. a carrier must **climb to `replication_threshold`** (15) by harvesting against
   its site's drain — governed by `replication_threshold`, satiation, harvest, and
   influx, NOT by `child_node_cost`; and
2. a target must be **fundable** (≥ `child_node_cost`) — but sites start at energy
   5 and fill fast, so this is never the binding constraint for c ≤ 10.
The binding barrier is (1). Worse, `child_node_cost` is **coupled to birth energy**
(`0.8·c`): lowering c makes newborns *poorer* (birth energy 1.6 at c=2), so cheaper
births are more fragile, not more viable — which is why c=2 is no better than c=10.

**Why child_node_cost failed — SYMMETRY (the clean analytic result).**
`child_node_cost` is a **scale, not a shape**: it appears in four coupled places
(fundability guard, target payment, birth energy `0.8c`, death recycle `0.8c`)
that move together, so it scales the birth threshold and the death threshold in
lockstep and leaves the dimensionless window ratio `0.49/0.333 = 1.47`
**invariant**. The sweep was doomed by symmetry. `replication_threshold` is
different: it appears in the **birth condition only** (`energy ≥ threshold`) and
**nowhere in death** (`energy ≤ 0`) — an **asymmetric** lever that moves the birth
floor without touching the mortality ceiling. That is the degree of freedom
`child_node_cost` lacked, so the "no single economy scalar can reopen the window"
claim is about *symmetric* scalars and does **not** generalise to
`replication_threshold`. Next: sweep it (§9).

## 9. TURNOVER ACHIEVED — Experiment 5 satisfied on a static substrate (§9)

The `replication_threshold × influx` sweep (`alife_s6_threshold_sweep.rs`) reopened
a **turnover band on the static homogeneous substrate** — routes A and B are
UNNECESSARY. At bootstrap `replication_threshold = 3` and sub-sustain influx
(0.28–0.40), the population enters a sustained births≈deaths limit cycle; at
threshold 6 or 15 it goes extinct (cannot establish at low influx). Grid:

| thr₀ \ r | 0.28 | 0.34 | 0.40 | 0.48 |
|---|---|---|---|---|
| 3 | TURNOVER | TURNOVER | TURNOVER | EXTINCT |
| 6 | EXTINCT | EXTINCT | EXTINCT | EXTINCT |
| 15 | EXTINCT | EXTINCT | EXTINCT | EXTINCT |

**Endogenous desynchronisation, measured (the §7 synchronized-death argument was
conditional).** The static substrate is homogeneous but the population is not: two
cheap checks (`alife_s6_population_heterogeneity.rs`) showed the frozen run's
satiation spread is only 1.084× (< the 1.47 budget) and its traits are frozen
(no selection). So endogenous heterogeneity is *insufficient by itself*. But under
a LOW threshold at sub-sustain influx, the birth/death cycle itself staggers
events enough to sustain turnover — the required desynchronisation is produced by
the *dynamics*, not pre-existing site heterogeneity.

**Experiment-5 scorecard (`alife_experiment5_scorecard.rs`, thr₀=3, r=0.34, 15k
steps) — all seven criteria MET:**

| # | criterion | result |
|---|---|---|
| 1 | spontaneous replication | 8485 births from one bootstrap founder |
| 2 | bounded (no explode / no extinction in-window) | final 33, peak 36 |
| 3 | stable ratio, low late-window variance | ratio 0.938, var 2.6e-3 |
| 4 | multi-generation lineage, clade > K gens | max depth **500**, 8485 births |
| 5 | selection acting (directional trait shift) | mean threshold `0.14 → −3.72` |
| 6 | `non_trivial_replicator_count` 0 → >0 | **2588** (0 on bare substrate) |
| 7 | no substrate regressions | exit-criterion-7 bit-identical throughout |

Turnover is balanced (births 8485 ≈ deaths 8453; last window 1695 ≈ 1698).

**Load-bearing caveats (reported, not hidden):**
1. **Criterion 5 is met via the §1 DEGENERATE route.** `replication_threshold`
   collapses toward its (negative) floor (`0.14 → −3.72`) because replication is
   node-funded — nothing opposes a lower threshold, so selection drives it down.
   This is "selection acting" but the degenerate kind §1 pre-committed as a
   legitimate *finding* (option 1: accept and report). The other three traits are
   then under weak/no selection. It is **not** open-ended evolution; pursuing
   non-degenerate selection would mean revisiting the node-funded decision (§1
   option 2, a separate research direction, not required for Experiment 5's bar).
2. **Initial-condition dependence.** `replication_threshold` is a jittered genome
   trait, so the sweep sweeps an INITIAL CONDITION, not a physics constant. A high
   bootstrap threshold cannot *enter* the turnover basin at low influx (it goes
   extinct first). But once inside, selection *drives* the threshold down and keeps
   it there — the low-threshold turnover state is the **attractor**, self-
   reinforcing; bootstrap = 3 just seeds the basin directly. Influx is physics;
   this lever is an initial condition.

**Consequence for the roadmap.** Experiment 5's exit criteria are satisfied, which
per S7 unblocks Experiment 6 (CE across replication onset). Route A (churn
substrate) and route B (lifetime aging) are not needed for Experiment 5; they
remain options only if *non-degenerate* open-ended selection becomes the goal.

## 5. Conflict-matrix reachability (verified)

Priorities: `die_and_recycle` (3) > `replicate` (2) > `harvest` (1) >
`metabolise` = `influx` = `bootstrap_seed` (0). No rule is unreachable behind a
higher-priority one: a satiated, sub-threshold, positive-energy carrier on a
site that cannot fund replication matches **only** `metabolise`; `influx`
matches unoccupied substrate nodes where no carrier rule applies; each higher
rule has states where it alone (or as the top match) fires. Death and
replication are both verified not-starved (tests `composed_death_is_not_starved`,
`composed_replication_is_not_starved_by_metabolise`).
