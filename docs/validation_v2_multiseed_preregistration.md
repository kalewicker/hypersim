# Multi-seed F2 replication — pre-registration

**Committed before any multi-seed CE was computed.** The F-track established that the
carrier/substrate boundary is distinguishable from an arbitrary same-sized partition **at
seed 12345** (WL depth 2, 6b vacuum-augmented model, 0/200 permutations, p < 0.005, null
support 6.47 × 10⁷ — `docs/validation_v2_findings.md` §F2.2). F6.2 recorded the largest
outstanding caveat: every CE number in the project is single-seed on a regime that establishes
only ~32 % of the time. This measurement converts "detectable at seed 12345" into a statement
about the **regime**, or shows that it cannot be.

Harness (written after this file is committed):
`crates/hg-sim-core/tests/f7_multiseed_boundary.rs`.

Nothing here changes a mechanic, an economy value, or a rule. The WL depth is **2**, fixed and
inherited from the F1 pre-registration (`92b3ef8`); re-selecting it is the forbidden move.

---

## Fixed design (identical to F2 except the seed)

- Config: `PureCrystal` 6×6 arity 3, influx 0.34, bootstrap threshold 3.0,
  `initial_substrate_energy` 5.0, `MaxParallel`, **6000 steps**, cadence 1.
- Model: **6b vacuum-augmented** (births ∅→c, deaths c→∅). This is the only model whose
  `EI_macro` is not structurally pinned; 6a reduces to `CE = 1/EI_micro` and cannot falsify
  anything, so it is not tested.
- Micro partition: WL classes at **depth 2**.
- Per-seed null: **200** group-size-preserving label permutations, shuffled in
  `TpmAccumulator::canonical_order()` (F3) so each seed's null is itself process-stable.

## Seed set (pre-committed, deterministic)

Seeds **1, 2, 3, …** in ascending order, until **30 seeds are INCLUDED** or **120 attempts**
are exhausted, whichever comes first. The exact attempted range and the included/excluded
disposition of every seed are reported. **Seed 12345 is deliberately NOT special-cased** — it
is outside 1..120, so this set is disjoint from the F2 result, and the F2 finding is treated as
one additional prior data point rather than folded in.

## Inclusion criterion (the test being well-defined)

A seed is **included** iff, on its vacuum-augmented TPM at depth 2:

1. `transitions ≥ 1000` (the standing CE gate), **and**
2. `carrier_classes ≥ 2` — the partition is non-degenerate, so the permutation null has
   support `C(M, k) > 1` and the seed's test *can* return a positive. A seed failing (2) is
   the depth-1 pathology at the seed level and is excluded from the boundary test, not scored
   as a pass or a fail.

Separately, and for comparability with F4's 32 %, the **surviving rate** (fraction of attempts
with ≥ 1 live carrier at the final observation) is reported. Inclusion and survival are
reported as distinct numbers.

## Per-seed statistics reported

Real CE, `EI_micro`, `EI_macro`, carrier classes `k`, total classes `M`, transitions, null
mean, null max, permutation p (with the rule-of-three upper bound when the count is 0), and the
null support `C(M, k)`.

---

## Aggregate decision rule (pre-committed — this is the whole point)

Let `S` = the set of included seeds, `|S| = n`. For each seed, `p_i` is its permutation
p-value and `Δ_i = real_CE_i − null_mean_i` its margin.

**Primary — fraction detectable.** `f = |{i ∈ S : p_i < 0.05}| / n`.

- **f ≥ 0.90** ⟹ the boundary is **robustly detectable across the regime**; the F2 result
  generalises and seed 12345 was representative.
- **0.50 ≤ f < 0.90** ⟹ detectable in the **majority** of established seeds; F2 partially
  generalises. Report the excepted seeds and what distinguishes them (carrier-class count,
  transition count) rather than averaging them away.
- **f < 0.50** ⟹ seed 12345 was **not representative**; the F2 result does not generalise, and
  the boundary's detectability is itself seed-dependent — a finding, not a failure.

**Secondary — cross-seed sign test.** `k⁺ = |{i ∈ S : Δ_i > 0}|`. Under a null of no
boundary signal, `Δ_i > 0` occurs with probability ≈ ½ per seed, so an exact two-sided
binomial test on `k⁺/n` tests whether the boundary is directionally consistent across seeds
independent of any single seed's p-value. Reported alongside the primary.

**Reversals.** Any seed with `Δ_i < 0` (real CE below its own null mean) is flagged
individually — that is the signature of a seed where an arbitrary partition beats the real
boundary, and it must not be hidden inside an average.

## Forbidden (carried from the fix plan)

- Dropping, re-ordering, or re-weighting seeds to move `f`.
- Re-selecting WL depth, per seed or globally.
- Changing the inclusion criterion after seeing the per-seed results.
- Reporting the median CE as a magnitude — F2.4 already showed CE magnitude is
  registry-dominated; this measurement is about **detectability**, not magnitude.

## What this does and does not settle

It settles whether the *detectability* demonstrated at seed 12345 holds across the establishing
regime. It does **not** revisit magnitude (F2.4), the vacuum-model construction caveat (F2.2),
or the 32 % establishment fragility (F6.2) — those stand regardless of the outcome here.
