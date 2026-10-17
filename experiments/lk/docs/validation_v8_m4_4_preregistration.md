# V8 / M4.4 — powering the selection question: pre-registration

**Committed before any 30-seed selection number exists.** Extends the criterion-5 validation
(`alife_criterion5_selection_vs_drift.rs`), which ran 10 seeds and found the repl-threshold shift is
inside the drift envelope (~1.2×), leaving criterion 5 "met degenerate-only". M4.4 powers that test
to ≥30 **established** seeds so the drift envelope and any selection signal are estimated with
enough seeds to be trusted, not asserted.

## The biology fact this rests on (unchanged)

Only 2 of the 4 genome traits are **read** by any rule: `replication_threshold` (key 11, by
`replicate`) and `satiation` (key 13, by `harvest`). `harvest_rate` (10) and `maintenance_cost` (12)
are set/copied/jittered but read by nothing, so their cross-seed spread is a **matched pure-drift
null** — the trait-mean random walk over near-clonal deep lineages at jitter 0.5, with no selection
possible.

## Design (fixed)

- Regime: turnover, `PureCrystal` 6×6, influx 0.34, bootstrap threshold 3.0,
  `initial_substrate_energy` 5.0, `MaxParallel`. **15 000 steps** (the scorecard length, longer than
  the validation's 6000, so slow selection has more room to separate from drift).
- Per seed: displacement = (last populated window mean − first populated window mean) of each trait
  mean, exactly as the validation defines it.
- Seed set: **1, 2, 3, …** ascending until **30 are ESTABLISHED** (≥ 2 populated windows so a
  displacement is defined) or **200 attempts** are exhausted. The **establishment rate** is reported
  (F6.2), because this regime is known seed-fragile.

## The drift null and the two read traits

- **Drift envelope** = the pooled spread (RMS displacement) of the two **inert** traits
  (`harvest_rate`, `maintenance_cost`) across established seeds. This is the matched null: same
  lineage structure, same jitter, zero selection.
- **`replication_threshold`** and **`satiation`** are each compared to that envelope by:
  1. a **magnitude ratio** `RMS(trait) / drift_envelope`, and
  2. a **sign test** — the fraction of established seeds displacing in the trait's modal direction,
     against the inert traits' own sign balance (which must be ≈ 50/50 for the null to be valid).

## Pre-committed reading (M4.4 targets `satiation` per the plan)

For **`satiation`** (the trade-off trait, the plan's named target), and reported the same way for
`replication_threshold`:

- **SELECTION** — magnitude ratio ≥ 2.0 **and** sign fraction ≥ 0.80 (binomial p < 0.05 at n ≥ 30)
  ⟹ a reproducible directional shift beyond drift: non-degenerate selection on that trait, and
  criterion 5 is met on stronger footing than "degenerate-only".
- **DRIFT-DOMINATED** — magnitude ratio < 2.0 **or** sign fraction < 0.65 ⟹ the shift is inside the
  drift envelope or not reproducibly directional: criterion 5 stays **met degenerate-only**, now on
  30 seeds rather than 10, and that standing is confirmed not overturned.
- **INTERMEDIATE** — otherwise ⟹ reported as bounded by the numbers, not forced to a branch.

**Null-validity check (required):** if the two inert traits are themselves sign-biased beyond
40–60%, the drift null is not clean and no selection verdict is read — the discrepancy is reported
instead.

## Forbidden

- Changing the thresholds, the trait targeted, or the seed rule after seeing displacements.
- Reading a selection verdict if the inert-trait null is itself sign-biased.
- Treating `harvest_rate` or `maintenance_cost` as anything but the null (they are read by no rule).
