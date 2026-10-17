# Complete-profile scaling trial findings

Date: 2026-09-30. Corrected profile `E2-complete-scaling-v2`.
Prospective contract: [corrected trial registration](ere_complete_scaling_trial_v2.md).
Measurement archive: `output/ere_complete_scaling_trial_v2_20260930`.
Preserved setup failure: `output/ere_complete_scaling_trial_20260930`.

## Fixed trial

The user authorized one scaling trial, not training. Six fixed productions
(four independent zero-logit valid-grammar draws, plus active/dormant controls)
are each evaluated from the same initial state at 100, 1,000 and 10,000 steps
with two fresh physics seeds: 36 scheduled cells, maximum 133,200 advances.
No production, seed, failed cell or null is replaced based on outcomes.
Each horizon has denominator 12. Actual matcher limits remain pooled 50k/50k;
the existing 2,000-node / 4,000-edge caps, complete-family validation and
nonadvancement checks remain unchanged. There is no optimizer, precheck panel,
held-out work, cap increase, retry or fuel refill.

This is a resource/completion/prefix diagnostic, not a repeat stateability panel
and not a measure of learned improvement. The initial ordinary world is fixed;
it may evolve under native rewrites. A small panel cannot certify all optimized
genotypes or large initial worlds. Scores retain the reuse expression with
log(horizon) normalization, so cross-horizon values are descriptive only.

## Architecture changes and provenance

`hg-sim-core::ere::signal::evaluate_with_horizon` generalizes the existing native
evaluator to a checked 2..10,000 horizon and records horizon plus per-step total
nodes/edges. The original `evaluate` / `evaluate_with_budget` wrappers still use
100 steps and strip only the new accounting fields. A development test compares
the resulting entire legacy native receipt against an archived control, not
merely against another new wrapper. Prefix tests, invalid-horizon refusal and
binding-cap null tests pass. No Engine/matcher/grammar/kernel/allocation or
optimizer implementation is changed by this trial.

`hg-train::ere::scaling` is a separate feature-gated fixed-schedule runner. It
verifies five exact historical seals and the preceding integrated engineering
receipt, then all 230 native/Cargo/optimizer pins. Exactly one source may differ:
`signal.rs`, and it must equal the prospectively specified mechanical horizon/
accounting transformation in full. Rust and Python independently check that
transformation. All other implementation bytes and native file coverage remain
exact. The typed grammar is rebuilt once and its catalogue identity rechecked.
The design, schedule, sources, binary and checker freeze before measurement;
receipts/source identity are rechecked after setup and before every cell.

The old receipt-bound 100-step profile is intentionally still strict. It now
refuses the changed live native identity rather than silently accepting an old
source receipt. Its frozen binary, output and audit remain valid historical
engineering evidence. The scaling profile explicitly carries the source bridge;
it does not relabel old gates as fresh current-source semantic measurements.

Extension points:

- `crates/hg-sim-core/src/ere/signal.rs`: shared native horizon API, legacy shape
  compatibility, graph/matcher ceilings and per-step accounting.
- `crates/hg-train/src/ere/scaling.rs`: exact registered design, source bridge,
  fixed schedule, one compile, cell receipts, prefix/control checks and summary.
- `crates/hg-train/src/ere/complete_admission.rs`: crate-private historical-only
  verification helper; public live admission is not relaxed.
- `configs/ere_complete_scaling_v2.json`: immutable design including six emission
  seeds and two physics seeds; unsupported requests fail before compilation/work.
- `scripts/audit_ere_complete_scaling.py`: frozen integrity/provenance, membership,
  pooled allowances, counters, caps, reuse score, controls, exact prefixes,
  fixed denominators and all-cell metric reporting.

The native API uses cooperative wall checks, not OS-level preemption or an RSS
limit. The runner has an 1,800-second setup/evaluation ceiling, 120 seconds per
cell and 1-GiB output ceiling with audit/seal headroom. NotRun cells remain in
the schedule. Errors without returned native receipts explicitly have unknown
partial work; aggregate counters cover receipted cells only. No unknown work is
reported as measured zero. The auditor is accounting/provenance, not independent
full physics replay or compiler-to-binary proof.

## Pre-launch verification

- 118 `hg-train` library tests pass, including the exact source bridge, real
  receipt verification, registered design/config/schedule equality, zero-work
  refusal, historical native receipt parity, longer-prefix parity and old live
  source-guard refusal.
- Eleven ERE plus two complete-witness integration tests pass; legacy complete-
  vocabulary training/maintenance still refuse with zero training-related work.
- Four native signal tests pass, including binding-cap null and wall refusal.
- Four scaling-auditor regression groups and five preceding-admission auditor
  groups pass. Synthetic graph-size fields in one Python unit fixture are
  explicitly development data, not scientific graph observations.
- Release complete-feature CLI builds; legacy `ere`-only CLI checks successfully.
- All historical prerequisite hashes verify before launch, including the 247
  sealed entries of the preceding bounded engineering receipt.

Development corrections to Rust casts, config seed/width/field serialization
and Python syntax were completed before the first freeze. Unit development
controls are separate from the fixed scaling panel.

### Preserved v1 setup failure and prospective v2 correction

V1 rebuilt the grammar in 170.848229 seconds, then failed at dormant-control
construction with `production outside legal grammar`. It incorrectly requested
Deactivate on inactive parent 4. Parent 4 has one legal Proposal Activate
production and zero Deactivate productions; the matched active initial parent 5
has the legal Deactivate production. The preceding successful runtime panel's
`matched_controls` already uses parents 4/5. This was a registration/constructor
defect, not a measured long-horizon failure.

V1 records zero native attempts, advances and terminal cells; no `emissions.json`
or cell directory was created. All 244 frozen source hashes, binary, catalogue
identity and 247 sealed outputs verify. Its original frozen full-result checker
refuses with missing `emissions.json`, not a pass. The archive, v1 config/contract,
frozen code and checker remain unchanged. Root seal SHA256:
`5ff6a7df7d9b38874ad30955c6c0c598386f53a9bc3f72035a2485b399df01d9`.

V2 is separately registered before any scaling physics, explicitly adds control
parents `[4,5]`, checks their legal catalogue support before expensive setup,
and checks decoded Core/Proposal equality, opposite activation and equal initial
ordinary-world hash before physics. A new regression catches the original `[4,4]`
defect with no grammar rebuild or Engine work. No random draw, physics seed,
horizon, threshold, resource ceiling or fixed denominator changes. There were no
v1 scientific trajectories to replace; the overall scaling measurement budget
remains 133,200 steps. This correction is disclosed, not a retroactive v1 pass.

## Trial status

The single v2 measurement invocation ended with an uncaught native assertion,
exit 101, during `p4-s1361-h10000` (the active control). **Scaling is not passed**.
No scientific retry, cell replacement, resumed physics or budget increase followed.

| Horizon | Scheduled | Complete | Interrupted, partial work unknown | NotRun |
| --- | --- | --- | --- | --- |
| 100 | 12 | 9 | 0 | 3 |
| 1,000 | 12 | 9 | 0 | 3 |
| 10,000 | 12 | 8 | 1 | 3 |
| Total | 36 | 26 | 1 | 9 |

Saved receipts contain 89,900 attempted/advancing native steps. This is a known
subtotal, **not total work**: the interrupted cell's in-memory steps were lost
when the process panicked. Total attempts/advances, its failure step and native
duration remain unknown. Zero optimizer/precheck/held-out work was performed.
The corrected grammar rebuild required 163.219832917 seconds. Whole-run wall
time was not persisted at the crash, so no precise final elapsed is fabricated.

All four drawn productions complete all three horizons with both seeds: 24
cells, all scores 0.0. This establishes bounded execution for those productions,
not positive proposal reuse or learned improvement. The active control completes
100 and 1,000 steps for seed 1361, with scores 1.0 and 100/1,000 credited fires.
Its 10,000-step cell is interrupted; the second active seed and all six dormant
cells remain NotRun. Prior dormant evidence is not substituted into this panel.

In saved cells, peak work/results are 16,536 / 2,382 and peak total nodes/edges
are 85 / 1,139. The active 1,000-step control has 78 nodes throughout; its total
edges rise from 140 at the first recorded step to 1,139 at step 1,000. Matcher
work rises from 2,550 to 16,536. Growth ratios here use the first recorded step,
not an invented pre-step graph measurement. These observed growth/resource
costs are distinct from the interrupted cell's unrecorded maximum.

### Native cause and what the fix would mean

`signal.rs:241` asserts accepted rewrites equal captured event count; the observed
values were 0 versus 1. `Engine::step` (`engine.rs:367–375`) returns immediately
when pooled enumeration is truncated, before `on_step_end` (`engine.rs:441`).
The signal observer's buffer is updated only by `on_step_end` and is not cleared
before a new step. Source analysis therefore identifies the late matcher-ceiling
path retaining the preceding rewrite event. The mismatch panics before the
evaluator can emit its intended `resource_cap` null receipt.

The existing one-step cap regression starts with an empty capture buffer and
cannot detect this delayed-cap case. The next engineering fix is to reset event
capture before each native step and add a delayed-cap-after-rewrite regression,
plus panic-safe fixed-denominator runner handling. Clearing stale events should
yield an honest cap/null, not make the growing active control pass the unchanged
50k/50k ceiling. No claim is made that this fix solves long-horizon resource
growth, and neither the native bug nor the scientific schedule was changed after
the measured failure.

### Crash recovery, sealing and audit classification

The producer's exact last manifest is preserved as `manifest_at_crash.json`.
The public manifest now records `incomplete`, the observed assertion, one unknown
partial-work cell, and unchanged measured counters/source snapshots. The 26
native receipts remain unchanged. Recovery adds one explicit Incomplete record
and nine explicit NotRun records, preserving all 36 scheduled cells and every
denominator. Recovery records are labelled post-process-exit; they are not native
evaluation receipts. `panic.txt` records the observed subprocess error.

A separate post-crash `recovery_auditor.py`, not the prospectively frozen full
checker, verifies all 244 frozen sources, binary, six prerequisites, catalogue,
26 saved cells' membership/native accounting/scores/caps, and all available exact
prefixes: 17 pairs and eight complete triples. It verifies the recovered ledger
and exact output coverage. `recovery_audit_pass:true` means integrity/partial
accounting only; `full_frozen_trial_audit_pass`, `measurement_complete`,
`scaling_panel_pass`, training and full-training authorization are all false.
The full-result tamper campaign was not run on this incomplete measurement and
is not reported as passed. No postmeasurement checker repair is promoted to a
scientific pass.

Final archive: 289 sealed entries, 49,968,014 bytes. Root seal SHA256:
`6609d0f03561a2f06d884e700179fb8e6feb8fad10aaf74134de6dccd35738a7`.
All earlier prerequisite receipts and the preserved v1 setup-failure seal verify
unchanged. Recheck partial recovery read-only with:

```sh
python3 -B output/ere_complete_scaling_trial_v2_20260930/recovery_auditor.py output/ere_complete_scaling_trial_v2_20260930
```

## Scope and next decision

`training_admitted` and `full_training_authorized` remain false regardless of
this panel's outcome. The original reserved 20×16×10,000-step training campaign
is not launched. Historical failed archives and corrected-checker lineage are
preserved. A pass supports only these fixed productions/seeds/world and unchanged
caps. It does not establish learned maintenance, held-out improvement or general
stochastic robustness. A failure identifies the specific bounded resource or
trajectory issue; it does not justify retries, production culling or relaxed caps.
This outcome supplies a concrete late-cap observer bug and a growing-control
resource boundary. Fix and verify those in development, then separately register
and authorize any new measurement; do not launch the reserved training campaign
or count the eight completed 10,000-step draws as complete-panel acceptance.

## Reproduction entry points

```sh
cargo test -p hg-train --features ere-complete-signal --lib --test ere --test ere_complete_witness
cargo test -p hg-sim-core --features hg-meta,repeated-edge-feasibility --lib ere::signal::tests
python3 -B scripts/test_audit_ere_complete_scaling.py
cargo build --release -p hg-train --features ere-complete-signal --bin ere-train
target/release/ere-train scaling-complete configs/ere_complete_receipts_v1.json configs/ere_complete_scaling_v2.json FRESH_OUTPUT
python3 -B FRESH_OUTPUT/source_snapshots/scripts/audit_ere_complete_scaling.py FRESH_OUTPUT --tamper-tests --seal
```

These are implementation entry points, not authorization to repeat the trial or
launch training. Run auditing from its frozen snapshot; do not alter historical
receipts or repair a measured checker into an unqualified pass.
