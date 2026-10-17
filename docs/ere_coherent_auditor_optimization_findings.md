# Coherent frozen auditor: profiling and optimization findings

Date: 2026-09-28. Development-only read-only profiling; no new Engine campaign.

## Result and selected implementation

Keep [verify_ere_complete_coherent_optimized.py](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_complete_coherent_optimized.py)
as the recommended development checker. In the serial fresh-process comparison,
the complete frozen pilot replay improved from **116.169 to
41.856 seconds**: **2.78× speedup**,
63.97% less wall time. The entire clock-free audit result is
exactly equal to the frozen baseline's, not merely its final pass flag.

The separate [v2 candidate](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_complete_coherent_optimized_v2.py)
also passes correctness but takes 50.766 seconds; do not select it
for integration. Its full-inventory serialization cache and per-trace unchanged-
fingerprint reuse were tested as an additional combined candidate. Constructing
exact, collision-free cache keys adds too much cost on this material. The v2
benchmark does not isolate the fingerprint-only change, so do not conclude that
fingerprint reuse alone is necessarily slower. Both candidates and every
diagnostic artifact are preserved, including the slower result.

No measured historical checker, producer, contract or receipt was rewritten.
The winning checker is a development implementation, **not** the frozen auditor
that originally accepted the pilot and **not yet** a producer-integrated science
profile. Scientific launch, numerical/gate passage and training remain false.
Do not edit the original receipt to install the optimized checker or run the
full trial with it without a new prospective profile.

## Basis and read-only diagnostic protocol

The prospective [development protocol](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_auditor_optimization_v1.md)
bounds profiling at 600 seconds and unprofiled comparisons at 300 seconds per
invocation. All runs use the sealed 100-step resource pilot as immutable input:
168 parents, 5,376 controls, 64 draws, 55 children, 22,300 audited native steps,
22 Neutral checks and 27 tamper categories. The benchmark invokes no Rust
producer or Engine; recorded world transitions are independently replayed.

[profile_ere_coherent_auditor.py](/Users/kalewicker/Documents/hypersim/scripts/profile_ere_coherent_auditor.py)
uses the archive's exact frozen helper modules for both baseline and candidates.
It snapshots checker/driver/helpers, pins the original archive seal, runs only
`--development --stage full --verify-only`, captures the complete audit result,
wall and RSS, and optionally emits cProfile data and ranked hotspots. It verifies
all receipt hashes and source pins again afterward. Outputs are confined to
fresh diagnostic directories outside the original archive, with their own seals.
Python assertions must be enabled; candidate/driver explicitly reject `-O/-OO`.
The candidate CLI also refuses freeze, seal, or any non-verify-only invocation.

The first exploratory baseline wall measurement overlapped about ten seconds of
tests. It is retained but is not the headline comparison. The reported baseline
and candidates were run serially after tests finished, with no concurrent
development benchmark/test job. Processes are fresh, but OS file caches,
background machine load and allocator state are not controlled. These are
single-run development observations, not confidence intervals or a stable
population/full-trial throughput claim.

## What the profile found

The instrumented original replay took 360.045 s.
Recursive `copy.deepcopy` consumed 289.030 cumulative seconds—about 80.3% of
instrumented wall time. Its body alone consumed 127.181 s. There were hundreds
of millions of dictionary lookup/object-identity/memo operations and 8,831,510
recursive list-cloning calls. The largest callers were per-step world replay
and edit-builder setup, repeatedly copying unchanged metadata and world data.

The winning candidate's instrumented full replay took
65.506 s. Its residual costs include inventory
validation (19.903 cumulative profiled seconds), independent complete decoding
(10.964), projected typed-world fingerprinting (8.939) and JSON material cloning
(8.001). These are instrumented costs: compare unprofiled timings below for
actual speedup. The historical baseline's assertion set and helper algorithms
were not weakened or replaced.

## Winning changes and safety argument

1. **Per-step outer-map copy-on-write.** `replay_step` copies the node and edge
   dictionaries, not every nested record. Existing values remain read-only
   within `consume_trace`. The already-validated tables have no property effects
   or relabels; the frozen binding checker only reads records; replay only
   deletes entries and inserts freshly constructed records. Every native event,
   ordered deletion/cleanup, allocation/generation, exact delta, metadata
   snapshot, typed-world fingerprint and terminal inventory is still checked.
   No graph/material/decoder result is cached in the winning implementation.
   Adding in-place data mutation or property effects later requires revisiting
   this ownership proof; do not extend the fast path silently.

2. **Detached JSON-wire clones for edit builders and allocator edge maps.**
   `material_copy` uses JSON encode/decode for known wire-shaped maps instead
   of general recursive copying. It preserves string keys, typed Int/Float/Bool
   records, numeric values, signed zero, Unicode, array order, repeated
   endpoints and generation-bearing handles, while detaching nested values.
   Builders still own every record they mutate and the independent allocation
   model still owns its edge records. This is not a general-purpose replacement
   for deepcopy: arbitrary Python object graphs, tuple semantics or non-string
   map keys are outside the JSON-wire contract.

3. **Bounded immutable handle parsing.** Cache at most 4,096 complete
   `NodeId/EdgeId(index v generation)` strings. Results are immutable numeric
   tuples, not graph objects or mutable IDs. Generation and handle kind remain
   part of the key; malformed strings still fail. The measured run had 960
   distinct keys, 7,331,463 hits and 960 misses. This cannot turn an old
   generation into a current handle.

The candidate recorded 17,407 detached material clones and 22,702 replay
outer-map copies. The latter includes extra internal tamper/context replays;
it is not a count of new Engine steps. Audited native attempts/advances remain
22,300 exactly.

## Serial unprofiled measurements

| Phase/resource | Frozen baseline | Winning candidate | v2 experimental |
| --- | ---: | ---: | ---: |
| Whole full replay wall | 116.169 s | 41.856 s | 50.766 s |
| Integrity + parent manifest | 1.287 s | 0.972 s | 1.140 s |
| All 5,376 control replays | 36.264 s | 22.178 s | 30.902 s |
| All parent trajectory replay | 55.019 s | 11.239 s | 10.859 s |
| Mutation + child replay | 21.647 s | 6.443 s | 6.766 s |
| Timing + all tamper checks | 1.907 s | 0.990 s | 1.061 s |
| Peak process RSS | 299.63 MiB | 300.22 MiB | 288.52 MiB |

Winning mutation/child replay is about
3.36× faster.
RSS is essentially unchanged in these runs; no sustained memory improvement or
full-campaign memory guarantee is claimed.

The v2 cache was bounded to 16 successful validation keys, each at most 256 KiB
(4 MiB maximum key payload); invalid/oversized inputs were never cached. It
returned fresh maps rather than cached mutable data, keyed all incidents,
indexes, typed fields, handles and ordered endpoints, and kept complete
decoding separate. Its run recorded 23,511 cache hits, 6,027 misses, 696,587
resident key bytes and 6,505 per-trace fingerprint reuses. Correctness and
boundedness passed, but combined performance regressed. It remains experimental,
not the selected implementation.

## Verification and preserved evidence

All six complete profiling/benchmark invocations returned `audit_pass=true`
and **identical clock-free result objects**, including exact resource debits,
genotype/executable counts, coverage, measurement status and all 27 named tamper
rejections. The shared result still says `numerical_pass=false`,
`gate_pass=false`, `training_admitted=false`, `development_only=true`.
It has zero Neutral soundness failures and the unchanged pilot's 10 freeze
catastrophes; optimization does not alter that scientific observation.

The winning [regression suite](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_coherent_optimized.py)
passes 21 tests: the original nine predicate tests plus differential checks on
all 64 raw/coherent edit/decoder transforms, all 32 damage types across four
representative parents, selected exact 100-step trace vectors (including growth,
Neutral and ordered-endpoint catastrophe cases), input non-mutation, typed clone
ownership, cache bounds/generation sensitivity, warm-replay tamper rejection,
read-only CLI refusal and disabled-assertion refusal.
The [experimental suite](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_coherent_optimized_v2.py)
passes 24 tests, adding stale validation/data checks, eviction/oversize behavior
and rejection of hidden changes during a no-event fingerprint reuse.
An initial v2 test incorrectly assumed parent 6 would stall; it was corrected
to use the first persisted empty-event trace as a regression fixture. No
campaign sample, classifier or acceptance criterion was changed.

Post-run seal verification checked every file in all six diagnostic outputs.
The original resource pilot's 241 sealed outputs, isolated smoke's 233, identity
diagnostic's 77, coherent-v1 smoke's 218 and failed raw full trial's 205 remain
byte-for-byte intact, with exact seal membership. No automatic scientific
retry, new native trajectory, threshold change, full trial or training occurred.

## Artifacts and reproduction

Diagnostic root:
[output/ere_coherent_auditor_optimization_20260928](/Users/kalewicker/Documents/hypersim/output/ere_coherent_auditor_optimization_20260928/baseline_timing_serial/result.json).

- `baseline_profile/`: full instrumented original replay, pstats and 60 ranked hotspots.
- `baseline_timing/`: exploratory overlapping timing, preserved but not headline.
- `baseline_timing_serial/`: clean unprofiled baseline and semantic result.
- `optimized_timing_serial/`: winning unprofiled replay and cache/copy counts.
- `optimized_profile/`: instrumented winner and residual hotspots.
- `optimized_v2_timing_serial/`: slower correct experimental replay.

Each directory includes its source snapshots, manifest/pins, result and seal.
For bounded read-only reproduction in a **fresh** diagnostic output directory:

```sh
python3 -B scripts/profile_ere_coherent_auditor.py \
  output/ere_complete_coherent_resource_pilot_20260928 \
  --checker scripts/verify_ere_complete_coherent_optimized.py \
  --output output/NEW_FRESH_DIAGNOSTIC_DIRECTORY --seconds 300
```

Do not point the diagnostic output at the measured receipt or an existing
benchmark directory. The profiler driver refuses either.

## Full-trial implications and next step

The winning mutation/child phase is 6.443 seconds for 64 draws. Scaling
the full replay phase by `10000/64=156.25` gives about
1006.697 seconds **before** fixed costs; the 900-second shared full-auditor
allowance is not conclusively satisfied.

A transparent illustrative fixed-cost proxy is
`41.856 - 6.443 + 22.178 = 57.592 s`:
one full replay's non-mutation work plus another control stage. It excludes
some freeze/seal/extra-integrity overhead not measured with this new checker.
The earlier pilot's sampled actual first-50-Neutral policy would imply roughly
5,206 children rather than repeating all 55-per-64 soundness traces.
Treating the complete mutation/child phase as trace-proportional gives about
667.467 s; scaling it as per-draw work gives about 1064.289 s.
These scenarios straddle the 900-second allowance and are not confidence
intervals, lower/upper bounds, or proof that a full trial fits. Edit replay
must run for every draw, Neutral and RCF-changing costs differ, and neither
current benchmark covers 10,000 distinct mutations. Full wall/RSS still need
fresh prospective measurement.

Next: integrate the winning changes into a **new** prospectively frozen
producer/auditor profile and obtain authorization for a fresh bounded resource
pilot. Preserve every current source/receipt and keep the same controls,
semantic checks and full-trial ceilings. Do not promote this read-only
benchmark into scientific or training admission.
