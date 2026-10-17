# Feasibility-aware categorical proposal: isolated adapter engineering complete

Date: 2026-10-03. Current profile `E2-categorical-feasibility-proposal-v2`.

## Outcome

Built an isolated feature-gated Rust adapter and offline binary, plus an independent
Python arithmetic checker. The separately frozen v2 engineering invocation passes
all five Rust geometry groups, five Python groups, ten detached result controls and
eight binary refusal controls. The feature-disabled library check also passes.
Nine existing categorical-estimator Rust groups pass in an additional targeted
live regression check; that extra result is not retroactively added to the sealed
engineering receipt. No native evaluation, grammar compilation, RNG sampling or
training was launched. No historical validation outcome was used for fitting.

The adapter protects **observed estimation caps**, not unseen caps. It provides
no native feasibility or scientific improvement result and does not admit training.
The prior comparison's weighted-validity failure and original checker failure
remain unchanged. This is an engineering mechanism ready for a separately
registered native development fixture, not a cleared scientific gate.

## Implemented mechanism

[`categorical_feasible.rs`](../crates/hg-train/src/ere/categorical_feasible.rs)
accepts an existing immutable legal-selection model, a base vector, and two
explicitly designated panels: utility estimation and risk estimation. The panels
must have complete denominators >=2, unique nonempty IDs, and disjoint emission/
physics seed identities within and across panels. Validation stage tags refuse.
Future callers/auditors must prove actual stage provenance; tags cannot detect
fraudulent relabeling of validation data.

The old utility LOO estimator remains unchanged. A separate fixed-one validity
gradient `mean((V-1)*score) = -mean_cap(score)` aligns with the derivative of the
finite cap-weight guard. Both gradients are centered within the three logit groups.
The fixed inward validity-cone margin is .1. A positive-utility direction is chosen
within that cone; otherwise base is retained. No-cap, all-cap and numerically
uninformative risk panels hold base and remain in future fixed denominators.

All nine predeclared radii `.1/2^j`, j=0..8, are computed from the same base and
rounded once to f32. The first/largest passing candidate requires bounded nonzero
actual delta, positive estimated utility gain, the inward validity linear condition,
and nonpositive ordinary empirical cap-weight effect. It uses estimation data only.
All candidates and risk weights are recorded; no outcome-based extra sampling,
complete-case filter, clipping or self-normalization occurs. If none passes, hold
base. Finite base logits outside +/-8 reject rather than clip.

The public arithmetic API is `categorical_feasible::propose`; it never creates an
Engine, compiles/samples a grammar or updates a training loop. The separate
[`ere-categorical-feasible`](../crates/hg-train/src/bin/ere-categorical-feasible.rs)
binary only accepts `engineering_synthetic_only` requests and fresh result paths.
The only existing learning-registration edits are two additive module/bin exports
in `ere.rs` and `hg-train/Cargo.toml`; no evaluator, reward, matcher, old categorical
estimator or admission implementation was changed.

## Preserved engineering revision, not a scientific retry

The frozen v1 prototype has 1,145 entries, root
`316ff752f98cf0e855524585ccf8b1f8bfcfc7197ed1609f41f4b2dd43a821f2`.
Its independent checker agrees with the implementation, but required nonzero-step
coverage fails: both mixed-risk cases withhold every candidate. Four other Python
groups, ten result controls, five Rust groups and the feature-disabled check pass.
The failed manifest/results remain intact and non-admissible.

V1 used a risk LOO gradient, which includes random sample-mean score terms; its
finite guard differentiated the fixed-one cap-weight estimand instead. For the
zero-context first candidate, v1 predicts positive validity linear gain .0026063152
yet increases empirical cap weight .0001417339. The guard correctly rejects it,
and reducing radius does not resolve the mismatch. This exposed a real mechanism
inconsistency rather than permission to weaken the coverage test or safety guard.

The separately disclosed v2 changes only the validity baseline to the fixed one
aligned with the finite estimand, the corresponding field/profile names, checker/
control references and revision documentation/lineage. The six synthetic cases,
labels, sample allocations, seeds and contexts are unchanged. Margin, ladder,
floors and guards were not tuned. The fixed-one identity follows E[score]=0 for
normalized probability laws; no exposed scientific outcome is used as a fitted
baseline or blacklist. The old science criterion and verdict are untouched.

Contracts: [v1](ere_categorical_feasible_v1.md), [current v2](ere_categorical_feasible_v2.md).

## Frozen v2 receipt and results

New archive:
[`ere_categorical_feasible_engineering_v2_20261003`](../output/ere_categorical_feasible_engineering_v2_20261003).
Its **1,156-entry** root is
`7625a3c612264ca4ee40e769e9bcb9857af885f7ac0d52ebb1eaabaae54eb861`.
It freezes **1,134 source files**, including current workspace Rust/TOML build
sources and pinned-policy/new Python/doc support. Source identity is explicitly
new-stage identity: differences from 481 historical inputs are two unrelated
Cargo dependency changes plus the two new additive learning exports. No claim
of source identity to the old measured implementation is made.

The copied release binary SHA-256 is
`c8270bb67ce33de2f09a5d2c572ec484d63381db7020b190fb81f91c2570f8ef`.
All frozen/live source and copied binary pins recheck. The failed prototype and
four science/ordered/historical audit roots verify unchanged. Elapsed engineering
time is 50.816155 seconds; archive size is 61,588,810 bytes including its seal.

The fixture uses the sealed catalogue's legal identities and explicitly synthetic
parent-threshold caps/operator rewards, not historical native outcomes. Each of
six cases supplies 64 utility and 512 risk observations. Cases reuse synthetic
identities to test counterfactual labels; they are not independent native replicates,
seed reservations or a performance sample. Mixed risk has 307 caps/512—deliberately
dense toy labels, not calibrated to the sparse native cap frequency.

| Context / synthetic risk labels | Selected ladder index | Requested radius | Actual norm | Observed cap-weight effect | Outcome |
| --- | ---: | ---: | ---: | ---: | --- |
| Zero / mixed | 5 | .003125 | .00312500000023 | −.0000586850801814 | Nonzero protected step |
| Alternating-quarter / mixed | 2 | .025 | .0250000124572 | −.000543434650528 | Nonzero protected step |
| Either context / all valid | None | None | 0 | 0 | Hold: no observed caps |
| Either context / all caps | None | None | 0 | 0 | Hold: no informative validity gradient |

The selected mixed-case utility linear gains are .000402237347984 and
.003095138924662. These are arithmetic estimates, not native observed improvement.
The nominal maximum radius .1 is not automatically used: f32 cone and finite-risk
checks shrink these steps. A future comparison must account for this changed
step-selection behavior, not claim old matched-radius equivalence.

## Checker and refusal scope

The independent checker reconstructs legal score gradients, utility LOO and fixed
validity baselines/gradients with the existing 2e-12 f64 tolerance. It then rebuilds
each f32 proposal bit-exactly from those independently checked native f64 gradient
receipts, checks scalar operations/risk weights, and reproduces the exact ladder
choice from the checked recorded scalars. This representation boundary is explicit;
it is not a blanket f32 tolerance or independent physics/RNG replay. The old
ordered-sum reference implementation is imported from pinned archived sources.

Ten copied-result controls reject changed admission/validation/certificate flags,
risk cap count, validity gradient, cone margin, selected index, output vector,
risk baseline, and an invented scientific-support field. Eight binary controls
refuse validation-tagged risk input, cross-panel ID, cross-role seed, missing draw,
unknown partial-cap reason, out-of-domain base, scientific purpose and overwrite.
The existing result bytes remain unchanged on overwrite refusal. These are
offline request/result controls, not native gate or full campaign tamper tests.

## Next bounded milestone

Preregister and build one **native development fixture**, not full training:
one block, two base contexts and independently resampled A/B utility/risk panels.
A concrete proposal is 16 utility + 512 risk emissions per estimate (four estimates)
and eight fresh base-only validation leaves/context × four physics seeds: **2,176
native evaluations**, 100-step horizon, up to 217,600 attempts. This is a proposed
budget requiring a frozen harness/auditor and launch approval, not an allocation
consumed by this task. It has no ES arm or scientific sign-test claim.

Retain no-cap/all-cap/zero-step outcomes, all caps and full denominators. Do not
resample until a risk panel sees caps or choose panel size after looking at its
results. Native emission reconstruction, actual source/metadata isolation,
known-cap handling, guard accounting and zero-work refusal need integration
verification. Native sparse-cap informativeness and practical step sizes remain
unknown. A clean engineering pilot would support implementation readiness only;
fresh scientific validation and explicit fair controls are still needed before
training or a categorical-feasibility success claim.
