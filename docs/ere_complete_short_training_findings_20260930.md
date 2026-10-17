# Reliability fix and 100-step training findings

Date: 2026-09-30. Completed profile:
`E2-complete-short-training-continuation-v2`.

The reliability fix is implemented and the authorized 20-generation,
16-candidate, 100-step training campaign completed. All 320 training evaluations,
160 perturbation pairs and 768 held-out evaluations are valid. The final trained
parameters improve the held-out point estimate over initialization by
0.05838775724120188, but the preregistered paired sign test gives p=0.109375.
The improvement criterion (positive primary mean and p<.05) is **not met**.
This is a completed training experiment with an unconfirmed improvement.

## Reliability implementation

`crates/hg-sim-core/src/ere/signal.rs` now clears the observer capture buffer
immediately before each native Engine step. A matcher truncation can return
before `on_step_end`; the reset prevents that path from retaining the preceding
step's rewrite event. The existing cap/null fitness semantics are unchanged.

Two delayed-cap regressions cover successful rewrites followed by a cap:
one uses a smaller development matcher budget, the other reaches the late cap
at the original 50k/50k limits and 10,000-step requested horizon. Both return
resource-cap null with no stale terminal event. The full-budget regression runs
in release mode and passed in 4.84 seconds. Its initial debug execution exceeded
the development 30-second deadline; it is explicitly ignored in the debug suite
and separately required in release. This development test does not resume or
replace the failed scientific scaling cell.

An existing regression compares an entire successful 100-step native receipt
against historical control evidence; it still passes. No Engine, matcher,
grammar, coherent kernel, allocator, optimizer, reward or cap implementation was
changed. The old scaling profile refuses the repaired native source identity;
its source-bridge test checks the archived scaling source. The new profile
independently checks the exact horizon bridge plus the one-line capture reset.

The new runner creates all fixed schedule slots before setup, records an
in-flight marker before evaluation, and writes each terminal receipt atomically.
It catches native panic/returned error, records Incomplete with unknown partial
work, and stops with remaining slots NotRun. Known native counters are committed
after the corresponding receipt is saved. A hard process kill can still leave an
in-flight marker; no per-step crash journal or OS preemption is claimed.

## Preserved checker failure and continuation

The first archive is `output/ere_complete_short_training_20260930`, registered
by `docs/ere_complete_short_training_v1.md`. All 120 fresh native prechecks
completed and passed before its frozen admission checker failed. The checker
compared the serialized Rust f32 alpha, widened to 0.05000000074505806, against
Python's decimal .05 using exact equality. The runner correctly stopped with
zero training and held-out native work, `training_admitted:false` and an
incomplete manifest. Original root seal:
`813245dfa5ca432714a27e41a911fac75aef2951848867cc811a178b32d5ea5c`;
all 1,662 entries remain unchanged and verify.

The disclosed accounting review derives the original frozen checker with
exactly one expected-alpha comparison correction. Original checker SHA256:
`e57fdb2807c86cf99fe63a1aa039ae0f776f16b56578caf79f70176ee9bd9fd3`;
derived checker SHA256:
`f55f2d823490bad6150000f1c224c227195569c50783ab2a7fe20aca9748727a`.
It verifies all native receipts, memberships, scores, pair masking, f32
updates, fixed denominators and unchanged P1–P4 thresholds. Its finding is
`original_frozen_checker_pass:false`, `postmeasurement_accounting_pass:true`.

V2 was prospectively registered before any training or held-out physics in
`docs/ere_complete_short_training_continuation_v2.md` and its config. It preserves
the original seed schedule and copies the 120 precheck receipts, five precheck
generation records, initial distribution and summary byte-for-byte. All 230
native/Cargo/optimizer pins match the first archive. It rebuilds the same grammar
once and freezes its new checker and 446 sources/protocol inputs before training.
The frozen checker verifies the copied prefix and its correction lineage before
the optimizer can run. No precheck trajectory is rerun or replaced, and no
training optimizer was restarted. V2 executes precisely the previously NotRun
320 training and 768 held-out slots. The original failed archive is not relabelled
as having passed its frozen checker.

## Actual budget and gate outcomes

| Stage | Scheduled/completed evaluations | Recorded advances | Origin |
| --- | ---: | ---: | --- |
| Stateability prechecks | 120 / 120 | 12,000 | First archive, reused byte-for-byte |
| Training | 320 / 320 | 32,000 | New v2 native work |
| Held-out comparisons | 768 / 768 | 76,800 | New v2 native work |
| Combined | 1,208 / 1,208 | 120,800 | 108,800 new advances in v2 |

Zero null, incomplete or NotRun evaluations remain in the combined schedule.
Unknown partial work is zero. All 160 training perturbation pairs are valid.
Training uses eight antithetic pairs per generation, shared emission/physics
seeds within each pair, zero initialization, sigma 1 / alpha .05 and a fresh optimizer
RNG independent of the precheck optimizer. The designated champion is the final
updated parameter vector, selected before examining held-out results.

Fresh prechecks: P1/P2/P3/P4 all true; runtime envelope true; 20/20 valid pairs;
baseline mean 0.2300775773359742, population SD 0.40049305421890885;
active-minus-dormant gap 1.0; maximum common-seed TV 0.109375. Training's maximum
common-seed distribution TV is 0.1875 and its final TV is 0.15625. Distribution
movement is distinct from held-out reward improvement.

The first invocation elapsed 158.850221875 seconds. V2 elapsed 206.171181625
seconds, including its 127.485720666-second grammar rebuild and final identity
checks. These figures exclude the separate post-run audit/tamper suite. All
actual evaluation durations are below 0.179 seconds. Across the combined native
receipts, peak matcher work/results are 23,142/2,982 under pooled 50,000/50,000;
peak total graph nodes/edges are 293/509 under existing 2,000/4,000 caps.

## Held-out result

Exactly 64 emission draws, four fresh physics seeds per draw, and three arms:
final trained parameters, zero-logit initialization, and a fresh standard-normal
parameter vector per random-resample draw. All 768 evaluations complete.
No held-out result selects a generation or candidate. Primary comparison uses
the 64 paired genotype means, each averaging its four physics seeds.

| Arm | Mean held-out reuse score |
| --- | ---: |
| Final trained parameters | 0.3428545963129335 |
| Untrained initialization | 0.28446683907173165 |
| Random-resample parameters | 0.20789234634565257 |

Primary champion-minus-untrained mean: 0.05838775724120188.
Secondary champion-minus-random mean: 0.13496224996728098.
Primary paired outcomes: five wins, one loss, 58 ties; fixed denominator 64.
One-sided exact paired sign p: 0.109375 (ties excluded from its binomial
denominator). The primary mean is positive but p exceeds .05, so
`held_out_improvement_supported:false`. The sign test concerns paired directions;
it is not a confidence interval for the mean reward difference.

## Verification and evidence boundary

The prospectively frozen accounting checker passes the entire combined schedule,
all copied-prefix identities and its correction lineage, native reward/budget/cap
accounting, all 25 precheck/training updates, champion identity and held-out
statistics. All 18 tampered archive copies are rejected. `audit_pass:true`,
`training_complete:true`, `held_out_improvement_supported:false`.

Final root seal:
`dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132`.
All 1,690 sealed entries verify, with exact file coverage. Final archive size
324,514,388 bytes, below the 512-MiB ceiling. The sealed `audit.json` is the
prospectively frozen checker's result; the original v1 accounting correction
remains explicitly separate in `prefix_accounting_review.json`.

Current development verification: 122 library tests pass; the release-only
delayed-cap test passes separately; 11 ERE and two complete-witness integration
tests pass; five Python regression groups pass. The Python regressions include
reconstruction of prior real optimizer updates, native receipt tampering, fixed
null denominators, and a complete continuation-admission check using temporary
copies of the original prechecks with zero new Engine work. The legacy ERE-only
CLI checks and the release complete-feature CLI builds.

All 446 live source/protocol hashes match the v2 freeze after measurement. The
five historical prerequisites are verified before setup and at completion.
Both failed scaling archives remain unchanged: v1 setup failure 247 sealed files,
v2 interrupted scaling 289 sealed files. The preceding bounded engineering
archive 247 files also verified before launch.

Admission now exists for this completed fixed-world, frozen-metadata, 100-step
complete-family optimization experiment. The legacy guards remain closed for
their unsupported scopes, and the reserved 20x16x10,000 campaign has not run.
The experiment does not establish learned repair, novelty, endogenous online
metadata change, growing-world scaling or sustained maintenance. The checker
reconstructs accounting and numerical updates; it does not independently replay
physics/RNG streams or prove compiler-to-binary correspondence.

## Extension points and next evidence

- `crates/hg-sim-core/src/ere/signal.rs`: native evaluation, cap/null semantics,
  observer reset and horizon API.
- `crates/hg-train/src/ere/short_training.rs`: versioned receipt/source guard,
  fixed schedule, durable cells, prefix continuation, precheck audit boundary,
  optimizer and final-parameter held-out comparison.
- `crates/hg-train/src/bin/ere-train.rs`: feature-gated short-training routes;
  existing routes retain their declared source identity requirements.
- `scripts/audit_ere_complete_short_training.py`: exact numeric serialization,
  disclosed original-checker correction, source lineage, native accounting,
  optimizer reconstruction, held-out statistics and tamper tests.
- `configs/ere_complete_short_training_continuation_v2.json`: exact budget/seeds.

The next useful measurement is a prospectively fixed larger fresh hold-out panel
for this same saved champion. Only six of 64 primary pairs were informative, so
the current test has little evidence about the direction despite its positive
point estimate. Keep the champion fixed and evaluate new draws, rather than
choosing a favorable generation from the existing panel. Any confirmation should
be reported separately with its own fixed sample size and criteria. Do not infer
1,000-step improvement from this 100-step result.

Measurement archive: `output/ere_complete_short_training_continuation_20260930`.
Key artifacts: `manifest.json`, `prefix_accounting_review.json`,
`precheck_audit.json`, `prechecks.json`, `champion.json`, `summary.json`,
`generations/`, `cells/`, `source_snapshots/`, `executed_binary` and root seal.
