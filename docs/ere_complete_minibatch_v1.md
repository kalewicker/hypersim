# Budget-matched single-emission versus minibatch training

Prospective design, 2026-10-01. Profile `E2-complete-budget-matched-minibatch-v1`.
The user requested work on the next experiment. This authorizes design,
implementation and bounded development verification; the scientific campaign
below is proposed and is not launched by this document.

## Question and methodological boundary

Does spending a fixed native rollout allocation on richer candidate estimates
produce better final distributions than spending it on more ES updates?
The preceding feasibility-aware experiment completed but did not establish
improvement: utility gain .00482467, sign p=.298307, equal champion/untrained
feasibility. Only 32/512 utility pairs differed. Its archived results remain
immutable; no seeds, outcomes or champions are reused as new scientific evidence.

| Training arm | Generations | Candidates/generation | Emissions/candidate | Native rollouts/replicate |
| --- | ---: | ---: | ---: | ---: |
| Single | 20 | 16 | 1 | 320 |
| Batch | 5 | 16 | 4 | 320 |

Each rollout requests 100 steps. Eight paired independent training replicates
start from 183 zero logits. Matching four-emission batches necessarily reduces
the number of updates under this allocation. The experiment tests this resource
allocation tradeoff, not a pure batching effect with update count held constant.
Alpha/sigma are not retuned after observing outcomes. Allocation is matched in
scheduled 100-step native rollouts, not wall time, host emission work or matcher
work. Caps can terminate early; saved work is not reassigned to another sample.

## Native scope, identity and objective

Keep the unchanged typed grammar, coherent kernel, fixed ordinary world,
metadata isolation, native reuse reward, opt-in matcher, MaxParallel/one rewrite,
pooled work/results 50,000/50,000 and graph caps 2,000/4,000. No new native
physics, matcher, reward or optimizer-core implementation is introduced.
The legacy guards and reserved 10,000-step training refusal remain in force.

Require the original complete short-training continuation root
`dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132`,
all 1,690 sealed files, its frozen parent/precheck audit and all 230 live native/
Cargo/optimizer pins. Also verify the previous feasibility trial root
`f51ba18e5cabdedae4bab811e1cb6a3c86654b21355fbad8d07156f95f853432`
and all 6,944 entries as diagnostic lineage, explicitly not improvement evidence.
No new precheck Engine work is scheduled.

Per rollout, measured valid native reuse stays in [0,1]; native resource caps
stay null. For ranking, use 1+f64(native score cast to f32) for valid outcomes,
0 only for measured resource caps. Candidate ranking uses the arithmetic mean
over its entire fixed batch, including capped entries; no masking or survivor
mean. Convert the sixteen candidate means to dense ordinal codes and pass them
to unchanged rank-shaped antithetic ES with eight pairs, sigma 1, alpha f32 .05.
Both arms use this same aggregation; batch size one exactly reproduces the
preceding adapter's ordering, ties and update. Both-capped/equal batches tie;
an entirely capped generation remains in the schedule with a zero update.

Mean batch utility is **not** a strict feasibility-first ordering of batches:
a high-reward partly capped batch can outrank an all-valid low-reward batch.
This is intentional fixed mean-constraint utility, not proof every genotype is
feasible. Unsupported nulls, invalid-family, nonadvancing, errors or panics
stop the campaign with saved terminal evidence/unknown partial work as applicable;
they never become a made-up zero score or utility. No cap limits are increased.

## Fixed seed coupling and schedule

Scientific master seed `0xE657000001` = 989302095873. Replicate r's optimizer
seed is master+1,000,000+r, shared across the two arms; optimizer RNG starts
fresh in each arm. Thus the first perturbations are coupled, but later learned
parameters and allocation of perturbations to samples can diverge.

Each replicate has the same 160 draw-pool indices 0..159 in both arms, each
evaluated for + and − candidates. With batch size k, generation g, pair p,
batch draw b, pool index n=(8*g+p)*k+b. Emission seed master+10,000+1,000*r+n;
physics seed master+100,000+100*r+n%4. The +/− arms share the whole batch's
emission and physics seeds. Single k=1 visits that same pool across twenty
generations; batch k=4 visits it across five. Replicate seed domains do not
overlap. No random resampling, candidate replacement or outcome-dependent
allocation occurs. Fixed training order: replicate, single then batch,
generation, pair, + then −, batch draw.

Save every final-generation champion before any held-out physics. Held-out
panel: 64 shared fresh emission draws, four physics seeds each. Emission seed
master+20,000+draw; physics master+200,000..200,003. The same panel is applied
to all sixteen champions. Run the zero-logit untrained baseline once (256 cells),
then replicate, draw, single/batch, physics. Sharing this panel reduces comparison
noise but does not multiply independent training replicates; interpretations
are conditional on this fixed held-out panel and narrow world.

Total proposed scientific work: 5,120 training + 4,096 champion held-out + 256
untrained held-out = 9,472 evaluations; at most 947,200 native attempts/steps.
Ceilings: one grammar rebuild, 3,600 seconds including setup/final checks,
cooperative 30-second cell deadline, 2-GiB output, 128-MiB receipt reserve,
16-MiB final reserve. Actual times/advances/caps are reported, not assumed equal.

## Registered analysis and stopping

The primary statistical unit is the **paired training replicate**, denominator
eight. Within each replicate/method average the four physics seeds per draw,
then all 64 draw utilities. Held-out descriptive U uses native f64 reuse without
the optimizer's f32 cast: 1+reuse for valid outcomes, 0 for measured resource caps.
Primary effect is the mean of eight batch−single replicate differences. Exact
one-sided paired sign test excludes ties only from its binomial denominator.
Do not treat 4,096 champion cells or 512 replicate/draw combinations as
independent training trials. Eight wins gives p=1/256; seven wins and one loss
gives p=9/256. Report effect sizes and all eight replicate outcomes, not p alone.

Supporting batching requires completed training/held-out work, every utility
outcome known, positive primary mean, p<.05, and aggregate batch feasibility
count at least single count (each /2,048). This point-count guard is not a
statistical noninferiority guarantee. Utility can trade reward against caps
across samples, so feasibility and native reward remain separate reports.
The once-evaluated untrained utility is descriptive baseline context only; no
extra confirmatory test or best-champion selection is introduced.

Native batch−single mean and sign p are separately available only when all
4,096 champion comparison cells are valid. Any native cap keeps that full native
comparison null; no complete-case substitution. Missing or unknown utility
outcomes keep the full primary utility mean/p null with denominator eight;
available pairs are diagnostic only. No interim significance checks, retries,
pooling of prior evidence, seed replacement or criterion changes. Any returned
failure stops the entire fixed schedule, including the other method, rather
than preferentially finishing/selecting a survivor.

## Development milestone and durable audit

Separate development master `0xE657D00001` = 989315727361. One replicate,
single two generations × sixteen × one draw versus batch one generation ×
sixteen × two draws: 32 rollouts per method. Two held-out draws × four physics
seeds for both champions plus untrained baseline: 24 rollouts. Total 88 evaluations,
at most 8,800 steps, 900-second ceiling. This bounded fixture uses the same
execution/audit path but always has `support_batching:false`; its horizon,
replicate count and batch size do not replace scientific evidence. It is allowed
for the present implementation task; the 9,472-cell scientific invocation is not.

Freeze complete schedule and all source/config/protocol/checker/test dependencies
and binary before physics. Precreate every NotRun slot. Save generation-start
noise and pre-update parameters before native work, then save its completed
aggregation/update; a partially evaluated batch cannot masquerade as completed.
Save in-flight marker before evaluation and each terminal row atomically before
counter commits. A hard kill can leave unknown in-flight work; no per-step crash
journal or OS-preemptive timeout promise is made. Recheck live identities after
compilation, every generation and held-out replicate, at completion; reverify
both parent and latest seals at completion. Store generation records without
redundant full emission-distribution panels to bound output growth.

Tests cover batch-one backward compatibility, fixed batch denominators/caps,
mean-versus-strict-feasibility distinction, matched seed pools, no cross-replicate
overlap, true replicate-level sign denominator, missing outcomes, partial batch
refusal, real native cap receipts and update tampering. The streaming auditor
reconstructs membership/native accounting with pinned parent helpers, all
candidate repetitions/coupling, means/ordinal ranks, f32 updates and champions,
replicate statistics, source/binary/exact seals and fixed resource allowances.
Detached tamper controls must reject altered batch membership/seed/budget,
candidate identity, survivor-mean imputation, updates, champions and false
rollout-level statistical denominators. Development and scientific classifications
must remain distinct. This is accounting/provenance, not physics/RNG replay or
compiler-to-binary proof, and establishes no maintenance/repair/novelty or
long-horizon generalization.
