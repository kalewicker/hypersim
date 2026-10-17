# V6 / K1 — turnover-vs-persistence criterion for arm B (PRE-COMMITTED)

Written and committed **before** any arm-B demographic or TWTF number exists, per the V6 rule
"K1 can move the verdict; run it first and do not pre-write the conclusion."

## What K1 asks

The H2 ABC split treated arm B (non-replicating, sustained by `relocate`) as "arm A minus
replication". That is only true if B is *sustained turnover* — individuals dying and being replaced
without heredity. If instead B is *persistence* — the same few individuals staying alive by grazing,
with little birth/death — then A and B differ in replication **and** in turnover-vs-persistence, and
the two-variable split has a hidden third variable. The positive property would then be sustained
**presence**, not sustained **turnover**.

Replication-per-se is excluded either way (C is non-replicating and undetectable; B is
non-replicating and detectable). K1 decides only what the *positive* property should be called.

## K1.1 — quantities reported (realized, never configured)

Carrier individuals are tracked by `NodeId` over the full 6000-step scoring window of the ABC run.
A **birth** is a carrier NodeId present at step t and absent at t−1; a **death** is one present at
t−1 and absent at t. Nothing is inferred from rule firings or rates.

- `births`, `deaths` (raw counts over the scoring window)
- `balance` = deaths / births (1.0 = demographically balanced)
- `mean_completed_lifetime` = mean over individuals that both were born and died in-window;
  individuals alive at the end are **censored** and reported separately, never averaged in
- `censored_frac` = fraction of ever-live individuals still alive at step 6000
- `per_capita_death_rate` = deaths / (mean standing population × steps) — the fraction of the
  standing population replaced per step, the node-count-independent turnover rate
- founders (the planted/bootstrapped initial carriers) are counted as births at their step of
  appearance, exactly as any other individual; no special-casing

## K1.2 — TWTF

TWTF is computed with the H0/J2 definition verbatim (`R_PERSIST = 3`, fraction of transitions whose
**source** class was live in fewer than R steps), on the **WL depth-2 partition** — the instrument
the H2 verdict rests on. Reported for A and B and for the **static floor** (the bare substrate with
no carriers), at both W=120 (H0-comparable) and W=6000 (the actual scoring window). Also reported
restricted to carrier-classed transitions, since the whole-graph TWTF is dominated by the 36
substrate nodes in every arm.

## K1.3 — PRE-COMMITTED VERDICT RULE

Let `r_A`, `r_B` be the per-capita death rates and `L_A`, `L_B` the mean completed lifetimes.

- **TURNOVER** — `r_B >= 0.5 * r_A` **and** `L_B <= 2 * L_A` **and** `censored_frac_B <= 0.5`
  ⟹ B has A-comparable turnover. "Sustained **turnover**" confirmed; A and B differ only in
  replication as the ABC design assumed. Proceed to K2/K3 on the confirmed verdict.

- **PERSISTENCE** — `r_B <= 0.1 * r_A` **or** `L_B >= 10 * L_A` **or** `censored_frac_B >= 0.5`
  ⟹ B is persistence. The verdict **sharpens to "sustained *presence* is load-bearing"**.
  Replication-per-se remains excluded, but the positive property is presence, not turnover.
  This is the finding; arm B is **not** to be redefined and `relocate` is **not** to be adjusted.

- **INTERMEDIATE** — anything else ⟹ report the demographic numbers and state the verdict as
  bounded by them, not asserted ("between persistence and turnover, closer to X by these numbers").

TWTF is corroborating evidence, not a tiebreaker: if it contradicts the demographic rule, both are
reported and the discrepancy is the finding.

## Explicitly not permitted

- Changing these thresholds after seeing the numbers.
- Treating a PERSISTENCE outcome as a bug in arm B.
- Running K2/K3 before this is read.
