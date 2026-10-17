# Complete witness codec: implementation and findings

Date: 2026-09-26. Profile `E2-witness-complete-v1`;
diagnostic `E2-complete-witness-controls-v1`.

## Outcome

The new opt-in completeness codec passes the frozen bounded engineering assay.
All **52 composite witness+payload label erasures remained invalid and
quarantined**, replacing the previous profile's 40 changed-body admissions with
zero. All **52 paid sealed-label repairs still recovered the exact original
program for one fuel**. Zero-fuel and binary-only controls recovered zero of
52 each. All624 specified negative cases remained quarantined through the
eight-step repair horizon. This is detection of lost material, not recovery of
lost witnesses or a universal corruption-proof representation.

The run completed 728 cases, 6,552 advancing engine steps, 728 separate paid
Seal preamble applications and 2,184 full-store checkpoints. The independent
audit passed, including recorded ownership/obligation checks and sixteen
in-memory tampering rejections. No matcher ceilings bound. There was one frozen
matrix invocation, exit0 in400.56 seconds, with no retries or omitted conditions.

209 distinct selected Rust tests passed. Additional controls preserve paid
activation of a genuinely distinct proposal, all eligible primitive bindings,
binary repair, repeated sealed repair until fuel exhaustion, malformed-record
quarantine, runtime traces, shared-loop parity and legacy behavior. Full semantic
gates, stateability, repeated-damage ecology and full training remain pending.
No full optimizer or held-out campaign was launched.

## Why this fixes the measured decoder failure

The old version7 reader can lose a relation from its typed projection when both
its witness and payload labels disappear. It can then interpret surviving
material as a different legal rule. The new reader does not infer completeness
from whichever relations happen to remain reachable.

The new codec explicitly declares a family-owned set of body nodes and each
node's required incoming/outgoing relation counts. Ownership links keep body
nodes reachable independently of the attacked relation labels. Missing pairs
leave unsatisfied obligations; lost ownership leaves a count/set mismatch;
owned material that disappears from the projection is rejected. These records
are actual mutable graph state. Neither the decoder nor a repair operation is
given a saved original program or allowed to mutate the live graph during read.

The previous version7 counterexample remains reproducible in a regression:
the old SLOT0 double-erasure case still decodes as a changed program. The old
reader was not silently repaired. Old evidence and its failed quarantine gate
remain historical facts, not retroactively promoted by this new result.

## Representation, versions and extension points

Family encoding is **10**, instruction encoding **11**, with the separately
identified `E2-witness-complete-v1` profile. Opcodes3..6 retain their names:
Seal, binary Repair, WitnessActivate and RepairSealed. Old family7 and
instruction8/9 codecs and operation identities remain unchanged.

| Graph-resident field | Meaning |
| --- | --- |
| `E_OWNS = META_BASE + 0x120`, `[root,body_node]` | One ordered binary ownership link for every body node other than the family root |
| Root property `0x200` | Exact number of owned nodes |
| Per-node property `0x201` | Declared schema node label, including the root |
| Per-node properties `0x220..0x239` | 26 required logical incidence counts: outgoing/incoming for each of the thirteen WITNESS_ROLES |

Required records are strict, uniquely present, nonnegative Int values, including
explicit zeros. Counts are bounded by the region-edge limit; owned-node count
is bounded by the region-node limit. Nodes/instructions have at most64 property
entries. A duplicated required key is rejected, not interpreted first-value-wins.

In the all-role fixture, the complete family has 43 nodes and 124 edges, compared
with 43 nodes/82 edges for the previous witness family. The added 42 binary links
own the 42 body nodes. Added declaration properties total 1,162: 1,118 role counts,
43 node-kind declarations and one root count. The three matrix instruction nodes
are isolated, giving 46 physical nodes in the before-attack store. These are
representation costs, not free hidden observer state.

The reader uses the existing bounded family walk, checks unique ordered/exclusive
ownership and exact owned-set closure, validates declared node types, then
requires a unique correctly labelled witness and one present correctly typed
binary or sealed ternary payload for every logical relation. It compares the
actual per-node relation counts with all declarations. Unexpected typed schema
edges are rejected. A temporary projection contains only present payloads and
the existing BECOMES relations; its entire body must be consumed by ordinary E2
family validation, including the dormant proposal. No missing relation or node
is supplied. Existing node/edge/radius/incidence bounds remain in force.

| Implementation | Role |
| --- | --- |
| [Complete codec](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/complete.rs) | Constants, seed-only ownership/declaration writer, bounded read-only validator/projector, instruction codec, catalogue identity |
| [Family dispatcher](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/program.rs) | Explicit family10 dispatch; existing readers retain their behavior |
| [Witness catalogue](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/repair.rs) | Parameterized native catalogue; old wrappers retain old versions/constraints |
| [Rule source](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/source.rs) | Explicit complete-profile switch, family/instruction admission, separate table identities |
| [ERE runtime](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere.rs) | New protocol installation, complete-family/instruction seed preflight, correct instruction-state measurements |
| [Sim tests](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_complete_witness.rs) | Matrix, primitive/activation controls, loaded-record faults, legacy isolation, bounds and runtime traces |
| [Fixture/checkpoint helpers](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/support/complete.rs) | New codec helpers; prior shared witness helper remains unchanged |
| [Training tests](/Users/kalewicker/Documents/hypersim/crates/hg-train/tests/ere_complete_witness.rs) | Shared-loop parity and no old-ledger/optimizer admission |
| [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_complete_witness.py) | Matrix/inventory/material/frame/payment audit and tampering controls |

The new catalogue contains43 productions:14 Seal,14 binary Repair, one
WitnessActivate and14 RepairSealed. All new-profile matched relation edges are
explicitly ordered, using the compiled matcher flag introduced by the previous
extension. CORE/GRAFT alias their endpoint to the same family root. Native guards
require family10/instruction11, finite Float spendable fuel1..32 and isolated
instructions. RHSs use ordinary replacement/effects, preserve logical declarations
and pay exactly one fuel. They do not repair ownership records or missing witnesses.

New instruction execution identities include the new profile; old identities
are not reused. Old sources quarantine family10/instruction11. New sources
quarantine old family/instruction encodings instead of silently upgrading them.
The runtime still requires explicit E2 pooled families, maintenance measurement,
zero family fuel, no static editor, at most eight instructions and aggregate
seed instruction fuel128. Seed rejection is validated on a temporary store
before modifying the caller. To use this profile, select `COMPLETE_PROTOCOL`
and seed operations3..6; the runtime chooses the complete writer/compiler.
Direct helper callers must use the corresponding `encode_complete_*` /
`compile_complete_operations` APIs and explicit source admission.

World CandidateRule grammar and the matcher implementation were not changed in
this turn. Future legitimate structural edits would need native operations that
keep these declarations coherent; this extension adds no such editor. Activation
changes only the active mode, so a distinct valid dormant proposal can still be
installed without freezing semantic behavior to the original executable table.

## Complete matrix and compile timeline

Four seeds984927043585..588 × thirteen roles × fourteen conditions;52 cases per
condition. Each has a paid targeted native Seal preamble, one ordinary native
attack/control step and eight actual new-profile GraphRuleSource steps.

| Condition | Cases | Valid after attack | Exact original after repair | Repair payments |
| --- | ---: | ---: | ---: | ---: |
| intact_paid | 52 | 52 | 52 | 0 |
| erased_paid | 52 | 0 | 52 | 52 |
| erased_zero | 52 | 0 | 0 | 0 |
| erased_binary_only | 52 | 0 | 0 | 0 |
| payload_deleted | 52 | 0 | 0 | 0 |
| payload_reversed | 52 | 0 | 0 | 0 |
| payload_erased_reversed | 52 | 0 | 0 | 0 |
| payload_foreign_erased | 52 | 0 | 0 | 0 |
| witness_erased | 52 | 0 | 0 | 0 |
| witness_deleted | 52 | 0 | 0 | 0 |
| both_erased | 52 | 0 | 0 | 0 |
| witness_reversed_payload_erased | 52 | 0 | 0 | 0 |
| duplicate_payload | 52 | 0 | 0 | 0 |
| duplicate_witness | 52 | 0 | 0 | 0 |

All104 valid final programs are exact originals:52 untouched controls and52
paid recoveries. All624 negatives stay invalid with zero world classes and one
family quarantine at every repair boundary. Paid-positive cases compile zero
world classes before their first repair, pay once, then install the restored
class at the next boundary. They do not receive installation or recovery credit
before that boundary. No RNG probes changed; negative repair-stage graphs and
fuel remain unchanged apart from advancing store step.

Pooled work/results limits50,000 each and max pairings16 were unchanged.
Maximum observed repair-stage work was **1,344**, results **1**. The four seeds
under deterministic MaxParallel are mechanism replications, not a stochastic
success estimate. Core and dormant proposal in the matrix are identical
eight-variable/eight-arity identity rules with one integer equality guard.
Separate distinct-proposal activation controls demonstrate a genuine installed
table change, but this matrix is not a nontrivial ambient ecology.

## Supplementary tests, limitations and provenance

Exhaustive native tests check40 payload targets for each of Seal, binary Repair
and RepairSealed:120 bindings, plus the one eligible WitnessActivate binding.
All preserve the expected program/mode and pay one fuel with unchanged RNG.
Binary repair is a supported separate operation, not evidence that it can repair
sealed damage. Repeated erasure of the same ternary allows two paid recoveries,
then remains invalid at fuel zero. A budget-one ceiling never applies a partial
rewrite, advances store step or consumes RNG/fuel; the same damaged graph
recovers under sufficient allowance.

Thirteen loaded-record faults cover ownership erasure/deletion/reversal/duplicates,
wrong or mistyped root counts, wrong schema kind, missing/mistyped/excessive or
duplicated relation records, schema-node deletion and property-count overflow.
A separate cross-family ownership control invalidates both attempted owners.
These supplementary faults are malformed-loaded-graph controls, not additional
native-attack ecological rollouts. The 728 matrix attacks are actual native
rewrites. A4,097-edge incidence flood is bounded/rejected without mutation;
insufficient region/radius limits reject seed installation atomically. Malformed
fuel/opcode/version, nonisolated instructions, unsupported arity and pre-witness
operations do not authorize native payment in their specified controls.

One pre-matrix local test invocation failed because it incorrectly assumed every
Seal binding targeted the single selected payload; Seal legitimately has bindings
to all remaining binary payloads. The corrected test selects each target's
bindings in its own iteration and covers all40 targets. No production change
was made for this correction, and no frozen matrix invocation was discarded.
The failure and successful commands are documented in the verification notes.

The engineering result is deliberately bounded. Completeness metadata can itself
be damaged and is not proactively protected or repaired here. Literal-value
substitutions and coordinated rewrites that keep all declarations coherent are
not claimed to be detected. Declaration counts are not a saved semantic checksum
or permission to reconstruct a lost witness. Quarantine does not imply ecological
survival, perpetual fuel, learned adaptation or recovery after authority loss.

Rust writes/restores HGSSv2 tagged-bincode full stores and checks IDs, typed
properties, endpoint order, step, fuel, decode, incidence multiplicities, label
indexes and reader nonmutation. Python independently checks all recorded
inventories and ownership/obligation material, exact decoder-result comparisons,
native graph deltas, per-step classes/quarantines/payments/fuel/RNG and wire frame
headers/lengths. It does not independently deserialize bincode or implement the
full CandidateRule decoder. Sixteen in-memory tampering controls reject false
recovery/payment, hidden ceilings, malformed/missing records and incomplete or
inconsistent timelines without altering raw evidence.

The frozen contract, compiled-in harness/two fixture sources, sixteen selected
production/lockfile snapshots and exact executed test binary were archived before
measurement. All snapshots match the current source files at completion. They
are selected provenance, not a complete standalone build tree or general resume
package. All six older witness/sealed-repair evidence checksum manifests verified
unchanged. The new manifest includes raw evidence and final reports/auditor.

Primary evidence:

- [Frozen contract](/Users/kalewicker/Documents/hypersim/docs/ere_complete_witness_v1.md)
- [Complete summary](/Users/kalewicker/Documents/hypersim/output/ere_complete_witness_20260926/summary.json)
- [Independent audit](/Users/kalewicker/Documents/hypersim/output/ere_complete_witness_20260926/receipt_verification.json)
- [Final status](/Users/kalewicker/Documents/hypersim/output/ere_complete_witness_20260926/status.json)
- [Verification commands and development failure](/Users/kalewicker/Documents/hypersim/output/ere_complete_witness_20260926/verification_tests.md)
- [Artifact checksums](/Users/kalewicker/Documents/hypersim/output/ere_complete_witness_20260926/checksums.sha256)
- [Prior failed completeness boundary](/Users/kalewicker/Documents/hypersim/docs/ere_sealed_repair_findings.md)

Contract SHA-256:
`9a4a4d976eef7a25d70db31499265ff97aefc377902ab8af2daadbdc716020f5`.
Compiled-in harness SHA-256:
`7bcdbe9d3311bd690c30095b8fb203b2b9a0b1a462991b310c6403a521fd7d12`.
Executed matrix binary SHA-256:
`7e5618d4d119e87e806242989c244daf13a1beed8bcbec15dade8f6c9c80a4dc`.

## Reproduce and next boundary

From the project root, using a fresh output directory:

```sh
ERE_COMPLETE_OUTPUT=/private/tmp/ere-complete-new-run cargo test -p hg-sim-core --features hg-meta --test ere_complete_witness complete_witness_matrix -- --nocapture
python3 scripts/verify_ere_complete_witness.py /private/tmp/ere-complete-new-run --test-rejections
cargo test -p hg-sim-core --features hg-meta --test ere_complete_witness -- --skip complete_witness_matrix --nocapture
cargo test -p hg-train --features ere --test ere_complete_witness
```

The next authorized design decision is a separately preregistered bounded
repeated-damage ecology with fuel/no-fuel and protected/unprotected comparisons,
including explicit ownership-record loss as a detector-only boundary. This work
does not launch that experiment or the reserved full campaign. New-profile full
semantic gates and stateability still need verification before training admission.
