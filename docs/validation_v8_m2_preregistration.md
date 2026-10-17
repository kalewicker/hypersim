# V8 / M2 — arm-C truncated-window reading, and the null-spread guard (PRE-COMMITTED)

**Committed before any truncated-window CE for arm C exists** and before the retroactive audit is
run. Per V8 rule 2, this measurement can weaken the headline verdict, and the reading below is fixed
now so it cannot be re-chosen afterwards.

## Why M2 presses

With arm D established as **definitional** — a constant graph makes every structural class function
constant, so its non-detection is near-tautological — **arm C carries the entire non-definitional
weight of the sustained-structural-activity verdict.** And C may not be able to.

C's carriers all die at step 120, so for 98% of the 6000-step window the carrier group is **empty**,
which is the known degenerate case (`K` = 1, CE = 0 by construction). So C's non-detection has two
candidate causes:

- **insufficient activity share** — substantive: activity must occupy enough of the TPM; or
- **partition degeneracy from an empty carrier group** — artifact.

The pressing evidence for the second: B′ has **6** structural events and detects (p = 0.010); C has
**12** (6 births + 6 deaths) and does not. If raw activity were the variable, C should beat B′. What
actually differs is that B′'s carrier group stays populated and C's empties — which looks like
partition well-formedness, not biology.

## M2.1 — what is measured

Arm C is re-scored over a window truncated to its **live phase**, where its carrier group is
populated throughout. Reported beside the full-window numbers: CE, **realised null spread**
(sd, min, max, distinct count — not merely support), z, p, `k_carrier`, `M`, transitions.

Truncation lengths **W = 100, 110, 120** (C's last-alive step is 120), so the reading does not hinge
on one arbitrary cut.

**Mandatory positive control.** A, B, B′ and D are scored on the *same* truncated windows. If arm A
— the known-detectable F8 baseline — fails to detect at W = 120, then no arm can detect at that
window, C's truncated non-detection would be a power artifact, and **no conclusion may be drawn from
it**. This control is required before any C reading is interpreted.

## M2.2 — PRE-COMMITTED READING

- **C still NOT detected on the truncated window** (with A detecting there) ⟹ C's non-detection is
  **substantive**. Activity share is the variable and the sustained-structural-activity verdict's
  empirical leg **holds**.
- **C IS detected on the truncated window** ⟹ the full-window non-detection was the **empty-group
  artifact**. "Duration is necessary" must be restated as **"a populated carrier group is
  necessary"** — a claim about partition well-formedness rather than about biology — and the verdict
  **weakens accordingly**. §5.9 and §9 get the weaker framing; the stronger one is not preserved.
- **A does not detect on the truncated window** ⟹ the truncated test has no power; report as
  inconclusive and draw nothing from C either way.
- **Either way**, report C's realised null spread. If C's truncated null is *also* degenerate, C is
  partly unmeasurable in the same way D is, and the empirical leg is thinner still — that is
  reported as its own finding.

## M2.3 — the null-spread guard (replaces the support bar)

**The defect.** `C(M,k) > 100` passed at 3.78 × 10² on arm D while the realised null had sd = 0.
Support counts the partitions a permutation *could* produce; it says nothing about whether they
*differ*. A permutation null that returns one value 200 times cannot detect anything, at any support.

**The replacement, committed here.** An arm's p-value is admissible iff, over the same
`N_PERM = 200` permutations:

1. `null_sd > 0`, **and**
2. `null_max − null_min > 0`, **and**
3. `distinct null CE values >= 10`.

Clauses 1–2 are the hard degeneracy test. Clause 3 is the near-degeneracy bar: with 200 draws, a
`p < 0.05` verdict needs fewer than 10 permutations to reach the real value, so a null concentrated
on fewer than ~10 distinct values cannot resolve that threshold reliably.

`C(M,k)` continues to be **reported** — it remains a necessary condition — but it is no longer
sufficient on its own.

**Known measurement caveat, stated in advance:** the distinct-value count is mildly sensitive to
floating-point summation order (HashMap row iteration), so clause 3 is evaluated with a tolerance
and clauses 1–2 (which are exactly 0 or not) carry the decision. This is why the bar is not keyed on
the distinct count alone.

**Retroactive application.** The guard is applied to **every** arm scored in the G-, K- and L-tracks,
and any arm whose published p-value rests on a degenerate or near-degenerate null is named.

**Firing test (V8 rule 5).** The guard ships with a test in which it **rejects**: arm D
(`null_sd = 0`, one distinct value) must be *refused* by the guard. A guard that has never rejected
anything is untested, and §6.1 records three guards that passed while the condition they guard
against was present.

## Forbidden

- Re-choosing the truncation length, or the reading, after seeing the numbers.
- Interpreting C's truncated result if the arm-A positive control fails.
- Preserving the sustained-structural-activity framing if C detects on the truncated window.
