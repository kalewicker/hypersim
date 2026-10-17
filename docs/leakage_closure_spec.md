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
