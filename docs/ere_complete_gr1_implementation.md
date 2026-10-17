# Complete-profile G-R1: harness, frozen auditor and development verification

Date: 2026-09-27. Assay profile `E2-complete-mutation-classification-v1`;
encoding `E2-witness-complete-v1`, family 10. The isolated implementation and
bounded development checks are complete. The subsequently authorized full
scientific trial also completed and passed its frozen audit, but failed the
mandatory structural numerical lines. **G-R1 has no passing acceptance receipt,
and training admission remains closed.** See
`ere_complete_gr1_full_trial_findings.md` and its separate authorization.

The immutable prospective contract remains `ere_complete_gr1_v1.md` and
`ere_complete_gr1_v1.json`. Their original wording describes their design-time
status; this document records the subsequent implementation without rewriting
the preregistration. No production mutator, runtime budget, admission guard,
historical assay, Cargo dependency or prior receipt was changed by this work.
The already-dirty workspace contains earlier project changes; this is not a
claim that those files match the repository's committed base.

## Implementation routing and extension boundaries

| Component | Responsibility |
| --- | --- |
| `crates/hg-sim-core/tests/ere_complete_gr1.rs` | Isolated `hg-meta` integration producer; parent preflight, unrepaired mutation kernel, damage matrix, native trajectories, resource accounting, stage barriers and frozen auditor invocation |
| `scripts/verify_ere_complete_gr1.py` | Standalone streaming archive auditor; independently reconstructs typed complete material, edits, program bytes, native world transitions, classifications and exact threshold accounting |
| `scripts/test_verify_ere_complete_gr1.py` | Read-only checker regressions, with optional existing development archive |
| `crates/hg-sim-core/tests/support/witness.rs` | Existing typed inventory helper, reused without modification |
| `scripts/verify_ere_complete_gates.py`, `verify_ere_complete_witness.py`, `verify_ere_complete_long_gates.py` | Existing canonical encoding, inventory and binding helpers, frozen and reused without modification |
| `docs/ere_complete_gr1_v1.md`, `.json` | Immutable counts, protocol, operators, controls, acceptance lines, resource ceilings and pinned prior scientific evidence |

The producer reads the public `semantic::parent_bank()`. Its assay-local world
builder reproduces the pinned historical fixture: the parent's Core LHS motif,
64 label-0 nodes carrying property 1 = Int(100), and 96 ring edges. World material
is allocated before the complete family so paired ordinary handles agree.
The Python checker independently derives both the expected bank descriptions
and world inventories. `family_parts` and the production evolutionary mutator
are not called on version-10 material.

The main producer extension points are `parents`, `raw_edit`, `attack`, `trace`,
`Tally` and `campaign`. Their independent checker counterparts are
`check_parent`, `raw_edit`, `control_edit`/`check_control`,
`replay_step`/`consume_trace`, `check_classification` and `tally`.
Changes to scientific operators, parent coverage, thresholds, horizons or caps
need a newly named prospective contract, not an edit to the frozen version-1
design or a rerun of a failed scientific receipt. Preserve the complete-source
path: a static-only rule reconstruction is not a substitute for this assay.

## Freeze, controls, traces and sealing

The campaign requires a fresh, explicitly named output directory and refuses
overwrite. Before measurement it copies the selected source/dependency files,
lockfile, exact executing test binary, immutable contract and pinned basis
receipts. It freezes all 168 parent inventories, the ordered draw/edit manifest
and all 5,376 attack descriptions in compressed canonical JSONL. The native
producer regenerates each manifest row and requires exact equality before use.

Every parent is complete-encoded and complete-decoded without increasing the
declared MetaLimits. The bank retains all 168 genotype states; its executable
RCF and presentation identities each have 112 unique values. These deliberate
duplicates are not dropped or represented as 168 independent executable rules.

All 32 damage attacks run for every parent with a matched clean source. Source
compile/read probes do not advance the Engine. The controls distinguish present
invalid families, quarantine and zero child classes from deleted/relabelled roots,
which must retire rather than receive an invented quarantine count. The matrix
covers all 13 relation roles, all 26 required-obligation keys and explicit-zero
obligations. Only after both native controls and the frozen stage audit pass
does `stage_barrier.json` permit the first mutation trajectory.

The 13-operator raw mutation adapter selects typed allele payload links, not
ownership traversal. It records sorted eligible handles, selector results,
primitive edits, native deletion cascades, raw deltas, applicability, logical
topology and metadata changes. Empty-target choices still consume their declared
RNG draw. It never regenerates ownership, incidence obligations or witnesses,
never codec-repairs a mutant and never replaces a quarantined draw.

Trajectories use the actual Engine and complete-enabled GraphRuleSource, epoch
Never after initial compilation, meta visibility false, MaxParallel, max pairings
1 and engine seed 42. There are no editors, instructions, fuel, EI, evaluator or
repair preamble. Matching is pooled 400,000 work / 50,000 results for this assay
only; production runtime remains 50,000/50,000. All 168 parent traces are cached;
child traces run for valid non-neutral draws and the first 50 eligible neutrals.

Start/step/end rows preserve typed inventories/deltas, initial and per-step full
and projected-world hashes, RNG witnesses, bindings, rewrite events, counters,
guard statistics, matching costs and cap flags. The checker reconstructs native
template application and cleanup, holds meta material fixed, and compares paired
typed world fingerprints as well as hash/rewrite-count vectors for both traced
Neutral soundness checks and NearNeutral comparisons. Unguarded rules
correctly omit guard-stat records; later guarded rules may remain unvisited when
a matcher cap stops enumeration.

Classification precedence is quarantine, unmeasured, Neutral, NearNeutral,
Catastrophic, Divergent. Failed baselines cannot make empty traces neutral.
The summary separately records `parent_baselines_complete`; any incomplete
required baseline makes overall measurement incomplete and prevents gate passage,
even when its parent is unsampled or all of its sampled mutants are quarantined.
Caps precede death/explosion predicates. Explosion does not permit early stopping;
all traces need the declared horizon unless a cap makes them Unmeasured. The
contiguous 50-zero window and strictly greater-than-fourfold size boundary are
checked separately, with overlapping catastrophe predicates counted once.
The primary denominator never includes the separate damage matrix. Exact integer
40% Valid / 10% NearNeutral / 25% catastrophe comparisons and all declared strata
are reconstructed independently; development receipts always have gate flags false.

Resource checks bound attempted/advancing steps, store size, matching, elapsed
time, output bytes, shared audit time and measured auditor RSS. Stream output is
charged as it is flushed, with conservative pre-emission room for failure/footer
records. The final auditor rechecks controls, replays traces, reconciles every
draw and denominator, and executes 18 distinct tamper-rejection categories.
Successful outputs are sealed by an exact path-and-SHA256 inventory. Only the
top-level `output_sha256.json` excludes itself; a nested basis checksum remains
covered evidence. Failures retain partial outputs and a non-admissible status.

## Archive layout

| Evidence | Files |
| --- | --- |
| Frozen provenance | `source_snapshots/`, `basis/`, `executed_test_binary`, `manifest.json`, `frozen_inputs_sha256.json` |
| Prospective material | `parents.jsonl.gz`, `draw_manifest.jsonl.gz`, `control_manifest.jsonl.gz` |
| Prerequisite control evidence | `controls.jsonl.gz`, `audit_controls.json`, `stage_barrier.json` |
| Native world evidence | `parent_traces.jsonl.gz`, `child_traces.jsonl.gz` |
| Classification and resources | `classifications.jsonl.gz`, `classification_summary.json`, `progress.json`, `audit_full.json`, `run_status.json` |
| Exact completed output inventory | `output_sha256.json` |

`run_status.json` distinguishes execution completion, measurement completeness,
numerical acceptance, development-only scope and scientific launch. No flag
in this harness admits training. Status elapsed/auditor/byte counters are recorded
before the final seal stage; the producer additionally checks the ceilings after
sealing. Final on-disk byte counts include that seal and status record.

## Verification and preserved development history

The clean development receipt is
`output/ere_complete_gr1_smoke_complete_20260927`. It uses all 168 parents and all
5,376 completeness controls, but only 64 mutation draws and eight-step horizons.
Its envelope is 300 assay seconds, 512 MiB total output, 120 shared audit seconds,
at most 1,856 attempted/advancing Engine steps, and 1 GiB auditor RSS. Build and
freeze are outside the assay timer, as declared in the prospective contract.

The final native and frozen stage/full audits pass, with 18 actual distinct
tamper rejections. Its 168 parent baselines and 28 required child traces complete
all eight steps: 1,568 attempted and advancing steps, with zero Unmeasured draws.
The 64-draw diagnostic cohort contains 36 Quarantined, 18 Neutral, seven
NearNeutral and three Divergent outcomes; 28 draws are Valid. All 18 eligible
Neutral soundness comparisons pass. There are no observed catastrophes within
these short horizons; this is not a scientific catastrophe-rate result.

The test invocation completed in 87.33 s. Pre-seal status records 85.79 assay
seconds and 65.36 shared stage/full auditor seconds. Final output is 7,091,691
bytes. The frozen full audit reports peak RSS 231,473,152 bytes, total matching
work 3,326,810 and results 371,854, with per-step peaks 8,646 work / 896 results.
All limits remain below the development ceilings. All 194 frozen hashes,
177 live-source comparisons and 205 sealed output hashes verify, including the
nested basis seal; there are no generated bytecode files in the archive.
The checksum inventory has 206 files on disk counting its own root seal.
Both numerical and gate flags are false, as expected for reduced development
coverage. No full scientific invocation or optimizer evaluation occurred during
the implementation/development task; the subsequent full trial is linked above.

Six ordinary Rust tests pass and the two archive-producing entry points remain
ignored by default. All 12 Python regression groups pass, including the four
archive-dependent groups; the corrected test suite also runs successfully from
the final archive's frozen scripts, not just the live workspace.

Earlier archives remain unchanged:

- `output/ere_complete_gr1_smoke_20260927`: controls and trajectories completed,
  but the frozen final auditor failed because it expected a guard-stat record
  for an unguarded rule. The archive retains its failed status; it was not repaired
  or promoted. The checker now reflects the Engine's sparse guard-stat semantics,
  with an explicit regression.
- `output/ere_complete_gr1_smoke_v2_20260927`: native and frozen stage/full audits
  passed and the output seal was generated. A subsequent read-only verification
  exposed the basename-versus-root-path checksum-filter bug. Its actual sealed
  hashes verify using the corrected live checker, including the nested basis
  seal. All 11 then-current live checker regression groups passed. Its frozen
  checker remains the earlier version; the new final directory freezes the
  corrected implementation and the subsequent review safeguards.
- `output/ere_complete_gr1_smoke_final_20260927` and
  `output/ere_complete_gr1_smoke_verified_20260927`: completed, audited and sealed
  development iterations for the corrected checksum check and traced-Neutral
  typed-world check, respectively. A final code review additionally made parent
  baseline completeness explicit in aggregate acceptance. These receipts remain
  unchanged, with their own frozen code; the current complete smoke freezes all
  safeguards together. This was not a scientific threshold change.

These are development iterations, not scientific seed shopping or scientific
retries. None of the earlier archives supplies a full-horizon G-R1 receipt.

### Test commands

Run ordinary producer checks (six pass; smoke and full assay remain ignored):

```sh
cargo test --release --locked -p hg-sim-core --features hg-meta \
  --test ere_complete_gr1 -- --test-threads=1
```

Run the read-only checker suite against the existing final archive (12 tests):

```sh
python3 -B scripts/test_verify_ere_complete_gr1.py \
  output/ere_complete_gr1_smoke_complete_20260927
```

A new bounded smoke requires a new unused path; do not rerun into the receipt:

```sh
ERE_COMPLETE_GR1_OUTPUT=/absolute/path/to/a/new-development-directory \
  cargo test --release --locked -p hg-sim-core --features hg-meta \
  --test ere_complete_gr1 complete_gr1_development_smoke \
  -- --ignored --exact --nocapture --test-threads=1
```

Existing complete-profile gate regressions also passed: four deterministic gate
tests and three long-gate tests, with their archive-producing assays ignored.
The two existing training-rejection tests passed under
`cargo test --locked -p hg-train --features ere --test ere_complete_witness`,
including zero optimizer/precheck/held-out steps on rejected configurations.
`git diff --check` passed. The immutable contract and its design-time source pins
remain unchanged.

## Limits and next authorization

This is a harness-development result, not the 10,000-draw/100-step scientific
classification. Its eight-step traces cannot observe the 50-step freeze predicate,
and its 18 eligible Neutral checks cannot supply the required 50-check scientific
coverage. Development classifications are diagnostic only. The independent checker
is not a second matching engine, independent ChaCha implementation or independent
Rust hash implementation: draw/RNG/native hash generation remains a Rust witness;
Python independently checks material, selector transformations, world deltas,
paired typed states and accounting. Scope is the fixed supported bank and guard
envelope, not arbitrary complete programs or broad ecological expressivity.

At the implementation-task boundary, the next step was separate approval for
**one** fresh full invocation: 10,000 raw
draws, 168 parent baselines, 100-step horizons, all 5,376 controls, at most
1,016,800 attempted/advancing steps, 3,600 assay seconds, 4 GiB archive output,
900 shared auditor seconds, 1 GiB auditor RSS, fixed store/MetaLimits and the
classification-only pooled 400k/50k envelope. No cap relaxation or automatic retry
is permitted. The ignored `complete_gr1_frozen_assay` entry additionally requires
the exact explicit approval environment sentinels; they are not set by normal tests.

Even a future passing scientific G-R1 receipt leaves new-profile optimizer
distribution/effect/signal-noise prechecks and explicit complete-profile admission
pending. The previously reserved optimizer budget is untouched.
