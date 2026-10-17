# Shared two-paid-edit learning trial V1

Prospective contract, 2026-10-04, before trial-native launch. This is a small
mechanism-learning trial, not scientific or full-training admission. It extends
the verified two-edit fixture without changing its physics, reward, vocabulary,
compiler or action interface. The original failed composition auditor remains
failed; this campaign prospectively uses the corrected topology proof V2.

## Fixed design

The adjacent JSON contract is authoritative for the numerical schedule. Three
policy seeds 993748717001/2/3 each initialize twelve shared logits to zero.
Update contexts, in fixed order, are [10,21], [10,31], [20,11], [20,31],
[30,11], [30,21]. Reserve [10,11], [20,21], [30,31] from all optimizer updates.
All nine pairs have already been exposed during engineering. This is withholding
combinations from optimization, not developer-blind or new-category evaluation.
The seeded cue normalization maps slot1's category to slot0's category row;
there are no pair-specific parameters. This tests transfer through that seeded
factorization, not autonomous grammar or semantic discovery.

For each seed: run the 48 reserved pair/action baseline cells; perform 12 complete
batches, each with two samples per six update contexts and two paired policies;
run 48 final reserved pair/action cells; then 18 correct-action control cells,
in frozen_paid/live_unpaid/live_disabled/preseeded_lure/one_fuel/reverse_paid order.
Live and initialized-reuse policies receive exactly the same two draws per
sample, but choose actions under their own current logits. Reuse reward is zero
under unchanged novelty exclusion, so reuse logits must remain exactly zero.
A frozen-uniform endpoint comparator requires no extra native cells: reweight
the same exhaustive native action panel under uniform logits. Both baseline and
final reweighting must remain identical. This is an exact policy expectation
over enumerated native outcomes, not a sampled final policy rollout.

Each slot draw uses the existing independent SplitMix64 DrawStream; two calls
per sample, never supplied to physics. Native seed = native_seeds[(cell_id/2)%2]
(integer division), pairing adjacent live/reuse physics seeds. Baseline/final
seed assignment also matches for every action. No seed replacement or retries.

Every policy update is theta + 4/12 * sum(reward * joint_score_gradient), baseline
zero, rate4 fixed, no clipping, warmup, stopping, branch selection or tuning.
Exactly 12 samples per policy batch, two per update context. Both policies use
pre-batch logits for all samples. Persist every native ledger before an update;
invalid receipts are not zero-imputed. Skip both paired updates if either batch
is incomplete; continue recording the fixed denominator, mark the trial invalid,
and do not retry. Reserved outcomes cannot enter any update or seed selection.

864 training + 288 endpoint + 54 control = 1,206 cells ×24 = 28,944 native steps;
72 policy updates total (36 live,36 reuse). Shared native deadline300 seconds,
outer Cargo timeout420 seconds. Inner work/results remain pooled50k/50k.
Full horizons, all seeds and all outcomes remain in the denominator. Native
errors/caps, incomplete evidence, or source drift invalidate accounting.

## Frozen acceptance

Accounting acceptance and learning criterion are separate. Negative learning
remains a complete, sealed trial. For **every** policy seed require:

- reserved mean expected reward improves by at least0.01 from the initial
  uniform baseline (baseline is9/384=0.0234375 in the verified fixture);
- all three reserved correct joint-action probabilities exceed1/16;
- initialized-reuse logits stay exactlyzero and its reserved endpoint is uniform;
- frozen-uniform expectation is unchanged between baseline/final native panels;
- frozen, unpaid, disabled, preseeded and one-fuel correct-action controls earn
  zero novelty reward; reverse-order correct controls earn9/24 with no early credit.

No pooled-seed rescue, threshold relaxation or further trial is authorized by a
negative outcome. This narrow criterion establishes neither unseen category
generalization nor sustained H-R1, ecological persistence or independent replay.

## Provenance and verification

Before launch freeze config, contract, complete active source closure, corrected
receipt auditor, learning auditor, tests and runner; copy snapshots before Cargo
build. Capture the executed binary. Verify the original617-entry composition
archive and corrected11-entry supplement under their exact root seals, plus
the prior758/283/187-entry learning/adapter/lifecycle archives and failed
preflight. All original active measurement pins must match except one additive
hg-train module export. No old body is edited or historical guard waived.
Disabled Evolving Minds source stays outside measurement pins; confirm Cargo
sim-core features are cognition/hg-meta before/after. Refuse active source drift.

Independent Python accounting reconstructs every native ledger using corrected
V2, exact cell schedule, paired seeds, two draw streams/states, joint probabilities,
likelihoods/gradients, updates, held-out endpoints and the unchanged decision.
Use math.fsum and fixed2e-12 arithmetic tolerance; draw/state identity is exact.
Detached tamper tests cover held-out leakage, denominators, samples, updates,
controls and endpoint/criterion corruption. Test synthetic optimizer/refusals
before launch, with no native learning work. Run one fresh archive only, retain
every failure, seal on completion/failure. No independent Engine/RNG replay claim.
