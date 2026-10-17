# Complete-profile 50k/50k signal confirmation

Date: 2026-09-30. Profile `E2-complete-runtime50k-signal-v1`.

## Outcome

The one registered fresh-seed run passes both the actual **50k/50k runtime
envelope** and **all four unchanged P1–P4 prechecks**. All 120/120 rollouts
complete 100 native steps and all 20/20 optimizer pairs are valid. There are
12,000 attempted/advancing steps, no null scores, rejected emissions, caps,
replacement attempts or seed changes. The prospectively frozen checker passes
without a postmeasurement correction and rejects all 23 tamper categories.
`training_admitted` remains false; no full campaign was launched.

Durable evidence:

- [Manifest](/Users/kalewicker/Documents/hypersim/output/ere_runtime50k_signal_panel_20260930/manifest.json): complete status, configured pooled 50,000/50,000 limits, frozen source/binary hashes and prerequisite identities.
- [Native summary](/Users/kalewicker/Documents/hypersim/output/ere_runtime50k_signal_panel_20260930/summary.json): `runtime_envelope_pass:true`, `precheck_pass:true`, `training_admitted:false`.
- [Frozen accounting audit](/Users/kalewicker/Documents/hypersim/output/ere_runtime50k_signal_panel_20260930/audit.json): agrees with native decisions; `reference_preflight:false`, 23 actual tamper rejections.
- [Rollout evidence](/Users/kalewicker/Documents/hypersim/output/ere_runtime50k_signal_panel_20260930/rollouts.jsonl): all scheduled native steps, event IDs, exact pooled per-rule allowances, aggregate usage and scores.
- [Completion seal](/Users/kalewicker/Documents/hypersim/output/ere_runtime50k_signal_panel_20260930/output_sha256.json): 460 verified entries, SHA256 `0099ecae4b7487e9ac975f73d813e7cbde0de430470dc66198bbafd7df16488c`.

## Fixed-denominator measurements

| Check | Unchanged requirement | Observed | Decision |
| --- | --- | --- | --- |
| Runtime envelope | All 120 evaluations finish 100 valid, advancing, uncapped steps at configured 50k/50k | 120/120, 12,000 advances, no caps; all evaluation limits and native pooled allowances verify | Pass |
| P1 | Forced active signature absent from initial 64 emissions | Absent; all 64 valid, 57 distinct signatures | Pass |
| P2 | Maximum common-seed TV ≥ .05 and all 20 ES pairs valid | .09375 = 6/64; 20/20 valid | Pass |
| P3 | Active-minus-dormant gap > twice population SD of all 32 baseline scores and > 1e-12 | Gap 1; SD .4330127018922193; twice SD .8660254037844386 | Pass |
| P4 | Baseline mean strictly between .02 and .98; no nulls | Mean .25; 32/32 valid scores | Pass |

All four active controls score 1, and all four dormant controls score 0. All
32 baseline, eight control and 80 ES evaluations complete the full horizon.

| Baseline | Parent | Operator | Role / production index | Scores at physics seeds 941 / 1051 / 1153 / 1259 |
| --- | --- | --- | --- | --- |
| 0 | 128 | Activate | Proposal / 0 | 0 / 0 / 0 / 0 |
| 1 | 99 | AddSlot | Proposal / 2 | 0 / 0 / 0 / 0 |
| 2 | 150 | AddPatternEdge | Proposal / 7 | 0 / 0 / 0 / 0 |
| 3 | 92 | AddLabel | Proposal / 2 | 0 / 0 / 0 / 0 |
| 4 | 109 | ChangePriority | Core / 0 | 0 / 0 / 0 / 0 |
| 5 | 143 | RemoveSlot | Proposal / 3 | 1 / 1 / 1 / 1 |
| 6 | 3 | RemoveLabel | Proposal / 0 | 0 / 0 / 0 / 0 |
| 7 | 34 | Activate | Proposal / 0 | 1 / 1 / 1 / 1 |

Within-genotype physics SD is zero; between-genotype SD equals the overall
.4330127018922193. Each baseline also has identical final ordinary-world hash
across its four physics seeds. This is a narrow deterministic MaxParallel
fixture, not general stochastic robustness or ecological diversity. Six of
eight registered baseline genotypes have zero credited reuse.

| Generation (zero-based) | Valid pairs | TV from initial panel | Cumulative theta L2 norm |
| --- | --- | --- | --- |
| 0 | 4/4 | .046875 | .142066721 |
| 1 | 4/4 | .03125 | .179026014 |
| 2 | 4/4 | .0625 | .219173786 |
| 3 | 4/4 | .0625 | .244489837 |
| 4 | 4/4 | .09375 | .301510736 |

Across the 80 ES scores, 58 are zero, 16 are one, four are
.8406206186877936 and two are .8360489289678587. TV need not increase
monotonically; P2 uses the registered maximum over actual updates. The panel
demonstrates score-sensitive optimizer distribution movement without invalid
pairs, not a learned champion or improved held-out fitness.

The preceding grammar panel passed at configured 400k/50k, with 120 valid
rollouts, 20 valid pairs and TV .109375. This new panel retains the same grammar
and criteria while genuinely enforcing 50k/50k, and also passes. Because seeds
are fresh, differences in baseline mean, score composition or TV are not a
controlled causal estimate of the budget change. Engineering parity on a
common control is a separate premeasurement test, not another scientific panel.

## Resources, audit and preserved evidence

Elapsed native campaign time is 180.919 seconds against the registered 900,
including 171.908 seconds for rebuilding the exact grammar catalogue. The
remaining emission/evaluation/optimizer work is approximately 9.011 seconds.
Peak per-step usage is 18,388 work / 1,194 results, against configured
50,000/50,000. There is no budget increase, fuel refill or resource-triggered
retry. Final sealed output is 45,932,544 bytes against 512 MiB.

The frozen checker verifies reward credit, scores, valid-pair accounting,
all rank updates, TV, baseline/control statistics, decisions, 504 memberships,
configured limits and actual per-rule remaining-work/result allowances. Its
23 rejected temporary-copy tamper types are score, fires, seed, duplicate ID,
step, credit, cap, pair, theta, epsilon, distribution, summary, world, membership,
mask, fallback, retry, catalogue, control, configured budget, evaluation budget,
native evaluator body and native per-rule allowance. None edits production
measurements. This remains accounting verification, not independent physics
replay or a full semantic gate remeasurement.

All 436 frozen source/contract/checker/test hashes verify and match their live
counterparts; the executed binary hash verifies. No postmeasurement source
correction was necessary. All 460 sealed entries verify with exact file-set
coverage, including nested prerequisite seals. Only the root seal excludes
itself. The preceding valid panel's 454, raw optimizer's 441, grammar's 438
and accepted closure's 364 entries remain unchanged and verify. The old failed
raw panel is still failed, and the preceding panel's original checker exception
is still preserved and disclosed.

Frozen checker SHA256:
`e14069ca24a5cd877b481041ed705cb8b016a483ca100d9c80f094c07b589b01`.
Executed binary SHA256:
`1c9502be2e609d29788e0e14b0e5d2cf7d03baacc281bc706e4b525795127af9`.

## What this clears, and what remains

The bounded optimizer/runtime precheck hurdle is cleared for this declared
grammar, kernel, isolated allocator, opt-in matcher, world and 100-step horizon.
Another identical panel is not needed to establish this specific result.

The next engineering task is a receipt-bound complete-profile admission/runner
path: bind the accepted semantic gates and this exact runtime/precheck identity,
retain the explicit historical checker-correction lineage, enforce 50k/50k and
test stale, missing, tampered, mismatched or failed evidence refuses execution
with zero optimizer/precheck/held-out steps. Legacy guards remain unchanged and
complete-profile training remains closed. A 100-step fixture is not proof that
a 10,000-step candidate horizon or growing ecological world fits these limits;
any larger training design must preserve caps, null-score denominators and
separate authorization. No learned maintenance, paid repair, novelty, general
seed replication, full-horizon scaling or training-success claim follows.

## Prospective design and implementation

The user authorized one 50k/50k run after the valid-production 400k/50k panel.
The [prospective contract](/Users/kalewicker/Documents/hypersim/docs/ere_runtime50k_signal_panel_v1.md)
has SHA256 `237ee67d8b95500a08d85a6e10819b3690eccb75fbfc4edc361d38ce66aa6392`.
This is a 120-attempt, 100-step diagnostic with fixed denominators, not full
training or a new G-R1 classification assay. Runtime-envelope completion and
P1–P4 stateability are distinct reported outcomes; training remains gated.

The new [runtime harness](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/runtime_signal.rs)
calls the existing native `signal::evaluate_with_budget` with pooled work and
results both exactly 50,000. Every evaluation records that configuration.
The only edit to the shared native evaluator source is adding `pub` to that
function's declaration; its entire body and the old default `evaluate`
entry point remain byte-for-byte unchanged. No Engine, matcher, coherent
kernel, grammar implementation, objective or optimizer code was changed.

Grammar source verification checks an exact whole-file visibility-only
transformation for this one file, not an unbounded source-pin exception.
Historical/current evaluator hashes are
`b26dd5d08224d7015d6267d3d43545f62c332a9427bead6ec1f26e352326485b` /
`92ba40ff39a3c4bc0eb8e563b09f7bbd24a2dca621e0ccfda3c9409bf72ad8cb`.
Other native/kernel/grammar/optimizer pins remain strict. Existing sealed
archives are untouched; older strict hash-pinned harnesses may correctly
refuse the newly public API source and are not silently grandfathered.

The harness verifies all accepted closure, grammar and previous diagnostic seal
entries before creating output, freezes 436 sources/contract/checker/test files
and the executed binary, and recompiles the grammar once with exact catalogue
hash equality. Controls verify matched final alleles and ordinary material
before physics. The previous checker exception is preserved and accepted only
as explicitly corrected diagnostic lineage. The corrected checker is adapted
and frozen prospectively for this run, not patched after its outcomes.

Fresh master seed is `0xE6523A0001`; physics seeds are 941, 1051, 1153, 1259.
The schedule is 32 baseline, eight active/dormant control and 80 ES attempts,
with the same five generations/four paired perturbations, 183 initially zero
logits, sigma 1, alpha .05 and tied-rank update rules. Each antithetic pair uses
common emission and physics seeds. No seed preview, retry, replacement, parent
fallback, repair, criterion relaxation or outcome-dependent extension is allowed.

The new [frozen accounting checker](/Users/kalewicker/Documents/hypersim/scripts/audit_ere_runtime50k_signal.py)
reconstructs reward credit, all scores, pair withholding, float32 rank updates,
TV and P1–P4. It verifies 504 grammar memberships, explicit evaluation limits,
and each native per-rule pooled work/result allowance and aggregate usage.
Thus a 400k configuration with low observed work cannot pass as 50k evidence.
The checker's read-only reference-preflight path processes the real prior
400k/50k archive under its own identity, returns no runtime pass, and is
unreachable from the production command-line audit/sealing path. This audit
is accounting/membership/provenance verification, not independent physics or
semantic decoding replay.

## Premeasurement verification

- 99 training unit tests pass, including seven new runtime-harness tests.
- 11 existing ERE integration tests and two complete-profile compatibility /
  zero-budget training-rejection tests pass.
- Four shared native evaluator tests pass, including binding-cap null behavior,
  expiry, genuine activation reuse and deterministic world construction.
- Ten runtime checker regressions pass, including the real prior 120-row /
  384-distribution-emission audit loop, catalogue-variable lifetime, strict
  runtime refusal of the prior 400k receipt, exact pooled allowances, grammar
  masks, tied ranks, fixed TV denominator and disjoint registered seeds.
- Release build and `git diff --check` pass; all four historical seal file sets
  verify before measurement (454 previous valid panel, 441 raw optimizer,
  438 grammar, 364 accepted closure entries).

The first engineering parity assertion incorrectly expected budget-allocation
metadata to be identical at 50k and 400k. It failed before scientific measurement.
The corrected regression explicitly verifies each budget's exact remaining-work
allowance, removes only that declared allocation field for comparison, and
requires all actual work/results, events, world hashes, scores and other
receipt fields to match. The corrected test was frozen before the single run;
no scientific failure was retried or relabeled by this engineering correction.

## Entry points

```sh
cargo build --release -p hg-train --features ere-complete-signal --bin ere-train
target/release/ere-train signal-runtime50k-complete FRESH_OUTPUT_DIR output/ere_complete_coherent_closure_full_20260929 output/ere_valid_production_grammar_20260929 output/ere_valid_optimizer_signal_panel_20260929
python3 FRESH_OUTPUT_DIR/source_snapshots/scripts/audit_ere_runtime50k_signal.py FRESH_OUTPUT_DIR --tamper-tests --seal
```

The scientific output directory must be fresh. These commands document the
registered workflow, not authorization to run another panel or full training.
Campaign resource/provenance failures stop incomplete; per-trajectory caps
retain null fitness in the original denominator. Runtime and training scopes
must remain separate even if all diagnostic prechecks pass.
