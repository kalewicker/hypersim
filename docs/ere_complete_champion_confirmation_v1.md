# Fixed-champion confirmation v1

Preregistered 2026-10-01 before new native evaluations. The user accepted the
recommendation to test the saved final champion on a larger fresh held-out panel.
Profile `E2-complete-fixed-champion-confirmation-v1`.

## Fixed input and scientific question

Use the final-generation parameters already selected in the completed 100-step
training campaign. Preserve its original 64-draw result: primary point gain
0.05838775724120188, five wins / one loss / 58 ties, sign p0.109375; its declared
improvement criterion failed. This new panel is an independent confirmation and
does not pool those earlier draws or change their interpretation.

Training archive `output/ere_complete_short_training_continuation_20260930`:
root SHA256 `dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132`,
1,690 sealed files. Champion file SHA256:
`14cf0b13540167b3a3b20082db71ddd9bbd761a5f5b0d81083dd6d719e24988f`.
Verify all original outputs and run its original frozen checker read-only before
setup. No optimizer, precheck physics, champion reselection or parameter update
occurs. Copy the champion file unchanged into the new archive.

## Exact schedule and resources

Master seed `0xE655000001`. Exactly 512 independent emission draws, four shared
fresh physics seeds per draw, three arms, horizon100: 6,144 evaluations and a
maximum614,400 native Engine attempts. The emission seed is master+2000+i for
i=0..511, shared across arms and physics. Physics seeds are master+100000..100003.
These differ from training, prior prechecks and the earlier held-out panel.

The arms are final champion, zero-logit initialization, and random-resample.
Random-resample draws a fresh 183-dimensional standard-normal parameter vector
using the unchanged Box-Muller/ChaCha8 implementation with seed master+3000+i;
reuse that vector for its four physics evaluations. Persist all vectors through
the native cell receipts. Evaluate in draw, arm [champion,untrained,random],
physics order. No score previews, replacement, favorable subsample selection,
interim significance checks, optional stopping or retry after failure.

Keep the finite typed grammar, coherent kernel, fixed initial ordinary world,
frozen decoded metadata, objective, MaxParallel/one-rewrite behavior, metadata
isolation, opt-in repeated-self-pair feasibility matching, pooled50k/50k matcher
limits and existing2,000-node/4,000-edge caps unchanged. All230 native/Cargo/
optimizer pins must match the completed training archive exactly, with exact
source-file coverage. There is no new native source bridge. Rebuild the grammar
once and require catalogue SHA256
`ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6`.

Ceilings: 3,600 seconds including setup, compilation and final source/parent
verification;30 seconds per evaluation, cooperatively checked;2 GiB output,
with128 MiB receipt reserve and16 MiB final headroom. No ceiling expansion or
fuel refill. Final accounting/tamper audit is separate from the native runtime
ceiling. Freeze the binary, full learning crate, all pinned sources, config,
contract, checker and regressions before setup and native measurement. Store the
schedule and snapshot separately so manifest checkpoints remain small.
Recheck all frozen/live source hashes after compilation, every64 draws and at
completion, and verify the parent archive at completion.

## Score and preregistered criteria

Unchanged reward: ln(1+max(credited Proposal fires-1,0))/ln(100). Only active
Proposal execution distinct from priority-normalized Core receives credit.
This measures bounded reuse of initial complete families and edits in a fixed
world; it cannot establish learned maintenance, repair, novelty or longer-horizon
behavior.

Average each arm's four physics scores for each emission draw. Primary statistic:
the mean of exactly512 paired champion-minus-untrained differences. Secondary:
champion-minus-random mean. Report all three arm means, paired wins/losses/ties,
complete-pair counts and fixed denominators. Successful zero scores remain zero;
cap/null/incomplete/NotRun outcomes remain in the denominator. Full estimates
are null if any required evaluation is not valid and complete. Report partial
valid counts without treating them as a smaller successful panel.

Confirmation criterion: all6,144 valid evaluations, positive primary mean gain,
and one-sided exact paired sign p<.05 across the512 genotype means; ties are
excluded only from the binomial sign-test denominator. The test concerns paired
directions, not a confidence interval for mean gain. This is one new primary
test on independent draws; report its result separately from the earlier pilot.
No statistical result selects a new champion or authorizes further training.

## Durable failures and frozen audit

Create every schedule slot as NotRun before setup. Write an in-flight marker
before evaluation and atomically persist each returned receipt before committing
its counters. Returned native caps produce Null and continue the fixed schedule.
A panic or returned evaluator error produces Incomplete with unknown partial
work and stops; all remaining slots stay NotRun. Known counts cover saved native
receipts only. A hard process kill can leave an in-flight marker; no per-step
crash journal or OS-level preemption is claimed.

The prospectively frozen checker verifies parent/champion seals, unchanged native
source identities, exact schedule, typed production membership, parameters and
matched emissions across physics, event credit, reward, pooled allowances, graph
caps, counters and independently computed sign statistics. It uses the pinned
original campaign's native-accounting helper, whose source is covered by the
parent seal. It streams one draw's cells at a time. Tamper controls apply changes
to isolated in-memory copies of metadata/cells/statistics after verifying the
real archive, avoiding repeated copies of a multi-gigabyte panel; source and seal
corruption tests use small temporary files and never mutate the real archive.
Freeze these control definitions before measurement and record actual rejection
counts. This audit does not replay native physics/RNG streams or prove
compiler-to-binary correspondence. Seal every output except the root seal,
preserve all previous failures unchanged, and write a Markdown findings report.
