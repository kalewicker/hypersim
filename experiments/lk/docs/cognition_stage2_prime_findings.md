# Stage 2′ — F4/F5 stateability, and the programme's go/no-go

Phase D5 of `hypersim_nn_experiments_plan_v2.md`, §9. Produced by
`crates/hg-train/tests/d5_stage2_prime_prechecks.rs` and
`crates/hg-train/tests/d5_f4_horizon_probe.rs`.

> **The exit criterion is met.** Two (objective, rule set) pairs clear all
> four prechecks: **F5 × `tri_two`** and **F4 × `wn_three`**. The §5.4
> constructor-yield gate passes decisively on both real substrates — 190–264
> confirmed constructors on `wn_three`, 47–58 on `tri_two` — retiring risk 2.
>
> **F5 × `tri_two` is the pair D6 should use.** F4 also passes, but part 4
> shows *what* it discriminates on: the arm F4 ranks highest on `tri_two`
> scores **1.001× its own sorted series**. On these substrates F4 is
> approximately a monotonicity detector, and a campaign maximizing it would
> be optimizing how smoothly the type histogram decays.
>
> Getting here required two instrument corrections, both of which fired on
> real data and both of which would have silently produced a wrong D5
> verdict.

---

## 1. Two corrections, before any verdict

### 1.1 F4's symbolization was collapsing every run to one symbol

v2 symbolized the window JSD onto a fixed `[0, 1]` grid, `floor(4 · jsd)`.
Measured, the type-histogram JSD never leaves the **first of those four
bins**:

| config | windows | pairs | distinct symbols | jsd range |
|---|---|---|---|---|
| `wn_three` 384/24 | 16 | 10 | **1** | 0.0116 – 0.0319 |
| `wn_three` 768/24 | 32 | 26 | **1** | 0.0051 – 0.0319 |
| `wn_three` 1536/24 | 64 | 58 | **1** | 0.0022 – 0.0319 |
| `wn_three` 384/12 | 32 | 26 | **1** | 0.0038 – 0.0294 |
| `wn_three` 384/6 | 64 | 58 | **1** | 0.0030 – 0.0852 |
| `tri_two` 384/6 | 64 | 58 | **1** | 0.0021 – 0.0226 |
| `growth_treadmill` 384/6 | 64 | 58 | **1** | 0.0031 – 0.0221 |

Every substrate the programme owns, at every horizon and window tried. The
series has sd 0.004–0.015 and the graph grows from 24 to 5992 nodes, so
these are not frozen runs — the quantizer simply had 100× the range it
needed.

Below `PI_MIN_PAIRS` that surfaced as `unmeasured`, which is why Stage I
reported F4 unmeasurable and diagnosed it as a *pair-count* problem
(`instrument_registry.md` §5.1). **That diagnosis was wrong.** Clearing the
pair bar does not make F4 informative; it makes F4 report a **measured
exact zero**, because the one-symbol branch was classified as the frozen
case. A substrate that grew 250× would have been published as carrying zero
predictive information.

**Fixed** (`fitness.rs`, `FITNESS_FORMULAS_VERSION` → `v2.1`): equal-count
(quantile) symbolization, the standard choice for mutual information on a
continuous channel, which cannot be defeated by the observable occupying a
narrow band. Two properties are preserved deliberately — a frozen substrate
still scores exactly 0 (constant JSD sends every cut to the same value), and
the marginals become uniform in every arm, so a P-3 spread is a difference
in temporal structure rather than in how far each arm's JSD range happened
to move.

The one-symbol branch is also now split on `f4_jsd_spread`: spread `0` is a
frozen substrate and a real `0.0`; spread `> 0` is a substrate that moved
and was not resolved, which is `unmeasured`. Under equal-count cutting the
second is unreachable, so it ships as a guard with a firing test rather than
as a live path — Rule 1 says an instrument that did not resolve does not get
to report a null, and that is exactly what v2 did.

After the fix, the same runs: `wn_three` 0.3218, `tri_two` 0.4603,
`growth_treadmill` 1.5988, against a ceiling of `log₂ 4 = 2`.

### 1.2 An empty constructor registry meant two opposite things

`constructor_reuse` keyed on `constructors.is_empty()`. That is bug 8's fix
over-firing in the other direction: an empty registry is *also* what a live
detector produces when there is nothing to detect, so every genuine zero was
being suppressed as a non-measurement.

It fired on real data inside this very phase. `tri_two` under the allocation
extreme returns an empty registry with `[causal]` **on** for 2 of 3 seeds,
so the arm was reported as `1.0000 ± 0.0000 (n=1, unmeasured=2)` and P-3
passed the cell on a mean computed from a single seed.

**Fixed**: `causal_enabled` is carried from the runner (`causal.is_some()`)
through `RunArtifacts` and `RolloutMetrics` into `Engagement`, and F3 and F5
gate on it instead of on a record count. The two cases are now distinguished
at the source rather than guessed from a length. The same arm now reads
`0.3333 ± 0.5774 (n=3, unmeasured=0)` — three measured seeds, one of which
found a constructor — and the cell passes on evidence.

`not_engaged()` reports `causal_enabled` rather than
`constructor_records_examined`, because a zero record count with the layer
live is a result.

---

## 2. The configuration, and why the expensive route was rejected

`384` steps, window `6`, burn-in `5` — **58 symbol pairs**, clearing
`PI_MIN_PAIRS = 50`.

`instrument_registry.md` §5.1 recorded two routes to enough pairs and argued
that the long-horizon one was principled because "the window is what a
symbol means". Measured, that argument does not survive:

| route | steps/window | pairs | trend | monotone ceiling | final nodes | secs |
|---|---|---|---|---|---|---|
| A — long horizon | 1536 / 24 | 58 | −0.877 | 1.5933 | 5992 | **96.6** |
| B — short window | 384 / 6 | 58 | −0.875 | 1.6059 | 1465 | **3.4** |

Route A costs **28×** for an **identical** drift confound, over a substrate
that grows four times as far — so its windows are comparing type histograms
of a much more strongly non-stationary object. And the comparability
argument is empty: F4 had never been measurable at window 24, so there is no
published number for route A to be comparable *to*.

Route B is used throughout.

---

## 3. The §5.4 constructor-yield gate

Plan §5.4 requires non-zero constructor yield under *some* policy on real
rule sets before F5 is stateable, and §11's risk 2 is that this fails
outright. `[causal]` on, `k = 4`, 384 steps:

| rule set | arm | records | confirmed | max cycles | F5 |
|---|---|---|---|---|---|
| `wn_three` | random | 305 | 190 | 1148 | 502.50 |
| `wn_three` | θ=0 | 497 | **264** | 1115 | 435.59 |
| `wn_three` | extreme | 44 | 33 | 81 | 48.49 |
| `tri_two` | random | 75 | **58** | 1207 | 810.55 |
| `tri_two` | θ=0 | 54 | 47 | 990 | 716.46 |
| `tri_two` | extreme | 0 | 0 | 0 | 0.00 |

**Gate passes on both.** Constructors are not rare outside fixtures — they
are abundant. Risk 2 is retired.

The `extreme` row on `tri_two` is the correction of §1.2 in situ: it reads
`0.00` (measured, the detector looked) rather than `unmeasured`.

---

## 4. The precheck matrix

`wn_three` (3 rules, `WhiteNoise`) and `tri_two` (2 rules, `TriLattice`),
`k = 4`, 3 seeds, arms = budget-matched random / `StaticBias` at θ=0 /
allocation extreme.

| objective | rule set | P-1 | P-2 | P-3 | P-4 | |
|---|---|---|---|---|---|---|
| F4 | `wn_three` | ok | ok † | ok | ok | **PASSES** |
| F4 | `tri_two` | ok | — ‡ | ok | ok | passes P-1/P-3/P-4 |
| F5 | `wn_three` | ok | — | **ABORT** | ok | |
| F5 | `tri_two` | ok | ok | ok | ok | **PASSES** |

† at a learning rate that traverses; see §4.2. ‡ not re-tested at that rate.

### 4.1 The one abort

**F5 × `wn_three`, `OBJECTIVE_BLIND`.** Spread across the class is 401.41
against a bar of 485.53 — twice the largest seed sd, which is random's
**242.76**. The arms are separated (222.85 / 452.77 / 51.37) but random's
own seed-to-seed variation swamps the separation. This is a clean,
correctly-reasoned abort: the objective may well see the class, but not at
three seeds on this substrate.

### 4.2 P-2, and why it needed a second measurement

P-2 needs a real θ trajectory, so it only runs inside a campaign
(`P2_MIN_GENERATIONS = 8`). At the config default `alpha = 0.01` both F4
cells aborted `OPTIMIZER_FROZEN` with `TV = 0.0000`.

That was **not** about F4. Two things say so. P-2's own diagnostic selects
the "step size too small" branch rather than "a direction the policy cannot
act on"; and `|dθ| ≈ 0.014` is the **same magnitude on the cell that
passed**, so at α = 0.01 over 8 generations θ has barely left its
initialization whatever the objective is. Measured directly:

| α | P-2 on F4 × `wn_three` |
|---|---|
| 0.01 | ABORT — allocation TV 0.0000, `|dθ|` 0.0142 = 0.00× drift scale |
| 0.20 | **PASS** — allocation TV 0.0508, `|dθ|` 0.2697 = 0.08× |
| 1.00 | **PASS** — allocation TV 0.2266, `|dθ|` 1.3149 = 0.38× |

So F4 × `wn_three` clears all four, and the α default is the thing that
needs choosing for D6 — the same class of finding as `docs/policy_config.md`
§6's note that σ = 0.02 is wrong for `StaticBias`, and for the same reason:
these defaults were set for a different parameterization.

**A limitation, stated rather than hidden.** These are 3-pair, 1-seed,
8-generation campaigns — a tenth of a real population. P-2 at Stage 2′ scale
is a weak instrument, and the plan's own "cost: minutes" budget for Stage 2′
is in tension with P-2 needing a campaign. The α sweep is what makes the
above a measurement rather than an inference, but a D6 campaign should run
P-2 again at full population.

---

## 5. What F4 is actually discriminating on

F4 passes P-3 on both substrates. P-3 does not ask *on what*, and here that
is the whole question.

Equal-count symbolization reads a monotone **drift** as temporal structure:
a strictly decreasing series becomes `0,0,…,1,1,…,2,…,3`, consecutive
symbols agree everywhere except at three block boundaries, and lag-1 MI
approaches F4's ceiling for a substrate whose type histogram is merely
settling down. Every substrate here drifts that way — Spearman trend against
window index runs **−0.72 to −1.00** — because they all grow.

`f4_drift` (§4A.6) recomputes F4 on the **sorted** series: same values, same
marginals, pure ordering, no dynamics.

| rule set | arm | F4 | trend | monotone ceiling | share |
|---|---|---|---|---|---|
| `wn_three` | random | 0.3571 | −0.803 | 1.6034 | 0.223 |
| `wn_three` | θ=0 | 0.7664 | −0.942 | 1.6015 | 0.478 |
| `wn_three` | extreme | 1.2574 | −0.954 | 1.6034 | 0.785 |
| `tri_two` | random | 0.2986 | −0.815 | 1.6006 | 0.187 |
| `tri_two` | θ=0 | 0.2164 | −0.720 | 1.6015 | 0.135 |
| `tri_two` | extreme | **1.6019** | **−0.999** | 1.5996 | **1.001** |

The last row is the finding. The arm F4 ranks highest on `tri_two` has a
JSD series that **is its own sorted version** — trend −0.999, share 1.001.
It scores 1.60 of a possible 2 while carrying no structure that sorting the
same values would not have supplied.

The monotone ceiling is ~1.60 on every substrate and arm, so `share` is F4
rescaled and cannot independently corroborate anything; the load-bearing
numbers are that single row's trend and its ratio to a ceiling it reaches.

**Consequence.** F4 is stateable, and a Stage 3′ campaign under it would
find a real optimum — but that optimum is "make the type histogram decay
smoothly", not predictive information about the dynamics in the sense §5.3
intended. `f4_drift` is deliberately a diagnostic and **not** subtracted
from F4: the drift here is not linear, so a correction would need a
detrending model, and the honest instrument reports both numbers.

This does not retract F4's precheck pass. It says what a result under F4
would and would not license, which is the thing plan v2 exists to get right.

---

## 6. The third rule set does not exist

Plan §9 asks for the prechecks on `wn_three`, `tri_two`, "and at least one
non-fixture rule set". There is no third one to run.

`policy_config.md` §2.5's liveness matrix contains exactly one co-live
pairing among the builtins — `binary_edge` + `triangle_subdivide`, plus
`growth_treadmill` on `WhiteNoise` — and **`wn_three` and `tri_two` already
are its two compositions**. Every other builtin ships a single rule, on
which `StaticBias` is provably budget-matched random and P-1 aborts before a
rollout.

P-1 was run over nine candidate (rule set, IC) pairings from the other
tracks. All nine abort:

| candidate | ICs tried | rules ever live | why |
|---|---|---|---|
| `lk_pair` | Empty, WhiteNoise, PureCrystal, TriLattice | `[]` | energy-gated LHS; no shipped IC carries energy |
| `crystal_battery` | PureCrystal, CrystalWithDefects, WhiteNoise | `[1]` | only the defect rule ever matches |
| `conservation_soak` | Empty, WhiteNoise | `[0, 1]` | both rules live, **never simultaneously** — sequential phases |

`conservation_soak` is the interesting one: it has two rules that both fire,
and still fails, because P-1 asks for *co*-liveness. Two rules that never
overlap in time give the softmax nothing to allocate between.

So the clause is satisfied by `wn_three` itself, which
`rules/wn_three.toml`'s own header already argues for: it composes three
unmodified builtin rules chosen by measured liveness, with nothing planted
to be learned, "unlike `fl1_two_rule`". The matrix is 2 substrates × 2
objectives, and that is the whole population available.

**This is a substrate-supply finding, and it bounds the programme.** Not
"we did not look"; P-1 looked at every multi-rule set in the repository.

---

## 7. Verdict

**Stage 2′ exit: MET.** Plan §10's D6 is unlocked — "Stage 3′ if D5 passes;
otherwise write up."

**Recommended pair for D6: F5 × `tri_two`.** It clears all four prechecks
with no interpretive caveat, its §5.4 gate passes with 47–58 confirmed
constructors, its arms separate by 822 against a bar of 20.5, and it is the
objective §5.4 argues is "more directly anti-uniform" than F4.

**F4 × `wn_three` is available as a second pair, with §5's finding attached
to any result it produces.**

**Carried into D6:**

1. `alpha = 0.01` freezes `StaticBias` at Stage 2′ scale. Choose it
   deliberately, as σ = 2.0 already was.
2. P-2 must be re-run at full population; §4.2's is under-powered.
3. F5's seed variance on `wn_three` (sd 242.76 on the random arm) is what
   aborted that cell. More seeds may state it; three do not.
4. F4 results are drift-flagged per §5.

**Not changed by this phase.** F1–F3's formulas are untouched, so every
C0–C5 number remains comparable. The `v2.1` bump is F4's symbolization
only, made before F4's first use in any campaign — no published F4 value
exists to revise, because until §1.1 there was no measurable one.

---

## 8. Provenance

| what | where |
|---|---|
| §1.1's symbolization table, §2's route comparison | `d5_f4_horizon_probe.rs::f4_horizon_and_window_probe` |
| §6's nine-candidate P-1 sweep | `d5_stage2_prime_prechecks.rs::third_rule_set_selection` |
| §3's yield gate | `…::f5_constructor_yield_gate` |
| §4's matrix | `…::stage2_prime_precheck_matrix` |
| §4.2's α sweep | `…::is_the_p2_abort_about_f4_or_about_alpha` |
| §5's per-arm decomposition | `…::f4_drift_decomposition_per_arm` |

All six are `#[ignore]`d and run explicitly; they are measurement passes,
not assertions. What *is* asserted, in `fitness.rs`'s unit tests:

- `f4_resolves_a_signal_confined_to_a_narrow_band` — the §1.1 regression,
  which recomputes the v2 grid's verdict inline so the test states what it
  protects against.
- `a_one_symbol_sequence_is_a_result_only_when_the_substrate_was_frozen` —
  the `f4_jsd_spread` guard.
- `an_empty_registry_means_opposite_things_with_the_layer_on_and_off` — the
  §1.2 correction.
- `the_drift_diagnostic_fires_on_a_purely_monotone_series` — §5's
  diagnostic, firing on a series with no dynamics and staying quiet on
  period-2 alternation.

Suite at close: **262 passed, 0 failed** across `hg-train`, `hg-cognition`
and `hg-sim-core`.

**Repo caveat, unchanged from D4.** `crates/hg-selfmod/` carries
uncommitted changes that do not compile (`EconomyParams` missing fields at
`runner.rs:147`). They are not part of the D-phase work, but they mean
`cargo test --workspace` fails; the three crates above are tested
explicitly for that reason.
