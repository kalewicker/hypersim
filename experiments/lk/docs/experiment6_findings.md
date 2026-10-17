# Experiment 6 — findings: causal emergence across the replication-onset boundary

> **SUPERSEDED IN PART by Experiment 6b** (`experiment6b_findings.md`). The CE = 0.37 headline
> below is inflated by the identity-macro artifact described in §1: with birth/death restored,
> `EI_macro` un-pins from 1.0 and CE falls to ≤ 0.25 (cadence-dependent), a clean *negative* on
> causal emergence. The structural 0→>0 bridge (this doc) still holds; the CE *magnitude* does
> not. Read this doc for the bridge; read 6b for the corrected CE.

Reported against `experiment6_preregistration.md` (committed before any CE number was
seen). Harness: `crates/hg-sim-core/tests/experiment6_ce_across_onset.rs`. Partition:
`crates/hg-causal/src/ei.rs::carrier_substrate_partition` (S7.2). Seed 12345 (pinned,
establishing). **Nothing was tuned to raise CE.**

## Headline

| arm | CE | EI_micro | EI_macro | macro_K | micro_M | transitions |
|---|---|---|---|---|---|---|
| **bare** (alife off) | **0.0000** | 2.0000 | 0.0000 | 1 | 4 | 215 964 |
| **live** (turnover regime) | **0.3665** | 2.7287 | 1.0000 | 2 | 9 | 414 559 |

**P1 (the 0 → >0 bridge): CONFIRMED** structurally. Pre-replication the carrier/substrate
partition is degenerate (no carrier classes → K = 1 → CE = 0). With replication on it
becomes a genuine 2-group partition (K = 2) with CE = 0.3665 > 0. This mirrors criterion
6's replicator crossing: the macro partition Experiment 6 coarse-grains over is empty *by
construction* on the substrate and non-empty *by biology*.

**P2 (Finding F1): CONFIRMED.** CE_live = 0.37 ∈ (0, 1) — below the conservative,
downward-biased bar, exactly as predicted. We did not target or reach CE > 1, and CE < 1
falsifies nothing.

**F-B (bare artifact): cleared** — bare CE is exactly 0 (degenerate K = 1). **F-C (gate):
cleared** — 414 k transitions ≫ 1000. **F-D (determinism): cleared** — the TPM
(`transition_count`) and `EI_micro` are **bit-identical** across independent runs; CE
itself is reproducible **to floating-point precision (~15 digits)** but not bit-identical,
because `TpmAccumulator` assigns dense class indices in hashmap-iteration order and EI is
permutation-invariant only *mathematically* — the last ULP of `EI_macro` (≈1.0) jitters with
the summation order. That is FP associativity, not scientific nondeterminism; the reported
CE is stable well beyond any digit that matters.

**P3 (within-run temporal):** CE is stable-positive across the run (0.3524 → 0.3663 as the
TPM accumulates), K = 2 from the first sample on. The sharp 0 → >0 crossing itself is
carried by the paired **bare-vs-live control**, not by the temporal trace (carriers
establish within the first few hundred steps, before the first trace sample).

---

## Two honest caveats that bound what this number means

### 1. The macro dynamics is TRIVIALLY the identity — `EI_macro = 1.0000` is pinned, not earned

`EI_macro = 1.0000` is not a coincidence; it is `log₂(K) = log₂(2) = 1` **exactly**, and it
will be exactly 1 for *any* run where carriers exist. The reason is structural: the micro
TPM records class transitions **only for nodes that survive** from one observation to the
next, and **a surviving node never crosses the carrier/substrate boundary** — a substrate
node never becomes a carrier, and a carrier stays a carrier until it dies. So the 2×2 macro
TPM is exactly the identity `[[1,0],[0,1]]`: each macro state deterministically maps to
itself. An identity TPM has determinism 1, degeneracy 0, hence `EI_macro = log₂(K)`.

Consequently **CE_live = EI_macro / EI_micro = 1 / EI_micro** — the macro numerator carries
no information about carrier *dynamics*; CE just tracks the inverse of the micro entropy.
The real replicator dynamics — birth, death, turnover — happens through node
**creation/deletion**, which the surviving-node transition model does not represent. The
0 → >0 crossing (K: 1 → 2) is a real structural fact; the CE *magnitude* under this
micro-dynamics should **not** be read as "how much causal emergence the replicators
produce."

**This is a measurement finding, not a bug**, and it is the honest result of Experiment 6
as specified: the carrier/substrate partition over the surviving-node WL-class TPM produces
a degenerate (identity) macro process. A CE that actually reflects replicator dynamics would
need a micro dynamics that includes birth/death events (e.g. transitions over a state space
that counts carriers, or an occupancy-field dynamics) — a **new pre-registration**, not a
tweak to this one.

### 2. Cadence had to be per-step, or carriers are invisible

The substrate EI cadence is the metrics window (128 steps). At that cadence the measurement
is **blind to carriers**: in the turnover regime a carrier lives ~tens of steps, so it
almost never survives two consecutive 128-step windows and never registers a transition —
the micro TPM degenerates to the 36 static substrate self-loops and the partition collapses
to K = 1 even in the live arm (observed directly: identical 5580-transition counts,
macro_K = 1 for both arms). **Cadence = 1 step** resolves carrier lifetimes so carrier
classes enter the micro dynamics. The cadence is part of the measurement's definition and is
carried in every `EiResult`; this is a timescale choice forced by the biology, not a tuning
knob.

---

## What actually unblocked this (carried from S6 validation)

- Turnover was achieved on a **static substrate** (asymmetric `replication_threshold`
  lever), so the carrier/substrate partition is over the **same micro state space** — the
  condition Experiment 6 requires. Confirmed clean by the paired substrate control (no
  back-reaction at any influx).
- **Criterion 5 is degenerate-only** (selection is the free node-funded collapse,
  drift-dominated). This does not affect the CE *setup* (which needs the partition + the
  onset boundary, both clean), but it bounds the biology: this is a stable, replicating,
  turning-over population **without** non-degenerate selection.
- The band is **seed-fragile**; seed 12345 establishes and is pinned. Establishment is not
  averaged over seeds that never entered the regime.

---

## Verdict

Experiment 6 is **runnable and run**. The pre-registered bridge (CE 0 → >0 across the
replication-onset boundary under the carrier/substrate partition) **holds structurally**,
deterministically, and with the books balanced — the S6 → S7 gate is genuinely cleared.

But the CE *value* (0.37) is bounded by caveat 1: under the surviving-node micro dynamics
the macro process is the identity, so CE only measures `1/EI_micro` and does not quantify
replicator causal power. **The honest Experiment-6 result is: replication onset makes the
carrier/substrate coarse-graining non-degenerate (the taxonomy validation the plan asked
for), but a CE that captures replicator dynamics requires a birth/death-aware micro dynamics
— the natural next pre-registration.** Per Finding F1 we did not, and must not, tune toward
CE > 1 to manufacture a stronger-looking result.
