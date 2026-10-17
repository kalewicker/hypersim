# Coherent auditor per-draw overhead: findings

Date: 2026-09-28 (America/Phoenix; measurements 2026-09-29 UTC).
Scope: read-only development profiling and optimization, authorized by
“alright let's carry on” after the integrated resource pilot's recommendation.
No native Engine, new resource pilot, full scientific trial or training was run.

## Outcome

The selected development candidate cuts primary-mutation nontrace overhead
from **2.822 to 1.588 seconds
(43.72%)** on the sealed 64-draw/100-step pilot. Serial full audit replay drops
from **43.150 to 41.113 seconds (4.72%)**.
The complete scientific audit result remains exactly equal to the frozen receipt.

The optimization eliminates duplicate reconstruction of the same draw; it
does not cache successful validation, normalize world identity, omit child
traces or weaken the classifier. All four diagnostics completed successfully;
29 regressions pass; the source receipt and historical evidence remain unchanged.
This checker is selected for **future prospective integration**, not installed
into a producer or substituted for the historical frozen auditor.

## Material and prospective protocol

Source receipt:
[integrated optimized pilot](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_optimized_resource_pilot_20260928/run_status.json).
The protocol was fixed before profiling in
[ere_coherent_draw_optimization_v1.md](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_draw_optimization_v1.md).

Every run executes the complete independent frozen replay of all 168 parent
baselines, 5,376 damage controls, 64 mutation requests, 55 child trajectories,
22 Neutral checks, 27 tamper categories, exact paired canonical material hashes
and source/frozen/output integrity. Engine attempts/advances represented in the
evidence stay 22,300/22,300; no new Engine steps are performed.

Outcomes remain 22 Neutral, 12 NearNeutral, 10 Catastrophic, 11 Divergent and
9 Rejected, with zero Unmeasured/cap hits and zero Neutral soundness failures.
Scientific minimum sample/coverage requirements remain unsatisfied.
Gate, training and scientific-launch admission remain closed.

The new driver profiles only the first full-audit primary tally/generator.
Its child replay is included in that instrumented phase and explicitly split
from residual nontrace overhead. Controls, parent replay, integrity and final
tamper checks still run but are unprofiled. This locates draw-local costs
without profiling the much larger fixed control phase.

## What the targeted profile showed

| Primary-draw profile quantity | Frozen checker | Candidate |
| --- | ---: | ---: |
| Checked edit replays | 128 | 64 |
| Complete decoder calls | 439 | 183 |
| Child trajectory replays | 55 | 55 |
| Profiled tally cumulative time | 9.877 s | 8.134 s |
| Edit replay cumulative time | 3.285 s | 1.677 s |

The baseline edits each request once to select the child trajectory, then
`check_classification` edits it again and decodes it again. Each edit replay
also decodes the raw counterfactual and coherent staged graph; the caller then
decodes the staged graph yet again. Thus redundant deterministic work consumes
much of the instrumented 4.205-second nontrace phase.

Cumulative profile times are nested; decoder time is already included in some
replay/caller costs and must not be added indiscriminately. Instrumented costs
are not the serial timing comparison or a statistical confidence interval.

## Selected implementation and extension constraints

New files:

- [Development checker](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_coherent_draw_optimized.py)
- [Targeted profile/full-replay driver](/Users/kalewicker/Documents/hypersim/scripts/profile_ere_coherent_draw_auditor.py)
- [Differential/adversarial regressions](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_coherent_draw_optimized.py)

The candidate derives from the integrated profile's checker, leaving that
checker, its producer, contracts, tests and measured receipt unchanged.

`_checked_draw(parent, draw)` reconstructs and checks every raw selector,
raw counterfactual, coherent action, allocation, complete decode, receipt,
delta/topology/metadata flag, and returns fresh owned staged material plus the
decode result already derived from that material. Nothing is reused across calls
or draws, and no result is keyed by handles or stored in a validation cache.

During the primary audit, that staged material is passed to the unchanged
read-only child replay. Its derived decode result is passed to the private
`_check_classification_decoded` continuation, which checks every row field,
selector/parent relationship, compiled report, soundness flag, outcome and
denominator as before. Independently decoding each child trace's initial
material remains mandatory and unchanged.

Public `raw_edit` and `check_classification` always reconstruct/check their
specific inputs afresh. Standalone tamper checks cannot reuse a warmed result.
The private continuation is not a public caller-supplied-cache API; its normal
audit caller provides only a result produced immediately by `_checked_draw`.

Ownership boundary: do not mutate parent/request, staged inventory or derived
result between checking the draw and classifying it. The unchanged child
replayer keeps initial material read-only, rejects data effects/relabels, and
uses its own allocation model. Future mutable replay effects require a renewed
ownership proof/profile, not merely retaining this private fast path.

AST regressions verify that the original raw-replay checking body and complete
classification-checking body survive the refactor unchanged, and that decoder,
inventory mapping, world fingerprint, allocation/builders, step/trace replay,
classification and tally logic remain verbatim. Typed records, exact NodeId/
EdgeId generations, ordered endpoints, RNG, caps, hash vectors and terminal
material checks are unaffected.

The CLI is hard read-only: only development `--verify-only` controls/full
replay is permitted. Freeze, seal, receipt writing, non-development calls and
Python `-O/-OO` are refused. Existing generation-sensitive immutable handle
caching remains bounded at 4,096; no new cross-input cache is added.

## Serial, unprofiled results

Fresh processes ran serially after all regression tests ended, with no
concurrent profile/test job. Each replay had a 120-second checker allowance
and the existing 1 GiB checker RSS cap.

| Measured phase/resource | Baseline | Candidate |
| --- | ---: | ---: |
| Full driver-timed audit invocation | 43.150 s | 41.113 s |
| Integrity and parent manifest | 1.028 s | 0.997 s |
| Controls replay | 22.688 s | 22.299 s |
| Parent replay | 11.553 s | 11.182 s |
| Mutation plus child replay | 6.589 s | 5.489 s |
| Child replay, included above | 3.767 s | 3.901 s |
| Mutation nontrace, remainder above | 2.822 s | 1.588 s |
| Paired material parity | 0.230 s | 0.222 s |
| Timing/tamper checks | 1.026 s | 0.877 s |
| Peak process RSS | 310.64 MiB | 310.59 MiB |

Do not double-count the child/nontrace subrows. Driver wall includes CLI audit
and exact scientific-result comparison after imports; it excludes initial
source copying and before/after receipt sealing checks. It is not a producer
campaign wall measurement.

This is one serial pair, not a repeated statistical benchmark. Variations in
unchanged control/parent/child phases cannot be attributed to this edit.
The clear causal evidence is halving checked draw replays, reducing decoder
calls, exact preservation of all checks, and the targeted nontrace reduction.
Controls and parents dominate this small pilot's full wall cost, so a modest
whole-audit improvement can have a larger per-draw scaling consequence.

Full candidate counters: 81 checked-draw evaluations (64 primary plus standalone
context/tamper reconstructions), 17,023 material clones versus 17,407 baseline,
22,702 replay outer copies unchanged, and 7,123,690 handle-cache hits / 960 misses.
Cache membership is 960/4,096. These are diagnostic operations, not new Engine
steps or removed native evidence.

## Updated full-trial sizing scenarios

No full trial or new producer-integrated sizing pilot has been run.
Hold the measured integrated pilot's fixed pre-seal audit proxy at
60.913 seconds. Let S = 10,000/64 = 156.25.
Use actual full-policy child selection: scaled observed non-Neutral children
plus only the first 50 eligible Neutral children.

```text
fixed audit proxy             60.912656 seconds
per-draw nontrace   S × 1.588026 = 248.129044
non-Neutral replay S × 2.255924 = 352.488144
50 Neutral replays 50 × (1.644602/22) = 3.737731
policy-aware audit estimate   665.267575 seconds
```

The corresponding current serial baseline estimate is 856.301
seconds. The earlier integrated pilot estimate was 867.468 seconds; those use
different local clock samples and are not independent bounds. Candidate
headroom against the unchanged proposed 900-second shared-audit ceiling is
234.732 seconds before final seal and new-profile qualifications.

Keep the original native policy estimate 2601.625 seconds
unchanged because no native code/trajectories changed. Add candidate audit,
the observed 1.619-second producer nonphase residual and 0.524-second seal/outer
proxy: about **3,269.036 seconds**, versus the unchanged proposed 3,600-second
full wall allowance. This is illustrative clock arithmetic, not a measured
full run, a runtime guarantee, confidence interval or scientific approval.

Sensitivity scenarios: scaling all sampled valid children, including excess
Neutral checks, gives 918.499 audit seconds;
treating every one of 10,000 requests as a child at the observed mean gives
1018.228 seconds. These do not implement the actual
first-50-Neutral policy, and are not worst-case bounds. The 64 draws do not prove
future acceptance/growth/edit costs, representativeness, output or RSS scaling,
required role/operator/activation coverage, or scientific threshold passage.

**Next recommendation:** port this selected draw-local optimization into another
prospectively frozen producer/checker profile, then run one separately authorized
fresh bounded resource pilot to measure integrated costs. Do not silently edit
the existing profile/receipts or launch the 10,000-draw trial/training from these
development estimates.

## Verification, diagnostics and provenance

All 29 regressions pass: existing predicates, all 64 edit/decode comparisons,
32 damage categories on four representative parents, exact 100-step vectors,
material ownership, typed/order/generation retention, malformed warm inputs,
fresh standalone classifier reconstruction, forged row/hidden edits, AST
preservation, read-only/-O refusal, bounded/exclusive writes and scientific
comparison-field retention. An initial new test selected the wrong parent trace
stream; its fixture was corrected to filter the sampled parent's trace. No
diagnostic or campaign failed/retried, and no native Engine was invoked.

Diagnostic root:
[ere_coherent_draw_optimization_20260928](/Users/kalewicker/Documents/hypersim/output/ere_coherent_draw_optimization_20260928).

| Diagnostic | Audit wall | Instrumentation | Sealed outputs |
| --- | ---: | --- | ---: |
| baseline_mutation_profile | 46.400 s | Targeted cProfile, 300-second cap | 10 |
| baseline_timing_serial | 43.150 s | None, 120-second cap | 8 |
| candidate_timing_serial | 41.113 s | None, 120-second cap | 8 |
| candidate_mutation_profile | 43.945 s | Targeted cProfile, 300-second cap | 10 |

All four independently require exact equality with the frozen full scientific
audit. Only phase clocks, RSS, child timing lists and optimization counters are
excluded; paired material hashes, scientific flags, every coverage/control/
resource count and tamper category remain in the compared result.
All use identical frozen helper modules, driver and protocol sources.
Each diagnostic snapshots sources, retains its result/phase clocks/RSS,
rechecks source/receipt hashes afterward and seals exact output membership.

Sixteen receipt seals verify: these four diagnostics, six earlier auditor
diagnostics, the integrated pilot and five earlier relevant historical receipts.
The integrated pilot still has 261 frozen inputs, 191 live/frozen source matches
and 274 sealed outputs, with unchanged seal
`c6631d665b2deeec286104cb3173662867dc93794451e17c20f9de08090d0986`.
Only each root output seal excludes itself; nested basis seals remain covered.

Candidate SHA-256:
`979452b031ac9a6470f758e194054cce3bd8983f61f2bca6215a2c2221684bc4`.
Driver SHA-256:
`de6b3049da0856c6667ed6eb6a453f83a14ee24f62c9270f5688f175b002ec7c`.

Recorded commands (existing output directories must not be overwritten):

```sh
python3 -B scripts/test_verify_ere_coherent_draw_optimized.py

python3 -B scripts/profile_ere_coherent_draw_auditor.py \
  output/ere_complete_coherent_optimized_resource_pilot_20260928 \
  --checker scripts/verify_ere_coherent_draw_optimized.py \
  --output output/ere_coherent_draw_optimization_20260928/candidate_timing_serial \
  --seconds 120
```

The baseline uses the receipt's frozen checker path. Targeted profile runs add
`--profile --seconds 300` and use their distinct fresh output directories.
A repeated diagnostic needs a new output directory and suitable authorization;
none is an Engine-campaign retry or permission for a full trial.
