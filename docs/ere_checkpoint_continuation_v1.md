# Complete-witness checkpoint/continuation verification

Frozen diagnostic contract, 2026-09-26. No training admission, optimizer,
semantic-profile relabelling, or broad runner restart support is authorized.

Implement an explicit same-build binary checkpoint for the complete-witness
execution core: full store/allocation state, engine RNG/rule table/compile
history/guards/budgets, graph-source previous regions/table/initialization,
and every runtime novelty/role/maintenance/payment/window/TPM ledger.
Checkpoint only after `after_step`, with no pending events. Restore without
reinstalling families/instructions, recompilation, graph edits or fuel payment.
Reattach exactly one runtime sink; caller-owned diagnostic observers are
explicitly reattached. Reject policies, unsupported sources/profiles,
mid-step snapshots, corrupt/truncated/trailing/version-invalid files.

Scientific diagnostic budget: one seed 984927043621, all 12 combinations of
A2/A3, repair fuel 0/2/8, ownership loss false/true, 64-step baselines. Fork
from completed boundaries 0/7/8/9/10/24/25/33/40/41/64, continuing each to64
with no resealing. Compare every remaining step: full node/edge IDs/data and
indexes, native events/bindings/statistics, world/family observation, installed
rule table/revision, RNG, ceilings and complete per-step ERE metrics. Expected
damage/repair/fuel outcomes remain those of the label-separated contract.
132 persisted binary checkpoints, including step64 zero-suffix boundary checks.

Additional RNG/window control: A2/fuel8/no ownership, SequentialRandom,
64 steps, window interval16, forks7/8/9. Require actual RNG consumption and
match every remaining step and all closed windows across an interrupted window.
This is 13 baselines, 135 forks, 6,316 advancing steps total (832 baseline +
5,484 suffix), no retries or omitted failed cells. Counts are diagnostic
comparisons, not independent scientific replicates.

Also resume cell11 (exposed/fuel8/ownership) and cell12 (RNG/window control)
from boundary8 in fresh invocations of the same frozen test binary. Compare
their complete 56-step suffixes and terminal ledgers to the archived baseline.
These two additional resumes bring the total to137 continuations and6,428
advancing steps (112 additional), not new independent trials.

Persist raw baseline/fork timelines, effective configs, checkpoints, contract,
implementation/fixture sources, executed binary and checksums in a fresh
directory before assessing whole-assay success. Separate local preflights from
frozen measurements. Do not overwrite any prior experiment evidence.

The new file is diagnostic, versioned `ERECP001`, size-limited to32MiB,
length-checked with a native64-bit accidental-corruption checksum; not an
authenticated or untrusted-input format. No compatibility across compiler,
dependency or source changes is promised. Output writers, evaluator/archive,
external observer/policy state, general runner restart and training state are
outside this boundary. Full new-profile G-R0–G-R6 and optimizer stateability
are still separate requirements even if continuation passes.
