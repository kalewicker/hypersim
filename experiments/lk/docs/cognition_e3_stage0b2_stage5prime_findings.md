# E3 — Stage 0b″ and Stage 5′

Phase E3 of `hypersim_nn_experiments_plan_v3.md`, §9.1, §9.2 and §10.
Produced by `crates/hg-train/tests/e3_stage0b2_stage5prime.rs`.

> **Stage 0b″ was built exactly as D4 §2.5 specified, and it does not work.**
> `triangle_subdivide` goes from 5/8 hard-bound violations under the old
> single-pair reference to **8/8** under the pooled one, minimum ratio 1.003.
> Averaging the reference over four disjoint pairs should have halved its
> variance; the impossibility it was supposed to remove got more frequent.
>
> **The reason is upstream of the normalizer D4 blamed.** Stage 0b calls
> `(seed, seed+1)` a *perturbed* pair, but the seed reaches the run only
> through the policy RNG and `ChaCha8Rng` at 4242 and 4243 produces
> uncorrelated streams. `seed + 1` is an independent draw, so the numerator
> and the denominator estimate the same quantity. Measured: the two
> distributions are separated by **1.1–1.4 standard errors**, where the
> premise needs near-total separation (§2). **There is no perturbation, so
> there is nothing for the ratio to normalize** — and that accounts for
> every symptom D4 §2.2 recorded.
>
> **Stage 5′'s pre-registered prediction is not held on any substrate, and
> fails in the term it was most confident about**: `ei_micro` was predicted
> to *rise* and **falls** by 3.2–7.6 sd on three of four. `ce_structural`
> was predicted to fall; it never falls, and rises significantly on one
> substrate only. On `tri_two` — the substrate §7.1 was written about —
> neither term moves, which is §7.2's own third outcome and an exit (§4).
>
> **F6 is a NO-GO** (§5), not on cost but because §7.1's first link is
> falsified: F5 does not push `ei_micro` up, so there is no measured
> suppression of emergence for a corrective objective to correct.
>
> The finding that survives is one nobody predicted in either direction:
> **F5's champions reduce micro effective information wherever the
> measurement has power.**

---

## 1. Stage 0b″ as specified

D4 §2.5's three clauses, all implemented in `diagnostics::stage0b_pooled`:

1. the independent reference is averaged over `REFERENCE_PAIRS = 4`
   **disjoint** pairs — no run appears in two pairs, so it is an average
   over independent samples rather than correlated re-uses of one
   trajectory;
2. the whole triple repeats over 8 base seeds (plan §5.2's floor is 5, and
   8 matches D4 §2.2 so old and new are read at equal power), with the
   ratio reported with its spread;
3. the hard bound stays live — a ratio above 1 is *counted*, not silently
   relabelled `Chaotic`.

A class is returned only when every draw agrees; otherwise `Undetermined`
**with the distribution attached**, per §9.1's instruction that a
distribution-spanning class is not resolved by fiat.

### 1.1 Results, both readouts, 8 draws

| rule set | r1 | ratio median (min–max) | violations | r2 | violations |
|---|---|---|---|---|---|
| `glider` | Undetermined ×8 | — (reference 0) | 0/8 | Undetermined ×8 | 0/8 |
| `binary_edge` | Undetermined ×8 | — (reference ≈0) | 0/8 | Undetermined ×8 | 0/8 |
| `growth_treadmill` | Undet. [Chaotic×6 Viable×2] | 0.901 (0.527–0.948) | 0/8 | Undet. [Chaotic×3 Viable×5] | 1/8 |
| `triangle_subdivide` | **Chaotic ×8** | 1.273 (1.003–1.938) | **8/8** | **Chaotic ×8** | 7/8 |
| `fl1_two_rule` | Undet. [Chaotic×4 Viable×4] | 0.866 (0.534–1.268) | 3/8 | Undet. [Chaotic×5 Viable×3] | 3/8 |
| `tri_two` | Undet. [Viable×5 Chaotic×3] | 0.755 (0.459–1.384) | 2/8 | Undet. [Viable×4 Chaotic×4] | 2/8 |
| `relay_tri` | Undet. [Viable×3 Chaotic×5] | 1.105 (0.589–1.454) | 5/8 | Undet. [Chaotic×7 Viable×1] | 5/8 |
| `fan_tri` | Undet. [Chaotic×6 Viable×2] | 1.167 (0.656–1.473) | 4/8 | Undet. [Chaotic×7 Viable×1] | 4/8 |
| `fan_pure` | Undet. [Chaotic×6 Viable×2] | 1.095 (0.288–1.228) | 5/8 | Undet. [Chaotic×6 Viable×2] | 4/8 |

`glider` returns `Undetermined ×8` with a reference of exactly 0 under both
readouts, which is what `policy_config.md` §4B.6's isomorphism-invariance
proof predicts and is the one reassuring row in the table.

**`triangle_subdivide` is the row that condemns the fix.** Its median ratio
is 1.273 and its *minimum over eight draws* is 1.003 — every draw exceeds a
bound that no real distance can exceed. Under D4's single-pair reference it
violated on 5/8. Pooling made it worse.

### 1.2 Two contributing details, neither of them the story

Recorded because they are real and would otherwise look like the
explanation:

- **Making the pairs disjoint removed a stabilizer.** D4's reference was
  `jsd(a, far)` — sharing run `a` with the numerator `jsd(a, b)`. That
  shared run correlates numerator and denominator, so an unusual `a` moves
  both together and the ratio is pulled toward 1. Disjointness is the
  statistically cleaner choice and it removed that pull.
- **A single draw over a mean-of-four is biased above 1.** By Jensen,
  `E[1/Ȳ] > 1/E[Ȳ]`, so even when numerator and denominator are identically
  distributed the expected ratio exceeds 1. Pooling *shrinks* this bias
  rather than creating it, so it cannot explain 5/8 → 8/8 either.

Neither accounts for a minimum of 1.003 across eight draws. §2 does.

---

## 2. The premise, tested: `seed + 1` is not a perturbation

Stage 0b's docstring is explicit that the perturbation channel is the seed:
the initial conditions here are constructed, so a different seed changes
only the policy RNG stream, and a budgeted stochastic policy is what makes
that stream matter.

But **a different seed is not a small perturbation.** `ChaCha8Rng` seeded at
4242 and 4243 produces completely uncorrelated output, so the two runs make
different choices from step 0 — which is precisely the condition the
*reference* pair is supposed to represent.

If that is right, the numerator and the denominator are samples from one
distribution, the ratio's expected value is ~1, and the classifier's
thresholds at 0.05 and 0.8 are cutting noise.

**Measured directly.** 12 adjacent pairs `(s, s+1)` against 12 far-apart
pairs, disjoint seeds throughout, r1 readout:

| substrate | adjacent | far | P(adjacent < far) | distance from 0.5 |
|---|---|---|---|---|
| `growth_treadmill` | 0.07410 | 0.00586 | **0.67 ± 0.12** | +1.44 sd |
| `triangle_subdivide` | 0.00880 | 0.01066 | **0.63 ± 0.12** | +1.10 sd |
| `fl1_two_rule` | 0.00835 | 0.00935 | **0.67 ± 0.12** | +1.39 sd |

Stage 0b's premise requires this dominance near **1.0** — the perturbed pair
staying systematically closer than two unrelated seeds is the entire content
of "the perturbation decayed". It is instead within 1.4 standard errors of a
coin flip.

### 2.1 The standard error is load-bearing

0.63–0.67 reads like a weak effect until the denominator is checked: the 144
pairwise comparisons come from **12 + 12 independent runs**, not 144
independent observations. The Mann–Whitney null SE is
`sqrt(n·m·(n+m+1)/12) / (n·m)` = 0.12 here, so the three values sit at
+1.10, +1.39 and +1.44 sd.

Reporting 0.67 without it would have been the same error this programme has
now made and caught in E2 §5 (a parametric z on a skewed distribution) and
in E2 §2.3 (a mean of ratios at n = 4). It is the third instance of one
shape: **a summary statistic quoted without the spread of the thing it
summarizes.**

`growth_treadmill` shows the same hazard from the other side — its *mean
ratio* is 12.637 while its dominance is 0.67, because the adjacent group has
a heavy upper tail. The mean is not a summary of that distribution and the
dominance is the statistic to read.

### 2.2 This explains D4 §2.2 completely

Every symptom follows from "numerator and denominator are the same
quantity":

| D4 observation | what one distribution predicts |
|---|---|
| ratios straddling 1 | expected ratio ≈ 1 |
| violations on ~half of draws | P(ratio > 1) ≈ ½ |
| classes flipping between draws | thresholds cutting noise |
| `binary_edge` ratio **15.818** | denominator occasionally near 0 |
| r2 not repairing anything | never a readout problem |

D4 §2.3 concluded "the fault is in the normalizer". It is more accurate to
say the fault is that **there is nothing to normalize**: the reference is
not an upper bound on the numerator, it is a second sample of it.

### 2.3 What Stage 0b would need to become a measurement

Not scheduled here, stated so it is not re-derived:

1. **A genuinely small perturbation** — a single flipped node label, or a
   forced divergence at one step with a shared RNG stream before and after.
   Two seeds cannot express "small", because the RNG has no metric.
2. **A shared-prefix design.** Run to step *k* once, fork, perturb one arm,
   continue both from the same state. The reference then remains "two
   independent runs" and is a genuine upper bound.
3. Only then do the pooled reference and the ≥5-draw discipline built here
   become useful — they are correct machinery attached to an ill-posed
   quantity.

---

## 3. Stage 0b″ against its exit criterion

> **Exit:** every rule set has a class with a reported spread, and no class
> rests on a single draw. Classes that remain distribution-spanning across
> two categories are reported as `Undetermined` with the distribution, not
> resolved by fiat.

**Met, in the letter and uselessly.** Every rule set returns a verdict
carrying its distribution and its violation count; no class rests on one
draw, and the API cannot produce one (`no_separation_class_rests_on_a_single_draw`).
Seven of nine rule sets come back `Undetermined` with a distribution
attached, which §9.1 explicitly counts as a satisfied exit.

**But the exit is satisfied by a statistic §2 shows is ill-posed**, so the
honest report is that Stage 0b has no admissible classes on any rule set —
including `triangle_subdivide`, whose unanimous `Chaotic ×8` is unanimous
agreement on a ratio that violated the hard bound in every draw producing
it. That is not a class; it is eight impossibilities in a row.

`docs/cognition_stage0_findings.md` §2's separation table was already
demoted to *unmeasured* by D4. It stays demoted, and the reason changes:
not "pending the §2.5 redesign" but **pending a perturbation**.

---

## 4. Stage 5′ — the pre-registered emergence prediction

Plan §7.2, recorded before this ran:

> **Pre-registered:** the champion shows higher `ei_micro` and lower
> `ce_structural` than budget-matched random at matched budget, with
> `ei_macro` ambiguous in sign.

Three arms at matched budget (k = 4), the E2/E4 campaign design unchanged,
champion genomes taken in-process from `CampaignOutcome` rather than
reloaded. Adjudicated on four substrates rather than the one §7.2 was
written for.

**Statistics are per hold-out seed, n = 5.** The ~42 emission windows inside
one rollout are one trajectory and are correlated; pooling 5 × 42 into a
single mean and quoting it bare would treat 210 dependent observations as
210 independent ones. A first pass did exactly that and its verdict —
"failed in both terms on 4/4" — does not survive the correction (§4.2).

### 4.1 Results

| substrate | Δ`ei_micro` | Δ`ce_structural` | Δ`ei_macro` |
|---|---|---|---|
| `tri_two` | −0.143 (**−1.48 sd**, flat) | +0.006 (+0.50 sd, flat) | −0.046 (−0.92 sd, flat) |
| `relay_tri` | −0.332 (**−3.23 sd**) | +0.051 (**+2.54 sd**) | +0.115 (+1.35 sd, flat) |
| `fan_tri` | −0.702 (**−5.06 sd**) | +0.031 (+1.80 sd, flat) | −0.161 (−1.81 sd, flat) |
| `fan_pure` | −1.103 (**−7.64 sd**) | +0.013 (+0.51 sd, flat) | −0.363 (**−2.53 sd**) |

All deltas are champion − random, in pooled per-seed sd.

**The prediction is not held on any substrate, and it is wrong in the term
it is most confident about.** `ei_micro` was predicted to *rise*; it **falls**
on three of four, by 3.2 to 7.6 sd. `ce_structural` was predicted to fall;
it never falls, and rises significantly on only one.

### 4.2 `tri_two` returns §7.2's third outcome, and it is an exit

On the substrate the tension was theorized for, **neither term moves**:
`ei_micro` −1.48 sd, `ce_structural` +0.50 sd, `ei_macro` −0.92 sd. That is
§7.2's outcome 3 verbatim —

> **Both flat** → F5's reuse pressure is too weak at this budget to move
> either term, and the tension is theoretical rather than operative on
> `tri_two`.

— and §7.2 is explicit that all three outcomes are exits, none a failure.

### 4.3 What C-4 does and does not get

§7.2 offered Stage 5′ as "C-4's strongest test: a *policy-level*
confirmation of a mechanism established at the budget level". C-4 holds that
`ei_micro` up implies `ce_structural` down — a negative relationship.

Taking only significant moves, that relationship is tested on three
substrates and **confirmed on one**:

| substrate | `ei_micro` | `ce_structural` | reads as |
|---|---|---|---|
| `relay_tri` | down (3.23 sd) | up (2.54 sd) | **consistent with C-4** |
| `fan_tri` | down (5.06 sd) | up but flat (1.80 sd) | weakly consistent |
| `fan_pure` | down (7.64 sd) | **unmoved** (0.51 sd) | **not what C-4 predicts** |
| `tri_two` | flat | flat | no test |

`fan_pure` is the informative one. `ei_micro` falls 21.5% at 7.64 sd and
`ce_structural` does not move, because `ei_macro` falls too (−2.53 sd). The
champion there is not trading micro determinism for macro structure; it is
shrinking **both levels roughly proportionally**, and a ratio is blind to
that.

So Stage 5′ does not deliver the policy-level confirmation §7.2 hoped for.
It delivers one consistent case, one weak one, and one case where a large
`ei_micro` move produced no CE response at all.

### 4.4 `ce_structural` is a ratio, and that is load-bearing here

`ce_structural = ei_macro / ei_micro` (`hg-causal/src/ei.rs:435`) — checked
in the source, not inferred from the numbers. A rise in it is therefore
**not** evidence of more macro structure; it is equally consistent with less
micro structure. `relay_tri`, the one substrate with a significant CE rise,
is also the only one whose `ei_macro` moved *upward* at all (+1.35 sd, itself
not significant), so even there the rise is carried mostly by the
denominator.

### 4.5 A size mechanism, suggested and not established

E4 §5 found champions running the smallest graph of any arm. Against the
`ei_micro` moves:

| substrate | champion `n_nodes` vs random | Δ`ei_micro` |
|---|---|---|
| `tri_two` | −0.2% | −1.48 sd (flat) |
| `relay_tri` | −1.6% | −3.23 sd |
| `fan_pure` | −7.7% | −7.64 sd |
| `fan_tri` | −13.1% | −5.06 sd |

The substrate whose graph size does not change is the one whose `ei_micro`
does not change, and the three that shrink all fall significantly. Rank
order agrees but for one swap (Spearman ρ = 0.8). **With n = 4 this is a
hint and is recorded as one** — it is the obvious candidate mechanism, it is
consistent with E4 §5, and it is not tested here.

---

## 5. F6 go/no-go — **NO-GO**

§7.4 gates F6 behind Stage 5′'s outcome and two cost checks. The cost checks
were never reached, because the outcome removes the motivation.

F6 exists to answer §7.1's claim that "the objective that made
selection-learning stateable is aimed against causal emergence". That claim
has a chain: F5 rewards repeated firing → higher `ei_micro` → (C-4) → lower
`ce_structural`. **§4.1 falsifies the first link.** F5's champions have
*lower* `ei_micro`, on every substrate where the term moves at all.

Four reasons, in order of weight:

1. **The premise is falsified.** F5 does not push `ei_micro` up. Whatever it
   selects for, it is not the higher micro determinism §7.1 assumed.
2. **`ce_structural` never falls.** Across four substrates it rises on one
   and is flat on three. There is no measured suppression of emergence for a
   corrective objective to correct.
3. **On `tri_two` the tension is not operative** (§4.2), and `tri_two` is
   the substrate §7.1 was written about.
4. **F6's Goodhart route is already visible, and the specified floor does
   not cover it.** §7.4 anticipated that "F6's cheapest optimum is degrading
   micro determinism" and specified a floor against the *noise* route — a
   Frozen-gate threshold on absolute structure metrics. The route measured
   here is **size**: `fan_pure`'s champion cuts `ei_micro` 21.5% while
   running a 7.7% smaller graph, and a graph of 1040 nodes against 1128 is
   nowhere near Frozen. A floor aimed at noise does not block shrinkage.

**Recommendation.** F6 is not built, and does not enter
`fitness_formulas.md`. If direct emergence reward is revisited, the target
should be `ei_macro` with a size control, not the `ce_structural` ratio —
§4.4 and §4.3's `fan_pure` row are both cases where the ratio is blind to
what actually changed.

### 5.1 What replaces it as the live question

The finding that survives is not about F6 at all: **F5's champions reduce
effective information at the micro level, substantially, on every substrate
where the measurement has power.** That was not predicted in either
direction and it is the first thing a Stage 6 phenotype contrast should be
pointed at.

---

## 6. Provenance

| what | where |
|---|---|
| §1's pooled sweep | `e3_stage0b2_stage5prime.rs::stage0b2_pooled_separation` |
| §1's clause-1 pin | `…::the_reference_is_an_average_over_disjoint_pairs` |
| §2's perturbation test | `…::adjacent_seeds_are_not_a_perturbation` |
| §3's exit assertion | `…::no_separation_class_rests_on_a_single_draw` |
| §4's emergence triple | `…::stage5_prime_emergence_triple` |

**Repo caveat, unchanged.** `crates/hg-selfmod/` carries uncommitted changes
that do not compile, so `cargo test --workspace` fails; `hg-train`,
`hg-cognition` and `hg-sim-core` are tested explicitly.
