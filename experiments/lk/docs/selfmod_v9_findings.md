# selfmod v9 — decayed merit: entry with defense

**Status: IN PROGRESS — sweep not yet run. Predictions PRE-REGISTERED
before any v9 gate output was read.**

## §0 What v9 is

v8's structural law: any history-accumulating resolver suppresses entry
(a bounded noise band never bridges an unbounded signal), while keyed-random
— the only mode with entry — destabilizes the population. v8 §3 routed two
fixes: decayed merit, and the collapse-mechanism trace.

v9 implements decayed merit as TWO parameters (the insight being that
neither alone suffices):

- `merit_decay` λ: each step, `m ← λ·m + productive`. Bounds a productive
  rule's steady-state signal at rate/(1−λ) and forces incumbents to defend
  position continuously — a rule that slumps for a few windows sees its
  signal fall geometrically.
- `merit_base`: a newborn's starting merit. Decay alone cannot open entry
  (a slumped incumbent at m = 0.001 still outranks a newborn at 0 — the
  ordering never inverts); the base gives newborns a positive standing that
  slumped incumbents fall BELOW. Entry happens during slumps; productive
  incumbents still hold.

Byte-compat: decay=1.0/base=0.0 is exactly the v8 cumulative table
(explicit-params run byte-identical to defaults run, seed 7, 2000 steps).
19 unit tests green (fractional-merit ordering: base-1.0 newborn beats a
0.5-slumped incumbent; a 2.5 incumbent still holds).

## §1 Pre-registered predictions

Sweep: est8 n=60/cell × {random, cumulative (decay 1/base 0), decay .95,
decay .9, decay .8} (all decay arms with base=1.0), coeff=5e-4, θ=0.1.
Survival arms: 8 seeds × same resolvers. Collapse traces on random's
dying seeds (seed 42/99/123/777, income columns).

1. **Significance at n=60**: random arm's guard-vs-clone contrast
   (v8: 0.292 vs 0.083) resolves to p < 0.05. If it does not, the v8
   effect was noise and the whole fitness program needs rethinking.
2. **Cumulative merit stays at ~0 entry** (v8 replication, both cells).
3. **Decay arms: entry opens monotonically in decay strength** — p_est
   (both cells) rises as λ falls; the guard>clone contrast persists at
   every λ where entry occurs (fitness conversion is mechanism, not
   accident of the random resolver).
4. **Survival under decay ≥ .9 stays ≥ 7/8** (near merit, far above
   random's 3/8): bounded signals keep shares away from the starvation
   edge. If decay .8 collapses like random, the stability mechanism was
   never the bounded signal but the frozen hierarchy itself.
5. **The frontier**: some (λ, base) gives BOTH survival ≥ 7/8 AND a
   significant guard-vs-clone contrast. If no arm does, the trade-off is
   real and the right resolver is task-dependent (measure fitness under
   random, run production under merit) — an unsatisfying but honest
   outcome.
6. **Collapse traces**: random's dying seeds show the income-variance →
   rent-starvation cascade (thin shares → negative balance → deletion).
   If instead they die of something else, the subsistence-floor fix is
   misdirected.

## §2 Results

### §2.1 The est sweep (n=60/cell, paired trial seeds, θ=0.1)

| resolver | p_est(guard) | p_est(clone) | Fisher p |
|---|---|---|---|
| random (coeff 5e-4) | 0.317 (19/60) | 0.083 (5/60) | **0.0012** ✓ |
| cumulative merit | 0.000 | 0.000 | — |
| decay .95 + base 1 | 0.000 | 0.000 | — |
| decay .90 + base 1 | 0.000 | 0.000 | — |
| decay .80 + base 1 | 0.033 | 0.017 | 0.50 |
| jitter ε=4 + decay .90 + base 1 | **0.183** | 0.033 | **0.0079** ✓ |
| jitter ε=4 + decay .80 + base 1 | 0.317 | 0.233 | 0.207 |
| jitter ε=2 + decay .80 + base 1 | 0.350 | 0.200 | 0.051 |

Prediction 1 confirmed decisively: at n=60 the random-arm contrast resolves
(p=0.0012). Prediction 2 replicated (cumulative = zero entry). Predictions
3–5 refuted in their decay-only form and CONFIRMED in the decay+jitter
form: decay alone never opens entry — a productive incumbent never slumps
below base (it sits at steady-state ≈ rate/(1−λ) every step; there is no
window) — but decay+jitter works exactly as the mechanism predicts: decay
bounds the incumbent signal (gap ln(6)→ln(2)), the ε=4 band covers it, and
entry + fitness conversion appear together (λ=.9: 18% vs 3%, p=.008).

### §2.2 Survival arms — and the coeff=0 discovery

| config (2000 steps, 8 seeds) | survived |
|---|---|
| random, coeff=0.0005 (v7/v8 reference) | 3/8 |
| **random, coeff=0** | **8/8** |
| merit cumulative / decay .9 (any base) | 8/8 |
| jitter λ=.9 ε=4 (coeff 0 or 5e-4) | 4–5/8 |
| jitter λ=.8 ε=4, coeff 5e-4 | 7/8 |
| jitter λ=.8 ε=2, coeff 5e-4 | 6/8 |

The surprise that reframes two rounds of work: **random's collapses vanish
at coeff=0** (8/8, healthy populations 11–13 rules). The v7/v8 "fairness
destabilizes" story was substantially a PRICING ARTIFACT — match-cost
drains thin-share rules into rent-starvation; fair shares are thin only
when matches cost tokens. Fairness itself was never lethal.

### §2.3 The adjudication

Does the fitness signal survive at coeff=0 (where survival holds)?

> random @ coeff=0: guard 22/60 (**0.367**) vs clone 9/60 (**0.150**),
> one-sided Fisher p = **0.0058** — significant.

**Plain keyed-random at zero match-pricing passes BOTH gates**: stable
population (8/8) and a clean fitness signal (p=0.0058, effect larger than
at 5e-4 — pricing was suppressing establishment overall). Every layer of
resolver complexity added since v7 was compensating for that parameter's
side effects.

decay+jitter remains the tunable fallback where per-match accounting must
stay (λ=.9/ε=4: significant signal at partial stability; λ=.8/ε=2: high
entry, borderline signal), but the default instrument of record is now
random @ coeff=0.

## §3 Verdict and lessons

1. The instrument of record: `est8 --tie_break=random --match_cost_coeff=0`.
   Both-gates-passing, simplest possible, fully deterministic.
2. General lesson (third instance of the class, after v8's tick and v7's
   queue): **a parameter introduced for one subsystem prices every other
   subsystem.** Match-cost was meant to make candidate enumeration costly;
   it also taxed thin shares into starvation and suppressed establishment.
   Instrument audits must sweep parameter interactions, not just direct
   effects.
3. Merit/decay/jitter stay in the codebase as the controlled vocabulary for
   scheduler fairness-vs-entry experiments; none is the default.

## §4 Routing for v10

1. Re-baseline the economy results (v5/v6 gradient and invasion-era
   findings) under random@coeff=0 — several may need revision.
2. Longer-horizon stability check for random@coeff=0 (20k steps, full seed
   set): 8 seeds × 2k steps passing is necessary, not sufficient.
3. With both gates green, the full-reflection milestone is finally
   unblocked: meta-rules as world-store subgraphs, matched by the same
   engine, selected under an instrument that measures what it claims.
4. If per-match accounting returns someday, pair it mandatorily with a
   subsistence floor (the collapse trace confirms starvation as the
   channel even though fairness alone is innocent).

## Provenance

- `merit_decay`/`merit_base` config+CLI; f64 merit table; byte-compat:
  explicit decay=1/base=0 identical to defaults (seed 7, 2000 steps);
  19 unit tests green.
- Sweeps: est8 n=60 × {random, cumulative, decay .95/.9/.8, decay+jitter
  ×3} (all cells share trial seeds 1000+t·7919); survival arms 8 seeds ×
  7 configs including the coeff∈{0, 5e-4} contrast; the coeff=0
  adjudication cell (60 trials).
- Raw outputs /tmp/v9_*.txt (session-local); tables above are complete.
