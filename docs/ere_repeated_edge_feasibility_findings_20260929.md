# Repeated-self-pair feasibility v1: findings

Date: 2026-09-29. Engineering kernel revision; not scientific acceptance.

## Outcome

Implemented a charged binary-self-pair feasibility prepass, available through
an explicit matcher API and the opt-in Cargo feature `repeated-edge-feasibility`.
**Default builds retain the legacy matcher.** The saved draw-9730 snapshot now
completes its pooled matching check in **9,018 work units**, under the unchanged
400,000-work / 50,000-result ceiling, without reordering any pattern edge.

| Saved snapshot, store step 91 | Work | Results | Truncated |
| --- | ---: | ---: | --- |
| Core, unchanged original search | 8,462 | 372 | false |
| Legacy Proposal control | 391,538 | 0 | true |
| New Proposal absence proof | 556 | 0 | false |
| New pooled total | 9,018 | 372 | false |

This is matcher-only evidence from a persisted terminal snapshot, not a
continuation of the child or a 100-step durability result. The old scientific
archive remains failed/unmeasured: all 340 file hashes verify unchanged; no
full audit, new completion seal, scientific retry or training occurred.

[Engineering design/work model](/Users/kalewicker/Documents/hypersim/docs/ere_repeated_edge_feasibility_v1.md).
[Machine-readable findings and source hashes](/Users/kalewicker/Documents/hypersim/docs/ere_repeated_edge_feasibility_findings_20260929.json).

## Implementation and extension points

[Matcher implementation](/Users/kalewicker/Documents/hypersim/crates/hg-engine/src/matcher.rs:104)
now exposes three counted entrypoints:

- `find_bindings_counted`: normal public/Engine dispatch. The optional feature
  selects the new work model; without it, dispatch remains legacy.
- `find_bindings_counted_legacy`: explicit original DFS/counter/prefix control,
  regardless of the build feature.
- `find_bindings_counted_feasible`: explicit new necessary-condition prepass
  followed by the original DFS when the rule remains possible.

The exported `MATCHER_WORK_MODEL` identifies normal dispatch as either
`legacy-dfs-v1` or `repeated-self-pair-feasibility-v1`. Explicit control API
calls may deliberately differ from that build-selected model. The feature is
forwarded by `hg-sim-core`; neither crate enables it by default.

Only binary `[v,v]` pattern edges participate, and only when a work limit is
present. Scan global arena edge IDs deterministically. An eligible edge must
have arity two, satisfy the optional edge label, have equal endpoint handles,
and pass the variable's node-label and metadata-ceiling binding constraints.
The check uses the same `bind_ok` logic with initially unassigned variables.
It does not use a label/index-presence shortcut or assume distinct variables
must bind distinct nodes. Metadata and wrong-label/arity inspections are charged.

If no eligible self-pair exists, no full binding can exist: return a complete
empty pool after charging the negative decision. If a probe succeeds, it proves
only possibility. The unchanged DFS still enforces cross-edge assignments,
edge injectivity, ordered endpoints, free variables and all guards. A single
self-pair cannot satisfy two injective pattern edges by being reused.

There is no compiled-rule/RCF/schema change, edge reordering, reassignment of
matched-edge positions, early property/degree/quantity guard evaluation,
cross-rule/step cache, or mutation of the snapshot/table/RNG. The existing
Engine step/selection/apply implementation still matches its frozen source.
Prior snapshot control tests explicitly call the legacy API so they continue
to check the original cap rather than accidentally measuring the new revision.
The original fixture and its historical source-provenance fields are unchanged.

The prepass must stay a necessary condition if extended to other arities.
In particular, non-injective variables make naive endpoint-multiplicity or
distinct-node counting unsafe. Higher-arity repetitions such as `[v,v,v]`
are intentionally not optimized in this version.

## Work model and finite-cap semantics

Charge one work unit per probe, inspected graph edge, binary endpoint equality
check, binding-eligibility check, and terminal negative proof. Use the existing
work/result guard for each charge. The prepass shares the same allowance with
DFS and emits no matches. If a charge is unavailable, mark truncated and do
not certify a complete negative. Zero result allowance also fails closed.

Core has no binary self-pair requirement, so its work and exact ordered binding
vector remain unchanged. Proposal has none of the required self-pairs in the
saved world, so the prepass proves its empty pool in 556 charged units. This
differs from the earlier 357-unit diagnostic that reordered the pattern and
used another accounting path; this implementation never reorders the pattern.

The pooled Engine debits actual work in native rule order, including prepass
charges. If any rule truncates, no already-completed Core pool, scheduled
binding or partial Proposal pool is applied, and no selection RNG is consumed.
The per-step ceilings are not raised, and runtime ceilings are not changed.

This is a **new work model**. For a possible self-pair, the prepass adds overhead
and may cause a cap where legacy DFS would have completed. When both runs
complete, exact ordered bindings and guard counters agree in the tested
fixtures. Finite-budget work/prefix/completion are not claimed equal; positive
prefix tests instead compare legacy DFS with the prepass charge subtracted
from its allowance. Do not extrapolate the one snapshot's improvement to
campaign runtime, overall gate success or all repeated-variable patterns.

## Verification

| Final check | Result |
| --- | --- |
| Default hg-engine unit/integration suite | 59 passed |
| Opt-in hg-engine unit/integration suite | 61 passed |
| Opt-in, no parallel: library + adjacency integration | 26 passed |
| Release saved-snapshot diagnostics, opt-in forwarded feature | 5 passed |
| Existing streaming timing regressions | 8 passed |

Counts overlap across builds; they are not a unique-test total. There are
13 new matcher unit tests, two feature-specific pooled Engine tests, and one
new saved-snapshot candidate check alongside the four historical controls.

The matcher differential includes 32 deterministic graphs × six patterns,
with labels, ordered edges, repeated incidence, metadata nodes, variable
distinctness and property guards. Compare the complete **unsorted** binding
vectors, including full node/edge generations and original matched-edge slots,
not just sets or canonical keys. Dedicated tests cover all four guard families
and declaration-order short circuit, positive and negative probes, edge
injectivity, non-injectivity, exact proof cost, zero/work/result caps and
legacy fallback when no work budget or binary self-pair is present.

The Engine tests verify a prepass cap after a completed Core pool applies
nothing and leaves store/step/RNG unchanged. A bounded legacy selection/apply
control compares eight small-fixture steps in each of MaxParallel,
SequentialRandom and SequentialWeighted: exact pools, rewrites, typed node/
edge data and incidence/label index order, subsequent node/edge allocation
handles and RNG probes agree. These are development tests, not scientific
trajectories or a second independent Engine implementation.

The complete no-parallel integration suite does **not** compile: a pre-existing
unguarded `rayon::ThreadPoolBuilder` reference at
[phase5_engine.rs:258](/Users/kalewicker/Documents/hypersim/crates/hg-engine/tests/phase5_engine.rs:258)
requires an optional dependency disabled by that build. That unrelated file
was not changed. Serial library and adjacency checks pass explicitly; do not
claim the entire serial suite is green. A transient new test type mismatch
(`usize` rewrite count versus `u64`) was corrected before the final passing runs.

[Execution record](/Users/kalewicker/Documents/hypersim/docs/ere_repeated_edge_feasibility_execution_20260929.txt).

Reproduction, engineering-only:

```sh
cargo test --locked -p hg-engine -- --test-threads=1
cargo test --locked -p hg-engine --features repeated-edge-feasibility -- --test-threads=1
cargo test --locked -p hg-engine --no-default-features --features repeated-edge-feasibility --lib --test matcher_adjacency -- --test-threads=1
cargo test --release --locked -p hg-sim-core --features repeated-edge-feasibility --test ere_9730_matcher_diagnostic -- --nocapture --test-threads=1
```

## Scientific boundary and next step

The saved child remains Unmeasured. Neither the new snapshot proof nor the
small development trajectories replace its missing 100-step continuation.
Full-auditor memory/shared runtime and post-timing tamper stages remain
unverified at scientific scale. Historical preflight source pins and the
old pending-launch/archive-absence condition are not repaired by this feature.
Source hashes in the JSON findings describe a dirty live engineering revision,
not a newly accepted or committed scientific release.

Next is a fresh bounded resource-pilot specification with an explicit active
feature/work model, refreshed live-source/binary provenance and independent
auditor compatibility. Compare unchanged compiled identity/world semantics
separately from intentionally changed work/cap counters; do not silently waive
the old contract's byte-exact trace-prefix requirement. A revised acceptance/
parity contract must be prospective, and pilot execution needs explicit scope
approval. Check positive-probe overhead and cap incidence across operators,
not only draw 9730, before considering a separately authorized full trial.

Gate/training remain false. Default matching remains legacy. The failed
scientific archive is preserved, not resealed or retrospectively admitted.
