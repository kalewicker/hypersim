# Bounded resource-allocation learning proposal v1 — unlaunched

This freezes a proposed next local-task comparison, not launch authorization.
The timed bridge must pass first. A versioned runner/auditor, zero-native
arithmetic/source/schedule preflight and explicit launch approval remain
required. Do not implement or launch fitting as part of the bridge fixture.

Use four fixed learner RNG seeds, two arms (`full`, `cues_masked`), uniform zero
initial parameters,100 batch updates and all8 exposed scenario/resource states
at every update. Pair the learner draw stream between arms; freeze the RNG
implementation and exact draw hashes at runner preflight, before native work.
Consume one draw per state including unpaid controls. Native physics retains
its existing fixed seed; these are four learner replicates, not four independent
physical worlds. Neither arm is told scenario/action labels or counterfactual
utility. The full arm reads wallet/forecast/bounty; the masked arm reads wallet
only. There is no held-out scenario or generalization claim.

Freeze each batch's parameters before all8 trajectories. At live step13 choose
from the binary head and allocate at14 through sim-core. Credit only the actual
trajectory's post-resolution novel ordinary firings plus final native voucher
credits. Do not use the saved response table, optimal actions or forced teacher
actions to train. Unpaid controls retain their physical outcomes/draws but have
zero actor credit (no admissible allocation); this mask is fixed by funding,
not chosen after observing reward. No hidden warmup, seed replacement,
outcome-selected retries, adaptive curriculum or best-checkpoint selection.

For funded cells, score-function gradient is
`((actual_utility - 9.5)/20) * grad(log P(sampled_action))`.
The constant9.5 baseline is the already verified neutral funded expectation,
fixed for both arms. Sum all8 cell contributions and divide by8, clip aggregate
L2 norm at1, ascend with learning rate1, clip parameters componentwise to[-8,8].
Apply one update only after a complete admissible batch. Preserve all samples,
probabilities, rewards, gradients and resulting parameters for an independent
auditor. No fitted critic, entropy bonus or other optimizer state. These are
prospective fixed choices, not a tuning search; a negative100-step result is
retained without increasing the budget or relaxing criteria within this version.

At the final100th update only, freeze every endpoint and confirm all8 states
with the same16 midpoint draws `(i+0.5)/16`. This is deterministic action-
coverage confirmation on an exposed authored task, not independent random
holdout or independent physics replay. Fresh native trajectories supply the
utility. Run16 fixed constant-action baseline cells (4scenarios ×2resources
×2actions), retaining unpaid cells. Compare funded means equally across all4
scenarios, with4 paired learner replicates always in the denominator.

Precommitted local learning criterion: full-arm pooled funded confirmation
mean≥11.5 (at least one-third of the verified1.5 utility headroom over constant
harvest11), strictly positive pooled full-minus-masked difference, and at least
3/4 full replicates strictly beating both constant harvest11 and their paired
masked endpoint. All accounting/provenance/resource/control criteria must also
pass. Report every replicate and every failed criterion. Do not confuse this
local-task learning predicate with original ERE scientific/training admission.

Maximum work, before any launch:

- Training:4 seeds ×2arms ×100updates ×8states =6,400 trajectories /
  204,800 native advances,6,400 Seal calls and800 optimizer updates.
- Confirmation:4 ×2 ×8 ×16 =1,024 trajectories /32,768 advances,1,024 Seal.
- Constant baselines:16 trajectories /512 advances,16 Seal.
- Total:7,440 trajectories /238,080 native advances,7,440 paid nonadvancing
  Seal calls. Resource/time/output limits and prospective abort accounting must
  be finalized in the launch runner contract before approval; no uncounted
  native preflight/recheck work.

This proposal is roughly7.44% of the earlier3.2million-step research budget, but
does not assume that past budget authorization automatically launches this
new objective. Scientific/sustained/independent-replay/training-admission flags
remain false. Learned control here would be an externally trained contextual
bandit allocating authored resource tokens, not evolved endogenous cognition
or fulfillment of the original rule-encoding plan's remaining scientific gates.
