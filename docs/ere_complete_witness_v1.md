# Complete witness ownership codec: frozen bounded contract

Date: 2026-09-26. New opt-in profile `E2-witness-complete-v1`.
Diagnostic `E2-complete-witness-controls-v1`. This is a completeness/engineering
assay, not an ecological efficacy test, full semantic gate or training admission.

## Representation and decision rules

Family encoding 10 and instruction encoding 11 are separate from the existing
family7/instruction8/9 profiles. Opcodes remain 3..6: Seal, binary Repair,
WitnessActivate and RepairSealed. Their new catalogue/identity belongs only to
this profile. Existing readers, codecs, native productions and outputs retain
their original behavior; old rule sources quarantine the new profile material.

Seed an ordinary witness family plus a typed binary ownership relation
`E_OWNS = META_BASE + 0x120`, ordered `[family_root,body_node]`, for every body
node other than the family root. The root declares its owned-node count at
property0x200. Every family/body node declares its own schema node label at
0x201 and all 26 required relation-incidence counts at properties0x220..0x239:
outgoing and incoming counts for each of the thirteen witness roles in the
existing WITNESS_ROLES order. Counts are strict nonnegative Int values bounded
by the region-edge limit, including explicit zero obligations. They count
logical binary relations, not sealing arity, edge IDs or redundant witnesses.

Ownership anchors every body node independently of its payload/witness labels.
The bounded read-only reader requires exactly the declared owned set, unique
correctly ordered ownership, declared node types, exclusive ownership, closed
typed schema topology, one correct witness and one present correctly typed
payload per required relation, and exact per-node role counts. The projection
must consume the entire declared body, including dormant material. The new
writer rejects inputs with unconnected/unconsumed schema material rather than
silently dropping it. Existing region node/edge/radius/incidence bounds apply;
no global unbounded scan, cached original or live mutation is permitted.
Complete-profile nodes/instructions have at most64 property entries. Required
metadata keys must occur exactly once; duplicate records are not first-value-wins.

Only present typed payloads are projected. Missing/erased/deleted ownership,
wrong/missing/mistyped counts/types, incomplete relation material and foreign or
duplicate ownership quarantine the family. Metadata is ordinary mutable graph
state, not an immutable oracle or a proof against coordinated rewriting of all
records. Literal-value substitution with otherwise complete topology is not
claimed to be detected. This profile does not introduce arbitrary structural
editing or regenerate lost witnesses.

The four new instruction variants use ordinary native matcher/conflict/apply
and finite Float fuel0..32, with spend eligibility1..32 and aggregate seed
fuel128. All new-profile witness/payload LHS edges are explicitly ordered;
CORE/GRAFT alias their role endpoint to the same family root. Instructions
require family10 and instruction11. Old operations retain their old constraints.
GraphRuleSource, runtime, metrics and shared-loop instrumentation must honor
explicit profile admission. Old instruction/family versions cannot silently
upgrade into the new source, nor may new receipts reuse the old v2 ledger.

## Frozen matched matrix

Repeat exactly the previous 728-case design: seeds984927043585..588,
thirteen roles, fourteen conditions, eight repair steps, no adaptive retries,
max pairings16, pooled work/results50,000 each. Use the same all-role eight-var,
eight-arity identity fixture with one integer guard, identical core and dormant
proposal. Seal the first payload of each role with one paid targeted native
preamble; resolve the recreated live witness afterward. Attack/control is one
ordinary native rewrite; subsequent steps use the actual new GraphRuleSource.

| Condition | Binary / sealed repair fuel | Required final result |
| --- | --- | --- |
| intact_paid | 0 / 1 | Exact original; no repair payment |
| erased_paid | 0 / 1 | Exact original; one sealed repair payment |
| erased_zero | 0 / 0 | Invalid/quarantined; no payment |
| erased_binary_only | 1 / 0 | Invalid/quarantined; no payment |
| payload_deleted | 0 / 1 | Invalid/quarantined; no payment |
| payload_reversed | 0 / 1 | Invalid/quarantined; no payment |
| payload_erased_reversed | 0 / 1 | Invalid/quarantined; no payment |
| payload_foreign_erased | 0 / 1 | Invalid/quarantined; no payment |
| witness_erased | 0 / 1 | Invalid/quarantined; no payment |
| witness_deleted | 0 / 1 | Invalid/quarantined; no payment |
| both_erased | 0 / 1 | Invalid/quarantined; no payment, ALL52 cases |
| witness_reversed_payload_erased | 0 / 1 | Invalid/quarantined; no payment |
| duplicate_payload | 0 / 1 | Invalid/quarantined; no payment |
| duplicate_witness | 0 / 1 | Invalid/quarantined; no payment |

The 52 paid positives must be invalid before repair, compile zero world classes
at their first repair boundary, pay one fuel, and compile the exact original at
the next boundary. All624 negative cases must remain invalid and contribute
zero world classes/one family quarantine throughout. No syntactically valid
changed body is promoted as survival or recovery. No ceiling/incomplete receipt
counts as a scientific negative or a pass. Native Seal preambles do not advance
engine step and are accounted separately from the6,552 matrix steps.

## Additional engineering controls and provenance

Cover every eligible binding for all40 payloads and all four new operations;
repeated same-ternary repair until finite fuel exhaustion; actual paid activation
of a distinct valid proposal and its next-boundary installation; legitimate
binary repair; corrupted ownership/count/type records (including record deletion,
foreign family borrowing and disappearing formerly reachable nodes); malformed
instructions and hard ceilings; bounded reads/nonmutation; old/new protocol
isolation, legacy composite counterexample preservation, runtime traces,
normal/instrumented production-loop parity and training/old-ledger rejection.

Before the matrix runs, save this frozen contract, compiled-in harness/support
and selected production/lockfile sources, exact executable and planned manifest
to a fresh output directory. Save before-attack(step0), after-attack(step1) and
after-repair(step9) HGSSv2 full-store checkpoints, preserving IDs, node property
variants, endpoint order, indexes, decode and fuel. Rust must restore and compare
the full stores and check reader nonmutation. Emit all per-step raw receipts
before outcome assertions. A separate audit checks the entire matrix, native
attack/repair graph deltas, recorded ownership/obligations, frame headers,
fuel/payment/class/quarantine/RNG timelines and checksums. It does not independently
deserialize bincode. In-memory tampering controls must reject false recovery,
free repair, hidden invalidity/ceilings and incomplete evidence.

Run in `output/ere_complete_witness_20260926`, refusing overwrite. Preserve and
disclose any failed invocations instead of replacing them. All previous
contracts/evidence remain frozen. No optimizer or held-out campaign is launched;
full semantic gates, stateability and repeated-damage ecology stay pending.
