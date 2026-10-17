# D0 development v1 engineering contract

Declared 2026-10-02 before the bounded output panel. Profile `evolving-minds-d0-development-v1`; no reproduction, mutation, selection, scientific assay, or training admission.

## Scope and grammar

One founder on one resource site, 24 ticks. The immutable genotype is an ordered graph of 1–8 instruction nodes with unique stable u32 instruction IDs, contiguous positions, ownership edges, and adjacent `NEXT_INSTRUCTION` edges. Canonical bytes `HG-D0-1`, u8 length, then each u32 little-endian ID and four u8 fields (opcode, destination, source, initial Bool). The graph is authoritative; projections are checked against it and never execute behavior.

Eight local slot descriptors are present at birth. Slots 0 and 1 contain common cue and independent-coin sensors; slots 2–7 start vacant. Generic instructions are `State(slot, initial)` (opcode 0), `Actuator(slot)` (1), `Wire(source, destination)` (2), and final `Halt` (3). State initial values are Bool; other unused fields must be zero/false. Each program has exactly one final Halt. No useful circuit is loaded by the host. Each material state, actuator, and connection is created by a native paid rewrite. Creation requires a vacant slot; at most one actuator is allowed; a wire requires existing distinct ports and a state/actuator destination without a writer. Recurrence between distinct slots is allowed, delayed and bounded by this horizon; self-wires and multiple writers fail.

## Execution, timing, and prices

Runtime status: 0 ready, 1 waiting for energy, 2 halted, 3 permanently failed. Cursor indexes instruction position; stable instruction IDs remain distinct from positions and arena handles. At most one development instruction is attempted per tick. Every successful instruction, Halt, or invalid-reference attempt dissipates 1 reserve unit. Failure leaves cursor unchanged and stops further development and action. Insufficient energy changes ready to waiting once, dissipates nothing and creates nothing. Waiting can execute if sufficient reserve becomes available under native laws; no hidden retry or repair occurs. A syntactically valid program may fail during construction.

Each tick uses 15 native passes: environment, world, cue sensing, coin sensing, six destination-slot buffer passes (2–7), one signal commit, development, action commit, settlement, maintenance. Slot passes avoid native node conflicts discarding adjacent reads. Buffer passes read only pre-commit SIGNAL; all buffers publish afterwards. A new wire first propagates on the next tick. Every state relay adds a one-tick delay. Unwired states retain their initial signal.

Initial site reserve 64, founder reserve 512 for ordinary fixtures (0 for waiting diagnostic). Site influx 8 when reserve <=56; site starts clock/slot 0. Resource/cue/coin schedule uses exactly the previously declared choice-v1 generator and seed 17. Only site 0 is materialized. The 512 table laws are compiled before dynamics; no host writes alter the world or phenotype during execution.

Maintenance price is 1 for each organism-owned material node, including root, slot descriptors, instruction nodes and sensors. Initial price 11 + program length; creating a state, actuator or connection adds 1. Maintenance drains the lesser of reserve and price and increments age. This deliberately funded construction fixture is not a viability calibration. No death or birth is implemented in this profile.

Only a halted program with an actuator can attempt one choice per tick. It pays 1 before the outcome, commits its actuator Bool, then receives 8 from the site if the committed choice equals local resource, site has >=8 and root has <=1000. Failure pays the same price without a refund or repeat. All graph reserves are finite nonnegative f64 ENERGY properties; NodeData.energy is zero. Transfers conserve energy, influx and dissipation are explicit.

## Fixed panel and checks

20 cells, fixed denominator, no outcome-dependent replacements: five programs (`cue`, `coin`, `constant0`, `constant1`, `delayed_cue`) crossed with informative, permuted and withheld cues; four informative runtime failures (`missing_port`, `duplicate_writer`, `duplicate_slot`, `second_actuator`); and informative `cue_waiting` with founder reserve 0. Every cell runs all 24 ticks, including failed/waiting cells. Exact declared programs and stable IDs are frozen in the runner and independently reconstructed by the checker.

Required engineering observations: five ordinary programs halt; at least two valid phenotype sizes and observable action streams differ; informative direct cue succeeds on all 22 eligible actions; added relay follows previous-tick cue; invalid programs fail once and pay once; waiting emits one status transition and no paid instruction; exact conservation, ownership, role/port grammar, immutable genotype, material-derived upkeep and at-most-one writer hold at every pass. These are engineering checks, not statistical acceptance criteria.

Completed-tick checkpoints include full material/allocator state, engine/RNG continuation, immutable program/world identity, all frames and rows, and accounting. Resume validates bounded full history by exact native replay before accepting continuation. Independent standard-library Python checking reconstructs every native event and also a separate slot-level reference interpreter and ledger. Native tests compare checkpoints across partial development, failure and waiting, and 1 versus 4 worker threads; a fresh-process continuation must reproduce the complete trace and rows.

Limits: <=40 nodes, <=64 edges, matcher work 100000/results 10000 per pass, <=8 MiB trace and checkpoint per cell, <=192 MiB total output, 60 seconds advancing per cell. Fresh directories and interrupted manifests preserve failures. `development_only=true`; scientific launch, gate, training and evolution flags remain false. Full D0 closure still requires paid faithful genotype copying, offspring runtime reset, failed-birth settlement and mutation calibration in later separately versioned packages.
