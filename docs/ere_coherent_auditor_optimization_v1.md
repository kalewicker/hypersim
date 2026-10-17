# Coherent auditor optimization: development diagnostic protocol

Date: 2026-09-28. User scope: profile and optimize the auditor. No new native
trajectories, scientific campaign, threshold change or training is authorized.

Use the sealed 100-step resource pilot at
`output/ere_complete_coherent_resource_pilot_20260928` as read-only material.
Preserve its measured checker, producer, contracts and sealed outputs. Make a
separate optimized checker development file; it is not the frozen auditor that
originally accepted that receipt and cannot retroactively replace it.

Before optimization, run one bounded cProfile full replay (600 seconds maximum).
Then compare fresh-process unprofiled baseline and optimized full replays, each
with a 300-second cap. If useful, run a separately bounded candidate profile to
locate residual costs. Every benchmark retains the full source/contract/seal,
168-parent, 5,376-control, 22,300-step, 64-draw, 22-Neutral and 27-tamper checks.
Use the identical frozen helper modules in every process. Save source snapshots,
profile hotspots, phase costs, RSS, exact clock-free semantic results and source/
receipt hashes in fresh diagnostic directories outside the measured archive.

Changes may eliminate redundant computation or copying, not validation. Any
cache must be bounded, immutable or dependency-aware and retain complete input
identity; test property/typed-data/handle/order/metadata mutations and stale-cache
rejection. Run differential fixtures and all existing independent regressions.
Require exact baseline/candidate semantic audit-result equality. Profiling wall
costs are instrumented, not directly comparable to unprofiled timing; compare
unprofiled runs and report memory tradeoffs and residual scalability uncertainty.

Do not overwrite historical evidence, silently increase failed-run caps, or
retry failed scientific trials. A future producer-integrated optimization needs
a new prospective profile and separately authorized resource pilot. Current
diagnostics leave gate, scientific launch and training admission false.

## Benchmark scheduling clarification, before final paired timings

The first exploratory baseline timing overlapped about ten seconds of regression
tests. Preserve it as exploratory evidence, not the headline comparison. Run an
additional serial fresh-process baseline and then candidate, each capped at 300
seconds, with no other development test/benchmark job running concurrently.
Use this serial pair for the reported speedup. This is another read-only timing
diagnostic, not an Engine-campaign retry or a scientific invocation. Preserve
all earlier profiling/timing outputs and retain single-sample uncertainty.

## Second candidate, after the first candidate's successful profile

Preserve optimized-v1 source/benchmarks. The residual profile motivates a
separate optimized-v2 development checker. Cache only successful inventory
structural validation by the complete typed JSON wire value, including handles,
ordered endpoints, incidents, indexes and step. Keep at most 16 keys of at most
256 KiB (4 MiB total key payload); larger inputs bypass the cache, invalid inputs
are never cached, and no decoder result or mutable graph is cached. Always
return fresh maps derived from the current input.

A fingerprint may be reused only within one trace after the independently
replayed step has established empty native events, zero property effects and
exact empty deltas. Continue all metadata, table, binding, usage, hash/RNG, cap
and terminal checks; reset fingerprint state at every trace start. Require
stale-data, invalid-incidence/index, oversize/eviction and hidden-delta tests,
plus another serial full candidate replay capped at 300 seconds and exact
clock-free equality with the frozen baseline. No Engine or full-trial launch.
