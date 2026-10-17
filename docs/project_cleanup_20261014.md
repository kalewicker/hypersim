# Evidence archiving and checkout cleanup — 2026-10-14

**Complete.** Three evidence archives were written to the flash drive, fully
restored into temporary Mac storage, and compared with their original inventories.
All compressed archive hashes, file contents, permissions, symlinks and original
hard-link groups passed. Originals were checked again before local removal.
No simulation, training or scientific rerun occurred.

## Archive location and scope

`/Volumes/B/Hypersim-archives-2026-10-14/`

| Archive | Compressed bytes | Verified entries | Original source paths |
|---|---:|---:|---|
| `main-evidence.tar.zst` | 1,811,200,322 | 217,749 | Root `output/`, `runs/`, `crates/hg-train/runs_stage2/`, `crates/hg-selfmod/runs/` |
| `lk-evidence.tar.zst` | 1,301,355,112 | 24,234 | Root `experiments/lk/output/` |
| `lk-historical-runs.tar.zst` | 177,021,414 | 22,877 | External `/Users/kalewicker/Documents/hypersim-lk/runs/` and `output/` |

Total compressed archive size: **3,289,576,848 bytes**, or **3.29 GB / 3.06 GiB**.
The drive additionally holds compressed file inventories, per-archive verification
receipts, a manifest, a copy of the project report and restoration instructions.
Its other files were not altered. Each archive fits the drive's FAT32 4-GB file
limit. Compression uses Zstandard 1.5.7 at level 3 with two workers; tar preserves
file contents and standard modes/links. macOS extended attributes are omitted.

The [archive verification receipt](project_archive_verification_20261014.json)
pins all archive and inventory SHA-256 values. The drive's `archive_manifest.json`
is the same creation/verification record; its `original_evidence_deleted:false`
describes the state before cleanup. The separate
[cleanup receipt](project_cleanup_receipt_20261014.json) records completed removal.

## Cleanup and retained source

Allocated checkout size fell from **68.80 GiB to 2.50 GiB**, reclaiming **66.30 GiB**,
measured by parent-directory `du -sk` before and after. Small subsequent
documentation files do not materially change the total. Most remaining storage
is `.git` history, approximately 2.5 GiB.

The five local evidence roots in the table were removed only after verified
backup. Rebuildable targets removed:

- `target/`
- `experiments/lk/target/`
- `experiments/cumulative/target/`
- `bins/hypersim-cli/target/`

Source, documentation, rules, configurations, contracts, manifests/lockfiles for
the source workspaces, scripts, vendored allocator code and Git history remain.
The cumulative experiment's LK path dependencies are intact. The external
original LK workspace, including its historical runs, remains outside this
checkout; it was backed up but not deleted. Temporary full-restoration copies
were removed after comparison.

Some removed historical artifacts were Git-tracked. Their working-tree deletions
are deliberate archival changes; no commit or Git-history rewriting was performed.
The root ignore rules now exclude output/run directories and old training outputs
from accidental re-addition.

## Historical documentation and checks

Artifact paths in old findings, the experimental report and the dated inventory
remain their **original restoration locations**. Their scientific results and
source identities did not change. The pre-cleanup JSON documentation inventory
is preserved instead of being regenerated with the evidence missing.

The October 13 documentation validation receipt is historical. Its validator
expects the original local evidence and dated documentation hashes; it is not a
post-cleanup health check. Restore evidence and use its matching frozen sources
for scientific archive checks. The new archive receipts document preservation
during cleanup. A missing local artifact after cleanup means “archived,” not
an unmeasured or failed original experiment.

## Restoring or checking an archive

Mount the flash drive. From the repository root, verify one archive against the
local pinned receipt:

```sh
python3 -B scripts/verify_project_archive.py \
  /Volumes/B/Hypersim-archives-2026-10-14/main-evidence.tar.zst
```

To restore and compare **every entry**, supply a new staging directory with
enough free space for the uncompressed data:

```sh
python3 -B scripts/verify_project_archive.py \
  /Volumes/B/Hypersim-archives-2026-10-14/main-evidence.tar.zst \
  /private/tmp/hypersim-evidence-restore
```

The tool refuses an existing staging directory, checks the compressed archive
and inventory hashes, restores all material, and compares file contents,
permissions, links and entry denominators. A failed restore stays in staging for
inspection. Successful staging is also retained for use. This does no native work.
Repeat with the other archive names as needed, using a different new staging path.

After verification, put the restored relative roots back under the original
source base shown in `archive_manifest.json`, avoiding any newer existing
evidence. Main and current LK evidence restore below
`/Users/kalewicker/Documents/hypersim`; the historical LK archive restores below
`/Users/kalewicker/Documents/hypersim-lk`. Old authorizations and absolute-path
identities may require those original locations. Do not overwrite code or run
a new campaign merely to make an archived reference resolve.

The final cumulative campaign is in `main-evidence.tar.zst`. Once restored at
its original path, its frozen `recheck_scientific_campaign.py` command and known
seal are documented in the [project report](project_experimental_report_20261013.md).

## Maintenance tools

- [Archive and full-restore workflow](../scripts/archive_project_evidence.py)
- [Guarded local cleanup workflow](../scripts/cleanup_archived_project_evidence.py)
- [Pinned archive verification/restoration](../scripts/verify_project_archive.py)

These tools do not reconstruct scientific evidence from documentation. The
archives preserve the actual evidence and frozen measured implementations.
