# Stage R — the re-analysis passes

Phase D3 of `hypersim_nn_experiments_plan_v2.md`, §8. Produced by
`crates/hg-train/tests/d3_stage_r_reanalysis.rs` and
`crates/hg-train/tests/d3_r2_emergence_headroom.rs`.

> **R-1 falsifies the central claim of v2.** §5.1's generalization —
> "selection-learning is unstateable against diversity objectives, because
> the variance-maximizing policy is the trivial one" — predicts that every
> non-uniform heuristic should be *worse* than budget-matched random. It is
> not. **13 credible comparisons show a hand heuristic beating uniform
> selection**, up to +68.76σ, across three of five rule sets. Plan v2 risk 4
> is realized, and the lead it names is real.

---

## 0. A correction to §8's premise, stated before the numbers

§8 opens: *"These consume no new rollouts and should run first."*

That is true of the **analysis** and false of the **data**. Stage 0a wrote
nothing. `runs/cognition/` is empty, the C0–C5 sweep printed its table to
stdout, and `docs/campaign_output.md` §1 makes that a design property —
"rollouts are never citable results … leave no artifact". There was no
stored dataset to re-sign.

So R-1 **re-ran the sweep**. That is a cost, not a compromise, and it makes
the result stronger than the plan asked for: the signed table is measured
under the v2 instrument — signed classifier, engagement counters, and the
fixed `RandomSubset` — rather than reconstructed from a pre-fix record.
Plan v2 §12 requires exactly this of any figure carried across bug 1, and
R-1 now satisfies it for the whole Stage 0a battery.

The same applies to R-2, which needed a fresh measurement pass for the same
reason.

---

## 1. R-1 — the signed Stage 0a table

5 rule sets × 5 budgets × 2 heuristics × 3 metrics = **150 comparisons**,
20 seeds each. Sign convention: `heuristic − random`, so **positive means
the heuristic beat budget-matched random**.

### 1.1 Two filters, applied before anything was counted

**Rule 2** (`docs/instrument_registry.md` §2.6): a cell reporting
`|σ| > 100` on 20 seeds is an instrument fault pending proof — a huge
effect and a collapsed denominator look identical in the output, and the
collapsed denominator is overwhelmingly more likely. It fired on **4 of
150**, all on `binary_edge`'s `interestingness`:

| cell | reported | random's sd |
|---|---|---|
| k=4 `degree_high` | +567.66σ | 3.71e−4 |
| k=4 `degree_low` | +503.47σ | 3.71e−4 |
| k=64 `degree_high` | **+29 197.94σ** | 1.35e−5 |
| k=64 `degree_low` | +1 488.70σ | 1.35e−5 |

All four are excluded. Their **deltas are still real** (+0.21, +0.19,
+0.40, +0.02) — what is not real is the σ, and a σ is what the exit
criterion is written in. This is the guard working on live data, applied
mechanically rather than by reading the table and deciding.

**The degree-regularity check.** `degree_high` and `degree_low` returned
*bit-identical* numbers on **18 comparisons**. Two opposite heuristics
agreeing exactly means the graph is degree-**regular**, so neither is
selecting on degree at all: both collapse to the same canonical tie-break
order. Affected: all of `glider` and `fl1_two_rule` (`GliderRing`, a
regular ring), `growth_treadmill` k∈{1,4}, and `binary_edge` k∈{4,64}
(`PureCrystal`).

Where this fires, the effect is real but it is **"deterministic non-uniform
selection"**, not "prefer high degree", and the write-up has to say so.
`triangle_subdivide` is the one rule set where the two arms diverge sharply
— `TriLattice` becomes irregular as it subdivides — which is what makes it
the decisive case below.

### 1.2 The verdict

| | count |
|---|---|
| comparisons | 150 (**146** credible after Rule 2) |
| heuristic **beats** random by > 2σ | **13** |
| heuristic **loses** to random by > 2σ | 19 |
| within ±2σ, or no denominator | 114 |

### 1.3 The 13 positive cells

| rule set | k | arm | metric | Δ | σ |
|---|---|---|---|---|---|
| `triangle_subdivide` | 1 | `degree_high` | interestingness | +0.0437 | +2.77 |
| `triangle_subdivide` | 4 | `degree_high` | interestingness | +0.2280 | +25.21 |
| `triangle_subdivide` | 16 | `degree_high` | **f2** | +0.8500 | +3.62 |
| `triangle_subdivide` | 16 | `degree_high` | interestingness | +0.3562 | +4.66 |
| `triangle_subdivide` | 16 | `degree_low` | interestingness | +0.2144 | +2.81 |
| `triangle_subdivide` | 64 | `degree_high` | interestingness | +0.3798 | **+68.76** |
| `triangle_subdivide` | 64 | `degree_low` | interestingness | +0.1120 | +20.27 |
| `binary_edge` | 16 | `degree_high` | interestingness | +0.1259 | +14.82 |
| `binary_edge` | 16 | `degree_low` | interestingness | +0.2358 | +27.75 |
| `fl1_two_rule` | 16 | `degree_high`≡`low` | interestingness | +0.1101 | +12.23 |
| `fl1_two_rule` | 64 | `degree_high` | interestingness | +0.3376 | +32.16 |
| `fl1_two_rule` | 64 | `degree_low` | interestingness | +0.2874 | +27.38 |

### 1.4 What this does to §5.1

**The strong form is false.** §5.1 argues that uniform selection maximizes
variety, every metric rewards variety, therefore uniform is the optimum *of
the objective* and selection-learning is unstateable against any of them.
If that held, no non-uniform heuristic could beat random on any metric. One
does, repeatedly, on three rule sets, at every finite budget on
`triangle_subdivide`.

**`triangle_subdivide` is the decisive case** because it is the one where
the two heuristics genuinely differ — the lattice becomes degree-irregular
under subdivision, so `degree_high` and `degree_low` are really selecting
on degree. And they separate hard and in *opposite directions*:

| k | `degree_high` on interestingness | `degree_low` on interestingness |
|---|---|---|
| 1 | **+2.77σ** | −33.10σ |
| 4 | **+25.21σ** | −37.53σ |
| 16 | **+4.66σ** | +2.81σ |
| 64 | **+68.76σ** | +20.27σ |

That is not a tie-break artifact and it is not noise. Preferring
high-degree bindings raises interestingness substantially; preferring
low-degree ones destroys it at tight budgets. **There is exploitable
structure in per-candidate context on this substrate**, which is the thing
C-2's negative was taken to have ruled out.

### 1.5 The sharper pattern: the metrics disagree with each other

Every positive cell but one is on **`interestingness`** (F1). Every large
negative is on **`active_persistent`**. On `triangle_subdivide` the two run
in opposite directions in the same cell:

| k | `degree_high` interestingness | `degree_high` active_persistent |
|---|---|---|
| 1 | +2.77σ | **−17.14σ** |
| 4 | +25.21σ | **−23.00σ** |
| 16 | +4.66σ | **−28.69σ** |
| 64 | +68.76σ | **−17.37σ** |

So the same heuristic, in the same cell, is strongly better on one
gate-relevant metric and strongly worse on another. §5.1 treats "every
metric in `fitness_formulas.md` rewards varied structure" as a single
property of the objective family. **It is not one property.** F1's
interestingness weighting and the active-persistence count are pulling
against each other, and which one a stage optimizes decides whether uniform
wins.

That reframes C-2 rather than overturning it. C-2 measured `StaticBias`
under **F2**, and F2 is a regime-occupancy classification — closer to the
active-persistence side, where uniform does win. The 13 positives are
almost all on F1, which Stage 2 never optimized.

### 1.6 What R-1 does *not* show

- **Not a learning result.** These are hand heuristics, not champions. That
  a fixed rule beats uniform says exploitable signal exists; it does not
  say an optimizer can find it, and `docs/campaign_output.md` §4.2's
  three-arm protocol still governs any claim that one did.
- **Not a `StaticBias` result.** `degree_high` selects on per-candidate
  context. `StaticBias` allocates across *rules* and cannot express "prefer
  high-degree bindings" at all. C-2 stands exactly as measured; what
  changes is the inference drawn from it.
- **Not transferable to `glider`.** Its whole block is a tie-break effect
  on a regular ring, and it is separately excluded from separation analysis
  as an isomorphic trajectory (`docs/policy_config.md` §4B.6).

### 1.7 The consequence for the programme

Plan v2 §9 gates Stage 3′ (`MlpScorer`) behind Stage 2′ finding a
non-diversity objective. R-1 says that gate may be aimed at the wrong
thing: **F1 already discriminates**, on a substrate already in the battery,
in favour of a policy class that scores per-candidate context — which is
exactly what `MlpScorer` is and what `StaticBias` is not.

That is a Stage 3′ configuration derivable today: `triangle_subdivide`,
`k = 16` or `64`, fitness F1, `MlpScorer`, three-arm protocol. It does not
require F4 or F5 to pass anything. It should be checked against P-1…P-4
before it is believed, and P-4 is the one to watch — `degree_high` reaching
`f2 = 1.0000` at k=16 and 64 says F2 saturates there, though F1 does not.

### 1.8 The exit cells, re-read with their signs

v1 reported these as the Stage 0a exit. Seven of twelve are the heuristic
**losing**:

| rule set | k | signed σ | reading |
|---|---|---|---|
| `growth_treadmill` | 1 | −3.72 | heuristic loses |
| `triangle_subdivide` | 1 | −33.10 | heuristic loses |
| `triangle_subdivide` | 4 | −37.53 | heuristic loses |
| `triangle_subdivide` | 16 | −28.69 | heuristic loses |
| `triangle_subdivide` | 64 | **+68.76** | heuristic wins |
| `binary_edge` | 4 | +567.66 | **excluded, Rule 2** |
| `binary_edge` | 16 | +27.75 | heuristic wins |
| `binary_edge` | 64 | +29 197.94 | **excluded, Rule 2** |
| `fl1_two_rule` | 1 | −9.80 | heuristic loses |
| `fl1_two_rule` | 4 | −15.41 | heuristic loses |
| `fl1_two_rule` | 16 | −14.50 | heuristic loses |
| `fl1_two_rule` | 64 | +32.16 | heuristic wins |

The C-1 conclusion is untouched and re-confirmed: **`k = ∞` is exactly
`+0.0000` on every metric of every rule set**, and every one of those cells
classifies `NO DIFFERENCE`. Selection has causal power only under scarcity.

`meets_exit` remains a magnitude test on purpose — Stage 0a asks whether
selection has causal power, for which a heuristic reliably *worse* than
random is just as decisive. What changed is that the sign is now carried
alongside, so the two questions can no longer be conflated in the reporting.

---

## 2. R-2 — emergence headroom vs micro determinism

`wn_three` / `WhiteNoise`, 384 steps, 5 seeds, `[causal]` on. 4 budgets ×
3 arms = 12 cells, **all 12 with the EI calculator engaged** (25–50
intervals each).

Hypothesis [Inferred, plan v2 §8]: `EI = (determinism − degeneracy) ·
log2(M)`, and causal emergence needs coarse-graining to reduce degeneracy
faster than determinism. Budget-constrained allocation already sharpens
micro causation — fewer, more disjoint, less-interfering rewrites per step
— removing the micro degeneracy the coarse-graining was compensating for.
**Emergence headroom is a function of micro slop.**

### 2.1 The cells

| k | arm | `ei_micro` | `ei_macro` | `ce_structural` |
|---|---|---|---|---|
| 1 | random | 6.2006 | 2.2730 | 0.3687 |
| 1 | untrained | 6.4844 | 2.2155 | 0.3421 |
| 1 | champion | 6.5493 | 2.3544 | 0.3605 |
| 4 | random | 6.1449 | 2.2475 | 0.3675 |
| 4 | untrained | 6.5898 | 2.3150 | 0.3519 |
| 4 | champion | 6.8471 | 2.3654 | 0.3465 |
| 16 | random | 5.1217 | 2.0255 | 0.3966 |
| 16 | untrained | **7.1632** | 2.3680 | **0.3307** |
| 16 | champion | 6.5642 | 2.3525 | 0.3595 |
| 64 | random | 5.0011 | 1.9212 | 0.3826 |
| 64 | untrained | 7.1347 | 2.1892 | **0.3066** |
| 64 | champion | 5.9236 | 2.0722 | 0.3505 |

### 2.2 The control — risk 6, and it passes

Risk 6: budget moves `ei_micro` and `ce_structural` together, so a pooled
anti-correlation is consistent with two effects of one cause. The
designated control is to correlate **across policy arms at fixed budget**,
where the budget is constant and only the allocation differs.

| budget rung | `r(ei_micro, ce_structural)` |
|---|---|
| k = 1 | −0.6087 |
| k = 4 | **−0.9928** |
| k = 16 | **−0.9870** |
| k = 64 | **−0.9999** |

**4 of 4 negative, three of them within 0.02 of −1.** The anti-correlation
survives holding budget fixed, so it is a relationship between
allocation-driven micro determinism and causal emergence, not the budget
moving both at once. **Risk 6 is controlled.**

Pooled over all 12 cells: `r = −0.8582`.

### 2.3 The confounded view, and its one exception

Across budget rungs, one arm at a time — the *confounded* direction,
reported as corroboration rather than as a second test:

| arm | `r` | `ce_structural` span | `ei_micro` span |
|---|---|---|---|
| random | −0.8715 | 0.0291 | 1.1995 |
| untrained | −0.7918 | 0.0355 | 0.6788 |
| champion | **+0.0153** | **0.0140** | 0.9235 |

The champion's `+0.0153` is not a counterexample. Its `ce_structural`
varies by **0.0140 across four points** — less than half the spread of
either other arm — so there is almost nothing there to correlate against. A
Pearson `r` over a near-flat series is arithmetically defined and carries
essentially no information, which is the null-spread hazard `f4_null_sd`
and the V-track's arm D are both about, one level up. The test now prints a
`LOW POWER` marker for exactly this case rather than leaving a near-zero
`r` to be read as "no relationship".

### 2.4 The result

**Interventions that increase micro determinism reduce causal emergence**,
measured at fixed budget across allocation schemes, on 4 of 4 rungs.

The framing matters and is easy to get wrong: this is **allocation under
scarcity, not policies**. Champion vs untrained on the Stage 5 headline was
+0.26σ. Nothing here is a learning result, and the mechanism is about how a
budget is spread across rules rather than about anything a policy learned.

It is also a contribution with nothing to do with learning at all. Anything
reading CE off a shaped run needs the unshaped run beside it, because a
change that improves every headline metric — deeper lattice, more tasks,
higher composition rate, higher macro EI — can simultaneously reduce the
macro scale's explanatory advantage. §3.3 is the standing form of that
caution and R-2 is now its evidence.

**Caveats.** One rule set, one IC, 5 seeds, one coarse-graining strategy
(`ce_structural`). The within-rung correlations rest on three points each —
enough to establish sign, not magnitude. `ce_energetic` and `ce_functional`
were not measured and the claim is not made about them.

---

## 3. R-3 — Stages 4 and 5 re-headlined

Editorial, not computational: both stages already carried untrained arms
(`docs/cognition_stage45_findings.md`), so the numbers do not move. What
moves is which number is the headline.

### 3.1 Stage 4 — task lattice

**Headline (was):** shaped dynamics raise composition rate +2.44σ, depth
+3.83σ, task count +3.94σ over budget-matched random.

**Headline (now): an allocation-class effect, with a mechanism.**
Group-uniform allocation across three rules produces ~8× the tasks of
binding-uniform sampling on a pool `binary_edge` dominates — 770.2 tasks vs
88.1, composition rate 0.2324 vs 0.1005. `binary_edge` supplies most of the
candidate pool on `wn_three`, so binding-uniform sampling spends almost
everything there and produces a narrow range of transitions; group-uniform
allocation spends a third of the budget on each rule and produces a much
wider range.

**Learning effect: null.** Champion − untrained is **+0.28σ**. Forty
generations of CMA-ES on top of the allocation class contribute nothing
measurable.

### 3.2 Stage 5 — effective information

**Headline (now): the class raises macro EI and *lowers* causal emergence.**
`ei_macro` 2.0005 → 2.3520 (untrained, +2.46σ); `ce_structural` 0.3925 →
0.3290 (**−2.39σ**). Both scales gain information and micro gains more, so
the coarse-graining explains *less* of the dynamics than it did under
uniform selection.

**Learning effect: null.** Champion − untrained is **+0.26σ**.

### 3.3 The standing caution

A policy that improves every headline metric — deeper lattice, more tasks,
higher composition rate, higher macro EI — can simultaneously **reduce** the
macro scale's explanatory advantage. Anything reading CE off a shaped run
needs the unshaped run beside it. This is now a standing analysis (R-2)
rather than a one-off observation.

### 3.4 Three stages, one confound

Stage 2's `+0.7636` at 3.11σ, Stage 4's `+2.44σ`, and Stage 5's `+2.84σ`
were all reported against budget-matched random and all describe the same
thing: `StaticBias` allocates group-uniformly, `RandomSubset` allocates
binding-uniformly, and on a pool one rule dominates those are very
different dynamics **before a single gradient step**. Against the untrained
arm the three margins are `+0.0000`, `+0.28σ`, `+0.26σ`.

Class effects are real findings and are written up as such here. They are
not results about learning, and the report now says which it is
(`docs/campaign_output.md` §4.2).

---

## 3A. Stage I — the retrospective demotion pass

Plan v2 §6.3 and §9. Produced by
`crates/hg-train/tests/d3_stage_i_demotion.rs`, run against the
configurations the published numbers were actually measured under.

### Result: no demotions.

| number | counters | status |
|---|---|---|
| Stages 4 & 5, all three arms | `ei = 10`, `tasks = 93 / 771 / 641`, `chains = 112 / 2083 / 2575`, `ctors = 85 / 418 / 254` | **stands** |
| Stage 0a, all rule sets | `f1_windows_scored = 7 / 7 / 3 / 3` | **stands** |

Stage 0a's EI, task-lattice and constructor counters all read zero, and
that is **not** a demotion: Stage 0a runs with `[causal]` off by design and
published no number of any of those kinds. An instrument that was never
*asked* to run is a different state from one that was asked and failed.

The pass carries its own firing test
(`the_demotion_pass_can_actually_detect_a_dead_instrument`), because "no
demotions" and "the check does nothing" would otherwise be
indistinguishable.

### 3A.1 What Stage I found that it was not looking for

**F4 is unmeasurable on every fixture the programme currently owns.**
`f4_pairs` reads 2–10 against `PI_MIN_PAIRS = 50`:

| fixture | steps | window | windows | `f4_pairs` |
|---|---|---|---|---|
| Stage 4/5 `wn_three` | 384 | 24 | 16 | 10 |
| Stage 0a `glider` | 512 | 64 | 8 | 6 |
| Stage 0a `growth_treadmill` | 512 | 64 | 8 | 6 |
| Stage 0a `triangle_subdivide` | 96 | 24 | 4 | 2 |
| Stage 0a `binary_edge` | 96 | 24 | 4 | 2 |

That is the guard working — a mutual information estimated from ten pairs
is not an estimate — but it blocks Stage 2′ from testing F4's stateability
until horizons and windows are chosen that clear
`horizon / window ≥ 56` (at `B = 5`), roughly 3.5× the Stage 4/5 fixture.

The two ways to get there are **not** equivalent. A longer horizon keeps
each symbol meaning what it meant; a shorter window changes it, and Stage
1's own sweep measured that input coupling falls from 0.93 at `w = 32` to
0.68 at `w = 4`. Whichever is chosen belongs in F4's measurement definition
rather than being picked to clear the bar. Recorded in
`docs/instrument_registry.md` §5.1.

> **Superseded by Stage 2′ (D5).** The observation above is correct;
> the *diagnosis* is not. F4 was unmeasurable for a second, prior reason —
> its fixed `⌊4 · jsd⌋` grid collapsed every run to **one symbol**, because
> the JSD lives in `[0.002, 0.085]` and the grid spans `[0, 1]`. Raising
> `f4_pairs` past 50 therefore does not make F4 informative; it makes F4
> report a *measured* zero. The `f4_pairs` guard was reading `unmeasured`
> for the wrong reason, and following the prescription above would have
> published a false null.
>
> The horizon/window recommendation is also reversed. Measured, the long
> horizon costs **28×** for an identical drift confound, and the
> comparability it was supposed to preserve does not exist — F4 had never
> been measurable at window 24. Stage 2′ uses 384 steps at window 6.
>
> `docs/cognition_stage2_prime_findings.md` §1.1;
> `docs/instrument_registry.md` §5.1.1.

---

## 4. Provenance

Every number above is measured post-bug-1, under the v2 instrument. R-1's
sweep and R-2's pass were both run fresh in Phase D3 rather than carried
over, which discharges plan v2 §12's requirement for the Stage 0a battery.

`glider`'s Stage 0a block is reported for completeness and is **not
citable** as a substrate result: its trajectory is a sequence of isomorphic
graphs, so every isomorphism-invariant metric — which is all of them — is
constant by construction (`docs/policy_config.md` §4B.6).
