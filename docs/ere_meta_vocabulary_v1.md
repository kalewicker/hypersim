# Schema-preserving meta vocabulary v1

Implementation/assay contract frozen 2026-09-26 before new vocabulary results.
Scope: a small explicit graph-encoded vocabulary and bounded mechanism/control
assay. No optimizer, full training, automatic graph repair or budget increase.

## Representation and semantics

Keep E2 world core/proposal families and their strict reader unchanged. Add an
opt-in finite instruction layer, protocol `E2-schema-preserving-vocabulary-v1`.
An instruction is an isolated meta-labeled graph node carrying version, opcode
and Float fuel. This is a **finite macro vocabulary**, not arbitrary new rule
syntax: its decoder expands graph opcodes into ordinary engine patterns,
templates, guards and property effects. The compiler never executes an edit.
Every edit is selected/applied by the existing matcher/conflict/apply pipeline.

Three intrinsic operations, each costing one unit of instruction fuel:

1. Activate: typed E2 root/proposal graft; mode 0 -> 1, preserving labeled link.
2. Deactivate: same link; mode 1 -> 0, preserving labeled link.
3. Transpose proposal slots 0/1: follow graft, proposal side linkage, RHS
   membership, and two typed variable slots. Recreate every matched schema edge
   with its correct label, swapping only the two endpoints. Distinct variables
   prevent a paid alias no-op. Shared LHS/RHS patterns may both change; this is
   an explicit reversible genotype edit, not an RHS-only guarantee.

The instruction itself survives; native degree/version/opcode/fuel guards exclude
malformed instruction nodes and exhausted fuel even between compile epochs.
Unknown opcodes/quarantined instructions have no cached-body fallback. Canonical
instruction rules deduplicate by opcode; multiple graph instruction nodes remain
real alternative bindings, not duplicated rule-table entries. Fuel is runtime
state, not an executable identity change. All instruction bindings are meta-only
and cannot receive structural world-reuse credit.

Preservation claims apply to initially valid E2 families and isolated primitive
firings. These operations cannot recreate missing nodes/links, do not repair a
quarantined family, and do not prevent concurrent destructive world-rule edits.
The isolated instruction storage is declared scaffolding, not proof that its
code was evolved or survives an adversarial exposure regime.

## Compatibility and instrumentation

New config field omitted when absent; ordinary E2/default matching remains
unchanged. Vocabulary requires explicit E2 pooled config, maintenance measurement,
and no simultaneous static seeded editor. Static A0/frozen A1 initially compile
the same world and vocabulary table; both keep that table frozen. A2 excludes
ordinary world-rule meta matches but permits the vocabulary. A3 remains fully
meta-visible, without label compartmentalization or an exclusion guard.

Maintain original seeded-family validity denominators. Also record instruction
validity/fuel and per-operation traces: step, opcode, bound instruction/target,
fuel before/after, whole-family validity and executable signature before/after.
Vocabulary event counts are a subset of non-static-editor meta events, explicitly
separated from destructive world-rule meta rewrites. No static editor is present
in this assay. Never call vocabulary firings learned discoveries.

Old gate/classification receipts are retained, not used to admit this new
semantic profile to training. Legacy and prior pilot entry points must reject
vocabulary configs; new full G-R/stateability admission is pending.

## Frozen verification and bounded assay

Unit/contract checks: all three opcodes round-trip/compile; invalid nodes/energy/
versions reject; exhausted or malformed nodes cannot fire; exact debit and
finite conservation; each primitive preserves valid whole families; transpose
changes a real decoded body and can reverse it; live activation changes actual
world execution; no world credit for meta-only operations; canonical source/RNG
identity and static/frozen identity; quarantine/retirement after opcode damage;
legacy contracts and explicit training refusal.

One new diagnostic assay: seed `0xE452390001`, four common seeds, A1/A2/A3
x 1,000 steps = **12 attempts / 12,000 requested-step ceiling**. Same existing
40-logit world grammar initialized at zero, 4x4 periodic binary PureCrystal,
MaxParallel max16, thread1, epoch10, metrics100, runtime50k work/50k results,
meta-region500, EI off. Four instruction nodes: Activate, Deactivate, Transpose,
Transpose, each fuel32 (128 total), replacing the old editor's 128-unit seed
allocation. Family editor fuel is zero, with no replenishment. Instructions are
shared by all matched arms; mode toggles and repeated transpositions are not
additional novelty reward. No retries, optimized genotype search or held-out run.

Report all errors, completed steps, caps, family/instruction validity, R/T/F/J,
paid vocabulary events, destructive world meta events and exact traces. A failed
mechanism assertion or any cap makes the diagnostic incomplete/failed. Even a
successful assay never passes pending full vocabulary gates or authorizes training.
Four-seed comparisons are descriptive; differences from prior seeds/editor
semantics are not controlled evidence of improvement.

Outputs: fresh `output/ere_meta_vocabulary_20260926`, manifest before evaluations,
effective configs and full ERE rows, primitive/control receipts, status/report,
checksums. Catalogue/contract tests are separate from the 12-rollout assay.

## Results

Pending; append findings without changing the frozen contract.

### Findings: 2026-09-26

The vocabulary is implemented and its engineering controls pass. The diagnostic
completed, but **useful unprotected maintenance is not established**. This is not
a training admission or a full G-R/stateability pass.

The frozen pre-result document is retained as
[preregistration.md](../output/ere_meta_vocabulary_20260926/preregistration.md),
SHA-256 `569d3b170b6191d5e345c73c6ebbb56dc7e4f8bf1d93396cbee89a47f2e3bbba`.
No assay settings, seeds, caps or requested steps were changed after measurement.

#### Implementation and extension map

| Component | Source | Contract |
|---|---|---|
| Instruction codec and finite macro compiler | `crates/hg-meta/src/vocabulary.rs` | Version4 isolated instruction; opcode0/1/2; finite Float fuel0..32; native typed guards and deterministic property effects |
| Graph rule source | `crates/hg-meta/src/source.rs` | Explicit opt-in; canonical opcode deduplication after sorted world rules, before static editors; malformed instructions quarantine without repair/fallback |
| Configuration, installation, attribution | `crates/hg-sim-core/src/ere.rs` | Explicit E2 pooled families, maintenance enabled, zero family fuel, no old editor, 1..4 instructions; validates before seeding |
| Window/partial-halt capture | `crates/hg-sim-core/src/runner.rs` | `run_ere_diagnostic` uses the existing production loop; retains trailing measured rows on loop errors, never applies capped partial pools |
| Trace schema | `crates/hg-schema/src/ere.rs` | Optional vocabulary fields omitted for old profiles; seeded-family denominators retained; instruction states measured every advancing step |
| Bounded diagnostic and CLI | `crates/hg-train/src/ere/vocabulary.rs`, `src/bin/ere-train.rs` | Fixed twelve-attempt assay; fresh output only; no optimizer, retry, old admission or held-out work |
| Old entry-point guards | `crates/hg-train/src/ere.rs`, `ere/e2.rs`, `ere/maintenance.rs` | Vocabulary configs cannot enter older campaigns/pilots with old receipts |
| Independent accounting audit | `scripts/verify_ere_vocabulary.jq` | Reconstructs event, score, energy, matched-material and status arithmetic from raw rows; not graph replay |

Activate and Deactivate each use three variables and one labeled graft edge.
Transpose uses **eight** variables and five labeled edges, including the ternary
proposal-side link. All matched edges are recreated with their original schema
labels; only the two slot endpoints change. All three use priority 100 and a
native one-unit fuel subtraction. No survival reward is credited to these rules.

To add an operation, extend the enum/codec and native expansion together; prove
that reader validity constraints are mirrored by executable guards, including
malformed same-op siblings under a cached table. Preserve opcode table ordering,
rule-ID attribution, bounds and deterministic compilation. Any semantic change
needs an explicit new protocol and new controls/admission; do not silently add
an operation to this frozen assay or widen the E2 world-rule envelope.

Traces are **whole-step pre/post observations**, not snapshots immediately around
an individual firing. Other disjoint world rewrites can invalidate another part
of a traced family during the same step. Executable/proposal signatures are
64-bit hashes of complete execution bytes, not collision-free stored graph
snapshots. Proposal signatures expose dormant edits that do not change the
executable table. Reader rejection remains authoritative for validity.

#### Completed assay and every rollout

All twelve attempts completed 1,000 steps: **12,000 actual/requested steps**,
zero errors and zero ceiling hits. Each spent exactly 128 instruction fuel:
32 Activate, 32 Deactivate and 64 Transpose firings. All 48 seeded instruction
nodes remained valid for all 1,000 steps and ended at fuel 0. Total paid
events/fuel spent: **1,536**. No static-editor event or energy replenishment.

Seed abbreviations below mean `980632010753 + offset`. R is normalized structural
proposal reuse, T the seeded-family valid-time fraction, F final valid-family
fraction, J the mean-of-run product R x T x F. Each row links to the raw effective
config, emission, observation and ten ERE windows.

| Offset | Arm | R | T | F | J | Structural world fires | World-rule meta events | Lost families | Raw receipt |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---|
| 0 | A1 | .270885 | 1 | 1 | .270885 | 8,000 | 0 | 0/4 | [000](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_000.json) |
| 0 | A2 | .270885 | 1 | 1 | .270885 | 8,000 | 0 | 0/4 | [001](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_001.json) |
| 0 | A3 | .132079 | .500 | .500 | .033020 | 80 | 5,892 | 2/4 | [002](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_002.json) |
| 1 | A1 | 0 | 1 | 1 | 0 | 0 | 0 | 0/4 | [003](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_003.json) |
| 1 | A2 | 0 | 1 | 1 | 0 | 0 | 0 | 0/4 | [004](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_004.json) |
| 1 | A3 | 0 | .018 | 0 | 0 | 0 | 272 | 4/4 | [005](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_005.json) |
| 2 | A1 | .291520 | 1 | 1 | .291520 | 15,864 | 0 | 0/4 | [006](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_006.json) |
| 2 | A2 | .291520 | 1 | 1 | .291520 | 15,864 | 0 | 0/4 | [007](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_007.json) |
| 2 | A3 | .142221 | .500 | .500 | .035555 | 112 | 8 | 2/4 | [008](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_008.json) |
| 3 | A1 | .270885 | 1 | 1 | .270885 | 8,000 | 0 | 0/4 | [009](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_009.json) |
| 3 | A2 | .270885 | 1 | 1 | .270885 | 8,000 | 0 | 0/4 | [010](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_010.json) |
| 3 | A3 | .132079 | .005 | 0 | 0 | 80 | 80 | 4/4 | [011](../output/ere_meta_vocabulary_20260926/rollouts/evaluation_011.json) |

The raw field `destructive_world_meta_events` is an overbroad label: its precise
definition is **ordinary meta events minus encoded vocabulary events**, i.e.
world-rule meta-touching/creating rewrites. Individual events are not proven
destructive or causally responsible for family loss. Raw receipts are unchanged;
the derived summary and table use the accurate name `world_rule_meta_events`.
The driver now emits that accurate name for future receipts; the audit accepts
both names and still reconstructs the count from original window events.

A1/A2 mean J=.208322, mean T=F=1, 31,864 structural world fires per arm. A3 mean
J=.017144, mean T=.255750, mean F=.25, 272 structural world fires and 6,252
world-rule meta events. **Twelve of sixteen A3 families failed, without recovery**:
eight at step 1, two at 11, one at 33 and one at 41. Final reasons: nine Sides,
three Variables. A3 accumulated 1,179 repeated quarantine decisions, not 1,179
distinct failed families. Explicit cleanup-loss counts were zero; schema damage
need not involve deleting an anchor or incident-edge cleanup.

#### Why paid edits did not yield useful live adaptation

Every matched A1/A2 pair has the same final graph hash, included/excluded window
metrics, structural firing maps and operation traces. A2 shows no post-seed
installations/retirements, no epoch-observed activations and no new RCF classes.
These receipts do not contain full per-step graph/RNG snapshots, so this is not
a fresh 100,000-step G-R3 identity result.

The traces show two-step cycles: Activate/Deactivate swap modes back, and
Transpose reverses its previous slot swap. The fixed ten-step compile cadence
samples restored states. In all 768 transposition traces, no decodable executable
signature changed; 724 show a changed decoded proposal body, with the remaining 44
undecodable after world interference. Safe real genotype edits occurred, but they
were dormant or canceled before changing the installed table. Positive J can
come from initial seeded alternatives and is not evidence of discovered rules.

The native patterns can also bind a still-present graft in a quarantined family;
they do not invoke the whole-family reader as a host-side guard. A3 paid for 290
firings on already-invalid families. No trace establishes reconstruction of
missing schema or a valid-family recovery. Adding these three operators is
**not a complete repair vocabulary**.

The 48 instruction nodes survived because their declared isolated storage has no
edges for this world grammar to match. This is engineered scaffolding, not
learned protection. A3 world rules still had ordinary access to family meta data.
No comparison to prior pilots is causal evidence of improvement: seeds and
editor semantics changed.

#### Verification, provenance and next decision

**115 distinct targeted tests passed**: meta20, sim-core ERE51, training14,
engine pooled/conflict 12, schema 4 and feature-off pipeline/snapshot/matcher 14.
The exhaustive primitive control checked **4,032 isolated bindings** across all
28 seed-bank core/proposal pairs, both modes and all three operators: complete
families survived, node/edge counts were conserved, exact fuel was debited and
the RNG state was unchanged. This finite catalogue test is not a proof for every
possible encoded graph. Separate controls demonstrate reversible body edits,
real activation-to-world-execution, cached malformed/exhausted rejection,
source retirement, static/frozen identity, no world credit and training refusal.

The independent audit reconstructed all eight accounting checks. Three in-memory
corruptions (paid count, fuel debit, valid-time count) were rejected. Previous
maintenance/grammar/classification/identity receipts checked before/after were
unchanged. Feature-off build and `git diff --check` passed. The generic diagnostic
error hook was subsequently centralized so all loop errors retain their trailing
window; its success/cap parity test passed. The raw counter-name clarification
and early legacy arm-sweep refusal were also added without a scientific rerun.
The successful assay was not rerun; provenance distinguishes executed and
final verification builds.

Executed assay binary SHA-256:
`1c748117adae6ba5d3b75552024dd6916e121fbca3fda0f6e9b580e94e219f5c`.
See [provenance](../output/ere_meta_vocabulary_20260926/provenance.json),
[test evidence](../output/ere_meta_vocabulary_20260926/verification_tests.md),
[derived summary](../output/ere_meta_vocabulary_20260926/derived_summary.json),
[arithmetic audit](../output/ere_meta_vocabulary_20260926/receipt_verification.json)
and [raw status](../output/ere_meta_vocabulary_20260926/status.json).

Subsequent result: the separately preregistered
[one-use activation persistence control](ere_edit_persistence_findings.md)
completed24/24 attempts and passed the protected mechanism in all four seed
groups. One paid edit persisted to the compile boundary and changed both the
installed world table and world rewrites at step11. A3's edited family was
invalid from step1, so unprotected maintenance and training admission remain
unestablished. This follow-up does not revise the earlier twelve-run results.

Do not launch optimization or treat `diagnostic_complete` as a maintenance pass.
The next narrow research step should precommit an **edit-persistence/compile-boundary
control** (for example a one-use edit held through one compile boundary), keeping
paid energy, resource caps and A3 exposure explicit. It should demonstrate actual
installed-table and world-trajectory change before testing any more extensive
maintenance/repair operators. Neither that follow-up nor new full vocabulary
G-R/stateability checks have been run. No optimizer/held-out steps were used.

Reproduce the arithmetic audit without rerunning physics:

```sh
jq -s \
  --slurpfile manifest output/ere_meta_vocabulary_20260926/manifest.json \
  --slurpfile evaluations output/ere_meta_vocabulary_20260926/evaluations.json \
  --slurpfile status output/ere_meta_vocabulary_20260926/status.json \
  -f scripts/verify_ere_vocabulary.jq \
  output/ere_meta_vocabulary_20260926/rollouts/*.json
```

CLI implementation: `cargo run --release -p hg-train --features ere --bin
ere-train -- vocabulary-assay configs/ere_e2_pooled.toml FRESH_OUTPUT_DIR`.
Do not rerun merely to improve this result; any new scientific test needs its
own preregistration and budget decision.
