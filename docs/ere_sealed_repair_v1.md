# Paid sealed-payload label repair v1

## Semantic contract

New opt-in profile: `E2-witness-sealed-repair-v1`. New finite instruction:
`RepairSealed`, opcode 6, instruction encoding 9. The family encoding stays 7
and the decoder is unchanged: it never reconstructs live graph material.
Existing Seal / binary Repair / WitnessActivate remain opcode 3/4/5, encoding
8 and v2 execution identity. Their rules and the old profile remain unchanged.
The old v2 and pre-witness profiles must reject opcode 6 before seeding or
quarantine it if it is encountered in an independently loaded rule-source graph.
The new profile admits only witness operations 3..6; existing training entry
points still refuse all vocabulary profiles.

RepairSealed uses the ordinary native matcher/conflict/apply path. It requires
an isolated typed instruction with finite Float fuel 1..32, a version-7 family
root, one present correctly labelled witness `[root,a,b]`, and a present
label-zero ternary payload `[a,b,root]`. It replaces the payload with its
witness-role label, preserves its arity/endpoints and the witness, and subtracts
exactly one fuel. It creates no nodes and no missing relations. The catalogue
contains fourteen productions for thirteen roles (left/right MEMBER variants).
CORE/GRAFT reuse the root variable for `a`; role endpoints retain typed guards.

Both matched edges have an explicit compiled order constraint. Ordinary rules
default to unordered matching, exactly as before. This new constraint is scoped
to the new macro, not exposed as new world grammar, and must be rejected by the
E1 CandidateRule envelope rather than silently discarded during conversion.
Malformed witness or payload order must not be canonicalized into a repair.
Work/results ceilings still stop partial enumeration without application or RNG
advance. Ordered matching has no extra search budget or RNG draws.

## Frozen bounded assay

Diagnostic: `E2-sealed-repair-controls-v1`, not a full A3 ecology or new admission
gate. Use four engine seeds 984927043585..984927043588, no retries, all thirteen
roles, and fourteen matched conditions listed below. The all-role fixture has
eight labelled variables, a single eight-arity identity edge per side and one
integer equality guard. Core and dormant proposal are identical. Select the
first binary payload of each role; a paid targeted native Seal first creates
the sealed payload. Other relation instances are covered by exhaustive binding
tests. Native targeted attacks use fixture-only endpoint markers, not repair
hooks. The original program is stored for assertions only, never supplied to
the repair operation.

Each case performs a paid Seal preamble without advancing engine step, one
native attack/control step, then eight actual GraphRuleSource repair steps with
every-step compilation, meta-visible world classes, max pairings 16 and pooled
work/results ceilings 50,000 each. There is one instruction for each of Seal,
binary Repair and RepairSealed, using finite fuel as specified below. Binary
Repair controls keep the new profile but set the new opcode's fuel to zero;
they do not alter the old v2 operation.

| Condition | Lesion | Binary / sealed repair fuel | Expected final |
| --- | --- | --- | --- |
| intact_paid | Sealed payload identity | 0 / 1 | Original; no repair payment |
| erased_paid | Sealed payload label 0 | 0 / 1 | Exact original; one sealed payment |
| erased_zero | Same label erasure | 0 / 0 | Invalid; no payment |
| erased_binary_only | Same label erasure | 1 / 0 | Invalid; no payment |
| payload_deleted | Delete sealed payload | 0 / 1 | Invalid; no payment |
| payload_reversed | Reverse role endpoints, typed payload | 0 / 1 | Invalid; no payment |
| payload_erased_reversed | Reverse role endpoints AND erase label | 0 / 1 | Invalid; no payment |
| payload_foreign_erased | Replace `b` by fresh same-label node, erase label | 0 / 1 | Invalid; no payment |
| witness_erased | Erase witness, leave sealed payload typed | 0 / 1 | Invalid; no payment |
| witness_deleted | Delete witness, leave sealed payload typed | 0 / 1 | Invalid; no payment |
| both_erased | Erase witness AND sealed payload | 0 / 1 | Invalid; no payment |
| witness_reversed_payload_erased | Reverse witness role endpoints AND erase payload | 0 / 1 | Invalid; no payment |
| duplicate_payload | Duplicate typed sealed payload | 0 / 1 | Invalid; no payment |
| duplicate_witness | Duplicate typed witness | 0 / 1 | Invalid; no payment |

4 × 13 × 14 = **728 cases**, 6,552 engine steps plus 728 paid Seal preamble
applications. No horizon expansion, selection or retries based on outcomes.
Four seeds under deterministic MaxParallel are mechanism replications, not
independent stochastic efficacy samples. Recoverable positive cases must be
invalid immediately after attack and have zero world classes at the first repair
compile; the restored class returns at the next boundary. Every negative remains
quarantined throughout. Ceilings/incomplete receipts fail the engineering assay,
not scientific evidence of no recovery.

## Evidence and additional tests

Before measurements, save the contract, compiled-in test and selected production
source snapshots, exact executable binary and planned manifest to a fresh output
directory. Save three HGSS v2 full-store checkpoints per case, before attack,
after attack and after repair. Verify wire round-trip, typed properties, IDs,
ordered endpoints, step, fuel, incidence/label indexes and decoded program.
Save raw per-step payments, validity, installed class/quarantine counts, match
work/results and RNG probes. Emit receipts before outcome assertions. An
independent audit checks the complete matrix, graph deltas, frame headers,
finite payments and checksums; it must not claim independent bincode decoding.

Engineering tests: every eligible new production binding for all forty sealed
payload instances; malformed witness/order and root alias rejection; ordinary
unordered matching unchanged; new ordered flag rejected by the E1 envelope;
finite repeated erasure of the SAME repaired ternary until fuel exhaustion;
zero/NaN/infinite/mistyped/excessive fuel, wrong opcode/version, non-isolated
instruction and unsupported arity; ceiling rejection; protocol isolation in
GraphRuleSource and EreRuntime; shared-loop measurements and no training
admission. Existing ERE, vocabulary, witness, feature-off and canonical-hash
regressions must pass. Old result files/contracts/checksums are not rewritten.

Full witness-extension G-R gates, stateability, stochastic/ecological recovery
and training remain pending regardless of the engineering assay outcome.
