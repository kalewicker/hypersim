# Evaluator Fixture Definitions

This document defines the six canonical fixtures that the evaluator (Phase 5) must implement and pass. These fixtures are pure Rust structs that generate `GraphSnapshot` sequences — they do not require the full engine.

Each fixture produces a sequence of evaluation windows (default 512 steps each) with specific properties. The evaluator must classify each into the expected regime.

## Fixture 1: FrozenGrid

**Description**: A regular grid graph; no edges change between snapshots.

**Construction**:
- Generate a 2D grid of nodes (e.g., 10×10 = 100 nodes)
- Connect each node to its 4 neighbors (von Neumann neighborhood) with 2-ary edges
- All nodes label 0, all edges label 0
- Snapshots are identical for all windows

**Expected regime**: `Frozen`

**Key metrics**:
- `accepted_rewrites` = 0 for all windows
- `activity` = 0
- `ce_structural` ≈ 0
- `type_histogram_jsd` = 0 (only one structure type)
- `new_type_bin_rate` = 0

## Fixture 2: UniformGrowthToy

**Description**: One new triangle added per window; structure type never changes.

**Construction**:
- Start with a single triangle (3 nodes, 1 3-ary edge, label 0)
- Each window: add one new triangle sharing one edge with existing structure
- All triangles have same WL hash (same label, same topology)
- Growth is linear: 1 triangle per window

**Expected regime**: `GrowthTreadmill`

**Key property**: `new_type_bin_rate` = 0 (no new structure types appear)

**Key metrics**:
- `n_nodes` increases by 3 per window
- `n_edges` increases by 1 per window
- `type_histogram_entropy` = 0 (single type)
- `dominant_type_fraction` = 1.0
- `active_persistent_structures` = 0 (nothing moves)
- `static_repeated_motifs` > 0 (triangles at fixed positions)

## Fixture 3: MovingGliderToy

**Description**: A 3-node cluster that shifts one edge position per window, preserving its WL hash but at new node IDs.

**Construction**:
- Create a "glider" pattern: 3 nodes in a line with two 2-ary edges (A-B, B-C)
- Each window: delete the trailing edge, add a new leading edge, shift the cluster
- Node IDs change (old nodes deleted, new nodes created) but WL hash of the pattern is invariant
- The pattern translates through empty space

**Expected regime**: `GenerativeCandidate` or `ActivePersistentStructure`

**Key property**: `active_persistent_structures` > 0, `displacement` > 0

**Key metrics**:
- `n_nodes` roughly constant (3 created, 3 deleted per window)
- `active_persistent_structures` ≥ 1
- `mobile_persistent_structures` ≥ 1 (if tracked separately)
- `displacement` > 0 (center of mass moves)
- `ce_structural` > 0 (macro has higher EI than micro)

## Fixture 4: StaticBlockToy

**Description**: A 4-node clique that never changes.

**Construction**:
- 4 nodes, all 6 possible 2-ary edges present (complete graph K4)
- All nodes label 0, all edges label 0
- Snapshots identical for all windows

**Expected regime**: `StaticOrdered` or `FrozenStructured`

**Key property**: `active_persistent_structures` = 0 (nothing moves)

**Key metrics**:
- `n_nodes` = 4, `n_edges` = 6 constant
- `activity` = 0
- `static_repeated_motifs` > 0 (the K4 itself)
- `active_persistent_structures` = 0
- `static_persistent_motifs` > 0

## Fixture 5: WhiteNoiseToy

**Description**: Node labels and edge endpoints randomize each window.

**Construction**:
- Fixed number of nodes (e.g., 50)
- Each window: assign random labels (0-9), rewire all edges randomly
- Or: generate completely new random graph each window
- Seed the RNG differently per window for true randomness

**Expected regime**: `WhiteNoiseSoup` or `SterileChaos`

**Key invariant**: Must **never** rank highest in interestingness score

**Key metrics**:
- `activity` high
- `entropy_rate` high
- `pi_normalized` near 0 (no causal structure)
- `ce_structural` ≈ 0
- `type_histogram_entropy` high (many types)
- `interestingness_score` must be lower than Fixtures 2, 3, 4, 6

## Fixture 6: SlowGenerativeGrowth

**Description**: Steady growth adding one triangle per window, BUT each new triangle has a unique label drawn from an incrementing counter, so `new_type_bin_rate` > 0 always.

**Construction**:
- Start with one triangle label 0
- Each window: add one new triangle with label = window_id (monotonically increasing)
- Triangles are disconnected from each other (or minimally connected)
- Each triangle has unique WL hash due to unique label

**Expected regime**: `GenerativeCandidate`, NOT `GrowthTreadmill`

**Key property**: Distinguishes from Fixture 2 via `type_histogram_jsd` > 0

**Key metrics**:
- `n_nodes` increases by 3 per window
- `new_type_bin_rate` > 0 (new label each window)
- `type_histogram_entropy` increases over time
- `type_histogram_jsd` > 0 (distribution changes)
- `interestingness_score` > Fixture 2 (treadmill)
- `gate_generative` = PASS, `gate_treadmill` = FAIL

## Truth Table Gate

All six fixtures must pass before Phase 6 (hg-causal) begins. The test is:

```rust
#[test]
fn fixture_truth_table() {
    let results = vec![
        run_fixture(FrozenGrid)           == Regime::Frozen,
        run_fixture(UniformGrowthToy)     == Regime::GrowthTreadmill,
        run_fixture(MovingGliderToy)      matches Regime::GenerativeCandidate | Regime::ActivePersistentStructure,
        run_fixture(StaticBlockToy)       == Regime::StaticOrdered,   // FrozenStructured dropped in v4 §7
        run_fixture(WhiteNoiseToy)        matches Regime::WhiteNoiseSoup | Regime::SterileChaos,
        run_fixture(SlowGenerativeGrowth) == Regime::GenerativeCandidate,
    ];
    assert!(results.iter().all(|r| *r), "Fixture truth table failed: {:?}", results);
}
```

Additionally, the white noise invariant:

```rust
#[test]
fn white_noise_never_wins_interestingness() {
    let scores = vec![
        ("FrozenGrid", run_fixture(FrozenGrid).interestingness),
        ("UniformGrowth", run_fixture(UniformGrowthToy).interestingness),
        ("MovingGlider", run_fixture(MovingGliderToy).interestingness),
        ("StaticBlock", run_fixture(StaticBlockToy).interestingness),
        ("WhiteNoise", run_fixture(WhiteNoiseToy).interestingness),
        ("SlowGenerative", run_fixture(SlowGenerativeGrowth).interestingness),
    ];
    let (best_name, _) = scores.iter().max_by_key(|(_, s)| *s).unwrap();
    assert_ne!(best_name, "WhiteNoise", "White noise ranked highest!");
}
```

## Implementation Notes

- Fixtures live in `hg-eval` crate as inline Rust code (not TOML configs)
- They implement a `Fixture` trait: `fn windows(&self) -> Vec<GraphSnapshot>`
- Each produces at least 10 windows for statistical stability
- The evaluator runs on these windows exactly as it would on real simulation output
- No engine, no rules, no RNG (except WhiteNoiseToy which needs controlled randomness)

## Minimal GraphSnapshot for Fixtures

Fixtures only need to populate the fields the evaluator reads:
- `nodes: Vec<NodeSnapshot>` — id, label, generation, properties, energy
- `edges: Vec<EdgeSnapshot>` — id, label, node_ids (Vec<NodeId>)
- `step: u64` — the physics step this snapshot represents

The evaluator computes WL hashes, types, persistence, etc. from this.