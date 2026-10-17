# Complete-witness checkpoint/continuation findings

## Result

The frozen `E2-complete-checkpoint-continuation-v1` diagnostic passed: 13
64-step baselines,135 checkpoint forks and two additional fresh-process
resumes,6,428 advancing steps total. Every remaining step and terminal ledger
matched its uninterrupted baseline. The single frozen invocation completed in
216.53s, exit0, with no retries, replacement cells or omitted conditions.
It ran alongside some regression checks; this is not a performance benchmark.

This establishes **same-build execution-core continuation equivalence** for
the declared fixtures, including quarantine, paid recovery, finite fuel,
ownership loss, consumed RNG and interrupted metrics windows. It is not general
runner/output/evaluator restart, optimizer stateability, semantic gate admission
or learned adaptation. No optimizer, precheck or held-out campaign was launched.

## Every condition and interruption

The first twelve cells reuse the labelled substrate, native paid Seal preamble,
CORE exposures8/16/24/32 and optional ownership erasure40 from the preceding
label-separated contract. Seed984927043621 is fixed; these are deterministic
engineering comparisons, not independent scientific replicates.

| Cell | Arm | Repair fuel | Ownership loss | Exact-valid baseline steps | Paid repairs | Final fuel | Checkpoint forks |
| --- | --- | ---: | --- | ---: | ---: | ---: | ---: |
| 00 | A2 | 0 | No | 64 | 0 | 0 | 11 |
| 01 | A2 | 0 | Yes | 64 | 0 | 0 | 11 |
| 02 | A2 | 2 | No | 64 | 0 | 2 | 11 |
| 03 | A2 | 2 | Yes | 64 | 0 | 2 | 11 |
| 04 | A2 | 8 | No | 64 | 0 | 8 | 11 |
| 05 | A2 | 8 | Yes | 64 | 0 | 8 | 11 |
| 06 | A3 | 0 | No | 7 | 0 | 0 | 11 |
| 07 | A3 | 0 | Yes | 7 | 0 | 0 | 11 |
| 08 | A3 | 2 | No | 21 | 2 | 0 | 11 |
| 09 | A3 | 2 | Yes | 21 | 2 | 0 | 11 |
| 10 | A3 | 8 | No | 60 | 4 | 4 | 11 |
| 11 | A3 | 8 | Yes | 35 | 4 | 4 | 11 |

All132 forks in this table start after completed boundaries
0/7/8/9/10/24/25/33/40/41/64 and continue to64. These cover uninitialized
source history at0, pre-lesion state7, damaged state8, paid recovery9,
reinstalled world classes10, exhausted-fuel damage24/25, fourth recovery33,
ownership damage40 and quarantine41. Boundary64 is a zero-suffix roundtrip
with terminal ledger/rule/guard/RNG comparisons, not an extra advancing run.

Cell12 is A2/fuel8/no ownership in **SequentialRandom** mode with16-step
metrics windows, forked at7/8/9. Its RNG actually advances; both per-step
event choices and all closed windows match across the interrupted window.
It is not a second efficacy condition or a randomized version of the A3
damage assay. The ordinary MaxParallel matrix does not consume engine RNG.

All135 checkpoints are written to disk and read back before reconstruction.
Cells11 and12 also resume their step8 files in separate invocations of the
exact archived executable, each checking56 advancing steps and terminal
ledgers. Recorded child process IDs differ from the parent; both exit0.
Accounting:832 baseline steps +5,484 in-process suffix steps +112 cold-process
suffix steps =6,428. Across the twelve ordinary baselines there are12 paid
repairs; repeated suffix payments belong to their independent restored
trajectories, not additional repairs of the original baseline or fuel renewal.

## What is saved and how to extend it

The implementation is additive diagnostic persistence. Production matching,
conflict/application, encoding, hazards, paid operations and admission physics
are unchanged. The guarded diagnostic environment installer is now public for
the test driver; its existing profile/visibility/bounded-rule checks remain.
The driver calls the normal Engine/EreRuntime compile/step/after_step methods;
it does not implement an alternate matcher or repair operation. This is not
a test of restart inside the full runner's output/evaluator loop.

Entry points are `hg_sim_core::ere::checkpoint::encode` and `decode`. Save at
initialized step0 or after `after_step`, optionally after `close_window`.
`encode` refuses a pending before-store/events, any selection policy,
unsupported source or non-complete profile. `decode` returns a fully owned
store, engine and runtime without reseeding/resealing/recompiling/paying fuel.
It creates exactly one new runtime event sink. Diagnostic capture observers
are explicitly reattached with their prior motif phase; arbitrary observers
and policies are not silently reconstructed.

Saved state includes:

- Full graph/store step, slot identities/generations, node data and properties,
  edge endpoint order/labels and allocation state. Future rewritten edge IDs
  and reconstructed incidence/label indexes match the baseline.
- Engine installed compiled rules/order, ChaCha8 RNG, step mode/pairing limit,
  statistics, guard counters, rule-table revision, pending compile report,
  last compile step, match policy/limits/usage and ceiling flag.
- GraphRuleSource anchors, limits, epoch, visibility/profile flags, diagnostic
  suffix, previous region bytes, canonical table bytes and initialized state.
  It is restored from its saved state, not a fresh source derived from the graph.
- Every runtime novelty/baseline/seen/class/role/reuse map; maintenance and
  instruction validity histories; activation, retirement/quarantine/payment
  ledgers; window start/revision/stability, previous-window histograms/member
  sets, complete TPM registries/counts and pending ticks.

The `ERECP001` frame stores length and a native64-bit FxHash accidental-
corruption checksum, followed by size-limited tagged binary state. The maximum
payload is32MiB. Compiled PropertyValue guards/effects/optional offsets use
tagged Int/Float/Bool adapters; raw TOML declarations remain untagged. RNG
serde support is enabled explicitly. Runtime config uses its self-describing
JSON representation because its intentional skip/default attributes cannot be
used directly in bincode; exact `serde_json/float_roundtrip` parsing is enabled.

When extending execution state, keep new fields inside the derived checkpoint
state, add interruption tests that exercise them, and deliberately version the
format if its shape changes. Reattaching a custom observer requires explicit
preservation of its own state. General runner restart would additionally need
output writer positions, evaluator/archive/hooks and any policy or external
observer state. This format does not promise cross-build/platform compatibility,
authenticate files or safely admit untrusted input.

## Independent verification and engineering history

The frozen independent auditor checks all13 cells and135 native frames,
including frame lengths/FxHash, every baseline/suffix timeline, graph indexes,
events/roles, installed tables/revisions, RNG continuity, cap-free completion,
fuel/payments, full ERE windows and terminal guard/maintenance ledgers. It
checks both separate-process suffixes and compares all12 ordinary raw baselines
against the earlier archived label trial's native steps and observations.
Python does not decode the full bincode object; Rust reconstruction and exact
continuation are the evidence for its engine/source/runtime contents.

Fifteen in-memory tamper controls reject shortened baselines/suffixes, missing
forks, wrong cuts, changed native events/RNG/fuel/ledgers, wrong budget policy,
false boundary/ceiling claims and truncated/trailing/wrong-version/corrupt
frames. Separate Rust controls reject nonquiescent, policy-bearing,
unsupported-source and wrong-profile saves and check compiled Int/Float/Bool
roundtrips. Neither the auditor nor execution sources were revised after
frozen measurement began. Scientific receipts are never edited for acceptance.

Two local test-construction errors were corrected before freeze: a wrong
`SelectAllPolicy` export path and a wrong raw JSON guard field name. The first
persisted in-process preflight passed, but both fresh-process comparisons
failed because default JSON float parsing did not retain all exact f64 values.
Their serialized trajectories agree under independent parsing. Enabling exact
float round-trip parsing resolved the comparison; both second-preflight cold
resumes passed. Both preflight directories, including failed child receipts,
remain in `development_preflight`, not substituted for scientific cells.

Selected core/engine/rules/meta/eval/causal regressions, ERE sim regressions,
training admission/persistence tests, workspace and feature-disabled checks,
new-file formatting/Python compilation and diff checks passed. Ignored probes,
historical frozen matrices and independent LK tests were not rerun. All nine
preceding witness/sealed/complete/repeated/label checksum manifests passed
unchanged. See `verification_tests.md` for commands and failure history.

Contract SHA-256:
`3f2c17afb2489e200daff9433e7d1b5da0b005af366ea2a6c8c0d2d2af875b2f`.
Frozen continuation harness SHA-256:
`8ec66777afe3bde4f4fbca32d4aa9ee43ffd15a12ba43e53e523db0853855f61`.
Checkpoint source SHA-256:
`018601be9fe890d17966682b0299457e06cbd0c93714d51732f01df030335999`.
Frozen executable SHA-256:
`8459c1bf2393a9b20acf0a70f66628c2de5e1b184f623c934d57731f7870c65c`.

## Artifacts and next boundary

- [All-condition summary](/Users/kalewicker/Documents/hypersim/output/ere_checkpoint_continuation_20260926/summary.json)
- [Independent audit](/Users/kalewicker/Documents/hypersim/output/ere_checkpoint_continuation_20260926/receipt_verification.json)
- [Fresh-process status](/Users/kalewicker/Documents/hypersim/output/ere_checkpoint_continuation_20260926/cross_process.json)
- [Verification commands and engineering failures](/Users/kalewicker/Documents/hypersim/output/ere_checkpoint_continuation_20260926/verification_tests.md)
- [Checksums](/Users/kalewicker/Documents/hypersim/output/ere_checkpoint_continuation_20260926/checksums.sha256)
- [Contract](/Users/kalewicker/Documents/hypersim/docs/ere_checkpoint_continuation_v1.md)
- [New-profile gate readiness map](/Users/kalewicker/Documents/hypersim/docs/ere_complete_profile_gate_status.md)

Next is a separately preregistered complete-profile deterministic gate harness
(G-R0/G-R4/G-R6 plus full G-R2 characterization), then full identity/protection
horizons and mutation classification. Optimizer stateability is still its own
precheck: restart equivalence does not establish a useful learnable signal.
Legacy admission guards remain closed; the reserved20×16×10,000-step training
budget is untouched. No subsequent gate or training campaign ran here.
