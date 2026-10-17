# Feasibility-aware training findings

Date: 2026-10-01. Profile `E2-complete-feasibility-training-v1`;
objective `feasible-first-reuse-v1`.

The authorized training campaign completed all 20 generations, 320 candidate
evaluations and 6,144 fresh held-out evaluations. The feasibility-aware adapter
worked: four capped training candidates participated in constraint-aware ranking
instead of causing their antithetic pairs to be masked. Native cap fitness stayed
null. No crashes, Incomplete/NotRun evaluations or unknown partial work remain.

**Improvement is not established.** The primary constraint-utility gain over
zero-logit initialization is +0.0048246711131474645, with 18 wins, 14 losses,
480 ties and exact one-sided paired sign p=0.29830744792707264. Champion and
untrained feasibility are identical, 2,040/2,048 each. The registered utility
improvement criterion fails. Native reuse improvement is separately unconfirmed:
20 held-out cap nulls require its full-panel means and official p-value to remain
null. An accounting audit pass does not turn either result into scientific success.

## Prospective design and implementation

See the frozen contract
[`ere_complete_feasibility_training_v1.md`](ere_complete_feasibility_training_v1.md)
and [configuration](../configs/ere_complete_feasibility_training_v1.json).
This is a new objective version, not a retroactive change to the preceding
unsuccessful champion confirmation. Neither earlier held-out panel is pooled
with this one.

Training starts from 183 zero logits, not the old champion: 20 generations,
16 candidates/eight antithetic pairs, 100 native steps each, sigma 1 and alpha
f32 .05. Master seed `0xE656000001` = 989285318657, fresh optimizer RNG master+10000.
Shared +/− emission seeds are master+5000+8*generation+pair; shared training
physics seeds are master+100000+(generation+pair)%4. No fresh precheck physics
is run; exact audited parent receipts and unchanged native implementation pins
provide the inherited bounded 100-step evidence.

The adapter orders measured outcomes by feasibility first, then native reuse
cast to f32 among feasible candidates. Capped candidates tie at the bottom.
Dense ordinal input codes avoid offset-rounding near-ties; the existing ES core
then computes tied ranks and antithetic differences across all sixteen scores.
One-capped pairs favor their feasible arm; both-capped pairs have zero difference.
All-capped generations are retained with zero updates. All-feasible rank ordering,
tie treatment and updates are identical to legacy ES on the same inputs.

Native fitness is never changed: valid reuse stays in [0,1] and capped reuse
stays null. A separate constraint utility is U=1+native reuse for valid outcomes,
U=0 only for measured resource caps. Unsupported null reasons, errors, panics
and unknown work stop the campaign rather than receive a utility score. The
per-candidate ordering is feasible-first; averaging utility across held-out
draws is not a lexicographic comparison of aggregate feasibility rates.

The new module is feature-gated `hg-train::ere::feasibility_training`. The existing
confirmation module exposes scoped receipt/snapshot/durability helpers, with
an additional-input freeze entry point; its original route keeps its existing
default input set. The new CLI route is `train-feasible-complete`.
No Engine, matcher, native evaluator, grammar, coherent kernel, reward, cap or
optimizer-core source changed. The legacy training guards remain closed for
their unsupported scopes; `full_training_authorized:false` continues to refer
to the unrun reserved longer-horizon campaign.

The final generation-20 parameters are the sole designated champion, saved
before held-out measurement. Distribution diagnostics use 64 common emission
seeds and no Engine work. Maximum training TV from initialization is .109375;
final TV is .078125. Distribution movement is not evidence of improved fitness.

## Fixed held-out panel and registered criteria

Exactly 512 fresh emission draws × four fresh physics seeds × three arms:
final champion, zero-logit untrained and fresh random standard-normal 183-vectors.
Emission seed master+2000+draw; random-vector seed master+3000+draw. Each arm's
genotype/random vector is shared across four physics replicas, using
989285518657, 989285518658, 989285518659 and 989285518660. Order is draw, arm,
physics seed. No result selects a different champion or changes the schedule.

Primary constraint analysis compares the 512 paired genotype utility means,
each averaging four physics seeds. Success requires complete known outcomes,
positive primary utility mean, sign p<.05, and champion feasible count at least
untrained count. The last condition is a point-count guard, not statistical
proof of noninferiority. Champion−random is secondary. Ties are excluded only
from the sign test's binomial denominator; the mean denominator stays 512.

Native reuse retains the prior, stricter all-valid requirement: if any of the
6,144 held-out native evaluations is null, full native arm means, paired
differences and official native sign p are null. No complete-case replacement
is reported as the primary mean. A utility result cannot count as a pass of
the earlier native-reuse confirmation criterion.

## Actual work and caps

| Stage | Terminal / scheduled evaluations | Valid | Resource-cap null | Native attempts | Advances |
| --- | ---: | ---: | ---: | ---: | ---: |
| Training | 320 / 320 | 316 | 4 | 31,808 | 31,804 |
| Held-out | 6,144 / 6,144 | 6,124 | 20 | 613,488 | 613,468 |
| Total | 6,464 / 6,464 | 6,440 | 24 | 645,296 | 645,272 |

All 160 training pairs participate in the new adapter; four contain one cap,
none contain two. The four capped + arms occur at generation/pair (zero-based)
(1,4), (3,7), (5,2), (10,0), on attempts 10, 73, 51 and 74 respectively.
Their − arms are feasible. The frozen checker verifies the finite ordinal codes
and every resulting update. These local constraint signals were genuinely
exercised, but only four observed caps do not establish broad avoidance learning.

| Held-out arm | Valid / fixed denominator | Feasibility | Cap nulls |
| --- | ---: | ---: | ---: |
| Champion | 2,040 / 2,048 | 99.609375% | 8 |
| Untrained | 2,040 / 2,048 | 99.609375% | 8 |
| Random | 2,044 / 2,048 | 99.8046875% | 4 |

Champion and untrained both cap on draws 149 and 208, across all four physics
seeds, on attempts 74 and 37 respectively. Random caps on draw 448, all four
physics seeds, on attempt 50. Thus there are five capped held-out draw/arm
genotypes. The same two unavailable native primary draws become known utility
ties because both arms cap. This does not fill in their native reuse scores.

All 24 caps are delayed after successful steps. Every terminal cap receipt has
matcher truncation true, zero advance and an empty rule-event list. The observer
capture reset remains reliable on these paths; there is no stale-event credit.
All initial ordinary-world identities are `c73b4b7afbe84357`.

## Held-out result

| Arm | Mean registered constraint utility |
| --- | ---: |
| Champion | 1.20625206495634 |
| Untrained | 1.2014273938431925 |
| Random | 1.1964611243485928 |

Primary champion−untrained utility: +0.0048246711131474645.
Secondary champion−random utility: +0.009790940607747555.
All 512 utility pairs are known: 18 wins, 14 losses, 480 ties.
One-sided exact sign p=0.29830744792707264; the positive point estimate fails
the registered p<.05 criterion. Feasibility difference is exactly zero in this
panel. `constraint_utility.improvement_supported:false`.

There are 510 complete native primary pairs (18 wins, 14 losses, 478 ties) and
509 complete native secondary pairs. Full native means, both native mean
differences and official native sign p remain null. `improvement_supported:false`.
These available-pair directions are descriptive, not a complete-case substitute
for the registered 512-draw estimate. The sign test tests paired directions,
not a confidence interval for the mean utility difference.

This is not a controlled comparison of new versus legacy training algorithms:
the old campaign used different seeds and a different champion. The controlled
all-feasible compatibility tests establish adapter equivalence on identical
inputs; they do not attribute differences between separate training runs to
the new cap policy. Lower raw cap counts than the previous fresh panel cannot
be treated as a causal before/after improvement.

## Verification, resources and immutable evidence

Archive: [`output/ere_complete_feasibility_training_20261001`](../output/ere_complete_feasibility_training_20261001).
Authoritative files: [manifest](../output/ere_complete_feasibility_training_20261001/manifest.json),
[summary](../output/ere_complete_feasibility_training_20261001/summary.json),
[frozen audit](../output/ere_complete_feasibility_training_20261001/audit.json).

The frozen checker passes all 29 detached tamper controls. It verifies all native
accounting, exact schedule and catalogue membership, candidate arithmetic,
constraint conversion, all twenty f32 updates (parameter comparison tolerance
2e-7), final champion, full-denominator statistics and resource/source identities.
Tiny p-value comparison uses a stricter absolute tolerance than ordinary means.
An initial development regression caught this nested-p tolerance issue, and a
detached fixture initially mixed raw f64 scores with legacy f32 pair scores;
both were corrected and retested before freezing or scientific measurement.
No postmeasurement correction, scientific retry or seed replacement occurred.

Final root seal:
`f51ba18e5cabdedae4bab811e1cb6a3c86654b21355fbad8d07156f95f853432`.
All 6,944 sealed entries verify with exact coverage; all 449 live source/protocol
inputs still match their prospective freeze. Champion file SHA256:
`73085a4c50552a07b3ebe3557e3bcc2978eabba5277ca6f62c10a6886e0cdc1f`.
Executed binary SHA256:
`7aca316e63080e83cab0f863f57714304d361b9c1c9ec5c3d3b42f1ac228fc71`.

Elapsed native campaign time including setup/final identity checks is
469.825456375 seconds, including the single 168.050525208-second grammar rebuild;
this excludes the separate final audit/tamper suite. Largest native evaluation
duration: .187918458 seconds. Peak matcher work/results: 50,000 / 6,149. Peak
graph nodes/edges: 293 / 610. Archive: 959,641,297 bytes. These are within the
registered 3,600-second campaign, cooperative 30-second cell, 2-GiB output and
2,000/4,000 graph limits; native work never exceeds the 646,400-step ceiling.

Development verification: 127 library tests pass, one pre-existing release-only
full-budget delayed-cap regression ignored in debug; the three adapter tests
pass again after adding explicit update-direction/zero-update assertions.
Eleven ERE and two complete-witness integration tests pass. Seven new Python
regression groups and four prior confirmation groups pass. Complete-feature
release CLI builds; legacy ERE-only CLI checks. The previously verified native
release cap regression is not rerun as a scientific scaling cell.

The parent training receipt retains its original seal and 1,690 entries; the
previous champion confirmation retains its seal and 6,598 entries. The initial
failed short-training and both failed scaling archives also verify under their
original seals (1,662, 247 and 289 entries). This trial changes none of that evidence.
The auditor uses the exact pinned parent's native-accounting helpers. It is not
independent physics/RNG replay or compiler-to-binary proof. No learned repair,
sustained maintenance, online metadata evolution, novelty or longer-horizon
scaling claim follows from this experiment.

## Extension points and next useful research step

- `crates/hg-train/src/ere/feasibility_training.rs`: outcome refusal, ordinal
  optimizer adapter, fixed schedule, receipt-bound launch and separate statistics.
- `scripts/audit_ere_complete_feasibility_training.py` and its regression file:
  independently reconstructed conversion, f32 updates, denominators and tampering.
- `crates/hg-train/src/ere/confirmation.rs`: shared parent identity, freeze and
  durable-write/seal helpers; preserve its original confirmation scope.
- `crates/hg-train/src/optim.rs`: unchanged tied-rank ES; no native reward/cap change.

The next research question is signal quality, not another unchanged champion
panel. Only 32 of 512 utility pairs differ, and each current training candidate
is estimated from one emission/physics rollout. A useful next proposal is a
prospectively budget-matched comparison of single-emission versus small,
common-seed multi-emission candidate batches, with feasibility and reuse still
reported separately. This can test whether a less sparse/noisy candidate estimate
provides enough directional signal, instead of merely adding more held-out tests.
Its batch aggregation, antithetic coupling, resource policy and compute allocation
need a new registered design before execution; a batch is not automatically a
feasible genotype guarantee. No next campaign has been launched.
