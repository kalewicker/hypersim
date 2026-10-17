# Selfmod v13 — subsequent fission and competitive establishment

Frozen before implementation and new continuation output, 2026-09-27.
This is a conditional mechanism comparison on v12 checkpoints, not new-seed
validation or a retuned economy. Work stays inside the isolated LK workspace.

## Inputs and panel

Use only the final validated v12 birth panel
`output/selfmod_v12_birth_v2_20260926`, seeds 11, 17, 23, 31, 47, 61, 83, 101.
Require its completed status, independent audit and exact replay verification.
Copy and hash each saved pre-fission `fork.json`, the faithful/prune post-birth
`birth.json`, and faithful/prune `score.json`, `rows.jsonl` and `final.json`.
Freeze source archive, binary, protocol and input hashes before scientific work.
Do not rewrite prior evidence. No shared prefix is advanced in v13.

For each seed, replay the same complete initial funded birth phase for faithful
and named R1-pruned offspring, exactly as v12. All siblings, transfers (50),
allowances, metadata and post-birth RNG states must match the saved v12 birth.
Fork each post-birth state into subsequent fission ON and OFF. OFF skips only
the funded-fission call starting at continuation step 1, consuming no birth
RNG draws, paying no future transfers and creating no future children. It
still runs native influx, rent/starvation, object matching/effects, proportional
income and allowance refill. ON remains native full fission with faithful
future children. Initial costs, resident copies and RunConfig are identical;
the switch is an explicit harness intervention, not a threshold/cap trick.
Root identities and lineages remain observational sideband records.

Four arms: faithful_on, faithful_off, prune_on, prune_off. Each advances exactly
2,000 additional object steps. Same operating point and ceilings as v12:
random tie break, match cost 0, rent 1, pool 12, influx 20/free cap 500,
full fission threshold 100/transfer 50, allowance max 200/refill 1,
canonical match cap 64, enumeration ceiling 50,000, world ceilings 500 nodes
and 1,000 edges, hard node stop 20,000. Mutation/abio/p_dup/p_del remain zero.
Do not replace seeds, modify horizons, tune prices or repair biological failures.
Continue logging after extinction; runtime/hard-stop failures are incomplete.

32 continuations times 2,000 = 64,000 scientific advancing steps, zero new
prefix steps. The ON trajectories must exactly reproduce saved v12 raw rows,
verdicts and final states. They are rerun controls, not independent replications;
only 16 OFF trajectories are new scientific conditions. Do not pool v12 and
v13 duplicates. Separate engineering preflight uses saved v12 precision-repaired
preflight checkpoints (7/42), four arms each, 64 steps: 512 excluded steps.
Separate-process seed 11 faithful_off/prune_off replay adds 4,000 excluded
verification steps. No automatic follow-up campaign.

## Criteria and interpretation

Unchanged bounded establishment criterion: executable child, complete uncensored
2,000 steps with no enumeration skips anywhere, selected-child descendants
alive at the end and productive in every final 250-step window, lineage income
at least lineage rent in the final 1,000 steps, and each seed member alive at
the end and productive in every final 250-step window. Require 6/8 to call an
arm replicated establishment. OFF descendants cannot be born, so lineage
survival there is direct child survival. Do not relax rent or window thresholds.
Invalid/censored productivity rates remain null; measured inactivity is zero.

Primary comparison: paired establishment ON versus OFF for each genotype.
Report seed-wise gains, losses and unchanged outcomes, whole-cycle persistence,
direct child/parent survival, lineage net income over the late interval,
per-live-copy productivity, copy-time, final copies/capital, resource gates
and enumeration skips. Survival alone does not establish self-financing.
If OFF rescues establishment in at least 6/8, conclude ongoing reproduction
is a limiting mechanism at this operating point; that still bundles future
birth transfers, population size, resource allocation and competition. It
does not isolate a pure birth-price or ancestry-drift effect. If OFF does not
rescue, resident competition/material constraints remain candidates; describe
observed gates without asserting an untested causal explanation. ON/OFF need
not keep the same scheduler RNG state after omitted births, but object tie
randomness is keyed independently; audit and replay all actual draws.

## Validation

Keep exclusive fresh outputs, streamed raw rows, birth checkpoints, final
snapshots, every gene/transfer and status. Test exact shared birth and first
object phase across the switch, disabled-fission eligibility suppression even
when parents qualify, conserved tokens and native refill, no birth RNG drift,
plus native ON parity. Audit every money/genealogy/allowance/gate/verdict row
independently using the existing v12 auditor primitives with an explicit OFF
eligibility check, never relaxing native eligibility for ON. Reject corrupted
OFF births and missing ON births as well as money/RNG/fate corruptions.
Freeze all engineering fixes before scientific output; any necessary later
repair gets a new output directory with retained failures and explicit parity.
