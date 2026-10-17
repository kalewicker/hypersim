# E2-graph-encoder-exact-gradient-v1: prospective archive-reuse diagnostic

Frozen before any measured optimizer trajectory, 2026-10-07. This is a new
offline engineering diagnostic, not a rescue or continuation of the negative
native encoder-learning trial. No Engine calls or new native attempts.

## Fixed question and budget

Does removing action-sampling variance allow the unchanged 110-parameter,
two-round, width-four raw-graph encoder to learn useful connectivity from the
saved development reward matrix? One deterministic initialized trajectory per
mode (graph and adjacency-masked); no artificial replicate seeds. Initialize
with SplitMix64 seed993748720001 and scale0.15. Perform exactly100 updates per
mode at fixed rate4. Save every before-theta, objective, full gradient and
after-theta. No clipping, backtracking, momentum, adaptive rate, retry, best
checkpoint selection, early stopping or outcome-dependent extension.

Checkpoint12 is the update-count-matched diagnostic against the prior12-update
sampled trial; checkpoint100 is a separately fixed longer-conditioning endpoint.
Both are reported separately, neither can retroactively promote the native trial.
No pooled success criterion and no selecting whichever checkpoint looks better.

## Objective and separation

Use the first policy seed's144 baseline live-paid cells (all9 worlds ×16 joint
actions), plus its18 blocking/reverse controls, byte-copied from source archive
`ere_graph_encoder_learning_trial_20261007`, seal
`232178828b8de4fc2c33e30f6a3ccc7b25d489a5deeb7aad788367a1d2e9cd5b`.
Verify exact membership, source hashes, wrapper bindings, native ledgers and raw
graph observations before optimization. Preserve all zeros; no teacher-selected
action labels, supplied graph distances or synthetic reward in the measured run.

The optimizer receives only the six ordered off-diagonal development worlds
[[10,21],[10,31],[20,11],[20,31],[30,11],[30,21]], each containing raw initial
inventory and all16 saved rewards in lexicographic action order. The three
diagonal reserved worlds are evaluation-only and never enter any gradient.

J(theta) = (1/6) sum_dev sum_a,b p0(a) p1(b) R(world,a,b).
grad J = (1/6) sum_dev sum_a,b p0(a) p1(b) R(world,a,b)
         grad log[p0(a)p1(b)].
theta_next = theta +4 grad J.

The denominator is six worlds, not96 actions or the number of positive rewards.
Joint probabilities already weight the actions. Rust uses analytic reverse-mode
score gradients; independent Python uses forward-mode differentiation of logits
and recomputes every expected gradient. Finite differences independently check
the objective derivative. Graph/masked inputs share observations and initializer.

## Fixed endpoint criteria and controls

At both checkpoints use unchanged full9-world criteria: graph expected reward
>=1/24+0.01, graph-minus-trained-mask >=0.01, mask <=1/24+2e-12, reserved mean
gain >=0.01, and every reserved correct-joint probability strictly above its
own nonuniform initial probability. All conditions and controls must pass.
No rounding at decision boundaries. Frozen initial/uniform expectations, the
archived hand-feature positive controls and blocking/reverse receipts stay
unchanged. Post-update held-out measurements cannot affect later updates.

Accounting completion is independent of a positive diagnostic endpoint.
Preflight refusal tests cover wrong world/action denominators, reserved leakage,
invalid rewards/theta/rate and gradients. Postrun tamper tests cover the measured
gradient, update, inputs, checkpoint and verdict. Preserve a failed run rather
than retrying it. Freeze sources, binary, feature closure, import provenance and
all outputs; verify historical seals before/after. Existing six explicit Finder
metadata exceptions remain exactly pinned, never broadened to a hidden-file rule.

## Claim boundary

Archive replay checks arithmetic and accounting, not independent physics.
All worlds are exposed development fixtures; the reserved split is compositional
evaluation, not unseen topology. A positive endpoint demonstrates offline
optimization only. No fresh native learning, ecological reliability, sustained
H-R1, source-identity claim, or full-training admission follows. Keep
training_admitted, scientific_acceptance, independent_physics_replay and
sustained_claim false. Preserve the historical native negative verdict.
