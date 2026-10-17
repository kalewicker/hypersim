# Coherent G-R1 closure campaign findings

Date: 2026-09-29. Profile `E2-complete-coherent-closure-v1`.

## Current outcome

The consolidated campaign passed G-R1. Its single authorized full assay
completed all 10,000 draws, all 168 parent baselines and all 4,886 selected
child trajectories. The frozen auditor independently confirmed every required
predicate, zero Unmeasured outcomes and all 27 tamper rejections. The run has
`status:complete`, `measurement_complete:true` and `gate_pass:true`, with a
verified root completion seal. Training admission remains false.

[Durable run status](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_closure_full_20260929/run_status.json).
[Frozen full audit](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_closure_full_20260929/audit_full.json).
[Machine-readable findings](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_closure_campaign_findings_20260929.json).

The user authorized the consolidated campaign with “ok let's try the
consolidated closure campaign”. Its contract is
[`ere_coherent_closure_campaign_v1.md`](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_closure_campaign_v1.md)
and the JSON companion. Markdown SHA256:
`75b801d0b03e0e746a8184625f582a71bdede5031f2f78abb7dba71c1b13dc6a`.
JSON SHA256:
`42bc764d215f3ac41783bdc3d7c787a8206f5bd2563a6b9daafaaeb515f6bffc`.

## Targeted trajectory prerequisite

Regenerated the exact original mutation schedule through draw 9730, checked
its request/material against the old frozen manifest, and replayed parent 139
and child 9730 from their initial encoded stores. Both completed 100 advancing
steps under pooled 400,000-work / 50,000-result limits. The child is NearNeutral.
The diagnostic took 143.840 seconds, including construction and regeneration
of 9,731 draws, and used 200 Engine attempts/advances.

Independent Python replay validated the compiled tables, pooled usage,
typed graph deltas, native events, allocation behavior, RNG witnesses and
classifications over both complete trajectories. The parent's full trajectory
and the child's first 91 measured steps agree with historical material after
excluding only the deliberately revised work-accounting fields. At attempted
step 92 the child uses 9,018 work units / 372 results and advances successfully.
Across its full trajectory, peak usage is 9,770 work / 404 results.

The independent trace audit took 28.251 seconds at 61,079,552 bytes peak RSS.
Receipts: [native replay](/Users/kalewicker/Documents/hypersim/output/ere_closure_replay_20260929/replay_result.json)
and [independent trace audit](/Users/kalewicker/Documents/hypersim/output/ere_closure_replay_20260929/trace_audit.json).

## Complete preserved-evidence audit prerequisite

The corrected auditor completed its full pipeline over the original 10,000-draw
archive: independent controls, mutation/decode/classification replay, parent and
child trajectories, pilot parity, streaming timing checks and all 27 tamper
categories. It took 686.901 seconds at 331,202,560 bytes peak RSS (315.86 MiB),
within the 900-second / 1 GiB diagnostic limits.

The external receipt has `audit_pass:true`, `gate_pass:false`, and
`measurement_complete:false`. It verifies the recorded evidence while retaining
the original incomplete child. All 340 old archive file hashes were checked
before and after; the original frozen checker, failure status and lack of a
historical completion seal remain intact.

[External corrected full audit](/Users/kalewicker/Documents/hypersim/output/ere_closure_preserved_audit_20260929/audit_full.json).

## Implementation and verification

The existing assay harness now has an explicit closure mode that verifies its
work model and all three prerequisite receipts before creating a new full-run
directory. Its ordinary mode requires the legacy work model. The closure mode
freezes the revised contract, current sources, selected matcher, executed binary,
prerequisite receipts and unchanged mutation population.

The closure auditor adapter verifies scientific contract fields are identical
to the original pinned JSON and validates prerequisite receipts from the frozen
source copies. It inherits the complete independent replay checks and streaming
timing stage. The old pilot's trajectory comparison excludes only `step.usage`
and `step.guards[*].enumeration_work`; every new usage record is still independently
checked against the existing limits. Native events, typed graph material, RNG,
guard outcomes, match results, truncation and cap flags must remain equal.

Relevant verification passed: 26 feasibility/pooled Engine tests, two closure
adapter regressions, eight streaming timing regressions, and the native fixture
matrix with 15 Python validator/classification regressions. An initial unittest
discovery command misinterpreted its positional argument as a fixture path;
the documented Rust fixture-producing entrypoint subsequently passed all 15.
The first targeted invocation stopped before any Engine execution because its
relative output path was resolved from the crate directory; the absolute-path
invocation performed the recorded 200-step replay.

## Full scientific decision

All 10,000 draws are classified with zero Unmeasured outcomes and
`numerical_pass:true`, with exact native/frozen-auditor summary agreement.
All 168 parent baselines and 4,886 selected child trajectories completed
100 steps. The independently audited rates below use every attempted draw
in each applicable denominator, including rejected edits.

| Stratum | Samples | Valid | NearNeutral | Catastrophic | Unmeasured |
| --- | ---: | ---: | ---: | ---: | ---: |
| overall | 10000 | 86.68% | 23.74% | 12.97% | 0 |
| structural | 8376 | 84.10% | 22.06% | 13.88% | 0 |
| active_parent_structural | 4159 | 84.44% | 37.70% | 10.96% | 0 |
| topology_operator | 5290 | 74.82% | 16.79% | 17.84% | 0 |
| active_parent_topology_operator | 2571 | 74.83% | 34.54% | 14.04% | 0 |
| actual_pattern_change | 4812 | 76.52% | 18.45% | 19.62% | 0 |
| active_parent_actual_pattern_change | 2345 | 76.63% | 37.87% | 15.39% | 0 |
| activation | 1624 | 100.00% | not required | 8.25% | 0 |

Native counts are 3,832 Neutral, 2,374 NearNeutral, 1,297 Catastrophic,
1,165 Divergent and 1,332 Rejected. The former Unmeasured draw is NearNeutral
in this new run. All 50 Neutral soundness checks pass; actual mode changes
are 824, including 298 trajectory-changing edits. No training has been launched.

The full test finished in 3,389.29 seconds (56 minutes 29.29 seconds).
The producer status records 3,388.701 seconds before final sealing;
its final limit check includes the seal. There were 505,400 Engine attempts
and advances. Shared auditor time recorded before the seal was 729.789 seconds;
the measured full-audit phase sum was 676.461 seconds. Peak full-auditor RSS
was 330,629,120 bytes (315.31 MiB). Peak per-step matching usage was
362,654 work / 40,492 results. Every resource predicate passed the existing
3,600-second / 4 GiB / 900-second / 1 GiB / 400k/50k envelope.

Final read-only verification checks all 364 sealed output files, all 351 frozen
inputs and all 225 live/frozen source comparisons. The complete archive has
365 files and 263,839,301 bytes (251.62 MiB). All 340 files in the historical
failed archive also still match the external preservation inventory.
There was one full invocation, no automatic retry, and no source or scientific
criterion change by this task during the full invocation.

[Captured invocation](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_closure_campaign_execution_20260929.txt).

## Meaning and next boundary

This closes the declared G-R1 searchability gate for the complete coherent
mutation assay with the opt-in feasibility matcher and its revised work model,
at classification-only 400k/50k limits. The numerical limits and scientific
criteria were retained. Repeating the fixed seed verifies the revised
implementation; it is not an independent seed replication. The original raw
mutation failure and the later incomplete coherent archive remain valid
historical findings in their own scopes.

The assay adapter has not yet become a shipped evolutionary mutation kernel.
Default matcher builds and 50k/50k runtime limits retain their prior behavior.
Other gate receipts retain their existing fixture and execution scopes.
The next work is to test the complete coherent genotype distribution, semantic
effects and fitness signal relative to noise, carrying the mutation-kernel and
matcher identities into those checks. This campaign launches no training and
does not change the existing admission guard.
