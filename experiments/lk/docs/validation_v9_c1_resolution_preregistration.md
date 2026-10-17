# V-track closeout / C1 — the resolution diagnostic: channel or instrument?

**Committed before any C1 number exists**, and before any score at any binning granularity. This
file is the C1.3 pre-registration the closeout plan requires, and it also fixes the C1.2 reading and
the C2 selection rule in advance so that neither can be chosen after seeing a table.

The question: **the metabolic channel excludes nothing in the ladder — every live arm detects, and
the only line it draws is active-vs-inert. Is that coarseness a property of the CHANNEL, or of the
INSTRUMENT?**

The prior is that it is the channel (selection is settled degenerate-only, and the metabolic CE test
finds all live carriers alike — two independent nulls). But F0 is the standing counter-example: an
under-resolved partition masquerading as a null reversed a headline once already (p = 0.495 → 0/200
on WL depth alone). So the diagnostic runs, and the rule is committed first.

---

## C1.1 — what is measured (no scoring, no nulls)

Exactly the shape of F0.2, on the metabolic partition instead of the WL one:

For each arm (A, B, B′, C, D, inert) and each depth `R ∈ {1,2,3}` and each candidate binning
granularity `G`: the total metabolic micro-class count `M`, split into **(substrate classes,
carrier classes)** — the group sizes. `k_carrier` is the diagnostic quantity, because F0's failure
was a carrier group of size **1** against nine substrate classes.

This file computes **no CE, no closure defect, and no null.**

## C1.2 — the pre-committed reading

The frozen instrument is `R = 2` at granularity `G_cur` (the N-track's 13-code signed magnitude
bucket). Read `k_carrier` for the **live** arms at that setting:

- **`k_carrier` ≈ 2 for the live arms** ⟹ the (9,1)-shaped degeneracy is recurring one level up.
  The metabolic channel's failure to separate the ladder is an **instrument** artifact.
  **Reopen per C2.**
- **`k_carrier` is large and the arms still score alike** ⟹ the coarseness is **substantive**: the
  channel genuinely does not differentiate live carriers. This confirms the two-null convergence and
  **closes the track**.
- **Intermediate** ⟹ report `k_carrier` against the realised per-class spread, and bound the claim
  by both. No slogan.

## C1.3 — the granularity selection rule, committed before any score

### The knob

The metabolic micro state is `(label, bucket(δ_t), …, bucket(δ_{t−R+1}))`. The knob is `bucket`.
Candidates, ordered **coarse → fine**, all bounded by construction (the H0.1 closure requirement):

| id | `bucket` | codes |
|---|---|---|
| `G_sign` | sign of δ only | 3 |
| `G_cur` | signed magnitude, 6 magnitude classes (**the frozen N-track instrument**) | 13 |
| `G_fine` | signed magnitude, 12 magnitude classes | 25 |
| `G_q05` | δ quantised to steps of 0.05, clamped to ±24 steps | 49 |
| `G_q01` | δ quantised to steps of 0.01, clamped to ±60 steps | 121 |

Note for the record: the closeout plan describes the status quo as "sign-of-change is coarse; binned
deltas less so". That describes **N4.3's retention statistic**, which does use sign-of-change. The
Phase-1/Phase-2 instrument already uses binned deltas (`G_cur`, 13 codes). Both are on the ladder so
the comparison is explicit rather than assumed.

### The rule (binding)

> **C1.3-R.** At the frozen depth `R = 2`, walk the candidates coarse → fine and select the
> **minimum** granularity `G` at which **every live arm** (A, B, B′, C, D) satisfies
>
> 1. `k_carrier(arm) ≥ K_MIN`, and
> 2. N1.1 admissibility: `k_carrier < M`, null support `C(M, k_carrier) > 100`, and the
>    realised-spread guard,
>
> where **`K_MIN` is derived, not chosen**: it is the number of **carrier WL classes arm A exhibits
> at the frozen structural depth 2** — the resolution of the structural instrument the metabolic one
> is tabled against in Phase 2.

`K_MIN` is baseline-relative for the same reason N0 v2 is: N0 v1 failed precisely because it used
absolute thresholds, and this program's precedent (H0.1) is *"whatever the trusted baseline exhibits
defines the bar."* An absolute bar here would be a number I could shop toward a verdict.

The bar's meaning: **the metabolic partition must resolve carriers at least as finely as the
structural partition it is being compared against**, or the two-channel table is not comparing like
with like.

If **no** candidate reaches `K_MIN`, the instrument is reported unresolvable at every available
granularity and C2.3's branch applies — the channel is uniform, by the longer route.

If `G_cur` **already** satisfies C1.3-R, the frozen instrument was not under-resolved, C1's
confirming branch holds, and **C2 does not run.**

## C2 — the retention ranking, also committed here (runs only if C1 reopens)

Retention is a **selection criterion, not a result** (plan rule 4). Definition, fixed now:

```
raw    = the finest metabolic description available: (label, q01(δ_t), q01(δ_{t−1}))
ret(G) = 1 − H(raw | G) / H(raw)          over CARRIER node-steps, post-burn-in
```

- **C2.1** — rank every admissible candidate by `ret(G)`. Admissible = passes C1.3-R clauses 1–2
  **and** is bounded/recurrent **and** passes the N1.4 stationarity gate under bounded churn. The
  boundedness and stationarity clauses are what stop the ranking from running away to the finest
  candidate for free; without them "rank by retention" trivially selects the finest partition.
- **C2.2** — score the ladder on the **best-retaining admissible** candidate only, at the full
  standard: null with realised spread, multi-seed, two-process reproducibility, inert as the
  calibrated zero, churn stationarity.
- **C2.3** — report the retention achieved. If the best admissible candidate still retains ≈ 1 bit,
  the channel is uniform after all and the track closes on C1's confirming branch.

Per N4.3's rule 2, retention is **class-level information retention, never causal independence.**

## Forbidden

- Selecting `G` by its effect on arm separation, on any p-value, or on any CE — the same forbidden
  move as selecting WL depth by its effect on CE.
- Re-deriving `K_MIN` after seeing the `k_carrier` table.
- Re-running C1 at a different `R` after seeing the result. `R = 2` is frozen from Phase 1.
- Reading a property verdict out of C1. C1 diagnoses the instrument only.

## The single sentence C1 must answer

**Whether the metabolic channel's coarseness belongs to the channel or to the instrument — and
therefore whether the V-track is finished or has one more thing to measure.**
