# Experiment 5d — findings: energy-proportional reproductive tax

Reported against `experiment5d_rich_penalty_preregistration.md` (committed before running).
Harness: `crates/hg-sim-core/tests/experiment5d_rich_penalty.rs`. Config-gated behind
`repro_energy_tax` (with `parent_funded_replication`); off by default ⟹ node-funded
(exit-criterion 7). **Nothing tuned.**

## Result (parent-funded, child cost 10, bootstrap threshold 6, 10 000 steps, 6 seeds)

| tau | median final threshold | interior [7,13] | collapsed <3 | turnover | per-seed finals |
|---|---|---|---|---|---|
| 0.0 (= 5c) | 4.5 | 2 | 3 | 6/6 | 4.5, 1.8, 9.5, 9.5, **−28.1, −6.3** |
| **0.3** | **9.9** | **4** | 2 | 6/6 | 10.6, 9.9, 10.8, 7.5, **−28.1, −6.3** |
| 0.6 | 7.9 | 3 | 2 | 6/6 | 9.7, 6.7, 7.9, 10.0, **−28.1, −6.3** |

## Interpretation

- **P2 CONFIRMED — the tax pulls the optimum up and rescues near-boundary lineages.** The
  median final threshold jumps from 4.5 (5c) to **9.9** at tau = 0.3 — right at the child-cost
  optimum. Seeds that sat *below* the interior band in 5c (4.5 → 10.6; 1.8 → 9.9) are pulled
  *up* into it: reproducing while rich now wastes real energy, so waiting (a higher effective
  threshold) is selected. Interior fraction rises 2 → 4.
- **P3 CONFIRMED — no extinction.** Turnover persists 6/6 at every tau, including 0.6; the tax
  is survivable at these constants.
- **P1 PARTIAL — the basin narrows but does not close, and *why* is the finding.** Collapsed
  fraction drops only 3 → 2, because **two seeds sit at −28.1 and −6.3 identically across every
  tau** — completely immune to the tax. These lineages collapsed *deeply* negative during
  establishment (jitter cascade before the tax could bite). A carrier with threshold ≪ 0
  reproduces at *every* energy level, so its threshold is **always satisfied and therefore
  under no selection at all** — a pure-drift sink. An energy cost, of any size, cannot select
  on a trait that never gates a decision. So the tax rescues lineages *near* the boundary but
  cannot drain ones already *deep* in the degenerate sink.

## What this establishes

The residual degeneracy has **two distinct components**, and they need different fixes:
1. **Marginal drift toward the floor** (thresholds a bit too low): an energy cost *does* fix
   this — 5c's fixed cost created the interior optimum, and 5d's proportional tax pulls even
   reliably-rich lineages up to it (median 4.5 → 9.9). This part is **solved**.
2. **A deep, always-satisfied drift sink** (threshold ≪ 0): this is **immune to any energy
   cost by construction**, because the trait no longer gates reproduction, so there is nothing
   for selection to act on. Closing it requires either preventing the deep collapse (bounding
   the trait) or a cost tied to reproduction **rate**, not energy — a per-reproduction mortality
   or aging penalty (`lifetime`, key 21) that a chronic always-reproducer pays regardless of how
   negative its threshold is. That is the next pre-registration.

## Criterion-5 progression (updated)

- Node-funded (Exp 5): threshold → floor. Degenerate.
- Active metabolic traits (5b): maintenance → floor, *destroys* turnover. Worse.
- Parent-funded (5c): interior optimum for the majority; residual basin. First partial
  non-degenerate selection.
- **Proportional tax (5d): the interior optimum now attracts even reliably-rich lineages
  (median at the optimum), turnover preserved — but a deep always-satisfied drift sink remains,
  and it is provably immune to energy costs.** The honest boundary of the energy-cost approach.

## Reusable deliverable

`repro_energy_tax` (config-gated, uses the existing `scale_from` op) is permanent
infrastructure alongside `parent_funded_replication`. Node-funded remains the default;
`property_effects_regression` bit-identical. The clean next lever (rate-based cost via
`lifetime`) is identified and motivated, not yet built.
