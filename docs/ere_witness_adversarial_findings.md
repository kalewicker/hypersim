# Witness repair: adversarial test findings

Date: 2026-09-26. Diagnostic `E2-witness-adversarial-tests-v1`, underlying
`E2-witness-repair-v2` unchanged.

## Outcome

The bounded engineering controls pass. The repair mechanism's demonstrated
scope remains **binary payload label erasure with a surviving correct witness**.
Witness damage, sealed ternary damage, typed endpoint reversal, deletion and
duplicate relations are rejected; they are not repaired. Separate native
foreign-endpoint and schema-node deletion tests also remain invalid, even with
32 available Repair fuel. This is a correctness/safety pass, not a pass for
general structural recovery or continual maintenance.

- 1,248 preregistered matrix cases completed, no retries or match ceilings.
- 11,232 engine steps: one attack/control step plus eight repair steps per case.
- 312 additional paid native Seal applications prepared the ternary targets.
  These are direct ordinary matcher/apply applications, not advancing engine
  steps; do not add them to the completed-step count.
- 52 paid binary repairs recovered the exact original complete program.
- 104 identity/no-attack controls stayed valid without repair payments.
- All 1,092 expected-negative cases stayed invalid throughout the repair horizon.
  This includes 52 otherwise recoverable lesions with zero fuel and 1,040
  unsupported lesions.
- 3,744 full-store checkpoints were written, read back and verified in Rust.
- An independent auditor validated every receipt, graph delta and HGSS frame;
  ten in-memory tampering controls were rejected without altering the evidence.
- 119 distinct Rust tests passed across the selected suites: 112 existing tests
  and seven new tests. Feature-off build checks also passed. This is not a claim
  that every unrelated workspace or the isolated LK experiment's tests pass.

Production decoder, repair catalogue, runtime, runner and training code were
not changed. Both prior witness-repair evidence manifests still verify in full.
The archived unsafe v1 result remains superseded, not silently rewritten.

## Matrix and interpretation

Four engine seeds (984927043585–984927043588), thirteen roles, twelve conditions,
Repair fuel 0/1, fixed eight-step repair horizon. Every condition has 104 cases:
four seeds × thirteen roles × two fuel settings.

| Condition | Cases | Valid after attack | Valid after repair | Repair payments |
| --- | ---: | ---: | ---: | ---: |
| no_attack | 104 | 104 | 104 | 0 |
| binary_erase | 104 | 0 | 52 | 52 |
| binary_delete | 104 | 0 | 0 | 0 |
| binary_rewire | 104 | 0 | 0 | 0 |
| witness_erase | 104 | 0 | 0 | 0 |
| witness_delete | 104 | 0 | 0 | 0 |
| witness_rewire | 104 | 0 | 0 | 0 |
| sealed_erase | 104 | 0 | 0 | 0 |
| sealed_delete | 104 | 0 | 0 | 0 |
| sealed_rewire | 104 | 0 | 0 | 0 |
| duplicate_payload | 104 | 0 | 0 | 0 |
| duplicate_witness | 104 | 0 | 0 | 0 |

The 52/104 binary-erasure result is entirely the matched fuel contrast:
all 52 fuel-one cases recover once, while all 52 fuel-zero cases fail to recover.
It is not a measured stochastic success probability. MaxParallel selection and
these rules draw no engine RNG; the four seeds are deterministic mechanism
replications, not independent ecological samples.

The all-role fixture contains eight labelled variables, an eight-arity identity
pattern and an integer equality guard. Both core and dormant proposal are
identical. It exercises all thirteen relation roles; the matrix attacks the
first payload of each role, not every instance. The separate primitive test
checks all forty binary payload instances and all their eligible bindings.
This intentionally differs from the earlier binary-world A3 maintenance assay.
There is no nontrivial ambient world population or simultaneous adversarial
ecology here, and no inference about learned adaptation is warranted.

The matrix's `rewire` condition reverses ordered role endpoints while preserving
their multiset. The codec is ordered; the ordinary matcher is unordered. The
separate structural test actually replaces an endpoint with a fresh same-label
node, or deletes the original schema variable with its incident-edge cascade.
It covers a SLOT relation in binary, witness and sealed form, with typed or
label-zero replacements, and fuel 0 or 32: eighteen engineering fixture controls.
It does not exhaust arbitrary foreign-node rewiring over all thirteen roles.

## Installed table, quarantine, payments and resource bounds

The repair stage uses the real `GraphRuleSource`, meta-visible world rules,
witness vocabulary and every-step compile boundaries. It does not substitute a
test-only cached family or static fallback. For all unsupported lesions, the
family contributes zero world classes and one family quarantine at every repair
step. The finite instructions remain independently compilable.

In the paid positive control, the first repair step compiles an invalid family
with zero world classes, then executes one paid native Repair. The second step
compiles the recovered family and installs its one world class. It remains
valid through the horizon, with no further payment. Identity controls retain
one world class throughout. Thus validity does not mean the family was installed
before its recovery boundary.

Each positive recovery debits exactly one fuel, replaces one present binary
label-zero edge with its correctly typed sealed ternary, and recovers the exact
original decoded core/proposal/activation state. No nodes are supplied from the
assertion's stored original program. Negative cases leave the graph unchanged
during repair apart from advancing store step, leave fuel unchanged, and never
install a world-class fallback.

Engines use pooled work/results limits 50,000 each and max pairings 16.
Maximum observed repair-stage match work was **991**, maximum results **1**;
no enumerations were truncated. Attacker work/results also remained within the
limits. All recorded engine RNG probes remain unchanged within their stages;
the native Seal preamble and primitive-binding tests likewise preserve RNG.

The ceiling regression uses budget 1: no partial rewrite, payment, store-step
advance or RNG consumption is permitted. Raising the allowance on that same
damaged graph engages the real paid positive control.

## Additional regressions and test-development note

The all-role primitive control checks forty Seal bindings, forty Repair bindings
and one WitnessActivate binding. It asserts at least one tested binding for each
payload, exact decoded semantics, finite payment and unchanged RNG. Activation
changes the decoded activation bit; with identical core/proposal this fixture
does not establish a new executable world behavior. Existing distinct-proposal
tests cover that separate mechanism.

Other controls cover two payments exhausting fuel before a third binary lesion,
erased/deleted witnesses withholding repair authority, and an intact second
family failing to supply a deleted payload to the damaged first family.

An initial local fuel-stress test incorrectly expected the same repaired edge
to be repairable again. It failed because the first Repair had already sealed
the target into a ternary. The test was corrected to use distinct still-binary
payloads; ternary damage remains an explicit negative control. This happened
before the frozen matrix ran, did not change production semantics, and was not
a discarded matrix attempt. The failure and correction are recorded in the
[verification notes](/Users/kalewicker/Documents/hypersim/output/ere_witness_adversarial_20260926/verification_tests.md).

## Checkpoint and receipt evidence

Each case directory contains `receipt.json` plus HGSS v2 checkpoints before
attack (step 0), after attack (step 1), and after repair (step 9). The format uses
tagged-bincode node properties, preserving Int/Float/Bool variants. The Rust
harness restores each full store and checks node/edge IDs, labels, ordered
endpoints, typed properties, generation, energy, incidence multiplicity, label
indexes, graph step, decode result and instruction fuel. Reader calls must leave
live-store serialization unchanged.

The Python auditor independently verifies matrix completeness, per-step fuel,
payment/validity/quarantine timelines, RNG probes, native attack graph deltas,
positive repair graph deltas, unchanged negative-control graph contents, and
snapshot frame version/step/length. It checks frame bytes and hashes all saved
artifacts, but does **not** independently decode bincode. Full-store round-trip
verification is the Rust harness's evidence. Checkpoints contain graph state;
RNG probes and seeds are in receipts, not a general simulation-resume package.

Primary evidence:

- [Frozen contract](/Users/kalewicker/Documents/hypersim/docs/ere_witness_adversarial_tests_v1.md)
- [Independent receipt audit](/Users/kalewicker/Documents/hypersim/output/ere_witness_adversarial_20260926/receipt_verification.json)
- [Complete matrix summary](/Users/kalewicker/Documents/hypersim/output/ere_witness_adversarial_20260926/summary.json)
- [Artifact checksum manifest](/Users/kalewicker/Documents/hypersim/output/ere_witness_adversarial_20260926/checksums.sha256)
- [Matrix and mechanism tests](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_repair_adversarial.rs)
- [Supplemental structural test](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_repair.rs)
- [Auditor with tampering controls](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_adversarial.py)

Frozen contract SHA-256:
`7b89b98b4a00712218bd08680bd81493344f1dbeee45fe6f3b4825054425579a`.
Compiled-in matrix test source SHA-256:
`fc0e8331dd799f021a4ea9fd925128cbfa583ff48d2f09182bf5abd9749a4bc5`.
Executed matrix test binary SHA-256:
`f6cb1ff2c7807827ca04ceb9cb611c9e2f776ef7af4e9062c5cf28dc358e9adb`.
The matrix run exited 0: one test passed, five filtered, 546.89 seconds.
The structural source and separate executed structural-test binary are also
archived; they are supplemental regressions, not part of the frozen 1,248 cases.
Twelve compiled-in source/lockfile snapshots are preserved before measurements.
They are selected implementation evidence, not a complete standalone build tree.

## Reproduce or extend

Run from the project root. The output directory must not exist.

```sh
ERE_ADVERSARIAL_OUTPUT=/private/tmp/ere-adversarial-new-run cargo test -p hg-sim-core --features hg-meta --test ere_repair_adversarial native_adversarial_matrix -- --nocapture
python3 scripts/verify_ere_adversarial.py /private/tmp/ere-adversarial-new-run --test-rejections
```

Without the output variable, the matrix uses a self-cleaning temporary directory.
Routine isolated controls can skip the approximately nine-minute matrix:

```sh
cargo test -p hg-sim-core --features hg-meta --test ere_repair_adversarial -- --skip native_adversarial_matrix --nocapture
cargo test -p hg-sim-core --features hg-meta --test ere_repair
```

An archived executed matrix test binary is available for replay without rebuilding
the semantics. Give it a fresh output directory and the complete test name:

```sh
ERE_ADVERSARIAL_OUTPUT=/private/tmp/ere-adversarial-binary-replay output/ere_witness_adversarial_20260926/executed_test_binary native_adversarial_matrix_has_exact_recovery_or_persistent_quarantine --exact --nocapture
```

## What remains and recommended next step

Do not expand training based on this test pass. Full witness-profile G-R gates,
stateability and training admission remain pending. No optimizer, held-out
evaluation or full ecological campaign was run in this turn.

If the research goal remains ongoing damage recovery, the next bounded semantic
change should address **label erasure of a sealed ternary payload while its
witness survives**. That is a narrow extension of the present redundancy,
unlike reconstructing missing endpoints or witnesses. It needs a new versioned
catalogue/contract, finite-fuel and unordered-binding safety controls, and a new
matched assay. Do not silently extend the current v2 repair rule or reuse these
receipts as evidence for the changed semantics. Witness destruction remains a
separate trust/redundancy problem.
