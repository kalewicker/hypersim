# Experiment 5b — findings: activating harvest_rate & maintenance_cost

Reported against `experiment5b_active_traits_preregistration.md` (committed before running).
Harness: `crates/hg-sim-core/tests/experiment5b_active_traits.rs`. Implements plan deviation #4
via the new `add_from_property`/`subtract_from_property` effect ops, config-gated behind
`genome_scaled_metabolism` (off by default). **Nothing tuned.**

## Result (turnover regime, 10 000 steps, 6 seeds)

| seed | Δharvest_rate | Δmaintenance | births | deaths | deaths 1st→2nd half | turnover |
|---|---|---|---|---|---|---|
| 12345 | +0.030 | −0.258 | 422 | 387 | 232 → 155 | persisting |
| 1 | +0.145 | −0.371 | 281 | 246 | 245 → 1 | **collapsing** |
| 7 | +0.094 | −0.197 | 120 | 85 | 85 → 0 | **collapsing** |
| 99999 | −0.005 | −0.012 | 59 | 24 | 24 → 0 | **collapsing** |
| 2024 | +0.022 | −0.014 | 83 | 48 | 46 → 2 | **collapsing** |
| 42 | +0.017 | −0.046 | 91 | 56 | 56 → 0 | **collapsing** |

- `maintenance_cost`: mean Δ **−0.150, down in 6/6 seeds (100%)** — vs sign-MIXED (~50%) pure
  drift in the fixed economy. A reproducible directional collapse.
- `harvest_rate`: mean Δ +0.050, up in 5/6 (83%) — weakly directional up, but tiny.
- **Turnover collapses in 5/6 seeds**: second-half deaths crash toward 0, all runs freeze at 36
  carriers (immortal saturation). Sites never go negative (scaled guard holds); determinism OK.

## Interpretation (P1, P2, P3 all confirmed)

- **P1 CONFIRMED — and it is self-destructive.** `maintenance_cost` collapses degenerately
  (100% down, sign-consistent) because nothing opposes a lower metabolic drain: less drain =
  live longer = more reproduction, at zero cost — the §1 free attractor again. But this trait's
  collapse **removes the mortality the whole regime depends on**: as the drain weakens, carriers
  stop reaching energy ≤ 0, so deaths crash and the population freezes at immortal saturation
  (36/36). The shift is *small* (−0.15) only because it needs to be — a tiny drop below the
  harvest income flips carriers immortal, the population freezes, and selection then stops
  (no reproduction, no turnover) before the trait can move further.
- **P2 partial.** `harvest_rate` drifts weakly up (satiation caps its benefit, so no runaway and
  no resource-depletion trade-off strong enough to pin an interior optimum). No clean
  non-degenerate selection.
- **P3 CONFIRMED.** Activating the metabolic traits did **not** yield non-degenerate selection.
  It added a degenerate free direction (`maintenance_cost` → down) that is *worse* than the
  earlier ones: the population selects away its own death mechanism and self-terminates the
  turnover regime. **Criterion 5 cannot be made non-degenerate this way.**

## What this means for the project

1. **Criterion 5 stays degenerate.** Every trait coupled to fitness under the node-funded
   economy collapses to a selfish boundary with no counterforce: `replication_threshold` → floor
   (crit-5), and now `maintenance_cost` → floor (destroying turnover). This is a strong,
   consistent confirmation of the degenerate-attractor thesis: **without a genuine trade-off
   (a cost that opposes the selfish direction), there is no non-degenerate selection here.**
2. **The plan's deviation #4 deferral was correct.** Keeping harvest/maintenance as *fixed*
   quanta for Experiment 5 was not just an expressibility workaround — an active
   `maintenance_cost` is *dynamically incompatible* with a mortality-driven turnover regime.
3. **The path to real selection is a trade-off, not more active traits.** The only routes left
   for non-degenerate criterion-5 selection are ones that install an opposing cost: §1 option-2
   (parent-side replication cost) for `replication_threshold`, or a metabolic model where the
   selfish direction carries a price (e.g. harvest_rate coupled to a maintenance penalty). Each
   is a fresh pre-registration.

## Deliverable status

The reusable infrastructure — the property-valued effect ops `add_from_property` /
`subtract_from_property` (the "property×property effect op that does not exist" named in
deviation #4) — is built, tested in isolation (`property_effects_apply.rs`: conservative
transfer + skip semantics), and exit-criterion-7 clean (new enum variants + a default-false
config flag; `property_effects_regression` bit-identical, all hg-rules/hg-engine tests green).
The genome-scaled economy is available via `genome_scaled_metabolism` for any future
trade-off experiment; Experiment 5b itself is a reported negative on rescuing criterion 5.
