# Authored resource-choice engineering findings

Date: 2026-10-01 (America/Phoenix). Profile `evolving-minds-choice-v1`.

The revised plan's first package is implemented: a native resource-choice task
where each attempt costs energy before success is resolved. An informative
reactive control supports paid descendants, while constant/random controls have
lower yield. This establishes useful information and behavioral headroom for
the bounded authored fixture. It does not establish evolved or learned behavior,
direct-versus-developmental advantage, or the full H1/H2 gates.

## Implemented contract

Each site has one Boolean resource subtype. An organism chooses one of two
mutually exclusive intake modes. The attempt spends 1 reserve and commits the
choice; settlement transfers 8 from site to organism only if the choice matches
the subtype and reserve guards pass. A failed attempt transfers nothing and
consumes that tick's action opportunity. Failure cannot probe the other action.

All five controllers—Inactive, Constant0, Constant1, Random and Reactive—start
with identical seed material and 24 reserve, develop the same two-node relay
for 2 reserve, and pay identical 5-node upkeep. Inactive makes no attempts.
Constant controllers ignore inputs; Random copies an independent local coin;
Reactive copies the local cue. These are supplied native rule templates.

All post-initialization dynamics use TOML rewrites. The environment's balanced
resource schedule, cue manipulation and site-local coin tables are fixed before
execution. Controllers access only their owned local sensor boundary, with no
clock, hidden subtype or future-state reads. The resource and noise streams are
independently derived and reconstructed by Python.

Paid faithful births, age/depletion death and recycling follow v1's fixed
economy. Every child inherits the controller and develops from the common seed.
Eleven phases separate influx, world state, sense, signal, development, attempt,
settlement, maintenance, birth, death and cleanup. Startup consumes a tick before
an organism can attempt intake.

The contract was declared before the native panel: seeds 17/29, 64 ticks, five
controllers crossed with informative/permuted/withheld cues, yielding 30 cells.
The minimum engineering margin was 16 harvested reserve above the strongest
uninformed control on each schedule, plus ancestry depth ≥3 and living
descendants at the endpoint. Seed 29 is a separately declared engineering test
schedule; two seeds do not supply a statistical generalization claim.

See [fixed contract](../experiments/evolving_minds/contracts/choice_v1.md) and
[native laws](../rules/evolving_minds/choice_v1.toml).

## Observed outcomes

| Informative-cue outcome | Schedule 17 | Schedule 29 |
| --- | ---: | ---: |
| Reactive harvested reserve | 616 | 616 |
| Constant0 harvested reserve | 0 | 96 |
| Constant1 harvested reserve | 48 | 0 |
| Random harvested reserve | 40 | 72 |
| Reactive margin over strongest uninformed control | 568 | 520 |
| Reactive paid births | 5 | 5 |
| Reactive ancestry depth | 5 | 5 |
| Reactive living descendants at tick 64 | 1 | 1 |
| Reactive failed attempts | 0 | 0 |

With cues withheld, Reactive's complete metrics match Constant0's. Constant,
random and inactive outcomes remain identical across cue manipulations, checking
that they do not use the cue. With permuted cues, Reactive harvested 8 on schedule
17 and 104 on schedule 29, and both lineages became extinct. Permuted cues use an
independent balanced schedule; empirical agreement need not be exactly 50%.
The complete receipt reports all cells, including extinct controls.

Every phase has zero reconstructed energy residual. Informative Reactive starts
with total reserve 280, receives influx 760, dissipates 514, and finishes at
526: `280 + 760 - 514 = 526`. Harvest is a conservative transfer, not supply.
Each informative run commits 77 paid attempts and successfully settles all 77.

## Evidence and verification

Final receipt: [verification.json](../output/evolving_minds_choice_verified_20261001/verification.json).
All 30 cells pass the frozen checker; its 60 source-inventory entries matched
the live files at verification. The final archive occupies about 134 MiB and
seals 312 evidence files. The frozen independent checker reconstructs schedules, native effects, eligible
process engagement, genotype/ownership, attempts, upkeep, births/deaths, lineage,
accounting, CSV and the prospective calibration criteria. It checks native
counter totals and checkpoint envelopes. It is a bounded semantic checker, not
a general matcher enumerator.

Four new native tests cover informative paid descent, committed failed actions,
withheld controls, checkpoints at 0/1/16/32/64, corrupted checkpoints, and exact
one-versus-four-thread trajectories. Existing nine lifecycle and two causal
control tests pass. Existing glider/triangle property-effects regressions pass
with the feature enabled and disabled, and the all-features compile check passes.

Twelve Python clean/tamper tests cover complete read-only panel verification,
phase omission, unpaid/missing attempts, wrong creation clocks, initial subsidies,
unfaithful offspring, changed controller or world stream, altered prices, false
training admission and checkpoint corruption.

The separate fresh-process diagnostic continuation from the schedule-29
informative midpoint compares every graph/event frame, all CSV-equivalent rows
and lineage against the uninterrupted run. Its evidence is
`continuation_verification.json` alongside the final verification receipt.

The development archive `output/evolving_minds_choice_development_20261001` is
retained. Its first frozen checker used the post-step clock to reconstruct node
creation, while native creation uses the pre-step clock. The new checker fixes
that caller's one-based indexing and adds a deliberate wrong-generation test.
The older engineering checker already used a zero-based loop correctly; it and
the older archives were preserved. The final archive freezes the correction;
no action law, physical price or calibration endpoint was tuned to the outcomes.

## Code and reproduction

The implementation extends the opt-in module with `choice.rs` and
`choice_output.rs`. The original material validator is extracted into a shared
helper; legacy v1 retains its original opcode and topology checks. Choice uses a
separate profile/header (`HGEMCH01`) and properties 241001–241009. Its fixed laws
are reproducibly derived from the pinned v1 laws by
`scripts/generate_evolving_minds_choice_rules.py`. Source inventory and frozen
readers accompany the receipt.

From the repository root, with fresh output paths:

```sh
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- choice output/evolving_minds_new_choice
python3 -B output/evolving_minds_new_choice/checker.py output/evolving_minds_new_choice
cargo test -p hg-sim-core --features evolving-minds --test evolving_minds_choice
EVOLVING_MINDS_CHOICE_FIXTURE=output/evolving_minds_new_choice python3 -B scripts/test_verify_evolving_minds_choice.py
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- choice-resume output/evolving_minds_new_choice/s29_informative_reactive/midpoint.checkpoint output/evolving_minds_new_choice_resume
```

## Remaining boundaries

The supplied reactive template reads the present cue directly; this task needs
no memory or learning. There is no mutation or multi-instruction D0 program yet.
The old globally contended birth counter remains and does not establish fair
competition between populations. Signal operations have no separate event price;
their material pays compulsory upkeep, with explicit attempt costs added here.
Full-history checkpoints and traces remain suitable for small fixtures only.
Diagnostic resume writes a new directory, not an in-place manifest continuation.

The next package is the minimal D0 semantics and execution needed to construct
and vary this choice behavior through paid local development. Copying/mutation,
failed-birth settlement, reproduction fairness, lawful lifecycle interventions
and scalable evidence remain later work. Scientific, training and evolution
flags remain false throughout this engineering receipt.
