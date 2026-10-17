# Reproducibility Policy Specification

**Requirement:** Same `SimConfig` + same `seed` → identical final graph state, identical CSV (byte-for-byte within floating-point tolerance), identical report. This must hold regardless of thread count.

## Mechanisms Required

### Mechanism 1: RNG Seeding

- Use `rand_chacha::ChaCha8Rng` seeded from `config.seed` (u64)
- All RNG use in the engine derives from this single root RNG
- Sub-RNGs for parallel workers are derived via deterministic scheme:
  ```rust
  // For rule-specific RNGs
  let rule_rng = ChaCha8Rng::seed_from_u64(root_seed.wrapping_add(rule_id as u64));
  // For worker-specific RNGs in Rayon
  let worker_rng = ChaCha8Rng::seed_from_u64(root_seed.wrapping_add(worker_index as u64));
  ```

### Mechanism 2: WL Hash Determinism

- Use `rustc_hash::FxHasher` (deterministic, not randomized) for WL neighborhood hashing
- **Do not use** `AHasher` (randomizes seed on program start) or `DefaultHasher` (randomized since Rust 1.36)
- Sort neighbor hash vectors **before** combining. This is required regardless of internal map ordering.

```rust
fn compute_wl_hash(node: &NodeData, store: &HypergraphStore, radius: u8) -> u64 {
    let mut hasher = FxHasher::default();
    // ... compute neighborhood ...
    // CRITICAL: sort neighbor hashes before combining
    neighbor_hashes.sort_unstable();
    for h in neighbor_hashes {
        h.hash(&mut hasher);
    }
    hasher.finish()
}
```

### Mechanism 3: Match Canonicalization

Before conflict resolution, sort all collected matches by a canonical key:

```rust
MatchKey = (rule_id: u32, node_ids: SmallVec<[u32; 8]> sorted ascending)
```

Process matches in this order. This makes conflict resolution deterministic even when Rayon collects matches in non-deterministic thread order.

### Mechanism 4: Conflict Resolution Order

Greedy conflict resolution: iterate matches in canonical key order. Accept match if its LHS nodes are all still unmodified. This is deterministic given Mechanism 3.

### Mechanism 5: Report Ordering

- CSV rows are written in `window_id` ascending order (they are written as produced, which is already monotone)
- `MetricsHistory` is a `Vec<MetricsRow>`, appended in order — no sorting needed
- KnowledgeGraph claims are appended in step order

### Mechanism 6: Graph Hash for Tests

The canonical graph hash (v4 §10) is computed as:

1. Sort all live nodes by `(label, generation, sorted-properties)` to obtain
   a canonical node ordering, independent of NodeId/slotmap allocation
   order. Assign each node a canonical index `0, 1, 2, …` under this
   ordering.
2. Remap every edge's node list through this canonical index table, then
   sort each remapped edge's index list ascending (edges are unordered sets
   of endpoints for hashing purposes).
3. Sort the full list of remapped, sorted edges lexicographically.
4. Hash the canonical node sequence, then the canonical edge sequence, with
   FxHasher (deterministic, no per-process random seed).

Two stores built from the same logical graph via different insertion orders
produce the same canonical node→index mapping and therefore the same hash.
This is what makes the hash meaningful across independently constructed but
logically identical graphs — not merely stable for a single construction
path. Critically, edges are hashed through **canonical indices**, never
through raw `NodeId::as_ffi()` (which encodes allocation order).

**Tie-breaking**: content-identical nodes are ordered by a WL-style
refinement signature (structure), never by allocation order. Known
residual limitation: nodes in a true automorphism orbit (e.g. an
unlabeled cycle) cannot be separated by any refinement, so such graphs
may still hash differently across insertion orders — full canonical
labeling is graph-isomorphism-hard and deliberately out of scope.
Same-construction-sequence determinism (what the engine's cross-thread
tests rely on) is unaffected by this limitation.

## Manifest Entries for Reproducibility Audit

Every run produces a `manifest.json` with:

```json
{
  "seed": 42,
  "thread_count": 8,
  "step_mode": "MaxParallel",
  "hypersim_version": "0.1.0",
  "rustc_version": "1.79.0",
  "target": "aarch64-apple-darwin",
  "build_features": ["rayon"]
}
```

The `thread_count` is the actual number of threads used (from `rayon::current_num_threads()` or configured pool size). The `step_mode` is one of: `MaxParallel`, `SequentialRandom`, `SequentialWeighted`. The `build_features` lists Cargo features enabled at compile time.

## Float Tolerance for CSV Comparison

When comparing CSV outputs for reproducibility tests:
- Integer columns: exact match
- Float columns: relative tolerance `1e-6` or absolute tolerance `1e-9`, whichever is larger
- `NaN` values: both must be `NaN` (use `f64::is_nan()`)

## Test Requirements

The following must pass for reproducibility validation:

```rust
#[test]
fn maxparallel_same_seed_same_result_at_2_and_8_threads() {
    let config = SimConfig { seed: 42, thread_count: 2, step_mode: MaxParallel, ... };
    let hash_2 = run_and_get_canonical_hash(config);
    
    let config = SimConfig { seed: 42, thread_count: 8, step_mode: MaxParallel, ... };
    let hash_8 = run_and_get_canonical_hash(config);
    
    assert_eq!(hash_2, hash_8);
}

#[test]
fn sequential_random_same_seed_same_result() {
    let config = SimConfig { seed: 42, step_mode: SequentialRandom, ... };
    let hash_1 = run_and_get_canonical_hash(config);
    let hash_2 = run_and_get_canonical_hash(config);
    assert_eq!(hash_1, hash_2);
}
```
## Jitter and the seeded stream (property_effects, 2026-07-09)

The `jitter` property effect consumes from the SAME seeded `ChaCha8Rng`
stream as step-mode selection, in documented deterministic order: accepted
bindings apply in conflict-resolution's sorted order, effects within a
binding in declaration order. Same seed + same config ⟹ same jitter
sequence, at any thread count (candidate collection parallelism never
touches apply).

Zero draws occur unless a jitter effect actually executes: effect-free
rules and non-jitter effects consume nothing (pinned by
`non_jitter_effects_consume_no_rng` and the canonical-hash regressions).
Adding or removing jitter effects in a ruleset legitimately changes the
stream — the config changed — but rulesets without jitter are
stream-identical to pre-feature behavior.
