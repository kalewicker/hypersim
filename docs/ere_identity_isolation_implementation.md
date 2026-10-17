# Allocator identity isolation: implementation and verification

Date: 2026-09-28. Engineering status: **implemented and verified for new opt-in
stores**. Scientific status: no new passing G-R1 receipt; training remains closed.

## What failed, and what changed

The coherent development receipt exposed two genuine operational failures of
dormant-genotype neutrality. Proposal AddVar leaves executable bytes unchanged,
but previously consumed the same allocator slots as subsequent ordinary-world
births. Draw 19 / parent 138 changed the first birth from `NodeId(96v1)` to
`NodeId(97v1)`; draw 27 / parent 30 changed it from `NodeId(89v1)` to
`NodeId(90v1)`. Both retained equal canonical world hashes. Hash equality was
therefore insufficient evidence of exact typed-world identity.

`HypergraphStore::new_metadata_isolated()` now separates allocation into ordinary
and metadata domains, for **both nodes and edges**. Ordinary indices are odd;
reserved metadata indices are even. Versions remain generational. Each lane has
an independent next index and recycled-slot stack. Adding or deleting metadata
cannot consume an ordinary slot, change its version, or reorder its free list.

The private implementation is `crates/hg-core/src/arena.rs`; public construction,
insertion routing and deserialization checks are in `crates/hg-core/src/graph.rs`.
This uses safe Rust, without dummy graph nodes, re-encoding, repair, or post-hoc
handle renaming. Endpoint order is never normalized to make a comparison pass.
The coherent transaction implementation, complete decoder, graph-rule compiler,
Engine, matching policy and training guard are unchanged.

## Extension and compatibility contract

Choose the mode before constructing any world or complete family:

```rust
let mut store = hg_core::HypergraphStore::new_metadata_isolated();
assert!(store.metadata_allocation_isolated());
// Construct the ordinary world and encode complete graph-resident families here.
// Existing add/get/remove/snapshot APIs continue to apply.
```

Creation labels at or above `0xFFF00000` select the metadata lane. Mixed-domain
edge endpoints remain supported, including repeated endpoints. Allocation-domain
membership is permanent for that handle: changing its label does not move it.
This is allocation isolation, not a prohibition on semantic interaction. A
metadata edit that deletes an ordinary endpoint's mixed incident edge can still
affect the world; activation or executable changes can also affect behavior.

`new()`, `default()` and `with_step()` retain legacy allocation behavior. Existing
checkpoints are not silently migrated and existing handles are not reassigned.
Consequently the old identity issue deliberately remains reproducible in legacy
mode. Production runners and the measured coherent-v1 harness have not been
switched to the new constructor. New experiments must opt in explicitly.

The arena adapter replaces the internal allocator in default stores too, so this
is a real core-code change, not merely a test-only workaround. Legacy keys,
iteration, recycling and serialized bytes were compared to actual SlotMap over
8,000 randomized operations, including restoration; graph-level legacy payload
interoperability and snapshot-property round trips also pass. This does not claim
benchmark parity or exhaustively validate every downstream production workload.

The outer graph payload keeps its five-field layout. Legacy arena payloads retain
the SlotMap sequence representation. Isolated arenas use slot zero's otherwise
invalid `version=1, value=None` marker; current readers preserve it, and old
SlotMap readers reject it rather than silently reverting to shared allocation.
Mixed node/edge allocation-mode payloads and malformed lanes are rejected.
Do not open isolated checkpoints with older binaries. This is an explicit
opt-in payload extension, not a global HGSS header-version change.

As with existing SlotMap serialization, restoration reconstructs free-list order
from slot positions rather than retaining historical removal order. Arbitrary
multi-vacancy continuation need not equal uninterrupted execution; do not infer
that stronger property from this receipt. Parent/child domain isolation survives
restoration, and the tested checkpoint pairs exactly match uninterrupted pairs.
Generation exhaustion in isolated mode fails closed. Interleaved lane padding
is vacant storage, not graph objects, but can increase slot/index storage toward
twice the larger lane's extent. No large-store performance benchmark was run.

## Bounded, frozen diagnostic

Prospective contract: `docs/ere_identity_isolation_v1.md`.
Native harness: `crates/hg-sim-core/tests/ere_identity_isolation.rs`.
Independent receipt comparator: `scripts/verify_ere_identity_isolation.py`.
Tamper regressions: `scripts/test_verify_ere_identity_isolation.py`.
Fresh sealed archive: `output/ere_identity_isolation_20260928`.

The harness reconstructs the same semantic parents and exact selected Proposal
patterns as the two original failures. It applies the unchanged coherent AddVar
transaction, checks complete decoding and unchanged RCF/execution bytes, and runs
the real complete-enabled GraphRuleSource and Engine—not a static surrogate.
Configuration is seed 42, MaxParallel, pooled 400k/50k matching, 100 steps per
trace, at most 2,000 total Engine steps, 300 seconds and 64 MiB persisted output.

| Parent / original draw | Legacy | Isolated | Isolated checkpoint | Isolated metadata recycling |
| --- | --- | --- | --- | --- |
| 138 / 19 | Failure reproduced at step 1 | Exact identity, 100 steps | Exact identity and continuation | Exact identity after AddVar → RemoveVar → AddVar |
| 30 / 27 | Failure reproduced at step 1 | Exact identity, 100 steps | Exact identity and continuation | Exact identity after AddVar → RemoveVar → AddVar |

Every trace includes 100 real ordinary-node births: these are not vacuous
no-rewrite comparisons. Total measured Engine work is **1,600 advancing steps**.
The producer measurement took 3.332 seconds; the complete test invocation,
including freezing and audit, took 6.03 seconds. The sealed archive is
4,378,698 bytes: 72 frozen source files, the native harness binary, 604 losslessly
interned full world snapshots, and 77 sealed files excluding the root seal itself.

Strict identity retains every ordinary NodeId and EdgeId, complete typed node
data, ordered edge endpoints, incidence and label indexes, world clocks,
accepted native events, ordinary birth/death handles and RNG probes. The Python
auditor independently checks these comparisons, graph-index consistency,
event/world birth-death correspondence, horizon completeness and budget bounds.
World interning only shares exactly equal snapshots; it does not rename handles
or approximate identity. The auditor is an independent evidence comparator,
**not** a second implementation of the Engine or the full coherent G-R1 auditor.

Reversing an ordinary edge's endpoints still yields an equal canonical hash but
fails exact typed-world identity. The negative control is retained. Read-only
frozen re-audit passes. Ten Python tests pass: the intact receipt is accepted,
and nine tamper types are rejected (handles, endpoint order, RNG, transitions,
truncated horizon, hidden legacy failure, promoted admission, excess matching
work, and erased negative control).

### Matching-cost limitation

Metadata scans still consume matching work. The first isolated pair exposed
parent/child work of 87/88 despite identical world state and events. This cost is
retained and bounded separately, not treated as typed-world state or erased from
the evidence. Maximum observed work/results were 9,196/404, well below 400k/50k;
no trace hit the matching ceiling. This fix does not claim dormant neutrality
under a binding budget, nor change the Engine's charging policy. A future assay
must keep ceilings and incomplete evidence explicit.

## Regression results and the old pin refusal

- Core, Engine and metadata suites: **99 tests passed** (33 / 46 / 20).
- Identity harness in debug and release: **3 passed / 1 explicitly ignored**;
  the ignored persisted diagnostic was separately run and passed.
- Unchanged coherent-kernel suite: **19 passed**.
- Unchanged raw G-R1 suite: **6 passed / 2 ignored**.
- Unchanged coherent-assay suite: **4 passed / 1 pin refusal / 2 ignored**;
  its native fixture matrix also passed nine Python tests.
- Existing training-admission checks: **2 passed**. Snapshot-wire checks: **2 passed**.

The old `freeze_copies_every_design_pin_before_measurement` test refuses the new
Cargo.lock hash. Two existing dependencies were added solely to test manifests:
bincode for hg-core compatibility tests and slotmap for the hg-sim-core legacy
reader fixture. No dependency version was upgraded. The old measured contract
pins the prior lock hash, so that refusal is correct; it was not bypassed or
updated to produce a green old-profile result. The current core implementation
and lock are a new source version, not the historically frozen version.

Both historical archives were checked read-only after this work: all 218 sealed
files in the coherent-v3 smoke and all 205 sealed files in the raw full trial are
unchanged. The old coherent contract hashes and coherent transaction source hash
also remain unchanged. Earlier statements about live sources matching archived
sources describe the earlier verification time, not the current modified core.

## Reproduction and next boundary

Read-only verification of the existing receipt:

```sh
python3 -B output/ere_identity_isolation_20260928/source_snapshots/scripts/verify_ere_identity_isolation.py output/ere_identity_isolation_20260928
PYTHONDONTWRITEBYTECODE=1 python3 output/ere_identity_isolation_20260928/source_snapshots/scripts/test_verify_ere_identity_isolation.py output/ere_identity_isolation_20260928
```

Ordinary regression tests:

```sh
cargo test --locked -p hg-core -p hg-engine -p hg-meta
cargo test --locked -p hg-sim-core --features hg-meta --test ere_identity_isolation -- --test-threads=1
```

The engineering failure is resolved for opt-in stores. The next experimental
step is a **new prospectively named coherent-assay profile**, with this constructor
selected at parent construction and fresh source/dependency pins, followed by a
bounded all-parent smoke. Preserve the old failed scientific and development
receipts. A full 10,000-draw assay requires separate authorization and still must
satisfy all original numerical, soundness, coverage and control requirements.
This two-parent diagnostic supplies no mutation-rate estimate and no admission.
Every new receipt retains `gate_pass=false` and `training_admitted=false`.

### Subsequent all-parent profile validation

The later user-authorized follow-up now implements a separately named prospective
isolated coherent assay and its all-parent eight-step smoke. All 168 baselines,
5,376 controls and 64 draws complete, with 22 successful Neutral soundness checks
and 27 rejected tamper categories. See
`ere_complete_coherent_isolated_assay_implementation.md` and
`output/ere_complete_coherent_isolated_smoke_20260928`. The original allocator
diagnostic remains unchanged. No full trial or training admission is implied.
