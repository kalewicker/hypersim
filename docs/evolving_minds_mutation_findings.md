# One-shot source mutation and native birth escrow: engineering findings

2026-10-02, America/Phoenix. Profile `evolving-minds-d0-mutation-v1` implements the next bounded portion of roadmap package 3. The [prospective contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_mutation_v1.md) freezes operator probabilities, request/copy prices, failure settlement, cells and horizons. No scientific evolution, training or acceptance gate was launched.

The source-parameter kernel samples no edit with probability 1/4; otherwise it samples a stable instruction ID and one of seven different source slots. A separate SplitMix64 stream records every raw draw, its indices, semantic target, parent/proposed genotype bytes and validity. It never uses arena handles to select a gene. Authored faithful, coin, invalid-Halt and missing-port controls are explicitly named and consume no random draws. Host sampling/validation and derived copy-rule compilation are experimental mechanisms; organisms do not invent or repair native laws.

Native reserve rules admit a halted, funded parent with a vacant neighbor, dissipate fixed request charge 1, and transfer child reserve 12 plus the existing copying fee into an escrow node. Sampling occurs once after admission. A syntax-invalid proposal receives a native full escrow refund; its request charge remains spent, no copying fee is charged, and no child exists. A syntax-valid proposal executes a native common-seed copy/reset, pays copying fee, and commits a new birth ID. Zero-energy escrow disposal precedes death and cleanup. A later developmental failure belongs to that committed child and remains paid. Each future tick may admit a new request under unchanged eligibility; no request internally redraws, repairs or substitutes a parent clone.

The same weighted family upkeep and physiology run with 45 native passes/tick. The global counter continues to serialize requests, and canonical handle order can affect physical contention. Semantic target independence does not establish reproductive fairness. The request charge is an explicit addition to the separate mutation profile; earlier family prices and receipts are unchanged.

Implementation: [sampler/records](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/evolving_minds/mutation.rs), [shared family runtime](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/evolving_minds/family.rs), [native rules](/Users/kalewicker/Documents/hypersim/rules/evolving_minds/mutation_v1.toml), [runner](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/evolving_minds/mutation_output.rs), [independent checker](/Users/kalewicker/Documents/hypersim/scripts/verify_evolving_minds_mutation.py).

## Fixed panel

[Final archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_mutation_verified_v2_20261002/report.md): six fixed cells, seed 17, informative world, all 64 ticks including extinct tails, 17,280 native advances. Literal controls and sampled kernels are distinct fixture mechanisms, not interchangeable scientific replicates.

| Program / mechanism | Requests | Committed births | Syntax rejected | Unreachable committed programs | Live at 64 | Depth | Raw draws |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Cue / faithful | 28 | 28 | 0 | 0 | 8 | 8 | 0 |
| Cue / sampled source | 11 | 3 | 8 | 2 | 0 | 1 | 33 |
| Constant1 / sampled source | 13 | 1 | 12 | 1 | 0 | 1 | 39 |
| Cue / authored coin | 8 | 8 | 0 | 0 | 0 | 3 | 0 |
| Cue / authored invalid Halt | 14 | 0 | 14 | 0 | 0 | 0 | 0 |
| Cue / authored missing port | 6 | 6 | 0 | 6 | 0 | 1 | 0 |

The checker reconstructs all 80 requests, 46 committed births, 34 refunds/rejections, 52 organism genotypes and 72 raw random draws. Every native pass has exactly zero conservation residual. In the invalid-Halt control, each request spends 1 and refunds 16.5; total request loss is 14, with no fallback child. The missing-port control commits six children and records six paid developmental failures.

The full one-source-edit neighborhood contains 21 proposals for cue and 28 for constant1. Each has seven syntax-valid proposals; only one cue neighbor and two constant1 neighbors can complete material construction. These are exhaustive grammar projections, checked independently, rather than estimates of survival or viable offspring fraction. This limited mutational access is an engineering finding and motivates further language/operator calibration before research. The fixed sampled cells ended extinct; they are retained outcomes, with no seed replacement or operator repricing.

## Verification

- Six mutation native tests cover rejection/refund/no fallback, valid copying and paid developmental failure, semantic IDs/duplicate rejection, RNG and complete continuation, one/four threads, and reversed node/edge allocation with exact continuation in that allocation. All 26 existing lifecycle, choice, D0, faithful-family and intervention integration tests also passed; the family and mutation tests were repeated after the continuation refinement.
- Eighteen adversarial/read-only checker tests cover omitted/reordered requests, changed/extra draws, wrong semantic targets/proposals, request/copy/refund charges, fallback children, changed settlement laws, missing reservation/disposal/cleanup events, stream state, checkpoint corruption and scientific admission flags.
- [Verification receipt](/Users/kalewicker/Documents/hypersim/output/evolving_minds_mutation_verified_v2_20261002/verification.json) binds 84 frozen sources, matching live files at verification. Native output before auxiliary receipts was 18,927,324 bytes, below the frozen 128 MiB cap. [Checks receipt](/Users/kalewicker/Documents/hypersim/output/evolving_minds_mutation_verified_v2_20261002/checks.json) includes all-features/feature-disabled builds and unchanged earlier archive seals.
- [Fresh-process continuation receipt](/Users/kalewicker/Documents/hypersim/output/evolving_minds_mutation_verified_v2_20261002/continuation_verification.json) starts after the first sampled request at tick 3 and must reproduce the subsequent ten proposals, future draws, journal, rows, lineage and final graph. Tick 16 alone occurs after this fixture's last request, so an additional tick-3 checkpoint was added prospectively before final sealing.

Mutation checkpoint envelope `HGEMM002` persists the allocation recipe as well as RNG, records, graph/allocator and native engine state. It supports ordinary and reversed-allocation continuation and explicitly reads the earlier `HGEMM001` engineering envelope. Faithful-family `HGEMF001` is unchanged. Checkpoints support completed ticks only; pending escrow/partial native passes remain unsupported boundaries. Payload bytes are not asserted canonical because map serialization can vary. Complete journal/evidence and interruption manifests remain bounded fixture storage; segmented crash recovery and a resource pilot remain package 5.

Reproduce using fresh directories:

```sh
cargo test -p hg-sim-core --features evolving-minds --test evolving_minds_mutation
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- mutation output/NEW_MUTATION_PANEL
python3 -B output/NEW_MUTATION_PANEL/checker.py output/NEW_MUTATION_PANEL
EVOLVING_MINDS_MUTATION_FIXTURE=output/NEW_MUTATION_PANEL python3 -B scripts/test_verify_evolving_minds_mutation.py
target/debug/evolving-minds mutation-resume output/NEW_MUTATION_PANEL/source_cue/mutation.checkpoint output/NEW_MUTATION_RESUME
```

## Preserved development evidence and next package

The [development archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_mutation_development_20261002/diagnostic_verification.json) completed six native cells and six independent cell replays, but its frozen final audit failed because the source inventory omitted the new mutation checker. Its original checker and failed receipt remain preserved. The inventory was corrected in a fresh archive. An early native test also found a missing TOML value entry in copy-rule specialization; the implementation now inserts the value explicitly, and the same tests pass.

The [first verified archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_mutation_verified_20261002/verification.json) passed its frozen checker and 18 adversarial tests. Its receipt is retained as a valid earlier engineering snapshot. A fresh v2 archive adds persisted allocation recipes and the tick-3 continuation checkpoint. These refinements change continuation evidence, not the kernel, prices, fixed cells or horizons.

Next: full lifecycle sham/freeze/restore controls and reproduction fairness. Measure independent and competing requests, site/allocation permutations and counter-induced contention before choosing a prospective fairness redesign. Then calibrate accessible useful variation and viable offspring with fixed denominators; insertion/deletion and broader mutation classes remain separate extensions. H1/H2 and evolutionary acceptance remain pending.
