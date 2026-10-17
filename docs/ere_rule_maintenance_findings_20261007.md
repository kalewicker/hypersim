# Newly born rule maintenance: recovery passes, continuity gap remains

2026-10-07. Native profile `E2-rule-maintenance-v1`; fixed-panel profile
`E2-rule-maintenance-panel-v1`. The [prospective contract](ere_rule_maintenance_v1.md)
and [fixed configuration](../configs/ere_rule_maintenance_v1.json) govern this
panel. This implements the [previous maintenance brief](ere_sustained_rule_maintenance_next_v1.md).

## Outcome

All63 cells complete96 steps:6,048 advancing native steps, plus63 separate
paid non-advancing Seal initialization applications. Zero optimizer updates,
skipped cells, scientific retries, seed replacements or criterion changes.
Independent graph/binding/payment/compile accounting passes. All9 learned
worlds produce the context-specified new rule and use it on three distinct
ordinary nodes. Funded exposed repair restores that exact post-birth program
after all four newly damaging lesions in every world. The action oracle also
passes. Exhaustion and unpaid controls refuse repair when fuel is absent.

This closes **finite post-birth paid restoration**, not continuous persistence.
The exposed cohort's first genotype lifetime is11 steps and first installed
lifetime10, both below the precommitted16-step exposure interval. Recovery
creates distinct later spells; it does not extend the first lifetime. Root
survival is right-censored at96 and cannot substitute for class survival.

The original literal all-touch ratio is numerically positive in several arms;
it is reported, not suppressed. Additional repair touches lower that denominator
without improving the first lifetime. This arithmetic qualification is not a
stronger persistence result or an original sustained H-R1 admission.
`scientific_acceptance`, `sustained_claim`, `training_admitted` and
`independent_physics_replay` remain false.

## Fixed execution and chronology

Nine previously exposed seven-node relational worlds; one deterministic native
seed993748718001. The confirmed exact-gradient checkpoint100 is imported byte
for byte. SplitMix64 stream993748723001 supplies18 draws: one joint action per
world, reused across six learned conditions. All9 samples select the correct
context pair. All decisions and all63 slots are saved before native application.
These worlds/arms are fixed cohorts, not63 independent stochastic replicates.
The engine RNG probes remain unchanged throughout.

Initial proposal literals are[99,98]. Birth edits consume their separate two
fuel units at completed steps1/5. The specified useful class appears at5,
installs at boundary6 and first fires on ordinary structure at7. Native paid
Seal at0 protects the proposal's first left-pattern E_SLOT0 relation, with
ordered payload `[pattern,variable,root]` and graph-resident witness material.
It is not a cached original program. A live graph reader retains the changed
literals; repair never restores the initialization body instead.

Static native hazards erase that payload's label at16/32/48/64. The source uses
epoch2; quarantine retires world rules at boundaries16/32/48/64. Funded repair
runs at17/33/49/65, and reinstatement occurs at boundaries18/34/50/66. Ordinary
execution is attributed to each actual pre-step compiled table, not to row
indices assumed to remain constant when a family is quarantined.

Funded genotype spells are `[5,16)`, `[17,32)`, `[33,48)`, `[49,64)` and
`[65,96]` (last right-censored). Installed spells are `[6,16)`, `[18,32)`,
`[34,48)`, `[50,64)` and `[66,96]` (last right-censored). This preserves both
the damage gaps and the separate recompilation lag.

## Every condition

All counts below cover9 worlds. Every arm spends18 birth units and9 Seal
initialization units; those are separate from repair fuel.

| Condition | Scheduled threats | Actual erases | Newly damaging / repeated | Paid repairs | Useful target ordinary fires |
| --- | ---: | ---: | ---: | ---: | ---: |
| Learned protected | 36 | 0 | 0 /0 | 0 | 405 |
| Learned exposed repair | 36 | 36 | 36 /0 | 36 | 369 |
| Learned exhaustion | 36 | 36 | 27 /9 | 18 | 171 |
| Learned unpaid | 36 | 36 | 9 /27 | 0 | 45 |
| Learned frozen source | 36 | 36 | 36 /0 | 36 | 0 |
| Learned no hazard | 0 | 0 | 0 /0 | 0 | 405 |
| Explicit action oracle | 36 | 36 | 36 /0 | 36 | 369 |

The protected arm has36 blocked opportunities due to the actual A2 META label
ceiling, not a cosmetic flag. No-hazard still has matched Seal/instruction
initialization. The frozen source repairs its live genotype using the initialized
repair catalogue but never installs the new ordinary class, so it earns no
novel-use credit. Oracle actions are an explicit capacity comparator, not
learned discovery credit.

Across the panel:216 scheduled hazard opportunities,36 blocked,180 actual
erases,144 newly damaging lesions,36 repeated erases,126 paid repairs and126
paid birth edits. Repair fuel is never replenished. The two-unit exhaustion
condition repairs at17/33, becomes newly damaged at48 and stays damaged;
its64 event is a repeated erase. The unpaid condition remains damaged after16.

## Lifetime measurements

Each row retains a fixed cohort of9. `≥` denotes a right-censored lower bound,
not a measured death or exact empirical half-life.

| Condition | Genotype first half-life | Installed first half-life | Mean actual touch interval | Literal ratio | Exceeds fixed16 |
| --- | ---: | ---: | ---: | --- | --- |
| Learned protected | ≥91 | ≥90 | 4 | unmeasured | unmeasured |
| Learned exposed repair | 11 | 10 | 7.111111 | pass | fail |
| Learned exhaustion | 11 | 10 | 9 | pass | fail |
| Learned unpaid | 11 | 10 | 12.6 | fail | fail |
| Learned frozen source | 11 | no installation | 7.111111 | ineligible | ineligible |
| Learned no hazard | ≥91 | ≥90 | 4 | unmeasured | unmeasured |
| Explicit action oracle | 11 | 10 | 7.111111 | pass | fail |

The unchanged survival instrument requires exact class-lifetime point estimates
for its numerical ratio flags; censored references are not promoted to an exact
pass. They do show persistence through the measured horizon without exposure.
All63 particular roots survive through96, with unmeasured empirical root
half-life and lower bound96. No missing target birth occurred, but the fixed
denominator implementation retains missing/ineligible births rather than
substituting a successful sampled class. Frozen installation remains missing.

Funded exposed touches are1/5/16/17/32/33/48/49/64/65, giving64/9≈7.111111.
Without repair, the touch mean is63/5=12.6. Both conditions still have exactly
the same first lifetimes11/10. This is why the separate16-step reference
comparison was fixed in advance. Seal is accounted separately at0 and clock
ticks earn no family edit-frequency credit.

## Implementation and extension map

| File/module | Responsibility / extension boundary |
| --- | --- |
| `crates/hg-sim-core/src/ere/rule_maintenance.rs` | New96-step native kernel, seven conditions, paid Seal, native seeded birth/clock/hazards, graph instructions, live/frozen compilation, read-only receipt capture and deduplicated full pre-step tables. Only this layer imports the engine. |
| `crates/hg-sim-core/tests/ere_rule_maintenance.rs` | All9-world24-step parity against the untouched old kernel; seven one-world96-step primitive controls. |
| `crates/hg-train/src/ere/rule_maintenance.rs` | Frozen encoder import/decisions and full schedule; invokes sim-core only. No optimizer or direct engine access. Retains failed/not-run slots without replacement. |
| `crates/hg-train/tests/ere_rule_maintenance.rs` | Import/deterministic decision refusals and the environment-enabled fixed panel entrypoint. |
| `scripts/verify_ere_rule_maintenance.py` | Independent finite rule constructors, complete initial-body decode, Seal binding reconstruction, graph event/effect replay, payment, distinct-node use, compile lag and lifetime measurement. |
| `scripts/ere_rule_survival_metrics.py` | Unchanged pure first-spell/censor/fixed-cohort instrument. Does not supply state to the native interpreter. |
| `scripts/run_ere_rule_maintenance.py` | Prospective freeze, source/history/feature/binary guards, preflight, one native launch, postflight accounting and sealing. Future launches now collect the auditor's local static import closure. |
| `scripts/recheck_ere_rule_maintenance_closure_v2.py` | Zero-native frozen-primary-auditor reproduction with separately sealed historical dependency supplement. |

Keep the original relational kernel, complete repair codec and survival
instrument unchanged. The only shared-module changes are two additive
`rule_maintenance` exports. The new hazard and clock motifs are explicit
seeded scaffolding; they are not newly learned endogenous rules. Structural
novelty comes from changed literal constraints relative to the complete initial
candidate catalogue, not from renaming, priority-only changes or activation.
The policy observes only the ordinary initial graph, not metadata or rewards.

For a new prevention/redundancy profile, fork a new versioned kernel and
contract. Do not edit this frozen native body, relax its validators or overwrite
its receipts. Distinguish scheduled threats, blocked/absorbed threats, new
lesions, repeated erases and all paid family touches. A later recovered spell
must remain separate; retain target classes fixed by context, not sample success.

## Verification, provenance and limitations

The primary archive is
[`output/ere_rule_maintenance_trial_20261007`](../output/ere_rule_maintenance_trial_20261007/manifest.json):
380 sealed entries and280 source pins. Root seal:
`4e1c6f359586c969d1c740527c7fe2ddcc4bfc99bd82b0cbdee91c9977db13ea`.
The prelaunch and executed test-binary hash is
`9589cfe43dda9260718f9f07690e6105a1794ad029393dc7ca6fad54b87f2208`.
Native panel test time40.76 seconds; full runner102.704 seconds. Final archive
size85,653,295 bytes, below256MiB. Sources/history guards pass before and after.
The preceding native confirmation, exact-gradient and negative sampled-learning
archives retain their original seals and meaning. Known six Finder exceptions
on older archives remain separately pinned; none is silently resealed.

The recorded launch preflight spends432 parity advances plus672 control advances
and7 Seal applications, separately from panel work. Both Rust native groups,
Rust import/refusal preflight, all5 Python postflight groups (including13 ledger
tampering cases), all6 unchanged survival groups and feature-off checking pass.
Earlier development tests/export attempts are not panel observations. This
turn's total development/preflight work is4,464 advances and24 Seal applications,
including the recorded launch preflight. One development control assertion had
incorrect event categorization (`complete_` prefix omitted); an absolute-path
export correction was also needed. Neither changed native mechanism criteria.
The failed export artifact remains at
`output/ere_rule_maintenance_preflight_20261007/failure.json`; successful
development fixtures remain in `output/ere_rule_maintenance_preflight_v2_20261007`.

The initial standalone frozen recheck failed to import an omitted helper module.
The sealed primary is **not self-contained for that standalone recheck**. Its
failed two-entry recheck is retained with seal
`fc6be5e243c95e098cdf5c7268fc9b16b1078a09536e38e9c8690b4260d13bc5`.
No primary artifact, auditor criterion, checkpoint or native cell was edited
or rerun. The9-entry
[`ere_rule_maintenance_closure_v2_20261007`](../output/ere_rule_maintenance_closure_v2_20261007/manifest.json)
supplement supplies exactly two omitted helper modules from their sealed
2026-09-27 historical snapshots, matching the live helper bytes used at launch.
It freezes the unchanged primary auditor, verifies every one of13 imported
project modules comes from primary/supplement snapshots, reproduces the full
audit and all5 test groups, and verifies the primary380 entries remain intact.
Supplement seal:
`9309163be350189e241aa4d9593f985f29b2351571630d0c61487b13f07020b3`.
It takes43.145 seconds and executes zero native steps. Historical helper
validation checks the prior root seal and selected helper entries, not a new
full re-audit of that older long-gate campaign. Future launch packaging was
fixed with recursive static-import collection and a passing packaging test;
that correction is separate from the preserved primary source snapshot.

This is ledger/accounting reconstruction, not independent Rust physics replay.
Unique ordinary labels guarantee and receipts assert three genuinely distinct
bound nodes. The assay covers one preselected sealed edge-label lesion, finite
fuel and96 steps in known worlds. It does not cover missing ownership/nodes,
unbounded autophagy, resource ecology, coevolving threats, novel topology
generalization or EI/CE. Repair depends on intact graph-resident witnesses and
sealed endpoints. Passing this mechanism does not close those original goals.

## Next justified boundary

Do not spend another encoder-training campaign on this gap: frozen birth
transfer and paid recovery both work. Test **finite paid prevention or redundant
representation** under the same four exposure opportunities, without blanket
A2 exclusion or free replenishment. Begin with one bounded native primitive
and zero/exhaustion controls before a new matched panel. Charge each absorbed
threat, retain the born program's exact bytes and ordinary use, and ask whether
its first genotype/installed spell stays continuous beyond16 steps. This is a
proposed next experiment, not implemented or launched here. Even a positive
finite shield result would still need a later ecological admission boundary.
