# ERE saved-receipt coverage, seed variation, and attainable-performance review

Date: 2026-10-03. Classification: exploratory analysis of exposed historical
receipts. No new native evaluations, Engine steps, grammar compilations, or
training updates. No scientific criterion or historical verdict changed.

## Decision

The current fixed-world reuse task has a near-maximal policy already expressible
with its existing 183 logits. Across every measured same-leaf, multiple-physics-
seed case, the recorded outcome and step trace are unchanged. Nevertheless,
55.74% of production leaves remain unmeasured, and production-level choices can
mix good outcomes with caps that the current policy cannot distinguish.

These findings argue for two different priorities:

1. If continuing to study this optimizer benchmark, spend new evaluations on
   distinct productions, use the known positive-control selector as a comparator,
   and retain a small, fixed seed-variation panel. Do not spend another campaign
   chiefly repeating unchanged leaves or tuning sparse risk estimates.
2. For the original project's scientific goal, move to a bounded **in-run rule
   birth and lifecycle fixture**, using the existing epoch/source machinery.
   Selecting an excellent rule before execution is not evidence that a novel
   rule arose during execution. Another sampler redesign cannot close that gap.

I recommend the second as the next main-project implementation step. Production
logits are a justified later extension for a task requiring particular edits,
but are not necessary to reach the ceiling of this existing reuse proxy.

## Scope and verification

The analysis includes ten completed native production-grammar campaigns, chosen
before the first analyzer invocation. It checks 45,408 terminal cell receipts,
including 45,285 valid cells and 123 resource-cap/null cells, and accounts for
4,534,539 historical attempted steps and 4,534,416 advances. These are previously
run steps, not compute consumed by this review.

Every selected input's exact seal coverage and file hashes are verified before
and after analysis. The common catalogue hash is
`ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6`.
All full frozen source maps are verified and retained in the result. The analyzer
uses the historically pinned checker stack, validates emission membership and
credit identity, and reconstructs native receipt accounting from saved traces.
It also checks the supplementary historical comparison audit and parent/runtime
lineage. This is **not independent Engine or RNG replay**.

The original categorical scientific archive still has its failed frozen audit.
Its separately authorized corrected historical accounting supplies the accounting
boundary here; neither the original failure nor its failed scientific validity
guard is overwritten. The analysis is not a retrospective acceptance test.

Excluded: earlier encoding/substrate campaigns, failed or incomplete scaling
attempts, zero-work preflights, synthetic fixtures, and unrelated LK/evolving-
minds experiments. This is an ERE production-grammar review, not a new inventory
of every project experiment. Development rows and forced controls are included
descriptively and remain identified by their original archive/stage.

Seven focused tests pass: duplicate/conflict handling; missing-mass preservation;
empty/full coverage; separation of outcome variance from trace variance;
incomplete-group handling; cap/unknown utility handling; and actual catalogue/
positive-control probability checks. A separate read-back checks the final seal,
manifest, test receipt, counts, leaf union, and group-table dimensions.

## What the policy actually controls

The catalogue has 18,956 production leaves, 3,232 nonempty parent/operator/role
groups, and 168 parents. A leaf is the full tuple
`(parent, operator, role, production_index)`, with Core=0 and Proposal=1.

The policy factorizes as:

```text
168 parent logits
  -> 13 shared operator logits, masked by the selected parent's legal support
  -> 2 shared role logits, masked by the selected parent/operator support
  -> uniform production_index within that group (no trainable logits)
```

For group g containing k productions, each leaf has probability `P(g)/k`.
`valid_grammar::emit_selected` builds that leaf's fixed program/store; theta
chooses the leaf but does not otherwise alter its program. Source:
[`valid_grammar.rs`](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere/valid_grammar.rs).

Execution signatures alone are not a valid reward-cache key. Two emissions can
have the same executable bodies but different core/proposal credit identity.
This review retains complete leaf identity and checks classes, signature,
initial-world hash, and root identity; it does not merge execution signatures.

## Coverage and repetitions

| Saved campaign | Cells | Distinct leaves | Distinct leaf/physics units within archive | Cap rows |
| --- | ---: | ---: | ---: | ---: |
| Short-training continuation | 1,208 | 480 | 983 | 0 |
| Champion confirmation | 6,144 | 972 | 3,888 | 44 |
| Feasibility training | 6,464 | 1,291 | 4,400 | 24 |
| Minibatch trial | 9,472 | 3,605 | 5,313 | 17 |
| Direction assay | 7,168 | 1,930 | 3,179 | 5 |
| Categorical scientific comparison | 12,288 | 4,067 | 12,287 | 29 |
| Feasible-v2 development | 2,176 | 1,776 | 2,176 | 4 |
| Categorical comparison development | 192 | 141 | 192 | 0 |
| Minibatch development | 88 | 64 | 70 | 0 |
| Direction development | 208 | 133 | 148 | 0 |

Distinct-count columns are not additive across archives. Across source partitions
there are 32,636 distinct source/leaf/physics units and 12,772 repeated rows
(28.13% of raw rows). Repeated units must have exactly matching recorded outcomes
and step-trace hashes; a conflict fails the analysis instead of being averaged.

The descriptive union covers:

- 8,390 / 18,956 leaves: **44.26%**; 10,566 leaves remain missing.
- 3,215 / 3,232 groups: **99.47%**; almost every group has been touched.
- All 168 parents.
- 76.28% of zero-policy leaf probability and 77.82% of alternating-0.25 policy
  probability. The policy is not uniform over leaves, so count coverage and
  probability coverage differ substantially.

The bottleneck is not absence of whole parent families. It is incomplete coverage
inside groups and repeated draws of already exposed leaves. The union is a
coverage description only; rewards from different source partitions are not
pooled into a policy-value claim.

## Source partitioning and seed variation

The ten archives contain eight distinct binary hashes. For descriptive analysis,
the reviewer partitions them by the hashes of 67 common native source/build
paths: 59 non-training crate source files plus eight Cargo manifest/lock paths.
All 59 common native source files match across the archives. The two resulting
partitions differ in `Cargo.lock` and `crates/hg-sim-core/Cargo.toml`.

This is deliberately a coarse reporting partition, **not a certificate of
whole-binary equivalence, current-checkout identity, or reusable native cache
portability**. Full source maps and binary hashes remain attached to each archive.
Within-archive seed comparisons are also reported independently.

| Reporting partition | Distinct leaves | Leaves with >=2 physics seeds | Score/reason variations | Cap variations | Step-trace variations |
| --- | ---: | ---: | ---: | ---: | ---: |
| Historical nine-archive partition `2100c8560ac7…` | 7,853 | 4,691 | 0 | 0 | 0 |
| Feasible-v2 development partition `79de2ad64dd5…` | 1,776 | 286 | 0 | 0 | 0 |

There are also zero varying cases in every archive considered separately. The
trace comparison covers recorded world hashes, rule IDs, usage, sizes, cap flags,
and advancement, not unrecorded RNG state. Tiny computed sample variances up to
`7.40e-32` are floating-point mean/subtraction roundoff; the recorded utility
values themselves are exactly equal within each observed multi-seed leaf case.

This is strong evidence that four routine physics repeats are adding little
information in this particular saved fixed-world fixture. It is **not** proof
of determinism for all unmeasured leaves, seeds, worlds, horizons, or dynamic
rule-source configurations. A future coverage panel should retain a predeclared
seed-variation check rather than assume this generalizes.

## Production-level heterogeneity

| Reporting partition | Groups with full production coverage | Fully covered groups with heterogeneous utility | Fully covered groups mixing cap outcomes |
| --- | ---: | ---: | ---: |
| Historical | 1,656 | 76 | 2 |
| Feasible-v2 development | 546 | 2 | 0 |

Another 252 historical and 26 feasible-v2 groups have observed utility variation
when incomplete and complete groups are both counted. Incomplete groups do not
receive an empirical uniform-group mean: unseen productions are not silently
removed from the denominator.

Concrete examples from complete historical groups:

| Parent / operator / role | Production outcomes U | Current uniform-production group mean | Best observed production U |
| --- | --- | ---: | ---: |
| 101 / 4 / Proposal | index0=2; index1=0 (cap) | 1 | 2 |
| 113 / 4 / Proposal | index0=2; index1=0 (cap) | 1 | 2 |
| 7 / 7 / Proposal | index0=2; index1=1 | 1.5 | 2 |

`U=1+score` for a valid full-horizon receipt and `U=0` for a resource cap.
These means average the observed distinct physics outcomes per production;
different productions can have different seed panels. They are not a common-
panel population estimate. The same 7/7/Proposal split is observed independently
in the feasible-v2 development partition.

No choice of the existing 183 logits can favor index0 over index1 **within**
101/4/Proposal. A production-specific logit layer would fix that particular
restriction. It would not ensure unseen-cap safety or establish rule emergence,
and it is unnecessary to attain this benchmark's global ceiling because a
single-production positive-control group already exists.

## Attainable performance under the current representation

The positive-control leaf is `(4,11,1,0)`: parent4, Activate operator11, Proposal,
production0. Its group contains exactly one production. It has valid score1
(`U=2`, the maximum) at all 16 distinct historical physics seeds and all four
distinct feasible-v2 development physics seeds where it was observed. Forced
controls are exposed engineering evidence, not independent learning success.

A finite, analytic policy assigns +8 to parent4/operator11/Proposal logits and
-8 to every other logit. It places **99.9980419%** of sampling probability on
this leaf. It is not a fitted optimizer update, and this review does not run a
new native panel under that policy.

To avoid extrapolating observed performance to unseen leaves, the analysis
retains all missing probability. Conditional on each leaf's recorded empirical
mean, missing eligible leaves get `U in [0,2]`; missing structurally ineligible
leaves get `U in [0,1]`; missing cap indicators get `[0,1]`.

| Partition / analytic policy | Observed probability mass | Conditional U range | Conditional cap-probability range |
| --- | ---: | --- | --- |
| Historical / zero logits | 0.741414276 | [0.890299529, 1.281713418] | [0.001311856, 0.259897580] |
| Historical / positive-control selector | 0.999997263 | [1.999982559, 1.999987818] | [0.0000000251, 0.0000027617] |
| Feasible-v2 / zero logits | 0.283283867 | [0.330923666, 1.390052054] | [0.0000770352, 0.716793169] |
| Feasible-v2 / positive-control selector | 0.999990389 | [1.999973543, 1.999990723] | [0.0000000017, 0.0000096124] |

These are **missing-support bounds conditional on measured leaf means**, not
confidence bounds on true future performance or population cap risk. Physics
panels differ by leaf, selection was historical/adaptive, and exposed controls
are included. Observed support is never renormalized. No complete fixed-seed
reward table or exact population gradient is claimed.

Nevertheless, the architectural conclusion is clear: strong performance is
available without increasing the parameter count. Difficulties in learning a
small safe update should not be treated as evidence that this grammar cannot
express a high-performing fixed-rule policy.

## Recommended next implementation: a small rule-lifecycle fixture

The original H-R1 asks for rules that arise after initialization, are RCF-distinct
from every initialized rule, fire on non-meta structure, and persist longer than
their replacement timescale. The current sampler evaluator installs its rules
once and uses `EpochPolicy::Never`:
[`signal.rs`](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere/signal.rs). Its reward credits repeated
use of the initially selected proposal. Even a perfect optimizer on that reward
does not demonstrate H-R1.

Implement one bounded engineering fixture before another training campaign:

1. Reuse `GraphRuleSource`, `EpochPolicy::Fixed`, the existing paid-edit vocabulary,
   complete-witness encoding, and ERE runner instrumentation. Do not introduce a
   second physics engine or another broad gate framework. Existing seams:
   [`source.rs`](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/source.rs),
   [`ERE runner`](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere.rs).
2. Record the complete initial RCF set and seeded-editor identities. Record the
   first post-initialization appearance, successful installation, and non-meta
   firing of a distinct rule. Birth, admission, and use must be separate events.
3. Track lifetime/replacement by exact rule class, not just survival of its root
   node. Audit existing lifetime telemetry before reusing it as the H-R1 endpoint.
   A changed rule in a surviving anchor is not survival of the original rule.
4. Use fixed live-epoch versus frozen-source and edit-disabled/unpaid controls.
   Show the edit is enacted through the paid native rewrite path, rather than
   injected by the runner. Preserve seeded motifs as seeded, not counted novelty.
5. Treat a scripted birth/control demonstration as engineering feasibility only.
   Only after the dynamic reward and causal controls work should a separately
   registered learner try to cause those events under a fixed compute budget.

The next artifact should be a compact fixture specification plus implementation
and diagnostic tests, not another large optimizer campaign. Choose its seeds,
horizon, denominators, censoring rules, and success criteria before native work;
seek separate approval for that compute. If the fixture cannot produce and
attribute a genuine post-initialization rule, fix that failure before training.

If the priority instead remains optimizer methodology, complete a source-bound
distinct-production panel, retain a small fixed seed probe, and compute finite-
catalogue sums/gradients only for the explicitly measured table. Use the known
positive-control selector to expose headroom; keep missing support explicit.
Adding production logits should be motivated by a task requiring mixed-group
choices, not by this already-solvable fixed-rule proxy.

## Reproduction and artifacts

Analyzer: [`analyze_ere_coverage_headroom.py`](/Users/kalewicker/Documents/hypersim/scripts/analyze_ere_coverage_headroom.py).
Tests: [`test_analyze_ere_coverage_headroom.py`](/Users/kalewicker/Documents/hypersim/scripts/test_analyze_ere_coverage_headroom.py).
The analyzer requires a fresh directory; it never overwrites the completed review.

```sh
python3 -B -m unittest discover -s scripts -p test_analyze_ere_coverage_headroom.py -v
python3 -B scripts/analyze_ere_coverage_headroom.py --output output/ere_coverage_headroom_review_reproduction
```

Completed artifact directory:
[`ere_coverage_headroom_review_20261003`](/Users/kalewicker/Documents/hypersim/output/ere_coverage_headroom_review_20261003).
It contains `analysis.json`, `leaves.json`, `groups.json`, `source_partitions.json`,
the test receipt, four source snapshots, a complete manifest, and an exact
ten-entry seal. Analysis wall time: 266.225 seconds, mostly receipt/hash checks.
The root SHA-256 of `output_sha256.json` is:

```text
2d92a1f528ddbf1086d36242fc75060f255d886dc1ad4cf63ab32552e5947d62
```

Original archives, parameters, criteria, and failed verification receipts remain
unchanged. This review grants no training admission, native-cache portability,
scientific acceptance, or endogenous-emergence claim.
