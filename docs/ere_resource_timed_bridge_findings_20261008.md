# Live resource-allocation bridge findings — 2026-10-08

## Outcome

The policy-to-native timing bridge is implemented and engineering-verified.
All 32 fresh native cells read their live step-13 inventory before choosing;
only the requested funded action pays at step 14. All four registered refusal
routes pass. Work is exactly 1,063 advances and 35 paid nonadvancing Seal calls,
with zero optimizer updates. No training was launched and no original ERE
scientific, sustained-survival, training-admission or independent-physics-replay
gate changes status.

## State of the new packages

| Package | Lifecycle | Interpretation |
| --- | --- | --- |
| `output/ere_resource_timed_bridge_20261008` | Failed, sealed; zero native work | Rust preflight passes, Python preflight catches its own lazy-generator refusal-ID bug before physics. |
| `output/ere_resource_timed_bridge_preflight_v2_20261008` | Complete, sealed | Separately corrected preflight, identical native/policy binary and criteria; the first native panel, full accounting and tests pass. |
| `E2-resource-on-policy-learning-proposal-v1` | Proposed, unlaunched | Frozen learning design only; runner/auditor/preflight and launch approval remain required. No fitted endpoint or campaign results exist. |

The failed package has 695 entries, root
`560ea51646703e3b3c10bd68e57b0e3637701796a1787767362b315316716e82`.
The correction changes one Python schedule-generator line so it produces the
already intended IDs 32–35 rather than 32/34/36/38. No native retry, changed
denominator, changed policy, criterion relaxation or promotion of the failed
manifest occurred. `preflight_correction.json` records the exact diff and
`failed_preflight_preservation_after.json` verifies preservation.

The completed package has 743 entries and 664 source pins, root
`cc8161fbdb6b5475f5bb5eeef3865da6dcf6ae69e53a7231a84102925d0294ba`.
Preseal size: 94,558,329 bytes (under 128 MiB). Complete workflow: 61.923 seconds;
native Rust fixture: 10.82 seconds. Both packages use the same compiled binary:
`f863f9d739aa86abccf998973a2178eaa2acf30030d4ed00f84e962c95d11eb3`.

## Architecture and extension interface

The original `defense_allocation.rs` and verified resource adapter stay byte
identical. The new default-off modules are:

- `hg-sim-core::ere::resource_timed_bridge`: a versioned kernel fork, owning
  the graph, engine, timing, native payments and physics. `run(scenario,
  resource, deadline, callback)` advances to 13, passes only `&Value` inventory
  to an `FnOnce` callback, validates its `Decision { step, request }`, switches
  allocation flags and finishes at 32. `request=None` is required when unpaid;
  a funded cell must request `Defend` or `Harvest`. Errors include known
  advance/Seal/callback counts. `catalogue(Option<Action>)` and
  `validate_decision` support zero-native preflight.
- `hg-train::ere::resource_timed_policy`: pure `choose(theta, mode, inventory,
  draw)` calls the existing adapter to derive features, compute probabilities,
  sample the supplied draw and return the step-bound optional request plus its
  inference receipt. It receives no engine, writable world, scenario label,
  historical utility table or future trajectory. It imports no engine crate.

| Time | Native/host boundary |
| --- | --- |
| Steps 1 and 5 | Native paid authored birth edits, unchanged |
| Through step 13 | Both allocation alternatives disabled |
| Between 13 and 14 | One policy callback on the live graph; timing, wallet, zero pending/credits/escrow, graph/RNG and deadline revalidation |
| Step 14 | Native matcher/arbitrator executes only the enabled funded allocation |
| Steps 15 and 16 | Native threat request and resolution, unchanged |
| Steps 17–32 | Record ordinary firings and final voucher credits for actual utility |

The switch changes only `enabled` flags of allocation editors in the installed
table and the existing shared GraphRuleSource. Compiler history and epoch 2
continue unchanged; no extra compile boundary is introduced. Graph mutation
and allocation payment remain native rewrite effects, not direct host writes.
All 650 other bodies from the previous 652-pin adapter package remain byte
identical; its two module-map exceptions are exact additive exports. All six
earlier allocation/adapter archive roots are verified before/after, along with
the separately failed bridge preflight root. `kernel_fork.diff` records the
full versioned kernel change against the unchanged original.

## Fixed panel and checks

The panel is four observation modes × four authored scenarios × paid/unpaid:
32 completed cells. Fixed probe theta is `[0.07,-0.11,0.13,-0.17]`; draws by mode
are 0.25/0.75/0.25/0.75. These deliberately exercise both binary branches; they
are not a learned endpoint, policy-quality test or fair mask-performance
comparison. The 16 funded cells make eight defend and eight harvest payments;
the 16 unpaid cells preserve their sampled actions but submit no request.
Total native voucher credits are 56, not energy/fuel.

All eight states have identical biological prefixes and observations across
the four modes through step 13. For each completed cell, the independent audit
checks graph-derived features, probability/draw/action agreement, request
funding and timing, disabled pre-choice editors, only permitted flag changes,
selected flags across subsequent epochs and actual step-14 payments. Utility
is recounted from native ordinary events and final payoff-node Int credits.

Every cell's initial/final inventory, biological states, events, genotype,
installed endogenous candidates, compiler reports and RNG probes agree with
the corresponding sealed historical action receipt. Runtime-table comparison
normalizes only the prospectively specified allocation-enabled flags. It does
not rewrite native events or substitute old outcomes for the new run. Historical
equivalence strengthens accounting but is not independent physics replay.

| Expected refusal | Advances | Seal | Callback calls |
| --- | ---: | ---: | ---: |
| Deadline already expired | 0 | 0 | 0 |
| Callback returns registered error | 13 | 1 | 1 |
| Callback binds step 14 rather than live step 13 | 13 | 1 | 1 |
| Callback requests allocation while unfunded | 13 | 1 | 1 |

All 36 slots remain in the denominator. Completed cells contribute 1,024
advances/32 Seal, expected refusals contribute 39/3, for totals 1,063/35.
There are 35 callbacks overall. Pure guards also reject absent funded requests,
invalid wallet values and invalid observation input.

Verification passes:

- Two zero-native Rust groups plus the explicit bounded native fixture.
- Synthetic accounting and deliberately corrupted timing, pre-choice flags,
  switch bodies, policy inputs/probabilities/draws, outcomes and refusal counts.
- Pre-native Python groups pass, with the real-native group explicitly skipped;
  postflight is six of six groups passing, including the real panel.
- Feature-off checks for both `hg-sim-core` and `hg-train`.
- Frozen archive audit/test recheck, without rerunning the ignored native fixture.

The same recheck also passes on a complete relocated archive copy in `/tmp`,
from a working directory outside the project, with zero new native advances
or updates. This verifies the stated no-live-source/historical-path fallback.

Authority is the completed archive's `manifest.json`, `audit.json`, native
`schedule.json`, `status.json`, all 36 cell receipts, binary/test/build receipts,
preservation reports, source snapshots and seal. The historical imports and
their original seal maps are copied into the archive; no historical-path
fallback is needed by the frozen verifier.

```sh
python3 -B scripts/recheck_ere_resource_timed_bridge.py output/ere_resource_timed_bridge_preflight_v2_20261008
```

For a relocated copy, invoke its own
`source_snapshots/scripts/recheck_ere_resource_timed_bridge.py` with the copied
archive's absolute path. Authenticate its seal against the known root above;
self-consistency alone does not authenticate an arbitrary replacement archive.
The recheck reruns normal zero-native Rust groups and frozen Python auditing,
not physics, training, or a new-toolchain build.

## Next boundary: bounded on-policy learning runner and auditor preflight

The prospective protocol and config are
`docs/ere_resource_learning_proposal_v1.md` and
`configs/ere_resource_learning_proposal_v1.json`, both frozen in this package.
The proposal is four learner seeds × full/cues-masked × 100 batch updates, all
eight states retained, zero initial parameters and sampled actual native
outcome credit. Confirmation uses only the final endpoint, with constant-action
baselines. Maximum work is 238,080 advances, 7,440 Seal calls and 800 updates;
launch is explicitly not authorized by the proposal.

Next implement its runner, independent credit/update/schedule auditor and
zero-native preflight. Freeze RNG implementation/draw hashes, deadline/output
limits and abort accounting before asking to launch. Use compact or streamed
receipts with explicit storage accounting: this 32-cell pretty-JSON bridge
archive is already about 90 MiB, so copying that layout blindly into thousands
of trajectories would be inappropriate. Keep every seed and failed route.

There is no inference of generalization from four exposed authored scenarios or
four learner seeds in one deterministic physical world. A successful local
contextual-bandit comparison would demonstrate externally learned allocation
on this task, not endogenous cognition or the original plan's remaining gates.
