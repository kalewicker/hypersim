# Instrument registry — engagement counters and predicted ranges

Governs how a number earns the right to be reported. Companion to
`docs/fitness_formulas.md` (what gets optimized), `docs/campaign_output.md`
(what gets written) and `docs/policy_config.md` (what gets configured).

Introduced by `hypersim_nn_experiments_plan_v2.md` §6, on the strength of
the single most transferable result of the C0–C5 session: **eleven
instruments returned plausible numbers, and exactly one of them was caught
by a test that existed at the time.** Every other one was caught by a human
noticing that the mechanism could not have produced the value.

This file exists so that noticing is not the mechanism.

---

## 1. The two rules

**Rule 1 — engagement.** Every measurement carries a counter of how many
times the instrument actually did work. A zero counter emits
`INSTRUMENT_NOT_ENGAGED`, and the value is written to the report as
`unmeasured`. It is never written as `0.0`.

> `unmeasured` and `0.0` are different report states and the schema keeps
> them different.

This is the direct generalization of the guard-engagement principle the
V-track arrived at independently: a guard that never rejects cannot support
a mechanism verdict (`docs/validation_v8_m4_3_guard_audit.md`), so an
instrument that never engages cannot support a measurement. Six of the
eleven C0–C5 bugs — 2, 4, 6, 7, 8, 10 — are one signature: **an instrument
present but never engaged, reporting a value anyway.** Rule 1 is the single
change that catches all six.

**Rule 2 — predicted range.** Every measurement carries a range the
mechanism says the value must lie in if the instrument is working, written
*before* the value is read.

> **Plausibility is not evidence.** A number is admitted only against a
> mechanism-derived prediction of its sign or order of magnitude.

A value outside its predicted range is an **instrument fault until proven
otherwise**, not a finding. Values that are arithmetically impossible fail
this automatically: perturbed pairs diverging more than independent ones
(bug 5), a `2.9e16σ` effect size (bug 4), a separation ratio above its own
stated upper bound.

Rule 2 does not catch the five logic-error bugs (1, 3, 5, 9, 11) as
reliably as Rule 1 catches the engagement class — bug 1 produced numbers
that were in range and merely wrong. It caught three of the five, which is
three more than the test suite did.

### 1.1 Ranges are per-mechanism, not per-observation

A predicted range is derived from what the quantity *is*, never from what
it was last time. `F4 ≤ log₂ PI_ALPHABET` because mutual information of a
`Q`-symbol variable with itself is bounded by `log₂ Q`; that bound holds on
a substrate nobody has run yet. A range fitted to observed values would
ratify whatever the instrument currently does, including its bugs, which is
the failure this file is built against.

Where no mechanism-derived bound exists, the registry says so and the
column carries no range. An honest blank is not the same as a range of
`(−∞, ∞)` and must not be written as one.

---

## 2. Registry

Columns: **counter** is the field asserted non-zero; **predicted range** is
Rule 2's bound; **mechanism** is why the bound holds. `hard` in the range
column marks bounds that are arithmetic rather than empirical — those can
be asserted in code with no calibration run.

### 2.1 Causal layer

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| `ei_micro` | `ei_intervals_emitted` | `[0, log₂ M]` (hard) | `EI = H_int − H_noise` over `M` live TPM rows; `H_int ≤ log₂ M` and `H_noise ≥ 0` (`docs/metric_formulas.md` §Causal layer) |
| `ei_macro` | `ei_intervals_emitted` | `[0, log₂ K]` (hard) | same identity over `K` macro groups; `K ≤ M` always |
| `ce_structural` | `ei_intervals_emitted` | `[0, ∞)`; `> 1` is causal emergence and **is not** expected on bare substrate | `ce = ei_macro / ei_micro`, defined `0` when `ei_micro ≤ 0`. The `CE < 1` bare-substrate baseline is the standing null |
| `wl_class_count` | `ei_intervals_emitted` | `[1, n_nodes]` (hard) | a WL class is a partition cell of the live node set |
| `task_lattice_depth` | `task_lattice_entries` | `[0, n_tasks]` (hard) | longest path in tasks of a DAG on `n_tasks` nodes |
| composition rate | `task_lattice_chains` | `[0, 1]` (hard) | realized / composable, both counted over the same chain set |

**`ei_intervals_emitted` is bug 7.** `ei_interval_steps` defaults to 1000.
At a 384-step horizon the interval never elapses, no EI is computed, and
every field comes through `None` — which averaged to `NaN` and read as
"shaping has no effect on EI". The counter is the number of intervals
actually computed *within the horizon*, and it is zero in exactly that
case.

`ei_minimum_transitions` (default 1000) is a **certainty threshold, not a
tuning knob** — the certainty table requires ≥ 1000 observed transitions
before an EI claim counts as Empirical. Lowering it to force emissions
would manufacture the measurement. When it suppresses a result, the correct
report state is `unmeasured`, which is what it now produces.

**`task_lattice_chains` is separate from `task_lattice_entries` on
purpose.** Composition rate is `undefined`, not `0.0`, when no composable
chain exists — a lattice with tasks but no A→B/B→C pair has nothing to
compose, and reporting `0.0` would say the substrate failed to compose when
it was never asked to.

### 2.2 Constructor registry

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| `n_confirmed_constructors` | `constructor_records_examined` | `[0, records_examined]` (hard) | confirmation is a filter over examined records |
| `mean_fidelity` | `constructor_records_examined` | `[0, 1]` (hard) | `confirmed_cycles / total`, a ratio of counts |
| F3 | `constructor_records_examined` | `[0, n · 1]` (hard) | `n · mean_fidelity`, `mean_fidelity ≤ 1` |
| F5 | `f5_records_examined` | `[0, max_i cycles_i]` (hard) | weighted-mean bound, `fidelity ≤ 1` (`docs/fitness_formulas.md` §4B.3) |
| `f5_top_share` | `f5_records_examined` | `[1/\|C\|, 1]` (hard) | it is a share of a positive total |
| `replicator_rate` | `constructor_records_examined` | `[0, 1]` (hard) | `n_replicators / max(1, n_constructors)` |

**This row is bug 8.** With `[causal]` disabled, `RolloutMetrics::
constructors` is empty, `n_confirmed_constructors` is 0, and F3 is
identically `0.0` for every arm — a table of zeros that reads as "no policy
affects constructor yield". The counter distinguishes *the causal layer
looked and found nothing* (`records_examined > 0`, F3 = a genuine `0.0`)
from *the causal layer never ran* (`records_examined == 0`, F3 =
`unmeasured`).

`Fixture::sim_config_causal` exists so that any fixture scoring F3, F5,
Stage 4 or Stage 5 has `[causal] enabled = true`. The counter is what
enforces it; the constructor is what makes enforcement passable.

### 2.3 Fitness functions

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| F1 | `f1_windows_scored` | `[0, 1]` (hard) | mean of per-window `interestingness_score`, which the evaluator clamps |
| F2 | `n_windows_scored` | `[−0.5, 1.0]` (hard) | `reward − 0.5 · penalty`, both fractions of `W` |
| `f2_reward` | `n_windows_scored` | `[0, 1]` (hard) | a fraction of `W` |
| `f2_penalty` | `n_windows_scored` | `[0, 0.5]` (hard) | `LAMBDA_PENALTY ·` a fraction of `W` |
| F4 | `f4_pairs` | `[0, log₂ PI_ALPHABET] = [0, 2]` (hard) | lag-1 MI of a `Q`-symbol variable is bounded by `log₂ Q`; `max(0, ·)` closes the bottom |
| `f4_null_sd` | `f4_pairs` | `[0, 2]`; **must be `> 0` whenever the symbol sequence has ≥ 2 distinct symbols** | the permutation null over a non-constant sequence is not a point mass |
| `kernel_held` | `probe_rows_collected` | `[0, 1]` (hard) | `rank_eps(X) / min(rows, dim)` |

`n_windows_scored == 0` is already `Err(FitnessError::NoScoredWindows)`
under v1 §1.1, which is the same discipline arrived at before this file
existed — a rollout that produced no scored window is invalid, not
zero-fitness, because "policy killed the run instantly" must not be
indistinguishable from "policy performed badly".

`f4_pairs < PI_MIN_PAIRS = 50` demotes F4 to `unmeasured` rather than
reporting a mutual information estimated from too few pairs to estimate
anything. This is bug 7's shape applied to an estimator instead of a
cadence.

### 2.4 Optimizer

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| genome movement | `generations_genome_moved` | `‖θ_g − θ_init‖₂` must exceed `3σ_init` by generation `min(10, G/4)` | a search whose steps are scaled to the parameterization leaves its initialization ball; see P-2 |
| allocation movement | `generations_genome_moved` | induced-allocation TV from the initial allocation `> 0.05` at the same generation | logit movement that produces no allocation movement is movement in a flat direction |
| champion margin | `holdout_seeds_evaluated` | no mechanism-derived bound — **blank by §1.1** | fitness ranges differ per function and the margin is the object of study |

**This row is bug 10.** `TrainingConfig`'s default `sigma = 0.02` is a step
size in **softmax-logit space**, where 0.02 moves the induced allocation by
under a percentage point. Forty generations of CMA-ES left the allocation
uniform to 4% and the campaign reported a clean negative that was actually a
search that never started. The two counters together are P-2, and they abort
with `OPTIMIZER_FROZEN` at generation 10 instead of producing a result at
generation 40.

The allocation-TV clause is load-bearing and is not redundant with the L2
clause. A genome can move a long way in logit space along a direction the
softmax is insensitive to — adding a constant to every logit moves θ
arbitrarily far and changes the allocation not at all. The L2 test alone
would pass on a search that had accomplished exactly nothing.

#### Correction to the plan's stated form of P-2

Plan v2 §4 states P-2 as a **conjunction**: abort unless `‖Δθ‖ > 3σ_init`
**and** allocation TV `> 0.05`. Implemented literally, that rejects the
genome it exists to bless. On the real σ=2.0 `wn_three` champion the
allocation moved from uniform to `[0.212, 0.777, 0.011]` — TV `0.4437`,
decisive — while `‖Δθ‖ = 5.39` sat under `3 × 2.0 = 6.0`. The conjunction
aborts a search that plainly worked.

The L2 clause is also **dimension-inconsistent** as written. `‖Δθ‖` grows
like `σ√n` under pure drift, so a scalar `3σ` bar is trivially passable for
a several-hundred-parameter `MlpScorer` genome and unreachably strict for a
three-parameter `StaticBias` one. It measures dimension, not movement.

So the implementation makes **the allocation clause decisive** and keeps L2
— normalized to `σ√n`, the drift scale — as the diagnostic that names which
failure occurred:

| `‖Δθ‖` | TV | reading |
|---|---|---|
| small | small | step size too small — **bug 10** |
| large | small | movement along a direction the policy cannot act on |
| any | large | traversing; proceed |

Both failures abort with `OPTIMIZER_FROZEN`, because both mean hours are
about to be spent on a search that has not started. `hg_train::prechecks`
carries a firing test for each row.

### 2.5 Policy RNG

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| policy stochasticity | `distinct_policy_seeds_drawn` | must equal the number of `(generation, member_idx, rollout_idx)` triples evaluated | `policy_seed = fxhash64(campaign_seed, generation, member_idx, rollout_idx, salt)` is injective in practice over the triples one campaign visits |
| hold-out spread | `distinct_policy_seeds_drawn` | `sd > SPREAD_EPS · (\|mean\| + 1)` for any stochastic policy | a policy that draws randomly produces seed-to-seed variation; zero spread means it did not draw |

**This row is bug 2.** Stage 0a's random control was constructed with its
policy seed pinned to the literal `1`, so every "seed" produced the same
selection and the arm had no spread. The classifier read that as
`Deterministic` and the finding would have been "these substrates are
seed-deterministic" — a statement about the harness presented as a
statement about the substrate.

A counter of distinct seeds *drawn* is the check, and it is deliberately
not a check that the seeds differ from each other by some amount: seeds are
hashes, and the failure was that the derivation was never called, not that
it collided.

### 2.6 Spread guard

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| σ-effect classification | `spread_guard_rejections` | over a full Stage 0a sweep, **must be `> 0`** | across 25 cells, some cell has an arm with no seed variance; a guard that rejected nothing over the whole grid is not engaged |
| effect size σ | — | `\|σ\| ≤ (range of the metric) / SPREAD_EPS` is not a useful bound; the **useful** one is that a cell reporting `\|σ\| > 100` is an instrument fault pending proof | a 100σ effect on 20 seeds means the denominator collapsed, not that the effect is enormous |

**This row is bug 4.** The first spread guard tested `pooled <= 0.0`. When
all 20 seeds return an identical value the sample standard deviation is
around `1e-16` rather than exactly zero — the mean is not bit-identical to
the samples — so a real difference divided by it produced effect sizes up
to `2.9e16σ` and a spurious "exit criterion MET" on cells with no seed
variance at all. The fix is a **relative** floor,
`sd > SPREAD_EPS · (|mean| + 1)` with `SPREAD_EPS = 1e-9`
(`hg_train::diagnostics::has_spread`), and the counter is what proves the
floor is doing work rather than sitting inert.

The `|σ| > 100` line is the clearest single instance of Rule 2 in this
file. It is not a claim that large effects are impossible; it is a claim
that a large effect and a collapsed denominator look identical in the
output and the collapsed denominator is overwhelmingly more likely.

### 2.7 Readout / probe

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| separation ratio (Stage 0b) | `probe_rows_collected` | `[0, ~1]`; **`> 1` is an instrument fault** | the perturbed-pair distance is bounded by the independent-seed distance if the readout is a metric and the reference is what it claims to be |
| memory capacity (Stage 1) | `readout_distinct_states` | `[0, min(n_delays, readout_rank)]` (hard) | `MC = Σ_d max(0, r²(d))`, each term `≤ 1`, and a rank-`r` readout cannot decode more than `r` independent delays |
| `input_coupling_r2` | `readout_distinct_states` | `[0, 1]` (hard) | it is an `r²` |
| readout non-degeneracy | `readout_distinct_states` | **must be `≥ 2`** for any separation or MC claim | a readout that emits one state cannot separate two trajectories, whatever they do |

**The last row is bug 6 and the `glider` finding.** Stage 0b's first build
perturbed a constructed initial condition through `PassThrough`, so there
was no perturbation and the diagnostic classified `Ordered` where nothing
had been done. Its second build compared churn *scalars* rather than
states, producing separation ratios above 1 against a supposed upper bound
— arithmetically impossible, and caught by Rule 2 rather than by a test.

`glider` then failed the same clause for an entirely legitimate reason: its
bucketed WL histogram is *exactly constant*, because the rule is a
conservative relabel on a symmetric ring. `readout_distinct_states == 1`.
That is not "no separation"; it is **no measurement**. `glider` is
invisible, not inert, and `readout_set = "r2"`
(`docs/policy_config.md` §4B) exists to find out which.

This also bounds Stage 1's headline. `MC = 0.0390` against a matched
ESN's `5.8237` is a joint measurement of substrate memory **×** readout
sensitivity, and the ESN readout sees its full state vector where the
hypergraph readout sees 16 buckets. The factor of 65 may be measuring the
projection, and until Stage 1′ runs under r2 the registry records that
comparison as **not dimension-matched**.

### 2.8 Event alphabet (A-track)

The substrate's **realised event space**: the distinct *(rule, local structural
context)* signatures a run produces. Definitions and the radius selection rule
are frozen in `docs/atrack_a1_preregistration.md`, committed at `a43ca3d` before
any cardinality was computed. Bare substrate, zero biology.

| measurement | counter | predicted range | mechanism |
|---|---|---|---|
| `\|A(R)\|` (alphabet cardinality) | `n_events` | `[1, n_events]` (hard) | a symbol must be realised by an event to be counted |
| `A_eff` (effective size) | `n_events` | `[1, \|A\|]` (hard) | `2^H` over a distribution on `\|A\|` symbols; `H ≤ log₂\|A\|` |
| `\|A(R)\|` vs R | `n_events` | non-decreasing in R (hard) | WL colours at round R+1 determine those at round R, so `wl_{R+1}` refines `wl_R` |
| `\|A_S1(R)\|` vs `A_rule_only` | `n_events` | `≥ A_rule_only` (hard) | S1 hashes `rule_id` as part of its input, so it refines the rule-only partition |
| `\|A_S2(R)\|` vs `\|A_S1(R)\|` | `n_events` | `≥ \|A_S1(R)\|` (hard) | S2 hashes S1's whole input as a prefix |
| `R*` (saturation radius) | `n_events` | `{1, 2, 3}` or absent | R = 0 is the no-context baseline; R = 4 is unverifiable because checking saturation at R needs R+1 measured |
| growth exponent `b` vs events | `n_events` | `≤ 1` asymptotically (hard) | `N(e) ≤ e`: one event contributes at most one new symbol |

**`n_events` is the engagement counter.** A rule whose LHS never matches the
initial condition produces a genuinely empty alphabet, and `INSTRUMENT_NOT_ENGAGED`
is the difference between *this substrate has one event type* and *this rule
never fired*. `tetrahedral` on a triangle lattice is precisely that case, which
is why the A-track gives it a K4 seed and records the choice.

**Two clauses here are not merely bounds but instrument faults if violated**:
non-monotonicity in R, and `|A_S2| < |A_S1|`. Both are asserted in
`crates/hg-alphabet/tests/a1_definitions.rs`.

**The registry's own Rule 2 caught a live defect during A-track construction**,
though not through a range: the growth classifier reported `r²_loglog = 0.000`
for a *perfect* plateau, because a zero-variance test written as `Σ(y − ȳ)² == 0`
leaks a ~1e−28 denominator when `ȳ` is the floating-point mean of n identical
values (`Σ ln 3 / 61 ≠ ln 3`). A fit quality of 0 on a flawless fit is
mechanism-impossible, which is what made it visible. Constancy is now detected
from the data (`y_max == y_min`), not from the variance.

---

## 3. What the counters do not cover

Rule 1 catches instruments that never ran. It does not catch instruments
that ran and computed the wrong thing. Bugs 1, 3, 5, 9 and 11 are that
class, and the one that matters most is bug 1:

> `RandomSubset` returned the first `k` bindings rather than a random `k`,
> because `rand 0.8`'s `partial_shuffle` permutes into the **tail** and
> returns `(tail, head)` — the code kept the head. The control arm was not
> a control.

It flipped Stage 1's input coupling from `−0.0175` to `+0.9261`, reversing a
confident conclusion about the injection seam, and changed 13 of 25 Stage 0a
cells. Every pre-fix number in the C0–C5 record is suspect and anything not
re-derived post-fix is treated as `unmeasured`
(`hypersim_nn_experiments_plan_v2.md` §12).

The counter for `RandomSubset` would have read *fully engaged* throughout.
It was selecting, every step, exactly `k` candidates, using its RNG. What
was wrong was which `k`.

The defences that do apply to this class, in order of how much they
actually bought:

1. **Mechanism-derived prediction** (Rule 2). Three of five.
2. **Property tests on the control arm specifically.** `RandomSubset` now
   carries `random_subset_samples_uniformly_not_just_reproducibly`: 400
   seeds × `k = 10` over 100 candidates, asserting that no candidate is
   *never* selected and that per-candidate counts span `0.4×`–`1.6×`
   expected. A reproducibility test alone passed the buggy version, because
   returning the first `k` is perfectly reproducible.
3. **Arithmetic impossibility as a first-class check.** Cheap, and it is
   what caught bugs 4 and 5.

The general lesson, stated so it survives this plan: **a control arm needs
a stronger test than the thing it controls for.** The experimental arm is
under suspicion by design and gets scrutinized; the control is assumed
correct and gets a determinism test.

---

## 4. Retrospective application

Plan v2 §6.3 requires the C0–C5 measurement suite to be re-run with
counters live before any new campaign, and any number whose counter reads
zero to be demoted to `unmeasured` in the findings record. Demotions are
recorded in §5 below, with the affected document and section, so that a
reader of `docs/cognition_stage0_findings.md` (or 2 / 45) is not required
to have read this file to know which numbers stand.

A demotion is **not** a retraction. `unmeasured` says the instrument did
not engage and the value carries no information; it does not say the
underlying claim is false. Re-deriving a demoted number with the instrument
engaged is a normal, expected outcome and restores it unchanged.

---

## 5. Demotion record

*(Populated by Stage I, plan v2 §9. Empty until then — an empty demotion
record after Stage I has run is itself a result, and is not the same as
this section not having been filled in.)*

| number | document | counter that read zero | status |
|---|---|---|---|
| — | — | — | Stage I not yet run |

---

## 6. Changelog

- **v1** (2026-07-27) — initial, per plan v2 §6 and Phase D0. Six
  instrument groups, two rules, the bug-class analysis in §3, and an empty
  demotion record.
- **v2** (2026-08-02) — §2.8 added: the A-track event alphabet, with seven
  hard bounds (two of them assertable as instrument faults) and `n_events` as
  its engagement counter. Records the `r²` defect Rule 2's reasoning surfaced.
