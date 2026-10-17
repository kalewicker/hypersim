# Repeated-self-pair feasibility v1: engineering kernel revision

Date: 2026-09-29. User: “alright let's continue on to what's next”.
Engineering implementation/testing only; no scientific retry, full-audit
revalidation, training, changed classification thresholds or archive writes.

## Narrow implementation

Add an opt-in `repeated-edge-feasibility` hg-engine build feature, forwarded by
hg-sim-core. Default builds keep legacy matching. Provide explicit legacy and
candidate counted matcher APIs for differential controls. No rule wire/schema,
canonical executable encoding, variable order or matched-edge position changes.

Only probe binary pattern self-pairs `[v,v]`, and only when a work limit is
present. Other repeated-variable hyperedges are out of scope. Before DFS, scan
graph edges in global arena order to find an eligible self-pair for each such
requirement. Check arity, optional edge label, repeated endpoint, node-label
constraint and metadata label ceiling. Do not evaluate property/quantity/degree
guards early, require distinct nodes for distinct variables, or assume different
pattern edges can consume the same edge. A positive probe is merely necessary,
not sufficient; use the unchanged DFS, original edge positions/order, edge
injectivity and guards. A negative probe proves the entire rule has zero matches.

No cross-rule/step cache. Never mutate snapshot, compiled rule or RNG. Never
reorder pattern edges. Unbounded-work and non-self-pair rules retain legacy
behavior. Candidate selection is explicit opt-in, not training admission.

## Charged work model

Each repeated-edge probe costs one unit. Each inspected graph edge costs one
unit, including wrong-label/arity records. Each binary endpoint equality check
costs one unit. Each eligibility/bind check costs one unit. A negative proof
costs one final unit; do not certify absence when that unit is unavailable.
Every charge uses the existing work/result guard and shares the same rule
allowance with subsequent unchanged DFS. Prepass graph scanning is never free.
Compiled-descriptor loop bookkeeping, like legacy candidate-index bookkeeping,
is not a counted enumeration unit. No instrumentation/schema field is added.

No matches are emitted during the prepass. Exhaustion sets truncated and
returns an empty partial pool; the existing Engine must halt without rewrites
or RNG sampling if any rule truncates. Work/result allowances remain pooled
and debited in native rule order at the existing 400k/50k ceiling. This is a
new work model, not counter equivalence to the failed scientific kernel.

## Bounded engineering verification

Differential tests compare exact ordered binding vectors, node/edge generations,
matched-edge positions and all guard counters to explicit legacy controls on
small labeled/ordered/repeated/incidence/metadata fixtures. Exercise positive,
negative, non-injective and edge-injective cases, result-only/unbounded-work
fallback, zero and exact work/result limits, cap-prefix accounting and absence
proof termination. Deterministic fixture loops are bounded and both APIs receive
explicit finite work limits except intentional tiny legacy-fallback tests.

Use the already persisted draw-9730 snapshot for matcher-only comparison at
400k/50k, never a trajectory replay or counterfactual classification. Engine
unit tests check native pooled debits, exact binding order and halted-step
store/RNG invariance; small engine fixture steps are development tests, not
scientific experiments. Run engine default/feature/no-parallel suites and the
historical matcher-only control tests. No failed archive is resealed.

Fresh scientific use requires a new contract/provenance, active feature/work
model declaration, independent auditor compatibility, full engineering preflight
and explicit launch approval. Same-build checkpoint claims do not extend to
cross-feature checkpoint reuse. The old full trial stays failed/unmeasured.
