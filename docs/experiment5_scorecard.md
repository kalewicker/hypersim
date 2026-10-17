# Experiment 5 — S6 gate report (scorecard)

**Plan deliverable (S6):** the influx sweep table + which exit criteria pass, with
the settled carrier ratio and its variance, and — the headline — whether
`non_trivial_replicator_count()` crossed **0 → >0**.

**Status: all seven S6.2 exit criteria MET.** Routes A (active substrate) and B
(`lifetime` aging) proved **unnecessary**: turnover was obtained on a *static*
substrate through the asymmetric `replication_threshold` lever.

This document is the settled report. The diagnostic journey (why influx alone and
`child_node_cost` alone both failed, and the symmetry argument that explains it) is
in [`experiment5_preregistration.md`](experiment5_preregistration.md) §7–§9. Here we
record only the final table, the scorecard, and the reproduction recipe.

---

## Reproduction recipe (this is what "fixed seed, reproducible" means now)

The committed regime lives entirely in [`configs/experiment5.toml`](../configs/experiment5.toml).
The two parameters the S6.1 sweeps discovered — the influx add-value and the bootstrap
`replication_threshold` — are now first-class `AlifeConfig` fields
(`influx_rate`, `bootstrap_replication_threshold`), not in-test patches, so the run is
reproducible through the production runner:

```
[alife]
enabled = true
seed_carrier_marker = true
initial_substrate_energy = 5.0
influx_rate = 0.34                      # sub-sustain: the turnover band
bootstrap_replication_threshold = 3.0   # seed the low-threshold attractor
```

`seed = 12345`, 6×6 pure crystal (36 sites), MaxParallel, 15 000 steps.

A third field, `child_node_cost` (default 10.0), is also exposed for completeness — a
symmetric scale over the birth/death economy that, by the §8 symmetry argument, does
**not** reopen turnover; it exists for stillbirth-floor experiments, not the Exp-5 regime,
so `experiment5.toml` leaves it at the default.

All three overrides are `Option`; when absent the runner loads the bare `alife_base.toml`
defaults (influx 2.0, threshold 15.0, child_node_cost 10.0), so a default alife run and
the no-`alife`-feature build are unchanged (exit-criterion 7).

Reproduced end-to-end through `runner::load_rules` + `runner::build_initial_store` in
`crates/hg-sim-core/tests/alife_s6_reproducible.rs` (no in-test rule mutation):

```
births=8485 deaths=8453  last-window b=1695/d=1698  final=33 peak=36  max_gen=500  ntr=2588
```

---

## S6.1 — influx sweep table

### Global influx sweep on the DEFAULT genome (bootstrap threshold 15), 36-site static crystal

| influx rate `r` | per-step source (36 sites) | regime | final carriers | peak |
|---|---|---|---|---|
| 0.10 – 0.48 | 0.30 – 1.44 | **EXTINCT** | 0 | 1 |
| 0.50 | 1.50 | FROZEN | 5 | 5 |
| 0.52 – 2.0 | 1.56 – 6.0 | FROZEN | 36 | 36 |

Sharp frozen↔extinct boundary at **r ≈ 0.49**, with **no turnover band at any
granularity**. The analytic sustain floor is `r* = M·H/(M+H) = 0.5·1/1.5 = 1/3 ≈ 0.333`,
but establishment requires `r ≈ 0.49` (a founder must climb 10→15 at ≫ 1/3 per step and
a neighbour must fill to child-cost). So the startup threshold (0.49) sits **above** the
sustain floor (0.333): anything that establishes is immortal, and the would-be turnover
window `(0.333, 0.49)` is squeezed shut. **Scarcity-driven mortality is insufficient on a
homogeneous substrate** (`alife_s6_influx_sweep.rs`).

`child_node_cost` does not help: it is a symmetric *scale* (birth energy = 0.8·c couples
to it), so it moves the birth and death thresholds in lockstep and leaves the 1.47× ratio
invariant — no turnover cell anywhere in a child_cost{2,5,10}×influx grid
(`alife_s6_child_cost_sweep.rs`).

### `replication_threshold` × influx sweep — the asymmetric lever

`replication_threshold` appears **only** in the birth condition (`energy ≥ threshold`),
never in death (`energy ≤ 0`). Lowering the bootstrap value lowers the startup floor
toward the sustain floor *without* touching the mortality ceiling — the degree of freedom
`child_node_cost` lacked. Bootstrap threshold × influx, 12 000 steps
(`alife_s6_threshold_sweep.rs`):

| bootstrap thr \ influx | 0.28 | 0.34 | 0.40 | 0.48 |
|---|---|---|---|---|
| **3** | **TURNOVER** | **TURNOVER** | **TURNOVER** | (established) |
| 6 | EXTINCT | EXTINCT | — | — |
| 15 | EXTINCT | EXTINCT | — | — |

Turnover appears at bootstrap `thr = 3` across the sub-sustain influx band. Turnover is
produced by the birth/death cycle's own desynchronization, not by pre-existing site
heterogeneity (frozen-run satiation spread max/min = 1.084 < the 1.47 desync budget —
`alife_s6_population_heterogeneity.rs`).

---

## S6.2 — exit-criteria scorecard (seed 12345, influx 0.34, bootstrap thr 3, 15 000 steps)

| # | criterion | result | verdict |
|---|---|---|---|
| 1 | Spontaneous replication from one ancestor | 8485 births from 1 bootstrap founder | ✓ |
| 2 | Bounded (no `Exploding`, no extinction in-window) | final 33, peak 36 | ✓ |
| 3 | Stable ratio, low late-window variance | ratio **0.938 ± 2.6e-3** (report value, not a threshold) | ✓ |
| 4 | Multi-generation lineage (clade persists > K gens) | max generation depth **500** | ✓ |
| 5 | Selection *acting* (distribution shift, not drift) | repl_threshold down in 6/6 seeds (directional SIGN), but magnitude inside the drift envelope | **⚠ DEGENERATE-ONLY** — met only via §1, and drift-dominated in magnitude (see validation below) |
| 6 | `non_trivial_replicator_count()` **0 → >0** | **2588** (0 on bare substrate by construction); crossing is a structural depth-K event | ✓ **← headline** |
| 7 | No substrate regressions (determinism, energy books, no-feature bit-identical) | ledger 0 divergence at every sweep point; cross-process boundary pinned; paired control clean | ✓ |

Asserted in `alife_experiment5_scorecard.rs` (regime discovered) and reproduced through
the production runner in `alife_s6_reproducible.rs` (config-driven).

> **The scorecard's criterion-5 ✓ did not survive validation and has been downgraded.**
> The single-seed shift "0.14 → −3.72" was measured with no drift null; §"CE-bridge
> validation" below shows that number is *inside* the neutral-drift band. Criterion 5 is
> met only in the pre-registered degenerate sense, and even that rests on sign-consistency,
> not magnitude. Criteria 1–4, 6, 7 stand.

### Caveat on criterion 5 (reported, not hidden)

Criterion 5 is met via the **§1 degenerate route**: replication is node-funded, so
*nothing opposes* a lower `replication_threshold`, and it collapses toward its floor. That
is a genuine directional distribution shift (not mere drift), but it is a self-limiting
degenerate route, not open-ended evolution — the other three traits are then under
weak/no selection. This was pre-committed in the preregistration §1 (option 1: accept and
report as a finding). Non-degenerate selection would require §1 option 2 (a parent-side
replication cost), which reopens the node-funded decision and is a *separate* direction,
not part of the Experiment-5 gate.

### Caveat on initial conditions

`replication_threshold` is a jittered genome trait, not a physics constant. A high
bootstrap threshold cannot *enter* the turnover basin at sub-sustain influx (it goes
extinct before establishing); but once inside, selection *drives* the threshold down, so
the low-threshold turnover regime is the self-reinforcing attractor. `influx_rate` is
physics; `bootstrap_replication_threshold` is an initial condition that seeds the basin.

---

## S6.3 — movement decision

**Movement is NOT needed for Experiment 5.** A stationary carrier population satisfies all
seven criteria; the plan's condition for adding a `move` rule ("only if stationarity
provably prevents a criterion") is not met.

Note the distinction from earlier stages: S4 added **birth-dispersal** (a replicating
carrier colonises an adjacent *empty* site) because same-site confinement provably
hard-failed criteria 4/5/6 — that was the pre-registered S6.3 escape hatch firing for
*dispersal*. **Motility** (a carrier rewiring its own occupancy edge to relocate an
existing individual) remains unimplemented and unneeded, and is deferred to a later phase.

---

## CE-bridge validation (2026-07-15) — the criteria S7 will trust, tested before S7

Before Experiment 6 coarse-grains over criteria 5 and 6, both were validated with
dedicated tests. This is where the scorecard's clean ✓ row was pressure-tested; one
criterion (5) did not survive and is downgraded above.

### Criterion 5 — selection vs. a matched drift null (`alife_criterion5_selection_vs_drift.rs`)

The genome has four traits but only **two are read by any rule**: `replication_threshold`
(read by `replicate`) and `satiation` (read by `harvest`). `harvest_rate` and
`maintenance_cost` are set, copied, and jittered identically but read by nothing (harvest/
metabolise use fixed quanta) — a **perfectly matched, in-run pure-drift null**: same jitter
(0.5), same birth/death lineage sampling, zero fitness coupling.

Result over 10 seeds (10 000 steps each):

| trait | coupling | mean Δ | |Δ| / drift env | reproducible sign |
|---|---|---|---|---|
| harvest_rate | inert (null) | ~0 | 1.0 (defines env ≈ 4.5) | sign-mixed |
| maintenance_cost | inert (null) | ~0 | 1.0 | sign-mixed |
| **replication_threshold** | coupled (FREE) | −5.45 | **1.22×** | **6/6 down** |
| satiation | coupled (trade-off) | +4.28 | 1.69× | 5/6 up (weak) |

**Finding:** the population trait-mean of a *non-selected* trait random-walks by ~±4.5 over
deep near-clonal lineages, so a single-seed "the trait shifted by a few units" claim is
uninformative. `replication_threshold` moves **consistently downward (6/6 seeds)** — a real
directional SIGN bias, distinguishable from the sign-mixed null — but its **magnitude sits
inside the drift envelope**, and `satiation` (the trade-off trait) shows no clean selection
beyond drift. So criterion 5 is met **only** via the pre-registered §1 free/degenerate
collapse (node-funded ⟹ no counterforce ⟹ runaway to floor), now *earned* against a matched
null rather than asserted from one seed. The scorecard's "0.14 → −3.72" is an effect size
inside the drift band, not a meaningful magnitude. **No non-degenerate, trade-off-resolving
selection is present.**

### Criterion 6 — NonTrivial robustness (`alife_criterion6_nontrivial_robustness.rs`)

- **Negative control (firewall):** hand-built trivial copiers — a one-off birth, and a
  *prolific-but-shallow* founder with 50 children none of whom reproduce — yield
  `non_trivial_replicator_count = 0` **even though every node shares the carrier label and
  genome**. Extending one branch to depth ≥ 2 flips it > 0. The count is **depth-driven, not
  label-driven** — it cannot be a WL-class/label artifact (it consults only parent/child
  topology, never `motif_class`).
- **Structural crossing:** in a real run the 0 → >0 crossing occurs exactly at the window
  where a lineage first reaches depth ≥ `NON_TRIVIAL_MIN_GENERATIONS` (2); no window before
  it had such a lineage. The crossing is a genuine **sustained-self-copying event**, not a
  first-carrier-gets-a-class coincidence.
- **Caveat (unchanged):** this is the ALife *lineage-depth* instrument (sustained self-copying
  to depth K), **not** the causal `ReplicatorKind::NonTrivial` (which is still never assigned).
  The criterion-6 headline uses the ALife measure; it validates that sustained replicators
  exist, which is what S7's partition needs — it does not certify replicator-taxonomy
  non-triviality.

### Criterion 7 — reproducibility spine (hard gate, outcome-independent)

- **Energy books balance at EVERY sweep point** (`alife_criterion7_energy_across_sweep.rs`):
  max ledger divergence 1e-10 … 1e-9 across influx 0.28 → 1.0, turnover and frozen regimes
  alike. The death-residual term (flagged at S3 as load-bearing) is tracked exactly and is
  empirically **0** — carriers always die at exactly 0 energy (0.5 metabolise quantum + `die`
  priority 3 ⟹ no negative overshoot), so the residual source never actually fires here.
- **Cross-process boundary reproducibility** (`alife_criterion7_sweep_reproducible.rs`): a
  pinned sweep signature (`0x9845f145815ae1ad`, recorded in a prior process) is re-verified
  in a fresh process. The sweep's determinism rests on deterministic **rewriting** (the
  regime is read from engine events; the WL/causal path is off), so this is the right spine;
  WL cross-process determinism (proven in `wl_cross_process`) is the *separate* spine for
  Experiment-6 CE.
- **Paired bare-substrate control at every sweep point**
  (`alife_criterion7_paired_control_sweep.rs`): the substrate subgraph is **(36 nodes, 85
  edges) — identical to the bare control — at all influx rates**; carriers only add nodes on
  the full graph. No back-reaction ⟹ the S7 substrate partition is clean.
- **No-feature / flag-off bit-identical:** `property_effects_regression` on both axes (prior).

### Band reality (`alife_band_reality.rs`)

- **Stationary by trend, not just variance:** late-window OLS slope −3.8e-6 carriers/step ⟹
  total systematic drift 0.1 % of the mean; a slow climb would fail this while passing the
  variance test. The band is genuinely stationary (ratio 0.938, var 2.65).
- **Seed-fragile:** establishment across the band is **71 % / 57 % / 57 %** (influx
  0.28 / 0.34 / 0.40, 7 seeds); seeds 1 and 7 never clear it. This is a **fragile turnover
  regime, not a robust carrying capacity** — the band edges are the synchronized-death
  discontinuity barely avoided, so which seeds clear it is stochastic. An S7 run must fix a
  known-establishing seed (12345 establishes) or report the establishment rate.

---

## What actually unblocked turnover (the fork) — and the S7 verdict

**Turnover was unblocked by the asymmetric `replication_threshold` lever on a STATIC
substrate** — *not* route A (churning substrate) and *not* route B (`lifetime` aging). The
threshold-vs-satiation artifact hypothesis was ruled out earlier at S5 soundness
(`alife_s5_soundness.rs`: repl_threshold mean 14.62 < satiation 20.04, 0/36 sterile). So the
substrate is static and the carrier/substrate partition is over the **same micro state
space**, exactly as S7 specifies — no coarse-graining assumptions change.

**S7 gating verdict:**
- **Partition + onset boundary (criterion 6): SOUND** → the CE setup S7 needs is clean and
  the replication-onset boundary is a real structural event.
- **Reproducibility + accounting (criterion 7): SOUND** across the whole sweep and across
  processes.
- **Selection (criterion 5): DEGENERATE-ONLY / drift-dominated** — a legitimate *negative*
  finding, not a blocker for CE measurement (CE needs the partition and the onset boundary,
  both clean; it does not need non-degenerate selection). But it bounds the biological claim:
  Experiment 5 achieved a **stable, spontaneously-replicating, turning-over population**
  (criteria 1–4, 6, 7) **without** non-degenerate open-ended selection.
- **Band: fragile** — S7 must pin an establishing seed or report establishment rate.

**Net: S6 unblocks S7 for the CE measurement**, with two documented caveats carried forward
(degenerate selection; seed-fragile band). Restate **Finding F1** in the S7 doc before
touching it: CE is biased *downward* on open registries, so CE > 1 clears a conservative bar
and CE < 1 falsifies nothing — **do not tune to produce CE > 1** (the same forbidden move as
tuning influx to make the band appear).

## What S6 unblocks (S7)

With a stable population and a non-zero `non_trivial_replicator_count`, Experiment 6 (CE
across the replication-onset boundary) is measurable: the functional coarse-graining now
has its real carrier/substrate partition over the same micro state space. Finding F1 still
stands — CE is biased *downward* on open registries, so a CE > 1 result clears a
conservative bar; do not tune to produce CE > 1.
