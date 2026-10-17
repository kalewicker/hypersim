# V9 / N0 — the burn-in criterion (COMMITTED BEFORE ANY CHURN MEASUREMENT IN THIS TRACK)

**Gate:** no churn measurement anywhere in the N-track is scored before this file is committed.

## Why this needs a criterion, not a number

The M1 gate initially fired **FALSE** because a pre-settling churn window was compared against
asymptotic growing proxies, and the burn-in length was then chosen *after* seeing that. The
requirement is principled — a pre-settling window measures startup, not steady state — but the
**length was post-hoc**. Structurally this is the same hazard as WL depth in the F-track: a free
parameter with demonstrated gate-flipping leverage, chosen after seeing its effect.

The fix is to commit a **rule that derives the length from the data**, so the length is an output,
not a choice.

## N0.1 — THE CRITERION (committed)

Let the run be divided into consecutive equal blocks of length `L_blk = 100` steps. Let `S_i` be the
**set of WL classes observed anywhere in block `i`**, and let

```
J(i) = |S_i ∩ S_{i+1}| / |S_i ∪ S_{i+1}|          (consecutive-block class-set Jaccard)
```

A run is **SETTLED from block `b`** iff **both** hold for every `i >= b`:

1. **Class-set stabilisation:** `J(i) >= 0.90`.
2. **`M` plateau:** `| |S_{i+1}| − |S_i| | / max(|S_i|, 1) <= 0.05` — the class count changes by no
   more than 5% between consecutive blocks.

The **burn-in length** is `T_burn = b · L_blk`, where `b` is the **smallest** block index satisfying
the above for all subsequent blocks. Scoring uses steps `T_burn + 1 … T_end` only.

Both clauses are required, and they have different failure modes on purpose (rule 4): Jaccard
catches *class replacement at constant count* (the growing-proxy signature — `growth_treadmill` holds
`M` at 92 per block while replacing its members, so clause 2 alone would pass it); the `M` plateau
catches *monotone accumulation* that a high Jaccard could mask when new classes are added without
old ones leaving.

**A run that never satisfies the criterion has NO valid scoring window** and is reported as
unmeasurable rather than scored from an arbitrary offset. `T_burn` must also leave at least 50% of
the run, or the window is reported as too short to score.

Thresholds (`0.90`, `5%`, `L_blk = 100`, 50% minimum retained) are fixed here, before any N-track
churn number exists. `L_blk = 100` is chosen to be well below the M-track's observed settling scale
(~1 block of 1000 at `N_BLOCKS = 6` over 6000 steps) so the criterion can resolve a shorter burn-in
than the M-track used, rather than being pre-committed to agree with it.

## N0.2 — What the criterion implies, and agreement with the M-track

Reported by the harness: the derived `T_burn` for the bare churn substrate and for churn-with-
carriers, beside the M-track's *effective* burn-in (it discarded block 0 of 6, i.e. 1000 steps).
**Any discrepancy is reported, not silently adopted.** If the criterion implies a materially shorter
or longer burn-in than the M-track used, the M-track's post-hoc choice is recorded as having been
approximately right or wrong on the derived rule's terms, and the derived value is the one used from
here on.

## N0.3 — FIRING TEST (rule 5, §6.1)

The criterion ships with cases in which it **rejects**:

- **`growth_treadmill`** — continuously replaces its class set (`J ≈ 0.015–0.07`), so clause 1 fails
  at every block and the run has **no settled window**. This is the primary firing case.
- **`triangle_subdivide`** — `J ≈ 0.004` and `M` grows without bound; both clauses fail.
- **A pre-settling window of the churn substrate itself** — scoring the churn run from step 0 must be
  rejected by the criterion (block 0 → 1 has `J ≈ 0.13` in the M-track measurement), which is exactly
  the mistake that flipped the M1 gate.

The test asserts that at least the growing proxies are rejected; a criterion that accepts everything
is not a criterion.

## Forbidden

- Changing `L_blk`, the Jaccard threshold, the plateau tolerance, or the 50%-retained rule after
  seeing any derived `T_burn`.
- Scoring a run from an offset the criterion did not derive.
- Reporting a run as measurable when the criterion found no settled block.
