# Paid faithful D0 inheritance: engineering findings

2026-10-02, America/Phoenix. Profile `evolving-minds-d0-family-v1` implements the faithful-copy portion of roadmap package 3. Mutation, invalid-birth escrow, allocation/semantic-target controls and scientific acceptance remain pending.

Generic native birth rules now copy complete ordered instruction graphs, including stable IDs, opcodes and typed parameters, and create only the common seed. Offspring cursor, age, status, sensor state, action flags and phenotype slots reset. Descendants build their own circuits through paid instructions. No host callback constructs children or controls reproduction.

The new price table was declared prospectively in a [separate contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_family_v1.md). It charges root 1, slot descriptor 1/16, instruction 1/8, and sensor/state/actuator/connection 1/4 per maintenance tick. The direct three-instruction circuit has upkeep 2.875; each correct action pays 1 and transfers 8. Birth charges parent for child reserve 12 plus copying fee `1 + (11 + program_length)/4`, pricing every common-seed node. Conservation uses exact binary fractions in f64. Earlier construction and choice profiles retain their own prices and receipts.

This profile also implements native paid turnover: maintenance and birth precede age/reserve death, remaining root reserve returns to its site, and 26 bounded cleanup passes remove inert tombstones and owned material. No tombstone survives a completed tick. Newborn physiology and maintenance begin next tick. One global counter still serializes births; fairness remains explicitly unaccepted.

Implementation: [family runtime](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/evolving_minds/family.rs), [native laws](/Users/kalewicker/Documents/hypersim/rules/evolving_minds/family_v1.toml), [runner](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/evolving_minds/family_output.rs), [independent checker](/Users/kalewicker/Documents/hypersim/scripts/verify_evolving_minds_family.py).

## Fixed panel and results

[Final verified archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_family_verified_v2_20261002/report.md): four fixed cells, seed 17, all 64 ticks and 11,008 native engine steps. The independent checker reconstructs world streams, local interpreter states, mandatory event coverage, native guards/effects, canonical birth and cleanup contention, offspring resets, full genealogy, conservation and the final graph. It checked all 38 births and 42 organism genotypes across the four cells.

| Cue mode | Program | Paid births | Deaths | Ancestry depth | Live at 64 | Correct / paid choices | Harvest |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Informative | Direct cue | 28 | 21 | 8 | 8 | 354 / 354 | 2832 |
| Permuted | Direct cue | 5 | 6 | 2 | 0 | 21 / 39 | 168 |
| Withheld | Direct cue | 3 | 4 | 1 | 0 | 7 / 19 | 56 |
| Informative | Inactive Halt | 2 | 3 | 1 | 0 | 0 / 0 | 0 |

Every birth copied its parent's canonical genotype exactly and reset runtime state. Descendants themselves paid for reproduction and development; ancestry depth includes eight inherited edges. The other three cells stayed in the denominator through tick 64, including their extinct tails. Conservation residual was exactly zero at every native pass. The informative cell closed with initial energy 320 + influx 2112 − dissipation 1744.25 = final energy 687.75. Harvest is a transfer and is not counted again as influx.

These outcomes show bounded faithful inheritance and resource-dependent turnover in a deliberately informative fixed world. They do not establish adaptation, a statistical effect, fair reproductive competition, general cognition or H1/H2 acceptance. Genotypes were frozen; mutation and optimization were disabled.

## Verification and continuation

- Five family native tests: faithful paid multigeneration turnover; inactive extinction over the full horizon; copying an eight-instruction program with nontrivial IDs/parameters and explicit child resets; checkpoint continuation at ticks 0/3/16/32/64 across births/deaths/cleanup; identical material/event history at one and four worker threads.
- All 21 existing lifecycle, choice, D0 construction and causal-control integration tests passed. All-features and feature-disabled library checks passed.
- Four-cell independent replay passed, followed by 14 read-only/adversarial tests for missing passes/births/cleanup, duplicated birth, counter changes, copying prices, inherited age, genealogy/genotype drift, creation handles, final reserves, checkpoints and admission flags.
- A fresh process resumed the informative family at tick 16. Complete journal (2,752 entries), 64 rows, lineage and final graph files reproduced byte for byte. Checkpoint payload bytes are not asserted canonical because map serialization order can vary. [Continuation receipt](/Users/kalewicker/Documents/hypersim/output/evolving_minds_family_verified_v2_20261002/continuation_verification.json).
- [Verification receipt](/Users/kalewicker/Documents/hypersim/output/evolving_minds_family_verified_v2_20261002/verification.json) binds 76 frozen source files; all matched live files at verification. The native archive occupied 11,044,901 bytes before auxiliary receipts, within 128 MiB. An initial/final graph plus complete event journal replaces redundant full graphs at every pass. Production segmented logs and crash recovery remain future work.

Reproduce with fresh directories:

```sh
cargo test -p hg-sim-core --features evolving-minds --test evolving_minds_family
cargo run -p hg-sim-core --features evolving-minds --bin evolving-minds -- family output/NEW_FAMILY_PANEL
python3 -B output/NEW_FAMILY_PANEL/checker.py output/NEW_FAMILY_PANEL
EVOLVING_MINDS_FAMILY_FIXTURE=output/NEW_FAMILY_PANEL python3 -B scripts/test_verify_evolving_minds_family.py
target/debug/evolving-minds family-resume output/NEW_FAMILY_PANEL/informative_cue/midpoint.checkpoint output/NEW_FAMILY_RESUME
```

## Preserved development failures

The [first development archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_family_development_20261002/diagnostic_verification.json) completed its four native cells but its frozen independent checker rejected the first child at site 1: a reused single-site validator expected slot 0. The eight-instruction native copy test also hit the unchanged matcher work cap because the pattern bound all gene owner spokes before its chain, creating a factorial search. The corrected pattern traverses the chain first and retains identical guards, prices, eligibility and caps; the eight-instruction test then passed.

The [second preliminary archive](/Users/kalewicker/Documents/hypersim/output/evolving_minds_family_verified_20261002/diagnostic_verification.json) completed all native cells but its checker selected cleanup by node handles across rule classes. Native canonical order is priority, local rule ID, then sorted handles. The corrected reference applies that order. Both archives, their original frozen checkers and diagnostic receipts remain preserved; only the fresh `verified_v2` archive supplies the final verification receipt. No price, ecological criterion, denominator or horizon changed in these corrections.

## Next package

Implement recorded one-shot genotype mutation proposals and explicit invalid-birth escrow settlement. Begin with a single instruction substitution or typed parameter edit, without outcome-dependent retries or fallback genotypes. Keep genotype graph authoritative; preserve stable semantic instruction targets; compare allocation permutations; enumerate accessible one-step phenotype variation. Full lifecycle sham/freeze/restore controls and removal of bookkeeping-induced birth bias must precede research acceptance. Larger trials still require separately frozen endpoints, independent worlds and resource budgets.

All receipts remain `development_only=true`; scientific launch, gate, training and evolution flags are false.
