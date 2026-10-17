# Complete-profile validated production grammar v1

Date: 2026-09-29. Implementation contract, frozen before grammar verification.
This defines a new generator, not a revision of the completed optimizer panel.
No optimizer experiment or training is authorized by grammar compilation.

## Domain and typing

Profile `E2-complete-valid-production-grammar-v1` retains the 168-parent bank,
13 coherent operators, Core/Proposal roles, complete family version 10 and
183 finite logits (168 parent + 13 operator + 2 role). The coherent kernel,
complete validator/compiler and native matcher are unchanged.

The grammar compiler deterministically enumerates the finite domain of
requests for each parent and role: each pattern/variable/slot combination,
both pattern sides, ordered variable pairs and label values 0 and 1.
Activation is enumerated for Proposal only. Pattern and variable handles use
kernel stable order; slots use kernel slot order. Requests are grouped by
operator, then role, in their documented enumeration order. No RNG, Engine,
objective, physics seed, reward, G-R1 classification or previous diagnostic
outcome is consulted in domain construction.

Each production is typed by applying the unchanged coherent transaction to
an independent parent clone and requiring `Applied`, valid complete decode
and compile, a changed decoded RuleProgram, and unchanged ordinary world.
Validation runs on metadata-isolated stores. Rejected and inapplicable/no-op
candidate requests are counted in compiler diagnostics, not sampled and
retried during emission. This is a validated finite grammar, not a claim
that unconstrained coherent edits are always valid. It does not expand to
arbitrary parents or multiple sequential edits without recompilation.

## Sampling

1. Sample parent with stable f64 softmax over all 168 parent logits.
2. Sample operator with softmax over operators having at least one legal
   production for that parent. Unavailable operators have zero probability.
3. Sample role with softmax over roles having legal productions for the
   selected parent/operator. Mode operators have Proposal support only.
4. Uniformly sample one request in that legal parent/operator/role group.

Use one ChaCha8 RNG seeded by the emission seed. Every selection consumes
one RNG draw even for singleton support. Logit differences are computed in
f64 so all finite f32 inputs, including extreme ranges, remain well defined.
Empty support or malformed theta is an error, never a parent fallback.
Masks depend only on the compiled valid grammar, not theta, fitness or seed.
Masked probabilities are renormalized; group selection is not uniform over
all productions. The new distribution must not be presented as the old
uniform-request mutation distribution.

Emission clones the selected immutable parent, applies the chosen kernel
request exactly once and verifies its decoded result against the compiled
type witness. A kernel/type mismatch fails closed; it is not resampled,
repaired or silently substituted. Emit full kernel receipt, grammar profile,
parent/operator/role support, production index/count, validated execution
signature and unchanged ordinary starting-world hash.

## Verification and boundary

Exhaustively validate legal production coverage, all parents and operators,
removed/no-op exclusions, stable enumeration and fresh-world request identity.
Test deterministic sampling, every logit group, extreme finite theta, malformed
theta, unavailable selections, known rejected-request regressions, metadata
noninterference and complete-profile zero-budget training rejection.
Export a reproducible engineering catalogue and compiler counts with source
and executed-binary hashes; verify the catalogue independently by accounting
and membership checks. Compiler transactions are not Engine steps or science
draws. All historical archives remain unchanged.

Next experiment, if requested, needs its own prospective seeds, design,
source/binary snapshot and unchanged validity/noise/traversal criteria. This
task does not rerun the optimizer panel or open training admission.
