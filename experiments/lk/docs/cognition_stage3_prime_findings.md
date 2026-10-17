# Stage 3′ — `MlpScorer` under the objectives that cleared Stage 2′

Phase D6 of `hypersim_nn_experiments_plan_v2.md`, §9. Produced by
`crates/hg-train/tests/d6_stage3_prime_mlp.rs`.

> **Stage 3′ passes on F5 × `tri_two`: champion − untrained = +382.02 at
> 14.67σ, champion − random = +341.09 at 10.85σ, no Goodhart flags, and the
> champion carries *more* active persistent structure than either baseline.**
> This is the programme's first positive learning result. F4 × `wn_three`
> aborts `OBJECTIVE_BLIND` at five seeds.
>
> **It took four instrument fixes to get a result that means anything, and
> every one of them would have produced a confident wrong answer.** P-3's
> far-end probe was `StaticBias`-shaped and provably inert on an `MlpScorer`
> (§1); my first fix for that scaled the probe by the optimizer's step size,
> turning P-3 into a σ-detector (§1.1); P-2's decisive statistic was
> per-rule, which an MLP can leave untouched while changing completely (§4);
> and `θ = 0` turns out to be the centre of a Gumbel-wide basin in which the
> policy is budget-matched random *whatever its weights are* (§5).
>
> The last one is the substantive finding. It also means the σ and α this
> stage runs at are **derived** (§5.3), not chosen — and the difference is
> not cosmetic: the mis-scaled search produced a champion that beat random
> by 1.76σ, below the bar, while the calibrated one beat it by 10.85σ.

---

## 1. A precheck defect found before any campaign ran

Plan §9 requires the three-arm protocol, and P-3's third arm is "the
allocation extreme". `allocation_extreme` is a statement about
`StaticBias`'s parameter space — one softmax logit at 8.0 puts over 99% of
the allocation on one rule, which is genuinely the far end of that class.

Applied to an `MlpScorer` genome it sets `w1[0] = 8.0` and leaves
everything else at zero. `w2` is part of everything else, so the one live
hidden row is multiplied by zero on the way out. Measured on a
`tri_two`-shaped scorer (433 parameters, feature dim 25):

| genome | `score(x)` | `score(y)` |
|---|---|---|
| θ = 0 | 0.000000 | 0.000000 |
| `allocation_extreme` | **0.000000** | **0.000000** |
| seeded draw | 0.136253 | −1.338772 |

So P-3 would have compared **three copies of one policy**, found no spread,
and aborted `OBJECTIVE_BLIND` — reporting "the objective cannot see the
class" when what actually happened is that the probe never left the origin.
That is the `MlpScorer` analogue of P-1's one-rule `StaticBias` case, and it
has the same remedy: refuse to draw a conclusion from a degenerate probe.

**Fixed** (`prechecks::class_extreme`): the far-end probe is chosen per
policy class — the logit extreme for `StaticBias`, a seeded draw at
`MLP_PROBE_SCALE` for `MlpScorer`. Getting that scale right took a second
pass; see §1.1.

**And checked, not trusted** (`prechecks::probe_genome_is_live`): the
campaign runner now scores two dissimilar feature vectors under the probe
genome and refuses to start if they tie. A probe arm that does not move the
policy makes P-3's spread meaningless, and that must abort as a defect in
the probe rather than pass as a finding about the objective. Firing test:
`the_static_bias_extreme_is_a_dead_probe_for_an_mlp`.

### 1.1 The fix's own first version was wrong, in a more interesting way

`class_extreme` first shipped scaled by the campaign's `sigma`, reasoning
that the probe should live where the optimizer will actually look.
**Both D6 campaigns then refused to start**, at `sigma = 0.5`:

| cell | P-3 spread | bar (2× largest seed sd) | verdict |
|---|---|---|---|
| F5 × `tri_two` | 41.43 | 106.01 | `OBJECTIVE_BLIND` |
| F4 × `wn_three` | 0.3686 | 0.5194 | `OBJECTIVE_BLIND` |

The same objectives on the same substrates separate the class by **306.15
over sd 38.61** and **1.1828 over 0.2597** at a fixed probe scale (§3). So
the abort was not about the objective at all.

The error is a category one, and it is the same shape as D5's α finding
wearing different clothes. P-3 asks *"does the objective assign different
values to at least two policies **in the class**"* — a question about the
class. Scaling its probe by the optimizer's step size makes P-3's verdict a
function of `sigma`, so a small `sigma` yields `OBJECTIVE_BLIND`: **a fact
about the optimizer, reported under the objective's abort code.** Plan v2
§4 separates P-2's question from P-3's precisely so that cannot happen, and
a σ-scaled probe quietly re-merges them.

`MLP_PROBE_SCALE = 2.0`, fixed and documented, with the scale justified by
the separation it achieves rather than by convention. Whether the *search*
can see the objective from one perturbation is a real and separate
question — and it is P-2's, measured on real steps rather than on a
synthetic genome.

The σ = 0.5 numbers are not discarded, because they answer that separate
question: one perturbation at σ = 0.5 moves F5 by 41.43 against a seed sd
of 53.00, i.e. the ES signal sits below seed noise there. That is why the
campaigns below run at **σ = 2.0** — chosen from measurement, exactly as
D5 chose α for `StaticBias`.

---

## 2. The plan's "untrained `MlpScorer`" arm is budget-matched random

`forward` returns `b2 + Σ_h w2[h] · relu(b1[h] + w1[h]·x)`. At θ = 0 every
term is zero, so every candidate scores 0.0. Gumbel-top-k over tied scores
is a uniform subset, and both `MlpScorer` and `RandomSubset` keep every
creation candidate and subsample only `matches` — so the two induce the
same distribution over selections.

They are **not** trajectory-identical: `RandomSubset` samples with
`partial_shuffle` and `MlpScorer` consumes Gumbel draws, so the same seed
walks two different paths through the same distribution and the canonical
hashes differ. The claim that matters is distributional, and it is measured
over the five frozen hold-out seeds:

| cell | random | mlp θ=0 | \|Δ\| in pooled σ |
|---|---|---|---|
| F5 × `tri_two` | 815.12 ± 38.61 | 830.50 ± 28.48 | **0.45** |
| F4 × `wn_three` | 0.5493 ± 0.2597 | 0.3901 ± 0.1844 | **0.71** |

Both well inside noise. **The untrained arm and the random arm are the same
arm.**

### 2.1 What that does to the exit criterion

Plan §3 is explicit that `champion − untrained` is the primary statistic
and `champion − random` is "never an exit criterion for a learning claim".
For this policy class at this initialization those two statistics
**coincide in expectation**.

That is not a defect in the plan — it is a property of a zero-initialized
network, and the right response is to say so rather than let the arm table
imply a control that is not there. Concretely: there is no
inductive-bias effect for the untrained arm to control for, because the
architecture contributes nothing before training. A **seeded** untrained
arm is therefore measured alongside — an untrained network in the ordinary
sense — and it is the arm that can show a class effect if one exists.

---

## 3. Prechecks

`k = 4`, 384 steps, window 6, five frozen hold-out seeds, hidden = 16.

| cell | n_params | P-1 | P-3 | P-4 |
|---|---|---|---|---|
| F5 × `tri_two` | 433 | 64/64 co-live | spread 306.15 over sd 38.61 | unbounded objective |
| F4 × `wn_three` | 449 | 64/64 co-live | spread 1.1828 over sd 0.2597 | random 0.5493, 1.4507 of room |

All pass. Probe genome reported live in both.

### 3.1 The arm that should worry us

| cell | random | mlp θ=0 | mlp seeded |
|---|---|---|---|
| F5 × `tri_two` | 815.12 ± 38.61 | 830.50 ± 28.48 | **524.34 ± 20.84** |
| F4 × `wn_three` | 0.5493 ± 0.2597 | 0.3901 ± 0.1844 | **1.5729 ± 0.0599** |

Under F4, an **untrained network with random weights already scores
1.5729** of a possible 2 — against random's 0.55 — and it does so in
**0.48 s** where the random arm takes 5.96 s. A 12× speedup on a fixed step
count means the substrate is barely growing: the seeded policy is
suppressing activity, and F4 is rewarding it for that.

`docs/cognition_stage2_prime_findings.md` §5 measured F4's monotone drift
ceiling on these substrates at ~1.60. **The seeded arm is at it.** So under
F4 the reachable optimum requires no training at all, and any champion that
finds it is finding a degenerate policy rather than a learned one. D5
predicted this shape from `StaticBias` arms alone; the MLP class confirms
it and makes it worse, because a random draw suffices.

---

## 4. A third precheck defect — P-2 could not see this policy class either

With P-3 fixed, both campaigns started and then aborted at generation 7:

```
OPTIMIZER_FROZEN: induced-allocation TV = 0.0000 against a bar of 0.05,
with |dtheta| = 0.5414  (F5 × tri_two)
with |dtheta| = 0.7020  (F4 × wn_three)
```

**`TV = 0.0000` is what a correctly-working `MlpScorer` search produces.**
P-2's decisive statistic is `RuleTrace::allocation()` — selections
normalized **per rule id**. `StaticBias` chooses a distribution over rules,
so any change in its behaviour must appear there. `MlpScorer` chooses among
*individual bindings*: it can rank every candidate differently while leaving
the per-rule proportions untouched. A per-rule statistic is structurally
blind to it.

This is the third instance of one shape, and the most pointed, because
`TracingPolicy`'s own doc comment warns about exactly this hazard —

> Computing `softmax(theta)` instead would be correct for `StaticBias` and
> meaningless for `MlpScorer`

— and it was right to wrap the real policy rather than read θ. But the
*statistic* it then derives from the wrapper is still a per-rule one. The
wrapper solved half the problem, and the doc comment describes the half it
solved.

The abort message compounded it: the prose "for StaticBias that is
softmax-logit space" was hardcoded, so a scorer campaign was handed a
logit-space explanation.

### 4.1 The fix — traversal by selection, for classes that select

`RuleTrace` now records a per-step hash of *which candidates were chosen*,
and `prechecks::selection_divergence` reports the first step at which a
θ_init probe and a θ_now probe differ. Both probes run from the **same sim
seed**, so engine and policy RNG are identical and the two rollouts stay in
lockstep for exactly as long as the two parameterizations make the same
choices. First divergence is a clean "different policies"; no divergence
over the whole probe is a clean "same policy". Nothing after the first
divergence is comparable — the graphs differ from the next step on — which
is why the statistic is the first index, not a rate.

P-2 now picks its decisive clause by class: per-rule TV for `StaticBias`,
selection divergence for `MlpScorer`. **Both are always reported**, and the
four failure messages are class-appropriate. Firing tests:
`p2_reads_a_scorer_by_its_selections_and_a_logit_policy_by_its_allocation`
and `selection_divergence_finds_the_first_differing_step`.

---

## 5. Why the search could not start: a Gumbel-sized flat basin at θ = 0

With P-2 reading the right statistic, the abort survived — and now says
something real:

```
selection divergence NONE over 64 probe steps
|dtheta| = 0.5414 = 0.01x the drift scale (sigma_init * sqrt(n) = 41.6173)
```

Seven generations moved the genome 0.54 in a 433-dimensional space where one
perturbation has norm 41.6, and **not one selection changed**.

The cause is architectural. `gumbel_top_k` adds `Gumbel(0, 1)` noise — sd
≈ 1.28 — to every raw score before ranking, so a scorer whose outputs differ
by much less than that is behaviourally identical to budget-matched random
*no matter what its parameters are*. And the output scale is **quadratic**
in the genome: a score is `Σ_h w2[h] · relu(b1[h] + w1[h]·x)`, and both
factors grow with `|θ|`. Measured over 24 candidate feature vectors on the
433-parameter scorer:

| `|θ|` | score spread | vs Gumbel sd 1.28 |
|---|---|---|
| 0 | 0.00000 | exactly flat |
| **0.5414** | **0.00548** | **234× smaller — where the campaign was** |
| 1 | 0.01880 | 68× smaller |
| 3 | 0.16918 | 8× smaller |
| 5 | 0.46994 | ~3× smaller |
| **7** | **0.92107** | **comparable** |
| 10 | 1.87974 | comparable |
| 20 | 7.51897 | 6× larger |
| 41.6 | 32.53011 | argmax |

So `θ = 0` is not merely a neutral start — it is the centre of a **wide
basin in which the policy cannot behave differently at all**, and the
basin's radius is set by the Gumbel temperature rather than by float
precision. Below `|θ| ≈ 3` the policy is random whatever its weights; above
`|θ| ≈ 20` it is a deterministic argmax the Gumbel draw cannot perturb.
Either extreme makes a search uninformative, in opposite ways.

This also explains §2 exactly: the untrained arm equals the random arm
because `θ = 0` is the deepest point of that basin.

### 5.1 The fix — initialize where the scorer can act

`prechecks::class_init` returns the campaign's starting genome per class:
zeros for logit classes (`StaticBias` at zero is the uniform allocation,
which is a genuinely good neutral start and is *not* degenerate), and a
seeded draw at `MLP_INIT_L2 = 7.0` for `MlpScorer` — the norm at which the
score spread matches the Gumbel scale, taken from the table above. Held as
an **L2 norm** rather than a per-component sd so it means the same thing
across genome sizes, exactly as P-2's drift scale does. Firing test:
`the_mlp_init_scale_lands_where_the_scorer_can_actually_act`.

### 5.2 A latent bug this exposed

The three-arm control pass hardcoded `Arm::Untrained`'s genome as
`vec![0.0f32; n_params]`. "Untrained" means *the genome the campaign
started from*, and those coincide only for classes whose neutral point is
zero. The moment any class initializes elsewhere, the arm table silently
scores a policy training never occupied — and the primary statistic is
`champion − untrained`. Now bound to `theta_init`. P-3's `theta = 0` probe
arm had the same defect and the same fix.

This was latent, not fired: every campaign to date initialized at zero, so
no published number is affected.

### 5.3 σ and α, derived rather than guessed

`OpenAiEs` evaluates candidates at `θ ± σ·ε` with `ε ~ N(0, I)`, and updates
by `Δθ = (α / (pairs · σ)) · Σ_p (r⁺_p − r⁻_p) · ε_p`. Two consequences the
table in §5 turns into constraints:

**1. σ sets where the *population* sits, and it must stay in the live band.**
`|σ·ε| ≈ σ√n`, so at `σ = 2.0` and `n = 433` the members sit at `|θ| ≈ 41.6`
— six times the init norm and deep in the argmax regime, where the Gumbel
draw no longer perturbs anything. Keeping the spread of the population to
about 30% of the init norm gives `σ√n ≈ 0.3 · 7 = 2.1`, so **σ ≈ 0.10**.

**2. α/σ sets the step, not α alone.** With ranks shaped to `[−0.5, 0.5]`
and typical antithetic differences `|r⁺ − r⁻| ≈ 0.3`,
`|Σ_p d_p ε_p| ≈ √(pairs · 0.09) · √n ≈ 25`, so
`|Δθ| per generation ≈ 1.56 · α/σ`. That model predicts `0.156` per
generation at `α = 0.2, σ = 2.0`, hence `0.156·√7 ≈ 0.41` over seven
generations of random-walking — against the **0.5414 measured**. Targeting a
step of ~7% of the init norm per generation gives `α/σ ≈ 0.32`, so at
`σ = 0.10`, **α ≈ 0.032**.

Note the sign: because σ divides the update, *raising* σ shrinks the step
while widening the exploration. The abort message's advice to "raise sigma"
is right about the symptom and wrong about the remedy for this class, and is
corrected accordingly.

---

## 6. Stage 3′ campaigns

16 pairs × 2 seeds × 30 generations, hidden = 16, `k = 4`, five frozen
hold-out seeds. Init at `MLP_INIT_L2 = 7.0` per §5.1.

### 6.1 F5 × `tri_two` — the exit criterion is met

At σ = 2.0, α = 0.2 (the pre-calibration run, 30.6 min):

| precheck | result |
|---|---|
| P-1 | PASS — 64/64 steps had ≥2 live rules |
| P-3 | PASS — spread 815.12 over max seed sd 43.93 |
| P-4 | PASS — unbounded objective, matched horizon instead |
| P-2 | PASS — TV 0.1016, **selection divergence at step 1**, \|dθ\| 0.5670 |

P-2 is worth pausing on: the *same* `|dθ| ≈ 0.57` that was frozen from a
zero init now diverges **at the first step**, because the init sits where the
scorer is behaviourally live. Same movement, opposite verdict — which is the
§5 basin argument confirmed from the other side.

| arm | fitness | active_persist | n_nodes | flags |
|---|---|---|---|---|
| random | 825.58 | 21.65 | 860.2 | 0 |
| untrained (init) | 784.65 | 20.11 | 859.0 | 0 |
| **champion** | **881.56** | **22.74** | 857.3 | **0** |

- **PRIMARY: champion − untrained = +96.90 at 3.65σ → exit MET.**
- Reported alongside: champion − random = +55.98 at **1.76σ**.

**Not Goodharted.** No tripwire fired, and the champion carries *more*
active persistent structure than either baseline (22.74 vs 21.65 / 20.11) at
an essentially identical node count. The failure mode F4 exhibits — winning
by shutting the substrate down — is absent here.

**The honest caveat, stated with the result.** The untrained arm (784.65)
scores *below* random (825.58), so part of the +96.90 is the champion
recovering from an initialization that is worse than chance. Against random
the margin is +55.98 at 1.76σ — real, and **under the 2σ bar**. Plan §3 is
explicit that `champion − untrained` is the exit criterion and
`champion − random` "is reported alongside it and is never the exit criterion
for a learning claim", so the exit is met on the statistic the plan
nominates. But a reader asking "does the trained scorer beat the simplest
baseline by a significant margin" gets *not at 2σ, on five seeds* — and that
question deserves its answer rather than the one the criterion happens to ask.

### 6.2 F4 × `wn_three` — `OBJECTIVE_BLIND`

```
spread across the class is 0.5832, under 2x the largest seed sd
(0.3161 -> bar 0.6321)
arms: campaign init = 0.6759 ± 0.2562
      allocation extreme = 1.1325 ± 0.3161
      budget-matched random = 0.5493 ± 0.2597
```

A near-miss (0.5832 against 0.6321) driven by F4's seed variance on
`wn_three`, which runs ±0.26 to ±0.32 on a scale whose ceiling is 2. The
arms *are* ordered as §3.1 predicted — the extreme is highest, at 1.13 — but
five seeds cannot separate them.

This abort is **σ-independent by construction** after §1.1: P-3's arms are
`class_init` and `class_extreme`, neither of which reads the campaign's
`sigma`. So it recurs identically under any step size, and the remedy is
seeds, not tuning.

---

## 7. The fourth arm

Plan §9 asks for untrained `StaticBias` alongside, as "the class-effect
reference the lattice and EI stages already established". It is a different
policy class, so it cannot come out of an `MlpScorer` campaign's own rows;
scored on the same frozen hold-out seeds:

| cell | `StaticBias` θ=0 | `StaticBias` extreme |
|---|---|---|
| F5 × `tri_two` | 721.42 ± 23.00 | **0.20 ± 0.45** |
| F4 × `wn_three` | 0.6612 ± 0.1658 | 1.1652 ± 0.2121 |

Two things read off this. Under **F5**, the allocation extreme is
catastrophic — 0.20 against 721 — so `tri_two`'s constructor reuse depends
on both rules firing, and a policy that concentrates on one destroys it.
Under **F4**, the same extreme *improves* the score (1.17 vs 0.66), in the
same direction as the seeded MLP. The two objectives disagree about what a
concentrated policy is worth, which is exactly the F3/F5 opposition §4B.1
predicts, now visible across objectives rather than within one.

---

## 7A. The same cells at the calibrated σ and α — the headline result

σ = 0.10, α = 0.032 per §5.3. **Both predictions recorded before the run
held exactly.**

1. **F4 × `wn_three` aborted `OBJECTIVE_BLIND` with spread 0.5832 against
   bar 0.6321 — byte-identical to the σ = 2.0 run.** P-3's arms are
   `class_init` and `class_extreme`, neither of which reads `sigma`, so the
   abort is σ-independent by construction, and it is. Had it moved, the §1.1
   fix would have been incomplete.
2. **The F5 cell took 72.2 min against 30.6.** Keeping the population near
   `|θ| ≈ 7` keeps the substrate active; σ = 2.0 pushed members to
   `|θ| ≈ 42` where the near-argmax policy suppressed activity and rollouts
   ran fast. A 2.4× cost for searching the right space.

### 7A.1 F5 × `tri_two`

| precheck | result |
|---|---|
| P-1 | PASS — 64/64 co-live |
| P-3 | PASS — spread 815.12 over max seed sd 43.93 |
| P-4 | PASS — unbounded objective |
| P-2 | PASS — TV **0.4297**, divergence at **step 0**, \|dθ\| **2.9508 = 1.42× drift** |

P-2 is the direct check that the calibration worked. At σ = 2.0 the search
had moved `0.01×` the drift scale in seven generations; here it has moved
**1.42×** — past one full perturbation, which is what a search that is
actually traversing looks like. The two knobs were picked to make that
happen and it happened.

| arm | fitness | active_persist | n_nodes | flags |
|---|---|---|---|---|
| random | 825.58 ± 41.82 | 21.65 | 860.2 | 0 |
| untrained (init) | 784.65 ± 33.62 | 20.11 | 859.0 | 0 |
| **champion** | **1166.67 ± 15.04** | **26.04** | 853.8 | **0** |

- **PRIMARY: champion − untrained = +382.02 at 14.67σ → exit MET.**
- champion − random = **+341.09 at 10.85σ**.

**§6.1's caveat is resolved.** Under the mis-scaled search the champion beat
random by only 1.76σ, and I flagged that a reader asking "does this beat the
simplest baseline significantly" got *no*. Under the properly-scaled search
the answer is **yes, at 10.85σ** — and the champion itself is far better
(1166.67 vs 881.56), which is the direct evidence that σ = 2.0 was searching
a policy space the initialization was not chosen for.

**Still not Goodharted, and now more clearly so.** No tripwire fired, and
the champion carries the *most* active persistent structure of any arm
(26.04 vs 21.65 random, 20.11 untrained) at a slightly *lower* node count.
`DEGENERATE_REUSE` — one constructor owning ≥90% of cycles on a collapsed
graph — did not fire, so the reuse depth is not concentration-by-collapse.
F5 and the held-out structural metrics agree here, which is precisely what
they do **not** do under F4 (§3.1).

### 7A.2 The verdict

**Stage 3′'s exit criterion is met on F5 × `tri_two`, and the result
survives both the primary and the secondary statistic.** Local
per-candidate context carries exploitable signal on this substrate under a
non-diversity objective — which is the first positive learning result the
programme has produced, against C-2's negative and the four objectives
before it.

**What limits it, stated rather than left to inference:**

1. **Under-powered relative to the plan's default** — 16 pairs vs 32, 2
   seeds/member vs 3, 30 generations vs 60. For a *positive* result this is
   conservative: a smaller search makes an effect harder to find, not
   easier. It would be the central caveat on a null; it is not one here.
2. **Five hold-out seeds**, so the pooled σ in "14.67σ" is estimated from
   five points and should not be read to two decimals. The effect size does
   not depend on that precision: +341 against a random-arm sd of 41.8 is
   eight sd's of separation.
3. **One substrate, one objective.** F4 × `wn_three` is blind at five seeds
   and `wn_three` under F5 failed P-3 back in D5. The claim is about F5 on
   `tri_two`, not about learned selection in general.
4. **`tri_two` is a two-rule composition**, and D5 §6 established there is
   no third non-fixture multi-rule substrate in the repository to replicate
   on. That bound is the programme's, not this stage's.

---

## 8. Housekeeping — two configs fixed

D5's standalone P-1 call passed `CognitionConfig::default()`, whose `budget`
is `None` — no truncation — while every other number in D5 used `k = 4`. The
co-liveness fraction it reported was therefore measured on an uncapped
substrate. Fixed; re-run, `tri_two` reads **64/64** co-live steps rather
than 2/64, and **the D5 verdict is unchanged** (3 of 4 cells pass, P-3 and
P-4 numbers bit-identical, since those arms always carried `k = 4`). No
published D5 number cited the old fraction.

The campaign runner was never affected: it passes `spec.cognition` to P-1,
which carries the real budget.

Second, a label. P-3's first arm was printed as `theta=0`, which stopped
being true the moment `class_init` gave scorers a seeded start — it is now
`campaign init`. A label that names the wrong thing is how the two arms in
§5.2 came to be conflated in the first place.

---

## 9. Provenance

| what | where |
|---|---|
| §1's probe table, §2's equivalence, §3.1 | `d6_stage3_prime_mlp.rs::untrained_mlp_is_random_and_what_a_rollout_costs` |
| §3's precheck matrix | `…::mlp_prechecks` |
| §6, §7A campaigns | `…::stage3_prime_campaigns` |
| §7's reference arm | `…::untrained_static_bias_reference_arm` |

Asserted rather than measured, in `prechecks::tests`:

| claim | test |
|---|---|
| §1 — the `StaticBias` probe is inert on an MLP | `the_static_bias_extreme_is_a_dead_probe_for_an_mlp` |
| §4.1 — traversal by class | `p2_reads_a_scorer_by_its_selections_and_a_logit_policy_by_its_allocation` |
| §4.1 — the divergence statistic | `selection_divergence_finds_the_first_differing_step` |
| §5.1 — the init scale is where the scorer can act | `the_mlp_init_scale_lands_where_the_scorer_can_actually_act` |

Suite at close: **278 passed, 0 failed** across `hg-train`, `hg-cognition`
and `hg-sim-core`.

One pre-existing test failed on the way and is worth recording, because it
is the discipline working. `d3_stage_i_demotion.rs`'s firing test asserted
that `[causal]` off demotes `constructor_records_examined` — the name D5
retired in favour of `causal_enabled` (`instrument_registry.md` §5.2). The
assertion's *intent* was right and unchanged; only the name of the fact it
checks moved, because the old one was a proxy that could not distinguish
"the layer never ran" from "it ran and found nothing". Updated to the new
gate, with the count still asserted zero alongside.

**Repo caveat, unchanged.** `crates/hg-selfmod/` carries uncommitted changes
that do not compile, so `cargo test --workspace` fails; the three crates
above are tested explicitly for that reason.
