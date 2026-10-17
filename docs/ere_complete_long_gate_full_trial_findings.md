# Complete-profile full G-R3/G-R5 trial findings

Date: 2026-09-27. Profile `E2-complete-long-horizon-gates-v1`, semantic
protocol `E2-witness-complete-v1`, family 10 / instruction 11.

## Outcome

**G-R3 and G-R5 passed in the preregistered scope.** One fresh release-build
invocation completed all 18 trajectories and exactly 540,000 advancing engine
steps. Native assertions, the frozen G-R3 prerequisite audit, the final frozen
standalone audit, and sealed archive integrity all passed. There were no retries,
replacement seeds, fuel refills, source/criteria changes, or shortened horizons.

This does not admit training or finish all complete-profile readiness work.
G-R1 mutation classification and optimizer prechecks remain pending. Neither
learned adaptation nor broad expressivity is established by these seeded motifs.

## Receipts and provenance

Archive:
[ere_complete_long_gates_full_20260927](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927).

- [run_status.json](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927/run_status.json): complete, `gate_pass=true`, `training_admitted=false`, 540,000 steps.
- [audit_GR3.json](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927/audit_GR3.json): prerequisite audit passed over 400,000 steps, 88 tamper rejections. Its overall-suite gate flag remains false because its scope is only G-R3.
- [stage_barrier.json](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927/stage_barrier.json): G-R3 native/audit passage before any G-R5 trajectory.
- [audit_full.json](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927/audit_full.json): all 18 cells passed, overall gate flag true, 396 inline tamper rejections.
- [manifest.json](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927/manifest.json): exact matrix, configurations, initial inventories/tables/program bytes, resource limits and 176 source files.
- [frozen_inputs_sha256.json](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927/frozen_inputs_sha256.json): all 181 premeasurement input hashes verify; all archived source files also match the live checkout.
- [output_sha256.json](/Users/kalewicker/Documents/hypersim/output/ere_complete_long_gates_full_20260927/output_sha256.json): exact sealed 245-file output inventory, excluding the checksum file itself; every hash verifies.

The sources, executed binary, fixtures and pinned prior bounded basis receipts
were frozen before steps. Per-cell canonical JSONL streams, complete scheduled
inventories, runtime windows and G-R3 CSVs are preserved. Old scientific and
failed/successful development archives remain unchanged. The scientific design
is byte-identical to its preregistration; subsequent approval is recorded in
`ere_complete_long_gate_full_authorization_20260927.md`, not by editing historical
“not approved/not implemented” design fields.

## Resource accounting

| Quantity | Actual | Approved ceiling |
| --- | ---: | ---: |
| Advancing engine steps | 540,000 | 540,000 |
| Assay elapsed, including stage/final audits, excluding build/freeze | 1,482.95 s (24.72 min) | 3,600 s |
| Stage/final semantic auditor elapsed | 660.89 s (11.01 min) | 900 s shared |
| Archive bytes, including source/binary and checksum inventory | 196,929,464 (187.81 MiB) | 2,147,483,648 |
| Protected binding-audit work, all seven A2 cells | 4,568 | 500,000 per cell, 50,000 per probe |
| Protected counterfactual result bindings, all A2 cells | 32 | 500,000 per cell, 50,000 per probe |

The release build took 31.27 seconds separately. Eight checker regression groups
were replayed against the existing development archive before launch, without
engine execution. The single Cargo scientific invocation took 1,483.74 seconds
including freeze/seal overhead. Post-run checksum/window-summary inspection took
approximately 0.28 seconds, with no engine steps or second full semantic replay.

No matching, store, region, wall, output or auditor limit caused a stop. The
checker enforces the 1 GiB resident ceiling while streaming; a numeric peak-RSS
receipt is not recorded, so this report does not invent one. All engine matching
used the declared single-threaded pooled 50,000 work/results policy. No optimizer,
held-out training evaluation, cloud compute or extra scientific invocation ran.

## G-R3: full static/frozen interpreter identity

| Fixture / arm | Steps | Core world events | Proposal world events | Result |
| --- | ---: | ---: | ---: | --- |
| identity_dormant / A0 | 100,000 | 100,000 | 0 | Passed |
| identity_dormant / A1 | 100,000 | 100,000 | 0 | Passed |
| oscillator_active / A0 | 100,000 | 50,000 | 50,000 | Passed |
| oscillator_active / A1 | 100,000 | 50,000 | 50,000 | Passed |

All four CSVs contain exactly 100,000 data rows plus the fixed header. Both
static/frozen pairs are byte-identical. The auditor also compares every paired
native event/binding, typed delta, guard counter, budget-use record, motif state,
full/projected hash and RNG record. Compile consumes no RNG; SequentialRandom
selection genuinely consumes it. Static and graph lifecycle reports are checked
separately rather than made artificially equal.

The dormant identity fixture really replaces edges but has no logical topology
change. The active fixture engages both seeded roles and changes exactly one
motif topology per step. Neither original proposal content nor identity-edge
replacement is reported as learned novelty. Family/inert instruction validity,
ordinary handles/data and motif boundaries remain intact throughout.

## G-R5: protected/exposed damage and finite paid repair

Every cell completed 10,000 steps. All 14 cells paid exactly one native CORE
Seal at step 1, with no preamble. There were 16 native RepairSealed payments.
All scheduled exposed payload/ownership/root hazards fired; all scheduled A2
hazards had zero protected bindings and a real visible counterfactual target.

| Cell | Final family valid | Anchor survival | Final repair fuel | Whole-family recoveries | World events |
| --- | --- | ---: | ---: | ---: | ---: |
| A2_fuel0_ownerfalse | Yes | 1 | 0 | 0 | 40,000 |
| A2_fuel0_ownertrue | Yes | 1 | 0 | 0 | 40,000 |
| A2_fuel2_ownerfalse | Yes | 1 | 2 | 0 | 40,000 |
| A2_fuel2_ownertrue | Yes | 1 | 2 | 0 | 40,000 |
| A2_fuel8_ownerfalse | Yes | 1 | 8 | 0 | 40,000 |
| A2_fuel8_ownertrue | Yes | 1 | 8 | 0 | 40,000 |
| anchor_loss_A2 | Yes | 1 | 8 | 0 | 40,000 |
| A3_fuel0_ownerfalse | No | 1 | 0 | 0 | 10,000 |
| A3_fuel0_ownertrue | No | 1 | 0 | 0 | 10,000 |
| A3_fuel2_ownerfalse | No | 1 | 0 | 2 | 29,992 |
| A3_fuel2_ownertrue | No | 1 | 0 | 2 | 29,992 |
| A3_fuel8_ownerfalse | Yes | 1 | 4 | 4 | 39,984 |
| A3_fuel8_ownertrue | No | 1 | 4 | 3 | 35,988 |
| anchor_loss_A3 | No | 0 | 4 | 4 before root loss | 39,968 |

All protected cells preserve the original complete family, body handles and
obligations for every step, with zero dangling meta cleanup. They retain four
world rewrites/topology switches per step. Seal ends exhausted at fuel 0;
RepairSealed is never needed in A2, so its fuel remains unchanged. This is paid
engagement plus protection, not continuous Seal action or continuous repair.

Exposed payload lesions occur at 2500/5000/7500/9992. Fuel 0 yields no repairs;
fuel 2 pays at 2501/5001; fuel 8 pays at 2501/5001/7501/9993. Source quarantine,
paid repair without world execution, and subsequent reinstallation follow the
declared pre-step compile convention. An invalid family never executes a cached
world table.

The ownership-loss/fuel-8 control is particularly important. Ownership erasure
at 9000 leaves the full family invalid. At 9993, the captured native relation
repair still spends fuel 5→4, with `family_valid_before=false` and
`family_valid_after=false`; both executable fields are null. It is a fourth
paid CORE repair but **not** a fourth whole-family recovery. World classes stay
empty from 9001 onward. No ownership guard, expenditure suppression or refund
was introduced.

The exposed anchor-loss control repairs all four payload lesions, then really
deletes the family root at 9996. Anchor survival becomes 0 and world classes
retire before 9997; absent roots are not counted as quarantine decisions. Body
material is not silently recreated. Native deletion/cleanup multiplicities,
unique physical edge losses and inventories agree under the existing engine
semantics. Its protected counterpart retains the root and supplies the visible
root-deletion counterfactual binding at the same scheduled prestate.

Total recorded world events are 875,924: 400,000 in G-R3 and 475,924 in G-R5.
Clock, hazard and paid-operation events are excluded from this activity count.
Generic projected hashes include the clock and are not used as a clock-free
activity claim.

## Limits and next work

The frozen checker independently replays typed graph material, bindings,
templates/effects, indices, complete-program bytes, native comparisons, payment,
visibility and lifecycle/window chronology. Rust remains the witness for
canonical graph hash generation and actual RNG/native execution; Python does
not claim to be a second engine implementation.

These are single-seed deterministic engineering controls on four labelled
motifs, not statistical replication, rich ecology, autonomous rule discovery or
learned adaptation. EI is explicitly disabled and no new G-R6 claim is made.
The earlier complete-profile expressivity result remains two supported rules
out of 45, with 43 exclusions.

Next preregister the complete-profile G-R1 mutation-classification envelope:
typed structural/metadata/ownership/obligation/instruction mutations, strata,
draw counts, invalid/unmeasured accounting, thresholds, resource limits and
independent replay. Do not silently reuse the older E2-v3 classification receipt.
Optimizer genotype-distribution, semantic-effect and signal/noise prechecks,
explicit complete-profile admission and a separately authorized training campaign
remain pending. The reserved 20 × 16 × 10,000 optimizer-step budget is untouched.
