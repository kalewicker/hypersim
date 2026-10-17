# G-GE2 findings — first panel light (slice 3, 2026-09-04)

First emission of the §5 blob panel from the R09-lineage driver
(`expr_r09.rs` `run_arm` step 6: per-epoch `observe_substrate` dual-scope
walk + `PanelEpoch` rows; site window accumulated run-to-date from
`EpochEvent.site`). Release, 40 epochs × 25 MaxParallel steps, seeds
0x90A1/0x90A3/0x90A5 — **two runs byte-identical** (arm table + all six
panel lines `diff`-clean).

## Arm table (unchanged harness readings)

| Arm | traj | max_n | shapes (post-burn-in +) | mut/inst/quar | paid/refused | accepted |
|---|---|---|---|---|---|---|
| A0 fee-zero | Saturating | 100 | 7 (+5) | 33/11/22 | 0/0 | 4002 |
| A1 main | Saturating | 74 | 5 (+2) | 31/10/21 | 5/0 | 7 |
| A2 excl-shadow | Saturating | 240 | 14 (+12) | 39/31/8 | — | 2 |

A0 control still ABSENT (SB-GE7 outer stands). Quarantine shares A0
0.667 / A1 0.677 — FM-GE3 watch flat. §10.3 decision unchanged: A1
novelty < A2 novelty.

Drift note (owned): A1 max_n reads 74 vs 72 in the pre-SB-GE3(b) run-2
figure. This is the first R09 since the kernel fix landed — rescued
slot-op draws install instead of quarantining, so a ±few-node drift in
the priced arm is the fix working, not nondeterminism (within this
state: byte-identical ×2). Decision figures (shapes, accepted,
paid/refused, audits 40/40, verdicts) are unmoved.

## Panel rows (final epoch; emission, not verdict)

| Arm | A_eff excl/incl | raw excl/incl | recur_excl | sing_excl | sites/H/pump | scored_gain | novel/event |
|---|---|---|---|---|---|---|---|
| A0 | 4.71 / 15.76 | 5 / 22 | 0.571 | 0.429 | 33 / 1.832 / false | 5 | 0.1724 |
| A1 | 2.00 / 12.74 | 2 / 17 | 0.000 | 1.000 | 31 / 1.857 / false | 2 | 0.0769 |
| A2 | 8.49 / 14.03 | 9 / 30 | 0.500 | 0.500 | 39 / 2.535 / false | 12 | 0.3529 |

Row-by-row first light:

- **A_eff + raw**: incl dwarfs excl on every arm (raw 17–30 vs 2–9) —
  the meta band dominates any single-scope alphabet. The M1 dual-scope
  discipline is load-bearing, not ritual: without it the panel would
  report region-band growth as substrate novelty.
- **Recurrent/singleton**: the priced arm (A1) collapses to 2 classes,
  zero recurrence, all-singleton — fees eat recurrence first. This is
  the FM-GE3 suppression curve in alphabet form; watch it in R5.
- **Step vs event**: scored novel/event ranks A2 (0.35) > A0 (0.17) >
  A1 (0.08) — same order as the shape verdict, so no binary_edge-style
  illusion in this harness; both abscissae agree here, which is itself
  a measurement (the instrument can show disagreement — slice 2 proved
  it — it just doesn't here).
- **Site pump**: 31–39 events clear the 20-event floor on all arms, but
  entropy stays high (1.8–2.5 nats) → no pump anywhere. Correct per the
  pre-commitment: volume without concentration is diffusion, and the
  verdict says so.
- **Edge row**: max_e tracks max_n on all arms (90/100, 57/74,
  238/240) — no B3 edge-only excursion here; the edge instrument is
  live and quiet, as it should be on non-excursion data.

## G-GE2 EXIT (declared)

Plan §5 demands each member firing on ≥1 synthetic or archived case.
Status, with evidence:

| Member | Firing proof (permanent) |
|---|---|
| A_eff verdict on effective, not raw | `verdicts_on_a_eff_not_raw_the_b6_trap` |
| Recurrent/singleton split | `treadmill_raw_equals_effective...`, `churn_recurs_and_treadmill_does_not` |
| Step/event divergence, verdict on per-event | `step_event_divergence_verdict_on_per_event` |
| Burn-in changes verdicts | `burn_in_transient_excluded_from_scored_gain` |
| Site pump (floor + concentration) | `site_pump_fires_on_concentration_not_volume`, `panel_row_marks_burn_in_and_reports_pump_state` |
| Dual scope separation | `walk_dual_scope_separates_meta_band` (real store) |
| Regime incl. Exploding abort | `expr_r09_mech` A0 RegimeExploding (cited) + `AbortGate` unit tests + runner wiring |
| Edge row decoupled from node row | `edge_panel_decouples_from_node_panel` (reused classifier) |

The panel is now emitted by a lineage driver and has read real
campaign-line data twice identically. What remains is the R5
panel-primary run — blocked on SB-GE7(b) loop conditions, which now has
its instrument waiting.
