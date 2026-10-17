# V-track closeout — the coarseness belongs to the CHANNEL. The V-track is complete.

Harnesses: `c1_metabolic_resolution.rs`, `c3_closure_defect_order_gate.rs`,
`c4_arm_c_confirmatory_sweep.rs`, `m4_4_selection_power.rs` (C4.2).
Pre-registrations committed before their numbers: **C1 `f99d423`**, **C4 `4fd9df1`**.

---

## The single sentence the plan asked for

**The metabolic channel's coarseness belongs to the CHANNEL, not to the instrument — so the V-track
is finished, and the next artifact is the write-up.**

The reason is stronger than the plan's "many classes that all score alike" branch anticipated, and it
is mechanical rather than statistical: **a carrier's energy delta takes exactly three raw values.**
There is no finer metabolic partition to build at fixed depth, because the underlying variable is
3-valued.

---

## C1 — the resolution diagnostic: it is the channel

### C1.1 — the partition's structure (the F0.2 table, one level up)

`K_MIN = 4`, **derived** as the plan required rather than chosen: arm A's carrier WL classes at the
frozen structural depth 2 (structural registry M = 200, 4 of them carrier).

Carrier class count at the frozen instrument (`G_cur`, R = 2):

| arm | carrier classes | largest class (% of carrier node-steps) | singleton classes |
|---|---|---|---|
| A replicating | **9** | 48.4% | 0 / 9 |
| B motile static | **7** | 33.1% | 0 / 7 |
| B′ motile diffuse | **6** | 33.3% | 0 / 6 |
| C token | **5** | 47.9% | 0 / 5 |
| D stationary | **5** | 33.4% | 0 / 5 |
| inert (control) | **1** | — | — |

Live-arm minimum is **5**, against `K_MIN = 4`.

### C1.2 — the pre-committed reading selects the confirming branch

Not `~2` classes, so this is **not** the (9,1)-shaped degeneracy recurring. Every live arm resolves
carriers at least as finely as the structural instrument it is tabled against, and the classes carry
realised mass — **no singletons anywhere**, largest class 33–48%. The coarseness is **substantive**.

### C1.3-R — the granularity rule selects a partition *coarser* than the one in use

The rule was committed at `f99d423` before this file existed. Applied:

| G | codes | live-arm k_carrier (A, B, B′, C, D) | passes |
|---|---|---|---|
| `G_sign` | 3 | [9, 7, 6, 5, 5] | **yes** |
| `G_cur` (frozen) | 13 | [9, 7, 6, 5, 5] | yes |
| `G_fine` | 25 | [9, 7, 6, 5, 5] | yes |
| `G_q05` | 49 | [9, 7, 6, 5, 5] | yes |
| `G_q01` | 121 | [9, 7, 6, 5, 5] | yes |

The rule takes the **minimum** passing granularity, so it selects `G_sign` — **coarser** than the
frozen instrument, which also passes. Under-resolution is excluded *a fortiori*: the bar is cleared
by a partition strictly coarser than the one actually used. **C2 does not run**, per the
pre-registration's own clause.

### The root cause, measured (C1.1c / C1.1d — added diagnostics, labelled as such)

These two were **not pre-registered**. They cannot move C1.2's branch, which is decided by
`k_carrier` alone and was already fixed. They are reported because "many classes" is only worth
believing if the classes are populated and if the knob genuinely fails rather than coincidentally
matching on the *count*:

- **Refinement (C1.1c): 0 carrier classes split, at every rung, on every live arm.** Going from 3
  codes to 121 — a 40× refinement — does not subdivide a single carrier class anywhere.
- **Root cause (C1.1d): a carrier's energy delta takes exactly THREE distinct raw values** — `−0.5`,
  `+1.0`, `0.0` — and the *same three* in all five live arms.

So sign-of-change is not a coarse approximation of the carrier metabolic channel; it is the channel's
**exact alphabet**. At depth 2 the maximum resolvable carrier class count is 3² = 9, and **arm A
attains exactly 9** — the instrument is already at its information-theoretic ceiling. The binning
knob has nothing to bin.

Note for the record: the closeout plan described the status quo as "sign-of-change is coarse; binned
deltas less so." That describes **N4.3's retention statistic**, which does use sign-of-change. The
Phase-1/Phase-2 instrument already used binned deltas. Both were put on the ladder, and it turns out
the distinction is empty here — they induce the same carrier partition.

### The one residual, named and not chased

**Depth, not binning, is the only axis with remaining leverage** (arm A: 9 carrier classes at R = 2,
24 at R = 3). `R = 2` is frozen from Phase 1, and both Phase 1's and C1's pre-registrations forbid
re-selecting it after seeing results — that is the F1 forbidden move exactly. Re-opening on depth
would be a **new** pre-registered exercise, not a post-hoc move inside C1. It is flagged here for the
write-up and deliberately **not run**.

---

## C3 — the sorted-order gate, and a second hazard N1.3 did not cover

**C3 found something the plan did not anticipate.** N1.3 fixed the **TPM registry order** by
recording in sorted `NodeId` order. That does not touch the order in which the **entropy sum** runs.
Every entropy in the V-track's estimators was

```rust
-m.values().map(|&c| p * p.log2()).sum::<f64>()   // m: HashMap
```

and `HashMap` iteration order is seeded per **process** by `RandomState`. Float addition is
non-associative, so a run recorded in perfectly sorted order still produced a different closure
defect in a different process.

| estimator | sorted sum (the gate) | raw HashMap-order sum (diagnostic) |
|---|---|---|
| within-channel closure defect | **bit-identical** across 3 processes | differs, max \|Δ\| ≈ 1.3 × 10⁻¹⁵ |
| cross-channel `H(micro\|macro)` | **bit-identical** | differs, ≈ 2.2 × 10⁻¹⁶ |
| cross-channel retention | **bit-identical** | differs, ≈ 2.2 × 10⁻¹⁶ |

- **C3.1** — the gate is asserted on **sorted summation order**, and the shipped estimators
  (`n1_metabolic_instrument.rs`, `n4_3_retention_on_churn.rs`, `m3_metabolic_dynamics.rs`) now sum
  that way, so the gate covers what actually runs. `n4_3`'s joint map additionally re-keyed by
  HashMap *enumeration position*, making its keys themselves per-process; it now keys on the real
  `(macro, micro)` pair.
- **C3.2 — n = 3 is load-bearing, not ceremonial, and this is demonstrated rather than asserted.**
  The harness checks whether any field agrees bit-for-bit across processes 0 and 1 while diverging at
  process 2. It finds them. **An n = 2 check would have passed those fields.** This is §6.1's fifth
  instance caught in the act, and it is why the remedy is repetition.
- **C3.3** — the raw divergence is reported as a diagnostic, so the guarantee reads as the
  conditional it is: reproducible **because** the sum is ordered, not because the statistic is
  order-invariant.

**Scope: nothing is retracted.** ~10⁻¹⁵ cannot move a permutation count comparing CE values separated
by ~10⁻². Verified directly — after the change, every published N-track number reproduces exactly
(arm A CE 0.265964, z 20.16; CD 2.5900; retention 0.14% / 3.86% / 7.12% / 0.00%; the `CD ≡ H(micro) −
H(macro)` identity still `true` in all six arms).

**One thing was genuinely broken and is now fixed.** `n1_metabolic_instrument.rs` still recorded its
TPM in raw HashMap order, so its own *index-dependent reproducibility fingerprint* differed between
runs — the very statistic N1.3 introduced to catch this. It now records sorted, and the fingerprints
are stable run to run.

---

## C4 — supporting items

### C4.1 — arm C's empty-group artifact, rejected under pre-registration

Fresh seed set (201–210, disjoint from 12345 and from Phase 2's 1–30), grid and reading fixed at
`4fd9df1`. **80/80 cells included** (arm A had power at every window and arm C's null passed the
spread guard everywhere), **0 detections**, `f = 0.000`, empty fraction spanning **7.7% → 98.0%**.

**The pre-registered `ρ` clause turned out to be degenerate, and I am not quoting it as evidence.**
Arm C's CE is **bit-identical across the entire window grid** on all 10 seeds, so every pair is a tie
and `ρ = 0.000` is mechanical. The reason is structural: once arm C's carriers die the graph is
constant, so every further step adds only self-loop mass to rows that are already pure self-loops —
the empty steps are **inert for this statistic**.

So the evidence is relocated to where it actually lives: **the smallest window**. At `W = 130` the
carrier group is populated for **92% of scored steps**, arm A **does** detect, arm C's null is
non-degenerate — and **0/10 seeds detect** (z between −1.19 and −0.77). That is precisely the cell
where the empty-group hypothesis predicts arm C *should* detect. **M2's pre-registered truncation
could not reach this cell** — at `W = 120` arm C's own null was degenerate — so the question that was
unanswerable in M2 is now answered, on confirmatory provenance.

### C4.2 — the last guard-audit gap is closed

Gap 1 was already closed by N4.2. **Gap 2 is closed here**: M4.4's null-validity check (both inert
traits ~50/50 in sign, else no selection verdict is read) now has a firing test on constructed input
— it **rejects** an 80%-down skew and a 12%-down skew, sits exactly at the pre-registered
[0.40, 0.60] interval from both sides, and still **accepts** the real run's 43%/53%. **Every V-track
guard now has a test in which it fires.**

---

## What this closes, and what it does not

**Closed.** The instrument question is settled in both directions: the metabolic channel is
genuinely uniform across live carriers, and the reason is that the channel's alphabet is three
symbols. Combined with selection settled degenerate-only at power (M4.4), the substrate is
characterised: **metabolism without evolution, structural activity without metabolic
differentiation.** Per plan rule 2 — a confirming diagnostic closes the track — **do not open a V10.**

**Not closed, and not this track's to close.** These are design questions, to be taken up on their
own terms:

- **The tri-ecology gate.** Its premise is *contradicted*, not merely unmet: it presupposes selection
  this substrate has been shown at power not to exhibit.
- **Loom / the endogenous-rule direction.** Independent of these results.
- **The closure-defect construct.** Arm D remains a maximal closure defect — 100% of the causal work
  in a channel the macro description cannot see — arrived at as a byproduct of a measurement failure.
  The retention statistic is reusable well beyond this substrate; it belongs in the LK line.

## Standing caveats carried forward

- Depth remains an unexplored resolution axis (above). Named, frozen, not chased.
- N0 v2's `J_ref`/`D_ref` derive from a single reference run (arm A, seed 12345).
- The Phase-2 expectation table was not blind; per-arm multi-seed was the predictive content.
- Magnitude remains out of scope (F2.4). Everything here is detectability.
- C1.1c/C1.1d are added diagnostics, not pre-registered; they corroborate C1.2's branch and cannot
  select it.
