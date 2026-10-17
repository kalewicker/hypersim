# Witness-backed sealing and paid repair v1: frozen contract

Preregistered 2026-09-26 before implementation results/scientific execution.
Assay and semantic protocol `E2-witness-repair-v1`; separate from all prior
vocabulary/gate receipts. No optimizer, held-out work or full training admission.

## Representation and intervention

The prior A3 fixture loses labels on binary schema edges under ordinary binary
world rewrites. Add explicit graph-resident ternary witnesses `[root,a,b]` for
each binary schema relation, with a finite witness label identifying its role.
Family root version5, new instruction version6. Ordinary version3 families and
version4 instructions remain unchanged. Witnesses are initialized material,
not discovered redundancy and not host caches. Witness-node count is unchanged;
extra graph edges and matching work must be counted. No world/meta matcher shield.

Each schema payload is either its original typed binary edge `[a,b]` or an
explicit typed sealed ternary edge `[a,b,root]`. The new bounded reader requires
exactly one correctly typed payload for every witness, validates the resulting
whole E2 family and rejects missing, damaged, duplicate or ambiguous payloads.
Ternary-to-binary decoding is representation interpretation, never insertion
into the live graph or filling a missing relation from a witness.

Three finite native macros, compiled to ordinary guard/effect/rewrite rules:
`WitnessActivate` sets dormant->active using the graft witness; `Seal` converts
an intact binary payload to its sealed ternary form; `Repair` converts a label0
binary payload with exactly the witnessed original endpoints to the correctly
typed sealed ternary form. Each firing subtracts exactly1 Float fuel. Seal and
Repair expand over a fixed finite schema-role catalogue, not inferred opcodes.
WitnessActivate has priority110; Seal/Repair priority100. No fuel replenishment,
reader repair, host runtime edits, static editor or fresh nodes. Missing edges,
changed endpoints, node/property damage and damaged witnesses are not repaired.
The matcher can remain active even when the whole family is quarantined: local
witness/fuel guards, not a host validity oracle, determine paid repair matches.

Sealing protects only against the declared binary world productions by arity,
not arbitrary damage or ternary world rules. This is engineered redundancy and
finite repair, not evolved maintenance, learning or full reflection.

## Fixed bounded diagnostic

Reuse the explicit four-family catalogue from the persistence control: binary
identity core priority0, dormant path-to-star proposal priority1. Four common
seeds starting `0xE5523A0001`; 4x4 periodic binary PureCrystal; MaxParallel16;
thread1; epoch10; global50k work/50k results; meta-region500; family fuel0;
EI/evaluator/legacy causal off; metrics interval1;100 steps per attempt.

Every condition starts with the same family/witness material and five isolated
instruction nodes: one WitnessActivate (fuel1); two Seal (fuel16 each when
enabled, otherwise0); two Repair (fuel32 each when enabled, otherwise0).
Maximum initial total fuel97, within128. Only arm and initial fuel vary.
Six conditions in order, each seed: A3 neither, A3 Seal only, A3 Repair only,
A3 both, A2 neither, A2 both. **24 attempts / 2,400 requested-step ceiling**.
No retries, tuning, seed filtering, optimized material or historical causal
comparison. Unit tests and isolated damage controls are separate engineering
checks, not additional research attempts or optimizer evaluations.

## Measurement and decision

Shared production loop, read-only per-step capture plus complete window traces.
Record original family validity/mode/execution bytes, installed world bytes,
projected world hashes/counts, exact RNG probes, caps, paid-operation traces,
fuel states, first invalid steps and real recovery transitions. Aggregate trace
before/after is step-level, not causal attribution to an individual rewrite.

Require all24 attempts complete with all100 measured steps, conserved fuel,
one unit per paid firing, no caps/missing measures, no compile RNG draw or
invented world credit. Record prevention-only outcomes even if prevention loses
the race before all relations are sealed. A narrow A3 combined maintenance
control passes only if every seed has all4 final families valid, at least one
measured invalid->valid recovery, paid Repair firing(s), and greater valid-time
fraction than its A3 neither control. Also require installed proposal world
firing(s), no static editor, valid fuel states and all paid counts within97.
Repair-only and prevention-only are independent descriptive contrasts, never
pooled or selected to rescue a failed combined result. A2 references must keep
all families valid and isolate basic representation/native-operation validity.

Report R/T/F/J descriptively with four-family denominators. If combined recovery
fails or is incomplete, report that outcome without changing the fixture,
budget or decision. Even a pass is this engineered finite witness-backed
mechanism only: full semantic gates/stateability and training remain pending.

## Preservation and artifacts

Fresh output `output/ere_witness_repair_20260926`: frozen contract, all effective
configs before evaluation, executed binary, source snapshots,24 raw receipts,
every-attempt/condition status, conservation and contrast audit, test evidence,
checksums and separate findings. Preserve old raw receipt bytes. Legacy training
routes reject this semantic profile. Findings never alter this frozen contract.
