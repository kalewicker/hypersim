# Exact-gradient graph encoder diagnostic: findings, 2026-10-07

## Outcome

`E2-graph-encoder-exact-gradient-v1` is **complete and independently accounted**.
The precommitted update12 checkpoint is negative. The separately precommitted
update100 endpoint passes every offline diagnostic criterion. This demonstrates
that the unchanged raw-graph encoder can learn useful connectivity on the saved
fixture when given exact expected-reward gradients and the longer fixed horizon.
It does **not** establish fresh native learning or admit full training. The prior
native sampled trial remains negative.

Exactly200 updates were completed:100 graph and100 masked, one deterministic
trajectory per mode, not three invented replicate seeds. No new native attempts,
action sampling, teacher-target loss, adaptive rate, clipping, line search,
checkpoint selection, retry, early stopping or post-hoc criterion change occurred.

## Frozen design and imported evidence

The contract is [ere_graph_encoder_exact_v1.md](ere_graph_encoder_exact_v1.md);
the executable design is [ere_graph_encoder_exact_v1.json](../configs/ere_graph_encoder_exact_v1.json).
Both were frozen before the measured trajectory. The architecture is unchanged:
110 parameters, width4, learned label embeddings, two tanh message-passing rounds
with learned self/neighbor weights, and shared literal-action readout. Graph and
blind modes have the same nonzero SplitMix64 initializer, seed993748720001,
scale0.15. Blind mode zeros only ordinary adjacency.

Imported the first policy seed's complete144 baseline action cells and18 native
controls from [the negative native trial](../output/ere_graph_encoder_learning_trial_20261007/manifest.json).
Every copied byte is bound to its original sealed hash; the independent auditor
rechecks wrappers, seeds, contexts, actions, the initial raw graph, native ledgers,
paid-edit accounting and rewards. The162 imported receipts contain3,888
previously executed native steps; these are **checked, not rerun**. No source
cell was selected by reward or by the later optimization outcome.

Six development worlds are [[10,21],[10,31],[20,11],[20,31],[30,11],[30,21]].
Each includes every one of16 joint actions, including zeros. Reserved worlds
[[10,11],[20,21],[30,31]] are endpoint-only: the optimizer accepts exactly the
ordered six-world development slice, never the full evaluation matrix.
The input contains only canonical ordinary labels and undirected adjacency,
not supplied distances, context-target features or the hand-feature teacher.

For each update, optimize

```text
J = (1/6) sum_development sum_a,b p0(a) p1(b) saved_reward(world,a,b)
grad J = (1/6) sum_development sum_a,b
         p0(a) p1(b) saved_reward(world,a,b) grad log[p0(a)p1(b)]
theta_next = theta + 4 grad J
```

The denominator is6 worlds, not96 actions or the number of positives. Joint
probabilities already supply the action weighting. The measured optimizer uses
no target-selected action labels. Correct-action labels appear only in endpoint
evaluation; arbitrary synthetic dense rewards are used only in arithmetic tests.

## Registered endpoints

Initial full graph reward is0.024530558773030106; initial masked reward is
0.02460668605372995. Initial reserved graph reward is0.024530526107222873.
These actual nonuniform values, not uniform1/16 probabilities, define improvement.

| Measurement | Update12 | Update100 | Fixed requirement |
| --- | ---: | ---: | --- |
| Full9 graph expected reward | 0.033289563372574014 | 0.3744935369783478 | >=0.051666666666666666 |
| Full9 trained-mask expected reward | 0.033154307877917115 | 0.04035971813255928 | <=1/24 +2e-12 |
| Graph minus trained mask | 0.00013525549465689957 | 0.3341338188457885 | >=0.01 |
| Reserved graph expected reward | 0.03328958748108403 | 0.37449354533307316 | Evaluate all3 |
| Reserved mean gain over initialization | 0.008759061373861156 | 0.34996301922585027 | >=0.01 |
| Every reserved joint probability improves | true | true | Required |
| All controls and accounting | pass | pass | Required |
| Offline diagnostic criterion | negative | pass | Report separately |

The12-update endpoint fails graph reward, graph-minus-mask and reserved mean
gain. All100-update endpoint conditions pass without rounding. Full9 expected
reward ranges0.37448795286342507–0.3744975726498576 across individual worlds,
close to the fixture's maximum9/24=0.375; there is no pooled-subset rescue.

Reserved correct-joint probabilities at100 updates are:

| World | Actual initial probability | Final probability |
| --- | ---: | ---: |
| [10,11] | 0.06538791396018175 | 0.9986536747743401 |
| [20,21] | 0.06540299496640958 | 0.9986345409691335 |
| [30,31] | 0.06545329993119164 | 0.9986601469211119 |

Frozen-initial and uniform expectations stay unchanged. The fixed archived
hand-feature seed001 positive control stays at0.12060241227162977; its
parameters remain byte-bound to the historical trial. All three historical
positive-control parameter vectors were checked for preservation, but this
diagnostic evaluates only the prospectively fixed seed001 vector. Blocking
controls earn zero, and reverse-paid controls9/24, as before.

## Read-only explanatory diagnostics, not additional experiments

After sealing the measured result, a separately sealed zero-update supplement
evaluated two arithmetic ablations on the same saved full9 matrix. They were
not preregistered success criteria and do not change either endpoint verdict.

| Saved graph checkpoint | Normal graph | Same theta, adjacency masked | Initial encoder + final readout |
| --- | ---: | ---: | ---: |
| 12 | 0.033289563372574014 | 0.03318327910817041 | 0.03324692611385851 |
| 100 | 0.3744935369783478 | 0.012502053389687545 | 0.04455546811521324 |

At100, graph parameter-block L2 changes are embedding/bias0.86548, round1
1.47583, round2 2.15023, readout vector3.14582 and output biases1.63436.
The masked model's neighbor-weight gradient cannot use adjacency; its full
reward remains under the registered blind ceiling. The graph model's dramatic
loss under adjacency masking, and loss when encoder weights are restored,
support learned graph-dependent computation on this fixture rather than only
generic edit bias. These ablations do not prove performance on other topologies.

## Interpretation and limits

The update-count-matched exact-gradient result remains negative. Thus removing
sampling variance **alone at12 updates** was insufficient. The100-update result
shows that the existing representation is adequate for this exposed fixture
under a longer exact-gradient path. It does not isolate which part of the
sampled native failure is due to horizon, sparse positive outcomes, variance or
optimization conditioning: no100-update sampled comparison was run.

The expected-reward objective queries an already measured complete action matrix.
It is a more informative optimizer than the sparse sampled native trial, not a
matched fresh-native campaign. Both development and reserved topologies are
exposed historical fixtures; reserved means excluded from these updates, not
never seen in research. One deterministic path supplies no replicate uncertainty.
The original negative trial and all earlier negative/failed routes remain intact.

`training_admitted:false`, `full_training_authorized:false`,
`scientific_acceptance:false`, `independent_physics_replay:false` and
`sustained_claim:false` remain in force. No ecological reliability, sustained
H-R1, learned rule-emergence or broad architectural sufficiency claim is made.

## Verification and immutable artifacts

Primary archive: [manifest.json](../output/ere_graph_encoder_exact_gradient_20261007/manifest.json).
Independent results: [audit.json](../output/ere_graph_encoder_exact_gradient_20261007/audit.json).
All200 full gradients/updates: [transitions.json](../output/ere_graph_encoder_exact_gradient_20261007/transitions.json).
Endpoint records: [checkpoints.json](../output/ere_graph_encoder_exact_gradient_20261007/checkpoints.json).
Provenance: [input_provenance.json](../output/ere_graph_encoder_exact_gradient_20261007/input_provenance.json).
Frozen-source recheck and ablations: [verification.json](../output/ere_graph_encoder_exact_recheck_20261007/verification.json).

Primary lifecycle status is complete, accounting_pass=true. Two targeted Rust
groups pass (finite differences/refusals, then the fixed offline run); Python
preflight passes5 applicable groups with1 measured-receipt group skipped, and
postflight passes all6 groups. Checks include wrong world/action counts and
order, reserved leakage, nonfinite/out-of-range rewards/theta/rate, zero-reward
identity, dense-reward denominator, all110 objective derivatives in each mode
against finite differences in Rust, independent Python derivative checks,
and measured gradient/update/input/checkpoint/verdict corruption refusals.
Feature-off Cargo checking passes. The executed Rust optimizer takes1.55s;
the full import/testing/audit/preservation runner takes30.3741672039032s.

The primary archive contains445 sealed entries plus its seal,35,389,576 bytes
including the seal,259 source pins and the executed test binary. Its root seal:

```text
4fbeff3d0fdd8c1579bd709530610c4c11efd122347692c5b43e9b9c874521e5
```

Executed binary SHA256:

```text
212e43f1c2fd4cedc7540afc43a7a79159bdd94ae8bf9bda553e6c5047a72db4
```

The frozen Python auditor independently reproduces the full result in11.91s;
all259 source pins and the binary match. The two-entry recheck supplement seal:

```text
04ea838f6ca6113ab866f41b91b6f0bfc8035f04594e27c81c1cee92c502d119
```

The prior2,920-entry native archive retains its original exact root seal
`232178828b8de4fc2c33e30f6a3ccc7b25d489a5deeb7aad788367a1d2e9cd5b`.
Of its252 source pins,251 remain exact; the only change is the additive
`pub mod graph_encoder_exact;` export, whose removal exactly reproduces the
previous file. No existing encoder, native kernel or optimizer body changed.
The267-entry encoder-engineering seal and all registered older seals/entries
were checked before/after and independently rechecked. Six older archives still
have their previously disclosed, separately pinned Finder `.DS_Store` extras:
their original exact-membership checks remain false, original sealed entries
remain intact, and no new metadata waiver or historical reseal was introduced.
Disabled chemistry/Evolving Minds feature work remains outside this closure.

## Extension map and next boundary

[graph_encoder_exact.rs](../crates/hg-train/src/ere/graph_encoder_exact.rs) exposes
`World`, `objective`, `update`, `design` and `run`. The optimizer accepts only
the ordered development slice; `run` imports archived raw inventories, binds
them to the existing pure initializer for endpoint consistency, and routes
reserved worlds only to the existing evaluator. No native runner is called.
[run_ere_graph_encoder_exact.py](../scripts/run_ere_graph_encoder_exact.py)
freezes sources and provenance, validates historical preservation, executes
tests and the fixed optimizer, audits, then seals complete or failed results.
[verify_ere_graph_encoder_exact.py](../scripts/verify_ere_graph_encoder_exact.py)
uses independent forward-mode AD rather than Rust's reverse-mode gradients.
The Rust/Python test files carry synthetic and measured refusal cases.
Any change to worlds, initialization, budget, reward source or criterion requires
a new versioned contract; do not overwrite these archives or tune this endpoint.

The recommended next evidence boundary is a **prospective fresh-native transfer
confirmation panel for the frozen update100 encoder**. Compare it with the
trained masked control, actual initializer and same-theta adjacency-masked
ablation, with decisions recorded before native execution and a fixed full panel.
This would test whether the saved learned policy produces paid edits and ordinary
rule use in new native execution. It is confirmation, not a claim of online
learning. Freeze its budget, seeds, controls and failure denominator before
launch; do not enlarge training yet or redesign an encoder that now solves the
existing offline fixture. That panel has not been launched in this turn.
