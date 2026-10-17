# Coherent harness and frozen auditor: development findings

Date: 2026-09-28. Profile `E2-complete-coherent-assay-v1`.
**Implemented and development-verified, not scientifically accepted.** The
diagnostic exposes two allocation-sensitive Neutral soundness failures. Passing
the evidence audit does not erase them or open training admission.

## Contract and extension routing

Prospective contract: `ere_complete_coherent_assay_v1.md` and `.json`; their
hashes are `a6d0053e49ea657a2279b125427d84b03b421d015ac5a9bdeab54e671ddac26a`
and `771d276a76a81f1391f00da2fb06d33b1088d7dc6ea6de568fe4298095734947`.
They preceded measurement and remain unchanged. The kernel stays pinned at
`43e65713ab9f4de6d8b8d8590a2cfdf3ff4c718153aa0dd5b64b41edeb0cd05f`.
No reader/compiler, kernel, production mutator, paid vocabulary, runtime budget,
admission guard, Cargo manifest/lockfile or old scientific receipt was changed.

| Component | Responsibility / extension point |
| --- | --- |
| `crates/hg-sim-core/tests/ere_complete_coherent_assay.rs` | Isolated producer, fresh-output checks, parent bank, frozen manifests, damage barrier, native trajectories and resource accounting |
| `tests/support/coherent.rs` | Unchanged exact-handle transaction; no RNG, target selection, fitness or repair |
| Producer `legacy_raw_edit` / `raw_edit` / `draw` | Pinned raw material counterfactual, same-choice coherent request, staged receipt and deterministic stream regeneration |
| Producer `typed_world` / `neutral_failure` / `outcome` / `Tally` | Exact typed-world comparison, soundness and prospective strata/threshold accounting |
| `scripts/verify_ere_complete_coherent.py` | Standalone independent edit/decode/control/trajectory/classification replay, frozen integrity and sealing |
| Auditor `CoherentBuilder` / `raw_edit` | Independently declared local primitive transitions and counter updates; never execute Rust or re-encode a mutant |
| Auditor `consume_trace` / `check_classification` / `tally` | Reconstruct native events/typed worlds; verify source tables, resources, soundness and exact denominators |
| `scripts/test_verify_ere_complete_coherent.py` | Native-fixture replay, refusal accounting, topology exclusions and tamper regressions |

This is a separately named adaptation of the old raw scaffold; the originals
remain intact. Frozen canonical/complete/binding helpers are reused. The new
checker carries independent coherent primitives and classification logic.
Scientific and development campaign entrypoints remain ignored by default.

## Evidence lifecycle and honest accounting

Copy every design-time pin, selected source, dependency lockfile, contracts,
exact running binary and pinned basis receipts before measurement. Freeze all
168 parent inventories, the entire ordered request manifest and all 5,376
original damage descriptions. Regenerate each draw exactly from the pinned
stream during execution; never condition target selection on fitness/validity.

Each draw archives the raw selector schedule, raw edits/decode counterfactual,
coherent request, original/staged inventories, status, primitive transcript,
delta and rollback witness. Raw counterfactuals are material evidence, not a
second raw trajectory campaign or reproduction of the old trial's rates.

Run all 32 attacks per parent with matched clean sources. Require native and
frozen controls before any trajectory; root deletion/relabeling must retire,
not receive fabricated quarantine. These controls do not enter mutation rates.

RejectedInput/Policy/Postcondition attempts remain unsuccessful in attempted
denominators, even if a policy refusal's unedited stage still decodes. Retain
the actual invalid stage; never replace it with a valid rollback parent.
Inapplicable label operations remain honest unchanged children with their own
stratum. No resampling, codec regeneration, global recount or hidden repair.

Retain the original operator-defined structural strata and numerical lines.
Add topology-operator and actual pattern-change strata, including active
subsets and prospective coverage minima. Actual pattern comparison ignores
ownership, witnesses, label/guard bodies, properties and edge IDs; bookkeeping
and same-endpoint edge replacement cannot manufacture topology passage.

All parent baselines are mandatory. Trajectories use the actual Engine and
complete source, epoch Never after initial compile, meta visibility false,
MaxParallel/one pairing/seed 42, pooled classification-only 400,000/50,000.
There are no instructions, editors, evaluator, EI, optimizer or repair preamble.
Frozen replay checks source lifecycle/table, bindings, world mutations,
metadata stability, caps, full horizons and exact typed terminal inventories.

## Development tests and preserved failures

Release/debug harness tests: **5 passed / 2 ignored**. The independent temporary
matrix has **96 native requests** (four existing parents, all operators and
both allele roles where applicable) and **9 passing Python checks**, including
valid policy-refusal stages that must remain rejected. The unchanged kernel
passes 19 tests, raw harness 6/2 ignored, training guards 2, and old checker 12.

Two persistent failed diagnostic archives are retained:

- `output/ere_complete_coherent_smoke_20260928`: freeze refused a missing pinned
  kernel-contract source copy; no controls/trajectories started. Copying every
  design-time pin is now enforced and regression-tested.
- `output/ere_complete_coherent_smoke_v2_20260928`: controls passed, but full
  replay rejected hash-only native classification when typed worlds differed.
  Its native summary is not accepted evidence. The producer now implements the
  already-declared typed-world condition; contracts/thresholds/kernel are intact.

An earlier relative-path invocation created no archive. Cargo runs tests in the
package directory, so reproduction must use a fresh absolute output path.
These are bounded development repairs/checks, not scientific retries.

## Clean diagnostic receipt

Archive: `output/ere_complete_coherent_smoke_v3_20260928`.

- All 168 eight-step baselines, 64 retained draws and 55 eight-step children:
  **1,784 attempted/advancing steps**, zero Unmeasured outcomes.
- **5,376 native/frozen damage controls pass**, all 13 relation roles, all 26
  obligation keys and explicit-zero declarations covered.
- Frozen full audit passes; **25 tamper categories reject**: the original 18,
  forged Neutral soundness, and six coherent-specific attacks including missing
  local witnesses, wrong counters, rollback promotion and false topology.
- **207 frozen inputs, 182 unchanged live sources, 218 sealed outputs** verify;
  exact membership includes nested basis seals. Final size: **8,829,813 bytes**.
- Pre-seal assay/shared-auditor times: **92.25 / 68.93 seconds**; whole test
  invocation **94.40 seconds**. Freeze/controls/full/seal auditing is charged to
  the same 120-second development limit. Full-audit peak RSS **238,419,968 bytes**.
  Matching peaks **8,646 work / 896 results**; no cap stopped a trace or audit.
- Statuses: **44 Applied, 11 Inapplicable, 9 RejectedPostcondition**. Outcomes:
  **22 Neutral, 10 NearNeutral, 23 Divergent, 9 Rejected**. All operators appear.
  Same-choice raw counterfactuals decode in 30 cases and fail in 34. These are
  diagnostic material counts, not independent scientific effect estimates.
- Actual pattern-change stratum: 29 attempts, 20 valid, 3 NearNeutral, 9 rejected;
  active subset: 13 attempts, 9 valid, 3 NearNeutral. No short-diagnostic fraction
  establishes scientific gate passage.

A separate **read-only frozen full replay passes**, with no new Engine steps,
archive writes, resealing or bytecode caches. `run_status.json` records complete
development measurement; numerical/gate/training flags stay false. Eight steps
cannot test the 50-step freeze line, and 22 Neutral checks do not supply 50.

### Two allocation-sensitive Neutral soundness failures

The 22 checks include **two failures**, not 22 successes. Draws 19/27 are dormant
Proposal AddVar edits of parents 138/30. Executable identity remains unchanged,
but adding a meta node shifts later ordinary-world allocation. First births:

| Pair | Parent birth | Child birth | World hash |
| --- | --- | --- | --- |
| parent 138 / child 19 | NodeId(96v1) | NodeId(97v1) | Both `53ff4d037cfde7fa` |
| parent 30 / child 27 | NodeId(89v1) | NodeId(90v1) | Both `ce5ae402a8ff48bd` |

RCF precedence still assigns Neutral identity, but typed-world soundness fails
and prevents numerical acceptance. Comparisons retain ordinary node handles,
full typed data, ordered endpoints and multiplicity, ignoring only edge handles.
Native equality uses losslessly compressed exact typed bytes, not another weak
hash. The independent auditor reconstructs the state from native events and
supplements Rust hashes with its own fingerprint. Hash/vector equality alone
no longer yields NearNeutral; typed mismatch follows catastrophe/divergence.

This is an allocation-sensitive identity boundary, not proof of learned activity
or necessarily a changed allocation-independent world topology. The diagnostic
does not establish harmlessness over 100 steps. Next investigate allocator
isolation or a separately justified prospective allocation-independent typed
equivalence, retaining exact negative controls. Do not silently ignore handles,
normalize away failures or change this contract after observing them.

## Replay commands and remaining limits

```sh
cargo test --release --locked -p hg-sim-core --features hg-meta --test ere_complete_coherent_assay -- --test-threads=1
cargo test --locked -p hg-sim-core --features hg-meta --test ere_complete_coherent_assay -- --test-threads=1
python3 -B output/ere_complete_coherent_smoke_v3_20260928/source_snapshots/scripts/verify_ere_complete_coherent.py output/ere_complete_coherent_smoke_v3_20260928 --development --stage full --verify-only --seconds 120
```

A new archive requires a **fresh absolute** `ERE_COMPLETE_COHERENT_OUTPUT` and
only the ignored `complete_coherent_development_smoke` test. Never reuse output.
The separately ignored full test additionally checks exact proposed step/cap
approval values; default tests/current authorization do not provide them.

The independent checker is not a second Engine, allocator or ChaCha
implementation: native hashes/RNG draws and arena ID allocation retain Rust
witnesses, while selection bounds/order, primitive semantic changes, typed
world transitions and classification/denominators are independently checked.
The host atomic transaction is not evidence for ecological paid-edit powers,
maintenance, population navigability or optimizer signal.

Original raw full receipt preservation still verifies 194 frozen inputs, 205
sealed outputs and 177 live sources. No new full trial, optimizer precheck or
training ran. Before any separately approved 10,000-draw/100-step trial, address
the bounded allocation/Neutral-identity finding above. G-R1 and training remain
closed; this development receipt cannot supply a scientific acceptance result.
