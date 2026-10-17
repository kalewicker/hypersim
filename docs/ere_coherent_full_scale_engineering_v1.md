# Full-scale coherent follow-up: engineering scope

Date: 2026-09-29. User request: “ok let's work on what's next”.

This authorizes engineering correction/testing of the timing checker and a
bounded diagnosis of draw 9730. It does not authorize scientific retry,
revalidation/admission, training, revised numerical criteria or larger caps.
The failed full archive and all its frozen inputs stay unchanged. Live source
changes belonging to other work are not reverted. Historical contracts retain
their original hashes and pending/failed point-in-time status.

## Timing correction

Change only the live timing check's classification retention: compare each
non-null trace ID to the next native child timing ID, consuming all canonical
rows and requiring both streams to finish together. Keep every native clock,
attempt count, coverage, sum, wall-limit and parsing assertion. No cross-draw
material cache and no full-row list. Existing frozen checker remains unchanged.

Test parity with the former list-based function on bounded synthetic fixtures,
including missing/extra/reordered/duplicate/null/empty IDs, malformed trailing
records, clock and counter tampering, and detached payload ownership. Then run
only the corrected timing stage on the actual 10,000-row failed archive in a
fresh subprocess, bounded to 120 seconds and 1 GiB RSS. It writes no receipt
inside that archive and does not execute the full auditor. This stage result
cannot stand in for full audit or scientific acceptance.

## Matcher diagnosis

Extract an immutable engineering fixture from child_9730's already persisted
terminal inventory, compiled start table and capped step 92. Verify source
file hashes against the external failure inventory. Preserve handle generations,
endpoint multiplicity/order, typed properties and the canonical recorded
incident/label indexes. Inventory indexes are sorted, not an attestation of
runtime insertion order. This case uses global arena scans for both rules;
do not generalize the fixture to index-ordered matching or Engine continuation.
Reconstruct a read-only matcher snapshot, not an Engine continuation. No
rewrite application, RNG sampling, allocation continuation or scientific
classification is performed. The source snapshot's core/engine/rules matcher
inputs are compared to live files before reproduction. Matcher, compiled-rule
and snapshot implementations must match. The live core graph/arena files have
already drifted from the frozen run; retain/report those hashes and check every
reconstructed live snapshot field. This explicitly chosen live engineering
build is not a scientific reproduction of the frozen binary.

At most 16 matcher calls, each at most 400,000 work / 50,000 results; combined
maximum 6,400,000 counted work. Reproduce pooled rule-order allowance manually
using find_bindings_counted. Compare the recorded work/results/truncation and
binding cardinality. Small fixtures must exercise repeated endpoints and
pattern-edge injectivity. An alternate pattern edge ordering is diagnostic
only: never install it into the engine or alter canonical program identities.
No unbounded matcher, Engine.step, capped-cell trajectory replay or campaign.

The objective is to identify a justified next optimization boundary, not to
claim the counterfactual 100-step outcome. A future matcher optimization needs
its own proof/tests for binding ordering, counters, finite-budget semantics,
identity, guards, conflicts and RNG before fresh scientific authorization.
