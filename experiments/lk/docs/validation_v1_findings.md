# V-track — findings: pre-experiment validation of the Experiment-5/6 stack

> # ⚠ SUPERSEDED IN ITS HEADLINE by the F-track
>
> **The V3.2 negative below was a measurement artifact, and the "boundary spurious" label
> was wrong.** See `docs/validation_v2_fix_plan.md` and `docs/validation_v2_findings.md`.
>
> This whole track ran at **WL iteration radius 1**. At that depth a carrier's neighbourhood
> is exactly `{site}`, and every site has an identical `base_hash` (ALife energy is property
> key 1, excluded from WL by the Q2 invariant) — so **every carrier hashed identically by
> construction**, regardless of which site it occupied. That is the entire origin of the
> (9, 1) partition, and hence of a permutation null with support ≈ 10. A test with ten
> members returning p = 0.495 **could not have returned a positive**.
>
> At the pre-registered depth 2 the carrier group resolves into 4 classes, the null support
> becomes 6.47 × 10⁷, and the real boundary is beaten in **0/200** permutations (p < 0.005).
> **The boundary carries partition-specific causal signal.** Note CE *fell* (0.2497 → 0.1633)
> while discriminability rose — the fix moved the magnitude unfavourably, which is the
> opposite signature from tuning.
>
> What survives unchanged: V0 (instrument sound), V1 (no fork), V2 (all four gates), V4
> (pipeline integrity), and the CE *coefficients* themselves. What is retracted: the V3.2
> verdict, the "boundary spurious" fork label, the V3.3 sensitivity range as quoted, and the
> V3.4 bound. Corrections are marked **[F5]** inline below; the surrounding text is left
> standing so the reasoning that produced the error remains legible.

Reported against `docs/validation_v1_plan.md` (written before any V-number was seen; copied
into the repo verbatim alongside these findings, matching the preregistration/findings
convention). The plan is the gate between "the code runs and the criteria pass" and "a
result I can report." Nothing in this track added a mechanic or tuned a parameter.

**Harnesses** (all passing):

| Phase | File |
|---|---|
| V0 | `crates/hg-sim-core/tests/v0_instrument_soundness.rs` |
| V1 | `crates/hg-sim-core/tests/v1_frozen_run_reads.rs` |
| V2 | existing: `alife_criterion5_selection_vs_drift.rs`, `alife_criterion6_nontrivial_robustness.rs`, `alife_band_reality.rs`, `alife_s6_influx_sweep.rs`, `alife_criterion7_{sweep_reproducible,energy_across_sweep,paired_control_sweep}.rs` |
| V3 | `crates/hg-sim-core/tests/v3_ce_validation.rs` |
| V4 | `crates/hg-sim-core/tests/v4_substrate_fingerprint.rs`, `v4_2_experiment_paired_control.rs`, clean-room worktree |

> **This document qualifies `experiment6_findings.md`, `experiment6b_findings.md`, and
> `experiment6c_findings.md`.** Those docs report CE magnitudes (0.3665 / 0.2500 / 0.3648)
> and are arithmetically correct and fully reproducible. What this track adds is that the
> **carrier/substrate partition those numbers are computed over is not distinguishable from
> an arbitrary partition of the same run.** 6b's "clean negative on causal emergence" is
> therefore not a negative *about the replication boundary* — it is a number that does not
> resolve the boundary either way. Read those docs for the coefficients; read this one for
> whether they mean what they were taken to mean.

---

## Headline

**The instrument is sound, the pipeline is reproducible, the biological crossing is real —
and CE across the replication boundary is not measuring that boundary.**

| Question | Answer |
|---|---|
| CE coefficient | **0.2497–0.2500** (6b, the only un-pinned model); 0.3665 (6a); 0.3648 (6c population) |
| Substrate it ran on | **Static** 36-node / 85-edge `PureCrystal` 6×6 arity-3, proven inert against a same-seed bare control |
| Did the boundary survive V3.2? | ~~**No.** p = 0.495~~ **[F5-1] RETRACTED** — p = 0.495 came from a null with ~10 members. At depth 2: **0/200, p < 0.005** |
| Fork branch | ~~**"Boundary spurious"**~~ **[F5-1] MISLABELLED.** Correct framing: **the boundary is real; the macro was underspecified** |

> **[F5-1] The fork label was wrong, and wrong in a way this document contradicts itself
> about.** "Boundary spurious" sits three sections away from V2.2, which establishes that the
> `NonTrivial` 0→>0 crossing coincides with a real depth-2 sustained self-copying event, and
> from V0.5, which establishes the classifier can withhold the label. The boundary was never
> in doubt. What was degenerate was the **macro description** — a 2×2 partition with a single
> carrier row. The two readings prescribe opposite remedies: a spurious boundary means
> *abandon*, an underspecified macro means *enrich the macro*. The entire F-track follows
> from taking the second reading, and it reversed the result.

**Nothing was tuned.** One assertion (V3.2) was converted to a reported verdict because the
plan pre-registered that outcome as a *finding*; no threshold, partition, influx rate, or
coefficient was altered to change it.

---

## V0 — Instrument soundness: **5/5 PASS**

The measuring device is sound. Every downstream number is trustworthy.

| Test | Gate | Result |
|---|---|---|
| V0.1 read-only proof | identical | `canonical_hash` **and** total-energy traces bit-identical with vs without `AlifeObserver`, 1500 live steps |
| V0.2 population conservation | holds every step | `carriers(t) = carriers(t−1) + births − deaths` held **every step** over 2000 steps (1124 births, 1088 deaths) |
| V0.3 SlotMap NodeId reuse | no ancestry bleed | `NodeId` is generational (`NodeId(1v1)`); a killed carrier's slot recycled 4× yielded fresh keys with zero inherited ancestry |
| V0.4 events-alone reconstruction | set equality | reconstructed live set == graph carrier set at **7 checkpoints** across 2000 steps |
| V0.5 NonTrivial negative control | separation | withheld from a trivial copier (depth 1 → 0); assigned to a sustained lineage (depth ≥2 → 1) |

**V0.5 had effectively run before** at unit level (`hg-alife/tests/lineage.rs`); it is now an
explicit two-carrier control. The classifier *can* withhold `NonTrivial`, so the 0→>0
crossing is not a label artifact.

V0.1 and V0.3 are additionally sound *by construction*: `Observer` receives
`&GraphSnapshot` and is invoked after all mutation and RNG draws (`engine.rs:208`), and
`NodeId` is a slotmap generational key. The tests convert "guaranteed" into "guarded."

---

## V1 — Frozen-run reads: **no fork**

No persisted ecology run exists on disk (the contamination firewall keeps only
substrate/LK runs), so V1 ran a fresh deterministic run of the canonical turnover regime —
which reproduces the frozen run bit-for-bit. Seed 12345, influx 0.34, bootstrap threshold
3.0, 15 000 steps, 33 live carriers at freeze.

### V1.1 — `replication_threshold` vs `satiation`: **DOES NOT FORK** ✅

| trait | mean | var | min | max |
|---|---|---|---|---|
| `replication_threshold` | **−3.716** | 0.749 | −6.094 | −2.508 |
| `satiation` | **23.896** | 0.699 | 22.127 | 25.682 |

Threshold sits far **below** satiation; **0/33** carriers are sterile-by-genome.
Reproduction is structurally reachable at equilibrium. The influx sweep, the
`child_node_cost` refutation, and the synchronized-death argument are **not** measuring a
misconfigured genome.

### V1.2 — satiation heterogeneity budget

`max/min = 25.682 / 22.127 = ` **1.161×**, below the 1.47× squeezed-turnover window. A
partial endogenous death-desynchronization budget exists but is not decisive; route A's
motivation is reduced, not eliminated.

### V1.3 — trait shift is drift + degeneracy, **not selection**

| trait | seed → freeze | Δ | reading |
|---|---|---|---|
| `harvest_rate` | 1.00 → −13.948 | **−14.948** | **inert** under the fixed-quantum economy ⟹ pure random walk |
| `replication_threshold` | 3.00 → −3.716 | −6.716 | **degenerate** — node-funded, nothing opposes it |
| `maintenance_cost` | 1.00 → 0.258 | −0.742 | inert |
| `satiation` | 20.00 → 23.896 | +3.896 | the only trade-off-bearing trait; small move |

The **largest** move belongs to a trait no rule reads. Criterion 5 is satisfied by drift on
inert traits plus a free collapse on the unopposed one.

### V1.4 — clade structure, and a refuted premise

`clade_count` 1 (single bootstrap founder), `surviving_subclades` 1, `max_generation` **500**,
8485 births / 8453 deaths, Newick well-formed (46 975 chars, parses).
Survival curve head: `[1, 6, 11, 15, 13, 12, 13, 12, 9, 9, 10, 9]`.

**The plan's "zero mortality" premise does not hold.** Realized mortality is 8453 deaths —
this is a live turnover curve, not a degenerate zero-mortality one.

---

## V2 — Verdict integrity: **4/4 gates pass**, two substantive findings

### V2.1 — Criterion 5 vs a matched drift null: **degenerate route only**

Null = the two inert traits (identical jitter, identical birth/death lineage sampling, zero
fitness coupling). Drift envelope (mean inert |Δ|) = **4.47**.

| trait | mean Δ | \|Δ\| | frac down | magnitude / drift |
|---|---|---|---|---|
| `replication_threshold` | −5.45 | 5.45 | **100 %** | 1.22 |
| `satiation` (trade-off) | +4.28 | 7.56 | 17 % | 1.69 |

`replication_threshold` is distinguishable from drift **by sign, not magnitude** — its
effect size sits *inside* the drift band. `satiation` shows no clean trade-off selection.
**Criterion 5 is met only via the pre-registered degenerate route.** The scorecard's
0.14 → −3.72 is not meaningful as a magnitude.

Also surfaced: the turnover regime **established in only 6/10 seeds**.

### V2.2 — Criterion 6 crossing: **structurally real** ✅

The 0→>0 crossing (window 0, ~step 200) coincides with an actual depth-2 sustained
self-copying event (`max_generation` 13, count 47). Trivial copiers carrying identical
labels are withheld. **Not a WL/label artifact.**

### V2.3 — Band reality: **real interval, but seed-fragile**

Two sweeps, reconciled:

- **Default bootstrap threshold (15):** no band at all — discontinuous EXTINCT (≤0.480) ↔
  FROZEN (≥0.500) at ~0.490 against pre-registered r\* = 0.3333. `TURNOVER band: []`.
- **Low bootstrap threshold (3, the turnover slice):** a genuine **interval** — TURNOVER at
  0.28 / 0.34 / 0.40, FROZEN at 0.50. Not a razor-thin single value.

**Trend, not just variance:** at seed 12345 the late window is genuinely stationary —
carriers mean 33.77, var 2.6525, OLS slope −3.757e−6 /step (systematic drift −0.02 carriers
across the window). Not a slow climb.

**Seed robustness — fails softness:** establishment 5/7 (71 %) at 0.28, 4/7 (57 %) at 0.34
and 0.40. ⟹ **a fragile turnover regime, not a robust carrying capacity.** Reported as
fragile, per the plan.

### V2.4 — Reproducibility spine: **all three clean** ✅

- **Two-process band identity:** sweep signature `0x9845f145815ae1ad` — matches the constant
  pinned in a prior process *and* reproduces across two fresh processes.
- **Energy books balance at every sweep point** (0.28 → 1.00), max divergence ~1e−9 (f64
  noise). The **death-residual term realizes to exactly 0.000**: carriers land precisely on
  zero rather than overshooting, so the overshoot-scaling energy source the plan flagged is
  accounted for and vanishes in this economy. The band is interpretable.
- **Paired bare-substrate control at every sweep point:** substrate (36 nodes, 85 edges)
  identical at all rates; carriers only *add* nodes. No back-reaction.

---

## V3 — CE validation: **the boundary did not survive** ❌

All comparisons below are computed from a **single** live run whose per-observation WL class
maps were retained, so every partition is scored on identical dynamics. Any CE difference is
attributable to the partition choice alone.

### Fidelity guard

The V3 reimplementation is asserted to reproduce the official coefficients: 6a reduces
exactly to `CE = 1/EI_micro` (identity macro), and 6b lands within 0.02 of the published
0.2500. This negative is a fact about the system, not about this file.

### Baselines (seed 12345, 6000 steps, cadence 1)

| model | CE | EI_micro | EI_macro | K | M | transitions |
|---|---|---|---|---|---|---|
| 6a surviving-only | 0.3665 | 2.7287 | **1.0000** | 2 | 9 | 414 559 |
| 6b vacuum-augmented | 0.2497 | 2.7265 | 0.6809 | 2 | 10 | 421 302 |

Real partition group sizes: **(9, 1)** — it isolates a **single carrier WL class** from nine
substrate-side classes. Verifiable from the two rows above: 6a (no ∅) has M = 9 and 6b
(with ∅) has M = 10, so the decomposition is 1 carrier class + 8 substrate classes + ∅.
All carriers are structurally alike (each is one node attached to one site), so WL assigns
them a single class. This is load-bearing for everything below.

### V3.1 — Reproducibility: **PASS** ✅

Two processes: CE, EI, K, M and transition counts **bit-identical**.

> **Methodological finding.** The first attempt was *not* reproducible, and the cause is
> exactly the failure mode V3.1 exists to catch. `TpmAccumulator` assigns dense class
> indices in **HashMap-iteration order**, which is `RandomState`-seeded per process. The CE
> coefficient is invariant to that permutation; **any statistic computed over partition
> indices is not.** The permutation null and the registry ranking silently differed between
> processes (null≥real 107 vs 115; closed-registry M=5 vs M=4), and the first V3.4 numbers
> (CE 0.76 / 0.62 / 0.54) were that artifact. Fixed by canonicalising on WL hash.
> **Any future analysis that ranks, permutes, or samples micro classes must canonicalise
> first.**

### V3.2 — Boundary reality: **NOT SURVIVED** ❌

Label-permutation null: the real group labels shuffled across classes, preserving group
sizes exactly (so `K` and macro dimensionality match); only the correspondence with
carrier-ness is destroyed. 200 replicates.

| model | real CE | null mean | null p95 | null max | null ≥ real |
|---|---|---|---|---|---|
| 6a surviving-only | 0.3665 | 0.2725 | 0.3665 | **0.3665** | 30/200 |
| 6b vacuum-augmented | **0.2497** | 0.2289 | 0.3117 | 0.3117 | **99/200 (p = 0.495)** |

- **6b** — the only model whose `EI_macro` is not structurally pinned, and therefore the only
  one that can falsify anything — puts the real boundary **at the null median**. An arbitrary
  same-sized partition does as well or better **half the time**. Margin +0.0208, far inside
  the null spread.
- **6a's apparent margin is the identity-macro artifact.** Its null *max* equals the real
  value **exactly** (0.3665), because any partition with no boundary crossings attains
  `EI_macro = log2(K)`, the maximum possible. 30/200 permutations tie it.

**Temporal form of the control: CANNOT BE CONSTRUCTED** (plan rule 4). Replication onset is
at observation 15; 3388 births occur; the longest birth-free stretch is **58 steps** against
the 800 required. In sustained turnover every window straddles replication, so the temporal
cut is undefined here. Its absence is reported, not engineered around.

### V3.3 — Coarse-graining sensitivity: **partition-fragile, temporally robust** ❌

| perturbation | CE mean | CE range |
|---|---|---|
| reassign 1 class | 0.2012 | **[0.0000, 0.3668]** |
| reassign 2 classes | 0.1963 | [0.0003, 0.3180] |
| reassign 3 classes | 0.2072 | [0.0832, 0.3668] |

| window start shift | 0 | +1 | +2 | +5 | +10 |
|---|---|---|---|---|---|
| CE | 0.249748 | 0.249800 | 0.249852 | 0.250008 | 0.250273 |

A sharp dissociation: CE barely moves when you change **where in time** you cut, and swings
across its **entire range including zero** when you change **which classes** you group. The
coefficient is measuring the partition choice, not the system.

> **[F5-2] The quoted range conflates two different perturbations.** The `0.0000` endpoint is
> the reassignment that moved the *sole* carrier class into the substrate group, leaving an
> empty carrier group, `K = 1`, and `CE = 0` **by construction**. That is a degenerate
> partition, not evidence of sensitivity — and quoting it against V3.4's ±0.07 registry
> effect compares two incommensurable things. It was only reachable at all because depth 1
> gave exactly one carrier class.
>
> At depth 2 (F2.3), with 4 carrier classes, **0/40 draws are degenerate at every j**, and the
> non-degenerate ranges are j=1 `[0.0991, 0.1585]`, j=2 `[0.0796, 0.1633]`, j=3
> `[0.0771, 0.1238]` — all at or below the real CE of 0.1633. So the real partition is a
> **local optimum**, not an arbitrary point in a wide band. The "partition-fragile" half of
> the dissociation does not survive; **the temporal-robustness half does** and is a real
> result worth keeping (CE moved only 0.249748 → 0.250273 across a +10 shift here, and
> 0.163256 → 0.163714 at depth 2).

### V3.4 — F1 bias direction: **weakly consistent, not clean** ⚠️

| registry | M | EI_micro | CE |
|---|---|---|---|
| open | 10 | 2.7265 | 0.2497 |
| closed to 6 active | 7 | 2.2400 | 0.2668 |
| closed to 4 active | 5 | 1.7263 | 0.2860 |
| closed to 3 active | 4 | 1.2691 | **0.3201** |
| closed to 2 active | 3 | 0.9239 | 0.2849 |

Direction matches F1 (closing the registry raises CE) but the trend is **non-monotonic** and
the magnitude (~+0.07) is dwarfed by the ±0.37 partition sensitivity of V3.3. F1 is **not**
cleanly cashed out; the caveat should stand as a caveat rather than a confirmed correction.

> **[F5-3] State it as a bound — and note the bound is depth-dependent, which flips it.**
>
> *At depth 1 (this track):* the registry lever is **+0.07** against a required climb of
> 0.25 → 1.0, i.e. **≈ 10× short**. On these numbers the F1 downward-bias argument **cannot**
> be invoked as a reason a real effect is masked, and F1 closes as an available rescue. This
> is the stronger, more useful claim the plan asked for, and it holds *at depth 1*.
>
> *At depth 2 (F2.4):* the lever grows to **+0.884** (closing the registry to 4 micro states
> yields CE = 1.0475) against a gap of **0.837** — the lever now **exceeds** the gap. The
> depth-1 conclusion therefore **does not generalise**. The correct statement at depth 2 is
> not "F1 is closed" but the sharper and less comfortable one: **the CE magnitude is
> registry-dominated and must not be reported as a quantity of emergence.** A CE > 1 obtained
> by collapsing to 4 states is precisely the forbidden move and is worth nothing.
>
> This does **not** touch F2.2's boundary result, which is a *within-registry* comparison
> (real vs permuted partitions at fixed registry) and so is unaffected by registry choice.
> The two claims to carry forward are: *the boundary is detectable* (F2.2, robust) and *the
> coefficient's magnitude is not a stable property* (F2.4).

---

## V4 — Full-stack integrity: **3/3 PASS** ✅

### V4.1 — no-`alife` build bit-identity

- **Compile-time half:** `cargo tree -p hg-alife` shows **zero** `hg-engine` occurrences
  anywhere in the transitive tree. The scope-§1.2 discipline holds.
- **Runtime half:** a bounded-ruleset substrate run (`glider` / `GliderRing` /
  `SequentialRandom`, so the run consumes the engine RNG stream every step) fingerprints
  **bit-identically** across two separate compilations:

  | build | nodes | edges | final hash | trace fingerprint |
  |---|---|---|---|---|
  | without `alife` | 30 | 30 | `0x68396e355eb21178` | `0x59d4dcbb7c2be117` |
  | with `alife` | 30 | 30 | `0x68396e355eb21178` | `0x59d4dcbb7c2be117` |

  *Method note:* rather than diffing against a pre-alife historical commit, the feature was
  toggled on the same commit — a stricter test of leakage, since it isolates the feature gate
  from every other change since that commit.

### V4.2 — paired control for the experiment run

On the exact CE configuration (seed 12345, influx 0.34, bootstrap thr 3.0, 6000 steps):

| metric | live | bare control |
|---|---|---|
| full-graph nodes | up to **71** | 36 |
| substrate (nodes, edges) | (36, 85) every window | (36, 85) |
| substrate mean degree | identical | identical |
| substrate `d_eff` | 1.084 | 1.084 |
| substrate regime | `Frozen` | `Frozen` |

Peak 36 carriers on the full graph, and **no window read as substrate `Exploding`**.
Substrate metrics stay attributable; anything beyond them is biology.

### V4.3 — clean-room reproduction

Fresh detached worktree at HEAD, pristine checkout (none of the V-track or LK working-tree
files), cold build, fresh target directory:

| quantity | working tree | clean room |
|---|---|---|
| 6a live CE | 0.3665 | **0.3665** |
| 6a EI_micro / EI_macro | 2.7287 / 1.0000 | identical |
| 6a transitions | 414 559 | **414 559** |
| 6a bare | CE 0.0000, K=1, 215 964 | identical |
| 6b Δ=1 CE | 0.2500 | **0.2500** (561 973 transitions) |

Not merely float epsilon — **transition counts are bit-identical**. The pipeline reproduces
from nothing.

---

## What this means

### The distinction that matters

This is **not** "the biology is fake." V0.5 and V2.2 both passed: the `NonTrivial` 0→>0
crossing is a genuine, structural, depth-2 sustained-self-copying event, and the bare arm is
degenerate by construction. The instrument is sound and the biological transition is real.

What failed is that **CE across the carrier/substrate partition does not detect it.** The
coefficient is dominated by partition arithmetic (V3.3: full-range swing under a
single-class flip) and micro-registry granularity (V3.4), and is statistically
indistinguishable from an arbitrary same-sized cut (V3.2).

This is a *sharper* negative than the branch the plan anticipated. The pre-committed
"CE < 1, reproducible ⟹ macro is not informationally closed over micro at this boundary"
reading assumed the number was *about* the boundary. It is not. The honest claim is
narrower and harder: **on a static substrate with a single carrier WL class, the
carrier/substrate partition carries no boundary-specific macro causal signal.**

### Limitations of this verdict

1. **The null has small support.** With group sizes (9, 1) the permutation null is
   effectively "which single class do you isolate?" across ~10 options. p = 0.495 is
   measured on a coarse discrete space. This limits the control's resolution — it does not
   rescue the headline, but a partition with more than one carrier class would test it
   better.
2. **The temporal control is undefined** on a sustained-turnover run (see V3.2).
3. **One regime, one substrate.** Everything here is the static-substrate turnover slice at
   seed 12345, a regime that itself establishes in only 57–71 % of seeds.

### Fork branch taken

> **Boundary spurious (V3.2 fails) ⟹ S7 computed a clean number across nothing. Report the
> stack up to the blocker; do not run experiments on a boundary that isn't real.**

Experiment 6 proper, the tri-ecology gate, and the discrete-substrate novelty question are
**not** licensed by the current CE result and should not be run on this boundary.

> **[F5-1, cont.] This branch was taken in error and is withdrawn.** The correct branch is
> *the boundary is real; the macro was underspecified*, and the remedy was to enrich the
> macro rather than abandon the experiment. At depth 2 the boundary is distinguishable at
> p < 0.005 against a null of 6.47 × 10⁷ partitions.
>
> Note that the two sections immediately above **already named the fix** — "a partition with
> more than one carrier class would test it better" and "differentiating carriers
> structurally … would widen the permutation null's support." The limitation was identified
> correctly and then not acted on; the error was in reporting a verdict as a finding while a
> known resolution limit sat unresolved beneath it. The lesson is procedural: when a
> control's *support* is small enough to be quoted as a caveat, that is a signal the test
> may be unable to return a positive at all, and support size should be computed **before**
> the p-value is interpreted, not after.
>
> The hold on Experiment 6 proper, the tri-ecology gate, and the discrete-substrate novelty
> question is **lifted with respect to this blocker** — but see `validation_v2_findings.md`
> for what the F2.4 magnitude result does and does not license.

> **[F5-4] Flag the experiment substitution.** Experiment 6 was specified as CE *across the
> replication-onset boundary* — a **temporal** before/after comparison. What was actually
> computed, here and in 6a/6b/6c, is CE of a carrier/substrate **spatial partition during
> sustained turnover**. V3.2 correctly reports the temporal control cannot be constructed
> (longest birth-free stretch 58 steps against 800 needed), but the deeper point is that on a
> **static** substrate the temporal experiment is **degenerate, not merely unconstructible**:
> V4.2 confirms zero back-reaction with (36, 85) unchanged at every window, so the
> pre-boundary micro dynamics are empty and a before/after comparison reduces to "nothing
> versus something." Both framings are blocked by the static substrate. The substitution of
> the spatial question for the temporal one was never made explicit and is recorded here.

> **[F5-5] Own the superseded reasoning.** The **1.47× heterogeneity requirement** and the
> route-A (churn substrate) / route-B (aging) framing built on it were conditioned on the
> sweep regime that had no band. Turnover was in fact unlocked by the asymmetric lever
> **bootstrap threshold 15 → 3**, at a measured satiation spread of only **1.161×** — well
> below the 1.47× that was held to be necessary. The heterogeneity-rescue hypothesis was
> never the operative mechanism. Recorded as a **prediction failure**, not quietly dropped:
> the requirement was derived from a regime that turned out not to be the one that mattered.

### What would move this forward

- **A partition with structure to detect.** The carrier group is a *single* WL class, so the
  macro can only ever be 2×2 with one carrier row. Differentiating carriers structurally
  (multiple carrier motifs) would give the partition something to resolve and would widen
  the permutation null's support.
- **A non-static substrate.** With an inert substrate, surviving nodes never cross the
  boundary (V4.2 confirms zero back-reaction), which is the root of the identity-macro
  problem 6b only partially repairs.
- **A different measure.** The V3.3 dissociation — temporally robust, partition-fragile —
  suggests the question "does replication onset produce macro causal structure" may need an
  instrument that is not partition-relative in this way.
- **Do not** pursue CE > 1 by registry or partition choice. V3.4 shows CE can be moved
  ~+0.07 by closing the registry and V3.3 shows ~±0.37 by reassigning classes; a CE > 1
  obtained that way would be worth less than the reproducible CE < 1 already in hand.
