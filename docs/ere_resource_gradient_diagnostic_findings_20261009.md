# Resource-allocation gradient diagnostic — 2026-10-09

## Outcome and scope

The saved-receipt expected-gradient and feature-margin diagnostic passes.
All eight models, 800 recorded updates and 6,400 sampled actions agree with
independent reconstruction. The native utilities and graph-token inputs also
reconstruct from the 16 authenticated historical action receipts. There is no
new simulation, controller fitting or campaign optimizer update.

The current head has enough representational capacity, and the frozen campaign
has no detected credit/update arithmetic error. Its learned low-bounty hazard
margin is nevertheless wrong in every Full endpoint. Expected utility ascent
in the shared coordinates can reduce defense in that state while improving the
other states. This makes a feature-coordinate/preconditioning change better
motivated than simply repeating or lengthening the same campaign. It does not
prove that noise is irrelevant or that a proposed replacement will train well.

The original trial remains complete, accounting-passed and local-learning-
negative. All original scientific/training/sustained/independent-replay flags
remain false. The authored capacity witness below is not a learned result.

## Package state and provenance

`output/ere_resource_gradient_diagnostic_20261009` is complete and sealed:
838 entries, nine source pins, 826 parent-import mappings, 18,133,971 bytes.
Workflow elapsed: 8.733 seconds. Root:
`36d512dbd096d6d7efb6cd1bff33c1e67c9b51200e6a7f601d6c2f15de406ec2`.

The known 8,968-entry parent seal was verified before and after the diagnostic:
`ee9f26d77bbfa63b9cc66ce0562cff33d636b99ebd4ec54290405f1e1eba69b8`.
Its 673 pinned live bodies and every sealed parent artifact remain unchanged.
The new package copies its parent seal/manifest/audit, fixed schedule/draw hashes,
proposal, final endpoints, all 800 updates, 16 native reference receipts and
the historical allocation audit. Three parent-frozen verification helpers and
six new diagnostic sources form the nine-file source closure.

The parent full native audit remains the authority for all 7,440 trajectories.
This diagnostic independently checks the copied subset and update chains, not
a second audit of the complete native trajectory archive or physics replay.
Every copied file is authenticated by its path/hash in the known parent seal.
Stored weights are evaluated directly; independently reconstructed update
arithmetic uses the existing 2e-8 tolerance rather than demanding bit identity
between Rust and Python floating-point accumulation.

An initial disposable scratch rehearsal caught that exact-float comparison
issue before a final package was created. The implementation was corrected to
anchor evaluations to recorded weights; no scientific criterion, tolerance,
capacity vector, parent source or experiment outcome changed. No failed
immutable diagnostic archive or native retry was created.

Evidence authorities:

- `manifest.json`: complete diagnostic, source/import custody, zero native/
  optimizer work, no fitting or launch authorization, unchanged parent.
- `diagnostic.json`: all reconstructed state utilities, 11 fixed parameter
  probes, every recorded pre-update expected gradient/variance, sampled residual,
  objective change and per-model aggregate. No checkpoint is selected.
- `audit.json`: independent enumeration, margins, model-wise quantities and
  finite-difference checks.
- `verification.json` and `tests.json`: exact frozen commands and receipts.
- `inputs/`: self-contained authenticated evidence subset, including all updates.
- `source_snapshots/` and `output_sha256.json`: frozen reproducibility and seal.

Nine test groups pass: complete audit; closed-form versus independent action
enumeration/finite differences; baseline cancellation; zero unpaid actor credit;
domains/aliasing; corrupted gradients/variance/margins; counts/admission; saved
action/credit corruption; parent-root corruption. All 44 funded-utility derivative
checks pass, with maximum absolute error 1.1722e-10 versus the frozen 2e-8 limit.

A complete relocated copy reproduces producer arithmetic, independent audit
and all tests from its own frozen sources and copied inputs, with no live-source
or parent fallback. Its known-root seal passes before and after. This adds zero
native advances and zero campaign optimizer updates.

## Capacity is present; the learned margins are not

The fixed four-parameter Full input is
`[1, wallet, forecast, bounty/12]`. On funded actor samples, wallet is always
one: bias and wallet columns are identical. The funded Full matrix has rank
three; the masked matrix has rank one because all four funded inputs alias.
Rank three is still sufficient for the required action partition.

The single preregistered authored vector `[-1,-1,4,-4]`, within [-8,8], gives:

| Funded state | Correct action | Logit | Signed correct-action margin |
| --- | --- | ---: | ---: |
| Hazard, low bounty | Defend | +1.333333 | +1.333333 |
| Hazard, high bounty | Harvest | -2 | +2 |
| Safe, low bounty | Harvest | -2.666667 | +2.666667 |
| Safe, high bounty | Harvest | -6 | +6 |

Its exact stochastic-policy funded mean is **12.027981833**, above 11.5.
This establishes bounded capacity on this exposed authored task. It was not
searched, fitted, installed into a native campaign or used to initialize training.
It cannot satisfy the original trial's learned-controller criterion.

For any masked policy, a common funded defense probability `p` gives exact
mean `11 - 3*p`, since constant defend averages 8 and harvest 11. Thus masked
cannot exceed 11 in exact expected utility. None of this implies generalization
to new native worlds or continuous bounty values.

## Every frozen endpoint

Exact means below integrate both actions with each saved endpoint's probabilities
and authenticated native utilities. They are diagnostics, not replacements for
the original 16-midpoint confirmation means or its acceptance criterion.

| Learner seed | Exact Full mean | Exact masked mean | Full low-hazard defense probability | Full low-hazard signed margin |
| --- | ---: | ---: | ---: | ---: |
| 993748721101 | 10.945100719 | 10.736570101 | 0.210843249 | -1.319850011 |
| 993748721102 | 10.974156899 | 10.737280426 | 0.172811814 | -1.565829001 |
| 993748721103 | 10.951400172 | 10.761893443 | 0.162823459 | -1.637368427 |
| 993748721104 | 10.972539679 | 10.735402943 | 0.236720263 | -1.170745476 |
| Pooled, fixed four-seed denominator | 10.960799367 | 10.742786728 | — | — |

Exact paired advantage is +0.218012639, but every Full endpoint still falls
below constant harvest 11 and the threshold 11.5. Confirmation quantization
therefore does not explain away the negative result. The original pooled grid
scores remain 10.9609375 / 10.8125 and their gap remains +0.1484375.

## Gradients, variance and coordinate interference

For a funded state with feature vector `x`, defense probability `p` and
authenticated utilities `Ud, Uh`, its contribution to the original expected
batch gradient is:

```text
p * (1-p) * (Ud-Uh) * x / (20*8)
```

Unpaid actor contributions are exactly zero. The exact funded-mean utility
gradient is 40 times this batch gradient: four funded states, an eight-state
batch denominator and reward scaling by 20. The fixed baseline 9.5 cancels in
expectation but affects conditional variance. A state-dependent baseline alone
would not change the expected direction in the same coordinates.

Across all 400 Full updates, 399 sampled gradients align positively with their
conditional expected gradients, and only one recorded update decreases exact
expected funded utility. Masked has 380/400 positive alignments and 20 decreases.
No gradient norm or parameter clipping occurs in any of the 800 updates.
Clipping is not the observed bottleneck, and the recorded estimator is usually
ascending the current expected objective.

Full endpoint expected-gradient norms are 0.00614–0.00801. Their conditional
signal-to-noise ratios are 0.515–0.581 under independent Bernoulli state draws;
sample residual RMS over all 100 steps is 0.0161–0.0196. These are model-based
policy-distribution quantities, not confidence intervals over the deterministic
native world, a significance test or proof that noise alone caused failure.

More concretely, let `z_low = theta dot [1,1,1,1/6]`. Its expected raw-gradient
direction is the dot product of that input with the verified batch gradient.
The following quantities are derived from the saved, independently verified
endpoint gradients; no gradient was applied:

| Full learner seed | Expected low-hazard logit direction |
| --- | ---: |
| 993748721101 | -0.001214605 |
| 993748721102 | +0.000508465 |
| 993748721103 | -0.000610784 |
| 993748721104 | -0.000234033 |

At three endpoints, overall expected utility ascent still *reduces* defense in
the one state where defense is best. At neutral initialization it does so for
all Full learners: gradient
`[-0.01875,-0.01875,+0.003125,-0.0239583333]` gives low-hazard logit direction
`-0.0383680556`. The two redundant bias/wallet coordinates and shared bounty
coordinate couple the four state decisions. Positive forecast learning does
not guarantee positive movement of the low-hazard decision margin.

Uniformly increasing learning rate scales this local direction without changing
its sign. Longer fitting may eventually reach better coordinates, but this
diagnostic does not establish that it will, or justify spending another native
campaign on that assumption. There is no missing input information, arithmetic
repair or larger model vocabulary currently required by the capacity check.

## Extension map and reproduction

The existing Rust runner, sim-core bridge, adapter, dependencies and frozen
learning proposal are untouched. New diagnostic-only files are:

- `configs/ere_resource_gradient_diagnostic_v1.json` and
  `docs/ere_resource_gradient_diagnostic_v1.md`: fixed parent identity,
  dimensions, tolerances, authored witness and zero-work limits.
- `scripts/ere_resource_gradient_diagnostic.py`: authenticated bounded copy,
  closed-form expectation/variance, every recorded update comparison, scratch
  verification, fresh output and seal. There is no native or fitting branch.
- `scripts/verify_ere_resource_gradient_diagnostic.py`: independent two-action
  enumeration, raw native-credit reconstruction, score/update arithmetic,
  feature rank, margins, finite differences and custody.
- `scripts/test_ere_resource_gradient_diagnostic.py`: nine regression/corruption
  groups, including retained denominators and zero-credit unpaid controls.
- `scripts/recheck_ere_resource_gradient_diagnostic.py`: frozen producer/auditor
  reproduction and tests on a self-contained copied package.

```sh
python3 -B scripts/recheck_ere_resource_gradient_diagnostic.py output/ere_resource_gradient_diagnostic_20261009 --expected-root 36d512dbd096d6d7efb6cd1bff33c1e67c9b51200e6a7f601d6c2f15de406ec2
```

## Recommended next boundary — proposed, not implemented

Build a versioned **fixed cue-contrast actor adapter and zero-native preflight**
before asking to launch another comparison. Retain four actor parameters, neutral
initialization, wallet-based admission/credit masking, the native timing and
payments, all unpaid slots, and the paired Full/masked controls. Derive features
only from the same graph-token forecast and bounty—not scenario names, graph
IDs, action labels, reward tables or an authored best-action lookup.

A prospective basis for the current binary cue domain is:

```text
f = 2*forecast - 1
b = 2*(bounty - 2)/10 - 1
Full actor input = [1, f, b, f*b]
CuesMasked actor input = [1, 0, 0, 0]
wallet remains in observation/admission and the fixed zero-unpaid-credit mask
```

For the four funded cue combinations, distinct Full feature vectors are
orthogonal and each has squared norm four. Thus the raw expected logit direction
for state `s` would be `p_s*(1-p_s)*(Ud_s-Uh_s)/40`, without cross-state terms.
This algebra motivates the proposal: it removes the measured coordinate
interference while preserving graph-cue inference and the four-parameter budget.
It is not a tested adapter, a fitted policy or a promise of gate passage.

The preflight should independently verify feature domains, masking, source
identity, score gradients and this per-state sign property, and explicitly
document the change from rank-three to rank-four Full features. Both arms retain
the same bias coordinate; masked cues are zeroed after transformation so masking
does not encode a false low-bounty/safe observation. Reject cue-domain changes
instead of silently assuming orthogonality outside the exposed binary task.

Only after that boundary passes should a new version preregister its own matched
trial, unchanged utility/control criteria and explicit native budget. The old
negative trial is never extended or reclassified. This remains scoped authored
resource allocation, not original ERE scientific admission or endogenous cognition.
