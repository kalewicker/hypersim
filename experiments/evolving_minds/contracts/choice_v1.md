# Authored resource-choice engineering contract v1

Frozen 2026-10-01 before the native panel. Profile evolving-minds-choice-v1.
This is a bounded H0/H1 task-interface calibration. No evolution or training.

Panel: seeds 17 and 29, 8 ring sites, 64 environmental ticks, 11 native phases.
Seed 17 is the development schedule; seed 29 is a separately declared test
schedule. Both are fixed here before execution; do not tune against test results.
Five authored controllers: inactive, constant0, constant1, random and reactive.
Cross informative, permuted and withheld cues: 30 cells; none may be dropped.

All start with the same common seed and 24 organism reserve, plus 32 reserve
per site. Each pays 2 for a two-node relay, then compulsory 5 per tick for five
owned nodes; genome material, sensor and ports have identical prices. Inactive
also builds/pays for the relay, but makes no intake attempt. Every newborn
receives 16 reserve and an unmodified controller from its parent; birth dissipates
2. The parent's total birth payment is 18, age requirement 2, reserve threshold
32. Maximum age 16 and reserve-depletion death/recycling follow v1. Births
still share v1's global counter: this calibration is not a fairness result.

Resource state is one Boolean subtype per site/time. The only successful choice
is the matching Boolean action. Site reserve is common to both subtypes, so
choosing the wrong subtype cannot discover and take the other one for free.
One committed attempt per mature organism per tick costs 1 before resolution.
Attempts require reserve >=1, a developed actuator and age >=1; newborns and
newly constructed actuators therefore have one-tick startup. Settlement transfers
8 site-to-organism on a correct choice if site reserve >=8 and organism reserve
<=40 after payment; otherwise it transfers nothing. Failure still spends the
attempt cost and the tick's action opportunity. Outcome closes the attempt in
all cases, and there is no second attempt/sensing cycle within that tick.

Each tick: influx, world-state update, sense, signal, development, attempt,
settlement, maintenance, birth, death, cleanup. Influx adds 8 at sites <=56.
All graph changes after scaffold initialization use native fixed TOML effects.
Environmental tables are compiled before execution from the frozen schedule;
the host never writes world/controller state between phases.

Resources: independently seeded balanced 32/32 Boolean schedule shared across
sites. Informative cue equals current subtype; permuted cue uses a separately
shuffled balanced schedule; withheld cue is false. Permuted cues need not have
exactly chance empirical agreement; record agreement. Random receives an
independent local Boolean coin per site/time. Every sensor has equal access to
cue/coin properties, but the fixed reactive rule reads only its cue and random
reads only its coin. No control reads future state, world clock/slot or hidden
subtype. Constant rules ignore both signals. Authored templates are an explicit
inductive bias, not evolved or learned computation.

Schedule algorithm: mix64 is the specified SplitMix64 output transform including
its initial increment. Balanced Fisher-Yates uses state increment then mix64,
rejection at (-n) mod n before reduction. Resource/cue stream XOR constants are
0x5245534f55524345 and 0x4355455f5045524d; coin is mix64(seed XOR
0x434f494e5f535452 XOR (site <<32) XOR tick) &1. Engine scheduler RNG is a
separate restored state. Persist all tables and the exact generated laws.

Primary calibration endpoint: cumulative harvested reserve per initial founder
over 64 ticks, retaining extinct controls with their observed outcomes. Minimum
engineering headroom: informative reactive exceeds the strongest constant/random
control by at least 16 reserve on each fixed schedule. It must produce paid
descendants reaching ancestry depth >=3 and have living descendants at tick 64
in both informative cells. Withheld reactive must match constant0 exactly except
immutable controller/genotype identity; record permuted-cue degradation. Zero
energy residual, faithful inheritance, complete phase/attempt/maintenance coverage
and independent replay are mandatory. This is a deterministic fixture criterion,
not a statistical significance or generalization claim.

Stop limits: 256 nodes/512 edges, 100000 matcher work/10000 results per phase,
16 MiB retained trace, 32 MiB checkpoint, 64 MiB arm output, 256 MiB panel output,
60 seconds advancing
time per arm. No culling, reseeding or replacement. Output ceiling currently
checked at final arm serialization; this is a bounded small-fixture writer, not
the scalable streaming output layer. Record an interrupted manifest on failure.

Checkpoints restore live material/allocator order, engine state, fixed schedule,
identity, ledger/lineage, clocks and full evidence at completed ticks. New header
HGEMCH01; old v1 checkpoints/readers remain separate. Signal/control/law prices
do not mutate. All scientific_run_launched, gate_pass, training_admitted and
evolution_enabled flags stay false even when engineering calibration passes.
