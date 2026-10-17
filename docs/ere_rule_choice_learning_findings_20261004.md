# Tiny paid-edit learning trial: findings and implementation

Date: 2026-10-04. Profile: `E2-rule-choice-learning-v2`.
Status: **complete; preregistered engineering-learning criterion passed**.
This is learned selection among seeded paid edits in exposed deterministic worlds,
not autonomous rule discovery, scientific H-R1 acceptance, or full training admission.

## Outcome

All three fixed policy runs increased conditional expected post-birth reward by
more than the preregistered 0.05 minimum. Matching-edit probability increased in
all nine run/context pairs. The paired preseeded-reuse learners received zero
novelty reward and remained exactly uniform. All frozen, unpaid, disabled and
preseeded endpoint controls received zero novelty reward.

| Policy seed | Baseline expected reward | Final expected reward | Gain | Final matching probabilities for contexts 10 / 20 / 30 |
| --- | ---: | ---: | ---: | --- |
| 993748715001 | 0.114583 | 0.279579 | +0.164996 | 0.698919 / 0.648159 / 0.482893 |
| 993748715002 | 0.114583 | 0.272717 | +0.158134 | 0.589522 / 0.672605 / 0.522933 |
| 993748715003 | 0.114583 | 0.277623 | +0.163040 | 0.732647 / 0.695901 / 0.388623 |

Every matching-edit probability started at 0.25. Expected rewards are weighted
over exhaustive native action panels at baseline and the fixed endpoint, not
sample means from selected training episodes or favorable checkpoints. There is
no significance claim for three policy-sampling runs and no held-out-world claim.

## What ran

The [V1 numerical contract](/Users/kalewicker/Documents/hypersim/docs/ere_rule_choice_learning_v1.md)
and [V2 source-scope correction](/Users/kalewicker/Documents/hypersim/docs/ere_rule_choice_learning_v2.md)
were fixed before trial-native execution. Each run used twelve zero logits,
12 simultaneous REINFORCE updates, two sampled actions per context/update,
rate1, baseline0, and an explicit SplitMix64 policy stream. The live and
preseeded-reuse learners shared prospective uniform draws but sampled from their
own policies. No reward lookup, adaptive rate, clipping, seed replacement,
outcome-based stopping, best-checkpoint selection or validation feedback occurred.

All 540 cells completed all 24 steps: **12,960 attempts/advances**, zero caps,
failures, skipped transitions, or unknown native work. Breakdown:

- 432 training cells: 216 live-paid and 216 preseeded-reuse, 10,368 native steps.
- 72 exhaustive baseline/final cells, 1,728 native steps.
- 36 final causal controls, 864 native steps.
- 36 live learner updates and 36 zero-reward control update calls: 72 total.
  The latter calls left every parameter exactly zero.

Each full native receipt was persisted before its reward entered an update.
Every reward came from the existing post-birth ordinary-event interface. The
two registered adapter physics seeds were alternated on fresh fixtures; they
are reused exposed deterministic seeds, not fresh independent physics evidence.

## Implementation / extension map

- [Learning module](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/rule_choice_learning.rs):
  fixed stream, complete-context `update`, output-before-update native cells,
  scheduled trial, endpoint weighting and unchanged criterion. It imports no
  Engine and performs no inner-graph host edits. `run` requires a fresh directory.
- [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_rule_choice_learning.py):
  checks every original native ledger through the prior independent auditor,
  reconstructs every draw, action, pre-update likelihood/gradient, parameter
  transition, full schedule, endpoint expectation and final verdict. Numerical
  comparison tolerance is 2e-12; draw identity and action membership are exact.
- [Runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_rule_choice_learning.py):
  source/feature preflight, frozen implementation and binary, retained native
  test receipt, independent audit/tests, feature-off check, post-run source checks,
  historical preservation, then exact archive sealing. Accounting failure marks
  a failed archive; a negative learning criterion remains a completed negative
  trial. The runner does not retry automatically.
- [Configuration](/Users/kalewicker/Documents/hypersim/configs/ere_rule_choice_learning_v2.json)
  fixes the complete schedule and limits. Changing seeds, update counts, task,
  action/reward definitions or success criteria requires another prospective
  version, not a silent edit of this result.

The prior native fixture, reward interface, canonical/lifetime helpers, editor
semantics and old scientific gates are unchanged. The only active historical
source drift is the additive `rule_choice_learning` export in hg-train.

## Source-preflight failure retained, not erased

The first V1 invocation stopped before invoking Cargo tests because its all-file
historical identity guard detected unrelated Evolving Minds changes. It contains
no native directory, native test receipt or policy update. Its generic failure
flag remains `native_work_unknown:true`; launch ordering and the retained failure
trace establish that this specific preflight spent **zero trial-native steps**.
The sealed failed attempt remains at
[V1 failed manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_choice_learning_trial_20261004/manifest.json),
root `c4059b1c62715a413add601eece8172da44ae384821a0354a814cce4b050091f`.

V2 prospectively narrowed measurement pins only for the explicitly disabled
Evolving Minds module and its feature-gated binary. It verified the Cargo feature
tree before/after: sim-core enables cognition and hg-meta, not evolving-minds.
Cargo manifests/lock and the lib.rs feature gate remain pinned. No scientific
criterion or numerical budget changed; no native result existed to influence
the correction. Unrelated edits were not reverted.

The feature-excluded historical drift comprises the Evolving Minds binary,
family.rs, mod.rs, source_inventory.rs and stream_output.rs. These are disclosed
as historical checkout drift, not claimed unchanged. All 194 other prior active
adapter pins match; its sole permitted active change is the additive export.
All 204 V2 measurement pins match their live inputs. V2 retains its feature and
historical checks before/after in the sealed output.

## Verification / provenance

Two Rust test groups and four Python groups pass, including 22 detached tamper
rejections: 14 sampling/update corruptions, six native-ledger corruptions, and
two endpoint corruptions. Feature-off Cargo checking passes. Native test time:
62.87 seconds; whole runner: 94.558 seconds; archive: about 95 MiB.

All 758 output seal entries verify. The frozen archived auditor reruns without
new native execution. Root seal SHA256:
`94ee4da611d6a1232f573770b102b6f573494cd9571b99df67cf014963317b63`.
Executed binary SHA256:
`b5e7b158f54baf06b49cabaf174255f1a5a2dd022113a140971247154cbdc684`.
The previous 283-entry adapter archive, 187-entry lifecycle archive and failed
V1 preflight verify under their unchanged seals. This is independent receipt
accounting, not an independent Engine/RNG physics replay.

See the [authoritative manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_choice_learning_trial_v2_20261004/manifest.json),
[independent audit](/Users/kalewicker/Documents/hypersim/output/ere_rule_choice_learning_trial_v2_20261004/audit.json),
and [native comparisons](/Users/kalewicker/Documents/hypersim/output/ere_rule_choice_learning_trial_v2_20261004/native/comparisons.json).
To verify saved evidence without retraining:

```sh
python3 -B output/ere_rule_choice_learning_trial_v2_20261004/source_snapshots/scripts/verify_ere_rule_choice_learning.py output/ere_rule_choice_learning_trial_v2_20261004/native
```

## Interpretation and next useful boundary

The selector now demonstrably learns to favor productive, paid, post-initialization
rule edits. Reward is no longer merely initialized-proposal reuse, and the same
reward exclusion prevents the reuse-control learner from changing its policy.
This resolves the narrow learning-mechanism question; increasing steps on this
same three-context bandit would add little evidence.

Next proposed task: a bounded **two-edit composition fixture** requiring two
separate paid literal changes before a newly created class can fire on ordinary
structure. First verify that partial edits get no firing credit and that native
epoch/lifetime attribution remains correct. Then use a parameter-sharing
selector and prospectively held-out label combinations, rather than a separate
lookup row for every combination. This tests a materially harder action grammar
and transferable composition instead of another longer memorization run.

This next task has not been launched. Current editor semantics are seeded,
contexts exposed and rewards deterministic. Class lifetimes remain right-censored;
replacement intervals and population half-life are unmeasured. Full scientific
H-R1, ecological persistence, generalization and full-project training admission
remain unestablished. `bounded_learning_authorized:true` denotes only this small
executed trial; `full_training_authorized:false`, `training_admitted:false` and
`scientific_acceptance:false` retain the wider admission boundary.
