# selfmod v5 findings — fitness gradient, and why efficiency still cannot be selected

**Status: the plan stopped at G5.3, as ground rule 7 requires.** The invasion gate failed;
§5.2 (E-buffer), §5.3 (E-grad) and §5.4 (E-world) were therefore **not run**. This document
reports the carry-in corrections, the gates that did pass, the E-stoich result, and a full
diagnosis of the G5.3 failure.

Figure: [selfmod_v5_invasion.svg](selfmod_v5_invasion.svg).
Calibration: [selfmod_v5_calibration.md](selfmod_v5_calibration.md).

---

## C1 — corrected μ = 0 accounting, and a further correction it forced

**[Measured]** The v5 plan was right that v4's sentence was inconsistent with its own table.
Re-derived from the run summaries (`churn3`, flux, μ = 0):

| | seeds | births |
|---|---|---|
| survive, never reproduced | 42, 99 | 0 |
| survive, heavy reproducers | 2, 777 | 803 clonal each |
| die, reproduced | 1, 3, 7, 123 | 5–13 clonal |

4 survivors (P = 0.50), 6 reproducers of which 4 died, 1642 clonal births total. v4's
findings have been corrected in place with a marked correction note.

**The 803-vs-5 contrast stands and is the cleanest controlled result in the program.** Seeds
2 and 777 survive 803 clonal offspring each at μ = 0; at μ = 0.05 — same seeds, same
configuration, mutation the only difference — both die after 5 mutant offspring each.

### But v4's carry-in #2 is now refuted [Measured, new]

E-stoich's control arm supplies a comparison v4 never ran: the same worlds at μ = 0 with
reproduction **suppressed entirely** (`repro_threshold` set unreachable).

| seed | with reproduction | without reproduction | reading |
|---|---|---|---|
| 1, 3, 7, 123 | dies | dies | intrinsic world instability |
| 2, 777 | **lives** | **dies** | reproduction is **load-bearing** |
| 42, 99 | lives | lives | stable either way |
| **total** | **4/8** | **2/8** | |

**Reproduction did not kill a single seed.** The four deaths v4 attributed to fission cost
occur identically when fission is impossible. Two seeds survive *only* because they
reproduce — 803 clonal copies acting as repair, replacing cycle members as they starve.

So v4's claim that "reproduction itself destabilizes the set, independent of variation" is
**refuted**, and with it the v4 carry-in that prioritised reproduction economics. The
correct statement: at μ = 0 deaths are intrinsic world instability; reproduction is net
protective; only *variation* is destabilizing. This also removes the plan's premise for
ranking `split`/`partial`/`refractory` fission modes as a primary lever — they were
implemented and are available, but the motivation for them is gone.

**[Inferred]** Seeds 2 and 777 survive by accumulating spontaneous redundancy (population
grows 3 → 6), which is the same mechanism §3.2's balanced-copy-number proposal was designed
to supply deliberately. That is indirect prior support for E-buffer's hypothesis (a),
which G5.3's failure prevented us from testing.

---

## C2 — E-stoich: the stoichiometry framing is **refuted as stated**

**Design.** `churn3`, μ = 0, natural fission suppressed so a forced duplication is the only
perturbation, applied at step 50 (the first attempt used step 500 and was void — the cycle
freezes at ~242, so the perturbation landed on an already-dead population; logged as an
excursion).

**[Measured]** 8 seeds per arm, response = time to freeze:

| arm | survival | median t_freeze | per-seed t_freeze |
|---|---|---|---|
| control (no duplication) | 2/8 | 624 | 242, 775, 964, 624, –, –, 294, 588 |
| R1 only | 2/8 | 528 | 180, 607, 884, 528, –, –, 284, 476 |
| R2 only | 2/8 | 658 | 242, 744, 956, 658, –, –, 294, 594 |
| R3 only | 2/8 | 624 | **identical to control, seed for seed** |
| balanced (all three) | 2/8 | 528 | **identical to R1-only, seed for seed** |

The C2 prediction was: unbalanced duplications die, balanced survives ≈ control. Neither
half holds.

- **No arm changes survival at all** (2/8 everywhere).
- **Balanced ≡ R1-only, exactly.** Balanced duplication does not restore control behaviour;
  it reproduces the effect of duplicating the single most impactful member.
- **R3 duplication has literally zero effect**, bit-identical to control on every seed.

**[Inferred] mechanism.** Rules generate ~1141 match candidates per step against
`match_cap_per_rule = 64`, and conflict resolution then accepts only ~5 node-disjoint
bindings per step. So a second copy of a member does not raise its reaction rate directly;
it raises that member's *share of the conflict-resolution slots*. Members whose realised
rate is substrate-limited (R3 — it needs label-3 nodes, which are scarce) are insensitive
to copy number; members that are slot-limited (R1) are sensitive.

**Caveat, stated plainly:** because of the cap, this arm tests perturbation of *realised
rates via slot competition*, not idealised reaction stoichiometry. C2's prediction fails
under either reading, since balanced perturbation is indistinguishable from the largest
single perturbation and nothing moves survival. The k-dependence of μ_c that C2 was invented
to explain therefore remains **unexplained** — the mutational-surface story was retired by
the plan, and the stoichiometric story does not survive its own test.

---

## Gates

- **G5.0 [Measured] PASS** — G0, G1 (bit-identical decode-vs-compiled, 3 seeds × 100 steps),
  G4.2 (all 8 effect-gate cases, including the stricter delete-and-recreate case kept per C4),
  flow-audit residual 0 on every run.
- **G5.1 [Measured] PASS — the v5 reframe is now a measurement, not an argument.** Under
  v4's flat economy, the variance of `net_income` across productive rules is **exactly
  0.000e0**, on all 8 seeds, at every logged step. Fitness in v1–v4 was binary; what the
  program called selection was a firing filter. This is the single most important
  confirmation in v5 and it validates the plan's §0 reframe completely.
- **G5.2 [Measured] PASS** — with proportional income and match-proportional cost, per-rule
  `net_income` variance is 1.5–12.6 depending on calibration (versus exactly 0 under v4).
  A gradient demonstrably exists.
- **G5.3 [Measured] FAIL** — see below. Plan stops here.
- **G5.4** — not reached.

---

## G5.3 — the invasion gate: FAIL, with a specific diagnosis

**Pre-registered threshold** (fixed before running): the invader's lineage share must reach
**≥ 2× its seed share in ≥ 6/8 seeds**.

**[Measured] Result at the calibrated operating point** (income_pool 12, coeff 5e-4,
base_rent 1, invader introduced at step 500 into an established population):
**invader share → 0.000 in 8/8 seeds**, median ratio 0.00, while residents survive 8/8 and
grow to 8–11 rules. **0/8 seeds meet the bar.**

### The invader really was superior

Per-member, per-copy, per-step diagnostics with the invader present:

| | resident R3 | invader |
|---|---|---|
| match candidates | 3640 | **5** |
| productive applications | 1.57 | 0.80 |
| productive per candidate | 0.00043 | **0.16** (370× better) |
| income | 3.03 | 0.29 |
| cost | 3.00 | 1.00 |
| **net income** | **+0.03** | **−0.71** |

The variant does half the productive work for 1/730th of the matching cost — exactly the
"strictly superior, tighter LHS" rule the gate calls for — and it goes bankrupt anyway.

### No parameter setting rescues it [Measured]

| base_rent | coeff | invader | residents | median share ratio |
|---|---|---|---|---|
| 1 | 5e-4 | extinct 0/8 | survive 8/8 | 0.00 |
| 1 | 1e-3 | extinct | **all die 0/8** | 0.00 |
| 1 | 2e-3 | extinct | **all die 0/8** | 0.00 |
| 1 | 5e-3 | extinct | **all die 0/8** | 0.00 |
| 0 | 2e-3 | survives 8/8 | survive 8/8 | 0.71 |
| 0 | 5e-3 | survives 8/8 | survive 8/8 | 0.67 |
| 0 | 1e-2 | survives 8/8 | survive 8/8 | 0.50 |

Removing base rent lets the invader persist, but its share still **declines** in every case.
It never grows anywhere on this sweep.

### The apparent displacement was an artifact — control run [Measured]

At `rent = 0, coeff = 2e-3` the invader's target R3 goes extinct in 8/8 seeds, which looks
like successful competitive displacement. **It is not.** The control — identical
configuration with no invader — shows R3 extinct in 8/8 seeds anyway: at that coefficient
R3's cost (0.002 × 3640 = 7.28) exceeds any income it can earn, so it is non-viable on its
own. The invader did not outcompete R3; the cost coefficient bankrupted it.

Running this control is what separates a real result from a satisfying-looking one, and it
turned a would-be pass into a confirmed failure.

### Diagnosis — and it is none of the three the plan anticipated

The plan offered: allocation quantization / cost coefficient too small relative to
`base_rent` / closure coupling. The measured cause is adjacent to the second but strictly
different, and it is structural:

> **Reproduction is gated on an absolute token threshold, so lineage growth tracks absolute
> net income. Proportional income makes income scale with productive *volume*, while
> efficiency only reduces *cost*. A rule can therefore be arbitrarily more efficient and
> still accumulate more slowly than a wasteful high-volume rule.**

The only way to make the cost term decide outcomes is to raise `match_cost_coeff` until it
bankrupts the high-volume rules — and at that point it bankrupts the population (rows 2–4
above) or, with rent removed, simply deletes the expensive member regardless of any
competitor. **The window in which efficiency is rewarded and the window in which the
population is viable do not overlap.**

So §3.1 did not achieve its stated aim. It successfully replaced binary fitness with a
continuous one (G5.2 proves the variance is nonzero), but the continuous gradient it
produces is still a **volume** gradient, not the efficiency gradient the plan specified.
Two coupled changes were made; a third is needed and was not identified in advance:
**reproduction must be gated on a rate or a ratio, not on an absolute accumulated stock.**

---

## What this does and does not license

**Does not license** the deeper conclusion the plan attached to a G5.3 failure — "no
reachable mutation can be beneficial, the substrate admits no improvement at all." That
reading assumed a failure would mean beneficial variants cannot be *selected*. What was
actually measured is narrower and more actionable: beneficial variants exist and are
identifiable (the invader is unambiguously better on the intended metric), but the
**reproduction criterion** cannot see them. The substrate is not flat; the fitness readout is
mis-specified.

**Does license** retiring two prior diagnoses:
- **D-gradient is not resolved but is now precisely located.** Novelty could not accumulate
  in v4 because fitness was binary (G5.1 confirms). Making fitness continuous is necessary
  and was achieved; it is not sufficient, because the continuous signal still measures volume.
- **D-world remains untested.** It was not reached, and it should not be built on this
  substrate until reproduction responds to efficiency, or the world experiment inherits the
  same blindness one level up.

---

## Routing for v6

1. **Fix the reproduction criterion first — this is the whole critical path.** Gate fission
   on *rate* rather than *stock*: e.g. reproduce when net income per step (or per match
   candidate) exceeds a threshold, or make `birth_cost` scale with the parent's match cost so
   wasteful lineages pay more to reproduce. Then re-run G5.3 unchanged. The invasion gate is
   the right gate and its threshold should not be relaxed; only the mechanism under test
   should change.
2. **Do not run E-buffer, E-grad or E-world until G5.3 passes.** Each measures selection on a
   substrate where selection has been shown not to respond to the intended trait.
3. **Re-open the μ_c(k) explanation.** Both candidate mechanisms are now retired — mutational
   surfaces (by the plan's C2) and stoichiometric imbalance (by E-stoich). The k-dependence is
   real and unexplained. The slot-competition observation above (substrate-limited members are
   copy-number-insensitive, slot-limited ones are not) is the most promising lead.
4. **Redundancy has indirect support and remains untested.** The two μ = 0 survivors persist
   by spontaneously accumulating copies. Balanced copy-number seeding is implemented and
   ready; it just needs a substrate where the gate passes first.

## Promotion assessment

**Unchanged: do not feature-gate the decode phase into `hg-engine::step()`.** The plan
flagged that a G5.3 pass would warrant reconsidering, since a substrate that is both durable
and improvable is a different promotion candidate. G5.3 did not pass. G1 continues to hold
bit-identically, and the library-driver pattern absorbed every v5 addition — a second income
mode, per-rule cost accounting, four fission modes, copy-number seeding, forced duplication,
and an invader injection path — with zero engine changes. There is still no result that would
be easier to obtain from inside the engine.

---

## Excursions and deviations (ground rule 6)

1. **E-stoich step-500 run was void** — the perturbation landed after the natural freeze
   (~242), making all five arms identical. Re-run at step 50. Both runs are retained
   (`ES_*` void, `ES2_*` valid).
2. **Two unplanned controls, both decisive**, and both added to make an existing claim
   attributable rather than to chase an outcome: the **reproduction-suppressed control**
   (refuted v4's carry-in #2) and the **no-invader control** at `rent=0, coeff=2e-3`
   (turned an apparent invasion success into a confirmed failure).
3. **`match_cost_coeff` / `base_rent` sweep beyond the plan's single calibration**, to
   establish that G5.3's failure is not a calibration accident. It is not.
4. **Match-cost is charged against the previous step's candidate count**, so the cost phase
   keeps its v4 position in the step order and the gradient arm stays order-comparable to the
   v4 arm. Noted as an implementation choice, not a deviation from intent.
5. **C4 honoured** — the stricter effect gate (delete-and-recreate scored sterile) is
   retained and still pinned by G4.2.
6. **C3 honoured** — v4 tagged `selfmod-v4`; all v5 work is on branch `selfmod-v5`, off the
   unrelated V8/V9 history.
