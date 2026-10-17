# Bounded resource-allocation learning campaign — complete, local criterion fails

## Outcome

The approved four-learner-seed, two-arm, 100-update comparison is complete.
Its independent frozen postflight audit passes all accounting, custody,
inference, native-event credit, optimizer and fixed-denominator checks.
The preregistered **local learning predicate fails**: Full has a small positive
advantage over CuesMasked, but does not beat constant harvest or reach 11.5.
No seeds were replaced, checkpoints selected, criteria relaxed or extra updates
launched. This is a retained negative learning result, not an execution failure.

Original ERE scientific acceptance, training admission, sustained-success and
independent-physics-replay flags remain false. Actual local fitting did occur;
`training_admitted:false` refers to original programme admission, not absence
of the 800 externally trained contextual-bandit updates recorded here.

## Exact evidence and experiment states

| Package | State | Meaning |
| --- | --- | --- |
| `output/ere_resource_learning_preflight_20261008` | Complete; engineering pass | Zero-native schedule, arithmetic, custody, compression and guard verification; no campaign updates |
| `output/ere_resource_learning_campaign_20261008` | Complete; accounting pass; local learning fail | All fixed training, endpoint-confirmation and baseline trajectories retained and independently audited |

Campaign profile: `E2-resource-on-policy-runner-v1`.
Campaign seal: 8,968 entries / 673 source pins, SHA256
`ee9f26d77bbfa63b9cc66ce0562cff33d636b99ebd4ec54290405f1e1eba69b8`.
Approved preflight root:
`72e7635f031dfa46cc42520cf1089b50fa0e0e774224c0b554c342cd119df319`.

Primary authorities inside the campaign archive:

- `manifest.json`: authorized launch, complete native/postflight lifecycle,
  exact source/binary pins, accounting and local-learning status.
- `campaign/manifest.json`: complete fixed denominator, known native work,
  actual compressed-cell bytes and exact work counters.
- `audit.json`: independent per-replicate scores and all three frozen criteria.
- `campaign/cells/cell-0000.json.gz` through `cell-7439.json.gz`: complete
  native timelines, inventories, policy observations, probabilities, sampled
  actions and actual utility, not just aggregate reward summaries.
- `campaign/updates/update-0000.json` through `update-0799.json`: every
  eight-state batch, contributions, clipping and before/after parameters.
- `campaign/ledger.jsonl`, `campaign/endpoints.json`, `schedule.json` and
  `draw_hashes.json`: ordered execution, final-only endpoints and paired draws.
- `native_receipt.json`, `verification.json`, copied inputs/source snapshots,
  and identical `preservation_before.json` / `preservation_after.json`:
  execution receipts and unchanged historical custody.
- `output_sha256.json`: immutable package seal; authenticate against the root
  above rather than treating self-consistency alone as authenticity.

The archive's own frozen Python auditor ran against its copied inputs and
saved native work, without a live-source or historical-input fallback.
After sealing, the known-root seal was independently checked. The pinned Rust
test binary passed both normal zero-native groups (the explicit saved-input
fixture was intentionally ignored), and its schedule output exactly reproduced
all 7,440 archived slots. Those supplemental checks added zero native advances
and zero optimizer updates. A second full campaign audit or new physics replay
was not performed this turn; the saved full postflight audit is authoritative.

## Work and resource accounting

| Phase | Trajectories | Native advances | Nonadvancing Seal calls | Optimizer updates |
| --- | ---: | ---: | ---: | ---: |
| Training | 6,400 | 204,800 | 6,400 | 800 |
| Final endpoint confirmation | 1,024 | 32,768 | 1,024 | 0 |
| Constant-action baselines | 16 | 512 | 16 | 0 |
| Total | 7,440 | 238,080 | 7,440 | 800 |

Complete launch-and-audit elapsed time: 4,677.305 seconds (77.96 minutes).
Total sealed output: 490,504,883 bytes (467.78 MiB), below the 1 GiB cap.
Compressed native cell receipts: 434,384,000 bytes, versus the preflight's
439,214,820-byte fixture-based estimate. Neither native execution nor the
separately bounded postflight audit exhausted its two-hour deadline. There
were no recorded failure, retry or partial-batch routes in this campaign.

Four seeds are learner-draw replicates on the existing fixed physical world
and physics seed, not four independent worlds. Each arm starts from four zero
parameters, consumes all eight state draws at every update, and uses the
paired SplitMix64 stream frozen before launch. All unpaid cells remain present
and contribute zero actor credit. Each complete eight-state batch is divided
by eight, norm-clipped at one, ascended with learning rate one and projected
componentwise to [-8,8]. Utility comes from actual post-resolution ordinary
firings plus native voucher credits; saved counterfactual response tables are
used only by the independent verifier, not the training runner.

## Final confirmation: every replicate

These are funded means over all four exposed scenarios and all 16 fixed
midpoint draws per state, at update 100 only. Masked means are identical on
this quantized confirmation grid; their exact learned parameters differ.

| Learner seed | Full | CuesMasked | Paired difference | Full beats harvest 11 and paired masked? |
| --- | ---: | ---: | ---: | --- |
| 993748721101 | 10.93750 | 10.81250 | +0.12500 | No |
| 993748721102 | 10.96875 | 10.81250 | +0.15625 | No |
| 993748721103 | 10.96875 | 10.81250 | +0.15625 | No |
| 993748721104 | 10.96875 | 10.81250 | +0.15625 | No |
| Pooled, all four | 10.9609375 | 10.81250 | +0.1484375 | 0/4 |

Always defend confirms at 8; always harvest at 11. Full is 0.0390625 below
constant harvest and 0.5390625 below the frozen 11.5 threshold.

| Preregistered local criterion | Actual | State |
| --- | --- | --- |
| Pooled Full funded confirmation mean >= 11.5 | 10.9609375 | Fail |
| Pooled Full-minus-masked difference > 0 | +0.1484375 | Pass |
| At least 3/4 Full replicates strictly beat both harvest 11 and paired masked | 0/4 | Fail |
| Accounting, custody, controls, complete denominator and resource limits | All pass | Pass |

The conjunction is false. Improvement over the neutral policy's previously
verified 9.5 expectation does not substitute for the frozen endpoint criteria.
Training-batch fluctuations and intermediate checkpoints are not acceptance
evidence. Confirmation is deterministic action coverage on the exposed authored
task, not a held-out task, a statistical confidence interval or independent
physics replay.

## Where the shortfall occurs

The following pools all four endpoints equally within each funded scenario.
It is a descriptive aggregation of the fully audited saved confirmation
utilities, not a new experiment or a changed scoring rule.

| Funded scenario | Full | CuesMasked | Always harvest | Authored best action / utility |
| --- | ---: | ---: | ---: | --- |
| Hazard, low bounty | 3.21875 | 2.37500 | 2 | Defend / 8 |
| Hazard, high bounty | 11.68750 | 11.75000 | 12 | Harvest / 12 |
| Safe, low bounty | 9.68750 | 9.87500 | 10 | Harvest / 10 |
| Safe, high bounty | 19.25000 | 19.25000 | 20 | Harvest / 20 |

Full improves low-bounty hazard allocation relative to masked, but still
harvests there most of the time. Its unnecessary defense in other scenarios
consumes that gain. The state-dependent authored headroom remains 12.5; this
campaign does not demonstrate that training can attain it at the frozen budget.
Unpaid confirmation utilities remain exactly 0 for both hazard states and 8
for both safe states in every model, with no admissible funded request or actor
credit. They were never dropped from batches or draw consumption.

The audited endpoint vectors below use `[bias, wallet, forecast, bounty/12]`.
Equal bias/wallet weights follow their collinearity on funded actor samples;
masked cue weights remain zero. Positive forecast and negative bounty weights
show a cue-sensitive head, but do not by themselves establish adequate control.

| Seed / arm | Bias | Wallet | Forecast | Bounty |
| --- | ---: | ---: | ---: | ---: |
| 993748721101 Full | -0.7393408675 | -0.7393408675 | 0.3523081262 | -1.1608584160 |
| 993748721101 masked | -1.1703366666 | -1.1703366666 | 0 | 0 |
| 993748721102 Full | -0.8465562756 | -0.8465562756 | 0.3346117377 | -1.2439691264 |
| 993748721102 masked | -1.1718164791 | -1.1718164791 | 0 | 0 |
| 993748721103 Full | -0.8539600991 | -0.8539600991 | 0.2656831989 | -1.1707885680 |
| 993748721103 masked | -1.2254767316 | -1.2254767316 | 0 | 0 |
| 993748721104 Full | -0.7113096352 | -0.7113096352 | 0.4551307377 | -1.2195416575 |
| 993748721104 masked | -1.1679129518 | -1.1679129518 | 0 | 0 |

Full precision remains in `campaign/endpoints.json` and all update receipts.
An optimization/feature-margin limitation is a plausible explanation, not yet
a verified cause. This result does not establish that the original plan is
impossible, that the native physics is faulty, or that longer fitting will solve
the shortfall.

## Implementation and extension boundary

No Rust, Python, dependency, physics, adapter, proposal or launch-contract source
was changed during this launch. The exact preflight binaries were reused:

- Campaign binary:
  `297c712d2828b95a1d9d34ac48126153483eeb80bb375c61087976ccf29bf12a`.
- Test binary:
  `f281d296bd501e8e0600d4fc70b052a9eb6ead5c4260e6a10b26b0e3e412d933`.

The boundary remains `hg-train::ere::resource_learning` (schedule and complete
batch updates) -> `resource_timed_policy` (pure graph-observation inference) ->
`hg-sim-core::ere::resource_timed_bridge` (engine, payments and native timing).
The independent `scripts/verify_ere_resource_learning.py` reconstructs policy
and optimizer arithmetic and checks native trajectories against authenticated
historical receipts. Future variants must be separately versioned rather than
editing this sealed experiment or reusing its approval for a new objective.

The preflight retains the feature-off and saved-input arithmetic/regression
evidence; see [runner architecture and preflight findings](ere_resource_learning_preflight_findings_20261008.md).

To reproduce the full saved campaign audit, with no physics or fitting:

```sh
python3 -B scripts/recheck_ere_resource_learning.py output/ere_resource_learning_campaign_20261008 --campaign
```

This recomputes accounting from saved work and checks the seal. It does not
constitute independent physics replay, rebuild verification or a new campaign.

## Recommended next evidence boundary — proposed, not run

Do a small zero-native **saved-receipt expected-gradient and feature-margin
diagnostic** before proposing another training campaign. Use the authenticated
eight-state/two-action native outcomes, frozen endpoints and existing update
receipts to separate representational headroom from credit/optimization speed:

1. Independently reconstruct graph inputs, endpoint probabilities and each
   state's contribution to the exact expected score-function gradient, retaining
   the original eight-state denominator and zero unpaid actor credit.
2. Check whether the existing bounded linear head can separate low-bounty
   hazard defense from the other three funded harvest choices, and quantify
   endpoint decision margins. Any authored reference vector is a capacity
   diagnostic, not learned evidence or a teacher-selected training update.
3. Compare expected gradient direction/magnitude with recorded sampled updates
   and fixed endpoint utility, without selecting checkpoints, fitting a new
   controller, inventing uncertainty from deterministic confirmation or changing
   the current campaign's criteria.

Only after that diagnostic should a separately frozen variant choose a justified
credit/feature/optimizer change and request its own launch budget. Do not extend
this run, tune against its confirmation panel and relabel it held-out, or jump to
structural growth/original scientific admission on the positive masked gap alone.
