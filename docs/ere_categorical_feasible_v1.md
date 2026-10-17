# Feasibility-aware categorical proposal v1: offline engineering contract

Date: 2026-10-03. Profile `E2-categorical-feasibility-proposal-v1`.
This is a new arithmetic adapter, not a change to the measured comparison,
native evaluator, old categorical update, reward, optimizer or admission rule.
No native campaign or training is authorized by this document.

## Design critique and choice

The completed comparison has positive categorical utility/alignment endpoints but
fails its weighted-validity guard. A linear utility/validity projection alone is
insufficient: a utility panel with no caps gives no measured risk gradient, and
linear safety does not guarantee safety after finite f32 rounding. Neither using
exposed validation caps as a fitted blacklist nor relaxing the old guard is valid.

Use a separately designated risk-estimation panel, disjoint in receipt IDs and
every emission/physics seed identity from utility estimation. One sample per
distinct joint emission/physics pair; complete fixed denominators >=2. The API
requires `utility_estimation`/`risk_estimation` stage tags and rejects validation
tags. Tags are not provenance proof: a future native harness/auditor must bind
them to its actual preregistered schedule. This adapter cannot detect dishonest
relabeling or supply independence beyond checked explicit input identities.

Native known caps retain null score, utility zero and validity zero; valid scores
use unchanged `1+reuse` utility and validity one. Unknown/unsupported reasons,
missing draws, duplicate IDs/seeds, illegal selections and nonfinite values reject.
Do not remove failed samples or resample until a cap is seen. If risk labels are
all valid, all caps, or have gradient norm <=1e-12, retain base with an explicit
reason. These no-update estimates remain in a future campaign's denominators.

## Fixed arithmetic

Reuse the existing raw utility LOO estimator at alpha-f32 .05; do not modify it.
Compute a separate validity LOO gradient from the risk panel using the same legal
selection score and ordered f64 sums. Center both f64 gradients within the three
logit groups. Proposal direction uses these gradients directly, not the old raw
f32 update difference: this is explicitly a new caller-level intervention.

For nondegenerate centered gradients, maximize the linear utility objective over
unit directions with validity-gradient cosine >=0.1. If utility's unit direction
already satisfies this inward margin, keep it. Otherwise use the validity unit
direction with coefficient .1 plus its normalized utility-perpendicular component
with coefficient sqrt(1-.1^2). Hold base if this direction has nonpositive utility
dot product or the perpendicular is degenerate (norm <=1e-12). Pure anti-alignment
is a conflict, not permission to sacrifice utility or feasibility.

Evaluate every member of a fixed nine-radius ladder `.1 / 2^j`, j=0..8, from the
same base. Round once to f32. Record all proposals, actual deltas, norm/max step,
linear utility/validity gains and individual risk weights. Select the first/largest
candidate satisfying all of:

- Nonzero actual delta; norm and absolute coordinate step <=.100001.
- Positive centered utility-gradient dot actual delta.
- Validity gain >= .1*||validity gradient||*||actual delta|| - 1e-12.
- Ordinary fixed-denominator empirical cap-weight effect
  `sum_cap(exp(logP_new-logP_base)-1)/N_risk <= 0`, with no clipping or self-normalizing.

If none passes, retain base; never choose using validation or issue extra native
draws. Base logits must be finite and within +/-8 to preserve finite complete
categorical support; reject instead of clipping. The margin, radius ladder and
gradient floors are fixed engineering choices, not scans fitted to held-out results.
The cone margin/floor are new method parameters, not alterations of the old
scientific acceptance threshold. Candidate selection is estimation-side backtracking.

This protects observed estimation caps only. No cap evidence means no update, not
an all-valid certificate. Unseen caps, estimation noise, distribution shifts and
future horizons can still invalidate feasibility. Larger independent risk panels
may be needed; their native budget must be chosen/preregistered before launch.

## Implementation and verification scope

`hg-train::ere::categorical_feasible` is a separate `ere-complete-signal`-gated
pure arithmetic module. Its public `propose(Model, theta, utility_panel, risk_panel)`
does not invoke Engine, RNG, grammar compilation or a training loop. Its separate
binary accepts only `engineering_synthetic_only` and refuses existing result files.
No legacy guard is widened. Two additive module/Cargo exports are documented;
the old native comparison sources and all sealed archives remain untouched.

The engineering fixture uses the already sealed legal catalogue's selection
identities and explicitly synthetic parent-threshold cap labels/operator rewards.
Six cases cover two original base contexts and mixed/all-valid/all-cap risk panels,
64 utility and 512 risk observations per case. No historical estimation or validation
outcome is reused or inspected by fixture construction. These are not sampled native
programs, physics, empirical cap frequencies or scientific estimator performance.

Freeze current Rust/TOML build sources, pinned-policy Python support, and new code/
tests/contract before offline build/invocation. Existing checkout divergence from
measurement sources is recorded, not retroactively claimed identical. Run the five
Rust geometry groups, feature-disabled check, release offline binary, five independent
Python groups and ten detached result controls. Binary refusal controls cover stage,
cross-panel IDs/seeds, incomplete denominator, unknown reason, bounds, admission
purpose and overwrite; save each result. Verify source/binary hashes and historical
seals afterward. Any failure remains failed/sealed; no automatic retry or reinterpretation.

The Python checker independently reconstructs score/LOO gradients with existing
2e-12 f64 tolerance, then reconstructs f32 proposals bit-exactly from those checked
native f64 gradient receipts. It rebuilds every ladder decision using recorded,
independently checked scalar operations. This representation boundary is explicit:
it is not a blanket f32 tolerance or independent physics/RNG replay. The former
ordered-sum correction is reused from its pinned historical snapshots.

## Next milestone, not launch permission

If engineering passes, preregister a small native feasibility-proposal development
fixture: explicit independent utility/risk seeds, fixed risk-panel allocation,
zero-gradient/no-cap holds, finite step guard, full-denominator outcomes and frozen
accounting. A subsequent fresh scientific comparison needs controls with identical
risk-panel access and proposal-selection rules, or must explicitly treat their
different information/step allocation as part of the comparison. A/B estimates
must not share a risk panel if claimed independently resampled; shared method
controls within an estimate require explicit coupling. Never treat this synthetic
adapter verification as a feasibility gate or training admission.
