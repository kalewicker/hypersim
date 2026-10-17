# Native raw-graph encoder learning: prospective V1 contract

Date: 2026-10-07. Profile `E2-graph-encoder-learning-v1`.
One bounded native trial after the verified zero-native encoder fixture. Native
physics, encoder architecture/parser/update, reward, compiler and old auditors
remain unchanged. No broad training/scientific admission or automatic retry.

## Information boundary and fixed initialization

Learn two 110-parameter encoders from actual live-paid native novelty reward:
ordinary seven-label one-hots and adjacency versus identical label inputs with
adjacency erased. Both use the unchanged two-round width4 message passing,
candidate readout and score gradient. No BFS/anchor-distance/active-triple
feature, task context, initialized-core metadata, literal target or future state
is supplied to either encoder. Known label vocabulary and seeded edit actions
remain scaffolding. Native reward/auditing still uses the old hand-feature
observation to validate this task; it is not routed into encoder logits.

Both modes in all three policy streams start from the exact engineering
initializer: SplitMix64 seed993748720001, 0.15×(2×draw−1), all110 parameters.
Initialization is neither zero nor tuned/warmed up; do not copy teacher-selected
diagnostic updates or trained hand-feature parameters into encoder weights.
Policy draw seeds993748721001/002/003 are fresh; native fixture seeds
993748718001/002 are reused. This varies policy sampling, not physics populations.

Three archived hand-feature final parameter vectors, fixed in config under
their exact prior seal, provide a **frozen positive readout control** on each
fresh exhaustive matrix. This is re-evaluation of historical learned weights,
not a third optimizer or a new matched-training comparison. They never enter
the encoders' logits, samples, gradients or updates.

## Schedule, budget and optimizer

Each seed:144 exhaustive live-paid baseline cells (nine worlds×sixteen action
pairs); twelve balanced batches;144 exhaustive final cells; eighteen correct-
action causal controls. Each batch visits six off-diagonal development pairs in
fixed order, twice each; each sample yields adjacent graph/blind live-paid cells.
Three diagonals [10,11]/[20,21]/[30,31] never enter updates. All nine worlds were
already exposed during engineering: reserved means **optimizer-held-out**, not
unseen topology or developer-held-out validation. Synthetic lure variants are
excluded because their native semantics have not been validated.

Two consecutive SplitMix64 uniforms per sample are shared between policies;
each samples its own actions from its own frozen pre-batch weights. Native seed
is native_seeds[(cell_id/2)%2] with integer division, matched within each pair
and at baseline/final. Persist a decision receipt before invoking Engine, bind
its raw-graph observation to the actual native initial inventory, and persist
the whole native cell before its reward enters an update.

Both receive true live-paid reward, baseline0, rate4, no clipping or warmup:

```text
theta_next = theta + (4/12) × sum(native_reward × joint_score_gradient)
```

Twelve complete samples per mode, two per each development pair, are required.
If either paired batch is invalid/incomplete, skip both updates, retain all
scheduled cells and mark the campaign invalid. Never impute malformed receipts
as zero or omit them. No early stopping, culling, seed replacement, curriculum,
outcome-dependent retry, extra updates or post-hoc criterion adjustment.

Fixed total:864 training +864 endpoint +54 control =1,782 cells×24 =42,768
native attempts/advances;72 updates (36 per mode). Shared native deadline420s;
outer Cargo timeout540s. Pooled native budgets remain50k/50k per step. Diagnostic
code/tests run zero native steps before the single frozen launch.

## Per-seed criteria and exact endpoints

Every seed must satisfy all criteria; no pooled rescue:

- Final graph balanced mean over **all nine worlds** ≥1/24 +0.01.
- Final graph minus actually trained blind balanced mean ≥0.01; blind ≤1/24
  with fixed2e-12 arithmetic tolerance. Removing edges changes effective gradient
  capacity; the mathematical all-policy information ceiling is the main control.
- Graph reserved mean gains ≥0.01 relative to that encoder's **actual nonuniform
  initial baseline**, and each reserved correct joint-action probability exceeds
  its own initial probability. Do not substitute uniform1/16 for this baseline.
- Both frozen initial encoders and uniform endpoint/native action matrices are
  unchanged. Frozen archived positive controls are unchanged and exceed1/24+0.01.
- Blocking frozen/unpaid/disabled/initialized-reuse/one-fuel controls earn zero;
  correct reverse-order controls earn9/24 without early credit.

Use exhaustive measured native outcomes to compute exact policy expectations,
not sampled final-policy reward. Retain both initial/final modes, every world,
every action, parameters, intermediate batches and all criteria. Reserved
performance is secondary and cannot replace the full nine-world denominator.
Learning-negative outcomes are complete and sealed, not test failures.

## Preservation and independent verification

Verify the267-entry engineering archive under root
`2f6bbcde44f7bdbb646687b4a511ff6dfbc5484e293ff35d9860fa034b97e54d`, all245 old
live sources except one additive train export, and every older referenced seal.
Freeze current source snapshots/config/contract/auditor/tests before launch,
capture executed binary, and require exact source/feature closure afterward.
Keep disabled Evolving Minds/chemistry work excluded and unreverted. Existing
registered chemistry Cargo/export differences remain recorded, not broadened.

Prelaunch read-only preservation found six post-seal Finder `.DS_Store` files
in older archives. Every originally sealed entry and root seal is unchanged,
but their old exact-membership checks now fail on those extra files. Preserve
them; separately preregister each exact archive-relative path and byte hash in
config, copy them into the new archive, and require the same hashes afterward.
Do not ignore arbitrary hidden files or change/reseal any historical archive.
The new preservation receipt distinguishes unchanged sealed entries from the
old exact-membership failure. Incidental Finder metadata never enters policy,
reward, optimizer, native accounting or scientific criteria.

Before launch test independent sampling, native-free updates/refusals, negative
outcome accounting, zero-reward trajectories and detached corruption. Native
accounting independently reconstructs all ledgers/rewards using the unchanged
old auditor. Independently recompute raw observations, masks, draw/state identity,
paired seeds,110-parameter probabilities/likelihoods/gradients, all72 updates,
initial/full/reserved/frozen/positive endpoints and the unchanged all-seed verdict.
Python forward-mode AD remains independent of Rust reverse-mode backpropagation;
math.fsum and fixed2e-12 comparison tolerance govern arithmetic, exact draw IDs
and state values govern randomness. Archive-only rechecks spend no native work.

This tests optimization of a raw-adjacency encoder on an exposed seeded task.
Even success would not establish learned vocabulary/grammar, unseen-category/
topology generalization, online timing, ecological persistence, sustained H-R1
or independent Engine/RNG replay. `training_admitted:false`,
`full_training_authorized:false`, `scientific_acceptance:false`, and
`independent_physics_replay:false` remain unchanged.
