# Witness repair adversarial engineering controls v1

## Frozen scope and decision rules

Diagnostic protocol: `E2-witness-adversarial-tests-v1`. Underlying semantics remain
`E2-witness-repair-v2` (family version 7, instruction version 8). This is an
engineering test of mechanism boundaries, not a new training campaign, an A3
ecological assay, a stochastic efficacy estimate, or completion of the full
semantic gates. Do not change the production decoder or repair catalogue to make
these negative controls recover.

Use four fixed engine seeds, 984927043585 through 984927043588. Each fixture has
identical dormant core/proposal rules: eight labelled variables, one eight-arity
identity edge on each side, and an integer equality guard on variable zero.
This exercises all thirteen witness roles. Select the first binary payload of
each role in store iteration order; other instances of the same role are covered
separately by exhaustive primitive-binding tests, not by this matrix.

For each role, cross twelve native attack conditions with Repair fuel 0 or 1:

| Condition | Target / native RHS | Expected after attack | Expected after repair |
| --- | --- | --- | --- |
| no_attack | Binary payload identity, typed | Original program valid | Valid; no payment |
| binary_erase | Binary payload, label 0 | Invalid | Exact original iff fuel 1; one payment |
| binary_delete | Binary payload, no replacement edge | Invalid | Invalid; no payment |
| binary_rewire | Binary payload, endpoint order reversed, typed | Invalid | Invalid; no payment |
| witness_erase | Ternary witness, label 0 | Invalid | Invalid; no payment |
| witness_delete | Ternary witness, no replacement edge | Invalid | Invalid; no payment |
| witness_rewire | Ternary witness, role endpoints reversed | Invalid | Invalid; no payment |
| sealed_erase | Sealed ternary payload, label 0 | Invalid | Invalid; no payment |
| sealed_delete | Sealed ternary payload, no replacement edge | Invalid | Invalid; no payment |
| sealed_rewire | Sealed ternary payload, role endpoints reversed | Invalid | Invalid; no payment |
| duplicate_payload | Binary payload replaced by two identical payloads | Invalid | Invalid; no payment |
| duplicate_witness | Ternary witness replaced by two identical witnesses | Invalid | Invalid; no payment |

Endpoint reversal deliberately preserves the endpoint multiset: this probes the
ordered codec under the ordinary unordered matcher. It does not sample arbitrary
rewiring to foreign nodes. Erasure, deletion and duplication are separate
conditions; no failure attribution may be inferred from validity alone.

Total: 4 × 13 × 12 × 2 = **1,248 matrix cases**, no retries. Each performs one
native attack/control step followed by eight repair-engine steps. Sealed cases
first pay for one targeted native Seal rewrite, selected by the bounded ordinary
matcher and applied with the ordinary native apply function. All engines use MaxParallel,
max pairings 16, pooled match work/results limits 50,000 each. A ceiling binding
fails the engineering control; a partial enumeration is not a completed result.
Repair stages use the actual GraphRuleSource at every-step compile boundaries,
with witness vocabulary enabled and meta-visible world rules. Invalid families
must contribute zero world candidates; the isolated finite instructions must
remain executable. No static repair fallback or cached original body is allowed.

Unique endpoint marker properties are seeded only during fixture setup to make
the native attack rule select a known edge and orientation. They are artificial
test instrumentation, not learned instructions or runtime maintenance. Attack
and Seal engines are separate explicit stages; this is not a simultaneous
adversarial world ecology. Repair fuel is never replenished. The harness knows
the original only for equality assertions, never to supply live repair material.

## Evidence and reproducibility

Before measurements, archive this contract, the compiled-in test source and the
executed test binary when `ERE_ADVERSARIAL_OUTPUT` names a fresh output directory.
Do not overwrite an existing output directory. Without that variable, use an
isolated temporary directory, keeping ordinary test execution self-contained.

For every case, preserve three HGSS v2 / tagged-bincode full-store checkpoints:
before attack, immediately after attack, and after eight repair steps. Read each
checkpoint back, verify node and edge IDs, labels, ordered endpoints, typed
properties, step, incidence and label indexes, decoded program and fuel. Record
per-step accepted rewrites, payments, validity, match work/results, ceiling flag,
compile quarantine/world class counts and RNG probes. Compilation and all
deterministic stages must not advance engine RNG. Reader calls must not mutate
the live graph. Assert finite payments equal fuel loss and exact semantic
round-trip on the recoverable positive control.

Emit each case receipt before its outcome assertions, so a failed assertion
does not erase that case's evidence. Keep unsupported damage explicitly invalid
through every repair step, not merely at the final checkpoint. Never interpret
these engineering passes as successful general damage recovery.

Additional regressions: exhaustive Seal and Repair bindings for every binary
payload of the all-role fixture, exhaustive WitnessActivate bindings, finite
fuel exhaustion under native damage to three distinct binary payloads, damaged
witnesses withholding repair authority, isolation from another family's body,
and fail-closed match ceilings. Repair seals its target, so repeated erasure of
that same now-ternary edge is intentionally not a recoverable fuel-stress case.
Use the existing ERE/training and feature-off regressions after the new suite.
Full G-R gates, stateability and training admission remain pending.
