# Relation-sensitive task and graph-conditioned policy: findings

Date: 2026-10-04. Profile: `E2-rule-relational-v1`.
Status: **complete; engineering separation and independent accounting passed**.
No optimizer updates, learning/generalization trial, full-training admission,
scientific acceptance or independent physics replay occurred.

## Outcome

The new task requires ordinary connectivity to select the useful pair of paid
literal edits. All nine worlds have the exact same seven ordinary node identities
and data, including labels [10,20,30,11,21,31,12]. Only the two ordinary edges
change: one first-category node → one second-category node → label12 anchor.
The other four nodes are disconnected lures. Consequently the label marginals
and allocation order cannot identify which edit pair is useful.

All **396 cells × 24 = 9,504 native attempts/advances** complete without caps,
native failures, partial horizons, retries or source waivers. The exhaustive
live matrix has exactly one rewarding action pair per world. Both correct paid
edits earn 9/24; wrong, partial and no-edit pairs earn zero. The same frozen,
unpaid, disabled, initialized-reuse and one-fuel controls earn zero; reverse-order
correct edits earn 9/24 without early credit. There are 522 paid edits / new
genotype classes and 324 credited ordinary firings across the whole panel.

| Selector / bound | Balanced mean expected reward | Meaning |
| --- | ---: | --- |
| Uniform four-way actions per slot | 0.023437500 | No graph-conditioned preference |
| Nonuniform category-only probe | 0.022192974 | Identical cue and probabilities in all worlds |
| Any graph-blind joint distribution: ceiling | 0.041666667 | Mathematical bound from the measured action matrix |
| Hand-set graph-conditioned probe | 0.336953563 | Representability check; **not trained** |

The graph probe exceeds the prospectively fixed 0.30 criterion, well above the
graph-blind ceiling. These values are exact expectations over all 16 measured
native action outcomes in every world, averaged over all nine worlds and both
seeds. They are not sampled policy rollouts or a learned performance estimate.
Every world, including all zero-reward actions, remains in the denominator.

## Why the comparison now depends on relations

For a graph-blind policy, the same input produces the same joint distribution
q in every matched world. Each of nine distinct nonzero action pairs is useful
in exactly one world. Thus balanced expected reward is

```text
(9/24) / 9 × sum(q[the nine useful action pairs]) ≤ 1/24.
```

This bound also covers correlated joint policies, not only the earlier
factorized selector. It does not claim that the category-only probe is optimal
or that a graph-blind learner was trained to its ceiling. The unchanged category
interface receives the same fixed cue [10,11] in every world; the bound applies
to any encoding of the identical ordinary label marginals.

The graph-aware interface reads the initial ordinary inventory, computes
shortest-path distance from label12, and supplies three distance2 indicators
for first-slot categories and three distance1 indicators for second-slot
categories. All nine observations differ. Features are invariant to consistent
ID remapping, collection/endpoint order and irrelevant metadata changes.
Malformed or post-initial observations refuse rather than become zero scores.

The readout is **hand-engineered**, not a learned graph neural representation.
Sixteen parameters—four biases and a shared 4×3 matrix—map those features to
the two four-way action distributions. The hand-set probe uses diagonal weight4
and zero biases; its result demonstrates capacity, not successful optimization.
Both action choices are supplied before execution; steps1/5 timing is seeded.

The initialized task-specific core still exposes the answer in metadata, as in
the preceding fixture. The policy observation intentionally excludes that core,
literal targets, receipt context, active-triple annotations, rule tables, future
states and rewards. Its API takes a validated `Observation`, not a context label.
The independent checker separately verifies the full metadata/native ledger.
This is a controlled task-observation boundary, not a claim of an unseeded world.

## Native provenance and lifetimes

The ordinary path becomes a star through the initialized core. A first paid
edit creates a partial class at step1, installed at boundary2. It never fires.
The second creates the complete class at step5, installed6; proposal firings
occur at 7,9,11,13,15,17,19,21,23. Partial genotype retirement is step5 and
installed retirement6. Complete lifetimes are right-censored at boundary24.
No observed complete-class death, population half-life or sustained H-R1 claim
is inferred. Root survival is not class survival.

Each edit consumes one Float fuel. No host graph writes occur during execution.
All seven ordinary nodes, including every lure, keep their identities and data;
only the active triple participates in world rewrites. Full typed node/edge
changes, epoch tables, canonical/execution identities, exact/structural initial
exclusions, payments, pooled 50k/50k budgets and RNG probes are independently
reconstructed. The saved RNG probes do not change within cells. This is receipt
accounting, not an independent execution of Engine/RNG physics.

## Implementation and extension map

- [Native task](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere/rule_relational.rs):
  fixed seven-node initializer; active-path wiring; two seeded paid editors;
  native compilation and graph/event recorder; preservation of all lure data.
  `initial_observation` constructs a graph without invoking Engine or spending
  native steps, allowing prelaunch observation tests.
- [Observation/policy/reward interface](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/rule_relational.rs):
  ordinary-only validated connectivity readout, shared sixteen-parameter linear
  logits, explicit two-draw sampling, stable joint likelihood/score gradient,
  and strict native-receipt reward. No optimizer exists in this module.
- [Prospective contract](/Users/kalewicker/Documents/hypersim/docs/ere_rule_relational_v1.md)
  and [config](/Users/kalewicker/Documents/hypersim/configs/ere_rule_relational_v1.json):
  fixed worlds, seeds, full matrix/controls, probe parameters, thresholds,
  deadlines, representational bound and interpretation limits.
- [Runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_rule_relational.py):
  fresh-only source/binary archive, zero-native preflight tests, exact one-panel
  launch, accounting/tamper checks, feature-off regression and preservation.
- [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_rule_relational.py):
  corrected cross-side target proof, all-seven-node ledger reconstruction,
  ordinary identity/data equality across worlds, full action matrix,
  independent BFS observation, policy arithmetic and native-weighted expectations.
- [Rust tests](/Users/kalewicker/Documents/hypersim/crates/hg-train/tests/ere_rule_relational.rs)
  and [Python tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_rule_relational.py):
  invariances, relation contrasts, malformed-input refusals, all 144 joint-action
  gradients over 16 parameters, graph-blind bound, full native causal matrix
  and detached observation/ledger/policy corruption.

The native module is a versioned isolated fork of the prior bounded fixture,
not another physics implementation: it still uses Engine, GraphRuleSource,
native editors and the unchanged canonical/lifetime helpers. This deliberately
duplicates orchestration to preserve every prior body/pin. A future shared-runner
refactor should be a separately verified change, not silently mixed into a trial.

## Verification and immutable evidence

Two Rust groups pass: zero-native observation/gradient/refusal checks, then the
fixed native panel. Five Python groups pass before launch, with two receipt-only
groups skipped; all seven pass afterward. Tests reject 10 malformed observations,
24 detached native ledger corruptions and eight policy/probe corruptions, plus
eight measured observation-scope/relation corruptions. The synthetic native
regression uses detached old receipts with added isolated lures; it is expressly
not counted as new native relational evidence. Development checker-scope/test
identity issues were fixed before launch; no postmeasurement correction occurred.
The feature-off Cargo check also passes.

All **643 archive seal entries** and all **231 active measurement pins** verify.
The frozen auditor reproduces the result from its own source snapshots with
zero new native steps. Native test execution: 44.03 seconds; complete runner:
67.125 seconds; archive: 72,373,165 bytes (about 69 MiB).

Root seal: `58949cc2b129ae8cc845c3068fb5eb5d8ca7a5ffc8d88f711f6384b4fda17bcd`.
Executed binary: `978302ed6c67f6aae9b92944b60153f2dca901b170123aef832c6e13f70743dd`.

The prior 1,445-entry shared-learning archive is intact under root
`e6b6ef1faa22f7c2a1af0787154fc7b13a3811bc038e4f4ac9b35babc5ba262f`.
Of its 223 active pins, 221 are unchanged; two differ only by additive exports
in sim-core/train. Older 617/11/758/283/187-entry composition, corrected-audit,
choice-learning, adapter and lifecycle archives retain their original seals,
as does the failed preflight. The original failed composition audit stays failed.
Prior native, learner, action/reward, compiler and auditor bodies were not edited.
Unrelated Evolving Minds work remains disabled, scoped out and unreverted.

See the [sealed manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_relational_development_20261004/manifest.json)
and [independent accounting](/Users/kalewicker/Documents/hypersim/output/ere_rule_relational_development_20261004/audit.json).
To recheck accounting without running native physics:

```sh
python3 -B output/ere_rule_relational_development_20261004/source_snapshots/scripts/verify_ere_rule_relational.py output/ere_rule_relational_development_20261004/native
```

## Next boundary

The task and graph-conditioned action interface are ready for a **separately
preregistered small paired learning trial**: learn from native novelty reward
under graph-aware and category-only observations, retaining common draw/physics
streams and causal controls. Score all nine balanced worlds to compare with
the 1/24 graph-blind ceiling; any update-held-out subset must be additional and
must not replace that full denominator. Freeze seeds, rate, budgets and all-seed
criteria before launch. Start graph weights from zero, not this hand-set probe.

That next trial asks whether reward-driven optimization learns to use the
relational readout. It still would not show autonomous relation/vocabulary
discovery, learned graph representation, online edit scheduling, ecological
persistence or full scientific admission. No further campaign was launched here.
