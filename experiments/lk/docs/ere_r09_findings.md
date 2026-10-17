# EXP-R09 findings — the fee-gated outer loop (GE plan §10)

Driver: `crates/hg-meta/src/expr_r09.rs`, test `exp_r09_outer_loop_a0_a1_a2`.
Harness: 40 epochs × 25 MaxParallel steps, protection on, burn-in = first
4 epochs unscored. Run with `cargo test --release -p hg-meta --lib expr_r09`.

## Run 1 (2026-09-03) — stop fired, plus an owned driver bug

A0 came back `Saturating`, not Expanding/Exploding → the pre-registered
stop-and-report fired: the fee was never binding, A1-vs-A0 suppression
unmeasurable. Per pre-reg, no tuning; report.

Owned driver bug in the same run: seeds `0x90A0/0x90A1/0x90A2` passed
through `XorShift(seed | 1)`, and `0x90A0|1 == 0x90A1|1` — A0 and A1 ran
kernel-identical trajectories. Fixed in run 2 (seeds `0x90A1/0x90A3/0x90A5`,
odd and distinct). The stop verdict itself was not a seed artifact (run 2
confirms), but run 1's A0-vs-A1 comparison was void.

## Run 2 (2026-09-03) — control absent, confirmed; RE-RUN post-SB-R6 (2026-09-08)

Pre-fix numbers are struck through the rewrite below — SB-R6 (mirror
key 0 = lit-value key) V4-quarantined every labeled region at epoch 1,
so the filed table measured the copy-rule trap, not the loop. Post-fix
table (`exp_r09_outer_loop_a0_a1_a2`, release, ×2 byte-identical):

| Arm | epochs | roots | traj | max_n | shapes_end (post-burn-in) | mut/inst/quar | paid/refused | audits | accepted_tot |
|---|---|---|---|---|---|---|---|---|---|
| A0 fee-zero | 40/40 | 32 | Saturating | 355 | 18 (+12) | 38/30/8 | 0/0 | 40/40 | 354 |
| A1 main | 40/40 | 28 | Saturating | 278 | 15 (+11) | 36/26/10 | 9/0 | 40/40 | 11 |
| A2 excl-shadow | 40/40 | 40 | Saturating | 460 | 31 (+24) | 38/38/0 | 0/0 | — | 2 |

A0 CONTROL VERDICT: **still ABSENT (SB-GE7)**. A1-vs-A0 suppression unmeasurable here.

## What the numbers say

1. **The fee mechanism works as built — firing collapses 354 → 11
   (32×).** Pre-fix this read 4002 → 7; the 4002 was SB-R6-amplified
   copy-trap churn (the only rule that survived the clobber), not loop
   firing. Same mechanism story at the corrected level: adults earn
   1/epoch after the 2-epoch throttle, seeds burn endowment in epoch 0,
   A0's free-firing rules churn labels/edges into ever-more match
   opportunities (positive feedback), A1's throttled rules leave the
   substrate static. Conservation held 40/40 audits; mapping dropped 0.
2. **There is no blob to suppress.** A0's 354 firings are node-neutral
   (32 roots ≈ 11 nodes each at max_n 355, Saturating); the explosive-
   compounding channel G-GE1 probed on fixture worlds never engages in
   this loop. Growth here is install-driven, not firing-driven. (This
   reading survives SB-R6: the level changed 10×, the shape didn't.)
3. **A2 (exclusion) is the novelty leader with no blobs either**: 38
   installs, 31 shapes, saturating at 460 nodes. The §10.3 decision
   (A1 ≥ A2 novelty) currently fails 15 vs 31 — wider than the filed
   5-vs-14, same direction. The comparison is still void while the
   control is absent.
4. **FM-GE3 watch: quarantine share 0.211 vs 0.278** (A0 vs A1, of
   kernel events — printed by the harness). The filed 0.667/0.677 level
   was clobber-inflated: most pre-fix quarantines were the instrument
   eating labeled regions, not protection verdicts. The independence
   reading stands (small gap, same direction), now at the true level.

## SB-GE7(a): mechanism test — FALSIFIED (2026-09-03)

`expr_r09_mech.rs`, gate positive control (subdivide fixture) as a live
region on `seed_world_fixed(8)`, W = 100 = 4 × 25-step windows:

| Arm | traj | max_n | accepted | paid/refused | window-end nodes |
|---|---|---|---|---|---|
| A0 | RegimeExploding | 208 | 200 | — | — |
| A1 | RegimeExploding | 120 | 100 | 4/96 | [70, 70, 70, 120] |

The falsifier fired as designed (§10.5/§10.6 REDESIGN-REOPEN); the
record below is the pre-fix evidence. Diagnosis, quantitative:

- Window 0 (endowment 3, snapshot solvent) forwards everything: 8 → 70.
  The 1.5×/10-step bar trips inside the first window, so the verdict is
  sealed before the throttle ever observes bankruptcy. A1-window-0 ≡ A0.
- Windows 1–2 (bankrupt observed) withhold everything: flat at 70.
  Withholding WORKS — the failure is purely the first-window grant.
- Window 3 (income drip, free = 1) re-opens the FULL window: 70 → 120.
  Snapshot throttle is binary per window — 1 token buys 25 free steps
  (~50 firings). The design gates whole windows; it prices nothing at
  the margin. R09-A1's freeze (7 accepted) is the all-windows-closed
  corner of the same mechanism, not a separate regime.

Redesign options: (i) stateful per-step debit in `FeePolicy` (forward
decrements free; bankruptcy bites next step — endowment 3 admits 3
firings, income 1 admits 1; exploder → ~8→11→12 Saturating; amends the
SB-GE5 split + solvent pin); (ii) zero endowment earn-to-fire
(param-only, coarser — newborns dormant 2 windows); (iii) shorter
windows (weak — open 5-step windows may still trip the bar). (i)
recommended; the mech falsifier becomes its regression pin.

## SB-GE7(a): RESOLVED — redesign (i) landed (2026-09-03)

`FeePolicy` now projects within-window spend: forwards while one more
firing fits the window-start balance, withholds past it
(`forwarded: Vec<u64>` per-rule counts, `withheld_total` diagnostic;
ONE policy object per window, rebuilt from post-boundary ledgers).
Boundary debit reconciles actuals (losers never pay). Mechanism
re-run, same harness:

| Arm | traj | max_n | accepted | paid/refused/withheld | window-end nodes |
|---|---|---|---|---|---|
| A0 | RegimeExploding | 208 | 200 | — | — |
| A1 | Saturating | 24 | 4 | 4/0/393 | [21, 22, 23, 24] |

Falsifier GREEN: exploder → Saturating, firing 200 → 4, debit
reconciles exactly (paid == accepted, refused == 0 — the projection
invariant end-to-end), growth arrested to the ~+1/window income drip.
R09 re-run under projection, post-SB-R6: A1 accepted 11, paid 9 /
refused 0, audits 40/40 balanced, trajectory Saturating max_n 278 —
SB-GE7 outer-loop verdict unchanged (A0 still saturates; the control
question is separate from the throttle mechanism, now proven).

## SB-GE7 (owned): outer-loop blob regime undemonstrated

R09 assumed the outer loop would reproduce the selfmod-style blob under
zero fees; it doesn't at 40×25 on a 10-node protected seed. Fee efficacy
against explosive FIRING is therefore unmeasured (the throttle itself is
measured and works). Two paths, in recommended order:

- **(a) Mechanism test (cheap, causal).** Install a known-exploding G-GE1
  (child, fixture) pair as a live region, run A0-vs-A1 windows on the
  fixture world, compare trajectories. Directly answers "do fees throttle
  explosive firing" without needing outer-loop emergence.
- **(b) Emergence-first.** Find loop conditions where A0 explodes (bigger
  world, longer windows, unprotected band, …), THEN re-run R09. Open-ended
  and tunable — higher p-hacking surface; do (a) first.

No harness tuning to chase the control (pre-reg commitment stands).

## SB-GE7(b): emergence search — PRE-REGISTRATION (2026-09-04)

One shot, four A0-only cells (fees cannot bind without the regime, so
A1 runs are unwarranted until a cell succeeds). Panel emitted on every
cell (G-GE2 exited — the search is interpretable). Single seed per
cell, no re-rolls: re-rolling winners is p-hacking with extra steps.

| Cell | Loop | Mechanism addressed |
|---|---|---|
| C1 reference | 10-node protected seed, 40×25 | coherence: must reproduce slice-3 A0 (Saturating, max_n ≈100, shapes ≈7) |
| C2 bigger world | 40-node protected seed, same 2 seed rules, scaled edges, 40×25 | finding §2: children dormant for lack of LHS match surface — 4× substrate |
| C3 longer windows | 10-node protected, 40×100 | compounding room per window |
| C4 unprotected band | 10-node, `meta_visible=true`, 40×25 | guard-free matching reaches the meta band itself (`protect.rs` A3 — the largest match-surface change available without new verbs) |

Seeds 0xB001/0xB003/0xB005/0xB007 (odd, distinct; engine stream fixed
at 42 as in R09).

Predictions: C1 Saturating (pin, not a prediction). C2 Saturating,
max_n moderately above reference (install-driven, cf. A2's 240).
C3 Saturating, accepted ≫ 4002 (same churn, more steps). C4
Saturating — widest bars; guard-free meta-band matching is the only
cell that opens a qualitatively new firing channel.

Decision rule (pre-committed): a cell SUCCEEDS iff its A0 node_traj
classifies Expanding or RegimeExploding. First success in listed order
C2 → C3 → C4 (C1 excluded by prior data) becomes the R5 loop, re-run
there full A0/A1/A2 + panel. No success ⇒ SB-GE7(b) CLOSED as a failed
attempt with panel evidence; R5 proceeds on the reference loop
panel-primary with fee-efficacy recorded unmeasurable. No new cells,
no seed re-rolls, no parameter nudges after seeing numbers.

## SB-GE7(b): results — RE-OPENED and RE-RUN post-SB-R6 (2026-09-08);
decision rule FIRES → C2

The filed 2026-09-04 table measured the clobber, not the loop (all
four cells ran A2-protection through the colliding key). Post-fix
table (`expr_r09::sb_ge7b_emergence_search`, release, one shot):

| Cell | traj | max_n | shapes (+scored) | mut/inst/quar | accepted | novel/event | panel excl/incl | sites/H/pump |
|---|---|---|---|---|---|---|---|---|
| C1 reference | Saturating | 287 | 12 (+7) | 37/27/10 | 7 | 0.2188 | 2.00/14.71 | 37/2.766/false |
| C2 bigger-world | RegimeExploding | 1310 | 19 (+15) | 34/27/7 | 1068 | 0.5172 | 1.12/3.37 | 34/2.420/false |
| C3 longer-windows | RegimeExploding | 11830 | 24 (+20) | 36/34/2 | 16414461 | 0.6452 | 1.56/1.97 | 36/2.676/false |
| C4 unprotected | Expanding | 2390 | 20 (+17) | 37/29/8 | 502 | 0.5000 | 1.05/2.43 | 37/2.713/false |

Harness verdict lines: SUCCESS on C2, C3, C4. Pre-committed rule
(first success C2 → C3 → C4): **R5 loop candidate is C2
bigger-world** — the R5-C2-LOOP follow-up is licensed (pre-registered
in `docs/ere_r5_findings.md`, same session).

Prediction marks, corrected: C1 class ✓. C2 SUCCESS (was predicted
Saturating — the miss is the finding: match surface scales with
substrate under live labeled rules). C3 SUCCESS with 16.4M accepted —
the "longer windows compound nothing" reading is dead; it measured
dormant rules. C4 SURVIVES (Expanding, 40/40, 502 accepted) — the
"unprotected band non-viable, closed permanently" verdict is
REVERSED, owned plainly: guard-free matching ate regions in 19 epochs
only because every labeled region was already half-eaten by the
instrument. Do not cite the closure; the direction is open again
(but C2 won the order, so C4 waits).

Findings, revised:

1. **Firing volume is still seed-contingent — and now rule-contingent
   too.** C1 under 0xB001 accepted 7; the filed 4002 reference is now
   354. The lottery stands (report distributions, never singletons),
   at corrected levels.
2. **Protection is load-bearing, but the old C4 evidence for it is
   void.** C4 survives uncapped guard-free. What survives of the
   finding is weaker: quarantines persist everywhere (7–10 per cell),
   and C2/C3 panels show excl collapse with recur_excl ≈ 1.0 (0.999,
   0.999) — concentration without recurrence diversity, pump false.
3. **Dormancy was instrumental, not structural.** 4× substrate under
   live rules wakes everything (C2 explodes, C3 explodes at 16M). The
   "variation quality" lesson inverts: the kernel generates match
   surface fine — it was the mirror key eating it. The v10 lesson
   stands in its general form (fixing an instrument exposes the next
   layer); the exposed layer here is an EMERGED regime, not a deeper
   defect.
4. **Panel behaved throughout**: pump false on all cells, incl≫excl
   everywhere. The instrument is not what failed — twice over.
