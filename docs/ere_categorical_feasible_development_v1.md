# Feasibility-aware categorical native development fixture v1

Date: 2026-10-03. Profile `E2-categorical-feasibility-development-v1`.
Prospective integration contract; separate approval is required for native execution.

## Fixed question and allocation

Can the unchanged [v2 adapter](ere_categorical_feasible_v2.md) be integrated with
native emission, cap accounting, proposal provenance and fresh validation?
This is one development block, not a scientific comparison or training campaign.
Two fixed 183-logit contexts are zero and alternating +.25/-.25. A/B estimates
within each context have independently seeded utility **16** and risk **512**
panels: four proposals and **2,112** estimation evaluations. All proposals and
full-support weights must be fixed before any validation. Validation samples
eight fresh base-only leaves/context with four distinct physics seeds/leaf:
**64** evaluations, **2,176** total, 100-step horizon, maximum **217,600** attempts.
The 2,128 distinct emissions each receive conditional leaf reconstruction.

The master seed is `0xE66200000000`; emission and physics domains are disjoint.
Exact IDs/order are regenerated independently in Rust/Python. Whole historical
explicit root schedule lists are reserved unless their preparation proves
unauthorized zero work. This checks listed identities, not implicit RNG streams.
No outcome-dependent replacement, extra draws, early culling, panel selection or
cap blacklist. Each panel, context and A/B remains in the fixed denominator.

## Unchanged mechanism and evaluator

Use `categorical_feasible::propose` v2 unchanged: utility LOO, fixed-one validity
gradient, centered groups, cone margin .1, floor 1e-12, nine radii .1/2^j,
actual-f32 bounds and nonpositive empirical risk-cap effect. No-cap/all-cap,
zero-gradient/conflict/no-passing-candidate outcomes retain base. A hold is a
legitimate complete outcome; the pilot need not discover caps or nonzero steps.
Saved requests derive exclusively from checked scheduled native estimation
receipts. Validation cannot enter those requests or choose a candidate.

Reuse the existing native Runner/evaluator, full receipt schema and legal
catalogue: pooled work/results 50k/50k, MaxParallel max_pairings=1,
node/edge limits 2,000/4,000 and opt-in repeated-self-pair feasibility. A native
score remains native; known cap is null with constraint utility 0 and validity 0.
Valid constraint utility is 1+score; validity 1. Unsupported/unknown outcomes
stop the fixed sequence, preserve the NotRun suffix and make official complete
summary null. In-flight unknown work blocks final accounting; never label it a cap.
Conditional leaf reconstruction is a deterministic receipt-label check, not
independent RNG or physics replay. It cannot prove every unseen metadata invariant.

## Validation and interpretation

For each context record vectors `[base, proposal_A, proposal_B]`, normalized
weights over 3,232 groups covering all 18,956 legal leaves, and A/B cosine
alignment (null for a zero vector). Native physics is evaluated only at base.
Each leaf has mean utility U and validity V over its four fixed physics draws.
Ordinary importance effects are mean `(U-1)*(w-1)` and `(V-1)*(w-1)` across all
eight leaves, including caps. No clipping/self-normalization/complete-case
filtering. Proposal physics and unseen-cap safety are **not** measured directly.
Report four cap counts, holds, chosen radii and point effects without sign tests,
power claims, success thresholds or promotion into scientific acceptance.
No ES arm, generations, optimizer update, admission or scientific support.

## Freeze, guards and resources

Freeze config, schedule, contexts, current source snapshots, launcher/checkers,
binary and lineage before invocation. New source stage is related to sealed v2
engineering by two exact additive exports and new files; in-scope prior sources
are checked, not retroactively relabelled. Concurrent changes restricted to the
excluded `evolving-minds` feature are enumerated and frozen as a distinct source
stage; they do not authorize shared ERE/evaluator drift. Preserve failed v1 and old science
archives. Parent/runtime accounting helpers remain pinned. Historical evidence
is lineage, not fitting data or authorization.

Native launch requires fresh direct `output/` child, explicit `--execute`, and a
sealed source/binary-identical zero-work preflight. Recheck live sources, exact
prepared hashes, catalogue, and independent frozen auditor before compilation.
`check-prepared` is read-only and cannot construct grammar/Engine state.
Unapproved `run-prepared` refuses before native work. No automatic retry or
in-place repair after a frozen failure. Resource failures retain incomplete
archives rather than selecting a replacement seed.

Setup/build/preflight/compile/native wall ceiling 900 seconds; per cell 30 seconds.
Output cap 2 GiB, receipt reserve 128 MiB, final reserve 16 MiB. Independent audit
ceiling 900 seconds, streaming complete native receipts into hash-bound lazy rows.
Current build/tests and zero-work preflight allocate **zero native evaluations**.
Native execution remains a separately approved next milestone.
