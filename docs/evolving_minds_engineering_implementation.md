# Evolving Minds: first engineering implementation

**Roadmap update (2026-10-01):** The [plan review](evolving_minds_plan_review_2026-10-01.md) revises the next-work sequence to choice-interface calibration before a larger D0 language. Results below remain the historical engineering receipt.

Date: 2026-09-29. Profile: `evolving-minds-engineering-v1`.

The first runnable foundation of the [adapted plan](evolving_minds_hypersim_implementation_plan.md) is implemented. It uses Hypersim's native graph, matcher, conflict resolution and typed rule effects. A paid authored relay develops from a common seed, acquires resources, reproduces faithfully and dies with recycling. An authored idle control cannot acquire resources and becomes extinct.

This is bounded engineering calibration. The outcomes below do not establish evolution, cognition, a developmental-encoding advantage or broad ecology readiness. H0/H1 have concrete coverage for this restricted profile; the full H1 and H2 gates remain open. Scientific campaigns and training were not launched, and all acceptance/admission flags remain false.

## Implementation map

| Component | Location | Responsibility |
| --- | --- | --- |
| Feature and CLI | `crates/hg-sim-core/Cargo.toml`, `src/bin/evolving_minds.rs` | Opt-in `evolving-minds`; paired smoke controls and diagnostic checkpoint continuation |
| Configuration and schema | `crates/hg-sim-core/src/evolving_minds/config.rs`, `docs/property_registry.md` | Bounded configuration, reserved labels/properties, explicit clocks |
| Fixed physical laws | `rules/evolving_minds/engineering_v1.toml` | Fifteen native rules; nine phases; ownership, prices, acquisition, birth, death and cleanup |
| Runtime | `crates/hg-sim-core/src/evolving_minds/runtime.rs` | One shared store; phase dispatch; read-only event capture; ledger, material and lineage validation |
| Persistence | `crates/hg-sim-core/src/evolving_minds/output.rs` | Exclusive outputs, durable manifests, full traces, source inventory and completed-tick checkpoints |
| Allocator continuation | `crates/hg-core/src/arena.rs`, `src/graph.rs` | Additive opaque allocator state needed for exact future handle reuse |
| Independent checker | `scripts/verify_evolving_minds.py` | Reconstruct rule effects, costs, coverage, ownership, lineage and CSV from persisted traces |
| Contract and pilot configuration | `experiments/evolving_minds/contracts/engineering_v1.md`, `configs/evolving_minds/development_smoke.toml` | Frozen engineering scope, limits and interpretation |

The runner is separate from the production simulation loop. Existing ERE/LK runners, training admission and their output archives were preserved. The normal graph serialization format is unchanged; only the new profile's checkpoints carry the explicit allocation state.

## Physical and execution contract

The default fixture has 16 ring sites, each starting with 32 reserve, and one founder with 24 reserve. Sites receive bounded influx. The single inherited instruction is either `Idle` or `BuildRelay`; genotype length is one. BuildRelay spends 2 reserve to construct an owned connection and actuator. Role-labelled SOURCE/TARGET incidence carries a Boolean resource signal to the actuator. Intake transfers 8 reserve from the occupied site, subject to native guards.

Obligatory upkeep costs 1 per owned node each environmental tick: 3 for the common seed and 5 for the developed relay. Birth removes 18 from the parent, transfers 16 to the child and dissipates 2 for construction. Children receive a faithful instruction copy and fresh common seed. A matched counter gives stable monotone birth IDs and limits the correctness fixture to one birth per tick. Death follows depleted reserve or age 16; remaining reserve returns to the site and owned material is removed.

Each tick executes environment, sense, signal, development, action, maintenance, birth, death and cleanup as separate native MaxParallel steps. A newly developed relay acts on the following tick. Newborn upkeep starts the following tick. Float reserves use exactly representable integer quanta; internal `NodeData.energy` remains zero. Per-phase conservation includes fixed development, birth-construction and upkeep sinks.

Native matching has explicit work and result limits. Invalid material, skipped required effects, accounting errors and exceeded ceilings interrupt execution; there is no ecological culling or replacement to hide resource failures. Defaults cap material at 256 nodes/512 edges, output at 64 MiB per arm, retained trace at one quarter of that allowance, and advancing execution at 60 seconds checked between ticks. Compilation, serialization and checking are outside that advancing-time ceiling.

## Checkpoint defect found and repaired

Ordinary arena deserialization reconstructs the free list from vacant slots and loses its original reuse order. This initially allowed resumed aggregate counters to match while later birth handles and exact graph histories diverged. Aggregate-only resume checks would have missed it.

The new checkpoint stores and restores allocator free-list order and frontiers alongside the graph and engine checkpoint. Restoration validates vacancy inventory and mode before changing either allocator. Tests cover ordinary and metadata-isolated allocators and transactional rejection of invalid state. Existing snapshot encoding remains unchanged; older checkpoint formats do not automatically acquire this stronger continuation contract.

Profile checkpoints include configuration, instruction, graph/allocator state, engine state/RNG/rule table, clocks, ledger, lineage, accumulated trace and completed rows. The bounded binary envelope checks length, checksum, trailing bytes and fixed-law identity. Only completed environmental-tick boundaries are supported. `resume` writes a fresh diagnostic continuation directory; it is not in-place continuation of an interrupted generic manifest.

## Persisted engineering receipt

Final paired receipt: [verification.json](../output/evolving_minds_engineering_final_20260929/verification.json). Both arms completed 64 ticks / 576 native phases on eight Rayon threads. The frozen checker verified each arm, and all 47 entries in its compile-time source inventory matched the live source at verification.

| Measured quantity | Authored relay | Authored idle |
| --- | ---: | ---: |
| Paid births | 28 | 0 |
| Deaths | 17 | 1 |
| Living organisms at tick 64 | 12 | 0 |
| Cumulative harvested reserve | 2672 | 0 |
| Cumulative influx | 2648 | 512 |
| Cumulative dissipation | 1939 | 24 |
| Final total reserve | 1245 | 1024 |
| Accounting residual | 0 | 0 |
| Advancing wall time, seconds | 0.944 | 0.331 |
| Persisted arm bytes | 12,463,356 | 4,468,174 |

Initial total reserve is 536, so the independently checked balances are `536 + 2648 - 1939 = 1245` and `536 + 512 - 24 = 1024`. Harvest is a transfer, not influx. These timings are a small debug-build engineering fixture, not a performance benchmark.

A separate fresh-process continuation from the relay midpoint reproduced the entire tagged graph/event trace and lineage files byte for byte, and all completed-tick metrics matched. See [continuation_verification.json](../output/evolving_minds_engineering_final_20260929/continuation_verification.json) and the continuation directory `output/evolving_minds_engineering_final_resume_20260929`.

Each arm contains a manifest, configuration, fixed laws, contract, source inventory, frozen checker, full JSONL trace, lineage, CSV, midpoint/final checkpoints, status and report. Earlier development receipts remain archived. Checksums detect accidental corruption; archive hashes are integrity receipts, not authenticated signatures. The source inventory covers this runner and its relevant lower layers, not a complete repository snapshot. Compiler metadata records `rustc --version` at invocation, not authenticated build-tool provenance.

## Verification completed

- Nine integration tests cover paid delayed development, turnover and idle extinction, exact checkpoints at ticks 0/1/5/8/16/32/64, corrupted checkpoints, matching/material ceilings, exclusive outputs, durable output exhaustion and one-versus-four-thread equality.
- Two runtime unit tests exercise owner-guard rejection/firing and reject bad upkeep, wrong signal types and hidden reserves before execution.
- Two allocator tests cover exact continuation and transactional rejection; six existing `hg-core` library tests pass.
- Existing glider/triangle property-effects regressions pass with the feature enabled and disabled, two tests in each configuration. `cargo check -p hg-sim-core --all-features` passes.
- Fifteen Python checker tests cover clean evidence and tampering: missing phases/events, duplicate upkeep, foreign owners, changed prices/initial reserve, unfaithful descendants, ancestry, checkpoint corruption/omission, native effect counters, CSV and false admission flags.

The Python checker independently applies this profile's declared effects and recomputes its accounting and coverage. It is not a full replacement matcher for arbitrary rule sets or a general hostile-input security auditor. No full workspace research suite or evolutionary assay was launched.

## Reproduction commands

Run from the repository root. Output paths must not already exist.

```sh
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- smoke output/evolving_minds_new_smoke --config configs/evolving_minds/development_smoke.toml
python3 scripts/verify_evolving_minds.py output/evolving_minds_new_smoke/active
python3 scripts/verify_evolving_minds.py output/evolving_minds_new_smoke/idle
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- resume output/evolving_minds_new_smoke/active/midpoint.checkpoint output/evolving_minds_new_resume
cargo test -p hg-sim-core --features evolving-minds --test evolving_minds
EVOLVING_MINDS_FIXTURE=output/evolving_minds_new_smoke/active python3 scripts/test_verify_evolving_minds.py
```

For archival checking, use the `checker.py` frozen inside the arm rather than assuming the live checker remains identical.

## Next implementation boundary

1. The first calibrated Signal/Action forks are implemented: sham, signal clamp, foreign owner and owner rescue. See [causal-control findings](evolving_minds_causal_controls_findings.md). Complete ecological intervention continuations remain future work.
2. Introduce a bounded multi-instruction D0 developmental schema and an independently validated faithful-copy/mutation contract. Keep physics prices immutable and record mutation provenance.
3. Create a separate ecology revision with resource trade-offs and useful choice headroom. Specify movement, signal-event costs, birth scheduling and fair direct-versus-developmental controls before collecting comparison evidence.
4. Reassess the full H1/H2 acceptance gates. Only then propose a preregistered H3 evolutionary run; lifetime plasticity and memory require their own later controls.

The current authored mechanism has no mutation, evolved program, movement, resource trade-off, learning, plasticity or general developmental language. Positive paid turnover establishes the lifecycle plumbing for this fixture; it does not close those research boundaries.
