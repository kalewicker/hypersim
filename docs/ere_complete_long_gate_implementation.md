# Complete-profile long gates: implementation and development evidence

Date: 2026-09-26. Profile `E2-complete-long-horizon-gates-v1`;
semantic protocol `E2-witness-complete-v1`, family 10 / instruction 11.

## Status and authorization boundary

The isolated producer and standalone streaming auditor are implemented. The
development suite exercises 8/16/64-step horizons. The full 540,000-step assay
has **not** been authorized or launched. G-R3/G-R5 scientific receipts remain
pending, and complete-profile training admission remains closed.

`ere_complete_long_gates_v1.md` and its JSON remain byte-for-byte frozen. Their
historical `preregistered_design_harness_not_implemented` status describes the
design task at creation; this report supersedes that implementation status,
not its fixtures, scientific horizons, resources or acceptance criteria.

## Implementation map

| Component | Responsibility |
| --- | --- |
| `crates/hg-sim-core/tests/ere_complete_long_gates.rs` | Opt-in producer, exact 18-cell matrix, native assertions, streaming deltas/CSV, inventories, source freeze, stage barrier and resource refusals |
| `scripts/verify_ere_complete_long_gates.py` | Independent typed graph replay, actual bindings/templates/effects, complete-program validation, boundary/payment/visibility/RNG/CSV/window/lifecycle checks and archive integrity |
| `scripts/test_verify_ere_complete_long_gates.py` | Eight checker regression groups; read-only against an existing development archive, with temporary corrupted test material |
| `Engine::compile_report()` in `crates/hg-engine/src/engine.rs` | Read-only lifecycle accessor; leaves the report available for the runtime's normal consuming accessor |

The engine accessor is the only production behavior surface added. It neither
changes compilation/matching/selection nor advances RNG. `flate2` is an added
test dependency, using the version already present in the dependency lockfile.
Existing bounded harnesses, auditors, contracts and scientific receipts are
unchanged.

All fixtures initialize before the measurement freeze. Ordinary node property 1
is explicitly `Float(0.0)`; `NodeData::new` alone does not initialize it. The
existing wire field `generation` is zero (the constructor's second argument),
and the store is at step zero. Initialization seeds the complete family,
isolated instructions, and G-R5 clock/markers; no host lesion, paid preamble,
policy or host graph mutation is used after initialization.

The full matrix has four 100,000-step static/frozen legs, twelve 10,000-step
repair/ownership cells and two 10,000-step root-loss controls. The producer
requires native G-R3 checks and byte-identical CSV, followed by the frozen
standalone G-R3 auditor, before starting any G-R5 engine trajectory. A saved
`stage_barrier.json` records that boundary. Stage-only audit cannot certify
the full suite; the final auditor checks all 18 cells.

## Bounded evidence and streaming

Per-cell `steps.jsonl.gz`, `inventories.jsonl.gz` and `windows.jsonl.gz` contain
canonical UTF-8 JSONL, streamed through deterministic gzip; the four G-R3 CSVs
remain uncompressed, with the exact preregistered header and formatting. Gzip
is a lossless container choice, not a horizon, semantic or acceptance change.
Output limits account for emitted compressed files and uncompressed CSVs.

Only adjacent bounded inventories and a current record/table dictionary are
held for native deltas. The runtime closes every 128 steps and at the final
partial window; the checker accumulates one window of counters. EI is disabled
and estimates remain null with reasons. The checker caches unchanged family
validation only when independently replayed deltas do not touch any family
material; lesions, Seal, repairs, ownership/root loss trigger revalidation.

Full typed inventories are saved at initialization, every 1,000 steps, the
terminal step, and lesion/deletion offsets -1/0/+1/+2. Deltas include all
physical removals, including dangling cleanup. Native root deletion can list
the same cleanup edge ID twice for an edge with a repeated root endpoint:
the engine's incidence accounting is multiplicity-preserving. The auditor
checks that native multiplicity and StepStats separately from unique physical
edge losses and the runtime's unique meta-cleanup count. No engine cleanup
semantics were changed to simplify the audit.

Before steps, the archive freezes exact relevant source modules, checker and
dependencies, test sources, manifests/fixtures/configurations/initial tables,
Cargo files, executed binary and pinned prior basis receipts. Python runs with
`-B`; exact source inventory excludes bytecode. A final output checksum inventory
seals the completed archive. Fresh output directories are mandatory.

Assay time/output/store/matcher caps fail closed, preserve partial evidence,
and do not trigger retries, fuel refills or parameter relaxation. The proposed
full-run limits remain 3,600 assay seconds, 2 GiB total output, 50,000 pooled
work/results per engine step, and 900 shared stage/final auditor seconds with
a 1 GiB resident-memory ceiling. Linux additionally uses an allocation hard
limit; macOS uses explicit resident checks and bounded streaming rather than
claiming a functioning `RLIMIT_AS`. These are ceilings, not timing predictions.

## Development receipts and verification

The final development archive is
`output/ere_complete_long_gates_smoke_final_20260926`.
Its 18 trajectories use 64 steps each: 1,152 advancing steps, never a scientific
substitute for 540,000. Compressed payload lesions occur at 8/16/24/56,
ownership loss at 40, and root deletion at 60. CORE Seal is genuinely paid by
the ordinary engine at step 1, with no paid preamble.

The final native assay plus stage/final audits took 20.43 seconds (3.39 seconds
in the two semantic auditor invocations; build/freeze excluded). Its 176 source
files match the live checkout, all 181 frozen input hashes verify, and the
245-file sealed output inventory verifies. Total archive size including the
executed binary is 21,350,999 bytes. No bytecode cache was generated.

Observed totals are 14 paid Seal events, 16 paid RepairSealed events, 28 exposed
payload hazards, three exposed ownership hazards and one actual root deletion.
The protected arm supplies 32 bounded counterfactual target-binding probes.
Every archive-level `gate_pass` and `training_admitted` flag is false.

The checks cover protected counterfactual target bindings, finite fuel, all
scheduled exposed hazards, quarantine/reinstallation boundaries, and the
ownership-invalid paid CORE repair at step 57. The root-loss control actually
deletes the family root at 60; absence at 61 retires world execution without
inventing a quarantine decision. A2 preserves the family and blocks all
environmental attacks while admitted paid instructions remain meta-visible.

The frozen auditor rejects 396 inline record corruptions. Additional regression
groups reject missing/duplicate fixtures, altered seeds, resource relaxations,
short traces, changed CSV headers/values, shifted hazards, missing protected
binding probes, visibility overrides, unpaid/wrong-target events, false
ownership recovery, corrupted root cleanup/retirement and checksum failure.
The observer/report/window noninterference tests compare runs with and without
extra capture/probing and extra window closure. Native cap tests cover output,
wall, store-node and matching/nonadvancement refusal.

Earlier development archives remain preserved:

- `ere_complete_long_gates_smoke_20260926`: G-R3 audit failed; no G-R5 started.
  Development found an omitted explicit ordinary-energy property and a checker
  assumption about a nonexistent birth-field name; final initialization now
  matches the preregistered substrate.
- `ere_complete_long_gates_smoke_v2_20260926`: G-R3 passed, but the final checker
  rejected property-array order/representation and cleanup-accounting assumptions.
  Corrected replay preserves native property insertion order and incidence
  multiplicity. This archive remains failed, not repaired or promoted.
- `ere_complete_long_gates_smoke_v3_20260926` and `..._v4_20260926`: successful
  development receipts for earlier checker versions. The final archive freezes
  the completed checker with lifecycle/window/guard-coverage regressions.

## Reproduction and next approval

Normal development tests (the full assay remains ignored):

```sh
cargo test -p hg-sim-core --features hg-meta --test ere_complete_long_gates
python3 -B scripts/test_verify_ere_complete_long_gates.py output/ere_complete_long_gates_smoke_final_20260926
```

Standalone replay of the frozen development evidence, without engine execution:

```sh
python3 -B output/ere_complete_long_gates_smoke_final_20260926/source_snapshots/scripts/verify_ere_complete_long_gates.py output/ere_complete_long_gates_smoke_final_20260926 --stage full --development
```

Omitting `--development` rejects this short archive. The full ignored entry
point additionally requires `ERE_LONG_GATES_APPROVED_STEPS=540000` and a fresh
absolute `ERE_LONG_GATES_OUTPUT` directory; setting an environment variable is
not a substitute for user approval. Use a release build for the long assay.
No full invocation was executed during this implementation task.

Next request approval for the declared 540,000-step resource proposal. G-R1
complete-profile mutation classification, optimizer distribution/effect/noise
prechecks, and explicit new-profile admission remain separate pending work.
The reserved 20 × 16 × 10,000 optimizer-step budget is untouched.
