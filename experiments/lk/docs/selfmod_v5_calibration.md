# selfmod v5 — calibration record

Pre-registered parameters and gate arithmetic. Fixed before the runs they govern.

## Invasion threshold (G5.3)

**Pre-registered before any invasion run:** the invader's lineage share must reach
**≥ 2× its seed share in ≥ 6 of 8 seeds**, measured at the end of the run against the
share recorded at the step it was introduced (`invasion_share_seed` → `invasion_share_final`,
both written to every `summary.txt`).

Justification: with 3 resident members plus one invader instance, seed share is ~0.09–0.10.
A 2× bar means the variant must roughly double its representation — enough to distinguish
genuine positive selection from drift in a population of ~10 rules, while not requiring
outright fixation, which a single beneficial member of a closed cycle should not be expected
to achieve.

The threshold was **not** relaxed after the result. It was missed by the maximum possible
margin: the invader's share went to exactly 0.000 in 8/8 seeds at the operating point, and
never exceeded 0.71× in any of the seven parameter settings swept afterwards.

## The invader (G5.3)

Resident `churn3` member R3 is `A(label 0), D(label 3)` with **no pattern edge** — two
isolated variables, so the matcher enumerates every (label-0, label-3) pair in the world.
The invader is identical except for one added pattern edge between the two variables, so it
enumerates only adjacent pairs.

Measured, per copy per step, at the operating point:

| | R3 | invader | ratio |
|---|---|---|---|
| match candidates | 3640 | 5 | 730× fewer |
| productive applications | 1.57 | 0.80 | 0.51× |
| productive per candidate | 0.00043 | 0.16 | **370× better** |

The variant is therefore strictly superior on the efficiency metric §3.1b was built to
reward, which is what makes its failure to invade informative rather than merely negative.

## Income and cost calibration (G5.2)

`income_i = floor(income_pool × productive_i / Σ productive_j)`, integer remainder handed
out one token at a time in canonical (sorted-key) order.
`cost_i = base_rent + match_cost_coeff × candidates_i(previous step)`.

Candidate operating points, 8 seeds × 2000 steps, `churn3`, μ = 0:

| income_pool | coeff | mean net income | residents surviving |
|---|---|---|---|
| 3 | 2e-4 | 0.00 (starved) | 0/8 |
| 6 | 2e-4 | +2.84 | 5/8 |
| 6 | 5e-4 | 0.00 (starved) | 0/8 |
| 9 | 5e-4 | +3.69 | 2/8 |
| **12** | **5e-4** | **+4.84** | **8/8** |
| 9 | 1e-3 | 0.00 (starved) | 0/8 |
| 15 | 1e-3 | +7.28 | 2/8 |
| 20 | 1e-3 | +8.13 | 8/8 |

**Operating point: `income_pool = 12`, `match_cost_coeff = 5e-4`, `base_rent = 1`.**

Chosen for 8/8 resident survival — an established, stable resident population is a
precondition for a clean invasion test — with the smallest positive mean net income among
the settings that achieve it. The plan asks for the seed set "near net ≈ 0"; the rows that
achieve exactly 0 are degenerate (universal starvation, 0/8 survival), so the nearest
*viable* point was taken instead and the deviation is recorded here.

Note that net income must be strictly positive for reproduction to occur at all, since
fission is gated on accumulating `repro_threshold` tokens — which is itself the structural
problem the G5.3 diagnosis identifies.

**G5.2 outcome:** per-rule `net_income` variance is 1.5–12.6 across these settings, versus
**exactly 0** under v4's flat income (G5.1). The pool binds and income is differentiated.

## G5.1 arithmetic

Under flat income every productive rule receives `fire_reward` and pays `rent`, so every
productive rule nets exactly `fire_reward − rent` and the variance is analytically 0. The
gate measures it rather than assuming it, because the reward is capped by `free`
(`pay = fire_reward.min(free)`) and a pool that ran dry mid-loop would break the equality.
It never does — `free` is pinned at its cap throughout, which is the v4 observation the
whole v5 reframe rests on. Measured variance: **0.000e0, all 8 seeds, every logged step.**

## E-stoich design (§5.1)

- μ = 0 and `repro_threshold = 10^7` (unreachable), so natural fission cannot occur and the
  forced duplication is the only perturbation.
- Perturbation step **50**. The first attempt used step 500 and was void: the `churn3` cycle
  freezes at ~242 and rules begin starving by ~300, so the perturbation landed on a dead
  population and all five arms came out identical.
- Response variable is **time to freeze**, not survival: survival is 2/8 in every arm
  including control, so it has no resolving power here.

**Confound recorded:** rules generate ~1141 candidates/step against `match_cap_per_rule = 64`,
and conflict resolution accepts only ~5 node-disjoint bindings/step. Duplication therefore
alters a member's share of conflict-resolution slots rather than its intrinsic reaction rate,
so this tests realised-rate perturbation, not idealised stoichiometry.

## Unchanged from v4

Novelty-slope threshold `1e-4` with the 20k/60k horizon discipline; persistence threshold
`P ≥ 0.6`; `world_ceiling = 500`, `edge_ceiling = 1000`, per-lineage creation budget
200 + 1/step; `match_cap_per_rule = 64`; flux economy (≡ stock, retained for the audit trail);
seeds {1, 2, 3, 7, 42, 99, 123, 777}.

## Implemented but unused

Built to plan and available, but not exercised because the plan stopped at G5.3:
balanced copy-number seeding (`copy_number`, §3.2), the `split` / `partial` / `refractory`
fission modes (§3.3), and per-member census metrics (`copy_number_*`, `stoich_deviation`).
The resource landscape (§3.4) was correctly **not built** — it was gated behind reaching §5.4.
