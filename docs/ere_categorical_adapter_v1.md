# Direct categorical adapter v1: engineering contract

Date: 2026-10-01. Profile `E2-direct-categorical-adapter-v1`.
This implements the next adapter milestone in the
[sampler mapping](ere_sampler_reward_mapping_20261001.md), not a native trial.

## Scope and fixed mathematics

The adapter is isolated in `hg-train`, behind `ere-complete-signal`. It does
not replace `OpenAiEs`, alter the native grammar/reward, grant admission or
invoke a simulator. Its catalogue model reads the existing 168-parent,
13-operator, two-role legal support and production counts without recompiling.

For a legal production `(p,o,r,k)` its probability is the product of parent,
parent-masked operator and parent/operator-masked role softmax probabilities,
divided by that group's production count. Score derivatives are one-hot minus
the corresponding conditional probability. Unsupported coordinates and
singleton-mask derivatives are zero; there is no production parameter.

At fixed finite f32 theta, calculate probabilities, log probability, score
derivatives and gradient accumulation in f64. The fixed objective is
`mean-constraint-utility-v1`: valid native score in [0,1] maps to 1+score;
only a measured `resource_cap` with null native score maps to utility zero.
Other nulls/errors/incomplete evaluations refuse estimation, never disappear.

One sample means one distinct emission seed and one already audited native
outcome. The API rejects duplicate IDs, duplicate emission seeds, partial
denominators and fewer than two samples. Repeated physics evaluations of one
emission must not masquerade as independent leave-one-out samples. Distinct
seeds alone do not prove statistical independence; a prospective caller must
establish its sampling design. No on-policy/RNG/physical validity is certified
by the pure arithmetic API.

Fixed baseline `leave-one-out-emission-mean-v1`:

```text
b_i = sum of utility over all other allocated samples / (n − 1)
gradient = (1/n) sum_i [(U_i − b_i) × score_i]
theta_after[j] = f32(f64(theta_before[j]) + f64(alpha) × gradient[j])
```

The baseline avoids self-centering bias for independent emissions. There is no
rank shaping, clipping, normalization, gauge projection, dropped cap, best-arm
selection or historical tuning. Alpha is an explicit finite nonnegative f32
argument; overflow refuses the proposal. The returned vector is an arithmetic
proposal, not an automatically applied training update. Receipts preserve raw
nullable native scores, reasons, seeds, all baseline/advantage terms, masks,
probabilities, full score gradients and actual f32 vectors.

Adding a constant to a parameter group preserves its probability law. Extreme
finite logits are normalized stably in f64; a selected categorical probability
that underflows to zero is refused rather than fabricating a finite log score.
Bit-exact finite-RNG-law differentiation is not claimed.

## Engineering checks, never native acceptance

Use exact known activation and structural-credit eligibility solely as
engineering oracles, with all 18,956 legal productions included. Verify the
enumerated score-weighted expectation gradient against central finite
differences at all 183 coordinates, in both original fixed contexts. Verify
each group's gauge sum, support coverage and probability normalization.
These oracles do not define or estimate expected native utility.

Reconstruct 16 historical base-only batches (eight blocks × two contexts,
16 distinct emission draws each, physics index zero only), plus all five
historical capped cells as individual score/utility examples. Preserve all
256 old base cells and five cap cells byte-for-byte and independently check
their historical native accounting. These are post-hoc engineering fixtures,
not new estimates of scientific benefit, fresh samples, trained champions or
replacement trials. Synthetic all-cap, equal-valid, mixed-cap and ordered
utility fixtures are separately labeled and make no physical claim.

The independent Python checker must reconstruct probability/derivative/LOO
arithmetic, compare f32 bits after serialization and reject detached mutations
of the result. Expected arithmetic tolerances: 2e−12 for scalar/vector
reconstruction, 1e−12 for gauge sums/normalization, and 2e−9 for full-catalogue
gradient-versus-central-difference comparisons at step 1e−4. These engineering
tolerances do not modify any scientific gate. The CLI refuses existing targets
before work and uses exclusive file creation against races.

Freeze source/checker/test/contract/binary hashes before the engineering CLI
invocation, bind the original sealed direction archive and pinned catalogue,
preserve failures and use a fresh output directory. Output is explicitly
engineering-only with zero native steps, zero grammar compilations, zero
training updates and `training_admitted:false`. No scientific launch command
is provided by this adapter.

## Future native comparison is separate

Before any native experiment, preregister fresh seed identities, on-policy
sampling, physics replication, all allocations, update scale, validation arms,
cap/partial treatment, resource ceilings and statistical denominator. Compare
native direction usefulness and reproducibility, not just oracle correctness.
The previous suggestion of 64 evaluations per estimate is a planning proposal,
not an authorization or completed design. No performance/variance improvement,
native expected reward or learned maintenance is established by this build.
