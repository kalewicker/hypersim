# Raw-graph encoder: engineering findings and architecture

Date: 2026-10-05. Profile `E2-graph-encoder-engineering-v1`.
Status: **complete; engineering and independent arithmetic accounting passed**.
New native steps: **0**. No on-policy encoder-learning campaign was launched.

## Outcome

A trainable message-passing encoder now consumes the ordinary graph directly,
without hand-coded anchor distances or active-category indicators. It has
differentiable embeddings, two learned message rounds and a shared candidate
readout, with balanced update/refusal support. This is a working trainable
implementation, not evidence of a learned useful representation yet.

The fixed panel has 54 cases: nine worlds × three topology variants × two
observation modes. Each retains all sixteen joint actions: 864 likelihood/
gradient/sampling records. Three Rust groups and all six postfixture Python
groups pass, including sixteen detached panel corruptions. Independent
forward-mode differentiation agrees with Rust reverse-mode backpropagation.
Frozen-source accounting, feature-off checking, 245 active source pins and
all 267 archive seal entries verify.

At the fixed initialization all nine base graphs produce different graph
probabilities. Blind inputs/probabilities/gradients are world-independent.
Added disconnected lure edges change graph probabilities by at most
0.000513395. This sensitivity is retained, not called invariance or successful
generalization. Reweighting untrained policies over the saved native matrix:

| Policy / bound | Full balanced nine-world reward | Scope |
| --- | ---: | --- |
| Uniform | 0.023437500 | Reference |
| New raw-graph encoder, untrained | 0.024530559 | Archive reuse, not learned performance |
| Same encoder, adjacency masked, untrained | 0.024606686 | Information-blind comparator |
| Any graph-blind joint policy ceiling | 0.041666667 | Bound on this measured task matrix |
| Prior trained hand-feature graph policies | 0.065215764–0.158716909 | Separate sealed campaign, unchanged |

The untrained graph encoder is slightly below its masked comparator and does
not exceed the ceiling. No learning-performance criterion was applied to its
initial weights, and this is not a new relation-learning success.

## Architecture and parameter layout

An opaque `Observation` contains seven sorted ordinary semantic labels and
7×7 symmetric Boolean adjacency. Labels index an implicit one-hot embedding
lookup. Canonicalization removes collection order, node/edge IDs and endpoint
order; it is not invariance to changing semantic labels or action vocabulary.

| Stage | Parameter indices | Count | Computation |
| --- | --- | ---: | --- |
| Label embedding and bias | 0–31 | 32 | Seven 4-vectors plus shared bias; tanh |
| Message round 1 | 32–67 | 36 | Self/neighbor 4×4 matrices and four biases; tanh |
| Message round 2 | 68–103 | 36 | Separate self/neighbor matrices and biases; tanh |
| Shared candidate readout | 104–107 | 4 | Dot product with candidate hidden state |
| No-edit/common-edit biases | 108–109 | 2 | Four-way logits per slot |
| Total | 0–109 | 110 | Trainable architecture |

For each ordinary node i, hidden width4:

```text
h0_i = tanh(E[label_i] + b0)
h(r+1)_i = tanh(h(r)_i Wself_r + sum(j adjacent i) h(r)_j Wneighbor_r + b_r)
logit(slot, none) = b_none
logit(slot, candidate) = b_edit + dot(v, h2_candidate)
```

Two rounds give a two-hop receptive field without BFS, shortest paths, degree
features or special anchor-distance extraction. Useful embeddings/messages/
hidden directions are parameterized. Candidate scoring still uses known
literal-label-to-edit mapping. Both actions are chosen before native execution,
and edit timing remains seeded.

The 110-parameter blind comparator retains label inputs but erases adjacency.
Neighbor weights cannot receive gradients without edges, so equal architectural
parameter count does not imply equal effective capacity. The full-panel 1/24
bound covers any world-independent joint policy and remains the main safeguard.

Initialization uses fixed SplitMix64 seed993748720001 and
0.15×(2×draw−1) for every parameter, without reward-dependent selection. Fully
zero parameters produce uniform output but zero embedding/message/readout-vector
gradients; only output biases can move. Tests demonstrate this failure. Small
nonzero initialization is explicit architecture, not a warmed-up successful policy.

## Input boundary and fixed topology panel

Require initial step0, all seven unique known ordinary labels, typed references,
simple undirected label0 binary ordinary edges, and no dangling references,
loops, parallel or mixed metadata/ordinary edges. Accept any simple topology,
including zero edges; do not identify an active triple or validate context.
Metadata-only structure, properties, cached incidence indexes, core metadata,
active-triple annotations and caller-supplied context/features never enter
input. IDs resolve endpoints and are then discarded. Semantic labels remain
fixed known vocabulary; the task core still contains information outside this
projection, so unseeded task discovery is not established.

Base inputs bind to nine saved native initial inventories. Detached variants
add an edge between inactive first-slot lures or a two-edge chain over three
inactive lures. The active path is unchanged. These are exposed **synthetic
input perturbations**, not native-validated reward worlds or optimizer-held-out
topology validation. No saved reward matrix is assigned to them. Future learning
must freeze its split and establish native semantics for new rewarding topology.

## Update support and diagnostic limits

`update` requires twelve samples, two per six off-diagonal development worlds.
Incomplete/unbalanced batches, reserved pairs, mixed modes, malformed rewards/
parameters/actions and invalid rates refuse. All parameters differentiate through
joint likelihood; zero rewards preserve parameters exactly.

One rate0.001 update per mode uses teacher-selected correct actions and detached
reward9/24. This intentionally tests plumbing, not on-policy sampling or new
native reward-driven learning:

| Mode | Mean joint log probability before | After | Changed encoder parameters / 104 |
| --- | ---: | ---: | ---: |
| Graph | -2.727006124 | -2.726838860 | 104 |
| Blind | -2.723907880 | -2.723744423 | 68 |

Independent accounting reconstructs both updates. Increased likelihood and
changed encoder parameters show differentiation/update support, not learned
relations, native generalization or scientific acceptance.

## Implementation and extension map

- [Encoder](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/graph_encoder.rs):
  `observe` validates/canonicalizes raw ordinary graph; `input` masks adjacency;
  `initialize` fixes the stream; `probabilities`, `sample`, `log_probability`
  and `score_gradient` implement choice/backprop; `update` enforces the fixed
  development denominator. Input/observation fields are private and serialize-only.
- [Rust tests and recorder](/Users/kalewicker/Documents/hypersim/crates/hg-train/tests/ere_graph_encoder.rs):
  complete base-gradient finite differences, invariances/refusals, diagnostic
  updates and panel persistence. Graph initialization never invokes Engine.
- [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_graph_encoder.py):
  separate raw parser, forward-mode dual-number derivatives, full fixed panel,
  sampling/update arithmetic, previous native ledger rechecks and descriptive
  expectations. Rust uses reverse-mode, not this derivative implementation.
- [Python tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_graph_encoder.py):
  finite differences, raw/mask/zero-init/update refusals, blind ceiling,
  complete accounting and sixteen detached panel corruptions.
- [Runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_graph_encoder.py):
  fresh source/input/binary archive, exact historical differences, feature
  closure, targeted zero-native checks, independent accounting and immutable seal.
- [Contract](/Users/kalewicker/Documents/hypersim/docs/ere_graph_encoder_v1.md)
  and [config](/Users/kalewicker/Documents/hypersim/configs/ere_graph_encoder_v1.json):
  fixed architecture, initializer, cases, tolerances, zero-native budget,
  synthetic-update limits and excluded disabled-feature scopes.

Old native, hand-feature observation/reward, learner, compiler and auditor
bodies remain unchanged. A future on-policy driver must persist decisions before
native execution, bind raw observations to actual inventories, persist complete
receipts before updates and retain every seed/zero-reward cell. Do not turn the
teacher-selected diagnostic into campaign evidence or rewrite this archive.

## Verification and immutable evidence

Three Rust groups pass in 1.36 seconds. Four Python groups pass prefixture with
two saved-panel groups skipped; all six pass afterward in 9.208 seconds. Rust
finite differences check 31,680 derivative components (all110 parameters ×
sixteen actions × two modes × nine worlds), epsilon1e-6/tolerance2e-8. Python
adds 1,980 finite-difference checks and reconstructs all 864 saved gradients
with independent forward-mode AD and fixed2e-12 arithmetic tolerance. Fourteen
Python malformed-input mutations, a mixed-edge refusal, additional update/mask
refusals and sixteen saved-panel corruptions reject. Feature-off Cargo checking
passes; targeted tests launch no older native campaigns.

All **267 seal entries** and **245 frozen/live source pins** verify. Frozen
source reproduces accounting with zero new native steps. Runner time25.434
seconds; archive10,515,848 bytes (about10 MiB).

Root seal: `2f6bbcde44f7bdbb646687b4a511ff6dfbc5484e293ff35d9860fa034b97e54d`.
Executed binary: `0323629a91fe90c3fc8c305ee5adaf4505820c530d83a96132f435b7a8bf6921`.

The previous 2,900-entry archive retains its seal; 144 complete saved native
action ledgers are rechecked without re-execution. Of its 238 old live pins,
232 match. Six differences are recorded: one additive train encoder export,
and five pre-existing unrelated chemistry additions in Cargo.lock and engine/
sim-core manifests/exports. Guards prove exact additive declarations and one
optional dependency-list change, not an active body waiver. Feature checks
keep checked-transitions/constructive-chemistry disabled, and sim-core on
cognition/hg-meta. Their implementations and disabled Evolving Minds sources
are prospectively excluded and preserved. All eight older referenced archives,
including original failures, retain seals/verdicts.

See [manifest](/Users/kalewicker/Documents/hypersim/output/ere_graph_encoder_engineering_20261005/manifest.json),
[audit](/Users/kalewicker/Documents/hypersim/output/ere_graph_encoder_engineering_20261005/audit.json)
and [historical differences](/Users/kalewicker/Documents/hypersim/output/ere_graph_encoder_engineering_20261005/historical_preservation.json).
Archive-only accounting, without invoking Engine:

```sh
python3 -B output/ere_graph_encoder_engineering_20261005/source_snapshots/scripts/verify_ere_graph_encoder.py output/ere_graph_encoder_engineering_20261005 --input output/ere_rule_relational_learning_trial_20261004
```

## Next experiment

Next is a separately preregistered **native raw-graph versus adjacency-masked
learning trial**, with identical architecture, initialization policy, optimizer/
draw/physics schedule and full-nine-world endpoints. Retain the old hand-feature
learner as a positive control; do not warm-start from it. Freeze budget, seeds,
rate, all-seed criteria and topology split before outcomes. Sparse rewards and
small initial representation gradients are unresolved optimization risks,
not reasons to promise success or adjust criteria afterward.

This fixture removes supplied distances from the implementation but does not
establish learned relations. Known labels/edit templates, predetermined timing,
seeded cores, reused exposed worlds, lack of ecological persistence and absence
of independent Engine replay remain. `training_campaign_launched:false`,
`training_admitted:false`, `scientific_acceptance:false`, and
`independent_physics_replay:false` remain unchanged.
