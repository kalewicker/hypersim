# Cognition Stage 0a / 0b / 1 — diagnostics findings

Phase C4 of `hypersim_nn_experiments_plan_v1.md`. These are the cheap gates
that run **before** any campaign, and each one is allowed to kill the
programme on a given rule set. A null here is a pre-committed finding, not a
disappointment (plan §11).

Produced by `crates/hg-train/tests/c4_diagnostics.rs`. Every number below is
measured; nothing is asserted from theory.

> **This document was rewritten after a bug was found in the control arm.**
> Every number in the first version was computed against a `RandomSubset`
> that was not sampling randomly (§0.4). The corrected numbers change the
> Stage 0a verdict on four of five rule sets, reverse the Stage 1
> conclusion, and retire the "these substrates are seed-deterministic"
> headline entirely. Nothing from the first version should be cited. The
> superseded claims are listed in §4.1 so that anything quoting them can be
> found.

---

## 0. Method and scope

### 0.1 Which rule sets, and why not all of them

A rule set only matches on an initial condition containing its LHS. On the
wrong IC it produces **zero candidates for the entire run**, and every policy
then trivially agrees because there is nothing to select — reporting such a
pairing as "selection-invariant" would be a statement about the initial
condition, not the substrate. The live pairings are fixed by the liveness
matrix in `docs/policy_config.md` §2.5.

`tetrahedral` is **absent from every table below** because it has no live
pairing among the shipped initial conditions. It is not a null result; it is
unmeasured, and it stays unmeasured until an IC containing its LHS exists.

### 0.2 Horizons

Horizons are per-fixture because candidate volume spans four orders of
magnitude across these rule sets (~56 000 candidates/step for `binary_edge`
against ~6 for `glider`). A single horizon would either starve the cheap
fixtures of windows or make the expensive ones dominate the runtime for no
extra coverage.

### 0.3 The Exploding early-abort gate is OFF for diagnostics

`[campaign.abort].exploding_windows` defaults to 2 consecutive Exploding
windows, which exists to stop a campaign cell eating the host. Diagnostics
are not campaign cells, and on an `Empty` initial condition that gate fires
on the **startup transient**: a graph going 0 → 3 → 6 nodes has an enormous
growth ratio for reasons that have nothing to do with its dynamics. Left at
the default, the Stage 1 run terminated at window 3 of 128 and reported
MC = 0 for want of samples. It is disabled in `Fixture::sim_config`, and
that is why.

### 0.4 The control arm was not random

**`RandomSubset` — the universal control that every result in this
programme is measured against — was returning the first `k` bindings in
engine order.**

The mechanism is a two-line misuse of `rand` 0.8:

```rust
matches.partial_shuffle(&mut self.rng, k);
matches.truncate(k);
```

`partial_shuffle` permutes into the **tail** of the slice and returns
`(tail, head)`. Truncating to the prefix therefore keeps the `k` elements
the shuffle had mostly left alone. Measured directly on 100 candidates at
`k = 16`, it returned exactly `[0, 1, 2, …, 15]`.

The fix is to use the slice the function hands back, which is correct
whichever end the implementation fills:

```rust
let (chosen, _rest) = matches.partial_shuffle(&mut self.rng, k);
matches = chosen.to_vec();
```

**Why the existing tests passed.** There were three, and each one is
satisfied by a first-k policy: it holds the budget exactly, it exempts
creation bindings, and it is deterministic in its seed *and* differs
between two seeds (a handful of elements do get swapped across the
boundary, so two seeds do not produce identical output). What no test
checked was the only property the control exists to have. The regression
test now measures it directly — over 400 seeds, every candidate in the pool
must be selected roughly equally often
(`random_subset_samples_uniformly_not_just_reproducibly`). A first-k policy
puts the entire mass on the first `k` and zero everywhere else, and fails
it by a mile.

The same misuse appeared in `MlpScorer`'s candidate cap, where it silently
restricted the network's view to the first 512 bindings in engine order on
every dense step. Fixed in the same place.

**Consequences, in order of severity.**

1. **Stage 1's verdict reverses.** Input coupling at delay 0 went from
   −0.0175 to **+0.9261** (§3). The first version concluded that the
   injected symbol never arrived and that Stage 1 was measuring a limit of
   the injection seam rather than the substrate. That conclusion was wrong:
   the symbol arrives cleanly and is gone one window later. Stage 1 is a
   substrate memory result after all.
2. **Stage 0a's classification changes on 13 of 25 cells** (§1), and the
   headline "four of five fixtures are seed-deterministic" is withdrawn.
   The determinism was the control's, not the substrate's.
3. **Stage 0b needed rebuilding for a second, independent reason** (§2).
4. **F-L0, F-L1 and F-L2 all still pass** against the corrected control,
   which is the reassuring part: the leak detector, the end-to-end learning
   validation, and the horizon contrast were not resting on the bug.

### 0.5 How a cell is classified

Stage 0a compares each heuristic against budget-matched random on three
gate-relevant metrics and reports the largest effect.

| class | meaning |
|---|---|
| **Measurable** | at least one arm has spread, so a σ exists |
| **Deterministic** | *neither* arm has spread — a real, reproducible effect with no denominator anywhere |
| **No difference** | every arm matches budget-matched random exactly |

Two floors matter here, both relative rather than absolute.

**Spread floor.** Twenty identical samples do not produce `sd = 0.0`: the
mean is not bit-identical to the samples, so `sqrt(Σ(x−m)²/(n−1))` lands
near 1e-16. An earlier `pooled <= 0.0` guard let that through and produced
σ values up to **2.9 × 10¹⁶**, with a spurious "exit criterion MET" on
cells that had no variance at all. Spread is now tested against
`SPREAD_EPS = 1e-9` times the metric's own magnitude.

**One-sided spread is measurable.** The first classifier required *both*
arms to vary, and that was too strict. `DegreeGreedy` has no RNG, and on a
constructed initial condition it is a point mass by construction — but the
budget-matched random baseline is not, and "how many baseline σ from the
baseline mean does the heuristic sit" is a well-defined one-sample z. It is
also what the plan's criterion means in substance: the spread that matters
is the spread of the thing being beaten. Requiring both arms to vary
discarded thirteen cells' worth of real measurements.

**A caveat on very large σ.** Several corrected cells report σ in the
hundreds or thousands (`binary_edge` at `k = 64` reaches 29 198). Those are
arithmetically correct and they are *not* the 1e16 artifact — the baseline
has genuine spread, it is simply very concentrated (sd ≈ 1e-5 around a mean
of 0.005). Read them as "the twenty baseline seeds agree closely and the
heuristic is nowhere near them", not as effect sizes. The Δ column is the
one to compare across cells.

---

## 1. Stage 0a — budget response curve and heuristic causal power

**Hypothesis.** Selection under budget has *any* causal power over regime
outcomes.

**Method.** Budget ladder `k ∈ {1, 4, 16, 64, ∞}`, 20 seeds per cell, three
zero-parameter arms — `RandomSubset` (the control), `DegreeGreedy(high)`,
`DegreeGreedy(low)`. The control's policy seed is now derived per rollout
(`derive_policy_seed`), the same way a campaign derives it; pinning it to a
constant, as the first sweep did, is a second way to make a baseline
non-random on top of §0.4.

### 1.1 Result summary

**25 cells: 15 measurable, 0 deterministic, 10 no-difference.**
(Was 2 / 13 / 10 against the broken control.)

**Stage 0a exit criterion (a > 2σ measurable effect somewhere): MET**, now
on **ten cells across four rule sets** rather than one cell on one.

| rule set | cells meeting the > 2σ bar | best |
|---|---|---|
| `triangle_subdivide` | `k = 1, 4, 16, 64` | 68.8σ (`k = 64`) |
| `binary_edge` | `k = 4, 16, 64` | Δinterest 0.396 (`k = 64`) |
| `fl1_two_rule` | `k = 1, 4, 16, 64` | 32.2σ (`k = 64`) |
| `growth_treadmill` | `k = 1` | 3.72σ |
| `glider` | none (1.82σ at `k = 4`) | — |

### 1.2 `glider` — GliderRing, 512 steps, window 64

| k | arm | F2 | F2 sd | apstruct | aps sd | verdict |
|---|---|---|---|---|---|---|
| 1 | random | 0.946 | 0.095 | 3.86 | 0.254 | |
| 1 | degree_high / low | 1.000 | 0.000 | 4.00 | 0.000 | measurable, 0.56σ |
| 4 | random | 0.507 | 0.270 | 2.69 | 0.720 | |
| 4 | degree_high / low | 1.000 | 0.000 | 4.00 | 0.000 | measurable, **1.82σ** (below bar) |
| 16 / 64 / ∞ | all three | 1.000 | 0.000 | 4.00 | 0.000 | no difference |

The only rule set that still fails the bar. The effect is real and in the
expected direction — a degree heuristic holds the glider's four-node track
intact where random selection breaks it — but at 20 seeds it does not reach
2σ. Interestingness is 0.432 in every cell of the table, which is its own
observation: on a conservative relabel rule the metric cannot move.

### 1.3 `growth_treadmill` — WhiteNoise, 512 steps, window 64

| k | arm | F2 | interest | sd | final n | verdict |
|---|---|---|---|---|---|---|
| 1 | random | 0.071 | 0.147 | 0.021 | 536 | |
| 1 | degree_high | 0.071 | **0.091** | 0.001 | 536 | **measurable, 3.72σ ← exit met** |
| 1 | degree_low | 0.071 | 0.146 | 0.023 | 536 | |
| 4 | random | −0.071 | 0.090 | 0.022 | 1429 | |
| 4 | degree_high | −0.057 | 0.054 | 0.015 | 1125 | measurable, 1.87σ |
| 16 / 64 / ∞ | all three | −0.071 | 0.071 | 0.018 | 2174 | no difference |

The one fixture whose initial condition is *sampled* rather than
constructed, so it has seed variance from the IC generator as well as from
the policy.

### 1.4 `triangle_subdivide` — TriLattice, 96 steps, window 24

The largest effects in the sweep, and the rule set most changed by the fix.

| k | random F2 | random apstruct | degree_high F2 | degree_low F2 | verdict |
|---|---|---|---|---|---|
| 1 | 1.000 | **10.10 ± 0.53** | 1.000 | **0.000** | measurable, **33.1σ** |
| 4 | 0.500 | **17.20 ± 0.70** | 1.000 | 0.000 | measurable, **37.5σ** |
| 16 | 0.150 ± 0.235 | **29.25 ± 0.99** | 1.000 | 0.000 | measurable, **28.7σ** |
| 64 | 0.000 | **38.95 ± 2.19** | 1.000 | −0.167 | measurable, **68.8σ** |
| ∞ | −0.167 | 1.00 | −0.167 | −0.167 | no difference |

The `random apstruct` column is where the bug was hiding in plain sight. It
read 1.00 in every row of the first sweep — identical to `degree_high` —
because taking the first `k` bindings in engine order is itself a strong
structural bias. Sampling properly, random builds *far more*
active-persistent structure than either heuristic (up to 38.95 against
1.00), and both degree heuristics suppress it. The direction of the whole
result was inverted.

### 1.5 `binary_edge` — PureCrystal, 96 steps, window 24

| k | random F2 | random interest | degree_high F2 | degree_low F2 | final n (r/h/l) | capW | verdict |
|---|---|---|---|---|---|---|---|
| 1 | 0.000 | 0.505 | 0.000 | 0.000 | 160 (all) | 0 | no difference |
| 4 | −0.167 | 0.275 | 0.000 | 0.000 | 434/197/252 | 0 | measurable, Δ 0.211 |
| 16 | −0.250 ± 0.085 | 0.022 | −0.333 | −0.167 | 1496/306/626 | 0 | measurable, Δ 0.236 |
| 64 | −0.333 | 0.005 | **0.000** | −0.167 | 5594/572/1943 | 0 | measurable, Δ 0.396 |
| ∞ | −0.333 | 0.001 | −0.333 | −0.333 | **84 600** (all) | **60** | no difference — see §5.2 |

At `k = 1` the three arms still agree exactly: with one binding selected
per step out of ~56 000 near-identical candidates on a pure crystal, which
one is chosen genuinely does not matter. That cell is a real null, not a
measurement failure.

### 1.6 `fl1_two_rule` — GliderRing, 192 steps, window 32

| k | random F2 | random apstruct | degree_high F2 | verdict |
|---|---|---|---|---|
| 1 | 0.760 ± 0.123 | 11.96 ± 0.81 | 1.000 | measurable, **9.8σ** |
| 4 | 0.700 | 18.45 ± 0.94 | 1.000 | measurable, **15.4σ** |
| 16 | 0.400 | 26.03 ± 1.52 | 1.000 | measurable, **14.5σ** |
| 64 | 0.100 | 31.56 ± 3.49 | 1.000 | measurable, **32.2σ** |
| ∞ | −0.100 | 5.00 | −0.100 | no difference |

The `k = 16` row still independently reproduces the budget band F-L1's own
calibration found (random 0.400 against both heuristics at 1.000) — that
agreement survived the correction, which is one reason to trust F-L1's
verdict.

### 1.7 The one result the correction did not touch

**At `k = ∞`, selection has no effect on any rule set.** Every arm is
identical in every `∞` row of every table above, before and after the fix.
That is the plan's §1.2 argument — "selection acquires causal power only
when rewrite bandwidth is scarce" — measured rather than assumed across
five rule sets and 20 seeds each, and it upgrades the budget from a tuning
knob to a precondition: **a campaign run at `k = ∞` cannot produce a result
whatever its optimizer does.** `stage2::run_case` refuses an unbounded
budget on these grounds rather than leaving it to be rediscovered.

The other durable observation is that the band closes at the *bottom* too,
and differently per rule set: `binary_edge` shows nothing at `k = 1` while
`triangle_subdivide` shows its second-largest effect there.

### 1.8 The classifier is sign-agnostic, and several headline cells are the heuristic being *worse*

Stage 0a asks whether selection has **any** causal power, so its classifier
compares `|heuristic − random|` and does not care which way the difference
points. That is correct for the hypothesis as the plan states it, and it is
why "no difference" and "a large effect" are the only two outcomes it needs
to separate.

It does mean the tables above must not be read as "the heuristic is
better". Checking the direction on the cells that carry the largest σ:

| cell | random | heuristic | direction |
|---|---|---|---|
| `growth_treadmill` k=1, interest | 0.147 | 0.091 (`degree_high`) | **heuristic worse** — and this is the exit-criterion cell |
| `triangle_subdivide` k=1, interest | 0.561 | 0.038 (`degree_low`) | **heuristic worse** (33.1σ) |
| `triangle_subdivide` k=64, interest | 0.208 | 0.588 (`degree_high`) | heuristic better (68.8σ) |
| `binary_edge` k=16, interest | 0.022 | 0.258 (`degree_low`) | heuristic better |
| `fl1_two_rule` k=1, apstruct | 11.96 | 4.00 (`degree_high`) | **heuristic worse** (9.8σ) |
| `glider` k=4, apstruct | 2.69 | 4.00 (`degree_high`) | heuristic better |

So the directions are mixed, and **Stage 0a's exit is met on a cell where
the heuristic loses to random**. Nothing above is retracted — "selection
under budget has causal power over regime outcomes" is exactly what a large
signed-either-way effect establishes, and that is the gate Stage 0a was
built to be.

But the downstream stages need the *signed* question, because a champion
has to beat budget-matched random, not merely differ from it. The
F-L3 sweep in `docs/cognition_stage2_findings.md` §6.1 asks the signed
version directly on a purpose-built fixture and finds that random wins
under all three fitness functions at every budget rung. §6.2 is why: this
metric family rewards variety, and uniform selection maximizes variety.

---

## 2. Stage 0b — separation / fading memory

**Hypothesis.** At least one rule set sits in the reservoir-viable band:
perturbations neither die instantly nor diverge to the between-seed
baseline.

This diagnostic was rebuilt twice, for two independent reasons. Both
rebuilds were forced by results that were arithmetically impossible rather
than merely disappointing, which is the useful kind of failure.

### 2.1 Rebuild 1 — the perturbation did not exist

The perturbation is a different simulation seed on the same configuration.
The seed reaches a run through exactly two channels: the initial-condition
generator, and the engine's RNG stream. `GliderRing`, `TriLattice` and
`PureCrystal` are **constructed**, so the first channel is constant; and
under `MaxParallel` with no property jitter the engine never draws from the
second. On those fixtures a `PassThrough` run is a bit-identical function
of the config, and "a different seed" is not a small perturbation — it is
no perturbation.

The classifier reported `Ordered` for these, which conflates "the
perturbation decayed" with "there was never a perturbation". Only the first
is a substrate property. It now returns **`Undetermined`**, and the
diagnostic runs a second arm — a **budgeted stochastic policy**
(`RandomSubset` at `k = 4`), where the seed decides which rewrites fire.
That is a real perturbation channel, and it is also the regime Stages 2–3
operate in, so it is the separation question those stages actually depend
on.

### 2.2 Rebuild 2 — the distance was not a distance

With a perturbation in place, three of five fixtures reported the perturbed
pair separating **more** than two independent seeds (ratios 1.69, 3.26,
5.56). A perturbed pair cannot exceed a fully-diverged reference, so the
measurement was wrong, not surprising.

The cause: separation was computed as `|jsd_a[i] − jsd_b[i]|`, the
difference of two runs' *window-to-window churn scalars*. That is not a
distance between trajectories. Two runs in completely different regions of
state space can churn at the same rate and score 0; two nearby runs can
differ in rate and score high.

Separation is now the **Jensen-Shannon divergence between the two runs'
state vectors** at each window, where the state is the same 16-bucket WL
type histogram Stage 1 reads out. Bounded, symmetric, zero exactly when the
coarse-grained states agree — and it makes the independent-seed reference
the upper bound the classifier always assumed it was.

A floor was added with it: `REFERENCE_FLOOR = 1e-4` on the *baseline*. A
ratio of two negligible numbers is arithmetic — `binary_edge` produced
plateau and baseline both below 1e-5 and a ratio of 0.995, which would have
been published as "Chaotic" on the third significant figure of noise.

### 2.3 Result

| fixture | arm | plateau | independent baseline | ratio | class |
|---|---|---|---|---|---|
| `glider` | passthrough | 0.00000 | 0.00000 | — | Undetermined |
| `glider` | random k=4 | 0.00000 | 0.00000 | — | **Undetermined** |
| `growth_treadmill` | passthrough | 0.00208 | 0.00205 | 1.012 | Chaotic |
| `growth_treadmill` | random k=4 | 0.00423 | 0.00249 | 1.699 | Chaotic |
| `triangle_subdivide` | passthrough | 0.00000 | 0.00000 | — | Undetermined |
| `triangle_subdivide` | random k=4 | 0.00964 | 0.01282 | 0.752 | **Viable** |
| `binary_edge` | passthrough | 0.00000 | 0.00000 | — | Undetermined |
| `binary_edge` | random k=4 | 0.00000 | 0.00000 | 0.995 | Undetermined |
| `fl1_two_rule` | passthrough | 0.00000 | 0.00000 | — | Undetermined |
| `fl1_two_rule` | random k=4 | 0.01014 | 0.00926 | 1.095 | Chaotic |

**Viable rule sets: `triangle_subdivide` (1 of 5). Stage 0b's exit
criterion (≥ 1 viable rule set) is MET.**

Note that the viable rule set *changed identity* between versions —
`growth_treadmill` was the sole Viable entry before and is Chaotic now.
Nothing about `growth_treadmill` changed; the instrument did.

Three things are worth reading off this table.

**The `passthrough` column is a statement about the harness.** Four of its
five rows are `Undetermined` by construction, and that is the correct
report rather than a gap in the data.

**Most of these substrates are Chaotic, not Ordered.** A one-seed
perturbation reaches the independent-seed level of divergence within the
run. That is the "no fading memory" end of the band, and it agrees with
Stage 1 measuring decodability gone by delay 1 — two instruments, built
from different code paths, reaching the same conclusion about the same
substrates.

**`glider` is invisible to this instrument, not inert.** Its bucketed WL
histogram is exactly constant: the rule is a conservative relabel on a
symmetric ring, so the multiset of WL classes never changes even though the
mark demonstrably moves. A coarse-graining that cannot see the only thing
happening will report no separation whatever the dynamics are. `glider`'s
`Undetermined` is about the probe, and a finer state vector would be needed
to say anything about it.

---

## 3. Stage 1 — reservoir characterization

**Hypothesis.** The substrate computes: its state carries linearly decodable
information about input history.

**Method.** `StimulusDriver` injects a random bit tape through paired
creation rules, one symbol per metrics window. The probe records a bucketed
WL type-histogram (16 buckets) at each window boundary as the state vector.
Readout is in-house ridge regression, scored **out of sample** on a held-out
tail split. `MC = Σ_d max(0, r²(d))` over delays 1…12.

### 3.1 Two measurement conventions worth stating

**Delays run from 1, not 0.** Delay 0 is the symbol being injected right
now, which is input rather than memory. A perfect `k`-tap delay line scores
`MC = k − 1` under this convention, not `k`. Delay 0 is reported separately
as `input_coupling_r2`, because it is the number that separates "the
substrate forgets within one window" from "the probe and the injection
never worked" — two very different findings, and §3.4 is why that
distinction earns its keep.

**r² is out of sample.** Reservoir results are often reported as in-sample
squared correlation, which on a 16-dimensional state with ~128 samples is
close to 1 by construction. The 70/30 split is not optional here.

### 3.2 Result

128 windows of 32 steps, 16-bucket WL state, one symbol per window, delays
1–12, ridge λ = 1e-6, 70/30 train/test split.

| delay | substrate r² | ESN(16) r² |
|---|---|---|
| 1 | +0.0170 | 0.9972 |
| 2 | −0.0259 | 0.9915 |
| 3 | −0.0306 | 0.9738 |
| 4 | −0.0999 | 0.8795 |
| 5 | −0.0194 | 0.6365 |
| 6 | −0.1210 | 0.6889 |
| 7 | +0.0133 | 0.3706 |
| 8 | +0.0076 | 0.2156 |
| 9 | −0.0079 | 0.0701 |
| 10 | −0.0900 | −0.2114 |
| 11 | +0.0011 | −0.2708 |
| 12 | −0.0077 | −0.1079 |

| quantity | substrate | ESN, matched dim 16 |
|---|---|---|
| **MC** = Σ max(0, r²) | **0.0390** | **5.8237** |
| max decodable delay (r² > 0.1) | none | 8 |
| **input coupling, delay 0** | **+0.9261** | 0.9970 |
| nonlinearity (XOR at d1, d2) | +0.0510 | 0.3618 |

**Stage 1 exit criterion (MC > 0 with r² > 0.1 at delay ≥ 3): NOT MET.**

### 3.3 The delay-0 number is the whole result

The plan pre-commits an interpretation for a Stage 1 null: "the substrate
scrambles input identity faster than one window — informative about the
substrate's intrinsic timescale, and it redirects effort toward slower rule
sets or shorter symbol windows".

**That is exactly what happened, and the delay-0 measurement is what
licenses saying so.** Input coupling is **+0.9261**: the readout recovers
the symbol being injected right now almost as well as a matched ESN does
(0.9970). The injection works, the probe sees it, the coarse-graining is
adequate to represent it. One window later it is gone — r² = +0.017 at
delay 1, and nothing above 0.1 at any delay.

So the substrate has an intrinsic timescale **shorter than one metrics
window** at this configuration. That is a substrate property, measured
against a same-dimension baseline on the identical tape, and it is the
plan's pre-registered failure reading rather than an unanticipated one.

### 3.4 The shorter-window follow-up, run

The plan's first named follow-up is a testable prediction, not just a
suggestion. If the memory is shorter than 32 steps but non-zero, then
shortening the symbol window brings the memory *inside* the sampling
interval and MC should rise; if MC stays flat, the memory is not merely
being under-sampled and the timescale story is wrong.

Swept with the symbol count held at 128 throughout, so the readout always
has the same number of training rows (`c5_stage1_timescale.rs`):

| window | steps | MC | input r² (d=0) | r² d=1 | max decodable delay |
|---|---|---|---|---|---|
| 2 | 256 | 0.0084 | 0.6043 | 0.0038 | none |
| **4** | 512 | **0.0892** | 0.6772 | −0.0088 | none |
| 8 | 1024 | 0.0780 | 0.6714 | −0.0161 | none |
| 16 | 2048 | 0.0387 | 0.8873 | 0.0174 | none |
| 32 | 4096 | 0.0390 | **0.9261** | 0.0170 | none |

**The prediction is confirmed in direction and refuted in magnitude.** MC
does rise as the window shortens — 0.0390 at window 32 to **0.0892 at
window 4**, a factor of 2.3 — so the memory genuinely was being
under-sampled. But at the optimum it is still 0.089 against the matched
ESN's 5.82, and **no delay is decodable above r² = 0.1 at any window**.

The sweep also exposes a trade-off the single-window measurement could not
see: **input coupling falls as the window shortens** (0.93 → 0.60), because
fewer steps per symbol means less structure built per symbol and a weaker
imprint for the readout to find. Short windows sample the memory better and
couple the input worse; the MC peak at window 4–8 is where those cross.

**Stage 1's exit criterion is NOT MET at any window**, and the "32 steps is
simply longer than the memory" explanation is now measured rather than
assumed — it accounts for a factor of 2.3, not for the factor of 65 between
the substrate and a matched ESN. The remaining follow-up the plan names,
slower rule sets, is not run here.

### 3.4 What this looked like before the control was fixed

Input coupling read **−0.0175**, and the first version of this document
concluded — at length — that nothing was arriving, that Stage 1 was
measuring a structural limit of the creation-rule injection seam (empty
LHS, can only attach to nothing), and that it was therefore *not* evidence
about substrate memory.

That reasoning was sound and the premise was false. With a control that
samples uniformly, input coupling is 0.93. **The lesson worth keeping is
the one that survived being wrong: MC ≈ 0 must always be read against a
delay-0 number.** Without it, "the substrate has no memory" and "the
apparatus is not connected" are the same measurement. `input_coupling_r2`
is a permanent part of the Stage 1 output for that reason, and it earned
the place twice — once by catching a real plumbing artifact in the first
fixture (§4.2), and once by refuting a plausible story about the seam here.

---

## 4. Corrections log

### 4.1 Superseded claims

Anything citing these should be re-checked against this document.

| withdrawn claim | corrected |
|---|---|
| "Four of five fixtures are seed-deterministic" | Withdrawn. The determinism was the control's. Constructed ICs plus `MaxParallel` do make the *substrate* seed-invariant, but every campaign and every corrected diagnostic has real hold-out variance through the policy RNG (measured: F2 sd 0.111 on `glider` from the policy seed alone, `c5_variance_probe.rs`). |
| "25 cells: 2 measurable, 13 deterministic, 10 no-difference" | 15 measurable, 0 deterministic, 10 no-difference. |
| "Exit met on exactly one cell (`growth_treadmill` k=1, 4.64σ)" | Exit met on ten cells across four rule sets; `growth_treadmill` k=1 is now 3.72σ. |
| "Stage 2 cannot use the 2σ criterion; run campaigns under `SequentialRandom`" | Withdrawn. Hold-out variance is real under `MaxParallel` via the policy RNG, so the criterion applies as written and no step-mode change is needed. |
| "Stage 1: input coupling −0.0175, so this measures the seam's input-coupling limit, not substrate memory" | Input coupling +0.9261. It *is* a substrate memory result: input arrives, and is gone within one window. |
| "`growth_treadmill` is the sole Viable rule set (Stage 0b)" | `triangle_subdivide` is, under a perturbation channel that exists; `growth_treadmill` is Chaotic. |
| Four `Ordered` labels marked "not assessed" | Now reported as `Undetermined` by the classifier itself, with a second arm that does classify. |

### 4.2 Fixtures and instruments that lied before they were fixed

Five now, and only one of them was caught by a test failing.

1. **The control arm** (§0.4) — surfaced by a Stage 2 design question, not
   by any test.
2. **Stage 1's first fixture destroyed its own substrate.** The reservoir
   originally ran on a `GliderRing`, whose 3-edges are all-unmarked and so
   match the `decay` rule exactly as injected tokens do. Node count
   collapsed 31 → 7 inside the first window and every WL bucket had sd
   0.0000 across 32 windows. MC came out 0.0000 — indistinguishable from a
   real finding. Now runs on `Empty`, where the token population *is* the
   reservoir.
3. **The abort gate truncated Stage 1 at window 3 of 128** (§0.3). The
   reported MC was 0 because there were three samples.
4. **The σ guard was defeated by float noise** (§0.5), producing 2.9e16σ
   and a spurious exit.
5. **Stage 0b measured a scalar difference and called it a distance**
   (§2.2), producing ratios above 1 against a supposed upper bound.

The pattern across all five: each produced a *plausible* number that would
have been written up as a finding. Four were caught by checking whether the
mechanism could produce the number, one by an impossible value. None were
caught by a test that existed at the time.

---

## 5. Go / no-go for Stages 2–3

| rule set | Stage 0a | Stage 0b | verdict |
|---|---|---|---|
| `triangle_subdivide` | measurable at `k = 1…64`, up to 68.8σ | **Viable** | **GO** — clears both gates outright |
| `binary_edge` | measurable at `k = 4…64` | Undetermined (probe-limited) | **GO** on 0a; also the only rule set where the engine cap binds |
| `fl1_two_rule` | measurable at `k = 1…64` | Chaotic | fixture, not a substrate — loop validation only |
| `growth_treadmill` | measurable at `k = 1` (3.72σ) | Chaotic | **GO**, narrow band (one rung) |
| `glider` | 1.82σ at `k = 4`, below bar | Undetermined (probe-blind) | **WEAK GO** — effect real, underpowered at 20 seeds |
| `tetrahedral` | — | — | **BLOCKED**: no live initial condition; unmeasured, not null |

### 5.1 The blocker this table does not show

**Every builtin rule set ships exactly one rule**, and `StaticBias` has one
parameter per rule. A masked softmax over a single logit is 1.0 whatever
the logit is, so on a single-rule set the genome cannot change the
selection at all: `StaticBias` is provably identical to budget-matched
random. Stage 2's hypothesis — "a per-rule preference vector can steer the
substrate" — is not false on these rule sets, it is **unstateable**.

Every GO above is therefore a GO for Stage 3 (where `MlpScorer` reads
per-candidate features, and where Stage 0a's degree effects say there is
signal to read), and none of them is a Stage 2 target as it stands. Stage 2
needs a rule set with ≥ 2 rules; `rules/wn_three.toml` and
`rules/tri_two.toml` are compositions of co-live substrate rules built for
that, and their own budget probe is in
`docs/cognition_stage2_findings.md`.

This is pinned in two places so it cannot be rediscovered the expensive
way: `hg-cognition`'s `a_single_rule_genome_cannot_change_selection`, and
a refusal in `campaign::run_campaign`.

### 5.2 `binary_edge` at `k = ∞` measures the engine, not the substrate

The only cell in the sweep where `max_pairings_per_step` bound: 60 windows
capped, 84 600 final nodes. That arm reports the engine's cap of 1000
rewrites/step, not an unbudgeted policy (`docs/policy_config.md` §2.3), and
it is excluded from §1.7's claim on those grounds — the arms agree there
because the cap, not the policy, is deciding.

### 5.3 What Stage 0a's direction says about Stage 3

On `triangle_subdivide` and `fl1_two_rule` the corrected tables show
budget-matched random building **far more** active-persistent structure
than either degree heuristic (38.95 vs 1.00 at `k = 64`). Both heuristics
suppress it, in the same direction, from opposite ends of the degree
ordering. A per-candidate scorer therefore has a large gap to work in, but
the useful policy is unlikely to be a monotone function of degree — which
is precisely the case `StaticBias` cannot express and `MlpScorer` can.
