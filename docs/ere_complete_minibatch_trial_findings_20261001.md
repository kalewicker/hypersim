# Budget-matched minibatch training: full-trial findings

Date: 2026-10-01. Profile `E2-complete-budget-matched-minibatch-v1`;
objective `mean-constraint-utility-v1`.

The authorized eight-replicate comparison completed its entire fixed schedule:
9,472 evaluations, 200 optimizer updates and sixteen final champions. The
prospectively frozen accounting auditor passes, rejecting all 31 detached
tamper controls. There were no crashes, unsupported outcomes, Incomplete/NotRun
cells or unknown partial work.

**Batching is not supported by this trial.** Four-emission batching has a
batch-minus-single mean held-out utility difference of
**−0.007154752879795889**, with four wins, four losses, zero ties and the registered
one-sided paired sign **p=0.63671875**. Both methods are feasible on all
2,048/2,048 held-out cells. Completion and feasibility criteria pass; positive
effect and p<.05 criteria fail. `support_batching:false` is the recorded result.
This does not establish single-emission superiority or equivalence, nor rule
out batching under other update allocations or hyperparameters.

## Authorization, design and comparison scope

The [launch authorization](ere_complete_minibatch_trial_authorization_20261001.md)
records the user's “alright let's continue” and the one-trial boundary. The
[prospective contract](ere_complete_minibatch_v1.md),
[configuration](../configs/ere_complete_minibatch_v1.json) and frozen snapshots
were unchanged for the scientific invocation. The earlier
[bounded development run](ere_complete_minibatch_implementation_20261001.md)
remains engineering evidence only, with separate seeds and no pooled outcomes.

| Method | Generations per replicate | Candidates | Emissions per candidate | Training rollouts per replicate |
| --- | ---: | ---: | ---: | ---: |
| Single | 20 | 16 | 1 | 320 |
| Batch | 5 | 16 | 4 | 320 |

Eight paired training replicates start from 183 zero logits. Each method receives
the same 160 emission/physics draw-pool indices, each used for positive and
negative candidates. Optimizer RNG seeds are paired across methods and distinct
across training replicates. Sigma 1 and alpha f32 .05 are unchanged. No optimizer
core, native physics, grammar, coherent kernel, reward, matcher or cap changes
occurred for this experiment.

Scientific master seed: `0xE657000001` = 989302095873. Replicate r's optimizer
seed is master+1,000,000+r, restarted separately in each method. For pool index
n=(8*g+p)*k+b, training emission seed is master+10,000+1,000*r+n and physics seed
master+100,000+100*r+n%4. Both methods visit n=0..159; their learned candidates
and use of successive perturbations differ. No previous champion or held-out
panel is reused as new evidence.

Every rollout requests the same native 100-step fixed-world horizon,
metadata-isolated allocation, MaxParallel/one rewrite, opt-in repeated-self-pair
feasibility matcher and pooled work/results 50,000/50,000. Node/edge ceilings
remain 2,000/4,000. Training uses complete fixed-batch means: valid utility is
1+f64(native f32 score), measured resource-cap utility is zero. Caps retain
null native fitness and remain in the batch denominator. Dense ordinal codes
feed the unchanged rank-shaped eight-antithetic-pair ES update. Mean batch
utility is not strict batch feasibility-first ordering.

All sixteen final-generation champions were saved before held-out physics.
The held-out panel has 64 shared fresh emission draws × four physics seeds
per champion, plus one zero-logit baseline on those same 256 cells. Emission
seeds are master+20,000+draw; physics seeds are 989302295873..989302295876.
No held-out outcome selects a different generation, champion or replicate.

The design compares **allocation of a fixed scheduled rollout budget**, not
batching at a fixed update count: the batch method has five versus twenty
updates per replicate. Actual advances, wall time, matcher work and host work
are not equalized. Savings from early caps were not reassigned. This tradeoff
must stay attached to the result.

## Registered held-out analysis

The primary statistical unit is the **paired training replicate**, fixed
denominator eight. Average four physics outcomes within each emission draw,
then all 64 draws within each method/replicate. The primary mean averages all
eight batch-minus-single differences. The common held-out panel does not turn
4,096 champion cells or 512 replicate/draw combinations into independent
training trials; interpretation is conditional on this fixed panel and world.

Held-out utility uses native f64 reuse, without the optimizer's training f32
cast. All champion held-out cells are valid, so utility is exactly 1+reuse.
The separate full-panel native comparison is consequently available; no
complete-case substitution or native-null imputation is needed.

| Replicate (zero-based) | Single native mean | Batch native mean | Batch − single utility | Training caps, single / batch |
| --- | ---: | ---: | ---: | ---: |
| 0 | 0.2603630230383672 | 0.2005632645151228 | −0.05979975852324437 | 0 / 0 |
| 1 | 0.2474382645151228 | 0.2161882645151228 | −0.03125 | 1 / 2 |
| 2 | 0.2318132645151228 | 0.2005632645151228 | −0.03125 | 1 / 1 |
| 3 | 0.2161882645151228 | 0.2630632645151228 | +0.046875 | 3 / 1 |
| 4 | 0.2005632645151228 | 0.2318132645151228 | +0.03125 | 0 / 0 |
| 5 | 0.2005632645151228 | 0.2161882645151228 | +0.015625 | 3 / 0 |
| 6 | 0.2161882645151228 | 0.2318132645151228 | +0.015625 | 3 / 0 |
| 7 | 0.2292515290302456 | 0.1849382645151228 | −0.04431326451512274 | 1 / 1 |

Utility means for each row are the corresponding native means plus one;
machine-precision values are in the sealed `summary.json`. All eight pairs
are retained, including negative results. Ties would be excluded only from
the sign test's binomial denominator; here there are no ties.

| Held-out arm | Mean utility | Mean native reuse | Valid / fixed cell denominator |
| --- | ---: | ---: | ---: |
| Single, average of all eight champions | 1.2252961423949185 | 0.22529614239491869 | 2,048 / 2,048 |
| Batch, average of all eight champions | 1.2181413895151227 | 0.2181413895151228 | 2,048 / 2,048 |
| Untrained baseline, evaluated once | 1.2161882645151227 | 0.2161882645151227 | 256 / 256 |

Primary utility difference: −0.007154752879795889; wins/losses/ties 4/4/0,
complete pairs 8/8, exact one-sided sign p=163/256=0.63671875.
Native difference: −0.007154752879795903, native sign p=0.63671875.
The small numerical difference between the two deltas reflects floating-point
summation, not differing outcomes.

Completion, known-utility and nonworse aggregate batch-feasibility criteria pass.
Positive mean and p<.05 criteria fail. Equal observed feasibility is a
point-count guard, not proof of statistical noninferiority or universal
feasibility. The sign test measures paired directions, not a confidence interval
for the mean effect.

Single-minus-baseline utility +0.009107877879795723 and batch-minus-baseline
+0.001953125 are **descriptive only**. The protocol introduced no additional
confirmatory baseline test or best-champion selection. Do not reinterpret these
small positive point estimates as confirmed training improvement or combine
them with older campaigns' panels.

## Actual work, caps and resources

| Stage | Recorded / scheduled | Valid | Resource-cap null | Native attempts | Advances |
| --- | ---: | ---: | ---: | ---: | ---: |
| Single training | 2,560 / 2,560 | 2,548 | 12 | 255,393 | 255,381 |
| Batch training | 2,560 / 2,560 | 2,555 | 5 | 255,766 | 255,761 |
| Untrained held-out | 256 / 256 | 256 | 0 | 25,600 | 25,600 |
| Single champion held-out | 2,048 / 2,048 | 2,048 | 0 | 204,800 | 204,800 |
| Batch champion held-out | 2,048 / 2,048 | 2,048 | 0 | 204,800 | 204,800 |
| Total | 9,472 / 9,472 | 9,455 | 17 | 946,359 | 946,342 |

All 200 started generations completed their updates: 160 single and 40 batch.
All 17 cap outcomes occurred in training and participated in fixed ranking.
Each capped trajectory attempted one more step than it advanced; its terminal
receipt has matcher truncation, zero advance and no fired-rule event. Captures
are clean; the earlier stale-event reliability fix remains in force.
The 17 attempt-minus-advance difference is fully accounted for, not unknown work.

Cap cell IDs (the exact frozen rows carry seeds and native step histories):

```text
train-r1-single-g11-p3-s1-b0        train-r1-batch-g0-p0-s-1-b1
train-r2-single-g4-p2-s1-b0         train-r1-batch-g3-p1-s1-b2
train-r3-single-g0-p5-s-1-b0        train-r2-batch-g1-p5-s-1-b0
train-r3-single-g1-p0-s-1-b0        train-r3-batch-g0-p2-s1-b0
train-r3-single-g13-p4-s1-b0        train-r7-batch-g0-p4-s-1-b3
train-r5-single-g7-p3-s1-b0
train-r5-single-g15-p5-s1-b0
train-r5-single-g16-p5-s1-b0
train-r6-single-g1-p6-s-1-b0
train-r6-single-g13-p2-s-1-b0
train-r6-single-g13-p4-s1-b0
train-r7-single-g4-p6-s1-b0
```

Native campaign elapsed time: 624.22621125 seconds, including one
143.719440209-second grammar compilation and setup/final checks. Separate final
audit time is outside that campaign measurement. Maximum recorded evaluation
duration: 0.2176485 seconds. Peak native matching work/results: 50,000 / 6,846;
peak nodes/edges: 293 / 610. Final sealed archive: 1,229,027,235 bytes.
All are within the registered 3,600-second, cooperative 30-second/cell and
2-GiB output ceilings; attempts are below the 947,200 maximum. These are
recorded cooperative limits, not OS-preemptive deadline guarantees.

## Frozen audit, integrity and evidence limits

Archive: [`output/ere_complete_minibatch_trial_20261001`](../output/ere_complete_minibatch_trial_20261001).
Durable status is `manifest.json`; registered results are `summary.json`,
and independent accounting checks are `audit.json`. The frozen auditor reports
`audit_pass:true`, `scientific_campaign_complete:true`,
`classification:budget_matched_minibatch_trial`, `support_batching:false`.
No postmeasurement checker correction, scientific retry, replacement, culling,
interim significance decision or criterion change occurred.

The auditor reconstructs the exact fixed schedule, all 200 candidate batches,
ordinal ranks and ES updates, all sixteen final champions, native receipt
accounting, utility/native means and eight-replicate statistics. It verifies
454 source/config/protocol/checker/test snapshots, executable identity,
catalogue identity and receipt lineage. All 31 detached tamper controls reject.
Generation starts precede evaluation; no partial batch is treated as an update.
Parameter arithmetic retains the pinned comparison tolerance <2e-7; means use
1e-12 tolerance, with a stricter absolute tolerance for p-values.

Final root seal (10,152 exact sealed entries, independently reverified):

```text
49a2bca085a7f7619c2bde3b5a25992a8fc16d8418d97165a2db3fb9db698806
```

Snapshot SHA256:
`151b63c97a064cd2c4d073509162b272102877076c70e8732b344853ce13de3f`.
Executed binary SHA256:
`ef787f97ac1347d2c0ddc11521bff2c18e0aefcd8b14756ef039d2be3c3f1732`.
All 454 corresponding live inputs and the live executable still match the
freeze after completion. Champion identities are included individually in
the root seal; no held-out-selected champion is promoted.

Historical seals and exact file coverage were reverified unchanged after this
trial: development 557, original short-training continuation 1,690,
feasibility training 6,944, champion confirmation 6,598, failed original short
training 1,662, failed scaling v1 247 and failed scaling v2 289 entries.
No older failed evidence was overwritten or repaired in place.

The checker uses pinned native accounting helpers and checks emitted catalogue
membership and recorded events. It does **not** independently replay native
physics/RNG, grammar emission RNG from parameters or compiler-to-binary
correspondence. `independent_physics_replay:false`. Accounting success does not
establish learned repair, online metadata evolution, sustained maintenance,
new semantic gate admission or 10,000-step scaling. The reserved longer-horizon
training campaign remains unrun (`full_training_authorized:false`).

## Diagnostic observation and next decision

A read-only post-hoc inspection of saved generation records finds:

| Diagnostic | Single | Batch |
| --- | ---: | ---: |
| Generations with all candidate means tied | 4 / 160 | 0 / 40 |
| Exactly zero parameter updates | 6 / 160 | 0 / 40 |
| Antithetic pairs with unequal ordinal scores | 499 / 1,280 | 252 / 320 |

Batching produces more non-tied pair comparisons per update in these records
(78.75% versus 38.984375%), but has one quarter as many updates and fewer
directional pairs in total. Unequal scores are **not** proof of accurate or
useful gradient direction. These post-hoc observations suggest a mechanism to
investigate; they do not change the registered conclusion or select parameters.

Recommended next step: a **fixed-parameter update-direction reliability assay**,
not another unchanged champion confirmation or automatic longer campaign.
Preregister a small panel of zero-logit and explicitly fixed, non-best-selected
parameter vectors; use fresh disjoint emission panels to ask whether independently
estimated rank-ES directions agree and whether a prescribed update improves
utility on a separate validation panel. Keep caps in the fixed denominators,
compare candidate-estimation reliability separately from update count, and
freeze sigma/alpha, budget and success criteria before measurement.

This would distinguish a noisy estimator from a direction that is reproducible
but unhelpful at the current objective/horizon. The present run alone cannot
resolve that question. Proposal/design work is the next decision; no assay,
retuning, additional campaign or new native work has been launched.

## Reproduction and review

The completed invocation used this fresh output; do not reuse it for a rerun:

```sh
target/release/ere-train train-minibatch-complete \
  output/ere_complete_short_training_continuation_20260930 \
  output/ere_complete_minibatch_trial_20261001
```

Read-only frozen re-audit (omit `--seal` when preserving the completed archive):

```sh
python3 -B output/ere_complete_minibatch_trial_20261001/source_snapshots/scripts/audit_ere_complete_minibatch.py \
  output/ere_complete_minibatch_trial_20261001 --tamper-tests
```

Harness/tests and extension points are mapped in the
[implementation report](ere_complete_minibatch_implementation_20261001.md).
Any new estimator, update allocation, objective or horizon requires a new
prospective identity and fresh evidence, not a revision of this frozen trial.
