# V9 / Phase 1 — validating the metabolic instrument: pre-registration

**Committed before any metabolic score, null, or multi-seed number exists.** Phase 2 does not begin
until N1.1–N1.4 clear.

## The instrument, defined precisely

M3 established a *discriminating* quantity (inert ≈ 0, D ≫ 0). Discriminability is satisfied by many
statistics, so the quantity must now become a **statistic with a null**, held to exactly the standard
the structural results were held to.

### The metabolic micro state (dynamics, not level)

The energy-band partition is excluded everywhere (~98% definitional: it measures a *level*). The
metabolic micro state is therefore keyed on **change**:

```
delta_t(node)  = ALife energy at t  −  ALife energy at t−1        (property key 1)
bucket(d)      = quantised signed magnitude of d
micro_state    = (label, bucket(delta_t), bucket(delta_{t−1}), …)   up to depth R
```

`bucket` maps a delta to a discrete code by sign and magnitude class, so the state space is bounded
and recurrent (the H0.1 closure requirement) while remaining purely dynamical. An **inert carrier has
`delta ≡ 0` forever**, so it occupies exactly one micro class and cannot be resolved — which is the
correct answer, not a defect.

### Resolution depth `R` — chosen by RULE, not by outcome

The F1 lesson (depth 1 was degenerate; depth 2 was right; re-selecting depth after seeing results is
the forbidden move) applies directly. So `R` is fixed by a rule evaluated on **arm A only** (the F8
baseline, whose structural answer is already known and is not the thing under test):

> `R` = the **smallest** depth in `{1, 2, 3}` for which arm A satisfies `k_carrier >= 2`,
> `k_carrier < M`, null support `C(M, k) > 100`, **and** the realised-spread guard.

The chosen `R` is then **fixed for every arm and every seed** in Phases 1 and 2. If no depth in
`{1,2,3}` qualifies, the instrument is reported as unresolvable and Phase 2 does not proceed.

### The statistic

Two quantities are reported for every arm; both use the same TPM and the same permutation null.

1. **CE on the metabolic channel** — `EI_macro / EI_micro` over the vacuum-augmented TPM of metabolic
   micro states, coarse-grained by the carrier/substrate partition. This is the **direct analogue of
   the structural test**, which is what makes the Phase-2 two-channel table apples-to-apples.
2. **Closure defect** `H(micro transition | macro transition)`, and its normalised form
   `H(micro|macro) / H(micro)` — the retention statistic that produced the shadow result.

**Direction of each is pre-committed:** for CE, a *detected* boundary means real CE exceeds the null
(same direction as the structural test). For the closure defect, a partition that *explains* the
metabolic dynamics gives a **lower** defect than a random same-sized partition, so its permutation
test is **one-sided in the opposite direction**, and its p-value counts `null <= real`.

## N1.1 — permutation null with the realised-spread guard applied FROM BIRTH

The guard is applied at construction, never retrofitted — a brand-new statistic is exactly where a
degenerate null passes unnoticed (arm D's structural null had support 3.78 × 10² and sd = 0).

Admissible iff `null_sd > 0` **and** `null_max − null_min > 0` **and** `>= 10 distinct` realised null
values, in addition to `k_carrier >= 2`, `k_carrier < M`, `C(M,k) > 100`.

Reported per arm: real score, null mean, null p95, null max, **realised spread (sd, min–max, distinct
count)**, count `null >= real` (or `<=` for the closure defect), p, and support.

## N1.2 — multi-seed, at the F8/K2 standard

- Seeds **1, 2, 3, …** ascending until **30 included** or **120 attempts**; disjoint from 12345, which
  serves as the fidelity anchor.
- **Inclusion:** `transitions >= 1000`, `k_carrier >= 2`, arm established per its own definition, and
  the realised-spread guard passes. A degenerate-null seed is **unmeasurable**, excluded from the
  denominator, and its count reported separately.
- **Decision:** `f` = fraction of included seeds with `p < 0.05`. `f >= 0.90` ⟹ regime property;
  `0.50 <= f < 0.90` ⟹ majority, excepted seeds reported individually; `f < 0.50` ⟹ seed-specific.
  Secondary: exact two-sided binomial sign test; reversals flagged individually. `n < 10` ⟹ vacuous.

## N1.3 — two-process reproducibility and a calibrated zero-point

- **Two separate OS processes** must produce **bit-identical** scores. Per the §6 lesson (statistics
  over indices can differ while a coefficient agrees), the check uses an **index-dependent** statistic
  — the ordered vector of micro-class hashes and the per-class TPM row sums — not just the scalar CE.
- **Inert zero-point as a measured band:** the inert arm is run across the same seed set and its score
  distribution (min, max, mean, sd, and how often it is degenerate) is reported, so "≈ 0" is a band
  with a spread rather than one eyeballed number.

## N1.4 — stationarity under bounded churn, with a second witness

Measured **post-burn-in per N0** and with **two witnesses of different failure modes** (rule 4), not
one:

1. **TWTF** on the metabolic micro states (`R_PERSIST = 3`, H0.1's definition).
2. **Consecutive-block class-set Jaccard** on the metabolic micro states (the M1 witness that caught
   TWTF's blind spot).

Both are reported against the static baseline and against the growing proxies, so the
growth-versus-churn distinction is visible in the new channel too.

**Gate:** the instrument must **discriminate AND be stationary**. Discriminating-but-unstable ⟹ it is
reported as **static-substrate-only**, and the churn-dependent parts of Phase 2 (N2.4) are blocked.

## N1.5 — convergent witnesses (reported, not gating)

The two M3 runner-ups are carried alongside:

- **energy-transition entropy** — needed window-normalisation to place the dead token below arm D;
- **per-carrier activity rate** — needed the same repair.

Reported: what repair each needed and **whether that repair is principled or ad hoc**, and whether all
three instruments agree on the Phase-2 arm ordering. Agreement across three statistics with different
failure modes is stronger than one perfect fit, and it hedges the home-team risk in the closure defect
being the project's own construct.

## Phase 1 gate

N1.1–N1.4 clear, with N1.5 reported. **Only then does Phase 2 begin.**

The single sentence Phase 1 must answer: **whether the closure-defect estimator on the energy channel
is a validated statistic — null-tested, multi-seed, reproducible, and stationary under churn — or only
a discriminating one.**

## Forbidden

- Re-selecting `R` after seeing any arm's result (it is fixed by the arm-A rule above).
- Retrofitting the spread guard.
- Reading a Phase-2 property verdict from anything in Phase 1.
- Reporting the closure defect alone without the convergent witnesses.
