# CC-1: atomic constructive chemistry engineering fixture

**Completed — 2026-10-05, America/Phoenix.** The unchanged [frozen contract](../experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json) passes all 41 cases and 82 worker variants. This establishes bounded scripted chemistry and failure atomicity under the declared laws. It supplies no kinetic, reproduction, training, evolutionary or sustained-open-endedness evidence.

## Implemented behavior

The default-off `hg-engine/checked-transitions` feature provides [owned checked graph staging](../crates/hg-engine/src/checked.rs). Patches use generational handles, typed expected property values, local creation references, explicit edge removals, exact incidence/absence observations and node/edge limits. Undeclared cleanup, missing references, repeated writes and property-type changes reject without touching the supplied graph. Existing rewrite effects and `Engine::step` are unchanged.

The default-off `hg-sim-core/constructive-chemistry` feature provides [private chemistry authority](../crates/hg-sim-core/src/constructive_chemistry/mod.rs), a [frozen fixture harness](../crates/hg-sim-core/src/constructive_chemistry/fixture.rs), and the `constructive-chemistry-cc1` binary. Admission validates typed schema, generational relations, secondary indexes, placement, valence, occupancy and finite ledgers. The public interface exposes a read-only graph, observations and handles; prepared commits are opaque.

`FormBond` consumes one activated F carrier's stored energy, makes one local A–B bond, and adds 2 heat quanta. It preserves every physical bead and species. The fixed accounting is `world_U=4*active_F+2*AB_bonds`, `reservoir_U=4*active_reservoir_F`, and `world_U+reservoir_U+heat=initial_total`. Material totals include activated and inactive reservoir fuel. Fields use checked signed 64-bit arithmetic; derived totals use checked signed 128-bit arithmetic and decimal-string receipts. Boundary fuel transfer is restricted to site 0 and pays or credits the finite appropriate reservoir.

Preparation clones graph, allocator continuation, ledgers, semantic frontier and accepted journal. Commit checks complete exact before-state bytes and SHA-256 plus a distinct admission-authority token, then publishes one owned state replacement. The token also prevents cross-world commit when two independent admissions have identical display identities. Every accepted event records its proposal, semantic allocation, tagged graph/property changes, ledger deltas and before-state hash. Rejections do not advance revision, allocator, frontier, ledger or accepted journal.

## Fixed-denominator findings

| Check | Outcome |
|---|---|
|41 declared cases ×1/4-worker settings | All 82 variants retained and independently verified |
| Shared fuel, both serial orders | Exactly one bond/fuel expenditure; second prepared commit rejects `StaleState`, then reprepare rejects `InactiveFuel` |
| Disjoint actions, both orders | Old preparation rejects after the first commit; reprepare accepts. Reversed orders have identical declared physical projections |
| Same serial order under 1/4 workers | Entire native graph, allocator, ledger, accepted journal, diagnostics and evidence are exact matches |
| Active/inactive boundary roundtrip | Two paid commits; original world material and reservoir totals restored, energy balanced, heat unchanged |
| Locality, distinctness, species, valence, capacity, finite reservoir and overflow | Every declared rejection matches the independent reference |
| Fuel deletion and slot reuse | Same arena index with a new generation; old preparation rejects stale, fresh use of old handle rejects missing |
| Cross-world commit | Rejects, including identical display identities; both worlds stay unchanged |
| Undeclared cleanup and wrong property variant | Generic stage rejects `UnexpectedCleanup` / `PropertyType` |
| Five injected staging faults | Live exact graph/allocator/ledger/frontier/journal unchanged |
| Five invariant mutants | Actual unpublished mutants reject at the declared material, energy or ledger predicate |
| Seven malformed initial inputs | All 14 variants retained; independent admission validation rejects the specific declared defects |

Across both worker settings there are 26 accepted commits,86 rejection decisions (including 14 malformed admissions),20 successful preparation-only records, and 372 checked published observations. These are scripted engineering observations, not independent scientific worlds. The missing-reservoir-debit mutant stages a funded out/in transfer on an unpublished copy and restores exactly the debit; it differs from the separate unpaid-bead mutant. No diagnostic mutant is published into its live case world.

## Independent verification and regressions

The [Python auditor](../scripts/audit_constructive_chemistry_cc1.py) implements its own dictionary-based material model and case scripts, with no Rust calls or arena dependency. It checks proposal order, decisions, normalized graph/ledger states, conservation, native wire/index consistency, allocator inventories, exact rejection identity, typed event deltas, trajectory continuity, accepted journals and complete final states. It also checks that each malformed case contains its declared defect. [Nine auditor tests](../scripts/test_constructive_chemistry_cc1_audit.py) cover valid evidence and corrupted denominator, offers, decisions, fuel, allocator, journal, malformed-input replacement and scientific flags.

All planned Rust checks pass:35 `hg-core`,59 default-engine,62 checked-engine,4 chemistry,4 Evolving Minds descendant and 3 ERE lifecycle tests, totaling 167 test executions. Default/feature engine runs repeat existing tests intentionally. The long descendant continuation test takes about 84 seconds; shared build/test commands total about 155 seconds against the frozen 1200-second ceiling. The final native fixture takes 0.730 seconds and independent audit 0.152 seconds, each against 300 seconds. Native evidence is 1,722,340bytes; the sealed archive has 15,450,920 payload bytes, well below 128MiB. No declared cap binds.

Reproduce in a new directory:

```sh
python3 scripts/verify_constructive_chemistry_cc1.py output/constructive_chemistry_cc1_NEW_RUN
```

The verifier freezes source/configuration, law binding and executable before native execution; retains failures; checks historical payloads **and their separately recorded seals**; and creates a final inventory seal. The frozen contract's prospective status is historical and remains unchanged. Runtime completion is recorded separately.

## Retained failures and evidence authority

The [development package](../output/constructive_chemistry_cc1_development_20261005) retains the initial dispatch failure, failed harness source, and two passing development native/audit receipts. The missed disjoint operation spelling was corrected; contract cases, criteria and denominator were unchanged.

The [first full package](../output/constructive_chemistry_cc1_20261005) has lifecycle state `Failed` because its wrapper compared a payload-only historical inventory with a directory list including the seal. Every historical payload hash matched; no historical evidence was altered. Its runtime audit and all tests passed, but this wrapper failure is retained and is not the completion authority. Seal SHA-256: `1181c31635f3f60ce9acf2065fcbe054edbf053a4c0a108fd60dc8bed3e1ad13`.

The [verified package](../output/constructive_chemistry_cc1_verified_20261005) is the completion authority: `manifest.json` says `Completed`; `completion.json` and `audit.json` establish the engineering gate. Its reuse receipt proves the only changed frozen source was the verifier. It reuses successful tests for unchanged Rust/config/contract/auditor bytes, reruns the frozen native fixture and independent/adversarial audits, and verifies 53 historical archives with 14,383 files including their seals. Its 82 native variant records are identical to the first package. Seal SHA-256: `823dfdf1796eb4b23ac2746c4a60081e16642c92a9500ebff44f4ca2adf11a9c`.

After sealing, an independent workspace edit added `graph_encoder` to `hg-train/src/ere.rs`. That crate is outside the checked chemistry/core/engine/profile dependency paths; the sealed evidence and every checked runtime source remain unchanged. The preservation receipt describes the workspace at verification completion. This later unrelated edit was preserved.

Contract/config SHA-256 remains `6faec71e73a4c3c5fb7bca384113143a270ee8f866ad7f2044fd2169b996e8e3`. Binary, law and frozen source-set bindings are in `provenance.json`. Lifecycle `final_step=26` counts accepted fixture commits across variants; it does not denote physical time. All scientific, training, evolution, sustained-claim, fairness, power and launch flags are explicitly false. Held-out Evolving Minds worlds remain unexposed; chemistry scientific seeds remain unassigned.

## Next boundary

**CC-2 starts with a prospective contract.** Freeze the physical reaction process and rate/multiplicity conventions, complete channel enumeration, tiny analytic cases, causal read provenance, new checkpoint identity, exact allocator/RNG/time continuation and interrupted-write recovery. The previously unenumerated legal cross-pairings must be included before any reaction-channel or scheduling claim. Keep the CC-1 laws and scripted receipts versioned separately from new kinetic laws. Retain every capped, failed and malformed route. Copying, membranes and evolutionary campaigns remain later gates.
