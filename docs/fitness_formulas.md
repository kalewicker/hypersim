# Fitness formulas — the learning layer's single source of truth

Companion to `docs/metric_formulas.md`, which governs the evaluator. This
document governs only what `hg-train` optimizes. The rule is the same one
v4 §4 imposes on the evaluator: **every constant a cognition stage
references traces to a formula in this file.** A stage that needs a number
not written here does not run until the number is written here.

Scope: the NN-experiments plan (`hypersim_nn_experiments_plan_v2.md`),
Stages 0–7. Nothing in this file may alter an evaluator output; fitness is
a *function of* `RolloutMetrics`, computed after the rollout, never during
it.

**Numbering note.** F4 and F5 (v2) are added as §4A and §4B rather than as
§5 and §6. `docs/campaign_output.md`, `docs/instrument_registry.md` and
several code comments cite this file's §5 (held-out metrics), §5.1
(`kernel_held`) and §6.3 (`DEGENERATE_OCCUPANCY`) by number. Renumbering
would silently invalidate every one of those citations, which is the same
failure mode §3.1 exists to prevent for `regime_occupancy`. Stable numbers
are worth an ugly heading.

---

## 1. Inputs

Every fitness function is a pure function of one `RolloutMetrics` value:

```rust
pub struct RolloutMetrics {
    pub final_canonical_hash: u64,
    pub windows: Vec<WindowMetrics>,      // one per evaluator window
    pub regime_occupancy: [u64; 11],      // Regime discriminant -> window count
    pub constructors: Vec<ConstructorSummary>,
    pub probe_rows: Vec<Vec<f32>>,
    pub n_nodes_initial: usize,           // store size before step 0 (§6.3)
    pub wallclock_ms: u64,
}
```

`n_nodes_initial` is an addition to the struct as sketched in the NN plan
§3; it is required by the `DEGENERATE_OCCUPANCY` tripwire (§6.3) and costs
one `usize` read at rollout start.

`WindowMetrics` carries the same quantities as `MetricsRow`
(`hg-schema::MetricsRow`); the rollout path computes them with the identical
evaluator code, it simply does not serialize them. Where a formula below
names a field (`interestingness_score`, `regime`, `n_nodes`), it means that
`MetricsRow` field, with that field's `docs/metric_formulas.md` definition.

### 1.1 Burn-in

```
burn_in_windows = B          (config [training].burn_in_windows, default 5)
scored_windows  = windows[B..]
W               = scored_windows.len()
```

**A rollout with `W == 0` is invalid, not zero-fitness.** It returns
`Err(FitnessError::NoScoredWindows)` and the campaign runner records the
member as failed rather than assigning it a number. Silently scoring an
empty window set as 0.0 would make "policy killed the run instantly"
indistinguishable from "policy performed badly", and the first is a bug
signal.

Rationale for burn-in at all: window 0 is always `Regime::Unknown` (no
`t-1`), the PI gate has a 4-window warm-up (`pi_warmup_windows`, metric
formulas §Threshold constants), and the type histogram needs several
windows before `new_type_bin_rate` is meaningful. `B = 5` clears all three.
`B` is a config knob, but changing it is a **fitness version bump** (§6).

---

## 2. F1 — mean interestingness

```
F1 = (1/W) · Σ_{w ∈ scored_windows} interestingness_score(w)
```

`interestingness_score` is defined in `docs/metric_formulas.md`
§Interestingness and computed by `hg_eval::classify::interestingness`. This
document does not restate it and must never redefine it — F1 is a *time
mean of an evaluator output*, nothing more.

Range: `[0, 1]` (the evaluator clamps per window; a mean of clamped values
is clamped).

Windows with `interestingness_score == None` (evaluator did not emit) are
excluded from both the sum and `W` for F1 only. If that leaves zero
windows, F1 is `Err(FitnessError::NoScoredWindows)`.

**F1 is the designated Goodhart specimen.** It is the richest signal and
the most gameable — a single scalar a policy can ride. It is retained as a
comparison arm precisely to measure how it gets gamed (plan §7.3). If F1
Goodharts instantly and uninformatively, it drops to a report appendix; it
does not become the primary.

---

## 3. F2 — target-regime occupancy (primary for Stages 2–3)

Let `occ(R)` = number of scored windows classified `R`.

```
target   = { ActivePersistentStructure, GenerativeCandidate }
penalized = { Exploding, WhiteNoiseSoup, Frozen }

F2 = (Σ_{R ∈ target} occ(R)) / W
   − LAMBDA_PENALTY · (Σ_{R ∈ penalized} occ(R)) / W
```

| Constant | Value | Meaning |
|---|---|---|
| `LAMBDA_PENALTY` | 0.5 | weight on the penalized-regime fraction |

Range: `[−0.5, 1.0]`. Not normalized to `[0,1]`: the asymmetry is
deliberate and readable — a policy that spends every window Exploding
scores −0.5, visibly worse than a policy that spends every window in
`StaticOrdered` (0.0, neither rewarded nor punished).

`Regime::Unknown` windows count toward `W` but are in neither set. A
policy cannot farm `Unknown`; it dilutes.

Why F2 is primary: the regime label is a *classification the evaluator
makes from multiple gate inputs* (`docs/gate_formulas.md`), not a single
scalar. Riding it requires satisfying a conjunction, which is a
meaningfully harder target than maximizing a weighted sum. This is a
hypothesis about gameability, not a guarantee — §5's tripwires exist
because it may be wrong.

**Known risk (plan §13.2):** if the penalty term dominates, ES will learn
timid policies that avoid penalized regimes without seeking target ones.
The campaign CSV therefore records `f2_reward` and `f2_penalty` as
**separate columns**, never only their difference, so the decomposition is
visible without a re-run. Rebalancing `LAMBDA_PENALTY` happens in this file
with a version bump (§6) — never as a config override.

### 3.1 Regime index order

`regime_occupancy[i]` is indexed by the `hg_schema::Regime` discriminant in
declaration order:

| i | Regime | in target | in penalized |
|---|---|---|---|
| 0 | `Frozen` | | ✔ |
| 1 | `StaticOrdered` | | |
| 2 | `GrowthTreadmill` | | |
| 3 | `WhiteNoiseSoup` | | ✔ |
| 4 | `SterileChaos` | | |
| 5 | `Exploding` | | ✔ |
| 6 | `FragmentedIsolation` | | |
| 7 | `Monoculture` | | |
| 8 | `ActivePersistentStructure` | ✔ | |
| 9 | `GenerativeCandidate` | ✔ | |
| 10 | `Unknown` | | |

This table is load-bearing: `[u64; 11]` is a positional array, and a future
`Regime` variant inserted anywhere but the end silently reindexes every
published campaign CSV. A unit test in `hg-train` asserts this mapping
against `Regime`'s own discriminants, so the reindex fails the build
instead of the science.

---

## 4. F3 — constructor yield (Stage 6)

```
F3 = n_confirmed_constructors · mean_fidelity
```

where `n_confirmed_constructors` counts `ConstructorSummary` entries
confirmed at rollout end (the registry's own confirmation criterion —
`hg_causal::constructor`, not a threshold introduced here), and

```
mean_fidelity = (1/n) · Σ fidelity_i     (0.0 when n == 0)
```

`fidelity` is the registry's `confirmed_cycles / total` ratio. F3 is `0.0`
when no constructor is confirmed — this is a genuine zero, not a missing
value, so unlike §1.1 it is scored rather than errored.

**`is_replicator` is reported alongside and NOT rewarded in v1.** Rewarding
the replicator flag would optimize against a boolean the causal layer sets
from a small number of observations; the flag rate is a held-out metric
(§5) so that Stage 6 can ask whether F3 produces replicators *without*
having asked for them.

Range: unbounded above (`n` grows with the run). F3 is therefore **not
comparable across horizons** — every F3 comparison must be at matched
`horizon_steps`. The campaign report states the horizon in every F3 table.

---

## 4A. F4 — predictive information (v2)

Plan v2 §5.1 diagnoses the C0–C5 negative as an *objective* failure: every
formula in §§2–4 rewards varied structure, uniform selection maximizes
variety, so uniform is the optimum of the objective and not merely the
optimum of the policy class. F4 and F5 are the two replacement objectives,
and §5.2 constrains them: **specified in structure, not in content.**
Neither may name what should be built.

### 4A.1 Correction to the plan's premise

Plan v2 §5.3 states that the PI proxy "already exists in
`docs/metric_formulas.md`". It does not. What exists there is
**persistent** information —

```
pi_normalized = Σ_x min(p_t(x), p_{t−1}(x))
```

— the type-mass overlap between consecutive windows, which
`docs/metric_formulas.md` §PI proxy describes in its own words as scoring
near 0 for white noise and **1 for frozen structure**.

That is the opposite of the property §5.3's argument depends on. The
argument is that PI "requires entropy **and** predictability jointly", so
that a frozen system scores zero and a maximally varied system scores zero
and the peak sits at structured-but-varied. `pi_normalized` has no entropy
term at all: it is maximized by doing nothing. An F4 built on it would be
a diversity objective run backwards — degenerate at the frozen end instead
of the uniform end — and would fail P-3 for a new reason rather than
escaping the old one.

So F4 is defined here from scratch, as a **lag-1 mutual information on the
window process**, which is the quantity §5.3's argument actually describes.
The evaluator's `pi_normalized` is untouched and stays what it is.

### 4A.2 Symbolization

F4 is computed over scored windows (§1.1), on a fixed 4-symbol alphabet:

```
j_w  = type_histogram_jsd(w)                      ∈ [0, 1]
s_w  = min(PI_ALPHABET − 1, ⌊PI_ALPHABET · j_w⌋)  ∈ {0 … PI_ALPHABET−1}

S    = (s_w)_{w ∈ scored_windows},     |S| = W
pairs = { (s_w, s_{w+1}) : w ∈ [0, W−1) },   N = W − 1
```

`type_histogram_jsd` is the choice because it is (a) already bounded to
`[0, 1]`, so one fixed binning is meaningful across rule sets whose raw
activity spans four orders of magnitude (`docs/policy_config.md` §2.5), and
(b) *per-window change* rather than per-window state, which is what makes
the frozen case exactly zero rather than merely small. A frozen substrate
emits `jsd = 0` every window, `S` is constant, and every quantity below is
identically 0 by construction rather than by luck.

Windows where `type_histogram_jsd` is `None` (the evaluator's first window,
and any window it did not emit) are **excluded from `S` before pairing**,
and the pairing then runs over the surviving sequence in order. This joins
two windows that were not originally adjacent; the alternative — breaking
the sequence at every gap — loses more pairs than the adjacency error costs
at these horizons. It is recorded in the row as `f4_gap_joins` so the
effect is visible rather than assumed.

### 4A.3 The statistic

```
Ĥ(X)  = − Σ_x p̂(x) log₂ p̂(x)              (plug-in, 0·log₂0 = 0)
Î(S)  = Ĥ(first coords) + Ĥ(second coords) − Ĥ(joint over pairs)

for r in 0 .. PI_SHUFFLES:
    S_r    = S permuted by ChaCha8Rng seeded fxhash64(final_canonical_hash, r)
    I_r    = Î(S_r)
μ_null = mean(I_r),   σ_null = sample sd(I_r)

F4 = max(0, Î(S) − μ_null)
```

| Constant | Value | Meaning |
|---|---|---|
| `PI_ALPHABET` | 4 | symbol count `Q`; ceiling on F4 is `log₂ Q` |
| `PI_SHUFFLES` | 64 | permutation-null replicates |
| `PI_MIN_PAIRS` | 50 | below this, F4 is `unmeasured` (§4A.5) |

**Why a permutation null rather than Miller–Madow.** The plug-in MI
estimator is biased upward by roughly `(K̂ − 1) / (2N ln 2)` bits, where
`K̂` is the number of occupied joint cells. At `Q = 4` and the horizons
these fixtures run (`W` in the tens), that bias is `~0.1–0.3` bits —
the same order as the signal, so an uncorrected `Î` would report
structure on an iid sequence. The shuffle null estimates that bias
**empirically, on this run's own alphabet, its own marginals, and its own
`N`**, which no closed form does. It is also the idiom the V-track already
uses for every CE claim, so the guard vocabulary (null spread, calibrated
zero) carries over unchanged.

`σ_null` is retained on the row (`f4_null_sd`) and is subject to the
null-spread guard: a null with no spread cannot support a margin, and the
V-track's arm-D result (`docs/validation_v7_l_track_findings.md`) is the
standing demonstration that a degenerate null produces an arithmetic
`p = 1.000` that reads as a measurement.

### 4A.4 Why F4 is not a diversity objective

`Î(S) = Ĥ(S′) − Ĥ(S′ | S)`. Raising it requires the marginal entropy term
**and** a conditional-entropy deficit at the same time:

| regime | `Ĥ(S)` | `Ĥ(S′|S)` | `Î` |
|---|---|---|---|
| frozen | 0 | 0 | **0** |
| iid across the alphabet | max | `= Ĥ(S′)` | **0** in expectation |
| structured and varied | > 0 | `< Ĥ(S′)` | **> 0** |

Uniform selection maximizes the first column. It has no mechanism for the
second. That is the whole of the argument for F4, and it is an argument
that uniform is not *automatically* optimal — **not** an argument that some
non-uniform policy can beat the substrate's natural `Î`. Plan v2 §5.3 flags
that gap explicitly and it is not closed here. P-3 is the test.

### 4A.5 Predicted ranges and engagement

Per plan v2 §6.2, every value is admitted only against a mechanism-derived
prediction written before the number is read.

| quantity | predicted range | mechanism |
|---|---|---|
| `F4` | `[0, log₂ PI_ALPHABET] = [0, 2]` | MI of a `Q`-symbol variable with itself is at most `log₂ Q`; the `max(0, ·)` closes the bottom |
| `F4` on a frozen arm | exactly `0` | `S` constant ⟹ `Î = 0` and every `I_r = 0` |
| `F4` on an iid arm | `[0, 3σ_null]` | `Î` and `μ_null` estimate the same quantity |
| `Î(S)` before correction | `[0, 2]` | same ceiling |
| `μ_null` | `(0, 2]`, strictly positive whenever `S` has ≥ 2 distinct symbols | finite-`N` bias is positive |

A `F4 > 2` or a `μ_null = 0` alongside a non-constant `S` is an
**instrument fault**, not a finding.

**Engagement counter: `f4_pairs = N`.** F4 is `Err(FitnessError::Unmeasured)`
when `N < PI_MIN_PAIRS`, and the campaign row records `unmeasured`, never
`0.0`. This is bug 7 (`ei_interval_steps = 1000` against a 384-step
horizon) generalized: an estimator whose sample is too small to estimate
anything must not emit a number, and the failure must be a distinct report
state rather than a plausible zero.

**Degenerate-alphabet flag.** When `S` has fewer than 2 distinct symbols,
`F4 = 0.0` is *measured* and correct (the frozen case), and the row sets
`flag_f4_degenerate_alphabet`. This is the one case where a zero is
admitted with no spread in the null, because the mechanism predicts it
exactly.

### 4A.6 Pre-registered failure readings

1. **F4 does not discriminate the policy class** (P-3 fails: spread over
   θ=0, allocation extreme, and random is under 2× seed σ). The substrate's
   unshaped dynamics already sit at their `Î` peak, or `Î` is insensitive
   to allocation. Read as: F4 joins F1–F3 as an objective selection-learning
   is unstateable against, and plan v2 §5.1's generalization strengthens
   from "diversity objectives" to "every objective expressible in this
   evaluator".
2. **F4 discriminates only active from inert.** If every live arm scores
   comparably and the only line F4 draws is against a do-nothing arm, F4 is
   answering a coarser question than the policy question and does **not**
   relocate the C-2 verdict. This is pre-registered because the V-track hit
   exactly this shape once already — its validated metabolic channel
   excluded nothing in the ladder and supplied a coarser account alongside
   the structural verdicts rather than replacing them
   (`docs/validation_v9_n_track_findings.md`). An objective that separates
   live from dead is not an objective that separates policies.
3. **F4 discriminates but is Goodharted immediately** — a champion raises
   F4 while `mean_active_persistent` and `archive_admission_rate` fall.
   Read via `FITNESS_STRUCTURE_DIVERGENCE` (§6.1) as with F1; the result
   stands with the flag attached (§6.4).

---

## 4B. F5 — constructor reuse (v2)

```
C        = { c ∈ constructors : confirmed at rollout end
                                and confirmed_cycles ≥ 1 }
T        = Σ_{i ∈ C} cycles_i                      (total confirmed cycles)

F5       = ( Σ_{i ∈ C} fidelity_i · cycles_i² ) / max(1, T)
```

Read it as: draw one confirmed cycle uniformly from all `T` of them; F5 is
the expected fidelity-weighted cycle count of the constructor that cycle
belongs to. It is a **reuse depth**, not a yield.

### 4B.1 Why F5 is not a diversity objective — exactly

Hold `T` fixed and `fidelity ≡ 1`. Then:

| allocation of `T` cycles | F5 |
|---|---|
| all on one constructor | `T² / T = T` |
| one each over `T` constructors | `T · 1 / T = 1` |

F5 spans `[1, T]` at fixed total yield and is maximized by **concentration**.
F3 over the same two configurations gives `1 · 1 = 1` and `T · 1 = T` — the
exact reverse. **F3 and F5 are opposed at fixed yield**, which is what
makes F5 a real test of the §5.1 diagnosis rather than a re-skin of it: a
policy cannot satisfy both by finding more structure, it has to choose
between spreading and repeating.

It also satisfies §5.2's constraint. F5 names no target structure; it says
only that *whatever* the substrate builds should be built again. That is a
statement about the shape of the dynamics, not about their content.

### 4B.2 Engagement — the §5.4 gate, and bug 8

**Engagement counter: `f5_records_examined = |constructors|`**, counted
*before* the confirmed filter.

`RolloutMetrics::constructors` is empty when `[causal]` is disabled. Under
v1 that made F3 identically `0.0` for every arm and produced a table of
zeros that reads as "no policy affects constructor yield" (plan v2 §6.1,
bug 8). F5 would inherit the identical failure. So:

- `f5_records_examined == 0` ⟹ F5 is `Err(FitnessError::Unmeasured)` and
  the row records `unmeasured`. It is **not** `0.0`.
- `f5_records_examined > 0` with `C` empty ⟹ F5 is a genuine `0.0`: the
  causal layer looked and found no confirmed constructor.

Those are different facts and the schema must keep them different.

Plan v2 §5.4 additionally requires that non-zero constructor yield be shown
to exist under *some* policy on the target rule set, **on a real
(non-fixture) rule set**, before F5 is stateable there. That is P-3 applied
to F5 and it is checked in Stage 2′, not here.

### 4B.3 Range, horizon, and predicted range

| quantity | predicted range | mechanism |
|---|---|---|
| `F5` | `[0, max_{i∈C} cycles_i]` | `fidelity ≤ 1`, and `Σ f·c² / Σ c ≤ max c` by the weighted-mean bound |
| `F5` | `≥ 1` whenever `C` is non-empty and `fidelity ≡ 1` | the spread-maximal configuration is the floor |
| `f5_top_share` | `[1/|C|, 1]` | it is a share |

`F5 > max_i cycles_i` is arithmetically impossible and is an instrument
fault. This is a **hard** predicted-range check — it needs no calibration
run and it fires on the class of bug that produced `2.9e16σ`.

F5 is unbounded above and grows with the run, so — exactly as for F3 —
**F5 comparisons are only valid at matched `horizon_steps`**, and every F5
table states the horizon.

### 4B.4 Burn-in does not apply

F3 and F5 read the constructor registry's end-of-rollout state, not the
window sequence, so `burn_in_windows` has no effect on either. `W == 0`
still invalidates the rollout under §1.1 — a rollout that produced no
scored window is malformed regardless of which fitness is active — but
`B` does not enter the formula.

### 4B.5 No penalty term, and what replaces it

Neither F4 nor F5 carries a penalty term. F2 has one because regime
occupancy has *named bad regimes* to penalize; F4 is already zero at both
degenerate ends by construction (§4A.4), and F5's degenerate optimum —
one constructor cycling forever — is precisely the thing it was built to
find, so penalizing it would defeat the objective.

What F5 does need is a way to tell "one constructor cycling forever on a
living graph" from "one constructor cycling forever on a graph the policy
collapsed to nothing". That is a tripwire, not a penalty: `DEGENERATE_REUSE`
(§6.5).

### 4B.6 Pre-registered failure readings

1. **F5 is identically `unmeasured` on real rule sets** — constructors are
   a fixture phenomenon. Read as: F5 is not an objective on this substrate
   family at all, and plan v2 risk 2 realized. This is a fact about the
   substrate's constructor yield, not about learning, and it must be
   reported as such rather than as a null result for policies.
2. **F5 is measured but does not discriminate the class** (P-3 fails).
   Same reading as F4 failure 1 — the generalization strengthens.
3. **F5 discriminates and `DEGENERATE_REUSE` fires on the champion.** The
   champion found the objective's degenerate optimum. Per §6.4 this is a
   finding — it is the most direct evidence available that "repeat what you
   build" is gameable by building almost nothing — and it is written up
   with the flag attached, not tuned away.

---

## 5. Held-out metrics — recorded every rollout, never optimized

Written to the campaign CSV for every rollout, for every fitness function,
including the ones that do not use them. Optimizing any of these is a
version bump and a rename, not a config change.

| Metric | Source | Why held out |
|---|---|---|
| `growth_exponent_nodes` | OLS slope of `log n_nodes` vs `log step` over scored windows | detects size-driven fitness |
| `growth_exponent_edges` | same for `n_edges` | as above |
| `mean_type_entropy` | mean `type_histogram_entropy` | structural diversity independent of the interestingness weighting |
| `mean_type_jsd` | mean `type_histogram_jsd` | distributional churn |
| `mean_mean_degree` | mean `mean_degree` | fragmentation check |
| `mean_active_persistent` | mean `active_persistent_structures` | the thing F2 is *supposed* to be a proxy for |
| `mean_static_persistent` | mean `static_persistent_motifs` | the static/active split — F2 must not be satisfied by static motifs |
| `archive_admission_rate` | `Σ new_signatures / W` | novelty independent of the novelty term in F1 |
| `mean_n_nodes` | mean `n_nodes` | the degenerate-occupancy denominator (§6.3) |
| `replicator_rate` | `n_replicators / max(1, n_constructors)` | see §4 |
| `f5_top_share` | `max_{i∈C} cycles_i / max(1, T)` | v2. F5's concentration made visible: `1.0` means a single constructor owns every confirmed cycle, which is F5's degenerate optimum (§4B.5) |
| `f4_null_sd` | `σ_null` from §4A.3 | v2. A zero here with a non-constant symbol sequence is an instrument fault, and a small one means the F4 margin has no null to be measured against |
| `f4_pairs` | `N = W − 1` after gap removal | v2. F4's engagement counter (§4A.5); `< PI_MIN_PAIRS` demotes F4 to `unmeasured` |
| `f5_records_examined` | `\|constructors\|` before the confirmed filter | v2. F5's engagement counter (§4B.2); `0` demotes F5 to `unmeasured` and is the direct guard against bug 8 |

The NN plan §7.2 lists "component count" among the held-out metrics. The
evaluator does not compute a general connected-component census — the only
component code in `hg-eval` is the chain/handle census, which is specific to
the leakage track's wormhole subgraph and is not defined on arbitrary rule
sets. `mean_mean_degree` stands in: it is the quantity the
`FragmentedIsolation` classification actually uses
(`mean_degree_frag_max = 0.5`), so it detects the same failure the plan was
reaching for using a number the evaluator already produces. A true
component count would be a new evaluator metric, and adding one to serve a
held-out column is exactly the contamination the firewall forbids.

The pairing `mean_active_persistent` / `mean_static_persistent` is the
sharpest of these: a policy that achieves target-regime occupancy by
producing *static* repeated motifs rather than active persistent structure
has gamed the classifier, and the two columns show it side by side.

### 5.1 Kernel-held metric

One further metric is computed inside `hg-train` from `probe_rows` and
written to the campaign CSV under the column name `kernel_held`. Its
definition lives in Appendix A of this file and **nowhere else** — in
particular it is never nameable from `[cognition]` or `[training]` config,
so no campaign can select for it and no policy author can see it while
designing features.

This is the near-term stand-in for the tri-ecology's kernel-held hidden
tests, and the pattern is the point: a metric that is measured but not
addressable. It is documented rather than secret — the discipline is that
it is not *configurable*, not that it is unknown to the reader.

---

## 6. Goodhart tripwires — computed by `hg-train`, mandatory report section

Each flag is a named constant here and a boolean column in the campaign
CSV, in addition to its appearance in the report's Suspicious Metric Flags
section (`docs/campaign_output.md` §4).

### 6.1 `FITNESS_STRUCTURE_DIVERGENCE`

Over any trailing window of `DIVERGENCE_GENS = 10` generations:

```
fires iff  (best_fitness[g] − best_fitness[g−10]) / |best_fitness[g−10]| > 0.25
      AND  every held-out structure metric in
           { mean_type_entropy, mean_active_persistent,
             archive_admission_rate, mean_type_jsd }
           has trailing-10 slope ≤ 0
```

Uses the champion's hold-out-seed values, not rotation seeds. If
`best_fitness[g−10] == 0`, the ratio test is skipped and the flag does not
fire (an absolute-improvement variant would need a scale this file cannot
supply for three different fitness ranges).

### 6.2 `SEED_OVERFIT`

```
fires iff  mean_fitness(rotation seeds S_g) − mean_fitness(holdout_seeds)
           > 2 · sd(fitness over holdout_seeds)
```

Evaluated per champion, at the generation the champion was recorded.
`holdout_seeds` are the frozen five in `[training]`; they are disjoint from
every `S_g` by construction (§7).

### 6.3 `DEGENERATE_OCCUPANCY`

```
fires iff  target-regime fraction ≥ 0.5
      AND  mean_n_nodes < COLLAPSE_FRACTION · n_nodes_initial
```

| Constant | Value | Meaning |
|---|---|---|
| `COLLAPSE_FRACTION` | 0.25 | shrink past this much of the IC and the occupancy is suspect |

**Note on provenance.** The NN plan's §7.3 phrased this tripwire as
occupancy "below the Frozen gate's floor". There is no such floor: the
`Frozen` classification is `activity == 0 && type_histogram_jsd == 0 &&
type_histogram_entropy > 0 && active_persistent_structures == 0`
(`hg_eval::classify`, `docs/metric_formulas.md` §Regime classification) —
purely dynamical, with no node-count term anywhere in it. The only absolute
node constant the classifier owns is `node_cap = 100 000`, and that is a
ceiling for `Exploding`, not a floor.

So this tripwire needs a floor of its own, and it is defined **relative to
the run's own initial condition** rather than as a new absolute. An
absolute floor would have to be chosen per rule set — the substrate rule
sets in `rules/` start at wildly different sizes — and would silently
misfire on every one it was not chosen for. `n_nodes_initial` is the node
count of the store as built, before step 0, recorded in `RolloutMetrics`
for exactly this purpose.

The flag means: *the policy achieved target-regime occupancy by shrinking
the graph to a quarter of what it was handed.* That is a claim about the
policy, and it is checkable without knowing anything about the rule set.

### 6.4 Flags are findings

A fired flag is written up in the campaign report and the campaign's
results stand with the flag attached. It is not an error, not a reason to
suppress the campaign, and not a reason to retune until it stops firing.
These experiments are deliberately optimizing against the evaluator;
**measuring how the evaluator gets gamed is a first-class result** and is
the most direct empirical input the tri-ecology design will get.

Retuning a fitness constant *because* a flag fired, without saying so, is
the one move this file forbids outright. Retuning it and saying so is a
version bump (§7).

### 6.5 `DEGENERATE_REUSE` (v2)

```
fires iff  f5_top_share ≥ TOP_SHARE_DEGENERATE
      AND  mean_n_nodes < COLLAPSE_FRACTION · n_nodes_initial
```

| Constant | Value | Meaning |
|---|---|---|
| `TOP_SHARE_DEGENERATE` | 0.9 | one constructor owning ≥ 90% of confirmed cycles |
| `COLLAPSE_FRACTION` | 0.25 | shared with §6.3 — same meaning, same denominator |

The flag means: *the policy maximized reuse depth by collapsing the graph
until only one thing was left to reuse.* It is the F5 analogue of §6.3 and
it is deliberately built from the same two ingredients — a concentration
share and the run's own initial node count — so that neither tripwire needs
a rule-set-specific constant.

Note what it does **not** fire on: high `f5_top_share` on a graph that
stayed its own size. That is F5 working. Concentration is the objective;
concentration *purchased by collapse* is the failure, and the conjunction is
what separates them.

---

## 7. Versioning

The fitness definition carries a version string written into every campaign
manifest and every campaign CSV header comment:

```
fitness_formulas = "v2"
```

Any change to a formula, a constant in §3, §4A, §4B or §6, the held-out
list in §5, or `burn_in_windows`' default bumps this version **and** the
change is recorded in §8's changelog. Campaign results computed under
different fitness versions are never compared in the same table without
both versions printed in that table.

**v1 → v2 is purely additive and every C0–C5 number stays comparable.**
F1, F2 and F3 are byte-for-byte unchanged, as are `LAMBDA_PENALTY`, `B`,
the §5 held-out metrics that existed in v1, and the three v1 tripwires. v2
adds two fitness functions, four held-out columns, and one tripwire that
can only fire under F5. A v1 campaign re-scored under v2 produces identical
F1/F2/F3 values; this is asserted by a unit test rather than argued
(`fitness::tests::v2_is_additive_over_v1`).

The evaluator's formulas stay frozen for the entire span of the NN plan
(plan §14). A moving fitness function would make every stage's comparison
meaningless; a moving *evaluator* would make every stage's comparison
meaningless twice over.

---

## 8. Changelog

- **v1** (2026-07-25) — initial. F1/F2/F3, `LAMBDA_PENALTY = 0.5`,
  `B = 5`, ten held-out metrics, three tripwires, kernel-held metric in
  Appendix A.
- **v2** (2026-07-27) — additive, per plan v2 §5.5. Adds **F4**
  (predictive information, §4A: `PI_ALPHABET = 4`, `PI_SHUFFLES = 64`,
  `PI_MIN_PAIRS = 50`) and **F5** (constructor reuse, §4B). Adds four
  held-out columns (`f5_top_share`, `f4_null_sd`, `f4_pairs`,
  `f5_records_examined`) and one tripwire (`DEGENERATE_REUSE`, §6.5,
  `TOP_SHARE_DEGENERATE = 0.9`). Introduces `FitnessError::Unmeasured` as
  a report state distinct from `0.0` (§4A.5, §4B.2). F1–F3 unchanged and
  frozen.

  **Correction carried in §4A.1:** plan v2 §5.3 cites an existing PI proxy
  in `docs/metric_formulas.md`. That proxy is *persistent* information and
  scores 1.0 on frozen structure — the opposite of the property the plan's
  argument for F4 requires. F4 is therefore defined here as a lag-1 mutual
  information on the window process, and the evaluator's `pi_normalized`
  is left untouched.

---

## Appendix A — the kernel-held metric

**Definition.** `kernel_held` is the **rank of the probe-row matrix**,
numerically thresholded:

```
X = probe_rows as a (n_probe_windows × probe_dim) f64 matrix,
    taken over scored windows only (post burn-in)
kernel_held = rank_eps(X) / min(n_probe_windows, probe_dim)     ∈ [0, 1]

rank_eps(X) = #{ σ_i : σ_i > EPS_RANK · σ_max }
EPS_RANK    = 1e-6
```

computed by SVD (`nalgebra`, already in the stack).

**What it measures.** The effective dimensionality of the state the probe
observes. A substrate whose probe rows all lie in a two-dimensional
subspace is producing a trajectory that no readout can decode richly,
however good its interestingness or regime occupancy looks. It is the
reservoir-computing quantity most directly upstream of Stage 1's memory
capacity, which makes it a natural check on Stages 2–3: a champion that
raises F2 while collapsing `kernel_held` has narrowed the substrate's state
space to a corner the classifier happens to like.

**Why held in the kernel.** Precisely because it is a good metric. If it
were addressable from `[cognition]`, it would become a fitness term, and
then it would stop being an independent check on the fitness terms. It is
recorded on every rollout, plotted in every campaign report, and optimized
by nothing.

`kernel_held` is `None` (empty CSV cell) when the rollout had no probe —
probes are optional (`RolloutRequest.probe`), and Stages 2–3 may run
without one. When a campaign's probe is absent, the campaign report says so
in the Suspicious Metric Flags section rather than leaving the reader to
infer it from a column of blanks.
