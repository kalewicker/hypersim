# R5 findings — panel-primary outer loop vs the EXP-R07 null

## PRE-REGISTRATION (2026-09-04, before any R5 number exists)

**Gate.** Plan §9 item 3: R5 runs the outer loop with the descoped
kernel (DESCOPE taken as fee-gating — kernel unchanged, all 10 ops),
§6 policies 1–5 in force, **panel primary**. Committed null: the
EXP-R07 linear kernel-reach curve (`docs/ere_r6_findings.md`).

**Harness.** `expr_r09::r5_panel_primary`: reference loop
(`LoopConfig { epochs: 150, ..default() }` — 150 = one R07 quartile,
so per-quartile yields compare directly against 61/59/60/52),
full A0/A1/A2 + per-epoch panel emission (inherited from `run_arm`).
Seeds 0x9501/0x9503/0x9505 (odd, distinct, unused by any prior run).
Release.

**Null numbers (hardcoded from `ere_r6_findings.md`, seed 0x600D).**
600 kernel-only epochs → 234 shapes; quartile yields 61 · 59 · 60 ·
52; mean rate 0.39 shapes/epoch. R07 ran NO engine firing between
epochs (pure kernel reach); R5 runs 25 MaxParallel steps per epoch, so
the comparison measures what firing + fees + protection add or cost —
it is not a like-for-like race, and any gap is read as suppression
(FM-GE3 form), never as a harness defect to tune away.

**Predictions.**

1. A1 shape yield lands BELOW the null rate (suppression): R09 pace
   5/40 = 0.125/epoch projects ~19/150 vs null 61/quartile. Sign
   predicted, magnitude with wide bars (finding: firing volume is
   seed-contingent, SB-GE7(b) #1 — the projection inherits that
   lottery).
2. A0 yield also below null but above A1 (R09 pace 7/40 = 0.175 →
   ~26/150): separates fee suppression from firing-mediated effects.
3. A2 near-null pace (R09 14/40 = 0.35 → ~52/150): exclusion costs
   little reach at this scale; the §10.3 decision (A1 ≥ A2 novelty)
   is expected to fail again, void or not.
4. Panel: pump false on all arms (no concentration channel in this
   loop — three prior runs agree); incl≫excl persists; A1 excl
   recurrence stays at or near zero (FM-GE3 alphabet form deepens with
   more epochs).
5. No blob-panel excursion vs the §2 register on any arm (all R09-line
   trajectories to date: Saturating). Any excursion is reported as a
   species recurrence or new species, never averaged away.

**Decision outputs.** (i) FM-GE3 suppression curve: A1 vs A0 vs null
yields + quarantine shares — the R5 primary outcome. (ii) Fee-efficacy
stays unmeasured if A0 saturates again (SB-GE7 outer stands; mech
falsifier remains the efficacy evidence). (iii) First panel-primary
verdicts: scored novel/event per arm, site pump state, dual-scope
alphabet rows — quoted as results, not diagnostics.

## Results (2026-09-04) — SUPERSEDED by post-SB-R6 re-run (2026-09-08) below.
Filed table measured the clobber (labeled regions quarantined epoch 1);
kept for the record, not cited.

| Arm | traj (edge) | max_n | shapes (+scored) | quarters (null 61/59/60/52) | mut/inst/quar | paid/ref | accepted | novel/event |
|---|---|---|---|---|---|---|---|---|
| A0 fee-zero | Saturating (Saturating) | 525 | 23 (+20) | 5/7/6/5 | 130/70/60 | 0/0 | 3507 | 0.1709 |
| A1 main | Saturating (Saturating) | 439 | 25 (+22) | 8/1/8/8 | 124/58/66 | 5/0 | 7 | 0.2018 |
| A2 excl-shadow | Saturating (Saturating) | 836 | 16 (+10) | 8/6/1/1 | 131/125/6 | — | 2 | 0.0847 |

## Results post-SB-R6 (2026-09-08, ×2 byte-identical)

| Arm | traj (edge) | max_n | shapes (+scored) | quarters (null 61/59/60/52) | mut/inst/quar | paid/ref | accepted | novel/event |
|---|---|---|---|---|---|---|---|---|
| A0 fee-zero | Saturating (Saturating) | 4930 | 45 (+34) | 16/9/13/7 | 144/111/33 | 0/0 | 14439 | 0.2636 |
| A1 main | Saturating (Saturating) | 1744 | 57 (+50) | 16/15/12/14 | 149/127/22 | 345/0 | 347 | 0.3759 |
| A2 excl-shadow | Saturating (Saturating) | 1021 | 42 (+34) | 15/8/8/11 | 128/128/0 | — | 2 | 0.2931 |

Panel (final epoch): A0 excl 1.44/incl 16.00 recur 0.909 sites 144 H
3.875; A1 excl 2.88/incl 15.57 recur 0.990 sites 149 H 4.061; A2 excl
8.49/incl 16.76 recur 0.500 sites 128 H 4.133. Pump false everywhere;
incl≫excl everywhere. Excursion check quiet on all six trajectories.
Roots 3→113 / 2→129 / 3→130 — install-driven growth, no firing-driven
blob on any arm. A1 audits 150/150 balanced.

Prediction marks, re-marked:

1. **A1 below null ~gone — 57 vs 61/59/60/52.** Rate 0.38/ep vs
   0.39: the priced kernel now yields AT the null rate. The 2–3×
   suppression was clobber artifact (dead rules yield nothing).
2. **A0 vs A1 FLIPPED: 45 vs 57.** Fee COST in novelty is now
   negative — the priced kernel out-yields fee-zero on these seeds.
   Single seeds per arm (carried caveat) — but the filed "tie" is
   dead either way. §10.3 decision (A1 ≥ A2 novelty, 57 vs 42) HOLDS,
   now with margin instead of reversal.
3. **A2 saturation dead — 42 shapes, quarters 15/8/8/11.** No
   Q1-burst-then-collapse; the exhaustion signature (125 installs →
   16 shapes) was dead rules repeating shapes. Exclusion stays linear
   like the full kernels, lowest of the three but alive.
4. **Panel: A2's excl row UNCHANGED at 8.49** (recur 0.500, sites
   128) — the one panel number the clobber never touched (A2 installs
   only via MID ops; its alphabet was always live). A0/A1 excl rows
   (1.44/0.909, 2.88/0.990) now show the same collapse-and-recurrence
   the filed table attributed to horizon effects. Recurrence collapse
   is a live-rules signature, not a fee signature — the "no fee
   fingerprint" reading stands, strengthened (A1/A0 excl rows differ
   in level, identical in kind).
5. **No excursions ✓.** All six trajectories quiet against §2.

FM-GE3 readout, corrected: yield ratio A1/A0 = 57/45 ≈ 1.27 — the
sign flipped; quarantine share 0.148 vs 0.229 (slight elevation of
the DROPPED direction — A1 quarantines less). FM-GE3 does NOT fire on
this loop at this horizon. Single-seed caveat stands, now joined by
the multiseed distribution (which supersedes point ties — see below).

R5 verdict, corrected: the priced full kernel yields at ~null rate
with no fee fingerprint in novelty shape, alphabet kind, or
quarantine behavior; the exclusion kernel stays linear. Fee-gating
dominates exclusion on novelty at horizon — recorded, per §10.3.

*(Filed panel paragraph, prediction marks 1–5, FM-GE3 readout, and R5
verdict of 2026-09-04 removed 2026-09-08 — superseded by the post-SB-R6
section above. See git history for the record.)*

## R5-MULTISEED — PRE-REGISTRATION (2026-09-04, before numbers exist)

R5's caveat is the whole point of this run: every arm was one kernel
seed, and SB-GE7(b)#1 proved firing volume is a seed lottery (same
loop: accepted 2 vs 4002). The A1≈A0 tie (25 vs 23) and the A2
saturation curve (8/6/1/1) are each one draw.

**Harness.** `expr_r09::r5_multiseed`: same 150-epoch reference loop +
panel, 5 FRESH odd distinct seeds per arm (0x9511–0x951F step 2;
R5's 0x9501/03/05 stay out — the distribution is independent of the
point it checks). Per seed: shapes_end, scored gain, accepted total,
trajectory class, audits (A1). Per arm: min/mean/max yield + the full
5-point table (distributions, never singletons — finding #1's own
medicine). Release; ×2 byte-identical runs. (As run after the second
amendment: A0 ×4 — 0x9517 retired for compute pathology.)

**Predictions.** (i) Accepted-volume variance across seeds DWARFS arm
gaps (range ≥ 2 orders of magnitude on at least one arm). (ii) The tie
stands iff the A0 and A1 5-point yield ranges overlap with |means| ≤ 5
apart (pre-committed tolerance, owned as judgment at this scale).
(iii) Saturation is structural iff ≥4/5 A2 seeds show second-half
decline (q3+q4 < q1+q2 per seed). (iv) No arm explodes on any seed
(all 15 trajectories quiet — the Saturating verdict is robust even if
volumes aren't).

**Decision outputs.** Tie stands iff (ii) holds; saturation robust iff
(iii) holds; (i) quantifies the lottery and binds all future
firing-volume claims to multi-seed reporting. Any explosion is a §2
event (report, never average).

**Amendment (before the grid ran to completion — protocol, not numbers).**
The first grid attempt hung: A0 seed 0x9517 ground full-CPU 7+ minutes
without finishing epoch 150's windows (3/5 A0 seeds had completed in
~1 min total). 4th constructor arg is a pairings cap, not a node
ceiling — nothing bounds store growth, and an amplifier seed goes
superlinear. Per standing policy 1 (ceilings logged binding/non-binding
every run): the grid runs with `max_nodes: Some(2_000)` (≈2× the largest max_n on any
well-behaved run, 934 — binds only true blowups; 20_000 was tried
first and does NOT bind in time — a 20-epoch probe of
the same seed/arm completes in 0.03s, so the grind is superlinear late
(match enumeration at large epoch with a large rule table; no lever
exists inside the published conditions: the 10k pairings cap...[truncated]
binding reported per seed as a fifth outcome alongside quiet/excursion.
Enforcement breaks the step loop only (epoch structure, panel rows,
audits continue — the trajectory past the bind is censored in firing
volume, logged, never silent). Default stays `None`: every published
run is non-binding-identical. Decision rules (i)–(iv) unchanged.

**Second amendment (retirement, not re-roll).** The re-run confirmed
0x9517 as compute-pathological: >12 CPU-min without completing 150
epochs. Retired for compute cause — its datum stands as finding (i)'s
extreme point (cost lottery ≥1000× vs ~1s siblings), the A0
distribution runs on 4 seeds (asymmetry owned), no replacement
(selecting a seed that completes would be survivorship bias on the
measured axis). Grid as run: A0 ×4, A1 ×5, A2 ×5 (seeds in
`r5_multiseed`).

## Results (2026-09-04) — SUPERSEDED post-SB-R6; corrected table
(2026-09-08, ×2 byte-identical):

A0 seeds bound the ceiling (125/141/98/0ep) and three of four arms
leave Saturating — the "ceiling never bound, 14/14 Saturating" record
below measured dormant rules. Corrected grid (A0 ×4 incl. retired
0x9517's asymmetry, A1 ×5, A2 ×5):

- A0 (n=4): yields {49, 52, 70, 33}, range 33–70, mean 51.0; accepted
  {6113, 1914, 475412, 407}, range 407–475412. Trajectories:
  RegimeExploding, RegimeExploding, Expanding, Saturating.
- A1 (n=5): yields {56, 32, 59, 44, 40}, range 32–59, mean 46.2;
  accepted {2, 15, 268, 341, 210}, range 2–341. All Saturating.
- A2 (n=5): yields {69, 38, 42, 71, 86}, range 38–86, mean 61.2;
  accepted {1802, 2, 702, 2, 2}. All Saturating — the constant-2
  signature is dead (fees-off live rules fire too).

Verdicts on the pre-registered rules, re-marked:

(i) **CONFIRMED again, wider.** Accepted-volume lottery 1000×+ on A0
(407→475412), 170× on A1, 900× on A2. Binding precedent stands.

(ii) **now PASSES as written — but superseded in meaning.** Ranges
overlap broadly ([33,70] vs [32,59]) and |51.0 − 46.2| = 4.8 ≤ 5.
The letter passes where it failed; the substance moved on: the
paired grid (below) isolates the fee effect cleanly, and A0's
distribution now contains excursions the tie rule never contemplated.
Cite the paired result, not this tolerance.

(iii) **FAILS — inverted.** A2 sits HIGHEST (max 86 > A0 max 70, mean
61.2 > 51.0), no Q1-collapse anywhere. The exhaustion mechanism was
dead rules repeating shapes. Robustness-vs-A0 claim withdrawn in both
directions: A2 out-yields A0 here.

(iv) **Saturating-robustness broken on A0.** 3/4 A0 trajectories
excursion (2 RegimeExploding + 1 Expanding), ceiling binds 98–141ep
on all three. A1/A2 10/10 Saturating, ceiling never bound. The
Saturating verdict now reads: PRICED and EXCLUDED kernels saturate
everywhere; the FEE-ZERO kernel explodes on most draws. That is the
control SB-GE7 always wanted — see R5-C2-LOOP.

Filed 2026-09-04 record (dormant-rules numbers, kept not cited):
yields A0 {55,28,32,28} mean 35.8, A1 {30,15,30,14,21} mean 22.0, A2
{12,22,14,27,18} mean 18.6; accepted A0 ≤18k, A1 ≤368, A2 constant 2.

## R5-PAIRED pre-registration (2026-09-04, before the harness exists)

Closes the two owned debts at once: the unpaired grid cannot separate
fee-suppression from seed luck (yield gap 35.8 vs 22.0), and
fee-efficacy vs explosive FIRING is unmeasured (the mech falsifier
proves the throttle on explosive growth — subdivide couples firing to
node-minting — while the multiseed's high-volume Saturating seeds,
A0 accepted up to 18k with flat trajectories, are the explosive-firing
regime fees must demonstrably crush).

**Harness.** `expr_r09::r5_paired`: same 150-epoch reference loop +
panel + ceiling (`max_nodes: Some(2_000)`, bound logged per seed),
5 FRESH odd distinct seeds (0x9541–0x9549 step 2 — nothing used
before stays out: R5's 0x950x, the grid's 0x951x–0x953x, retired
0x9517), EACH seed run under BOTH A0 and A1. Per seed: A0/A1 shapes_end,
scored gain, accepted totals, trajectory classes, A1 audits. Reported:
per-seed yield gap (A0−A1 shapes), per-seed volume ratio (A0/A1
accepted), full 5-row table. Asserts mirror `r5_multiseed`
(survival, mapping, panel rows, A1 conservation) — decision rules
evaluated here, not gated there. Release; ×2 byte-identical runs.
Compute-pathology protocol (from 0x9517): a cell hanging >15 min wall
is retired for compute cause with its partial datum recorded, never
replaced.

**Predictions.** (P1) Yield: fee-suppression is ESTABLISHED iff all 5
per-seed gaps share sign (sign test, one-sided p≈0.03); direction
reported as-is (suppression OR boost — the prior is two-sided in
spirit, the test counts). 4/5 or worse = UNESTABLISHED, gap reported
as seed-dominated. (P2) Firing: a seed is an explosive-firing trial
iff A0 accepted ≥ 1000 (pre-committed floor — R09-routine 4002 counts,
single-install dribbles don't). Fee-efficacy is ESTABLISHED iff on
every explosive trial A1 accepted ≤ A0/10 (order-of-magnitude crush);
PARTIAL iff A1 < A0 everywhere but under 10× somewhere; FAILED iff any
explosive trial has A1 ≥ A0 (fees don't bind the regime that matters).
Seeds with A0 < 1000 contribute nothing to P2 (no regime to crush).
(P3) Shape: no arm explodes on any seed (Saturating throughout, as in
14/14 + mech-A1); any excursion is a §2 event, reported never averaged.

## R5-PAIRED results (2026-09-04) — SUPERSEDED post-SB-R6; corrected
table (2026-09-08, ×2 byte-identical, harness prints PAIRED SUMMARY:
gaps [3, 0, -1, 0, 0] mean +0.4):

Ceiling binds on one pair (0x9543: 88ep A0, 7ep A1); the other four
never bind. Trajectories: 9/10 Saturating — 0x9543 A0 is
RegimeExploding, the first trajectory excursion on the R09 line.

| seed | A0 shapes/accepted | A1 shapes/accepted | gap | crush |
|---|---|---|---|---|
| 0x9541 | 41 / 7 | 38 / 7 | +3 | 1.0× |
| 0x9543 | 56 / 1218 | 56 / 377 | +0 | 3.2× |
| 0x9545 | 50 / 3679 | 51 / 489 | −1 | 7.5× |
| 0x9547 | 56 / 3154 | 56 / 174 | +0 | 18.1× |
| 0x9549 | 58 / 4 | 58 / 4 | +0 | 1.0× |

(P1) **UNESTABLISHED, and the substance-resolution is dead too.**
Gaps [+3,0,−1,0,0]: not 5/5 one sign, and the exact-identity zeros
are gone (2/5 zeros; shapes roughly DOUBLE pre-fix everywhere —
24→41, 18→56 — the clobber had been halving the reachable set).
Seed-dominated gap reporting stands; yield-IDENTITY-under-pricing is
retracted (it measured one surviving rule, not the loop).

(P2) **PARTIAL (downgraded from ESTABLISHED).** Explosive-firing
trials (A0 ≥ 1000): three (1218, 3679, 3154). A1 < A0 on all three
but the crush is 3.2–18.1× — 0x9543's 3.2× misses the pre-committed
10× bar. Direction holds everywhere it matters; order-of-magnitude
does not. The "56–625× decisive" record measured copy-trap firehoses.
0x9541/0x9549 (A0 = 7/4, below floor) are quiet-seed controls: A1 is
7/7 and 4/4 byte-for-byte — fees still spare the garden.

(P3) **First excursion.** 0x9543 A0 RegimeExploding (bound 88ep) —
a §2 event, reported never averaged. The other nine arms Saturating.

Net, corrected: the fee thesis is DOWNGRADED, not dead — firing
crushed 3–18× where it matters (not 50–600×), invisible where it
doesn't; yield moves ±3 within-seed (not identical). Track debt
REOPENED on this axis: fee-efficacy at order-of-magnitude awaits the
R5-C2-LOOP retest, where the regime is emerged on-loop rather than
drawn by lottery.

Filed 2026-09-04 record (dormant-rules numbers, kept not cited):
gaps [0,+3,0,0,0], crush 56–625× on four trials, 10/10 Saturating,
ceiling never bound.

## R5-C2-LOOP — PRE-REGISTRATION (2026-09-08, before the harness exists)

Licensed by the SB-GE7(b) decision rule firing post-SB-R6: under the
corrected instrument C2 (bigger world) classifies RegimeExploding, so
C2 conditions become the R5 loop. This re-runs the R5 panel-primary
protocol THERE — the fee-efficacy-vs-explosive-firing measurement P2
could never cleanly make, now with an emerged regime on-loop.

**Harness.** `expr_r09::r5_c2_loop`: `LoopConfig { epochs: 150,
seed_nodes: 40, ..default }` (R5 protocol: 150 epochs, panel every
epoch, no ceiling — C2 hit max_n 1310 uncapped at 40ep, so a ceiling
would measure the ceiling; max_n reported as datum), full A0/A1/A2 +
panel. Kernel seeds reuse R5's 0x9501/03/05 (direct comparability with
the 10-node R5 point estimates). Asserts mirror `r5_panel_primary`
(survival, mapping, panel rows, A1 conservation) — decision rules
evaluated here, not gated there. Release; ×2 byte-identical runs.
Compute-pathology protocol as in R5-PAIRED (retire >15min wall for
compute cause, partial datum recorded, never replaced).

**Predictions.** (C1) A0 explodes (RegimeExploding or Expanding — the
C2 cell did at 40ep; an A0 excursion here is EXPECTED, not a §2
event). (C2) Fee-efficacy: A1 accepted ≤ A0/10 (the order-of-magnitude
bar P2-paired missed at 3.2× — this is its clean retest, same bar).
(C3) Shape: A1 and A2 Saturating (fees/exclusion contain what the
unpriced loop cannot). (C4) Novelty: A1 ≥ A2 shapes (§10.3 decision
retest — R09 15-vs-31 and R5 57-vs-42 disagree; C2 breaks the tie).

**Decision outputs.** (i) C2 crush ratio (paired-P2's successor).
(ii) §10.3 verdict on the C2 substrate. (iii) Whether the emerged
regime preserves linear yield (quarters) or goes superlinear.

## R5-C2-LOOP results (2026-09-08, ×2 byte-identical)

| Arm | traj (edge) | max_n | shapes (+scored) | quarters (null 61/59/60/52) | mut/inst/quar | paid/ref | accepted | novel/event |
|---|---|---|---|---|---|---|---|---|
| A0 fee-zero | RegimeExploding (RegimeExploding) | 20821 | 47 (+36) | 16/9/12/10 | 145/112/33 | 0/0 | 82238 | 0.2769 |
| A1 main | Saturating (Saturating) | 2034 | 57 (+50) | 16/15/12/14 | 149/127/22 | 616/0 | 636 | 0.3759 |
| A2 excl-shadow | Saturating (Saturating) | 1069 | 42 (+34) | 15/8/8/11 | 128/128/0 | — | 20 | 0.2931 |

Panel: A0 excl 1.01/incl 1.85 recur 1.000 sites 145; A1 excl
8.99/incl 20.58 recur 0.981 sites 149; A2 excl 5.46/incl 18.51 recur
0.983 sites 128. Pump false everywhere. Excursion flags: A0 both
(pre-registered expectation, not a §2 event); A1/A2 quiet. Roots
3→3→114 / 2→2→129 / 3→3→130 (start→min→end). A1 audits 150/150 balanced.

Prediction marks: (C1) ✓ — A0 explodes on-loop, both scopes.
(C2) ✓ **ESTABLISHED on-loop: 82238 → 636 = 129×.** The
order-of-magnitude bar P2-paired missed is cleared where the regime
is emerged rather than drawn — the measurement the track owed since
SB-GE7(a), now outside any fixture. (C3) ✓ — A1 and A2 Saturating;
fees contain what fee-zero cannot. (C4) ✓ — A1 57 ≥ A2 42, third
reading in agreement (R09 15-vs-31 failed, R5 57-vs-42 held, C2
57-vs-42 holds): §10.3 for fee-gating on novelty, two to one.

Decision outputs: (i) crush 129× on-loop. (ii) §10.3 HOLDS on C2.
(iii) the emerged regime does NOT preserve linear yield on A0
(quarters 16/9/12/10 sag; max_n 20821 uncapped) — but A1's quarters
(16/15/12/14) stay flat at ~quarter-null rate: pricing linearizes the
explosion it cannot fully suppress (636 accepted still flows).

Coincidence owned, not hidden: A1/A2 shapes+quarters are
numerically identical to the 10-node R5 (57/16-15-12-14, 42/15-8-8-11)
while accepted/panel/max_n all differ. Same kernel seeds → same
install stream → same shape classes discovered; the bigger world
changes what FIRES, not what gets built. A shape-level null result
for substrate scaling, stated as datum.

Net: SB-GE7(b) is now CLOSED for real — the control emerged, the
licensed retest ran, efficacy measured on-loop at 129×. The GE track
has no open debt on the fee axis under the corrected instrument.

## R5-C2-GRID — PRE-REGISTRATION (2026-09-08, before the harness exists)

The 129× is a single draw (kernel seeds 0x9501/03/05), and this
track's binding precedent (multiseed finding i) says single draws are
lottery tickets — including R5-C2-LOOP's own. This runs the paired
grid on the C2 substrate: distributions, never singletons.

**Harness.** `expr_r09::r5_c2_grid`: `LoopConfig { epochs: 150,
seed_nodes: 40, ..default }` (C2-loop conditions, no ceiling —
C2-loop A0 hit max_n 20821 uncapped; a ceiling would measure the
ceiling), full A0/A1/A2 + panel. 5 FRESH odd distinct seeds
(0x9551–0x9559 step 2 — everything used before stays out). EACH seed
under ALL THREE arms (paired within-seed; the unpaired lesson is
paid for). Per seed: shapes_end, accepted totals, trajectory classes,
A1 audits, per-seed gap (A0−A1), crush (A0/A1), A1-vs-A2 novelty.
Asserts mirror `r5_paired` (survival, mapping, panel rows, A1
conservation) — decision rules evaluated here, not gated there.
Release; ×2 byte-identical runs. Compute-pathology protocol as before.

**Predictions.** (G1) A0 explodes on a MAJORITY of seeds (≥3/5
RegimeExploding/Expanding — emergence is the substrate's, not one
draw's). (G2) Fee-efficacy DISTRIBUTION: on every explosive trial
(A0 ≥ 1000) A1 < A0, and ≥10× crush on a majority of explosive
trials (PARTIAL-or-better by the paired-P2 rule; ESTABLISHED iff all
trials clear 10×). (G3) Shape: A1 and A2 Saturating on all seeds
(containment is structural, not lucky). (G4) Novelty: A1 ≥ A2 shapes
on a majority of seeds (§10.3 on distribution — fourth reading).

**Decision outputs.** (i) Crush-ratio distribution (the 129× grown
up). (ii) Linearization check: A1 quarters flat on all/majority of
seeds? (iii) §10.3 verdict on the C2 distribution. (iv) Emergence
rate: fraction of A0 seeds exploding (substrate property datum).

## R5-C2-GRID results (2026-09-08) — PARTIAL: 3/5 seeds + 1 compute kill

The run died on 0x9557-A0: SIGKILL, no panic/assert output, mid-cell —
presumed memory kill on the uncapped explosive substrate (0x9553's
surviving sibling reached max_n 51114). 0x9557 recorded as
COMPUTE-KILLED (cause: machine, not loop — same precedent as 0x9517),
never replaced; 0x9559 UNRUN (the binary died first — not imputed,
not skipped-by-choice). Completed cells, release:

| seed | A0 shapes/accepted | A1 shapes/accepted | A2 shapes/accepted | gap | crush | nov A1−A2 |
|---|---|---|---|---|---|---|
| 0x9551 | Sat 43 / 48 | Sat 42 / 58 | Sat 21 / 20 | +1 | 0.8× | +21 |
| 0x9553 | RE 51 / 8572008 (max_n 51114, 1249s) | Sat 51 / 479 | Sat 46 / 20 | +0 | 17895.6× | +5 |
| 0x9555 | RE 49 / 37539 (max_n 1940) | Sat 52 / 587 | Sat 64 / 30 | −3 | 64.0× | −12 |
| 0x9557 | KILLED (machine) | — | — | — | — | — |
| 0x9559 | unrun | — | — | — | — | — |

A1/A2 trajectories: 6/6 Saturating. A2 quarantines 0 on all seeds
(MID ops never meet a guard — consistent everywhere).

Prediction marks, conditional:

- (G1) **UNDECIDED, capped.** 2/3 completed A0 explode; the kill
  cannot count either way; even a 0x9559 explosion caps the rate at
  3/5. Emergence on C2 is seed-contingent (2 explode / 1 fully quiet
  / 1 beyond-machine) — the substrate enables, the draw decides.
- (G2) **ESTABLISHED-conditional, 2/2.** Explosive trials clear 10×
  with room (17,895×, 64×); direction 2/2. Small N, stated.
- (G3) **Holds on completed, 6/6.** Containment has not failed once
  (priced or excluded, any substrate, any seed to date).
- (G4) **Split, 2/3 for A1.** Second A2-lead datum on C2 (64 vs 52;
  cf. multiseed A2-mean-highest). Readings now: fail, hold, hold,
  2/3 — §10.3 holds on balance, weakest exactly where the regime is
  emerged. The honest form: fees beat exclusion on novelty except
  where exclusion's install-happy kernel finds rich substrate.

Protocol breaches, owned: (a) 0x9553-A0 ran 1249s past the 15-min
retirement rule — kept the completed datum (strictly more
informative than partial; nothing re-rolled or replaced). The rule
needs a memory clause, not just a wall clause — written into the
recommendation below. (b) Test marked `#[ignore]` post-hoc (harness
hygiene, not a result change — an OOMing default-suite test is a
hazard, not a finding).

Scale finding (the kill AS datum): uncapped C2-loop regimes can
exceed this machine's memory within 150 epochs. C2-loop's 20k nodes
was not the ceiling of the ceilingless. RECOMMENDATION (not a result):
future long-horizon C2 work runs WITH `max_nodes` + logged binding
(standing policy 1) — binding-epoch fraction becomes part of the
datum, and the kill class is retired by construction.

Net: the 129× grows up into a distribution of {17895×, 64×} on
explosive draws with one quiet seed and one beyond-machine draw.
Fee-efficacy holds everywhere it can be measured; emergence itself
is the lottery now, not suppression.
