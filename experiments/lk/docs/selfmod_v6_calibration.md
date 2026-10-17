# selfmod v6 — calibration record

Pre-registered parameters and gate arithmetic. Each section is written **before**
the runs it governs, and is not edited afterwards except to fill in measured values
in the places marked for them.

## Ground rule 7 — the standing conditioning statement

Every result in v3, v4 and v5 was produced at `match_cap_per_rule = 64` with
canonical-order truncation. That was introduced in v3 as a performance excursion
and never swept. Any v6 comparison that crosses the cap axis states this
explicitly, and any prior finding quoted alongside a v6 number is labelled with
the cap it was measured at.

## Operating point (inherited from v5, unchanged)

`gates::v6_base_cfg` is the single definition, so P0a, P0b, P1, G6.2 and P0c
cannot drift apart:

```
economy      = flux
income_mode  = proportional
income_pool  = 12
match_cost_coeff = 5e-4
base rent    = 1
mu           = 0        (except where an experiment sweeps it)
abio = p_dup = p_del = 0
world        = 200 nodes, 300 edges, 4 labels
world_ceiling = 500, edge_ceiling = 1000
creation budget = 200 + 1/step per lineage
```

Deviations from v5: none. The v5 calibration's justification for
`income_pool = 12, coeff = 5e-4` (8/8 resident survival at the smallest positive
mean net income) carries over unchanged.

## P0a — baseline-stable seed definition [pre-registered 2026-07-28, before any P0a run]

**A world seed is `baseline-stable` iff, under `churn3` at the operating point,
μ = 0, no perturbation, 20000 steps, the run both:**

1. **survives** — terminates on `budget` with `final_rules > 0`; and
2. **never freezes** — `time_to_freeze` is `none`, i.e. there is no run of 50
   consecutive steps with zero productive applications.

Both clauses are required. Survival alone is not enough: a population holding
tokens while producing nothing is not a live cycle, and v5's freezes at 242–964
steps are exactly the noise this phase exists to remove.

**The operative seed list is the `churn3` one**, because every downstream
invasion and selection experiment in v6 runs on `churn3`. `churnK` is measured
on the same 24 seeds and reported alongside, but does not gate the list.

**Seed set (24):** the eight v4/v5 seeds `{1, 2, 3, 7, 42, 99, 123, 777}` plus
`{4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20}`. The original eight
are deliberately included so that every prior v4/v5 result remains locatable
within the new set.

**If fewer than 8 seeds qualify**, that is reported as the finding — it would mean
the sustaining sets are far more fragile than G2/G4.4 implied — and the basin
itself becomes the object of study rather than a precondition for P0c.

**World-seed predictors recorded** for every seed, to test what forecasts
instability: initial per-label node counts, initial edge density (edges ÷ nodes),
mean candidate concentration on the top-8 contested nodes, and mean node-support
fraction of the accepted set.

## G6.2 — Pareto construction gate [pre-registered before any v6 invasion run]

A candidate invader is **admitted** to G5.3 only if, measured on the
baseline-stable seeds in a shared world alongside the residents:

1. `applied_productive` per copy per step **≥** the resident member's, **and**
2. `candidates_generated` per copy per step **<** the resident member's.

Clause 1 is the one v5's gate specified but never enforced. v5's invader failed it
by roughly half (0.80 vs 1.57 productive applications per copy per step) while
passing clause 2 by 730×, and that asymmetry is the reason its failure to invade
was uninterpretable.

### Two deviations from the gate as first implemented, both logged (ground rule 6)

**D1 — scored by substitution, not coresidence.** The first implementation added
the candidate *alongside* R3 and compared the two in one world. That is circular:
the candidate differs from R3 only by a guard, so the two contest the same
conflict-resolution slots and R3's measured rate falls exactly as the candidate's
rises. On that design R3's own rate swung 8× across candidates (0.086 → 0.729)
and v5's invader scored a work ratio of **1.08**, contradicting v5's own measured
0.51. The gate now measures each variant in R3's *place* — same role, same worlds,
nothing to suppress. Coresident ratios are still reported, labelled, because the
displacement they show is what the invasion run has to survive.

**D2 — a survival clause was added, and paired support required.** Both are
tightenings; neither relaxes a threshold.
- *Survival:* v5's invader scored a 1.086 work ratio while killing the population
  in 16/16 seeds. Its rate was a mean over the ~30 steps before collapse, which is
  not comparable with a rate over a live 2000-step run. A candidate that cannot
  hold R3's role is not Pareto-superior to it.
- *Paired:* the response is **bimodal** in θ (see below), so a 2% difference in
  means is not on its own evidence. Admission now also requires the variant to
  beat the control on a majority of seeds, seed by seed.

### Measured, 16 baseline-stable seeds × 2000 steps, cap 64 canonical

Control, unmodified `churn3`: R3 productive/copy/step **0.7346**, candidates/copy/step
**3990.2**, population survives 16/16.

| candidate | prod/c/s | vs R3 | paired | cand/c/s | vs R3 | paired | surv | verdict |
|---|---|---|---|---|---|---|---|---|
| v5 (added pattern edge) | 0.7976 | 1.086 | 9/16 | 1.6 | 0.0004 | 16/16 | **0/16** | reject: dies |
| guard energy>0.01 | 0.7307 | 0.995 | 15/16 | 3975.0 | 0.996 | 4/16 | 16/16 | reject: work |
| guard energy>0.02 | 0.1224 | 0.167 | 0/16 | 84.2 | 0.021 | 16/16 | 16/16 | reject: work |
| guard energy>0.05 | 0.1223 | 0.167 | 0/16 | 82.6 | 0.021 | 16/16 | 16/16 | reject: work |
| guard energy>0.08 | 0.7261 | 0.989 | 13/16 | 3659.6 | 0.917 | 15/16 | 16/16 | reject: work |
| **guard energy>0.1** | **0.7501** | **1.021** | **13/16** | **3541.9** | **0.888** | **15/16** | **16/16** | **ADMIT** |
| guard energy>0.15 | 0.1222 | 0.166 | 0/16 | 77.0 | 0.019 | 16/16 | 16/16 | reject: work |
| guard energy>0.2 | 0.1220 | 0.166 | 0/16 | 74.8 | 0.019 | 16/16 | 16/16 | reject: work |
| **guard energy>0.25** | **0.7629** | **1.039** | **10/16** | **2837.0** | **0.711** | **16/16** | **16/16** | **ADMIT** |
| guard energy>0.5 | 0.1222 | 0.166 | 0/16 | 50.8 | 0.013 | 16/16 | 16/16 | reject: work |
| guard energy>0.75 | 0.4333 | 0.590 | 1/16 | 486.9 | 0.122 | 14/16 | 14/16 | reject: dies |
| guard energy>0.9 | 0.1417 | 0.193 | 0/16 | 13.5 | 0.003 | 15/16 | 15/16 | reject: dies |

**Admitted to G5.3: `guard energy>0.1`** (primary) and `guard energy>0.25`
(secondary, larger cost advantage but weaker paired work support). θ was selected
*on this gate*; the invasion threshold below was fixed beforehand and is untouched.

**The response is bimodal, and that is itself a result.** Productive rate takes one
of two values — ~0.73 or ~0.122 — and which one a variant lands in is not monotone
in θ: {0.01, 0.08, 0.1, 0.25} land high, {0.02, 0.05, 0.15, 0.2, 0.5} land low.
Rule specificity therefore does not map smoothly onto realised productivity in this
substrate; it selects among basins. Any claim that selection can *climb* an
efficiency axis has to contend with that.

## G5.3 — invasion threshold [carried over from v5 verbatim, NOT relaxed]

The invader's lineage share must reach **≥ 2× its seed share in ≥ 6 of 8 seeds**,
measured at end of run against the share recorded at the step it was introduced.

Only the mechanism under test changes in v6 (invader construction). The bar,
the measurement, and the paired no-invader control (G6.3) are identical to v5.

### Measured, 8 baseline-stable seeds × 2000 steps, invader introduced at step 500

| invader | work vs R3 | cand vs R3 | invades | grew | R3 final (inv / control) |
|---|---|---|---|---|---|
| `guard energy>0.5` | 0.166× | 0.013× | **8/8** | 8/8 → 4–5 copies | 0 / 1–2 |
| `guard energy>0.25` | 1.039× | 0.711× | **6/8** | 6/8 → 2–3 copies | 0 / 1–2 |
| `guard energy>0.1` | 1.021× | 0.888× | 3/8 | 3/8 | 0–1 / 1–2 |
| `guard energy>0.01` | 0.995× | 0.996× | 0/8 | 0/8 | 1–2 / 1–2 |
| clone of R3 | 1.000× | 1.000× | 0/8 | 0/8 | 1–2 / 1–2 |
| v5's invader | dies in substitution | 0.0004× | 0/8 | 0/8 | 1–2 / 1–2 |

All eight paired controls survive in every arm, so G6.3 is satisfied and the
displacement claims are legitimate.

**The threshold is met by `guard energy>0.25` at exactly 6/8 and was not
touched.** It is nonetheless not usable as evidence, because the same threshold
is exceeded by `guard energy>0.5`, which does 6× *less* productive work, and is
unreachable by a behaviourally identical clone. Invasion strength is monotone in
candidate reduction across the whole table and bears no relation to productive
work. See `selfmod_v6_findings.md` §P0c.

### Null controls added in v6 (not in v5)

- **Neutral clone** — R3 verbatim, no guard. Establishes what an arriving copy
  achieves for free, from its founding endowment and the population's
  reproduction dynamics alone. v5 compared its invader against zero; the correct
  comparison is against this. Measured: **0/8**, the clone goes extinct, because
  it shares R3's `class_hash` and its later `K_SEQ` puts it behind R3 for every
  contested binding.
- **Priority override** — the invader's `priority` field can be set
  independently of its structure, to test queue position as a cause rather than
  a correlate. Measured: raising v5's invader and the clone to priority 1 leaves
  both at 0/8, so queue position alone is not sufficient.
- **Cost-signal ablation** — `match_cost_coeff = 0`. Measured: both guard
  variants then invade 8/8, and `guard energy>0.25` *improves* from 6/8 to 8/8.
  Match cost modulates invasion strength but is not what makes invasion possible.
