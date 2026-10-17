# Native raw-graph encoder learning: findings and next boundary

Date: 2026-10-07. Profile `E2-graph-encoder-learning-v1`.
Execution/accounting status: **complete and independently verified**.
Learning status: **negative in all three seeds under the frozen criteria**.
One native launch, no retries, extra updates or changed thresholds.

## Outcome

The raw-adjacency encoder learns a modest increase in expected native reward,
but does not demonstrate the required useful relation selection. All three
final graph results are below even the 1/24 graph-blind ceiling, and graph-minus-
trained-blind gaps are far below the required0.01. The archived hand-feature
positive controls still perform well on the fresh native matrix. The native
task/reward remains productive; this particular encoder/optimizer/budget does
not pass the relational-learning criterion.

Initial graph full-panel expectation is0.024530559; initial masked expectation
is0.024606686. Initial reserved graph expectation is0.024530526. The required
graph full-panel threshold is1/24+0.01 =0.051666667. Each seed must pass on its
own; neither a pooled mean nor favorable held-out subset can rescue it.

| Policy seed | Graph final: all nine | Blind final: all nine | Graph minus blind | Graph reserved gain | Full/contrast verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| 993748721001 | 0.035152277 | 0.034872432 | 0.000279846 | 0.010621806 | Fail / fail |
| 993748721002 | 0.032468170 | 0.032380320 | 0.000087850 | 0.007937658 | Fail / fail |
| 993748721003 | 0.034508537 | 0.034321611 | 0.000186926 | 0.009978046 | Fail / fail |

All reserved correct joint probabilities improve relative to their actual
nonuniform starting values. Only seed001 meets the reserved mean gain0.01;
seed003's0.009978046 is below it and is not rounded into a pass. The primary
full-panel criteria fail in every seed regardless of these secondary results.

| Policy seed | Reserved [10,11] probability | [20,21] | [30,31] | Frozen old hand-feature control reward |
| --- | ---: | ---: | ---: | ---: |
| 993748721001 | 0.093803721 | 0.093656616 | 0.093758322 | 0.120602412 |
| 993748721002 | 0.086601164 | 0.086531331 | 0.086612979 | 0.158716909 |
| 993748721003 | 0.092065349 | 0.091956299 | 0.092046930 | 0.065215764 |

These are exact expectations reweighted over exhaustive measured native action
outcomes, not sampled final-policy averages. All nine worlds, sixteen action
pairs and three policy seeds remain in the denominator. Diagonal pairs are
optimizer-held-out only: every world was already exposed in engineering.
No topology or unseen-category generalization claim is supported.

## Fixed budget, optimizer and native controls

All **1,782 cells ×24 =42,768 native attempts/advances** and72 updates complete:
432 baseline,864 training,432 final and54 causal-control cells. Each learner
receives144 training cells per seed and twelve balanced updates. No caps,
native failures, incomplete/skipped batches, early stops or source waivers occur.
The audit counts2,790 paid edits and1,269 credited novel ordinary firings.

Both modes use the unchanged110-parameter width4/two-round encoder, exact
engineering initializer seed993748720001 with scale0.15, baseline0 and rate4.
Two samples per each of six off-diagonal worlds form a twelve-sample batch:

```text
theta_next = theta + (4/12) × sum(native_reward × joint_score_gradient)
```

Fresh policy draw seeds993748721001/002/003 share two uniforms per paired
sample. Graph/blind policies sample their own distributions, with the same
native seed on each adjacent pair. Native fixture seeds993748718001/002 are
reused; this is three policy-sampling streams, not three physics populations.
No teacher-selected engineering updates or archived positive-control weights
initialize the encoders. Both learners receive actual live-paid reward.

Both frozen initial encoders, uniform expectations and the complete native
baseline/final action matrices are unchanged. Archived hand-feature readouts
remain fixed, unchanged and above0.051666667 on fresh native outcomes. They are
historical learned-weight evaluation, not a third matched optimizer campaign.
Frozen/unpaid/disabled/initialized-reuse/one-fuel controls earn zero novelty;
correct reverse-order controls earn9/24 with no early credit. Synthetic lure
topologies are excluded because their native semantics have not been verified.

The unchanged task still requires two fuel-paid edits, excludes initial exact/
structural rule classes, distinguishes birth/installation/firing, and credits
strictly post-birth ordinary events using actual pre-step installed tables.
These mechanisms work in this trial; their success is not a relational-learning
pass, population half-life claim or sustained H-R1 acceptance.

## Information boundary and independent accounting

The encoder input is the ordinary-only canonical seven-label graph plus
adjacency; the blind mode erases adjacency. Task context, core metadata, literal
targets, active-triple annotations, hand-coded distances and future states never
enter encoder logits. The old hand-feature observation remains solely in native
reward validation and separate frozen positive-control evaluation. Known label
vocabulary, seeded candidate edit templates and preselected edit timing remain.

All864 decision receipts are persisted before invoking Engine. Each raw-graph
observation binds to the actual native initial inventory, and each full native
cell is persisted before its reward enters an update. Invalid observations or
incomplete paired batches cannot become zero-imputed training samples.

The independent auditor reconstructs every native ledger/reward, raw observation,
mask, exact draws/states, paired seeds,110-parameter likelihoods/gradients,72
updates, actual initial baselines, full/reserved/frozen/positive expectations
and unchanged all-seed verdict. Python forward-mode AD remains independent of
Rust reverse-mode backpropagation. Negative learning is accepted as a complete
outcome by both the Rust test and auditor; test success is not learning success.

## Read-only post-hoc diagnosis, not acceptance evidence

The following uses all three saved final native matrices and frozen endpoint
arithmetic after the trial. It changes no weights in the archive, runs no
physics/optimizer and cannot alter the preregistered verdict. Masking adjacency
with the final graph parameters fixed measures a policy-readout counterfactual;
restoring initial encoder layers while retaining final readout is another
parameter-block diagnostic, not a native causal intervention.

| Policy seed | Graph final | Same graph weights, adjacency masked | Initial encoder, final readout | Positive training cells, graph / blind |
| --- | ---: | ---: | ---: | ---: |
| 993748721001 | 0.035152277 | 0.034905112 | 0.035045242 | 16 / 16 |
| 993748721002 | 0.032468170 | 0.032404699 | 0.032440638 | 9 / 9 |
| 993748721003 | 0.034508537 | 0.034349522 | 0.034449198 | 14 / 14 |

The connectivity readout advantage under fixed final graph weights is only
0.000063471–0.000247165. Most of the reward gain survives restoring all104
encoder parameters to their initial values. Matched training actions agree in
143/144,144/144 and144/144 pairs, respectively; positive reward counts agree
in every seed. Output-bias parameter-change L2 norms are0.819/0.563/0.757,
versus embedding norms0.0167/0.0127/0.0150. Different block units/dimensions
make these norms descriptive, not causal proof.

Taken together, this suggests that the present sparse-reward optimization
chiefly improves generic edit selection, with little qualifying connectivity
benefit. It does **not** prove that the architecture cannot represent the task,
that exact-gradient optimization would succeed, or that more sampled steps
would solve it. No such follow-up was run.

## Implementation and extension map

- [Native learning driver](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/graph_encoder_learning.rs):
  fixed schedule, pre-execution decision persistence, actual observation binding,
  balanced paired updates, exhaustive `endpoint`, `comparison`, frozen readout
  controls and negative-complete result handling. It delegates encoder/reward/
  native execution to existing unchanged modules, not a new physics engine.
- [Encoder API](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/graph_encoder.rs):
  private validated raw observations, graph/masked inputs, fixed initializer,
  message passing, likelihood/gradient and balanced `update` (unchanged).
- [Prospective contract](/Users/kalewicker/Documents/hypersim/docs/ere_graph_encoder_learning_v1.md)
  and [config](/Users/kalewicker/Documents/hypersim/configs/ere_graph_encoder_learning_v1.json):
  immutable seeds, budget, rate, actual-baseline criteria, positive-control
  parameters/origin and separately pinned historical Finder metadata.
- [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_graph_encoder_learning.py):
  native/raw-observation reconstruction, independent forward-AD decisions and
  updates, complete initial/full/reserved/control comparisons and verdict.
- [Runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_graph_encoder_learning.py):
  fresh-only source freeze, zero-native preflight, one native launch, binary
  capture, historical preservation, postlaunch tests and immutable sealing.
- [Rust tests](/Users/kalewicker/Documents/hypersim/crates/hg-train/tests/ere_graph_encoder_learning.rs)
  and [Python tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_graph_encoder_learning.py):
  nonuniform-baseline regression, update refusals, zero-reward trajectories,
  negative-outcome accounting and detached/measured corruptions.

## Verification and immutable evidence

Two targeted Rust groups pass: zero-native endpoint/update checks, then the
single fixed native trial. Five Python groups pass prelaunch with two receipt-
only groups skipped; all seven pass afterward in83.798s. Tests reject twenty
transition, sixteen criterion, twelve native/raw-observation and four measured
corruptions, plus batch/mask refusals and exact incidental-file pin violations.
The feature-off Cargo check passes. Active sim-core features remain cognition/
hg-meta; disabled Evolving Minds and chemistry work is preserved and excluded.

All **2,920 output seal entries** and **252 frozen/live source pins** verify.
Frozen-source accounting reproduces the negative verdict without new native
steps. Native test time193.78s; complete runner350.620s; archive298,408,401
bytes (about285 MiB).

Root seal: `232178828b8de4fc2c33e30f6a3ccc7b25d489a5deeb7aad788367a1d2e9cd5b`.
Executed binary: `7068769ab787c6fb24ad96ce9ed5789acab0da184213be0f731826a8cec17477`.

The267-entry engineering archive remains intact;244 of its245 old live sources
match, with one proved additive train export. Every originally sealed entry and
root seal in all older referenced archives is unchanged, including failures.
Prelaunch inspection found six new Finder `.DS_Store` files in older archives:
their **old exact-membership checks fail on those extras**. The new contract
pins each exact path/hash, copies them separately and verifies the same bytes
afterward. No historical file is removed, old seal rewritten, general hidden-
file exception introduced or scientific criterion relaxed. See the preservation
receipt for `original_exact_membership_pass` versus `sealed_entries_unchanged`.

See [manifest](/Users/kalewicker/Documents/hypersim/output/ere_graph_encoder_learning_trial_20261007/manifest.json),
[audit](/Users/kalewicker/Documents/hypersim/output/ere_graph_encoder_learning_trial_20261007/audit.json),
[test receipt](/Users/kalewicker/Documents/hypersim/output/ere_graph_encoder_learning_trial_20261007/verification_test_receipt.json)
and [preservation receipt](/Users/kalewicker/Documents/hypersim/output/ere_graph_encoder_learning_trial_20261007/historical_preservation.json).
Archive-only accounting:

```sh
python3 -B output/ere_graph_encoder_learning_trial_20261007/source_snapshots/scripts/verify_ere_graph_encoder_learning.py output/ere_graph_encoder_learning_trial_20261007/native
```

This is independently implemented arithmetic/ledger accounting, not independent
Engine/RNG physics replay or a separately designed scientific replication.

## Recommended next step

Before another native campaign, run a **small exact expected-reward-gradient
diagnostic on the saved development action matrix**, with the same raw-graph
and masked encoders and the reserved worlds still excluded from updates.
Enumerating the sixteen saved native action outcomes removes sampled-action
variance without supplying hand-coded distance features. This would help
separate sparse credit assignment/gradient conditioning from representation
limitations, using zero new native steps and a new fixed prospective contract.
It would be archive-reuse optimizer engineering, not fresh native generalization
or a retroactive rescue of this failed learning criterion.

No follow-up optimizer run or native trial is launched here. Full-training/
scientific admission remains false; fixed labels/templates/timing, exposed
worlds and lack of ecological persistence/sustained H-R1 remain limitations.
The sealed manifest records `learning_criterion_pass:false`,
`training_admitted:false`, `full_training_authorized:false`,
`scientific_acceptance:false`, `independent_physics_replay:false`.
