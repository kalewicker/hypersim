# F-track — findings: the V1 negative was a resolution artifact

Reported against `docs/validation_v2_fix_plan.md`. Nothing in this track added a mechanic,
changed the economy, or altered a rule. The one knob it touched — WL iteration depth — was
**pre-registered before any coefficient at the new depth was computed** (§F1 below).

**Harnesses** (all passing):

| Phase | File |
|---|---|
| F0 | `crates/hg-sim-core/tests/f0_partition_resolution.rs` |
| F1 | `docs/validation_v2_f1_depth_preregistration.md` (committed `92b3ef8`) |
| F2 | `crates/hg-sim-core/tests/f2_ce_validation_depth2.rs` |
| F3 | `crates/hg-causal/src/ei.rs` (`canonical_order`), `crates/hg-causal/tests/tpm_index_order_cross_process.rs` |
| F4 | `crates/hg-sim-core/tests/f4_satiation_power.rs` |
| F5 | inline `[F5-n]` corrections in `docs/validation_v1_findings.md` |
| F6 | `docs/validation_v2_preregistrations.md` |

---

## The single sentence

**The carrier group resolves into more than one WL class at depth 2, and the V1 negative was
therefore an artifact of the partition's resolution rather than a measurement.**

## Headline

| Question | V1 (depth 1) | F-track (depth 2) |
|---|---|---|
| carrier WL classes | **1** | **4** |
| partition group sizes | (8, 1) | (196, 4) |
| permutation-null support | **~10** | **6.47 × 10⁷** |
| real CE (6b, un-pinned) | 0.2497 | **0.1633** |
| null mean | 0.2289 | 0.0440 |
| null ≥ real | 99/200 (p = 0.495) | **0/200 (p < 0.005)** |
| verdict | "boundary spurious" | **boundary carries partition-specific causal signal** |

**The direction of travel is the strongest evidence this is not tuning.** F1 pre-registered
that greater depth would *lower* CE (larger `M` → larger `EI_micro` → smaller
`EI_macro/EI_micro`). It did: 0.2497 → 0.1633. The fix improved **discriminability** while
moving the **magnitude** unfavourably. Tuning produces the opposite signature.

---

## F0 — the diagnostic

### F0.1 — the configured depth

The CE pipeline ran at **WL radius 1**, as a hardcoded literal at four separate call sites
(`experiment6_ce_across_onset.rs:102`, `experiment6b_birthdeath_ce.rs:95`,
`v3_ce_validation.rs:101`, `runner.rs:1045`), never as a parameter or shared constant.
`hg-causal` and `hg-eval` agree at `WL_RADIUS = 1`; `hg-selfmod` independently chose a
configurable default of **3**. The CE stack was internally consistent — the problem was the
value, not a mismatch.

### Why depth 1 could not have worked

`compute_wl_hash`'s `base_hash` is `WL_SEED + label + energy` **field**, and ALife energy
lives in *property key 1*, which WL deliberately excludes (the Q2 invariant, so per-carrier
genome traits cannot mint a class each). Therefore:

- every substrate site has an **identical base hash**;
- a carrier has exactly **one** neighbour, its occupancy site;
- so at radius 1 every carrier hashes to `f(carrier_base, {site_base})` — **identical for
  every carrier, regardless of which site it occupies**.

Sites split at radius 1 only because their neighbour-multiset *sizes* differ (degree,
occupancy). The site's own neighbourhood — the thing that actually varies — first enters the
carrier's hash at **radius 2**. The (9, 1) partition was structurally forced by the depth.

### F0.2 — carrier classes by depth (no CE computed)

| d | total M | carrier classes | substrate classes | group sizes |
|---|---|---|---|---|
| **1** | 9 | **1** | 8 | **(8, 1)** |
| **2** | 145 | **4** | 141 | (141, 4) |
| 3 | 1427 | 71 | 1356 | (1356, 71) |
| 4 | 3834 | 1021 | 2813 | (2813, 1021) |
| 5 | 6503 | 2356 | 4147 | (4147, 2356) |

Depth 1 reproducing `M = 9` matches the `M = 9` reported for 6a in
`validation_v1_findings.md` exactly — confirming this diagnostic measures the same partition
the CE pipeline built. **Gate passed: carriers resolve at d = 2.**

### F0.3 — occupancy

Carriers occupy **4 of the 8** depth-1 substrate classes (44.5 / 44.4 / 5.5 / 5.6 %,
n = 10 109). Occupancy is *spread*, not concentrated on one class, so genuine site
heterogeneity was available to inherit — carriers simply could not express it at depth 1.
This rules out the dispersal-confinement branch of the F0 decision tree.

---

## F1 — depth pre-registration

Committed in `92b3ef8`, **before** any CE at a depth other than 1.

- **Rule 1 (minimum resolving depth) selects d = 2.** Decided from the F0.2 table alone,
  which contains no CE.
- **Rule 2 (convergence depth) selects nothing.** `M` never stabilises in 1..5, growing
  9 → 145 → 1427 → 3834 → 6503, with 2356 carrier classes at d = 5 against only ~35 live
  carriers. That is the open-registry class-explosion regime the WL Q2 comment warns about,
  not convergence. Recorded as an **absence** rather than substituted with the largest tested
  depth.

The pre-registration states explicitly that d = 2 is *also* the depth least likely to depress
CE, so that coincidence could not later be mistaken for a favourable post-hoc choice.

---

## F2 — the battery at depth 2

### F2.1 — reproducibility ✅

CE, EI, K, M, transition counts **and all derived statistics** bit-identical across three
processes. This required fixing two further instances of the F3 hazard (below).

### F2.2 — boundary reality: **REVERSED** ✅

| model | real CE | null mean | null p95 | null max | null ≥ real |
|---|---|---|---|---|---|
| 6a surviving-only | 0.1783 | 0.0449 | 0.0739 | 0.1058 | 0/200 |
| **6b vacuum-augmented** | **0.1633** | 0.0440 | 0.0742 | 0.0893 | **0/200** |

`p < 0.005` (rule-of-three 95 % upper bound ≈ 0.015) at a null support of **6.47 × 10⁷**
distinct partitions. V1's p = 0.495 came from a null with roughly ten members — a test that
**could not have returned a positive**.

The temporal form of the control remains **unconstructible** (longest birth-free stretch 58
steps against 800 needed) and is reported as absent, per plan rule 4.

### F2.3 — sensitivity, degenerate cases separated ✅

| perturbation | non-degenerate CE mean | non-degenerate range | degenerate draws |
|---|---|---|---|
| reassign 1 class | 0.1218 | [0.0991, 0.1585] | **0/40** |
| reassign 2 classes | 0.1018 | [0.0796, 0.1633] | **0/40** |
| reassign 3 classes | 0.0931 | [0.0771, 0.1238] | **0/40** |

Real CE 0.1633 sits **at or above every non-degenerate range** — the real partition is a
**local optimum**, not an arbitrary point in a wide band. With 4 carrier classes no small
perturbation can empty the carrier group, so V1's `0.0000` endpoint (which was the `K = 1`
degenerate partition) does not arise at all. Temporal robustness is preserved:
0.163256 → 0.163714 across a +10 window shift.

### F2.4 — the F1 bound: **flips against the headline** ⚠️

| registry | M | EI_micro | CE |
|---|---|---|---|
| open | 200 | 5.6088 | 0.1633 |
| closed to 6 active | 8 | 1.9593 | 0.5222 |
| closed to 4 active | 6 | 1.4248 | 0.6475 |
| closed to 3 active | 5 | 1.3279 | 0.6922 |
| **closed to 2 active** | **4** | **0.8741** | **1.0475** |

Max registry lever **+0.8843** against a gap-to-1.0 of **0.8367** — the lever now **exceeds**
the gap. At depth 1 the lever was only +0.07 and V1 concluded F1 was closed as a rescue;
**that conclusion does not survive at depth 2**.

The honest statement is not "F1 is closed" but the sharper one: **the CE magnitude is
registry-dominated and must not be reported as a quantity of emergence.** A CE > 1 obtained
by collapsing the registry to 4 micro states is precisely the forbidden move and is worth
nothing.

**This does not touch F2.2**, which is a *within-registry* comparison (real vs permuted
partitions at fixed registry) and is therefore unaffected by registry granularity.

---

## F3 — the index-order invariant, made structural

The V3.1 methodological finding bit **three separate times** — a permutation-null p-value, a
top-N registry ranking, and a `hashes[i]` mis-attribution — always with CE reproducing
perfectly while the statistic drifted.

- **F3.1** — `TpmAccumulator::canonical_order()` is now the single blessed implementation,
  carrying the explanation of why CE cannot detect the hazard plus a `debug_assert` on its
  own output. `ei_dump` calls it instead of re-deriving the sort inline, so production and
  every analysis harness share one rule.
  > The plan asked for an assertion that indices are hash-sorted *at construction*. That
  > cannot hold: `index_of` assigns in first-seen order by design, and changing it would
  > churn dense indices for every consumer (including pinned `ei_dump` output) to no benefit,
  > since CE is already invariant. One shared helper achieves the intent without the churn.
- **F3.2** — a self-exec'd two-process test using statistics that **rank and permute**, with
  a fixture that deliberately drives `record()` from `HashMap` iteration so there is a real
  hazard to catch, and CE pinned alongside as a control. **Verified to have teeth**: swapping
  `canonical_order()` for raw dense order makes it fail across processes.
- **F3.3 audit** — every pre-existing consumer was already safe:

| consumer | status |
|---|---|
| `hg-causal/src/observer.rs::ei_dump` | canonical (sorts by hash; now shared) |
| `hg-causal/tests/ei_calculator.rs` | safe — `group_of` by hash *value* |
| `experiment6b_birthdeath_ce.rs` | safe — `group_of` per dense id, CE only |
| `experiment6c_population_ce.rs` | safe — `group_of` by hash value (count N) |
| `v3_ce_validation.rs`, `f2_ce_validation_depth2.rs` | canonicalised |

**Why it never surfaced before:** no prior consumer computed a statistic *over* indices. That
category arrived with the V-track's permutation nulls.

---

## F4 — criterion 5: `satiation` was underpowered, not selected

V2.1 rested on n = 6 established seeds, where `satiation` looked like the one trait that
might carry non-degenerate selection — 83 % directional, 1.69× the drift envelope, a *larger*
effect ratio than `replication_threshold`'s. F4 replicates the identical measurement at the
pre-registered n ≥ 30.

| trait | mean Δ | \|Δ\| | directional | sign-test p | \|Δ\|/drift |
|---|---|---|---|---|---|
| null `harvest_rate` | — | 3.78 | 17/30 up | 0.585 | — |
| null `maintenance_cost` | — | 3.64 | 12/30 up | 0.362 | — |
| `replication_threshold` | −3.91 | 4.18 | **27/30 down** | **8.43 × 10⁻⁶** | 1.13 |
| `satiation` | −0.68 | 3.83 | 12/30 up | 0.362 | 1.03 |

Drift envelope 3.71. Both inert traits are sign-mixed and non-significant, so the null is
trustworthy.

**F4.2 pre-committed rule: sign test significant AND ratio > 1.5. `satiation` fails both
(p = 0.362, ratio 1.03).** The V1 verdict stands, now properly powered at n = 30 rather
than 6.

The V2.1 signal did not merely weaken — it **dissolved and changed sign**: 83 % up → 40 % up,
mean +4.28 → −0.68, ratio 1.69 → 1.03. It was small-n noise. `replication_threshold` is
unchanged in character: reproducible by **sign** (27/30 down, p = 8.4 × 10⁻⁶) but with
magnitude still **inside** the drift band (1.13×) — the degenerate collapse, confirmed at
higher power.

### F4.3 — the structural cause

Deviation #4 kept harvest and metabolise on fixed quanta rather than genome-scaled, so
`harvest_rate` and `maintenance_cost` are read by no rule: **2 of 4 genome traits can only
drift**. Of the remaining two, `replication_threshold` is unopposed under node-funded
replication. That leaves `satiation` as the **only** trait able to express a trade-off at
all. **The deviation that made the economy tractable is what made selection undetectable** —
criterion 5 was close to unsatisfiable *by construction*. This is a pre-committed-deviation
consequence, recorded beside the result rather than discovered later.

---

## F5 — corrections applied to `validation_v1_findings.md`

Applied inline as `[F5-n]` blocks, with the surrounding text left standing so the reasoning
that produced the error remains legible:

1. **Fork re-labelled.** "Boundary spurious" contradicted V2.2 in the same document. Correct
   framing: *the boundary is real; the macro was underspecified.* The remedies are opposite —
   abandon vs enrich — and the F-track follows from the second reading.
2. **V3.3 range qualified.** The `0.0000` endpoint is the `K = 1` degenerate partition, not
   evidence of sensitivity.
3. **F1 bound stated quantitatively, and its depth-dependence recorded** (see F2.4: the
   depth-1 conclusion does not generalise).
4. **Experiment substitution flagged.** Experiment 6 was specified as CE *across the
   replication-onset boundary* — a temporal comparison. What was computed is a
   carrier/substrate **spatial** partition during sustained turnover. On a static substrate
   the temporal experiment is **degenerate, not merely unconstructible**: V4.2 confirms zero
   back-reaction, so pre-boundary micro dynamics are empty and before/after reduces to
   "nothing versus something."
5. **Superseded reasoning owned.** The 1.47× heterogeneity requirement and the route-A/route-B
   framing were conditioned on a regime with no band. Turnover was unlocked by bootstrap
   threshold 15 → 3 at a satiation spread of only 1.161×. Recorded as a **prediction
   failure**, not quietly dropped.

A procedural lesson is recorded with correction 1: V1's own "Limitations" section already
named the fix ("a partition with more than one carrier class would test it better") and did
not act on it. **When a control's support is small enough to be worth quoting as a caveat,
that is a signal the test may be unable to return a positive at all — support size should be
computed before the p-value is interpreted, not after.**

---

## F6 — pre-registrations

See `docs/validation_v2_preregistrations.md`: (F6.1) `harvest_rate` has drifted to −13.9 from
a seed of 1.0 and is a trap for any future run that reverses deviation #4 on an evolved
population; (F6.2) turnover establishment is **32 % (30/94)** on an unbiased sweep, not the
57–71 % the record carried from small curated seed sets.

---

## F8 — multi-seed replication: the boundary is detectable across the regime

Pre-registration: `docs/validation_v2_multiseed_preregistration.md` (committed `43314c2`,
before any multi-seed CE). Harness: `crates/hg-sim-core/tests/f7_multiseed_boundary.rs`. The
F2 boundary test — identical bar the seed — was run across seeds 1..30 (disjoint from 12345,
which is retained as an independent prior and reproduced exactly through this streaming code
path as a fidelity anchor: CE 0.1633, 0/200).

| quantity | value |
|---|---|
| attempts | 30 (all 30 included) |
| surviving to end | 8/30 (**27 %**, consistent with F6.2's 32 %) |
| **fraction with permutation p < 0.05** | **30/30 = 1.00** |
| cross-seed sign test (real CE > null mean) | 30/30, binomial **p = 1.9 × 10⁻⁹** |
| reversals (real CE below its own null) | **none** |
| median real CE | 0.1604 |
| min null support | 6.4 × 10⁶ |

**Pre-committed verdict (f ≥ 0.90): ROBUSTLY DETECTABLE ACROSS THE REGIME.** The F2 result
generalises; seed 12345 was representative. The carrier/substrate boundary carries
partition-specific causal signal as a property of the establishing regime, not of one seed.
Even the two weakest seeds (17, 26 — lowest transition counts, smallest support ≈ 8–10 × 10⁶)
clear the bar at p ≤ 0.010.

**A stronger sub-finding than the pre-registration anticipated: detectability does not require
survival.** The pre-registration expected ~32 % inclusion, budgeting 120 attempts for 30
included. Instead **all 30 attempted seeds were included**, because a seed that ultimately goes
extinct still passes through a bloom-and-turnover transient generating ~220 k carrier/substrate
transitions with 4 carrier classes — enough to define and pass the test. The boundary is
detectable in the 22 seeds that did **not** survive to the end, as well as the 8 that did.
Survival correlates only weakly with the coefficient: surviving seeds run ~421 k transitions
and CE ≈ 0.164–0.169, non-surviving ~220 k and CE ≈ 0.155–0.162 — both far above their nulls
(≈ 0.04–0.07).

This **closes the "generality across seeds" caveat** that F6.2 flagged as the largest
outstanding one on F2. It does **not** touch the other caveats below.

---

## What is and is not licensed now

**Established.** The carrier/substrate boundary is distinguishable from an arbitrary
same-sized partition at WL depth 2 on the 6b vacuum-augmented model — **at seed 12345**
(p < 0.005, null support 6.47 × 10⁷) **and across the establishing regime** (F8: 30/30 seeds
p < 0.05, cross-seed sign test p = 1.9 × 10⁻⁹, no reversals), including seeds whose population
does not survive. The real partition is a local optimum under perturbation and is temporally
robust. The V1 "boundary spurious" verdict is withdrawn.

**Not established, and not claimed.**

1. **That CE > 1, or that "causal emergence" occurs.** CE is 0.1633 — coarse-graining still
   loses information. What changed is that the partition demonstrably *detects* the boundary,
   not that the macro beats the micro.
2. **Any magnitude claim.** F2.4 shows registry choice alone can move CE by +0.88, more than
   the distance to 1.0. The coefficient's *value* is not a stable property of the system.
3. ~~**Generality across seeds.**~~ **[CLOSED by F8]** Detectability generalises across the
   regime (30/30 seeds, p = 1.9 × 10⁻⁹). Note this is **detectability**, not magnitude — the
   CE *value* remains single-seed and registry-dominated (caveat 2 stands). And the CE numbers
   in Experiment 6a/6b/6c specifically remain single-seed measurements; F8 licenses the
   boundary's *reality across seeds*, not those particular coefficients' values.
4. **Non-degenerate selection.** F4 confirms criterion 5 is met via the degenerate route only,
   at n = 30.

**One interpretive caveat on F2.2 that should not be buried.** The 6b model adds ∅↔carrier
transitions *by construction* — only carriers are born and die — so "the carrier group is
special with respect to turnover" is partly baked into the model's design. What is *not*
baked in is the depth-1 → depth-2 change: the identical construction failed at depth 1
(p = 0.495). The improvement is genuinely resolution; the absolute result partly reflects the
vacuum model's design.

**The next measurement, now done (F8):** multi-seed F2 replication converted "detectable at
seed 12345" into "detectable across the regime" — 30/30 seeds, and even in seeds that do not
survive. See the F8 section above.

**The measurements that remain genuinely open**, none of which F8 addresses:

- **The vacuum-model construction caveat.** F8 shows detectability is seed-robust, but every
  seed uses the same 6b model in which only carriers touch ∅ by construction. Seed variation
  cannot separate "the boundary is biologically special" from "the model only lets carriers
  cross." A measurement that could: a partition-agnostic control class that is also born/dies
  (if the substrate had turnover), or a non-vacuum micro dynamics.
- **Magnitude.** F2.4 stands: the CE *value* is registry-dominated and is not a stable
  property. No amount of seed replication changes this.
- **A non-static substrate**, which is the root of the identity-macro problem and the reason
  the temporal form of Experiment 6 is degenerate here (F5-4).
