# Fixed cue-contrast adapter — engineering preflight passes

## Outcome

The default-off four-parameter cue-contrast adapter is implemented and its
zero-native preflight passes. Its Full funded inputs are orthogonal: the
expected raw-gradient logit direction has the correct action sign in every
funded state at both fixed arithmetic probes, without cross-state terms.
This addresses the coordinate interference identified by the saved-gradient
diagnostic. It is not a trained controller or a guarantee of finite-run success.

Funding is explicitly separate from actor features. Paid safe-state and paid
CuesMasked cases receive valid actor credit despite a negative/zero second
actor coordinate; unpaid cases receive none even under synthetic unequal
utility. Requests remain bound to validated step-13 inventory and wallet.
Native timing, graph projection, execution, payments and old runner bodies
are unchanged. No native advances, Seal calls or optimizer updates occurred.

The old campaign remains complete, accounting-passed and learning-negative.
All original scientific/training/sustained/independent-replay flags remain false.

## Exact package state

`output/ere_resource_contrast_preflight_20261009` is complete and sealed:
720 entries / 681 source pins. Root:
`8c8357ea23b2a40a5a6d9c2e8dc2f9aab4d44c29acb4e3d44f6785864c57778e`.
Elapsed workflow: 21.235 seconds. Total sealed bytes: 19,856,534, below 128 MiB.
Pinned test executable SHA256:
`008436c04a6dbf82bbbc04b20c9a0b3fd472ae7ffee8da2dfe92ee9d752ad2d0`.

Authenticated predecessors, unchanged:

- Complete negative learning campaign, 8,968 entries:
  `ee9f26d77bbfa63b9cc66ce0562cff33d636b99ebd4ec54290405f1e1eba69b8`.
- Saved-gradient diagnostic, 838 entries:
  `36d512dbd096d6d7efb6cd1bff33c1e67c9b51200e6a7f601d6c2f15de406ec2`.
- All eight earlier allocation/adapter/bridge packages, including their failed
  and correction stages, checked against their existing known roots/counts.

Of the campaign's 673 pinned bodies, 672 are byte-identical. The only existing
source change is `pub mod resource_contrast_adapter;` in the train ERE module
map; removing that exact single export restores its parent source hash. All
nine diagnostic-pinned bodies are unchanged. Eight new sources make the
681-file closure. No Cargo manifest, lockfile, dependency, engine, native kernel,
old adapter, old timed policy, old learning implementation or criterion changed.

The package freezes all copied source bodies, 16 original native action receipts,
the historical allocation audit, both predecessor manifests/audits/seal maps,
the exact compiled test binary, fixture records, build/test/audit receipts and
matching before/after preservation reports. `manifest.json` is lifecycle/work
authority; `audit.json` supplies the independent engineering result;
`contrast_records.json` contains the full arithmetic and pure-request evidence.
Known-root seal verification authenticates package identity, not just internal
self-consistency.

A relocated complete archive copy reproduces the binary's normal tests and
saved-input fixture, exact fixture JSON, independent audit and six Python
corruption groups from its own frozen sources/inputs. It needs neither live
source nor a parent-folder fallback. The known-root seal passes before/after.
This is arithmetic/decision reproduction, not native physics replay or a rebuild.

## Features and admission are different channels

The adapter reuses the old strict graph-only observer. It accepts only canonical
pre-allocation step-13 inventories with isolated wallet/forecast/payoff tokens,
correct typed properties and wallet {0,1}, forecast {0,1}, bounty {2,12}.
IDs, ordering, scenario/action/outcome convenience fields are not actor inputs.
Invalid domains remain refusals even when the corresponding cue is masked.

From validated tokens, `f=2*forecast-1` and `b=2*(bounty-2)/10-1`:

| Mode | Actor input | Unique actor inputs over eight states | Funded rank | Squared actor norm |
| --- | --- | ---: | ---: | ---: |
| Full | `[1,f,b,f*b]` | 4 | 4 | 4 |
| ForecastMasked | `[1,0,b,0]` | 2 | 2 | 2 |
| BountyMasked | `[1,f,0,0]` | 2 | 2 | 2 |
| CuesMasked | `[1,0,0,0]` | 1 | 1 | 1 |

Paid and unpaid versions share the same actor input/probabilities. Wallet
instead gates the returned request and actor contribution. Masking happens
after contrast transformation and also zeroes the interaction: absent cues
are not silently interpreted as safe/low-bounty values.

This is not merely an invertible reparameterization of the old rank-three
funded head. The fixed interaction gives rank four, although parameter count
stays four. The old duplicated bias/wallet actor column is removed; both new
arms have the same unit bias coordinate. Feature norms differ by mask and from
the old basis, as explicitly recorded above. Any comparison must describe this
representation intervention, not claim identical hypothesis classes or geometry.

## Independent checks and exact expectation result

The saved-input fixture uses two fixed arithmetic vectors: neutral zeros and
the existing probe `[0.07,-0.11,0.13,-0.17]`. Neither is fitted, installed into
physics or selected by performance. The 16 source receipts reconstruct actual
post-resolution target ordinary firings and final typed voucher credits.
Their utility pairs are used only for preflight arithmetic, never decision input.

The fixture and independent auditor check all eight states, four masks and
both probes: 64 records, 128 action score gradients, 128 funding-bound actor
contributions and 128 pure step-13 decisions. Two-action utilities, probabilities,
boundary sampling and the draw-based requests agree. All 768 finite-difference
checks pass, with maximum error 1.3076e-9 versus the frozen 2e-8 limit.

The four funded Full feature vectors have Gram matrix `4I`. Independently
enumerating the original funded credit and all eight batch states gives:

```text
G = sum_funded p_s*(1-p_s)*(Ud_s-Uh_s)*x_s / (20*8)
x_s dot G = p_s*(1-p_s)*(Ud_s-Uh_s) / 40
```

Unpaid actor contribution stays zero and the denominator remains eight. There
are eight Full per-state sign checks (four funded states at each probe); all pass.
The other masks correctly lose the corresponding cue separation; the Full
orthogonality guarantee is not asserted for their aliased inputs.

| Funded state | Correct direction for defense logit | Neutral expected direction | Probe expected direction |
| --- | --- | ---: | ---: |
| Hazard, low bounty | Increase | +0.037500 | +0.037500 |
| Hazard, high bounty | Decrease | -0.025000 | -0.024960 |
| Safe, low bounty | Decrease | -0.012500 | -0.012455 |
| Safe, high bounty | Decrease | -0.075000 | -0.070841 |

The old basis's neutral low-hazard expected logit direction was -0.0383680556;
the new basis makes it +0.0375 while preserving neutral action probabilities.
This is an algebraic/derivative property of the exposed four-state cue domain.
It does not guarantee the sign of a sampled update, projected/clipped updates,
finite-budget learning, unseen worlds, other cue domains or original ERE gates.
No expected or sampled preflight gradient was applied to parameters.

## Tests and verification boundaries

- Four normal Rust groups pass: feature/mask/provenance invariance; funding
  independent of actor coordinates; domain/head/draw/action/utility/graph
  refusals; orthogonal Full inputs. Explicit saved-input fixture also passes.
- Six independent Python groups pass: complete audit; feature/mask/wallet;
  scores/credit/requests; gradient signs/domains; counts/admission/utility;
  predecessor-root custody. Corruption tests reject altered paid-masked credit,
  invented unpaid requests/credit, wrong step, wrong sign, scores and utility.
- Old adapter normal tests (three), old timed-policy/bridge normal tests (two)
  and old learning normal tests (two) pass. Native fixtures remain ignored;
  the old expired bridge refusal has zero advances/Seal/callbacks.
- Both `hg-train` and `hg-sim-core` feature-off checks pass. ERE remains default
  off. No new dependency or executable training entry point is added.
- Rust formatting and `git diff --check` pass. Before/after historical seals
  and registered source-body preservation checks agree.

## Extension map and important incompatibility

- `crates/hg-train/src/ere/resource_contrast_adapter.rs`: strict observer reuse,
  separate `funded`, fixed signed `input`, bounded probabilities/sampling,
  score/utility arithmetic, funding-bound `contribution`, pure `choose` request.
  `choose` takes only parameters/mode/inventory/draw; there is no utility-table,
  engine, fitting or parameter-update input.
- `crates/hg-train/tests/ere_resource_contrast_adapter.rs`: normal guards and
  the explicit zero-native saved-input arithmetic/decision fixture.
- `configs/ere_resource_contrast_adapter_v1.json` and
  `docs/ere_resource_contrast_adapter_v1.md`: frozen domains/probes, funding,
  derivative tolerance, orthogonality criterion and admission boundaries.
- `scripts/run_ere_resource_contrast_adapter.py`: zero-native scratch build/
  test/audit, exact source/parent custody, fresh package and seal; no launch branch.
- `scripts/verify_ere_resource_contrast_adapter.py`: independent signed-feature
  arithmetic, raw native utility, request funding, derivatives, batch directions,
  masking/Gram and custody. It uses no Rust head output as the mathematical oracle.
- `scripts/test_verify_ere_resource_contrast_adapter.py` and
  `scripts/recheck_ere_resource_contrast_adapter.py`: corruption tests and frozen
  binary/audit reproduction from a relocated archive.

Do **not** pass contrast features to old `resource_learning::batch_update` or
the old `resource_timed_policy`. Those implementations use a nonnegative head
and infer funding from input[1], which is now a signed/zero forecast coordinate.
The new module is not wired into them. A future versioned runner must carry the
validated observation/funding separately and call this adapter's contribution;
its receipts must preserve the new `feature_profile`. Live old launcher source
guards are not bypassed to accept the changed module map. Old sealed binaries
and their frozen-source rechecks remain intact.

To reproduce this package without physics or fitting:

```sh
python3 -B scripts/recheck_ere_resource_contrast_adapter.py output/ere_resource_contrast_preflight_20261009 --expected-root 8c8357ea23b2a40a5a6d9c2e8dc2f9aab4d44c29acb4e3d44f6785864c57778e
```

## Next boundary — not yet implemented or launched

Build a separately versioned contrast learning runner and freeze its matched
four-seed/two-arm/100-update protocol. Retain neutral initialization, all eight
states/draws, zero unpaid credit, denominator eight, final-only confirmation,
constant-action baselines, known work caps and the original local utility/control
criteria. Check the full schedule, credit bindings and optimizer arithmetic in
a zero-native runner preflight before asking for launch approval. No warm start
from authored/probe weights, teacher action, adaptive curriculum, old negative
trial extension or automatic new compute authorization follows from this pass.
