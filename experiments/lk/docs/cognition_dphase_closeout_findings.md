# D-phase closeout — selection-learning is stateable, and it took five instrument fixes to find out

Closes `hypersim_nn_experiments_plan_v2.md`, phases D0–D6. Synthesizes the
four per-phase records:

| doc | phase | stages |
|---|---|---|
| `cognition_stage_r_findings.md` | D3 | Stage R (R-1, R-2, R-3) + Stage I |
| `cognition_stage_0b1_prime_findings.md` | D4 | Stage 0b′, Stage 1′ |
| `cognition_stage2_prime_findings.md` | D5 | Stage 2′ |
| `cognition_stage3_prime_findings.md` | D6 | Stage 3′ |

Specs revised in the process: `fitness_formulas.md` (v2 → **v2.1**),
`instrument_registry.md` (v1 → **v3**), `campaign_output.md`,
`policy_config.md`. Three C0–C5 records amended in place with supersession
blocks rather than rewritten.

---

## The single sentence

**Selection-learning on hypergraph rewriting is stateable and it works — a
trained `MlpScorer` beats both budget-matched random and its own
initialization on F5 × `tri_two` — but every step toward that result was
gated by an instrument that was measuring the wrong thing, and the
instruments are the more transferable half of the output.**

C0–C5's verdict was that selection-learning is unstateable. That verdict was
correct **about the four objectives and the one policy class it tested**, and
plan v2's §5.1 generalization from it — "unstateable against every diversity
objective" — is now falsified twice: once by R-1's hand heuristics (D3) and
once by a trained network (D6).

---

## 1. What the programme now claims

| claim | status | evidence |
|---|---|---|
| A trained per-candidate scorer beats random and its own init | **positive** | D6: +382.02 at 14.67σ vs untrained, +341.09 at 10.85σ vs random, 0 flags |
| Non-uniform hand heuristics beat uniform selection | **positive** | D3 R-1: 13 credible comparisons, up to +68.76σ, on 3 of 5 rule sets |
| `StaticBias` × F1/F2/F3 is unstateable or null | **unchanged from C0–C5** | C-2, and D5's F5 × `wn_three` `OBJECTIVE_BLIND` |
| Memory capacity is bounded below one rewrite window | **negative, and publishable** | D4 Stage 1′: MC 0.0390 → 0.0364 under a 3× wider readout |
| Emergence headroom is a function of micro slop | **positive** | D3 R-2: 4 of 4 within-rung correlations negative, three within 0.02 of −1 |
| Separation classes (Stage 0b) | **withdrawn** | D4: the ratio exceeds its own hard bound on a majority of seed triples |
| F4 measures predictive information | **withdrawn** | D5 §5, D6 §3.1: it is approximately a monotonicity detector |

The headline positive and the headline negative are about **different pairs**,
and that is the substance of the D-phase: C-2 was not wrong, it was narrow.

---

## 2. The instrument thesis

Five defects were found across D5 and D6. Every one of them returned a
plausible number or a plausible abort code, and **not one would have been
caught by the test suite as it stood**.

| # | The instrument | What it stood in for | What it could not see |
|---|---|---|---|
| 1 | `constructor_records_examined == 0` | "the causal layer ran" | a live detector that found nothing |
| 2 | `f4_pairs < PI_MIN_PAIRS` | "F4 resolved the observable" | a quantizer 100× too coarse |
| 3 | P-3's σ-scaled probe | "the far end of the policy class" | that σ is P-2's variable, not P-3's |
| 4 | `RuleTrace::allocation()` | "the policy's behaviour changed" | a per-candidate policy re-ranking everything |
| 5 | `vec![0.0]` | "untrained" | a class whose neutral point isn't zero |

**They are one shape: a proxy standing in for a fact it cannot observe.**
That is a sharper statement than plan v2 §6.1's engagement-counter rule,
which catches the case where an instrument *never ran*. All five of these
ran. They reported confidently about something adjacent to the question.

Two consequences are now written into `instrument_registry.md` §5.4:

- **A precheck's own probe is an instrument** and needs the same discipline
  as the measurements it gates — a stated mechanism, a check that it engaged,
  and independence from the variables the *other* prechecks own.
- **A policy class's neutral point is a measured property, not a
  convention.** `θ = 0` is correct for `StaticBias` (uniform allocation) and
  degenerate for `MlpScorer` (§4 below).

### 2.1 The two that would have published a false result

Defects 3, 4 and 5 caused *aborts* — expensive, but the failure mode is a
campaign that doesn't run. Two were worse:

**F4's symbolization (defect 2)** would have published a measured zero. The
fixed `⌊4·jsd⌋` grid put every window in the first of four bins on **every
substrate at every horizon and window tried**, because the type-histogram JSD
lives in `[0.002, 0.085]`. Below `PI_MIN_PAIRS` that surfaced as
`unmeasured`, which is how Stage I mis-diagnosed it as a sample-size problem.
Above it, the one-symbol branch classified the run as *frozen* and returned
`0.0` — for a substrate that grew from 24 to 5992 nodes. Following Stage I's
own prescription (raise the pair count) would have produced a confident
"zero predictive information".

A sample-size counter cannot detect a resolution failure. `f4_jsd_spread` now
separates "the substrate was frozen" from "the substrate moved and we could
not see it".

**The empty constructor registry (defect 1)** fired on real data inside D5:
`tri_two` under the allocation extreme returns an empty registry with
`[causal]` **on** for 2 of 3 seeds, so P-3 passed a cell on a mean computed
from one seed. It is bug 8's fix over-firing in the opposite direction —
suppressing every genuine zero.

---

## 3. What each phase established

### D3 — Stage R and Stage I

**R-1 falsified plan v2's own central claim.** §5.1 predicted every
non-uniform heuristic should be *worse* than budget-matched random. 13
credible comparisons say otherwise, up to **+68.76σ**, across three of five
rule sets — surviving both the `|σ| > 100` instrument filter and a
degree-regularity control. Plan risk 4 was realized on the first real test,
which is the outcome a risk register is for.

**R-2 confirmed the emergence-headroom mechanism** with the confound
controlled: correlating `ei_micro` against `ce_structural` *within* budget
rungs (rather than pooled, where budget moves both) gives 4 of 4 negative,
three within 0.02 of −1.

**Stage I found no demotions** — and found, unasked, that F4 was unmeasurable
everywhere. That secondary finding was the more consequential one, and its
diagnosis was wrong (§2.1).

### D4 — Stage 0b′ and Stage 1′

**Stage 1′ confirmed its pre-registered null.** MC moved 0.0390 → 0.0364
under a readout three times as wide against a dimension-matched ESN — a 7%
move in the *wrong* direction for the readout hypothesis. The sharpest number
is the one that did move: `input_coupling_r²` at delay 0 went 0.9261 → 1.0000,
so the readout recovers the injected symbol **exactly** and still finds
nothing one window later. That is a substrate result, not a projection
artifact.

**Stage 0b′ withdrew a statistic rather than a readout.** The separation
ratio exceeds its own hard bound on a majority of seed triples under r1
*and* r2. Every published Stage 0b class is one draw from a distribution
spanning two classes. The instructive part: the single-triple table was
internally consistent and told a satisfying story ("r2 repairs two faulty
ratios and reclassifies a rule set") and was wrong in every particular. What
caught it was running the same measurement eight times.

### D5 — Stage 2′

**Exit met, 3 of 4 cells.** The §5.4 constructor-yield gate passed
decisively — 190–264 confirmed constructors on `wn_three`, 47–58 on
`tri_two` — retiring plan risk 2. F5 × `tri_two` passed all four prechecks
cleanly.

**F4 passes its prechecks and does not measure what it claims.** Its arm
ordering tracks how monotonically the JSD series decays, and the top arm on
`tri_two` scores **1.001× its own sorted series** — 1.60 of a possible 2 for
a sequence carrying nothing that sorting would not supply. `f4_drift` reports
this alongside every F4 value and is deliberately *not* subtracted: the drift
is non-linear, so a correction would need a model the programme does not
have.

**There is no third non-fixture multi-rule substrate.** P-1 was run over nine
candidate (rule set, IC) pairings from other tracks; all nine abort.
`conservation_soak` is the instructive failure — both its rules fire, never
simultaneously. This bounds replication for the whole programme, not just
this stage.

### D6 — Stage 3′

**The exit criterion is met, on the primary and secondary statistic.**

| arm | fitness | active_persist | n_nodes | flags |
|---|---|---|---|---|
| random | 825.58 ± 41.82 | 21.65 | 860.2 | 0 |
| untrained (init) | 784.65 ± 33.62 | 20.11 | 859.0 | 0 |
| **champion** | **1166.67 ± 15.04** | **26.04** | 853.8 | **0** |

Not Goodharted: no tripwire fired, and the champion carries the *most* active
persistent structure of any arm at a slightly *lower* node count. F5 and the
held-out structural metrics agree here, which is exactly what they do **not**
do under F4.

---

## 4. The finding worth keeping

Of the five defects, one is a fact about the architecture rather than about
the code:

**`gumbel_top_k` adds `Gumbel(0, 1)` noise to raw scores, and an MLP's output
scale is quadratic in its genome. So there is a wide basin around `θ = 0` in
which the policy is budget-matched random *whatever its weights are*, and the
basin's radius is set by the Gumbel temperature rather than by float
precision.**

| ‖θ‖ | score spread | reading |
|---|---|---|
| 0 | 0.00000 | exactly flat |
| 0.5414 | 0.00548 | where seven generations of search had reached |
| 7 | 0.92107 | comparable to the Gumbel sd of 1.28 |
| 41.6 | 32.53011 | deterministic argmax |

This explains three separate observations at once: why the untrained arm
equalled the random arm, why P-2 correctly reported that no selection had
changed after real parameter movement, and why the σ = 2.0 search (population
at ‖θ‖ ≈ 42) found a worse champion than the calibrated one.

It also makes the σ/α choice derivable rather than tuned. The ES update is
`Δθ = (α/(pairs·σ))·Σ(r⁺−r⁻)·ε`, so σ **divides** the step while widening
the exploration — and `|Δθ| ≈ 1.56·α/σ` per generation, a model checked
against the measured 0.5414 *before* being used to pick α. The difference is
not cosmetic: the mis-scaled search beat random by 1.76σ (below the bar), the
calibrated one by 10.85σ.

---

## 5. What this closes, and what it does not

**Closed.**

- Plan v2's build sequence D0–D6, in full.
- Plan risk 2 (F5 identically zero on real rule sets) — retired, decisively.
- Plan risk 3 (`glider` under r2) — closed with a proof rather than a
  measurement: its trajectory is a sequence of isomorphic graphs, so every
  isomorphism-invariant readout is constant on it by construction.
- Plan risk 4 (§5.1 too strong) — realized, twice.
- The Stage 2′ go/no-go, and the Stage 3′ exit criterion.

**Not closed.**

- **Stages 4′/5′ and 6.** Unlocked by D6 producing a champion, scoped by the
  plan, unrun.
- **Stage 0b's redesign.** The fix is specified (average the independent-seed
  reference over disjoint pairs; report spread; keep the hard bound as an
  abort) and unbuilt. Recorded in D4 §2.5 as outside that phase's scope.
- **F4's status.** It is stateable and it measures monotone drift. Whether a
  detrended variant measures anything else is unaddressed; the honest state
  is that F4 should not be used without its drift columns.
- **The endogenous-rule question (Stage 7).** Plan §9 argued that a Stage 2′
  *failure* would strengthen the case for moving there sooner. Stage 2′
  passed, so that argument does not fire — but the substrate-supply bound in
  D5 §6 is an independent reason it may still be the right next move.

---

## 6. Standing caveats

1. **One substrate, one objective.** The positive result is F5 on `tri_two`.
   `wn_three` under F5 fails P-3; F4 is blind on `wn_three` at five seeds.
   The claim is not about learned selection in general.
2. **No third substrate exists to replicate on** (D5 §6). This is a property
   of the repository's rule sets, and it bounds every claim above.
3. **Under-powered searches.** D6 ran 16 pairs × 2 seeds × 30 generations
   against the plan's default 32 × 3 × 60. For the *positive* result this is
   conservative — a smaller search makes an effect harder to find. It would
   be the central caveat on a null.
4. **Five hold-out seeds.** "14.67σ" has its pooled σ estimated from five
   points and should not be read to two decimals. The effect size does not
   depend on that precision: +341 against a random-arm sd of 41.8.
5. **P-2 at Stage 2′ scale is weak.** It needs a campaign, and plan §9
   budgets Stage 2′ at "minutes". D5's P-2 evidence came from 3-pair,
   1-seed, 8-generation runs, and the α sweep is what made it a measurement
   rather than an inference.
6. **Stage 0b's classes are withdrawn, not retracted.** The runs happened;
   what is missing is a statistic that turns them into a class.
7. **`hg-selfmod` does not compile** (uncommitted changes, not D-phase work),
   so `cargo test --workspace` fails. The three D-phase crates are tested
   explicitly: **278 passed, 0 failed**.

---

## 7. Provenance

Every number above traces to a harness in
`crates/hg-train/tests/`: `d3_stage_r_reanalysis.rs`,
`d3_r2_emergence_headroom.rs`, `d3_stage_i_demotion.rs`,
`d4_stage0b_stage1_r2.rs`, `d5_f4_horizon_probe.rs`,
`d5_stage2_prime_prechecks.rs`, `d6_stage3_prime_mlp.rs`. All are
`#[ignore]`d — they are measurement passes, not assertions.

The claims that *are* asserted, and so cannot silently regress:

| claim | test |
|---|---|
| F4 resolves a signal in a narrow band | `fitness::tests::f4_resolves_a_signal_confined_to_a_narrow_band` |
| a one-symbol sequence is a result only when frozen | `fitness::tests::a_one_symbol_sequence_is_a_result_only_when_the_substrate_was_frozen` |
| an empty registry means opposite things layer on/off | `fitness::tests::an_empty_registry_means_opposite_things_with_the_layer_on_and_off` |
| the drift diagnostic fires on pure monotonicity | `fitness::tests::the_drift_diagnostic_fires_on_a_purely_monotone_series` |
| the `StaticBias` probe is inert on an MLP | `prechecks::tests::the_static_bias_extreme_is_a_dead_probe_for_an_mlp` |
| traversal is read per policy class | `prechecks::tests::p2_reads_a_scorer_by_its_selections_and_a_logit_policy_by_its_allocation` |
| the divergence statistic itself | `prechecks::tests::selection_divergence_finds_the_first_differing_step` |
| the init scale is where the scorer can act | `prechecks::tests::the_mlp_init_scale_lands_where_the_scorer_can_actually_act` |
| the demotion pass can detect a dead instrument | `d3_stage_i_demotion.rs::the_demotion_pass_can_actually_detect_a_dead_instrument` |

**A note on how the D-phase went.** A large share of what it produced is
about instruments rather than about hypergraphs — five defects, one
withdrawn statistic (Stage 0b), one objective that does not measure what it
claims (F4). That is not a complaint about the code; it is the plan working
as designed. Plan v2 exists because C0–C5 produced eleven instrument bugs and
caught one of them with a test. The D-phase produced five more, caught all
five **before** publication, and each one now ships with a test that fires.

The substrate results are real — a positive learning margin, a publishable
memory null, a confirmed emergence mechanism, and a bound on how far any of
it can be replicated. But on the current evidence the instrument catalogue is
the more portable output, because it transfers to any programme optimizing
against its own evaluator, and the margins do not.
