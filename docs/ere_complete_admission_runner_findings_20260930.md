# Complete-profile receipt-bound admission and bounded runner

Date: 2026-09-30. Engineering profile `E2-complete-receipt-bound-runner-v1`.
Contract: [prospective limits](ere_complete_admission_runner_v1.md).

## Implementation and extension boundary

The separate feature-gated `hg-train::ere::complete_admission` module binds
five exact historical root seals, complete archive membership/content and 230
current native/Cargo/optimizer source pins. Request validation precedes all
receipt verification and grammar compilation. It checks complete protocol,
valid-production grammar, isolated allocation, opt-in feasibility matcher,
fixed ordinary world, MaxParallel/one pairing, pooled 50k/50k, 183 finite logits,
100 steps, one engineering evaluation, zero optimizer generations/held-out
evaluations, 900 seconds and 512 MiB. Unknown JSON fields are rejected.

`Prepared` has private fields and no deserializer; a serialized pass flag is
not an execution permit. Production callers cannot override seal pins or the
native repository root. `complete_runner` revalidates evidence and current
implementation before setup and again after setup, immediately before native
evaluation. It rebuilds the existing typed grammar rather than importing
caller-supplied executable JSON, and requires its exact registered catalogue
hash. It never imports `hg-engine` or adds a parallel simulator.

Runner order: fresh-output check → request/receipt/source admission → frozen
source/binary snapshot → optional zero-work preflight exit → grammar rebuild
and catalogue identity → admission recheck → unchanged native evaluation at
50k/50k → counts/status/output seal. Failed requests/evidence refuse before
compilation or physics. Existing output and outputs inside prerequisite archives
are refused before writes. No retry, replacement, fallback or cap increase.

Null fitness remains null. If the native evaluator fails after starting but
before returning its receipt, partial bounded step counts are unknown (`null`),
not falsely zero. Earlier failures record measured zero work. Resource checks
occur around setup and evaluation; they are cooperative checks, not an OS-level
preemptive timeout or memory limit. Output bytes are checked, not process RSS.

Main extension points:

- `crates/hg-train/src/ere/complete_admission.rs`: strict request schema,
  root-seal registry, predicate checks, source coverage, process-local readiness.
- `crates/hg-train/src/ere/complete_runner.rs`: preflight/engineering execution,
  freeze, counts, truthful failure states and exact output sealing.
- `crates/hg-train/src/bin/ere-train.rs`: explicit, separate CLI routes.
- `scripts/audit_ere_complete_admission.py`: frozen provenance/accounting audit;
  consumes the already sealed runtime accounting helpers, not physics replay.
- `configs/ere_complete_receipts_v1.json`: paths relative to the bundle file;
  callers select locations but cannot replace the accepted root identities.
- `configs/ere_complete_bounded_request_v1.json`: registered single verification
  job, 183 zero logits, emission seed 7 and physics seed 101.

No native evaluator, matcher, coherent kernel, grammar implementation, optimizer,
Cargo manifest/lock or old experiment archive was edited for this task. Legacy
training/maintenance rejection paths remain in place.

## Verification

- 112 `hg-train` library tests pass, including 13 new admission/runner tests.
- Eleven legacy ERE integration tests and two complete-witness integration
  tests pass; vocabulary-bearing training/maintenance still refuse with zero
  optimizer, precheck and held-out work.
- Five Python auditor regression groups pass, including read-only verification
  of all five exact archives and accounting against an existing native row.
- Release complete-feature CLI builds; legacy `ere`-only CLI checks successfully.
- The broader package test command passed its then-current 110 library tests
  but was interrupted during unrelated, expensive cognition diagnostics. It is
  not reported as a full-package-suite pass. The final 112-test library run and
  the targeted integration runs above completed independently.

Refusal coverage includes unsupported 10,000-step/full-training schedules,
wrong protocol/grammar/world/matcher/allocator/budgets/mode, malformed logits,
unknown fields, unsafe and duplicate checksum paths, symlinks, altered/missing/
extra sealed files, legacy receipts, classification/runtime scope substitution,
changed gate predicates, stale implementation, output preservation, and evidence
changed between readiness and execution. All mutations use temporary copies.

## Zero-work preflight

Archive: `output/ere_complete_admission_preflight_20260930`.
State `ready_for_bounded_execution`; all six work counters are zero.
Frozen audit passes all 12 tamper cases. All 239 frozen sources and the executed
binary are covered; 244 sealed entries, 8,150,376 total bytes.
Root seal SHA256:
`2fd9fc702f1d74d3a4d66e046072b04671dd0040a73ff1a51bf407140b94579c`.
Readiness is informational and cannot be loaded as an execution capability.

## Integrated engineering verification

The single registered job completed in
`output/ere_complete_bounded_engineering_20260930`, with no replacement invocation.

| Measurement | Result |
| --- | --- |
| Terminal state | `complete` |
| Grammar compilations | 1 |
| Native attempts / advances | 100 / 100 |
| Optimizer / precheck / held-out steps | 0 / 0 / 0 |
| Score / credited proposal fires | 0.0 / 0 |
| Emission | Parent 26, operator 2 (`RetargetSlot`), Proposal role |
| Peak native matching work / results | 2,576 / 390 |
| Setup-inclusive elapsed / grammar rebuild | 193.879445125 / 188.691822708 seconds |
| Frozen audit / tamper rejections | Pass / 22 of 22 |
| Frozen sources / required implementation pins | 239 / 230 |
| Sealed entries / final bytes | 247 / 38,112,551 |
| Training admission / full-training authorization | False / False |

The zero score is retained as a valid execution result. This job tests the
admission-to-native-runner connection, not optimization or improvement. No cap,
null, nonadvancing trajectory, retry, fallback or postmeasurement audit correction
occurred. The frozen accounting checker is not independent physics replay.

Root seal SHA256:
`a988f131bebbf6c2e14393bf6c2226737908c628ca7111a1ca3ff1c66b4657fd`.
All 239 frozen source hashes match current files. All five prerequisite archives
verify unchanged (114 / 245 / 364 / 438 / 460 sealed files), as do the previous
valid-panel and raw failed optimizer archives (454 / 441 files). The earlier
failure remains a failure and is not an accepted execution prerequisite.

Development fixes occurred before either new archive was frozen: Rust type
annotations and a JSON read ceiling increased to 256 MiB to accommodate the
38-MiB historical G-R6 report. Neither fix changed native measurement code or
historical evidence.

## What this admits, and what remains closed

Historical semantic receipts are checked for integrity and their declared
predicates, not remeasured on current source. G-R2 remains two supported rules
out of 45; finite paid repair is not learned maintenance. The corrected earlier
400k/50k checker's original failure remains explicitly disclosed.

This installs a real receipt-bound **bounded engineering execution** path,
not a complete-profile training campaign. `training_admitted` and
`full_training_authorized` remain false everywhere. The 20×16×10,000-step
budget is untouched. A 100-step fixture cannot establish 10,000-step or growing-
world scaling, held-out improvement, or learned maintenance.

The next useful boundary is a prospectively registered larger-horizon/resource
scaling contract and fixed schedule, with caps/null denominators and explicit
launch approval. Extending `Request` to accept 10,000 steps without that evidence
would bypass the intended boundary; version the admission profile instead.

## Commands

```sh
cargo test -p hg-train --features ere-complete-signal --lib
cargo test -p hg-train --features ere-complete-signal --test ere --test ere_complete_witness
python3 -B scripts/test_audit_ere_complete_admission.py
cargo check -p hg-train --features ere --bin ere-train
cargo build --release -p hg-train --features ere-complete-signal --bin ere-train
target/release/ere-train complete-preflight configs/ere_complete_receipts_v1.json configs/ere_complete_bounded_request_v1.json FRESH_OUTPUT
target/release/ere-train complete-bounded configs/ere_complete_receipts_v1.json configs/ere_complete_bounded_request_v1.json FRESH_OUTPUT
python3 -B FRESH_OUTPUT/source_snapshots/scripts/audit_ere_complete_admission.py FRESH_OUTPUT --tamper-tests --seal
```

Commands for new output are implementation entry points, not permission to
repeat this job or launch scientific/training campaigns. Frozen audit `--seal`
adds its own report and refreshes only that new archive's root seal. Existing
historical prerequisites are always read-only.
