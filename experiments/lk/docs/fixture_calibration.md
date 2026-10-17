# LK0 Fixture Calibration (EXP-L1)

## LK0R recalibration (plan v2, 2026-07-12) — p_peak curve and phase thresholds

**Status: LK0R gate passed.** Suite revised per v2 §3.3
(`small_world_wormhole_unimodal_with_recorded_p_peak`,
`small_world_cut_channel_monotone_or_demoted`,
`expander_precondition_fails_loudly_when_underranged`,
`p_peak_shifts_with_r_max`); sweep: `p_peak_calibration_sweep` (--ignored),
seeds 3000–3009 (10 for n ≤ 128², 3 for 256²), full census.

### p_peak(n, R_max) — argmax of mean wormhole_frac over the p grid

| n | cap 8 | cap 12 |
|---|---|---|
| 48² (test fixture) | ≈ 3e-2 | ≈ 5e-3 |
| 64² | 3e-2 (peak 0.0174 ± 0.0012) | 1e-2 (peak 0.0039 ± 0.0005) |
| 128² | ≥ 5e-2 (still rising at grid edge) | 1.5e-2 (peak 0.0090 ± 0.0003) |
| 256² | ≥ 5e-2 (still rising) | 3e-2 (peak 0.0163 ± 0.0003) |

**Crossover validation (v2 §3.2): PASSES in both directions.** p_peak moves
DOWN with R_max at every n (a larger cap lets fewer helper shortcuts
reroute a removed one) and UP with n at fixed cap (helper shortcuts must
land within cap-balls of *both* endpoints; that probability falls as the
torus grows, so more shortcuts are needed to reach mutual redundancy).
Both directions are the crossover mechanism's predictions; F-LK1 stands.

### Cut-channel monotonicity verdict (v2 §1.2 pre-commitment): **DEMOTED**

`area_law_defect` is NOT the dense-phase monotone: it rises through the
sparse phase then falls across the crossover (0.093 → 0.134 → 0.176 →
0.074 at p = 0/1e-3/1e-2/5e-2 on 48², seed 42). Per the pre-commitment it
is demoted to phase-plane coordinate only. The promotion of `dim_mismatch`
to dense-channel scalar is **blocked on the reserved spectral-dimension
observer** (d_spec is not implemented); until it lands, the phase plane is
the only dense-phase readout, and `d_bfs` inflation serves as the
dissolution signature (below).

### Phase-boundary thresholds (consumed by §1.3's gate and §1.5's quadrants)

Implemented in `hg_observe::locality::classify_phase(wormhole_frac,
area_law_defect, d_bfs)`. Because the cut channel is demoted (it FALLS in
the dissolved phase on small tori: sw(48², 5e-2) → defect 0.074), two
independent dissolution signatures are used:

| boundary | area_law_defect | d_bfs |
|---|---|---|
| Dense (dissolved) | ≥ 0.28 | or ≥ 2.7 |
| Crossover | ≥ 0.20 | or ≥ 2.5 |
| Sparse | below both | below both |

Reference points: clean 2d (0.093, 1.70) → Sparse; sw(1e-2) (0.176, 2.31)
→ Sparse; clean 3d (0.015, 2.44) → Sparse; sw(5e-2) (0.074, 2.81) → Dense
via d_bfs; expander (0.287, 2.86) → Dense; tree (0.444, 1.61) → Dense via
defect. Missing defect or d_bfs → Crossover (never Sparse by default).
**Calibrated for 2d/3d lattice substrates**; genuinely higher-dimensional
substrates require re-calibration before the phase gate is trusted.

### LK-R gate results (v2 §6.3) — the rule family SHIPS

All gates pass on the reference cell and on the ENTIRE 9-cell sweep
(c ∈ {5, 10, 20} × E₀ ∈ {5e4, 1e5, 4e5}; q = θ/2 = 50, τ = θ/4 = 25
fixed; `lkr_parameter_sweep`, --ignored):

- **G1** zero Exploding windows, node count era-bounded (plateau =
  lattice + E₀/(q+c)) — all 9 cells.
- **G2** one subdivision per step for the whole era; era = E₀/(q+c)
  steps exactly (quench arithmetic pinned by the audit test). The
  250-window science floor is therefore a budget choice: E₀ =
  250 · window · (q+c).
- **G4** cut_exponent < 0.8 for 100% of the era in all 9 cells. BUT see
  the EXP-L2 finding: the phase classifier still reads Dense late in
  long eras (defect climbs via d_bfs suppression) — cut alone is not a
  sufficient geometric criterion, consistent with its demotion.
- **G5** canonical hash identical across runs and thread counts.
- Priority order (subdivide=1 > transport=0) is PINNED and measured
  order-SENSITIVE (swapping changes the canonical hash) — hazard 4's
  alternate-priority rerun is a real control.

**Dynamics phenotype (recorded):** downhill-only transport can never
re-concentrate energy to θ (children sit in the |Δ| < 2τ dead zone), so
the seed is the only super-θ node and extrudes 1D chains — sustained,
bounded, geometric by G4, and a prolific EMERGENT-wormhole source (see
leakage_experiment_findings.md). The structural front is pinned at the
seed by construction of this rule shape.

**Audit as tripwire:** the first sweep attempt drifted in every cell —
the harness had formatted the subtract constant as a TOML Int, which the
Float effect skips on variant mismatch. The §6.2 invariant caught it
immediately. Any audit drift is an accounting bug; never rationalize it.

### P0 gate re-audit under G4-new (plan v3 §2, 2026-07-15)

The v2 sweep outputs were not preserved (ephemeral dir), so the 9-cell
sweep was RERUN (same seed 42, params, harness; `lkr_parameter_sweep`
with `HYPERSIM_SWEEP_OUT`) with per-cell `leakage.csv` persisted to
`runs/p0_gate_reaudit/` — a deviation from v3 §2's "no reruns", forced
by the missing record and noted here. G4-new = ≥ 80% of sustained-era
windows Sparse on `classify_phase` (thresholds: this doc, LK0R section;
cited in every run manifest as `classifier_calibration`).

| cell | era subs | audit | G1 | G2 | G4-old (cut) | G4-new (sparse frac) |
|---|---|---|---|---|---|---|
| c5, E₀=5e4 | 908 | exact | pass | pass | 1.00 | **1.00 PASS** |
| c5, E₀=1e5 | 1817 | exact | pass | pass | 1.00 | **1.00 PASS** |
| c5, E₀=4e5 | 7271 | exact | pass | pass | 1.00 | **0.33 FAIL** (12 sp / 12 cr / 12 dn) |
| c10, E₀=5e4 | 832 | exact | pass | pass | 1.00 | **1.00 PASS** |
| c10, E₀=1e5 | 1666 | exact | pass | pass | 1.00 | **1.00 PASS** |
| c10, E₀=4e5 | 6666 | exact | pass | pass | 1.00 | **0.36 FAIL** (12 sp / 11 cr / 10 dn) |
| c20, E₀=5e4 | 713 | exact | pass | pass | 1.00 | **1.00 PASS** |
| c20, E₀=1e5 | 1428 | exact | pass | pass | 1.00 | **1.00 PASS** |
| c20, E₀=4e5 | 5713 | exact | pass | pass | 1.00 | **0.43 FAIL** (12 sp / 9 cr / 7 dn) |

**Reading (refines the P0 expectation):** G4-new does not fail on every
cell — it fails on exactly the three long-era cells (E₀ = 4e5), at every
c, with the sparse→crossover→dense crossing landing near step ~2.4k in
each. The six passing cells simply quench before the crossing. The dense
crossing is **era-cumulative, not parameter-local**: wormhole production
accumulates until the phase plane tips, so any era long enough to meet
the 250-window science floor (E₀ ≥ 250·window·(q+c)) crosses. G4-old
reads 1.00 in all nine cells including the three phase-plane failures —
F-LK6's hollowness, reconfirmed on fresh data.

**Status re-label (P0 deliverable):** the rule family's shipped status
is *bounded sustained structural dynamics (G4-new: FAIL at science-floor
era lengths)*, pending P1. The geometry-sustaining quadrant remains
unfilled.

**Abort-gate calibration (v2 §7):** binary_edge's Exploding
classification is TRANSIENT — a 2-window burst, then GrowthTreadmill
while absolute growth continues. The gate fires only when k ≤ the burst
length; the default k = 3 would sail past this dynamic. k = 2 is the
tested working value for burst-type exploders.

### Generator preconditions (F-LK3, v2 §3.1)

`expander_checked(n, d, r_max, seed)` asserts `n > 4·d·(d−1)^(r_max−1)`;
`small_world_checked(width, p, r_max, seed)` asserts torus diameter
(width/2) > r_max. The v1 calibration fixture expander(16384, 3) is
UNDER-RANGED at cap 12 by this formula (needs n > 24576) — consistent
with its measured wormhole_frac of 0.777 rather than ~1. The registrable
instantiation is **expander(32768, 3): wormhole_frac 0.844** and the
under-ranged call now panics (pinned by test).


Status: **gate passed** (2026-07-11). Suite: `crates/hg-eval/tests/locality_fixtures.rs`.
All numbers below are `[Measured]` on static fixtures with seed 42 (sweeps: seeds
1000–1009), instrument defaults `detour_r_max = 12`, `ball_sources = 32` (d_eff
protocol, shared BFS, `r_max = 10`).

## Measured fixture values

| fixture | detour p50/p95 | wormhole_frac | cut_exponent (r²) | d_eff | area_law_defect |
|---|---|---|---|---|---|
| lattice_2d_tri(48) | 2 / 2 | 0.0000 | 0.504 (1.000) | 1.697 | 0.093 |
| lattice_3d(12) | 2 / 2 | 0.0000 | 0.605 (0.972) | 2.440 | 0.015 |
| small_world(48, 1e-3) | 2 / 2 | 0.00087 | 0.581 (0.999) | 1.808 | 0.134 |
| small_world(48, 1e-2) | 2 / 2 | 0.00258 | 0.743 (0.996) | 2.309 | 0.176 |
| small_world(48, 5e-2) | 2 / 2 | 0.0000 (see finding 2) | 0.719 (0.947) | 2.814 | 0.074 |
| expander(16384, 3) | ∞ / ∞ | 0.777 | 0.938 (0.999) | 2.863 | 0.287 |
| ring(200) | ∞ / ∞ | 1.000 (documented degeneracy) | 0.000 (1.000) | 0.855 | 0.169 |
| tree(3, 9) | — | — | 0.943 (0.997) | 2.288 | — |
| tree(2, 12) | ∞ / ∞ | 1.000 | 0.823 (0.970) | 1.611 | 0.444 |

Exit-criterion ranking holds on both instruments with p₁ = 1e-3, p₂ = 1e-2:
`lattice < sw(1e-3) < sw(1e-2) < expander` on wormhole_frac
(0 < 0.00087 < 0.00258 < 0.777) and on area_law_defect
(0.093 < 0.134 < 0.176 < 0.287).

## Calibration findings (plan-v1 deviations, documented not silently coded)

1. **Expander instantiation.** A d-regular expander's edge detour is
   ≈ log_{d−1}(n). Flagging wormholes at `detour_r_max = 12` with the plan's
   d = 4 requires n > 3¹² ≈ 531k nodes — past test budgets. Calibrated fixture:
   `expander(16384, 3)` (log₂ 16384 = 14 > 12). The verdict (wormhole_frac
   > 0.5, cut_exponent > 0.9) is about the fixture *family*, and holds.

2. **wormhole_frac is non-monotone in p — the small-world transition.** The
   plan expected "all leakage metrics strictly monotone in p". Measured:
   monotone in the sparse regime (0 → 1e-3 → 1e-2), then **collapse to 0 at
   p = 5e-2** — with ~350 shortcuts on a 48×48 torus the shortcut population
   is mutually redundant: a removed shortcut's endpoints reconnect through
   *other* shortcuts within the BFS cap, so no single edge is load-bearing.
   Real phenomenon, not an instrument bug; it means wormhole_frac measures
   *individually load-bearing* shortcuts, and EXP-L2's "planted population
   decayed" verdict must therefore also consult detour_p95 and cut/defect,
   not wormhole_frac alone, at p ≥ ~5e-2.

3. **Binary trees plateau below the "→ 1" idealization.** cut_exponent(tree(2,
   depth)) ≈ 0.823 at depth 12 *and* 14 (leaf-heavy source sampling + the
   climb-to-root regime). Branching 3 gives 0.943. Both are far above every
   clean lattice; the test asserts > 0.9 at b = 3 and > 0.75 at b = 2.

4. **Lattice baseline defect ≠ 0 exactly.** area_law_defect(lattice_2d_tri) =
   0.093, not 0, because d_eff = 1.697 (the known small-r +1-offset bias of
   the ball-growth slope on a degree-6 lattice), while cut_exponent lands at
   0.504. Defect values are comparative, not absolute; the clean-geometry
   reference value **0.09 ± 0.02 (2d), 0.02 (3d)** is the baseline against
   which live-run defects should be read.

## LK3 twin-harness calibration

All three validity controls pass (`crates/hg-sim-core/tests/twin_harness.rs`,
`crates/hg-eval/tests/twin_metric.rs`): control twin damage ≡ 0, ID-permutation
damage ≡ 0 (the ball-WL-profile metric is NodeId-free by construction), and on
the reference dynamic **v_damage = 1.0088 vs v_eff = 0.9961** (agreement to
0.013).

- The plan's "equal to 3 decimals" is unattainable in principle: anchor
  distances are integers and front_radius is a p95 over 64 anchors, so
  v_damage is quantized. Adopted tolerance: **0.05**.
- **v_damage is defined over the expansion phase only** (windows that extend
  the running max of front_radius). On a convergent dynamic like the
  excitation front, both branches reach the same all-excited state, so damage
  is TRANSIENT — post-peak windows measure recovery, not propagation, and
  including them halves the estimate (measured 0.43 vs 1.01). A persistent
  residual (the removed edge itself) keeps a small damage_frac near the site
  indefinitely; that is structure, not spread.

## Sensitivity: p_min vs sample budget (the null-result floor)

Criterion: mean wormhole_frac over 10 seeds > 3σ above the p = 0 baseline
(baseline is exactly 0 with zero variance on the clean lattice). Swept
p ∈ {5e-4, 1e-3, 2e-3, 5e-3, 1e-2} on small_world(48, p), ~6.9k edges.

| detour_sample_edges | p_min at 3σ |
|---|---|
| 64 | **not reached** in tested range |
| 256 (default) | **not reached** in tested range |
| 1024 | **not reached** in tested range |
| full census | ≤ 5e-4 (smallest tested) |

## LK2 cone-analysis calibration (live dynamics)

Reference dynamic: `rules/excitation_front.toml` on `TriLatticeExcited`
width 32, MaxParallel, 64 steps. All acceptance criteria met:

- **Reference front speed: v_eff = 0.9961 sites/step**, identical across
  seeds 42/7/100 (deterministic dynamic, seed-independent p = 0 lattice).
  On the clean lattice: 0 of 2370 dynamics descendants anomalous; the
  pure-relabel rule produces zero cleanup-channel descendants.
- **Geometric metric (normative clarification over plan v1 §1.3):** the
  cone analysis measures distance in the *wormhole-censored* reference
  snapshot (edges with §1.1 detour > `detour_r_max` removed). Raw graph
  distance makes the instrument vacuous — influence travels along edges of
  the same graph, so a shortcut jump is metric-legal by construction. The
  plan's own criterion (d) is only satisfiable against the censored metric.
- **Criterion (d) revised by measurement:** anomalous_frac_dynamics is NOT
  monotone increasing in p. It peaks at the sparse onset and collapses as
  shortcuts become mutually redundant (the finding-2 small-world
  transition, on live dynamics): p = 0.002 → 0.0574, 0.005 → 0.0176,
  0.008 → 0.0223, 0.01 → 0.0045, 0.015 → 0.000. The exit criterion that
  matters — zero false positives at p = 0, clear detection at sparse
  p > 0 — holds and is pinned by
  `crates/hg-sim-core/tests/cone_analysis.rs`.
- A torus must be larger than the detour cap to be censorable at all:
  diameter(width 24) = 12 = default `detour_r_max`, so NO edge on a
  24-torus is ever a wormhole; cone tests use width 32 (diameter 16).

**Consequence (binding on all later experiments):** at the default sample
budget of 256 edges the expected wormhole hit count in the sparse regime is
below 1 (256 × ~0.0026 ≈ 0.7 at p = 1e-2), so seed-to-seed variance swamps
the signal. A null wormhole_frac result at default budget is **not evidence
of locality** for p ≤ 1e-2 populations; it is a finding only relative to
this table. Experiments needing sparse-regime sensitivity must either raise
`detour_sample_edges` to the full edge count (cost: one truncated BFS per
edge per window) or cite this floor. EXP-L8 extends this curve to live
dynamics if commissioned.
