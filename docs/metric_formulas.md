# Metric Formulas

Canonical definitions for every metric in `MetricsRow`. Every numeric
constant a fixture or gate references traces to a formula here. Fields are
marked **[computed]** (substrate pipeline produces them every window) or
**[reserved]** (defined, but `None`/`NaN` until a later layer needs them —
excluded from the completeness gate).

Windows: metrics are computed per evaluation window of `window_len` steps
(default 512; fixtures use 1 snapshot = 1 window). `t` denotes the current
window, `t-1` the previous. If `max_steps` is not a multiple of
`window_len`, the trailing steps form a **partial final window**: it is
emitted as a normal CSV row (per-step ratios divide by its actual length),
never silently dropped.

## Type histogram (the basis for most metrics)

For each live node `v`, its **type** is `WL(v, r=1)` — the deterministic
Weisfeiler-Leman neighbourhood hash at radius 1 (see
`docs/reproducibility_policy.md` for hash determinism requirements).
The **type histogram** `H_t` maps each distinct WL hash (a "type bin") to
its node count. `p_t(x) = count_t(x) / n_nodes_t`.

**WL identity is label + energy + structure — never the properties map.**
A node's base hash folds in its `Label`, its `energy` `f32` field, and its
neighbourhood; the per-key `properties` map is deliberately excluded. This is
the ALife Q2 invariant: continuous, per-individual genome traits (and ALife
energy, stored in property key 1) must NOT mint a fresh WL class per carrier,
or the class registry grows without bound and `ei_micro = log2(M)` (§ EI, and
the open-registry pathology below). The invariant is positive and un-rottable —
*any* property write leaves every WL hash unchanged — and is pinned by
`hg_core::hash::tests::wl_hash_invariant_under_property_change`. Anyone adding a
property key gets this for free and need not touch the hasher.

## Substrate-only metric variants (hg-alife S0.2) [computed]

When carriers are hypergraph nodes (the intrinsic ALife design), they enter
`n_nodes`, `d_eff`, and the regime classifier. A carrier bloom would then read
as the *substrate* `Exploding` (the classifier fires on `growth_ratio`/
`n_nodes`). To keep substrate metrics attributable to substrate physics, hg-eval
provides label-filtered variants (`hg_eval::substrate`):

- `substrate_subgraph(snapshot, carrier_label)` builds the substrate subgraph as
  a standalone store — every node NOT bearing `carrier_label`, plus every edge
  all of whose endpoints survive (occupancy edges drop out). Labels, energy,
  generation, and properties are preserved, so substrate WL classes are faithful.
- `substrate_structural(...)` → `{n_nodes, n_edges, mean_degree, d_eff}` and
  `substrate_regime(computer, ...)` → the regime, both by running the EXISTING
  metric functions on the filtered store (no metric logic duplicated). The
  substrate regime threads its OWN `WindowComputer`, so growth/novelty are
  measured on the substrate's history, never contaminated by carrier births.

Invariant (tested, `hg-eval/tests/substrate_metrics.rs`): these are independent
of the number of carriers attached, and a carrier bloom that reads `Exploding`
on the full graph does not on the substrate. hg-eval stays ALife-agnostic — the
carrier label is a parameter; `hg-sim-core` supplies `hg_alife::CARRIER_LABEL`.

These are now **emitted** per window to a SEPARATE `substrate.csv` (like
`ecology.csv`/`leakage.csv`, so `metrics.csv` is byte-identical without the
feature/flag; tested `alife_substrate_control.rs`). Columns (`hg_schema::SubstrateRow`):
`window_id, step_end, substrate_n_nodes, substrate_n_edges, substrate_mean_degree,
substrate_d_eff, substrate_regime`, plus the same-window `full_n_nodes` and
`full_regime` so the carrier pollution is visible at a glance. The runner threads a
substrate-dedicated `WindowComputer` (and a paired full-graph one) across windows
so substrate growth/novelty are measured on the substrate's own history. The
per-window `d_eff` seed matches the full-graph path, so the two `d_eff` values are
directly comparable.

**Paired bare-substrate control (S0.2).** `alife_substrate_control.rs` runs an
ALife config and a same-seed control with `[alife] enabled = false`, and asserts
the ALife run's `substrate_n_nodes`/`substrate_n_edges` equal the bare control's
full-graph `n_nodes`/`n_edges` **window by window** — i.e. carriers do not perturb
substrate physics, so substrate metrics stay attributable. (The current biology
runs a static substrate, so the control is exact; with an active substrate this
test would detect carrier back-reaction on the substrate.)

## `ecology.csv` — the ALife per-window emission (hg-alife S5, `alife` feature)

An ALife run (`[alife] enabled = true`, `alife` feature) writes a SEPARATE
`ecology.csv` beside `metrics.csv`, one row per metrics window — the same
separate-file discipline as `leakage.csv`, so `metrics.csv` stays byte-identical
without the feature/flag (exit criterion 7; tested in `alife_emission.rs`). It is
produced by `hg_alife::AlifeObserver`, which reconstructs the lineage and ecology
from the rewrite event stream alone (`replicate`/`bootstrap_seed`/`die_and_recycle`).

Columns (`hg_alife::EcologyRow`):
- `window_id, step_end` — window index and its final step.
- `carriers, substrate_nodes, carrier_ratio` — live population and carriers /
  substrate-node ratio (0.283 is a comparison datum, not a threshold).
- `births, deaths` — this window; `bootstrap_total, replication_total` — cumulative
  by origin (spontaneous-replication evidence); `cumulative_deaths`.
- `max_generation, clade_count` — lineage depth and distinct founders.
- `non_trivial_replicators` — the criterion-6 quantity: carriers that produced ≥1
  offspring AND sustain a lineage ≥ `NON_TRIVIAL_MIN_GENERATIONS` (=2) deep. 0 on a
  bare substrate by construction; goes 0→>0 with the biology.
- `{harvest_rate,repl_threshold,maint_cost,satiation}_{mean,var}` — per-trait mean
  and variance over the live carriers (`NaN` when the population is empty). A
  directional shift here across windows is Experiment-5 criterion 5 (selection).

## Structural counts [computed]

- `n_nodes`, `n_edges` — live counts at window end.
- `mean_degree` = `Σ_v |incident_edges(v)| / n_nodes` (`None` if
  `n_nodes = 0`).
- `d_eff` **[Estimated]** — effective dimension by ball growth: sample
  S = 32 sources (ChaCha8 seeded from `run_seed XOR window_id·φ64` — never
  thread_rng), BFS each to radius R_max = 10 with an N-ary hyperedge
  making all endpoints pairwise distance 1, average N(r) across sources,
  then least-squares slope of `log N(r)` vs `log r` over r ≥ 1 excluding
  saturated radii (mean ball ≥ 0.9·n_nodes) and stalled growth. `None`
  when fewer than 3 usable points (graph too small/degenerate;
  n_nodes < 8 short-circuits). Disconnected graphs estimate from each
  source's reachable ball — global connectivity is not required.
  Reference values: 1D ring ≈ 1, 2D grid ≈ 2, binary_edge filament
  ≈ 0.9, triangle_subdivide sheet ≈ 2.5 and climbing while it fills.
- `spectral_dim`, `diameter` — **[reserved]**.

## Activity [computed]

- `accepted_rewrites`, `attempted_matches` — summed over the window's steps.
- `accepted_per_step` = `accepted_rewrites / window_len` (the rewrite
  throughput column; for a partial final window, divides by the actual
  number of steps in that window).
- `activity` = **node-identity churn**: `|sym-diff(live NodeIds_t,
  live NodeIds_{t-1})| / |union|` ∈ [0, 1]; `0.0` ⟺ the two snapshots
  contain exactly the same nodes. This is the universal definition — the
  same for fixture windows and engine runs — and is what the regime
  classifier's activity thresholds refer to. It is deliberately NOT
  rewrite throughput: an in-place relabel churns ids without high
  throughput and vice versa; use `accepted_per_step` for throughput.
- `activity_cv` = coefficient of variation (σ/μ) of per-step
  `accepted_rewrites` within the window; `0.0` when μ = 0.

## Diversity and novelty [computed]

- `type_histogram_entropy` = `-Σ_x p_t(x) · log2 p_t(x)` (0 when one bin).
- `dominant_type_fraction` = `max_x p_t(x)`.
- `type_histogram_jsd` = Jensen–Shannon divergence (log base 2, range
  [0, 1]) between `p_t` and `p_{t-1}`; `0.0` for the first window.
- `new_type_bin_rate` = `|bins in H_t never seen in any earlier window of
  this run| / window_len`. First window: all bins are new by definition, so
  the classifier treats window 0 as warm-up (regime `Unknown`).
- `entropy_rate` = `|type_histogram_entropy_t − type_histogram_entropy_{t-1}|`;
  `0.0` for the first window.

## PI proxy (persistent information) [computed]

PI measures how much type-mass **persists** across consecutive windows —
white noise scores near 0, frozen structure scores 1:

```
overlap_t = Σ_x min(p_t(x), p_{t-1}(x))      ∈ [0, 1]
pi_normalized = overlap_t                     (None for the first window)
```

- `pi_sample_size` = number of nodes hashed (all nodes when
  `n_nodes ≤ pi_sample_cap = 4096`, else a deterministic
  every-k-th-canonical-node sample of 4096).
- `pi_sample_strategy` = `"full"` or `"stride"` accordingly.

## Persistence structures [computed]

For each type bin present in both `t-1` and `t`, compare the **member
NodeId sets** `M_{t-1}(x)`, `M_t(x)`:

- **static persistent**: `M_t(x) = M_{t-1}(x)` (identical ids) —
  counted in `static_persistent_motifs`.
- **active persistent**: bin persists but `M_t(x) ≠ M_{t-1}(x)` (the
  structure type survived while its constituent nodes changed — e.g. a
  glider translating) — counted in `active_persistent_structures`.
- `static_repeated_motifs` = number of bins in `H_t` with count > 1.

## Causal layer [computed by hg-causal]

- **EI (effective information)** over the WL-class transition matrix.
  `counts[i][j]` accumulates transitions of nodes that exist at BOTH
  observation points with the same `NodeId` (created/deleted nodes are
  not transitions; self-transitions ARE counted — diagonal mass).
  Observations are window-to-window (where WL classes are computed), so
  the TPM describes window-timescale dynamics; the cadence in steps is
  recorded in every `EiResult` because it is part of the measurement's
  definition. With the row-stochastic TPM over the `M` live rows (rows
  with zero count are excluded):
  - `interventional = uniform(1/M) @ TPM`; `H_int = H(interventional)`
  - `H_noise = mean over rows of H(row)` (Shannon, bits, `0·log2 0 = 0`)
  - `EI = H_int − H_noise`
  - `determinism = 1 − H_noise / log2(M)`, `degeneracy = 1 − H_int /
    log2(M)`; identity: `EI = (determinism − degeneracy) · log2(M)`.
  - `M = 1` guard: EI = determinism = degeneracy = 0.
  - No result (and no knowledge-graph claim) below
    `ei_minimum_transitions` (default 1000) observed transitions.
- `ei_micro` = EI of the raw WL-class TPM. `ei_macro` = EI of the
  coarse-grained macro TPM: `T_macro[A][B] = (1/|A|) Σ_{i∈A} Σ_{j∈B}
  T_micro[i][j]` (average over live source-group members keeps macro
  rows row-stochastic).
- `ce_* = ei_macro / ei_micro` (0 when `ei_micro ≤ 0` — white noise
  yields CE 0, not ∞). CE > 1 = causal emergence; NOT expected on bare
  substrate (correct null baseline). Strategies: `ce_structural`
  (partition by exemplar dominant incident arity × degree band),
  `ce_energetic` (energy quartiles — collapses to K = 1 → CE = 0 on
  uniform-energy substrate, correct), `ce_functional` (constructor
  registry: confirmed ≥ 0.9 fidelity & ≥ 10 cycles / reactant < 0.5 /
  candidate / inert — the constructor-experiment strategy; empty on ALife
  runs), `carrier_substrate` (the **Experiment-6** partition: group WL
  classes by `motif_class`, substrate 0 / carrier ≠0; compact K, so it is
  degenerate K = 1 → CE = 0 when no carrier classes exist and a genuine
  K = 2 once carriers appear — off by default, enabled by
  `ei_coarse_graining.try_carrier_substrate`), optimal (declared,
  unimplemented, off by default). `ce_best`/`ce_best_strategy` = max across
  enabled strategies. One result set per strategy per `ei_interval_steps`
  (default 1000). NB `carrier_substrate` is surfaced via the summary
  `EiResultRow` + `ei_dump.json`, NOT a `metrics.csv` column (bit-identical
  schema). See `docs/experiment6_findings.md` for the key caveat: over the
  surviving-node TPM the carrier/substrate macro TPM is the identity, so
  `EI_macro` is pinned to log₂(K) and CE reduces to 1/`EI_micro`.
### EI on open-ended class registries (measured 2026-07-09)

- On a substrate whose WL-class registry GROWS (churned neighborhoods
  mint fresh classes), each class is typically observed as a source
  exactly once at window cadence → every TPM row is one-hot →
  determinism ≡ 1.0 and **EI_micro ≈ log2(M) regardless of whether the
  dynamics are ordered or random**. Measured: white noise
  (growth_treadmill on WhiteNoise, SequentialRandom), 304 classes,
  EI_micro = 8.25 bits = log2(304), determinism exactly 1.0, zero
  multi-destination rows.
- The "EI ≈ 0 for white noise" prediction holds ONLY for fixed state
  spaces (the `ei_white_noise_is_zero` unit test pins that case
  correctly). Scope it accordingly.
- **CE bias direction:** macro groups recur (real transition
  multiplicity) while micro classes are one-shot (inflated to ceiling),
  so CE is biased DOWNWARD on open registries. Consequences: (1) the
  null baselines (CE < 1 on crystal and noise) are extra-trustworthy;
  (2) Experiment 6's CE > 1 bar is conservative — a positive result is
  strengthened, but a real effect could be masked (false-negative
  risk). Revisit the measurement definition (e.g. a
  recurrent-class-restricted micro TPM) only if Experiment 6 nulls out
  AND this bias is the suspect — not preemptively.
- `wl_class_count` = M(t), the registry size at each EI emission — the
  novelty-accumulation curve (Hash Chemistry quantity: linear vs
  accelerating class minting). EI_micro stability is equivalent to
  registry saturation on these substrates.

- `task_lattice_depth` = longest path length (in tasks) of the task
  lattice DAG; 0 when empty.
- `constructor_count`, `replicator_count` — from the constructor registry.

## Novelty archive [computed]

- `archive_size`, `archive_capacity` (default 10 000), `archive_evictions`
  (FIFO), `new_signatures` = signatures first seen this window. A
  signature is a type bin's WL hash.

## Interestingness [computed]

Weighted sum with penalties, clamped to [0, 1]:

```
diversity  = type_histogram_entropy / log2(max(2, n_type_bins))   ∈ [0,1]
novelty    = min(1.0, new_type_bin_rate · window_len / max(1, n_type_bins))
persistence= pi_normalized (0 if None)

raw = 0.30·persistence + 0.40·novelty + 0.30·diversity
    − 0.50·[growth_ratio ≥ 1.5]          (exploding penalty)
    − 0.30·[dominant_type_fraction ≥ 0.9] (monoculture penalty)
    − 0.40·[pi_normalized < 0.2]          (noise penalty: nothing persists)

interestingness_score = clamp(raw, 0, 1)
```

`growth_ratio` = `n_nodes_t / max(1, n_nodes_{t-1})`.

The noise penalty is what enforces the fixture invariant that WhiteNoiseToy
must never rank highest: its persistence term is ≈ 0 **and** it pays the
0.40 penalty, capping it below any persistent-structure fixture.

## Threshold constants (single source of truth)

| Constant | Value | Used by |
|---|---|---|
| `growth_explode_min` | 1.5 | Exploding regime, exploding penalty |
| `node_cap` | 100 000 | Exploding regime |
| `growth_min` | 1.01 | GrowthTreadmill regime |
| `new_type_bin_rate_min` | 0.01 | novelty boundary: "invents nothing" regimes require `rate <= min`, GenerativeCandidate requires `rate > min`. NOT 0: WL radius-1 classes are degree-sensitive, so a growing hub mints ~1 fresh bin per WINDOW (constant per window, so short windows inflate this per-step rate — use window >= 512 when watching treadmills) |
| `jsd_noise_min` | 0.15 | WhiteNoiseSoup, SterileChaos |
| `pi_noise_max` | 0.2 | WhiteNoiseSoup, noise penalty |
| `activity_chaos_min` | 0.5 | SterileChaos |
| `mean_degree_frag_max` | 0.5 | FragmentedIsolation |
| `dominant_monoculture_min` | 0.9 | Monoculture, monoculture penalty |
| `interestingness_noise_max` | 0.5 | white-noise gate |
| `sustain_windows` | 3 | generative gate |
| `treadmill_stuck_windows` | 8 | treadmill gate |
| `pi_warmup_windows` | 4 | PI gate |
| `pi_jump_max` | 0.5 | PI gate |
| `min_transitions` (EI) | 32 | ei_macro |
| `pi_sample_cap` | 4096 | PI sampling |

## Regime classification (first match wins, evaluated top to bottom)

Window 0 of a run is always `Unknown` (no `t-1` to compare against).

1. `Exploding` — `growth_ratio ≥ growth_explode_min` **or** `n_nodes > node_cap`.
2. `Frozen` — `activity = 0` ∧ `type_histogram_jsd = 0` ∧
   `type_histogram_entropy > 0` ∧ `active_persistent_structures = 0`
   (an unchanging lattice whose boundary induces multiple WL classes —
   a crystal). The APS condition keeps movers out: a pure-relabel glider
   on a symmetric ring has zero node churn AND an identical window
   histogram (jsd = 0), yet its mark class persists with changing
   members — that is motion, not freeze.
3. `StaticOrdered` — `activity = 0` ∧ `type_histogram_jsd = 0` ∧
   `active_persistent_structures = 0` ∧
   `type_histogram_entropy = 0` (a single unchanging equivalence class —
   one coherent motif).
4. `WhiteNoiseSoup` — `type_histogram_jsd ≥ jsd_noise_min` ∧
   `pi_normalized < pi_noise_max`.
5. `SterileChaos` — `activity ≥ activity_chaos_min` ∧
   `type_histogram_jsd ≥ jsd_noise_min` ∧ `new_type_bin_rate ≤ new_type_bin_rate_min`.
6. `FragmentedIsolation` — `mean_degree < mean_degree_frag_max`.
7. `GrowthTreadmill` — `growth_ratio ≥ growth_min` ∧
   `new_type_bin_rate ≤ new_type_bin_rate_min`. Checked before Monoculture: a growing
   single-type system is the treadmill archetype, not a monoculture that
   happens to grow.
8. `Monoculture` — `dominant_type_fraction ≥ dominant_monoculture_min` ∧
   `activity > 0`.
9. `ActivePersistentStructure` — `active_persistent_structures ≥ 1` ∧
   `new_type_bin_rate ≤ new_type_bin_rate_min` (it moves, but invents nothing new).
10. `GenerativeCandidate` — `new_type_bin_rate > new_type_bin_rate_min` ∧
    (`active_persistent_structures ≥ 1` ∨ `growth_ratio ≥ growth_min`):
    novelty carried by either a moving persistent structure or genuine
    growth (SlowGenerativeGrowth grows new types without anything moving).
11. `Unknown` — anything else.

`ActivePersistentStructure` sits immediately before `GenerativeCandidate`
(§7 of the v4 plan): the two are distinguished solely by novelty, so a
glider that also introduces new type bins clears the higher bar and is
classified `GenerativeCandidate` via rule 10 (rule 9's novelty-free
condition excludes it).
