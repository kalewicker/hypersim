# Corrected witness-backed damage prevention and paid repair v2

**Implemented and verified for the bounded binary-world fixture.** Every
combined A3 run recovered all four original families by step21, then kept
them valid through step100. Protocol `E2-witness-repair-v2`, executed2026-09-26.
Full training and new full semantic gates remain pending.

## Why there are two versions

The first version completed its predeclared24 runs and passed the narrow
combined control. An additional all-bindings engineering test then exposed
unsafe endpoint permutations: unordered hyperedge matching could reverse a
schema role and invalidate the decoded family. That implementation is not
promoted. [Its observations and counterexample](ere_witness_repair_findings.md)
are preserved, including the executed binary and source.

Version2 adds typed endpoint guards to every native production and separates
left/right membership variants. It uses family tag7/instruction tag8, a
[new frozen contract](ere_witness_repair_v2.md), and a fresh output directory.
The same complete condition/seed matrix was precommitted and run once under
corrected semantics—no seed filtering or rewriting the v1 outcome.

**Total scientific work this turn:48 attempts /4,800 completed steps**, split
24/2,400 per protocol; no optimizer or held-out evaluations in either.

## Corrected control results

All24 v2 attempts completed100 steps with no error, retry, cap or missing capture.
Four common seeds984927043585–984927043588 produced identical reported dynamics.
Maximum matcher work13,786/results1,496, below unchanged50k/50k ceilings.

| Condition, per run | Valid-time T | Final valid | Recoveries | Seal / Repair / Activate | World proposal fires | Joint J |
|---|---|---|---|---|---|---|
| A3 neither | .5 | 2/4 | 0 | 0 /0 /1 | 0 | 0 |
| A3 prevention only | .5 | 2/4 | 0 | 32 /0 /1 | 0 | 0 |
| A3 repair only | .275 | 1/4 | 3 | 0 /54 /1 | 640 | .018528 |
| **A3 combined** | **.895** | **4/4** | **4** | **32 /32 /1** | **640** | **.241202** |
| A2 neither | 1 | 4/4 | 0 | 0 /0 /1 | 720 | .274412 |
| A2 combined | 1 | 4/4 | 0 | 32 /0 /1 | 720 | .274412 |

The seven preregistered contrast checks passed in all four seed groups:
complete measurements, all final families valid, observed recovery, paid repair,
better valid time than A3 neither, actual proposal world firings, and intact
protected references. All conditions start with the same five instruction nodes,
four-family catalogue and witness graph; only arm and initial maintenance fuel
vary. “Neither” means no Seal/Repair, not no activation: all conditions include
one WitnessActivate with fuel1.

Per combined run, the four original families had84,85,94,95 valid steps:
358/400 gives T=.895. Every family became invalid and recovered exactly once.
Initial invalid steps were1,1,13,16. All four were valid after21 and remained
valid through100;16/16 final families across the four combined runs.
The last paid maintenance action occurred at21. Spending was65 units from97:
32 Seal +32 Repair +1 activation. Repair pools retained14 and18 units;
all instruction nodes remained valid and no fuel was replenished.

The activated proposal first fired in the world at21 (640 firings per run,
2,560 total in combined A3). At advancing step11 the actual installed table
contains only the two valid cores. At21, compilation at store step20 installs
three recovered cores plus the proposal; the fourth family recovers during21.
At31 the next compilation installs all four cores plus the proposal.
Post-step family recovery is not retroactively credited to an earlier table.

Prevention-only cannot rescue already damaged material. Repair-only achieves
some recovery but does not maintain the whole bank: it ends with one valid
family and10 unused repair fuel units. Final invalid bodies fail `Members`;
no full graph/event replay was captured to pinpoint each terminal relation.
Do not equate leftover energy with sufficient repair coverage, or attribute
all unsupported damage to a single mutation from aggregate traces.

R=.2694990723045588 for combined A3; J=R*T*F=.2412016697125801.
Scores retain four-family denominators. This is a deliberately matchable,
hand-built catalogue, not evolution, discovery, learned adaptation or an EI
result. Engine RNG probes did not change during compilation or stepping: the
four seeds replicate a deterministic fixture, not four stochastic samples.

## Mechanism and implementation

Seed one explicit typed ternary witness `[root,a,b]` for each of the64
original binary schema relations. The live payload is either typed binary
`[a,b]` or typed sealed ternary `[a,b,root]`.
Extra64 witness edges are counted; the meta region has69 nodes/136 edges.

The bounded version7 reader requires exactly one correctly typed present
payload per witness. It interprets a present sealed payload in an isolated
decoding projection, then applies whole E2 validation. It never fills a
missing payload, repairs the live store, or falls back to a cached body.

Three graph-encoded version8 instruction macros use the ordinary engine:

- WitnessActivate, priority110: a graft witness permits dormant->active; pays1.
- Seal, priority100: turns an intact binary payload into a typed ternary payload;
  recreates its witness unchanged; pays1.
- Repair, priority100: a label0 binary payload with the witnessed endpoints is
  restored and sealed in one native rewrite; recreates its witness; pays1.

Fourteen Seal and fourteen Repair productions cover13 fixed schema roles
(core, graft, membership, eight slots, label, guard), with separate membership
side variants. Together with activation, there are29 native productions.
Typed endpoint labels orient every legal binding despite unordered edge
matching. Guards also enforce root/instruction versions, explicit opcode,
isolated instruction nodes and Float fuel1..32. Instruction copies have separate
finite pools; normal shared-node conflict resolution controls concurrency.

This is **engineered witness redundancy and arity protection against binary
productions**, not a general damage shield. Ternary world rules, deletion,
rewired endpoints, node/property damage and witness corruption are not covered.
Full validity is measured during actual A3 exposure; no A2-style matcher shield,
host runtime repair, static editor or infinite fuel source was added.

| Source | Extension seam |
|---|---|
| `crates/hg-meta/src/repair.rs` | Strict witnessed encoder/reader and typed native expansion; new roles/repair classes need explicit versioning and exhaustive binding controls. |
| `crates/hg-meta/src/vocabulary.rs` | Opcodes3/4/5 and version8 for the new protocol; old opcodes0/1/2 retain version4. Explicit multi-production compiler; fuel-independent code identity. |
| `crates/hg-meta/src/program.rs`, `source.rs` | Version dispatch, correct per-production attribution/table identity, independent valid instruction compilation while a whole family is quarantined. |
| `crates/hg-sim-core/src/ere.rs` | New opt-in protocol/writer, validation before seeding, max8 instructions/128 total fuel. Outer enum remains AlternativeFamiliesV3; new protocol selects actual witnessed encoding. |
| `crates/hg-train/src/ere/repair.rs` | Fixed six-condition diagnostic, full step/window capture, per-instruction fuel ledger, pure contrasts, fresh outputs and pre-evaluation snapshots. |
| `crates/hg-train/src/bin/ere-train.rs` | `repair-assay BASE_CONFIG FRESH_OUTPUT`; no optimizer/admission reuse. |
| `scripts/verify_ere_repair.jq` | Independent v2 profiles, validity/recovery, fuel, score, contrast and budget audit; `verify_ere_repair_v1.jq` preserves the old audit. |

Ordinary matcher, conflicts, apply, world productions, legacy instruction
semantics and default profiles are unchanged. Old training entry points refuse
these vocabulary configs; previous E2 gates cannot admit either new version.

## Every corrected attempt

All rows completed100/100 requested steps; none is omitted.

| Receipt | Seed | Condition | T | Final valid | Recoveries | Paid total | Proposal fires |
|---|---|---|---|---|---|---|---|
| [000](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_000.json) | 984927043585 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 |
| [001](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_001.json) | 984927043585 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 |
| [002](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_002.json) | 984927043585 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 |
| [003](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_003.json) | 984927043585 | A3_both | 0.895 | 4/4 | 4 | 65 | 640 |
| [004](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_004.json) | 984927043585 | A2_neither | 1 | 4/4 | 0 | 1 | 720 |
| [005](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_005.json) | 984927043585 | A2_both | 1 | 4/4 | 0 | 33 | 720 |
| [006](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_006.json) | 984927043586 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 |
| [007](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_007.json) | 984927043586 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 |
| [008](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_008.json) | 984927043586 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 |
| [009](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_009.json) | 984927043586 | A3_both | 0.895 | 4/4 | 4 | 65 | 640 |
| [010](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_010.json) | 984927043586 | A2_neither | 1 | 4/4 | 0 | 1 | 720 |
| [011](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_011.json) | 984927043586 | A2_both | 1 | 4/4 | 0 | 33 | 720 |
| [012](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_012.json) | 984927043587 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 |
| [013](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_013.json) | 984927043587 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 |
| [014](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_014.json) | 984927043587 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 |
| [015](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_015.json) | 984927043587 | A3_both | 0.895 | 4/4 | 4 | 65 | 640 |
| [016](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_016.json) | 984927043587 | A2_neither | 1 | 4/4 | 0 | 1 | 720 |
| [017](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_017.json) | 984927043587 | A2_both | 1 | 4/4 | 0 | 33 | 720 |
| [018](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_018.json) | 984927043588 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 |
| [019](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_019.json) | 984927043588 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 |
| [020](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_020.json) | 984927043588 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 |
| [021](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_021.json) | 984927043588 | A3_both | 0.895 | 4/4 | 4 | 65 | 640 |
| [022](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_022.json) | 984927043588 | A2_neither | 1 | 4/4 | 0 | 1 | 720 |
| [023](../output/ere_witness_repair_v2_20260926/rollouts/evaluation_023.json) | 984927043588 | A2_both | 1 | 4/4 | 0 | 33 | 720 |

[Derived summary](../output/ere_witness_repair_v2_20260926/derived_summary.json)
contains exact scores, every original family's checks/loss/recovery/final
validator, fuel and operation totals.
[Contrasts](../output/ere_witness_repair_v2_20260926/contrasts.json) and
[status](../output/ere_witness_repair_v2_20260926/status.json) preserve decisions.

## Verification and next boundary

112 targeted tests pass:21 driver/training,57 ERE simulation,20 meta library,
14 feature-off regressions. The previously failing every-Seal-binding check
now passes; exhaustive isolated repair bindings cover all13 schema roles,
including labels, guards and slots0..7, with exact semantic roundtrip,
one-unit payment and no RNG draws. Missing payload/damaged witness and malformed/
exhausted instruction controls still fail closed. Shared-loop parity, cap,
corrupt-ledger and cross-protocol controls pass. Not a full-workspace test claim.

The independent receipt audit passed all seven check groups and rejected three
in-memory tamper cases (false fuel debit, fabricated recovery, wrong family
validity). Raw receipts remain unchanged. All seven compile-time source
snapshots match current source; the saved binary matches the executed binary.
[Tests](../output/ere_witness_repair_v2_20260926/verification_tests.md),
[audit](../output/ere_witness_repair_v2_20260926/receipt_verification.json),
[corruption checks](../output/ere_witness_repair_v2_20260926/corruption_controls.json),
[provenance](../output/ere_witness_repair_v2_20260926/provenance.json),
[checksums](../output/ere_witness_repair_v2_20260926/checksums.sha256).

Executed binary SHA256:
`4c51085674193936ca9da36035581a07ca44c22cbff713e790248b463d72ef2e`.
Frozen contract SHA256:
`bde3d47d95ce408153519148bbee44f8a5c86fcaea1931ee0994258e9339fe51`.

This establishes a bounded finite prevention-plus-paid-repair mechanism under
the declared A3 binary grammar. It transitions to protected ternary payloads,
then requires no further maintenance payment after21. It is **not continual
general-purpose self-repair**. Hashes/aggregate traces are not full-graph replay
or event-level causal attribution. Scientific v1/v2 raw attempts are preserved,
and selected older persistence/vocabulary/maintenance receipts were unchanged.

Next: separately preregister adversarial witness/ternary/endpoint/deletion
damage with finite fuel and full-graph checkpoints, then verify new full semantic
gates before considering optimization. That additional campaign was not run.
Full training remains refused: zero optimizer and held-out steps.
