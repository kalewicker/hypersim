# G-R1 Mutational-Neutrality Protocol (EXP-R01)

**Status:** specification + implementation (`crates/hg-selfmod/src/gr1.rs`).
Gate: `hg-selfmod gate_gr1`. CI structural test: `tests/gr1.rs`.

## 1. Question

Does the rule encoding admit a navigable mutation→semantics map, or is
discrete rule space brittle regardless of encoding? This gate decides whether
the track continues at all — and it costs CPU-hours, no campaign compute.

## 2. Sampling design

* **Kernel:** the production kernel — `meta::mutate_once_with_op`, the same
  15 operators with the same fixed-draw discipline evolution uses
  (`OP_NAMES` order: jitter_guard, mutate_label, add/drop_pnode_constraint,
  add/drop_lhs_var, add/drop_pedge, add/drop_rhs_create, add/drop_rhs_edge,
  toggle_delete, grow/shrink_arity). Characterizing the real kernel, not an
  idealized one.
* **Kernel gaps (finding, not bug):** the kernel has **no** priority
  mutation, **no** slot-endpoint retarget (arity changes only via grow/shrink),
  and **no** guard key/op mutation (values only). Priority is therefore
  unevolvable in the current kernel — every mutant inherits its parent's
  priority. Recorded here so a later kernel extension is a deliberate
  decision, not drift.
* **Parents:** uniform over `gr1_seed_set()` (≥20 rules, pairwise
  RCF-distinct, all decoding; composition: v2 seeds + abio minimal + churn3
  + churnK + G0/G1 fixtures + RCF bases + D1 tie specs + edge create/delete).
* **One mutation per sample:** install parent alone, apply exactly one
  operator draw (inapplicable draws leave the subgraph unchanged and classify
  Neutral — the effective-clonal channel, counted honestly).
* **Full gate:** `GR1_SAMPLES = 10_000` samples. CI runs 300 samples /
  30-step trajectories (structural only, not thresholds).

## 3. Classification predicates

| Class | Predicate |
|---|---|
| `Quarantined` | decode fails ≥1 validator (validator recorded) |
| `Neutral` | valid and RCF-identical to parent (no trajectory needed; ≤50 spot-checked — RCF-equality ⇒ behavioral-equality soundness tripwire, panics on violation) |
| `NearNeutral` | valid, RCF-distinct, `canonical_hash` identical for all `GR1_TRAJ_STEPS = 100` steps on the fixture graph |
| `Catastrophic` | valid, trajectory differs, and mutant is dead or explosive: zero productive fires over the window, or final nodes/edges > `GR1_EXPLODE_MULT = 4` × initial |
| `Divergent` | valid, trajectory differs, non-catastrophic (information-only extension of the plan's classes; thresholds don't constrain it) |

Fixture graph: `seed_world(64 nodes, 96 edges, 4 labels)` from
`GR1_WORLD_SEED`; trajectories share `GR1_TRAJ_SEED` and unlimited budgets
(the economy is not what is measured). Parent trajectories cached by RCF.
Performance guard (classification-exact): a trajectory stops early once
nodes/edges exceed `4×` initial — explosion already decided, remaining steps
would only burn match cost; the shortened hash vector still differs from any
non-exploded parent and final counts keep the verdict exact.

## 4. Thresholds and pre-committed interpretations

`Valid = Neutral + NearNeutral + Divergent + Catastrophic` (all
non-quarantined). **PASS:** `Valid ≥ 0.40`, `NearNeutral ≥ 0.10`,
`Catastrophic ≤ 0.25`. **Below `Valid = 0.10`:** encoding not searchable —
redesign, do not tune the loop. **0.10–0.40:** proceed only with a smaller
mutation kernel, recorded as an encoding finding. A FAIL is a measurement,
not an implementation error.

**SB-R1 split:** report Valid fraction separately for at-cap parents (any
edge at `max_arity`) vs off-cap parents — arity-boundary mutations
quarantine by construction and must not silently drag the headline.

## 5. Streams and constants

| Constant | Value |
|---|---|
| `GR1_SAMPLES` / `GR1_TRAJ_STEPS` | `10_000` / `100` |
| `GR1_SEED` / `WORLD_SEED` / `TRAJ_SEED` | `0x5231_…0001` / `…0002` / `…0003` |
| `GR1_WORLD_NODES` / `EDGES` | `64` / `96` |
| `GR1_EXPLODE_MULT` | `4` |
| `GR1_NEUTRAL_SPOTCHECK` | `50` |
