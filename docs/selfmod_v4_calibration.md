# selfmod v4 — calibration record

Pre-registered parameters and gate outputs that the v4 experiments depend on.
Everything here was fixed **before** the E1 grid was run.

## G4.3 — influx calibration (`I`)

Probe: flux economy, `churn3` seeded, μ = 0, 8 seeds × 1000 steps, per-`I` grid.

| I | seeds reaching first fission | median `time_to_first_fission` | froze before fissioning |
|---|---|---|---|
| 5 | 6/8 | 55 | 0/8 |
| 10 | 6/8 | 55 | 0/8 |
| 20 | 6/8 | 55 | 0/8 |
| 40 | 6/8 | 55 | 0/8 |
| 80 | 6/8 | 55 | 0/8 |
| 160 | 6/8 | 55 | 0/8 |

**Result: PASS.** The latency target (`time_to_first_fission ≤ 200`, no freeze
first) is met from `I = 5` upward.

**Calibrated operating point: `I = 20`.**

Two things this table settles, both worth stating plainly:

1. **Latency is completely insensitive to `I` across a 32× range.** At seed
   scale (3 rules), a rule's income is capped by `fire_reward` per productive
   step, never by the size of `free` — the pool is simply never the binding
   constraint. So `I` cannot be calibrated *against latency*; the plan's
   assumption that funding rate is set by `I` holds only once the population is
   large enough to compete for the pool.
2. `I` is therefore chosen on the other quantity it sets — the throughput
   carrying capacity **N\* ≈ I / rent = 20 rules** — which leaves headroom well
   above the 3-rule seed and far below `rule_cap = 400`.

The 2/8 seeds that never fission are not a calibration failure: their churn
settles into a low duty cycle and never reaches `repro_threshold`. That is a
property of the world those seeds generate, not of the influx — it is identical
at every `I`. The gate criterion accordingly requires the median over
*reproducing* seeds, not universal reproduction.

## G4.4 — minimal sustainer (`churnK`), k = 1

```
v0(label 0) —— v1(label 1)   ⇒   v0 → label 1, v1 → label 0, edge re-emitted
```

One rule. The matched edge is consumed by engine semantics and re-emitted
verbatim, so node and edge counts are exactly conserved; the endpoint labels
swap, so the rewrite is productive and regenerates its own precondition. Every
such edge in the world flips in parallel under MaxParallel (bindings are
node-disjoint).

| set | k | dead windows | distinct hashes over last 32 consecutive steps | period |
|---|---|---|---|---|
| `churnK` | 1 | 0/100 | 2 | 2 |
| `churn3` | 3 | 0/100 | 32 | none ≤ 8 |

**Result: PASS** — a genuine k = 1 sustainer exists, and `churn3` remains
aperiodic, so E3 has a real k contrast rather than a degenerate one.

**Measurement note (excursion).** The first G4.4 run reported `churnK` as
non-sustaining. That was an aliasing artifact: the criterion sampled the
canonical hash at 100-step window boundaries, and `churnK`'s attractor has
period 2, so every sample landed on the same phase and read as frozen. The
criterion now samples **consecutive** steps at the end of the run and reports
the detected period. `churn3`'s verdict is unchanged either way.

## Novelty-slope threshold

**`NOVELTY_SLOPE_THRESHOLD = 1e-4` new classes per step**, declared as a
constant in `runner.rs` and written into every `summary.txt`.

Justification, fixed before any v4 run: with `log_every = 100` over 20000 steps
the final third spans ~6667 steps, so the smallest slope distinguishable from
zero is one new class across that window, 1/6667 ≈ 1.5e-4 per step. 1e-4 sits
just below that, so it admits any genuine single-class gain while excluding
exact zero. Slopes are ordinary least squares over the logged points of the
final third (more robust than an endpoint difference).

Persistence threshold is the plan's own: survival ≥ 0.6 of seeds.

## Economy parameters

| | stock | flux |
|---|---|---|
| `total` | 2000 (conserved) | 2000 (initial endowment only) |
| `influx` | — | 20/step |
| `free_cap` | — | 500 (overflow dissipated) |
| rent | 1, recycled to `free` | 1, **dissipated** |
| starvation | tokens → `free` | tokens → `free` |
| `fire_reward` | 2, effect-gated, capped by `free` | same |
| `repro_threshold` / `birth_cost` | 100 / 50 | 100 / 50 |
| predicted N\* | total / birth_cost = 40 | I / rent = 20 |

The flow audit `cumulative_influx == bound + free + cumulative_dissipated` is
asserted every step in both modes; `cumulative_influx` is seeded with the
initial endowment so the identity holds from step 0. In stock mode influx and
dissipation are both zero and the audit degenerates to v3's
`free + Σ tokens == total`, which keeps the stock arm bit-comparable to v3.

## Carry-in fixes

- **F1 — effect-gate edge completeness.** The plan's concern was that v3's gate
  might see only labels. It does not: v3 already checked created/deleted nodes
  **and** edge-set content. G4.2 pins all 8 cases. One deviation from the plan's
  literal wording is deliberate — see the findings' deviation note.
- **F2 — genesis grace window.** `grace_steps = 100`; the terminate-at-0 check
  is suppressed inside it, so a startup transient no longer contaminates
  survival counts (v3 E0 lost a seed this way).
- **F3 — per-lineage creation budget.** `creation_budget_init = max = 200`,
  `creation_refill = 1`/step, charged on **net** node+edge creation. An
  edge-neutral, node-neutral churner spends nothing and is never throttled; a
  grower exhausts its own inherited allowance. Global `world_ceiling = 500` and
  `edge_ceiling = 1000` remain as backstops only.

## E4 addendum — F3 allowance calibration

| configuration | survival (`churn3 + grower`, flux, μ=0) | freeze |
|---|---|---|
| v3 all-or-nothing global clamp | 0/8 | — |
| `creation_budget_init/max = 200`, `refill = 1`/step | 1/8 | 7/8, median step 194 |
| `creation_budget_init/max = 40`, `refill = 0` | 0/8 | 8/8, median step 40 |

The loose allowance exceeds the world's headroom (200 initial + 1/step against 300 nodes
below `world_ceiling = 500`), so the global backstop still engages. Tightening it made the
outcome worse, not better — see the findings' E4 section for why the grower turns out to be
a substrate source for the churn cycle rather than purely a competitor.
