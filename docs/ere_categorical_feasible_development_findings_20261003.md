# Categorical-feasibility native development pilot: complete, no improvement claim

Date: 2026-10-03. Profile `E2-categorical-feasibility-development-v1`.

## Outcome

The user-approved, fixed **2,176-evaluation** native pilot completed once. The
independent frozen receipt/math auditor passes. There are **2,172 valid outcomes,
four retained cap/nulls and zero unknown partial-work cells**. All four proposal
records and both validation panels complete. No optimizer generation, training
update, training admission or scientific support is claimed.

The mechanism produced two nonzero proposals in the zero context; the two
alternating-quarter-context proposals correctly held base because their fixed
risk panels observed no caps. Both moving proposals have **negative** fresh
validation utility point effects. The two holds remain in the fixed denominator.
This establishes bounded native integration/accounting, not better utility or
unseen-cap safety. A degenerate-alignment representation mismatch remains between
the contract prose and the reused helper; see below. Thus accounting passes,
but full prose-contract conformity is not claimed.

Contract: [development v1](ere_categorical_feasible_development_v1.md).
Prerequisite: [sealed zero-work preflight](ere_categorical_feasible_development_preflight_findings_20261003.md).
Adapter: [unchanged proposal v2](ere_categorical_feasible_v2.md).

## Fixed allocation and resource accounting

| Item | Recorded result |
| --- | ---: |
| Utility estimation | 4 × 16 = 64 evaluations, all valid |
| Risk estimation | 4 × 512 = 2,048 evaluations, four cap/nulls |
| Fresh base-only validation | 2 contexts × 8 leaves × 4 physics = 64 evaluations, all valid |
| Total terminal evaluations | 2,176 / 2,176 |
| Native attempted / advancing steps | 217,208 / 217,204 |
| Fixed maximum attempted steps | 217,600 |
| Conditional leaf reconstruction checks | 2,128 / 2,128 |
| Proposal starts / completed estimates | 4 / 4 |
| Grammar compilations | 1 |
| Grammar compile wall | 165.923560708 seconds |
| Setup plus native campaign wall | 310.98658254209823 seconds, ceiling 900 |
| Unknown partial-work / in-flight unknown | 0 / false |
| Optimizer generations / training updates | 0 / 0 |

All caps occur in risk estimation for the zero context: one in A, three in B.
Their native scores remain null, utility/validity zero, and they participate in
the full 512-draw risk denominators. Each cap records two attempts and one advance;
the 2,172 valid cells each record the full 100 advances. No cap was discarded,
replaced or retried. No additional risk draws were requested to make the no-cap
panels informative. No native proposed-arm validation physics was run.

Matching remains the unchanged pooled 50k/50k evaluator, MaxParallel
max_pairings=1, node/edge limits 2,000/4,000 and opt-in repeated-self-pair
feasibility. The deterministic leaf reconstruction checks are not independent
physics/RNG replay. All four proposals were saved before fresh validation.

## Proposal choices

All four utility panels retain their complete 16-draw denominators; all risk
panels retain 512. A/B panels have distinct scheduled emission/physics identities.
The unchanged cone, gradient floor, nine-radius ladder, actual-f32 conditions
and empirical risk-cap guard select the following outcomes.

| Context / estimate | Risk caps / 512 | Selected index | Requested radius | Actual delta norm | Observed risk-cap weight effect | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Zero / A | 1 | 0 | .1 | .1000000000907281 | −.00008729334788174577 | Nonzero proposal |
| Zero / B | 3 | 3 | .0125 | .012500000167753026 | −.000008300203842079214 | Nonzero proposal |
| Alternating quarter / A | 0 | None | None | 0 | 0 | Hold: `no_observed_caps` |
| Alternating quarter / B | 0 | None | None | 0 | 0 | Hold: `no_observed_caps` |

The zero-context estimated utility linear gains are .014776612281325903 and
.0017279093404794885; estimated validity linear gains are
.0000886190019295937 and .000008289009119510382. These are estimation arithmetic,
not measured improvement. The empirical guard reduces weight on the *observed*
risk-panel cap samples; it does not establish a population cap bound.
All ladder candidates are saved, not only the chosen ones. The slight .1-norm
rounding excess in A remains within the unchanged .100001 actual-f32 ceiling.

Zero-context A/B alignment is .2462995515019899. Both second-context deltas are
zero, so their cosine is mathematically undefined, not measured orthogonality.

## Fresh validation: point effects, not a scientific test

The saved vector order is `[base, A, B]`. Each context has eight fresh base-only
leaves, each averaged over four fixed physics seeds. Full-support importance
weights cover all 18,956 leaves in 3,232 legal groups. Ordinary complete-denominator
effects use `(U-1)*(w-1)` and `(V-1)*(w-1)`, without clipping, self-normalization,
complete-case filtering, resampling or proposal selection using validation.

| Context | A utility effect | B utility effect | A / B validity effects | Base mean utility | Base mean native score |
| --- | ---: | ---: | --- | ---: | ---: |
| Zero | −.0017311133588347096 | −.0006328730076639221 | 0 / 0 | 1.1050775773359742 | .1050775773359742 |
| Alternating quarter | 0 | 0 | 0 / 0 | 1.25 | .25 |

Neither moving proposal has a positive held-out utility point estimate in this
small panel. This is not statistical proof of harm or method inferiority: one
development block/eight leaves per context has no preregistered sign test or
scientific acceptance threshold. Do not promote the positive estimated linear
gains into a held-out improvement result.

All 64 validation cells are valid, so every validation validity residual V−1 is
zero and every corresponding importance validity effect is exactly zero. That
does **not** validate safety on unseen caps. The risk panels' sparse cap counts
and the no-cap holds demonstrate why complete integration can coexist with weak
feasibility information. The second context's zero effects follow identical
base-hold distributions, not a successful learned change.

## Contract / implementation representation discrepancy

The frozen development contract says A/B alignment is null for a zero vector.
The harness reuses `direction::alignment`, which emits `score:0.0`,
`degenerate:true`, `norm_a:0.0`, `norm_b:0.0` for the second context. The frozen
auditor reconstructs the same helper behavior and accepts it. Consequently its
accounting pass is not evidence that this prose requirement was enforced.

The two base holds, selected proposals, validation effects and fixed denominators
are unaffected. There is no alignment-based selection, sign test or scientific
gate in this fixture. Interpret the flagged score as **undefined**, not as an
orthogonal-direction measurement. Do not average it as an informative cosine.
Preserve the saved contract, native records and auditor unchanged. Before a
follow-on uses alignment, prospectively align the new contract/harness/checker
representation and add explicit zero-vector coverage; no retrospective rewriting
or silent correction of this archive.

## Immutable provenance and audit

Native archive: `output/ere_categorical_feasible_development_20261003`.
The sealed manifest is `status:complete`, `reason:null`; official summary is
measurement-complete but `scientific_support:false`, `training_admitted:false`,
`unseen_cap_feasibility_certificate:false`.

**3,347 sealed entries**, 324,034,520 bytes including the seal (below 2 GiB).
Root SHA-256:

```text
eb6c262b7fdc5299b2133f09a83609354b3d55f05a6825c27434142c55f667b7
```

The approved launch binds the source/binary-identical 3,345-entry preflight root:

```text
380519aea2ed283ea2e34afca55e80a4a136609d89c91be74fa2da6437bc9536
```

All 1,145 frozen source bindings and copied executable verify. A post-run live
check finds no drift in those bindings. Binary SHA-256:

```text
84339fbba342d78298dbe8f248c846344355ed77227b8661bf48f0a22ba25596
```

The launcher verifies sealed source registration, current historical seed
reservation, exact schedule/config/contexts, parent/runtime accounting lineage,
and a fresh zero-work frozen audit before native compilation. The final frozen
auditor independently checks complete native receipts/counters, cap utility,
phase order, exact estimation-request provenance, score gradients/baselines,
v2 proposal arithmetic, full support and fresh validation effects. Its f32
proposal reconstruction uses independently checked native f64 gradient receipts;
it does not provide independent Engine or RNG replay.

Existing preflight evidence includes four harness Rust tests, five adapter tests,
nine categorical tests, six Python groups, six launch refusals and four detached
zero-work audit controls. Those are preflight controls, not new full-native
campaign tamper tests. The final native archive adds its full frozen accounting
pass; no postmeasurement checker correction was required. Earlier failures and
sealed experiments are not overwritten, pooled into this panel or rerun.

## What this clears, and what it does not

Bounded emission/evaluator/proposal/validation integration and complete native
accounting are exercised. Observed-cap finite proposal protection operates in two
native panels, while honest no-cap holds operate in two others. The alignment
representation qualification above remains unresolved for a future stage.

Training remains unadmitted. Better utility, population cap safety, reproducible
nonzero proposals across both contexts, and multi-generation learning are not
established. Next work should first resolve degenerate-alignment semantics in a
new source stage and design a fixed-budget test of utility-signal reliability and
sparse-risk informativeness. Do not automatically repeat this pilot, increase risk
draws after seeing caps, relax no-cap holds, or start an unchanged training campaign.
Any new scientific comparison needs fresh registration, fair risk/step-selection
controls and separate compute approval.
