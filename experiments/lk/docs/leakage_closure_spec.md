# Closure-Defect Spec (LK4)

Normative companion to hypersim_leakage_plan_v1.md §1.4/§7. The estimator
lives in `crates/hg-causal/src/closure/`; the feature layouts below are
FROZEN — changing either is a spec revision (update this file, the field
lists in `closure/features.rs`, and the extraction in the runner together).

## Definition

CD = I( macro(w+1) ; micro(w) | macro(w) ) in bits/window, estimated by a
ridge predictor pair (hand-rolled normal equations, standardized features,
unpenalized intercept, λ = 1e-2) under blocked time-series CV (5 contiguous
folds, never shuffled). Gaussian residual model with per-output-dimension
train-residual variances (floored at 1e-9). CD = mean excess test
log-likelihood per window / ln 2.

- **Windows**: one sample per leakage emission (`[leakage] cadence_steps`),
  so the cadence is part of the metric's definition (same caveat as EI).
- **Certainty**: `Inferred` at ≥ `closure_min_windows` (default 200), else
  `Unknown` — emitted either way, never trusted below the floor.
- **Negative CD**: estimator pathology (finite-sample overfit of the
  augmented model), reported via `pathology_negative_cd` and the
  `closure_defect_negative` suspicious flag; never a finding.
- Requires evaluator + leakage both enabled (regime/activity come from the
  evaluator; detour histogram from the LocalityObserver).

## macro(w) — 19 dims (RE-FROZEN by plan v2 §1.4; next change = plan revision)

| slot | field | source |
|---|---|---|
| 0 | d_eff | shared ball-profile fit (0.0 when absent, documented imputation) |
| 1 | cut_exponent | LocalityObserver (0.0 when absent) |
| 2 | wormhole_frac | LocalityObserver |
| 3 | wormhole_components | LocalityObserver (v2 phase-plane coordinate) |
| 4 | area_law_defect | LocalityObserver (v2) |
| 5 | dim_mismatch | reserved (0.0 until the spectral observer lands) |
| 6 | activity | evaluator WindowFeatures |
| 7 | wl_class_count | distinct classes in the shared same-window WL map |
| 8–18 | regime one-hot | 11 variants, order: Frozen, StaticOrdered, GrowthTreadmill, WhiteNoiseSoup, SterileChaos, Exploding, FragmentedIsolation, Monoculture, ActivePersistentStructure, GenerativeCandidate, Unknown |

v2 §9 extension: CD is interpreted JOINTLY with the phase-plane position —
a dissolved-quadrant CD is not comparable to a sparse-quadrant CD.

## micro(w) — 28 dims

| slots | field |
|---|---|
| 0–5 | detour histogram fractions: [1,2], (2,4], (4,8], (8,12], (12,r_max], ∞ |
| 6–21 | top-16 WL-class frequency vector (fractions of n, descending, zero-padded) |
| 22–24 | degree moments: mean, var, skew (clique-adjacency distinct-neighbor degree) |
| 25–27 | component-size top-3 (fractions of n, union-find over hyperedges) |

**Clock-property exclusion (plan v6 §1.3, by name).** No node property
enters micro(w): the WL-class frequencies (slots 6–21) use WL hashes,
which exclude the per-key property map (the ALife Q2 invariant,
`docs/property_registry.md`), and slots 0–5 / 22–27 are purely
structural. In particular the v6 **countdown** key (7) — a monotone
function of time by construction — is excluded, and so are the window
index and any cumulative counter; admitting one would inflate CD_raw by
exactly the confound the CD_clock baseline controls. Pinned by
`micro_features_ignore_countdown_and_all_properties`
(crates/hg-causal/tests/closure.rs): two snapshots identical in structure
but differing in countdown (and other property) values produce
byte-identical micro(w).

## The tuple protocol (plan v3 §1.2, FROZEN 2026-07-15)

CD is reported as a TUPLE, never a scalar:
`(CD_raw, CD_clock, n_windows, seed_spread)`.

- **CD_raw** = excess of macro+micro over macro-only (the v1 scalar).
- **CD_clock** = excess of macro+micro over **macro+clock**, where clock
  = window index + phase one-hot (`clock(w)` — 4 dims, layout frozen by
  `CLOCK_FIELDS`: window_index, phase_onehot[3] in Sparse/Crossover/
  Dense order). On a nonstationary trajectory ANY monotonically trending
  micro feature beats the macro baseline by telling time; it cannot beat
  the clock. **Interpretation requires CD_clock; CD_raw alone may not be
  quoted in findings** (v3 hazard 4: this binds every CD consumer,
  including EXP-L7 reruns).
- **seed_spread** = max − min of CD_clock over the seed set;
  campaign-level (null in per-run reports). A CD claim requires
  signal > spread.

  **SCOPE CORRECTION (v8 B2, 2026-07-30): on this substrate seed_spread
  is an INSTRUMENT-NOISE floor, not a replication.** Under MaxParallel
  the seed does not vary the dynamics — conflict resolution is a
  deterministic greedy pass over a canonical key and no LK-R rule draws
  RNG, so a multi-seed sweep re-observes ONE trajectory. Measured
  directly: the step, energy, subdivision, fusion and churn accumulators
  are byte-identical across seeds 42/43/44 in both regimes
  (`exp_l18_seed_does_not_vary_the_dynamics`, asserted in both
  directions so it cannot pass vacuously). What the seed does reach is
  the sampled instruments — the 256-edge detour sampler feeding
  `detour_histogram[6]`, and the spectral estimates — so CD does vary
  across seeds, but only through measurement noise, with the dynamical
  component identically zero.

  The gate is **retained unchanged**: a signal that cannot clear the
  instrument's own sampling noise is certainly not a measurement, so
  `signal > spread` remains necessary. What changes is the language. It
  may not be described as replication, and clearing it does not license
  a claim that the result reproduces across independent runs of the
  dynamics — no such runs exist here. Producing them would require a
  stochastic step mode or varied initial conditions, neither of which is
  in the current protocol.
- **stratum** = `all` | `sparse_only`. The sparse-only fit keeps only
  transitions with BOTH windows Sparse (`closure_sparse_only.json`,
  written alongside); expected `Unknown` below the 200-window floor at
  current era lengths — recorded as such, and the certainty gate is
  load-bearing there (small-n strata produce arbitrarily large CD
  values that must never be read).

`closure.json` per run:

```json
{ "cd_raw_bits": 0.041, "cd_clock_bits": 0.017, "n_windows": 412,
  "seed_spread_bits": null, "stratum": "all", "cv_folds": 5,
  "macro_fields": [...], "micro_fields": [...], "clock_fields": [...],
  "certainty": "Inferred", "pathology_negative_cd": false }
```

Validation (crates/hg-causal/tests/closure.rs): macro-closed synthetic
CD ≈ 0 (correctly flagged); planted leak positive at 3σ across 10 seeds
under BOTH baselines (a genuine leak is not a clock); synthetic pure
monotone-trend micro: CD_raw > 0 with CD_clock ≈ 0 (the discriminator);
window floor; fold contiguity; flag-sign coupling; tuple schema golden.

## Negative-CD diagnosis (v3 §5.4, EXP-L11, 2026-07-15)

The p = 0 control arm's CD = −29.7 (flagged) was rerun under the tuple:
CD_raw = −29.7, CD_clock = −38.4 — the pathology DEEPENS under the
clock baseline. That sign pattern identifies it as a **blocked-CV
artifact under nonstationarity**, not mere variance: on a trajectory
whose feature distribution drifts monotonically, every held-out fold is
out-of-distribution for the training folds, and the 28-dim micro
augmentation has more capacity to fit fold-local structure that fails
to extrapolate — so macro+micro loses to the LOWER-capacity baselines
on test likelihood, and loses harder to macro+clock (which absorbs the
drift with 4 dims). Estimator variance is ALSO material — the 5-seed
p_peak spread is ~15 bits (see leakage_experiment_findings.md) — but
variance alone is sign-symmetric and cannot explain the control arm's
reproducibly negative CD. Consequence, per the v1 rule: negative CD
remains an estimator pathology and never a finding; nonstationary arms
additionally need the phase-stratified protocol once era lengths
support it.

## Window-floor verdict (v6 §5, A3, 2026-07-18)

`crates/hg-sim-core/tests/a3_window_floor.rs` re-windowed one 26k-step
run (transport_first q75t25, no gate — a nonstationary dissolution
trajectory) by dumping the per-window closure vectors at cadence 25
(`dump_closure_samples`) and subsampling to effective cadence
{25,50,100,200,500}, 3 seeds. Result (CD_clock mean ± cross-seed spread;
lag-1 autocorrelation of the top-WL-class micro feature):

| eff cadence | n_windows | CD_raw | CD_clock ± spread | autocorr₁ |
|---:|---:|---:|---:|---:|
| 25 | 1040 | −15.9 | −47.6 ± 1.8 | 0.997 |
| 50 | 520 | 4.9 | −65.9 ± 2.7 | 0.993 |
| 100 | 260 | 42.5 | −109.4 ± 12.7 | 0.985 |
| 200 | 130 | −65.6 | −359.6 ± 184.5 | 0.966 |
| 500 | 52 | 733.8 | −1399.7 ± 118.2 | 0.895 |

**Verdict: the floor is STATISTICS-limited, and the limiter is
autocorrelation, not raw window count.** Two facts settle it: (1)
CD_clock is pathologically NEGATIVE at every cadence — on a
nonstationary trajectory the 4-dim clock baseline absorbs the drift and
the 28-dim micro augmentation overfits, exactly the EXP-L11 diagnosis,
independent of cadence; (2) refining cadence multiplies raw windows
(52 → 1040) but lag-1 autocorrelation rises to 0.997, so adjacent
windows are near-identical and the blocked-CV folds are not independent
— the EFFECTIVE sample size does not grow with the window count.

Consequences, fixed before A4's CD protocol:
- **Re-windowing is NOT a legitimate route to satisfy the ≥ 200 floor.**
  The extra windows are autocorrelated; they do not buy independent
  evidence. The v6 §1.4 "window count, not era length" hope is refuted
  for this estimator on this substrate.
- **The floor is restated as an effective-sample requirement:** ≥ 200
  windows AND lag-1 autocorrelation low enough that the
  autocorrelation-corrected effective count (N·(1−ρ)/(1+ρ)) clears the
  floor. At ρ = 0.997 the 1040-window cadence-25 series has effective
  count ≈ 1.6 — decisively below the floor.
- **EXP-L7R needs genuinely longer STATIONARY dynamics**, not finer
  windows: a real steady state (A4 coexistence, if it exists) where the
  feature distribution does not drift, so decorrelated windows accrue.
  CD is not run on a nonstationary/frozen stratum regardless of how many
  windows it nominally has.

## EXP-L7R attempt 1 (v7 §5, Q7-5, 2026-07-19) — PRECONDITIONS NOT MET

**Outcome: CD remains unmeasured. The stratum is stationary and in
range, but its EFFECTIVE sample is ~5× short of the floor.** Recorded in
full because the shortfall is now quantified rather than merely
suspected.

Configuration: swap × transfer × k = 300 × E₀ = 4e5, 60k steps, cadence
100, 5 seeds; steady segment production-rate-defined (pre-quench only).
Driver `crates/hg-sim-core/tests/q7_5_exp_l7r.rs`. Audit exact in every
cell.

| seed | raw stratum | ρ (lag-1, micro) | **effective N** | P2 drift / var-ratio | verdict |
|---|---:|---:|---:|---:|---|
| 42 | 332 | 0.780 | **41.2** | 0.086 / 0.64 | fails P1 (effective) |
| 43 | 181 | 0.796 | **20.6** | 0.855 / 0.16 | fails P1, P2 |
| 44 | 351 | 0.793 | **40.7** | 0.079 / 0.51 | fails P1 (effective) |
| 45 | 347 | 0.797 | **39.1** | 0.014 / 0.50 | fails P1, P2 (boundary) |
| 46 | 225 | 0.755 | **31.6** | 0.530 / 5.16 | fails P1, P2 |

**Two preconditions held and one did not.** The good news is real: every
window of every run classified Sparse (600/600), so the coexistence
regime genuinely does not cross phases — the stationarity that all prior
CD attempts lacked. Seeds 42 and 44 also passed the P2 drift and
variance tests. What fails is the sample size, and it fails decisively.

**A correction to this round's own instrument.** The first version of the
driver implemented P1 as a **raw** window count (≥ 200 consecutive
windows) and, on that basis, authorized seeds 42 and 44 and printed
"SIGNAL ABOVE SPREAD (first valid leakage-in-bits measurement)" with
CD_clock = 3.06 and 2.78 bits/window. That verdict is **withdrawn**: the
A3 section above had already restated the floor as an *effective*-sample
requirement, and the driver did not implement the restatement. Applying
it, no seed clears 200 — the best is 41. The numbers are recorded here as
**not authorized** and are not a measurement of anything.

**What the shortfall costs, quantified.** Modelling the features as
AR(1), ρ = 0.78 at cadence 100 gives a correlation time τ ≈ 400 steps,
and the effective sample is `T/(2τ)` for a stratum of duration T —
**independent of cadence**, which is the same reason A3 gave that
re-windowing cannot help. Reaching an effective 200 therefore needs a
steady segment of

> T ≥ 200 · 2τ ≈ **1.4–1.7 × 10⁵ steps**, i.e. **≈ 5× longer** than the
> 3.3 × 10⁴ achieved here.

Since transfer mode's era scales as E₀/c, that is a budget of roughly
E₀ ≈ 2 × 10⁶ at these parameters — a well-posed, purely quantitative
requirement, and the first time the track has been able to state one.

**Diagnostic worth keeping (not a claim).** In these strata CD_clock
(3.06, 2.78) exceeded CD_raw (0.73, 1.00). That ordering is the opposite
of every nonstationary run in the track's history, where the clock
absorbed the drift and CD_clock collapsed or went negative. It is what a
genuinely stationary series should produce — a window index carries no
information in a steady state, so adding it only costs degrees of
freedom. The sign pattern is consistent with the stratum being what it
claims to be; it is not evidence about leakage, and is not quoted as
such.

## EXP-L7R attempt 2 (v8 §5, B3, 2026-07-31) — COMMISSIONED, PRECONDITIONS STILL NOT MET, AND THE REASON MOVED

**Outcome: CD remains unmeasured. The sample-size precondition PASSES for
the first time in the track; the stationarity precondition now fails —
and the failure is traced to a defect in the precondition itself, not to
the dynamics.**

Configuration: swap × transfer × k = 300 × **E₀ = 2e6**, 300k steps,
cadence 100, 5 seeds, **under the passive clock** (`gate = "age"`) — the
first CD attempt on dynamics no instrument is participating in. Audit
exact in every cell. Commissioned via the D3-A branch of the v8 §5
pricing table.

| seed | raw stratum | ρ | **effective N** | P2 drift / var-ratio | verdict |
|---|---:|---:|---:|---:|---|
| 42 | 1382 | 0.698 | **246** | 0.058 / **0.31** | fails P2 |
| 43 | 1380 | 0.694 | **249** | 0.072 / **0.35** | fails P2 |
| 44 | 1380 | 0.704 | **239** | 0.086 / **0.33** | fails P2 |
| 45 | 1377 | 0.700 | **243** | 0.066 / **0.34** | fails P2 |
| 46 | 1375 | 0.703 | **240** | 0.069 / **0.36** | fails P2 |

0 of 5 seeds authorized; the estimator did not run and no CD number
exists to quote.

### What the budget bought, and what the pricing got right

The attempt-1 requirement was met on its own terms. Effective N is
239–249 against the floor of 200, from raw strata of ~1380 windows at
ρ ≈ 0.70 — and ρ did **not** rise with segment length (0.722 at E₀ = 4e5
→ 0.694–0.704 here), which is the assumption the whole extrapolation
rested on. The B3 pilot's two projections bracketed the result: the
linear projection said 234, the measured-exponent correction said 183,
the truth was 239–249. **The sublinear correction was itself an
over-extrapolation from two points**, and is recorded as such — it was
the conservative error, but it was an error.

Price: 1567 s/seed measured against 1160 s/seed projected (×1.35). The
D3 branch (≤ 1 h/seed → commission) is unaffected, so the decision was
robust to the miss even though the number was not.

### Why P2 failed: the stratum mask is gated on the wrong instrument

Drift is small — mean census 295 → 313 over 138,000 steps, 5.8–8.6%,
comfortably inside the 25% bar. What fails is the **variance ratio**:
0.31–0.36 against a [0.5, 2.0] band, i.e. the second half is ~3× as
variable as the first.

Quartiles locate it precisely (seed 42; seed 43 identical in shape):

| quartile | mean | sd |
|---|---:|---:|
| 1 | 291.5 | 74.0 |
| 2 | 299.4 | 56.2 |
| 3 | 297.6 | 60.1 |
| 4 | **330.6** | **151.9** |

Three quarters of the segment are stationary. The fourth is not, and the
boundary that ends the stratum is diagnostic: at step 139,800 the mask
breaks because **`wormhole_count_est` falls to 0** — while
`n_subdivisions` keeps climbing (192,862 → 193,576 by step 300,000) and
the phase stays `sparse` throughout. Production does not stop. The phase
does not change. **The sampled census crosses its own detection
threshold.**

That is the Q7-1 instrument again. `wormhole_count_est` is a 256-edge
sample of a threshold-gated quantity — a filament link registers only
above a detour that requires chain length ≥ 10 in swap — and Q7-1
established it is *not* the instrument of record; the paper's §5 switched
to the gen>0 standing census for exactly this reason. The EXP-L7R stratum
mask, pre-registered in v7, was never updated with it. So as the true
population drifts toward the visibility threshold the sampled count
becomes progressively noisier (the quartile-4 variance blow-up), then
reads zero and terminates the stratum.

**The P2 failure is therefore an artifact of the gate, not a property of
the dynamics** — and it compounds with the v8 §0 finding that this census
is sampled, so its noise is seed-dependent with zero dynamical component.

### What is NOT done here, and why

The mask is not corrected and re-run in this round. Correcting an
instrument after seeing which way it made a gate fail is
post-hoc — even when the correction is right, and this one is right. The
same discipline retired the original CD headline and withdrew the v7
verdict.

Pre-registered for v9 instead:

1. **Stratum mask on the instrument of record.** Replace
   `wormhole_count_est > 0` with the gen>0 standing census > 0, computed
   exactly rather than sampled. Predicted effect: the stratum extends
   past step 140k, and the quartile-4 variance blow-up disappears because
   it was the sampler approaching threshold.
2. **Named alternative it excludes.** If the variance ratio still fails
   with the corrected mask, the destabilization is real — the coexistence
   steady state is not variance-stationary over ~10⁵ steps — and that is
   a finding about the regime, not about the instrument. It would also
   mean the P2 test has never been passed by a long segment, only by
   short ones, which changes what §6's stationarity milestone claims.
3. **No relaxation of either bar.** The [0.5, 2.0] variance band and the
   200 effective-sample floor stay exactly as they are.

**Standing position: CD is still unmeasured, for the third distinct
reason** (v6: no stationary regime; v7: effective sample 5× short; v8:
stationarity gate reading a threshold-limited sampled instrument). Each
time the obstacle has been identified and quantified rather than argued
around, and each time the previous one stayed fixed.

## C0 memo — what a CD error bar is over (v9 §2, GATE, 2026-07-31)

**Written before attempt 3 runs. The estimator's configuration is frozen
by this memo; nothing below may be adjusted after any attempt-3 output is
seen (v9 hazard 1).**

### The problem

Every CD report in this track has carried `seed_spread` as its
uncertainty, and v8 established that on this substrate the seed does not
vary the dynamics: conflict resolution is a deterministic greedy pass
over a canonical key, no LK-R rule draws RNG, and the torus is
vertex-transitive so seed *placement* is relabelled away. Five "seeds" are
one trajectory observed five times through a stochastic instrument. So
`seed_spread` has a dynamical component of identically zero, and the
quantity it has always measured is sampler noise. An error bar has been
reported for three rounds without a stated ensemble.

### Decision: single-trajectory CD, with two separately named uncertainties

CD is declared **a property of this deterministic trajectory**, not of the
LK-R family and not of a distribution over initial conditions. Two
uncertainties are reported, each naming its ensemble:

1. **`bootstrap_ci`** — a moving-block bootstrap over the stationary
   segment. This is the estimator's own sampling uncertainty for a single
   stationary series, and it is the number the decision rule uses.
2. **`sampler_spread`** — max − min of CD_clock across the five persisted
   runs. Renamed from `seed_spread` (v9 §1.2) because that is what it
   measures. Reported, never described as replication.

**Claim language, fixed here:** results are stated as "this trajectory's
closure defect", with the family-level claim explicitly out of scope.
Anything stronger requires the IC ensemble below.

### Pre-registered bootstrap parameters (frozen)

- **Resampling unit:** window transitions within the stationary segment,
  in their original order within a block.
- **Scheme:** moving-block bootstrap. Draw ⌈n/b⌉ blocks of `b` consecutive
  transitions, with replacement, from all `n − b + 1` start positions;
  concatenate; truncate to `n`; re-run the tuple estimator; repeat.
- **Block length `b` = 20 windows = 2000 steps.** Rationale fixed in
  advance: lag-1 feature autocorrelation is ρ ≈ 0.70, giving an AR(1)
  correlation time τ_w = −1/ln ρ ≈ 2.8 windows ≈ 280 steps, so b ≈ 7 τ —
  the standard "several correlation times" choice, and comfortably above
  the v8 plan's ≥ τ ≈ 400-step floor.
- **Pre-registered sensitivity triple:** b ∈ {10, 20, 40} windows, **all
  three reported**, with **b = 20 as the headline**. This is a robustness
  disclosure, not a selection: the headline block length is fixed here and
  does not move on the basis of which one gives a narrower interval.
- **Replicates B = 1000**, seeded deterministically (bootstrap draws are
  the one place in this track where RNG is deliberate; the seed is
  recorded so the interval reproduces).
- **Interval:** percentile, 2.5 / 97.5.

**Block joins carry no transitions (refinement, recorded).** The first
draft of this memo disclosed a limitation — that concatenating blocks
creates one spurious window-to-window transition at each join, ~5% of the
dataset at b = 20 — and accepted it as standard. During implementation,
before any estimator output existed, a clean fix turned out to be
available within the frozen estimator: `estimate_stratum` retains a
transition only when `mask[i] && mask[i+1]`, so emitting a **separator
window with `mask = false` between blocks** removes every join transition
while keeping all b genuine ones. Each block contributes `b + 1`
consecutive windows (mask true) followed by one separator (mask false),
yielding exactly b real transitions and zero spurious ones.

This is a strictly tighter implementation than the memo first described,
it changes no estimator code, and it was adopted **before any attempt-3
number was computed** — recorded here rather than silently, because the
rule this round operates under is that the configuration is frozen once
output is seen, not once it is written.

### Pre-registered decision rule for attempt 3

A CD claim requires **both**:

- the bootstrap 95% interval on CD_clock **excludes 0**, and
- the point estimate **exceeds `sampler_spread`**.

Requiring both keeps v7's "signal > spread" bar intact while adding the
interval the single-trajectory semantics demands. Failing either is the
v9 §3.2 "signal below uncertainty" row: recorded as the fourth
quantified obstacle, with the bars unmoved.

### The IC-ensemble alternative, and why it is deferred

Genuine dynamical replication needs a symmetry-breaker, and on a
vertex-transitive torus with zero-draw scheduling every available one —
planted defects at varied positions, paired seed sites, lattice-size
families — **changes the system rather than the realization**. An
ensemble built from them would answer a different question (how CD varies
across a family of systems) and would not be an error bar on this one.

**Disposition: deferred to a confirmation round after a single-trajectory
CD lands**, as a pre-registered consistency check — ensemble CD
compatible with the single-trajectory value within stated uncertainty —
and explicitly *not* as attempt 3's error bar. Recorded so that the
tradeoff is owned rather than discovered later.

## EXP-L7R attempt 3 (v9 §3.1–3.2, C1, 2026-07-31) — NOT AUTHORIZED; the v8 prediction is REFUTED

**Outcome: CD remains unmeasured, for a fourth distinct reason. The v8
pre-registered prediction failed, and the correction it prescribed —
though right, and necessary — was not sufficient.** Zero new compute:
everything below is the persisted E₀ = 2e6 runs re-analysed.

Driver: `crates/hg-sim-core/tests/c1_regate_attempt3.rs`.

### The corrected census, and why it was computable at zero cost

The gen>0 standing census needs no snapshots. The gen-0 lattice is
non-removable under the age gate (lk_fuse's generation guard, pinned by
`bg_lattice_stays_non_removable_though_it_passes_the_age_guard`), so

> census(w) = n_nodes(w) − 1024, exactly,

with `n_nodes` already in `metrics.csv`. Sanity-checked rather than
assumed: `n_nodes` never dips below 1024 in any of the five runs.

### §3.1 result — the prediction is refuted

| run | stratum old → new | steps covered | drift | var ratio | P2 | ρ | eff N |
|---|---|---|---:|---:|---|---:|---:|
| s42 | 1382 → 1437 | 400–144,000 | 0.078 | **0.17** | FAIL | 0.707 | 247 |
| s43 | 1380 → 1437 | 400–144,000 | 0.078 | **0.17** | FAIL | 0.705 | 248 |
| s44 | 1380 → 1437 | 400–144,000 | 0.078 | **0.17** | FAIL | 0.712 | 242 |
| s45 | 1377 → 1437 | 400–144,000 | 0.078 | **0.17** | FAIL | 0.710 | 244 |
| s46 | 1375 → 1437 | 400–144,000 | 0.078 | **0.17** | FAIL | 0.713 | 240 |

The prediction was that the quartile-4 variance inflation would
disappear, being an artifact of the sampled census growing noisy at its
detection floor, and that the variance ratio would enter [0.5, 2.0].
**Neither happened.** The ratio got *worse* (0.31 → 0.17), and the
quartile structure is not the one v8 inferred:

| quartile | mean | sd |
|---|---:|---:|
| 1 | 367.8 | **82.9** |
| 2 | 419.8 | 6.8 |
| 3 | 421.3 | 6.4 |
| 4 | 431.0 | **217.6** |

Two quartiles are inflated, not one, and the middle two are far tighter
than the sampled instrument ever showed (sd/mean ≈ 1.6%).

**What the exact census reveals — the diagnosis v8 could not reach.**
Reading the census trajectory directly:

- **steps 0–~30,000: a fill ramp**, census 0 → ~410, driven not by the
  lifetime (k = 300 steps) but by the *production rate* itself ramping up
  (p: 0 → 1.4/step) as energy spreads from the seed across the lattice.
  This is quartile 1.
- **steps ~30,000–125,000: a genuine plateau**, census 411–426, the
  tightest steady state the track has produced.
- **steps ~125,000–150,000: the quench.** Production decelerates as the
  budget empties (p 1.57 → 0.40 → 0) and the population drains to 0. This
  is quartile 4.
- **steps 150,000–300,000: dead.** Half the run is post-quench.

**So the fourth obstacle is this: the mask's `producing` clause tests a
rising counter, not a steady rate.** `n_subdivisions` keeps ticking up
throughout the quench, so the pre-registered mask admits both the fill
ramp and the drain, and the variance test then measures a transient
against a steady state. The sampled census had been *hiding* this by
truncating the stratum near step 144,000 for its own unrelated reason.

**v8's correction was right and is retained.** It is what made this
diagnosis possible: the exact census resolves a fill ramp, a plateau and
a quench that the threshold-limited sampler blurred into one series. It
simply was not the whole defect.

### §3.1's exclusion branch — also fails

The v8 pre-registration said that if the variance ratio still failed, the
destabilization is real and attempt 3 runs on the Q1–Q3 sub-stratum only
if effective N is independently re-cleared there. Executed as written:

| run | Q1–Q3 n | ρ | eff N | verdict | drift | var ratio |
|---|---:|---:|---:|---|---:|---:|
| s42 | 1077 | 0.584 | 283 | clears | 0.090 | **112.4** |
| s43 | 1077 | 0.585 | 282 | clears | 0.090 | **112.4** |
| s44 | 1077 | 0.587 | 280 | clears | 0.090 | **112.4** |
| s45 | 1077 | 0.588 | 279 | clears | 0.090 | **112.4** |
| s46 | 1077 | 0.589 | 278 | clears | 0.090 | **112.4** |

Effective N clears comfortably; **P2 fails far worse** — the sub-stratum
drops the quench but keeps the fill ramp, so the first half is a
transient (sd 82.9) and the second half is the plateau (sd 6.5).

### §3.2 — attempt 3 did not run

0 of 5 runs cleared the gate. P1 passes on every run (eff N 240–248,
floor 200) and P2 fails on every run. **No CD number was computed**, and
none is quoted. `sampler_spread` is undefined for an empty authorized
set.

### What is deliberately NOT done

The plateau (quartiles 2–3) is visibly stationary and would very likely
pass P2. **It is not adopted as a stratum in this round.** Selecting a
window after seeing where the variance is small is post-hoc window
selection — the error this programme has already paid for twice.

### Pre-registered for the next round (fixed here, before any re-analysis)

1. **Mask on a production *rate* plateau, not a rising counter.** Replace
   `n_subdivisions rising` with: the per-window production rate is within
   **±20%** of its median over the run's producing windows. The band is
   fixed here at 20% and does not move. Mechanistically motivated: a
   metered budget must quench, and a seeded lattice must fill, so both
   ends of any run are transients by construction — this is derivable
   from the mechanism without inspecting the variance.
2. **Prediction:** the stratum becomes the plateau (≈ steps 30k–125k,
   ~950 windows), the variance ratio enters [0.5, 2.0], and effective N
   stays above 200 (ρ ≈ 0.70 on ~950 windows gives ≈ 165 — **so this may
   fail the effective-N floor**, and if it does, the honest reading is
   that the E₀ = 2e6 budget buys a plateau that is stationary but too
   short, and the next budget is the fix).
3. **Named alternative it excludes:** if the plateau clears both gates and
   CD is then computed, a null result there cannot be blamed on
   nonstationarity or sample size — it would be an estimator-variance
   floor, the genuinely fourth obstacle in the sense §3.2 anticipated.
4. **No bar moves.** [0.5, 2.0], 200 effective samples, and the C0
   decision rule stand exactly as written.
