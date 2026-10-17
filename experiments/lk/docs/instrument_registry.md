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
Rule 2's bound; **mechanism** is why the bound holds; **blind spot** is
Rule 3's — what the proxy cannot see. `hard` in the range column marks
bounds that are arithmetic rather than empirical, assertable in code with no
calibration run.

**An instrument with an empty blind-spot entry is not documented; it is
asserted.** A `—` is admissible only where the instrument measures exactly
the fact it names, and that claim is itself a claim.

### 2.1 Causal layer

| measurement | counter | predicted range | mechanism | **blind spot** |
|---|---|---|---|---|
| `ei_micro` | `ei_intervals_emitted` | `[0, log₂ M]` (hard) | `EI = H_int − H_noise` over `M` live TPM rows; `H_int ≤ log₂ M` and `H_noise ≥ 0` (`docs/metric_formulas.md` §Causal layer) | the counter says intervals were emitted, not that the TPM had support at each |
| `ei_macro` | `ei_intervals_emitted` | `[0, log₂ K]` (hard) | same identity over `K` macro groups; `K ≤ M` always | the coarse-graining is *given*, not learned — this measures a partition, not the substrate |
| `ce_structural` | `ei_intervals_emitted` | `[0, ∞)`; `> 1` is causal emergence and **is not** expected on bare substrate | `ce = ei_macro / ei_micro`, defined `0` when `ei_micro ≤ 0`. The `CE < 1` bare-substrate baseline is the standing null | a ratio cannot distinguish "macro rose" from "micro fell"; §5.3 and F6's floor both exist for this |
| `wl_class_count` | `ei_intervals_emitted` | `[1, n_nodes]` (hard) | a WL class is a partition cell of the live node set | WL radius is fixed; a structure finer than the radius is invisible (V-track J0) |
| `task_lattice_depth` | `task_lattice_entries` | `[0, n_tasks]` (hard) | longest path in tasks of a DAG on `n_tasks` nodes | depth over a lattice built from *observed* tasks; unobserved composition is not absence |
| composition rate | `task_lattice_chains` | `[0, 1]` (hard) | realized / composable, both counted over the same chain set | `undefined` at zero chains is reported, but "few chains" and "many chains" share one denominator |

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

| measurement | counter | predicted range | mechanism | **blind spot** |
|---|---|---|---|---|
| `n_confirmed_constructors` | `causal_enabled` | `[0, records_examined]` (hard) | confirmation is a filter over examined records | the registry's own confirmation criterion; a structure it cannot pattern-match is not reported as missed |
| `mean_fidelity` | `causal_enabled` | `[0, 1]` (hard) | `confirmed_cycles / total`, a ratio of counts | a ratio over confirmed records only — says nothing about what was never confirmed |
| F3 | `causal_enabled` | `[0, n · 1]` (hard) | `n · mean_fidelity`, `mean_fidelity ≤ 1` | grows with the run; comparable only at matched horizon |
| F5 | `causal_enabled` | `[0, max_i cycles_i]` (hard) | weighted-mean bound, `fidelity ≤ 1` (`docs/fitness_formulas.md` §4B.3) | rewards reuse depth, which raises `ei_micro` — it cannot see that it is aimed against emergence (v3 §7.1) |
| `f5_top_share` | `causal_enabled` | `[1/\|C\|, 1]` (hard) | it is a share of a positive total | concentration on a collapsed graph and on a healthy one look identical without `mean_n_nodes` |
| `replicator_rate` | `causal_enabled` | `[0, 1]` (hard) | `n_replicators / max(1, n_constructors)` | `is_replicator` is a flag the causal layer sets from few observations |

**This row is bug 8, and then its own over-correction.** With `[causal]`
disabled, `RolloutMetrics::constructors` is empty, `n_confirmed_constructors`
is 0, and F3 is identically `0.0` for every arm — a table of zeros that reads
as "no policy affects constructor yield".

The first fix keyed on `records_examined == 0`, which suppressed **every
genuine zero** as well, because a live detector that finds nothing also
returns an empty registry (§5.2). The counter column above therefore reads
`causal_enabled` — a flag from the runner, not a count of outcomes. The
distinction the schema must keep is *the causal layer looked and found
nothing* (`causal_enabled`, F3 = a genuine `0.0`) from *the causal layer
never ran* (`!causal_enabled`, F3 = `unmeasured`).

`Fixture::sim_config_causal` exists so that any fixture scoring F3, F5,
Stage 4 or Stage 5 has `[causal] enabled = true`. The counter is what
enforces it; the constructor is what makes enforcement passable.

### 2.3 Fitness functions

| measurement | counter | predicted range | mechanism | **blind spot** |
|---|---|---|---|---|
| F1 | `f1_windows_scored` | `[0, 1]` (hard) | mean of per-window `interestingness_score`, which the evaluator clamps | a time mean — a run that is interesting then dead scores like one that is steadily mediocre |
| F2 | `n_windows_scored` | `[−0.5, 1.0]` (hard) | `reward − 0.5 · penalty`, both fractions of `W` | occupancy only; says nothing about *what* was built inside a target regime |
| `f2_reward` | `n_windows_scored` | `[0, 1]` (hard) | a fraction of `W` | — (reported separately from the penalty precisely so the decomposition is visible) |
| `f2_penalty` | `n_windows_scored` | `[0, 0.5]` (hard) | `LAMBDA_PENALTY ·` a fraction of `W` | — |
| F4 | `f4_pairs`, `f4_jsd_spread` | `[0, log₂ PI_ALPHABET] = [0, 2]` (hard) | lag-1 MI of a `Q`-symbol variable is bounded by `log₂ Q`; `max(0, ·)` closes the bottom | **it measures monotone drift, not predictive information** (§5.3, v3 §7.3); `f4_pairs` alone cannot see a resolution failure |
| `f4_null_sd` | `f4_pairs` | `[0, 2]`; **must be `> 0` whenever the symbol sequence has ≥ 2 distinct symbols** | the permutation null over a non-constant sequence is not a point mass | a shuffle null removes order, not trend — it does not calibrate against drift |
| `kernel_held` | `probe_rows_collected` | `[0, 1]` (hard) | `rank_eps(X) / min(rows, dim)` | rank of the *readout*, so it inherits every blind spot in §2.8's readout rows |

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
| separation ratio, **stability** | `seed_triples_measured` | a class must be stable across triples | one triple is one draw; see §2.7.1 |
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
That is not "no separation"; it is **no measurement**.

`readout_set = "r2"` was built to find out whether that was a resolution
limit, and the answer is no: it is a **theorem**. Measured, r2 also resolves
one state on `glider` — 0 of 43 columns vary — while the run accepts eight
rewrites in every window and a *positional* readout resolves three states.
The trajectory passes through **isomorphic graphs**, so every
isomorphism-invariant readout is constant on it by construction, and r1, r2
and the whole evaluator metric family are isomorphism-invariant. `glider` is
excluded from separation analysis with that as its reason
(`docs/policy_config.md` §4B.6).

The registry-level lesson, which generalizes past this rule set: **a readout
blind to the group the dynamics act within reports no motion, whatever the
dynamics do.** The V-track reached the identical shape from the other side —
its arm D was structurally constant while its ALife energy changed on
5999 of 5999 steps, because WL hashing excluded that property channel
(`docs/validation_v7_l_track_findings.md`). Two tracks, two groups, one
failure mode. `readout_distinct_states` is the counter that catches it in
both.

This also bounded Stage 1's headline, and **Stage 1′ has now resolved it**:
under r2 the readout is 48-dimensional against a 48-unit ESN and `MC` moves
0.0390 → 0.0364 — a 7% move, in the wrong direction for the readout
hypothesis. The Stage 1 null is a substrate result. `input_coupling_r2`
also rises to **exactly 1.0000** at delay 0 with still nothing at delay ≥ 1,
which is the discrimination r1's 0.9261 could not supply on its own
(`docs/cognition_stage_0b1_prime_findings.md` §1).

Two corrections while it was there: the v1 comparison **was**
dimension-matched at 16 vs 16 — what was never matched is information
content per dimension — and the measured ratio is `5.8237 / 0.0390 = 149`,
not the 65 the plan quotes.

### 2.7.1 The stability clause — added by Stage 0b′

`stage0b` derives its perturbed partner *and* its independent reference
from one base seed, so **one call is one seed triple**, and the C0–C5 run
made exactly one call per rule set.

Measured over 8 triples, the ratio exceeds its hard bound on a **majority**
of draws for `growth_treadmill` (5/8 under r1, 5/8 under r2) and
`triangle_subdivide` (5/8, 4/8), reaching 4.238 — and 15.818 on
`binary_edge`. The classes are correspondingly unstable:
`triangle_subdivide`'s published "Viable, ratio 0.752" comes back Viable on
**2 of 8** triples, and `fl1_two_rule` is Chaotic ×5 under r1 and Viable ×5
under r2.

The fault is the **denominator**: a single independent-seed pair is not a
stable upper bound on these substrates, and it occasionally collapses —
bug 4's signature in a different measurement. So:

> A separation class computed from one seed triple is **not admissible**.
> The reference must be averaged over several disjoint pairs, the ratio
> reported with its spread, and a class refused when that spread crosses a
> class boundary.

This is a design defect in Stage 0b present since C0–C5 and **not** fixed
by the readout work — r2's ratios are no better behaved than r1's. Every
Stage 0b class in the C0–C5 record is demoted to `unmeasured` pending the
redesign (`docs/cognition_stage_0b1_prime_findings.md` §2.5). Per §4 that
is a demotion, not a retraction: the runs happened, and what is missing is
a statistic that can turn them into a class.

---

### 2.8 Quantizers — observable range vs design range (v4)

Rule 2 bounds a measurement's **output**. That is insufficient on its own:
F4's output was inside `[0, 2]` and meaningless, because the *input* to its
quantizer lived nowhere near where the quantizer assumed. Hence Rule 4:

> Every quantizer, binning, or thresholding step declares the **expected
> range of the observable it consumes**, and that declaration is checked
> against the quantizer's design range.

Populated by E0's audit (`docs/cognition_e0_quantizer_audit.md`), measured on
both owned substrates. A new substrate re-opens every row — which is the
point of declaring them rather than assuming them.

| quantizer | consumes | design range | measured range | **blind spot** |
|---|---|---|---|---|
| F4 symbolization (v2.1) | window type-histogram JSD | equal-count quantiles of the run's own series | `[0.002, 0.085]` | adapts to the observable, so **none of this class**; it cannot see monotone drift (§5.3) |
| ~~F4 symbolization (v2)~~ | window type-histogram JSD | `[0, 1]` fixed grid | `[0.002, 0.085]` | **retired** — every window in bin 0 on every substrate, `used = 0.000` |
| `features.rs` priority | rule priority `i32` | unbounded via `tanh(x/8)` | `{0}` on every owned substrate | cannot report that its own observable is constant |
| `features.rs` arity | bound-node count | `[0, 8]` | `[2, 3]` | — under-uses range, loses no information |
| `features.rs` degree (v1) | node incident-edge count | `[0, 64]` | `[1, 130]` | values above 64 collide; 0.3–6.0% of values |
| `features.rs` degree (**v2**) | node incident-edge count | `[0, ∞)` via `r/(1+r)` | `[1, 130]` | cannot saturate; still relative to *mean* degree, so a uniformly dense graph and a uniformly sparse one look alike |
| `features.rs` candidate count (v1) | pool size per step | `[0, 4096]` | `[150, 16974]` | pools above 4096 collide; **49–79% of steps** |
| `features.rs` candidate count (**v2**) | pool size per step | `[0, ∞)` via `r/(1+r)` | `[150, 16974]` | cannot saturate; measures pool *per node*, so it cannot distinguish a big graph from a dense one |
| `readout.rs` degree | window mean degree | `[0, 64]` | `[2.75, 5.61]` | — under-uses range (0.043–0.048) |
| `readout.rs` size | window node count | `[0, 65536]` | `[37, 1539]` | — under-uses range |
| WL bucketing | node WL hash | `wl_buckets = 16` | hash, not continuous | collisions are silent; two structurally distinct nodes may share a bucket |

**The distinction this table has to carry.** Under-using a range loses no
information; **saturating** one destroys distinctions the substrate actually
makes. Only the second is a defect. A registry that flagged "used < 1" as
uniformly bad would rank three sound instruments alongside a dead one — which
is why the audit's `used` figure is a diagnostic and never a target, and why
`v2_encodings_do_not_saturate` asserts on the saturation column instead.

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

**Stage I ran on 2026-07-28** (`crates/hg-train/tests/d3_stage_i_demotion.rs`),
against the configurations the published numbers were actually measured
under rather than a synthetic one.

### Result: no demotions.

| number | document | counters | status |
|---|---|---|---|
| Stages 4 & 5, all three arms | `cognition_stage45_findings.md` | `ei=10`, `tasks=93 / 771 / 641`, `chains=112 / 2083 / 2575`, `ctors=85 / 418 / 254` | **stands** |
| Stage 0a, all rule sets | `cognition_stage0_findings.md` | `f1_windows_scored = 7 / 7 / 3 / 3` | **stands** |
| **every Stage 0b class** | `cognition_stage0_findings.md` §2 | counters fine; the *statistic* is invalid (§2.7.1) | **DEMOTED** — added by Stage 0b′, not by the counter pass |

Every instrument a published number depends on engaged. An empty demotion
record after Stage I has run is itself a result, and is not the same as
this section not having been filled in.

Stage 0a's `ei_intervals_emitted`, `task_lattice_entries` and
`constructor_records_examined` all read **zero**, and that is **not** a
demotion: Stage 0a runs with `[causal]` off by design and published no EI,
task-lattice or constructor number. §4's "a demotion is not a retraction"
cuts both ways — an instrument that was never *asked* to run is a different
state from one that was asked and failed.

### 5.1 What Stage I found instead: F4 is unmeasurable on the current battery

The pass was not looking for this, and it is the more consequential result.

`f4_pairs` — F4's engagement counter — reads **2 to 10** across every
fixture in the programme, against `PI_MIN_PAIRS = 50`:

| fixture | steps | window | windows | `f4_pairs` |
|---|---|---|---|---|
| Stage 4/5 `wn_three` | 384 | 24 | 16 | **10** |
| Stage 0a `glider` | 512 | 64 | 8 | **6** |
| Stage 0a `growth_treadmill` | 512 | 64 | 8 | **6** |
| Stage 0a `triangle_subdivide` | 96 | 24 | 4 | **2** |
| Stage 0a `binary_edge` | 96 | 24 | 4 | **2** |

So **F4 would report `unmeasured` on every fixture the programme currently
owns.** That is the guard working exactly as designed — a mutual
information estimated from ten pairs is not an estimate — but it means
Stage 2′ cannot test F4's stateability (plan v2 §9) without first choosing
horizons and windows that produce enough scored windows.

The requirement is arithmetic: `f4_pairs = W − 1` where
`W = horizon/window − burn_in`, so F4 needs

```
horizon / window  ≥  PI_MIN_PAIRS + 1 + burn_in  =  56   (at B = 5)
```

— roughly a **3.5×** increase in windows over the Stage 4/5 fixture. Either
the horizon grows or the window shrinks, and the two are not equivalent: a
shorter window changes what each symbol means, and Stage 1's window sweep
already measured that input coupling falls as windows shorten (0.93 at
w=32, 0.68 at w=4). Whichever is chosen has to be stated as part of F4's
measurement definition, not picked to clear the bar.

This is recorded here rather than in `fitness_formulas.md` because it is a
fact about the **fixtures**, not about the formula. F4's definition is
unchanged.

> ### 5.1.1 Superseded by Stage 2′ — the diagnosis above is wrong
>
> **The pair count was never the blocker, and clearing it would have
> produced a false result.** Phase D5 ran F4 at 58 pairs and found the
> symbol sequence still collapses to **one distinct symbol** — on every
> substrate, at every horizon and window tried. The fixed `floor(4 · jsd)`
> grid assumed the JSD uses its `[0, 1]` range; measured, it lives in
> `[0.002, 0.085]`, entirely inside the first of four bins.
>
> Under `PI_MIN_PAIRS` that surfaced as `unmeasured`, which is what §5.1
> saw and mis-attributed to sample size. **Above** it, the one-symbol
> branch classified the run as the frozen case and returned a *measured*
> `0.0` — so following §5.1's own prescription would have published "zero
> predictive information" for a substrate that grew from 24 to 5992 nodes.
> A guard reading `unmeasured` for the wrong reason concealed a Rule 1
> violation behind it.
>
> Two clauses of §5.1 are withdrawn:
>
> * **"F4's definition is unchanged"** — it changed. `v2.1` replaces the
>   fixed grid with equal-count (quantile) symbolization
>   (`fitness_formulas.md` §4A.2).
> * **"a fact about the fixtures, not about the formula"** — it was a fact
>   about the formula. The fixtures were never the problem: at 58 pairs and
>   window 6, the *same* fixture measures fine once symbolized correctly.
>
> The horizon/window recommendation is also reversed. §5.1 argued the
> long-horizon route was the principled one because "a shorter window
> changes what each symbol means". Measured, the long route costs **28×**
> for an **identical** drift confound (trend −0.877 vs −0.875, monotone
> ceiling 1.59 vs 1.61) over a substrate that grows four times as far, and
> the comparability it was supposed to buy does not exist — F4 had never
> been measurable at window 24, so there was no published number to be
> comparable to. Stage 2′ uses 384 steps at window 6.
>
> **What generalizes.** `f4_pairs` is a *sample-size* counter, and no
> sample-size counter can detect a **resolution** failure. The programme
> had one guard on F4's engagement and it was watching the wrong axis:
> enough samples of an unresolved observable is still an unresolved
> observable. §3's "harder bug class" now has a measured instance.
> `f4_jsd_spread` is the added guard — it separates "the substrate was
> frozen" from "the substrate moved and we could not see it", which is the
> distinction the one-symbol branch was silently collapsing.
>
> Full account: `docs/cognition_stage2_prime_findings.md` §1.1.

### 5.2 Second correction from Stage 2′ — an empty constructor registry

`constructor_reuse` and F3's `select` gate both keyed on
`constructors.is_empty()` / `constructor_records_examined == 0`. That is
**bug 8's fix over-firing in the opposite direction**: an empty registry is
also what a live detector produces when there is nothing to detect, so
every genuine zero was suppressed as a non-measurement.

It fired on real data. `tri_two` under the allocation extreme returns an
empty registry with `[causal]` **on** for 2 of 3 seeds; the arm was reported
as `1.0000 ± 0.0000 (n=1, unmeasured=2)` and P-3 passed a Stage 2′ cell on a
mean computed from one seed.

`causal_enabled` is now carried from the runner (`causal.is_some()`) through
`RunArtifacts` → `RolloutMetrics` → `Engagement`, and F3/F5 gate on it. The
same arm reads `0.3333 ± 0.5774 (n=3, unmeasured=0)` and the cell passes on
evidence. `not_engaged()` reports `causal_enabled` rather than
`constructor_records_examined`.

**Registry consequence (§2.2).** A record count is not an engagement
signal — it is an *outcome*. Engagement has to be reported by the thing that
either ran or did not. Both of this phase's corrections have that shape:
a counter standing in for a fact it cannot observe.

### 5.3 A guard added, not a demotion — F4's drift diagnostic

Equal-count symbolization reads a monotone **drift** as temporal structure.
Every substrate in the programme drifts (Spearman trend against window
index: −0.72 to −1.00) because they all grow, and on `tri_two` the arm F4
ranks highest scores **1.001× its own sorted series** — F4 at 1.60 of a
possible 2 for a sequence carrying nothing that sorting would not supply.

`fitness::f4_drift` reports `trend`, `monotone_ceiling` and
`share_of_ceiling` alongside F4. It is deliberately **not** subtracted: the
drift is not linear, so a correction needs a detrending model the programme
does not have. It ships with a firing test
(`the_drift_diagnostic_fires_on_a_purely_monotone_series`).

No value is demoted — F4 has never been published. What is registered is
that an F4 result must be read with its drift share attached.

### 5.4 Four defects found by Stage 3′ (D6)

The first two are in P-3's far-end probe genome; the third is P-2's
decisive statistic; the fourth is the campaign's initialization. All four
would have produced a confident wrong answer, and three of them under the
wrong abort code.

**1. The probe was `StaticBias`-shaped.** `allocation_extreme` puts one
softmax logit at 8.0 — the far end of `StaticBias`'s parameter space.
Applied to an `MlpScorer` genome it sets `w1[0] = 8.0` and leaves `w2` at
zero, so the one live hidden row is multiplied by zero on the way out and
**every candidate scores exactly 0.0** — the same policy as θ=0, which is
the same policy as random. Measured on a 433-parameter scorer: `score` is
`0.000000` for two dissimilar feature vectors under both genomes. P-3 would
have compared three copies of one policy and aborted `OBJECTIVE_BLIND`.

Fixed by `prechecks::class_extreme` (per-class probe) and guarded by
`prechecks::probe_genome_is_live`, which scores two dissimilar vectors and
refuses to start if they tie. Firing test:
`the_static_bias_extreme_is_a_dead_probe_for_an_mlp`.

**2. The fix's first version scaled the probe by the campaign's `sigma`**,
and both D6 campaigns then refused to start at `sigma = 0.5` — F5 ×
`tri_two` spread 41.43 against a bar of 106.01, F4 × `wn_three` 0.3686
against 0.5194 — while the same cells separate by 306.15/38.61 and
1.1828/0.2597 at a fixed probe scale.

This is the sharper of the two. P-3 asks whether the objective separates the
**class**; scaling its probe by the optimizer's step size makes P-3's
verdict a function of `sigma`, so a small `sigma` produces an
`OBJECTIVE_BLIND` abort — **a fact about the optimizer reported under the
objective's abort code**. Plan v2 §4 separates P-2's question from P-3's
exactly so that cannot happen, and a σ-scaled probe quietly re-merges them.
`MLP_PROBE_SCALE = 2.0` is fixed and justified by the separation it
achieves.

**3. P-2's decisive statistic was per-rule.** `RuleTrace::allocation()`
normalizes selections by rule id, which is what `StaticBias` chooses.
`MlpScorer` chooses among individual bindings and can re-rank every
candidate while leaving the per-rule proportions untouched — measured, TV
was `0.0000` on both D6 cells at generation 7 while `|dtheta|` had moved.
`selection_divergence` adds a per-step selection signature and P-2 now
picks its decisive clause by class, reporting both. Firing tests:
`p2_reads_a_scorer_by_its_selections_and_a_logit_policy_by_its_allocation`,
`selection_divergence_finds_the_first_differing_step`.

**4. `theta = 0` is not a neutral start for a scorer.** `gumbel_top_k` adds
`Gumbel(0, 1)` noise (sd ≈ 1.28) to raw scores, and an MLP's output scale is
quadratic in its genome, so there is a wide basin around the origin in which
the policy is budget-matched random *whatever its weights are* — measured
score spread 0.0055 at `|theta| = 0.54`, against 0.921 at `|theta| = 7`.
`class_init` starts scorers at `MLP_INIT_L2 = 7.0`, where the spread matches
the Gumbel scale. Firing test:
`the_mlp_init_scale_lands_where_the_scorer_can_actually_act`.

This also exposed a latent bug: the `Untrained` control arm hardcoded
`vec![0.0]` rather than the campaign's starting genome, and
`champion − untrained` is the primary statistic. Bound to `theta_init`.
Latent only — every campaign to date started at zero.

**Registry consequence.** A precheck's own probe is an instrument, and it
needs the same discipline as the measurements it gates: a stated mechanism,
a check that it engaged, and independence from the variable the *other*
prechecks own. All the D5/D6 instrument findings share one shape —
**something standing in for a fact it cannot observe**: a record count for
engagement, a sample-size counter for resolution, an optimizer's step size
for a class's expressiveness, a per-rule allocation for a per-candidate
policy, and a zero vector for "untrained".

---

## 6. Changelog

- **v1** (2026-07-27) — initial, per plan v2 §6 and Phase D0. Six
  instrument groups, two rules, the bug-class analysis in §3, and an empty
  demotion record.
- **v2** (2026-07-28) — Phase D5. §5.1 superseded by §5.1.1: F4's
  unmeasurability was a resolution failure, not a sample-size one, and the
  §5.1 prescription would have published a false zero. §5.2 records the
  `causal_enabled` correction. §5.3 registers `f4_drift`. Both corrections
  share one shape — a counter standing in for a fact it cannot observe —
  which is the first measured instance of §3's harder bug class.
- **v4** (2026-07-30) — Phase E0, per plan v3 §6. Adds the mandatory
  **blind-spot column** to every registry table: an instrument with an empty
  entry is asserted, not documented. Adds **§2.8**, the quantizer register,
  with each quantizer's declared observable range checked against its design
  range. Retires `constructor_records_examined` from the counter column in
  §2.2 in favour of `causal_enabled`, correcting text that still described
  the pre-D5 gate.
- **v3** (2026-07-29) — Phase D6, §5.4. Two defects in P-3's far-end probe
  genome: a `StaticBias`-shaped probe that is provably inert on an
  `MlpScorer`, and a σ-scaled probe that reported an optimizer fact under
  `OBJECTIVE_BLIND`. Adds `class_extreme`, `probe_genome_is_live` and
  `MLP_PROBE_SCALE`, `selection_divergence`, `class_init` and
  `MLP_INIT_L2`. Establishes that **a precheck's own probe is an
  instrument** subject to this registry, and that a policy class's neutral
  point is a measured property rather than a convention.
