# Campaign output — artifacts, layout, and the contamination firewall

Governs everything `hg-train` writes. Companion to
`docs/fitness_formulas.md` (what gets optimized) and `docs/policy_config.md`
(what gets configured).

v4's output invariant is *one CSV + one Markdown report per run*. A
training campaign is 50 generations × 64 members × 3 seeds ≈ 10 000 policy
evaluations; 10 000 reports would destroy that discipline rather than
honour it. This document defines the three unit types that preserve it.

---

## 1. Three units

| Unit | Engine | Evaluator | Artifacts |
|---|---|---|---|
| **Run** | full | on | one `metrics.csv`, one report, one `manifest.json` — unchanged from v4 |
| **Rollout** | full, same step loop | on | **none.** Returns `RolloutMetrics` in memory |
| **Campaign** | N rollouts | on | exactly one `campaign.csv`, one `campaign_report.md`, one `campaign_manifest.json` |

**Rollouts are never citable results.** They exist only inside campaigns,
leave no artifact, and no claim may reference one. A rollout that produced
an interesting number is re-run as a validation run before it is discussed.

**Champions graduate to validation runs.** A champion policy is only ever
claimed about after ordinary scientific runs — full pipeline, held-out
seeds, real report — with the policy active and flagged (§5).

The rollout path uses the identical step loop, matcher, conflict resolver,
and evaluator as a scientific run. The only difference is that nothing is
written. This is enforced by construction (`run_rollout` wraps the same
internal loop the controller drives), not by convention, and is tested by
the F-L0 hash-equality check.

---

## 2. Directory layout

```
runs/substrate/<run_id>/                 # policy = none, ALWAYS.
    metrics.csv                          #   baselines and substrate science
    report.md                            #   live here and only here
    manifest.json

runs/cognition/<campaign_id>/
    campaign.toml                        # full config: [cognition] + [training]
    campaign.csv                         # exactly one, always
    campaign_report.md                   # exactly one, always
    campaign_manifest.json               # status record, resume point
    champions/
        gen_0000.policy.toml
        gen_0042.policy.toml
    validation/
        <standard run dirs>              # full scientific runs, policy-active
```

`campaign_id` is `<experiment_name>_<campaign_seed>` , or
`<experiment_name>_<campaign_seed>_TEST` under `fixed_provenance`, mirroring
`run_id` derivation so campaigns are as reproducibly named as runs.

---

## 3. `campaign.csv`

One row per rollout. Written through `hg_output::CsvWriter<CampaignRow>` —
no new output infrastructure is needed; the generic writer already emits the
`#hypersim_csv_v=…,run_id=…,seed=…,created=…` provenance comment and derives
the header from the row type's field names.

`run_id` in that comment line carries the `campaign_id` and `seed` carries
the `campaign_seed`, so a campaign CSV is self-identifying by the same
convention as a run CSV.

### 3.1 Columns

**Identity**

| Column | Type | Notes |
|---|---|---|
| `generation` | u32 | |
| `member_idx` | u32 | index within the population |
| `antithetic` | i8 | `+1` / `−1` for ES pairs, `0` for CMA-ES |
| `sim_seed` | u64 | the rollout's simulation seed |
| `rollout_idx` | u32 | index within `seeds_per_member` |
| `policy_rng_seed` | u64 | derived per `docs/policy_config.md` §5 |
| `genome_hash` | u64 | hash of the parameter vector |
| `is_holdout` | bool | true for champion evaluations on frozen seeds |

**Fitness — always decomposed**

| Column | Type | Notes |
|---|---|---|
| `fitness` | f64 | the active fitness function's value |
| `fitness_fn` | string | `f1_interestingness` \| `f2_regime_occupancy` \| `f3_constructor_yield` \| `f4_predictive_information` \| `f5_constructor_reuse` |
| `f1_mean_interestingness` | f64 | computed every rollout regardless of `fitness_fn` |
| `f2_reward` | f64 | target-regime fraction, **before** penalty |
| `f2_penalty` | f64 | `LAMBDA_PENALTY ·` penalized fraction |
| `f3_constructor_yield` | f64 | |
| `n_confirmed_constructors` | usize | |
| `mean_fidelity` | f64 | |
| `f4_predictive_information` | f64 \| `unmeasured` | v2. `docs/fitness_formulas.md` §4A |
| `f5_constructor_reuse` | f64 \| `unmeasured` | v2. §4B there |

**Arm identity (v2)** — the three-arm protocol is per-row, not only per-report:

| Column | Type | Notes |
|---|---|---|
| `arm` | string | `random` \| `untrained` \| `champion`. Every row carries one; there is no default and no unlabelled row |

All **five** fitness functions are computed and recorded on every rollout
even though only one is optimized. They cost nothing beyond the rollout that
already happened, and having them means a campaign under F2 can be read
against F1 and F3 afterwards without a re-run — which is the difference
between "we noticed the divergence" and "we would have had to pay for the
campaign twice to notice it."

F4 and F5 are recorded on every row **including rows where they are
`unmeasured`**, because the reason a column is empty is itself data: an
`f5_constructor_reuse` column that is uniformly `unmeasured` says the causal
layer never ran, which is a different campaign-level fact from a column
that is uniformly `0.0`.

`f2_reward` and `f2_penalty` are separate columns and never only their
difference: `docs/fitness_formulas.md` §3 flags penalty domination as a
live risk, and it is invisible in the sum.

**Regime occupancy** — eleven columns, `occ_frozen` … `occ_unknown`, in the
`Regime` discriminant order fixed in `docs/fitness_formulas.md` §3.1. Raw
window counts, not fractions; `W` is recoverable as their sum minus the
burn-in.

**Held-out metrics** — the ten columns of `docs/fitness_formulas.md` §5,
plus `kernel_held` (§5.1), same names.

**Engagement counters (v2)** — `docs/instrument_registry.md` §2. Every one
is a raw count, written on every row, and asserted non-zero before the
measurement it gates is admitted.

| Column | Type | Gates |
|---|---|---|
| `ei_intervals_emitted` | usize | `ei_micro`, `ei_macro`, `ce_structural`, `wl_class_count` |
| `task_lattice_entries` | usize | `task_lattice_depth` |
| `task_lattice_chains` | usize | composition rate — `undefined`, not `0.0`, at zero |
| `constructor_records_examined` | usize | F3, `mean_fidelity`, `replicator_rate` |
| `f5_records_examined` | usize | F5, `f5_top_share` |
| `f4_pairs` | usize | F4; `< PI_MIN_PAIRS` demotes it |
| `f1_windows_scored` | usize | F1 (windows where the evaluator emitted a score) |
| `probe_rows_collected` | usize | `kernel_held` |
| `distinct_policy_seeds_drawn` | usize | any seed-spread claim |

**Tripwires**

| Column | Type |
|---|---|
| `flag_fitness_structure_divergence` | bool |
| `flag_seed_overfit` | bool |
| `flag_degenerate_occupancy` | bool |
| `flag_degenerate_reuse` | bool (v2) |
| `flag_f4_degenerate_alphabet` | bool (v2) |
| `flag_instrument_not_engaged` | bool (v2) — set when any counter above read zero on this row |

**Provenance and cost**

| Column | Type | Notes |
|---|---|---|
| `final_canonical_hash` | u64 | the determinism check, per row |
| `n_nodes_initial` | usize | `DEGENERATE_OCCUPANCY` denominator |
| `n_windows_scored` | usize | `W`; `0` means the row is a failure row |
| `status` | string | `ok` \| `no_scored_windows` \| `sim_error` |
| `wallclock_ms` | u64 | |
| `budget` | usize | recorded per row: it is the load-bearing variable |
| `cap_bound_steps` | u64 | steps where the 512-candidate cap bound (`docs/policy_config.md` §4.4) |
| `pairings_cap_bound_steps` | u64 | steps where `max_pairings_per_step` bound (§2.3 there) |

The last two exist because both caps silently change what the experiment
means, and neither is visible in any other output. `pairings_cap_bound_steps
> 0` on a `k = ∞` arm means that arm is measuring the engine cap, not the
absence of a budget.

### 3.2 Failure rows are written, not dropped

A rollout that errors — `no_scored_windows`, or a `SimError` — writes a row
with `status` set and `fitness` empty. It is not silently omitted and it is
not scored `0.0`. A campaign where 30% of members died is a campaign with a
problem, and that fact must survive in the CSV rather than living only in a
log line. The optimizer treats a failure row as "no information from this
member" (excluded from rank shaping), which is different from "this member
scored badly".

---

## 4. `campaign_report.md`

Sections, in order. The first four mirror the run report's structure (v4
§11) so a reader who knows one knows the other.

1. **Campaign Parameters** — algo, population, sigma, alpha, generations,
   seeds per member, seed rotation, fitness function, horizon, budget,
   `fitness_formulas` version, `feature_set`, policy type, rule set,
   `campaign_seed`, git revision, `config_hash`.
2. **Fitness Trajectory** — best / mean / worst per generation, plus the
   champion's hold-out fitness per generation. Rotation and hold-out
   trajectories are plotted on the same axes; the gap between them *is*
   §6.2's overfit signal and should be readable without arithmetic.
3. **Arm Table** — the mandatory three-arm comparison (§4.2). Replaces
   v1's "Champion vs Baselines", which specified the control that could
   not catch the confound it was meant to catch.
4. **Held-out Metric Trajectories** — the ten metrics plus `kernel_held`,
   champion values per generation. This is where structure-vs-fitness
   divergence becomes visible before the flag fires.
5. **Suspicious Metric Flags** — mandatory, and mandatory even when empty
   (it then reads "No flags fired."). One subsection per fired flag: which
   generations, the numbers that tripped it, and a written interpretation.
   Also states explicitly when the campaign ran without a probe, so that a
   blank `kernel_held` column is never left to inference.
6. **Failure Census** — counts by `status`. Present even when all-ok.
7. **Cost** — total rollouts, total wallclock, steps/sec achieved, and the
   `campaign_hours` formula's realized value against its pre-campaign
   estimate.
8. **Precheck Record** (v2) — the P-1…P-4 results
   (`hypersim_nn_experiments_plan_v2.md` §4) with their measured values,
   not merely "passed". A campaign that ran is a campaign whose prechecks
   passed; recording *by how much* is what makes a marginal pass visible
   before it becomes a marginal result.

### 4.1 Flags are findings

A fired flag is written up and the campaign's results stand with the flag
attached. It is not an error, not grounds for suppressing the campaign, and
not grounds for retuning until it stops firing. These experiments
deliberately optimize against the evaluator; measuring how the evaluator
gets gamed is the point, not a side effect. See `docs/fitness_formulas.md`
§6.4.

### 4.2 The Arm Table — mandatory, and why v1's control was not enough

Every campaign and every measurement pass reports **three arms at matched
budget and matched seeds**:

| arm | policy | what it isolates |
|---|---|---|
| `random` | budget-matched `RandomSubset` | the budget effect alone |
| `untrained` | the policy class at initialization — θ=0, or class-default params | the **allocation-class** effect |
| `champion` | the trained champion | the **learning** effect |

The table gives, for each arm: **per-seed values** on the five frozen
hold-out seeds (not only the mean — a mean and an sd hide a bimodal arm),
mean ± sd, and then both margins with both σ figures:

```
champion − untrained   (± σ)      <- PRIMARY
champion − random      (± σ)
untrained − random     (± σ)
```

**`champion − untrained` is the primary reported statistic.** `champion −
random` is reported alongside it and is **never** the exit criterion for a
learning claim. A stage whose `champion − untrained` margin is within seed
noise reports **"no learning effect"** in its headline regardless of how
large `champion − random` is.

#### Why this is not a refinement

v1 §5.2 required comparison against budget-matched `RandomSubset`, "nothing
else." That exclusion is the defect. The policy *class* is an intervention
independent of any training: `StaticBias` allocates budget group-uniformly
across rules, `RandomSubset` allocates it binding-uniformly across
candidates. On a rule set where one rule dominates the candidate pool those
are very different dynamics **before a single gradient step**.

Measured consequence: on `wn_three` the champion beat budget-matched random
by `+0.7636` at `3.11σ`, 5/5 seeds, no Goodhart flags — v1's exit criterion,
met — and an **untrained** `StaticBias` at θ=0 scored identically.
`champion − untrained = +0.0000`. The contamination propagated to two more
stages: Stage 4's `+2.44σ` composition rate and Stage 5's `+2.84σ` `ei_macro`
are both `+0.28σ` and `+0.26σ` against untrained. **Three stages' headline
numbers described an allocation-policy difference, not learning.**

#### Class effects are findings, and must be labelled as such

The Stage 4 lattice result — group-uniform allocation across three rules
producing ~8× the tasks of binding-uniform sampling on a pool `binary_edge`
dominates — is a genuine, mechanistically explained result about
allocation. It is simply not a result about *learning*, and the report must
say which it is. A campaign report that presents a class effect without
naming it as one is malformed in the same way as one missing the untrained
arm.

#### Cost

One extra rollout set per campaign — the untrained arm needs no training,
only evaluation on the hold-out seeds. It is the cheapest control in the
programme, it raises campaign cost by roughly 50% at the measurement stage
and by nothing at the training stage, and it is non-optional.

**A campaign report without the untrained arm is malformed** and the
report writer refuses to emit one, in the same way §5.2's provenance
sentence is emitted rather than typed.

### 4.3 `unmeasured` is not `0.0`

Both the CSV and the report distinguish three states for every measurement:

| state | CSV | report | meaning |
|---|---|---|---|
| measured | the value | the value | the instrument engaged and produced this |
| `unmeasured` | the literal string `unmeasured` | `unmeasured`, never a dash or a blank | the instrument's engagement counter read zero (`docs/instrument_registry.md` §1) |
| absent | empty cell | stated in Suspicious Metric Flags | the measurement does not apply to this rollout — e.g. `kernel_held` with no probe |

A reader must never have to infer which of the three a blank means, and an
aggregate must never average across them: `mean(unmeasured, 0.4, 0.6)` is
not `0.333`, it is `0.5` over two values with the third reported as
suppressed. The report states the suppression count next to every mean.

This is the schema change plan v2 §6.1 identifies as catching six of the
eleven C0–C5 instrument bugs on its own. The concrete precedent: three arms
of `NaN` from an EI interval longer than the horizon, which would have read
as "shaping has no effect on EI", and a table of `0.0` from `[causal]` being
off, which would have read as "no policy affects constructor yield".

---

## 5. The contamination firewall

The evaluator-as-clipboard discipline keeps `hg-eval` from contaminating
substrate physics. A policy inverts that on purpose — it exists to shape
the dynamics. The firewall is what keeps the shaped data out of the
substrate record.

### 5.1 Path separation is structural, not conventional

`runs/substrate/` accepts only `policy = none` output. `hg-train` **cannot**
write there: there is no output-path key in `[training]`, the campaign root
is derived as `runs/cognition/<campaign_id>/`, and the derivation is not
config-reachable. This is checked by a unit test that asserts the derived
path starts with `runs/cognition/`.

Substrate-characterization baselines, regime-boundary maps, and every
constant feeding `docs/metric_formulas.md` are computed exclusively from
`runs/substrate/`.

### 5.2 Policy-active runs are self-identifying

Every validation run's report carries, in **Run Parameters**: the policy id,
the genome hash, and the campaign id. Its **Observations** section must open
with exactly:

> Dynamics in this run were shaped by selection policy `<id>`; regime labels
> describe the shaped system, not the substrate.

This sentence is emitted by the report writer, not typed by a human, so it
cannot be forgotten.

The corresponding manifest fields are `policy_id`, `genome_hash`, and
`campaign_id`, all `Option` and all `None` for substrate runs — an additive
manifest change, absent from every existing run's JSON.

### 5.3 The regime vocabulary stays shared

A policy-shaped run genuinely *is* in `GenerativeCandidate` if it classifies
that way. The classifier is not forked and the labels are not renamed. What
changes is provenance: **shaped classifications never update substrate
baselines.** The distinction is which corpus a number may join, not what the
number means.

### 5.4 No borrowed gate semantics

`ScienceLocked` is not overloaded, reused, or re-signalled by the cognition
layer. That event keeps whatever meaning v4's gate engine gives it.

---

## 6. Champion serialization

```toml
# runs/cognition/<campaign_id>/champions/gen_0042.policy.toml
schema_version = 1
campaign_id    = "lk_fuse_v1_20260725"
generation     = 42
policy         = "mlp_scorer"
feature_set    = "v1"
n_rules        = 6            # feature dim depends on it; a genome is
wl_buckets     = 16           # meaningless against a different rule count
budget         = 16
hidden         = 16
genome_hash    = 0x9f3c...
fitness_fn     = "f2_regime_occupancy"
holdout_fitness = 0.412
params         = [ ... ]      # flat f32 vector, exact round-trip
```

`genome_hash` is a **hex string**, not an integer. TOML integers are `i64`,
so a `u64` hash with the high bit set fails to serialize at all
("out-of-range value for u64 type") — which would make roughly half of all
champions unsaveable, and only on the campaigns unlucky enough to produce
one. `hg_cognition::genome::genome_hash_hex` is the single formatter, and
`genome::tests::a_high_bit_hash_still_serializes` pins it.

`params` is the flat vector produced by `Parameterized::n_params` /
`set_params`, written with enough precision for an **exact** `f32`
round-trip — the Phase C1 exit criterion is bit-exact genome round-trip,
and a champion that reloads to a slightly different vector reproduces a
slightly different run, which would quietly break the determinism contract
rather than loudly break a test.

`n_rules`, `wl_buckets`, and `feature_set` are recorded because the feature
dimension is `n_rules + wl_buckets + 7`. Loading a champion against a rule
set with a different rule count is rejected at load with a clear error, not
reshaped to fit. A genome trained under one feature layout has no meaning
under another.

---

## 7. Resume

`campaign_manifest.json` records `last_completed_generation` and the
optimizer state (θ for ES; mean, covariance, and step size for CMA-ES). An
interrupted campaign resumes from the last completed generation, appending
to `campaign.csv` — still exactly one CSV. Resume is a Phase C3 exit
criterion, not an afterthought: a campaign is hours long, and a crash at
generation 55 of 60 that forced a restart from zero would make the cost
model in the NN plan §12 a fiction.

Rows are flushed per generation, so an unresumed crash still leaves a
readable, complete-through-generation-N CSV.
