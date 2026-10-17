# Locality as a Balance, Not a Geometry

*An attractor characterization of the LK-R rule family.*
*Companion to the substrate characterization report; LK track, v5 / Phase R5.*
*Draft 2026-07-17.*

---

## Abstract

Within the energy-metered LK-R rule family on a triangulated torus, spatial
locality — the absence of long-range shortcuts ("wormholes") in the graph
metric — is **not a fixed property of the geometry**. It is a property of the
balance between two dynamical processes: *extrusion*, the production of
1-D filament structure by metered subdivision, and *removal*, the
reabsorption of that filament by generation-guarded fusion. Locality is an
attractor exactly when a removal process exists and outpaces extrusion; the
geometry dissolves into a wormhole-dense phase otherwise. We exhibit both
halves — a dissolution phase (v2–v3), a removal-dominated attractor with a
measured decay constant and a structurally bit-restored endpoint (R1) — and
we map the boundary between them (R3), finding that in this family the
balance knob is **bistable rather than continuous**: the removal process is
all-or-nothing, so no interior steady-turnover state exists. Every claim is
conditional on the scaffolding named in §1.

---

## 1. Scaffolding conditionality (stated once)

Everything below holds *within* a specific, imposed scaffolding, and nowhere
do we claim these are emergent. Stated once, precisely, and not re-hedged
per paragraph:

- **Dissipative metering.** Subdivision costs a fixed quantum `c` per event,
  drawn from a finite seeded budget `E₀` placed at one site. Boundedness of
  the structural phase is *imposed* by this dissipation, not discovered.
- **Seeded budget.** All energy enters at t=0 at a single site; there is no
  influx. Every process is ultimately budget-limited, and every run quenches.
- **Deterministic MaxParallel.** The engine fires a maximal node-disjoint set
  of rule bindings each step, with a deterministic tie-break. There are no
  temporal "eras" in the scheduler — production and removal are concurrent
  processes, ordered only where they contend for a shared node (§4 depends on
  this).
- **Generation-guarded fusion.** The removal rule reabsorbs only *emergent*
  filament (generation ≥ 1); the original lattice edges and any planted
  handles are non-removable by construction. The attractor is therefore an
  attractor of the emergent structure, over a fixed lattice skeleton.

Reference parameters: θ=100 (subdivision threshold), q (child energy), c=10
(dissipation), τ (transport quantum), on a 32² torus. The exact energy
audit `E_total + c·N_sub + Σ E_fused = E₀` holds to a residual of exactly
0.0 at every window of every run cited here; any nonzero residual halts
interpretation, and none occurred.

---

## 2. The dissolution side (v2–v3)

With production alone (metered subdivide + downhill transport, no removal),
the seed is the only site that ever re-reaches θ, and it extrudes **1-D
filaments** — chains of degree-2, generation>0 nodes — radially into the
lattice. Six filaments form, one per lattice direction off the seed, and
each subdivision event adds exactly one link to one filament: filament
production and wormhole production are the *same event*.

Each interior link of a filament is a graph-metric shortcut. Its detour —
the shortest return path when the link is cut — is **2L+2**, not L+2 (the
seed's own bulk edges are consumed into the filaments, so the return path
must descend a second inflated chain). This retrodiction — a zero plateau
of wormhole count until a filament crosses the R_max/2 detour threshold,
then a slope-1 rise through the origin — matches the measured census
(slope 1.02–1.04). The era is cumulative: the graph crosses from a sparse
to a wormhole-dense phase as the filaments lengthen, and **growth dissolves
its own locality**. This is the dissolution phase: production drives the
geometry away from locality, monotonically, until the budget quenches.

---

## 3. The attractor side (v4 + R1)

### 3.1 Suppression at birth (v4)

Introduce the removal rule (fuse: a degree-2, generation≥1 midpoint is
removed and its endpoints rejoined, its energy vanishing onto the ledger)
and run production + removal together. Under MaxParallel there are no eras:
a fuse binding disjoint from a growth binding fires in the *same* step.
The result is **suppression at birth** — the emergent-wormhole population is
zero at every window, in every clean arm, both priority regimes. Roughly
83% of the endowment exits through the fused ledger; only the sink (the
seed's sub-θ remainder) survives. This is the attractor in its degenerate
limit: no population ever accumulates, so there is nothing to decay. The
demonstration was half complete — an attractor with no observed basin flow.

### 3.2 The basin: exact exponential relaxation (R1 / EXP-L15)

To exhibit the missing half we start from a *populated* far-from-attractor
state — the archived post-quench pump state, six filaments of L₀ = 785
links, interiors flat at q so production is dead — and enable removal.

- **Decay constant.** Fusion under MaxParallel selects a maximal *node*-
  disjoint set of interior triples {x, w, y}; on a filament this is every
  third interior node, so length obeys L(t) = L₀·(2/3)ᵗ exactly. The fitted
  rate is **λ = ln(3/2) ≈ 0.405 per step** — the family's headline number.
  (The naive prediction of alternate-node halving, λ = ln 2, assumed
  *edge*-disjoint bindings; the node-disjoint scheduler is what makes it
  2/3. Same concurrency-semantics correction as §3.1.)
- **Restoration endpoint.** When a filament reaches length 1 its last node
  is a fusable degree-2 midpoint, and its removal restores the original
  lattice edge. The terminal state is **structurally bit-identical to the
  clean lattice** (whole-graph WL hash equality, label- and property-blind:
  1024 nodes, 3072 edges). Every joule of filament energy is reabsorbed;
  the restored lattice is energy-dead and ledgered exactly.
- **Census follows.** The wormhole count decreases monotonically to zero
  once the longest filament drops below the R_max/2 = 6 detour threshold.

Removal dominates from a populated start and drives the geometry back to
the clean lattice at a measured, mechanism-derived rate. With §3.1's
suppression limit and §3.2's basin flow, the attractor demonstration is
complete: **locality is an attractor of the LK-R family whenever removal
exists.** Concurrent production during relaxation (a fresh budget seeded at
a new site) does not interact — the old population decays at λ while the
new production is suppressed at birth, ledger exact throughout.

---

## 4. The balance (R3 / EXP-L16): bistable, not continuous

The remaining question is whether a *partial* removal rate holds a nonzero
population in steady turnover — a balance point between dissolution and the
attractor. We throttle fusion with an energy gate `f_cap`: a filament node
is fusable only once its energy drops below `f_cap`. The intent was an
aging timer — a fresh child carries q, and drains toward f_cap over
`t_age ≈ (q − f_cap)/τ` steps — with standing population ∝ t_age × rate.

**The knob is structurally inert, and the reason is a clean negative
result.** Downhill-only metered transport with a 2τ dead zone never
produces a filament node below q: a fresh child at energy q is conducted
*up* to a plateau (≈65 at (q,τ)=(50,15); flat at q for (75,25)), not drained
down. The interior energy distribution is a near-**spike** at the plateau,
not a spread. So the guard's admitting region (energy < f_cap < q) is
disjoint from the interior energy support (≥ q), and at every f_cap below
the plateau the throttled run is **bit-identical to the fuse-disabled
control** — pure growth, fuse never firing (guard rejects = evals, ledger
empty). Any f_cap above the plateau admits the entire spike at once,
recovering unthrottled suppression (§3.1). The transition is a **jump with
no band between** — bistability, with the threshold at the conduction
plateau, not at the naive q.

The straddle at (q,τ)=(50,15), plateau ≈ 65, is unambiguous:

| f_cap | side | verdict | Σ E_fused | churn | steady census |
|------:|------|---------|----------:|------:|--------------:|
| 60 | below plateau | STEADY (frozen) | 0 | 0 | 1206 |
| 66 | above plateau | EMPTY (suppressed) | 80,275 | 958 | 0 |
| 80 | above plateau | EMPTY (suppressed) | 80,275 | 958 | 0 |
| 100 | above plateau | EMPTY (suppressed) | 80,275 | 958 | 0 |

Below the plateau the ledger is empty (fuse never fires, identical to
pure growth); a single step above it the entire endowment routes through
the ledger and the population is suppressed. The three above-plateau
columns are bit-identical (Σ E_fused and churn to the unit), confirming
the gate saturates the moment it admits the spike. There is no f_cap that
yields an intermediate standing census.

This is the pre-committed "energy gate too coarse" row of the plan: an
energy gate cannot be a *rate* control in this family, because the state
variable it reads (interior energy) has no spread for the gate to select a
fraction of, and never enters the sub-q region the aging-timer premise
assumed. A finer mechanism — f_cap annealed per window, or a removal rule
gated on filament *age* rather than energy — is a v6 question, not
improvised here.

**Consequence for the thesis.** The attractor claim gains its boundary:
within the LK-R family as built, locality holds in the removal-dominated
limit and dissolves in the production-dominated limit, with **no interior
steady state** between them. The balance is real but bistable; a continuous
balance point does not exist under an energy-gated removal rule.

---

## 5. Negative results as findings

- **Non-locality is not self-amplifying (V3, closed negative).** Under the
  fair test (swap regime), the mouths of a planted handle energize but no
  far-side nucleus amplifies: gradient-driven diffusion flattens the profile
  before far-side accumulation reaches θ. Handles are passive conductors.
  Tested once, under the pre-registered fair condition, negative.
- **The closure-defect estimator does not get relaxed standards (CD
  retirement, protocol case study).** v2's headline CD = +15.99 bits/window
  did not survive a 5-seed replication (clock-baseline spread ≫ mean); it
  was retired, and CD is now reported only as a tuple on a gated stratum.
  In this round **EXP-L7R did not run**: it requires a steady-turnover band
  of ≥ 200 consecutive Sparse windows with standing population, and R3
  produced no turnover band at all (every throttled cell was frozen or
  suppressed; the longest Sparse run, 117 windows of *frozen* structure,
  is both below the floor and not turnover). Per the standing discipline,
  CD **waits** for a family that produces a genuine steady state — it does
  not get run on a frozen or absent stratum.

---

## 6. [Speculative] closing

*This paragraph is speculative, tagged, and nothing in the body depends on
it.* A physical geometry that stays local under structure-changing dynamics
would, on the evidence here, be one whose microdynamics is
**removal-dominated** — where whatever process extrudes non-local structure
is outpaced by a reabsorption process, and where that reabsorption is not a
tunable rate but an all-or-nothing sink. The LK-R family suggests that
"space looks local" and "space has an efficient way to delete shortcuts"
may be the same statement, and that the absence of an interior balance point
— locality as bistable rather than dialable — could be a feature of a clean
effective layer rather than a limitation of one. Offered as a framing for
comparison, not a result.

---

## Appendix: provenance

- Dissolution (§2): `runs/expL_v2_archive/`, `runs/q0_retrodiction/`,
  `runs/exp_l9b/`; findings in `docs/leakage_experiment_findings.md` (v2–v3).
- Suppression (§3.1): `runs/exp_l14/`, `runs/exp_l14_16sq/`.
- Basin (§3.2): `runs/exp_l15/`; driver `crates/hg-sim-core/tests/exp_l15.rs`;
  restoration check `crates/hg-eval/src/restoration.rs`.
- Balance (§4): `runs/exp_l16/`; driver `crates/hg-sim-core/tests/exp_l16.rs`;
  interior-energy diagnosis `crates/hg-eval/tests/energy_probe.rs`.
- Lifetime instrument: `crates/hg-eval/src/lifetimes.rs` (offline;
  `lifetimes.csv` per run).
- Every cited run's energy audit residual is exactly 0.0; the fuse boundary
  is pinned by gate FG5 (`crates/hg-sim-core/tests/lk_fuse_gates.rs`).
