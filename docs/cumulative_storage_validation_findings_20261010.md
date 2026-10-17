# Cumulative evidence storage validation — 2026-10-10

**Lossless compact storage passes the saved-evidence validation.** The conservative
observed campaign projection is **2.93 GiB**, including filesystem/index overhead,
1 GiB for other evidence, and separate scratch and emergency reserves. This fits
the unchanged 8 GiB comparator. It replaces the unsupported *uncompressed-format*
projection, without changing the experiment's seeds, denominators or criteria.

This is an observed storage-feasibility result, not a universal bound on future
compression. Production spool integration and whole-campaign readiness remain
false. The validation executes **zero native rewrites**.

## What was validated

The codec is fixed before validation: gzip level 6, timestamp zero, one independent
object per original file. There is no JSON rewriting, record deletion, cross-record
compression, deduplication, outcome selection or codec search. Each index retains
the original name, byte length, SHA-256 and ordinary permission bits, plus the
compressed object's size and SHA-256. The original archive seal is itself retained
and authenticated. It remains the authority over the entire decoded denominator.

Two immutable corpora are included in full:

| Saved corpus | Original bytes, including seal | Compact bytes, including index | Files |
|---|---:|---:|---:|
| Campaign adapter | 42,676,156 | 3,374,010 | 925 |
| Durable-journal fixture | 24,057,552 | 3,476,429 | 804 |
| Total | **66,733,708** | **6,850,439** | **1,729** |

The measured logical reduction is **89.73%**. The compact copies occupy 11,771,904
allocated bytes on this filesystem; allocation overhead is included separately in
the campaign projection. Native binaries, sources, command receipts, failed routes,
unknown-work ledgers and the torn staging file are all retained. All twelve earlier
archives and all 149 source pins from the adapter boundary remain unchanged.

Two decoding paths verify every original byte: the compact reader uses bounded
zlib decoding; the independent byte audit uses Python's gzip reader. The original
frozen adapter and durability auditors then run through a read-only decoded path
view and return their **exact original semantic results**. Journal records and
native binaries are never materialized on disk. Only authenticated Python auditor
sources are temporarily extracted: 307,593 and 244,808 bytes, respectively.

The adapter audit still retains its 5,380–5,392 historical native-work interval;
the durability audit still retains its 2,745–2,775 interval. Compression neither
executes that work nor changes uncertain actions into completed ones.

## Publication and corruption checks

Objects are written and synced before publication; the synced index is published
last. A directory without a committed index, or containing an extra staging file,
is not accepted as a complete store. Reads check encoded size/hash, bounded decoded
length, gzip completion, absence of trailing/concatenated members, decoded hash and
the original sealed file denominator. No existing destination is overwritten.

All **18** corruption/publication checks pass: changed index or payload, truncated
or concatenated gzip, trailing bytes, excessive expansion or declared size, path
escape, duplicate paths, missing/extra objects, absent index, pending publication,
changed decoded hash, wrong original root, symlink objects, publication cap and
attempted overwrite. Repacking the tiny fixed test corpus is deterministic.

The complete storage package passes original and relocated known-root checks.
The relocated check needs only that package's compressed stores and frozen source;
it does not open the original corpus directories or replay native actions.

## Full-denominator capacity calculation

The denominator is unchanged from the preceding benchmark: 188 worlds, sixteen
ticks each, 78,960 possible uncached assays and 617 batches. The deliberately
conservative record formula retains 6,768 world slots and 159,154 assay slots,
including the earlier allowances for intent/result records and headers.

The largest observed compressed world record is 22,171 bytes; the largest observed
compressed assay record is 3,019 bytes. They are independently compressed records,
so repeated benchmark programs do not receive a cross-record deduplication benefit.

| Capacity component | Bytes |
|---|---:|
| Record projection, rounded individually to 4 KiB blocks | 818,225,152 |
| Index reserve, 1 KiB per projected record | 169,904,128 |
| Filesystem metadata reserve, 4 KiB per projected record | 679,616,512 |
| Other evidence reserve | 1,073,741,824 |
| Bounded raw scratch | 134,217,728 |
| Retained emergency raw evidence | 268,435,456 |
| **Projected peak** | **3,144,140,800 (2.93 GiB)** |
| Fixed comparator | 8,589,934,592 (8 GiB) |
| **Headroom** | **5,445,793,792 (5.07 GiB)** |

Compressed record payload alone projects to 630,539,254 bytes. The larger peak
above is the decision value; the result does not rely on ignoring small-file
allocation, indexes or temporary copies. The 1 GiB other-evidence allocation is an
explicit reserve, not a measurement of the unfinished campaign measurement output.
Unseen records may be less compressible. Actual retained bytes must therefore be
charged and capped by the production runner; this is not guaranteed completion.

## Time and implementation boundary

The entire validation takes 13.163 seconds. Packing and checking the two stores
takes 2.068 and 1.759 seconds. Complete byte/semantic verification takes 4.930 and
1.942 seconds. Scaling the slower observed per-byte costs to the *uncompressed*
26.89-billion-byte projection and doubling compression/verification costs gives
10,144 seconds of overhead. Adding the preceding 9,426-second native-command
projection gives **19,571 seconds (5.44 hours)**, inside the proposed six-hour
workflow comparator. This conservative timing estimate is not a deadline guarantee;
its margin is smaller than the storage margin.

The next implementation boundary is a bounded production spool. Each completed or
failed journal unit needs a seal, then compact publication, independent verification
and sync before its **owned temporary** raw copy can be released. Native continuation
must restore only its bounded source unit, and actual storage must be reserved before
new work. Incomplete compression and raw failure evidence must remain charged, with
a fail-closed stop at the fixed cap. The current codec's 128 MiB decoded-store limit
is a development/per-unit limit: it does not propose a single expanded multi-GiB
campaign archive. Existing historical archives must never be removed by that flow.

The storage format and saved-evidence reader are implemented and tested. That online
spool controller and the final census/lineage measurement integration are not yet
implemented. The scientific readiness and launch gates remain closed.

## Sealed evidence

Package: `output/cumulative_storage_validation_20261010`.
Root: `a31f079f7227c2bab62b7d3f1c454949e3742dc775139859c093aafda34752c8`.
There are 1,901 sealed entries, 156 source pins and 8,261,390 payload bytes.

- `contract.json`: fixed codec, corpus, reserves and acceptance conditions.
- `stores/`: all compact objects and committed indexes.
- `byte_and_semantic_audits.json`: complete byte and frozen-auditor results.
- `pack_metrics.json`, `projection.json`: actual sizes/times and prospective totals.
- `commands/`, `source_snapshots/`, `preservation.json`: execution and custody evidence.

External recheck receipts:

- `output/cumulative_storage_validation_20261010_recheck.json`
- `output/cumulative_storage_validation_20261010_relocated_recheck.json`

```sh
python3 -B output/cumulative_storage_validation_20261010/source_snapshots/experiments/cumulative/scripts/recheck_storage.py output/cumulative_storage_validation_20261010 a31f079f7227c2bab62b7d3f1c454949e3742dc775139859c093aafda34752c8
```
