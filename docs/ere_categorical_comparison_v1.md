# Categorical versus ES: prospective native direction comparison

Date: 2026-10-01. Profile `E2-categorical-versus-es-direction-v1`.
This preregisters the next native comparison after the verified adapter.
**No native harness exists for this design yet; no development/scientific
run or training is launched or authorized by this document.** Separate launch
approval follows implementation, frozen audits and a clean development receipt.
The [machine-readable plan](../configs/ere_categorical_comparison_v1.json)
is the numerical/schedule contract; any correction requires a separately
identified prospective revision, not editing a measured design.

## Question and improvements to the previous assay

At the same fixed native evaluation budget, does direct categorical estimation
produce independently reproducible directions that improve expected constraint
utility, and improve it more than rank-ES directions at a matched step size?
This is a local direction comparison, not multi-generation learning or a claim
that the existing optimizers have interchangeable gradient targets.

The previous assay failed its frozen alignment/usefulness criteria. Its read-only
mapping found very small distribution movement, much greater shared-CDF parent
churn, and only five reward-changing forward comparisons. The categorical
adapter passes full-catalogue engineering checks but has no native benefit result.
Preserve all of those archives and verdicts; do not pool their outcomes here.

Two prospective changes address the diagnosed interface:

1. Center each raw direction within its three softmax groups and use a single
   fixed L2 step radius for both methods. This removes group-constant gauge
   components and raw step amplitude as comparison confounders. It changes the
   assay intervention, **not** either optimizer's shipped update or old results.
2. Validate every proposal by probability-reweighting one common, fresh,
   base-sampled native panel. Every sampled leaf contributes a continuous weight
   contrast instead of comparing only occasional inverse-CDF rule switches.
   This is an importance-weighted categorical-model estimand, not newly observed
   native rollouts of each proposed vector or a guarantee of lower variance.

Keep the two old, fixed, length-183 contexts: all zeros; alternating +.25 at
even indices and −.25 at odd indices. No trained champion, favorable seed,
activation reward or historical parameter selection enters the experiment.

## Matched allocation and independent estimates

Eight independent blocks each contain both contexts and both methods. Each
method supplies two estimates A/B at the base context; never sequential updates.
All estimates and forward/reverse vectors must be saved before any validation.

| Method, per estimate | Scientific native allocation | Update reconstruction |
| --- | ---: | --- |
| ES | Eight antithetic pairs, 16 candidates × four emissions = 64 | Existing sigma-one, alpha f32 .05, fixed batch-mean ordinal/rank adapter and unchanged `OpenAiEs` |
| Categorical | 64 distinct base-context emissions, one physics evaluation each | Existing raw-utility score estimator with emission-level LOO baseline and alpha f32 .05 |

ES positive/negative candidates share emission/physics seeds within each pair
and batch draw, as antithetic controls. Different pairs, contexts, blocks,
methods and A/B panels use disjoint seed identities. Unlike the previous assay,
ES A/B use **independent perturbation bases** as well as disjoint native panels.
Categorical A/B sample independently at their base context; physics repeats are
not counted as independent LOO samples. This tests independently resampled
directions, not conditional alignment given a common ES basis.

Matching native evaluations does not match information structure: ES uses 32
distinct paired seeds per estimate while categorical uses 64 distinct joint
emission/physics seeds. That is part of the two estimator designs, not a pure
variance comparison holding every coupling fixed. ES retains its existing f32
native-score cast for candidate ranking; categorical retains native f64 utility.
Record both raw scores/utilities and exact numerical operations. No native null
is treated as a valid reuse score: measured caps participate as utility zero.

## Fixed direction intervention

For each method/estimate save its raw f32 post-update vector at alpha f32 .05.
Compute `d_j=f64(raw_after_j)−f64(base_j)`. Subtract the mean of `d` separately
over parent [0,168), operator [168,181), and role [181,183). If the centered
norm is exactly zero, use base as forward, mark degeneracy, and retain all work.
Otherwise use:

```text
forward_j = f32(f64(base_j) + 0.1 × centered_d_j / ||centered_d||₂)
reverse_j = f32(2 × base_j − forward_j)  [native f32 operations]
```

The radius .1 is fixed once as a round engineering-scale intervention, not an
alpha scan, estimate-specific tuning or choice from held-out effects. Save raw,
centered and actual rounded directions/norms and exact vectors. Reject nonfinite
arithmetic, actual norm or any absolute coordinate step above .100001, for both
forward and reverse. Do not clip, rescale a second time or replace a direction.
Reference f64 sums use left-to-right accumulation in the saved coordinate/draw/
context/block order, without fused multiply-add. Cast operations are explicit;
the future native implementation and checker must agree on saved f32 bits.
The matched-radius intervention may amplify noisy small directions; the fresh
native test is meant to reveal that, not presume it helps.

The old scientific contract and adapter's raw-update API remain unchanged.
This new assay's centered/radius-matched proposals are a separate caller-level
operation. It cannot claim results for unmodified alpha-.05 training updates.

## Shared native validation and importance-weighted estimand

Per block/context, sample 128 fresh base-context leaves. Evaluate each complete
leaf at four fresh physics seeds, independent of all estimation panels. Record
every native evaluation. Average all four constraint utilities per leaf:
`U_i=mean_p(1+reuse_ip for valid; 0 for measured cap)`. Keep cap native scores null.
Let `V_i` be its four-physics valid fraction, not an all-or-nothing leaf deletion.

Compute the full legal **leaf** probability under base and each of eight virtual
proposals (four forward, four reverse). Do not use executable-signature masses,
which can lose proposal-credit identity. With `w_i=Pproposal(leaf_i)/Pbase(leaf_i)`:

```text
utility_effect(proposal) = mean_i [(U_i − 1) × (w_i − 1)]
validity_effect(proposal) = mean_i [(V_i − 1) × (w_i − 1)]
estimated_proposal_utility = observed_base_mean_utility + utility_effect
```

The fixed constant 1 is a control variate, not a replacement reward or fitted
held-out baseline. Because both complete probability laws normalize,
`Ebase[w−1]=0`, so subtracting 1 preserves the expected utility difference under
the categorical model. A cap contributes through utility zero, never disappears.
Use the ordinary fixed denominator 128. No clipping, self-normalization,
effective-sample-size denominator, complete-case mean or outcome-based filtering.
Save weight/log-probability terms and actual utilities for independent auditing.

The evaluator accepts an emitted program/store and physics seed, not theta:
the parameter vector influences selection only. Therefore this model-based
identity is applicable to this fixed grammar/world. Before native launch, source
inspection and development must check that leaf reconstruction, metadata/world,
proposal credit and evaluation are unchanged by supplied logits or emission seed
beyond catalogue selection. Any violation blocks the design; do not substitute
signature equivalence, ignore an extra dependency or redefine the target.
Finite-RNG threshold differentiation/bit-exact sampling-law equality is not
claimed: as in the adapter, the model is the implemented f64 softmax/uniform
categorical law and fresh seeded native sampling is treated accordingly.

Fixed coordinate step bounds yield per-stage probability ratio within
`exp(±2×.100001)`, hence full leaf ratios within `exp(±6×.100001)`:
approximately .5488083 to 1.8221298. Audit full-support normalization and this
bound for **every legal leaf**, not only observed leaves, before validation.
Uniform production-index probabilities cancel within each fixed parent/operator/
role group, so the 3,232 group ratios cover all 18,956 leaf ratios exactly;
retain full leaf identities for native outcomes and credit accounting.
A weight outside the bound is a hard accounting failure, not a clip request.
Also report weight ranges, second moments and unnormalized probability-mass
estimates descriptively; do not select proposals using these diagnostics.

No proposed-arm native availability count exists in this design: its native
receipts are base-only validation. Weighted validity is an estimated point
effect, not proof of feasibility/noninferiority or all-valid proposal support.
Likewise estimated proposal utilities are not empirical proposed-arm means,
complete expected reward enumeration or learned maintenance. Baseline native
reuse full-panel mean is null if any baseline evaluation is capped; constraint
utility/effect estimates remain available for known measured caps.

## Seeds and exact order

Scientific master `0xE66000000000` = 253299991248896; development master
`0xE66100000000` = 253304286216192. Let block r, context c, method m=0 ES/1
categorical, estimate e=0 A/1 B, pair p, batch draw b, direct draw i, validation
draw d and physics j. Set `J=1,000,000*r+100,000*c+10,000*m+1,000*e`.

- ES noise: master+10,000,000+J; independent A/B draws via unchanged ChaCha8/
  standard-normal implementation; save every f32 perturbation.
- ES emission/physics: master+20,000,000+J+4*p+b and
  master+40,000,000+J+4*p+b, shared only for its +/− candidates.
- Categorical emission/physics: master+20,000,000+J+i and
  master+40,000,000+J+i, 64 distinct science draws per estimate.
- Validation emission: master+60,000,000+1,000,000*r+100,000*c+d.
- Validation physics: master+80,000,000+1,000,000*r+100,000*c+4*d+j.

Scientific and development seed spaces are disjoint, including noise/emission/
physics domains. Check for collisions with recorded prior ERE schedule domains
before launch; a collision requires an explicit prospective revision before work,
not silently replacing seeds. Estimation order: block/context/method/estimate,
then ES pair/+−/batch or direct draw. Validation order: block/context/draw/physics.
No native proposal/reversal evaluations or extra validation seeds are scheduled.

## Statistics and fixed acceptance

For each method/block/context, alignment is cosine of actual rounded forward
directions A/B; either exactly zero norm assigns alignment zero, with explicit
degeneracy. Average both context alignments per block. For each context, average
A/B utility effects for each method. Form eight block values:

```text
C_r = mean_context(categorical mean-forward utility effect)
D_r = mean_context(categorical effect − ES effect)
L_r = mean_context(categorical A/B alignment)
```

The unit is the **paired estimator block**, denominator eight—not contexts,
directions, leaves or physics evaluations. Official means retain all blocks and
actual values. Use fixed ±1e−12 sign deadband: above it is positive, below its
negative is negative, within is tie. Exclude ties only from the exact one-sided
binomial sign-test denominator. With no non-ties p=1. Unknown direction or
utility makes all confirmatory means/p null; never use survivor blocks.

`support_categorical_direction` requires scientific mode, complete fixed work,
all outcomes known, mean C/D/L each >1e−12 with each corresponding p<.05,
categorical mean utility effect >1e−12 in both contexts, and aggregate
categorical weighted validity effect ≥−1e−12. The validity guard is only a
point-estimate guard, not statistical noninferiority or admission. This is an
all-properties conjunction; no single passing endpoint is promoted as overall
success. ES alignment/usefulness, method alignment differences, reversals,
forward-minus-reverse, individual contexts/blocks and weight diagnostics are
descriptive only. No extra reverse-direction hypothesis, alpha selection or
post-hoc threshold adjustment. Small positive effects are not automatically
practically important or evidence of multi-generation learning.

## Budget, failure and development

| Layout | Estimation | Base-only validation | Total native evaluations | Maximum attempts |
| --- | ---: | ---: | ---: | ---: |
| Science: eight blocks, 64/method/estimate, 128 validation draws | 4,096 | 8,192 | 12,288 | 1,228,800 |
| Development: one block, 16/method/estimate, eight validation draws | 128 | 64 | 192 | 19,200 |

Development uses one emission per ES candidate (k=1 versus k=4), with
eight pairs unchanged, and 16 direct emissions. It has eight direction estimates;
science has 64. Every layout validates four physics evaluations per base draw.
Development always classifies engineering-only and has support false, even if
all numerical point criteria happen to pass.

Native 100-step fixed world, metadata isolation, typed grammar/coherent kernel,
MaxParallel/one rewrite, opt-in repeated-self-pair matcher, pooled 50k/50k,
node/edge ceilings 2,000/4,000, original reward and optimizer core remain fixed.
One grammar compilation; 3,600 s science / 900 s development including setup,
30 s cooperative cell deadline, 2 GiB output with 128 MiB receipt and 16 MiB
final reserves; separate final auditor ceiling 900 s. These are limits, not
runtime/size guarantees. The scientific budget is a proposal requiring approval.

Measured caps retain utility zero and null native reuse. Unsupported failures,
partial unknown work, panics, deadline or archive/source violations stop the
whole sequence with all remaining NotRun slots and official statistics null.
No retry, replacement, dropped cap, reused unused work, interim significance,
early statistical success or criterion change. Persist InFlight before native
calls; commit native receipts/work before advancing schedule state. No per-step
crash journal or OS-preemptive timeout is promised.

## Provenance, narrow source registration and implementation requirements

Bind the failed direction root/7,686 entries as diagnostic lineage only, the
passing adapter engineering root/734 entries as arithmetic evidence only, and
the preserved failed adapter packaging root/732 entries as historical failure.
The config pins their exact seals. None supplies scientific success for this
comparison. No archived outcome is used as a new estimation/validation sample.

The plan check proves the existing adapter's two learning registration changes
exactly: the hg-train Cargo manifest only appends the required-feature offline
binary; the module registry only adds the gated categorical export. Of the 459
historical paths, 452 other ERE/shared paths must still match exactly. Five
paths within the separate `evolving-minds` feature are explicitly excluded from
this prospective ERE source guard, with historical and observed live hashes
reported individually. The initial broad preflight detected concurrent choice
panel work in those five paths; none was edited or reverted by this task.
The unchanged sim-core library and Cargo manifest gate that module/binary
behind `evolving-minds`, which the ERE feature does not enable. New native
builds must explicitly disable that feature and bind their own complete source
inventory and feature/build receipt. This is not permission to ignore shared
source drift, enable evolving-minds in a comparison, or loosen any historical
admission guard. These observations are not an evolving-minds correctness audit.
The plan also checks all 462 non-excluded inputs from the adapter's 467-entry
source inventory, including its newly added implementation and engineering checks.
A future native
harness's exact additional learning registrations must be captured in a separate
implementation source receipt before development, while native crates and
`optim.rs` remain pinned to their existing identity. Changed native semantics
require a new prospective design, not a registration exception.

Before recommending scientific launch, implement/test the new native runner and
independent frozen accounting auditor; freeze all sources/config/contract/tests,
schedule, contexts, source-registration receipt and binary before native work.
Check raw estimators, independent noise identities, matched-vector arithmetic,
all-leaf probability/weight bounds, fixed effects/deadbands/statistics, caps and
NotRun/in-flight accounting. Development and detached tamper controls must pass
without postmeasurement checker changes. Recheck sources after compile and each
estimation context/validation block, and seals at completion. Preserve failures
and register any corrections/continuations separately. Auditor scope remains
accounting/provenance, not independent physical/RNG/compiler replay.

The current task creates and validates this prospective plan only. The next
implementation task is the native harness and auditor, followed by separate
development verification and scientific launch approval. No experiment result,
training admission or variance reduction has been established here.
