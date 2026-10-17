# selfmod v10 — re-baseline under the corrected instrument

**Status: IN PROGRESS. Predictions PRE-REGISTERED before any v10 run
launched. Instrument of record (v9 §2.3): keyed-random @ coeff=0.**

## §0 What v10 is

Three measurement tasks from v9 §4 routing — no new mechanisms:

- **Part A (stability)**: 20k-step long-horizon check, 3 arms × 8 seeds:
  random@coeff=0 (the instrument), cumulative merit (historical default),
  ruleid (historical reference). v9's 8/8 was 2k steps; this is the
  "necessary, not sufficient" follow-up.
- **Part B (Pareto re-score)**: G6.2 substitution scoring of `guard θ=0.1`
  vs resident R3 under random@coeff=0, against the historical v6 verdict
  (ADMIT under canonical@priced). Threaded tie_break/coeff params.
- **Part C (evolution probe, EXPLORATORY)**: first open-ended run under
  the corrected instrument — mu=1.0, random@coeff=0, 20k steps, 8 seeds.
  Labelled exploratory because nothing upstream measured evolution under
  a both-gates-passing instrument; any result is hypothesis-generating.

## §1 Pre-registered predictions

1. **Part A**: random@coeff=0 survives ≥ 6/8 seeds at 20k steps with mean
   final population ≥ 8 rules. If it degrades progressively (population
   declining slope), the 2k horizon masked a slow instability and the
   instrument needs a subsistence mechanism before v11+. Merit arm: 8/8
   (its v3–v6 record was at 20k). RuleId arm: 8/8 but lowest diversity.
2. **Part A secondary**: M_data (world WL-class count) under random@coeff=0
   exceeds merit's at 20k (fair scheduling spreads work across more world
   sites; v9's M_data hint at collapse vs 350).
3. **Part B**: under random@coeff=0 the guard's candidate-efficiency edge
   has no income channel (matches are free), so its admission must come
   from rate parity alone. Predict vp/r_prod ∈ [0.85, 1.05]: the
   energy>0.1 filter skips mostly-subproductive binds, so in-role rate
   stays near R3's. If instead vp/r_prod < 0.85, the guard's v9
   establishment win must be re-attributed (income-per-firing or
   node-selection advantages) — either way the Pareto vocabulary
   ("same work, fewer candidates") is incomplete without pricing.
4. **Part C**: majority survival (≥5/8) with positive M_rules slope on
   survivors (novelty threshold 1e-4). First honest look at whether the
   system diversifies when selection pressure exists (proportional
   income), scheduling is fair, and entry is possible. No strong prior —
   that is the point.

## §2 Results

### §2.1 Part A — long-horizon stability (20k steps, 8 seeds)

| arm | survived | final rules | M_data range |
|---|---|---|---|
| **random @ coeff=0** | **7/8** | 12,12,12,12,12,12,13 | 1791–2704 |
| cumulative merit @ 5e-4 | 8/8 | 10–13 | 316–360 |
| ruleid @ 5e-4 | 8/8 | 9–11 | 350–578 |

Prediction 1 confirmed: random@coeff=0 holds to 20k steps (the single loss
is seed 1, which died late — its 2k-step run had been among the healthy).
No progressive degradation in survivors: populations sit at 12–13 rules
at both horizons. Prediction 2 confirmed emphatically: world diversity
under random@coeff=0 is **6–8× higher** than merit's (M_data ~2000 vs
~330). Fair scheduling doesn't just share the world, it keeps the world
diverse — merit concentrates work on high-merit sites and freezes world
structure (M_data ≈ 330 flat across seeds is a frozen world, not a
healthy one).

### §2.2 Part B — G6.2 Pareto re-score under random@coeff=0

**G6.2 PASSES**: `guard energy>0.08` admitted (productive ratio 1.016×,
candidate ratio 0.902×, 15/16 paired wins on candidates, 16/16 survive).
The θ=0.1 variant also admits (1.027 / 0.915). Prediction 3 confirmed:
without pricing, the guard's edge expresses as rate parity plus fewer
enumerated binds — the substitution-arm measurement works unchanged.
The historical v6 verdict survives re-scoring; the *instrument* change
(v7's resolver fix) is what makes its coresidence numbers meaningful
now (θ=0.08 coresidence 27.87 vs R3's own arm, reported labelled).

Notable: the extremes die (θ≥0.25 reject:dies — too few matches to fund
fission), reproducing v6's bimodal θ-response under the new instrument.

### §2.3 Part C — first evolution probe: total collapse at every μ > 0

| μ | seeds | survived |
|---|---|---|
| 0.0 (Part A control) | 8 | 7/8 |
| 0.05 | 4 | **0/4** |
| 0.1 | 4 | **0/4** |
| 0.3 | 4 | **0/4** |
| 1.0 | 8 | **0/8** |

Every seeded run dies (`collapse_no_rules`), but M_rules spikes first
(4–35 classes registered at μ=1.0): mutation GENERATES novelty and the
population still dies. The signature is an **error catastrophe**: at
birth, fissioned token capital flows to a random mutant; the mutational
spectrum produced by the 15 structural operators is overwhelmingly
sterile or worse (v3 documented malformed offspring as "allowed junk
DNA" — under fair scheduling, junk children actually receive the
resources and then fail). Income falls, rent finishes the collapse.
At μ=0 the same dynamics survive indefinitely.

The critical realization: **v3–v6 never faced this because their
instrument suppressed entry entirely.** Under the ruleid queue, newborn
mutants could not take contested nodes, so populations were de facto
clonal regardless of μ — the old runs measured nothing about mutation.
Under the corrected instrument, the system's actual response to variation
is visible for the first time, and it is: no viable variation exists in
the current operator set.

## §3 Verdict

Parts A and B validate the corrected instrument end-to-end: stability at
long horizon, Pareto scoring working, diversity 6–8× higher than the
frozen-merit regime. Part C delivers the round's central finding and it
redirects the program:

**The bottleneck is no longer scheduling (fixed, v7), nor economy pricing
(fixed, v9), but VARIATION QUALITY.** The 15-operator mutation set
produces almost exclusively non-viable rule variants; there is currently
nothing for selection to select. Open-ended flow on this substrate
requires either (a) mutators that produce viable-by-construction
variants, or (b) a viability filter between birth and full capital
commitment.

## §4 Routing for v11

1. **Assay the spectrum**: single-mutant fitness distribution — for each
   of the 15 operators applied once to each seed member, measure the
   child's productive rate vs parent in isolation. This tells us whether
   ANY operator is a source of viable variation or whether the set needs
   replacement wholesale.
2. **Viability screening**: fission currently commits half the parent's
   capital unconditionally. A probation mechanism (child gets working
   capital, not endowment; unfired-and-unproductive children are
   reclaimed after N steps) would convert the catastrophe into selection
   pressure instead of extinction. Design must respect v8/v9's lesson:
   any such rule prices other subsystems — audit before adoption.
3. Re-run the evolution probe only after (1)/(2): the target line is
   survival at μ ≥ 0.05 with positive M_rules slope on survivors.
4. Full-reflection milestone stays queued behind a surviving probe.

## Provenance

- Part A: phaseB 20000 steps × {random@coeff=0, merit@5e-4, ruleid@5e-4}
  × 8 seeds, log_every=1000.
- Part B: gate62 --tie_break=random --match_cost_coeff=0, 16 seeds,
  2000 steps, full θ grid (gate62 CLI now threads tie_break/coeff).
- Part C: phaseB 20000 steps × μ ∈ {1.0 (8 seeds), 0.3, 0.1, 0.05
  (4 seeds each)}, random@coeff=0 throughout.
- Raw outputs /tmp/v10_*.txt (session-local); tables above are complete.
