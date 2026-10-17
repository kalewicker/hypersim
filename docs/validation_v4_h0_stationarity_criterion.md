# V4 / H-track — H0.1: the CE-stationarity criterion (committed BEFORE measurement)

Plan: `~/Downloads/validation_v4_churn_substrate_plan.md`. H0 is a **hard gate**: it decides,
on paper + cheap measurement over *existing* code, whether CE over a fixed carrier/substrate
partition stays well-defined once the substrate itself moves. This document is the H0.1
deliverable — the written stationarity criterion and its decision rule — committed **before any
stationarity number is measured** (H0.2/H0.3), so the gate cannot be moved to fit the answer.

Nothing here requires the churn substrate to exist. Per the forbidden-response list, the churn
substrate is **not** built until this gate clears.

---

## What CE-over-a-fixed-partition assumes

CE = EI_macro / EI_micro is computed from a transition-probability matrix (TPM) accumulated over
a window of steps. The micro state space is the set `M` of Weisfeiler–Leman (WL) classes seen in
the window; each class contributes one TPM row (its outgoing transition distribution); the
carrier/substrate partition assigns each class to one of two groups; the macro TPM coarse-grains
the rows by that assignment. For this to be a meaningful quantity over a **fixed** partition, the
window must satisfy:

1. **Fixed, closed state space.** `M` must be an effectively-closed finite set across the window:
   the classes the TPM is built over must be *the same classes*, not an ever-growing set of
   never-recurring ones. If new classes are minted every step and never seen again, the TPM is a
   pile of singleton rows over an ill-defined, expanding state space, and `EI_micro`/`EI_macro`
   are computed over a moving target.
2. **Stable assignment.** Each class's carrier-vs-substrate label must not change within the
   window. **This is automatic here and is not the failure mode:** the WL base hash folds in the
   node's `Label` (`hash.rs:44,150`), so carrier (label 100) and substrate (label 0) classes are
   disjoint by construction — no WL class ever flips carrier↔substrate. The static F8 pipeline
   relies on exactly this (`is_carrier_class` is an intrinsic, permanent property of a hash).
3. **Well-sampled rows.** Each class must recur enough within the window to accumulate a
   transition *distribution*, not a one-shot transient. A TPM whose mass is dominated by classes
   visited once or twice is noise dressed as dynamics; the permutation null then shuffles labels
   over ephemeral classes — the **F0 artifact in a new guise** (support inflated by junk classes).

Because assignment stability (2) is free, **the whole gate reduces to state-space closure (1) +
row sampling (3): does the WL class set stay closed and well-sampled across the window once the
substrate moves?**

---

## The measurable criterion

The metrics below are chosen to be **node-count-independent where it matters**, because the only
substrate-motion sources available at H0.3 (`growth_treadmill`, `triangle_subdivide`) are
*unbounded* — they grow the node count — and a metric that merely counts classes would confound
"more nodes ⟹ more classes" with the thing under test, "motion ⟹ ephemeral classes". A genuinely
*bounded* churn substrate (H1) would hold `n(t)` flat while identity turns over, so the decisive
question is whether **class identity** churns, not whether class *count* grows. The primary
metric is therefore a fraction of transitions, not a count.

**PRIMARY — transition-weighted transient fraction (TWTF).** A WL class is *persistent* if it is
live (has ≥1 node) in ≥ `R` distinct steps of the window, *transient* otherwise. TWTF = the
fraction of all recorded TPM transitions whose **source class is transient**. Low TWTF ⟹ the TPM
mass sits on recurrent, well-sampled classes ⟹ the partition is stationary enough. High TWTF ⟹
the TPM is dominated by ephemeral classes ⟹ CE is over an ill-defined space. TWTF is a ratio, so
it is not inflated merely by node growth. Fix `R = 3` (a class must be live in ≥3 steps to count
as a real recurrent state; committed here, not chosen post hoc).

**SECONDARY, corroborating:**
- **State-space closure `M(t)`.** Cumulative distinct-class count vs step. Under a *closed* space
  the new-class rate `dM/dt` decays and `M(t)` plateaus; under non-closure `M(t)` grows ~linearly
  (constant `dM/dt`). Reported as: new classes minted in the last window-quartile as a fraction of
  total `M`, and whether `dM/dt` is decreasing.
- **Live-class step-to-step persistence.** Of the classes live at step `t`, the fraction also live
  at `t−1` (a Jaccard-style overlap). Node-count-independent. Static substrate ⟹ near-1 (fixed
  sites revisit configs); fast identity churn ⟹ drops.
- **Sampling ratio `T/M`.** Transitions per class. Must stay `≫ 1` (and `≫` the ~1 that makes rows
  singletons). A closed, well-sampled TPM keeps `T/M` large and *rising* with window length; a
  non-closing one has `T/M` flat or falling as the window grows (M grows as fast as T).

Dependence on window length `W` and transition count `T`, as the plan requires: lengthening `W`
raises `T` (better sampling) **only if `M` is closed**. If `M` grows with `W`, then `T/M` does not
improve and TWTF does not fall — a longer window cannot rescue a non-closing space. This is exactly
what H0.4's windowing rescue would exploit *in reverse*: a **shorter** window can keep a slowly-
churning partition locally stationary, at the cost of `T` (hence null support).

---

## Pre-committed decision rule (relative to the trusted F8 static baseline)

The F8 static-substrate run is a **validated** result the report relies on; whatever
(TWTF, closure, persistence, T/M) it exhibits defines the **tolerated floor** — CE is known to be
meaningful there. H0.2 measures that floor; H0.3 measures the same metrics under substrate motion.
Committed rule:

- **PROCEED to H1** iff motion keeps every metric in the same regime as F8: TWTF stays low and
  `≤ ~2×` the F8 baseline (and `< 0.5` absolute); `M(t)` still plateaus; `T/M` stays `≫ 10`; live-
  class persistence stays high. I.e. substrate motion does **not** categorically change how
  ephemeral the partition is.
- **STOP → H4** iff motion breaks closure or sampling: TWTF becomes the **majority** (`> 0.5`) and
  `≫` F8, **or** `M(t)` grows ~linearly with no `dM/dt` decay, **or** `T/M` collapses toward `O(1)`
  / `< 10`. Any one of these means a moving substrate turns the TPM into transient noise and CE
  over a fixed partition is not well-defined — the churn rung would break the instrument.
- **BORDERLINE → H0.4** otherwise (motion worsens the metrics past `~2×` F8 but not into the STOP
  regime): test whether a shorter, pre-registered scoring window restores stationarity while
  keeping null support `≫ 10`.

I commit, before measuring, that **linear `M(t)` growth, or a transition-weighted transient
majority, or `T/M → O(1)`, under substrate motion ⟹ STOP and hand off to H4** — I am not free to
reinterpret those as "proceed" after seeing them.

---

## The single sentence H0 must answer

**Whether CE over a fixed carrier/substrate partition remains well-defined once the substrate
moves — and therefore whether the churn rung is an experiment (H1–H3) or a dead end (H4).**
