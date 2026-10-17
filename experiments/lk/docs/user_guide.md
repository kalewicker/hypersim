# Hypersim User Guide

*Guide version 1.0 — covers Hypersim 0.1.0 (plan v4 substrate-only pipeline).*

This is the practical manual: how to build, configure, run, and read
Hypersim, and where to plug in when you extend it. The nine specification
documents in `docs/` remain the source of truth for exact formulas and
contracts; this guide links to them rather than restating them.

Chapters end with a **Future work** block where planned features will
slot in — see also chapter 14 (Roadmap). When you add a feature, extend
the matching chapter and bump the guide version above.

---

## Contents

1. [What Hypersim is](#1-what-hypersim-is)
2. [Building](#2-building)
3. [Quick start](#3-quick-start)
4. [Core concepts](#4-core-concepts)
5. [Running simulations: the CLI](#5-running-simulations-the-cli)
6. [Configuration reference](#6-configuration-reference)
7. [Writing rule sets](#7-writing-rule-sets)
8. [Run outputs](#8-run-outputs)
9. [The evaluator: metrics, regimes, gates](#9-the-evaluator-metrics-regimes-gates)
10. [The causal layer](#10-the-causal-layer)
11. [The GUI workbench](#11-the-gui-workbench)
12. [Reproducibility and performance](#12-reproducibility-and-performance) — incl. [12.5 the locality & leakage track](#125-the-locality--leakage-track-lk)
13. [Extending Hypersim](#13-extending-hypersim)
14. [Roadmap and reserved seams](#14-roadmap-and-reserved-seams)

---

## 1. What Hypersim is

Hypersim is a research workbench for **hypergraph rewriting systems**: you
define a set of rewrite rules in TOML, seed an initial hypergraph, and the
engine repeatedly matches and applies rules. Around that substrate sit two
science layers:

- an **evaluator** that classifies each evaluation window into one of 11
  dynamical regimes, scores interestingness, and enforces eight
  quality gates; and
- a **causal language layer** that names recurring structures
  (WL-hash attributes), tracks transformations between them (tasks),
  detects structures that persist while catalyzing transformations
  (constructors), and accumulates a certainty-tagged knowledge graph.

Every run produces exactly one CSV, one Markdown report, and a
`manifest.json` that is the durable, authoritative record of run status.
Same configuration + same seed ⇒ same result, at any thread count.

**Future work:** an ALife layer (`hg-alife`) and a policy/cognition layer
(`hg-cognition`) are reserved but intentionally absent until the
substrate-only pipeline is proven clean end to end.

## 2. Building

Requirements: a recent stable Rust toolchain (edition 2021). macOS, Linux,
and Windows should all work; the GUI uses `eframe`/`egui`.

```sh
cargo build --workspace              # debug
cargo build --workspace --release    # what you want for real runs
cargo test  --workspace              # full suite (fast)
```

Always use `--release` for runs beyond a few thousand steps — match
enumeration is the hot loop and debug builds are ~20–50× slower.

## 3. Quick start

```sh
# 1. Sanity-check the evaluator against its six canonical fixtures
cargo run --release -p hypersim-cli -- fixtures

# 2. Run a bundled experiment
cargo run --release -p hypersim-cli -- run --config configs/example_binary_edge.toml

# 3. Open the results
open output/binary_edge_demo_*/report.md      # 11-section report
open output/binary_edge_demo_*/metrics.csv    # one row per window

# 4. Launch the GUI workbench (live backend)
cargo run --release -p hypersim
```

## 4. Core concepts

**Hypergraph.** Nodes carry a `label: u32`, a `generation` (the step they
were created), typed `properties` (`Int`/`Float`/`Bool` — see
`docs/property_registry.md`), and an `energy: f32`. Edges are *hyperedges*:
an unordered set of 2+ endpoints plus a `label`. Node and edge identities
are generational (`NodeId`/`EdgeId`) — stale ids never dangle, lookups on
deleted entities return `None`.

**Rewriting.** Each rule has a left-hand side (LHS) pattern and a
right-hand side (RHS) replacement over shared variables. Matching is
**non-injective by default**: two variables may bind the same node unless
a `vars_distinct` guard forbids it. Applying a rule deletes the matched
edges and any LHS-only nodes (dangling edges are cleaned up), keeps
surviving variables' nodes, creates the RHS-only nodes and all RHS edges.
The full contract, including the conflict-set definition, is
`docs/rewrite_contract.md`.

**Steps and windows.** One `step` = collect candidate bindings from all
enabled rules, resolve conflicts, apply the accepted set. Metrics are
computed every `metrics_window_interval` steps — one **window** = one CSV
row = one evaluator/causal observation.

**Determinism.** Candidates are canonically sorted before selection, so
thread count never changes results (`docs/reproducibility_policy.md`).

**WL hash (types).** A node's "type" is its deterministic Weisfeiler-Leman
neighbourhood hash at radius 1: identical local structure ⇒ identical
type, wherever (and *whenever*) it occurs. The type histogram drives most
evaluator metrics and all causal-layer identity.

## 5. Running simulations: the CLI

```
hypersim-cli run --config <FILE> [--steps N] [--seed N] [--out-dir DIR] [--threads N]
hypersim-cli fixtures
hypersim-cli cone --run-dir <DIR> [--delta N] [--events N] [--seed N] [--slack F]
hypersim-cli twin --config <FILE> [--branch-step N] [--windows N] [--window-len N]
                  [--perturb remove-edge|add-edge|control] [--anchors N] [--out-dir DIR]
hypersim-cli leakage-table <RUN_DIR>...
hypersim-cli chains --run-dir <DIR>
```

- `run` executes a config headlessly to completion and prints the artifact
  paths, final step, and the final canonical graph hash. The flags
  override the corresponding config fields.
- `fixtures` runs the six-fixture evaluator truth table
  (`docs/fixture_definitions.md`) and exits nonzero if any fixture
  misclassifies — a quick self-test that the science layer is intact.
- `cone` (LK2) joins a run's `causal_events.jsonl` with its snapshots and
  writes `cone.csv` + `cone_summary.json` — effective front speed `v_eff`,
  anomalous fractions split into dynamics and cleanup channels, dispersion
  probe by LHS size. Needs the run to have set `[leakage] cone_events_enabled`
  and `[output] snapshots_enabled`.
- `twin` (LK3) branches a run at `--branch-step`, applies one hyperedge
  perturbation in branch B, and writes `twin.csv` (label-invariant damage
  front). MaxParallel only (RNG zero-draw precondition). `--perturb control`
  is the zero-damage validity check.
- `leakage-table` (LK5) ranks run directories by steady-state leakage
  profile and fires the campaign-level suspicious flags. Runs are ranked
  by `cd_clock_bits` (v3 tuple; falls back to the v1 scalar for old
  runs).
- `chains` (EXP-L9, v3 §3) censuses every snapshot in a run dir —
  maximal paths of degree-2, generation > 0 nodes — and writes
  `chains.csv`. Needs `[output] snapshots_enabled` and a generation-
  tracking rule family (`[lk_rules] g_max`).

Logging is span-based `tracing`; enable with `RUST_LOG`:

```sh
RUST_LOG=info  hypersim-cli run ...   # run start/end summaries
RUST_LOG=debug hypersim-cli run ...   # + per-window spans
```

**Future work:** a `campaign` subcommand for multi-run sweeps (see §14);
`report` re-rendering from a sealed run directory.

## 6. Configuration reference

A run is one TOML file (see `configs/`). Every section beyond the top
level is optional with the defaults below — config schema stays additive,
so old configs keep parsing as fields are added.

### Top level

| Field | Type | Default | Meaning |
|---|---|---|---|
| `experiment_name` | string | — (required) | Human name; part of derived run ids. |
| `run_id` | string | `""` | Explicit run id. Empty ⇒ `<experiment>_<seed>_<timestamp>`. |
| `out_dir` | path | `"output"` | Parent directory; the run writes into `<out_dir>/<run_id>/`. |
| `seed` | u64 | — (required) | Master seed for generators and the engine RNG. |

### `[rules]`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `source` | `"Builtin"` \| `"File"` \| `"Inline"` | `"Builtin"` | Where rules come from. |
| `builtin_name` | string | — | One of `triangle_subdivide`, `binary_edge`, `tetrahedral`, `glider`, `growth_treadmill`. |
| `file_path` | path | — | Rule TOML on disk (e.g. `rules/wolfram_analog.toml`, `rules/excitation_front.toml` — the LK2 reference dynamic). |
| `inline_toml` | string | — | The rule TOML embedded as a string. |

### `[initial]`

| Field | Type | Default | Used by |
|---|---|---|---|
| `condition_type` | see below | `"PureCrystal"` | all |
| `width`, `height` | usize | 10, 10 | crystals |
| `arity` | usize | 2 | crystals (2 = von Neumann grid, 3 = triangle lattice), `MinimalSeed` |
| `periodic` | bool | false | crystals |
| `defect_density` | f64 | 0.0 | `CrystalWithDefects` |
| `node_count`, `edge_count` | usize | 30, 40 | `WhiteNoise` |
| `max_arity` | usize | 2 | `WhiteNoise`, `SparseSoup` |
| `label_range` | u32 | 3 | random generators (labels drawn from `0..label_range`) |
| `avg_degree` | f64 | 2.0 | `SparseSoup` |

`condition_type` values: `PureCrystal`, `CrystalWithDefects`,
`WhiteNoise`, `SparseSoup`, `MinimalSeed`, `GliderRing`, `TriLattice`,
`TriLatticeExcited`, `Empty`. Use `Empty` together with a scheduled
creation rule (§7.6) to build the world from rules alone.

`TriLattice` is a triangulated torus of `width` × `width` nodes plus
planted shortcut edges at density `defect_density` (the locality-track
initial condition, §12.5); `TriLatticeExcited` is the same with node (0,0)
marked label 1 — the substrate for the excitation-front reference dynamic.

### `[engine]`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `step_mode` | `"MaxParallel"` \| `"SequentialRandom"` \| `"SequentialWeighted"` | `"MaxParallel"` | §7.7 |
| `max_steps` | u64 | 10 000 | Run length. |
| `metrics_window_interval` | u64 | 512 | Steps per evaluation window. |
| `max_pairings_per_step` | usize | 1000 | Cap on accepted *ordinary* rewrites per step (MaxParallel); scheduled creation firings are exempt. **The main growth throttle.** |
| `thread_count` | usize | all cores | Rayon pool size for matching. Results never depend on it. |

### `[evaluator]`, `[causal]`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `evaluator.enabled` | bool | true | Regimes, gates, interestingness, window metrics. |
| `causal.enabled` | bool | true | Tasks, constructors, claims, EI. |
| `causal.wl_radius` | u8 | 1 | **Reserved** — the causal layer currently always uses radius 1. |

### `[causal_language]`, `[ei_coarse_graining]` — the EI/CE calculator

| Field | Type | Default | Meaning |
|---|---|---|---|
| `causal_language.ei_interval_steps` | u64 | 1000 | Emit one `EiResult` per enabled strategy every this many steps. |
| `causal_language.ei_minimum_transitions` | u64 | 1000 | Below this many observed class transitions no EI is emitted and no CE claim is made (Empirical requires ≥ 1000). |
| `ei_coarse_graining.try_structural` | bool | true | Partition classes by exemplar topology (dominant incident arity × degree band). |
| `ei_coarse_graining.try_energetic` | bool | true | Partition by exemplar energy quartile (uniform energy ⇒ K = 1 ⇒ CE = 0, correct). |
| `ei_coarse_graining.try_functional` | bool | true | Partition by constructor-registry role (confirmed / reactant / candidate / inert) — the scientifically central strategy. |
| `ei_coarse_graining.try_optimal` | bool | false | Exhaustive partition search — declared, unimplemented. |

### `[leakage]` — the locality & leakage track (§12.5)

Read-only observers plus offline analysis quantifying how much the
substrate "leaks" into the emergent geometric layer. All quantities are
window-timescale and `[Estimated]` (or `[Inferred]` for the closure
defect); step-level dynamics are unmeasured.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `enabled` | bool | true | Emit `leakage.csv` (detour spectrum + cut scaling per window). |
| `cadence_steps` | u64 | 500 | Steps per leakage emission; align with the EI cadence to share windows. |
| `detour_sample_edges` | usize | 256 | Edges sampled for the detour census. **See the sensitivity floor** (`docs/fixture_calibration.md`): at 256, sparse planted populations (p ≤ 1e-2) do NOT separate from baseline — raise to ≥ the edge count for sparse-regime sensitivity. |
| `detour_r_max` | usize | 12 | Truncated-BFS cap; a pair beyond it makes the edge a wormhole (detour ∞). |
| `ball_sources` | usize | 32 | BFS sources for cut scaling (shares the d_eff ball pass). |
| `cone_events_enabled` | bool | false | Write `causal_events.jsonl` (input to `hypersim-cli cone`, LK2). |
| `cone_sample_events` | usize | 128 | Cone analysis event sample budget (offline). |
| `cone_delta_steps` | u64 | 200 | Cone causal-future depth (offline). |
| `twin_anchors` | usize | 64 | Twin-harness anchor count (`hypersim-cli twin`, LK3). |
| `twin_anchor_radius` | usize | 3 | Twin ball-profile radius. |
| `closure_min_windows` | usize | 200 | Below this many windows the closure defect is emitted with `certainty: Unknown`. |
| `phase_p_peak_source` | string | `"fixture_calibration.md"` | Provenance pointer for the phase thresholds (v2) — a document reference, deliberately not a number. |
| `tracked.enabled` | bool | false | `[leakage.tracked]`: exact lineage tracking of planted shortcuts → `tracked.csv` (EXP-L2 arms). |
| `tracked.recompute_cadence` | u64 | 500 | Steps between exact tracked-set detour recomputations. |

The leakage track adds these commands (§5) and artifacts (§8): `leakage.csv`
always; `cone.csv`/`cone_summary.json` from `hypersim-cli cone`;
`twin.csv` from `hypersim-cli twin`; `closure.json` when the evaluator and
leakage are both on. Reporting integrates a **Locality & Leakage**
subsection (report §8) and the anti-gaslight flags
(`expander_fixture_not_flagged_leaky`, `cut_fit_r2_below_threshold`,
`closure_defect_negative`, plus campaign-level flags from
`hypersim-cli leakage-table`). Full spec: `hypersim_leakage_plan_v1.md`;
calibration: `docs/fixture_calibration.md`; closure field lists:
`docs/leakage_closure_spec.md`.

### `[lk_rules]` — the LK-R energy-metered rule family (v2 §6)

Scaffolding for `rules/lk_pair.toml` (= `lk_metered_subdivide` +
`lk_transport`): initializes energy (property key 1) to 0.0 on every node
and places the seed budget E₀, then audits the exact invariant
`E_total(t) + c·N_subdivisions(t) = E₀` into leakage.csv
(`audit_residual` must be exactly 0.0; drift fires
`audit_invariant_drift` and halts interpretation). The rule constants
(θ, q, c, τ) live in the rule files; keep `c`/`e_total` here in sync.

| Field | Default | Meaning |
|---|---|---|
| `enabled` | false | Seed energy + run the audit. |
| `theta`/`q`/`c`/`tau` | 100/50/10/25 | Mirror of the rule-file constants (audit inputs). |
| `e_total` | 1e6 | Seed budget E₀; era = E₀/(q+c) steps. |
| `seed_placement` | `"center"` | `center`, `corner`, or the v3 §4 encounter placements: `at_tracked_mouth` (seed ON the first planted shortcut's mouth) / `matched_offset` (the node farthest from every mouth — the ambient-timing control). The last two require planted shortcuts (`initial.defect_density > 0`). |
| `priority` | `"subdivide_first"` | v4 §1.2: the priority order is a SWEPT AXIS. `transport_first` remaps the LK rule priorities at load (transport above subdivide; fuse stays last). Recorded in every manifest, with the §1.3 derived flag `children_born_in_dead_zone` (= q ≤ 2τ). |
| `fuse.enabled` | false | v4 §7: append `rules/lk_fuse.toml` (filament reabsorption; generation-guarded, so planted/original edges are non-removable by design). The audit extends to `E + c·N_sub + Σ E_fused = E₀`, exact; leakage.csv gains `e_fused_total` and `churn_events`. |
| `fuse.churn_window_steps` | 20 | Fuse-then-resubdivide at the same site within this window = one churn event (measured, never prevented). |
| `g_max` | unset | v3 §3 generation cap. Setting it initializes generation (key 6) = 0.0 on every IC node — required by the capped rule families (`rules/lk_pair_capped.toml`, `rules/lk_pair_depth_capped.toml`), whose guard silently not-matches on a missing key (hazard 1). The guard VALUE lives in the rule file; keep them in sync. |

Boundedness and sustainedness are IMPOSED by this scaffolding; every
claim from LK-R runs is conditional on the sustained bounded structural
phase (v2 §6.4 — the report repeats this automatically).

### `[campaign.abort]` — Exploding early-abort (v2 §7)

`exploding_windows` (default 2 — v3 §1.3: burst exploders show only 2
Exploding windows; the old default 3 sailed past them) consecutive
Exploding-classified windows
terminate the run with verdict `ExplodingAbort` (manifest `Interrupted`,
flag `exploding_abort_triggered`, window trace in the observations).
Calibration note: burst-type exploders (binary_edge) show only ~2
Exploding windows before reclassifying as GrowthTreadmill — set k ≤ the
burst length (`docs/fixture_calibration.md`).

### `[output]`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `snapshots_enabled` | bool | false | Write a binary store snapshot per window into `snapshots/` (HGSS format, `docs/schema_versions.md`). Required by `hypersim-cli cone`. |
| `append_log_enabled` | bool | false | Append each window's `MetricsRow` to `metrics.hgal` (HGAL format). |
| `fixed_provenance` | bool | false | Test mode: fixed run id + epoch timestamp so whole-file byte comparison is valid (`docs/reproducibility_policy.md`). Not for real runs. |

## 7. Writing rule sets

Full grammar: `docs/rule_grammar.md`. This section is the guided tour.
A rules file is one or more `[[rules]]` tables.

### 7.1 A minimal rule

```toml
[[rules]]
name = "edge_split"          # required, unique within the file
[rules.lhs]
nodes = ["a", "b"]           # pattern variables
edges = [["a", "b"]]         # one 2-ary hyperedge over them
[rules.rhs]
nodes = ["a", "b", "m"]      # a, b survive; m is created
edges = [["a", "m"], ["m", "b"]]
[rules.rhs.node_labels]
m = 2                        # created nodes default to label 0
```

Variables shared between LHS and RHS survive with their `NodeId` intact.
LHS-only variables are deleted (with dangling-edge cleanup); RHS-only
variables are created at the current step's generation.

### 7.2 Label constraints

```toml
[rules.lhs.node_labels]      # sparse map by variable — absent = any label
a = 1

[rules.lhs.edge_labels]      # sparse map by EDGE INDEX into lhs.edges
0 = 7                        # edge 0 must have label 7; others unconstrained

[rules.rhs.edge_labels]      # sparse map by edge index into rhs.edges
1 = 5                        # created edge 1 gets label 5; absent = 0
```

Edge labels are sparse **maps keyed by index**, never parallel arrays —
TOML cannot express a null placeholder inside an array (plan v4 §5.1).

### 7.3 Property guards

```toml
[[rules.lhs.node_property_guards]]
var = "a"
property = 1                 # key 1 = energy by convention (docs/property_registry.md)
op = "gt"                    # eq | ne | lt | gt | lte | gte
value = 0.5                  # TOML literal syntax picks the type: 0.5=Float, 5=Int, true=Bool
```

Semantics are variant-exact: comparing a stored `Int` against a `Float`
threshold makes the guard **false** — no coercion, no panic. A missing
property also fails the guard.

### 7.4 Distinctness guards

Matching is non-injective by default. Opt into distinctness per rule:

```toml
[[rules.lhs.var_guards]]
type = "vars_distinct"
vars = ["a", "b", "c"]       # expands to all pairwise ≠ constraints
```

Symmetric patterns (triangles, cliques) almost always want this;
without it, a triangle pattern will happily match a single doubled edge.

### 7.5 Priority and weight

```toml
priority = 5   # i32, default 0 — higher wins conflicts in MaxParallel
weight   = 2.0 # f64, default 1.0 — sampling weight in SequentialWeighted
enabled  = true
```

Priority is honored inside conflict resolution (sort key
`(Reverse(priority), rule_id, bound nodes)`), **not** by reordering rules:
`rule_id` is always the file position.

### 7.6 Creation rules

A rule with an **empty LHS** is a creation rule and **must** carry a
schedule; a rule with a non-empty LHS **must not**. Both directions are
parse errors.

```toml
[[rules]]
name = "seed_world"
[rules.lhs]
nodes = []
edges = []
[rules.rhs]
nodes = ["x", "y"]
edges = [["x", "y"]]
[rules.schedule]
mode = "once"                # fires at step 0
# or: mode = "every_n_steps" / n = 1000
```

Due creation rules contribute a zero-bound-node binding into the same
candidate pool as ordinary matches; an empty node set can never conflict,
so creation firings are always accepted alongside whatever else fires.

### 7.7 Step modes

- **MaxParallel** — apply every non-conflicting candidate (up to
  `max_pairings_per_step`), deterministic greedy order. The default.
- **SequentialRandom** — one uniformly random candidate per step
  (seeded RNG).
- **SequentialWeighted** — one candidate per step, sampled by rule
  `weight`. Good for rare-event rules like `crystal_battery`'s
  defect rule (weight 0.005).

### 7.8 Pitfalls

- **Match blowup.** Candidate enumeration touches every candidate edge per
  pattern edge per step. Rules like `triangle_subdivide` on a dense
  lattice multiply their own match count each firing — keep
  `max_pairings_per_step` small (≤ 8) or growth turns superlinear fast.
- **Exploding regime.** Node growth ≥ 1.5× per window classifies as
  `Exploding`. That's the classifier being honest, not broken; throttle
  the pairing budget or window length.
- **Relabeling a survivor is not a thing.** `rhs.node_labels` applies to
  *created* nodes only. To change a node's label, delete and recreate it
  (see the label-flip rules in `crates/hg-sim-core/tests/fifty_k_run.rs`).

**Future work:** `property_effects` (write properties from rules — TOML
shape fully specified in `docs/rule_grammar.md`/plan §5.2, engine hook
point already present, lands with `hg-alife`).

## 8. Run outputs

Each run creates `<out_dir>/<run_id>/`:

```
manifest.json         durable status record — the authoritative one
metrics.csv           one row per window (schema: docs/schema_versions.md)
report.md             the 11-section report
leakage.csv           one row per leakage window ([leakage] on; §12.5)
closure.json          closure-defect TUPLE, [Inferred] (evaluator + leakage on)
closure_sparse_only.json  same tuple, Sparse-only stratum (v3 §5.3)
chains.csv            written by `hypersim-cli chains` (v3 §3)
guards.csv            per-window guard eval/reject counters (v4 §1.1; leakage on)
causal_events.jsonl   optional ([leakage] cone_events_enabled; cone input)
cone.csv              written by `hypersim-cli cone`
twin.csv              written by `hypersim-cli twin`
ei_dump.json          raw TPM for the EI verifier (causal on)
snapshots/            optional (HGSS binary store snapshots per window)
metrics.hgal          optional (HGAL append-log of MetricsRow records)
```

The leakage-track files carry a `claim_type` column/field on every row
(`measured` / `estimated` / `inferred`) — the epistemic tag is part of the
schema, never inferred by the reader. Schemas:
`leakage.csv` (v2) = `step, detour_p50, detour_p95, wormhole_frac,
cut_exponent, cut_fit_r2, area_law_defect, d_bfs, d_spec, dim_mismatch,
wormhole_count_est, wormhole_components, phase, cone_status, e_total_now,
n_subdivisions, audit_residual, claim_type`;
`tracked.csv` (v3 §4) = `window, step, tracked_edge_count,
tracked_components, tracked_wormhole_count, tracked_detour_p50,
tracked_detour_p95, throat_length_max, far_nucleus_flag,
mouth_energy_max, t_far_nucleus, t_ambient_front_at_far, claim_type`;
`chains.csv` (v3) = `snapshot_step, chains_count, chain_length_p50,
chain_length_p95, chain_length_max, claim_type`;
`guards.csv` (v4) = `window, rule, guard_id, evals, rejects, claim_type`
(guard_id = `prop{i}` / `cmp{i}` / `deg{i}` by declaration index; guards
short-circuit, so later guards see fewer evals; vars_distinct is a
binding constraint, not a counted guard);
`cone.csv` (v2) = `step, event_id, lhs_size, delta, cone_radius_p95,
anomalous_frac_dynamics, anomalous_frac_cleanup, excluded_descendants,
censor_set_size, cone_status, claim_type`;
`twin.csv` = `window, step, damage_frac, front_radius, v_damage_running,
anchor_attrition, claim_type`.

**manifest.json** carries provenance (seed, thread count, step mode,
version, config hash) and the run-state machine
(`Created → Running → Paused → Completed/Interrupted`,
`docs/run_state_machine.md`). Every transition is committed to disk when
it happens — a crashed process leaves a truthful manifest, and a
re-attached GUI or post-hoc tool reads the manifest, never the (transient)
event stream. A failed run seals as `Interrupted`.

**metrics.csv** starts with a comment line
(`#hypersim_csv_v=1,run_id=...,seed=...,created=...`), then a header
derived from `MetricsRow` field names, then one row per window. Absent
optional values are empty cells. Column meanings and exact formulas:
`docs/metric_formulas.md`; which columns are [computed] vs [reserved] is
defined there too.

**report.md** has 11 canonical sections (plan §11): Manifest, Run
Parameters, Campaign Summary, Gate Status (+ Fixture Truth Table), Regime
Summary, Evaluator Details, Causal Language Summary (+ Constructor
Ecology), Final Metrics Table, Suspicious Metric Flags, Observations,
Recommended Next Actions. Values are epistemically tagged `[Measured]`
(directly counted) or `[Estimated]` (derived/proxy). Every section is
present; a section with nothing to say states why.

## 9. The evaluator: metrics, regimes, gates

Per window, the evaluator computes the **type histogram** (WL radius-1
hash per node) and derives: entropy, dominant-type fraction,
Jensen–Shannon divergence vs. the previous window, new-type-bin rate,
node-churn activity, the **PI proxy** (persisting type-mass overlap —
white noise ≈ 0, frozen structure = 1), and static vs. **active**
persistent structures (same type surviving with *different* node ids —
the glider signature). Exact formulas and all threshold constants:
`docs/metric_formulas.md`.

### The 11 regimes

First match wins, top to bottom (window 0 is always `Unknown`):

| # | Regime | Signature |
|---|---|---|
| 1 | `Exploding` | growth ≥ 1.5×/window or > 100k nodes |
| 2 | `Frozen` | zero churn, zero JSD, entropy > 0 (a crystal) |
| 3 | `StaticOrdered` | zero churn, zero JSD, entropy = 0 (one motif) |
| 4 | `WhiteNoiseSoup` | high JSD, PI < 0.2 (nothing persists) |
| 5 | `SterileChaos` | high churn + JSD, zero novelty |
| 6 | `FragmentedIsolation` | mean degree < 0.5 |
| 7 | `GrowthTreadmill` | growing, zero novelty |
| 8 | `Monoculture` | one type ≥ 90%, active |
| 9 | `ActivePersistentStructure` | movers present, zero novelty |
| 10 | `GenerativeCandidate` | novelty + (movers or growth) |
| 11 | `Unknown` | anything else |

**Interestingness** = 0.30·persistence + 0.40·novelty + 0.30·diversity,
minus penalties for exploding, monoculture, and noise; clamped to [0, 1].
The noise penalty is what guarantees white noise never ranks highest.

### The eight gates

Predicates: `docs/gate_formulas.md`. In brief: **Fixture** (the
six-fixture truth table passes — evaluated at run start), **WhiteNoise**
(noise never scores interesting), **Pi** (PI stable after warmup),
**Generative** (GenerativeCandidate sustained 3 windows with novelty),
**Treadmill** (fails if 8 straight novelty-free treadmill windows),
**Mobile** (a mover observed), **Controller** (reserved — permanently
`Pending` in this build), **Completeness** (every [computed] metric
populated by run end).

`science_unlocked` requires all eight to pass and is therefore
**structurally unreachable** until the cognition layer supplies the
controller gate — intended. Judge substrate runs by the Gate Status
section, not that flag.

### The six fixtures

`hypersim-cli fixtures` runs FrozenGrid, UniformGrowthToy,
MovingGliderToy, StaticBlockToy, WhiteNoiseToy, and SlowGenerativeGrowth
through the same evaluator code as live runs and checks their expected
regimes (`docs/fixture_definitions.md`). If you touch classifier logic or
thresholds, this is your regression net.

## 10. The causal layer

Identity is the WL radius-1 hash: an **attribute** names one structural
class. Per window boundary the layer compares surviving nodes' classes:

- **Task** — an observed class transformation `input → output`.
  Witnessed 3+ times ⇒ `Confirmed` / `Empirical`. Tasks whose outputs
  feed other tasks' inputs form the **task lattice**;
  `task_lattice_depth` is its longest chain.
- **Constructor** — a class that *persists* (same class, same node ids
  across the boundary) within 2 hops of a transformation it plausibly
  catalyzed. Two hops, not one: at radius-1 WL, a direct neighbour of a
  transforming node changes class itself, so a true catalyst is
  necessarily buffered one node away. Constructors accumulate
  confirmed/degraded cycles ⇒ fidelity; population growth while
  persisting flags a **trivial** replicator; certainty climbs
  `Intuition → Empirical (3 confirmed cycles)` and caps there — Law is
  proven by mathematics, never earned by observation.
- **Claims** — subject/predicate/object statements with certainty tags.
  The predicate vocabulary is a closed enum (`persists_across`,
  `transforms_into`, `catalyzes`, `replicates`, `emerges_at`,
  `vanishes_at`) — deliberately no free-form escape hatch. Functional
  predicates get conflict detection.
- **EI / CE** — effective information of the cumulative WL-class
  transition matrix (`EI = H_int − H_noise`; see
  `docs/metric_formulas.md`), at the micro level and under
  coarse-graining partitions. `ce_* = ei_macro / ei_micro`; CE > 1 means
  the macro description has MORE causal power than the micro (causal
  emergence). On bare substrate CE < 1 is the expected null baseline —
  the interesting result is reserved for the ALife condition
  (Experiment 6). Every result records its cadence (steps between
  observations): the number is only meaningful together with it.

Read the report's **Constructor Ecology** subsection as: "these structural
classes stayed themselves while the world changed around them, this
reliably."

## 11. The GUI workbench

```sh
cargo run --release -p hypersim        # live backend (default config baked in)
```

Layout: graph canvas (center), dashboard tabs (right), status bar
(bottom), menu (top).

- **Run menu** — Start / Pause / Resume / Speed / Quit. Pause and stop
  take effect at a step boundary with proper manifest transitions
  (`Running → Paused → Running`, or `→ Interrupted` on quit); closing the
  window mid-run also seals the manifest as `Interrupted` — it is never
  left lying as `Running`. **Speed** throttles the sim to a multiplier of
  a 60 steps/s base (×¼ … ×4, or Max = uncapped, the default); it takes
  effect at the next step boundary and is purely a pacing sleep — it
  cannot affect results, only wall time. The top-bar hz readout shows the
  actual measured step rate per window.
- **Experiment menu** *(live builds)* — start a new run without leaving
  the GUI. Pick a builtin rule set (each maps to a preset mirroring
  `configs/example_*.toml`; adjustable seed and node count), or **Load
  config…** to open any `SimConfig` TOML via a native file dialog. A bad
  file shows a dismissable red error bar — it never crashes the GUI. The
  menu is disabled while a run is `Running`/`Paused` (Quit it or let it
  finish first); starting a new experiment seals the previous run's
  manifest before the new one begins. The science gate is untouched:
  `ScienceLocked` stays exactly as the build dictates.
- **Headless runs** *(live builds, same menu)* — launch the selected
  builtin or any config TOML as a *background* run: its own thread, its
  own output directory, completely independent of the run on the canvas
  (and never gated on it — run several at once). A bottom panel shows
  one row per background run with a progress bar, a Stop button, and the
  final outcome (`Completed`/`Interrupted` + run directory, or the
  error). Closing the workbench stops and seals every background run's
  manifest before exit.
- **View menu / camera** — the canvas has a full camera: **drag** to pan,
  **scroll** to zoom (anchored at the cursor), **`f`** or *View → Fit to
  view* to frame the whole graph. The view auto-follows the growing graph
  until you first pan or zoom; `f` re-frames on demand.
- **Color menu** — node color modes: **L** by label, **E** by energy
  (blue → red), **G** by generation.
- **Tabs** (registry-driven, easy to add — see §13): *Metrics* (node/edge
  counts, PI, interestingness, entropy/JSD plots), *Regime & Gates*
  (current regime, gate table, regime history), *Knowledge Graph* (task
  lattice, constructor ecology, claims).
- **Canvas layout** — an incremental force-directed layout (positions
  persist across frames and relax continuously while the sim runs). New
  nodes seed near their neighbors, so subdivision structure unfolds in
  place instead of shuffling. The layout is GUI-side only and never feeds
  back into the engine, so it cannot affect determinism. Node radius and
  edge width scale with zoom and node count (never fixed pixels), so
  edges stay legible at full-graph zoom; *View → Emphasize edges* is the
  connectivity-comparison mode (thicker edges, dimmer nodes — the view
  for judging tree vs mesh). Graphs over 5 000 nodes reach the canvas as
  a contiguous BFS-sampled fragment (a valid subgraph with its real
  local structure), so what you see stays representative — the top bar's
  step counter and the Metrics tab always reflect the full graph.
- Hovering a canvas node shows its real `NodeId` — the same identifier
  the causal layer and inspector use; there is no GUI-only id scheme.

The mock-data shell (hg-app-api's `MockSimController`, no engine
dependency) is what `hg-gui` compiles to without the `live` feature; it
serves a fixed 12-window fixture and exists for GUI development and tests.

**Current limitations (= future work):** single-step and reset are not
yet implemented (pause/resume/stop are); config files can be loaded but
not edited in the GUI; the force layout runs on the UI thread (fine to a
few thousand nodes; a background layout thread is future work).

## 12. Reproducibility and performance

**Guarantees** (`docs/reproducibility_policy.md`): same `SimConfig` +
same seed ⇒ identical final canonical graph hash, exactly, at any thread
count; CSV data rows cell-wise identical with numeric tolerance (the
schema comment line's `run_id`/`created` differ by design). In practice
this implementation currently achieves *byte-identical* CSV data rows
across thread counts — the determinism test compares them raw.

Mechanisms worth knowing when you extend anything: candidate bindings are
canonically sorted before selection; all float accumulations over hash
maps iterate in sorted-WlHash order (hash-map order is not stable across
processes and float addition is not associative); WL and canonical hashing
use seeded FxHasher, never pointer- or allocation-derived values.

For byte-exact whole-file comparisons in test tooling, set
`output.fixed_provenance = true`.

### 12.5 The locality & leakage track (LK)

A standalone instrument suite that asks: how much does the substrate
*leak* into the emergent geometric layer — i.e. what does the geometric
description miss? It rides existing runs (cadence aligned with the EI
windows) via read-only observers plus offline analysis; it never mutates
the engine. Four instruments, all with epistemic tags in their output:

1. **Detour spectrum** (`leakage.csv`): for a sampled edge, the graph
   distance between its endpoints with the edge removed. Short = geometric;
   beyond `detour_r_max` = a *wormhole*. Reports `detour_p50/p95`,
   `wormhole_frac`.
2. **Boundary-cut scaling** (`leakage.csv`): fit of log(edges straddling a
   BFS ball) vs log(ball size). `area_law_defect = cut_exponent −
   (d_eff−1)/d_eff` — ≈ 0 for clean d-dimensional geometry, → volume
   scaling for trees/expanders. Shares the d_eff ball-BFS pass.
3. **Cone mismatch** (`hypersim-cli cone`): causal reach vs geometric
   distance. `v_eff` (front speed), `anomalous_frac`, with the engine's
   dangling-cleanup exception reported in a **separate channel** so
   bookkeeping never masquerades as substrate non-locality.
4. **Closure defect** (`closure.json`): CD = I(macro(w+1); micro(w) |
   macro(w)) in bits/window via a ridge predictor pair under blocked
   time-series CV. CD ≈ 0 ⇒ the geometric layer is informationally closed;
   CD > 0 ⇒ leakage, in bits. `[Inferred]`; `Unknown` below the window
   floor; negative CD is a flagged estimator pathology, never a result.

The **twin harness** (`hypersim-cli twin`) is the counterfactual
counterpart: branch, perturb one edge, and measure a **label-invariant**
damage front (radius-r WL ball profiles — never NodeIds, which diverge
globally downstream of any perturbation).

**Phase gating (v2)** — `wormhole_frac` is unimodal in shortcut density
(F-LK1): it peaks at p_peak(n, R_max) and collapses past the small-world
crossover where shortcuts become mutually redundant. The detour and cone
channels are therefore valid ONLY in the calibrated sparse phase; every
window carries a `phase` column (sparse/crossover/dense) and cone rows
outside it carry `cone_status = out_of_phase` and no numbers. All EXP
verdicts are phase-plane trajectories in (wormhole_frac,
area_law_defect), never a single scalar's direction. The p_peak curve
and thresholds live in `docs/fixture_calibration.md`.

**Interpretation discipline** — the instruments are read *jointly*, never
singly (a bare ring has high detour without redundancy; only the cut
instrument separates it from a real expander), and every null result is a
finding only relative to the recorded sensitivity floor
(`docs/fixture_calibration.md`: at the default 256-edge detour budget,
sparse planted populations do not separate from baseline). No quantity in
code, CSV, or report text references quantum gravity; the closure defect
and excess mutual information are classical stand-ins, and any holography
framing lives in write-ups only. Full plan: `hypersim_leakage_plan_v1.md`;
findings so far: `docs/leakage_experiment_findings.md`.

**Performance:**

- Use `--release`. Debug is for tests.
- `max_pairings_per_step` is your growth throttle (§7.8).
- Matching cost scales with candidate edges per pattern edge; LHS
  `edge_labels` constraints let the matcher use the label index instead
  of scanning all edges.
- Window cadence sets science cost: WL-hashes every node (sampled above
  4096 nodes), so very short windows on very large graphs dominate
  runtime.
- Leakage cost is dominated by the detour census: `detour_sample_edges`
  truncated BFS per emission window. The ball-BFS for cut scaling is
  shared with d_eff (one pass). Raising `detour_sample_edges` toward the
  edge count for sparse-regime sensitivity is the main leakage-side knob;
  the cone and twin analyses are offline and do not affect run throughput.

## 13. Extending Hypersim

The architecture is built around seams. The ones that exist today:

**Add an Observer** (watch the run without touching the engine):
implement `hg_observe::Observer` (`on_step_end`, `on_window_complete`,
`on_run_end` — all read-only `GraphSnapshot`), then
`engine.add_observer(Box::new(yours))` or drive it from `hg-sim-core`'s
window loop like the evaluator. Depend on `hg-core` + `hg-observe` only.

**Add a RuleSelectionPolicy** (bias *which* candidates fire — the
neural-guidance seam): implement `hg_observe::RuleSelectionPolicy::select`
returning indices into the candidate slice; install with
`engine.set_policy(..)`. The conflict resolver still runs after you, so a
policy can never violate node-disjointness.

**Add a MotifClassifier** (richer attribute classes):
implement `hg_causal::MotifClassifier`, install via
`CausalObserver::with_classifier`. `NullMotifClassifier` is the default.

**Add a builtin rule set:** drop the TOML in `rules/`, add its
`include_str!` arm in `crates/hg-rules/src/builtins.rs`, extend
`BUILTIN_NAMES`, and the parse-and-compile test picks it up automatically.

**Add a metric:** field on `hg-schema::MetricsRow` (additive — CSV header
derives from field names) → compute in `hg-eval::window` or `hg-causal`
→ populate in `hg-sim-core::runner::assemble_row` → **document the
formula in `docs/metric_formulas.md`** and mark it [computed]. If the
completeness gate should require it, list it there.

**Add a gate / regime:** gates live in `hg-eval::gates` with names in
`hg-schema::GateName`; regimes in `hg-schema::Regime` +
`hg-eval::classify` (mind the priority order). Both have a documentation
contract: `docs/gate_formulas.md` / `docs/metric_formulas.md` first, code
second, and the fixture truth table must still pass.

**Add a GUI tab:** push a `(&'static str, TabRenderer)` into
`hg-gui::tabs::TabRegistry::default` — tabs are registry-driven.

**Add a report section:** extend `hg-schema::RunSummary` (additive),
render in `hg-report::writer`, fill in `hg-sim-core::runner::build_summary`,
and update the section list test.

House rule inherited from the plan: **docs are load-bearing.** Every
constant a fixture or gate references must trace to
`docs/metric_formulas.md`; if implementation and spec disagree, fix one
and say which, in the doc itself.

## 14. Roadmap and reserved seams

Stubs below are placeholders — replace each with a real chapter/section
when the feature lands.

### 14.1 `hg-alife` *(reserved — not yet implemented)*
Energy dynamics and `property_effects` (the apply-time hook point already
sits between RHS node creation and edge creation), replicator ecology on
top of the constructor registry, `MotifClassifier` implementations.
Cargo feature on `hg-sim-core`/`hg-gui`, mirroring `hg-engine/parallel`.

### 14.2 `hg-cognition` *(reserved — not yet implemented)*
`RuleSelectionPolicy` implementations (learned or scripted controllers),
the **controller gate** going live (which finally makes
`science_unlocked` reachable), controller-nudge audit trail in the
manifest and report.

### 14.3 Campaigns *(not yet implemented)*
Multi-run sweeps (seed ranges, config grids), a real Campaign Summary
report section, cross-run comparison tooling. Until then:
`for seed in 1 2 3; do hypersim-cli run --config c.toml --seed $seed; done`.

### 14.4 GUI: run control and config editing *(partially implemented)*
Mid-run pause/resume/stop work (RunControl, polled at step boundaries
with durable manifest transitions), experiments can be started from the
GUI (builtin presets + TOML loading via `rfd`, with camera pan/zoom/fit
and a force-directed canvas layout), the speed throttle works
(`SetSpeed` → RunControl, paced at step boundaries), and headless
background runs can be launched and stopped from the Experiment menu.
Still open: single-step and reset; config *editing* in the GUI
(`syntect` is in the workspace for this); moving the force layout to a
background thread for very large graphs; snapshot scrubbing using the
HGSS files.

### 14.5 Reserved metrics *(defined, always empty today)*
`spectral_dim`, `diameter` (graph-geometry; `d_eff` is now implemented —
ball-growth fit, see docs/metric_formulas.md), `ei_micro` +
`ce_structural`/`ce_energetic`/`ce_functional` (micro-scale EI and
causal-emergence deltas). Formula slots are reserved in
`docs/metric_formulas.md`.

### 14.6 Post-hoc tooling *(not yet implemented)*
Snapshot loader (HGSS → store) for replay/inspection, `metrics.hgal`
readers, report regeneration from a sealed run dir.

---

*Maintaining this guide: keep chapters aligned with the numbered spec
docs, add new chapters at the end of the relevant part rather than
renumbering, and record notable behavior changes in the chapter's
**Future work** block until they get their own section.*
