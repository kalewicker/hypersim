# 100-step training continuation v2

Registered 2026-09-30 before any optimizer or held-out native evaluation.
Profile `E2-complete-short-training-continuation-v2` completes the training work
the user authorized. The v1 invocation stopped after all 120 native prechecks
passed, because its frozen checker compared the widened f32 learning rate
0.05000000074505806 against Python's decimal .05 using exact equality.
Its original frozen-checker result remains failed. Its sealed archive remains
unchanged at `output/ere_complete_short_training_20260930`, SHA256 root
`813245dfa5ca432714a27e41a911fac75aef2951848867cc811a178b32d5ea5c`, 1,662 files.
V1 records 12,000 precheck steps and zero training/held-out steps.

The corrected accounting review derives its checker from the archived original
with exactly one comparison-site change, replacing the expected alpha with the
exact widened f32 value. Verify the original root seal and all native/precheck/
optimizer accounting, every precheck threshold and every NotRun training slot.
Persist the original/corrected checker hashes, exact textual correction and
`original_frozen_checker_pass:false`, `postmeasurement_accounting_pass:true`.
This is a disclosed review after measurement; it cannot make v1's frozen checker
pass. No native precheck is repeated, repaired, resampled or replaced.

The continuation verifies all 230 native/Cargo/optimizer source hashes match v1,
including the repaired evaluator. Copy the 120 cell receipts, five precheck
generation records, initial distribution and precheck summary byte-for-byte into
a fresh output directory, preserving their original seeds and counters. Rebuild
the grammar once and verify its existing catalogue identity; no native precheck
steps occur in v2. The v2 checker and regressions freeze prospectively before
training. They audit the copied prefix and its declared corrected lineage before
entry into the optimizer. The training optimizer still resets to zero logits and
master+10000; it never inherits precheck parameters.

All remaining v1 scientific criteria, objective, 20x16x100 training schedule,
held-out draws/physics/control arms, final-generation champion choice, cap/null
handling, minimum pairs, resources and independent sign statistic are unchanged.
The original 1,208-slot schedule remains visible; v2 executes only the untouched
320 training and 768 held-out slots. New work ceiling 108,800 native steps;
combined with the existing 12,000-step prefix, total scientific ceiling remains
120,800. This is one continuation invocation, with no optimizer restart or
outcome-based retry. Record reused counters separately from new work.

The original v1 contract's no-second-invocation rule is prospectively superseded
only for this accounting-only failure, before any R5 training work. This decision
does not relax failed scientific thresholds. Preserve v1 as an incomplete archive
and both prior failed scaling archives unchanged. The final report must disclose
this failure and correction even if the remaining training succeeds.
