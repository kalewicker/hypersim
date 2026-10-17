# Full-scale coherent follow-up: engineering findings

Date: 2026-09-29. Scope: timing-memory repair and bounded matcher diagnosis,
not scientific retry or full-audit revalidation.

## Outcome

The live auditor's full-payload retention bug is fixed. Its corrected timing
stage passes on the actual 10,000-row failed-trial material in **23.836 seconds**
at **46,301,184 bytes (44.16 MiB) peak RSS**, under the unchanged 120-second
engineering bound and 1 GiB RSS ceiling.

The separate matching cap is reproduced exactly without an Engine step.
The Proposal requires a self-pair `[3,3]`, but none exists in the terminal
snapshot. A diagnostic selective-first edge order establishes zero matches
in 357 counted work instead of truncating at 391,538. That ordering is **not
installed** in the kernel. It changes work accounting and finite-budget
behavior, so it is not a scientifically neutral wall-time optimization.

The scientific trial remains failed/unmeasured. All 340 failed-archive files
still match the external preservation inventory. No full audit, completion
seal, trajectory replay, scientific rerun or training was performed.

[Engineering scope](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_full_scale_engineering_v1.md).
[Machine-readable findings](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_full_scale_engineering_findings_20260929.json).

## Implemented timing correction

In [the live checker](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_complete_coherent_draw_assay.py:1505),
replace the list of every classification payload with streaming ordered
comparison. For each non-null classification trace ID, consume exactly the
next native child timing entry. Both sequences must exhaust together.

Every canonical classification row is still parsed, including null-ID tail
rows. Exact order, multiplicity, missing/extra entries, parent coverage,
individual clocks, attempted-step totals and phase sums remain checked.
Time/RSS guards remain enabled. No material objects survive across draws;
the stream is explicitly closed on success/failure. The original frozen
checker and historical contracts were not edited.

The existing function does not itself forbid a duplicate trace ID if the
timing sequence has the same duplication. That remains the upstream
classifier's responsibility: this repair preserves the legacy timing
predicate instead of silently changing scientific validation boundaries.

The [read-only profiler](/Users/kalewicker/Documents/hypersim/scripts/profile_ere_coherent_timing_streaming.py)
checks preserved hashes for the actual classifications, timing files, manifest
and status before processing them. Attempt/child totals are explicit previously
recorded cross-check inputs, not a new independent trajectory replay. It prints
engineering diagnostics and writes nothing inside the failed archive.
[Persisted stage-only profile](/Users/kalewicker/Documents/hypersim/docs/ere_coherent_timing_streaming_profile_20260929.json)
has full-audit/gate/training flags false. Its 44.16 MiB observation does not
bound the complete auditor's peak memory or shared runtime.

## Matcher diagnosis

The [fixture extractor](/Users/kalewicker/Documents/hypersim/scripts/prepare_ere_9730_matcher_fixture.py)
reads the preserved child stream, verifies its hash, and selects the compiled
start table, attempted step 92 and terminal inventory for `child_9730`.
The [engineering fixture](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/fixtures/ere_9730_matcher_snapshot.json)
preserves every live node/edge handle generation, typed property, endpoint
order/multiplicity and canonical inventory index. The Rust harness checks all
these fields after reconstruction, not just cardinalities.

Vacant-slot/free-list history and native index insertion order are not present
in this inventory. They are deliberately not claimed or resumed. The original
target rules take global arena candidate scans: all LHS edges are unlabeled
and the second Proposal edge shares no variable with the first. Thus the
reconstructed canonical incident/label index order cannot change this case's
finite-work reproduction. This fixture is not a general Engine checkpoint.

The matcher, compiled-rule and snapshot sources match the scientific frozen
snapshot. Live core graph/arena sources had already changed in unrelated work;
their frozen/current hashes are retained in the fixture and JSON findings.
They were not reverted. Exact reconstructed data plus the observed counter
equality support this engineering reproduction, not a claim that the live
binary equals the original scientific binary.

At world/store step 91: 159 ordinary nodes and 279 ordinary edges. Metadata
also remains in the snapshot; removing it would change counted enumeration.
The original Proposal pattern is `[[0,1,2],[3,3]]`. The variable 3 is unbound
when edge 2 is reached, so its global pair scan is repeated for each preceding
triple assignment. A pair of distinct endpoints cannot satisfy `[3,3]`.
There are zero eligible repeated-endpoint pairs in the entire snapshot.

| Rule/case | Work | Results | Truncated |
| --- | ---: | ---: | --- |
| Original Core | 8,462 | 372 | false |
| Original Proposal, remaining pooled allowance | 391,538 | 0 | true |
| Original total | 400,000 | 372 | true |
| Diagnostic Proposal, `[3,3]` first | 357 | 0 | false |

The harness invokes `find_bindings_counted` directly with manually debited
pooled rule-order allowances, never constructs/steps an Engine and never
applies a rewrite or samples RNG. It compares the original result exactly to
persisted work/results/truncation, and proves the snapshot remains unchanged.
Across the two successful engineering test executions, 16 matcher calls
were made, each limited to at most 400k work / 50k results: aggregate counted
work is bounded by 6.4 million. No unbounded matcher was run.

Trying the selective edge first is a local diagnostic, not a compiled-table
change. On a small positive fixture, translated matched-edge identities and
complete binding sets agree. That is not proof of equal finite-budget prefixes,
emission order, guard/work counters, edge deletion/allocation order, conflict
selection or RNG/trajectory behavior. The 100-step counterfactual remains
unmeasured, and draw 9730 is not relabeled or removed from any denominator.

## Verification and known historical test failures

- Eight new Python timing regressions pass: legacy/candidate predicate parity,
  read-only ownership, ordered coverage, clocks/counts/sums, full/development
  boundaries, 10,000 large non-retained payloads, stream closure and actual
  canonical parsing of a malformed null-ID tail.
- The existing predicate suite has nine passes and six native-fixture skips;
  no temporary native fixture was supplied in this invocation.
- Four Rust matcher tests pass: exact recorded cap, absent-self-pair/selective
  diagnosis, repeated-variable and distinct-pattern-edge constraints, and
  small positive binding/translated-edge identity parity. The initial fixture
  comparison stopped before matching on scalar energy `0` versus `0.0` JSON
  spelling; the comparison now checks the explicitly-f32 value separately
  while retaining exact tagged property variants. No graph value was changed.
- Existing integration suite: **31 pass, 2 fail**. One historical preflight
  test requires the old arena source hash, which already differs in the live
  checkout. The other requires the proposed full-run directory not to exist;
  that directory now contains the approved failed run. These assertions and
  immutable pins were not weakened or rewritten. This is not an all-green
  current-revision launch preflight.

[Final Rust execution](/Users/kalewicker/Documents/hypersim/docs/ere_9730_matcher_diagnostic_execution_20260929.txt).

Reproduction commands, each engineering-only:

```sh
python3 -B scripts/test_ere_coherent_timing_streaming.py
python3 -B scripts/profile_ere_coherent_timing_streaming.py output/ere_complete_coherent_draw_full_20260929 docs/ere_complete_coherent_draw_full_20260929_preservation.json --attempted 505392 --children 4886
cargo test --release --locked -p hg-sim-core --test ere_9730_matcher_diagnostic -- --nocapture --test-threads=1
```

## Next implementation boundary

The smallest justified next kernel proposal is a budgeted structural
feasibility/selectivity check for repeated-variable edges, before expanding
irrelevant disconnected triples. Preserve public/canonical pattern order and
original matched-edge positions. Charge any scan to a prospectively specified
work model, honor the same pooled 400k/50k ceiling, and prove no required match
is pruned (including metadata visibility, labels, repeated endpoints and edge
injectivity). General edge reordering is a broader alternative, not required
by this diagnosis.

Any optimization that clears this cap necessarily changes recorded work or
finite-budget completion relative to the failed frozen kernel. Treat it as a
new implementation revision requiring fresh provenance and explicit kernel
preflight, not as resealing the old run. Test complete binding/identity parity
and deterministic order, conservative exact-cap behavior, guard accounting,
no partial pool application, and unchanged store/RNG on truncation before
asking for a fresh scientific run. Full-auditor memory/runtime and all tamper
checks still need full-scope validation under a separately approved plan.

Gate/training remain false; this request authorized engineering work only.
