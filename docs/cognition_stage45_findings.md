# Cognition Stages 4 & 5 — task lattice and effective information

Phase C5 of `hypersim_nn_experiments_plan_v1.md`, Stages 4 and 5.
Measurement passes: no new training.

Produced by `crates/hg-train/tests/c5_stage45.rs`. Substrate `wn_three`,
`k = 16`, 384 steps, 10 seeds per arm, `[causal]` on.

> **Both pre-registered directions resolve, and neither is about learning.**
> Shaped dynamics beat random on task-lattice composition (+2.44σ) and on
> macro EI (+2.84σ). But the trained champion and an **untrained** genome
> are indistinguishable (+0.28σ, +0.26σ), so what is being measured is a
> **policy class**, not a learned policy. And the causal-emergence
> coefficient moves the *opposite* way to macro EI (§3.2), which is the
> most interesting number here.

---

## 1. Three arms, and why the third one is not optional

| arm | policy |
|---|---|
| `random` | budget-matched `RandomSubset` — the plan's comparator |
| `untrained` | `StaticBias`, θ = 0 |
| `champion` | `StaticBias`, the σ = 2.0 `wn_three` champion (`0x405a3aa651eeac0f`) |

Stage 2 established that its champion is not better than an untrained
`StaticBias` (`docs/cognition_stage2_findings.md` §3). So a
champion-vs-random contrast here is a contrast between **allocation
schemes** — `StaticBias` allocates budget across rule *groups*,
`RandomSubset` samples uniformly over *bindings* — and not between learned
and unlearned dynamics.

Stages 4 and 5 ask about *shaped* dynamics rather than *learned* ones, so
that contrast is still the right one to run. But reporting it as learned
shaping would be wrong, and the untrained arm is what makes the difference
visible rather than assumed.

---

## 2. Stage 4 — task lattice

**Hypothesis.** Policy-shaped dynamics populate the task lattice with
composed tasks at a higher rate than random dynamics.

**Composition rate** is the plan's definition made precise: over the set of
distinct (input, output) tasks, for every composable chain A→B, B→C, does
the composed task A→C also appear? Rate = realized / composable. Self-loops
(A == C) are excluded — A→A is not evidence of composition. Undefined
rather than zero when no chains exist.

| arm | lattice depth | tasks | composition rate | chains |
|---|---|---|---|---|
| random | 27.00 ± 5.62 | 88.1 ± 19.7 | 0.1005 ± 0.0482 | 132 |
| untrained | 54.80 ± 6.41 | 770.2 ± 38.3 | 0.2324 ± 0.0134 | 2212 |
| **champion** | **63.50 ± 12.26** | 732.3 ± 230.7 | **0.2464 ± 0.0695** | 2574 |

| contrast | depth | task count | composition rate |
|---|---|---|---|
| champion vs random | **+3.83σ** | **+3.94σ** | **+2.44σ** |
| untrained vs random | +4.61σ | +22.41σ | +3.73σ |
| **champion vs untrained** | — | — | **+0.28σ** |

**Stage 4's exit: the pre-registered direction is confirmed.** Composition
rate more than doubles under shaped dynamics (0.10 → 0.25), the lattice is
more than twice as deep, and it holds eight to nine times as many tasks.

**And it is entirely the policy class.** Champion vs untrained is +0.28σ.
The `StaticBias` allocation scheme is what populates the lattice; forty
generations of CMA-ES on top of it contribute nothing measurable.

The mechanism is not mysterious. `binary_edge` supplies most of the
candidate pool on `wn_three`, so binding-uniform sampling spends almost all
of its budget there and produces a narrow range of transitions.
Group-uniform allocation spends a third of the budget on each rule and
produces a much wider range — more distinct attribute transitions, hence
more tasks, more chains, and more opportunities for a chain's composition
to also be witnessed.

---

## 3. Stage 5 — effective information

**Hypothesis.** Shaped dynamics increase the macro-over-micro EI advantage.
**Pre-registered direction:** shaped > random at the attribute level.

| arm | ei_micro | ei_macro | ce_structural |
|---|---|---|---|
| random | 5.1027 ± 0.1816 | 2.0005 ± 0.1388 | **0.3925 ± 0.0313** |
| untrained | 7.1506 ± 0.1067 | 2.3520 ± 0.1467 | 0.3290 ± 0.0208 |
| **champion** | 6.7799 ± 0.5191 | **2.3880 ± 0.1337** | 0.3529 ± 0.0190 |

| contrast | ei_macro | ce_structural |
|---|---|---|
| champion vs random | **+2.84σ** | −1.53σ |
| untrained vs random | +2.46σ | **−2.39σ** |
| champion vs untrained | +0.26σ | — |

### 3.1 The pre-registered direction holds at the attribute level

`ei_macro` rises from 2.00 to 2.39 under shaping, at +2.84σ. Stage 5's exit
asked only that the comparison exist with tight enough seed variance to be
directional; it does, and it points the way the plan pre-registered.

### 3.2 But causal emergence moves the other way

`ce_structural` — the macro-over-micro advantage, which is the quantity the
causal-emergence programme actually cares about — **falls** under shaping,
0.3925 → 0.3290 (untrained, −2.39σ) and → 0.3529 (champion, −1.53σ).

Both scales gain information; micro gains more. `ei_micro` rises 5.10 →
6.78/7.15, a larger absolute and relative move than `ei_macro`'s 2.00 →
2.39. The coarse-graining therefore explains *less* of the dynamics than it
did under random selection, even though the macro level itself is more
informative.

**This is the sharp result of the pair, and it is a genuine tension with
the pre-registration.** "Shaped dynamics increase the macro-over-micro EI
advantage" is **false** on this substrate, while "shaped > random at the
attribute level" is true. The plan wrote the hypothesis one way and the
exit criterion the other, and the two come apart here.

The reading that fits both numbers: group-uniform allocation makes the
substrate do more different things (§2 — eight times the tasks), which adds
information at every scale. The WL coarse-graining is tuned to structure it
was already seeing; the extra variety lands disproportionately below it.
Shaping buys richness, not abstraction.

That is worth stating as a caution for the causal-emergence programme: a
policy that improves every headline metric — deeper lattice, more tasks,
higher composition rate, higher macro EI — can simultaneously *reduce* the
macro scale's explanatory advantage. Anything reading CE off a shaped run
needs the unshaped run beside it.

---

## 4. Two instrument traps, both of which return plausible zeros

Recorded because they are the same failure mode twice and neither announces
itself.

**`[causal]` is off in the Stage 0 diagnostic config.** F3 reads
`n_confirmed_constructors`, which comes from constructor detection; with
causal disabled, `RolloutMetrics::constructors` is empty and F3 is
identically 0.0 for every arm. An F3 comparison run that way is a table of
zeros that reads as "no policy affects constructor yield". `Fixture` now
has `sim_config_causal` for anything scoring F3, Stage 4 or Stage 5.

**`ei_interval_steps` defaults to 1000, which is longer than most fixture
horizons here.** At 384 steps the interval never elapses, no EI is ever
emitted, every row's `ei_micro`/`ei_macro`/`ce_structural` is `None`, and
the averages come through as `NaN`. The first Stage 5 run produced exactly
that — three arms of `NaN` that would read as "shaping has no effect on EI".
The interval is now scaled to the horizon (≈ three emissions per run).

`ei_minimum_transitions` is deliberately **left at its default of 1000**.
It is a certainty threshold, not a tuning knob — the certainty table
requires ≥ 1000 observed transitions before an EI claim counts as
Empirical — and lowering it to force emissions would be manufacturing the
measurement rather than making it.

---

## 5. Stage 6 is not run

Stage 6 re-runs **Stage 3's best configuration** with F3 fitness and
compares champion phenotypes across fitness functions. Stage 3 is gated
behind F-L3, which fails (`docs/cognition_stage2_findings.md` §6), so
Stage 6 has no configuration to re-run.

The part of Stage 6 that could be salvaged without Stage 3 — comparing F2-
and F3-optimized `StaticBias` champions — is not worth the campaign time on
the evidence here: Stage 2 showed the untrained genome is already optimal
under F2, and the F-L3 sweep showed random beats every heuristic under F3
as well (§6.1 there). Two campaigns whose champions are both predicted to
be uniform would not produce a phenotype contrast.

What would make Stage 6 meaningful is the same thing that would unblock
Stage 3: **a fitness that rewards a specified target structure** rather
than counting varied structure. That is a `docs/fitness_formulas.md` change
with its own pre-registration.
