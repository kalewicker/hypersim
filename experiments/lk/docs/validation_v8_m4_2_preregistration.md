# V8 / M4.2 — multi-seed arm B′: pre-registration

**Committed before any multi-seed arm-B′ CE exists.** Mirrors the F8 (`43314c2`) and K2 (`38944b1`)
pre-registrations; deviations are stated and are forced by the arm, never by a preferred outcome.

## Why

Arm B′ (non-replicating, motile, on the `diffuse` substrate) detects at **p = 0.010** — 2 of 200
nulls beat it — on **6 structural events** in 6000 steps, with only **59 distinct** realised null
values. It is currently the **sole support for "even a little structural activity suffices"**, which
is a load-bearing clause of the sustained-structural-activity verdict. That is thin, and it is the
cheapest available hardening.

## Fixed design (identical to the L-track arm B′ except the seed)

- Config: `PureCrystal` 6×6 arity 3, influx 0.34, bootstrap threshold 3.0,
  `initial_substrate_energy` 5.0, `MaxParallel`, 6000 steps.
- Rules: `alife_base` + overrides, **minus `replicate`**, **plus `relocate`**, **plus `diffuse`** —
  byte-identical to `l1_presence_vs_motility.rs`.
- **N = 6 planted markers**, fixed (inherited; re-selecting is the forbidden move).
- WL depth **2**; 6b vacuum-augmented streaming TPM; 200 size-preserving permutations shuffled in
  `TpmAccumulator::canonical_order()`. No pipeline change.
- Fidelity anchor: seed 12345 must reproduce the L-track arm-B′ numbers (k_carrier = 2, M = 40,
  p = 0.010) or the run is void.

## Seed set

Seeds **1, 2, 3, …** ascending until **30 are INCLUDED** or **120 attempts** are exhausted. Seed
12345 is outside 1..120, so the set is disjoint from the published result.

## Inclusion criterion

A seed is **included** iff:

1. `transitions >= 1000`;
2. `carrier_classes >= 2`;
3. **established-sustained** — at least one carrier alive at step 6000;
4. **the M2.3 null-spread guard passes** — `null_sd > 0`, `null_max − null_min > 0`, and ≥ 10
   distinct realised null values.

Clause 4 is new to this pre-registration and is forced by the M2.3 finding: arms D and (on truncated
windows) C produced degenerate nulls that the old support bar accepted. A seed whose null is
degenerate is **unmeasurable**, not a failure to detect, and is excluded from the denominator rather
than scored as a negative. **The count of such seeds is reported separately** — if many B′ seeds are
degenerate, that is itself a finding about how thin this arm's measurability is.

## Decision rule (pre-committed)

`f` = fraction of included seeds with `p < 0.05`.

- `f >= 0.90` ⟹ B′'s detectability is a **regime property**; "even a little structural activity
  suffices" may be stated flat.
- `0.50 <= f < 0.90` ⟹ majority; partial generalisation, excepted seeds reported individually.
- `f < 0.50` ⟹ the published p = 0.010 was **seed-specific**; the clause it supports must be
  withdrawn and the verdict re-qualified.

Secondary: exact two-sided binomial sign test on `real CE > own null mean`. Reversals flagged
individually. `n < 10` included seeds ⟹ **vacuous**, no verdict read.

Also reported, since it is the substance of the claim: **the realised structural-event count
(relocations) per seed**, so "a little activity" is quantified rather than assumed.

## Forbidden

- Dropping or re-ordering seeds to move `f`.
- Changing N, the WL depth, or the inclusion criterion after seeing per-seed results.
- Counting a degenerate-null seed as a non-detection.
