# Raw-graph encoder: prospective engineering contract

Date: 2026-10-05. Profile `E2-graph-encoder-engineering-v1`.
Build a trainable graph encoder before a separately budgeted learning campaign.
This fixture spends **zero native steps** and makes no learned-performance claim.
Use saved initial inventories from the sealed paired relational trial with root
`a0ccb05305ecaaa151be4bb110f6235b8c375aa50b5c3f93837dddb70f5f22a8`.

## Input and architecture

The new opaque observation contains only canonical ordinary label one-hots and
undirected binary adjacency. Seven fixed known labels, sorted by label, provide
a canonical representation independent of collection order, node/edge IDs or
edge endpoint order. IDs are used only to resolve endpoints and then discarded.
All ordinary labels must be unique and present. Validate initial step0, typed
references, label0 binary ordinary edges, no loops/parallel edges, dangling
references or mixed ordinary/metadata edges. Metadata-only edges are ignored.
Accept zero through 21 ordinary edges; no BFS, shortest paths, anchor-distance
features, degree features, active-triple extraction or context validation.
Metadata/properties/incidence caches never enter policy input.

Known label vocabulary and candidate-to-edit mapping are retained scaffolding,
not learned vocabulary. Graph-blind input retains the same seven one-hots but
erases every adjacency entry. Both modes use the same 110-parameter architecture.

Let hidden width4. An affine label embedding followed by tanh has 32 parameters.
Two separate message rounds each have a learned 4×4 self matrix, learned 4×4
neighbor matrix and four biases (36 parameters per round):

```text
h0_i = tanh(E[label_i] + b0)
h(r+1)_i = tanh(h(r)_i Wself_r + sum(j adjacent i) h(r)_j Wneighbor_r + b_r)
logit(slot, none) = b_none
logit(slot, candidate) = b_edit + dot(v, h2_candidate)
```

The final shared vector and two biases have six parameters: total110. Candidate
readout is equivariant to consistent node relabeling by ID, but the fixed seven
semantic labels are not interchangeable. Softmax selects among no-edit and
three known labels per slot. Both actions are still chosen before execution.
All embeddings/message weights/readout parameters are trainable. No hard-coded
useful target, path rule, distance-to-anchor feature or successful probe weights.

Use the existing SplitMix64 stream, fixed seed993748720001, and
theta_i =0.15×(2×draw_i−1) for all110 parameters. Small nonzero initialization
avoids the dead gradient of a fully zero-initialized multilayer encoder. This
initialization is not selected by measured reward. Also test the all-zero
network's uniform output and vanishing encoder/readout gradients explicitly.

## Fixed engineering panel

Nine saved ordinary graphs in their existing order, each in three variants:
base; one added edge between the two inactive first-slot lures; a two-edge chain
over three of the four inactive lures. The active path is unchanged. The latter two
are detached synthetic observations, **not native reward receipts or unseen
generalization evidence**. Variant construction uses context outside the policy
to identify inactive nodes; context is never serialized into the opaque input.
For both graph/blind modes:54 policy records, all sixteen joint actions each,
864 likelihood/gradient/sampling records. Fixed perturbations are exposed
engineering tests only; no optimizer-held-out topology learning claim.

Require ID/order/endpoint/irrelevant-metadata invariance; all nine base inputs
distinct; blind inputs/probabilities/gradients world-independent; graph logits
change under connectivity changes at the fixed initialization. Do not require
or tune a reward-performance threshold for this untrained network. Added lure
edges need not leave its output unchanged; retain the measured sensitivity.

Rust analytic reverse-mode gradients must match centered finite differences for
all110 parameters, sixteen action pairs and both modes across the nine base
graphs (epsilon1e-6, absolute tolerance2e-8). An independent Python forward-mode
derivative implementation reconstructs every saved panel gradient with fixed
2e-12 arithmetic tolerance and performs its own finite-difference checks.

Implement balanced update/refusal support: exactly two samples from each of
the six off-diagonal development worlds; forbid diagonal/reserved samples,
unbalanced/incomplete batches, mixed modes and malformed data. Demonstrate one
small update per mode (rate0.001) on twelve synthetic arithmetic samples:
correct actions selected by the driver, detached reward9/24. This intentionally
teacher-selected update tests plumbing and gradient direction only. It is **not**
on-policy optimization, a reward-driven learning result, or new native evidence.
Require independently reconstructed update and positive mean joint likelihood
change; retain zero-reward identity and zero-init dead-gradient tests.

Reweight the untrained policies over one saved complete native action matrix
per world as descriptive archive reuse. Keep uniform and the previous fixed
learned graph/blind positive/negative controls. Verify the 1/24 blind ceiling on
all nine worlds. Do not assign the saved reward matrix to synthetic lure variants.

## Preservation and launch scope

Do not change old native, observation/reward, learner, compiler or auditor
bodies. Add a train module export only. Freeze current inputs before the fixture,
archive executed binary and verify all source pins/feature closure after it.
Historical archives retain exact seals. Since the previous trial, five files
contain unrelated feature-gated chemistry additions: Cargo.lock,
hg-engine Cargo.toml/lib.rs and hg-sim-core Cargo.toml/lib.rs. Verify those exact
additive declarations/dependency change, preserve them, and ensure
checked-transitions/constructive-chemistry are disabled. Their implementation
files are prospectively excluded with disabled Evolving Minds files; no active
physics source body drift is allowed. Record full historical differences.

Targeted tests only: no old native campaign test is invoked. Fresh-only sealed
fixture; no historical rewrites or native retry. Accounting is independent
arithmetic/observation verification, not independent Engine replay.
`training_campaign_launched:false`, `training_admitted:false`,
`scientific_acceptance:false`, `independent_physics_replay:false` stay false.
Any subsequent on-policy encoder-learning campaign needs a separate prospective
budget, initialization/seeds, controls, topology split and all-seed criteria.
