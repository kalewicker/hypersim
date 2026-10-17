# Schema Versioning Specification

This document defines the versioning scheme for all persistent file formats in Hypersim.

## CSV Schema Version

**Format**: The first row of every CSV is a comment line (leading `#`). Parsers that don't understand it skip it.

```
Row 1:  #hypersim_csv_v=1,run_id=<id>,created=<ISO8601>,seed=<n>
Row 2:  column header row
Row 3+: data rows
```

**Version semantics**:
- Version integer increments on any breaking column change (addition, removal, rename, type change)
- Adding a column at the end is **not** breaking if it's nullable with default — but we still increment version for clarity
- Old version files log a warning and are not auto-migrated in this version

**Current version**: `1`

**Example Row 1**:
```
#hypersim_csv_v=1,run_id=e01_42_20260601T143022,seed=42,created=2026-06-01T14:30:22Z
```

**Columns (v1)**:
```
window_id,step_start,step_end,
n_nodes,n_edges,accepted_rewrites,attempted_matches,accepted_per_step,
d_eff,spectral_dim,diameter,mean_degree,
activity,activity_cv,entropy_rate,
pi_normalized,pi_sample_size,pi_sample_strategy,
ce_structural,ce_energetic,ce_functional,ei_micro,ei_macro,
task_lattice_depth,constructor_count,replicator_count,
regime,
new_type_bin_rate,type_histogram_entropy,dominant_type_fraction,type_histogram_jsd,
active_persistent_structures,static_repeated_motifs,static_persistent_motifs,
archive_size,archive_evictions,new_signatures,archive_capacity,
interestingness_score,
gate_fixture,gate_white_noise,gate_pi,gate_generative,
gate_treadmill,gate_mobile,gate_controller,gate_completeness,
science_unlocked
```

All columns always present in the header. Absent optional values serialize as an **empty cell** (the csv crate's serde `None`), not `NaN`. Gate columns: `Pass` | `Fail` | `Pending`. Booleans: `true` | `false`.

## Append-Log Binary Format (`HGAL`)

The general-purpose durable record stream (`hg-output::AppendLogWriter`).
Payloads are **pre-serialized opaque bytes** — hg-output has no internal
dependencies (v4 §2.2), so what a record *is* (a claim, an event, …) is the
caller's business.

```
Bytes 0-3:   magic b"HGAL"
Bytes 4-5:   version u16 little-endian (current: 1)
Bytes 6+:    sequence of (u32 LE length prefix, payload bytes) records
```

**Append-only**: records are appended during the run.

**Reading** (`AppendLogReader`):
1. Read 4 bytes, verify `b"HGAL"`
2. Read 2 bytes (u16 LE), verify version == 1
3. Loop: read 4 bytes (u32 LE) = length, then read `length` bytes
4. Clean EOF at a record boundary ends the stream

**Migration policy (v1)**: If version mismatch detected on read, log a warning and treat as if the file does not exist. No auto-migration.

## Snapshot Binary Format (`HGSS`)

One file per snapshot (`hg-output::SnapshotWriter`), payload again opaque
pre-serialized bytes (the caller serializes `HypergraphStore` however it
likes — bincode in hg-sim-core).

```
Bytes 0-3:   magic b"HGSS"
Bytes 4-5:   version u16 little-endian (current: 2)
Bytes 6-13:  step u64 little-endian
Bytes 14-21: payload length u64 little-endian
Bytes 22+:   payload bytes
```

**Reading** (`SnapshotReader`):
1. Read 4 bytes, verify `b"HGSS"`
2. Read 2 bytes (u16 LE), verify version == 2
3. Read 8 bytes (u64 LE) = step
4. Read 8 bytes (u64 LE) = payload length, then that many payload bytes

**Version history**:
- **v2** (2026-07-10): node `PropertyValue`s are serialized in a tagged
  wire form (`hg-core::wire`), so `Int`/`Float`/`Bool` are distinguishable
  on disk and the payload round-trips. The domain `PropertyValue` keeps
  `#[serde(untagged)]` for TOML ergonomics; only the persistence boundary
  is tagged. **v1 snapshots that contained node properties are
  unrecoverable in principle** — untagged bincode wrote the variant's raw
  content with no discriminant, so the type information was never on disk.
- **v1**: original; property-free snapshots only were readable in practice.

**Migration policy**: Same as HGAL — version mismatch logs a warning and
is treated as missing. No auto-migration (uniform version gate; v1 bytes,
property-free or not, are not read by a v2 reader).

## Manifest JSON Format

**No version field in v1** — the manifest is not versioned independently; it's tied to `hypersim_version` field.

```json
{
  "seed": 42,
  "thread_count": 8,
  "step_mode": "MaxParallel",
  "hypersim_version": "0.1.0",
  "rustc_version": "1.79.0",
  "target": "aarch64-apple-darwin",
  "build_features": ["rayon"],
  "run_id": "e01_42_20260601T143022",
  "experiment_name": "triangle-subdivide",
  "state": "Completed",
  "start_time": "2026-06-01T14:30:22Z",
  "end_time": "2026-06-01T14:35:10Z",
  "max_steps": 1000000,
  "final_step": 1000000,
  "config_hash": "sha256:abc123..."
}
```

The `config_hash` is a SHA256 of the canonicalized `SimConfig` (excluding `run_id` and `out_dir`).

## Annotations JSON Format

```json
{
  "version": 1,
  "annotations": [
    {
      "node_id": [123, 5],
      "step": 42108,
      "text": "Interesting local structure",
      "author": "user"
    }
  ]
}
```

Node ID is serialized as `[idx, gen]` tuple (slotmap key).

## Version Compatibility Matrix

| Component | Current Version | Backward Compatibility |
|---|---|---|
| CSV | 1 | None (warn + treat as missing) |
| Knowledge Graph | 1 | None (warn + treat as missing) |
| Snapshot | 1 | None (warn + treat as missing) |
| Manifest | N/A | Tied to hypersim_version |
| Annotations | 1 | None (warn + treat as missing) |

## Implementation Notes

- All binary formats use little-endian for multi-byte integers
- bincode configuration: standard (no size limit, no limit on recursion)
- `slotmap` keys serialize as `(u32, u32)` pairs via serde
- `SmallVec` serializes as length-prefixed sequence
- `AHashMap` serializes as vec of (key, value) pairs

## Future Versioning Strategy

When a breaking change is needed:
1. Increment version number in this document
2. Add new parser that handles both old and new versions (if feasible)
3. Or keep old parser and add new parallel format
4. Update migration policy section

For v1: No auto-migration. This is acceptable for a research tool where runs are relatively short and reproducible.