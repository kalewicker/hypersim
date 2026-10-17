# selfmod v8 — the calibrated null and the jitter hybrid

**Status: IN PROGRESS — sweep not yet run. Predictions below are
PRE-REGISTERED (written before any est8 output was read).**

## §0 What v8 is

v7 left two loose ends, both routed for v8:

1. **The null was never calibrated.** "Share ≥ 2× seed share in ≥ 6/8"
   demands growth from a single copy, but a true neutral's dominant outcome
   from n=1 is absorption at zero. v8 replaces the criterion with
   establishment probability — p_est = P(lineage holds ≥ 2 copies at
   step 2000 AND population alive) — and compares the guard variant against
   the neutral clone binomially over 24 trials/cell (`est8`).
2. **The resolver endpoints both failed.** Merit freezes the hierarchy
   (newborn lockout); random dissolves the population (5/8 collapse).
   v8 adds `TieBreak::Jitter`: key = ln(merit+1) + ε·(u−½), a bounded noise
   band in log-merit space. At ε=0 it IS Merit; newborns tie at 0 and
   separate by noise alone; incumbents' history dominates once the gap
   exceeds ε. (A linear multiplier 1 + ε(u−½) was tried first and rejected:
   at ε ≥ 2 it goes NEGATIVE — a merit-10000 incumbent could lose outright.
   The unit test caught this; log space cannot go negative.)

## §1 Pre-registered predictions

Sweep: est8 (24 trials/cell) × tie ∈ {random, jitter ε=1, jitter ε=2,
jitter ε=4, merit}, coeff = 5e-4, θ = 0.1. Survival arms: phaseB churn3
8 seeds × {random, jitter ε=2, merit}.

1. **p_est(clone) > 0 under random** — v7's 0/8 was 8 trials of a
   low-probability event, not impossibility. Guess: p_est(clone) ≈ 0.1–0.3.
2. **p_est(guard) > p_est(clone) under random** — the v7 signal (4/8 vs
   0/8) replicates as rates; significance needs the bigger n.
3. **Survival: jitter ε=2 ≈ merit (8/8), both > random (3/8)** — the noise
   band is small enough that incumbents keep their shares.
4. **Establishment: p_est rises with ε** — newborns need noise to enter;
   ε=1 (band ±0.5 in log space) may be too tight to beat an incumbent at
   ln(m+1) ≈ 2–3; ε=4 should open the door.
5. **The frontier exists**: some ε gives BOTH 8/8 survival AND
   p_est(guard) − p_est(clone) ≥ random's contrast. If no ε in {1,2,4}
   does, the hybrid idea is refuted and the resolver search continues.

## §2 Results

### §2.1 The calibrated null (est8, 24 trials/cell, coeff=5e-4, θ=0.1)

| tie | p_est(guard) | p_est(clone) | one-sided Fisher p |
|---|---|---|---|
| random | **0.292** (7/24) | **0.083** (2/24) | **≈ 0.05–0.07 (borderline)** |
| jitter ε=1 | 0.000 | 0.000 | — |
| jitter ε=2 | 0.000 | 0.000 | — |
| jitter ε=4 | 0.042 (1/24) | 0.000 | ≈ 0.5 |
| merit | 0.000 | 0.000 | — |

Predictions 1–2 confirmed in DIRECTION, with a correction: the neutral
null is CALIBRATED, not zero — p_est(clone) = 0.083 under a fair coin
(v7's 0/8 was small-n sampling, not impossibility) — and the guard
establishes 3.5× more often than the null. **However, the significance
figures originally published here were computed by a buggy tail**
(wrong row totals in the hypergeometric; printed < 0.0001). Corrected:
7/24 vs 2/24 gives p ≈ 0.05–0.07 — borderline, NOT significant at 0.05.
The honest v8 conclusion is therefore: the fitness signal is REAL ENOUGH
TO MEASURE (the instrument works, the direction replicates v7's 4/8-vs-0/8)
but 24 trials/cell cannot yet resolve it at conventional significance;
v9's first job is more trials (n ≈ 50–100/cell puts p ≈ 0.05 within reach
if the true rates are near these estimates).

All five cells share trial seeds (1000 + t·7919), so cells are paired
across resolvers — the between-mode comparisons below are not confounded
by trial draw.

### §2.2 Survival arms (churn3, 8 seeds, 2000 steps)

| tie | survived | mean copies |
|---|---|---|
| merit | 8/8 | 11.75 |
| jitter ε=4 | 7/8 | 9.0 |
| jitter ε=2 | 6/8 | 9.6 |
| random | 3/8 | 3.9 |

Jitter buys stability back monotonically-ish with ε (6/8 → 7/8 vs random's
3/8) but never reaches merit's 8/8-with-more-lineages. Prediction 3 was
half right: jitter ≫ random, but jitter ≠ merit.

### §2.3 The hybrid refuted — and why it had to be

Prediction 4–5 refuted: establishment under jitter is ~0 at every ε tried,
even though survival improves. The mechanism is structural, not tunable:

**any noise band bounded in ε around a signal that grows without bound
loses forever.** Incumbents reach ln(m+1) ≈ 4–5 by invasion time (1500
productive steps); a ±ε/2 band of 0.5–2 cannot bridge that gap. Raising ε
until it could (ε ≈ 8–10) would make the band wider than the entire
incumbent signal — i.e. random with extra steps. Bounded jitter on
cumulative merit is not a midpoint between the endpoints; it IS the merit
endpoint, plus decorative noise.

The general statement, which supersedes v7's §3.5: **under any
history-accumulating resolver, entry is structurally suppressed** — a
newborn can only take queue position from an incumbent whose accumulated
history it cannot match, so fitness expresses itself only through the
fission channel (which also requires winning matches, so: not at all).
Merit-world is a closed society: stable, more lineages, zero entry.
Random-world is an open one: real entry, real fitness conversion,
fragile population. There is no dial between them of the form
"history + bounded noise".

### §2.4 Verdict

- The instrument of record for fitness measurement is **keyed-random +
  est8**: it is the only configuration in which entry occurs and fitness
  differences convert into establishment differences at known rates.
- Random's fragility must be attacked where it lives — the income-variance
  → starvation cascade — not by re-injecting history into the queue.
- The natural candidate that escapes the §2.3 impossibility is
  **decayed merit**: m ← λ·m + productive, bounding m* ≤ rate/(1−λ).
  Unlike cumulative merit, position must be DEFENDED continuously — an
  incumbent that slumps for a few windows becomes beatable, so entry
  happens during slumps while productive incumbents still hold. This is
  v7's own hazard-note escape hatch ("a windowed/decaying estimator is
  indicated"), now with the measurements that motivate it.

## §3 Routing for v9

1. Implement `TieBreak::Decay` (λ configurable, e.g. 0.95/0.9/0.8) and run
   the same est8 × survival grid. Pre-register: p_est(guard) rises with
   decay strength; survival stays nearer merit than random at moderate λ.
2. Diagnose random-collapse directly (income trace of dying seeds): if the
   cascade runs through rent-starvation of thin shares, a subsistence floor
   is the complementary fix and may combine with decayed merit.
3. Only after a resolver passes BOTH gates (entry + stability): re-run G6.2
   substitution scoring and revisit the v6 cap×budget conditioning.

## Provenance

- `TieBreak::Jitter` + `jitter_eps` config/CLI; log-space key after the
  linear form failed `test_jitter_history_dominates_when_gap_large`
  (negative keys at ε ≥ 2). 14 unit tests green.
- `v8_establishment` gate (`est8` CLI): p_est over N trials/cell, paired
  trial seeds, one-sided Fisher via Lanczos lgamma.
- Sweep: est8 × {random, jitter 1/2/4, merit} × 24 trials; survival arms
  8 seeds × {random, jitter 2/4, merit}. Raw outputs in /tmp/v8_*.txt
  (session-local), tables above are complete copies.
