# Paid sealed-payload repair: implementation and findings

Date: 2026-09-26. Native profile `E2-witness-sealed-repair-v1`;
completed diagnostic `E2-sealed-repair-controls-v3`.

## Outcome and limits

The narrowly scoped extension is implemented: **a present sealed ternary payload
whose label was erased can be repaired, for one fuel, using its surviving
correct witness**. All 52 matched paid-positive cases recovered the exact original
decoded core, proposal and activation state. Zero fuel and the old binary-only
Repair each recovered zero of 52 otherwise identical damaged sealed payloads.
All forty payload instances also passed exhaustive eligible-binding controls.

**The original broader quarantine criterion failed.** Erasing both witness and
payload labels produced syntactically valid, changed programs in 40 of 52
composite-damage cases. These were already valid immediately after attack,
before repair. None recovered the original, paid for repair or consumed fuel.
They are not repair successes. The version-7 decoder was deliberately not
changed to conceal this inherited completeness boundary.

The completed characterization run contains 728 cases, 6,552 advancing engine
steps, 728 separate paid Seal preamble applications and 2,184 verified full-store
checkpoints. Its receipt audit passes, including thirteen tampering-rejection
controls. The narrow paid-mechanism flag is true; `all_negative_quarantined`
and `original_broader_gate_pass` are false. Full semantic gates, stateability,
ecological recovery and training admission remain pending. No new training
campaign or held-out evaluation was launched.

## Representation and implementation

The new operation is `RepairSealed`, opcode **6**, instruction encoding **9**.
Family encoding remains **7**. Existing Seal, binary Repair and WitnessActivate
keep opcodes 3/4/5, encoding 8 and their v2 execution identities. The new profile
admits witness operations 3..6, not pre-witness operations 0..2. Old runtime
profiles reject opcode 6 before seeding; old graph rule sources quarantine it.

The native production requires an isolated typed instruction with finite Float
fuel 1..32, a version-7 family root, a correctly labelled witness
`[root,a,b]` and a present label-zero payload `[a,b,root]`. Its RHS recreates
the typed witness and typed ternary on the same nodes, debiting exactly one
fuel through the ordinary property-effect/apply path. It creates no nodes,
supplies no deleted relation and never receives the assertion's stored original
program. Native rewrites allocate replacement edge IDs; endpoint order and
decoded semantics, rather than identity of the removed edge IDs, are preserved.

The finite catalogue contains fourteen productions for thirteen roles, because
MEMBER has left- and right-side variants. Typed endpoint guards orient each role.
CORE/GRAFT explicitly alias `a` to the same `root` variable: another same-labelled
root cannot supply authority through the ordinary non-injective matcher.

Both LHS edges of the new operation additionally require exact endpoint order.
This matters because a malformed sealed payload and its valid original have
the same endpoint multiset. A new compiled `PatternEdge.ordered` flag restricts
the ordinary matcher's endpoint enumeration; it does not introduce a second
matcher, hidden mutation, new work allowance or RNG draw. Raw/world rules
default to unordered matching. Existing witness macros remain unordered.
The E1 CandidateRule conversion explicitly rejects the ordered constraint,
rather than dropping it and silently changing executable meaning.

| Extension point | Responsibility |
| --- | --- |
| [Vocabulary](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/vocabulary.rs) | Opcode/name/serde enum, versioned instruction codec, execution identity and profile admission |
| [Repair catalogue](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/repair.rs) | New native production catalogue; existing family reader remains unchanged |
| [Compiled patterns](/Users/kalewicker/Documents/hypersim/crates/hg-rules/src/compiled.rs), [matcher](/Users/kalewicker/Documents/hypersim/crates/hg-engine/src/matcher.rs) | Opt-in compiled endpoint-order constraint, ordinary counted matching |
| [World envelope](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/schema.rs) | Reject ordered maintenance productions outside the E1 world envelope |
| [Rule source](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/source.rs) | Explicit `sealed_repair_enabled` switch and instruction quarantine |
| [ERE runtime](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere.rs) | Opt-in runtime profile, witness-family seeding, limits and existing metrics/traces |
| [V2 diagnostic ledger](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/repair.rs) | Refuse extension receipts in the old v2 accounting ledger |
| [Sim tests](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_sealed_repair.rs), [fixture support](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/support/witness.rs) | Native attacks, mechanism tests, matrix and full-store checkpoints |
| [Training tests](/Users/kalewicker/Documents/hypersim/crates/hg-train/tests/ere_sealed_repair.rs) | Shared-loop parity, old-ledger rejection and no optimizer admission |
| [Receipt auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_sealed_repair.py) | Independent receipt/inventory/frame audit; original-recovery and quarantine flags stay separate |

The runtime retains E2 pooled matching, explicit families, maintenance
measurement, zero family fuel and no static editor; witness profiles permit at
most eight instructions and aggregate instruction fuel 128. To enable the
extension, select the exact new vocabulary protocol and seed a
`MetaInstructionSeed { operation: MetaOperation::RepairSealed, fuel: ... }`.
Encoding the instruction alone does not upgrade an old profile. The runtime
integration test contains a complete executable configuration and paid
invalid-to-valid trace. The old bounded training diagnostic is not a new
extension driver: its v2 ledger intentionally refuses these receipts.

## Complete matched matrix

Four seeds 984927043585..984927043588 × thirteen relation roles × fourteen
conditions, with no omissions or horizon expansion. Each condition has 52 cases.
Every case first uses a paid targeted native Seal, then one native attack/control
step and eight real GraphRuleSource steps with every-step compilation.

| Condition | Cases | Valid after attack | Valid after repair | Exact original after repair | Repair payments |
| --- | ---: | ---: | ---: | ---: | ---: |
| intact_paid | 52 | 52 | 52 | 52 | 0 |
| erased_paid | 52 | 0 | 52 | 52 | 52 |
| erased_zero | 52 | 0 | 0 | 0 | 0 |
| erased_binary_only | 52 | 0 | 0 | 0 | 0 |
| payload_deleted | 52 | 0 | 0 | 0 | 0 |
| payload_reversed | 52 | 0 | 0 | 0 | 0 |
| payload_erased_reversed | 52 | 0 | 0 | 0 | 0 |
| payload_foreign_erased | 52 | 0 | 0 | 0 | 0 |
| witness_erased | 52 | 0 | 0 | 0 | 0 |
| witness_deleted | 52 | 0 | 0 | 0 | 0 |
| both_erased | 52 | 40 | 40 | 0 | 0 |
| witness_reversed_payload_erased | 52 | 0 | 0 | 0 | 0 |
| duplicate_payload | 52 | 0 | 0 | 0 | 0 |
| duplicate_witness | 52 | 0 | 0 | 0 | 0 |

The 104 exact-original final programs comprise 52 untouched controls and 52 paid
recoveries. Of 624 negative cases, 584 remain invalid and quarantined; 40 are
valid changed bodies. Four seeds under deterministic MaxParallel are mechanism
replications, not a stochastic efficacy estimate. The all-role fixture has
eight labelled variables, eight-arity identity sides and one integer equality
guard, with identical core and dormant proposal. This is not a nontrivial
ambient ecology, an adaptation experiment or a classification gate.

For each paid-positive case, the first repair boundary compiles zero world
classes and quarantines the damaged family, then applies one native repair.
The next boundary installs the restored world class. No further payment occurs.
Negative graphs remain unchanged through repair apart from advancing step;
fuel and RNG remain unchanged. The 40 changed-body cases contribute one world
class and zero family quarantines from the first repair boundary onward, but
receive no repair credit.

Pooled work/results limits were 50,000 each, with max pairings 16. Maximum
observed repair-stage work was **1,350**, maximum results **1**; no ceilings bound.
The run exited 0 in **399.06 seconds**. Test success means the revised
characterization assertions hold, not that the failed original quarantine
criterion has become true.

## Composite-damage counterexample

The unchanged reader walks the typed family region and projects present correctly
typed witness/payload pairs. When both labels are erased, a relation can vanish
from that projection. The remaining material can still describe a different
legal program. Rejecting malformed present witnesses does not guarantee that
the original complete body is still present.

Changed-body acceptance occurred for all eight SLOT roles, HASLABEL and GUARDS,
four seeds each: 40 cases. CORE, GRAFT and MEMBER composite erasures remained
invalid: twelve cases. In the first counterexample,
`984927043585_role03_both_erased`, the core shrank from eight variables to seven
and lost its guard; the proposal remained unchanged. Repair fuel stayed one.
The same post-attack decoder result persisted through all eight repair steps.

This is evidence against blanket quarantine under composite label loss, not
evidence that RepairSealed invents or repairs the changed body. No decoder
ownership/completeness repair was implemented. A stricter codec needs a separate
versioned design and controls for legitimate edits versus lost components;
using a cached original program as a repair oracle would change the model.

## All assay invocations, including failures

| Invocation | Preserved receipts | Result | Runtime |
| --- | ---: | --- | ---: |
| controls-v1, `output/ere_sealed_repair_20260926` | 8 full-horizon receipts, plus incomplete ninth case | Exit 101: stale witness ID in attack setup | 7.32 s |
| controls-v2, `output/ere_sealed_repair_v2_20260926` | 53 full-horizon receipts, including counterexample | Exit 101: real composite-damage quarantine failure | 27.47 s |
| controls-v3, `output/ere_sealed_repair_v3_20260926` | All 728 | Exit 0: complete characterization; original broader gate remains failed | 399.06 s |

Seal recreates its matched witness as well as its payload. The first harness
incorrectly retained the pre-Seal generational EdgeId; revision 2 resolved the
live witness after sealing and added a stale-ID regression. This was a harness
error, not a discarded semantic outcome. Revision 2 then exposed a real decoder
boundary. Revision 3 changed only the composite-damage characterization and
reporting contract, not production semantics, seeds, conditions, fuel or horizon.
All three frozen contracts, binaries, sources and raw outputs are retained.
The earlier 61 receipts are disclosed separately and not pooled into the final
728-case denominator. Including their recorded horizons, the three invocations
contain 789 full-horizon receipts and 7,101 advancing engine steps; the first
run's incomplete setup is additional work, not a completed case.

## Verification and provenance

198 distinct selected Rust tests passed on the implemented extension: 63 existing
ERE/adversarial nonmatrix controls, 20 hg-meta tests, 23 hg-train tests (including
two new), 70 engine/rules tests, 14 feature-off regressions and eight new sim
tests including the characterization matrix. `cargo check --workspace`, feature
checks, Rust test formatting and `git diff --check` also passed. This count does
not erase the earlier failed quarantine assertion and does not claim the entire
workspace test suite or isolated LK experiment passed. See the archived
verification notes for commands and test-development corrections.

Additional new controls cover all forty eligible native bindings; root alias
and endpoint-order rejection; repeated erasure of the same ternary with two
payments and failed third repair at fuel zero; malformed fuel/version/opcode,
nonisolated instructions and unsupported arity; budget-one truncation without
payment, mutation or RNG advance; old-profile isolation; real runtime traces;
shared production-loop parity; old-v2 ledger rejection and training refusal.

Each case saves before-attack step0, after-attack step1 and after-repair step9
HGSS v2 checkpoints. Rust restores tagged-bincode full stores and checks node
and edge IDs, typed properties, ordered endpoints, incidence multiplicity,
label indexes, decode result, fuel and reader nonmutation. The independent Python
audit checks recorded inventories/indexes, exact decoder-result comparisons,
graph deltas, payment/class/quarantine timelines, RNG, matrix completeness and
wire frame headers/lengths. It does **not** independently deserialize bincode.
Thirteen in-memory tampering attempts are rejected, including false composite
recovery, hidden changed-body counts and false quarantine promotion.

Selected compiled-in production sources/lockfile, fixture/test sources and the
exact executed matrix binary were archived before measurement. They are not a
complete standalone build tree or general simulation-resume package. RNG probes
and seeds accompany graph checkpoints in receipts. All three older witness
evidence checksum manifests and both failed extension invocation manifests
verified without altering their artifacts.

Primary evidence:

- [Completed receipt audit](/Users/kalewicker/Documents/hypersim/output/ere_sealed_repair_v3_20260926/receipt_verification.json)
- [Complete matrix summary](/Users/kalewicker/Documents/hypersim/output/ere_sealed_repair_v3_20260926/summary.json)
- [Final status and explicit failed gate](/Users/kalewicker/Documents/hypersim/output/ere_sealed_repair_v3_20260926/status.json)
- [Frozen characterization contract](/Users/kalewicker/Documents/hypersim/docs/ere_sealed_repair_v3.md)
- [Original v1 contract](/Users/kalewicker/Documents/hypersim/docs/ere_sealed_repair_v1.md)
- [Failed v2 contract](/Users/kalewicker/Documents/hypersim/docs/ere_sealed_repair_v2.md)
- [Failed quarantine invocation status](/Users/kalewicker/Documents/hypersim/output/ere_sealed_repair_v2_20260926/status.json)
- [Verification commands and notes](/Users/kalewicker/Documents/hypersim/output/ere_sealed_repair_v3_20260926/verification_tests.md)
- [Artifact checksum manifest](/Users/kalewicker/Documents/hypersim/output/ere_sealed_repair_v3_20260926/checksums.sha256)

Frozen v3 contract SHA-256:
`0becb0794c3c3208f402254943eeea2ca39fe995cf64915ed5c9239a2d64da3c`.
Compiled-in test source SHA-256:
`251f9a0179fa1ad1f0cd1b0236c5effa0b4f3df3559b02da03c59ff30794243b`.
Executed matrix binary SHA-256:
`997f0e9dd4383736f0f819eaa3f2db22bfe380147afa0d23f2504aabd8e0f40b`.

## Reproduction and next decision

Run from the project root; the output directory must not exist.

```sh
ERE_SEALED_REPAIR_OUTPUT=/private/tmp/ere-sealed-repair-new-run cargo test -p hg-sim-core --features hg-meta --test ere_sealed_repair sealed_repair_matrix -- --nocapture
python3 scripts/verify_ere_sealed_repair.py /private/tmp/ere-sealed-repair-new-run --test-rejections
cargo test -p hg-sim-core --features hg-meta --test ere_sealed_repair -- --skip sealed_repair_matrix
cargo test -p hg-train --features ere --test ere_sealed_repair
```

The next design decision is whether to version a stricter completeness/ownership
representation before ecological testing. The present extension establishes paid
label repair with surviving authority; it does not establish blanket quarantine
or recovery after that authority itself is lost. Do not admit the reserved full
training campaign on these engineering results alone.
