# Complete-family 100-step training v1

Registered 2026-09-30 before fresh prechecks or scientific physics. The user
authorized the reliability fix and this 100-step campaign after the scaling
failure. Profile `E2-complete-short-training-v1` explicitly supersedes the
requirement to complete a 10,000-step diagnostic for this shorter campaign.
The previous scaling result remains incomplete and failed. The legacy complete
training and maintenance guards retain their existing refusal behavior.

## Implementation and admission

Carry the five exactly sealed deterministic, long-gate, closure, finite-grammar
and runtime receipts from `configs/ere_complete_receipts_v1.json`. Historical
scopes and the earlier corrected-checker lineage remain explicit. Verify every
sealed file before setup and after measurement. Verify all 230 pinned native/
Cargo/optimizer sources. The only permitted native difference is the previously
registered whole-source horizon/accounting bridge, followed by exactly one
capture-buffer reset between reading the current step and calling Engine::step.
Rust and Python independently check that entire-source transformation.

The capture reset prevents an Engine early exit from retaining a previous event.
Regression controls must cover a cap after actual rewrites, both with a smaller
development budget and at the original 50k/50k long horizon, plus exact comparison
of an uncapped 100-step receipt against historical native evidence. These are
development tests and do not replace or resume the failed scientific panel.

Freeze the complete learning crate, all pinned sources, contract, config, checker,
checker regressions and executing binary before rebuilding the grammar once.
Require identical catalogue SHA256
`ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6`.
Recheck frozen/live identity after compilation, before each precheck/training
generation, before held-out work and at completion. Keep the fixed ordinary
world, frozen decoded metadata, coherent kernel, 183-logit typed grammar,
MaxParallel/one rewrite and pooled 50,000 work/50,000 results unchanged.

## Exact schedule and budget

Master seed `0xE654000001`. Zero initial logits; OpenAI-ES sigma 1, alpha .05,
ChaCha8 and average tied ranks. No score previews, retry, seed replacement,
fitness-based culling or extra campaign after failure.

All native evaluations have horizon 100. The schedule has 1,208 fixed slots:

| Stage | Evaluations | Maximum native steps |
| --- | ---: | ---: |
| Fresh stateability prechecks | 120 | 12,000 |
| Training: 20 generations x 16 candidates | 320 | 32,000 |
| Held-out: 64 emissions x 4 physics seeds x 3 arms | 768 | 76,800 |
| Total | 1,208 | 120,800 |

Common distribution: 64 emission seeds master+1000..1063, no Engine work.
Precheck baselines: eight emissions master+2000..2007, each at physics seeds
1601,1699,1801,1901. Matched controls: parent4/Activate and parent5/Deactivate,
Proposal/production0, master+3000, each at these four seeds. Verify matched Core,
Proposal and ordinary world with opposite activation before control physics.
Precheck ES: five generations, four antithetic pairs, two arms, two physics seeds
1601/1699; shared emission master+4000+4g+p. Precheck optimizer seed is master.

P1: active signature absent from initial common distribution. P2: maximum
common-seed distribution TV >= .05 and all 20 pairs valid. P3: active-minus-
dormant mean > twice baseline population SD and > 1e-12. P4: baseline mean
strictly between .02 and .98. Null baselines fail. Runtime envelope additionally
requires all 120 evaluations to complete 100 advancing, valid, uncapped steps.
If any condition fails, training and held-out slots stay NotRun. No repair or
replacement of the scientific precheck schedule follows that outcome.
The prospectively frozen checker verifies fresh prechecks and the zero training
counts before entry into training; persist its admission audit in the archive.

Training optimizer is reset to zero logits and RNG seed master+10000; it never
inherits precheck updates. Eight pairs per generation, emission master+5000+8g+p
shared by plus/minus arms. One physics seed per candidate, cycling the independent
training seeds [2003,2111,2203,2309] by (g+p) mod4. Cap/null fitness withholds
both pair members; successful zero reuse is numeric zero. Each generation needs
at least four complete pairs; otherwise persist the skipped update and stop.
Flat fitness legitimately makes a zero update. Keep the denominator eight pairs
in optimizer scaling. No early stopping based on favorable/unfavorable fitness.

## Objective, champion and held-out comparison

Unchanged reward: ln(1+max(credited Proposal fires-1,0))/ln(100). Only active
Proposal execution distinct from priority-normalized Core receives credit.
Training learns a distribution of initial complete families and edits in a fixed
world; this objective measures bounded Proposal reuse. It cannot establish learned
repair, online metadata adaptation or long-horizon maintenance.

The designated champion is the final generation's updated parameter vector,
selected prospectively. No generation or candidate is chosen using held-out
scores. Persist all perturbations, pair scores, distributions and parameter
updates. Native evaluation receipts include events, budgets, caps and graph sizes.

Held-out emission seeds master+6000..6063 and physics [2411,2503,2609,2707] are
fresh and shared across champion, zero-logit untrained and random-resample arms.
For each emission draw, random-resample uses a new 183-dimensional standard-normal
parameter vector with RNG seed master+7000+i, shared across its four physics seeds.
Persist those vectors. Mean each genotype's four seed scores; primary statistic
is the mean of 64 paired champion-minus-untrained differences. Secondary statistic
is champion-minus-random. Preserve all nulls and the fixed denominator: full
estimates become null if any held-out evaluation is incomplete/null. Report valid
paired counts separately. Support for improvement requires a positive primary
mean and one-sided exact paired sign p<.05 over the 64 genotype means, excluding
ties from the sign-test denominator. This test concerns the sign of paired
differences; do not describe it as a confidence interval for the mean.

## Failure, resources and audit

Create all schedule slots as NotRun before setup. Write each current-cell marker
before evaluation; atomically replace that slot with its native receipt afterward.
Catch native panic/returned error, persist Incomplete with unknown partial work,
and stop the campaign. Remaining slots stay NotRun. Known counters cover returned
native receipts only. A hard process kill can still leave an in-flight marker;
the runner does not claim OS-level crash recovery or independently recorded steps
inside a panicked evaluation. Resource caps produce native null receipts.

Ceilings: 3,600 seconds including setup and final identity verification; 30 seconds
per evaluation, cooperatively checked; 512 MiB output with 64 MiB receipt reserve
and 16 MiB final headroom. Existing 2,000-node/4,000-edge caps remain. No refill or
ceiling expansion. Failures preserve the archive; one scientific invocation only.

Freeze and test an accounting checker before the run. It verifies source bridges,
all prerequisite seals, fixed schedule, memberships, native reward/cap accounting,
candidate arithmetic, pair masking, f32 optimizer updates, prechecks and held-out
statistics. It seals all outputs except its own root seal. It does not independently
replay native physics or RNG streams or prove compiler-to-binary correspondence.
Training admission applies solely to this registered 100-step campaign after its
fresh prechecks; larger horizons and the reserved 20x16x10,000 campaign remain
outside this authorization. Report the outcome in Markdown even if prechecks fail.
