# One-use activation persistence control v1

Preregistered 2026-09-26 before execution. This is a narrow mechanism test,
not training, a repair experiment or a full vocabulary gate/stateability pass.
The instruction vocabulary/engine semantics stay v1; the assay protocol is
`E2-one-use-activation-persistence-v1`.

## Fixed intervention and controls

Four identical explicit E2 families: binary identity core (priority0), dormant
path-to-star proposal (priority1): LHS `[a,b],[b,c]`, RHS `[a,b],[a,c]`.
This is an existing E2 catalogue production, deliberately chosen as a controlled
matchable positive control, not learned or optimized seed material.

One isolated Activate instruction per rollout. Edit condition fuel1; no-edit
negative control fuel0. Both carry the same instruction opcode/node and compile
the same initial table. Native guards/effect pay one unit; no Deactivate,
Transpose, replenishment, host-side mode write or reader repair. No instruction
timing/guard changes. The exhausted instruction persists and cannot fire again.

Same 4x4 periodic binary PureCrystal, MaxParallel max16, thread1, epoch10,
runtime global50k work/50k results, meta-region500, family fuel0, EI/evaluator/
legacy causal off. Four common engine seeds starting `0xE552390001`.
A1 frozen table, A2 protected/live, A3 unprotected/live; edit and no-edit each:
**4 x 3 x 2 x 100 steps = 24 attempts / 2,400 requested-step ceiling**.
Metrics interval1. No retry, seed selection, tuning, optimization or held-out run.
Unit/observer controls are separate from these fixed research attempts.

## Frozen measurements and decision

Use the shared production loop with read-only optional per-step capture. Record
pre/post meta-excluded world hashes, world node/edge counts, full installed world
candidate execution bytes in rule-ID order, revision, pre/post compile and
post-step RNG probes, actual match work/results/caps, and original seeded family
validity/mode/execution bytes. Keep complete window traces and trailing partial
error rows. A hash is a diagnostic, not a stored full graph or collision-free
identity proof. Decode failures in installed world-table capture are explicit.

A successful protected persistence control requires, for every common seed:

1. Exactly one paid activation at step1, fuel1->0, no later encoded firing, no
   static editor. All seeded families/instructions remain valid for100 steps.
2. A1/A2 edit trajectories share initial installed world bytes, world hashes and
   RNG through completed step10. One family remains active after the edit and
   its decoded execution bytes stay changed through the compile boundary.
3. At advancing step11 (compile at store step10), A2's installed world bytes
   change and differ from A1's frozen bytes. A2 has genuine structural-proposal
   world firing(s) and a different meta-excluded world hash by step100. Record
   exact first table difference, world difference and credited world firing.
4. A1/A2 no-edit traces remain identical in installed world bytes, world hashes
   and RNG throughout; no active family or structural-proposal credit appears.
5. Compilation consumes no RNG; all attempts stay within original caps. A cap,
   malformed capture, missing steps or failed assertion makes the affected
   contrast incomplete/failed, with no inferred success or retried attempt.

A3 is an exposure diagnostic, not a required success: record whether edited
material stays valid through the boundary, table/world differences and family
loss/recovery. Never reinterpret A2 protection as unprotected maintenance or
meta-only code changes as world effects. No-edit A3 differs in exposure from A1;
do not attribute that contrast solely to compilation. Native local activation
may still fire in a family concurrently damaged elsewhere; step traces are
aggregate observations, not event-level causal damage attribution.

Report R/T/F/J descriptively using the existing four-family score convention;
no new initial genotype novelty/discovery claim. Full training remains refused,
regardless of this control's outcome. All comparisons use predeclared seeds and
fixtures; no previous pilot is a causal comparator.

## Artifacts and implementation scope

Fresh `output/ere_edit_persistence_20260926`: preregistration snapshot, manifest
with all effective configs before evaluations, executed binary copy, source
snapshots,24 raw receipts/full traces, contrast decisions, status, independent
arithmetic/contrast audit, corruption controls, report and checksums. Preserve
prior raw vocabulary/gate/pilot receipts. Changes are limited to optional
diagnostic capture, a bounded driver/CLI, tests and documentation; no matcher,
conflict, apply, decoder vocabulary or production-default change.

## Findings

Pending; append after the frozen assay without revising the contract.
