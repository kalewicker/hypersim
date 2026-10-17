# Bounded D0 development: implementation and engineering verification

2026-10-02, America/Phoenix. Profile `evolving-minds-d0-development-v1` implements roadmap package 2. Full D0 inheritance closure, H1/H2 acceptance and scientific evolution remain pending.

The genotype now contains several generic graph-native instructions. Native rewrites create individual states, actuators and connections, advance a paid instruction cursor, and halt or fail explicitly. The host initializes a common seed and the immutable instruction graph; it does not construct the useful circuit or choose behavior during execution.

## Implemented behavior

- Ordered 1–8 instruction graphs, eight local slots, typed Bool initialization, unique stable instruction IDs, and an allocation-independent canonical projection checked against the graph.
- `State`, `Actuator`, `Wire`, `Halt`; one paid instruction per tick. Every instruction and failed construction attempt costs 1. Waiting, permanent failure and successful halt are distinct. Invalid references, duplicate slots, second actuators and multiple writers fail without repair or retry.
- General D0 ownership, material/property, role/port and relation validation. Material-derived upkeep includes genotype and slot-map storage. No hidden reserves, foreign connections, undeclared properties or unprogrammed material are admitted.
- Separate previous/next signal buffers. Six destination-slot passes avoid native node conflicts dropping adjacent reads, followed by one commit pass. New wires start propagating next tick; a state relay adds exactly one tick. Distinct-slot recurrence is bounded; self-wires fail.
- Existing frozen choice world generator, paid choice commit and settlement, explicit influx/dissipation and exact conservation. This profile has one founder and no reproduction or death.
- Completed-tick checkpoints preserve graph handles, opaque allocator continuation, engine/RNG state and the entire bounded history. Resume checks persisted history against exact native replay before accepting continuation. Semantic state is compared because hash-map serialization order does not make checkpoint payload bytes canonical.
- Fresh exclusive output directories, durable manifests, full phase traces, frozen sources/checkers/contracts, interrupted-prefix preservation, and separate diagnostic resume command.

Implementation: [D0 interpreter](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/evolving_minds/d0.rs), [generic laws](/Users/kalewicker/Documents/hypersim/rules/evolving_minds/d0_v1.toml), [runner](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/evolving_minds/d0_output.rs), [independent checker](/Users/kalewicker/Documents/hypersim/scripts/verify_evolving_minds_d0.py), [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_development_v1.md).

## Fixed engineering panel

[Final archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_d0_verified_20261002/report.md): 20 preregistered construction fixtures, all 24 ticks and all 7,200 native advances checked. Five programs crossed with informative, permuted and withheld cues; four runtime-failure fixtures; one underfunded waiting fixture. The independent checker separately reconstructs the native rewrite effects and a slot-level interpreter that never uses recorded events to choose its expected behavior.

Informative-cue outcomes, seed 17:

| Program | Instructions paid | Owned material | Paid choices | Successes | Harvest |
| --- | ---: | ---: | ---: | ---: | ---: |
| Direct cue | 3 | 16 | 22 | 22 | 176 |
| Coin | 3 | 16 | 22 | 13 | 104 |
| Constant false | 4 | 18 | 21 | 10 | 80 |
| Constant true | 4 | 18 | 21 | 11 | 88 |
| Cue through state relay | 5 | 20 | 20 | 9 | 72 |

All five ordinary programs halted. Their different action counts include their different development durations; these fixtures establish execution and variation, not a statistical comparison of program quality. Constants and the coin control had identical rows/actions across cue modes. The relay followed the previous tick's cue under every mode. Every invalid program failed exactly once, charged its failed attempt, and made no choices; the zero-reserve founder waited once and executed no paid instruction. Conservation residual was exactly zero at every native pass.

[Verification](/Users/kalewicker/Documents/hypersim/output/evolving_minds_d0_verified_20261002/verification.json) records full independent reconstruction and 68 frozen source identities. All 68 matched current workspace files at verification. Native panel output before auxiliary audit receipts was 74,645,881 bytes, within the 192 MiB ceiling; this full-history layout is still for bounded fixtures.

## Checks and continuation

- Six D0 native integration tests: paid circuit/action variation, one-shot failure/waiting, grammar/identity bounds, maximum-length recurrence and rejected self-wires, checkpoints at ticks 0/1/2/3/4/12/24 in normal/failed/waiting states, and identical graph/event trajectories at one versus four workers.
- One native unit test rejects genotype/owner/upkeep/hidden-energy/property tampering and altered checkpoint history even with a recomputed envelope checksum.
- Fifteen independent Python tests, including a read-only full-panel audit and tampered phase coverage, payments, initial budgets, genotype, ownership, maintenance, creation clock, halt, world stream, law price, admission flags and checkpoints.
- Existing nine lifecycle, four authored-choice and two causal-control native tests passed. All-features and feature-disabled library checks passed.
- A fresh process resumed the delayed-cue founder from tick 2, while development was incomplete. All 361 frames and 24 metric rows reproduced byte-identical files. Checkpoint payload byte identity is not asserted. [Continuation receipt](/Users/kalewicker/Documents/hypersim/output/evolving_minds_d0_verified_20261002/continuation_verification.json).

The earlier [development archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_d0_development_20261002/report.md) is retained with its original frozen checker. It also passed, but predates the final runner error-path cleanup, additional recurrence test and checker statistics cross-check. It is not a replacement scientific result. Earlier lifecycle, causal-control and authored-choice archives remain unchanged under their recorded seals.

Reproduce in fresh directories:

```sh
cargo test -p hg-sim-core --features evolving-minds --test evolving_minds_d0
cargo test -p hg-sim-core --features evolving-minds --lib evolving_minds::d0::tests
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- d0 output/NEW_D0_PANEL
python3 -B output/NEW_D0_PANEL/checker.py output/NEW_D0_PANEL
EVOLVING_MINDS_D0_FIXTURE=output/NEW_D0_PANEL python3 -B scripts/test_verify_evolving_minds_d0.py
target/debug/evolving-minds d0-resume output/NEW_D0_PANEL/informative_delayed_cue/partial.checkpoint output/NEW_D0_RESUME
```

## Limits and next implementation package

This is a deliberately funded construction fixture: founder reserve 512 and one maintenance unit per owned node. The smallest active circuit costs 16 per tick while it can harvest at most 8. That arithmetic precludes sustainable ecological reproduction at these prices. Its finite-horizon survival therefore provides no viability or inherited-adaptation evidence. Choice-v1's earlier viable authored descendants do not transfer viability to this new genotype/material economy.

Package 3 should first declare a separate prospective ecology profile with fixed, material-derived prices and a reproduction ledger. Check analytically that useful intake can cover upkeep plus copying/offspring reserves, while inactive behavior and unnecessary material still cost resources. Freeze that table before evaluating the new lifecycle; preserve both existing profiles and their receipts.

Then implement atomic paid faithful copying of the complete instruction graph and common seed, offspring cursor/slot/signal reset, explicit child allocation and failed-birth escrow settlement. Demonstrate several paid generations and exact continuation before introducing recorded one-shot mutation proposals. Begin with one substitution or parameter edit, preserve failed proposals and denominator, and audit accessible one-step phenotype variation and allocation independence. Full lifecycle causal controls, birth scheduling fairness and scalable segmented storage remain later packages.

Every receipt remains `development_only=true`; scientific launch, gate, training and evolution flags are false. No optimizer, evolutionary campaign or research acceptance gate was run or promoted.
