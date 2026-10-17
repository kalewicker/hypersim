# Fixed-parameter update-direction reliability assay

Prospective design, 2026-10-01. Profile `E2-complete-direction-reliability-v1`.
The user requested the next step after the budget-matched minibatch trial.
This task implements/tests the assay and a separate bounded development fixture.
The scientific layout below requires separate launch approval and is not
authorized by this document. No reserved longer-horizon training is authorized.

## Question and smallest falsifiable milestone

The prior allocation trial completed but did not support batching: eight paired
training replicates, utility difference −.00715475, sign p=.63671875. Batches
had more non-tied comparisons per update, not demonstrated useful directions.
Do not pool that held-out panel, select its best champion or change its result.

At fixed parameters, does independently resampling emission/physics panels
produce aligned rank-ES updates, and do these updates help on fresh validation?
This separates emission-panel variability from the fewer-update allocation
tradeoff. Perturbations are shared across the two estimates: this tests
**conditional reproducibility given an eight-direction perturbation basis**,
not reliability under independently sampled bases, an unbiased mathematical
gradient or general learning success. Reversed-update controls are descriptive
checks of direction versus nondirectional finite-step effects.

Two predetermined parameter contexts, both length 183:

- Context 0: all zero logits.
- Context 1: +.25 at even indices, −.25 at odd indices (zero-based).

These are not trained champions or outcome-selected vectors. Eight independent
blocks evaluate both contexts; no adaptive context, seed or hyperparameter choice.
Each block/context estimates A and B with the same eight antithetic standard-normal
perturbations, sigma 1, alpha f32 .05 and sixteen candidates. Each candidate
uses four emission/physics draws. A/B panels are disjoint. Caps have native null
reuse but known zero utility in the fixed mean; no masking or survivor denominator.
Use the existing dense ordinal mean-utility adapter and unchanged rank-ES update.
Every estimate starts afresh from its fixed context, not from the other estimate.

Save starts/noise/exact IDs before physics, then both estimated post-update
vectors. All 32 estimates finish before any validation. Validate exactly five
arms: base, forward A, reverse A, forward B, reverse B. Forward is the unmodified
f32 ES post-update vector; reverse is componentwise f32(2*base−forward), with no
normalization, clipping or alpha tuning. Use 16 fresh emission draws × four
physics seeds for every arm/block/context. No validation-based estimate selection.
Even a zero update retains its full validation allocation and reverse control.

## Seeds and exact schedule

Scientific master `0xE658000001` = 989318873089; development master
`0xE658D00001` = 989332504577. Block r, context c, estimate e=0/1, pair p,
batch draw b and validation draw d:

- Perturbation RNG seed master+1,000,000+10*r+c, restarted for A/B.
- Estimation emission master+10,000+1,000*r+100*c+40*e+k*p+b.
- Estimation physics master+100,000+100*r+10*c+4*e+b%4.
- Validation emission master+20,000+1,000*r+100*c+d.
- Validation physics master+200,000+100*r+10*c+physics_index.

Separate domains across stages/blocks/contexts/estimate panels; positive/negative
candidates share their batch seeds, all five validation arms share each draw's
emission/physics seeds. Estimation order: block, context, A/B, pair, +/−, batch.
Validation order: block, context, draw, arm, physics. Shared seeds reduce noise,
not increase independent observations. Results are conditional on these two
contexts, the declared perturbation basis sampling and narrow fixed world.

## Registered statistics and acceptance

Actual direction is f64(forward−base), using the saved f32 vector values.
For each block/context compute cosine(A,B). If either norm is exactly zero,
assign alignment zero and record degeneracy; do not drop the context/block or
invent a cosine. This is a zero-for-degenerate alignment score, not always a
mathematical cosine. Average the two context scores to obtain eight block scores.

Within each block/context/arm average four physics utilities per draw, then all
16 draws. U=1+native f64 reuse for valid outcomes; U=0 only for measured resource
caps. Primary usefulness effect per block is the equal average over two contexts
of ((forward_A+forward_B)/2−base). Mean all eight block effects; no winner choice.
The reverse effects and forward-minus-reverse difference are descriptive only.

Primary unit **paired estimator block**, fixed denominator eight. Separately
report means and exact one-sided sign tests for positive block alignment and
positive usefulness effect. Ties are excluded only from each binomial denominator,
not either eight-block mean. Missing unknown utility/direction keeps official
mean/p null and denominator eight; no complete-case replacement.

Supporting reproducible, locally useful updates requires complete fixed work,
known outcomes, positive alignment mean with p<.05, positive usefulness mean
with p<.05, positive usefulness means in both fixed contexts, and aggregate
forward valid count at least twice baseline valid count (two forward arms versus
one baseline, same fixed per-arm denominators). This count guard is not statistical
noninferiority. Requiring both primary properties is an intersection-union claim;
there is no declaration that either single passing property establishes the
combined claim. A one-property result is diagnostic, not full assay success.
Do not test/exploit reversals, individual contexts, individual blocks or alpha
variants as additional confirmatory discoveries. Native full-panel means remain
null if any validation native outcome among the five arms is capped.

## Native scope, budgets, failure and provenance

Native 100-step fixed world, typed grammar/coherent kernel, metadata isolation,
MaxParallel/one rewrite, opt-in repeated-self-pair matcher, pooled 50k/50k,
nodes/edges 2,000/4,000; unchanged native reward, evaluator and optimizer core.
Bind original short-training parent seal/1,690 files, all 230 live native/Cargo/
optimizer pins and frozen admission/precheck audit. Bind prior minibatch root
`49a2bca085a7f7619c2bde3b5a25992a8fc16d8418d97165a2db3fb9db698806`,
all 10,152 sealed entries and its negative result as diagnostic lineage only.

Scientific: 2,048 estimation + 5,120 validation = 7,168 evaluations,
maximum 716,800 attempts, 32 one-step estimates (zero training generations).
Ceilings: one grammar compile, 3,600 seconds including setup/final checks,
cooperative 30-second/cell deadline, 2-GiB output, 128-MiB receipt reserve,
16-MiB final reserve. Actual caps/work/time reported, not nominally equalized.
No retries, culling, replacement, interim significance, budget reuse or criterion
change. Unsupported outcomes/errors/panics stop the whole fixed schedule.
In-flight work is marked unknown until terminal receipt/counter commit; no
per-step crash journal or OS-preemptive deadline promise.

Development: one block, both contexts, two draws/candidate, two validation draws
× four physics × five arms. 128 estimation + 80 validation = 208 evaluations,
maximum 20,800 attempts, four estimates, 900-second ceiling and 2-GiB cap.
Separate seeds, same execution/audit path, always `support_direction:false` and
development-only classification; it cannot establish scientific reproducibility.

Freeze sources/config/contract/auditors/tests, complete schedule, contexts and
binary before native work. Recheck live sources after compile and each estimation
context/validation block, then parent/latest seals at completion. Preserve every
NotRun slot on failure. Final seal/auditor verifies native accounting, fixed
batch means/ranks, candidate arithmetic, both updates, exact reversals and all
statistics. Checker scope is accounting/provenance, not independent replay of
native physics, grammar/perturbation RNG from seeds or compiler-to-binary correspondence.
Detached tamper controls and bounded development must pass before recommending
the separate scientific launch. Any failure remains a failed archive with a
disclosed correction/continuation, never a silently repaired measurement.
