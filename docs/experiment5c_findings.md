# Experiment 5c — findings: parent-funded replication (§1 option 2)

Reported against `experiment5c_parent_funded_preregistration.md` (committed before running).
Harness: `crates/hg-sim-core/tests/experiment5c_parent_funded.rs`. Config-gated behind
`parent_funded_replication` (off by default ⟹ node-funded, exit-criterion 7 preserved).
**Nothing tuned.**

## Headline

**The first (partial) NON-DEGENERATE criterion-5 selection in the project.** Making the parent
pay the child cost from its own energy gives `replication_threshold` a genuine interior optimum
near the child cost that the majority of lineages settle at, and it preserves turnover — both
clear improvements over the node-funded floor-collapse. But the degenerate floor is **not
eliminated**, so convergence is partial, not universal.

## Result (child cost 10, turnover regime, 10 000 steps, 6 seeds)

| bootstrap repl_threshold | established | final repl_threshold (per seed) | interior [7,13] | collapsed <3 | turnover |
|---|---|---|---|---|---|
| 6  | 6/6 | 4.5, 1.8, 9.5, 9.5, −28.1, −6.3 | 2 | 3 | 6/6 |
| **10** | 6/6 | **9.8, 11.0, 3.2, 9.9, −2.1, 10.2** | **4** | 1 | **6/6** |
| 15 | 1/6 | 15.2 | 0 | 0 | 0/1 |
| 20 | 0/6 | (extinct) | — | — | — |

Across all established runs: **6 interior (near the child cost 10) vs 4 collapsed, of 13.**

## Interpretation

- **P1 — interior optimum: PARTIALLY confirmed.** Seeded *at* the child cost (bootstrap 10),
  **4 of 6 lineages settle tightly at ~10** (9.8, 11.0, 9.9, 10.2) — a real interior attractor
  where node-funding had none. The mechanism is exactly the pre-registered trade-off: a carrier
  must hold energy ≥ child cost to survive reproducing, so thresholds far below the cost are
  selected against. But the attractor is **not global**: 4/13 lineages still collapse toward the
  floor (some to −28), because an energy-RICH carrier can reproduce at a low threshold and still
  stay above the cost — the survival penalty only bites the poor. So parent-funding **opposes**
  the degeneracy without **eliminating** it.
- **P2 — establishment: confirmed, with a twist.** High bootstrap thresholds FAIL to establish
  (15 → 1/6, 20 → 0/6): satiation caps carrier energy at 20, so a founder needing energy ≥ 15–20
  to reproduce almost never can. Establishment is best when seeded near/below the child cost.
  (The predicted low-threshold-founder death is real too, but the satiation ceiling is the
  dominant establishment limit.)
- **P3 — turnover persists: CONFIRMED.** 6/6 alive at bootstraps 6 and 10 — unlike 5b, where
  `maintenance_cost` collapse froze the population immortal. Because reproduction now costs the
  parent energy, the population never goes immortal; births ≈ deaths throughout.
- **P4 — criterion-5 verdict.** This is the project's first trait shifting to (and clustering
  at) an **interior** value under a real trade-off — genuine non-degenerate selection — but it
  is **partial**: it co-exists with a residual degenerate basin. Honest headline: *parent-funding
  is the first mechanic that produces non-degenerate selection here, and it does so only
  partially.*
- **F-C — conservation/determinism: CONFIRMED.** Ledger balances to −2 per birth (parent −10,
  child +8; only the payer moved), max divergence 1.5e-9; runs bit-reproducible.

## Why it is not a hand-injected fitness force (the §8.1 discipline)

The child cost, the birth energy, and the −2 birth entropy are all **unchanged** from
node-funding — only *who pays* moved from the target site to the parent. No fitness term was
added to any genome; the selection pressure is an emergent consequence of a physical accounting
choice (an organism funds its offspring from its own reserves). The ledger identity is preserved
by construction. This is the "argued on merits, core-mechanic change" the pre-registration
required, not a knob tuned to manufacture selection.

## Where this leaves criterion 5

- Node-funded (Exp 5): `replication_threshold` collapses to the floor — degenerate.
- Active metabolic traits (5b): `maintenance_cost` collapses and *destroys* turnover — worse.
- **Parent-funded (5c): a real interior optimum for the majority + turnover preserved —
  the first non-degenerate selection, though partial.**

To make convergence *universal* (close the residual degenerate basin), the next lever would be
to make the survival penalty bite even the energy-rich — e.g. a parent cost that scales with
energy, or a per-reproduction mortality risk. Each is a separate pre-registration. Experiment 5c
is reported as a **partial success**: the mechanic works, its limits are measured, nothing tuned.

## Reusable deliverable

`parent_funded_replication` (config-gated) + the earlier `add_from_property`/
`subtract_from_property` ops are permanent infrastructure. Node-funded remains the default, so
all prior results and calibrations stand unchanged.
