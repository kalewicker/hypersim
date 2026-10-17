# Relation-sensitive task and graph-conditioned interface V1

Registered 2026-10-04 before native work. Engineering-only extension of the
successful shared two-edit learning trial; no optimizer or new training campaign.
The next boundary is whether ordinary graph structure can make a useful action
depend on more than the supplied category lookup cues.

## Matched task

Initialize exactly seven ordinary nodes with labels [10,20,30,11,21,31,12], in
that order and with identical data. Across all nine worlds, their identities,
labels, energies, generations and properties are identical. Only two ordinary
label0 edges vary: connect one first-category node to one second-category node,
then that second-category node to the label12 anchor. Four nodes are disconnected
lures. All nine category pairs are balanced and exposed. Node allocation order
must not reveal which pair is active.

Each cell uses the existing path-to-star core and star-to-path proposal motifs
on the active triple. The initialized core is task-specific and encodes the
answer in metadata; this is explicitly a seeded fixture. The policy must not
observe that metadata, its literal targets, the `context` receipt field, the
declared active triple, rule tables, future states, chosen actions or rewards.
It receives only a validated projection of the **initial ordinary inventory**.
No extra rules, host writes during execution, reward changes or edit discounts.

Two proposal literals start99/98, edited natively at steps1/5 for one Float fuel
each. Fixed(2) recompilation, ordinary novelty exclusions, full24-step denominator,
initialization-lure controls, paid-edit attribution and censoring are unchanged.
Each slot chooses no-edit or one of its three labels. Exhaust every16-way live
action pair; then correct-action frozen/unpaid/disabled/preseeded/one-fuel/reverse
controls. Fixed order: two seeds993748718001/002, nine lexicographic worlds,
row-major actions, controls in JSON order. Total396 cells /9,504 steps, pooled
50k/50k, MaxParallel16, shared180-second native deadline. No culling, retries,
seed replacement, hidden warmup or outcome-dependent continuation.

Both correct paid edits should earn9/24 with first proposal firing7, then every
odd step through23. Every partial/wrong/no-edit and blocking control should earn
zero; reverse edits should earn9/24 without early credit. Ordinary lures stay
unchanged and cannot participate in native world rewrites.

## Observation and policy

Compute shortest-path distances from the unique ordinary label12 anchor using
only undirected ordinary label0 binary incidence. Refuse malformed, duplicated,
mixed meta/ordinary, extra/missing ordinary edges, wrong marginals, non-path or
post-initial observations. Node IDs are references, never numeric features.
Slot0's three features are distance2 indicators for labels10/20/30; slot1's
three features are distance1 indicators for labels11/21/31. All four lures must
be unreachable. This is a **hand-engineered relational readout**, not a learned
graph neural encoder or autonomous relation discovery. The policy still has
to map features to action probabilities; no correct action is returned by its API.

Sixteen parameters shared between slots: four biases and a4×3 matrix. Logit for
action a = bias[a] + sum(weight[a,k] * observed_feature[slot,k]). Joint sampling,
log likelihood and score gradients factorize across slots, with explicit two
draws separate from physics. Verify finite-difference gradients for all nine
worlds,16 actions and16 parameters; stable likelihoods, input refusals and sampling
are tested without native execution. Features must be invariant to consistent
ID remapping, collection/endpoint ordering and irrelevant metadata changes;
swapping the ordinary relation must change the relevant features/probabilities.

The category-only comparator gets the same cue[10,11] in every world because
ordinary label marginals are identical; it is the unchanged12-logit interface.
No active-pair context is supplied to it. Any graph-blind joint distribution,
even beyond factorized policies, has balanced mean reward at most(9/24)/9=1/24:
each of the nine disjoint correct pairs is useful in just one world, and their
probability mass sums to at mostone. This is a mathematical representational
bound, not a claim that a baseline was trained to its optimum.

Record uniform and nonuniform category-only probes, plus a hand-set graph probe
(all biases zero, action1/2/3-to-feature0/1/2 diagonal weights4, others zero).
Compute exact expectations by reweighting measured native action outcomes,
including every wrong/partial cell. Require identical category-only probabilities
across worlds, both expectations under1/24, and graph probe mean above0.30.
The graph probe is **not trained**; it verifies representability, not learning.
Uniform graph logits must give9/384. Retain all nine worlds; do not choose easy
worlds after inspecting results. No held-out/generalization claim in this stage.

## Preservation and auditing

Implement versioned new modules, leaving prior native, action/reward, learner,
compiler and auditor bodies unchanged. The native version is an isolated fork
of the fixed composition fixture to introduce ordinary lures; document this
duplication as a bounded engineering choice, not a replacement engine. Reuse
the unchanged canonical/lifetime helpers. The new independent native checker
retains the corrected topology proof, reconstructs all seven-node inventory and
every graph delta, and enforces isolation of the four lures.

Freeze config, contract, active source closure, binary, observation/policy tests,
runner and checker prospectively. Verify the1,445-entry learning archive and all
older referenced seals; live drift may consist only of two additive exports.
Keep unrelated Evolving Minds work disabled, out of scope and unreverted;
check the Cargo feature tree before/after. Require a fresh output directory,
preserve/seal failures, and fail closed on source drift or invalid work.
Run zero-native observation/arithmetic/refusal tests before launch; then one
fixed native panel, independent full-ledger/expectation audit and detached
tamper regressions. Accounting is not independent Engine/RNG replay, sustained
H-R1, half-life, ecological persistence or scientific/full-training admission.

If this engineering gate passes, separately preregister a small reward-driven
graph-conditioned learning trial against the graph-blind comparator. Do not
launch it here or treat the hand-set probe as evidence that learning succeeded.
