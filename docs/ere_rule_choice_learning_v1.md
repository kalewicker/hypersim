# Tiny paid-edit learning trial V1 — prospective contract

Registered 2026-10-04 before new native execution. User authorization to continue
the verified adapter covers this small engineering-learning milestone only.
Original scientific/admission gates are unchanged; the reserved full training
campaign is not authorized. This contract freezes one invocation, not retries.

## Question and budget

Can a uniform contextual selector learn productive paid edits from actual native
post-birth ordinary firing, without rewarding initialized rule reuse?

Three policy runs use seeds 993748715001–993748715003. Each starts with twelve
zero f64 logits and runs exactly 12 updates, two draws per context per update
(six samples). Live-paid and preseeded-lure learners share the same prospective
uniform draws but each samples from its own current policy. Streams are disjoint
across policy runs and from physics RNG. All 432 training cells run all 24 steps.
The paired preseeded-lure learner receives the same event-derived novelty reward
interface; it is not artificially assigned zero by the optimizer.

Each policy run also executes an exhaustive 12-cell live-paid panel at baseline
and at the fixed final endpoint (three contexts × four actions). Both policies'
conditional expected rewards are weighted over the same measured outcome table.
Final controls run the matching edit in frozen-paid, live-unpaid, live-disabled,
and preseeded-lure arms for every context/run (12 cells/run). Total: 432 training
+ 108 evaluation/control cells = 540 cells = 12,960 maximum native steps.
No warmup, best checkpoint, outcome-based stopping, seed replacement, hidden
evaluation, or extra optimizer search. One shared 180-second native deadline;
fixed pooled 50k/50k and fixed 24-step native horizons.

The existing adapter's two registered physics seeds are alternated by global
cell index, restarting a fresh native fixture on every cell. These are reused
exposed deterministic worlds, not fresh physics replicates or held-out evidence.
The three replicates concern policy-sampling variability only. This design
deliberately reuses the verified native/action/reward API without altering its
module bodies, seed registry, scientific semantics, or old archive.

## Fixed arithmetic

For each context, estimate the REINFORCE gradient as the mean of its two
`(native_reward - 0) * score_gradient` terms. Apply one simultaneous
`theta_next = theta + 1.0 * gradient` update after the complete six-cell batch.
No adaptive rate, entropy bonus, LOO baseline, normalization, clipping, projection,
reward lookup, or optimizer-dependent admission. Pre-update probabilities,
likelihoods, gradients, actions, draws, rewards, and both parameter vectors are
recorded. Draws use the specified deterministic SplitMix64 stream: add
0x9e3779b97f4a7c15 modulo 2^64; mix with shifts30/27/31 and multipliers
0xbf58476d1ce4e5b9 / 0x94d049bb133111eb; `(mixed >> 11) * 2^-53`.

Each cell writes its full native ledger before optimizer updates. Rewards come
from `hg_train::ere::rule_choice::reward`, never the known reward table or action
correctness. Invalid/capped/incomplete cells retain their receipt and invalidate
the trial; a bad batch is not given zero reward and is not updated. Remaining
scheduled cells are still attempted within the shared deadline, with subsequent
invalid batch transitions explicitly skipped. No failure may become a pass by
removing a run, context, action, or control.

## Endpoints fixed before execution

Engineering-learning criterion is the conjunction of:

1. All 540 native cells complete/admissible and independently accounted; all
   scheduled batches/updates occur, with no skips or unknown work.
2. Each of three live policy runs gains at least 0.05 conditional expected reward
   from uniform baseline, computed using its exhaustive native endpoint panels.
3. Matching-edit probability strictly increases in all nine run/context pairs.
4. Every preseeded-lure training reward is zero and its parameter vectors remain
   exactly zero; its expected live-paid reward stays at baseline.
5. Every final frozen/unpaid/disabled/preseeded control has zero novelty reward.

No significance test is claimed for three runs; report individual gains and
failures. This is an exposed finite contextual bandit, not autonomous editor
evolution, unseen-context generalization, sustained H-R1 survival, population
safety, independent physics replay, or full-project training admission. Class
lifetimes remain right-censored and replacement intervals unmeasured.

## Verification and preservation

Pin the complete existing adapter archive seal before launch; retain all old
module/test/auditor bodies. Add only a versioned hg-train learning module and
its export, contract, test, runner, and independent auditor. Freeze implementation
sources and binary before native work. Python reconstructs every native ledger
using the verified earlier auditor, then independently reconstructs draws,
sampling, gradients, transitions, full denominators, endpoint expectations, and
the unchanged criterion. Rust tests check update numerics/refusals. Detached
tamper tests must reject action/draw/reward/transition/coverage/control changes.
Seal successful and failed outputs separately. Never overwrite a failed attempt.
