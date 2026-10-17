# v2 engineering correction

The original v1 invocation stopped in setup with `production outside legal grammar`,
one grammar rebuild and zero attempted/advancing physics steps. Its frozen archive
and original checker failure remain unchanged at `output/ere_complete_scaling_trial_20260930`.
This v2 is the corrected implementation of the user's authorized scaling trial,
not a replacement for measured outcomes. Use the existing matched control bank:
parent 4 Activate / parent 5 Deactivate; both produce the same Core/Proposal rules
with opposite activation flags. Preflight validates legal catalogue membership
before rebuilding, and emitted decoded control equality before any physics.
All draws, seeds, horizons, ceilings, fixed denominators and decision rules remain
unchanged. Exactly one v2 measurement invocation in a fresh output directory.

# Complete-profile 50k/50k scaling trial v2

Prospective registration, 2026-09-30. User authorized the scaling trial after
receipt-bound bounded-runner verification. Profile `E2-complete-scaling-v2`.
This is a fixed-panel resource diagnostic, not optimizer training or a gate
threshold change. Exactly one fresh full invocation; no outcome-dependent retry.

## Fixed schedule and resources

Six productions: four independent zero-logit valid-grammar draws with emission
seeds 0xE653000101 through 0xE653000104, followed by parent 4 Proposal Activate
and parent 5 Proposal Deactivate controls, production index zero, seeds 0xE653000105/106. All use
the same existing fixed-world initializer; worlds may evolve during execution.
Physics seeds 1361 and 1471. Each production/physics pair is run independently
from the same initial state at horizons 100, 1,000, 10,000, in that order.
All 36 cells remain in fixed denominators; maximum 133,200 advances. No logits
are optimized, no held-out evaluations, no ES updates or stateability rerun.

Actual pooled matcher limits remain exactly 50,000 work / 50,000 results per
step, MaxParallel with one pairing. Metadata-isolated allocation and opt-in
repeated-self-pair-feasibility matching remain required. Existing native caps:
2,000 total nodes / 4,000 total edges, native matcher ceiling, invalid complete
family or nonadvancement => terminate that cell with null fitness, never zero.
Score is the unchanged reuse expression normalized by log(cell horizon);
cross-horizon fitness values are descriptive, not an optimization comparison.
Record per-step total nodes/edges, native work/results, events, world hash and
family validity; emission metadata counts are part of the existing graph caps.

Total setup+evaluation wall ceiling 1,800 seconds; per-cell cooperative wall
ceiling 120 seconds; output ceiling 1 GiB including audit/seal headroom. These
are cooperative checks, not OS preemption/RSS limits. A cell hitting wall/output
limit is retained as null/incomplete; partial native work is unknown if the
native API returns an error without a receipt. No hidden work is called zero.
Global ceiling leaves pending cells explicitly NotRun. Do not increase limits,
refill fuel, discard a failed cell or substitute a production/control.

## Evidence and implementation bridge

Verify all five exact historical seals from the preceding admission contract,
their predicates/scopes, and the integrated engineering receipt seal
`a988f131bebbf6c2e14393bf6c2226737908c628ca7111a1ca3ff1c66b4657fd`.
Historical semantic gates retain original source versions/fixtures. The original
400k/50k checker's failure remains disclosed; no historical evidence is mutated.

All 230 native/Cargo/optimizer source pins retain exact coverage. Permit one
mechanical registered change to `hg-sim-core/src/ere/signal.rs`: expose a checked
2..10,000 horizon argument in the existing native evaluator, replace only three
horizon uses (loop, completion, normalization), and add horizon/graph-size
accounting. Legacy wrappers continue at 100 steps and remove only the new
accounting fields. An exact transformation checker plus receipt parity tests
binds the bridge; no matcher, grammar, allocation, objective rule, physics or
optimizer changes are permitted. The old admission profile stays strict and
correctly refuses changed live native identity. Its frozen receipts remain valid
historical artifacts, not live permits.

Rebuild the typed grammar once and require catalogue SHA256
`ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6`.
Freeze schedule, registration, sources, executed binary and checker before
compilation/measurement. Revalidate prerequisites/source identity after setup
and at each execution boundary. The new scaling entry point validates the exact
schedule/resources before compilation or Engine work. Old training guards are
not relaxed. All failed/diagnostic archives remain preserved.

## Analysis and fixed decision rules

Report completion/null/NotRun counts with denominator 12 per horizon, actual
attempts/advances, time, graph-size and matcher peaks, growth ratios, and each
cell's result. Verify exact native per-rule pooled allowances and independently
reconstruct reuse credit/score. For every pair, shorter traces must equal the
prefix of longer traces until a recorded failure, including graph hashes,
events, budget usage, sizes, validity/caps and advances. Equal initial emissions
are required, never emitting a new production at a longer horizon.

`measurement_complete`: all 36 scheduled cells have a terminal receipt (a
cap/null is terminal); NotRun or opaque-error missing native receipt => false.
`scaling_panel_pass`: all 36 cells complete uncapped at their declared horizons,
all 12 cross-horizon triples have exact prefixes, active/dormant controls retain
expected proposal credit (horizon/zero), all resource limits respected and frozen
accounting audit passes. Report failure without retry if any condition fails.
Passing supports only these six productions/two seeds/fixed initial world, not
arbitrary optimized genotypes, large initial worlds, learned maintenance,
held-out improvement or general stochastic robustness. Zero scores are valid
completed cells, not evidence of improvement. Even a pass does not authorize
the reserved 20×16×10,000-step training campaign. Training flags remain false.

Frozen audit must verify every source/binary/output/prerequisite hash, exact
schedule/membership/configuration, fixed denominators, native counters/prefixes,
graph/matcher caps, scores and controls, with temporary-copy tamper tests. This
is provenance/accounting, not independent full physics replay. Test failures
and fixes before freezing are development evidence; no postmeasurement checker
repair is permitted as an unqualified pass.
