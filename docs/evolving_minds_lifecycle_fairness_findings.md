# D0 lifecycle controls and birth arbitration

Finalized 2026-10-03 (America/Phoenix); native archives retain their 20261002 identifiers. These are bounded engineering results. Scientific launch, research gates, training, evolution and reproductive fairness remain false.

Lawful communication interventions now operate through native rules over complete paid lifecycle ticks. A separate birth-arbitration diagnostic identifies global-counter serialization and tests a local-counter prototype. The full family/mutation ecology still uses its original global counter. This work advances package 4 without declaring fairness complete.

## Implementation and scope

The opt-in `evolving-minds` feature remains off by default. Orchestration stays in `hg-sim-core`; the existing engine, rule language and graph store apply all physical writes. [Contract](../experiments/evolving_minds/contracts/d0_lifecycle_fairness_v1.md), [native controls](../rules/evolving_minds/lifecycle_v1.toml), [arbitration laws](../rules/evolving_minds/arbitration_v1.toml), [runtime](../crates/hg-sim-core/src/evolving_minds/lifecycle.rs), and [diagnostic](../crates/hg-sim-core/src/evolving_minds/arbitration.rs) define separate versioned profiles.

A typed communication flag gates signal prepare on the existing paid connection. Native controls bind genotype instruction 101, same-owner material and actual ports, then clear actuator signal and buffer. Disabling preserves genotype, structure, ownership and maintenance costs. All four arms use the same explicitly zero external control charge. An extra control pass after sensing yields 44 native passes per tick; physiology, age, copying and disassembly continue. Newborn connections receive control at their next signal boundary, and completed-tick material passes ordinary validation.

All arms fork the exact same sham-origin completed-tick-3 checkpoint, after the first birth. Sham enables communication; Freeze disables it from tick 4 onward; Rescue disables then restores atomically in each control pass; Release disables for ticks 4–8 and restores at tick 9. Rescue calibrates immediate restoration. Release retains the consequences of actual interruption. All arms complete 64 ticks, including extinct tails, with seed 17 and the authored cue program. These controls concern cue communication and funded descent; they do not measure memory or learning.

The `HGEML001` checkpoint includes treatment metadata, full native execution state/history, RNG and allocator continuation. Legacy family/mutation readers remain supported, and profile-specific CLI resume commands reject incompatible checkpoints. Only completed-tick boundaries are supported. Serialized bincode bytes are not canonical because of map ordering; exact continuation is assessed through native state and canonical artifacts.

## Full lifecycle results

| Arm | Births | Deaths | Maximum depth | Live at tick 64 | Successful / attempted choices | Harvested | Influx | Dissipated | Final energy |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Sham | 28 | 21 | 8 | 8 | 354 / 354 | 2832 | 2112 | 1744.25 | 687.75 |
| Freeze | 3 | 4 | 1 | 0 | 7 / 19 | 56 | 312 | 120 | 512 |
| Rescue | 28 | 21 | 8 | 8 | 354 / 354 | 2832 | 2112 | 1744.25 | 687.75 |
| Release | 28 | 22 | 7 | 7 | 310 / 316 | 2480 | 1984 | 1595.5 | 708.5 |

Every energy residual is zero. The independent checker verified 11,264 native advances, 87 paid faithful births and 91 genotype instances across the four fixed arms. Sham/Rescue rows, lineage and final graph are byte-identical; their journals are identical after normalizing only the two declared control-law names. Release differs from Sham, as expected after five interrupted ticks. Different arm-dependent resource influx remains explicitly accounted for.

A fresh process resumed Release from its tick-8 checkpoint and completed the original horizon. Its entire 2,816-entry journal, 64 rows, lineage, final graph and control metadata match the uninterrupted Release files byte for byte. This demonstrates restoration with preserved history, rather than rewinding to Sham.

Evidence: [independent verification](../output/evolving_minds_lifecycle_verified_v2_20261002/verification.json), [continuation receipt](../output/evolving_minds_lifecycle_verified_v2_20261002/continuation_verification.json), [checks](../output/evolving_minds_lifecycle_verified_v2_20261002/checks.json), and [seal](../output/evolving_minds_lifecycle_verified_v2_20261002/output_sha256.json).

## Birth arbitration results

This is a four-pass diagnostic with two identical Halt founders, eight ring sites, fixed offered targets and paid warmup/copying. It is separate from the full lifecycle panel. The global policy binds the family counter; the prototype copies child IDs from disjoint per-site streams and removes the shared counter binding. Shared parent/site material still excludes physically competing offers.

The fixed denominator is 64 cells: two policies × disjoint/competing offers × eight site rotations × forward/reversed allocation. Every cell retains all native bindings, canonical selection keys, conflict classifications, paid events and full material frames.

| Policy / offers | Cells | Accepted births per cell | Allocation reversal changes accepted-parent set |
| --- | ---: | ---: | ---: |
| Global / disjoint | 16 | 1 | 4 of 8 rotation pairs |
| Global / competing | 16 | 1 | 6 of 8 rotation pairs |
| Local / disjoint | 16 | 2 | 0 of 8 rotation pairs |
| Local / competing | 16 | 1 | 6 of 8 rotation pairs |

The independent checker reconstructed 128 offers, 80 accepted births and all 256 native advances. Global disjoint offers conflict only through the counter; local disjoint offers have no conflict and both commit. Local disjoint event order changes in four rotation pairs, but both parents remain accepted. That is an ordering change, not a winner change. Competing outcomes retain allocation-order sensitivity under both policies. Local IDs are unique sparse site streams, not the contiguous family frontier.

Evidence: [independent verification](../output/evolving_minds_arbitration_verified_v2_20261002/verification.json), [contention summary](../output/evolving_minds_arbitration_verified_v2_20261002/contention_summary.json), [checks](../output/evolving_minds_arbitration_verified_v2_20261002/checks.json), and [seal](../output/evolving_minds_arbitration_verified_v2_20261002/output_sha256.json).

## Verification and preserved diagnostics

Both final archives bind 97 frozen source entries, all matching the live execution sources at verification. Independent Python models check the native event effects and guards, ownership/material grammar, paid costs, offspring resets, treatment timing, candidate completeness, conflict selection, lineage, endpoint rows, flags and root summaries. Python checks checkpoint envelopes/checksums; native replay verifies the opaque checkpoint contents.

All 36 selected Rust integration tests passed: four new lifecycle/arbitration tests and 32 existing regression tests. They include complete lifecycle controls, checkpoint rejection/corruption, all 64 arbitration recipes and one/four-thread equality. All 12 lifecycle and 13 arbitration adversarial/read-only checker tests passed. Feature-enabled and feature-disabled checks passed. Rule regeneration matches both archived law files exactly. All 17 previously sealed Evolving Minds archives retain their exact inventories and hashes.

The first arbitration archive exhausted its 16 MiB output budget after retaining 59 result files. It remains Interrupted with a negative diagnostic receipt. Compact JSON corrected storage amplification in fresh fixed-denominator archives, keeping the same physical laws, offers, criteria and caps. All 59 retained results are semantically identical to their final compact counterparts. Earlier completed lifecycle/arbitration archives retain their own frozen checkers and positive audit receipts; the final v2 archives strengthen verification of root summaries and research flags without changing treatment/cell criteria.

A lifecycle adversarial test invocation overlapped the final archive writer. Its checkpoint-copy test encountered a not-yet-written journal, and its read-only hash test observed the writer's new files. That failed invocation is retained in the checks receipt as an orchestration diagnostic. Rerunning the unchanged suite after the native writer completed passed all 12 tests. No runtime/checker change was made for that race. During preflight, removing the global counter from the local birth pattern also required removing its distinct-variable guard; the corrected laws passed native parsing before any completed arbitration run.

The final native lifecycle archive occupied 16,405,386 bytes and the arbitration archive 5,664,648 bytes before audit receipts/seals. These small, full-history artifacts remain a calibration fixture; streaming live-state checkpoints and output recovery are still future work.

## Next milestone

Declare and implement a separate full-ecology reproduction profile. It must admit simultaneous disjoint births without global bookkeeping contention, preserve one-shot mutation escrow and energy settlement, and define reproducible contested-site selection independently of graph handles. Verify mixed genome lengths, invalid proposals, parent/target conflicts, descendants, cleanup, allocation/site permutations, thread counts and checkpoint continuation. Separate environmental RNG pairing from endogenous event consumption where needed.

A local counter alone does not provide unbiased contested-site selection, concurrent invalid-birth settlement, lineage equality or scientific acceptance. Package 4 remains open on those properties. After that, address streaming evidence/resource pilots and prospectively calibrated viable mutation neighborhoods before H1/H2 decisions or evolutionary trials.
