# Evolving Minds live checkpoints and segmented evidence

Implemented 2026-10-03 as the opt-in `evolving-minds-d0-streaming-v1` engineering storage profile. It runs unchanged admission-v1 physiology and scheduling. Scientific launch, H1/H2 passage, training, evolution and broad reproductive fairness remain false.

## Implementation and recovery

The [prospective contract](../experiments/evolving_minds/contracts/d0_streaming_v1.md) fixes seed 17, three baseline recipes, six interruption mechanisms and a 64-tick horizon. The [live-state implementation](../crates/hg-sim-core/src/evolving_minds/family.rs) retains graph/allocator, engine/RNG, recipe/schedule, cumulative ledgers, live birth records and founder/depth metadata. It drains one completed tick's events, accounting, lottery, proposals and lineage deltas before the next tick; extinct lineage and settled requests are removed from live memory.

`HGEMS001` checkpoints contain no historical journal, rows, proposals or extinct lineage. Earlier full-history checkpoint formats remain readable. A new optional SHA-256 dependency is enabled only with `evolving-minds`.

The [segment writer](../crates/hg-sim-core/src/evolving_minds/stream_output.rs) appends complete JSONL tick bundles. Every eight ticks it syncs the log, checkpoint and transparent state attestation, then publishes an exclusive immutable commit marker and syncs the directory. The marker binds header, completed clocks, log byte offset and prefix hash, checkpoint/state hashes and previous marker hash. Recovery validates the highest published commit, then restores live native state without executing the historical simulation. A damaged published commit causes rejection. Pending files, partial tails and unpublished checkpoints are preserved and ignored.

Recovery creates a fresh directory linked to the exact source commit. It does not append to the source. Chains reject gaps, duplicates, symlinks, cycles and unsupported depth. Native restore compares checkpoint contents to the bound transparent attestation and validates the live material/accounting state. Integrity hashes do not authenticate a producer; independent reconstruction remains required before interpreting results.

## Fixed interruption results

All six declared recoveries reached tick 64 and reproduced the full-history trace exactly. Application-level faults exercise specific write stages; they do not establish durability under an actual power failure or every filesystem.

| Interrupted source | Native tick reached | Committed recovery tick | Preserved evidence |
| --- | ---: | ---: | --- |
| Clean stop | 8 | 8 | Complete committed prefix |
| Torn payload | 9 | 8 | Half-written tick-9 tail |
| Torn checkpoint | 16 | 8 | Complete payload through 16; partial unpublished checkpoint |
| Before commit publication | 16 | 8 | Complete checkpoint/state at 16 and partial pending marker |
| After publication, before manifest progress | 16 | 16 | Published marker plus captured stale tick-8 manifest |
| Fixed 262144-byte quota | 11 | 8 | Complete uncommitted ticks 9–10; tick 11 was not appended |

The quota was not adjusted after execution. All interrupted sources remain in the fixed denominator. The additional fresh-process CLI continuation uses the before-publication source, starts at tick 8 and reaches 64.

## Resource measurements

The final pilot's manifest elapsed time was 132.156 seconds on a shared machine. Summed categorized timers measured 108.459 seconds native advancement, 1.321 seconds checkpoint serialization/attestation generation, 3.177 seconds of timed I/O, and 6.655 seconds loading the six source cuts. Timed I/O excludes some log-sync, manifest, serialization and inventory work; manifest elapsed includes the complete pilot. Concurrent tests/audits affected wall time, so these are observed costs rather than isolated benchmarks. Each source load took about one second; historical native execution was not replayed during restore.

| Baseline at tick 64 | Full-history checkpoint bytes | Live checkpoint bytes | Reduction factor | Segment payload bytes |
| --- | ---: | ---: | ---: | ---: |
| Faithful disjoint | 989107 | 31098 | 31.81 | 2144165 |
| Mixed competing | 961960 | 30725 | 31.31 | 2085408 |
| Mixed settlement | 662803 | 15802 | 41.94 | 1451817 |

Measured peak retained evidence was 46 native pass entries, one accounting row, one admission round and two current requests. Peak live/prunable lineage was eight records. Checkpoints ranged from 8403 to 31098 bytes. This removes embedded historical state from checkpoints; append-only evidence, periodic attestations and independent audit still consume resources. The approximately 31 MiB pilot is bounded to 64 ticks and does not establish long-horizon scaling. [Resource receipt](../output/evolving_minds_streaming_verified_20261003/resource_analysis.json) reports the underlying measurements.

## Verification and evidence

The independent Python checker validates frozen sources, every published prefix and segment link, checkpoint envelope, live lineage/metadata and scope flags. It reconstructs native physiology with the independent admission/D0 model using explicitly temporary derived audit views, without fabricating old checkpoints. It audits all three baselines, six interrupted prefixes and six recovery chains, then compares complete journals, rows, lineage, admission records and final graphs with the preserved full-history fixture.

The development archive independently reconstructed 87 published states in 266.518 seconds. The final archive plus separate CLI continuation reconstructed 95 states in 275.994 seconds, with all nine complete pilot chains matching the preserved full-history journals, rows, lineage, admission records and final graphs. The final panel freezes 111 source files and has 417 sealed files occupying 32182260 bytes before the seal itself. All six source interruptions and their uncommitted evidence are retained.

Native tests passed 21 cases across admission, family, mutation, lifecycle and streaming, including exact live/full-history comparison, continuation, pruning, one/four-thread identity, source preservation and refusal to fall back from corrupted published state. Fourteen Python adversarial tests passed. Twenty-four feature-disabled engine tests passed; `hg-sim-core` and `hg-output` library targets also compiled, but contain zero unit tests. The existing admission checker passed its 15 regression tests. All 29 earlier sealed Evolving Minds archives retain their original inventories and hashes.

- [Final fixed panel](../output/evolving_minds_streaming_verified_20261003/manifest.json)
- [Final independent reconstruction receipt](../output/evolving_minds_streaming_verified_20261003/independent_replay.json)
- [Native/adversarial test receipt](../output/evolving_minds_streaming_verified_20261003/test_verification.json)
- [Fresh-process continuation](../output/evolving_minds_streaming_process_resume_20261003/manifest.json)
- [Earlier archive integrity](../output/evolving_minds_streaming_verified_20261003/prior_archive_integrity.json)

## Next package

Package 5 closes the bounded storage/recovery engineering milestone. Package 6 should prepare an explicit H1/H2 decision and an H3a proposal. First calibrate paid viability and behavior across the complete inherited source-edit neighborhoods, preserve invalid and extinct cases, and establish an informative task with room for improvement. Broad reproductive opportunity remains unresolved beyond the tested scheduler/bookkeeping mechanisms. Freeze independent-world seeds, endpoint, controls, effect criteria and resource limits before any scientific selection run. This storage implementation does not authorize or admit that run.


Follow-up (2026-10-03): the [H1/H2 review and exhaustive paid source-neighborhood calibration](evolving_minds_h1_h2_decision_20261003.md) are complete for one exposed world. H3a remains no-go under current funding; see the [conditional design](evolving_minds_h3a_conditional_proposal_20261003.md).
