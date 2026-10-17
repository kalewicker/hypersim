# Complete-profile optimizer signal findings

Date: 2026-09-29. Profile: `E2-complete-optimizer-signal-v1`.

## Outcome and next decision

The single preregistered diagnostic completed all 120 scheduled attempts, but
the optimizer precheck does **not** pass. Genotype diversity, actual executable
distribution movement and a clear positive control are present. The blocker
is invalid emitted genotypes, not matching performance or a flat objective.

94 attempts completed all 100 native Engine steps. The other 26 were rejected
before physics and retain null scores. There were no matching/store caps,
invalid post-step families or shortened native trajectories. Total attempted
and advancing Engine steps: 9,400, below the 12,000 ceiling. Measurement took
7.543 seconds. Training admission remains false; the reserved
20-generation x 16-candidate x 10,000-step campaign was not launched.

Next recommended change: a validity-preserving, typed emission grammar. It
should construct only legal selector/operation combinations before physics,
using complete decode/compile validity, never fitness or outcome-dependent
resampling. The generic 13-operator kernel and G-R1 damage controls should
remain available. Do not delete or replace rejected members in this archive,
weaken the 20/20 pair requirement, or replace null scores with zero. Freeze
the revised grammar prospectively and then evaluate the same checks under
a separately specified fresh panel. No further panel was launched here.

## Fixed checks

| Check | Observed evidence | Decision |
| --- | --- | --- |
| P1 executable effect | Active forced control has a signature absent from the initial 64-emission panel | Pass |
| P2 actual distribution traversal | Maximum common-seed TV is 0.0625, above 0.05; only 14/20 antithetic pairs are usable | Fail: validity requirement |
| P3 signal exceeds initial baseline noise | Active-minus-dormant mean gap is 1.0; 3/8 baseline genotypes are rejected, so the fixed-denominator baseline SD is null | Not evaluable; fails closed |
| P4 nonsaturated initial mean | The same incomplete baseline prevents a fixed-denominator mean | Not evaluable; fails closed |

The initial emission panel has 57/64 admitted genotypes and 52 distinct admitted
executable signatures. Rejected emissions remain a null category in the
64-member empirical distribution denominator. No rollout is added, replaced
or retried because of these outcomes.

Distribution TV by completed generation: 0, 0.015625, 0.015625, 0.015625,
0.0625. Usable pairs by generation: 3, 3, 2, 3, 3. Generations 1 and 3 make
no update; all valid pairs in generation 1 are tied at zero, and both arms
of each usable pair in generation 3 have equal scores. The other generations
make objective-supported updates. This is important because the previous
index-based tie ranking would have moved parameters even on flat fitness.

## Baseline and control accounting

Parent IDs refer to the unchanged 168-parent complete-profile semantic bank.
Operator IDs follow the preregistered 13-operator coherent kernel ordering.
Each row has four physics seeds; no invalid row is excluded from admission.

| Emission | Parent | Operator / role | Status | Four-seed scores |
| --- | --- | --- | --- | --- |
| 0 | 116 | RemoveSlot / Proposal | RejectedPostcondition: Slots | All null |
| 1 | 156 | RemovePatternEdge / Proposal | RejectedPostcondition: Variables | All null |
| 2 | 103 | AddVar / Proposal | Applied | All 0.8854260058 |
| 3 | 77 | ChangeLabel / Core | Inapplicable | All 0.8961958447 |
| 4 | 71 | Activate / Proposal | Applied | All 0 |
| 5 | 37 | RetargetSlot / Proposal | Applied | All 1 |
| 6 | 163 | AddSlot / Core | RejectedPolicy: arity cap | All null |
| 7 | 139 | ChangePriority / Core | Applied | All 0 |

The forced active and dormant controls use the same parent 4 and starting
ordinary world, with coherent Activate/Deactivate edits. Active control:
100 structurally distinct Proposal rewrites, R=1, across all four seeds.
Dormant control: no credited Proposal rewrites, R=0, across all four seeds.
Priority-only clones never earn structural reuse credit.

Across all 120 attempts: baseline 20/32 valid; controls 8/8 valid; ES 66/80
valid. Rejections are 22 complete postcondition failures and four arity-policy
failures. Operator breakdown over attempted evaluations: RemoveSlot 10,
RemovePatternEdge 10, AddSlot 4, RetargetSlot 2. These are repetitions across
matched physics seeds, not 26 independent mutation samples. Postcondition
validator counts are Slots 10, Variables 8 and Guards 4. Rejected material
and its kernel receipt are retained, rather than replacing it with the parent.
Ten otherwise valid ES evaluations belong to pairs whose other arm failed;
they are archived but supply no update. Fourteen complete pairs supply 56
paired evaluations in total.

Descriptively, the five fully valid baseline genotypes have a conditional
mean 0.5563243701 and population SD 0.4559960921. Their four physics-seed
scores and final ordinary-world hashes agree within each genotype. Conditional
within-genotype objective SD is zero in this small fixture. These selected-valid
statistics are exploratory, **not** the registered baseline statistics or a
substitute P3/P4 pass. They do not establish general seed robustness.

## What was implemented

- `hg-sim-core/src/ere/coherent.rs`: shared atomic kernel port, preserving
  the accepted helper's transaction rules, staged rejection receipts and
  no-reencoding/no-clock-advance behavior. The historic assay helper and
  accepted archive are unchanged.
- `hg-sim-core/src/ere/signal.rs`: 183-logit complete-family grammar,
  metadata-isolated fixed ordinary world, real Engine evaluation and native
  rewrite-ID capture. All candidates start with identical ordinary material.
  Proposal reuse is credited only when its priority-normalized canonical
  class differs from Core; frozen metadata validity is measured after each
  step. This is not paid repair, learned maintenance or endogenous novelty.
- `hg-train/src/ere/signal.rs`: deterministic 32 baseline / 8 control /
  80 ES attempt schedule, common emission and physics seeds, strict paired
  withholding, persisted distributions/perturbations/updates, resource limits,
  immutable-prerequisite checks and premeasurement source/binary snapshots.
  Training still owns no Engine implementation.
- `hg-train/src/optim.rs`: exact fitness ties receive average ranks. This
  intentionally changes tied ES numerics for future runs; old evidence remains
  bound to its historical sources/binaries. Distinct-score ranking is unchanged.
- `scripts/audit_ere_complete_optimizer_signal.py`: independent archive
  accounting audit of scores, event credit, caps, schedules, pair withholding,
  float32-aware rank updates, TV, denominators and decisions, with 13 real
  temporary-copy tamper tests and an output completion seal.

The CLI is deliberately separate from legacy training. Feature
`ere-complete-signal` forwards the opt-in feasibility matcher. No default
matcher or runtime/admission budget was changed. This diagnostic uses the
closure's 400k/50k envelope, not the 50k/50k production runtime envelope.

## Verification and evidence scope

All 88 training-crate unit tests, 11 existing ERE integration tests, two
complete-profile shared-loop/admission tests and four new backend tests pass.
The kernel-port integration test compares receipts, programs and staged store
bytes for all 168 parents x two roles x 13 requests: 4,368 comparisons pass.
Tests cover deterministic emission, fixed-world noninterference, rejection as
null, priority-clone exclusion, forced controls, capped/null rollouts, expired
wall limits, flat-fitness stationarity and paired withholding. The existing
complete-profile full campaign still rejects with zero optimizer/precheck/
held-out steps.

The independent accounting audit passes, including 13 tamper rejections.
It is **not** a second native-physics replay or a G-R1 semantic auditor.
The archive freezes 430 source/contract/auditor files plus the executed native
binary before measurement. The producer checks 60 unchanged native/semantic
source files against the accepted closure receipt. All 364 entries in the
original closure completion seal still verify after this work.

Peak diagnostic matching usage is 18,386 work / 2,388 results. Completed
archive size is 22,472,215 bytes (about 21.4 MiB), 442 files, including the
completion seal with 441 entries. No resource ceiling bound the result.
Completion-seal SHA256:
`4a71c1c44fe85f1291cfe9290c7c5f725f66b37a7ed138b5280af7c92c245018`.

## Entry points and persisted outputs

Design: [prospective contract](/Users/kalewicker/Documents/hypersim/docs/ere_complete_optimizer_signal_v1.md).
Results: [summary](/Users/kalewicker/Documents/hypersim/output/ere_complete_optimizer_signal_20260929/summary.json),
[manifest](/Users/kalewicker/Documents/hypersim/output/ere_complete_optimizer_signal_20260929/manifest.json),
[accounting audit](/Users/kalewicker/Documents/hypersim/output/ere_complete_optimizer_signal_20260929/audit.json),
[all attempt receipts](/Users/kalewicker/Documents/hypersim/output/ere_complete_optimizer_signal_20260929/rollouts.jsonl).
The five `generation_*.json` files include every epsilon, paired score,
theta-before/after vector and common-seed distribution panel.

For a separately specified future panel, use a fresh output directory:

```sh
cargo run --release -p hg-train --features ere-complete-signal --bin ere-train -- signal-complete NEW_OUTPUT_DIR output/ere_complete_coherent_closure_full_20260929
python3 scripts/audit_ere_complete_optimizer_signal.py NEW_OUTPUT_DIR --tamper-tests --seal
```

The completed archive is immutable evidence; do not reuse its directory or
reinterpret its rejected baseline rows as a passing precheck.
