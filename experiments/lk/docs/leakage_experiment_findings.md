# Locality & Leakage — Experiment Findings (LK track)

## v10 round (2026-08-05): the generality round (G0…) — in flight

Plan: hypersim_leakage_plan_v10.md. Removes the paper's four named
scaffolds one axis at a time.

### G0.1 — the guard-shadow audit: F-LK33's conclusion holds, its mechanism is backwards

Full record: `docs/guard_shadow_audit_v10.md`.
Driver: `crates/hg-sim-core/tests/g0_guard_shadow.rs`.
Artifact: `runs/v10_g0/guard_shadow_audit.log` (2 tests, exit 0).

`lk_fuse`'s generation/degree guard pair, audited as a pair for the first
time (v10 §1.4), over 182 runs with guard counters and 19,323 persisted
snapshots.

- **The order claim is REFUTED.** `bind_free_vars` chains guard *families*
  with `&&` — property → compares → degree → age — so the generation guard
  runs FIRST, and the degree guard sees only what generation admits. The
  degree guard rejected **0** bindings in 8.25×10⁸ evaluations and was
  **never evaluated at all** in 65 runs (the `f_cap` arms, where prop1
  short-circuits ahead of it). The inert instrument of the pair is the
  DEGREE guard — the exact inverse of the prediction, which was read off
  declaration order in the TOML. **Declaration order is not evaluation
  order**; proposed as a standing rule for v11 §1.
- **The reachability claim is CONFIRMED.** No gen-0 node reaches degree 2
  in 19.2M stamped observations, on either the clean-lattice (degrees
  `{6}`) or the planted-handle (`{6,7,8}`) population. Deleting the
  generation guard would admit nothing: the v4 §1.4 non-removal guarantee
  is the **degree** guard's. The mechanism is that gen-0 degree is a
  topological invariant of the family — no rule changes any node's degree
  — and the counters witness it independently: generation's rejects per
  window are *exactly* constant in all 182 runs.
- **Unasked-for third result.** Every gen>0 node in every snapshot is at
  exactly degree 2 (`{2: 14,252,898}`) — no branch point has ever formed.
  So both guards are individually redundant, but asymmetrically:
  generation's redundancy is **provable**, degree's is merely
  **contingent** on no process yet creating a branch point.
- **A near-miss worth recording.** The audit's first pass fired §2.1's
  erosion alternative on 3.5M nodes. Attribution: `expL_v2_archive` and
  `exp_l10` carry the generation property on **zero** nodes and `exp_l12`
  on 96% — v3-era runs predating the stamp, where the whole emergent
  filament reads as skeleton and every midpoint is degree 2 by
  construction. In the stamped fraction the count was zero everywhere. The
  test now measures only where the property exists, prints the blind spot
  rather than dropping it, and asserts the family list so the scope cannot
  go stale. The distance between "the lattice is eroding" and the truth was
  one unchecked instrument precondition.
- **Corrections shipped:** paper §1's scaffolding bullet (was
  "generation-guarded fusion") and `rules/lk_fuse.toml`'s v8 comment (which
  said the generation guard is what keeps the lattice non-removable and
  must not be weakened — wrong in both halves).
- **G3 disposition unchanged: defer**, now on proof rather than inference.
  Eroding the skeleton needs the *degree* guard relaxed, i.e. vertex
  contraction, a different operation from a chain-link unzip.
- **v10 App. B amended:** `guards.csv` needs no `shadowed_by` column.
  Precedence is already recoverable — each family's `evals` equals the
  previous family's `evals − rejects` — and this audit needed no schema
  change. The cascade identity is documented instead.

## v9 round (2026-07-31): close-out (C0…C3) — two pre-registrations, both refuted

Plan: hypersim_leakage_plan_v9.md. Almost entirely zero-compute
re-analysis of persisted runs. **Both of this round's pre-registered
predictions came back negative, which is what pre-registering them was
for.**

### C0 — the ensemble decision (GATE)

Written before the estimator ran (`docs/leakage_closure_spec.md`). CD is
declared a property of **this deterministic trajectory**, with two
separately-named uncertainties: a moving-block bootstrap
(b = 20 windows headline, sensitivity at 10/40, B = 1000, percentile
2.5/97.5, deterministic seed) and `sampler_spread` across the five runs.
The decision rule requires **both** — interval excludes 0 *and* point
estimate exceeds sampler_spread. Claim language fixed: "this
trajectory's closure defect", family-level claims explicitly out of
scope. The IC-ensemble alternative is deferred with its tradeoff owned:
on a vertex-transitive torus with zero-draw scheduling, every available
symmetry-breaker changes the system rather than the realization.

One refinement was made during implementation and recorded rather than
made silently: emitting a **mask=false separator between bootstrap
blocks** removes every spurious join transition (`estimate_stratum`
retains a transition only when `mask[i] && mask[i+1]`), which is strictly
tighter than the limitation the memo first disclosed. Adopted **before**
any attempt-3 number existed.

### C1 §3.1 — the corrected mask: the v8 prediction is REFUTED

The exact census needed no snapshots: the gen-0 lattice is non-removable
under the age gate, so **census = n_nodes − 1024 exactly**, and
`n_nodes` is already in `metrics.csv` (sanity-checked: it never dips
below 1024).

- **Predicted:** the quartile-4 variance inflation disappears (it was the
  sampled census growing noisy at its detection floor) and the variance
  ratio enters [0.5, 2.0]. **Neither happened.** The ratio got *worse*
  (0.31 → 0.17) on all five runs, and **two** quartiles are inflated, not
  one: 367.8±82.9 | 419.8±6.8 | 421.3±6.4 | 431.0±217.6.
- **The exact census supplied the diagnosis the sampled one could not.**
  A metered run has three phases: a **fill ramp** (~30k steps, driven by
  the production rate itself climbing as energy spreads from the seed —
  not by the k=300 lifetime), a **plateau** (census 411–426, the tightest
  steady state in the track, sd/mean ≈ 1.6%), and a **quench** as the
  budget empties (p 1.57 → 0.40 → 0, population drains to 0 by ~150k;
  the remaining half of the 300k-step run is dead).
- **So the fourth obstacle is: the mask's `producing` clause tests a
  RISING COUNTER, not a steady rate.** `n_subdivisions` keeps ticking up
  throughout the quench, so the stratum straddles a transient at each end
  and the variance test compares transient against steady state.
- **v8's correction was right, necessary, and insufficient** — it is what
  made this structure visible at all.
- **Exclusion branch executed as written:** Q1–Q3 clears effective N
  (278–283) and fails P2 far worse (ratio 112) — it drops the quench and
  keeps the ramp.

### C1 §3.2 — attempt 3 did not run

0/5 authorized. P1 passes on every run (eff N 240–248), P2 fails on every
run. **No CD number was computed and none is quoted.** The plateau is
visibly stationary and is deliberately **not** adopted as a stratum:
selecting a window after seeing where the variance is small is the error
this programme has already paid for twice. A production-*rate* criterion
(±20%, band fixed in advance) is pre-registered for the next round, with
its own likely failure stated — ~950 plateau windows at ρ≈0.70 gives
eff N ≈ 165, below the floor.

### C1 §3.3 — the metronome check: REFUTED

F-LK29 predicted fuse-fuse contention ≈ 0 in the pump, which would have
upgraded β = 1.00 from observed to derived. Measured: **1666 at every k**
— exactly `n_subdivisions`, the full budget. One fuse-fuse contention per
production event, invariant in k. Per hazard 6 the exactness is reported
as **measured and unexplained**, with no same-round retrofit. The sharper
open question is now *why Little's law is exact despite contention at
precisely one event per production event.*

### C1 §3.4 — k-split: UNTESTED

The hypothesis is about the local state around each expiring node;
`guards.csv` carries per-window aggregates only. Within-cell fuse-loss /
transport-activity correlations are high (0.79–0.98) but dominated by the
shared era envelope; across-k normalisations are population-size
artifacts. Recorded at certainty `Inferred`, not promoted, no runs
commissioned.

### C2 — hygiene

- `NodeData::generation` → **`birth_step`**, with
  `#[serde(rename = "generation")]` preserving the persisted name.
  Acceptance check is a real archived v2-era snapshot loading through the
  production reader (`archived_snapshot_round_trips_after_birth_step_rename`).
  Registry gains a disambiguation section for the three "generation"s.
- **`age_gate_without_generation_guard`** shipped (`age_gate_flags`),
  wired into the run's suspicious flags, with a fixture asserting both
  directions — the production ruleset must not trigger it, and a
  guard-stripped `lk_fuse` must.
- `seed_spread_bits` → **`sampler_spread_bits`** plus a
  **`spread_ensemble`** enum (`folds | sampler | bootstrap | ic`).
  `serde(alias)` keeps archived `closure.json` readable; the golden
  schema test caught the rename and was updated deliberately.

### D4 — did not convene

Contingent on a valid CD at k = 300. None was produced, so no companion
cells were commissioned.

## v8 round (2026-07-30): the instrument comes out (B0…B4)

Plan: hypersim_leakage_plan_v8.md. v7 found that `lk_tick`, added as a
passive measurement clock, was the dominant scheduler actor in the runs it
existed to observe. v8 replaces it with a clock that claims no bindings
and measures what that was worth.

Full deliverables: `docs/grammar_audit_v8.md` (B0),
`docs/back_action_v8.md` (B2, the table and the P1–P4 verdicts).

### B0 — the birth step was already in the store

**Gate: PASS, at design order (a) rather than (c).** The plan budgeted for
a new property key (`birth_step`) plus a new effect op to stamp it. Not
needed: `NodeData::generation` is populated with `store.step()` at every
creation site and never written again, and `GraphSnapshot::step()` is
already in scope where guards are evaluated. The only addition is a guard
*kind*, `node_age_compares`, evaluated in the same post-candidate filter
as the property and degree guards — so it touches neither matcher
enumeration nor conflict resolution, and the §2 stop condition does not
fire. **No key was reserved; key 8 is still free; key 7 is retained
unreused so archived ticked runs stay readable.**

- **Hazard §8.1 does not fire, for a reason worth recording.** The plan
  worried that a snapshot loaded into a run whose step counter restarts
  makes every stamped node instantly ancient. The counter does not
  restart: `step` is a serialized field of the store, and the runner's
  `for step in 0..max_steps` is a separate loop variable. Birth steps and
  the current step stay in one frame across a load.
- **The real semantic change is the opposite one, and it is a loss.** The
  countdown guard protected the original lattice for free — gen-0 nodes
  carried no key 7 and a missing property rejects. An age guard cannot:
  every node has a birth step and IC nodes read as born at step 0, hence
  maximally OLD, passing the guard at every k. Protection now rests
  entirely on lk_fuse's own generation and degree guards. Asserted in two
  places rather than commented.
- **Cost delta: 0.7–2.1%, and it does NOT offset the budget.** Matched
  state, medians over 9 interleaved repeats (a single A-then-B pass swung
  ±50% on identical inputs and would have been read as signal). Conflict
  resolution gets 18–25% cheaper; matcher enumeration barely moves
  (0.3%), because the tick's isolated-var scan is O(N) with a trivial
  constant while the multi-edge rules dominate by two orders of
  magnitude. **This sharpens F-LK25 rather than softening it: the
  instrument was cheap and still dominated the scheduler. Back-action is
  not a cost phenomenon and profiling will never find it.**

### B1 — the passive clock, and BG6 as a structural rather than statistical gate

BG1–BG6 all pass. BG2 is the one that makes EXP-L18 a controlled
comparison: on the *same* scripted throat AG2 used, a link fuses on
exactly the step `step − birth_step` first reaches k — the same step the
countdown mechanism fused on. The two clocks are semantically identical;
only their scheduler footprint differs. BG6 is structural: with no
instrument rule loaded there is nothing for the contention counter to
match, so `contention:instrument_losses` is 0 by construction — and is
still written every window as an explicit zero, because a missing row and
a zero row must not be the same observation.

§3.3's micro-feature exclusion was extended rather than inherited: the v6
guarantee covered *properties*, and the birth step is a **field**, so
`micro_features_ignore_birth_step_and_age` pins it on its own channel.

### B2 / EXP-L18 — the pump cap was an instrument artifact (P3), and β falls (P1)

**A correction to the plan's cell list, forced by the first output:
"3 seeds" is not 3 replicates.** Under MaxParallel the seed does not vary
the dynamics — conflict resolution is a deterministic greedy pass and no
LK-R rule draws RNG, so the step/energy/subdivision/fusion/churn
accumulators are byte-identical across seeds. Only the sampled
instruments (the 256-edge detour sampler, spectral estimates) differ.
Pinned in both directions by `exp_l18_seed_does_not_vary_the_dynamics`.
**This reaches backwards into the CD tuple protocol:** `seed_spread`
estimates instrument sampling noise with the dynamical component
identically zero. Still a valid floor, but not the replication its name
implies.

- **P3 CONFIRMED, decisively. The pump cap does not exist.** L\*/k =
  0.986, 0.995, 0.999, 0.999 at k = 10, 30, 100, 300 under the passive
  clock, against a previously reported flat L\* = 12 across the same 30×
  range. p = 0.993–0.994 at every k (the one-event-per-step
  serialization limit) where the tick gave 0.541 → 0.020. `n_sub` reaches
  the full budget 1666 in every passive cell, against 1164 / 360 / 120
  under the tick — **the clock stopped the pump from spending its own
  energy**. β_clean = 1.00: the balance law is exact in the pump.
  Serialization pins **p**, not L\*; the prior claim conflated them.
- **P1 splits by k, and the split is informative.** β_clean = 1.03–1.05
  at k ≥ 100 (the plan's named alternative — fuse-fuse negligible,
  L\* = k·p essentially exact) and 1.07–1.18 at k ≤ 30. Instrument
  blocking **rose** with k; intrinsic blocking **falls** with k. The
  ticked β looked like one smooth function of k and was a sum of two
  opposite dependences.
- **P1's attribution is refuted in its specifics.** The residual was
  predicted to be checkable against the fuse-loses-to-fuse column. The
  directional counter says `lk_fuse` loses overwhelmingly to
  **`lk_transport`** (fuse-fuse is 8–37% of fuse's losses). The residual
  is contention among the physics rules — P1's substantive claim — but
  not the fuse-fuse node-sharing the prediction reasoned from. What does
  hold is ordinal: per-node-step fuse-loss rate ranks the four cells in
  exactly the same order as β − 1. The ratio is not constant (0.26–0.97),
  so this is corroboration, **not a second estimator**, and is reported at
  that strength.
- **P2 holds where β is resolvable.** β_L and β_f agree to 1.2–2.0% at
  k ≥ 100, 4.3–13% at k ≤ 30 — where β − 1 is itself only 0.07–0.18, so
  the discrepancy is comparable to the effect. Domain stated as k ≥ 100.
- **P4 CONFIRMED.** β_clean budget-invariant to 1.5%. Q7-3's
  decomposition re-verified at the new β: measured L\*(300)/L\*(100) =
  2.534 vs predicted 2.535 at E₀ = 4e5, and 2.128 vs 2.127 at 1e5.
- **Control validity.** The ticked arm reproduces the v7 published
  numbers to three decimals from a differently written driver on a
  differently obtained census. That is what licenses reading every
  difference as back-action rather than as a change of instrument.

**Back-action, quantified:** in swap the instrument inflated the standing
population by 13–74% purely through blocked removal while barely touching
production (distributed payers). In the pump it destroyed the
k-dependence entirely and throttled production up to 50×.

### B3 / EXP-L7R attempt 2 — the sample precondition PASSES; stationarity fails; the gate is what broke

Full record: `docs/leakage_closure_spec.md`, "attempt 2".

**Pricing (D3-A, commission).** Pilot at k=300 under `gate="age"`:
3.67 → 3.76 ms/step, eff N 14.5 → 46.9 across E₀ 1e5 → 4e5. **My first
pricing driver projected eff N LINEARLY (234, clears the floor). Its own
pilot refutes that** — eff_N ∝ E₀^0.847, giving 183, short. Fixed to
print both and declare STRADDLES-THE-FLOOR when they disagree across it;
same error family as A3's "finer cadence buys more windows". The
per-step-cost flatness check (×1.02 over a ×4 span) was also passing
while saying nothing about a ×5 extrapolation, and now fits a cost
exponent and flags out-of-span targets. Since the price was low, the run
was commissioned to **measure** effective N rather than project it.

**Outcome at E₀ = 2e6, 300k steps, 5 seeds, passive clock (0.44 h/seed):**

- **P1 PASSES for the first time in the track**: effective N = 239–249
  against the floor of 200, raw strata ~1380 windows. Decisively:
  **ρ did NOT rise with segment length** (0.722 at 4e5 → 0.694–0.704
  here), which is the assumption the whole extrapolation rested on. The
  linear projection (234) was nearly right and **my sublinear correction
  was itself an over-extrapolation from two points** — the conservative
  error, but an error.
- **P2 FAILS on all 5 seeds**: drift fine (5.8–8.6%), variance ratio
  0.31–0.36 against [0.5, 2.0].
- **The failure is the GATE, not the regime.** Quartiles 1–3 are
  stationary (means 291.5/299.4/297.6, sds 74.0/56.2/60.1); quartile 4 is
  not (mean 330.6, sd 151.9). The stratum ends because
  `wormhole_count_est` reads 0 — while `n_subdivisions` keeps climbing to
  the end of the run and the phase stays Sparse. Production does not stop
  and the phase does not change: **the sampled, threshold-limited census
  crosses its own detection floor**, growing noisier as it approaches it.
  That is exactly the Q7-1 instrument the paper's §5 already replaced
  with the gen>0 census; the EXP-L7R mask was never updated.
- **NOT re-run with a corrected mask in this round.** Fixing an
  instrument after seeing which way it made a gate fail is post-hoc even
  when the fix is right. Pre-registered for v9 with its prediction and
  the alternative it excludes (if the variance ratio still fails on the
  corrected mask, the destabilization is real and the stationarity
  milestone's scope shrinks further).

**CD is unmeasured for the third distinct reason** — v6: no stationary
regime; v7: effective sample 5× short; v8: stationarity gate reading a
retired instrument. Each obstacle identified and quantified, none of the
bars relaxed.

### Correction to the v7 close-out: the suite was not green

The v7 round was recorded as ending with "full workspace tests green".
It did not. `phase10_pipeline::headless_run_produces_csv_report_manifest_with_zero_listeners`
was failing, and had been since v7's directional contention counters
began emitting `contention:<winner>` rows — that makes `guards.csv`
appear on any run with contention, and the test asserted the run
directory contains only `metrics.csv` and `leakage.csv`. The v8 suite run
caught it; the assertion is corrected to admit `guards.csv` as the third
leakage-track artifact (the invariant it guards — exactly one *metrics*
CSV — is checked separately and was never at risk).

Recorded here rather than quietly fixed, because the failure mode is the
one this track keeps cataloguing: a green-suite claim that was not
verified against the actual output.

## v7 round (2026-07-19): audit-and-close (Q7-1…Q7-6)

Plan: hypersim_leakage_plan_v7.md. Four open items in the A4 result,
settled before the paper ships.

### Q7-1 — census-zero: threshold, not sampling; and the domain is regime-dependent

Full deliverable: `docs/chain_census_l17.md`. Three results:

1. **Every census zero is out-of-range, not sampling.** An exact,
   unsampled wormhole count over all filament-incident edges agrees with
   the shipped 256-edge sampler in every cell — including reading exactly
   0.0 wherever the sampler reads 0.0 (and tracking it within a few
   percent where nonzero: 33.8 vs 38.7, 54.4 vs 58.8, 165.1 vs 190.2,
   856.0 vs 861.4). The plan's "sampling artifact" row is refuted by
   direct measurement.
2. **The detour law is REGIME-DEPENDENT** — measured, not assumed:
   ≈ 2L+2 in the pump (F-LK14's formula, which holds because the seed's
   own bulk edges are consumed into chains so a return path descends a
   *second* chain) but ≈ **L+3** in swap, where the ambient lattice
   around a filament is intact. Visibility at R_max = 12 is therefore
   L ≥ 6 (pump) but **L ≥ 10 (swap)**. Applying the correct threshold,
   every reading in the sweep is consistent and no anomaly remains.
3. **The swap population is not "~15 chains".** That figure came from
   EXP-L12, a no-removal configuration. Under removal the population is
   many more, much shorter structures whose count grows with k (2.7 →
   23.1 vanish, 12.8 → 55.5 transfer) with a **median chain length of 1
   at every k** and a heavy tail (max 80.9). Only 7–14% of chains are in
   range even where the census registers. The pump holds **exactly 6.0
   chains × ~2.0 links** at every k.

**Consequence:** for k ≤ 30 (and the pump at every k) the coexistence
regime is entirely local by the track's own definition — sub-threshold
refinement, reported as the clean result it is. At k = 100–300 in swap
there is a genuine standing wormhole population (54–165 edges, exact).
The v7 §1.1 instrument-range precondition now ships as a schema field,
`LeakageRow.census_in_range`, computed exactly (the detour of an edge on
the longest filament chain) so it encodes no regime formula.

### Q7-2 — tick-blocking: CONFIRMED, and it blocks production too

Full deliverable: `docs/contention_audit_v7.md`. **Concurrency failure
#7**, and larger in scope than F-LK22 predicted.

- **Check 2 (signature):** ~half the standing population is expired but
  still present (50–57% pump, 26–51% swap), **77–99% of removal-ready
  nodes are blocked by a still-ticking neighbour**, the blocked share
  rises with k, and the unblocked ones are almost entirely at chain ends
  (a gen-0 anchor neighbour is never tick-eligible). The k=30 swap cell
  is the control that proves rather than fits the mechanism: chains are
  single links there, both neighbours are anchors, blocking is only 19%,
  and residence/k is the lowest in the sweep.
- **Check 1 + 1b (quantitative, non-circular):** residence/k from
  Little's law (L\*/p) and from the expired share alone (1/(1−f), which
  references neither L\* nor p) agree to **0.6–2.8% across all seven
  cells**. Blocking accounts for the *entire* residence excess, leaving
  nothing for the era-averaging the paper currently credits.
- **Check 3 (direct instrument):** a directional contention counter
  (`resolve_conflicts_counted`, pinned byte-identical to the legacy
  resolver) shows everything losing to `lk_tick`: in 3000 pump steps at
  k=100, `lk_fuse` loses 34,206×, `lk_transport` 17,520×, and
  **`lk_metered_subdivide` 16,962×**.

**The unanticipated half:** `lk_tick` sits at priority 2, above
subdivide and transport, so it blocks *production* as well as removal.
The countdown mechanism, introduced as a passive measurement clock,
is the dominant scheduler actor — **measurement back-action**, the
round's most general lesson. Per hazard §8.2 this is measured and
reported, **not fixed**, in v7; a tick guard or priority change is a v8
option because either alters the lifetime semantics of the A4 sweep it
would explain.

### Q7-3 — the k=300 departure was a STATISTIC artifact; the linear law is restated

Pre-registered test: rerun k=300 at 4× budget (E₀ 1e5 → 4e5, transfer
mode), with k=100 as the control. Measured with Q7-1's **instrument of
record** (gen>0 standing census — the Q7-3 driver's own `L*` column
reads the sampled wormhole census, which Q7-1 showed undercounts, so the
cells were re-measured before interpretation).

| E₀ | k | L\* | p | L\*/k | L\*/(k·p) | 1/(1−f) from expired share |
|---|---:|---:|---:|---:|---:|---:|
| 1e5 | 100 | 127.0 | 0.805 | 1.27 | **1.58** | 1.59 |
| 1e5 | 300 | 241.6 | 0.439 | **0.81** | **1.83** | 1.79 |
| 4e5 | 100 | 188.4 | 1.164 | **1.88** | **1.62** | 1.61 |
| 4e5 | 300 | 534.5 | 0.890 | 1.78 | **2.00** | 1.96 |

- **The departure is resolved: it was never a saturation.** `L*/k`
  conflates the production rate, which is budget-sensitive (p doubles
  from 0.439 to 0.890 at k=300 when the budget quadruples). The
  budget-INVARIANT quantity is `L*/(k·p)` = residence/k = the blocking
  factor β(k): 1.58 → 1.62 at k=100 and 1.83 → 2.00 at k=300 across a 4×
  budget change, and independently predicted by the expired share to
  within 2% in all four cells.
- **But the paper's linear law does not survive as stated.** `L*/k` is
  *not* a constant ≈ 1.27: it is 1.27 at (k=100, E₀=1e5) and 1.88 at
  (k=100, E₀=4e5). The 1.27 figure was budget-specific. The correct
  statement, with its domain per v7 §1.3:

  > **L\* = β(k) · k · p**, where β(k) = 1/(1−f) is the blocking factor
  > measured from the expired share (≈1.6 at k=100, ≈1.9–2.0 at k=300,
  > budget-independent) and p is the realized production rate (measured
  > per cell; budget- and regime-dependent, not predicted). Verified for
  > k ∈ [10, 300] and E₀ ∈ [1e5, 4e5] in swap × transfer.

  The naive L\* = k·p is the β → 1 (unblocked) limit, approached only
  where chains are single links so both neighbours are gen-0 anchors.
- **The decomposition is quantitatively exact.** At E₀=4e5 the measured
  L\*(300)/L\*(100) = 2.84; predicted from 3 · β(300)p(300)/β(100)p(100)
  = **2.84**. The apparent sublinearity in k is entirely β and p moving
  in opposite directions (β up with k via blocking, p down with k via
  the same blocking).

### Q7-5 / EXP-L7R — preconditions NOT met; CD stays unmeasured, shortfall quantified

Full record: `docs/leakage_closure_spec.md`. swap × transfer × k=300 ×
E₀=4e5, 5 seeds, steady segment production-rate-defined. Audit exact
everywhere.

- **The stationarity precondition HELD** — 600/600 windows Sparse in
  every seed, and seeds 42/44 passed the pre-registered drift and
  variance tests. This is the first time the track has had a stratum that
  genuinely does not cross phases.
- **The sample-size precondition FAILED, by ~5×.** Lag-1 autocorrelation
  of the micro features is ρ ≈ 0.76–0.80, so the **effective** sample
  N(1−ρ)/(1+ρ) is **31–41 against the floor of 200**, despite raw
  stratum lengths of 225–351 windows.
- **Self-correction:** the first version of the Q7-5 driver checked the
  *raw* count, authorized two seeds, and printed "first valid
  leakage-in-bits measurement" with CD_clock = 3.06 / 2.78. **That
  verdict is withdrawn** — A3 had already restated the floor as an
  effective-sample requirement and the driver had not implemented the
  restatement. The driver now computes effective N; the numbers are
  recorded as not authorized.
- **The requirement is now quantitative:** τ ≈ 400 steps ⇒ a steady
  segment of **≈1.4–1.7 × 10⁵ steps (≈5× longer)**, i.e. E₀ ≈ 2 × 10⁶ at
  these parameters. Cadence cannot substitute (effective N = T/2τ is
  cadence-independent — the A3 result).
- **Diagnostic, not a claim:** CD_clock > CD_raw here, the reverse of
  every nonstationary run in the track's history, which is what a truly
  stationary series should give (the window index is uninformative in a
  steady state). Consistent with the stratum being what it claims; not
  evidence about leakage.

### Q7-4 — the pump cap: a genuine throttle, mechanism identified

Pre-registered row 1 lands. Production rate falls as 1/k —
p = 0.541, 0.194, 0.059, 0.020 at k = 10, 30, 100, 300, so k·p is flat
at 5.41 → 6.00 — while L\* stays at 12.1–12.4. The relation is
L\* = 2·k·p exactly, because blocking makes residence 2k (confirmed
independently in Q7-2 check 1b: 2.00–2.33 across the pump k-sweep).

**Mechanism, now identified rather than named:** the same clock gates
both directions. `lk_tick` refuses the fuse that would shorten a chain
*and* the subdivision that would lengthen it, so production and removal
stay locked in proportion and the population sits at the structural
fixed point Q7-1 measured to the decimal — **6.0 chains × ~2.0 links =
12**. The paper's "production self-throttling" sentence, which named an
effect without a mechanism, is replaced by this.

## v6 round (2026-07-18): A0–A1 (grammar audit, transfer ledger)

Plan: hypersim_leakage_plan_v6.md.

### A0 — grammar audit (GATE): PASS

`docs/grammar_audit_v6.md`. All five questions answered empirically
(`crates/hg-engine/tests/tick_cost_a0.rs`). Key results: a single-node
LHS decrement rule needs **zero** matcher/engine change (the isolated-var
path already handles it); single-node bindings are node-disjoint so all
fire every step (parallel clock, verified against `conflict.rs`); the
`countdown > 0` guard is its own floor (no negative); tick cost is O(N)
at ~13–57 ns/node, i.e. ~40–450 µs/step at the EXP-L17 scale (~3–8k
nodes) — acceptable, no §4.3 fallback; key 7 (`countdown`, Int) reserved
and round-trips through the snapshot codec. **Fifth appearance of the
concurrency class, owned in advance:** a tick binding can lose its node
to a competing edge-rule (transport) the same step, so effective tick
rate < 1 on transported nodes — exactly what §4.2's `k` vs `2k/3`
alternatives discriminate; AG2 measures it rather than assuming.

### A1 — transfer-on-fuse ledger: energy conserved in-graph, era ~6× longer

`crates/hg-sim-core/tests/a1_transfer_ledger.rs`; engine mechanism
`RewriteTemplate::transfer_energy_to_survivor` (generic, injected by the
runner when `ledger_mode = "transfer"`; `vanish` default is byte-identical
to v5, canonical-hash regression green). On fuse, w's energy relocates to
the lowest-generation surviving endpoint (tie-break lowest NodeId) instead
of vanishing.

- **Audit closes on `E_total + c·N_sub = E₀` exactly**, with the vanished
  ledger term staying 0 — energy conserved in-graph, as the plan requires.
- **Era stretch confirmed:** at (q=50, τ=15, c=10, E₀=1e5), transfer runs
  6768 subdivisions vs vanish's 1235 (5.5×; predicted E₀/c ÷ E₀/(q+c) =
  6×) — and transfer is *still producing* at 14k steps (c·6768 = 67,680 of
  100,000 dissipated), so only the c sink bleeds per cycle, exactly the
  §3 mechanism. Attacks the window-floor problem from the production side.
- **No limit cycle (hazard 3):** churn/subdivision is 0.20 in transfer
  (1371/6768) vs 0.78 in vanish (958/1235) — the re-armed energy spreads
  rootward, it does not grow-fuse-grow at one site. **Pre-registered
  transfer-mode churn threshold (before A4): reference ratio ≤ 0.3; flag
  a churn/n_sub ratio > 0.5 in any A4 countdown+transfer cell as a
  candidate grow-fuse-grow limit cycle for diagnosis.**

### A2 — countdown removal mechanism: the first dispersed gate variable

`rules/lk_tick.toml` (single-node LHS decrement), runner-injected when
`[lk_rules.fuse] gate = "countdown"`: subdivision endows the child
`countdown = k` (key 7, Int); `lk_tick` decrements every live countdown
each step; fuse admits at `countdown == 0` (∧ gen ≥ 1 ∧ degree == 2).
Nodes born at different steps carry different countdowns — dispersion by
construction, the §1.2 precondition. Gates
(`crates/hg-sim-core/tests/a2_countdown_gates.rs`), all green:

- **AG1** lk_tick guard engaged: evals > 0 AND rejects > 0 (not a
  never-evaluated instrument).
- **AG2** lifetime exactness: on a scripted throat with countdown k=3, w
  ticks k, 2, 1, 0 (exactly −1/step) and fuses on the step *after* it
  reaches 0 — lifetime = k+1 steps, exact, no transport competition.
- **AG3** dispersion: mid-era interior countdown distribution has
  p10 ≠ p90 (contrast the v5 energy gate's delta-at-plateau).
- **AG4** audit exact in BOTH ledger modes (vanish: residual 0; transfer:
  E_total + c·N_sub = E₀, e_fused 0).
- **AG5** determinism: independent runs byte-identical (zero-draw
  preserved through the countdown path).

Priority: `lk_tick` = 2 (above transport 1, subdivide 0, fuse −1 in
transport_first), so on the isolated-chain case ticks are never stolen
and lifetime is exactly k; the tick-vs-transport conflict on a populated
filament (F-LK17 class) is what AG2's live analogue and A4's L*=k vs
2k/3 rows measure. §1.3 CD exclusion pinned: micro(w) reads no property
(WL classes exclude the property map; slots are structural), so the
countdown clock cannot inflate CD — `closure.rs::
micro_features_ignore_countdown_and_all_properties`.

### A3 — window floor: STATISTICS-limited, and the limiter is autocorrelation

Full verdict + table in `docs/leakage_closure_spec.md`. Re-windowing a
26k-step run to effective cadence {25…500} (3 seeds) shows CD_clock
pathologically negative at *every* cadence and lag-1 autocorrelation
rising 0.895 → 0.997 as cadence shrinks: 52 → 1040 raw windows buys an
effective sample of ≈ 1.6. **Re-windowing is not a route to the ≥ 200
floor**; the floor is restated as an effective-sample requirement, and
EXP-L7R needs longer *stationary* dynamics. (v6 §1.4's "window count,
not era length" hope is refuted for this estimator.)

### A4 / EXP-L17 — coexistence EXISTS, but L\* is independent of k

Runs `runs/exp_l17/`; driver `tests/exp_l17.rs`; standing-population
instrument `crates/hg-eval/tests/l17_standing_census.rs`. Audit exact in
every cell, both ledger modes.

**Two corrections were needed before any row could be read, both owned:**

1. *Driver bug (mine).* The first pass defined the steady segment as
   "second half of a 30k-step run", which at E₀=1e5 lands entirely
   POST-QUENCH (era = E₀/(q+c) ≈ 1666 events ≈ 1666 steps). p = 0 there,
   so L\* = k·p = 0 held trivially and every cell read EMPTY. A
   pre-registered statistic satisfied without the mechanism being tested
   — the exact failure mode §1.1 was written to prevent. Fixed: the
   segment is now the windows where production is live (n_sub rising)
   and the fill transient (step > k) has passed; a cell with p = 0 is
   `NO_ERA`, not a result.
2. *Instrument threshold.* `wormhole_count_est` reads **0 in every
   countdown cell** — but that is the instrument's detection threshold,
   not the population. An interior link registers as a wormhole only
   once its detour 2L+2 exceeds R_max = 12, i.e. on chains of length
   ≥ 6 (Q0/F-LK14); a standing population of ~12 links spread over the
   pump's 6 chains is ~2 links/chain, structurally present and entirely
   invisible to it. **Third instrument-inertness case in the track**
   (after EXP-L9's x-cap and v5's f_cap), and the first where the
   inert instrument would have produced a confident FALSE NEGATIVE.
   The standing population is therefore read as the generation > 0 node
   count off the persisted snapshots — no threshold, and exactly the
   quantity L\* = k·p predicts.

**Pump results (E₀=1e5, seed 42, standing census = gen>0 nodes):**

| cell | L\* | live countdown mean | p | k·p |
|---|---:|---:|---:|---:|
| vanish k=10 | 12.5 ± 0.5 | 5.5 | 0.541 | 5.4 |
| vanish k=30 | 12.2 ± 0.6 | 15.4 | 0.194 | 5.8 |
| vanish k=100 | 12.0 ± 0.7 | 50.6 | 0.059 | 5.9 |
| vanish k=300 | 11.8 ± 1.1 | 153.7 | 0.020 | 6.0 |
| transfer k=10 | 36.1 ± 8.8 | 5.4 | 2.662 | 26.6 |
| transfer k=30 | 35.8 ± 7.3 | 15.8 | 0.956 | 28.7 |
| transfer k=100 | 34.4 ± 7.0 | 50.3 | 0.282 | 28.2 |
| transfer k=300 | 27.1 ± 7.5 | 140.5 | 0.072 | 21.6 |
| ctrl nofuse | **1514 ± 380** (→1666) | – | 0.994 | – |
| ctrl f_cap 40 | **1490 ± 380** | – | 0.994 | – |
| ctrl countdown=0 | **7.0 ± 0.0** | – | 0.994 | – |

- **Coexistence EXISTS.** Every countdown cell holds a bounded standing
  population under sustained production, while both no-removal controls
  run away to the full era's output (1514/1490 = every link ever made).
  A lifetime gate does what the energy gate could not: it holds a
  population instead of choosing between 0 and everything. **This
  answers v6's question affirmatively and overturns the v5 bistability
  reading's scope** — bistability was a property of *energy* gating, not
  of the family.
- **But the mechanism prediction FAILS: L\* is k-INDEPENDENT.** L\* ≈ 12
  (vanish) and ≈ 27–36 (transfer) across a **30× range of k**. This is
  none of the three §4.2 alternatives (L\*=k, 2k/3, unbounded) — it is a
  fourth: a structural cap. The gate itself is provably working — the
  live-countdown mean tracks **k/2** across the whole sweep (5.5, 15.4,
  50.6, 153.7), the steady-state age distribution of a population with
  deterministic lifetime k, which is also the AG3 dispersion holding
  live.
- **The system self-regulates production:** k·p is near-constant
  (5.4–6.0 vanish; 21.6–28.7 transfer) — p falls as ~1/k. So it is not
  that L\* = k·p with p fixed; rather L\* is pinned by something else and
  the production rate accommodates.
- **Ledger mode moves the cap** (12 → ~35), consistent with transfer
  recycling energy rootward and sustaining ~5× more in-flight structure;
  transfer churn/n_sub = 4190/8456 = 0.50 at k=10, at the
  pre-registered A1 flag threshold — recorded, and it falls with k
  (0.16, 0.15, 0.13), so it is a short-lifetime effect, not a limit
  cycle.
- **The cap is HARD:** L_peak = 13 in every pump/vanish cell, at every k.
  Production self-throttles to match it (p ≈ cap/k), which is *why*
  k·p is constant — the causality runs cap → p, not p → L\*. It is not
  budget exhaustion: at k=300 production stops with 93,760 of E₀=1e5
  still in the graph.

**Swap regime (transport_first) — the bottleneck probe, and the payoff.**
EXP-L12 established that swap has DISTRIBUTED payers (max_gen 2, 14–15
chains) instead of the pump's 6 seed-anchored chains. If the pump's cap
is structural, it must move:

| cell | L\* (gen>0) | L\*/k | p | L\*/(k·p) |
|---|---:|---:|---:|---:|
| swap vanish k=10 | 2.7 ± 0.9 | 0.27 | 0.188 | 1.44 |
| swap vanish k=30 | 6.5 ± 1.2 | 0.22 | 0.190 | 1.14 |
| swap vanish k=100 | 36.0 ± 6.8 | 0.36 | 0.177 | 2.03 |
| swap vanish k=300 | 81.8 ± 26.4 | 0.27 | 0.141 | 1.93 |
| swap transfer k=10 | 12.8 ± 5.1 | 1.28 | 1.032 | 1.24 |
| swap transfer k=30 | 37.7 ± 15.1 | 1.26 | 0.956 | 1.31 |
| swap transfer k=100 | 127.0 ± 67.2 | 1.27 | 0.805 | 1.58 |
| swap transfer k=300 | 241.6 ± 188.9 | 0.81 | 0.439 | 1.83 |

- **L\* SCALES WITH k in swap** — 2.7 → 6.5 → 36 → 82 (vanish) and
  12.8 → 37.7 → 127 → 242 (transfer) over k = 10 → 300, with L\*/k
  near-constant at ≈ 1.27 across k = 10/30/100 in transfer. **The
  balance law's form is confirmed where production is distributed**;
  L\*/(k·p) sits at an order-unity prefactor 1.1–2.0 rather than exactly
  1 (see below).
- **So the pump's k-independence is a PUMP-SPECIFIC structural cap, not
  a property of the countdown mechanism.** The discriminator is the
  production geometry: the pump's seed monopoly means only one node ever
  exceeds θ, and under MaxParallel node-disjointness that node can join
  exactly ONE binding per step — production is serialized at a single
  shared anchor, the same mechanism that serialized v5's terminal decay
  tail. Swap's distributed payers remove the anchor and the balance law
  expresses itself.
- **The prefactor is NOT a clock artifact.** A0 Q2 flagged that a
  single-node tick binding could lose its node to a competing edge-rule
  binding, which would give lifetime k/ρ and inflate L\*. Measured
  directly off consecutive snapshots
  (`crates/hg-eval/tests/l17_tick_rate.rs`, Δcountdown/Δstep on nodes
  live in both frames): **ρ = 1.000 exactly** in every cell where the
  cadence resolves it (pump and swap, vanish and transfer, 221–14,654
  paired observations). Ticks are never stolen; realized lifetime is
  exactly k. The residual 1.1–2.0 prefactor is therefore a
  production-burstiness / era-averaging effect (p is a time-average over
  an era in which production is intermittent), not a mechanism defect —
  recorded as the open quantitative residual.
  *(An earlier version of the census tool used its own era heuristic
  rather than leakage.csv's production segment, which included
  post-quench drain windows and inflated L\*/p to ~2k uniformly; the
  numbers above use the same segment for both quantities. Caught and
  fixed before interpretation.)*

**Pre-committed row: COEXISTENCE, with the mechanism qualified** — v6 §6
row 1 in the swap regime (L\* ∝ k), row 2 in the pump (coexistence
exists but L\* ≠ k·p, deviation identified as a structural production
cap). **This overturns the SCOPE of v5's bistability result:** a gate on
a *dispersed* variable produces genuine coexistence, so bistability was
a property of energy gating specifically — exactly what v6 §1.2's
dispersion precondition was written to expose. The family has a
continuous rate knob after all, wherever production is not
anchor-serialized.

**EXP-L7R: still NOT authorized, now for a quantitative reason.** The
best strata are swap_vanish k300 (131 consecutive Sparse windows with
census > 0) and swap_transfer k300 (127) — the first genuinely
*dynamical* standing spectra the track has produced, but still short of
the ≥ 200 floor, and A3 has restated that floor as an effective-sample
requirement (autocorrelation-corrected). A longer swap×transfer×large-k
run at E₀=4e5 is the natural candidate to clear it; not commissioned
here.

## v5 round (2026-07-17): R1 (EXP-L15, relaxation)

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
