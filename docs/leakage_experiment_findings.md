# Locality & Leakage — Experiment Findings (LK track)

## v5 round (2026-07-17): R1 (EXP-L15, relaxation)

Plan: hypersim_leakage_plan_v5.md.

### EXP-L15 — basin dynamics: exact exponential relaxation, lattice bit-restored

IC: the archived post-quench pump state (`runs/exp_l13/q75t25_pump_s*`,
six throats of L₀ = 785 links radiating from the seed, interiors flat at
q = 75 → transport and subdivide dead by energetics; fuse-limited
relaxation by construction). Run at the ARCHIVED params (q75/τ25, cap ∞,
subdivide_first), IC provenance copied into the run manifest
(`ic_provenance`; audit baseline = loaded energy sum, counters restart
at 0). Runs: `runs/exp_l15/`; driver `tests/exp_l15.rs`.

**Arm (a), quiescent (seeds 42/43/44 — trajectories IDENTICAL, the pump
morphology is seed-independent on the vertex-transitive torus):**

- **Decay law: exact exponential, ratio 2/3 per step — not 1/2.** The
  plan's "roughly alternate nodes fuse per step" assumed EDGE-disjoint
  bindings, but MaxParallel conflict resolution is **NODE-disjoint**
  (`hg-engine/src/conflict.rs`: a fuse binding claims {x, w, y}), so the
  chain MIS takes every THIRD interior node: L(t) = L₀·(2/3)^t exactly
  (523, 349, 233, … each step exactly ⌈2L/3⌉-ish; trace archived in the
  test output and snapshots). Fitted λ = 0.393 ≈ ln(3/2); the
  pre-registered acceptance (fitted decay within a factor-2 band of
  ln 2) **PASSES**. Third instance of the F-LK13 error class
  (concurrency-semantics assumption not checked against MaxParallel at
  design time — v5 §1.1 item 2 now has a worked example on the removal
  side). **The family's decay constant: λ = ln(3/2) ≈ 0.405/step.**
- **Terminal tail is serialized by the shared anchor:** all six throats
  anchor on the seed node, so node-disjointness admits only ONE terminal
  fuse per step — after the exponential leg the census walks 6→5→…→0
  one chain per step. Mechanism-consistent, recorded.
- **Restoration endpoint: RESTORED.** Whole-graph structural WL hash of
  the terminal state equals the clean 32² lattice fixture (1024 nodes /
  3072 edges; label-invariant, property-blind — the geometry claim).
  The lattice is structurally bit-restored; every joule of throat
  energy (352,275 = 4695 interiors × 75) reabsorbed through the fused
  ledger; audit residual exactly 0.0 at every emission on the loaded
  baseline.
- **Census follows:** wormhole_count_est monotone → 0 after the longest
  chain drops below R_max/2 = 6 (the corrected 2L+2 threshold). PASS.

**Arm (b), concurrent (same IC + fresh E₀ = 4e5 at the corner):** the
fresh budget ran its full era (n_sub = 4705 = E₀/(q+c) exactly) under
suppression-at-birth — first snapshot already shows only 6 stub chains
of length ≤ 2 (the EXP-L14 phenomenology, now on a populated
background); wormhole census 0 at every emission; churn 0; final state
fully reabsorbed (0 chains). Fused ledger closed at 704,550 = 2 ×
352,275 exactly — old throats + new era, both fully reabsorbed. Removal
of standing structure and active production do not interact: the old
population decays at its own λ while new production is suppressed at
birth, and the audit stays exact throughout on the combined baseline.

**Verdict: the basin demonstration is complete.** v4 showed suppression
at birth (the degenerate decay limit); EXP-L15 shows a POPULATED
far-from-attractor state relaxing to the clean lattice with an exact,
mechanism-derived decay constant and a bit-restored endpoint. R5 may
now cite "locality is an attractor" with both halves in hand (per the
pre-committed row), with the balance question (R3) supplying the
boundary of the claim.

### EXP-L16 — balance sweep: f_cap INERT at the pinned (75,25); instrument-never-engaged

Grid as pre-registered (v5 §4): transport_first × (75,25) × f_cap ∈
{10,25,40,60,74} × E₀ ∈ {1e4,1e5} × 5 seeds + fuse-disabled and
f_cap-disabled controls + the §3 pump-regime cells. Runs:
`runs/exp_l16/`; driver `tests/exp_l16.rs`; audit exact everywhere.

- **Every throttled cell is IDENTICAL to the fuse-disabled control** —
  at both budgets, at every f_cap up to 74: e_fused = 0, churn = 0,
  f_cap guard rejects = evals (1.97e6 at E1e4; 9.81e7 at E1e5). No
  gen ≥ 1 wedge node ever held energy < 74 at evaluation time — Q2's
  row (c) diagnosis (interior freezing = PROFILE FLATNESS at (75,25))
  confirmed at both budgets, now with guard-counter receipts.
- **The pinned cell cannot answer the balance question.** Per the
  EXP-L9 lesson this is recorded as INSTRUMENT-NEVER-ENGAGED, not as
  "monotone growth under throttling": the aging timer t_age ≈
  (q − f_cap)/τ never matures because nothing drains. The scaling test
  is vacuous (zero variance across f_cap; r undefined). The plan's §3
  "live corner" citation names Q1/Q2 drainage — but the demonstrated
  drainage cell is Q2 row (b), (q,τ) = (50,15); (75,25) is row (c).
  Grid-provenance slip, owned; the sweep design is rerun unchanged at
  (50,15) as a DOCUMENTED EXTENSION
  (`exp_l16_live_corner_extension_q50t15`).
- Controls behaved: f_cap-disabled reproduces v4 suppression at birth
  (census 0 at every window, churn 803 at E1e5); fuse-disabled shows
  the frozen post-quench census (STEADY verdicts here are FROZEN
  structure, not turnover — sparse strata ≤ 22 windows at E1e5, far
  below the ≥ 200 authorization floor, so EXP-L7R stays unauthorized
  by these cells regardless).
- **Pump-regime prediction (pre-registered in §3): CONFIRMED as
  designed** — flat-at-q never drains, fuse goes dead (evals 454,194,
  rejects = evals, ledger empty), pure growth returns.

**Mechanism (energy_probe.rs on the persisted snapshots):** interior
(gen>0, degree-2) node energy is a near-SPIKE at a PLATEAU *above* q, not
a spread reaching below it. At (75,25): flat at 75. At (50,15): min 50
(freshly born), but p10 = p50 = p90 = 65 (conducted plateau), max 80 —
i.e. a fresh child at q is conducted UP to 65, never down toward f_cap.
The aging-timer premise ("untouchable until it drains via transport") is
INVERTED: downhill-only metered transport with a 2τ dead zone has no
sub-q interior for the guard's admitting region (energy < f_cap < q) to
intersect. Hence every f_cap < plateau is bit-identical to fuse-disabled.

### EXP-L16 extension + straddle — bistability, threshold at the conduction plateau

The §3 "live corner" citation names Q1/Q2 drainage, whose demonstrated
cell is Q2 row (b), (q,τ) = (50,15), not the pinned (75,25) = row (c).
Documented extension (`exp_l16_live_corner_extension_q50t15`): the SAME
sweep design at (50,15), f_cap ∈ {10,25,40,49} < q, E₀=1e5, 3 seeds +
controls. Result: **all four throttled cells bit-identical to the
fuse-disabled control** (rejects = evals = 1.024e8, e_fused 0) — the
inertness is not specific to the profile-flat cell; it is the transport
model.

Straddle (`exp_l16_bistability_straddle_q50t15`, plateau ≈ 65):

| f_cap | side | verdict | Σ E_fused | churn | steady census |
|------:|------|---------|----------:|------:|--------------:|
| 60 | below | STEADY (frozen) | 0 | 0 | 1206 |
| 66 | above | EMPTY (suppressed) | 80,275 | 958 | 0 |
| 80 | above | EMPTY (suppressed) | 80,275 | 958 | 0 |
| 100 | above | EMPTY (suppressed) | 80,275 | 958 | 0 |

**BISTABILITY (pre-committed §4 row): CONFIRMED.** The transition is a
jump at the conduction plateau — below it fuse never fires (= pure
growth), one step above it the whole endowment routes through the ledger
(= v4 suppression), and the three above-plateau columns are bit-identical
to the unit. No f_cap yields an intermediate standing census: an energy
gate cannot be a rate control here because the interior energy it reads
is a spike with no spread to select a fraction of. **The attractor claim
gains its boundary: locality holds in the removal-dominated limit,
dissolves in the production-dominated limit, with NO interior steady
state between.** A continuous balance knob (annealed f_cap, or an
age-gated rather than energy-gated removal rule) is a v6 question, not
improvised. R5 thesis text takes the bistability branch.

**EXP-L7R: NOT AUTHORIZED — explicitly waits.** No cell produced a
steady-turnover band; every throttled cell was frozen or suppressed. The
longest consecutive-Sparse run (117 windows at E1e4) is both below the
≥200 floor and frozen structure, not turnover. Per §6 / §1.4, CD waits
for a family that produces a genuine steady state; it is not run on a
frozen or absent stratum.

## v4 round (2026-07-16): Q0–Q2, F1–F3 (fuse / removal dynamics)

Plan: hypersim_leakage_plan_v4.md.

### Q0 — retrodiction check: FAIL on the pre-registered statistic, mechanism CONFIRMED on diagnosis

The pre-registered fit (slope 1.0 ± 0.1, x-intercept 60 ± 20 events) ran
on the persisted P0 cells and FAILED (window cadence 200 cannot resolve
a 60-event lag; intercepts land negative). The pre-registered fallback
(fine cadence) resolved it, and per the Q0 pre-commitment interpretation
paused for diagnosis. Diagnosis (`runs/q0_retrodiction/`, cadence-25
run with per-snapshot chain census):

| events | max chain L | wormhole_count_est |
|---|---|---|
| 25 | 5 | **0** |
| 50 | 9 | 56 |
| 75 | 13 | 81 |
| 100 | 17 | 106 |

The plan's arithmetic was wrong twice; the mechanism is right. (1)
Interior-link detour is **2L + 2, not L + 2**: the seed's original bulk
edges are themselves consumed into chains, so the return path must
descend a SECOND chain (n + m + 2 with m ≈ n under round-robin). Flip
at 2L+2 > R_max=12 → L ≥ 6 (~36 events), exactly between the 25-event
zero and the 50-event jump. (2) The backlog cancels the intercept: all
~L interior links flip together, restoring count ≈ events — the
correct retrodiction is a zero plateau followed by a slope-1 line
through the origin, which is what the data shows (slope 1.02–1.04).
**Reading confirmed quantitatively; Q1/Q2 interpretation unblocked.**
Meta-lesson recorded: pre-registered STATISTICS need the same derivation
review as instruments (F-LK11's class extends to predictions).

## v3 probe round (2026-07-15): P0 re-audit, EXP-L9/L9b, EXP-L11, P4

Plan: hypersim_leakage_plan_v3.md. Normative changes landed: G4-new
(phase-classifier gate; cut_exponent in no gate), CD tuple protocol
(leakage_closure_spec.md), abort default k = 2, generation property
(key 6, property_registry.md), classifier-calibration provenance in
every manifest. Harness note: the v2 sweep/arm outputs had been left in
an ephemeral directory and were partially lost; surviving EXP-L2 arms
are archived under `runs/expL_v2_archive/`, and every v3 harness
persists via `HYPERSIM_SWEEP_OUT`.

### P0 — gate re-audit under G4-new

Full table in fixture_calibration.md. Headline: G4-new fails on exactly
the three long-era cells (E₀ = 4e5; sparse frac 0.33–0.43) and passes
the six short-era cells at 1.00 — the dense crossing is
**era-cumulative, not parameter-local**; any science-floor era crosses.
G4-old (cut) reads 1.00 on all nine cells including the three failures
(F-LK6 reconfirmed on fresh data). Shipped status re-labelled: *bounded
sustained structural dynamics (G4-new: FAIL at science-floor eras)*.

### EXP-L9 — the pre-registered generation cap is structurally INERT [Measured]

Sweep ran as specified (g_max ∈ {1,2,4,8,∞} × 5 seeds, 32²/E₀=4e5
reference cell, `runs/exp_l9/`). **Every g_max cell is identical to its
same-seed ∞-control** — phases, wormhole rate (~1.0/event), chain
census, node counts; G5 spot-check exact. Cause, proven by the census:
the extruded chains are 6 per run (one per lattice direction off the
seed) and every chain node is generation 1 — under downhill-only
transport only the gen-0 seed ever re-reaches θ, so a guard on the
SUBDIVIDER's generation never binds. By the frozen thresholds this
reads "persistence", but the persistence row's premise ("rate unchanged
despite capped chains") is false — nothing was capped. **Recorded as
instrument-never-engaged, NOT as a mechanism verdict; the
channelization hypothesis is not retired on this evidence.** (The dual
of hazard 1: a never-binding guard fakes persistence the way a
missing-key guard would fake collapse.)

### EXP-L9b — depth cap (EXPLORATORY corrected instrument): PARTIAL

Corrected discriminator: guard on y, inheritance from y
(rules/lk_pair_depth_capped.toml) — generation counts filament depth.
Thresholds frozen in the harness before running (collapse < 5% of
control wormhole count with chains ≤ g_max; persistence ≥ 50%; else
partial). Results (`runs/exp_l9b/`, 5 seeds, audit exact everywhere):

| g_max | era subs | final wormholes vs control | chains (n / len) | verdict |
|---|---|---|---|---|
| 1 | 2333 | 0.31–0.38 | 6 / 394 | **partial** |
| 2 | 3506 | 0.45–0.53 | 6 / 590 | persistence (borderline) |
| 4 | 4674 | 0.65–0.72 | 6 / 788 | persistence |
| 8 | 5608 | 0.81–0.89 | 6 / 944 | persistence |
| ∞ | 6666 | 1.00 | 6 / 1111 | control |

**The load-bearing observation:** at every cap, chain length =
era_subs / 6 exactly — six chains, one link per subdivision, and every
link is a wormhole edge. **Subdivision and wormhole production are the
same event in this rule family**; a cap can only throttle the event
rate (each blocked direction reopens after ~2 transport deliveries
re-concentrate a mouth to θ — the rate-vs-g_max shape is smooth and
saturating, not thresholded), never the per-event yield. "Collapse"
was structurally unavailable: geometry-dissolving growth is not a side
effect of this dynamics, it IS the dynamics. Corollary: capped arms
stay sparse far longer (g_max = 1: zero dense windows in 4/5 seeds) —
the cap delays the era-cumulative crossing by slowing wormhole
accumulation, which independently corroborates P0's reading.

**P1 outcome for D1: PARTIAL** (per the corrected instrument;
pre-registered EXP-L9 recorded as invalid-instrument alongside).

### EXP-L11 — CD tuple hardening: the v2 headline number does not survive

p_peak arm × 5 seeds + p = 0 control, rerun under the tuple estimator
(`runs/exp_l11/`, 260 windows each):

| seed | CD_raw | CD_clock |
|---|---|---|
| 42 | +15.99 | +5.81 |
| 43 | −1.09 | −7.88 |
| 44 | +2.49 | −1.65 |
| 45 | +13.11 | +6.84 |
| 46 | +14.75 | +7.14 |

Seed spread ≈ 15 bits (CD_clock max−min) vs mean ≈ +2.1: **signal <
spread, so no CD claim stands** (the §5.2 pre-commitment). Roughly 60%
of seed 42's CD_raw was clock (15.99 → 5.81) — the F-LK8 confound was
real. The v2 finding "CD = +15.99 bits, first valid live non-closure"
is RESTATED as: *CD at 260 windows on this trajectory is
indistinguishable from estimator variance once era progress is
controlled; CD_raw is retired from findings per the tuple protocol.*
Phase-stratified (sparse-only) strata: 0–38 usable windows → Unknown
everywhere, as pre-committed (and the certainty gate visibly earns its
keep: the n=5 stratum "fits" produce million-bit garbage that the
Unknown label correctly quarantines). Control-arm negative-CD
diagnosis (blocked-CV artifact under nonstationarity, deepens under
the clock baseline): leakage_closure_spec.md.

### EXP-L10 — designed encounter: far_nucleus is a placement artifact; conduit question settled by direct probe

Arms {at_tracked_mouth, matched_offset} × {uncapped, depth-cap g1} × 5
seeds (`runs/exp_l10/`, EXP-L2 geometry, audit exact everywhere; the
capped arm uses the EXP-L9b instrument since the pre-registered cap is
inert — the "only interpretable if collapse/partial" condition is met).

- **far_nucleus fired in every at-mouth arm** (t = 100–200, the first
  tracked windows) and in NO offset arm — **and failed the hazard-4
  priority swap in all 10 cases → tie-break artifact, recorded; V3
  remains untested by this channel** (the pre-committed third row).
  Mechanism of the artifact: with the seed placed ON a mouth, the
  seed's own extrusions are "fresh growth adjacent to a tracked
  endpoint" from window 1 — the probe detects the placement itself.
- `mouth_energy_max` reads ~E₀ in mouth arms (the seed IS a mouth) —
  instrument note recorded: the column answers "did the encounter
  happen" (yes, by construction), not "did the handle conduct".
- `t_ambient_front_at_far`: 2300–2500 uncapped, 4600–5900 at g1 — the
  depth cap slows the front by ~2× at matched step counts, consistent
  with EXP-L9b's rate throttling. (The "front" here is chain extrusion
  reaching far censored distances, not lattice-surface traversal.)
- **Direct conduit probe** (rerun of uncapped×mouth×s42 with snapshots;
  `runs/exp_l10/conduit_probe/`): after 26k steps the seed's own handle
  NEVER conducted — far mouth E = 0.0; 34/35 shortcuts have both mouths
  at 0.0 (the 1 exception is a single 25-unit ambient transfer from an
  adjacent fresh chain node, not through-handle transport). Diagnosis
  per hazard 3, and it is NOT the dead-zone offset: a super-θ node
  always wins its per-node conflict against transport (subdivide is
  priority 1, conflicts serialize per node), so the seed executes ZERO
  transport events for the whole era, and post-quench it sits below
  every gradient. **Non-conduction is attributed to rule/priority
  structure, not handle geometry — it does not count against V3. V3
  exits this round untested at the physics level; testable only under
  a rule shape where a super-θ node can transport (or a lower-θ
  co-dynamic).**

### P4 — residual-energy autopsy: UNIFORM (the pre-committed nuance)

Offline over EXP-L9b final snapshots (dinf/d1/d4, seed 42): mean node
energy is flat along chains in every root→tip decile (50.0 exactly on
the ∞-control; 40.0 at g_max=1; 47.5 at g_max=4) and flat across
generations. NOT tip-concentrated. The recorded nuance: **each link is
born at q = 50 and immediately dead-zones against both neighbors
(|Δ| < 2τ) — filaments are energetically frozen at birth.** "Forward
extrusion" is extrusion of structure at the seed boundary; energy never
travels along a chain at all. This also strengthens the fuse/reabsorb
step-3 design target: reabsorbable energy sits uniformly along
filament, q per link, exactly as the audit arithmetic says.

### D1 — step-3 decision (v3 §7, convened 2026-07-15 after P1 + P2 closed)

**P1 outcome: PARTIAL** (EXP-L9b corrected instrument; the
pre-registered EXP-L9 recorded alongside as instrument-never-engaged).
Applying the pre-committed table's partial row:

> **Step 3 proceeds with fuse designed as filament reabsorption**
> (targeting generation > 0 degree-2 chains), and P4 runs first to
> locate where reabsorbable energy actually sits.

P4 has run: reabsorbable energy sits **uniformly along filament, q = 50
per link** (not tip-concentrated) — the fuse design target is "unzip a
chain link, recover q, delete the midpoint", applicable anywhere along
a filament, and the audit invariant extends naturally
(E + c·N_sub − q·N_fuse... exact bookkeeping to be specified in v4).
The rate-vs-g_max shape (smooth, saturating: 0.35 / 0.51 / 0.70 / 0.84
of control at g = 1/2/4/8) says throttling is transport-limited
re-concentration, not a threshold — mechanistically consistent with
reabsorption as a continuous counter-process.

**P2's modulation:** V3 exits untested (far_nucleus = placement/
tie-break artifact; non-conduction = priority-structure artifact), so
no conduit-coupling arm is added; the taxonomy for the EXP-L2 rerun on
the capped family simplifies to V0–V2 **without closing V3** — it
returns whenever a rule shape allows a super-θ node to transport.

**Standing v4 preconditions carried from the table:** fuse targeting
likely wants a degree guard, which the grammar does NOT currently have
(node_property_guards / node_property_compares / var_guards only) —
the grammar audit is a v4 precondition, confirmed still open. The
census identity (subdivision ≡ wormhole production) sharpens step 3's
question: the geometry-sustaining quadrant can only be filled by a
counter-process, not by constraining growth.

**Additional v4 note (from EXP-L9's inertness):** any future guard-
based instrument must ship with a "guard engagement" counter (how many
bindings the new guard actually rejected) — a zero-rejection sweep is
uninterpretable, and this round only caught it because the census made
the morphology visible.

### EXP-L12 (Q1) — priority swap: FIRST DISTRIBUTED GROWTH; row = mixed → D2 row A

`transport_first` at reference, E₀ ∈ {1e4, 1e5, 1e6} × 3 seeds
(`runs/exp_l12/`, audit exact everywhere; cells are byte-identical
across seeds — the dynamics is zero-draw and the clean-lattice IC is
seed-independent, recorded):

| E₀ | events | chains (n / p50) | max gen | conducting | sparse windows |
|---|---|---|---|---|---|
| 1e4 | 86 | 15 / 6 | **2** | 76/86 | 40/40 (all) |
| 1e5 | 1188 | 15 / 87 | **2** | 1015/1188 | 113/120 |
| 1e6 | 12209 | 14 / 906 | **2** | 10400/12209 | 16/600 |

**max_gen = 2 in every cell: non-IC nodes re-reached θ and subdivided —
the first distributed growth this family has ever produced.** Multiple
payers (14–15 chains vs the pump's 6), most chain nodes conduct, and
the guard counters confirm massive transport engagement (1.3e9 evals at
1e6). The "pump-or-freeze dichotomy" row is refuted; the relaxation-
then-grow dynamics is real. But no cell holds a Sparse stratum ≥ 200
windows (the dense crossing is still era-cumulative: 1e6 crosses hard),
so the CD prerequisite is not met here → **row = mixed; D2 row A
default** (fuse targets the swap regime; F3 runs both).

### EXP-L13 (Q2) — unfreeze cells: freezing is PROFILE-FLATNESS, not the q ≤ 2τ coincidence

(q,τ) ∈ {(50,15), (75,25)} × both priorities × 3 seeds
(`runs/exp_l13/`). Pump cells at q > 2τ are morphologically IDENTICAL
to reference (6 chains, seed-only, ≤6 conducting nodes): interior chain
nodes have equal-energy neighbours, so no gradient ≥ 2τ ever exists
regardless of q — **F-LK10's dead-zone-at-birth flag describes only the
child-vs-empty-bulk gradient; interior freezing is caused by flat local
profiles and is NOT parameter-local.** Swap × (50,15): every chain node
conducts (5236/5236) yet max_gen = 1 — pre-committed row (b),
*energetically live, structurally frozen* (energy spreads but dilutes
below θ). Swap × (75,25): zero conduction — row (c) with diagnosis in
hand (children born into an equilibrated background where no ≥ 2τ gap
exists). P4's "frozen at birth" is restated: frozen by profile
flatness.

### D2 — fuse design gate (convened 2026-07-16 on Q1 + Q2)

Q1 = mixed → **row A default: fuse targets the swap regime; the pump
two-era arm runs alongside in F3** (both regimes are real and cheap).
F1 engine work shipped regime-independent as scheduled: guard counters
+ guards.csv (§1.1), `node_degree_compares` (§6.2 — precondition check
found the incidence index is per-slot, so the guard dedups, O(deg); the
pinned distinct-edge semantics is test-pinned on multi-incidence),
priority axis + manifest fields (§1.2/§1.3), fused-energy ledger with
the extended exact invariant (§6.3). FG1–FG4 all pass
(crates/hg-sim-core/tests/lk_fuse_gates.rs); FG2's churn threshold
pre-registered at 0 for reference pump params, measured 0. Design note
recorded: under MaxParallel, fuse bindings disjoint from the seed's
event CAN fire during the growth era (conflicts are per shared node,
not global), so "fuse activates only at quench" is an empirical claim
F3 measures, not a scheduling guarantee.

### Q1 V3 rider — first fair test of V3: NEGATIVE

At-mouth encounter arm under the swap (32², p = 0.005, E₀ = 4e5,
`runs/exp_l12/l12_v3_rider_*`): transport now demonstrably reaches
handle mouths (3/15 shortcuts hold energy at final step; the seed shed
its entire budget), the ambient front reached the far-mouth distance at
step 4000 — and **far_nucleus never fired**. No amplification
signature: handles behave as ordinary edges in the diffusion field.
V3's status moves from "untested" (v3 round: artifacts) to **tested
once, negative** — conduit self-amplification is unsupported under the
only physics where it could have appeared.

### EXP-L7R attempt outside F3 — prerequisite still unmet, recorded

Swap cells at E₀ = 2e5 × 5 seeds (`runs/exp_l7r/`): the sparse stratum
collapsed to ~9 windows (the crossing is budget-sensitive; 1e5 was near
the optimum) and closure sampling is metrics-window-based (n = 50 at
window 500) — every tuple correctly reads certainty Unknown, including
a seed-46 stratified fit of −3e5 bits at n = 7 that the floor
quarantines. A ≥ 200-window LIVE sparse stratum does not exist in the
fuse-less swap regime; the CD prerequisite waits for F3's steady row,
exactly as the plan's sequencing anticipated.

### EXP-L14 (F3) — the removal experiment: SUPPRESSION AT BIRTH (the decay row's limit)

Arms {pump, swap} × {clean, planted} × 3 seeds, 32², E₀ = 4e5, fuse
enabled (`runs/exp_l14/`; a 16²/6e4 first pass with identical verdicts
is archived in `runs/exp_l14_16sq/` — it ran first because the
pre-optimization matcher scanned all edge pairs for the fuse wedge,
O(E²)/step; adjacency-guided matching in hg-engine restored the full
scale, with binding-set and canonical-hash regressions pinning
equivalence). Audit exact in all 12 arms under the extended invariant.

**Every clean arm, both regimes: the emergent-wormhole population is
ZERO at every window** (peak = final = 0 across 120–300 windows; final
chain census 0/0). The pre-registered decay row is met in its limit —
not decay-after-growth but **suppression at birth**: under MaxParallel,
fuse bindings disjoint from the growth event fire in the same step, so
each filament link is reabsorbed within a window of its creation. The
predicted two-era pump dynamics (growth era → reabsorption era) never
appears; production and removal run concurrently, and removal wins
everywhere the growth event isn't standing. A decay constant is
unmeasurable at window resolution (nothing ever accumulates); the
headline numbers are the suppression itself, plus the ledger: ~83% of
E₀ exits through fused midpoints (e_fused ≈ 3.34e5 of 4e5), with the
seed still completing its full era (6666 subdivisions, pump).

- **Churn is a swap-regime phenomenon:** 0 in every pump arm (FG2's
  pre-registered threshold, confirmed at scale) vs ~5,070 per swap arm
  — distributed payers re-subdivide fuse-rejoined sites. Measured, not
  prevented, exactly as §7 designed.
- **Planted handles: non-removal holds, with a REFINEMENT nuance.** The
  conserved object is the handle census (tracked_components constant in
  every planted arm). The tracked EDGE SET is not bit-constant: a
  planted edge adjacent to growth can be legitimately subdivided
  (lineage closure follows the replacement pair) and its gen-1 midpoint
  then fused back (closure follows the rejoined edge) — refinement and
  reabsorption of a handle, never removal. The F3 harness assertion was
  corrected from bit-constancy to census-constancy accordingly; the v4
  §1.4 scope statement stands. Planted-arm wormhole trajectories
  (12–24 → 0–12) are the planted baseline under census sampling noise,
  not emergent dynamics.
- **EXP-L7R remains unauthorized:** the steady row did not land — the
  emergent leakage spectrum is not a standing dynamical object under
  these parameters, it is suppressed; and no arm holds a ≥ 200-window
  live sparse stratum. The CD question needs a fuse-rate knob (v5) that
  lets a nonzero population persist.

**F3 conclusion (with §6.4 scaffolding conditionality, both priority
regimes, 32² and 16² scales):** within this rule family, once removal
dynamics exists, locality is an attractor in the strongest form the
family can express — the geometry-dissolving structure never
accumulates at all. Per the plan's closing line, the v5 authorization
is the decay row's: **the attractor write-up**, with the fuse-rate /
detailed-balance question (can a tuned production/removal balance hold
a persistent, CD-measurable leakage spectrum?) as its natural
companion.

## EXP-L2 v2 — geometric-phase robustness under sustained growth (2026-07-12)

Run per plan v2 §9 on the G-gated LK-R rule family (`rules/lk_pair.toml`,
reference params θ=100/q=50/c=10/τ=25). Arms: 48² TriLattice ×
p ∈ {0, p_peak/10 = 5e-4, p_peak = 5e-3, 10·p_peak = 5e-2}, seed 42;
long arms E₀ = 1.5e6 → 250-window era (26 000 steps, window 100), tracked
lineage on. **Scaffolding scope (mandatory, §6.4): boundedness and
sustainedness are imposed by dissipation + diffusion seeding; every claim
below is conditional on the sustained bounded structural phase.**

**Audit invariant: exact (residual = 0.0) at all 260 windows of both long
arms**; era quenched at n_sub = 24 999 ≈ E₀/(q+c), as the arithmetic
predicts.

### The dominant result is NOT in the verdict taxonomy: emergent wormhole production

On the p = 0 control arm the refinement front **manufactures wormholes at
≈ 1 per subdivision** (sampled census: wf 0.016 → 0.785, count est
110 → 25 055 over the era), organizing into handles at ≈ 1 per 32
subdivisions (components 13 → 775). The mechanism: downhill-only transport
cannot re-concentrate energy to θ, so the seed is the only super-θ node
and **extrudes 1D chains** whose every edge has detour > cap. The §9
control-arm pre-commitment ("any nonzero steady emergent-wormhole rate is
a standing result") is met at the strongest possible level.

### Phase-plane trajectory: growth dissolves its own background

Both long arms migrate **sparse → dense** (defect crosses 0.28 near step
9k; d_bfs falls as 1D chain mass dominates). Geometry here is unstable
not to the planted shortcuts but to the growth morphology itself. Note
the instructive G4 tension: `cut_exponent` stays < 0.8 throughout (G4
passes as defined) while the calibrated phase classifier reads Dense —
consistent with the LK0R demotion of the cut channel; cut alone is not a
sufficient geometric-phase criterion, the phase plane is.

### Planted-population verdict: inert residue (V1-degenerate)

The tracked lineage is **bit-constant for 260 windows** on the p_peak arm
(35 edges, 34 handles, 26 wormholes, throat 2, detours cap-saturated at
∞, `far_nucleus` never fired). The front never consumes a planted
shortcut. No stretching measurable (already beyond cap), no throat
formation, no V3 amplification, and no dilution *of the tracked count*
(the sampled wf rise is emergent production, not the plant). At
10·p_peak the planted set registers **zero** wormholes (346 edges,
265 handles, exact detours p95 = 9 ≤ cap) — the LK0R mutual-redundancy
crossover, confirmed on live tracked data.

### EXP-L7: first valid live closure defect

- p_peak arm: **CD = +15.99 bits/window, 260 windows, certainty
  Inferred, no pathology** — the first closure defect above the floor.
  Phase-plane position: mostly Dense, so per the extended pre-commitment
  this is a dense-quadrant number, comparable only within-quadrant.
- p = 0 arm: CD = −29.7 bits with `closure_defect_negative` FIRED —
  estimator pathology on the strongly non-stationary control trajectory;
  recorded, not a finding. No valid within-quadrant control comparison
  exists this round; that requires a second valid-CD arm.

### Harness event worth keeping

The first parameter-sweep run showed audit drift in every cell — traced
to the sweep harness formatting `50.0 + c` as a TOML *integer*, which the
Float `subtract` effect skips on variant mismatch (engine behaving as
specified), so the seed never paid. **The audit invariant caught a
harness bug on first contact**, which is exactly the anti-gaslight job it
was given (§6.2).


Pre-committed interpretations are in hypersim_leakage_plan_v1.md §9. What
follows is the recorded outcome of the feasible experiment pass, seed 42,
`fixed_provenance`. Compute note: the heavy full matrix (6 rules × 4 ICs ×
5 seeds, and the ≥200-window closure runs) is not run here — the builtin
rules either EXPLODE the lattice (wolfram_analog, binary_edge → tree in
tens of steps: 256-node lattice → 57k nodes by step 64) or stay FROZEN on
a clean lattice (glider, growth_treadmill: their LHS patterns need marks a
clean lattice lacks). This is itself EXP-L2's substrate-characterization
result, recorded below.

## EXP-L1 — Fixture calibration (the gate)

**PASS.** See docs/fixture_calibration.md. All instrument rankings hold;
p_min and the small-world redundancy transition are recorded; three
plan-v1 deviations documented (expander sizing, wormhole non-monotonicity
at dense p, binary-tree plateau).

## EXP-L2 — Is locality an attractor?

Rule-phenotype classification (the pre-committed three-way split):

| rule | on TriLattice | phenotype |
|---|---|---|
| excitation_front (relabel) | structure invariant | **frozen leakage**: planted population neither decays nor grows |
| wolfram_analog | explodes to tree (d_eff→1, cut→volume) | **geometry unstable**: cannot sustain the lattice |
| binary_edge | explodes to tree (wormhole_frac→1.0, cut 0.24–0.34) | **geometry unstable** |
| glider, growth_treadmill | no matches on clean lattice | inapplicable substrate (frozen) |

**Headline finding (excitation front, structure-preserving):**
wormhole_frac(t) is FLAT across all 12 windows at every density — the
pure-relabel dynamic leaves the geometry exactly as planted. This is the
"steady-state plateau" branch: for this rule class the substrate's
leakage is precisely its initial condition, neither self-purifying nor
unstable.

- p = 0.0: wormhole 0, cut_exponent 0.504, area_law_defect 0.093 — bit-for-
  bit the LK0 clean-lattice baseline (the live observer and offline
  instrument agree, as fixture-parity requires).
- p = 0.01: wormhole 0.00083 (constant), cut climbs to ~0.71, defect ~0.16.
- p = 0.03: wormhole 0.0 at the 4800-edge budget — the calibrated
  redundancy transition (finding 2): denser shortcuts are mutually
  redundant, so fewer are individually load-bearing; the cut instrument
  still registers the elevation (defect 0.05–0.15 vs 0.093 clean).

**Instrument-jointness confirmed live:** on binary_edge's tree, detour and
cut agree (wormhole 1.0 AND cut→volume); on the ring degeneracy (LK0) they
disagree and are read jointly. Neither is trusted singly.

## EXP-L5 — Cone census (excitation front)

`hypersim-cli cone`, geometric (wormhole-censored) metric, Δ = 40:

| condition | v_eff | dynamics anomalous | cleanup channel |
|---|---|---|---|
| p = 0.0 | 1.021 | 0 / 1755 | 0 (pure relabel) |
| p = 0.01 | 0.809 | 3 / 1417 | 0 |
| p = 0.03 | 0.804 | 0 / 1260 | 0 |

Clean lattice: zero anomalies, v_eff ≈ 1 site/step — the reference cone.
Planted shortcuts produce a small positive anomaly count at the sparse
onset (consistent with the LK2 calibration's onset-peak shape). The
cleanup channel is empty throughout: the relabel rule deletes nothing, so
no engine bookkeeping masquerades as non-locality.

## EXP-L6 — Counterfactual fronts (twin harness)

Remove-edge twin on the p = 0.01 lattice: constant damage_frac 0.031,
front_radius 3, v_damage 1.5 — the removed edge stays a LOCAL residual
(the front does not spread because excitation converges), so damage is a
bounded scar, not a propagating wave. Control twin: damage ≡ 0
(v_damage = NaN, no front). Both validity controls hold on live runs.

## EXP-L7 — Closure defect

On the 12-window feasible runs CD is emitted with **certainty = Unknown**
(below the 200-window floor) and the short-run augmented model overfits
into large-magnitude negative values, correctly raising
`closure_defect_negative`. This is the discipline behaving as designed:
below the floor the number is not a result. The estimator's VALIDITY is
established by the synthetic sign tests (crates/hg-causal/tests/closure.rs):
planted-leak CD ≈ 12.2 bits at 3σ over 10 seeds; macro-closed CD ≈ 0.
A trustworthy live CD needs a bounded, non-exploding dynamic run for ≥ 200
windows — not available in the builtin rule set on a lattice.

## Net

The instruments are calibrated, gated, and produce coherent live readings
across four surfaces (detour/cut, cone, twin, closure) with the
epistemic tags and floors enforced. The scientific bottleneck is not the
LK machinery but the rule library: no shipped rule sustains a non-trivial
GEOMETRIC phase on a lattice long enough to ask whether locality is an
attractor under structure-changing dynamics. That rule is the missing
ingredient — the same gap the ALife track's node-funded replicator is
meant to fill.
