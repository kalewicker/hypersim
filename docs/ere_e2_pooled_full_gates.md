# E2 pooled full gate verification

Verification plan frozen 2026-09-26 before these downstream gate runs, following
the user's request to verify the gates after the 400k/50k classification pass.
No optimizer, grammar changes, four-arm research or self-maintenance campaign
is authorized by this verification task.

## Scope and receipt association

Retain E2 `AlternativeFamiliesV3`, `MatchBudgetPolicy::Pooled`, existing native
gate fixtures and thresholds. Run new full downstream receipts in
`output/ere_e2p_gates_20260926`:

- G-R2: characterize **every shipped TOML rule**, compare all supported rules for
  1,000 steps with direct versus graph-decoded execution on identical fixtures.
  Preserve the existing 2,000,000-work / 100,000-result matching budgets and
  2,000-node / 4,000-edge caps. No unsupported rule is silently dropped; report
  outside-envelope reasons and supported fraction. Any supported divergence or
  unmeasured trajectory fails this gate.
- G-R3: **100,000 steps** of static A0 versus frozen graph-source A1 identity,
  both receiving the identical inert E2 seed regions. Preserve runtime matching
  limits of 50,000 work / 50,000 results per step, SequentialRandom mode and
  seed 42. Compare every full canonical graph hash and canonical per-step CSV.
  Record actual progress, rewrite/match engagement, RNG equality and ceiling
  engagement so matching halts cannot masquerade as identity.
- G-R0/4/5/6: canonical-form equivalence/distinction pairs; all nine validator
  positive controls; **10,000 protected steps** with a live paid seeded editor;
  a deliberate executable-family change suppressing both TPM-derived EI values.
  Preserve 50,000/50,000 runtime budgets and all existing protection semantics.

Raw downstream receipts retain their correct `E2-alternatives-pooled-v1`
identity; no historical classification receipt is copied, edited or promoted.
A separate verification record will explicitly associate these full pooled
gate receipts with the previously passed
`E2-alternatives-pooled-400k50k-v1` classification receipt, whose global limits
apply to the classification assay only. The association is gate-specific, not
a claim that all experiments used 400k/50k or that failed pooled-v1 G-R1 passed.

Independently recheck classification thresholds, semantic strata, exact paired
counter changes and oracle replay using the existing read-only verifier. Recheck
downstream receipt horizons, protocol/policy identities, supported-rule coverage,
all validator controls and live-editor engagement. Preserve the original
research/training admission guard; verification is not permission to train.

## Reproduction

```sh
cargo test -p hg-sim-core --features hg-meta --test ere_native --test ere_e2 --test ere_pooled
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- gr2-e2p output/ere_e2p_gates_20260926
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- gr3-e2p output/ere_e2p_gates_20260926
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- fixtures-e2p output/ere_e2p_gates_20260926
jq -n --slurpfile gr1 output/ere_e2p_400k50k_20260926/exp_r01.json --slurpfile gr2 output/ere_e2p_gates_20260926/exp_r02.json --slurpfile gr2_header output/ere_e2p_gates_20260926/exp_r02_encoding.json --slurpfile gr3 output/ere_e2p_gates_20260926/exp_r03.json --slurpfile fixtures output/ere_e2p_gates_20260926/fixture_gates.json -f scripts/verify_ere_pooled_full_gates.jq
```

## Results

**G-R0 through G-R6 verified within the declared E2 envelope and gate-specific
resource profiles.** Classification evidence is reused unchanged from the
already completed 400k/50k trial; the other full gates were executed afresh.
This is a verification association, not a pooled-v1 G-R1 pass or training unlock.

| Gate | Result | Evidence and scope |
|---|---|---|
| G-R0 | Passed | 20 variable-renaming/order equivalences plus 20 priority distinctions; 40 canonical-form pairs |
| G-R1 | Reverified, passed at 400k/50k | 10,000 draws / 168 parents / 100 steps; zero unmeasured; original receipt and eight-case oracle replay preserved |
| G-R2 | Passed within envelope | All 45 shipped rules accounted for; two supported rules identical for 1,000 steps each; 43 explicitly outside envelope |
| G-R3 | Passed | 100,000 advancing A0/A1 steps; every full graph hash, canonical CSV and RNG probe equal; no matcher ceiling |
| G-R4 | Passed | All nine validator positive controls each quarantined exactly once |
| G-R5 | Passed for protected fixture | 10,000 advancing protected steps / 10,000 accepted rewrites / 32 paid seeded-editor events |
| G-R6 | Passed | Deliberate executable-family change marks the window unstable and suppresses both meta-included and meta-excluded EI values |

## Expressivity envelope

Supported rules are `crystal_grow` in `crystal_battery.toml` and `wolfram_like`
in `wolfram_analog.toml`. Both direct and graph-decoded trajectories engaged and
matched for the full 1,000-step horizon. There were no supported divergences or
unmeasured cases. **Coverage is only 2/45 = 4.44%**; the gate does not establish
general support for the project's rule library.

The 43 exclusions retain the existing first-blocking-reason policy:

| First blocking reason | Rules |
|---|---:|
| Distinctness constraints | 35 |
| Scheduled creation | 4 |
| Property effects | 3 |
| Enabled/weight semantics | 1 |

These counts do not imply that removing just that feature would admit every
listed rule; other blockers may exist behind the first reason. This native E2
gate is distinct from the historical `hg-selfmod` prototype's broader converter
and differential-world assay; no historical prototype coverage is imported.

## Non-vacuous identity and protection evidence

Identity completed all **100,000 actual steps in both worlds**, with 100,000
accepted rewrites and 200,000 attempted matches in the static leg. The decoded
leg's per-step counts match through exact canonical CSV comparison
(2,988,924 bytes per leg). Every full canonical graph hash and RNG probe matched;
RNG was actually consumed. Peak per-engine matching use was 24 work units and
two results; no matcher ceiling engaged. This fixture is a frozen core with an
identical dormant proposal, not a 100,000-step active-proposal learning run.

The protected fixture completed all 10,000 world steps with surviving protected
structure, zero matcher ceilings, 10,000 accepted rewrites, and 32 live paid
editor events. Peak use was 56 work units and five results. The seeded editor is
declared scaffolding; its 32 events are not proof of evolved computation or
unprotected self-maintenance. G-R6 recorded `changed_rule_set_stable = false`
and both EI fields explicitly null after the deliberate rule change. Null is
suppressed/unmeasured EI, not a measured zero.

The gate harness was instrumented to expose these progress, engagement, cap and
RNG fields. Identity now refuses zero-step or capped/non-advancing comparisons;
protection also checks actual world advancement. Engine execution, matching,
production limits and genotype semantics were not changed.

## Receipt verification and regression checks

The independent read-only verifier passed every G-R0–G-R6 check, including raw
protocol/policy identities, resource profiles, full horizons, accounting for all
45 library rules, all nine validator controls, protected progression and both
stationarity-suppressed EI fields. The original paired classification verifier
was rerun and again passed its exact eight-case counter-delta and oracle checks.

Five deliberately corrupted in-memory receipt checks were all rejected:
short identity (99,999 completed steps), a capped identity run, wrong policy,
a missing validator control, and unsuppressed EI. No source receipt was edited
for these negative checks. **37 targeted regression tests passed** (native ERE
16, semantic E2 seven, pooled ERE 11 including two new instrumentation tests,
ERE training three). `git diff --check` passed. SHA-256 comparisons confirmed
the original classification, replay and failed pooled-v1 baseline were unchanged.

Wall times excluding compilation were G-R2 **8.458 s**, G-R3 **8.144 s**, and
fixtures **0.234 s**. The three runs were launched concurrently, so these are
run receipts, not isolated performance benchmarks. Executable SHA-256:
`f6a361b0a086d3ef30f3af0941d102e9f30dd73b54be6bdc8b3044b71f862a86`.

## Artifacts and remaining work

- [Gate association and checksums](../output/ere_e2p_gates_20260926/gate_verification.json)
- [Expressivity rows](../output/ere_e2p_gates_20260926/exp_r02.json)
- [Expressivity protocol and limits](../output/ere_e2p_gates_20260926/exp_r02_encoding.json)
- [Full identity receipt](../output/ere_e2p_gates_20260926/exp_r03.json)
- [Validator/protection/stationarity receipt](../output/ere_e2p_gates_20260926/fixture_gates.json)
- [Research status](../output/ere_e2p_gates_20260926/research_status.json)
- [Read-only gate verifier](../scripts/verify_ere_pooled_full_gates.jq)
- [Preserved classification trial](ere_e2_pooled_400k50k_trial.md)

The verification association is **not integrated into the research/training
admission API**, which intentionally still refuses to substitute the larger-
budget classification receipt into pooled-v1. A future authorized campaign must
explicitly register its gate-specific resource profile; it must not relabel
failed receipts or silently broaden production budgets. No training, precheck,
held-out evaluation, four-arm research or epoch sweep ran. The reserved
20 x 16 x 10,000 optimizer budget is untouched.

The remaining substantive work is meaningful proposal variation/activation in
the production grammar, stateability checks and protected-versus-unprotected
self-maintenance controls. Passing these bounded implementation gates is not
an endogenous-learning or scientific self-maintenance result.

## Subsequent authorized work (2026-09-26)

The later [E2-aware grammar pilot](ere_e2_grammar_pilot.md) added explicit
gate-specific admission and meaningful core/proposal/activation emissions without
changing these historical receipts or the legacy campaign guard. Its 58-attempt
bounded pilot completed, but stateability and unprotected maintenance checks
failed. No full training campaign was launched. The statements above describe
the earlier gate-verification task, not the later pilot's scope or status.
