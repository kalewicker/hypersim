# Budget-matched minibatch experiment: implementation and development verification

Date: 2026-10-01. Profile `E2-complete-budget-matched-minibatch-v1`.

The next experiment's design, feature-gated harness and frozen streaming auditor
are implemented. The bounded development invocation completed all 88 evaluations
and 8,800 native steps; its frozen audit passes all 31 detached tamper controls.
This report preserves the development milestone. The user subsequently approved
the scientific eight-replicate comparison, which completed all 9,472 evaluations
and passed its frozen audit. It does not support batching: utility difference
−0.00715475, four wins/four losses, p=.63671875. See the separate
[full-trial findings](ere_complete_minibatch_trial_findings_20261001.md).
The development receipt remains engineering-only, not scientific efficacy
evidence or a broader training admission.

## What the experiment tests

See the [prospective contract](ere_complete_minibatch_v1.md) and
[machine-readable designs](../configs/ere_complete_minibatch_v1.json).

| Scientific method | Generations | Candidates/generation | Emissions/candidate | Scheduled rollouts/replicate |
| --- | ---: | ---: | ---: | ---: |
| Single-emission | 20 | 16 | 1 | 320 |
| Four-emission batch | 5 | 16 | 4 | 320 |

Eight paired training replicates, zero-logit initialization, shared draw pools
and paired optimizer seeds. Hold out the same 64 emission draws/four physics
seeds for all sixteen final champions; evaluate the untrained baseline once.
Each rollout requests the unchanged native 100-step horizon at pooled 50k/50k.
No previous champion, held-out seed or prior outcome is reused or pooled.

The design asks whether spending compute on richer candidate estimates beats
spending it on more updates. Equal allocation means the batch method receives
fewer updates. It is not a batching-only causal experiment with update count
held constant, and it does not equalize wall time, host work or actual advances
lost to early caps. Unused allocations are not reassigned.

Primary analysis uses eight paired **training-replicate** utility means, not
4,096 correlated champion cells as independent observations. Exact one-sided
paired sign p<.05, positive batch-minus-single utility and a nonworse aggregate
batch feasibility count are required. Seven wins/one loss yields p=9/256;
eight wins yields p=1/256. Report all eight replicate effects, feasibility and
native reuse separately. Shared held-out data limits interpretation to this
fixed panel; there is no broad maintenance or generalization claim.

## Code and extension map

- `crates/hg-train/src/ere/minibatch.rs`: fixed scientific/development layouts,
  coupled schedules, exact lineage checks, batch means/ordinal codes, durable
  generation starts, final champions and replicate-level statistics.
- `crates/hg-train/src/ere/feasibility_training.rs`: shared `Runner` now has
  module-scoped visibility; its evaluation body, native cap handling, in-flight
  marker, atomic rows and counter commits are unchanged.
- `crates/hg-train/src/ere/confirmation.rs`: existing parent/source guard,
  snapshot, live recheck and seal helpers are reused without new body changes.
- `scripts/audit_ere_complete_minibatch.py`: independent schedule, aggregation,
  candidate arithmetic, ordinal ranks, updates, champions, fixed statistics,
  provenance and native-accounting audit.
- `scripts/test_audit_ere_complete_minibatch.py`: seven regression groups.
- `crates/hg-train/src/ere.rs` and `src/bin/ere-train.rs`: complete-feature-only
  module and CLI routes. Legacy feature isolation/refusals remain intact.

Native valid reward remains measured reuse; native cap reward remains null.
Training aggregates fixed batches of utility 1+f64(native f32 score) for valid
outcomes, 0 for measured resource caps. Capped entries stay in the batch
denominator. Dense ordinal codes feed the unchanged rank-shaped ES core.
Batch-size-one reproduces the preceding adapter's ordering/ties/update.
The held-out descriptive utility uses native f64 reuse without that training cast.

Mean batch utility can prefer a partly capped, high-reward batch to an all-valid,
low-reward batch. This is declared behavior, not strict batch feasibility-first
ordering. Unsupported failures are never given an imputed utility. The same
eight-pair sigma-1/alpha-.05 ES core is used in both methods; no reward, grammar,
coherent kernel, Engine, matcher, evaluator, cap or optimizer-core changes occur.

Generation records now save pre-update parameters, noise and exact expected
cell IDs **before** native evaluation. A completed record separately saves means,
ordinal pairs and post-update parameters. An interrupted candidate batch remains
a generation start, not a completed update. This improves returned-failure
accounting; it does not promise per-step crash journaling or OS-preemptive deadlines.

## Development result, not a scientific comparison

Archive: [`output/ere_complete_minibatch_development_20261001`](../output/ere_complete_minibatch_development_20261001).
Development seed `0xE657D00001`, different from the reserved scientific master
`0xE657000001`. The entire development layout was fixed before native work:
one replicate, two single-emission generations versus one two-emission batch
generation, sixteen candidates, two held-out draws/four physics seeds.

| Stage | Completed / scheduled | Native attempts/advances |
| --- | ---: | ---: |
| Single training | 32 / 32 | 3,200 / 3,200 |
| Batch training | 32 / 32 | 3,200 / 3,200 |
| Untrained baseline | 8 / 8 | 800 / 800 |
| Single champion held-out | 8 / 8 | 800 / 800 |
| Batch champion held-out | 8 / 8 | 800 / 800 |
| Total | 88 / 88 | 8,800 / 8,800 |

Zero caps/nulls, Incomplete/NotRun cells, crashes or unknown partial work.
All three generation starts and completed updates verify. Both champions are
fixed before held-out evaluation. This tiny panel ties: both utility means 1.0,
both native means 0.0, one paired tie, p=1.0, feasibility 8/8 per method.
`support_batching:false`, `scientific_campaign_complete:false` and
`classification:bounded_development` are authoritative. Do not infer efficacy,
scientific seed robustness or four-emission/eight-replicate scalability from it.

Actual elapsed campaign time: 212.36056 seconds including one 164.798477125-second
grammar rebuild and setup/final checks; separate final audit time is excluded.
Maximum native evaluation duration .110414 seconds. Peak matching work/results
18,388 / 2,388; peak graph nodes/edges 190 / 497. Final archive 31,674,718 bytes,
within the registered development 900-second / 2-GiB ceilings.

## Verification and provenance

All 131 library tests pass (one pre-existing release-only full-budget cap regression
ignored in debug), eleven ERE and two complete-witness integration tests pass,
seven new auditor regression groups and seven prior feasibility groups pass.
Complete-feature release CLI builds; legacy ERE-only CLI checks. A new refusal
test saves a wrong-parent diagnostic archive and passes the zero-work auditor
with zero compilation, native steps or optimizer updates. Batch-one compatibility,
fixed cap denominators, mean-versus-feasibility ordering, replicate-level sign
statistics, disjoint replicate seed pools and partial-generation records are tested.

The prospectively frozen auditor reconstructs all three actual updates and
rejects 31 detached tamper controls, including altered draw membership/types,
candidate repetition, batch means, ranks, updates, source/digest integrity and
false rollout-level statistical denominators. Parameter arithmetic uses the
pinned parent's comparison tolerance <2e-7; mean/statistic comparisons use
1e-12 relative/absolute tolerances, with stricter p-value absolute tolerance.
No postmeasurement checker correction or development rerun occurred.

Final root seal:
`c626db144036da9a4e305996d289b7997cf38b1a57bbe7edc20f3833bcb88dc3`.
All 557 sealed entries verify with exact coverage; all 454 live frozen inputs
match. Executed binary SHA256:
`ef787f97ac1347d2c0ddc11521bff2c18e0aefcd8b14756ef039d2be3c3f1732`.
The parent short-training seal/1,690 files and prior feasibility trial seal/6,944
files are verified before setup and at completion. Older failed short-training
and scaling archives retain their original seals and 1,662 / 247 / 289 entries;
the prior confirmation retains its 6,598-entry seal. No historical evidence is
overwritten or promoted.

The checker uses exact pinned parent accounting helpers. It checks emitted
catalogue membership and records, but does not independently replay grammar
RNG choices from parameters, native physics/RNG streams or compiler-to-binary
correspondence. The fixed-world/frozen-metadata scope establishes no learned
repair, online metadata evolution, sustained maintenance or long-horizon scaling.

## Historical launch boundary and subsequent completion

Completed development invocation (do not reuse this output directory):

```sh
target/release/ere-train minibatch-development-complete \
  output/ere_complete_short_training_continuation_20260930 \
  output/ere_complete_minibatch_development_20261001
```

Frozen audit:

```sh
python3 -B output/ere_complete_minibatch_development_20261001/source_snapshots/scripts/audit_ere_complete_minibatch.py \
  output/ere_complete_minibatch_development_20261001 --tamper-tests --seal
```

At the development milestone, separate approval was required for **one**
scientific invocation under the registered eight-replicate 947,200-step maximum /
3,600-second / 2-GiB ceilings. The user then approved it in
[`ere_complete_minibatch_trial_authorization_20261001.md`](ere_complete_minibatch_trial_authorization_20261001.md).
The separate scientific layout ran once into the fresh
`output/ere_complete_minibatch_trial_20261001` via `train-minibatch-complete` and
the same prospectively frozen audit. All 9,472 cells and 200 updates completed;
17 training caps remain honestly recorded. The 10,152-entry seal verifies.
The negative registered result and next diagnostic proposal are in the
[full-trial report](ere_complete_minibatch_trial_findings_20261001.md).
No new precheck physics, retry or automatic follow-on campaign was launched.
