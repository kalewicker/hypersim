# Prospective allocator identity isolation v1

Scope: resolve the dormant-metadata allocation interference observed in coherent
development draws 19 (parent 138) and 27 (parent 30). This is a bounded engineering
diagnostic, not a replacement G-R1 assay, numerical gate, or training admission.
The measured coherent v1 contract, kernel, harness, auditor and archives remain
unchanged. No full trial is authorized by this work.

## Mechanism and compatibility

Add an opt-in `HypergraphStore::new_metadata_isolated()` constructor. Allocate
ordinary and reserved metadata labels (at or above `0xFFF00000`) in independent
generational domains, for both nodes and edges. Odd indices belong to ordinary
objects; even indices belong to metadata objects. Each domain has its own next
index and LIFO recycled-slot list. Dispatch is determined at insertion; relabeling
an object never migrates its handle. Mixed-domain edge endpoints are supported.
No dummy graph objects, re-encoding, post-hoc handle renaming, or repair is used.

Existing/default stores retain legacy SlotMap allocation, iteration and serialized
layout. Opt-in stores carry a distinct sentinel marker in their arena payloads;
new readers preserve the mode and old SlotMap readers must reject them. Enabling
isolation on an existing graph or checkpoint is not supported: build a new store.
Serialization retains vacant-slot versions; as with existing SlotMap serialization,
deserialization reconstructs free-list ordering rather than preserving historical
removal ordering. Round-trip continuation comparisons use identically round-tripped
parent and child, and separately compare continuation to uninterrupted execution
where there is at most one recycled slot per domain. Generation exhaustion in the
isolated mode fails closed rather than confusing an unused lane with a wrapped slot.

## Precommitted bounded verification

- Compare legacy insertion/removal, stale-handle behavior, iteration, and serialized
  bytes to the actual SlotMap reference, including recycled generations and restore.
- Verify isolation under unequal metadata birth/death histories, both node and edge
  reuse, incidence and label indexes, mixed-domain edges, relabeling, cloning and
  binary serialization; reject the opt-in payload with a legacy reader.
- Reconstruct semantic parents 138 and 30, apply the same dormant Proposal AddVar
  requests as the failing draws, and execute the real complete-enabled graph-rule
  source and Engine. Run parent/child pairs for 100 steps with seed 42, max-parallel
  mode and pooled 400k/50k limits, retaining exact typed projected worlds, ordinary
  handles, ordered edge endpoints, transitions and RNG states.
  Match-work accounting is retained and independently bounded, but is not a
  typed-world identity field. Metadata scans can still cost work; this fix does
  not change charging or claim neutrality at a binding matching ceiling.
- Require the legacy pairs to reproduce the failure and isolated pairs to eliminate
  it, with actual ordinary births (not a vacuous no-rewrite comparison). Include a
  checkpoint continuation pair and metadata recycling comparison.
- Keep the negative control in which endpoint reversal has an equal canonical hash
  but a different exact typed world. Such a change must still fail strict identity.
- Freeze relevant source hashes in a fresh diagnostic receipt and use a separate
  read-only Python auditor for exact world/transition/RNG comparison. Bound the
  diagnostic to 2,000 Engine steps, 300 seconds, and 64 MiB persisted evidence.

All receipts explicitly retain `gate_pass=false` and `training_admitted=false`.
Integrating this constructor into a future assay requires a new prospective profile,
fresh source pins, and separate approval for any full campaign.
