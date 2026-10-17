# Feasibility-aware categorical proposal v2: aligned risk estimand

Date: 2026-10-03. Profile `E2-categorical-feasibility-proposal-v2`.
This is a disclosed offline engineering revision, not a scientific retry or an
alteration of the completed categorical-versus-ES trial. No native campaign is
authorized. The [v1 contract](ere_categorical_feasible_v1.md) remains historical;
the following risk-gradient definition supersedes its validity LOO rule.

## Preserved v1 engineering failure

The frozen first prototype at `output/ere_categorical_feasible_engineering_20261003`
has 1,145 entries, root
`316ff752f98cf0e855524585ccf8b1f8bfcfc7197ed1609f41f4b2dd43a821f2`.
Five Rust groups, feature-disabled check and independent result reconstruction
pass. Four of five Python groups pass, including ten altered-result controls;
the required nonzero-step coverage group fails because both mixed-risk cases
withhold all candidates. All-valid/all-cap cases correctly hold. Preserve this
failed manifest and its result; no native physics ran.

At zero base context, the first candidate has positive validity LOO linear gain
.002606315200 but *increases* empirical capped-leaf weight by .000141733895.
Halving the radius still increases it. The issue is not a weakened test or f32
casting: the LOO validity gradient includes random sample-mean score terms,
whereas the finite cap-weight guard differentiates a fixed-one control-variate
estimand. A direction can satisfy one and violate the other at every small step.

## Exact v2 change and justification

Let `V_i` be one for a known valid risk sample, zero for a known cap, and
`s_i = grad log P_base(leaf_i)` using the unchanged legal-selection score. Use

```text
g_V = sum_i ((V_i - 1) * s_i) / N_risk = -sum_cap(s_i) / N_risk
cap_effect(delta) = sum_cap(exp(logP_proposal - logP_base) - 1) / N_risk
```

For normalized full-support probability laws, E[score]=0. Thus the fixed constant
one preserves the expected validity gradient; it is not a reward replacement or
a fitted baseline. More importantly, `g_V` is exactly the negative derivative of
the same empirical cap effect used by the finite guard. Ordered f64 accumulation
and ordinary complete denominators are required. No self-normalization, clipping,
extra sampling or outcome-selected denominator is introduced.

Only the risk baseline changes: fixed ones replace validity LOO means. Utility
LOO estimation and its alpha-f32 .05 call remain unchanged. Output field
`risk_fixed_baselines` replaces `risk_loo_baselines`, and the profile is v2. The
six synthetic cases, labels, seeds, sample allocations and contexts remain the
same; no historical estimation/validation outcome is used. The v1 failure is a
mechanism diagnosis, not a held-out scientific result or threshold-selection run.

## Unchanged proposal and failure rules

Two explicitly tagged, fixed-denominator estimation panels have distinct IDs and
joint emission/physics seed identities across and within panels. Require known
scores/null reasons and legal selection support. A future native auditor must
prove actual stage provenance: tags cannot detect relabelled validation receipts.
Base logits are 183 finite f32 values within +/-8; never clipped. Center utility
and validity gradients within [0,168), [168,181), [181,183).

Hold base for no observed caps, all caps, validity norm <=1e-12, utility norm
<=1e-12, degenerate perpendicular or nonpositive utility objective. Otherwise use
the fixed validity cone cosine .1: keep utility's unit direction if feasible;
otherwise combine .1 times validity unit with sqrt(.99) times normalized utility
perpendicular. Every no-update estimate remains in future fixed denominators.

Save all nine finite candidates at radius `.1/2^j`, j=0..8, f32-rounded once from
base. Choose the first/largest with nonzero actual norm, norm/max coordinate
<=.100001, positive utility linear gain, validity linear gain
>=.1*||g_V||*actual_norm-1e-12, and the empirical cap effect <=0. Otherwise hold
base. Candidate selection uses only risk/utility estimation panels, not validation.
The fixed .1 margin, nine-radius ladder, floor and bounds are not changed from v1.

Observed cap-weight protection is not a certificate for unseen caps. No-cap panels
hold rather than assert safety. Risk-panel size/power, true native feasibility,
held-out utility, stochastic robustness and multi-generation learning remain
unknown. Scientific acceptance, including the *old* validity guard and verdict,
is unchanged; v2 has no native success claim.

## Verification and next boundary

Freeze v2 sources and the unchanged fixture before an offline invocation in a
fresh archive. Pin/recheck the failed v1 prototype and four historical science/
audit roots. Require the same five Rust/five Python groups, nonzero mixed-risk
coverage, all-valid/all-cap holds, independent score/baseline/gradient checks,
exact proposal f32 operations from checked f64 receipts, ten result controls and
eight binary refusals. Original 2e-12 f64 checker tolerance remains explicit;
linear/finite guards use recorded independently checked scalars, never a sign
waiver. No automatic retry follows a frozen verification failure.

Only if engineering passes should a prospectively registered native development
fixture be proposed. It needs genuinely disjoint utility/risk schedules, fixed
risk allocation with honest no-cap holds and fresh validation. Fair method
comparisons must declare equal risk-panel access/step-selection treatment or
acknowledge these differences. A/B risk panels must be independent if A/B
reproducibility is claimed. No scientific seed, native evaluation or training
update is allocated by this engineering revision.
