# One-use activation persistence: results and implementation

Executed 2026-09-26. **The protected persistence control passed in all four
predeclared seed groups. Unprotected maintenance did not.** This is a mechanism
test of a deliberately matchable hand-built production, not optimization,
learned discovery, repair or full vocabulary gate admission.

Protocol: `E2-one-use-activation-persistence-v1`; unchanged instruction semantics:
`E2-schema-preserving-vocabulary-v1`. The
[frozen preregistration](ere_edit_persistence_v1.md) remains byte-for-byte
unchanged. Findings are separate so later builds do not embed retrospective
results as the preregistration. The executed snapshot is also preserved
[in the receipt directory](../output/ere_edit_persistence_20260926/preregistration.md).

## Outcome and temporal control

All **24 attempts completed 100 steps: 2,400 actual / 2,400 requested steps**.
No scientific retries, cap errors, missing step measurements, optimizer
evaluations or held-out evaluations. All nine protected assertions passed for
each of the four common seeds, 984926978049 through 984926978052.

| Observation | Frozen A1 edit | Protected/live A2 edit | Unprotected/live A3 edit |
|---|---|---|---|
| Step1 | One paid Activate; fuel1 to0 | Same | One paid Activate; target already invalid in aggregate post-step measurement |
| Completed steps1–10 | Edited family valid and active; old world table | Same world table, projected world hashes and RNG as A1; edited family valid/active | Two of four families invalid from step1, including the edited target |
| Advancing step11, compilation at store step10 | Old table remains frozen | First installed-table difference, first projected-world difference, first structural-proposal world firing | Invalid families quarantined; edited proposal never installed |
| Through step100 | Zero proposal fires; all four families valid | 720 structural-proposal world fires; all four families valid | Zero proposal fires; two families valid, no recovery |

No-edit A1/A2 traces matched for all 100 completed steps in installed world
execution bytes, projected world hashes and RNG, with zero active families and
zero proposal credit. The instruction node/opcode is present in both conditions;
only its initial finite fuel differs. A1 editing is not an inert graph: its
graph-encoded activation succeeds, but its installed table stays frozen.

A2's edit changed the installed world table from four identity candidates to
five candidates, adding the path-to-star structural proposal at step11.
World node/edge counts stayed16/64: the outcome is a structural rewrite,
not growth. At step11 the projected hash was `44d60e3fa1a563c5`, versus
`90f65adcc6fe99c7` for the frozen control. Full candidate execution bytes are
stored rather than only table hashes. There were 2,880 A2 proposal fires across
the four edit runs and twelve paid activations across all twelve edit runs.

This resolves the previous assay's two-step edit cancellation *for this
one-use control*: an exhausted instruction cannot undo the activation before
the ten-step compilation boundary. It does not establish that longer-lived,
multi-operation programs adapt or maintain themselves.

## Exposure diagnostic and limitations

Every A3 run lost families0 and1 at completed step1; families2 and3 remained
valid for all100 steps. The edited target was family0. Its first reported
failure was `Bounds` in edit runs, then `Sides` by the final measurement;
family1's failure was `Sides`. Final failure was `Sides` for both invalid
families in both conditions. No family recovered.

Across the four A3 edit runs, **8 of16 families became invalid**. The four A3
no-edit runs also lost8 of16. All A3 runs had T=.5, F=.5 and J=0; there were18
quarantine decisions per run (repeated decisions, not18 distinct lost families).
The edit runs had799 world-rule meta events each versus800 without editing.
This counter measures world-rule meta touch/create events, not an independent
destruction count. Family-validity measurements, rather than this event count
or anchor survival, establish the damage. All instruction nodes themselves
remained valid with fuel0.

The native local activation and ordinary meta-visible world matches can occur
within the same step; aggregate post-step traces do not attribute damage to an
individual event. Do not claim that Activate itself caused the loss, that a
fuel-only contrast isolates all A3 dynamics, or that A2's engineered protection
demonstrates unprotected maintenance.

These four engine seeds produced the same reported dynamics in this fixed
fixture. No engine RNG probe changed during compilation *or stepping* in any
attempt. Thus this is deterministic mechanism replication across seeded runs,
not four stochastic samples or a non-vacuous stochastic RNG-consumption test.
Existing separate RNG-positive-control regressions passed. Maximum observed
match work was11,302 and results1,408, below the unchanged50k/50k caps.

R is the existing priority-normalized structural-class reuse score, T is
whole-family valid time /400 possible family-steps, F is final valid families/4,
and J=R*T*F. A2 edit R=J=.27441164294272646; all other conditions have R=J=0.
These are descriptive, initial material was not discovered, EI was disabled,
and no initial-genotype novelty credit is claimed. Hash equality is a
diagnostic, not full-graph archival or a collision-free identity proof.

## Every attempt

Each group used the six predeclared conditions in the same order. The entire
matrix is retained, including the negative exposure outcomes.

| Receipt | Engine seed | Arm | Edit | Steps / lifecycle | Paid activations | Proposal world fires | T | F | J |
|---|---|---|---|---|---|---|---|---|---|
| [000](../output/ere_edit_persistence_20260926/rollouts/evaluation_000.json) | 984926978049 | A1 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [001](../output/ere_edit_persistence_20260926/rollouts/evaluation_001.json) | 984926978049 | A1 | yes | 100 / complete | 1 | 0 | 1 | 1 | 0 |
| [002](../output/ere_edit_persistence_20260926/rollouts/evaluation_002.json) | 984926978049 | A2 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [003](../output/ere_edit_persistence_20260926/rollouts/evaluation_003.json) | 984926978049 | A2 | yes | 100 / complete | 1 | 720 | 1 | 1 | 0.274412 |
| [004](../output/ere_edit_persistence_20260926/rollouts/evaluation_004.json) | 984926978049 | A3 | no | 100 / complete | 0 | 0 | 0.5 | 0.5 | 0 |
| [005](../output/ere_edit_persistence_20260926/rollouts/evaluation_005.json) | 984926978049 | A3 | yes | 100 / complete | 1 | 0 | 0.5 | 0.5 | 0 |
| [006](../output/ere_edit_persistence_20260926/rollouts/evaluation_006.json) | 984926978050 | A1 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [007](../output/ere_edit_persistence_20260926/rollouts/evaluation_007.json) | 984926978050 | A1 | yes | 100 / complete | 1 | 0 | 1 | 1 | 0 |
| [008](../output/ere_edit_persistence_20260926/rollouts/evaluation_008.json) | 984926978050 | A2 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [009](../output/ere_edit_persistence_20260926/rollouts/evaluation_009.json) | 984926978050 | A2 | yes | 100 / complete | 1 | 720 | 1 | 1 | 0.274412 |
| [010](../output/ere_edit_persistence_20260926/rollouts/evaluation_010.json) | 984926978050 | A3 | no | 100 / complete | 0 | 0 | 0.5 | 0.5 | 0 |
| [011](../output/ere_edit_persistence_20260926/rollouts/evaluation_011.json) | 984926978050 | A3 | yes | 100 / complete | 1 | 0 | 0.5 | 0.5 | 0 |
| [012](../output/ere_edit_persistence_20260926/rollouts/evaluation_012.json) | 984926978051 | A1 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [013](../output/ere_edit_persistence_20260926/rollouts/evaluation_013.json) | 984926978051 | A1 | yes | 100 / complete | 1 | 0 | 1 | 1 | 0 |
| [014](../output/ere_edit_persistence_20260926/rollouts/evaluation_014.json) | 984926978051 | A2 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [015](../output/ere_edit_persistence_20260926/rollouts/evaluation_015.json) | 984926978051 | A2 | yes | 100 / complete | 1 | 720 | 1 | 1 | 0.274412 |
| [016](../output/ere_edit_persistence_20260926/rollouts/evaluation_016.json) | 984926978051 | A3 | no | 100 / complete | 0 | 0 | 0.5 | 0.5 | 0 |
| [017](../output/ere_edit_persistence_20260926/rollouts/evaluation_017.json) | 984926978051 | A3 | yes | 100 / complete | 1 | 0 | 0.5 | 0.5 | 0 |
| [018](../output/ere_edit_persistence_20260926/rollouts/evaluation_018.json) | 984926978052 | A1 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [019](../output/ere_edit_persistence_20260926/rollouts/evaluation_019.json) | 984926978052 | A1 | yes | 100 / complete | 1 | 0 | 1 | 1 | 0 |
| [020](../output/ere_edit_persistence_20260926/rollouts/evaluation_020.json) | 984926978052 | A2 | no | 100 / complete | 0 | 0 | 1 | 1 | 0 |
| [021](../output/ere_edit_persistence_20260926/rollouts/evaluation_021.json) | 984926978052 | A2 | yes | 100 / complete | 1 | 720 | 1 | 1 | 0.274412 |
| [022](../output/ere_edit_persistence_20260926/rollouts/evaluation_022.json) | 984926978052 | A3 | no | 100 / complete | 0 | 0 | 0.5 | 0.5 | 0 |
| [023](../output/ere_edit_persistence_20260926/rollouts/evaluation_023.json) | 984926978052 | A3 | yes | 100 / complete | 1 | 0 | 0.5 | 0.5 | 0 |

See [derived summary](../output/ere_edit_persistence_20260926/derived_summary.json)
for per-attempt losses, first invalid steps, validators, scores and exact values;
[contrasts](../output/ere_edit_persistence_20260926/contrasts.json) for all assertions;
and [status](../output/ere_edit_persistence_20260926/status.json) for budget totals.

## Implementation and extension seams

| Component | Responsibility and extension constraint |
|---|---|
| `crates/hg-schema/src/ere.rs` | New separate `EreStepObservation` / `EreFamilyStepState` types. Full execution bytes are hex; RNG probes use16-digit hex strings so JSON tooling does not round64-bit integers. Existing metric serialization is unchanged. |
| `crates/hg-sim-core/src/runner.rs` | Optional `RunHooks.ere_step_capture` and `run_ere_instrumented_diagnostic`, using the existing production loop. Capture only advancing completed steps; partial cap/error accounting remains explicit. Normal entry points leave capture disabled. |
| `crates/hg-sim-core/src/ere.rs` | Read-only `observe_completed_step`: meta-excluded world, installed world candidates excluding native instruction rules, revisions, matcher usage, full seeded-family decoding. Installed-candidate decode failure becomes explicit null, never inferred success. |
| `crates/hg-train/src/ere/persistence.rs` | Explicit dormant catalogue fixture, bounded configure/evaluate/contrast/driver. Fuel0/1, one Activate, no reverse instruction/refill/static editor. Saves all configs, preregistration, binary and four source snapshots before evaluation. Fresh-directory-only. |
| `crates/hg-train/src/bin/ere-train.rs` | `persistence-assay BASE_CONFIG FRESH_OUTPUT`; no optimizer route or admission reuse. |
| `crates/hg-train/tests/ere_persistence.rs` | Budget/fixture guards, short normal-versus-instrumented exact graph/history/window parity, step11 table installation, cap accounting, missing controls and incomplete observations. |
| `scripts/verify_ere_persistence.jq` | Independent reconstruction of profiles, matched material, step validity, fuel, scores, contrast decisions and status from saved rows. Not independent graph replay. |

No matcher, conflict resolution, apply semantics, vocabulary guard/timing,
decoder repair, production default or training-admission semantics changed.
To extend the instrumentation, preserve its opt-in/read-only behavior and
parity tests. To change the fixture, operations, caps or decision rule, create a
new protocol and preregistration; do not edit or overwrite this receipt set.

## Verification and provenance

**87 targeted tests passed**:18 training/driver,51 ERE simulation,14 feature-off
regression and4 schema roundtrip tests. Feature-off check and release build
passed. The short observer-parity test compared the actual final full graph
hash, standard history and ERE rows with/without capture, not just a new summary.
Cap and missing-ERE controls cannot invent completed-step observations.
This is not a full-workspace test claim.

The independent audit passed all nine checks. Three in-memory mutations
(changing step11 installed bytes, breaking a pre-step world-hash chain, and
falsifying a fuel debit) were rejected with exit5. Raw receipts were never
rewritten. Selected prior vocabulary, maintenance and classification receipts
had unchanged SHA256 checksums before/after.

Executed binary SHA256:
`8dde3352b8edfa2e711ecd646093cd014ed03d0f399da3bda57a36e9a1b4bdf4`.

Frozen preregistration SHA256:
`23d6254fd323d48b596189abc951c00111eb639fa957120292b9091457c5430a`.

The saved binary matches the executed release binary. All four compile-time
source snapshots match their current source files. Supporting artifacts:
[test evidence](../output/ere_edit_persistence_20260926/verification_tests.md),
[receipt audit](../output/ere_edit_persistence_20260926/receipt_verification.json),
[corruption controls](../output/ere_edit_persistence_20260926/corruption_controls.json),
[provenance](../output/ere_edit_persistence_20260926/provenance.json) and
[checksums](../output/ere_edit_persistence_20260926/checksums.sha256).

## Current decision

Protected edit persistence and real installed-world consequences are verified
for this control. **Full vocabulary gates/stateability remain pending; full
training remains refused, with zero optimizer and held-out steps.**

The next unresolved mechanism is damage prevention or paid schema-local repair
under A3 exposure, not compilation timing. A useful next design would require
an explicit new intervention and matched damage/repair controls with whole-body
validity and fuel accounting. No such redesign or additional run was performed
here, and no hidden host repair or broader protection was introduced.
