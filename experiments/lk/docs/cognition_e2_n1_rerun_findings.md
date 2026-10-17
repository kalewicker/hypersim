# E2 — the D6 re-run under N-1, and what score normalization changed

Phase E2 of `hypersim_nn_experiments_plan_v3.md`, §4 and §10. Produced by
`crates/hg-train/tests/e2_n1_rerun.rs`.

> **N-1 does what plan §4 said it would, and the confirmation is stronger
> than the prediction: the policy is a function of θ's *direction alone*,
> exactly, and `‖θ‖ ∈ {1, 7, 42, 420}` produce byte-identical selections
> (§1).** But the same fact demotes three things that were load-bearing.
> `MLP_INIT_L2` stops being a tuning parameter (§1.3). The α/σ derivation's
> binding constraint **inverts** — σ was bounded below by signal visibility
> under raw scoring and is bounded above by local linearity under N-1 (§2).
> And the untrained arm stops being a point: `class_init` samples a
> *direction*, and across directions it spreads far wider than across seeds
> (§3), which changes the denominator the primary statistic needs.
>
> **E2's exit is met and plan §4's pre-registered expectation holds on both
> clauses** (§6). The margin survives — `champion − random` 10.85σ → 9.76σ —
> and the calibrated-vs-mis-scaled gap narrows as predicted, 9.09σ → 6.19σ.
> The prediction's *reason* clause turns out to be incomplete in a useful
> way: the gap narrowed by a third and stopped, because basin navigation was
> one of two causes and σ = 2.0 is still a random restart per generation,
> which N-1 neither addresses nor could (§4.2).
>
> **The result is not what the largest number in the table says.**
> `champion − untrained` reads 39.74σ against D6's 14.67σ, and that inflation
> is entirely the *baseline* falling, not the champion rising (§4.1). Powered
> to thirty draws, the claim that survives every statistic is: the champion
> beats budget-matched random by 9.76σ, beats **30/30** untrained directions
> of its own class, and exceeds the best of those thirty by **6.3%** —
> reliable learning of modest value (§5).

---

## 1. N-1 makes the policy a function of direction, exactly

### 1.1 The algebra, and why the constant term matters

A score is `s(θ) = b₂ + Σ_h w₂[h]·relu(b₁[h] + w₁[h]·x)`. Scale the whole
genome by `c > 0`. `relu` is positively homogeneous, so the hidden layer
contributes `c²·q(x)` and the output bias contributes `c·b₂`:

```
s(cθ) = c·b₂ + c²·q(x)
```

Neither term is invariant. But N-1 subtracts the pool mean before dividing,
and **`c·b₂` is identical for every candidate**, so it cancels in the
numerator. The surviving `c²` then cancels between numerator and
denominator. For `c²·sd(q) ≫ ε` the standardized scores are therefore
*exactly* invariant under `θ → cθ`.

The `b₂` cancellation is not incidental. Without the mean subtraction —
N-2, the temperature-scaling alternative plan §4 lists — the bias term
survives as a `1/c` contamination and the invariance is only approximate.
That is a reason to prefer N-1 beyond the ones §4 gives.

### 1.2 Measured, behaviourally, not argued

Selection divergence against a `‖θ‖ = 7` reference over a 64-step probe,
each norm compared to **its own** reference:

| `‖θ‖` | N-1 | raw |
|---|---|---|
| 0 | differs at step 0 | differs at step 1 |
| 10⁻⁴ … 10⁻² | differs at step 0 | differs at step 1 |
| 0.1 | differs at step 2 | differs at step 1 |
| 0.5 | differs at step 25 | differs at step 1 |
| **1** | **none — the same policy** | differs at step 1 |
| **7** | **none** | none (identity) |
| **42** | **none** | differs at step 0 |
| **420** | **none** | differs at step 0 |

Three orders of magnitude of genome norm, byte-identical selections. F5 on
the frozen hold-out agrees to every printed digit: `‖θ‖ = 7` and `‖θ‖ = 42`
both give **527.8449 ± 12.4916**.

The control matters here, because "no divergence" is also what a dead
instrument reports: a *different direction* at the same norm diverges at
step 0. And the raw column is compared against a raw reference rather than
an N-1 one — an earlier version of this table compared everything to the
N-1 reference, which can only restate that the two norms differ and
supports no conclusion about scale dependence within either.

### 1.3 The ε floor is a residual basin, and it is declared

`pool_standardize` divides by `sd + ε` with a fixed `ε = 1e-6`. A floor has
the same shape as the defect being removed: below `c²·sd(q) ≈ ε` the
denominator stops tracking the numerator and the spread shrinks with `‖θ‖`
again. The table locates that edge between `‖θ‖ = 0.5` and `1`.

**So N-1 shrinks the basin radius from ≈ 3 to ≈ 1 rather than deleting
it.** Plan §4 predicted "any small perturbation now produces real selection
change"; that is true above the floor and false below it, and the
difference is two orders of magnitude of θ rather than a rounding detail.

`MLP_INIT_L2 = 7.0` keeps its value and loses its justification. It was
chosen as the norm where raw score spread matches the Gumbel scale (D6
§5.1); §1.2 retires that reasoning entirely. It stays because it is two
orders of magnitude clear of the floor and because holding it fixed keeps
D6 and E2 genomes comparable — **not** because 7 is better than 42, which
the measurement says it demonstrably is not. It is no longer a tuning
parameter, which is a downgrade in status rather than in value.

### 1.4 The neutral point

`θ = 0` gives `sd = 0` gives uniform. On the frozen hold-out it scores
**830.4954 ± 28.4792** against budget-matched random's **815.1215 ±
38.6068** — 0.45 pooled sd, random-equivalent, as plan §5.1 requires to be
measured rather than asserted.

This reproduces D6 §2 numerically while changing the mechanism: the arm is
random-equivalent by *degenerate normalization* now, not by sub-threshold
spread.

---

## 2. σ's binding constraint inverts

### 2.1 What D6's derivation rested on

D6 §5.3 fixed σ by keeping the *population* inside the Gumbel-live band:
`σ√n ≈ 0.3·MLP_INIT_L2`, hence σ ≈ 0.10. §1.2 voids that constraint —
there is no band, because there is no `‖θ‖` dependence to have a band in.

The lower bound came from a measurement, though, and measurements outlive
the models built on them: at σ = 0.5 under raw scoring one perturbation
moved F5 by 41.43 against a seed sd of 53.00, i.e. below the noise the ES
would have to climb through. That is the constraint worth re-measuring.

### 2.2 Re-measured — and it no longer binds

Mean `|r⁺ − r⁻|` over 4 antithetic pairs, against the seed sd at the same
genome. Measured at **two** starting directions, because §3 shows one is
not a sample:

| σ | angle | at the campaign's init draw | at a second draw |
|---|---|---|---|
| 0.02 | 3.4° | 114.33 = **9.15×** | 28.99 = 0.90× |
| 0.05 | 8.5° | 211.61 = **16.94×** | 12.19 = 0.38× |
| 0.10 | 16.6° | 489.01 = **39.15×** | 66.00 = 2.05× |
| 0.30 | 41.7° | 667.24 = **53.41×** | 550.61 = 17.11× |
| 1.00 | 71.4° | 328.58 = 26.30× | 294.39 = 9.15× |
| 2.00 | 80.5° | 292.26 = 23.40× | 304.57 = 9.46× |

Two things fall out, and the second is the important one.

**The signal criterion is direction-dependent by an order of magnitude.**
At the campaign's own init, σ = 0.02 already clears seed noise 9×. At the
second draw the same σ is *under* the floor, and σ = 0.10 is marginal at
2.05×. The two points disagree about the lower bound. They also differ in
fitness — 527.84 against 1090.85 — in the direction that makes sense: the
better direction is the flatter one, consistent with sitting nearer an
optimum. **σ cannot be calibrated from a single probe point**, and D6
calibrated from one.

**`|r⁺ − r⁻|` stops being a signal statistic at large σ.** It is
non-monotone, peaking at σ = 0.30 and falling at σ ≥ 1. That is what it
should do if the large-σ values are not signal at all: at σ = 2.0 the two
arms sit 80.5° apart, so `|r⁺ − r⁻|` is measuring the spread between two
*unrelated* policies. §3 measures that spread independently — direction sd
224.50, so two independent draws differ by `≈ 2·224.50/√π ≈ 253` in
expectation. The observed σ = 2.0 values are **292.26 and 304.57**.

> At σ = 2.0 the antithetic difference is statistically indistinguishable
> from drawing two fresh genomes. The "search" is a random restart per
> generation, and it was so under D6's mis-scaled arm too.

A σ criterion built on `|r⁺ − r⁻|` would have licensed exactly that.

### 2.3 The criterion that survives — local linearity

`OpenAiEs` estimates a gradient from `(r⁺ − r⁻)`, and that is a gradient
only if `r(θ ± σε)` is approximately affine across the perturbation. Measured
directly, per pair, with the **same** ε draws at every σ so the rows differ
in step size alone:

```
kappa = |r⁺ + r⁻ − 2·r₀| / |r⁺ − r⁻|
```

The numerator is the second difference — precisely the curvature the linear
model discards; the denominator is the signal it extracts. `κ ≪ 1` is the
regime ES was derived for.

Measured at the campaign's own init, `r₀ = 518.2209`:

| σ | angle | `\|r⁺−r⁻\|` | `\|r⁺+r⁻−2r₀\|` | **κ** | reading |
|---|---|---|---|---|---|
| 0.02 | 3.4° | 121.28 | 13.35 | **0.11** | locally linear |
| 0.05 | 8.5° | 295.97 | 96.29 | **0.33** | locally linear |
| 0.10 | 16.6° | 379.72 | 233.23 | **0.61** | marginal |
| 0.30 | 41.7° | 473.43 | 431.20 | **0.91** | marginal |
| 1.00 | 71.4° | 309.61 | 703.70 | **2.27** | curvature-dominated |
| 2.00 | 80.5° | 315.99 | 705.18 | **2.23** | curvature-dominated |

Monotone through σ = 1 and then flat — which is the behaviour to expect if,
past σ ≈ 1, the perturbation is a fresh draw and κ has stopped varying with
σ because it is reporting the objective's generic curvature scale rather
than anything local. §2.2's independent argument puts the random-restart
boundary in the same place.

This is the criterion that survives N-1, because it is a statement about
the objective's shape rather than about the Gumbel geometry N-1 deleted.

**A statistic correction, recorded because it changed a verdict.** κ was
first computed as a mean of per-pair ratios. With four pairs and a noisy
denominator that is the classic unstable ratio estimator: at σ = 0.10 one
pair landed near `|r⁺ − r⁻| = 0` and the mean-of-ratios read **11.15**
against an aggregate of 0.61, i.e. it reported curvature domination three
rows early. The table above is the ratio of means; the naive value is
printed alongside it in the test so the divergence stays visible rather
than being hidden by the choice.

### 2.4 The verdict: no single σ satisfies both bounds

| | wants | because |
|---|---|---|
| signal, at the **flat** direction | σ ≥ 0.10 | §2.2: σ = 0.05 sits at 0.38× seed sd there |
| locality, at the **steep** direction | σ ≲ 0.07 | κ = 0.61 already at σ = 0.10 |

**These do not overlap.** The landscape's local scale varies across
directions at a fixed norm by more than a global σ can accommodate, which
is a property of the objective on this substrate and not a calibration
failure to be tuned away. The honest conclusion is that a fixed σ is the
wrong instrument here and a per-generation adaptive one (which CMA-ES
already provides) is the right one — recorded as a forward finding, not
acted on inside E2.

D6's σ = 0.10 sits exactly at the crossover. It remains a defensible
choice; it is not defensible *for D6's stated reason*, and E2's arm A runs
at it for comparability rather than because the re-derivation selected it.

### 2.5 α

`|Δθ| ≈ 1.56·α/σ` is a statement about `OpenAiEs`, not about the policy, so
it carries across N-1 unchanged. What does not carry is the *target*: D6
aimed at ~7% of the init norm per generation, and under scale invariance
"% of the init norm" is not a meaningful unit — the norm is inert. The
meaningful unit is angular, and at α = 0.032, σ = 0.10 the step is
`atan(1.56·0.32 / 7) ≈ 4.1°` per generation against a sampled width of
16.6°, i.e. about a quarter of the region sampled. That is a defensible
ratio and it is the re-derived justification. The value is unchanged.

**And the norm does not stay at 7.** Δθ has a radial component that is
behaviourally inert but arithmetically real, so ‖θ‖ drifts and both the
sampled width and the angular step shrink with it — an unintended annealing
schedule invisible in the fitness column. `genome_l2` (§6) is the column
that makes it visible; §4 reads it.

---

## 3. The untrained arm is a draw, not a point

Under N-1 `class_init` samples a **direction** (§1.2), so the campaign's
untrained arm is one draw from a distribution. Eight draws at `‖θ‖ = 7`,
each scored on the five frozen hold-out seeds:

| draw | F5 | vs random, in seed sd |
|---|---|---|
| 0 | 483.38 ± 79.24 | −8.59 |
| 1 | 934.89 ± 15.13 | +3.10 |
| 2 | 982.39 ± 39.06 | +4.33 |
| 3 | 840.25 ± 90.25 | +0.65 |
| 4 | 631.44 ± 31.89 | −4.76 |
| 5 | 366.93 ± 14.80 | −11.61 |
| 6 | 726.46 ± 31.00 | −2.30 |
| 7 | 910.97 ± 45.18 | +2.48 |

```
across DIRECTIONS   734.59 ± 224.50   (range 366.93 .. 982.39)
across SEEDS        ±  38.61
ratio                 5.81
```

Reproduced independently in the campaign pass (`e2_campaigns`) to the
printed digit, and measured there for `feature_set = "v2"` as well:

| features | class mean | direction sd | range |
|---|---|---|---|
| v1 | 734.59 | 224.50 | 366.93 … 982.39 |
| v2 | 729.14 | 242.61 | 357.9 … 957.1 |

E0's rescale therefore leaves the untrained *class baseline* essentially
where it was. That is worth having before reading §4's arm C: any
champion difference between arm A and arm C is a difference in what the
encoding makes learnable, not a shifted starting point.

Two consequences. (A third reading — that the class baseline sits *below*
random, since 734.59 < 815.12 — is what these eight draws appear to say and
is **wrong**; §5 draws thirty and finds a distribution whose mean is not a
summary of it.)

**The primary statistic needs a second denominator.**
`champion − untrained` divided by pooled *seed* sd answers "did training
beat **this draw**". The question the arm's name implies — "did training
beat an untrained network of this class" — has 224.50 in the denominator,
not 38.61. Both are reported in §4; neither replaces the other, and the
plan's nominated statistic is not quietly swapped out.

This does **not** affect the A-vs-B comparison that E2 exists to make: both
campaigns start from the same `campaign_seed`, hence the same draw, so the
direction variance is held fixed between them.

### 3.1 An instrument gap this opens

`docs/campaign_output.md`'s Neutral Point block (v3 §5.1) specifies
`neutral_vs_random_sigma` in pooled hold-out sd — a **seed** denominator.
For a class whose neutral point is a draw, that column understates the
uncertainty by the ratio above. Recorded here rather than silently fixed,
because the fix is a spec change and belongs with the next output-format
revision.

---

## 4. The campaigns

Three, all under N-1, all F5 × `tri_two`, all 16 pairs × 2 seeds × 30
generations from `campaign_seed = 9_600_000` — so all three start from the
same untrained draw and differ only in the variable named.

| | features | σ, α | champion | − untrained | − random | wall |
|---|---|---|---|---|---|---|
| **A** | v1 | 0.10, 0.032 | **1148.81 ± 21.12** | +624.50 at **39.74σ** | +323.23 at **9.76σ** | 70.0 min |
| **B** | v1 | 2.0, 0.2 | 956.32 ± 30.57 | +432.01 at 19.49σ | +130.74 at **3.57σ** | 46.1 min |
| **C** | v2 | 0.10, 0.032 | 1115.76 ± 28.35 | +590.95 at 19.60σ | +290.19 at 8.12σ | 68.2 min |

Common to all three: random 825.5782 ± 41.8234, untrained 524.31 ± 6.93
(524.82 ± 31.84 under v2). **Every precheck passed and no tripwire fired in
any arm.**

The random arm is bit-identical to D6's — 825.5782 ± 41.8234 — as it must
be, since `RandomSubset` does not produce scores and so cannot be affected
by how scores are normalized. It is the free consistency check on the whole
comparison.

### 4.1 Arm A against D6 — the margin survives, and the reason it grew is not learning

| | D6 (raw) | E2 (N-1) |
|---|---|---|
| random | 825.58 ± 41.82 | 825.58 ± 41.82 |
| untrained | 784.65 ± 33.62 | **524.31 ± 6.93** |
| champion | 1166.67 ± 15.04 | 1148.81 ± 21.12 |
| champion − untrained | +382.02 at 14.67σ | +624.50 at 39.74σ |
| champion − random | +341.09 at 10.85σ | +323.23 at 9.76σ |

**The champion barely moved** (1166.67 → 1148.81, inside a pooled sd). What
moved is the *untrained* arm, by −260, on the same genome and the same
seeds. §1.2 explains it exactly: at ‖θ‖ = 7 under raw scoring the score
spread is 0.92 against a Gumbel sd of 1.28, so the policy was only
*partially* expressed and sat closer to random than its parameters imply.
N-1 expresses it fully, and this particular direction is a bad one.

So `champion − untrained` inflating from 14.67σ to 39.74σ is **not** evidence
that N-1 improved learning. It is evidence that the baseline fell. The
statistic that did not depend on the baseline — `champion − random` —
essentially held, 10.85σ → 9.76σ. Reading the 39.74 as a strengthened
result would be exactly the mistake the three-arm protocol exists to
prevent, and it is stated here rather than left for a reader to notice.

**Not Goodharted.** `active_persistent` structure: champion 26.05 > random
21.65 > untrained 14.24, at essentially equal node counts (858.8 / 860.2 /
854.4). The champion carries the most persistent structure of any arm while
scoring highest, which is the agreement between F5 and the held-out
structural metrics that D6 §7A.1 also found.

### 4.2 Arm B — the pre-registered prediction, and the part of the gap N-1 cannot fix

Plan §4 predicted the calibrated-vs-mis-scaled advantage would **shrink**.
On `champion − random`, the statistic the prediction names:

| | calibrated | mis-scaled | gap |
|---|---|---|---|
| D6 (raw) | 10.85σ | 1.76σ | **9.09σ**, a 6.16× ratio |
| E2 (N-1) | 9.76σ | 3.57σ | **6.19σ**, a 2.73× ratio |

**The prediction held.** It holds on absolute fitness too: the two
champions were 285.11 apart under raw scoring and are 192.49 apart under
N-1, a 32% narrowing.

**But the gap did not close, and §2 says why it could not.** The pre-fix
gap had two independent causes and N-1 removes exactly one:

1. *Basin navigation* — at σ = 2.0 the population sat at ‖θ‖ ≈ 42, deep in
   the argmax regime where the Gumbel draw perturbs nothing. N-1 deletes
   this failure mode outright.
2. *Non-locality* — at σ = 2.0 the antithetic arms sit 80.5° apart, κ = 2.23,
   and `|r⁺ − r⁻|` is indistinguishable from the spread between two fresh
   draws (§2.2). **This is a random restart per generation, and N-1 does not
   touch it**, because it is a property of σ against the objective's
   curvature rather than of the score scale.

The measured narrowing is therefore a *measurement of cause 1's size*: about
a third of the original gap was the basin, and two thirds was the search
never being local. That is a sharper result than the prediction asked for.

P-2 makes cause 2 visible directly: arm B moved `|dθ| = 0.5854 = 0.01×` its
drift scale over the probe — the same 0.01× that D6 §5 reported as a frozen
search — yet its selections diverge at step 2, so the policy is live. Same
movement, opposite verdict, because under N-1 movement in a 41.6-wide
perturbation is behaviourally real. The search is not frozen; it is
thrashing.

### 4.3 Arm C — `feature_set` v2 is a null on this cell

| | v1 (arm A) | v2 (arm C) |
|---|---|---|
| untrained | 524.31 ± 6.93 | 524.82 ± 31.84 |
| champion | 1148.81 ± 21.12 | 1115.76 ± 28.35 |
| champion − random | 9.76σ | 8.12σ |

The champion difference is −33.05 against arm sds of 21–28, i.e. about 1.3
pooled sd. **E0's rescale makes no measurable difference to what is
learnable here**, in either direction, and §3's class baselines say the same
about the starting point (734.59 vs 729.14).

This is a null, and it is not an argument against v2. E0's case for the
rescale was that six fixed scales were mis-calibrated against their
*observable* ranges, with candidate count 78.9% saturated on `tri_two` —
and saturation destroys distinctions the encoding could otherwise carry.
That defect is real whether or not this particular cell's champion exploits
it. The forward reason to keep v2 is E4: substrates authored under plan §8
will have different scales, and a fixed scale calibrated on `tri_two` has
no claim on them. **The reason is not a measured gain, and this record says
so rather than letting the rescale acquire one retroactively.**

One incidental difference worth noting: arm C's untrained arm has 4.6× the
seed sd of arm A's on the same direction (31.84 vs 6.93). The v2 encoding
makes this genome more seed-sensitive without moving its mean.

### 4.4 The class-level control — the one number that does not clear the bar

| arm | champion − untrained **draw** (seed sd) | champion − class **mean** (direction sd) |
|---|---|---|
| A | +624.50 at **39.74σ** | +414.22 at **1.85** |
| B | +432.01 at 19.49σ | +221.73 at 0.99 |
| C | +590.95 at 19.60σ | +386.62 at 1.59 |

The plan's exit criterion is `champion − untrained > 2σ on frozen hold-out
seeds`, and on that statistic **arm A passes at 39.74σ**. The class-level
question — did training beat *an untrained network of this class*, rather
than the one draw the campaign happened to start from — puts 224.50 in the
denominator instead of 6.93, and arm A gets **1.85**, under 2.

These are not in conflict; they are different questions, and §3 is why both
have to be asked. The 39.74 is inflated by a bad draw (§4.1) and the 1.85
is estimated from eight draws. §5 powers it.

---

## 5. The class control at thirty draws — and a statistic that assumed a shape the data does not have

§4.4's 1.85 rests on an sd estimated from eight draws, where the sd of an
sd is about 25%. Thirty draws, same protocol, same five hold-out seeds:

```
30 draws:  686.31 +- 372.61     (min 12.47, max 1080.26)
deciles:   12  22  284  455  624  840  924  982  1007  1059  1080
```

**The distribution is severely left-skewed, and a mean-and-sd is not a
summary of it.** A minority of directions produce essentially dead policies
— two draws under 25 — while deciles 5 through 10 are packed into
840…1080. The eight-draw sample missed the dead tail entirely (its minimum
was 366.93), which is why its sd was 224.50 and the thirty-draw sd is
372.61.

That resolves §3's apparent finding the other way. The eight-draw *mean*
(734.59) sits below budget-matched random (825.58) and reads as "the
architecture at random weights actively hurts". The thirty-draw median is
**840**, above random, and **16/30 draws beat random outright**. The mean is
dragged below random by the dead tail, not by typical behaviour. A random
`MlpScorer` direction is roughly a coin flip against budget-matched random,
with a minority of catastrophic draws — which is a different class fact
from the one the mean implied, and the earlier statement is withdrawn.

### 5.1 The two statistics disagree, and the parametric one is wrong

| statistic | value | what it assumes |
|---|---|---|
| parametric, `(champ − mean)/sd` | **1.24** direction-sd | a normal upper tail |
| nonparametric, draws reaching the champion | **0/30** | nothing |

The parametric z *fell* from 1.85 to 1.24 on more data — not because the
champion got worse but because the dead tail inflated the sd and pulled the
mean down. Both moves are in the denominator and the numerator of a
statistic whose question is about the **upper** tail. It is measuring the
wrong end of the distribution, and on a shape this skewed it has no
interpretation. **It is rejected, not merely caveated.**

The nonparametric count is the answer: **no random direction, in thirty,
reaches the trained champion.**

### 5.2 What the upper tail does say — the honest limit on the result

| | |
|---|---|
| champion (arm A) | 1148.81 |
| best of 30 draws | 1080.26 |
| gap | **6.3%** |

So the two readings that both survive:

> **Training is reliable — 0/30 random directions match the champion — and
> its margin over the best random direction is 6.3%.**

High reliability, modest value. That is a more informative pair than either
"39.74σ" or "1.85 sd", and neither of the two would have surfaced it: the
seed-denominator statistic is inflated by a bad draw, and the
direction-denominator statistic is measuring a tail it cannot see.

### 5.3 What this does to the exit criterion

Nothing, and that is worth stating explicitly rather than leaving implicit.
Plan §9's exit is `champion − untrained > 2σ on frozen hold-out seeds`;
arm A returns 39.74σ and the criterion is met. §5.1–5.2 do not contradict
it — they answer a *different* question that the criterion does not ask,
and the D-phase's own lesson is that the answer to an unasked question is
still data. The learning claim that survives all four statistics is:

> On F5 × `tri_two`, a trained `MlpScorer` beats budget-matched random by
> **9.76σ**, beats **30/30** untrained directions of its own class, and
> exceeds the best of those thirty by **6.3%**.

---

## 6. Adjudicating plan §4's pre-registered expectation

Plan §4, recorded before any E2 campaign ran:

> Pre-registered expectation: the margin survives and the calibrated-α
> advantage over the mis-scaled search (10.85σ vs 1.76σ) shrinks, because
> the mis-scaling was a basin-navigation failure that N-1 removes.

**Both clauses hold.** The margin survives (§4.1: `champion − random`
10.85σ → 9.76σ, and the exit criterion is met). The advantage shrinks
(§4.2: gap 9.09σ → 6.19σ, ratio 6.16× → 2.73×, absolute 285.11 → 192.49).

The *reason* clause is where the measurement adds something the prediction
did not contain. §4.2 shows the gap narrowed by about a third and stopped,
and §2 identifies what the remaining two thirds is: at σ = 2.0 the search is
a random restart per generation — κ = 2.23, arms 80.5° apart, `|r⁺ − r⁻|`
statistically indistinguishable from two fresh draws. **Basin navigation was
one of two causes, not the cause**, and N-1 removes only the one it was
designed to remove. The prediction was right about direction and incomplete
about mechanism, and the incompleteness is the useful part.

### 6.1 Plan risk 1

> If the margin does not survive E2, the honest reading is that D6's margin
> depended on the basin's geometry — which would be a finding about the
> architecture, not about the substrate.

**Risk 1 did not fire.** The margin survives, and §4.1 shows the champion
itself is essentially unchanged (1166.67 → 1148.81, inside a pooled sd). D6's
result did not depend on the basin's geometry. What did depend on it was
D6's *untrained baseline*, which the basin held artificially close to
random — so the one number that moved is the one the risk did not name.

---

## 7. Instrument changes shipped with this phase

| change | why it is not cleanup |
|---|---|
| `genome_l2` on every row (CSV schema 3 → 4) | N-1 makes the radial coordinate behaviourally inert while σ and α stay absolute, so a drifting norm shrinks the sampled angular width `atan(σ√n/‖θ‖)` and the angular step together. A campaign can anneal itself monotonically with **nothing in the fitness column showing it**, and `genome_hash` cannot distinguish a norm change from any other change. |
| `docs/policy_config.md` §4C | E1 shipped `score_norm` as a config key and `config.rs` referenced a `§4C` that was never written; the key table also still claimed only `"v1"` features were accepted. |
| `docs/policy_config.md` §4D | Plan v3 §10's outstanding E0 item: `MLP_PROBE_SCALE` and `MLP_INIT_L2` declared, with the reason each is deliberately *not* config. |
| `docs/policy_config.md` §4E | The ≥5-draw gate discipline, with the `GATE_UNDERDRAWN` / `GATE_UNSTABLE` split stated as a distinction rather than two names for one refusal. |

---

## 8. Provenance

| what | where |
|---|---|
| §1 — scale invariance, the ε floor, the neutral point | `e2_n1_rerun.rs::where_the_neutral_point_sits_under_n1` |
| §2.2 — the σ sweep at two directions | `…::sigma_re_derived_from_the_antithetic_signal` |
| §2.3 — local linearity | `…::sigma_bounded_above_by_local_linearity` |
| §3 — the 8-draw direction spread | `…::where_the_neutral_point_sits_under_n1` |
| §4 — the three campaigns | `…::e2_campaigns` |
| §5 — the 30-draw class control | `…::the_class_control_at_thirty_draws` |

Reproducibility observed in passing, and worth recording because nothing
forced it: the 8-draw direction sweep returns 734.5872 ± 224.4963 in both
`where_the_neutral_point_sits_under_n1` and `e2_campaigns`, two separate
test binaries' invocations; §2.3's raw measurements are bit-identical across
the two runs of that test; and the `random` arm is 825.5782 ± 41.8234 in
all three campaigns and in D6.

`the_class_control_at_thirty_draws` hard-codes arm A's champion (1148.8103)
rather than recomputing it. That is deliberate — the test adjudicates a
published number and must fail loudly if that number is ever re-derived
differently, rather than quietly following it.

**Repo caveat, unchanged.** `crates/hg-selfmod/` carries uncommitted changes
that do not compile, so `cargo test --workspace` fails; `hg-train`,
`hg-cognition` and `hg-sim-core` are tested explicitly for that reason.
