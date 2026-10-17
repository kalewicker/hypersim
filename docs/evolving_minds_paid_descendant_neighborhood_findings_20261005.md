# Paid descendant source-neighborhood calibration — 2026-10-05

This engineering calibration implements the next boundary after the [descendant route diagnosis](evolving_minds_descendant_route_findings_20261004.md). Its [prospective contract](../experiments/evolving_minds/contracts/d0_paid_descendant_neighborhood_v1.md) freezes both fresh worlds, all35 substitutions, unchanged controls, early targeting and the common64-tick horizon before outcomes.

## Implementation and evidence

Native profiles `evolving-minds-d0-paid-descendant-neighborhood-v1` / `evolving-minds-d0-paid-descendant-streaming-v1` apply one supplied proposal per founder family at its first paid descendant request by24, breaking simultaneous ties by semantic parent birth ID. Founders supply the previously declared useful edit after native payment; all other copies are faithful. Parents are unchanged. Native grammar, reservation, copying, child B24 endowment, refunds, development and maintenance settle every proposal. No viability filtering or late replacement is allowed.

`HGEMDN01` persists immutable configuration and at most two original pre-settlement receipts after their organisms die. Old checkpoint layouts remain unchanged. Independent replay regenerates target selection and the receipt cache at every published boundary; provenance cannot be established from live cache validation alone.

Artifacts are [native evidence](../output/evolving_minds_descendant_development_20261004/manifest.json), [prospective freeze](../output/evolving_minds_descendant_audit_20261004/prospective_freeze.json), [full replay](../output/evolving_minds_descendant_audit_20261004/independent_replay.json), and [full-denominator summary](../output/evolving_minds_descendant_audit_20261004/full_denominator_summary.json). Native execution completes73 initial cells /74 segments /4672 ticks /214912 passes:72 main cells on601/607 and one separate exposed-world17 recovery. The recovery stops at24 and resumes in a fresh process; its interrupted prefix remains evidence, not a new world.

## Fixed outcomes

Early exposure: 144/144 family targets; tick range 17..17. Missing requests remain in the denominator: 0. All target child lifetimes are observed to64; 0 are right-censored.

| Substitution route | Targets /140 |
|---|---:|
| no_early_request | 0 |
| syntax_rejected | 84 |
| paid_unreachable | 40 |
| reachable_unchanged | 0 |
| reachable_edited | 16 |

Across substitutions, 16/140 target children complete and 8/140 make a native paid reproductive request. The four unchanged controls complete 4/4 and reproduce 4/4. These are root-level outcomes, not summed overlapping subtree yields.

| World | Family | Denominator | Syntax valid | Reachable | Completed | Reproducing | No early request |
|---|---:|---:|---:|---:|---:|---:|---:|
| 601 | 1 | 35 | 14 | 4 | 4 | 2 | 0 |
| 601 | 2 | 35 | 14 | 4 | 4 | 2 | 0 |
| 607 | 1 | 35 | 14 | 4 | 4 | 2 | 0 |
| 607 | 2 | 35 | 14 | 4 | 4 | 2 | 0 |

The two reproducing edits are `g102 0→1` and `g102 0→3`, which change the input to State slot2 while preserving the cue input to the actuator. Each produces four reproducing target roots, with96 harvested and one paid child per root, matching the unchanged target-root totals. This is consistent with behavioral neutrality; exact action-sequence equivalence and inherited class propagation remain the next independent boundary.

Both actuator-input edits, `g103 0→1` (Coin) and `g103 0→2` (Delayed), complete in all four world/family targets but have zero funded parent ticks and zero reproductive requests. They retain29 and28 reproduction-funding-blocked ticks respectively across their four roots. Early births remove the former censoring uncertainty without removing the observed funding barrier.

The40 unreachable births include32 absent-port substitutions (sources4..7 at either Wire) and8 self-wire substitutions (`g102 0→2`, `g103 0→3`). Self-wires are materially rejected by the existing construction law, despite passing typed genotype syntax. They pay copying and child endowment, record development failure and receive no rejection refund.

## Every substitution retained

Each row contains the same four predeclared world/family targets. Counts below refer only to the target child; rejected proposals have no child. Full lifetimes, refunds, payment accounts and descendant records remain in the replay artifact.

| Edit | Early /4 | Completed /4 | Reproducing /4 | Child requests | Child harvest | Funded parent ticks | Funding blocked ticks | Censored |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| g100_s1 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g100_s2 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g100_s3 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g100_s4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g100_s5 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g100_s6 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g100_s7 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g101_s1 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g101_s2 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g101_s3 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g101_s4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g101_s5 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g101_s6 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g101_s7 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g102_s1 | 4 | 4 | 4 | 4 | 384.0 | 4 | 44 | 0 |
| g102_s2 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g102_s3 | 4 | 4 | 4 | 4 | 384.0 | 4 | 44 | 0 |
| g102_s4 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g102_s5 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g102_s6 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g102_s7 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g103_s1 | 4 | 4 | 0 | 0 | 120.0 | 0 | 29 | 0 |
| g103_s2 | 4 | 4 | 0 | 0 | 144.0 | 0 | 28 | 0 |
| g103_s3 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g103_s4 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g103_s5 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g103_s6 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g103_s7 | 4 | 0 | 0 | 0 | 0.0 | 0 | 0 | 0 |
| g104_s1 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g104_s2 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g104_s3 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g104_s4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g104_s5 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g104_s6 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| g104_s7 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

## Verification and boundaries

All658 published states pass independent exact graph, event, genotype, ancestry, request, ledger and lifetime reconstruction.6 new native tests,16 relevant legacy native tests,11 new Python tests,29 legacy Python tests and11 shared-replay regression tests pass. Current references additionally verify27 states from three old profiles. The exposed recovery fixture retains both tick17 target receipts at64 after all organisms die. Pre-outcome harness invocation/schema corrections are recorded in the validation receipt; no fresh-world run is retried.

Native panel wall time 692.233s; summed advancement 366.141s; full independent audit 369.863s. Maximum segment advancement 6.532s, checkpoint 23439 bytes, segment 2838685 bytes. All prospectively fixed caps pass.156 source files and the binary SHA are bound before outcomes; the native input remains read-only during replay.

All47 preceding sealed archives /9135 files are checked unchanged. The native panel and audit archive have separate final file inventories. Worlds601/607 are now exposed development worlds; held-out211/223/227/229/233/239/241/251 remain unexposed.

Engineering exposure or edited reproduction does not establish adaptive improvement, fairness, ecological power, sustained class propagation or scientific acceptance. All science, training, evolution, power and launch flags remain false.

## Next boundary

Before changing SourceKernel or launching H3a, independently compare behavioral equivalence and inherited class propagation for every substitution against unchanged controls using these sealed traces. Retain all140 substitutions and distinguish a changed internal state connection from improved action/reproductive behavior. Keep root survival, repeated founder supply and inherited class persistence separate. New native work would require another prospective contract with fresh fixed diagnostic worlds; the scientific proposal remains conditional.
