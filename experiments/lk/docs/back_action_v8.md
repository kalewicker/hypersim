# Measurement Back-Action, Quantified — v8 / EXP-L18

**What this document is.** v6 introduced `lk_tick`, a single-node rule, to
give the fuse a lifetime gate with dispersion. v7 discovered that the rule
was the dominant actor in the scheduler it was meant to observe (F-LK25).
v8 replaces it with a clock that claims no bindings, reruns the
coexistence measurement, and reports the difference. **That difference is
the quantified version of "there is no passive observer at the rule
level."**

Method: each cell is run **twice**, from an identical config with one
field changed (`gate = "countdown"` vs `gate = "age"`). The archived v7
runs are not used as the control — they were produced by a driver whose
`L*` was `wormhole_count_est`, which Q7-1 showed is threshold-limited, and
a back-action table must not be a difference of instruments. The ticked
arm here reproduces the v7 published numbers (§4), which is what licenses
reading the difference as back-action.

Driver: `crates/hg-sim-core/tests/exp_l18.rs`.
Mechanism and gates: `docs/grammar_audit_v8.md`,
`crates/hg-sim-core/tests/b1_age_gates.rs`.

---

## 0. A precondition that had to be fixed first: "3 seeds" was not 3 replicates

Before any cell could be read, the first output forced a correction to the
plan's own cell list. **Under MaxParallel the seed does not vary the
dynamics.** Conflict resolution is a deterministic greedy pass over a
canonical key, and no LK-R rule draws RNG, so three seeds produce one
trajectory: the `step`, node-count, `n_subdivisions` and `E_fused` columns
are byte-identical across seeds 42/43/44 in both regimes.

What the seed *does* vary is the sampled instruments — the 256-edge detour
sampler and the spectral estimates. So:

- a multi-seed spread on **L\*, p, β, f, n_sub, contention** is not a
  replication and is not reported as one here;
- a multi-seed spread on the **wormhole/detour** statistics is real, but
  it measures instrument sampling noise against a dynamical variance of
  exactly zero.

Pinned by `exp_l18_seed_does_not_vary_the_dynamics`, asserted in both
directions so it cannot pass vacuously: the dynamics columns must match
*and* at least one sampled column must differ.

**This reaches backwards.** The CD tuple protocol uses seed spread as its
guard against overinterpretation, and CD's micro features include the
sampled detour histogram. On this substrate that spread therefore
estimates instrument noise with the dynamical component identically zero.
It remains a valid *floor* — a signal that cannot clear the instrument's
own sampling noise is certainly not a measurement — but it is not the
replication check the name suggests, and EXP-L7R's certainty language
should say so.

---

## 1. P3 — the pump cap VANISHES. It was an instrument artifact.

The sharpest test in the round, and it comes back clean. Q7-4 attributed
the pump's flat standing population to the clock gating both directions
of the dynamics. Remove the clock:

| k | L\* ticked | L\* passive | L\*/k passive | p ticked | p passive | β_L ticked | β_L passive | instrument losses (ticked) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 10 | 12.4 ± 1.2 | 9.9 ± 1.1 | 0.986 | 0.541 | 0.994 | 2.301 | **0.992** | 48,605 |
| 30 | 12.2 ± 0.4 | 29.9 ± 1.1 | 0.995 | 0.194 | 0.994 | 2.100 | **1.001** | 126,596 |
| 100 | 12.2 ± 0.4 | 99.9 ± 1.1 | 0.999 | 0.059 | 0.994 | 2.046 | **1.004** | 138,030 |
| 300 | 12.1 ± 0.2 | 299.8 ± 1.2 | 0.999 | 0.020 | 0.993 | 2.009 | **1.006** | 140,369 |

(pump × vanish × E₀ = 1e5; instrument losses are 0 in every passive arm,
asserted before the row is written — BG6.)

**L\* = k, to three significant figures, at every k.** The "hard
structural cap of 12, independent of k over a 30× range" — reported in v6
and given a mechanism in v7 — does not exist under a clock that claims no
nodes. The k-dependence returns exactly as P3 predicted.

Three things make this unambiguous rather than suggestive:

1. **p → 1.** Production is 0.993–0.994 events/step at every k under the
   passive clock — the seed-serialization limit, one event per step. Under
   the tick it fell as 1/k (0.541 → 0.020). So the clock was throttling
   *production*, which is exactly the mechanism Q7-4 named; what Q7-4 got
   wrong was calling the resulting flatness a property of the geometry.
2. **The budget was never spent.** `n_sub` reaches 1666 — the full
   E₀/(q+c) — in every passive cell. The ticked arm reached 1666 only at
   k = 10, and 1164 / 360 / 120 at k = 30 / 100 / 300. The clock stopped
   the pump from spending its own energy budget.
3. **β_clean = 1.00.** The balance law is not merely restored, it is
   *exact*: L\* = k·p with no scheduler term at all, which is P1's named
   alternative ("β_clean ≈ 1 exactly would mean fuse-fuse contention is
   negligible at these densities").

The serialization itself survives — p ≈ 1 *is* the shared-anchor
serialization, now measured cleanly. What does not survive is the claim
that serialization pins L\*. It pins **p**; L\* then follows k.

## 2. P1 — β falls, and the residual is real but differently attributed

swap × transfer, both budgets, β_clean = the passive β_L:

| k | E₀ | β_L ticked | β_L passive | β_f ticked | β_f passive | f ticked → passive |
|---:|---|---:|---:|---:|---:|---|
| 10 | 1e5 | 1.243 | 1.068 | 1.351 | 1.227 | 0.260 → 0.185 |
| 30 | 1e5 | 1.314 | 1.180 | 1.350 | 1.280 | 0.259 → 0.219 |
| 100 | 1e5 | 1.577 | 1.044 | 1.595 | 1.065 | 0.373 → 0.061 |
| 300 | 1e5 | 1.836 | 1.029 | 1.796 | 1.017 | 0.443 → 0.017 |
| 10 | 4e5 | 1.289 | 1.083 | 1.287 | 1.195 | 0.223 → 0.163 |
| 30 | 4e5 | 1.320 | 1.176 | 1.324 | 1.229 | 0.244 → 0.186 |
| 100 | 4e5 | 1.619 | 1.050 | 1.624 | 1.063 | 0.384 → 0.060 |
| 300 | 4e5 | 2.003 | 1.033 | 1.955 | 1.017 | 0.489 → 0.016 |

**P1 splits by k, and the split is itself informative.** At k ≥ 100,
β_clean = 1.03–1.05: the plan's named alternative, fuse-fuse contention
negligible, L\* = k·p essentially exact. At k ≤ 30, β_clean = 1.07–1.18:
genuinely above 1, so residual intrinsic blocking exists.

Note the **inversion**. Instrument blocking rose with k (β_ticked 1.24 →
2.00); intrinsic blocking *falls* with k (β_clean 1.18 → 1.03). These are
opposite dependences, which is why the ticked β looked like a single
smooth function of k and was not one.

**The plan's attribution is refuted in its specifics.** P1 predicted the
residual would be verifiable against the fuse-loses-to-fuse column. The
directional counter says otherwise — under the passive clock, `lk_fuse`
loses overwhelmingly to **`lk_transport`**, a physics rule:

| k (E₀=1e5) | fuse → transport | fuse → subdivide | fuse → fuse | fuse-fuse share |
|---:|---:|---:|---:|---:|
| 10 | 10,280 | 1,084 | 6,739 | 37% |
| 30 | 78,887 | 2,770 | 7,230 | 8.1% |
| 100 | 68,669 | 1,665 | 6,795 | 8.8% |
| 300 | 56,399 | 928 | 7,655 | 11.8% |

So the residual is **intrinsic contention among the physics rules**, which
is P1's substantive claim, but it is not principally the fuse-fuse
node-sharing the plan reasoned from. The corroboration that does hold is
ordinal: normalising total fuse losses by the standing population and the
era length, the per-node-step loss rate ranks the four cells in *exactly*
the same order as β_clean − 1 does (k=30 > k=10 > k=100 > k=300, both):

| k | era steps | fuse losses | per node-step | β_clean − 1 |
|---:|---:|---:|---:|---:|
| 10 | 6,400 | 18,103 | 0.257 | 0.068 |
| 30 | 6,800 | 88,887 | 0.399 | 0.180 |
| 100 | 8,100 | 77,129 | 0.118 | 0.044 |
| 300 | 12,700 | 64,982 | 0.030 | 0.029 |

The ratio between the two columns is not constant (0.26–0.97), so this is
corroboration of the attribution, **not a second estimator**. Stated as
such rather than promoted.

## 3. P2 — the estimators still agree where β is large; less so where it is small

β_L and β_f (the latter referencing neither L\* nor p) agree to **1.2–2.0%
at k ≥ 100** under the passive clock — the regime where β − 1 is
well-determined and where v7's 0.6–2.8% agreement was established. At
k ≤ 30 they diverge by 4.3–13%.

This is not a failure of the decomposition, and it is not hidden: at
k ≤ 30, β_clean − 1 is itself only 0.07–0.18, so a 0.1 absolute
discrepancy between the estimators is comparable to the whole effect.
P2's check retains its force exactly where the quantity it checks is
resolvable, and the honest statement of its domain is **k ≥ 100**.

## 4. P4 — the balance law survives, budget-invariant, with the decomposition exact

β_clean is budget-invariant to 1.5% at every k (1.068/1.083, 1.180/1.176,
1.044/1.050, 1.029/1.033 across E₀ = 1e5 → 4e5). The law stands as

> **L\* = β_clean(k) · k · p**, β_clean = 1.03–1.18 falling with k,
> budget-independent, verified for k ∈ [10, 300] and E₀ ∈ [1e5, 4e5] in
> swap × transfer — and **β_clean = 1.00 in the pump**, where the law is
> exact.

Q7-3's decomposition test re-run at the new β, at E₀ = 4e5:
measured L\*(300)/L\*(100) = 307.4/121.3 = **2.534**;
predicted 3·β(300)p(300)/(β(100)p(100)) = **2.535**.
At E₀ = 1e5: measured **2.128**, predicted **2.127**. The exactness v7
reported is not an artifact of the ticked β — it holds at the clean one.

**Control validity.** The ticked arm reproduces the v7 published numbers
to three decimals (L\* 127.0, 241.6, 188.4, 534.5; β 1.577, 1.836, 1.619,
2.003 against v7's 1.58, 1.83, 1.62, 2.00), from a differently written
driver on a differently obtained census. This is what licenses reading
every difference in this document as back-action rather than as a change
of measurement.

## 5. The back-action itself

| regime | what the instrument did to the measurement |
|---|---|
| swap | Inflated the standing population by **13–74%** (534.5 vs 307.4 at k=300) purely through blocked removal. Production was barely touched (p 0.890 → 0.992 at k=300, ~1.03 → ~1.03 at k=10) because payers are distributed, so blocking one node does not stop the pump. |
| pump | **Destroyed the k-dependence entirely** and throttled production by up to 50× (p 0.020 vs 0.994 at k=300), so the measured "structural cap" was a property of the clock. Also prevented the run from spending 93% of its energy budget at k=300. |

The generalisable statement, and the one that belongs in the methods
appendix: the instrument was **cheap** — under 2% of step time
(`docs/grammar_audit_v8.md`, Q5) — and still did all of the above. Back-
action is not a cost phenomenon and cannot be screened for by profiling.
A rule that claims bindings competes with the physics however little work
it does.

---

## Provenance

- P3 (pump): `exp_l18_pump_cap`, 24 runs, pump × vanish × E₀ = 1e5 ×
  k ∈ {10,30,100,300} × {countdown, age} × 3 seeds (seed-invariant per §0).
- P1/P2/P4 (swap): `exp_l18_swap_balance_law`, 16 runs, swap × transfer ×
  E₀ ∈ {1e5, 4e5} × k ∈ {10,30,100,300} × {countdown, age}, seed 42 (the
  plan's 3 seeds were dropped to 1 on the §0 finding, which is a
  correction to the cell list, not a reduction in evidence).
- Every cell's energy audit residual is exactly 0.0; every passive cell's
  `instrument_losses` is 0, asserted before the row is written.
- Standing census is the gen>0 node count over the production era (the
  Q7-1 instrument of record), read off snapshots, with the era read from
  `leakage.csv` so L\* and p refer to one segment.

---

# Addendum (v9 C1 §3.3–3.4, 2026-07-31)

## §3.3 metronome check — REFUTED. β = 1.00's mechanism is open.

F-LK29 proposed that the pump's exact β = 1.00 has a derivable cause:
round-robin extrusion spaces births on any one chain exactly 6 steps
apart, a deterministic lifetime spaces expirations identically, so no two
adjacent nodes are ever simultaneously expired, fuse-fuse contention is
structurally zero, and Little's law holds exactly. The check was
pre-registered as ≈ 0 in the already-logged contention counters.

Read from `runs/v8_l18/pump_vanish_age_k*_s42/guards.csv`:

| k | fuse → fuse | fuse → transport | fuse → subdivide | instrument losses |
|---:|---:|---:|---:|---:|
| 10 | **1666** | 0 | 0 | 0 |
| 30 | **1666** | 0 | 0 | 0 |
| 100 | **1666** | 0 | 0 | 0 |
| 300 | **1666** | 0 | 0 | 0 |

**Not zero.** The spacing mechanism as stated is refuted, and per v9
hazard 6 the pump's β = 1.00 is reported as **measured and unexplained**;
no replacement mechanism is proposed in this round.

One measurement is recorded alongside it, as a fact and not an
explanation: fuse-fuse losses equal **exactly 1666 at every k**, which is
exactly `n_subdivisions` — the full E₀/(q+c) budget — in those same
cells. One fuse-fuse contention per subdivision event, invariant in k.
The sharper form of the open question is therefore not "why is there no
contention" but **why Little's law is exact despite there being
contention**, at a rate of precisely one event per production event.
That question is left open rather than answered.

(For contrast, the same table shows the passive clock's BG6 holding: zero
instrument losses in every pump cell. And `lk_fuse` loses to nothing else
at all in the pump — the 1666 are its entire contention budget.)

## §3.4 k-split hypothesis — UNTESTED (Inferred), not promoted

The v8 candidate for β − 1 falling with k — young expirees dying into
energetically active neighbourhoods, old ones into equilibrated ones — is
a claim about the *local* state around each expiring node. `guards.csv`
carries per-window aggregates only, so the decisive quantity is not in
the persisted data.

What could be computed, and why it does not resolve it:

- Within each swap cell, per-window fuse losses correlate strongly with
  per-window transport activity (r = 0.98, 0.93, 0.79, 0.80 at
  k = 10, 30, 100, 300). This is dominated by the shared era envelope —
  both quantities rise and fall with the production era — and is not
  evidence about neighbourhoods.
- Normalised across k, transport evals per standing node-step fall
  monotonically (549.7 → 39.2 from k = 10 to 300), but that is the
  standing population growing against a fixed lattice, i.e. a
  normalisation artifact rather than a change in local activity.
- The ratio of fuse losses to transport activity is non-monotone
  (0.0005, 0.0021, 0.0015, 0.0008), peaking at k = 30 — the same cell
  where β − 1 peaks. Four points, confounded, no per-node information.

**Verdict: untested.** Recorded at certainty `Inferred`, not promoted to
a mechanism, and no runs are commissioned for it (v9 §3.4). Resolving it
needs per-node instrumentation — the age of each fused node together
with its neighbourhood's transport activity at the moment of fusion —
which is a new instrument and, by v9 §1.1's own design order, would need
its back-action characterised before it produced a result.
