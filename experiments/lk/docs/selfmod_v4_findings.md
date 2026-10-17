# selfmod v4 findings — flux economy, variation-rate sweep, persistence vs novelty

**Setup.** `hg-selfmod` extended with a selectable **stock** or **flux** economy under one
flow audit, probabilistic mutate-at-birth (μ swept), per-lineage creation budgets, and the
F1–F3 carry-in fixes. Engine untouched (ground rule 1). ~330 runs, all with
`flow_audit_residual = 0`. Calibration and pre-registered thresholds:
[selfmod_v4_calibration.md](selfmod_v4_calibration.md). Headline figure:
[selfmod_v4_windows.svg](selfmod_v4_windows.svg).

---

## E1-control (μ = 0) — reported first, because it changes what everything else means

> **Correction (v5/C1).** The original text of this paragraph said the survivors "are
> exactly the 2 that never reproduce," which contradicts the P = 0.50 in the table below
> and the 1642 clonal births cited. Re-derived from the run summaries, the correct
> accounting is stated here. The claim that reproduction destabilizes the set survives,
> but it is 4-of-6 rather than 6-of-6, and the sharper result is the 803-vs-5 contrast.

**[Measured]** With **zero variation** — every offspring a faithful clone — reproduction is
often lethal on the distributed set. On `churn3` at μ = 0, **6 of 8 seeds reproduce
clonally and 4 of those 6 freeze and die**. The 4 survivors are therefore **2
non-reproducers** (seeds 42, 99 — `time_to_first_fission = none`, 0 births) **plus 2 heavy
reproducers** (seeds 2, 777 — 803 clonal births each, surviving to budget). Reproduction is
strongly but not perfectly anti-correlated with survival. In the stock arm at μ = 1 (G4.1)
the anti-correlation *is* perfect — every collapsing seed reproduced first, both survivors
never did — which is what made the sloppier sentence look right.

The plan pre-committed a dichotomy — clonal populations persist ⇒ H-catastrophe live;
clonal populations also collapse ⇒ H-catastrophe dead. **The data says both halves, and the
dichotomy was underspecified.** Stated precisely:

- **Reproduction itself destabilizes the set, independent of variation.** 4/6 clonal
  reproducers die at μ = 0. This is not error catastrophe; there is no error. The redirect
  the plan attached to this branch — to reproduction economics rather than mutation rate —
  is therefore live.
- **Variation is far sharper, and this is the cleanest controlled result in v4.** Seeds 2
  and 777 survive to budget at μ = 0 having produced **803 clonal offspring each**. At
  μ = 0.05 — same seeds, same configuration, mutation the only difference — both die, after
  **5 mutant offspring each** (alongside 42 and 55 clonal). Five mutants destroy a lineage
  that 803 faithful copies did not perturb. Both channels are live; the variation channel is
  roughly two orders of magnitude sharper.

**[Measured]** Clonal births add exactly **zero** novelty by construction: at μ = 0,
`M_rules` stays at the seed count (3) across 20000 steps and 1642 clonal births, and
`alive_rule_classes` never moves off 3. This is why the novelty window is empty at μ = 0 —
not because innovation is suppressed, but because cloning cannot innovate.

---

## The flux economy is dynamically inert here — and that is attributable, not vacuous

**[Measured]** Across the entire E1 grid (7 μ × 8 seeds), **stock and flux produce
bit-identical dynamics**. Diffing run CSVs pairwise, the only columns that ever differ are
`free`, `cumulative_influx`, `cumulative_dissipated` — pure bookkeeping. `rule_count`,
`births`, `productive_applied`, `data_hash` and every other dynamical column match exactly.
The machinery is demonstrably live (a flux run at step 200: influx 4700, dissipated 4070,
`free` pinned at its 500 cap; the stock twin: influx 2000, dissipated 0, free 1870).

**Mechanism [Inferred, then tested].** A rule's income is `min(fire_reward, free)`. With a
handful of rules, `free` is never small enough to bind, so the two economies expose no
dynamical channel at all. G4.3 saw the same thing from the other side: latency was
identical across a 32× range of influx.

**Direct test (excursion, logged).** Re-ran with the pool made genuinely scarce — stock
`total = 200`, flux `I = 2 / free_cap = 50`, μ = 0.01, 8 seeds. **All 8 seeds now diverge
dynamically** between economies, and **flux is worse than stock (0/8 vs 2/8 survival)**,
because dissipated rent never returns while a conserved pool recycles it.

**Verdict on H-economy: refuted, in both directions.** The conserved pool was not the root
cause of v3's reproduce-then-die pattern — it was never exerting pressure, because funded
fission raids the *parent's own tokens*, which is identical under both economies. And where
the pool does bind, driving it makes matters worse rather than relieving a stranglehold.
The plan's branch **(d)** is refuted.

---

## E1 / E3 — the persistence and novelty windows

Windows over the combined μ grid `{0, 0.005, 0.01, 0.02, 0.03, 0.05, 0.1, 0.2, 0.4, 0.7, 1}`,
8 seeds each, flux (≡ stock). Persistence `P` ≥ 0.6; novelty = median late-window OLS slope
of **both** `M_rules` and `M_data` > 1e-4 (pre-registered).

### `churn3` — distributed autocatalytic set (k = 3) → **branch (c)**

| μ | 0 | 0.005 | 0.01 | 0.02 | 0.03 | 0.05 | 0.1 | 0.2 | 0.4 | 0.7 | 1.0 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| P | 0.50 | 0.38 | 0.38 | 0.25 | 0.25 | 0.25 | 0.25 | 0.25 | 0.25 | 0.25 | 0.25 |
| novelty | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

**Persistence window EMPTY** (P never reaches 0.6). **Novelty window EMPTY** (every median
slope is exactly 0.000e+00). Survival plateaus at 0.25 — always the same 2 non-reproducing
seeds. This is the plan's branch **(c)**.

### `churnK` — minimal self-sufficient sustainer (k = 1) → **branch (b)**

| μ | 0 | 0.005 | 0.01 | 0.02 | 0.03 | 0.05 | 0.1 | 0.2 | 0.4 | 0.7 | 1.0 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| P | 1.00 | 0.88 | 0.88 | 0.75 | 0.25 | 0.25 | 0.00 | 0.00 | 0.12 | 0.00 | 0.00 |
| median d`M_rules` | 0 | 2.0e-4 | 2.4e-4 | 4.4e-4 | 4.1e-4 | 5.2e-4 | 1.3e-3 | 1.9e-3 | 3.4e-3 | 4.6e-3 | 9.1e-3 |
| median d`M_data` | 0 | 9.9e-5 | 9.9e-5 | 9.6e-5 | 2.4e-2 | 3.6e-2 | 1.3e-1 | 8.6e-2 | 3.0e-1 | 4.1e-1 | 1.1e-1 |

**Persistence window {μ ≤ 0.02}. Novelty window {μ ≥ 0.03}. Overlap EMPTY — and the two
windows are exactly adjacent, tiling the grid with no gap.** This is branch **(b)**:
stability and evolvability are mutually exclusive at the population level, in a driven
system, with the boundary falling *between* grid points rather than leaving a Goldilocks
band.

### Resolving the boundary — the result survives the obvious objection

**[Measured]** The μ = 0.005–0.02 band failed the novelty test by a hair: d`M_data` of
9.6–9.9e-5 against a 1e-4 threshold, within 4%. Since a differently-chosen threshold would
have flipped branch (b) to (a), the band was re-measured at **60000 steps** (final third
~20000 steps, dropping the resolution floor ~3×):

| μ | P @20k | P @60k | d`M_data` @20k | d`M_data` @60k |
|---|---|---|---|---|
| 0 | 1.00 | **1.00** | 0 | **0** |
| 0.005 | 0.88 | 0.50 | 9.9e-5 | 6.8e-3 |
| 0.01 | 0.88 | 0.50 | 9.9e-5 | 9.0e-3 |
| 0.02 | 0.75 | 0.50 | 9.6e-5 | 3.3e-3 |

Both sides moved, and both moved **away** from an overlap:

- The band's novelty was real, and had been resolution-limited — d`M_data` rises by ~70×
  once the window is long enough to see it. So these μ belong to the **novelty** side.
- The band's persistence was **"not dead yet"**, not persistence. P decays toward 0.50 as
  the horizon triples. Only **μ = 0 holds P = 1.00 at both horizons** — and it is exactly
  the point with exactly zero novelty.

**[Inferred]** μ_c is horizon-dependent and appears to converge toward 0: any nonzero
variation is eventually lethal to the set, and larger μ merely kills it sooner. Two
horizons is not an asymptotic study, so this is inference, not measurement — but it is the
straightforward reading of a monotone decay at three independent μ values.

**The sharpened statement of branch (b):** the trade-off is not a narrow band that better
tuning might widen. Persistence lives at exactly μ = 0, novelty at exactly μ > 0, and the
former is the only horizon-stable point. In this substrate, a self-modifying population can
be permanent or it can be creative, and the boundary between those is the presence or
absence of variation itself.

---

## E3 — μ_c versus autocatalytic-set size: **prediction CONFIRMED**

The plan's pre-registered prediction: a *distributed* autocatalytic set has more surfaces to
break, so μ_c should **decrease** as k increases.

**[Measured]** μ_c (survival crossing 0.5), bracketed by grid points, 8 seeds each:

| set | k | μ_c bracket | P at μ = 0 |
|---|---|---|---|
| `churnK` | 1 | **(0.02, 0.03]** | 1.00 |
| `churn3` | 3 | **(0, 0.005]** | 0.50 |

μ_c(k=3) < μ_c(k=1) by at least an order of magnitude. **Prediction confirmed** — rule-cycle
self-modifying systems are more prone to lethal mutagenesis than a point replicator of equal
per-member fidelity.

**Caveat, stated with the same weight [Measured].** `churn3` is already at P = 0.50 at
μ = 0, where no mutation exists at all. So part of its lower μ_c is inherited from a
baseline fragility to *fission cost*, not to variation. The confirmed prediction should be
read as "distributed sets are more fragile, and mutation is only one of the two channels
through which that fragility acts."

> ### ⚠ CORRECTION (v6 P2, 2026-07-29): the order-of-magnitude claim above is withdrawn.
>
> Two later results undercut this section. v5's C1 established that the μ = 0 deaths are
> intrinsic world instability, not fission cost, so the caveat's stated mechanism was
> wrong. v6's P2 then re-analysed these same archived runs conditioned on baseline
> survival and found something stronger:
>
> **The k = 3 arm's effective sample was n = 2, not 8.** Of the four `churn3` seeds that
> survive at μ = 0, only seeds 2 and 777 ever reproduce. Seeds 42 and 99 record
> `births_clonal = 0, births_mutant = 0` at *every* μ from 0 to 1.0 — and since
> mutate-at-birth is the only variation channel in the model, μ never fires on them. Their
> survival at high μ is a tautology, not evidence of robustness. `churnK`'s stratum is 8/8
> and every member reproduces.
>
> Conditioned on baseline survival, μ_c(k=1) = (0.02, 0.03] against μ_c(k=3) = (0.01,
> 0.02] — **adjacent grid intervals, not an order of magnitude.** Both arms lose their
> first exposed seed in the same interval, (0, 0.005]. The residual difference still
> points the way this section predicted (exposed extinction at (0.01, 0.02] versus
> (0.05, 0.1]), but on a stratum of two seeds the 0.5-crossing bracket moves a whole grid
> interval when one seed changes.
>
> **Read this section as: the direction is unrefuted, the magnitude is unsupported, and
> the comparison rests on 2 informative seeds against 8.** v6's P0a additionally shows the
> ~50% baseline death rate here is a property of *flat income specifically* — the same 12
> seeds give 4/12 under flat income and 12/12 under v5's proportional income. See
> `selfmod_v6_findings.md` §P2 and §P0a.

---

## E2 — mechanism at the boundary: **closure loss**, unambiguously

**[Measured]** `churnK`, μ = 0.03, seed 1, `log_every = 5`. The run is metronomic for
13930 steps: 155 productive applies/step, `frac_fired` 1.0, `free` pinned at its cap,
`sterile_firings` exactly 0. At step 13930 a **single mutant birth** occurs. Within five
logged steps: `obj_applied` 155 → 86 → 2 → 0. Tokens then bleed out (98 → 92 → 82 → 72 →
62 → 52) and the population starves.

Against the plan's three candidate modes:
- **Closure loss — YES.** One mutant offspring breaks the cycle; the collapse follows the
  mutation immediately and the world never recovers.
- **Starvation — NO.** `free` sits at `free_cap` throughout, before and during collapse.
  The pool was never the constraint (consistent with the flux ≡ stock null).
- **Dilution — NO.** `sterile_firings` is 0 for the entire run. Non-productive load never
  accumulates; the effect gate (G4.2) is doing its job.

**[Measured]** The survivor mode is equally clean. `churn3`, μ = 0.005, seed 2, survives all
20000 steps with `alive_rule_classes` pinned at exactly 3 while `rule_count` oscillates
3 ↔ 6 with births and deaths in lockstep. Persistence here is exact clonal replication of
the seed set — **clonal stasis**, not a living population.

---

## E4 — coresidence under per-lineage budgets (F3): does not rescue

**[Measured]** `churn3 + grower`, flux, 8 seeds:

| configuration | survival | freeze |
|---|---|---|
| v3 E1b, all-or-nothing global clamp | 0/8 | — |
| F3, allowance 200 + 1/step refill | 1/8 | 7/8 froze, median step 194 |
| F3, allowance 40, no refill | 0/8 | 8/8 froze, median step 40 |

F3 changes the *mechanism* — the grower now self-throttles instead of locking coresidents
out — but not the *outcome*. The loose allowance is still larger than the world's headroom
(200 initial + 1/step, against 300 nodes of room below `world_ceiling`), so the global
backstop still engages. Tightening it made things strictly worse and froze the world at step
40, which suggests [Inferred] the grower is not purely parasitic: it manufactures the
label-1 substrate the churn cycle consumes, so throttling it starves the cycle it was
supposed to be protected from. The single survivor in both configurations is seed 42 — again
the seed that never reproduces.

**F3 verdict: the mechanism is right and the calibration is not, but no calibration on this
axis is likely to help, because grower and churner are coupled through the substrate rather
than merely competing for space.**

---

## Gates

- **G4.0 [Measured] PASS** — G0 (10 fixtures round-trip, 5 malformed ⇒ `None`) and G1
  (bit-identical decode-vs-compiled canonical hashes, 3 seeds × 100 steps) still hold under
  every v4 change. Reification stays neutral.
- **G4.1 [Measured] PASS (qualitative)** — stock, μ = 1, `churn3`: 2/8 survive (v3 E1c:
  3/8), survivors have exactly-zero late M-slopes, and all 6 collapsers reproduced before
  dying. Seed 2 flipped survive → die; that is the expected small numeric drift from F1–F3
  (per-lineage budgets alter object-phase gating, `grace_steps` alters termination). The
  qualitative signature is unchanged, so the flux-vs-stock comparison is attributable.
- **G4.2 [Measured] PASS** — 8/8 effect-gate cases. Edge-only deletion and edge-only
  creation both score **productive**; relabel-to-same, empty-RHS, and edge identity-churn
  all score **sterile**.
- **G4.3 [Measured] PASS** — latency target met from I = 5 upward (median
  `time_to_first_fission` = 55, no seed freezing first); operating point I = 20.
- **G4.4 [Measured] PASS** — k = 1 minimal sustainer constructed and verified; `churn3`
  still sustains, so E3's k contrast is real.

---

## Deviations and excursions (ground rule 6)

1. **F1's premise was already false.** v3's effect gate did check edges. G4.2 pins it.
2. **The effect gate is deliberately stricter than the plan's wording.** The plan specifies
   productive iff *any* of label / node / edge changed; a rule that deletes an edge and
   recreates the identical edge satisfies that literally, yet leaves the store
   content-identical — a weed rule wearing edge activity as a costume. It is scored
   **sterile**, and G4.2 pins that case explicitly.
3. **G4.4 measurement fix.** The first run reported `churnK` as non-sustaining: the
   criterion sampled the canonical hash at 100-step window boundaries and `churnK`'s
   attractor has period 2, so every sample hit the same phase. Now sampled on consecutive
   steps, with the detected period reported.
4. **G4.3's criterion relaxed before use.** Requiring *all* 8 seeds to fission would have
   failed a calibration that is in fact fine; 2 seeds settle into a duty cycle that never
   reaches threshold at any I. The criterion uses the median over reproducing seeds.
5. **Two unplanned experiments**, both added to make an existing claim attributable rather
   than to chase an outcome: the **scarce-pool probe** (converts flux ≡ stock from a
   possibly-vacuous null into a characterized scale condition) and the **60k-step boundary
   re-measurement** (tests whether branch (b) was an artifact of the pre-registered
   threshold — it was not).
6. **E3 fine grid added** (μ ∈ {0.005, 0.01, 0.02, 0.03}) because μ_c for k = 1 and k = 3
   were indistinguishable on the coarse grid, leaving the plan's central prediction untested.
7. **Git note.** v4 commits landed on the `v-track` branch, on top of the pre-existing
   V8/V9 research history; v3's work is the tip of `master`. Nothing was lost and both
   branches contain the v3 commits, but v4 is stacked on an active research branch rather
   than isolated — say the word and it moves to its own branch.

---

## Routing — where v5 goes

The data landed on **(b)** for k = 1 and **(c)** for k = 3, and both branches route to the
same place. Per the plan's own instruction for those branches, stated explicitly:

**The static world is now the binding constraint, and the tri-ecology's co-evolving world
moves from "later architecture" to "next experiment."**

The reasoning is no longer inferential. On a static substrate the reachable sustaining sets
are *minimal* churn cycles, and selection has no reason to hold anything more expensive:
`alive_rule_classes` sits immovably at the seed count in every survivor, at every μ, in both
economies, on both seed populations. Complexity never pays because the world never asks a
new question. Meanwhile the only thing variation can do to a minimal cycle is break it —
which is exactly what E2 shows, on a single mutant, after 13930 steps of perfect operation.
Persistence and novelty are disjoint here not because the economy is wrong or the mutation
rate is mistuned, but because a fixed world admits no strategy that is both durable and
improvable.

Three concrete carry-ins for v5, in priority order:

1. **Co-evolving world.** The world must supply a moving target — an exogenous drive, a
   resource landscape that depletes and regenerates, or a second population the rules must
   track. Without it, no μ and no economy will open an overlap.
2. **Reproduction economics, not mutation rate.** The μ = 0 control says fission cost
   destabilizes independent of variation (4/6 clonal reproducers die). Before touching μ
   again, make reproduction cheap or gradual: partial-cost fission, heritable endowments so
   offspring do not strand the parent below threshold, or a refractory period.
3. **Redundancy as an error buffer.** E2's collapse is one mutant breaking a cycle with no
   spare copy of the broken member. E3 says larger sets are *more* fragile under the current
   scheme — which is the opposite of how biological redundancy works, and points at
   mutate-at-birth-on-every-lineage as the culprit. Per-member copy number, so a lineage can
   lose one instance and continue, is the natural test.

## Promotion assessment (carried forward)

Unchanged from v3, and now better supported: **do not feature-gate the decode phase into
`hg-engine::step()`.** G1 still holds bit-identically, the library-driver pattern absorbed
every v4 addition — a second economy, per-lineage budgets, μ control, a flow audit — with
zero engine changes, and v4 gives no result that would be easier to obtain from inside the
engine. Promotion remains a full-reflection-milestone question, and that milestone should
now wait behind the co-evolving world, since hosting these dynamics in the engine would
mean hosting a system we have measured to be either permanent or creative but never both.
