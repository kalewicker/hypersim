# Paired graph-aware / graph-blind learning V1

Prospective contract dated 2026-10-04, before this trial's native launch. Extend
the verified relation-sensitive fixture with one bounded learning campaign.
No native fixture, observation/reward interface, compiler, prior learner or
auditor body is changed. This is not full-training or scientific admission.

## Fixed experiment and information boundary

Three new policy seeds 993748719001/002/003 each initialize two sixteen-parameter
policies to zero. Both use four biases plus a shared 4×3 matrix, the same stable
four-way softmax, joint likelihood/score gradient, rate4, baseline0 and exactly
twelve updates. No hand-set probe, warmup, clipping, curriculum, seed selection,
early stopping, retry or tuning. Draw two independent policy uniforms per sample
from the existing SplitMix64 stream, shared between policies and never supplied
to physics. Live-paid native reward is used by **both** learners; no artificial
zero rewards or frozen optimizer are used for the blind comparator.

Graph input is the validated initial ordinary connectivity observation from
the unchanged relational interface. Blind input masks connectivity to category
presence marginals: three equally present categories per slot, normalized to
[1/3,1/3,1/3] in both slots. Both policies have sixteen trainable parameters.
They receive no receipt context, metadata, initialized core, declared active
triple, literal targets, future states or rewards as policy inputs. The constant
blind input is identical across all nine worlds. Its weights can learn a useful
marginal action distribution, but its full balanced reward cannot exceed1/24.
The mask also changes feature norm; the all-policy information ceiling, not
only an empirical comparison between different inputs, is the primary safeguard.

Build the initial ordinary observation without stepping Engine, sample under
the pre-batch parameters, and persist an immutable per-cell **decision receipt
before** invoking native physics. Verify that its observation equals the actual
native initial-inventory projection. Persist each full native receipt before
its reward enters any update. Context is used only by the fixed driver/schedule,
batch denominator and evaluation bookkeeping, never to choose logits/actions.

## Schedule and complete denominator

Per policy seed: enumerate all144 live action cells (nine worlds ×16 actions)
at baseline; run12 batches; enumerate the same144 cells at final; then18
correct-action controls on the three reserved worlds. Training contexts in order
are [10,21], [10,31], [20,11], [20,31], [30,11], [30,21], two samples each.
The three diagonals [10,11], [20,21], [30,31] never enter optimizer updates.
All worlds have been exposed in engineering; "held out" means optimizer-held-out,
not unseen by developers or an unseen category.

Each sample produces adjacent graph and blind live-paid cells, using their own
sampled action pairs under shared draws. Native seed is native_seeds[(cell_id/2)%2]
(integer division), pairing their physics seeds. Exhaustive baseline/final seed
assignment also matches per world/action. Use existing fixture seeds
993748718001/002; no new physics-seed universe is claimed. Each batch uses
unchanged pre-batch logits. Update each policy by

```text
theta_next = theta + (4/12) × sum(reward × joint_score_gradient)
```

Require exactly twelve valid samples, two per each development world, and the
correct observation mode. Refuse malformed rewards/parameters/actions, incomplete
batches, mixed modes and reserved-world samples. If either native paired batch
is incomplete, skip both updates, retain the entire fixed schedule and mark the
campaign invalid. Invalid observations/receipts are never zero-imputed or culled.

Order controls as frozen_paid, live_unpaid, live_disabled, preseeded_lure,
one_fuel, reverse_paid. Full denominator: 864 training +864 endpoint +54 control
=1,782 cells ×24 =42,768 native attempts/advances. Total72 updates (36 graph,
36 blind). Shared native deadline420 seconds; outer Cargo timeout540 seconds.
Native pooled work/results50k/50k, all payment, novelty/lifetime exclusions and
24-step horizons remain unchanged. No automatic second trial is authorized.

## Frozen criteria

Accounting completeness and learning success are separate. Negative learning
is a complete sealed outcome. For **every policy seed**, require:

- Graph-aware final mean expected reward over **all nine balanced worlds** is
  at least1/24 +0.01. No subset or pooled-seed replacement of this denominator.
- Graph-aware final balanced reward exceeds the actually trained blind final
  balanced reward by at least0.01; the latter remains at or below1/24.
- On the three optimizer-held-out diagonals, graph-aware mean expected reward
  gains at least0.01 over its uniform baseline, and every correct joint-action
  probability exceeds1/16. This secondary criterion cannot replace the primary.
- Frozen-uniform expectations and the native action matrix are unchanged from
  baseline to final. All blocking native controls earn zero novelty reward;
  correct reverse-order controls earn9/24 without early credit.

Endpoints are exact policy expectations reweighted over exhaustive native action
receipts, not sampled final-policy reward. Evaluate both policies on the same
native panel and retain the blind reserved subset even if unfavorable. Record
every seed, every action, every intermediate batch and all final parameters.
No changed rate, threshold, update count or rerun follows a negative outcome.

## Provenance and independent checks

Freeze the contract/config, active source closure, tests, runner and auditor
before launch; capture source snapshots and executed binary. Verify the643-entry
relational archive under its exact root seal, plus all older referenced archives.
All231 prior active sources must match except one additive hg-train module export.
Do not edit old bodies or waive source checks. Exclude only disabled Evolving
Minds source, verify Cargo sim-core cognition/hg-meta features before/after,
and preserve unrelated changes. Fresh-only archive; seal failures without retry.

Run zero-native optimizer/policy/sampling/refusal and independent synthetic/tamper
tests before launch. Graph-mode arithmetic must equal the previous interface,
including finite-difference joint gradients; blind inputs/gradients must be
context-independent. Independently reconstruct all native ledgers using the
unchanged relational auditor, then decision receipts, exact draws/states,
mode masks, paired seeds, probabilities, likelihoods, gradients, balanced updates,
all-nine and reserved expectations, controls and the unchanged all-seed decision.
Use math.fsum and fixed2e-12 arithmetic tolerance; draw/state identity is exact.
Repeat frozen accounting from archived source without new physics.

This trial asks whether optimization learns to use a hand-engineered relational
readout. It cannot establish autonomous vocabulary/relation discovery, a learned
graph encoder, online edit timing, population half-life, sustained H-R1,
ecological persistence or independent Engine/RNG replay. Scientific/full-training
admission remains false regardless of the learning result.
