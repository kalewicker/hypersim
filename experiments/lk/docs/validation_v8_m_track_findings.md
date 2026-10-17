# V8 / M-track — the churn substrate is measurable, the empirical leg survives, and the
# missing instrument has a candidate

Reported against the V8 plan. Harnesses: `m1_h0_full_criterion.rs`, `m2_arm_c_and_null_guard.rs`,
`m3_metabolic_dynamics.rs`, `m4_1_diffuse_gradient.rs`, `m4_2_multiseed_bprime.rs`,
`m4_4_selection_power.rs`. Pre-registrations committed before their numbers: M2 `0b5fd17`,
M3.1 `71b072c`, M4.2 `0ae74b3`, M4.4 `c9a31ee`. Seed 12345 unless a multi-seed set is stated.

---

## The single sentence

**The bounded-churn substrate is measurable after all — WL passes every clause of the H0.1 criterion
and an independent second witness, once scored after burn-in; the verdict's one empirical leg
survives its degeneracy check, though the pre-registered truncation could not show it and an
exploratory extension had to; and an instrument that distinguishes a carrier that is *metabolising*
from one that merely *exists* does exist — the closure-defect estimator on the ALife energy channel,
which scores the pre-committed inert control at exactly zero and arm D far above it.**

---

## M1 — the H0 correction HOLDS, on all four clauses and a second witness

The L-track correction rested on **one** clause of the H0.1 criterion (TWTF); the three secondary
clauses (M(t) closure, live-class persistence, T/M) were never evaluated. All four now are, and
bounded churn passes every one (TWTF 0.000, new-classes-in-last-quartile 0.000, T/M 5580, live
persistence 0.999).

The **second witness** — window-to-window WL class-set Jaccard, a coarser timescale than the
step-to-step metrics — **agrees**: at the scoring window (W = 6000) bounded churn has steady Jaccard
1.000 (bare and with carriers) against the trusted F8 static baseline of 0.773, while the growing
proxies sit at 0.004–0.070 and get *worse* with window length. The discriminator is the **trend**:
churn's per-block Jaccard *rises* monotonically (settling), the growing proxies are flat-low
(asymptotic). H0's FAIL was about substrate **growth**, not node turnover.

**The witness earned its place by catching what TWTF structurally cannot.** On the churn-with-carriers
run, ~215 of 231 WL classes are **startup-transient** (per-block M: 228 → 34 → 16 → 16 → 16 → 16),
minted while the substrate fills to the cap and never seen again. TWTF misses them because each lived
≥ R = 3 steps — exactly the blind spot K1 flagged. **Operational consequence:** the churn substrate
must be scored **after a burn-in**, or the F0 support bar is computed over junk classes (the F0
artifact in a new guise, H0.1 clause 3).

*Transparency:* the steady-state aggregation and the matched window were chosen after seeing the
window-mean dragged down by startup pairs; neither was pre-registered — only H0.1's four clauses
were. Every per-pair Jaccard is printed so any other aggregation can be applied.

## M2 — the empirical leg survives, but not the way the pre-registration expected

With arm D definitional, **arm C carries the whole non-definitional weight** of the
sustained-structural-activity verdict, and C might have failed only because its carrier group is
empty for 98% of the window (K = 1, CE = 0 by construction).

**The pre-registered truncation could not resolve it.** Scoring C over its live phase (W =
100/110/120, "populated throughout") gave a **degenerate null** (sd = 0, 1 distinct value) — because
those windows end at or before C's die-off, so C's only in-window structural events are its 6 births
at step 1, putting it in arm D's blind spot. This exposed a **flaw in my own pre-registration**:
requiring the window be "populated throughout" excluded C's deaths, i.e. excluded exactly the
activity that makes C measurable. The positive control (arm A) *does* detect at those windows, so the
failure is specific to C.

**An exploratory extension resolved it** (labelled post-hoc, not substituted for the pre-committed
result): at windows that *contain* the deaths (W = 130/150/200/400), C's null is non-degenerate and
C returns **z = −0.83, p = 0.930 — identical to the full window**, across empty-group fractions from
8% to 98%. The empty tail is inert padding: after step 120 the graph is constant, so those steps add
uniform self-transitions that leave CE unchanged. That is evidence **against** the empty-group-artifact
hypothesis, so the empirical leg holds — on exploratory rather than pre-registered footing, which is
stated plainly.

**M2.3 — the null-spread guard.** `C(M,k) > 100` counts partitions a permutation *could* produce, not
whether they differ; it accepted arm D at 3.78×10² while the realised null had sd = 0. Replaced by:
`null_sd > 0` **and** `null_max − null_min > 0` **and** ≥ 10 distinct realised values. Applied
retroactively to all 10 published G/K/L arm-partition pairs, **only "D stationary WL" is rejected**
(watch-list, passing but low: B′ WL 59 distinct, C WL ~20). Ships with a **firing test**: the old bar
accepts arm D, the new guard rejects it (rule 5).

## M3 — an instrument that sees dynamics, not level, exists

The inert-carrier control was committed before any candidate (M3.1) and built by **rule-set
composition** (keep only `bootstrap_seed`), so the firewall is untouched; it is verified genuinely
inert (6 carriers, alive all 6000 steps, energy-change rate exactly 0).

**The gate** (inert ≈ 0, D ≫ 0) is cleared by all three candidates. Checking the *full* pre-committed
table (all four arms, not just the gate) separates them:

| candidate | inert | C token | D stat. | A repl. | C below D? | full table |
|---|---|---|---|---|---|---|
| 1 energy-transition entropy (lifetime-norm) | 0.000 | 1.663 | 1.592 | 1.711 | no | **fails C row** |
| 2 per-carrier activity rate (lifetime-norm) | 0.000 | 1.000 | 1.000 | 0.983 | no | **fails C row** |
| **3 closure defect (bits)** | **0.000** | **0.852** | **0.919** | 0.907 | **yes** | **matches, no repair** |

Candidates 1 and 2 need window-normalisation to place the dead token below arm D (a post-hoc repair,
reported as a variant). **Candidate 3, the closure defect, matches all four rows as specified** —
vindicating the M3.3 prediction that the project's own construct is the natural fit, and making it the
recommended instrument.

**The sharpest number in the track:** the normalised closure defect `H(micro | macro) / H(micro)` is
**1.0000 for arms C and D and 0.9986 for arm A** — the structural (WL) macro description explains
**0.00%** of the metabolic (energy) micro uncertainty in C and D, and 0.14% in A. The Q2 architectural
blind spot is not special to arm D; **arm D is the extreme case of a gap present in every arm.** All
three candidates are stationary under bounded churn (D's closure defect 0.9193 → 0.9190).

*Scope:* clearing the M3.1 gate is a **necessary** condition — it rules out the definitional signal —
**not a finished instrument.** None of these is yet a permutation-tested detectability statistic with
a null, so none yet produces a p-value comparable to the WL results.

## M4 — supporting items

- **M4.1 — `diffuse` characterised bare on imposed gradients.** The L1.2 firewall test showed only
  *inertness on a uniform field*. Against five imposed gradients (single point / half-and-half /
  single sink / staircase / random), influx removed, `diffuse` is **conservative** (drift 0.00e0 on
  all five), **quantised** (no node changes by more than the declared 1.0/step), **non-negative**,
  **topology-inert** (n and edges flat), and **self-limiting** (reaches equilibrium and stops). Newly
  visible: transport is strictly nearest-neighbour, one quantum per node per step, so it tops up only
  nodes *adjacent* to a surplus node (half-and-half leaves 12 sub-threshold nodes with no donor
  neighbour). `diffuse` is now characterised physics.
- **M4.2 — multi-seed B′.** The sole support for "even a little activity suffices" was p = 0.010 on
  one seed. Now **30/30 seeds detect** (sign test p = 1.9×10⁻⁹, zero degenerate nulls); every included
  seed has exactly 6 relocations and detects, weakest null spread 62 distinct values. **Regime
  property.** The seed-12345 draw was the *weakest* of the 30. The new degenerate-null inclusion clause
  fired 0 times (correctly — no arm-D-like cases among live B′ populations).
- **M4.3 — guard audit.** Most V-track criteria have a firing test (null-spread guard rejects arm D;
  H0.1 + second witness reject the growing proxies; M3.1 gate rejects candidates 1–2; firewall asserts
  conservation). Two gaps logged: the M4.2 null-spread *inclusion* clause has no positive firing case
  among sustained seeds (inert here), and M4.4's null-validity check has no dedicated firing case.
  Neither blocks a claim.
- **M4.4 — powered selection.** At **30 established seeds** (32% establishment, 15 000 steps), with a
  clean drift null (inert traits 43%/53%, envelope 6.33): **satiation is drift-dominated** (ratio
  1.01, sign fraction 0.50, p = 1.000). `replication_threshold` is a real, strong sign bias (97% down,
  p < 0.001) whose magnitude nonetheless sits **inside** the drift envelope (ratio 1.16). **Criterion 5
  stays met degenerate-only, now confirmed at power.**

---

## L3 / report-level consequences

- **§5.6 (H0 correction):** upgrade from "TWTF suggests churn is stationary" to **"churn passes the
  full H0.1 criterion and an independent second witness, scored after burn-in."** Record the
  **burn-in requirement** as an operational condition of every churn-substrate experiment.
- **§5.9 / §9 (arm C / empirical leg):** the sustained-structural-activity verdict's empirical leg
  **holds** — but state honestly that the pre-registered truncation was degenerate and the support is
  the exploratory constant-CE-across-empty-fraction result, plus the multi-seed B′ regime property.
- **§9 / §10 (instrument):** the metabolic-dynamics gap has a **candidate instrument** (closure defect
  on the energy channel), with the striking supporting number that WL explains ~0% of the metabolic
  channel in every arm. Not yet a null-tested statistic.
- **Scorecard criterion 5:** confirmed **degenerate-only at 30 seeds**; the downgrade stands.
- **What is now unblocked** (M1.3, scoped not run): the temporal experiment (§8) and the born/dying
  substrate control both become constructible on the burn-in-scored churn substrate, using **WL**.

## Scope and standing caveats

- M1's steady-state aggregation and M2's resolving windows are **post-hoc**, labelled as such; the
  pre-registered criteria are the four H0.1 clauses and the populated-throughout truncation (which
  failed).
- M3's candidates are **not yet null-tested**; the gate is necessary, not sufficient.
- Everything WL-side is single-instrument on this substrate family; the Q2 blind spot is architectural
  and bounds every structural claim (§10).
- Magnitude remains out of scope (F2.4).
