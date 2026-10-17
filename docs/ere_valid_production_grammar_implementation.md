# Complete-profile validated production grammar

Implemented 2026-09-29 under
[the versioned contract](/Users/kalewicker/Documents/hypersim/docs/ere_valid_production_grammar_v1.md).
Profile: `E2-complete-valid-production-grammar-v1`.

## What changed

There is now a separate grammar whose sampled domain contains only validated,
changed complete-family productions. It addresses invalid **emission** choices
in the previous optimizer diagnostic without changing the accepted coherent
kernel, validators, physics, objective, prior evidence or admission guards.

This is a finite validated-production grammar, not a proof that arbitrary
coherent edits are safe. It supports one edit on the fixed 168-parent bank.
All 13 operators remain in the domain globally, but individual parent/operator/
role combinations may be unavailable. The domain is constructed before random
sampling and never consults rewards, trajectories or the previous panel's
accepted/rejected member IDs.

## Verified result

The engineering compiler enumerated 26,624 candidate requests and retained
18,956 legal changed productions. All 168 parents have nonempty support,
ranging from 37 to 245 productions per parent. All 13 operator families remain
represented. There are 1,136 unavailable parent/operator/role groups, explicitly
masked rather than sampled and rejected. Compiler outcomes outside the legal
domain are 2,060 postcondition refusals, 144 policy refusals, 2,780 Applied
no-ops and 2,684 inapplicable requests. These are finite-domain enumeration
counts, not probabilistic mutation rates or optimizer failure denominators.

| Operator | Legal productions |
| --- | ---: |
| AddSlot | 2,752 |
| RemoveSlot | 1,200 |
| RetargetSlot | 4,644 |
| AddVar | 872 |
| RemoveVar | 672 |
| AddPatternEdge | 5,760 |
| RemovePatternEdge | 632 |
| AddLabel | 1,528 |
| RemoveLabel | 196 |
| ChangeLabel | 196 |
| ChangePriority | 336 |
| Activate | 84 |
| Deactivate | 84 |

All seven new integration tests pass, including exhaustive single-attempt
emission of every legal production, typed world equality and future ordinary
node/edge allocation continuation. The final exhaustive test invocation took
402.87 seconds in the optimized test build while other verification ran.
All 88 existing training unit tests, 11 ERE integration tests and two complete-
profile compatibility/admission tests pass. Training still rejects the complete
profile with zero optimizer/precheck/held-out steps.

The engineering export compiled the domain and checked 64 zero-logit emission
seeds in 181.607 seconds, with no Engine steps or optimizer rollouts. All 64
emissions are valid, Applied and non-fallback, and have 60 distinct executable
signatures. This sample is a generator check, not a passed P1/P2/P3/P4 panel.

The independent accounting/membership audit passes all ten tamper categories.
It verifies 433 frozen source/contract/checker hashes and the executed binary.
The completed archive has 439 files / 24,968,908 bytes, including a root seal
covering 438 entries. All 433 live/frozen source comparisons match at completion.
The fresh-directory guard was tested and refused reuse without modifying the
archive. The earlier optimizer archive's 441 sealed entries and closure
archive's 364 sealed entries remain unchanged and verify.

Archive: [manifest](/Users/kalewicker/Documents/hypersim/output/ere_valid_production_grammar_20260929/manifest.json),
[catalogue](/Users/kalewicker/Documents/hypersim/output/ere_valid_production_grammar_20260929/catalogue.json),
[accounting audit](/Users/kalewicker/Documents/hypersim/output/ere_valid_production_grammar_20260929/audit.json),
[64 emission receipts](/Users/kalewicker/Documents/hypersim/output/ere_valid_production_grammar_20260929/emission_checks.json).
Completion-seal SHA256:
`ccc22798899e426f93cd2801020fef0e93391e8790a23a214d22b625caa58891`.

Next is a separately frozen fresh optimizer-signal panel using this grammar.
Validity by construction does not establish uncapped runtime fitness, useful
reward gradients, distribution traversal, learned maintenance or admission.

## Generator architecture

```text
Compile finite request domain without physics
                  |
        168-parent softmax
                  |
    masked 13-operator softmax
                  |
       masked 2-role softmax
                  |
   uniform legal request selection
                  |
       one coherent transaction
```

The numeric layout remains 183 finite logits: parent 0..168, operator 168..181,
role 181..183. Masked probabilities are renormalized within each group; this
is deliberately not the previous uniform-selector grammar's distribution.
All parents retain nonempty support. Each operator is available somewhere;
unsupported operator or role choices do not cause a sampled failure or force
replacement of that parent. Mode changes are Proposal-only and available only
in the direction that changes the decoded program.

The compiler enumerates stable kernel pattern, variable and slot handles,
both sides, ordered variable pairs and label values 0/1. It independently
stages every candidate request with the unchanged kernel and retains only
Applied requests whose decoded RuleProgram changes and passes complete
decode/compile. It reports no-ops, inapplicable requests and refusals as
compiler-domain statistics, not optimizer samples. Stochastic emission never
tries those excluded requests, retries a rejected child, repairs it, consults
fitness or substitutes the parent.

Sampling uses ChaCha8 seeded solely by the emission seed. Softmax differences
are computed in f64, including for extreme finite f32 logits. There are three
categorical sampling calls and one uniform integer selection call. Standard
unbiased integer sampling may internally consume multiple RNG words; this
is not retrying genotype emission. The selected genotype receives exactly
one kernel transaction. Empty support, bad theta or a compiled-witness mismatch
is an error, never a fallback.

Every production stores its request and exact decoded program as a type
witness. Emission verifies Applied status, complete decode/compile, exact
witness equality and unchanged typed ordinary-world material. Receipts include
grammar profile, supported operator/role IDs, production index/count, full
kernel actions, executable/canonical signatures and single-attempt/no-fallback
flags. Canonical equivalences or dormant Proposal changes are allowed; a
changed decoded program is not automatically executable novelty or fitness.

Ordinary material comparison intentionally does not require byte-identical
**metadata allocator history** after allocating or pruning metadata. It compares
the projected typed graph including handles, data, incident edges and label
indexes. Exhaustive tests also extend both stores with ordinary nodes and an
edge, checking future ordinary allocation/material parity separately.

## Source boundaries and extension points

- [sim-core grammar](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere/valid_grammar.rs)
  owns enumeration, type witnesses, immutable parent stores and emission.
- [training adapter/exporter](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/valid_grammar.rs)
  exposes the grammar API and exports a source/binary-frozen engineering
  catalogue. It imports no Engine implementation.
- [exhaustive tests](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_valid_grammar.rs)
  validate every legal production, every available logit branch, edge cases,
  known invalid requests, deterministic sampling and allocation continuation.
- [accounting checker](/Users/kalewicker/Documents/hypersim/scripts/audit_ere_valid_grammar.py)
  verifies catalogue counts, masks, request classes, production uniqueness,
  selected-emission membership and source/binary integrity. It has ten
  temporary-copy tamper categories. It is not an independent semantic decoder
  or native physics replay.

Compile once and share the immutable grammar across optimizer evaluations:

```rust
let grammar = hg_train::ere::valid_grammar::compile()?;
let emission = grammar.emit(&theta, emission_seed)?;
```

The returned value is the existing complete-profile `signal::Emission` type,
so an authorized future runner can use the unchanged native evaluation backend.
Do not rebuild the domain for every candidate or reinterpret compiler refusals
as failed optimizer rollouts.

To add a production, extend `requests`, keep stable ordering and bounded labels/
targets, and run exhaustive validation plus the mask/receipt audit. Extending
the parent bank or logit layout requires a new profile; the current compiler
rejects a bank length other than 168. Arbitrary inner graphs, sequential edits,
paid repair, learned maintenance, matching budgets and training admission are
outside this grammar's claim. Do not weaken a validator to increase coverage.

## Commands and evidence boundary

```sh
cargo test --release -p hg-sim-core --features hg-meta,repeated-edge-feasibility --test ere_valid_grammar
cargo run --release -p hg-train --features ere-complete-signal --bin ere-train -- grammar-complete FRESH_OUTPUT_DIR
python3 scripts/audit_ere_valid_grammar.py FRESH_OUTPUT_DIR --tamper-tests --seal
```

The exporter freezes sources, contract, checker and executed binary before
compilation and records engineering-only scope with zero Engine steps and
zero optimizer rollouts. Reusing an output directory is rejected. The earlier
`signal-complete` command still uses its v1 generator; it has not silently
switched distributions. A new optimizer panel needs a new prospectively frozen
profile and seeds before physics. No such panel or training was launched here.
