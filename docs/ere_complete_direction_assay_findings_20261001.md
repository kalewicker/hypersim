# Fixed-parameter update-direction assay: full scientific findings

Date: 2026-10-01. Profile `E2-complete-direction-reliability-v1`;
objective `mean-constraint-utility-v1`.

The authorized assay completed all **7,168 evaluations**, **32 direction
estimates** and eight validation blocks. Its prospectively frozen accounting
audit passes all **40 detached tamper controls**. There were five measured caps
during estimation, no validation caps, no crashes, no Incomplete/NotRun cells
and no unknown partial work. The full schedule stayed within the registered
716,800-attempt, one-hour and 2-GiB limits.

**Reproducible, locally useful updates are not supported.** Mean alignment is
positive, 0.19185659709456104, but six positive/two negative blocks give exact
one-sided sign p=0.14453125, failing p<.05. Mean forward-minus-base utility is
**−0.008831761437623792**, with zero wins, five losses, three ties and registered
improvement sign p=1.0. Context mean utility effects are zero and negative,
not positive in both contexts. Completion and feasibility pass; the required
alignment/usefulness conjunction fails. `support_direction:false`.

This is a negative result under the fixed design, not proof of universal
estimator failure or statistically confirmed harm. No opposite-direction test,
best-block choice, retuning, retry or additional campaign is introduced.

## Authorization and unchanged prospective design

The user explicitly authorized launch with “ok let's launch it”; see the
[launch record](ere_complete_direction_assay_authorization_20261001.md).
The [prospective contract](ere_complete_direction_v1.md),
[machine-readable layouts](../configs/ere_complete_direction_v1.json),
harness, tests and checkers remained unchanged from the passing
[development freeze](ere_complete_direction_implementation_20261001.md).
All 459 live inputs and the executable matched before launch and after completion.
The development fixture is not pooled into this assay.

Two fixed parameter contexts, both length 183: zero logits (context 0), and
alternating +.25/−.25 by zero-based index (context 1). Neither is an old champion
or chosen from held-out outcomes. Eight independent estimator blocks evaluate
both contexts. For each block/context, estimates A/B share eight antithetic
standard-normal perturbations but use disjoint emission/physics panels.
Each estimate starts afresh from the fixed context and evaluates sixteen
candidates × four emissions, sigma 1 and alpha f32 .05. No sequential training
occurs (`optimizer_generations:0`).

Scientific master `0xE658000001` = 989318873089. Perturbation RNG seed
master+1,000,000+10*block+context, restarted for A/B. Estimation emission seed
master+10,000+1,000*block+100*context+40*estimate+4*pair+batch_draw; estimation
physics seed master+100,000+100*block+10*context+4*estimate+batch_draw%4.
Positive/negative candidates share each batch's seeds; A/B panels are disjoint.

All 32 estimates and sixteen five-arm vector sets were saved before validation.
Five fixed validation arms: base, forward A, reverse A, forward B, reverse B.
Forward is the original f32 ES post-update vector; reverse is the exact f32
2*base−forward operation, without clipping, normalization or alpha tuning.
Validate 16 fresh emission draws × four physics seeds per arm/block/context.
Emission seed master+20,000+1,000*block+100*context+draw; physics seed
master+200,000+100*block+10*context+physics_index. All five arms share each
validation draw's seeds. No validation-based selection is performed.

The native 100-step fixed world, typed grammar/coherent kernel, metadata
isolation, MaxParallel/one rewrite, opt-in repeated-self-pair matcher, pooled
50k/50k budgets, node/edge ceilings 2,000/4,000 and native reuse reward are
unchanged. Batch utility includes all four entries: 1+native f32 reuse for valid
estimation outcomes and zero only for measured caps. Dense ordinal scores feed
the unchanged rank-ES core; caps retain native null fitness. Validation uses
native f64 reuse in utility, not the optimizer's f32 cast.

## Registered unit, criteria and all eight block outcomes

Primary unit **paired estimator block**, fixed denominator eight—not sixteen
contexts, 32 estimates, hundreds of draws or 5,120 validation cells. Average the
two context alignment scores within each block. Alignment uses actual saved
f32 forward vectors converted to f64 before subtraction from base; an exactly
zero direction would receive alignment zero and an explicit degeneracy flag,
without dropping its allocation. All 32 observed directions were nonzero.

Within each context/arm average four physics utilities per draw, then all
16 draws. The block utility effect equally averages the two contexts' effects
((forward A+forward B)/2−base). Both estimates remain in the average. Means retain
all eight blocks; ties are excluded only from the corresponding binomial test.
No missing outcomes occur, so official means and p-values are available.

| Block (zero-based) | Mean alignment | Utility effect, zero context | Utility effect, alternating context | Block utility effect |
| --- | ---: | ---: | ---: | ---: |
| 0 | −0.02734491367045608 | 0 | 0 | 0 |
| 1 | +0.35834658161005817 | 0 | −0.026269394333993557 | −0.013134697166996778 |
| 2 | −0.5187031312766459 | 0 | 0 | 0 |
| 3 | +0.30580925931728514 | 0 | 0 | 0 |
| 4 | +0.4957098034032362 | 0 | −0.03125 | −0.015625 |
| 5 | +0.18132238897924852 | 0 | −0.026269394333993557 | −0.013134697166996778 |
| 6 | +0.4876976267723439 | 0 | −0.03125 | −0.015625 |
| 7 | +0.25201516162141846 | 0 | −0.026269394333993557 | −0.013134697166996778 |

Alignment: mean 0.19185659709456104; six positive, two negative, zero ties;
exact one-sided sign p=37/256=0.14453125. Context alignment means are
0.23093414710783997 and 0.15277904708128215, descriptive only. Actual direction
norms range 0.04760154978134476..0.14583368953523088; no degenerate cosine case.
Positive average alignment alone does not pass the registered reproducibility
criterion or show high accuracy across independently sampled perturbation bases.

Usefulness: mean −0.008831761437623792; zero wins, five losses, three ties;
exact one-sided improvement sign p=1.0. Context mean effects are
0 and −0.017663522875247584. Do not reinterpret the five negative blocks using
an unregistered opposite-direction test as a confirmed-harm claim.
These sign tests assess paired directions, not confidence intervals on mean
effects. The two-property acceptance rule is a conjunction, not permission to
promote whichever test looks more favorable.

| Registered criterion | Result |
| --- | --- |
| Complete fixed schedule, known outcomes | Pass: 7,168/7,168, all eight blocks |
| Positive alignment mean and sign p<.05 | Fail: positive mean, p=.14453125 |
| Positive usefulness mean and sign p<.05 | Fail: negative mean, p=1.0 |
| Positive usefulness in both fixed contexts | Fail: zero / negative |
| Aggregate forward valid count ≥ twice baseline count | Pass: 2,048 ≥ 2×1,024 |
| Overall support for reproducible, useful directions | **False** |

The feasibility point-count guard is not statistical noninferiority. Results
are conditional on the two fixed contexts, emission panels, 100-step world and
shared perturbation-basis design, not broad generalization or learned maintenance.

## Validation arm means and native availability

All five arms are valid on 1,024/1,024 cells. Native full-panel means are
therefore available; there is no complete-case substitution or cap imputation.

| Arm | Mean constraint utility | Mean native reuse | Valid / denominator |
| --- | ---: | ---: | ---: |
| Base | 1.1760581586951395 | 0.17605815869513955 | 1,024 / 1,024 |
| Forward A | 1.1649619844033905 | 0.16496198440339035 | 1,024 / 1,024 |
| Reverse A | 1.169029698414901 | 0.16902969841490098 | 1,024 / 1,024 |
| Forward B | 1.169490810111641 | 0.16949081011164116 | 1,024 / 1,024 |
| Reverse B | 1.1764367134334521 | 0.17643671343345227 | 1,024 / 1,024 |

Registered descriptive forward-minus-reverse utility:
−0.005506808666660867. Reverse B's slightly higher point estimate than base is
not a new confirmatory result or grounds for selecting/reversing the optimizer.
No p-value or best-arm promotion is introduced for reversal controls.

## Actual work, caps and resources

| Stage | Recorded / scheduled | Valid | Resource-cap null | Native attempts | Advances |
| --- | ---: | ---: | ---: | ---: | ---: |
| Estimation | 2,048 / 2,048 | 2,043 | 5 | 204,572 | 204,567 |
| Validation, five arms | 5,120 / 5,120 | 5,120 | 0 | 512,000 | 512,000 |
| Total | 7,168 / 7,168 | 7,163 | 5 | 716,572 | 716,567 |

Every validation arm uses 102,400 attempts/advances. All 32 started estimates
finish, including the batches containing caps; unused early-cap work is not
reassigned. The five terminal caps are:

| Estimation cell ID | Attempts / advances |
| --- | ---: |
| `estimate-r0-c0-e0-p4-s1-b2` | 2 / 1 |
| `estimate-r2-c1-e1-p3-s-1-b0` | 73 / 72 |
| `estimate-r3-c0-e0-p1-s-1-b2` | 73 / 72 |
| `estimate-r3-c1-e1-p4-s1-b2` | 73 / 72 |
| `estimate-r6-c0-e0-p6-s1-b1` | 51 / 50 |

Each is a measured `resource_cap`, native score null, utility zero, terminal
matcher truncation, zero terminal advance and empty rule-event list. The
five attempt-minus-advance difference is exactly accounted for; there is no
stale-event credit or unknown work. This exercises, but does not establish
broad effectiveness of, constraint-aware estimation.

Campaign elapsed: 556.7348685 seconds, including one 163.164121291-second
grammar compilation, setup and final checks; separate final audit time excluded.
Maximum native evaluation: .267595667 seconds. Peak matching work/results:
50,000 / 6,150; peak nodes/edges: 293 / 616. Final sealed archive:
**932,474,740 bytes**. All within registered resource ceilings. The cell deadline
is cooperative, not an OS-preemptive timeout guarantee.

## Post-hoc visibility diagnostic: sparse response, not a new criterion

Read-only inspection of the completed validation receipts compares each arm
with base at the same block/context/draw. The four physics utilities are exactly
equal within every recorded draw/arm; this is an observed property of this
panel, not a justification for changing future replication or denominators.
Counts below use physics-averaged draw outcomes, not independent-cell claims.

| Context | Compared arm | Signature changes / 128 draws | Utility changes | Utility wins / losses |
| --- | --- | ---: | ---: | ---: |
| Zero | Forward A | 1 | 0 | 0 / 0 |
| Zero | Forward B | 6 | 0 | 0 / 0 |
| Alternating | Forward A | 7 | 3 | 0 / 3 |
| Alternating | Forward B | 4 | 2 | 0 / 2 |
| Zero | Reverse A | 2 | 0 | 0 / 0 |
| Zero | Reverse B | 2 | 1 | 1 / 0 |
| Alternating | Reverse A | 9 | 2 | 0 / 2 |
| Alternating | Reverse B | 4 | 1 | 0 / 1 |

Only **18/512 forward draw comparisons** change emission signature (3.515625%);
only **5/512** change utility (.9765625%), all decreases in the alternating
context. Zero-context forward updates change some emitted genotypes but never
their recorded reward. The five forward losses coincide with an active base
family becoming inactive. That co-occurrence does not isolate activation as the
sole cause: parent/operator choices can also change.

For example, `validate-r4-c1-d8-forward_a-p0` changes native reuse 1→0 with
active→inactive. Conversely the descriptive reverse control
`validate-r4-c0-d0-reverse_b-p0` changes reuse 0→1 with inactive→active. These
are specific saved cases, not selected champions or a reverse-direction success.

This supports investigating **sampler-to-reward sensitivity** rather than
assuming non-tied candidate rankings translate into visible beneficial small
updates. It does not prove the ES sign is wrong, identify an optimal alpha or
justify relaxing gates. Longer training or another unchanged confirmation would
not by itself resolve the observed sparse validation response.

Recommended next step: a read-only mapping audit using the frozen grammar,
parameter group layout and these receipts to identify which logits control
reward-relevant family activation/operator choices, which changes are invisible
at the current horizon, and why the recorded forward changes lost reward.
Then preregister a targeted sensitivity/estimator experiment if needed. Do not
automatically pick the best arm, tune alpha on this panel or promote a new
surrogate objective. No such follow-on work or native campaign was launched.

## Frozen accounting audit and immutable provenance

Archive: [`output/ere_complete_direction_assay_20261001`](../output/ere_complete_direction_assay_20261001).
`manifest.json` is durable status; `summary.json` is the registered result;
`audit.json` reports `audit_pass:true`, `scientific_campaign_complete:true`,
`classification:fixed_parameter_direction_assay`, `support_direction:false`.
No checker, protocol, configuration or source changed during measurement;
no postmeasurement checker correction or scientific rerun occurred.

The auditor reconstructs all 32 batches/updates, shared perturbation bases,
sixteen exact reversal/vector sets, candidate and native receipt accounting,
alignment scores and eight-block utility statistics. All forty detached tamper
controls reject. All 459 source/config/contract/checker/test snapshots and the
executed binary verify. Final root seal, **7,686 exact sealed entries**, independently
reverified after the frozen final audit:

```text
ba88c696014fa42ca612f4c20baadb1818cc6c5757fd41594c87a1e4bc1a185a
```

Snapshot SHA256:
`f616072632b9cd549e8c8dca9d36b89fcd4fb2369b5250ddd05c183b89bc4bf3`.
Executed binary SHA256:
`9657c3513f8c08d8a683e3790141157632cf61bc062837f65095ef98581538fb`.
The same binary/input freeze as development is intentional; scientific layout,
schedule and seeds are distinct. All corresponding live inputs and executable
still match after measurement.

Development/684, prior minibatch/10,152 and original training parent/1,690 sealed
entries reverify unchanged after completion. Earlier feasibility/6,944,
confirmation/6,598, failed short training/1,662 and failed scaling/247 and 289
entries were verified unchanged during this launch. No older archive was edited
or resealed, and no historical result was pooled into the current tests.

The checker uses pinned native accounting helpers and arithmetic tolerances:
parameter updates <2e-7; means/alignment 1e-12; stricter absolute p-value tolerance;
reversal vectors exact after f32 conversion. It does not independently replay
native physics/RNG, regenerate grammar/perturbation RNG from seeds or prove
compiler-to-binary correspondence. `independent_physics_replay:false`.
An accounting pass is not scientific success, new semantic admission, learned
repair/maintenance or longer-horizon readiness. Reserved longer-horizon training
remains unrun (`full_training_authorized:false`).

## Review commands

Completed invocation—do not reuse the occupied output for another measurement:

```sh
target/release/ere-train direction-assay-complete \
  output/ere_complete_short_training_continuation_20260930 \
  output/ere_complete_direction_assay_20261001
```

Read-only frozen re-audit (omit `--seal` to preserve the completed archive):

```sh
python3 -B output/ere_complete_direction_assay_20261001/source_snapshots/scripts/audit_ere_complete_direction.py \
  output/ere_complete_direction_assay_20261001 --tamper-tests
```

Code/extension points and development verification are in the
[implementation report](ere_complete_direction_implementation_20261001.md).
Any changed estimator, objective, context panel, update scale or horizon needs
its own prospective identity and fresh evidence, not revision of this trial.
