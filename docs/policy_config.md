# Policy configuration — `[cognition]` and `[training]`

Governs the config surface of the learning layer
(`hypersim_nn_experiments_plan_v2.md`). Additive per v4 §6.4: every section
here is optional, and a config with no `[cognition]` section produces
byte-identical behaviour to one built before the section existed.

Companion documents: `docs/fitness_formulas.md` (what gets optimized),
`docs/campaign_output.md` (what gets written),
`docs/instrument_registry.md` (what earns the right to be reported).

**v2 changes, all additive:** `[cognition.probe].readout_set` splits the
probe readout from the policy `feature_set` (§4B); two new `fitness`
values; a note on why the `sigma` default is wrong for `StaticBias` (§6).
Nothing in §§1–4 changed, so every C0–C5 config still parses and still
means what it meant.

---

## 1. `[cognition]` — read by `hg-sim-core`

```toml
[cognition]                       # Option<CognitionConfig>; absent = no policy
policy = "mlp_scorer"             # none | pass_through | random_subset
                                  # | degree_greedy | rule_bias
                                  # | static_bias | mlp_scorer
budget = 16                       # max pattern-match bindings selected per step
wl_buckets = 16                   # feature 5's bucket count, B
feature_set = "v1"                # frozen feature list (§4)
rng_seed_salt = 0                 # folded into the policy RNG derivation (§5)

[cognition.mlp]
hidden = 16

[cognition.degree_greedy]
direction = "high"                # high | low

[cognition.rule_bias]
weights = [1.0, 0.5, 2.0]         # one per rule, hand-set; length must equal n_rules

[cognition.probe]                 # v2 — the READOUT, not the policy (§4B)
readout_set = "r1"                # r1 | r2
top_n_wl = 16                     # r2 only: WL-hash multiset depth
```

| Key | Type | Default | Notes |
|---|---|---|---|
| `policy` | string | `"none"` | `"none"` is equivalent to omitting the section |
| `budget` | usize | `16` | `0` is rejected at config load, not silently treated as ∞ |
| `wl_buckets` | usize | `16` | must be ≥ 1; changing it changes `feature_set` semantics (§4) |
| `feature_set` | string | `"v1"` | the **policy** feature list (§4); only `"v1"` is accepted |
| `rng_seed_salt` | u64 | `0` | lets one campaign re-run with different policy noise at fixed sim seeds |
| `probe.readout_set` | string | `"r1"` | the **probe** feature list (§4B); `"r1"` or `"r2"` |
| `probe.top_n_wl` | usize | `16` | r2 only; ignored under r1 |

`budget` is `Option<usize>` in the deserialized type; `budget = "inf"` is
**not** spelled as a string. Unbounded budget is expressed by omitting the
key, which selects `PassThrough` semantics for the truncation step (the
policy still runs, still scores, and still returns everything).

### 1.1 The section is inert by construction

`CognitionConfig` is `Option`-wrapped in `SimConfig` and behind the
`cognition` Cargo feature on `hg-sim-core`, mirroring how `alife` is
handled today. A build without the feature does not have the field; a build
with the feature and no section never constructs a policy and never calls
`Engine::set_policy`, so `Engine::policy` stays `None` and the step loop's
policy block is not entered at all
(`crates/hg-engine/src/engine.rs:148`).

This is testable and is tested: the F-L0 leak detector asserts that a
`PassThrough` rollout produces the same `final_canonical_hash` as the same
config with `[cognition]` absent, on every Phase-4 fixture.

---

## 2. Budget semantics

### 2.1 What the budget is

The policy returns **at most `budget` pattern-match binding indices per
step**, plus every creation binding (§3). Truncation happens inside the
policy. The engine is not modified and does not know a budget exists.

### 2.2 Why a budget exists at all

Under `StepMode::MaxParallel` the engine applies every non-conflicting
match. If conflicts are rare, "select everything" ≈ engine default and a
policy's choices barely register in the dynamics. Selection acquires causal
power only when rewrite bandwidth is scarce, so the budget is the
load-bearing experimental variable, not a performance knob.

### 2.3 The budget is not the only throttle — `max_pairings_per_step`

The engine already caps accepted rewrites per step:

```rust
StepMode::MaxParallel => resolve_conflicts(&ordinary)
    .into_iter()
    .take(self.max_pairings_per_step)   // default 1000
    .collect(),
```

Two consequences the NN plan does not state, and both matter for Stage 0a:

1. **`k = ∞` is not unconstrained.** The plan's budget sweep
   `k ∈ {1, 4, 16, 64, ∞}` has `∞` meaning "policy truncates nothing",
   which is still capped at `max_pairings_per_step = 1000` downstream. On
   rule sets where a step routinely offers more than 1000 non-conflicting
   matches, the `∞` arm is measuring the engine's cap, not the absence of a
   budget. **Stage 0a must record `attempted_matches` per step** and state,
   per rule set, whether the cap ever bound. Where it did, the `∞` arm is
   reported as `k = 1000`, not as `∞`.
2. **The two throttles compose in the wrong order to be interchangeable.**
   The budget cuts *before* conflict resolution and `max_pairings_per_step`
   cuts *after*. A budget of `k` therefore yields at most `k` accepted
   rewrites but usually fewer (the resolver drops conflicting members of
   the selected set), whereas `max_pairings_per_step = k` yields exactly
   `min(k, n_disjoint)`. Budget-matched comparisons must match the
   *budget*, never substitute the engine cap for it.

`max_pairings_per_step` is left at its default for all cognition work
unless a stage says otherwise in writing, so that it is a constant across
arms rather than a second uncontrolled variable.

### 2.4 Budget-matched comparison is mandatory

Every learned policy's result is reported as an improvement over
`RandomSubset` **at the same budget**, never against unconstrained
`MaxParallel`. Comparing a budgeted learned policy against an unbudgeted
default confounds "learned preferences" with "throttled dynamics", and the
throttle alone reshapes dynamics (plan §13.1).

---

### 2.5 Which rule sets can be swept at all — measured liveness

A rule set only matches on an initial condition containing its LHS. On the
wrong IC it produces **zero candidates for the entire run**, and selection
is then not weakly causal, it is undefined: there is nothing to select.
Stage 0a must not sweep such a pairing and report "selection-invariant" —
that would be a statement about the initial condition, not the substrate.

Measured over 64 steps, `attempted_matches` summed across windows
(`crates/hg-sim-core/tests/fl0_leak_detector.rs`):

| rule set | PureCrystal | TriLattice | GliderRing | WhiteNoise | CrystalWithDefects |
|---|---|---|---|---|---|
| `triangle_subdivide` | 0 | 774 912 | 0 | 73 728 | 0 |
| `binary_edge` | 1 328 256 | 1 354 128 | 0 | 1 311 920 | 1 328 256 |
| `tetrahedral` | 0 | 0 | 0 | 0 | 0 |
| `glider` | 0 | 0 | 384 | 0 | 0 |
| `growth_treadmill` | 0 | 0 | 0 | 576 | 0 |

Three things follow for Stage 0a.

**`tetrahedral` has no live pairing among the shipped initial conditions.**
It cannot be swept until an IC containing its LHS exists. Reporting it as
selection-invariant would be a category error.

**Candidate volume spans four orders of magnitude.** `binary_edge` offers
~56 000 candidates per step on a crystal; `glider` offers ~6. A single
budget ladder `k ∈ {1, 4, 16, 64, ∞}` therefore means completely different
things across rule sets — on `glider` every rung above 4 is equivalent to
`∞`, while on `binary_edge` even `k = 64` is a 0.1% throttle. Budgets must
be chosen per rule set relative to its own candidate volume, which is what
§13.1's "treat k as a per-rule-set constant chosen from the curve" amounts
to in practice.

**`max_pairings_per_step` binds hard on the dense rule sets.** At ~56 000
candidates per step against a cap of 1000, `binary_edge`'s `k = ∞` arm is
measuring the engine cap and nothing else (§2.3). The `pairings_cap_bound_
steps` column exists for exactly this case.

---

## 3. The creation-rule exemption

**Creation bindings are exempt from the budget and from learned
selection.** Scheduled creation rules (v4 §5.3) are environment physics —
input injection, seeding — and a learned policy must not be able to starve
them by spending its budget elsewhere.

### 3.1 The predicate

A creation binding is identified by `Binding::is_creation()`:

```rust
pub fn is_creation(&self) -> bool {
    self.bound_nodes.is_empty() && self.matched_edges.is_empty()
}
```

**Policies must use `is_creation()`, not `bound_nodes.is_empty()`.** The NN
plan §5.6 sketches the `MlpScorer` partition as
`cands[i].bound_nodes().is_empty()`, which is the weaker predicate: a rule
whose LHS matches edges but binds no node variables would be misclassified
as a creation binding and would bypass the budget. `is_creation()` is also
exactly the predicate the engine uses for its own creations/ordinary split
(`crates/hg-engine/src/engine.rs:166`), so using it keeps the policy's
partition and the engine's partition in agreement by construction rather
than by coincidence.

### 3.2 The engine does not enforce this

The engine's comment is explicit that the exemption is the policy's job:

> Due creation firings are always accepted (v4 §5.3) — they can never
> conflict … (A `RuleSelectionPolicy` ran before this split, so a policy
> may still deliberately drop a creation candidate.)

The policy runs *before* the creations/ordinary split, so a policy that
omits a creation index has already starved it; nothing downstream restores
it. Every policy in `hg-cognition` therefore passes creation indices
through unconditionally, and a shared unit test (`creation_exempt`) asserts
this for every policy in the zoo. The one deliberate exception is
`StimulusDriver` (§3.3).

### 2.6 `StaticBias` cannot spend less than the budget

Two policies in the zoo treat an exhausted rule differently, and the
difference is load-bearing:

- **`RuleBias`** drops a zero-weighted rule outright and **leaves the budget
  unspent**. A budget of 16 against 3 admissible candidates selects 3.
- **`StaticBias`** re-normalizes its softmax over the rules that still have
  candidates and draws until the budget is filled. A budget of 16 against 3
  candidates of the preferred rule selects those 3 **and 13 of something
  else**.

So `StaticBias` can express "prefer rule A to rule B" but *not* "use rule A
and then stop". On any rule set where the correct behaviour is to do less
than the budget allows, the correct behaviour is **unreachable** — and a
campaign there converges to the best of a set of bad options while behaving
perfectly correctly at every step. Measured on F-L1
(`crates/hg-train/tests/fl1_learnable_signal.rs`): at `k = 8` a maximally
A-preferring genome scores 1.00, at `k = 16` the same genome scores −0.50,
because the preferred rule only offers ~6–7 bindings per step.

Two consequences:

1. **Choose `k` at or below the correct rule's typical binding count**, not
   from the default. This is a second, independent reason for §13.1's "treat
   k as a per-rule-set constant chosen from the curve".
2. **Calibrate a fixture with the policy class under test.** Calibrating
   F-L1 with `RuleBias` showed a clean margin at `k = 16` that `StaticBias`
   then could not collect — the calibration was measuring an arm the trainee
   cannot reach. This is not hypothetical; it happened, and the campaign
   converged to the worst possible fitness before it was caught.

`MlpScorer` behaves like `RuleBias` here: it Gumbel-ranks and truncates, so
a candidate it scores at `−∞` is simply not selected and the budget goes
unspent. Only `StaticBias` has the fill-the-budget property, as a direct
consequence of the plan's "budget allocated by softmax over rule logits,
uniform within a rule's group" specification.

### 3.3 `StimulusDriver` is the sanctioned exception

`StimulusDriver` exists specifically to gate creation rules — it implements
reservoir input injection by admitting exactly the creation binding
matching `tape[step / W]` and dropping its partner. It is **0 parameters
and never learned**, so no optimizer can discover creation-starvation as a
strategy. Composition with a learned policy is `PolicyStack`: the driver
handles creation bindings, the learned policy handles the remainder, and
neither sees the other's partition.

---

## 4. `feature_set = "v1"` — frozen

Per candidate binding, computed from `GraphSnapshot` only. No causal-layer
calls: v1 features must be cheap enough to run on every candidate of every
step of every rollout of every generation.

| # | Feature | Dim | Definition |
|---|---|---|---|
| 1 | rule one-hot | `n_rules` | `1.0` at `binding.rule_id`, else `0.0` |
| 2 | priority | 1 | `tanh(priority / 8.0)` |
| 3 | arity | 1 | `bound_nodes.len() / 8.0`, clamped to `[0,1]` |
| 4 | degree mean / max / min | 3 | `ln(1 + deg) / ln(1 + DEG_SCALE)`, `DEG_SCALE = 64`, clamped to `[0,1]`; all three `0.0` when `bound_nodes` is empty |
| 5 | WL bucket counts | `wl_buckets` | histogram of `wl_hash(node, radius 1) % B` over bound nodes, divided by `max(1, bound_nodes.len())` |
| 6 | log candidate count | 1 | `ln(1 + n_candidates) / ln(1 + 4096)`, clamped to `[0,1]` |
| 7 | rule share | 1 | fraction of this step's candidates sharing `rule_id` |

Total dimension: `n_rules + wl_buckets + 7`. At defaults
(`wl_buckets = 16`) that is `n_rules + 23`, matching the plan's count.

Every feature is normalized into roughly `[0, 1]` (feature 2 into
`[-1, 1]`). The normalization constants `DEG_SCALE = 64` and the
`4096` candidate-count scale are part of `feature_set = "v1"`: changing
either is a `feature_set = "v2"`, not a tuning knob, because a champion
genome trained under one normalization is meaningless under another.

### 4.1 Degree

Degree is `snapshot.edges_incident_to(node).len()` — incident hyperedge
count, not neighbour count. On a hypergraph these differ, and the incident
count is the one the snapshot answers in O(1).

### 4.2 WL hashes are cached per step, and computed by whichever strategy is cheaper

`compute_wl_hash_snapshot(snapshot, node, 1)` is memoized per distinct bound
node in an `FxHashMap<NodeId, WlHash>` keyed by `snapshot.step()`; the cache
clears when the step counter advances. Without it, a step offering 512
candidates over 8-node bindings would recompute up to 4096 radius-1 hashes.
Measured, the cache is worth **22×** on a 4096-node substrate: 512
three-node bindings over shared nodes extract in 36 µs, over distinct nodes
in 802 µs (`benches/scoring.rs`, `wl_hash_cache`).

Filling that cache has two strategies, and the extractor picks between them
per step (`FeatureExtractor::prime_wl`):

- **per-node ball walks** — `compute_wl_hash_snapshot` for each distinct
  bound node, ~0.48 µs each, cost proportional to *nodes touched*;
- **one whole-graph refinement pass** — `compute_wl_class_map_snapshot`,
  O(radius · E), ~0.066 µs per *graph* node, cost independent of how many
  nodes the step touches.

`hg-core` documents and property-tests the class map as bit-identical to the
per-node walk, so this is a cost decision with no effect on any feature
value — and `features::tests::both_wl_strategies_give_identical_features`
asserts the byte-identity directly, because a divergence would make a
champion score differently depending on how many nodes its step happened to
touch.

The switch is `distinct_bound_nodes · WL_CLASSMAP_RATIO > n_nodes`, with
`WL_CLASSMAP_RATIO = 8` from the measured ~n/7.3 crossover, rounded so that
being slightly late to switch is preferred to switching early on a large,
sparsely-touched graph. Priming happens **after** the 512-candidate cap, on
the subset that will actually be scored: priming from the full pool would
trigger the whole-graph pass on steps where the scorer touches a handful of
nodes.

### 4.5 Measured cost, and an open question for Phase C2

From `benches/scoring.rs` on a 4096-node ring substrate (release, M-series,
6 rules, `wl_buckets = 16`, `hidden = 16`, arity-3 bindings), one `select`
call:

| Candidates offered | `MlpScorer::select` | `RandomSubset::select` |
|---|---|---|
| 64 | 150 µs | — |
| 256 | 357 µs | — |
| 512 (the cap) | **410 µs** | 0.93 µs |
| 2048 | 423 µs | — |

Two things are readable here and both matter.

**The cap works.** 2048 candidates cost essentially the same as 512, which
is what `CAND_SCORE_CAP` is for. Cost is bounded in the candidate pool.

**The ≤10% overhead target is not obviously reachable.** At 410 µs per step,
feature extraction plus scoring is 10% of step wallclock only if a step
takes ≥ 4.1 ms — about **240 steps/s**. The NN plan §12 illustrates its cost
model at 10 000 steps/s; at that rate the policy would be ~80% of step time,
not 10%, and the campaign-hours formula would understate reality by roughly
5×.

This is not resolvable in `hg-cognition`, which has no step loop to measure
against — the ratio's denominator is Phase C2's to supply. What C1 supplies
is the numerator, and the note that if C2 measures the substrate materially
faster than ~240 steps/s, one of three things has to give:

1. lower `CAND_SCORE_CAP` (cost is close to linear below the cap, so 128
   would buy roughly 3×) — cheapest, and costs only selection breadth;
2. drop feature 5, the WL bucket block, in a `feature_set = "v2"` — WL
   hashing is the dominant term, but this removes the only structural
   signal in the feature set and would make F-L3 much harder to pass;
3. move the ≤10% target, and re-derive the campaign-hours estimate from a
   measured steps/sec rather than the plan's illustration.

Option 1 first. The decision belongs in C2's write-up with the measured step
time attached, not here.

---

## 4B. `readout_set` — the probe feature list (v2)

v1 conflated the policy feature set and the probe readout under one
`feature_set = "v1"` key. They are different objects with opposite
requirements:

| | policy feature | probe readout |
|---|---|---|
| computed per | **candidate binding** | **window** |
| how often | every candidate of every step of every rollout | once per window |
| requirement | **cheap** | **complete** |
| consumer | `MlpScorer` scoring, Gumbel-top-k | Stage 0b separation, Stage 1 memory capacity |

A policy feature that is expensive makes every campaign unaffordable
(§4.5: 410 µs per `select` call is already ~80% of step time at 10 000
steps/s). A probe feature that is *incomplete* makes every separation and
memory claim a joint statement about the substrate and the projection —
which is what happened.

They therefore version separately. `readout_set` lives under
`[cognition.probe]` and is **not** part of the genome contract: a champion
is meaningless under a different `feature_set` (`docs/campaign_output.md`
§6) but is completely indifferent to `readout_set`, because no policy ever
sees a probe row.

### 4B.1 `readout_set = "r1"` — current, retained

The 16 WL-bucket histogram used throughout C0–C5:

```
r1[b] = #{ nodes n : wl_hash(n, radius 1) % wl_buckets == b }  / max(1, n_nodes)
```

Dimension `wl_buckets` (16 at defaults). Retained **unchanged** so that
every C0–C5 number remains reproducible and any r1/r2 disagreement is
attributable to the readout rather than to a silent revision of the
baseline. `r1` reproducing C0–C5 exactly is Phase D2's exit criterion.

### 4B.2 Why r1 is not sufficient — the `glider` proof case

`glider`'s bucketed WL histogram is **exactly constant** across its entire
run. The rule is a conservative relabel on a symmetric ring: it moves a
label around a cycle without changing any node's degree, any edge, or the
multiset of WL classes. The histogram r1 computes is invariant under
exactly that operation.

So Stage 0b classified `glider` `Undetermined` and Stage 0a found no
budget cell above 2σ. Neither is a statement about `glider`'s dynamics.
**`glider` is invisible, not inert** — the readout is blind to the only
thing happening.

That is an independent demonstration that r1 has blind spots, and it
applies directly to Stage 1's headline. `MC = 0.0390` is a joint
measurement of substrate memory **×** readout sensitivity, and the ESN
comparison compounds it: the ESN readout sees its full 16-dimensional state
vector while the hypergraph readout sees 16 buckets of a hash. The factor
of 65 may be measuring the projection.

`input_coupling_r2 = 0.926` at delay 0 with nothing at delay ≥ 1 is the
signature of **both** hypotheses — a genuinely short intrinsic timescale,
and a readout that resolves the injected symbol but not its consequences —
so it cannot discriminate on its own and must not be cited as if it does.

### 4B.3 `readout_set = "r2"`

r1's block, plus:

| # | Block | Dim | Definition |
|---|---|---|---|
| 1 | r1 WL buckets | `wl_buckets` | unchanged from §4B.1 |
| 2 | raw WL multiset | `top_n_wl` | the `top_n_wl` most frequent **raw** `wl_hash(n, 1)` values by count, ordered by (count desc, hash asc), each as `count / max(1, n_nodes)`; zero-padded |
| 3 | degree moments | 4 | mean, variance, skew, max of incident-hyperedge count over live nodes; each scaled `ln(1+x)/ln(1+DEG_SCALE)` and clamped to `[0,1]`, with skew shifted by `tanh` into `[−1,1]` |
| 4 | edge count by label | `n_edge_labels` | count of edges per declared label, divided by `max(1, n_edges)` |
| 5 | components | 2 | connected-component count as `ln(1+c)/ln(1+n_nodes)`, and largest-component node fraction |
| 6 | raw size | 2 | `ln(1 + n_nodes) / ln(1 + SIZE_SCALE)` and the same for `n_edges`, `SIZE_SCALE = 65536`, clamped |

Total: `wl_buckets + top_n_wl + n_edge_labels + 8`. At defaults with a
3-label rule set, `16 + 16 + 3 + 8 = 43`.

**Block 2 is the block that unblinds `glider`.** r1 hashes into 16 buckets
and normalizes, which is invariant under a relabel that permutes classes
within the multiset. The *raw* hashes, ranked, are not: moving a label
around a ring changes which hash values are present and in what proportion.
The ordering tiebreak (`hash asc`) is specified because ties are common on
small graphs and an unspecified order would make the readout
process-dependent — the same order hazard the V-track's C3 found in
`HashMap::values()` summation (`docs/validation_vtrack_closeout_findings.md`).

**Block 5 needs a note it would be dishonest to omit.**
`docs/fitness_formulas.md` §5 declines to add a connected-component census
as a held-out *evaluator* metric, on the grounds that adding an evaluator
metric to serve the learning layer is exactly the contamination the
firewall forbids. Block 5 is not that. It is computed in `hg-cognition`,
from a `GraphSnapshot`, for a probe row that no evaluator sees and no
`metrics.csv` carries. `hg-eval` is untouched. The firewall runs between
the crates, and this stays on the correct side of it.

### 4B.4 Cost, and why it is affordable here but not in §4

Per **window**, not per candidate. Blocks 1–2 share one whole-graph WL
refinement pass (`compute_wl_class_map_snapshot`, O(radius · E)) — the same
pass §4.2 already selects between; blocks 3, 4 and 6 are O(V + E) scans;
block 5 is one union–find at O(E α(V)). Total O(E α(V)) per window against
a window of at least several steps of rewriting.

The identical computation in §4's policy path would be prohibitive because
it would run per candidate per step — which is precisely why the two
feature lists must not share a version key.

### 4B.5 r1/r2 disagreements are results

Stage 0b′ re-classifies every rule set under r2 and **documents every
disagreement with r1** rather than replacing r1's verdict silently. A rule
set that is `Undetermined` under r1 and `Viable` under r2 has had a readout
problem diagnosed; a rule set that is `Chaotic` under both has had its
verdict strengthened. Overwriting the r1 column would destroy the only
evidence that distinguishes the two.

Pre-registered for `glider` specifically: r2 **may not fix it**. A
conservative relabel on a symmetric ring may be invisible to any
coarse-graining that is not the identity. If block 2 is also constant on
`glider`, it is excluded from separation analysis **with that stated as its
reason**, rather than carried as `Undetermined` indefinitely.

---

### 4.3 Node age is deliberately absent

Age requires a birth-step property convention that substrate rules do not
currently write (`docs/property_registry.md`), and adding engine-side
bookkeeping to serve a policy feature is precisely the contamination the
firewall forbids. If age proves necessary it enters as
`feature_set = "v2"` with its own registry entry, additively.

### 4.4 Candidate cap

`MlpScorer` scores at most `CAND_SCORE_CAP = 512` pattern-match candidates
per step; above that it takes a uniform random subsample of 512 using the
policy RNG. The cap bounds scoring cost on pathological candidate counts.
It is a constant of `feature_set = "v1"`, and Stage 0a records how often it
binds per rule set — a rule set where it binds on most steps is one where
the policy is choosing from a sample rather than from the pool, and that
must be said out loud in the campaign report rather than discovered later.

---

## 5. Policy RNG derivation

All policy stochasticity flows from `ChaCha8Rng`. `thread_rng` is banned in
`hg-cognition` and `hg-train`, enforced by a `#![deny]`-style grep test in
CI alongside the existing reproducibility checks.

```
policy_seed = fxhash64(
    campaign_seed,
    generation,
    member_idx,
    rollout_idx,
    rng_seed_salt,
)
```

`rollout_idx` is the index of the seed within `seeds_per_member`, so the
two rollouts of one member on two sim seeds get different policy noise —
otherwise antithetic pairs would share a Gumbel draw and the common-random-
numbers argument would be doing less work than it appears to.

The determinism contract, which extends `docs/reproducibility_policy.md` to
the learning layer rather than exempting it:

> Same `SimConfig` + sim seed + policy parameters + policy RNG seed ⟹
> identical `final_canonical_hash` and identical fitness.

Tested directly in Phase C1 (identical `(snapshot, candidates, params, rng
seed)` ⟹ identical selection vector) and in Phase C2 (whole-rollout
bit-determinism).

### 5.1 Why Gumbel noise rather than deterministic top-k

`MlpScorer` perturbs each score by `−ln(−ln u)`, `u ~ U(0,1)`, before
sorting — the Gumbel-top-k trick, which samples `k` items without
replacement from the softmax over scores.

The reason is ES-specific. Under deterministic top-k, small parameter
perturbations usually do not change which `k` candidates win, so fitness is
locally flat and the ES gradient estimate is mostly zero. Gumbel noise makes
selection a smooth function of the scores in expectation, which is what ES
needs to see a gradient at all. It is a search-tractability device, not a
modelling choice — and it is why the policy RNG seed is part of the
determinism contract above rather than an afterthought.

---

## 6. `[training]` — read by `hg-train` only

`hg-sim-core` ignores this section entirely. It is present in the campaign
config so that one file fully determines a campaign.

```toml
[training]
algo = "openai_es"                # openai_es | cma_es
population_pairs = 32             # 32 antithetic pairs = 64 rollouts/generation
sigma = 0.02
alpha = 0.01
generations = 60
seeds_per_member = 3
seed_rotation = "per_generation"  # fixed | per_generation
fitness = "f2_regime_occupancy"   # f1_interestingness | f2_regime_occupancy
                                  # | f3_constructor_yield
                                  # | f4_predictive_information   (v2)
                                  # | f5_constructor_reuse        (v2)
horizon_steps = 20000
burn_in_windows = 5
holdout_seeds = [900001, 900002, 900003, 900004, 900005]
```

| Key | Default | Notes |
|---|---|---|
| `algo` | `"openai_es"` | `cma_es` is for `StaticBias` (< 10 params); see plan §8.2 |
| `population_pairs` | 32 | rollouts per generation = `2 · pairs · seeds_per_member` |
| `sigma` | 0.02 | ES perturbation scale. **This default is wrong for `StaticBias` and P-2 now catches it** (v2): 0.02 is a step in *softmax-logit* space, where it moves the induced allocation by under a percentage point. Forty CMA-ES generations at this value left the allocation uniform to 4% and returned a clean negative from a search that never left the origin. `StaticBias` campaigns use `sigma = 2.0`; the default is retained for `MlpScorer`, whose parameters are not logits |
| `alpha` | 0.01 | ES step size |
| `generations` | 60 | |
| `seeds_per_member` | 3 | |
| `seed_rotation` | `"per_generation"` | `"fixed"` is for debugging only and flags `SEED_OVERFIT` by construction |
| `fitness` | `"f2_regime_occupancy"` | formulas in `docs/fitness_formulas.md` §2–4, §4A, §4B |
| `horizon_steps` | 20000 | F3 comparisons require this matched across arms |
| `burn_in_windows` | 5 | changing it is a fitness version bump |
| `holdout_seeds` | five fixed | must be disjoint from every rotation seed set |

### 6.1 Seed schedule

All members within generation `g` share the seed set
`S_g = { hash(campaign_seed, g, j) : j < seeds_per_member }`. `S_g` rotates
per generation, which prevents seed memorization; the frozen
`holdout_seeds` are disjoint by construction and used only for champion
evaluation.

This buys both comparability properties without compromise: population
fitness is comparable *within* a generation (common seeds), and champion
trajectories are comparable *across* the campaign (fixed hold-outs).

Disjointness is asserted at campaign start, not assumed: `hg-train` checks
`S_g ∩ holdout_seeds == ∅` for every `g` before the first rollout and
refuses to start otherwise. A collision would silently invalidate
`SEED_OVERFIT` for the whole campaign, and it is cheap to rule out up
front.

### 6.2 `[training]` cannot choose an output path

There is no output-directory key. The campaign path is derived
(`runs/cognition/<campaign_id>/`) and `runs/substrate/` is unreachable from
`hg-train` by construction — see `docs/campaign_output.md` §5.
