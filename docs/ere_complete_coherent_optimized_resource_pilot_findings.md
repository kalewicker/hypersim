# Integrated optimized coherent resource pilot: findings

Date: 2026-09-28 (America/Phoenix; measurement occurred 2026-09-29 UTC).
Profile `E2-complete-coherent-optimized-resource-pilot-v1`.
Archive: [sealed pilot](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_optimized_resource_pilot_20260928/run_status.json).

## Outcome and admission boundary

The separately frozen optimized-v1 producer/checker integration completed its
single authorized 64-draw, 100-step resource pilot, with every required control
and independent audit passing. All seven canonical material streams and the
classification summary are exactly equal to the original resource pilot.
This is successful **development/resource measurement**, not scientific
acceptance. No full 10,000-draw assay, production mutator/store migration,
optimizer campaign or training was launched. No limits or criteria were relaxed,
and no campaign retry occurred.

| Quantity | Measured result |
| --- | ---: |
| Parent baselines | 168, all complete for 100 steps |
| Child trajectories | 55, all complete for 100 steps |
| Engine attempts / advances | 22,300 / 22,300 |
| Damage controls / matched clean siblings | 5,376 / 5,376 |
| Relation roles / obligation keys | 13 / 26, explicit zero included |
| Neutral / NearNeutral | 22 / 12 |
| Catastrophic / Divergent / Rejected | 10 / 11 / 9 |
| Unmeasured / cap hits | 0 / 0 |
| Neutral soundness checks / failures | 22 / 0 |
| Independent tamper categories rejected | 27 |
| Unique genotype / executable-RCF parents | 168 / 112 |

`measurement_complete` and `audit_pass` are true. `numerical_pass`,
`gate_pass`, `scientific_run_launched` and `training_admitted` remain false.
Only 64 of the required 10,000 draws and 22 of the required 50 Neutral checks
were measured; scientific coverage/minimum-count requirements remain unsatisfied.
The long-horizon catastrophes, late growth/freezes and classification precedence
are the same as in the original pilot; they were not removed by the optimization.

Total matching work/results remain 41,122,494 / 4,075,735; peaks are
18,396 / 2,388, below the unchanged 400,000/50,000 per-step limits.
The complete clock-free scientific audit result is equal to the original
`audit_full.json`, after excluding only the prospectively added parity/timing/
optimization diagnostics and clock/RSS fields.

## What was integrated and where to extend it

The original producer, frozen checker, selected development checker, slower v2
candidate, historical contracts and receipts remain unchanged. The new profile
has separate files:

- [Prospective Markdown contract](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_optimized_resource_pilot_v1.md)
  and [exact JSON contract](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_optimized_resource_pilot_v1.json):
  parameters, authorization boundary, source/evidence hashes and paired parity.
- [Rust producer and tests](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_complete_coherent_optimized_resource_pilot.rs):
  fresh output, complete parent/control/draw freeze, controls-first barrier,
  Engine trajectories, native classification, shared audit budget and final seal.
- [Independent optimized checker](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_complete_coherent_optimized_resource_pilot.py):
  material/allocator/typed-world replay, independent classification and accounting,
  exact paired comparison, cap checks and stage receipts.
- [Native-fixture checker tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_complete_coherent_optimized_resource_pilot.py)
  and [integration/differential tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_coherent_optimized_integration.py).

The Rust source is text-identical to the original producer after normalizing
profile/file/test/environment names and the two new evidence-basis keys.
Native mutation, fixture, allocation, matching, trajectory selection and
classification code therefore did not change.

The port's material-copy, handle parser, inventory mapping, world fingerprint,
replay, trace consumption, decoder, raw edit, builder/allocation classes,
classifier and tally are AST-identical to the winning optimized-v1 implementation.
It uses detached JSON-wire map clones, read-only nested values with copied outer
replay maps, and a 4,096-entry immutable generation-sensitive handle cache.
No inventory-validation, decoder or fingerprint-result caching is integrated.
The rejected combined-v2 candidate is not used.

Extension constraint: existing nested replay records must stay read-only.
Current checked tables reject data effects/relabels and replay creates/deletes
records rather than editing shared values. Supporting in-place material effects
requires a new ownership proof and prospectively revised profile, not quietly
extending the optimization to mutable records.

The new CLI refuses old profile receipts, non-development invocation, Python
`-O/-OO`, write-stage `--verify-only`, symlinks, root-external writes and
overwrites. Writes use exclusive creation and check output limits before writing.
The new producer/checker workflow generates this profile's freeze/controls/full/
seal receipts; it has no full-science entrypoint.

## Exact evidence parity and integrity

The final independent audit compares SHA-256 of decompressed canonical bytes for:

1. `parents.jsonl.gz`
2. `draw_manifest.jsonl.gz`
3. `control_manifest.jsonl.gz`
4. `controls.jsonl.gz`
5. `parent_traces.jsonl.gz`
6. `child_traces.jsonl.gz`
7. `classifications.jsonl.gz`

All are identical to the pinned original 100-step resource pilot.
Parsed classification summaries also match exactly. The frozen
[audit receipt](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_optimized_resource_pilot_20260928/audit_full.json)
contains all seven decompressed hashes. Gzip headers, profile envelopes,
native clocks and audit clocks are excluded only from paired semantic parity;
their actual files are still covered by frozen/output hashes.

Verified 261 frozen inputs, 191 live/frozen source matches and 274 sealed outputs.
Only this archive's root output seal excludes itself; nested basis seals remain
covered. All five earlier relevant historical receipts and all six optimization
diagnostic receipts also verify, unchanged, including the rejected candidate.
The new output seal is
`c6631d665b2deeec286104cb3173662867dc93794451e17c20f9de08090d0986`.

The larger archive includes pinned original material streams and prior
benchmark/source evidence for prospective parity. This is intentional provenance
overhead, not newly produced Engine evidence.

## Resource measurements

| Resource | Original pilot | New integrated pilot | Pilot ceiling |
| --- | ---: | ---: | ---: |
| Outer invocation wall | 277.710 s | 182.050 s | 900 s producer wall |
| Pre-seal producer wall | 277.110 s | 181.526 s | 900 s |
| Shared pre-seal auditor wall | 160.923 s | 67.704 s | 600 s including seal |
| Final output | 11,329,482 bytes / 10.80 MiB | 18,367,235 bytes / 17.52 MiB | 512 MiB |
| Full-checker peak RSS | 289.80 MiB | 317.36 MiB | 1 GiB |
| Engine attempts | 22,300 | 22,300 | 23,200 |

The integrated pilot is 1.53× faster overall, using 34.45% less outer wall;
shared pre-seal audit wall falls 57.93%. Peak full-checker RSS is
332,775,424 bytes, safely below its limit but higher than the historical pilot.
No inference of reduced memory usage is made.

Status timing is captured before sealing, as in the original producer.
The final seal is charged to the producer's shared audit limit and final cap
check, but its duration is not retroactively written into sealed status.
The 0.524-second difference from pre-seal status to outer completion includes
seal/finalization and Cargo/process overhead; it is not an independently measured
seal-only time. The campaign test completed successfully in 181.89 seconds and
the shell's built-in timer reported 182.05 seconds. No sandbox-sensitive
`/usr/bin/time -l` wrapper was used.

### Native work, unchanged policy

| Native phase | Original | New |
| --- | ---: | ---: |
| Damage controls | 15.644 s | 15.646 s |
| Parent baseline phase | 72.954 s | 70.877 s |
| Mutation phase, including children | 25.940 s | 25.679 s |
| Child traces, subset of mutation phase | 24.667 s | 24.428 s |
| Mutation nontrace overhead | 1.273 s | 1.251 s |

The 22 Neutral child traces consume 9.724 seconds;
the 33 other valid child traces consume 14.704 seconds.
Mean native child time is 0.444 seconds. Local variation in
native clock costs is not a change in executable evidence.

### Frozen full-audit phases

| Phase | Original frozen pilot | New integrated pilot |
| --- | ---: | ---: |
| Integrity and parent manifest | 1.293 s | 1.102 s |
| Controls replay | 36.879 s | 22.512 s |
| Parent trajectory replay | 58.990 s | 11.387 s |
| Mutation and child replay | 22.668 s | 6.792 s |
| Child replay, subset of preceding row | Not separately measured | 3.916 s |
| Mutation nontrace, remainder of that row | Not separately measured | 2.875 s |
| Additional paired material comparison | Not present | 0.215 s |
| Native-timing and tamper checks | 2.001 s | 0.909 s |
| Non-overlapping phase sum | 121.831 s | 42.916 s |

Do not add the child/nontrace subrows to their parent phase a second time.
The sum is 2.84× faster, including the new parity cost; it excludes Python startup,
receipt writing and stage orchestration. Standalone controls-stage replay took
22.730 seconds plus 1.205 seconds integrity/parents. Its work is intentionally
repeated in the full audit.

Neutral replay totals 1.654 seconds over 22 children;
other valid replay totals 2.263 seconds over 33.
The mean child replay is 0.071 seconds. Diagnostic counters:
17,407 detached material clones, 22,702 replay outer-map copies, and 7,331,463
handle-cache hits / 960 misses (960 live entries, maximum 4,096).
These include auditor-internal checks, not additional native Engine steps.

## Full-trial sizing: narrow point-estimate margin, not approval

The full scientific allowances remain unchanged and unapproved: 3,600 seconds
producer wall, 900 seconds shared auditor wall, 4 GiB output, 1 GiB auditor RSS
and maximum 1,016,800 advancing steps. This pilot does not support running
10,000 draws without a new prospectively frozen profile.

Let `S = 10,000/64 = 156.25`. The new timing split removes the prior ambiguity
between per-draw editing/classification overhead and per-child trace replay.
A policy-aware point estimate keeps all 168 baselines and controls fixed,
scales observed non-Neutral child work, and includes only the full policy's
first 50 eligible Neutral child checks.

Auditor estimate:

```text
fixed pre-seal proxy = 67.704168 - 6.791512 = 60.912656 seconds
per-draw nontrace    = S × 2.875013        = 449.220709 seconds
non-Neutral replay  = S × 2.262888        = 353.576283 seconds
50 Neutral replays  = 50 × (1.653612/22)  =   3.758208 seconds
total                                        867.467856 seconds
```

The fixed proxy includes freeze/startup, controls stage, parent replay, full
controls replay, parity and tamper work already measured before sealing.
It excludes final seal time and is not a proof that source/provenance overhead
or paired comparison stays fixed in a future full profile.
Only 32.53 seconds (3.61%) remain against 900 before those qualifications.
Mutation nontrace overhead alone contributes about 449 seconds after scaling.

Native policy estimate is 2601.625 seconds:
fixed native controls/parents 86.523, scaled nontrace
195.454, observed non-Neutral child time scaled by S,
plus 50 Neutral children at their observed mean. Add the auditor estimate:
3469.092 seconds. Adding the observed 1.619-second
nonphase producer residual and 0.524-second seal/outer proxy yields about
3,471.236 seconds, leaving roughly 129 seconds under the 3,600-second full wall
cap. This is a clock-based scenario, not guaranteed headroom.

For sensitivity, scaling the entire observed mutation/replay phase, including
all sampled Neutral children, gives an auditor estimate of
1122.086 seconds; treating all 10,000 draws as children at the
observed mean gives 1222.224 seconds. Those are not the actual
first-50-Neutral policy and are not worst-case bounds. Corresponding native
scenarios are 4098.856 and
4723.436 seconds. The policy-aware step estimate is
537,425, including parents and 50 Neutral children; fractional expected draw
counts are an extrapolation, not an observed future campaign.

The 64 requests do not establish representativeness, tail costs, memory/output
scaling, full scientific coverage or a confidence interval. The full draw set
can differ in acceptance, freezing, growth and edit/decoder costs. Current
exact-parity checks compare only this pilot to its 64-draw predecessor; they
are not a ready-made full-campaign acceptance contract.

**Recommended next bounded task:** profile the remaining per-draw
selector/material-edit/decoder/classification overhead read-only on this sealed
receipt, preserving every independent check and all historical evidence.
Its projected 449-second contribution is now the main actionable scaling cost.
Seek more resource margin before committing to a newly preregistered full trial;
do not auto-launch or increase caps because this point estimate narrowly fits.

## Verification and reproduction

Before measurement:

- 11 release Rust tests passed; the campaign test remained ignored.
- Rust-emitted all-operator/both-role fixture matrix passed all 15 Python checks.
- All 24 integration/differential Python tests passed, including AST identity,
  input ownership, exact 100-step replay vectors, type/order/generation retention,
  historical-receipt refusal, exclusive bounded writes, symlinks and `-O`.
- A fixture-only macOS `/var` versus `/private/var` alias mismatch was fixed
  by resolving its temporary root. Final hashes were repinned before measurement.
  No campaign failed or was retried.

Executed exactly once from the repository root:

```sh
time env ERE_COMPLETE_COHERENT_OPTIMIZED_RESOURCE_PILOT_OUTPUT=/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_optimized_resource_pilot_20260928 \
  cargo test --release --locked -p hg-sim-core --features hg-meta \
  --test ere_complete_coherent_optimized_resource_pilot complete_coherent_optimized_resource_pilot \
  -- --ignored --exact --nocapture --test-threads=1
```

Do not repeat into this existing archive; fresh-directory refusal is intentional.
After completion the frozen checker also passed a separate **read-only**
full replay in 42.837 outer seconds under a 120-second verification allowance.
That post-seal verification ran no Engine steps, wrote no receipt, and is not
part of the producer's campaign resource measurement. Hash checks confirmed
the new and historical receipts remained intact afterward.

```sh
python3 -B output/ere_complete_coherent_optimized_resource_pilot_20260928/source_snapshots/scripts/verify_ere_complete_coherent_optimized_resource_pilot.py \
  output/ere_complete_coherent_optimized_resource_pilot_20260928 --development --stage full --verify-only --seconds 120
```

Timing evidence is locally attested, not independently reconstructed.
The preregistration, source/binary snapshot, canonical streams, audited
classifications, frozen input list and output seal are the durable record.
