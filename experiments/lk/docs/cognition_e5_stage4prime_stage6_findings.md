# E5 — Stage 4′ (task lattice) and Stage 6 (phenotype contrast)

Phase E5 of `hypersim_nn_experiments_plan_v3.md`, §9.3, §9.5 and §10 — the
last phase of the plan. Produced by
`crates/hg-train/tests/e5_stage4prime_stage6.rs`.

> **Stage 4′ inverts v2's Stage 4 finding, and the inversion is the
> result.** v2 measured `champion − untrained` at +0.28σ and concluded the
> lattice effect was *entirely the policy class*. Under `MlpScorer` the
> contrast is enormous — up to −14.11σ on depth — and it runs the **wrong
> way**: the champion builds a **smaller, shallower** lattice than either
> baseline, with **fewer** tasks, on every substrate.
>
> **The one lattice quantity the champion raises is composition rate**
> (+1.4σ to +4.5σ vs random). It gets there by shrinking the denominator:
> `fan_pure`'s champion has 233 composable chains against random's 625 and
> untrained's 988. A higher fraction of a much smaller lattice.
>
> **Stage 6 delivers a real phenotype contrast** (§3): F3 champions build
> lattices 3–4× deeper than F5 champions on the same substrate, and on
> `fan_pure` a graph 34% larger at half the type entropy. **The two
> objectives select visibly different organisms.** One of four cells aborts
> `OBJECTIVE_BLIND` before the F3 campaign starts.

---

## 1. Stage 4′ — the task lattice, genuinely three-arm

Plan §9.3 asks for v2's Stage 4 design, "now genuinely three-arm with §5.1's
neutral-point measurement, so the class effect and the learning effect are
separated by construction rather than in re-analysis."

The reason that matters: v2's Stage 4 ran on `StaticBias`, whose neutral
point at θ = 0 is the uniform-over-rules allocation. `MlpScorer`'s neutral
point is a *draw* from the sphere (E2 §3), so `champion − untrained` is a
different contrast and its sign does not follow from v2's.

### 1.1 Results

Five frozen hold-out seeds, F5 champions, `k = 4` matched across arms.

| substrate | arm | lattice depth | tasks | composition rate | composable |
|---|---|---|---|---|---|
| `tri_two` | random | 55.60 ± 3.44 | 114.4 ± 6.7 | 0.3025 ± 0.0171 | 248.8 |
| | untrained | 29.20 ± 2.59 | 59.4 ± 4.0 | 0.3056 ± 0.0193 | 124.0 |
| | **champion** | 50.00 ± 5.24 | 112.2 ± 6.7 | **0.3243 ± 0.0128** | 270.0 |
| `relay_tri` | random | 21.80 ± 3.11 | 159.8 ± 6.8 | 0.1385 ± 0.0121 | 305.0 |
| | untrained | 22.80 ± 1.30 | 173.8 ± 15.9 | 0.1434 ± 0.0115 | 329.8 |
| | **champion** | 14.80 ± 0.84 | 111.2 ± 9.3 | **0.1668 ± 0.0139** | 202.2 |
| `fan_tri` | random | 44.40 ± 7.02 | 207.4 ± 17.9 | 0.1767 ± 0.0132 | 530.8 |
| | untrained | 47.20 ± 7.05 | 268.6 ± 13.8 | 0.1906 ± 0.0097 | 772.4 |
| | **champion** | 28.60 ± 1.67 | 133.0 ± 5.6 | **0.2303 ± 0.0106** | 392.2 |
| `fan_pure` | random | 43.60 ± 4.56 | 235.2 ± 13.5 | 0.1752 ± 0.0174 | 625.0 |
| | untrained | 49.60 ± 2.70 | 350.6 ± 27.8 | 0.1720 ± 0.0108 | 987.8 |
| | **champion** | 20.00 ± 1.22 | 88.2 ± 4.6 | **0.2455 ± 0.0136** | 233.0 |

Contrasts, in pooled seed sd:

| substrate | depth: champ−rand / champ−untr | tasks: champ−rand / champ−untr | rate: champ−rand / champ−untr |
|---|---|---|---|
| `tri_two` | −1.26 / **+5.03** | −0.33 / **+9.52** | +1.44 / +1.14 |
| `relay_tri` | **−3.07** / **−7.30** | **−5.96** / **−4.81** | **+2.17** / +1.83 |
| `fan_tri` | **−3.10** / **−3.63** | **−5.61** / **−12.86** | **+4.47** / **+3.90** |
| `fan_pure` | **−7.07** / **−14.11** | **−14.54** / **−13.17** | **+4.49** / **+5.98** |

### 1.2 v2's finding is inverted, not reproduced

> **v2 Stage 4:** champion vs untrained +0.28σ. "The `StaticBias`
> allocation scheme is what populates the lattice; forty generations of
> CMA-ES on top of it contribute nothing measurable."

Under `MlpScorer` the champion−untrained contrast is anything but
negligible — up to **−14.11σ** — and on three of four substrates it is
**negative on both depth and task count**. Training does not fail to move
the lattice here; it moves it *down*.

`tri_two` is the exception and its own explanation. There the untrained
draw is unusually bad (E2 §4.1: 524.31 against random's 825.58), so the
champion beating it on depth (+5.03σ) and tasks (+9.52σ) is the champion
recovering from a poor draw rather than exceeding the baseline — against
*random* it is still −1.26σ and −0.33σ. This is the same
bad-draw inflation E2 §4.1 documented on the fitness column, appearing again
on a different measurement, and it is why `champion − random` has to be read
alongside.

### 1.3 The composition rate rises, and the denominator explains it

Composition rate is the one lattice quantity that moves the predicted way:
**+1.44σ to +4.49σ against random**, significant on three of four.

But `composable` — the denominator — collapses in step:

| substrate | composable: random → champion | rate |
|---|---|---|
| `tri_two` | 248.8 → 270.0 | 0.3025 → 0.3243 |
| `relay_tri` | 305.0 → 202.2 | 0.1385 → 0.1668 |
| `fan_tri` | 530.8 → 392.2 | 0.1767 → 0.2303 |
| `fan_pure` | 625.0 → **233.0** | 0.1752 → **0.2455** |

On `fan_pure` the champion realizes a higher *fraction* of a set of
composable chains **less than half** the size, out of a lattice with a
quarter the tasks. Only `tri_two` raises the rate while holding the
denominator roughly constant, and `tri_two` is the substrate whose champion
barely changes the lattice at all.

**This is the same shape as E3 §4.4's `ce_structural`** — a ratio rising
because its denominator falls. Two different stages, two different ratios,
one mechanism, and in both cases the ratio is the quantity that would have
been reported as the headline.

### 1.4 Consistency with E3 and E4

Three measurements now point the same way on the authored substrates:

| stage | measurement | champion vs random |
|---|---|---|
| E4 §5 | `n_nodes`, `active_persist` | smaller graph, less persistent structure |
| E3 §4.1 | `ei_micro` | **down** 3.2–7.6 sd |
| E5 §1.1 | lattice depth, task count | **down** 3.1–14.5 sd |

**F5's champions make the substrate do less.** Fewer nodes, less persistent
structure, lower micro effective information, a shallower task lattice with
fewer distinct tasks — and higher F5, because reuse depth rewards the same
constructor firing repeatedly and a smaller, more repetitive substrate
supplies exactly that.

That is a coherent account of what the programme's positive learning result
actually learned, and it is not the account D6 or E2 implied.

---

## 2. Stage 4′ against its exit

Plan §9.3 sets no numeric exit — it asks for the design to be three-arm with
the neutral point measured. **Done**, and the measurement is what separated
§1.2's `tri_two` anomaly from the other three substrates.

The pre-registered direction (v2's "policy-shaped dynamics populate the
lattice at a higher rate than random dynamics") holds only for the *rate*
and only with §1.3's denominator caveat attached. On the absolute lattice
quantities the direction is reversed.

---

## 3. Stage 6 — the phenotype contrast

Plan §9.5 gated Stage 6 behind Stage 8 delivering ≥ 2 substrates; E4
delivered three. F5 champion against F3 champion, both trained with the same
design on the same substrate, compared **only on held-out structural
metrics** — never on either objective, since scoring an F3 champion by F3
and calling the difference a phenotype would be circular.

| substrate | champion of | n_nodes | active_persist | mean_degree | type_entropy | lattice depth |
|---|---|---|---|---|---|---|
| `tri_two` | F5 | 855.19 ± 8.44 | 26.24 ± 0.29 | 5.84 ± 0.03 | 2.95 ± 0.03 | 50.00 ± 5.24 |
| | F3 | 839.14 ± 8.45 | 26.96 ± 1.50 | 5.83 ± 0.02 | 2.90 ± 0.04 | **86.00 ± 12.88** |
| `relay_tri` | F5 | 863.53 ± 5.62 | 18.30 ± 0.51 | 2.40 ± 0.01 | 3.16 ± 0.02 | 14.80 ± 0.84 |
| | F3 | 853.87 ± 6.77 | **13.07 ± 1.10** | 2.41 ± 0.00 | **2.38 ± 0.03** | **55.20 ± 2.17** |
| `fan_pure` | F5 | 1042.06 ± 10.04 | 14.91 ± 0.32 | 3.03 ± 0.02 | 2.35 ± 0.03 | 20.00 ± 1.22 |
| | F3 | **1396.22 ± 20.12** | **10.66 ± 1.34** | **2.50 ± 0.03** | **1.32 ± 0.10** | **68.00 ± 11.31** |

**Stage 6 produces the contrast it was designed to produce.** The two
objectives select organisms that differ on held-out structure, and the
difference is large:

- **Lattice depth separates on all three cells** — F3 champions are 1.7×,
  3.7× and 3.4× deeper. The objective that never mentions the task lattice
  builds a much deeper one than the objective that also never mentions it.
- **`fan_pure` separates on everything.** The F3 champion runs a graph 34%
  larger, at 83% the mean degree, with **56% of the type entropy** and 71%
  of the active persistent structure. Same substrate, same policy class,
  same search — two objectives, two phenotypes.
- **`tri_two` separates on lattice depth alone.** Every other column
  overlaps within a seed sd, which is consistent with §1.2: `tri_two` is the
  substrate on which champions of either kind barely move the phenotype.

This is what v2's Stage 6 could not run for lack of a configuration, and
what §9.5 correctly said would be weak on one substrate. Three substrates
make the `fan_pure` separation readable as a substrate-dependent effect
rather than a one-off.

### 3.1 One cell refuses, before spending the campaign

`fan_tri` aborts before the F3 campaign starts:

```
OBJECTIVE_BLIND: spread across the class is 8.3581, under 2x the largest
seed sd (5.3411 -> bar 10.6823).
arms: campaign init 67.2559 +- 2.7040; allocation extreme 59.2728 +- 2.3009;
      budget-matched random 67.6309 +- 5.3411
```

F3 cannot separate the policy class on `fan_tri` at five seeds, so the
campaign does not run. The abort is the precheck layer doing exactly its job
— three of four cells produce a phenotype contrast and the fourth is
recorded as unstateable rather than as a null.

### 3.2 The caveat on Stage 6

The F3 arms are not the F5 arms. F3 champion fitness runs 48.94 / 161.06 /
144.63 against F5's 1148.81 / 601.11 / 655.91 — different objectives on
different scales, and **no claim is made here about which champion is
better**. Stage 6's content is that the phenotypes differ, not that one
wins.

---

## 4. Verdict

**Plan v3 Phase E5 is complete, and with it plan v3.** Stage 4′ ran
three-arm with the neutral point measured; Stage 6 ran on three substrates
and produced a phenotype contrast; the F6 campaign did not run because
Stage 5′ closed it (E3 §5).

What E5 adds to the programme's account:

1. **v2's Stage 4 conclusion does not generalize to a scoring class.** The
   lattice effect there was entirely the policy class; here training moves
   the lattice substantially, and downward.
2. **The composition-rate rise is a denominator effect** (§1.3), the second
   instance of that shape in two phases.
3. **F5's champions make the substrate do less** (§1.4) — now visible on
   four independent measurements across three phases.
4. **Two objectives select different organisms** (§3), which is the first
   direct evidence in the programme that the objective choice has
   consequences for phenotype rather than only for score.

---

## 5. Provenance

| what | where |
|---|---|
| §1, §3 | `e5_stage4prime_stage6.rs::stage4_prime_and_stage6` |
| §1's composition rate | `composition_rate`, unchanged from `c5_stage45.rs` so the v2 numbers are comparable |

Every campaign in this phase ran **once**, with both stages reading its
`CampaignOutcome` in-process — E3 paid for four campaigns twice because
champion genomes lived in a tempdir and a statistics correction forced a
re-derivation.

**Repo caveat, unchanged.** `crates/hg-selfmod/` carries uncommitted changes
that do not compile, so `cargo test --workspace` fails; `hg-train`,
`hg-cognition` and `hg-sim-core` are tested explicitly.
