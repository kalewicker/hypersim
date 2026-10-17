# V9 / N0 v2 — the burn-in criterion, recalibrated against the trusted baseline

**Supersedes `validation_v9_n0_burnin_criterion.md` (d853f94), which FAILED.** This file is
committed before any burn-in length is re-derived under it.

## What went wrong with v1, stated plainly

v1 committed **absolute** thresholds: over 100-step blocks, settled iff consecutive-block WL
class-set Jaccard `>= 0.90` and `|ΔM|/M <= 0.05`.

Run against a calibration case v1 did not include — **the F8 static run, the baseline every published
structural result rests on** — v1 **rejects it**: F8's consecutive-block Jaccard at `L_blk = 100` runs
0.55–0.76 and never reaches 0.90.

A criterion that rejects the run the whole report already relies on is not measuring stationarity; it
is measuring block size. At `L_blk = 100` the metric resolves rare-class fluctuation that the
M-track's 1000-step blocks averaged over. **So v1's rejection of churn-with-carriers is NOT evidence
that churn is non-stationary** — it is evidence that v1's bar is unusable.

Per v1's own forbidden list I may not retune its thresholds in place. v1 is therefore recorded as a
**failed criterion**, and this is its replacement.

**The underlying mistake, for the record:** I set absolute thresholds where this program's
established practice is **baseline-relative**. H0.1 says it explicitly — *"whatever the F8 static run
exhibits defines the tolerated floor — CE is known to be meaningful there."* v1 departed from that
precedent without reason. v2 returns to it.

## v2 — THE CRITERION (baseline-relative, following H0.1's precedent)

Block length `L_blk = 100` is retained (it is the resolution at which the question is being asked;
the defect was the thresholds, not the block size).

On the **F8 static reference run** (arm A, seed 12345, 6000 steps), compute over consecutive blocks:

```
J_ref = min_i  Jaccard(S_i, S_{i+1})            (the reference run's WORST block pair)
D_ref = max_i  | |S_{i+1}| - |S_i| | / max(|S_i|,1)
```

A run is **SETTLED from block `b`** iff for every `i >= b`:

1. `J(i) >= J_ref`, **and**
2. `|ΔM|/M (i) <= D_ref`.

`T_burn = b · L_blk` for the smallest such `b`, and at least `50%` of the run must remain, or the run
is reported as too short to score.

**Reading:** "settled" means **no less stationary than the worst stretch of the run whose CE the
report already trusts.** F8 itself settles at `b = 0` by construction, which is the point — the bar is
calibrated, not vacuous, and it cannot reject the baseline.

`J_ref` and `D_ref` are **derived from the reference run and reported**, not chosen. They are computed
once, from arm A, and then applied unchanged to every other run.

## Why this is not threshold-shopping

The numbers are no longer free parameters: they are **outputs of the reference run**. I do not get to
pick them, and I could not have tuned them toward a churn verdict without changing the baseline
itself. The only judgement retained is the choice of reference run, and that choice is forced — F8 is
the run the program's published results already depend on.

## Firing test (rule 5) — unchanged in force

The criterion must still **reject**:

- `growth_treadmill` (Jaccard ≈ 0.015–0.07, far below any plausible `J_ref`) — asserted;
- `triangle_subdivide` (Jaccard ≈ 0.004, `M` unbounded) — asserted;
- a **pre-settling** churn window scored from step 0 (block 0→1 Jaccard ≈ 0.13) — asserted.

A criterion that accepts everything is not a criterion; v1's failure was the opposite error, and v2
is tested against both.

## What is reported

- `J_ref`, `D_ref` as derived from arm A.
- `T_burn` for the bare churn substrate and churn-with-carriers.
- Agreement or disagreement with the M-track's post-hoc 1000-step choice, **reported either way**.
- The v1 failure above, which stands in the record rather than being quietly replaced.

## Forbidden

- Changing `L_blk`, the reference run, or the min/max definitions of `J_ref`/`D_ref` after seeing any
  derived `T_burn`.
- Presenting v1's churn rejection as a finding about churn.
- Scoring any run from an offset the criterion did not derive.
