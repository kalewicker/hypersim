# Coherent G-R1 closure campaign

Date: 2026-09-29. Profile `E2-complete-coherent-closure-v1`.

The user authorized this campaign with “ok let's try the consolidated closure
campaign”. This authorizes the two prerequisite checks and, if both pass, one
fresh full assay. No additional approval is required between these stages.

The campaign uses the implemented `repeated-self-pair-feasibility-v1` matcher
and the corrected streaming timing check. It repeats the existing fixed seed
and all 10,000 mutation requests under a revised work model; it is a regression
and closure experiment, not an independent seed replication.

The parent bank, coherent mutation kernel, classifications, 100-step horizon,
all required strata, zero-Unmeasured rule, soundness checks, controls, and
scientific thresholds remain those of the prior draw assay. The total full
assay envelope is 1,016,800 Engine attempts, 3,600 wall seconds, 4 GiB output,
900 shared auditor seconds, 1 GiB auditor RSS, one producer thread, and pooled
400,000 work / 50,000 results per step. Store ceilings remain 2,000 nodes and
4,000 edges. The feasibility prepass charges its work against the same limit.

Before the full assay:

1. Reconstruct draw 9730 using the original selector schedule and initial
   encoded store. Run its parent and child for 100 steps each, with a 200-attempt,
   300-second, 512 MiB diagnostic bound. Verify exact trajectory equivalence
   through the 91 originally measured child steps, including native events,
   handle generations, deltas and RNG states. Verify the original draw material
   and initial states match. Independently audit both new complete trajectories.
2. Complete the corrected auditor over the preserved 10,000-draw archive,
   including all tamper checks, within 900 seconds and 1 GiB RSS. Write its
   receipt to a separate directory and verify every preserved file hash before
   and after. Its expected gate result is false because the original child
   remains incomplete.
3. Run the existing positive/negative feasibility and pooled-engine regressions
   to cover added probe work, finite-cap behavior, ordering and RNG equivalence.

The scientific invocation runs only after all prerequisites pass. Freeze this
contract, current sources, matcher selection, executed binary, launch receipt,
parents, mutation requests and controls before measurement. Historical source
pins are retained in the old archives; the new JSON contract pins the current
design sources separately. The new frozen auditor verifies the selected work
model and uses the same independent replay and classification checks.

For the prior pilot comparison, compare parents, controls, mutations and
classifications exactly. Compare pilot trajectories exactly after excluding
only pooled usage records and guard `enumeration_work` values, whose work model
has deliberately changed. Independently validate every new usage record against
the unchanged limits. Keep all native events, graph material, RNG states,
guard evaluations, match results, cap flags and trajectory decisions in this
comparison. Record the accounting exception explicitly in the audit receipt.

If a prerequisite fails, stop before the full invocation and report the cause.
If the full invocation fails, preserve its evidence and stop this campaign.
There is no automatic second full invocation or criterion/budget adjustment.
Passing requires complete measurements, independent audit, native/auditor
agreement, all scientific predicates and a root completion seal. Training
admission remains false pending the optimizer signal checks.

Campaign outputs:

- `output/ere_closure_replay_20260929`
- `output/ere_closure_preserved_audit_20260929`
- `output/ere_complete_coherent_closure_full_20260929`
- `docs/ere_coherent_closure_campaign_findings_20260929.md` and `.json`
