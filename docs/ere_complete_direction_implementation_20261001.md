# Update-direction reliability: implementation and development verification

Date: 2026-10-01. Profile `E2-complete-direction-reliability-v1`.

The fixed-parameter assay, complete-feature CLI, durable estimate records and
frozen streaming auditor are implemented. One bounded development invocation
completed all **208 valid evaluations / 20,800 native attempts and advances**,
four independent emission-panel estimates and the five-arm validation controls.
The prospectively frozen auditor passes all **40 detached tamper controls**.
There was no postmeasurement checker correction or development rerun.

This report preserves the development milestone. The user subsequently approved
one full scientific invocation, which completed all **7,168 evaluations** and
passed its frozen 40-control audit. The registered direction criterion fails:
alignment mean .19186 with p=.14453125; utility effect −.00883176 with p=1.0.
See the separate [full-assay findings](ere_complete_direction_assay_findings_20261001.md).
The development fixture remains engineering evidence only. No automatic follow-on
campaign or reserved 10,000-step training was launched.

## Why this experiment, and what saved records cannot establish

The completed [minibatch allocation trial](ere_complete_minibatch_trial_findings_20261001.md)
did not support batching: mean utility difference −.00715475, four wins/four
losses, paired sign p=.63671875. Its frozen evidence remains untouched.
Read-only inspection reconfirms 499/1,280 single and 252/320 batch antithetic
pairs had unequal ordinal scores; four single generations were entirely tied,
and six single updates were exactly zero. Batch had no entirely tied generation
or zero update, but only one quarter as many updates. These are diagnostics,
not evidence that the more frequent per-update signals pointed usefully.

Those generation records do not contain two separately resampled estimates
at the same parameters and shared perturbation basis. Comparing consecutive
updates would confound parameter movement and perturbations with estimator
noise. The new assay supplies that missing controlled comparison.

See the [prospective contract](ere_complete_direction_v1.md) and
[machine-readable layouts](../configs/ere_complete_direction_v1.json).

## Fixed comparison and methodological improvements

Two outcome-independent contexts: 183 zero logits, and alternating +.25/−.25
logits by zero-based index. No trained champion, best seed or previous held-out
result selects the context panel. Eight independent blocks evaluate both.
At every block/context, A and B share eight standard-normal perturbations but
use disjoint emission/physics panels. Each estimate starts from the fixed
context, uses sixteen candidates × four emissions and one unchanged sigma-1 /
alpha-f32-.05 rank-ES update. Native cap fitness remains null; measured cap
utility is zero and participates in the complete fixed-batch mean.

This isolates **conditional emission-panel reproducibility given the sampled
perturbation basis**, not reliability across independently sampled bases or
an unbiased mathematical gradient. All 32 estimates finish and their vectors
are fixed before any scientific validation.

Validation uses base, forward A, reverse A, forward B and reverse B, all on
the same fresh 16 emission draws/four physics seeds within each block/context.
Reverse is the exact saved-vector f32 operation 2*base−forward; no step
normalization, clipping, alpha tuning or validation winner selection. Zero
updates keep their entire scheduled validation allocation. Reverse outcomes
are descriptive controls, not an extra confirmatory search.

Primary statistics have **eight estimator blocks**, not sixteen contexts or
thousands of cells. Each block's alignment averages the two context cosines;
an exactly zero direction receives alignment zero with an explicit degeneracy
flag, rather than being discarded. Each block's usefulness averages over both
contexts and both forward estimates relative to base. Both alignment and
usefulness must have positive means and exact one-sided sign p<.05, both context
mean usefulness effects must be positive, and forward feasibility must be no
worse by the registered aggregate count guard. The required conjunction is
not an opportunity to declare success from either single passing property.
Native full-panel means are withheld if any validation arm has a native cap;
utility does not fill in native scores. Missing work retains the fixed denominator
and null official means/p-values, not survivor estimates.

| Scientific stage | Evaluations | Maximum native attempts |
| --- | ---: | ---: |
| 8 blocks × 2 contexts × 2 estimates × 16 candidates × 4 emissions | 2,048 | 204,800 |
| 8 blocks × 2 contexts × 16 draws × 5 arms × 4 physics seeds | 5,120 | 512,000 |
| Total | 7,168 | 716,800 |

The native world, grammar, coherent kernel, matcher, MaxParallel/one rewrite,
100-step horizon, 50k/50k pooled budgets, node/edge caps and native reward are
unchanged. These are one-step hypothetical estimates, not sequential training:
`optimizer_generations:0`. `training_admitted:true` in the inherited runner
indicates the receipt-bound bounded native envelope, not long training permission.

## Code and extension map

- `crates/hg-train/src/ere/direction.rs`: fixed layouts/seed schedule, parameter
  contexts, direction/reversal arithmetic, block statistics, parent/latest
  checks, durable starts and estimates, validation and failure receipts.
- `crates/hg-train/src/ere/minibatch.rs`: reused fixed-batch utility/ordinal
  adapter; its implementation is unchanged by this task.
- `crates/hg-train/src/ere/feasibility_training.rs`: reused native `Runner`,
  in-flight marking, atomic terminal rows, cap handling and committed counters;
  no body or visibility change in this task.
- `crates/hg-train/src/ere/confirmation.rs`: reused snapshot, seal, live recheck
  and exact original-parent helpers; unchanged.
- `scripts/audit_ere_complete_direction.py`: independent design/schedule,
  batch/candidate/update reconstruction, shared-basis checks, exact reversals,
  alignment, block statistics, provenance and native accounting.
- `scripts/test_audit_ere_complete_direction.py`: seven regression groups,
  including actual historical native cap and optimizer-noise fixtures.
- `crates/hg-train/src/ere.rs` and `src/bin/ere-train.rs`: complete-feature-only
  module and `direction-development-complete` / `direction-assay-complete`
  routes. Legacy feature isolation and refusal tests remain intact.

Estimate records save context, noise, pre-update parameters and exact cell IDs
before native physics. After all candidate batches finish, the same atomic
record saves means, ordinal pairs and post-update parameters. Partial starts
cannot become completed estimates; subsequent estimate starts cannot skip an
unfinished/missing estimate. All five validation vectors and alignment records
are saved separately before validation. No overlapping candidate batch is
counted as an independent training replicate.

Campaign timing starts before output/schedule creation and includes setup,
grammar compilation and final checks. The shared native evaluation deadline
is cooperative, not OS-preemptive. Hard-killed in-flight work can remain
unknown; this does not promise a per-step crash journal.

## Development fixture, not scientific inference

Archive: [`output/ere_complete_direction_development_20261001`](../output/ere_complete_direction_development_20261001).
Master `0xE658D00001` = 989332504577; scientific master `0xE658000001`
= 989318873089 remains unused. One block, both contexts, two emissions per
candidate, two validation draws/four physics seeds. All allocations and analysis
were fixed before measurement. The separate layout always withholds science.

| Development stage | Recorded / scheduled | Valid | Cap nulls | Attempts / advances |
| --- | ---: | ---: | ---: | ---: |
| Estimation | 128 / 128 | 128 | 0 | 12,800 / 12,800 |
| Validation | 80 / 80 | 80 | 0 | 8,000 / 8,000 |
| Total | 208 / 208 | 208 | 0 | 20,800 / 20,800 |

Four started estimates completed; all are nonzero. Context alignment scores
are −0.19537636409799825 and −0.06223058803239804, mean −0.12880347606519815.
Context utility effects are 0 and −.25, block mean −.125. Both one-block sign
p-values are 1.0. Validation utility arm means, in registered order, are
[1.5, 1.25, 1.5, 1.5, 1.5]; native means [.5, .25, .5, .5, .5]. All five arms
are valid 16/16, and forward-minus-reverse descriptive utility is −.125.
One of eight base-versus-forward emission-signature comparisons changes in
the tiny validation fixture; this is post-hoc visibility context, not a scientific
criterion or independent physics count. It demonstrates that the harness can
record a genuine validation change rather than only identical controls.

These tiny estimates do not justify changing the scientific seeds, panels,
sigma/alpha, thresholds or schedule. `classification:bounded_development`,
`scientific_campaign_complete:false`, `support_direction:false` remain
authoritative even if a future development fixture were positive.

Campaign elapsed: 216.724668833 seconds including one 161.727664708-second
grammar compilation and final checks; separate audit time excluded. Maximum
native evaluation: .110817583 seconds. Peak work/results: 18,398 / 2,388;
peak nodes/edges: 290 / 512. Final sealed size: **47,429,672 bytes**, within
the development 900-second / 2-GiB ceilings. No crashes, unsupported outcomes,
Incomplete/NotRun cells or unknown partial work.

## Verification and immutable evidence

135 hg-train library tests pass; one pre-existing release-only delayed-cap
regression is ignored in debug (native implementation unchanged). Eleven ERE
and two complete-witness integration tests pass. Seven new Python audit groups,
seven minibatch groups and seven feasibility groups pass. Targeted Rust tests
pass again after the final timer change. Complete-feature release CLI builds;
legacy ERE-only CLI checks; `git diff --check` passes. A wrong-parent fixture
refuses with zero grammar compilation, attempts, estimates or optimizer updates
and passes its zero-work audit.

The frozen end-to-end audit reconstructs all four estimates, their shared
perturbation bases, reversals and block statistics, then rejects 40 detached
tamper controls. These cover design/types, schedules, native membership/credit/
caps/allowances, candidate repetition, means/ranks/updates, partial batches,
adaptive contexts, changed shared noise, reversals, alignment, validation-based
selection, statistical unit/denominator, p-values and source/seal integrity.
The final sealed archive is re-audited and exact coverage/hashes independently
verified. No source/checker/config/contract changed after measurement began.

Root seal, **684 exact sealed entries**:

```text
ee9f28214bf67aebba4410b60f269ae6d3935acd5929e372f55527d51a035361
```

Snapshot SHA256:
`f616072632b9cd549e8c8dca9d36b89fcd4fb2369b5250ddd05c183b89bc4bf3`.
Executed binary SHA256:
`9657c3513f8c08d8a683e3790141157632cf61bc062837f65095ef98581538fb`.
All **459 live frozen inputs** and the live binary match the development freeze.
Original parent/1,690 entries and prior minibatch/10,152 entries reverify unchanged
after completion. Earlier feasibility/6,944, confirmation/6,598, failed short
training/1,662 and failed scaling/247 and 289 entries were also reverified
unchanged during this task. No historical archive was edited or resealed.

Audit arithmetic uses the pinned parameter tolerance <2e-7, 1e-12 statistic
tolerance and stricter p-value absolute tolerance. Direction norms/cosines
reconstruct the underlying f32 values before f64 subtraction; reversal checks
require exact f32 vectors. The checker does not independently regenerate
perturbation or grammar RNG choices from seeds, replay native physics/RNG or
prove compiler-to-binary correspondence. `independent_physics_replay:false`.
Accounting success does not establish useful gradients, learned repair,
maintenance, online metadata evolution or longer-horizon readiness.

## Historical launch boundary and scientific completion

Completed development invocation (do not reuse its output):

```sh
target/release/ere-train direction-development-complete \
  output/ere_complete_short_training_continuation_20260930 \
  output/ere_complete_direction_development_20261001
```

Read-only frozen re-audit:

```sh
python3 -B output/ere_complete_direction_development_20261001/source_snapshots/scripts/audit_ere_complete_direction.py \
  output/ere_complete_direction_development_20261001 --tamper-tests
```

At the development milestone, separate approval was required for one eight-block
assay under the registered 7,168-evaluation / 716,800-attempt maximum,
3,600-second / 2-GiB limits. The user explicitly authorized launch in
[the launch record](ere_complete_direction_assay_authorization_20261001.md).
`direction-assay-complete` ran once into the then-fresh
`output/ere_complete_direction_assay_20261001` with the separate scientific
layout and unchanged frozen audit. All 32 estimates and eight validation blocks
completed; five estimation caps remain recorded. The 7,686-entry seal verifies.
See the [full-assay findings](ere_complete_direction_assay_findings_20261001.md)
for the failed registered criteria, sparse-response diagnostic and next mapping
audit proposal. No retry, retuning, further assay or reserved longer-horizon
training was authorized or launched.
