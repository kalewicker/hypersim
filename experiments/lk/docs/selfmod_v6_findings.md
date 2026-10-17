# selfmod v6 — findings

Tags: **[Measured]** = read off a run. **[Inferred]** = argued from measurements.

Branch: `selfmod-v6` off `selfmod-v5`. `hg-engine`, `hg-core`, `hg-schema` untouched
(ground rule 1). Flow-audit residual 0 on every step of every run.

**Standing conditioning statement (ground rule 7).** Every result in v3, v4 and v5 was
produced at `match_cap_per_rule = 64` with canonical-order truncation. P1 shows that
setting is not a neutral performance detail. Where a v6 number is compared with a
pre-v6 one, the cap is stated.

---

## Headline

**P0c's pre-registered threshold is met — and the controls it is paired with show the
threshold does not measure fitness.**

A variant verified Pareto-superior by G6.2 (`guard energy>0.25`: 1.039× R3's productive
work for 0.711× its candidates) invades in 6/8 seeds, exactly the bar v5 set. But a
variant doing **6× less** productive work invades in **8/8** and grows twice as large,
and a behaviourally **identical** copy of the resident invades in **0/8**. Invasion
strength tracks candidate-count reduction and is unrelated to productive output. The
gate cannot tell a better rule from a worse one, so neither its pass nor v5's failure
supports a conclusion about the reproduction criterion.

---

## G6.0 / G6.1 — regression and instrument

**[Measured]** G0 (10 fixtures round-trip, 5 malformed → None), G1 (bit-identical on
3 seeds × 100 steps), G2 (10000 steps, 0 dead windows), G4.2 (all 8 effect-gate cases).

**[Measured]** The v6 instrumentation is dynamics-neutral: replaying archived v5 run
`E5_flux_mu0_churnk_seed42` under the v6 binary reproduces all **601 logged world
canonical hashes bit-identically** over 60000 steps.

**[Measured] G6.1 PASS.** The five-way breakdown
`generated = truncated + rejected_by_conflict + applied_sterile + applied_productive`
closes exactly, per rule per step, asserted inline on every step of every run, across
caps {8, 64, unlimited} and both truncation orders. The aggregate is independently
cross-checked against per-member totals.

---

## P0b — which fate dominates: **truncation, overwhelmingly**

**[Measured]** `churn3`, 8 seeds × 200 steps, cap 64 canonical, 8.0M candidates:

| fate | share |
|---|---|
| truncated by cap | **98.30%** |
| lost to conflict resolution | 1.63% |
| applied, sterile | **0.0000%** |
| applied, productive | 0.0655% |

Per member, candidates generated per copy per step: R1 2.1, R2 1.1, **R3 4391.5**.
R3 alone accounts for essentially all enumeration, because it is the one member with
two label-constrained variables and **no pattern edge**, so it enumerates the full
(label-0 × label-3) product.

**[Measured] Sterile applications are exactly zero**, in every configuration measured.
Nothing that reaches application is wasted. This settles P0b's routing question in the
"truncation/conflict loss" branch: a guard cannot preserve productive work by excluding
matches that were going to be sterile, because there are none. Any guard necessarily
changes *which* bindings win the draw.

**[Measured]** Removing the cap inverts the picture entirely: at unlimited cap,
generated falls to 155.4 per copy per step for R3 (28× fewer) and the productive share
rises to 3.67% (56× higher). The enormous candidate volume at cap 64 is not a property
of the rules — it is a state the capped dynamics drive the world into.

---

## P1 — the contention regime: **branch (a) and branch (c), both confirmed**

### The cap is load-bearing for survival

**[Measured]** `churn3`, μ = 0, 8 baseline-stable seeds × 20000 steps, at the operating point:

| cap | order | survived | froze | generated/step | truncated | conflict | accepted/step |
|---|---|---|---|---|---|---|---|
| 8 | canonical | **8/8** | 0/8 | 5250.0 | 0.9958 | 0.0037 | 2.44 |
| 64 | canonical | **8/8** | 0/8 | 4840.4 | 0.9828 | 0.0166 | 2.70 |
| 256 | canonical | 3/8 | 4/8 | 1063.7 | 0.9193 | 0.0786 | 4.61 |
| 1024 | canonical | **0/8** | 7/8 | 4.5 | 0.6461 | 0.3454 | 5.80 |
| unlimited | canonical | **0/8** | 7/8 | 0.8 | 0.0000 | 0.9629 | 6.00 |
| 64 | **random** | **0/8** | 8/8 | 1.2 | 0.9400 | 0.0351 | 4.89 |

At μ = 0.05 every cap gives 0/8.

**Branch (a) holds.** Survival goes 8/8 → 3/8 → 0/8 as the cap is raised. Every
sustaining result the programme has produced since v3 sits inside a narrow cap window
of roughly ≤ 64–256.

**Branch (c) holds, and is the sharper result.** Compare the two `64` rows: identical
cap, identical number of candidates admitted, differing only in *which* ones.
Canonical survives 8/8; random survives 0/8. **Canonical ordering is load-bearing
independently of the cap.**

### But the cap does not break rewriting — it interacts with the creation budget

**[Measured] P1a**, same rulesets with the economy removed entirely (no tokens, no
reproduction, unlimited creation budgets, G2's sustaining criterion):

| cap | 8 | 64 | 256 | 1024 | unlimited | 64 random |
|---|---|---|---|---|---|---|
| sustains | 8/8 | 8/8 | 8/8 | 8/8 | 8/8 | 8/8 |
| productive/step | 2.44 | 3.01 | 6.44 | 21.07 | **76.06** | 59.11 |

**Every cap sustains, with zero dead windows.** Productive throughput rises
monotonically with the cap. So the high-cap collapse in the table above is not a
failure of the rewriting.

**[Measured]** The cause is the interaction with v4's F3 per-lineage creation budget.
At unlimited cap, varying only that budget:

| creation budget | 200 (v4 default) | 2 000 | 20 000 | 1 000 000 |
|---|---|---|---|---|
| survived | 0/8 | 0/8 | 0/8 | **8/8** |

**[Inferred]** The grower R1 fires ~76×/step at unlimited cap against an allowance of
200 + 1/step, calibrated for the cap-64 rate of ~1/step. It exhausts its allowance
within a few steps, is skipped, and the cycle stalls for want of its stage; the rules
then starve at rent with no income. A step trace confirms the shape: 59–67 productive
applications/step for 8 steps, then 34 → 7 → 2 → 0, then 25 steps of exactly zero
productive work while tokens bleed at 4/step.

**The sustaining regime is set jointly by two engineering guards** — `match_cap_per_rule`
(a v3 performance excursion) and the F3 creation budget (a v4 fix) — neither of which
was introduced as a model of anything. That is the substrate every selfmod result since
v3 has been measured in.

---

## P0a — the basin, and a correction to C3

**[Measured]** 24 world seeds × 20000 steps, μ = 0, no perturbation, operating point:
`churn3` survives **23/24** with 1 freeze (seed 19, at step 156); `churnK` survives
**24/24** with 0 freezes.

**Baseline-stable seeds (23):** all of `{1,2,3,7,42,99,123,777}` — the entire v4/v5 seed
set — plus `{4,5,6,8,9,10,11,12,13,14,15,16,17,18,20}`. Because the original eight all
qualify, every v6 experiment runs on exactly the seeds v5 used, and the comparison with
v5's failure is exact rather than approximate.

**[Measured] C3's premise was wrong, and the cause is identified.** C3 assumed ~50%
baseline death was a property of the ruleset–world pair. It is a property of the
**income mode**. Same 12 seeds, same ruleset, same μ = 0, differing only in economy:

| income mode | survived | freezes |
|---|---|---|
| flat (v4) | **4/12** | 8, at steps 194–642 |
| proportional, pool 12 (v5/v6 operating point) | **12/12** | 0 |

The four flat-income survivors are `{2, 42, 99, 777}` — exactly the v4 `churn3` μ = 0
stratum that P2 independently identifies. **[Inferred]** proportional income at pool 12
funds roughly twice the accumulation rate of flat income, so lineages reach the fission
threshold and build the redundant copies that v5's C1 identified as the repair mechanism.

**[Measured] What "baseline-stable" actually looks like.** The surviving populations are
productive throughout (1–2 productive applications/step, no zero windows) but the world
sits in a 2-cycle: sampling `canonical_hash` on even steps over 1000 steps yields **2
distinct values**, on odd steps **500 distinct values**. `canonical_hash` folds in each
node's `generation` stamp, so the odd-step "novelty" is the creation timestamp of a
transient node that the next step deletes, not structural novelty. Sampling at 100-step
multiples — every logged row in every run since v3 — lands on the same phase every time
and reads as a constant. This is the same aliasing artifact G4.4 hit, in a new place.

---

## P2 — conditional μ_c(k): **branch (a), v4's claim does not survive**

Zero new compute; archived v4 runs only, all at cap 64.

**[Measured]** v4 reported μ_c(k=1) = (0.02, 0.03] against μ_c(k=3) = (0, 0.005], read as
"lower by at least an order of magnitude" and taken as confirming that distributed
autocatalytic sets are more fragile.

**[Measured]** The `churn3` baseline-surviving stratum is `{2, 42, 99, 777}` (4/8). Of
those, **only seeds 2 and 777 ever reproduce**; 42 and 99 record `births_clonal = 0,
births_mutant = 0` at every μ from 0 to 1.0. Mutate-at-birth is the only variation
channel in the model, so μ never fires on them — their survival at μ = 1.0 is a
tautology, not robustness. `churnK`'s stratum is 8/8 and every member reproduces
(400 births each at μ = 0).

| | k=1 (`churnK`) | k=3 (`churn3`) |
|---|---|---|
| baseline-surviving | 8 | 4 |
| ...of which reproduce (μ can act) | **8** | **2** |
| μ_c conditional | (0.02, 0.03] | (0.01, 0.02] |
| first exposed loss | (0, 0.005] | (0, 0.005] |
| exposed extinction | (0.05, 0.1] | (0.01, 0.02] |

**v4's E3 compared 2 informative seeds against 8.** Conditioning on baseline survival
alone collapses the gap to adjacent grid intervals. Both arms lose their first exposed
seed in the same interval. At n = 2 the 0.5-crossing bracket moves a full grid interval
when a single seed changes, so **the order-of-magnitude claim is not supported in either
direction**. The direction of the residual effect still favours v4's hypothesis
(extinction at (0.01, 0.02] vs (0.05, 0.1]), but on n = 2 that is weak evidence.

A marked correction note has been added to `selfmod_v4_findings.md` §E3.

---

## G6.2 — a Pareto-superior variant exists

Two design corrections, both logged in the calibration doc (D1, D2): the gate is scored
by **substitution** rather than coresidence (coresidence is circular — the variant and
its own template contest the same slots, so the resident's measured rate falls exactly
as the variant's rises, and on that design v5's invader scored 1.08 against v5's own
measured 0.51), and a survival clause plus paired per-seed support were added (both
tightenings).

**[Measured]** 16 baseline-stable seeds × 2000 steps. Control R3: 0.7346 productive per
copy per step, 3990.2 candidates, population survives 16/16.

| candidate | prod/c/s | vs R3 | paired | cand/c/s | vs R3 | surv | verdict |
|---|---|---|---|---|---|---|---|
| v5 (added pattern edge) | 0.7976 | 1.086 | 9/16 | 1.6 | 0.0004 | **0/16** | reject: dies |
| guard energy>0.01 | 0.7307 | 0.995 | 15/16 | 3975.0 | 0.996 | 16/16 | reject: work |
| **guard energy>0.1** | **0.7501** | **1.021** | 13/16 | **3541.9** | **0.888** | 16/16 | **ADMIT** |
| **guard energy>0.25** | **0.7629** | **1.039** | 10/16 | **2837.0** | **0.711** | 16/16 | **ADMIT** |
| guard energy>0.5 | 0.1222 | 0.166 | 0/16 | 50.8 | 0.013 | 16/16 | reject: work |

**[Measured] The response is bimodal and non-monotone in θ.** Productive rate takes one
of two values, ~0.73 or ~0.122, and which one a variant lands in does not follow the
guard's selectivity: {0.01, 0.08, 0.1, 0.25} land high, {0.02, 0.05, 0.15, 0.2, 0.5}
land low. **[Inferred]** the split is predicted by where the variant's `class_hash` sorts
relative to R1's: 10 of 11 variants land in the high mode exactly when they sort *after*
R1. `decode_all` orders rules by `(class_hash, K_SEQ)`, and `Binding::canonical_key` is
`(Reverse(priority), rule_id, sorted nodes)`, so within a priority class every binding of
a lower-`rule_id` rule is considered before any binding of a higher one. Rule specificity
does not map smoothly onto realised productivity here; it selects among basins.

---

## P0c / G5.3 — the gate is met, and the gate is not a fitness test

Every arm paired with a no-invader control on identical configuration (G6.3). All
controls survive, so displacement claims are legitimate. 8 baseline-stable seeds,
2000 steps, invader introduced at step 500, threshold **unchanged from v5**.

**[Measured]**

| invader | work vs R3 | candidates vs R3 | invades | grew past 1 copy | R3 final (inv arm / control) |
|---|---|---|---|---|---|
| `guard energy>0.5` | **0.166×** | 0.013× | **8/8** | 8/8 → 4–5 copies | **0** / 1–2 |
| `guard energy>0.25` | **1.039×** | 0.711× | **6/8** | 6/8 → 2–3 copies | **0** / 1–2 |
| `guard energy>0.01` | 0.995× | 0.996× | 0/8 | 0/8 | 1–2 / 1–2 |
| clone of R3 (identical) | 1.000× | 1.000× | **0/8** | 0/8 | 1–2 / 1–2 |
| v5's invader | dies in substitution | 0.0004× | **0/8** | 0/8 | 1–2 / 1–2 |

**On the pre-registered criterion this is a PASS** (`guard energy>0.25`, 6/8 ≥ 6/8), and
it is a real one: the invader genuinely reproduces, the population does not collapse
(10–13 rules, same as control), and R3 is driven to zero in 8/8 seeds. v5's invader
reproduces its 0/8 failure exactly, confirming the harness.

**The controls make that PASS uninterpretable as evidence about fitness:**

1. **A strictly worse variant invades harder.** `guard energy>0.5` does **6× less**
   productive work per copy per step than R3 and invades 8/8 to 4–5 copies, against the
   Pareto-superior variant's 6/8 to 2–3 copies. Invasion strength is **monotone in
   candidate-count reduction** (0.996× → dies, 0.711× → 6/8, 0.013× → 8/8) and shows no
   relationship to productive work (0.995×, 1.039×, 0.166× across those same three).
2. **A behaviourally identical copy cannot invade at all.** The clone shares R3's
   `class_hash`, so it ties and is broken by `K_SEQ`; installed later, it is ordered
   behind R3 for every binding and gets nothing. It reaches 0 copies in 8/8. So does
   `guard energy>0.01`, which differs from R3 by ~1%.
3. **Removing the cost signal does not remove the effect.** At `match_cost_coeff = 0`
   both guard variants invade 8/8 — the expensive one *improves* from 6/8 to 8/8. So
   match cost modulates invasion strength but is not what makes invasion possible.

**[Inferred] What the gate is actually measuring.** With 98.3% of candidates truncated in
canonical order, a rule's realised output is set by *which region of the world its
surviving 64 candidates land on*. R3's canonically-first 64 are maximally degenerate —
they share the lowest-keyed node, so conflict resolution accepts about one of them. A
variant whose guard removes that region lands its truncated set elsewhere, stops
colliding with R3, and wins slots R3 never contested; a variant that lands on the *same*
region loses every tie-break and starves. This is the same force P1 isolated: canonical
ordering at 8/8 versus random ordering at 0/8, same cap. Invasion outcome here is a
property of truncation geometry, not of the economy.

### Verdict against the plan's pre-committed branches

The plan's branch (a) says a pass means "v5's diagnosis was wrong; skip P3; unblock P4".
**That routing is not supported and should not be taken.** The pass is real but the
instrument that produced it is not fitness-discriminating, so it is not evidence that
v5's reproduction-criterion diagnosis was wrong — and equally, v5's failure was never
evidence that it was right. Branch (c) is closer in spirit: the efficiency axis is not
*measurable* here, though for a different reason than the plan anticipated — not because
no Pareto-superior variant can be constructed (G6.2 built one), but because invasion
outcome is dominated by a truncation artifact that swamps the fitness signal.

**P3 and P4 are both blocked**, on a defect neither was designed to address.

---

## Deliverables and reproduction

```
cargo build --release -p hg-selfmod
./target/release/hg-selfmod gates      # G6.0 regression
./target/release/hg-selfmod gate61     # G6.1 fate accounting
./target/release/hg-selfmod p0a        # basin, 24 seeds
./target/release/hg-selfmod p0b        # fate breakdown
./target/release/hg-selfmod p1         # cap sweep + ordering arm
./target/release/hg-selfmod gate62     # Pareto construction gate
./target/release/hg-selfmod p0c        # invasion + controls
./target/release/hg-selfmod orderprobe # rule-order diagnostic
python3 crates/hg-selfmod/p2_conditional_muc.py
```

Figure: `docs/selfmod_v6_pareto.svg` — the two axes G6.2 gates on, with v5's invader and
v6's candidates placed against the Pareto region, annotated with invasion outcome.

---

## Routing for v7

**Do not build P3's reproduction modes or P4's experiments yet.** Both assume the
invasion gate measures fitness, and v6 shows it does not.

1. **Fix the instrument first.** The single highest-value change is to remove
   `rule_id` from `Binding::canonical_key`'s sort, or to break ties among equal-priority
   bindings by something merit-bearing or random rather than by rule identity. As it
   stands, two coexisting rules of equal priority do not share work: the canonically
   earlier one takes first claim on the whole world. This affects **every multi-rule
   result since v3**, not just the invasion gate. Note that this lives in `hg-engine`, so
   it is the first thing in six versions that ground rule 1 forbids fixing in place — v7
   should either lift that rule for this specific change or implement conflict resolution
   locally in `hg-selfmod`, and say which.
2. **Re-establish the cap on evidence, and state the exposure.** The operating cap stays
   64 not because it is unbiased but because it is one of only two settings where the
   population survives at all. Prior results are conditioned on the joint
   cap × creation-budget regime, and any future comparison must say so.
3. **Then, and only then, re-run G5.3.** The invader construction machinery (G6.2,
   substitution scoring, the neutral-clone null) is built and validated; it can be reused
   unchanged once the tie-break is fixed. The neutral clone should reach roughly its seed
   share, and a Pareto-superior variant should beat it. Neither holds today.
4. **§7's world-sourced income remains correctly deferred.** It merges the gradient and
   moving-target variables, and the gradient question is not yet answerable.
