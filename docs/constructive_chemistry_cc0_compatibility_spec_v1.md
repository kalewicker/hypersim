# CC-0: bounded constructive chemistry compatibility specification v1

**Frozen design: 2026-10-05, America/Phoenix.** CC-0 specifies the next engineering fixture. The feature, transaction API and chemistry runtime below are proposed; this specification does not mark CC-1 implemented. Ordering follows the [integrated roadmap](evolving_minds_constructive_chemistry_integrated_plan_20261005.md). The executable fixture denominator and limits are frozen in the [CC-1 contract](../experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json).

## 1. Compatibility decision

Use Hypersim's existing `HypergraphStore` and generational handles. Add a default-off `constructive-chemistry` feature in `hg-sim-core`, propagating a proposed default-off `checked-transitions` feature in `hg-engine`. Keep orchestration and physical validation in `hg-sim-core::constructive_chemistry`; put only generic checked graph staging in `hg-engine::checked`. No new workspace, crate duplication, host organism controller, global generated rule table, or replacement of legacy rewrite semantics is needed for CC-1.

The current engine's atomicity is observational: an accepted rewrite completes before observers run, but mismatched property effects can be skipped after structural changes. Chemical rejection requires stronger failure atomicity. Preserve the existing `apply_binding`/`Engine::step` behavior, integer saturation and codecs in their current profiles; the new opt-in seam uses checked edits and all-or-nothing publication.

CC-1 is a **scripted transaction fixture**, not a chemical-rate simulation. It offers explicit legal or adversarial proposals in fixed order. It has no RNG, physical clock, biological identities, mutation, replication, membrane, operator compiler or optimization objective. A proposal list is not claimed to enumerate every physically eligible channel. CC-2 will introduce a separately frozen complete-channel kinetic reference.

## 2. Source-level basis

| Source | Observed capability / gap | Design consequence |
|---|---|---|
| `hg-core/src/graph.rs` | Store derives Clone; allocation_state exposes free-list/frontier continuation; get/add/remove node/edge APIs exist | Clone in memory for staging; use graph plus explicit allocator state for exact fingerprints and future checkpoints |
| `hg-core/src/arena.rs` | Clone retains free-list ordering; ordinary serialization writes slots and reconstructs free lists on deserialize | Do not implement rollback by graph-only serialize/deserialize; discarded clone must not consume live handles |
| `hg-core/src/property.rs` | Int=i64, Float=f64, Bool; node storage can contain duplicate keys | Conserved quantities use checked integers; schema rejects duplicate/missing/wrong-type properties |
| `hg-engine/src/engine.rs` | Current step selects and applies bindings in place; effect mismatch increments skip counts; Int arithmetic saturates | New staging seam; no skipped chemical effects or legacy arithmetic for chemistry |
| `hg-core/src/graph.rs` removal | Node deletion removes all incident edges automatically; add_edge does not reject missing endpoints itself | Validate every endpoint and deletion footprint before editing; unexpected cleanup is a transaction error |
| `hg-core/src/graph.rs` generation | NodeData.generation is a birth/step field, not a mutation version | Use a private profile-wide commit revision, not generation as a stale-read guard |
| `hg-meta/src/source.rs` | Derived shared table installs at epochs | No use for physical eligibility in this fixture; later operators need source-presence checks at commit |
| `hg-sim-core/Cargo.toml` | Composition is the only direct engine importer; no chemistry feature yet | Keep the new profile default-off and separate from D0/ERE configuration and checkpoints |

The CC-0 review reserves the following namespace after a targeted current-source/config/rule/registry collision scan. This is a reservation, not a claim that these properties are already present in running graphs.

## 3. CC-1 graph schema

### 3.1 Labels and properties

| Label | Hex | Meaning |
|---|---|---|
| SITE | `0xE2000000` | Fixed spatial site; administrative substrate |
| BEAD | `0xE2000001` | One physical unit of A/B/C/D/F |
| BOND | `0xE2000002` | Relation node implementing one physical A–B link |
| ADJACENT | `0xE2000064` | Binary unordered SITE–SITE edge |
| PLACED | `0xE2000065` | Binary unordered BEAD–SITE edge |
| BOND_A | `0xE2000066` | Binary BOND–A incidence |
| BOND_B | `0xE2000067` | Binary BOND–B incidence |

| Key | Type | Allowed nodes / value |
|---|---|---|
| 243001 `semantic_id` | Int | Every node, globally unique positive i64 |
| 243002 `site_index` | Int | SITE only, 0/1/2 |
| 243003 `species` | Int | BEAD only, A=0,B=1,C=2,D=3,F=4 |
| 243004 `activated` | Bool | BEAD only; true allowed only on F |
| 243005 `bond_kind` | Int | BOND only, A–B=0 |
| 243006 `port_a` | Int | BOND only, 0 |
| 243007 `port_b` | Int | BOND only, 0 |

Property sets are exact, with unique keys; unknown labels/properties fail schema validation. Legacy `NodeData.energy` must be positive zero bits; legacy energy property1 is absent. Energy is derived from material state, not duplicated in a mutable generic Float field. Creation generation equals the accepted commit revision. Normal initial graph generation/step is0; graph step and revision advance once per accepted transaction, never on rejection. The declared overflow cases admit a diagnostic initial heat/frontier/revision at its type maximum, with a corresponding valid baseline and graph step; these are explicit guard fixtures, not continuation or long-lived-world evidence.

Space is the fixed nonperiodic three-site chain `0—1—2`, with SITE semantic IDs1/2/3. No loop/parallel adjacency edges, coordinate aliases or changes to S are allowed. Site capacity is8 physical beads; bond relation nodes do not consume additional spatial capacity. Every bead has exactly one PLACED edge. Initial material semantic IDs begin at10; new IDs come from a checked monotone i64-compatible frontier, independently of arena handles.

An A/B/C/D bead has one port0 and valence at most1; F has valence0. Only A–B links are supported in v1. A BOND has exactly one BOND_A and one BOND_B incidence to distinct correctly typed beads. It cannot exist without endpoints or supply free catalytic/computational capacity. Its zero material mass is administrative representation of a constrained pair interaction; its state energy is charged explicitly. Arbitrary zero-mass assemblies and hidden data fields are invalid. C/D are inventory types with no reaction actions in CC-1, retained for explicit schema compatibility, not inferred chemical diversity.

### 3.2 State and ledgers

The private profile state contains graph, immutable world/profile/law identity, commit revision, semantic-ID frontier, finite reservoir counts, accumulated heat, material/energy baselines, and accepted-event records. Outside code receives read-only access; every successful state edit, including fixture setup mutations, changes revision through a controlled helper. Raw mutable store references must not escape.

Closed material vector is `(N_A,N_B,N_C,N_D,N_F)`, counting only BEAD nodes, including activated/inactive fuel. Closed actions preserve it exactly. Open actions balance world material against a finite reservoir's signed complementary inventory. Compute totals independently from graph rather than trusting a cached counter.

Define this bounded toy energy function, in integer quanta:

`U_world = 4 × activated_F_count + 2 × valid_AB_bond_count`

`U_reservoir = 4 × reservoir_activated_F_count`

`U_world + U_reservoir + heat = initial_energy_total`

Heat and reservoir inventories are nonnegative i64 with checked arithmetic. Derived material/energy totals and immutable baselines use checked i128, encoded as decimal strings in JSON so overflow tests can begin with valid large ledgers without overflowing the total itself. There is no work term in CC-1 and no ad hoc organism fee. This is a declared abstract stored-energy model, not a calibrated molecular chemistry. A bond stores2 quanta; consuming activated fuel supplies4, of which2 become heat. Inactive F retains its material identity and location. Do not credit F* energy a second time.

## 4. Allowed actions

### 4.1 FormBond(A,B,F*)

Three distinct bound BEAD handles and semantic identities must exist. Their species are A,B,F respectively; fuel is activated. All three are placed at the same site: CC-1 uses same-site spatial radius0. Both participant ports are free. The complete local schema, incidence/absence constraints and energy ledger must hold before and after.

The action changes fuel activation true→false, creates exactly one BOND node with ports0/0 and one incidence for each endpoint, increments heat by2, allocates one new semantic ID, and appends one accepted record. It does not create, delete, move or relabel physical beads. Material delta is zero; world U delta−2 and heat delta+2 balance. This applies to any qualifying A/B/F trio, not a privileged ID pair.

### 4.2 TransferFuelIn / TransferFuelOut

Boundary transfers occur only at site0 under a separately labelled external-boundary action. Transfer-in consumes one unit of declared activated or inactive reservoir F and creates one matching BEAD plus PLACED edge, if capacity permits. Transfer-out removes one free F and its exactly declared PLACED edge and credits that reservoir subtype. Activated transfers carry4 energy quanta; inactive transfers carry0. Heat stays unchanged. Combined world/reservoir material and energy remain constant.

These actions test open accounting; scripted boundary proposals are explicit external interventions, not autonomous organism behavior or a source of unlimited matter. Reservoir exhaustion rejects. Transfers may never use generic node deletion to erase A/B/C/D or a bonded assembly.

### 4.3 Complete edit and cleanup envelope

Physical validators compare proposed writes against the exact allowed-action template, including all created labels/properties and changed properties/edges. Reject extra edits, external node references, missing/wrong types, parallel/self edges, excessive valence, illegal placement, negative quantities or overflow. A node deletion must declare all incident edges; unexpected automatic cleanup rejects before publication. Malformed initial graphs fail admission with a retained diagnostic and no accepted events.

Fault injection points are diagnostic only: after fuel update, after bond-node creation, after first incidence, after ledger update and immediately before publication. Each injected failure must leave live state and allocator exactly unchanged. Recoverable allocation/serialization/output failures before publication reject or halt explicitly; process termination/OOM and durable recovery are CC-2 scope, not a transactional success claim.

## 5. Proposed checked-commit seam

Names are a proposed API, not existing Rust exports:

```rust
// hg-engine::checked: generic graph staging, behind checked-transitions.
fn stage_checked(
    before: &HypergraphStore,
    patch: &CheckedGraphPatch,
) -> Result<StagedGraphTransition, CheckedStageError>;

// hg-sim-core::constructive_chemistry: sole physical publication authority.
fn prepare(world: &ChemistryWorld, action: &ChemicalAction)
    -> Result<PreparedChemicalCommit, ChemicalRejection>;
fn commit(world: &mut ChemistryWorld, prepared: PreparedChemicalCommit)
    -> Result<ChemicalEvent, ChemicalRejection>;
```

`CheckedGraphPatch` names generational handle reads, expected typed values, exact incidence/absence observations, checked property replacements and ordered structural edits with local temporary IDs for new nodes. No unchecked arithmetic, arbitrary closures, jitter, unresolved references or silent skipped effects. Property replacements require the expected old value and the same value variant; multiple writes to one property are rejected rather than creating chained effect semantics. Structural edits use add/remove operations that maintain core indexes, not raw edge-data mutation. `StagedGraphTransition` carries a private fully staged store and resolved handles/deltas; it cannot mutate live state.

`PreparedChemicalCommit` is opaque and binds world identity, original revision and exact-state fingerprint to staged graph, staged ledger/frontier/revision and complete accepted receipt. All guards refer to the old state; all action effects are validated as one unit. Exact fingerprints encode graph data **and allocator continuation**, profile/law identity, revision, semantic frontier, reservoirs, heat and accepted records. A WL/canonical hash alone is insufficient. Avoid nondeterministic index-map iteration in fingerprint encoding: serialize the complete store wire and allocator metadata into a recursively key-sorted, bounded typed JSON value, preserving all array order, vacant-slot generations, adjacency/index data and tagged property types. Include the private profile fields in the same encoding. Sorting object keys is normalization of unordered index maps, not reconstruction from live nodes; graph-only live snapshots omit allocator/vacant-slot state. All legacy floating fields are restricted to positive zero, and scalar ledger arithmetic remains integer. Fingerprints require the same encoding/build version and compare exact encoded bytes as well as their SHA-256 index.

Preparation clones the live store in memory; it does not serialize/deserialize a graph-only snapshot. It stages every edit, rejects unlisted cleanup and validates full small-world schema/material/energy balance. Stage ledger/frontier/event data before publication, including fallible buffer reservations. Commit rechecks world identity/revision/fingerprint and publishes the complete owned profile state using one infallible replacement. Publish callbacks/observers run afterward and cannot mutate it. A later output error is an output failure, not rollback of an already accepted reaction; durable publication belongs to CC-2.

The initial stale-read policy is intentionally conservative: **any accepted intervening edit invalidates all previously prepared commits**, even disjoint ones. It prevents phantom insertions and stale absence/occupancy checks without adding mutation versions to shared core nodes. Reprepare a disjoint proposal against the new state before committing it. Fine-grained dependency revisions and parallel commit batches require a later proven extension.

Diagnostic rejected-attempt records are returned to the caller separately; they do not advance physical revision, ID frontier, accepted journal, graph step, heat, reservoir, RNG or time. No RNG/time exists in this fixture. Fault/administrative setup edits are separately labelled, never counted as chemical successes.

## 6. Determinism and disjoint composition

The shared-fuel fixture uses four beads A10/A11/B20/B21 and activated F30 at site0. Offer `(A10,B20,F30)` and `(A11,B21,F30)` in each fixed order. Exactly one accepts; the second prepared on the old revision rejects stale. Its newly prepared form then rejects spent fuel. Other A/B pairings are legal under the generic action but are not sampled by this scripted test; CC-1 makes no channel-distribution claim.

The disjoint fixture supplies two F units and offers `(A10,B20,F30)` / `(A11,B21,F31)`. A stale second preparation still rejects under the global revision policy; reprepare it and both actions accept. Reversing their serial order yields the same material organization, fuel states and energy/heat totals under a physical projection of bead identities/species/activation/placement, bond endpoint roles/ports and ledger totals. This projection excludes bond administrative IDs, arena handles, creation generations, commit revision and receipt order; these are nonphysical metadata in CC-1 and remain intact in the raw artifacts. Allocation IDs and ordered receipts need not be equal across intentionally different commit orders. Within a fixed case/order, one- and four-worker candidate-preparation variants must publish the **same serial order and exact graph/allocator/ledger/accepted evidence**; worker completion cannot choose outcomes.

The frozen panel has41 cases ×2 worker-count variants =82 case variants. Worker counts1/4 exercise preparation only; all commits remain serial. Initial-admission and invariant-only cases are repeated under the two harness settings without being mistaken for parallel reactions. No worker count is a scientific replicate. Cases are deterministic invariant checks, not independent worlds or a seed campaign. Performance limits halt the fixture rather than drop candidates or partial transactions.

## 7. Evidence, implementation order and limits

CC-1 outputs initial state, every proposal/rejection, accepted graph/property/ledger deltas, final exact graph and allocation state, semantic event IDs, source/binary/config/law hashes, material/energy reconstruction and explicit false admission flags. All listed cases are retained, including malformed initial worlds and faults. A fresh dictionary-based Python reference uses semantic IDs and checked integer arithmetic; it must not call or copy the Rust physical validator. Compare accepted/rejected decisions, material/energy deltas and normalized semantic graph; additionally verify native allocator preservation and exact same-order worker determinism independently.

Implement in order: feature/schema and constructor; generic graph staging; whole-state physical prepare/commit; closed/open ledger cases; adversarial/fault cases; independent checker and artifact publication; existing-profile regressions. CC-1 creation is gated on a source freeze and all contract case definitions. No quantitative evolution threshold is required for these exact engineering invariants.

The frozen envelope is deliberately small:3 sites, at most16 physical beads,32 total nodes,64 edges,32 declared action offers per case,1MiB exact native state per case,16MiB artifacts per case and128MiB total output. Site capacity8 remains a physical rule; graph/event/log caps are engineering limits. Native execution cap300s, independent audit300s, focused build/tests1200s. Record measured costs; do not silently raise binding limits. Compilation/cache size is separate from experiment output. Full stochastic time, resumable durable checkpoints and scaling are CC-2; prototype exact state export does not claim resumable simulation.

## 8. CC-0 completion and next step

CC-0 completion requires the source-gap audit, collision-free registry reservation, fixed law/schema, explicit atomicity API and publication ownership, declared scheduling/RNG scope, independent-checker design, resource envelope, and machine-readable CC-1 denominator. It is a specification milestone, not a passing reaction fixture. The next implementation step is **CC-1** under the linked frozen contract. All scientific, training, evolution, sustained, fairness, power and launch flags remain false.
