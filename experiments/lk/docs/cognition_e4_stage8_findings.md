# E4 — authored substrates, and Stage 8 replication

Phase E4 of `hypersim_nn_experiments_plan_v3.md`, §8.3, §9.4 and §10.
Produced by `crates/hg-train/tests/e4_authored_substrates.rs`. Substrates:
`rules/relay_pair.toml`, `rules/fan_merge.toml`.

> **Exit MET, on three substrates rather than one.** `champion − untrained`
> clears 2σ on `relay_tri` (2.85σ), `fan_tri` (8.56σ) and `fan_pure`
> (11.03σ). §9.4's failure clause — "the D6 margin is a property of
> `tri_two`'s specific rule pair" — is ruled out: `fan_pure` differs from
> `tri_two` in **both** the rule pair and the initial condition and returns
> the largest margin of the three.
>
> **And the class control is better here than on `tri_two`.** E2 §5 found
> the parametric direction-z unusable on `tri_two` because a dead tail
> skewed the distribution; on all three authored substrates that tail is
> **absent**, the distribution is near-symmetric, and the statistic clears
> 2 (2.44–2.62) as well as returning 0/30. §4.2 argues this is not luck —
> it is the cyclic-dependency criterion paying off in a way it was not
> designed for.
>
> **What does not replicate is the F5/structure agreement.** On `tri_two`
> the champion carried the *most* active persistent structure of any arm.
> On all three authored substrates it carries less than budget-matched
> random while running the smallest graph (§5). No tripwire fired, and the
> pattern is consistent across three independent substrates.

---

## 1. The authoring criterion works, and this is the sharpest evidence for §8.2

D5 ran nine candidate (rule set, IC) pairings through P-1. **All nine
aborted**, and §8 concluded that substrate supply is the binding constraint
and that "more compute cannot fix this. Only authoring can."

Two rule sets authored against §8.2's criterion, six pairings:

| pairing | P-1 (5 draws) | co-live steps | P-3 | P-4 |
|---|---|---|---|---|
| `relay_pure` | **5/5** | 63/64 | PASS 123.82 / 19.26 | PASS |
| `relay_tri` | **5/5** | 63/64 | PASS 97.82 / 16.34 | PASS |
| `relay_wn` | **5/5** | 64/64 | **ABORT** `OBJECTIVE_BLIND` 29.88 vs bar 89.02 | PASS |
| `fan_pure` | **5/5** | 63/64 | PASS 161.69 / 24.41 | PASS |
| `fan_tri` | **5/5** | 63/64 | PASS 116.17 / 12.89 | PASS |
| `fan_wn` | **5/5** | 64/64 | PASS 121.55 / 19.58 | PASS |

Nine empirical aborts became six passes at P-1 and five clean sweeps of
P-1/P-3/P-4. The screen cost nothing and ran first; both files passed it
before any rollout.

**The 63-versus-64 split is a confirmation, not noise.** `PureCrystal` and
`TriLattice` start every node at label 0, so the down-rule has no substrate
until step 1 creates it; `WhiteNoise` starts with mixed labels and is
co-live from step 0. That is precisely the mechanism the files were
authored around, visible in the one place it should be.

`relay_wn` aborting P-3 is recorded rather than dropped: the pairing is
co-live but F5 cannot separate the policy class on it, and 5 of 6 is the
honest yield.

### 1.1 P-1 at five draws, not one

D5 ran P-1 once per pairing. E2 §5 is the standing argument against that —
its 8-draw class control gave a materially wrong answer that 30 draws
corrected — and plan §5.2 forbids a go/no-go resolved by whichever draw ran
first. All six pairings are unanimous at five draws, so nothing here rests
on the discipline; it is cheap and the next substrate may not be unanimous.

---

## 2. The two substrates are deliberately unlike each other

Two substrates that replicate a result the same way are weaker evidence
than two that replicate it differently.

| | `relay_pair` | `fan_merge` |
|---|---|---|
| rule A | +1 node, +1 edge | +2 nodes, +2 edges (branches out) |
| rule B | +1 node, +1 edge | **+1 edge, +0 nodes** (closes a triangle) |
| the two rules | structurally **identical** | growth vs densification |
| what a policy is choosing between | which label to consume | two different *kinds* of change |

`relay_pair` makes the rules interchangeable except for the label they
consume and emit, so rule identity is the only axis a policy can act on,
with no confounding difference in what a firing costs or builds.
`fan_merge` does the opposite: mean degree moves in opposite directions
depending on the allocation.

### 2.1 Plan risk 2

> Designing a rule set for cyclic dependency is designing for a property,
> and the standing prior against imposing structure applies.

Each file's header carries the argument; in summary: **a label cycle
specifies which node is eligible next, not what shape the graph takes.**
Nothing is planted as a right answer, no structure carries reward, and the
composition is fixed by the dependency criterion rather than by what would
be learnable. Each header also states the condition under which its
substrate would have to be declared contaminated — behaviour traceable to an
authored motif. §5's finding is the one place that condition needs checking,
and §5.1 checks it.

---

## 3. Stage 8 — the campaigns

**Identical design to E2 arm A**: 16 pairs × 2 seeds × 30 generations,
σ = 0.10, α = 0.032, N-1, `feature_set` v1, `campaign_seed = 9_600_000`,
the same five frozen hold-out seeds. **Nothing was retuned per substrate.**
E2 §2.4 established that no single σ is right everywhere, so per-substrate
tuning would confound "the result reproduces" with "the result reproduces
once σ is fitted for it."

| | `tri_two` (E2 A) | `relay_tri` | `fan_tri` | `fan_pure` |
|---|---|---|---|---|
| random | 825.58 ± 41.82 | 580.99 ± 9.74 | 576.99 ± 11.96 | 517.04 ± 9.41 |
| untrained | 524.31 ± 6.93 | 557.25 ± 18.12 | 485.27 ± 12.02 | 406.57 ± 27.94 |
| champion | 1148.81 ± 21.12 | 601.11 ± 12.02 | 677.38 ± 29.37 | 655.91 ± 15.56 |
| **− untrained** | +624.50 at **39.74σ** | +43.86 at **2.85σ** | +192.11 at **8.56σ** | +249.34 at **11.03σ** |
| − random | +323.23 at 9.76σ | +20.12 at **1.84σ** | +100.39 at 4.48σ | +138.87 at 10.80σ |
| wall | 70.0 min | 4.6 min | 7.2 min | 6.8 min |

Every precheck passed on every cell and no tripwire fired anywhere.

**Exit MET.** Three substrates other than `tri_two` clear the criterion, and
`fan_pure` — which shares neither the rule pair nor the initial condition
with `tri_two` — is the strongest of them on both statistics.

`relay_tri` is the weak one and is reported as such: it meets the exit at
2.85σ but its `champion − random` is **1.84σ, under the bar**. The two
questions differ (§4) and only one of them is the plan's criterion, but a
reader asking "does it beat the simplest baseline" gets *not at 2σ* on that
substrate.

### 3.1 P-2 reading the right statistic, visibly

`relay_tri` passed P-2 with induced-allocation `TV = 0.0078` — **below the
0.05 bar** — on selection divergence at step 2. That is D6 §4.1's
class-aware clause doing exactly its job: a per-rule statistic is
structurally blind to a scorer, which can rank every candidate differently
while leaving per-rule proportions untouched. Under the pre-D6 P-2 this
campaign would have aborted `OPTIMIZER_FROZEN` on a search that was working.

---

## 4. The class control — and here the authored substrates beat `tri_two`

E2 §5's protocol: 30 untrained directions, same five hold-out seeds, both a
parametric z and the nonparametric rank.

| substrate | 30 draws | champion | parametric | nonparametric | champion over best draw |
|---|---|---|---|---|---|
| `tri_two` | 686.31 ± 372.61 | 1148.81 | **1.24** | 0/30 | +6.3% |
| `relay_tri` | 493.43 ± 41.14 | 601.11 | **2.62** | 0/30 | +9.5% |
| `fan_tri` | 531.35 ± 56.20 | 677.38 | **2.60** | 0/30 | +4.9% |
| `fan_pure` | 494.00 ± 66.24 | 655.91 | **2.44** | 0/30 | +10.0% |

**0/30 on every substrate**, and unlike `tri_two` the parametric statistic
also clears 2 everywhere.

### 4.1 Why the parametric statistic became usable

E2 §5.1 rejected the direction-z on `tri_two` because the draw distribution
was severely left-skewed and the z was therefore measuring the wrong tail.
Compare the deciles:

```
tri_two    12  22 284 455 624 840 924 982 1007 1059 1080
relay_tri 380 426 462 478 491 506 512 523  525  528  549
fan_tri   422 450 467 497 513 530 553 567  579  596  646
fan_pure  352 388 434 464 492 496 505 533  548  580  596
```

`tri_two` has **dead directions** — draws at 12 and 22, policies that
essentially shut the substrate down. The authored substrates have none: the
worst of ninety draws is 351.75 against class means near 500. The
distributions are near-symmetric, so a mean-and-sd summarizes them and the z
means what it says.

### 4.2 That is the design criterion paying off where it was not aimed

The cyclic dependency was authored to satisfy P-1 — sustained co-liveness,
so neither rule can exhaust the other's substrate. **The same property makes
the substrate policy-proof**: because each rule regenerates the other's
LHS, no allocation a policy can choose starves the system, so there is no
direction in genome space that produces a dead run. `tri_two`'s rules carry
no such guarantee, and its dead tail is what that absence looks like.

So a criterion adopted for co-liveness also removed the pathology that made
the class control unreadable. Worth recording because it was not designed
for and is not obvious from §8.1, and because it predicts the same
well-conditioned baseline for any future substrate authored the same way.

**This is a claim about the class baseline, not about the champion**, and it
cuts both ways: a substrate with no dead directions is one where a random
direction is a *harder* baseline to beat, which is the conservative
direction for the margins in §3.

### 4.3 "Reliable but modest" replicates too

E2's summary of `tri_two` was reliable learning of modest value — 0/30, and
+6.3% over the best random direction. The authored substrates give
**+4.9% to +10.0%**. Both halves transfer, including the limitation.

---

## 5. What does not replicate: F5 and structure stop agreeing

D6 §7A.1 and E2 §4.1 both leaned on the champion carrying the *most* active
persistent structure of any arm, at an essentially identical node count —
F5 and the held-out structural metrics agreeing, which is what distinguishes
a learning result from a Goodhart one.

| substrate | `active_persist` random / untrained / champion | `n_nodes` random / untrained / champion |
|---|---|---|
| `tri_two` | 21.65 / 20.11 / **26.05** | 860.2 / 854.4 / 858.8 |
| `relay_tri` | **19.55** / 18.79 / 18.34 | 882.4 / 866.4 / 868.5 |
| `fan_tri` | **17.57** / 15.25 / 16.87 | 1135.1 / 1049.3 / **986.3** |
| `fan_pure` | **17.06** / 13.59 / 14.77 | 1127.7 / 1064.4 / **1040.5** |

On all three authored substrates the champion carries **less** persistent
structure than budget-matched random, while running the **smallest** graph
of any arm. On `relay_tri` it is below the untrained arm too — the champion
scores highest on F5 while carrying the least structure of any arm measured.

`flag_degenerate_reuse` did not fire on any run: no single constructor owns
≥90% of cycles on a collapsed graph, so this is not the failure the tripwire
defines. But it is the same direction, it is consistent across three
independent substrates and two unlike rule sets, and it points at F5 rather
than at any one substrate: **on these substrates a smaller, more repetitive
graph scores better, and F5 is partly rewarding that.**

### 5.1 Is this the contamination §2.1 warned about?

No, and the distinction matters. Risk 2's contamination test is whether
interesting behaviour is traceable to an **authored motif** — the triangle
`merge` closes, say. This is not that: the effect appears on `relay_pair`,
which contains no closed motif at all and whose rules only hang a pendant
node off a matched edge. It appears on two rule sets with opposite
structural effects and on two different initial conditions.

An effect common to every authored substrate and absent on `tri_two` is a
statement about what the four substrates do **not** share, and the candidate
is F5's own preference — which `tri_two` happens to satisfy differently.
The substrates are not contaminated; the objective is doing something on
them that it did not do on `tri_two`.

**This is a finding for E3/Stage 5′, not a defect to fix here.** Plan §7.1
already argues that F5 selects for the property that destroys emergence,
and this is the first direct evidence for that on more than one substrate.

---

## 6. Verdict

**Plan §10's E4 exit: MET.** Three substrates other than `tri_two` carry the
margin; the single-substrate caveat does not become the headline.

What the programme can now say, with every statistic reported:

> A trained `MlpScorer` beats budget-matched random and beats **30/30**
> untrained directions of its own class on **four** substrates spanning
> three rule pairs and two initial conditions, exceeding the best of thirty
> random directions by **4.9%–10.0%**.

What it must say alongside:

1. **`relay_tri`'s `champion − random` is 1.84σ**, under the bar, though its
   class control is the strongest parametric z of the four.
2. **The F5/structure agreement does not generalize** (§5). It was load-
   bearing in D6's and E2's readings and it is absent — in fact reversed —
   on every authored substrate.
3. **The margins are much smaller than `tri_two`'s** in absolute terms
   (+43.86 … +249.34 against +624.50), though the *relative* margin over the
   best random direction is comparable.
4. **One pairing of six aborts P-3** (`relay_wn`), so the authoring criterion
   yields co-live substrates reliably and F5-discriminable ones slightly
   less so.

**Carried forward.** §5 is the concrete case plan §7.1 predicts, and it
lands in E3/Stage 5′'s lap: the emergence prediction now has four substrates
to be adjudicated on rather than one, and the F5-rewards-shrinkage
observation is a pre-registered expectation for it rather than a
post-hoc reading.

---

## 7. Provenance

| what | where |
|---|---|
| §1's screen pass | `e4_authored_substrates.rs::authored_substrates_pass_the_static_screen` |
| §1's P-1 at five draws | `…::p1_across_pairings_at_five_draws` |
| §1's P-3/P-4 | `…::p3_p4_on_surviving_pairings` |
| §3's campaigns, §5's arm tables | `…::stage8_campaigns_on_authored_substrates` |
| §4's class control | `…::class_control_on_authored_substrates` |

The class-control test hard-codes each champion from §3 rather than
recomputing it, for the reason E2's version does: it adjudicates published
numbers and must fail loudly if they are ever re-derived differently.

**Repo caveat, unchanged.** `crates/hg-selfmod/` carries uncommitted changes
that do not compile, so `cargo test --workspace` fails; `hg-train`,
`hg-cognition` and `hg-sim-core` are tested explicitly.
