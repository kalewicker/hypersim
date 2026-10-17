# Resource-token observation/action adapter findings — 2026-10-08

## Outcome and experiment state

The adapter is implemented and engineering-verified against every saved
defense-allocation state. It resolves the old encoder's missing resource/cue
inputs and step0-only API without modifying that encoder. No native advances
or optimizer updates were run. This is not learned allocation, an original ERE
gate pass, endogenous cognition, sustained survival or independent physics
replay; all corresponding admission flags remain false.

| Package | Lifecycle | Meaning |
| --- | --- | --- |
| `output/ere_resource_adapter_20261008` | Failed, sealed | Rust groups and receipt fixture pass; original Python verifier fails on its malformed graph-ID regex. No completed audit. |
| `output/ere_resource_adapter_accounting_v2_20261008` | Complete, sealed | Separately corrected parser accounting, unchanged Rust binary/config/input/criteria; full independent audit and tests pass. |

The failed package remains 681 entries, root
`a34244da71bc81d656e1979b55ac0fa75ee0ec6cd4689ab0d1135028aec38c38`.
The corrected package contains 688 entries and 652 source pins, root
`37d364eae4cdee1d3c6f6481b247d7df93762ccd16358e65cb53563bb903b157`.
Its preseal size is18,335,997 bytes and workflow time2.624 seconds. The identical
compiled test binary in both packages has SHA256
`18d163078679c3e56f2918ac21620a2355588bd64193888a4a1023fb5956716d`.
The correction is recorded in `accounting_correction.json`, with an exact
verifier diff and failed-package preservation before/after.

## What was implemented

`crates/hg-train/src/ere/resource_adapter.rs`, exported under the default-off
`ere` module, contains:

- `observe(inventory)`: read unique isolated wallet, forecast and bounty nodes
  by native labels, accepting step13 and the frozen typed value domain. Reject
  missing/duplicate tokens/properties, malformed references, linked tokens,
  nonzero pending/credits, wrong types, wrong timestamps and out-of-domain
  values. This is a task projection, not general graph validation.
- `input(mode, observation)`: canonical `[1, wallet, forecast, bounty/12]`;
  independently mask forecast, bounty or both. IDs, node order, ordinary
  topology, scenario/action names and wrapper convenience values are not inputs.
- `initialize`, `probabilities`, `sample`, `score_gradient`,
  `expected_utility`: four-parameter logistic defend/harvest head, neutral
  initialization, explicitly supplied sampling draw, analytic score and
  expected-utility derivatives. Reject nonfinite/invalid inputs, bad draws,
  invalid actions and saturated logits. No optimizer or fitted endpoint exists.
- `request(observation, action)`: map0/1 to the sim-core `Defend`/`Harvest` enum;
  refuse either request with wallet0. It does not execute, reserve resources,
  advance time or revalidate a live native world. No direct engine import.

The old110-parameter ordinary-graph encoder, physics, utility schedule and
previous auditors remain unchanged. The314-pin corrected allocation package
has313 source bodies byte-identical; the sole exception is the exact additive
`pub mod resource_adapter;` train export. All four allocation archives and the
failed first adapter package verify unchanged before/after.

## Verification and fixed-denominator results

All16 original receipts are imported with their native seal map. Pairing yields
eight exactly equal pre-choice observations across the two authored actions.
The previous sealed corrected audit supplies target identity and utility
provenance; the new independent Python verifier recounts post-resolution
ordinary firings from saved events and adds saved native harvested credits.
Utilities are never passed into the observation/head as features.

| Test boundary | Result |
| --- | --- |
| Rust observation/order/reference invariance and masks | Pass |
| Rust16 malformed/out-of-domain observation cases | All refused |
| Rust arithmetic, gradients, draw boundaries and action admission | Pass |
| Fixed16-receipt binary fixture | Pass;32 state/mode records |
| Independent audit | Pass;64 action-score records,32 utility records |
| Independent central differences |384 checks pass; maximum error6.987970202487759e-10, tolerance2e-8 |
| Python auditor regression groups |6/6 pass, including mutated records, custody, denominator and input failures |
| Feature-off `hg-train` check | Pass |
| Frozen self-contained archive recheck | Pass; reruns3 ordinary Rust groups plus explicit receipt fixture, all6 Python groups and exact binary output reproduction |
| Existing encoder arithmetic/invariance regression | Pass separately, without training or native advancement |

Full inputs distinguish8/8 states. Forecast-masked and bounty-masked inputs
each have4 unique states; both-cues-masked inputs have2 (wallet still visible
for admission). Unpaid actions are refused but their probabilities, tied
utilities and zero utility gradients remain in the fixed denominator. This
refusal does not imply a paid policy chose an optimal action.

Neutral initial action probabilities are exactly0.5/0.5 in every mode/state;
funded expected utility is9.5. The fixed nonzero parameter probe is arithmetic
only, not an endpoint, learned controller or selected policy. Previous task
headroom remains12.5 for the authored optimum,11 for always-harvest and8 for
always-defend; no claim of improvement over these baselines is made here.

## Evidence and reproduction

Authority: the corrected archive's `manifest.json`, `audit.json`,
`adapter_records.json`, `rust_tests.json`, `receipt_fixture.json`,
`python_tests.json`, `feature_off.json`, `preservation_before.json`,
`preservation_after.json`, `accounting_correction.json` and
`failed_preservation_after.json`. `inputs/` includes all16 exact native receipts,
the prior utility audit/completion manifest and all four original seal maps.
`source_snapshots/` includes the implementation, contract, verifier/tests,
wrapper and Rust source/manifests used for the build.

Recheck, without editing either archive:

```sh
python3 -B scripts/recheck_ere_resource_adapter.py output/ere_resource_adapter_accounting_v2_20261008
```

The helper validates membership/hashes and dispatches into the archive's frozen
script. It reruns the pinned binary in a temporary directory using copied
archived inputs, compares exact generated records, runs frozen Python tests and
audits the original archive. It requires no original allocation directory or
live-source input fallback. Standalone copied archive usage:

```sh
python3 -B /absolute/copied/archive/source_snapshots/scripts/recheck_ere_resource_adapter.py /absolute/copied/archive
```

The known root above is the trusted reference for initial seal authentication;
self-consistency alone is not authentication of an arbitrary replacement
archive. This recheck reproduces inference/arithmetic, not physics or a fresh
build in a new toolchain.

## Next evidence-producing boundary

Separately preregister a small, fixed-seed full-versus-cue-masked learning
comparison, with constant-action baselines, identical resource/native budgets,
unpaid controls, all seeds retained and fixed acceptance criteria. Freeze
sampler/credit/update rules, initial parameters, development/confirmation
boundaries and maximum work before launch. First connect the adapter to a
sim-core-owned observe-at13 / decide-before14 / native-allocation receipt
bridge, with timing/resource revalidation and an independent harness audit.

Saved response-table arithmetic is suitable for preflight but must not be
reported as fresh on-policy native learning. A future native runner must issue
the sampled request and pay for its actual outcome, not select an action using
counterfactual utility. Do not fit the old aliased schema or silently substitute
this authored voucher objective for the original research goals. The current
adapter passes engineering; training is still unlaunched and unadmitted.
