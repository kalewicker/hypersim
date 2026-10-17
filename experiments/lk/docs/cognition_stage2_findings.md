# Cognition Stage 2 — evolved rule-selection bias

Phase C5 of `hypersim_nn_experiments_plan_v1.md`, Stage 2.

**Hypothesis.** A per-rule preference vector can reliably steer the
substrate into target regimes.
**Exit as written.** Champion beats budget-matched random by > 2σ on the
five frozen hold-out seeds, no Goodhart flags, on ≥ 1 non-fixture rule set.

Produced by `crates/hg-train/tests/c5_stage2.rs` and
`crates/hg-train/tests/c5_stage2_landscape.rs`. Prerequisites: Stage 0a's
exit is met (`docs/cognition_stage0_findings.md` §1) and F-L1 passes.

> **Headline: the exit criterion is met, and it does not mean what it
> appears to mean.** On `wn_three` the champion beats budget-matched random
> by 0.7636 at 3.11σ with 5/5 seeds and no Goodhart flags. An **untrained**
> `StaticBias` at θ = 0 scores exactly the same. The margin is a difference
> between two policy *classes*, present before any training, and CMA-ES
> contributed nothing to it. §3 is the control that establishes this.
>
> **Second headline: the negative survives a correctly-sized optimizer.**
> The default step size (σ = 0.02) provably prevented CMA-ES from moving at
> all in logit space (§4.1). At σ = 2.0 it moves decisively — to
> allocations of `[0.212, 0.777, 0.011]` and `[0.001, 0.999]` — and the
> champions still do not beat an untrained genome: +0.0000 on `wn_three`,
> **−0.0182** on `tri_two`. Both substrates behave the way §5 predicts.

---

## 1. Stage 2 could not run on any shipped rule set

**Every builtin rule set ships exactly one rule.** `StaticBias` has one
parameter per rule and allocates budget by a masked softmax over the rule
logits. A softmax over a single logit is 1.0 for every value of that logit,
so on a single-rule set the genome cannot change the distribution at all
and the policy is **identical to budget-matched random**.

Stage 2's hypothesis is therefore not false on `glider`,
`triangle_subdivide`, `binary_edge` or `growth_treadmill` — it is
**unstateable** there. A campaign would have run to completion, produced a
champion, and reported a hold-out margin that was entirely policy-RNG
noise.

Pinned in two places so it cannot be rediscovered the expensive way:
`hg-cognition`'s `a_single_rule_genome_cannot_change_selection`, and a
refusal in `campaign::run_campaign`.

### 1.1 The composed substrates

`docs/policy_config.md` §2.5's liveness matrix has two co-live pairings, so
multi-rule substrates are constructible from measured-live rules rather
than invented:

| rule set | IC | rules | why |
|---|---|---|---|
| `rules/wn_three.toml` | `WhiteNoise` | `binary_edge`, `triangle_subdivide`, `growth_treadmill` | all three measured live on the same IC |
| `rules/tri_two.toml` | `TriLattice` | `binary_edge`, `triangle_subdivide` | comparable candidate volume (1.35M vs 0.77M), where `wn_three` has one rule supplying ~95% of the pool |

These are **substrates, not fixtures**: no rule is planted as the right
answer and the composition is fixed by which rules happen to be co-live.
The one edit is that priorities are flattened to 0, so that a measured "the
policy chose rule X" is not confounded with "the conflict resolver
preferred rule X".

`k = ∞` is refused outright by `stage2::run_case`: Stage 0a measured that
at unbounded budget every arm is identical on every rule set, so a campaign
there cannot produce a result whatever the optimizer does.

---

## 2. The campaigns

CMA-ES over `StaticBias`, F2 fitness, 40 generations, 3 rotation seeds per
member, 5 frozen hold-out seeds, `k = 16`, 384-step horizon. Two step
sizes, for the reason in §4.1.

| rule set | σ | champion | untrained (θ=0) | random | vs untrained | vs random | paired σ | exit as written |
|---|---|---|---|---|---|---|---|---|
| `wn_three` | 0.02 | 1.0000 ± 0.0000 | 1.0000 | 0.2364 ± 0.2456 | +0.0000 | **+0.7636** | **3.11** | **MET** |
| `wn_three` | **2.0** | 1.0000 ± 0.0000 | 1.0000 | 0.2364 ± 0.2456 | **+0.0000** | +0.7636 | 3.11 | MET |
| `tri_two` | 0.02 | 1.0000 ± 0.0000 | 1.0000 | 1.0000 ± 0.0000 | +0.0000 | +0.0000 | undefined | not met |
| `tri_two` | **2.0** | 0.9818 ± 0.0407 | 1.0000 | 1.0000 ± 0.0000 | **−0.0182** | −0.0182 | −0.45 | not met |

Champion allocations (softmax of the genome, which is what the policy
actually uses — raw logits are shift-invariant and not comparable across
campaigns):

| rule set | σ = 0.02 | σ = 2.0 |
|---|---|---|
| `wn_three` | `[0.313, 0.336, 0.351]` | **`[0.212, 0.777, 0.011]`** |
| `tri_two` | `[0.482, 0.518]` | **`[0.001, 0.999]`** |

**At σ = 0.02 both allocations are uniform to within 4%** — after 40
generations, on a 3- and a 2-parameter problem, CMA-ES had not moved from
its initialization. **At σ = 2.0 it moves decisively**, committing 78% and
99.9% of the budget to a single rule. That contrast is the direct
measurement behind §4.1.

**And the champions still do not beat an untrained genome.** The
`vs untrained` column is +0.0000 and −0.0182. Searching properly and
searching not at all reach the same score.

### 2.1 `tri_two` has no headroom, and that is my error

At σ = 0.02, champion and random both score exactly 1.0000, so the margin
is 0 and σ is undefined. The budget was chosen from the Stage 2 budget
probe on the grounds that `k = 16` left headroom — random scored
0.607 ± 0.083 there — but **the probe ran at a 192-step horizon and the
campaign ran at 384**, where random is already at the ceiling. Choosing a
budget at one horizon and running the campaign at another is the
methodological error, and it is worth stating plainly because it is
invisible in the result: the cell looks like a clean null and is actually a
comparison against a saturated comparator.

The σ = 2.0 row rescues the cell only partly. It has a real number
(−0.0182, 0/5 wins) and the search demonstrably worked, so "no genome beats
untrained here" is now supported. But every arm still sits at or within
0.02 of the F2 ceiling, so what the cell can distinguish is limited by §5
regardless of the optimizer. A budget re-picked **at the campaign's own
horizon** against a comparator measured below ceiling remains the right
thing to do before treating `tri_two` as a strong result in either
direction.

---

## 3. The control that decides what the margin means

`StaticBias` allocates budget across rule **groups** by softmax, so it is
group-size independent by design. `RandomSubset` samples uniformly over
**bindings**, so it allocates proportional to group size. On `wn_three`,
where `binary_edge` supplies most of the candidate pool, those are very
different policies **before any training happens**.

So "champion beats budget-matched random" does not establish that the
optimizer found anything. The control that separates the two questions is
`StaticBias` at **θ = 0** — the untrained member of the champion's own
class, scored on the same five hold-out seeds with the same paired policy
RNG:

| arm | `wn_three`, k = 16 | what it tells you |
|---|---|---|
| budget-matched random | 0.2364 ± 0.2456 | the plan's comparator |
| **untrained `StaticBias`, θ = 0** | **1.0000 ± 0.0000** | the champion's class, before training |
| trained champion | 1.0000 ± 0.0000 | after 40 generations |

**Champion − untrained = 0.0000.** The entire reported margin is
`StaticBias`-the-class beating `RandomSubset`-the-class. Training
contributed nothing measurable.

This is a finding about the **exit criterion**, not only about this
substrate: as the plan writes it, Stage 2's exit is satisfiable by a policy
class that has never been trained, whenever that class differs from
uniform-over-bindings in a way the substrate rewards. Any future campaign
comparison should report champion-vs-untrained alongside
champion-vs-random; they answer different questions and the first sweep
conflated them.

---

## 4. Was there anything to find?

Two explanations survive §3 and they call for different follow-ups: the
landscape is flat (nothing to find, a substrate result), or it has
structure the optimizer missed (a tuning problem). A direct genome sweep
separates them for a small fraction of a campaign's cost — score a spread
of genomes including the one-hot extremes that commit essentially the whole
budget to one rule, and look at the range.

Five frozen hold-out seeds, 384-step campaign horizon, F2. Genome spread is
**0.0000 … 1.0000 in every cell** — the landscape is emphatically **not
flat**; several one-hot genomes are catastrophic.

`wn_three`, `k = 4`:

| genome | F2 |
|---|---|
| budget-matched random | 1.0000 |
| uniform (θ = 0) | **1.0000** |
| all-in on rule 0 (`binary_edge`) | **0.0182** |
| suppress rule 0 | 1.0000 |
| all-in on rule 1 (`triangle_subdivide`) | 1.0000 |
| suppress rule 1 | 1.0000 |
| all-in on rule 2 (`growth_treadmill`) | **0.0000** |
| suppress rule 2 | 0.8182 |

Here the structure is all *below* the initialization: the optimum is a
broad plateau at F2 = 1.0 that **contains θ = 0**, with cliffs at the
extremes. CMA-ES did not fail to find the optimum, it started on it.
"Converged after 40 generations with a uniform genome" and "was already
optimal at generation 0" are the same run. At `k = 16` the genomes behave
the same way while the comparator moves — random drops to 0.2364, θ = 0
stays at 1.0000 — which is the §3 policy-class gap widening as the budget
tightens.

**`tri_two` does not behave that way, and it changes the conclusion:**

| genome | `k = 16` | `k = 64` |
|---|---|---|
| budget-matched random | 0.9455 | 0.7818 |
| uniform (θ = 0) | **0.9455** | **0.9091** |
| all-in on rule 0 (`binary_edge`) | 0.0000 | 0.0000 |
| **suppress rule 0** | **1.0000** | **1.0000** |
| all-in on rule 1 (`triangle_subdivide`) | 1.0000 | — |
| suppress rule 1 | 0.0000 | — |

Read on its own, this says a reachable genome scores **1.0000 where the
untrained one scores 0.9455**, so there was a +0.055 improvement available
that the campaign missed.

**That reading is wrong, and §4.2 is why.** The sweep derives its policy
seeds with a hardcoded member index that matches `wn_three`'s CMA-ES λ but
not `tri_two`'s, so its rows are on a different policy-RNG stream from the
campaign's. Under the campaign's own seeds, θ = 0 on `tri_two` scores
1.0000, not 0.9455 — and the σ = 2.0 campaign then went and found
`[0.001, 0.999]`, which is precisely the "suppress rule 0 / all-in on rule
1" genome this table flags, and scored **0.9818**, slightly *worse* than
untrained.

So the apparent `tri_two` headroom was an artifact of comparing two
policy-RNG streams, and the honest conclusion is the same one `wn_three`
gives directly: **on both substrates the untrained genome is already at the
optimum.** The sweep's genuine contribution is the part that is
seed-internal — the spread of 0.0000 … 1.0000 showing the landscape has
real structure, all of it *below* θ = 0.

### 4.1 The step size was still wrong, and fixing it did not change the verdict

`TrainingConfig`'s default `sigma` is **0.02**, and that default is wrong
for this policy class — independently of the artifact above.

`TrainingConfig`'s default `sigma` is **0.02**, and that default is wrong
for this policy class. `StaticBias`'s parameters are softmax **logits** and
the genome is initialized to all-zeros. Expressing "spend the budget on
rule 1, not rule 0" requires the logits to separate by O(1)–O(10) — the
one-hot probes above use ±6. A 40-generation campaign at σ = 0.02 explores
a neighbourhood of roughly ±0.1, over which the softmax is nearly uniform,
every population member selects almost identically, and the rank-shaped
fitness signal CMA-ES needs is flat. It has no gradient to follow and stays
where it started.

**Measured, by re-running both campaigns at σ = 2.0** (as
`stage2_<name>_k16_s2p00`; σ is carried per case and into the campaign id,
so a step-size change is a new campaign rather than a resume of the old
manifest):

| rule set | allocation at σ = 0.02 | allocation at σ = 2.0 |
|---|---|---|
| `wn_three` | `[0.313, 0.336, 0.351]` | `[0.212, 0.777, 0.011]` |
| `tri_two` | `[0.482, 0.518]` | `[0.001, 0.999]` |

At the corrected step size CMA-ES commits 78% and 99.9% of the budget to a
single rule. **The optimizer was not converging at σ = 0.02, it was
stuck** — and that is now a measurement rather than an inference.

**Fixing it does not change the verdict.** Champion-minus-untrained is
+0.0000 on `wn_three` and −0.0182 on `tri_two`; the properly-searched
champion on `tri_two` is marginally *worse* than doing nothing. A search
that explores the space properly and a search that never leaves the origin
score the same, because §5 means there is nothing above the origin to
reach.

This is the difference between a negative that is worth reporting and one
that is not: the first version of this section read the evidence as "there
was something to find and the optimizer missed it", which would have made
Stage 2 uninterpretable. Two extra campaigns settled it.

### 4.2 A caveat on the random row

The sweep derives its policy seeds with a hardcoded member index, which
matches `wn_three`'s CMA-ES λ but not `tri_two`'s. Its "budget-matched
random" row is therefore not seed-identical to the campaign's, which is why
`tri_two` random reads 0.9455 here and 1.0000 in §2. The **genome-to-genome
comparison** — what §4 rests on — is unaffected: all genomes within a cell
share one policy seed per hold-out seed.

---

## 5. Why the plateau is there: F2 saturates under a budget

The proximate cause is that F2 has a ceiling that most policies reach.

`F2 = target-regime fraction − 0.5 × penalized fraction`, penalizing
`Exploding`, `WhiteNoiseSoup` and `Frozen`. Under a budget of `k`
selections per step, **node growth is bounded by ~k per step by
construction**, so growth is linear in the horizon whatever the policy
does, and `Exploding` — which is a growth-*ratio* gate — is essentially
unreachable. The only penalized regime a budgeted run can fall into is
`Frozen`, at the opposite end.

So under a tight budget, F2 rewards "keep producing novel structure and
don't stall", and on a branching rule set nearly any non-degenerate policy
does that. This was measured independently while building F-L3 (§6):
budget-matched random sat at F2 = 1.000 at every budget rung from `k = 1`
to `k = 64` and at every horizon out to 3072 steps.

**This is the most transferable result here, and it constrains Stage 3 as
much as Stage 2.** A fitness function that saturates cannot rank policies,
and no optimizer — CMA-ES over 3 parameters or ES over ~545 — can improve
on a score that is already at its maximum. Before Stage 3 runs, it needs
either a cell where the comparator is demonstrably below ceiling, or a
fitness with headroom (F1 is not saturated; F3 rewards concentration, where
F2 rewards not-stalling).

---

## 6. F-L3 is not built, and Stage 3 is gated behind it

The plan makes F-L3 the gate for Stage 3: "an environment where the *same*
rule is beneficial applied at low-degree sites and harmful at high-degree
sites … Without this fixture, 'MLP beats StaticBias' has no environment in
which it is even possible, and Stage 3 would be uninterpretable."

Four designs were built (`crates/hg-train/rules/fl3_context_dependence.toml`,
`crates/hg-train/tests/fl3_context_dependence.rs`). None clears the bar.

**Provenance note.** Only **version 4** was measured for this document —
the budget curve and the three-fitness gate in §6.1 below. The rows for
versions 1–3 restate the design record kept in the fixture file's own
header comments; they are not runs made while writing this section, and
anything depending on their exact numbers should re-derive them rather
than cite this table.

| version | mechanism | outcome (v1-v3 from the fixture's design record, v4 measured here) |
|---|---|---|
| 1 | `sprout` + degree-guarded `overload` + no-op `filler` | `Frozen`, F2 = −0.500 on **every** arm at all six budget rungs. `filler` matched every unmarked edge and swamped the pool. Worse, had it worked, the winning move would have been "downweight filler" — a pure rule-identity decision, exactly what `StaticBias` is good at. |
| 2 | drop `filler`; burnout threshold raised above the initial degrees (measured: 5 marked nodes, max degree 4, threshold 9) | still `Frozen`. `sprout` created only *unmarked* leaves, so marked nodes were a finite, shrinking population and the run had to stall — it did, at 53 nodes. |
| 3 | `sprout` attaches the new node to both endpoints and marks it, so sites branch | dynamics at last, but budget-matched random pinned at F2 = **1.000** at every rung and every horizon to 3072 steps. No headroom. `degree_high` held the graph at 390 nodes against random's 1816 — the signal exists, it just isn't rewarded. |
| 4 | burnout **deletes** the hub and its edges instead of relabelling it | random still 1.000 at every rung; both degree arms strictly worse (`degree_low` 0.244, `degree_high` 0.551 at `k = 16`). |

### 6.1 It is not F2's fault — measured

The natural diagnosis after version 4 was §5: F2 saturates, so build the
fixture against a fitness with headroom instead. `fitness::evaluate`
computes all three from one rollout, so testing that costs one sweep
(`fl3_gate_under_each_fitness`, `[causal]` **on** so F3 reads real
constructor counts rather than a column of zeros).

Version 4, `WhiteNoise`, 384 steps, 12 seeds:

| k | arm | F1 | F2 | F3 | constructors |
|---|---|---|---|---|---|
| 1 | **random** | **0.5121** | **1.0000** | **26.41** | **31.3** |
| 1 | degree_low | 0.3120 | 0.5769 | 14.32 | 14.5 |
| 1 | degree_high | 0.2874 | 0.6795 | 9.16 | 9.3 |
| 16 | **random** | **0.5067** | **1.0000** | 14.90 | 16.3 |
| 16 | degree_low | 0.1825 | 0.2436 | **15.42** | 15.8 |
| 16 | degree_high | 0.2434 | 0.5513 | 10.30 | 10.6 |
| 64 | **random** | **0.4736** | **1.0000** | **12.45** | 14.1 |
| 64 | degree_low | 0.4075 | 0.8846 | 12.41 | 12.6 |
| 64 | degree_high | 0.1287 | 0.2051 | 11.76 | 11.9 |

**Budget-matched random wins on all three fitnesses at all six budget
rungs.** Best heuristic gain over random: 0.00σ almost everywhere, peaking
at 0.15σ. The gate fails, and it fails for a reason the F2 story does not
cover:

**F1 and F3 are not saturated here.** F1 runs 0.47–0.52 for random against
0.13–0.41 for the heuristics; F3 runs 12–26 against 9–15. There is plenty
of headroom in both. Random is not winning because the metric is blind —
it is winning because it is genuinely better.

### 6.2 The transferable finding: this metric family rewards variety, and uniform selection maximizes variety

Interestingness counts novel type production; F2's target regimes reward
sustained generation; F3 counts confirmed constructors. All three are
**variety-rewarding**, and a policy that concentrates selection anywhere —
on low-degree sites, on high-degree sites, or on one rule — produces less
varied structure than one that spreads it. Uniform selection is not a
baseline that happens to be hard to beat on these substrates; it is close
to the arg-max of the whole metric family.

That reframes Stage 2's negative as well. "Rule identity carries no
exploitable signal" is the wrong reading; the better one is **the objective
prefers the uniform policy, and both `StaticBias` at θ = 0 and
`RandomSubset` are approximately uniform in the ways these metrics
measure.** A learned selection policy can only win against a target that
rewards *specific* structure rather than *varied* structure — and no
fitness in `docs/fitness_formulas.md` currently does.

That is a constraint on the learned-selection programme as a whole, not on
this fixture, and it is the most useful thing C5 produced.

### 6.3 Stage 3 is not run

That is the gate working as designed, not a shortfall: without F-L3, "MLP
beats `StaticBias`" has no environment in which it is possible, and a
Stage-3 result on real rule sets would be uninterpretable either way.

What would unblock it is **a fitness that rewards a specified target
structure** — hitting a named motif, matching a target degree distribution,
maximizing yield of one constructor class rather than counting distinct
ones. Version 4's mechanism can stay as it is; the deletion polarity is
right and hub placement is genuinely destructive. It is the objective, not
the substrate, that has to change, and that is a `docs/fitness_formulas.md`
change with its own pre-registration rather than a fixture tweak.

---

## 7. What this leaves

Recorded as measured, with the follow-ups they imply.

1. **Stage 2's exit criterion, as written, is satisfiable without
   learning.** Report champion-vs-untrained alongside champion-vs-random.
   Cheap, and it should be standard for Stage 3 too.
2. **The default step size is wrong for logit parameters** (§4.1). σ = 0.02
   leaves CMA-ES at its initialization for 40 generations; σ = 2.0 reaches
   99.9%-committed allocations. It did not change the verdict here, but it
   would silently invalidate any campaign where the verdict *did* depend on
   search. `MlpScorer` has ~545 parameters on a different scale again, so
   **Stage 3 needs its own step-size justification rather than inheriting
   `TrainingConfig`'s default** — and the cheap way to get one is the same
   genome sweep used here, run before the campaign rather than after.
3. **Stage 2's substantive answer is negative and it is now robust.** On
   both composed substrates the untrained genome is already optimal, at two
   step sizes three orders of magnitude apart, with the search demonstrably
   working at the larger one. Per the plan's pre-commitment, that "sharpens
   Stage 3's question rather than killing it": whatever structure exists is
   context-dependent or nonexistent, and §5 says why F2 cannot see it
   either way.
4. **`StaticBias` vs `RandomSubset` is a real and large effect** —
   1.0000 vs 0.2364 at `k = 16` on `wn_three`. Group-size-independent
   allocation is doing genuine work; it is simply not *learned* work. That
   is a result about the policy class and is worth stating positively.
5. **F2 saturates under a budget** (§5), which caps what any Stage 2 or
   Stage 3 campaign on these substrates can show. The next campaign should
   be sized against a comparator measured below ceiling **at the horizon it
   will actually run at** — §2.1 is what happens otherwise.
6. **The `tri_two` cell at σ = 0.02 was uninformative** — champion and
   random both at exactly 1.0000, because its budget was chosen for
   headroom measured at a 192-step horizon and the campaign ran at 384
   (§2.1). The σ = 2.0 re-run gives it a real, if negative, answer.
7. **F-L3 needs a fitness with headroom, not a better mechanism.** F1 is
   unsaturated on these substrates (0.183–0.512 across the F-L3 arms) and
   F3 rewards concentration where F2 rewards not-stalling. Version 4's
   deletion mechanism plus F1 or F3 is the next thing to try.
