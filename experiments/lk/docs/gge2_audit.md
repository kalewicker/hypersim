# G-GE2 audit — blob-panel status vs GE plan §5 (2026-09-04)

Pre-registration (§5) binds: for every R5+ run the 8-row panel is
computed, reported, and treated as PRIMARY outcome. G-GE2 exit =
engagement proof per member (each demonstrated firing on ≥1 synthetic
or archived case). This doc maps each row to its implementation status,
records pre-commitments made while no run numbers exist, and slices
what remains.

## Row status

| # | Instrument (§5) | Status | Evidence |
|---|---|---|---|
| 1 | A_eff trajectory + raw \|A\| | INSTRUMENT LANDED, unwired | `hg-meta/src/gge2.rs`: `S1` radius-1 keys, `AlphabetHist::{a_eff,raw}` + 3 engagement tests (treadmill archetype, churn archetype, B6-trap verdict-on-A_eff). Driver walk + per-epoch emission = slice 2 |
| 2 | Recurrent-symbol fraction & singleton share | INSTRUMENT LANDED, unwired | same module (`recurrent_fraction`, `singleton_share`); treadmill 0/1 vs churn 1/0 pinned. Post-burn-in discipline = driver feeds post-cut histograms (`burn_in_cut`) |
| 3 | Novelty vs STEP and vs EVENT | WIRED (R09 driver) | `expr_r09.rs` reports RCF+SHAPE curves vs epoch and vs kernel event; burn-in first 10% unscored. Engagement proof (event-rate illusion case) = slice 2 |
| 4 | Burn-in window | WIRED as discipline + helper | R09 unscored first 4/40 epochs; `gge2::burn_in_cut` (total/10) pinned. Firing proof (transient-inflation case) = slice 2 |
| 5 | `meta_rewrite_site_entropy` | INSTRUMENT LANDED, unwired | `gge2::{site_entropy, pump_verdict}` + pump-vs-diffuse test. Site input (per-root install/mutate counts) needs epoch-event granularity = slice 2 |
| 6 | Dual meta-excluded/meta-included | DISCIPLINE, unwired in hg-meta | Precedent `expr_r10.rs` (`label < META_BASE` filter); M1 rule recorded below. Slice 2 wires dual histograms into the panel rows |
| 7 | Regime classifier + Exploding abort | BOTH LANDED + WIRED | `gge1::classify_trajectory` reused in R09; `AbortGate` unit-tested (`aborts_at_exactly_window_k`, `non_exploding_resets_the_streak`) and wired in runner campaign loops (`runner.rs:1214,1756,1865`). Slice 2 cites, no new code (reuse per §5) |
| 8 | Edge-count alongside node-count | WIRED (R09 driver) | per-step node+edge trajectories recorded. Firing proof (edge-only excursion, B3 shape) = slice 2 |

## Pre-commitments (made here, before any panel numbers exist)

- **PUMP verdict (M2):** `PUMP_EVENT_FLOOR = 20` events, `PUMP_ENTROPY_CEIL
  = 0.5` nats. Below 20 events entropy is noise; at ≤0.5 the mass sits on
  essentially one site (90/10 already reads 0.33). `pump_verdict =
  events ≥ floor && entropy ≤ ceil`, interpretation verbatim from ERE
  plan M2 (pump = metering finding, not encoding).
- **S1 radius:** 1 (§5's "pre-registered radius" read literally).
  Neighbor = hyperedge co-member (slice-2 walk decision; if the walk
  uses graph-adjacency instead, the audit is amended with the reason).
- **Site key substitution (owned deviation):** ERE spec phrases site
  entropy over edit-site WL class; G-GE2 implements over region-root
  identity. Rationale: roots ARE the meta-level sites, and WL-classing
  them would fold a second instrument's failure modes into the pump
  detector. Revisit iff a pump hides across roots sharing one WL class
  (no such case observed).
- **M1 dual-scope rule for the hg-meta panel:** substrate-count metrics
  (trajectories, A_eff family) computed over meta-excluded AND
  meta-included sets, both reported; rule-shape novelty (RCF) reports
  single-scope with reason (rules are not substrate nodes —
  "unmeasured with a stated reason" per M1 verbatim).

## Slice 2 (LANDED 2026-09-04): engagement proofs + row inputs

- `EpochEvent.site: Option<NodeId>` (`epoch.rs`): every applied edit
  names its parent root (counts quarantined too — the edit happened at
  that site); `None` on empty kernel / Nothing draws. FEE_ZERO path
  forwards it (A0-equivalence tuple extended). Pinned by
  `transition_reports_the_mutation_site` (site live post-install;
  non-vacuous) — row 5's driver input now exists.
- `gge2::{observe_substrate, SubstratePanel, PanelEpoch, novel_per_event,
  scored_gain}`: one store walk → dual counts + both S1 histograms (the
  excluded scope ignores meta nodes entirely, M1 at the key level);
  one-row assembly (burn-in flag, dual alphabet rows, site pump state);
  per-event verdict axis; scored-gain math matching the R09 driver.
- Engagement proofs (all permanent tests, all GREEN):
  rows 1–2 `verdicts_on_a_eff_not_raw_the_b6_trap`,
  `treadmill_raw_equals_effective_with_no_recurrence`,
  `churn_recurs_and_treadmill_does_not`; row 3
  `step_event_divergence_verdict_on_per_event` (per-step 10 vs
  per-event 0.01); row 4 `burn_in_transient_excluded_from_scored_gain`
  (whole-run 10 vs scored 2); row 5 `site_pump_fires_...` +
  `panel_row_..._pump_state` (floor + concentrated verdict); rows 6+8
  `walk_dual_scope_separates_meta_band` (real store, 4/3 + 2/1, border
  key differs by scope); row 8 `edge_panel_decouples_from_node_panel`
  (reused `classify_trajectory`: node Balanced, edge non-quiet); row 7
  cited — `expr_r09_mech` A0 RegimeExploding + `AbortGate` unit tests +
  runner wiring (`runner.rs:1214,1756,1865`), reuse per §5, no new code.

## Slice 3 (LANDED 2026-09-04): R09-lineage emission + G-GE2 exit

- `run_arm` step 6 (`expr_r09.rs`): per-epoch `observe_substrate` walk
  on the post-install store + `PanelEpoch::from_observed`; site window
  accumulated run-to-date from `EpochEvent.site` (granularity decision:
  one site max per epoch makes per-epoch entropy degenerate, so the
  driver reports the cumulative window — the only windowing that can
  reach the 20-event floor in 40 epochs). Rows reported, never gated
  (`panel.len() == survived_epochs` asserted).
- First light: release ×2 byte-identical (arm + panel lines
  `diff`-clean). A0 4.71/15.76, A1 2.00/12.74 (zero recurrence —
  FM-GE3 alphabet form), A2 8.49/14.03; pump false everywhere (volume
  without concentration); incl≫excl on all arms (M1 load-bearing).
  A1 max_n 74 vs pre-SB-GE3(b) 72 — first R09 since the kernel fix,
  owned as rescued-draw drift; decision figures unmoved.
- G-GE2 EXIT declared: every §5 member has a permanent firing proof
  (table in `docs/gge2_findings.md`). R5 panel-primary next, blocked
  on SB-GE7(b) — the instrument is waiting.

Original slice-3 items, disposition: (1) done — step 6 above; (2) done —
`EpochEvent.site` (slice 2) + run-to-date accumulation, no signature
breaks needed; (3) done — proofs landed in slice 2, table in findings
(B3 cited via the reused-classifier decoupling test on synthetic
series, not a fan-out fixture run — owned substitution); (4) done —
exit declared, with the correction that `docs/gge2_findings.md`
carries the first panel light from the R09 lineage; the R5
panel-primary reading has since landed (see below).

## R5 panel-primary (LANDED 2026-09-04)

`expr_r09::r5_panel_primary`, reference loop ×150 epochs (one R07
quartile), full A0/A1/A2 + panel, ×2 byte-identical. A0 23 / A1 25 /
A2 16 shapes vs null 61/quartile; A0/A1 quarters flat (linear at ~40%
of null), A2 declining 8/6/1/1 (MID kernel exhausts — installs repeat
shapes). FM-GE3 does NOT fire (yield ratio 1.09, quarantine 0.53 vs
0.46, identical excl-alphabet rows). §10.3 decision HOLDS at horizon
(25 vs 16). No §2 excursions; pump false; A1 audits 150/150. Full
record: `docs/ere_r5_findings.md`.
