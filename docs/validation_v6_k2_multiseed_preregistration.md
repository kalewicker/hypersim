# V6 / K2 — multi-seed arm B: pre-registration

**Committed before any multi-seed arm-B CE exists.** Mirrors the F8 pre-registration
(`docs/validation_v2_multiseed_preregistration.md`, commit `43314c2`) clause for clause; every
deviation is stated explicitly below and is forced by arm B being a different arm, never by a
preference about the outcome.

## Why

Arm B — a non-replicating carrier population sustained on the static substrate by `relocate` — is
the only novel positive in the H2 ABC experiment (A was already known from F8, C from the G-track),
and it is **single-seed (12345)**. That is exactly the epistemic status F2 held before F8. K2 gives
arm B the same hardening.

K1 (commit `815d2fd`) already established that arm B sustains by **persistence**, not turnover
(6 individuals, 0 deaths in 6000 steps, 248 realized relocations), so what K2 hardens is the
**sustained-presence** verdict, not a turnover claim.

## Fixed design (identical to H2.2 arm B except the seed)

- Config: `PureCrystal` 6×6 arity 3, influx 0.34, bootstrap threshold 3.0,
  `initial_substrate_energy` 5.0, `MaxParallel`, **6000 steps**.
- Rules: `alife_base` + overrides, **minus `replicate`**, **plus `relocate`** — byte-identical
  to `h2_step3_split.rs`.
- **N = 6 planted markers**, fixed. Inherited from H2.2; the density sweep (N = 1, 6, 12, 24 all
  sustain; N = 36 saturates and dies) is already reported and re-selecting N here is the
  forbidden move.
- Model: 6b vacuum-augmented streaming TPM. Micro partition: **WL depth 2** (F1 pre-registration;
  re-selecting is forbidden). The energy-band partition is reported alongside as a secondary
  column but the **primary decision rule reads the WL column**, because that is the partition the
  H2 verdict rests on and the one K3 will decompose.
- Per-seed null: **200** size-preserving label permutations shuffled in
  `TpmAccumulator::canonical_order()`. No pipeline change of any kind.

## Seed set (pre-committed, deterministic)

Seeds **1, 2, 3, …** ascending until **30 are INCLUDED** or **120 attempts** are exhausted,
whichever comes first. Seed 12345 is outside 1..120, so this set is **disjoint** from the H2.2
result, which is treated as one additional prior data point and is not folded into `f`.

A **fidelity anchor** runs seed 12345 through this harness first and must reproduce H2.2's arm-B
numbers (k_carrier = 4, null ≥ real 0/200); if it does not, every per-seed number below is
untrustworthy and the run is void.

## Inclusion criterion

A seed is **included** iff all three hold:

1. `transitions >= 1000` — the standing CE gate;
2. `carrier_classes >= 2` — the partition is non-degenerate, so the null has support
   `C(M, k) > 1` and the seed's test *can* return a positive;
3. **established-sustained**: at least one carrier alive at step 6000. This is the arm-B analogue
   of F8's survival gate and is the operational meaning of "a sustained non-replicating
   population". A seed whose bloom dies early is arm **C**, not arm B, and is excluded from arm
   B's test rather than scored as a failure of it.

Criterion 3 is the only addition to F8's inclusion rule, and it is forced by arm B's definition.
The **establishment rate** (included / attempted, and sustained / attempted separately) is
reported as F6.2 requires, so the fragility of the regime is visible next to the result.

## Aggregate decision rule (pre-committed)

Let `S` be the included seeds, `n = |S|`, `p_i` the permutation p-value, `Δ_i = real_CE_i −
null_mean_i`.

**Primary — fraction detectable on the WL partition.** `f = |{i : p_i < 0.05}| / n`.

- `f >= 0.90` ⟹ arm B's detectability is a **regime property**; the sustained-presence verdict may
  be stated flat rather than as a seed-12345 draw.
- `0.50 <= f < 0.90` ⟹ detectable in the **majority** of established seeds; partial
  generalisation. Report the excepted seeds and what distinguishes them (k_carrier, transitions),
  not an average over them.
- `f < 0.50` ⟹ the ABC result was **seed-specific**; the verdict reverts to single-seed and the
  inversion of the G-track reading is not established at regime level.

**Secondary — cross-seed sign test.** Exact two-sided binomial on `k⁺ = |{i : Δ_i > 0}|` against
p = ½, reported alongside the primary.

**Reversals.** Every seed with `Δ_i < 0` is flagged individually and never hidden in an average.

`n < 10` included seeds ⟹ the measurement is **vacuous** and is reported as such; no verdict is
read from it.

## Forbidden

- Dropping, re-ordering, or re-weighting seeds to move `f`.
- Changing N, the WL depth, or the inclusion criterion after seeing per-seed results.
- Selecting the seed set or the inclusion criterion by their effect on the verdict.
- Reporting median CE as a magnitude (F2.4: magnitude is registry-dominated).
