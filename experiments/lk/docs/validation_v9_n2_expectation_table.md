# V9 / N2.1 — Phase 2 expectation table, with honest provenance

Written after the Phase 1 gate cleared. **This is not a blind pre-registration, and it must not be
presented as one.** The reason is structural and is worth recording.

## A tension in the plan's own design, stated plainly

N1.1 requires the instrument's null to be reported **per arm** ("Report per arm: real score, null
mean/p95/max, realised spread, count null ≥ real, p, and null support size"). N2.1 requires the
Phase-2 expectation table to be committed **before any scoring**. These cannot both be satisfied:
validating the instrument *is* scoring the arms.

So by the time this table is written, the per-arm metabolic CE numbers already exist and I have seen
them. Rather than dress up hindsight as prediction, every row below is flagged:

- **[SEEN]** — the value was already produced by Phase 1 validation; this is a *record*, not a
  prediction.
- **[NEW]** — genuinely not yet computed when this file was committed.

## The expectation table

| arm | structural result | metabolic CE result | provenance | reasoning |
|---|---|---|---|---|
| A replicating | detected, z = 6.56 | detected, **z = 20.16** | [SEEN] | births/deaths give the richest delta history |
| B motile, non-repl | detected, z = 2.28 | detected, **z = 14.72** | [SEEN] | harvest/metabolise cycling |
| B′ low-activity | detected, z = 1.18 (p = .010) | detected, **z = 17.96** | [SEEN] | metabolically busy despite 6 structural events |
| C token | **not** detected, z = −0.83 | detected, **z = 9.12** | [SEEN] | alive 120 steps *does* register metabolically |
| D stationary | **unmeasurable** (null sd = 0) | detected, **z = 11.72** | [SEEN] | energy changes 5999/5999 steps |
| inert control | n/a | **unresolvable** (k = 1, 0/10 seeds) | [SEEN] | delta ≡ 0 ⟹ one micro class, by construction |

**[NEW] and therefore genuinely predictive** — the questions Phase 1 did *not* answer:

1. **Multi-seed per arm.** N1.2 hardened only arm A (30/30). Whether B, B′, C, D each detect across
   seeds is unmeasured. *Prediction:* A, B, B′, D reach regime property (≥ 0.90); **C is the one to
   watch**, because its 120-step lifetime gives it far fewer transitions and its structural null was
   the near-degenerate one.
2. **Cross-channel rank correlation.** Whether the metabolic ordering is *inversely* related to the
   structural ordering, quantified rather than eyeballed.
3. **Retention on churn (N4.3).** Whether the ~0% structural-retention result survives a moving
   substrate.

## The headline question, answered honestly

The plan's sharpest question — **"is arm D, structurally invisible, the strongest signal on the
metabolic channel?"** — is already answerable from Phase 1, and the answer depends on the statistic:

- **On the permutation-tested boundary statistic (CE): NO.** Arm A is strongest (z = 20.16); arm D is
  **fourth of five** (z = 11.72).
- **On a raw per-carrier activity rate: YES.** Arm D is highest (0.9998).

These are not in conflict; they answer different questions. CE asks *"is the carrier/substrate
partition informative about the dynamics?"*; the activity rate asks *"how much does an average
carrier's energy move?"* Arm D's six carriers each move maximally, but the *partition* separating
them from substrate is less informative than arm A's, because arm A's births and deaths give the
carrier class a richer and more distinctive transition structure.

**Phase 2 must report both and must not quote the flattering one.** The quotable version of the
shadow result — "the structurally invisible arm is metabolically the loudest" — is **true only of the
activity rate, and false of the validated boundary statistic.** That distinction is the finding.

## What Phase 2 therefore delivers

The **two-channel table** (N2.2), not a single verdict, plus:

- per-arm multi-seed on the metabolic channel (the [NEW] item that carries real evidential weight);
- the cross-channel rank comparison, stated with the statistic-dependence above;
- the pre-committed reading (N2.3) applied to the CE statistic, since that is the one that passed
  Phase 1's gate.

## Pre-committed reading for the [NEW] multi-seed item

- **C reaches regime property on the metabolic channel** ⟹ the channels genuinely disagree about arm
  C: structurally undetected, metabolically detected across seeds. The structural finding is
  **relocated, not retracted** — C is a real shadow-level negative and a metabolic positive.
- **C fails to reach it** ⟹ arm C is weak on both channels and the disagreement is single-seed.
- Either way, **no adjustment of either channel to reconcile them** (rule 6).
