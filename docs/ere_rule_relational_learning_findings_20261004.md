# Paired relational learning: findings and extension map

Date: 2026-10-04. Profile: `E2-rule-relational-learning-v1`.
Status: **complete; independent accounting and learning criteria passed for all three seeds**.
One preregistered native campaign, no retries or changed criteria.
Full-training/scientific admission and independent physics replay remain false.

## Outcome

Starting from zero, reward-driven optimization learns to use the existing
hand-engineered ordinary-connectivity features. Every graph-aware learner
exceeds both the trained graph-blind comparator and the mathematical ceiling
for any graph-blind joint action distribution on the full balanced nine-world
panel. All three optimizer-held-out pairs improve in every seed.

The uniform baseline is **0.0234375**. The graph-blind ceiling is **1/24 =
0.041666667**; the preregistered graph threshold is **0.051666667**. Each seed
must independently meet that threshold, beat its trained blind comparator by
0.01, gain 0.01 on the reserved mean, and improve every reserved correct-action
probability above 1/16. No pooled-seed or reserved-subset rescue is permitted.

| Policy seed | Graph: all nine | Trained blind: all nine | Graph minus blind | Graph: reserved three | Reserved gain | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| 993748719001 | 0.120602412 | 0.031712656 | 0.088889757 | 0.125192786 | 0.101755286 | Pass |
| 993748719002 | 0.158716909 | 0.033774271 | 0.124942638 | 0.159664697 | 0.136227197 | Pass |
| 993748719003 | 0.065215764 | 0.027800752 | 0.037415012 | 0.066317805 | 0.042880305 | Pass |

These are exact policy expectations over the exhaustive measured native action
matrix, not sampled final-policy averages. Every world and every zero-reward
action stays in the denominator. The third seed is materially weaker and is
retained, not replaced. Policy seeds vary action draws, not independent physics.

Reserved correct joint-action probabilities, ordered [10,11], [20,21], [30,31]:

| Policy seed | [10,11] | [20,21] | [30,31] | Blind reserved expected reward |
| --- | ---: | ---: | ---: | ---: |
| 993748719001 | 0.491076874 | 0.184755190 | 0.325710228 | 0.034553213 |
| 993748719002 | 0.343241587 | 0.432185202 | 0.501890788 | 0.037724140 |
| 993748719003 | 0.117084750 | 0.219792798 | 0.193664895 | 0.027931590 |

The reserved combinations never enter optimizer updates, but all nine worlds
were exposed during engineering. This is **optimizer-held-out transfer**, not
developer-unseen validation, unseen-category generalization or a confidence
interval for a broader task distribution.

## Fixed budget and actual training signal

All **1,782 cells × 24 = 42,768 native attempts/advances** complete. There are
72 updates: 36 per observation mode, twelve per policy seed. No caps, native
failures, incomplete/skipped batches, early stopping, retries or source waivers
occur. The audit counts 2,712 paid edits and 1,260 credited novel ordinary fires.

| Phase | Cells | Native steps | Purpose |
| --- | ---: | ---: | --- |
| Baseline | 432 | 10,368 | Three exhaustive nine-world × sixteen-action panels |
| Training | 864 | 20,736 | Two matched learners, six development pairs, two samples, twelve batches |
| Final | 432 | 10,368 | Same exhaustive native matrices; exact endpoint expectations |
| Controls | 54 | 1,296 | Six causal controls on each reserved pair, per seed |
| Total | 1,782 | 42,768 | Full fixed denominator |

Both policies receive real live-paid native rewards; neither blind learner nor
its optimizer is frozen. Positive training cells out of 144 per learner are
graph 18/24/8 versus blind 9/14/4, respectively. These sampled training counts
are descriptive, not the final endpoint criterion. The blind policies learn
nonuniform marginals and improve above uniform while remaining below 1/24.

Both models have sixteen trainable parameters: four biases and a shared 4×3
matrix. They start at zero and use the same rate4, baseline0 score-gradient
update with twelve balanced samples. Logits remain at pre-batch values until
all paired samples finish:

```text
theta_next = theta + (4/12) × sum(native_reward × joint_score_gradient)
```

The graph input selects distance2 first-category and distance1 second-category
indicators from the label12 anchor in the validated ordinary graph. The blind
input is [1/3,1/3,1/3] for each slot, reflecting identical category marginals.
Its input and action probabilities are identical across all nine worlds. The
mask changes feature norm as well as available information; equal parameter
count does not imply identical effective feature geometry. The mathematical
all-policy blind ceiling is the primary safeguard against that ambiguity.

Draws come from policy seeds 993748719001/002/003, with two SplitMix64 uniforms
per sample shared between learners. Each policy selects its own actions using
its own logits. Adjacent paired cells receive the same native seed from the
existing 993748718001/002 fixture pair. Physics and policy streams stay separate.
No new independent physics-seed universe is claimed.

## Native controls and information boundary

Every initial world has the same seven ordinary node identities/data and label
marginals. Only the active two-edge path changes. Exactly one action pair per
world earns 9/24 through two fuel-paid native edits and subsequent ordinary
firings. Wrong, partial and no-edit actions earn zero.

Frozen, unpaid, disabled, initialized-reuse and one-fuel controls all earn zero
novelty reward. Correct reverse-order controls earn 9/24, with no early credit.
The native baseline/final action matrices and frozen-uniform expectations are
unchanged. Actual pre-step installed tables govern firing attribution, and exact
plus structural initialization exclusions govern novelty. Class birth,
installation and first firing remain distinct; lifetimes are right-censored.

Training writes each of 864 decision receipts **before** invoking native
execution. The receipt contains validated observation, graph/blind input, frozen
parameters, exact draws/states, actions, probabilities, likelihood and gradient.
Its observation must equal the actual native initial-inventory projection.
The full native cell is persisted before reward enters an update. A mismatch
invalidates the trial rather than becoming a zero reward.

The readout API accepts a validated observation, not a context label. Context
is driver/batch/endpoint bookkeeping only. Policy inputs exclude initialized
core metadata, literal targets, active-triple annotations, receipt context,
future state, rule tables and rewards. The initialized task core still contains
the answer outside that projection; this remains a controlled seeded task.

## Implementation and safe extension points

- [Learner and paired driver](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/rule_relational_learning.rs):
  opaque mode-specific `Input`, sixteen-parameter policy arithmetic, sampling,
  score gradient, balanced `update`, exhaustive `endpoint`, pre-execution
  decision persistence and fixed `run` schedule. `update` refuses mixed modes,
  malformed rewards/actions, incomplete or unbalanced batches and reserved pairs.
- [Existing graph interface](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/rule_relational.rs):
  unchanged ordinary-only observation validator and strict native reward.
  Graph-mode policy arithmetic is checked against this existing implementation.
- [Existing native fixture](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere/rule_relational.rs):
  unchanged initializer, paid editors, Engine/GraphRuleSource execution and
  full graph/event/lifetime recording. This learner introduces no new physics.
- [Contract](/Users/kalewicker/Documents/hypersim/docs/ere_rule_relational_learning_v1.md)
  and [config](/Users/kalewicker/Documents/hypersim/configs/ere_rule_relational_learning_v1.json):
  prospective budgets, seeds, information mask, optimizer, denominators,
  deadlines, all-seed criteria and interpretation limits.
- [Runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_rule_relational_learning.py):
  fresh-only archive, frozen source closure, zero-native preflight, one native
  launch, executed-binary capture, postlaunch checks and immutable sealing.
  Learning-negative outcomes remain complete; accounting failures remain failed.
- [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_rule_relational_learning.py):
  reconstructs unchanged native ledgers, rewards, mask, exact draw streams,
  pairing, gradients, updates, complete action expectations and final verdict.
  It checks exact receipt membership and rejects optimizer-held-out leakage.
- [Rust tests](/Users/kalewicker/Documents/hypersim/crates/hg-train/tests/ere_rule_relational_learning.rs)
  and [Python tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_rule_relational_learning.py):
  graph/blind arithmetic, finite differences, sampling, batch refusals,
  negative-outcome accounting and detached/measured tamper controls.

Only one additive train module export changes an older active source file.
Prior native, observation/reward, learner, compiler and auditor bodies remain
unchanged. A new architecture or task should receive a new contract/version;
do not change this campaign's snapshots, optimizer or auditor and reseal it.
The native Rust test runs a whole campaign: use the archive-only command below
to inspect evidence without spending another native budget.

## Verification and immutable evidence

Two targeted Rust groups pass: zero-native matched arithmetic/update refusals,
then the single full native trial. Five Python groups pass before launch with
two receipt-only groups skipped; all seven pass afterward. Detached tests reject
24 transition, twelve criterion and twelve native corruptions, plus four
measured transition/mask/endpoint corruptions and separate batch refusals.
The feature-off Cargo check passes. The sim-core feature closure remains
`cognition`/`hg-meta`; unrelated disabled Evolving Minds work is not reverted.

All **2,900 archive seal entries** and **238 active measurement source pins**
verify, including identical source snapshots. The frozen archived auditor
reproduces the full verdict with zero additional native steps. Native test time
is 197.16 seconds; runner time is 294.133 seconds. Archive size is 287,898,226
bytes (about 275 MiB).

Root seal: `a0ccb05305ecaaa151be4bb110f6235b8c375aa50b5c3f93837dddb70f5f22a8`.
Executed binary: `9bfde139bfffa0edbcea4c135c8f0f63f5749b06dba973849ea51aeca30d7850`.

Historical preservation checks agree before/after. All 643 entries of the
relational engineering archive remain intact; 230 of its 231 active sources
are identical, with the remaining train export proved additive-only. The
1,445-entry shared-learning, 617-entry originally failed composition,
11-entry corrected supplement, 758-entry choice-learning, 283-entry adapter,
187-entry lifecycle and failed-preflight archives retain their original seals.
The originally failed composition audit is not rewritten or promoted.

See the [sealed manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_relational_learning_trial_20261004/manifest.json),
[independent audit](/Users/kalewicker/Documents/hypersim/output/ere_rule_relational_learning_trial_20261004/audit.json)
and [test receipt](/Users/kalewicker/Documents/hypersim/output/ere_rule_relational_learning_trial_20261004/verification_test_receipt.json).
Archive-only accounting, without invoking native physics:

```sh
python3 -B output/ere_rule_relational_learning_trial_20261004/source_snapshots/scripts/verify_ere_rule_relational_learning.py output/ere_rule_relational_learning_trial_20261004/native
```

This is independent arithmetic/ledger accounting, **not** a separately
implemented Engine/RNG replay or an independently designed replication.

## Interpretation and next boundary

The project has moved beyond an untrained capacity probe: optimization now uses
relation-sensitive observations to choose productive paid edits, with a trained
information-blind comparator and an independently checked full-panel bound.
Another larger sweep on these same nine worlds would chiefly measure scale or
reliability, not resolve the remaining capability gap.

The next useful extension is a **small learned graph encoder**, replacing the
hand-coded anchor-distance/category indicators while retaining this learner as
a positive control and the category-only learner as a negative information
control. First fix an ordinary-only, permutation-invariant graph input and
prospective development/held-out topology panels; exclude context and seeded
core metadata again. Then budget a separate trial to test learning the relational
representation, rather than supplying it. No such extension/trial is launched
by this report.

Current limitations remain explicit: fixed labels and seeded edit templates,
both action choices made before native execution, predetermined edit timing,
hand-engineered relation extraction, exposed deterministic worlds, reused
physics seeds, no ecological persistence/population half-life or sustained H-R1.
`training_admitted:false`, `full_training_authorized:false`,
`scientific_acceptance:false`, and `independent_physics_replay:false` remain
unchanged. This campaign authorizes only its bounded mechanism-learning scope.
