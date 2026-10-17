# Hypersim storage cleanup — 2026-10-01

Removed the old Rust incremental caches, reducing their allocated size by 7.49 GiB. Project size: 55.20 GiB before; 49.73 GiB at final check.

The deletion command completed successfully. A small `ere_categorical_adapter` incremental cache was recreated during the session and preserved. No matching build or simulation process remained at the final process check. Source, outputs, frozen executables, compiled dependencies, and Git history were preserved. Future debug builds may take longer while incremental caches rebuild.

## Locations

Values use `du -sk` allocated disk usage. Parent and child rows overlap; do not sum them. Hard-linked build files can appear in separately measured directories, so the 7.49 GiB cache measurement does not equal the project reduction. The project itself decreased by 5.47 GiB. APFS snapshots may affect actual recovered free space.

| Location | Before | Final | Contents |
|---|---:|---:|---|
| `.` | 55.20 GiB | 49.73 GiB | Entire project |
| `target` | 14.66 GiB | 9.77 GiB | Main Rust build artifacts |
| `experiments/lk/target` | 1.73 GiB | 1.22 GiB | Isolated LK build artifacts, selected by scripts/lk.sh |
| `bins/hypersim-cli/target` | 0.40 GiB | 0.35 GiB | Additional CLI build artifacts |
| `output` | 7.72 GiB | 7.72 GiB | Main experiment records and frozen snapshots |
| `experiments/lk/output` | 24.19 GiB | 24.19 GiB | LK experiment records and replay artifacts |
| `runs` | 5.04 GiB | 5.04 GiB | Earlier simulation outputs and checkpoints |
| `.git` | 1.42 GiB | 1.42 GiB | Git history |
| `/private/tmp` | 2.78 GiB | Unchanged | Hypersim frozen/preflight source and build archives plus small caches; preserved |
| `/Users/kalewicker/.cargo` | 0.32 GiB | Unchanged | Shared Cargo registry and tooling |
| `/Users/kalewicker/.cache/uv` | 2.78 GiB | Unchanged | Shared Python package cache; attribution spans projects |
| `/Users/kalewicker/.cache/codex-runtimes` | 3.78 GiB | Unchanged | Shared Codex tool runtimes; attribution spans projects |

## Cleanup receipt

- `/Users/kalewicker/Documents/hypersim/target/debug/incremental`: 6.81 GiB before, 86.9 MiB at final check.
- `/Users/kalewicker/Documents/hypersim/experiments/lk/target/debug/incremental`: 0.71 GiB before, 0.0 MiB at final check.
- `/Users/kalewicker/Documents/hypersim/bins/hypersim-cli/target/debug/incremental`: 0.06 GiB before, 0.0 MiB at final check.

## Generated files

The preserved output areas contain **92,356 files**: 56,423 in `output`, 20,801 in `experiments/lk/output`, and 15,132 in `runs`. Most are JSON records, binary checkpoints, JSONL traces, logs, and frozen source snapshots. These three areas occupy roughly 37 GiB combined.

Largest LK runs: `selfmod_v23_fresh50k_20260928` (about 4.1 GiB), `selfmod_v22_maintenance_20260928` (3.7 GiB), and `selfmod_v17_durability_20260927` (2.8 GiB). Large main outputs include the minibatch trial (1.2 GiB), feasibility training (929 MiB), and direction assay (905 MiB).

Outside the project, `/private/tmp/hypersim_selfmod_*` directories hold frozen/preflight source and executable archives. Their temporary location does not establish that they are disposable. The entire inspected `/private/tmp` directory occupies about 2.8 GiB. Shared user caches also exist outside the project, but cannot be attributed entirely to Hypersim. Codex chat history is about 485 MiB across projects; Codex application logs about 11 MiB. No `~/.codex/worktrees` directory, project `.cargo` directory, or `~/.cargo/config.toml` was found.

## Further cleanup

Repeat incremental-cache cleanup only while builds are idle. A full Cargo clean would reclaim additional compiler artifacts but also remove executables referenced by launch instructions, so rebuilding would be required. For a larger reduction, archive completed output directories to another disk with checksums and verified replay dependencies before removing local copies. Preserve failed and diagnostic archives. Do not blanket-delete `output`, `runs`, or `/private/tmp/hypersim_*`.

No compiler configuration changed and no tests or builds were launched for this cleanup. Machine-readable sizes and receipts are in `storage_cleanup_2026-10-01.json`. The audit covers the project and selected external locations, not the entire disk.
