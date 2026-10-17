# Property Registry

Plain documentation table (§3 of the v4 plan) of which node-property key
conventionally holds which `PropertyValue` variant. This is **not** loaded
or enforced by the parser — it is a convention for human rule authors. The
parser accepts any key; comparison semantics (`Guard::NodePropertyOp`) are
variant-exact: mismatched variants make the guard return `false`, never
panic or coerce.

`PropertyValue` variants: `Int(i64)`, `Float(f64)`, `Bool(bool)`. In TOML,
the literal's own syntax selects the variant (`100` → Int, `0.8` → Float,
`true` → Bool) via `#[serde(untagged)]`.

| Key (u32) | Name | Variant | Meaning |
|---|---|---|---|
| 0 | `age` | Int | Steps since the node was created. |
| 1 | `energy` | Float | Energy proxy; drives GUI color mode `E` (blue→red). |
| 2 | `activation` | Bool | Generic on/off flag for rule guards. |
| 3 | `charge` | Int | Signed integer count/charge. |
| 4 | `generation_tag` | Int | Author-assigned cohort id (distinct from the store's structural `generation`). |
| 5 | `fidelity` | Float | Replication fidelity marker (reserved for hg-alife). |
| 6 | `generation` | Float | LK v3 §1.4 subdivision generation: IC nodes 0.0 (initialized by `[lk_rules] g_max`), children parent + 1 via `scale_from ×1.0; add 1.0` (order pinned by test). Guarded by the capped rule family (rules/lk_pair_capped.toml); consumed by the chain census. Float because `scale_from` produces Float. Distinct from the store's structural `generation` field. |
| 7..=63 | — | — | Unassigned; free for experiment-local use. |

Energy is stored as a first-class `f32` on `NodeData` **and** may also be
mirrored into property key 1 when a rule needs to guard on it; the registry
records key 1 as the Float energy convention so guards and the renderer
agree without a formal schema.

## ALife reservation (2026-07-09)

ALife genome keys will be claimed in this registry when `hg-alife` lands.
Key 1 (energy, Float) is unchanged and is the key the node-funded
replication mechanic reads and writes via `property_effects`
(docs/rule_grammar.md).

### ALife key block (reserved 2026-07-10, `hg-alife` S2.1)

| Key (u32) | Name | Variant | Meaning |
|---|---|---|---|
| 10 | `harvest_rate` | Float | Genome trait (deferred as active trait in Exp. 5). |
| 11 | `replication_threshold` | Float | Genome trait; carrier replicates when energy ≥ this. |
| 12 | `maintenance_cost` | Float | Genome trait (deferred as active trait in Exp. 5). |
| 13 | `satiation` | Float | Genome trait; carrier stops harvesting when energy ≥ this (crowding via §6.2). |
| 20 | `carrier_marker` | Bool | One-shot bootstrap seed marker; set by the generator, cleared by `bootstrap_seed`. |
| 21 | `lifetime` | Float | Carrier age counter (reserved; not active in Exp. 5). |

## WL / Q2 invariant (2026-07-10) — WL hash excludes the properties map

### Evolving Minds engineering profile (2026-09-29)

Feature `evolving-minds`, profile `evolving-minds-engineering-v1`, uses a
separate fixture store. Metabolic energy remains key 1 Float; the `energy`
field remains zero. These keys are not added to the ERE encoding envelope.

| Key | Name | Variant | Meaning |
| --- | --- | --- | --- |
| 240001 | owner | Int | Immutable organism birth identity on internal material |
| 240002 | opcode | Int | Single authored instruction: Idle=0, BuildRelay=1 |
| 240003 | developed | Bool | Common seed has completed its one instruction |
| 240004 | age | Int | Completed mandatory maintenance ticks |
| 240005 | signal | Bool | Generic sensor/actuator state |
| 240006 | occupied | Bool | Site occupancy, independently checked against topology |
| 240007 | birth_id | Int | Monotone non-reused organism identity |
| 240008 | parent_id | Int | Parent birth identity; founder uses 0 |
| 240009 | next_id | Int | Matched birth-counter state |
| 240010 | maintenance | Float | Fixed material upkeep: common seed 3, relay 5 |

Ordinary-material labels occupy `0xE1000000..0xE1000007`; role relations occupy
`0xE1000064..0xE100006B`, below the ERE reserved metadata band. See the
[engineering contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/engineering_v1.md).

### Evolving Minds authored choice profile (2026-10-01)

`evolving-minds-choice-v1` is a separate fixed-law profile using the same role
labels. Its inherited opcode (240002) denotes Inactive=0, Constant0=1,
Constant1=2, Random=3 or Reactive=4; every controller develops a paid relay.
These opcode meanings do not change `evolving-minds-engineering-v1`.

| Key | Name | Variant | Meaning |
| --- | --- | --- | --- |
| 241001 | world_clock | Int | Native site schedule cursor, unavailable to controllers |
| 241002 | resource_kind | Bool | Physical resource subtype; settlement law only |
| 241003 | local_cue | Bool | Current local sensory input |
| 241004 | local_coin | Bool | Independent exogenous randomness for authored random control |
| 241005 | sensor_coin | Bool | Owned sensor's local coin projection |
| 241006 | controller | Int | Immutable authored controller identity on organism anchor |
| 241007 | attempted | Bool | Exactly one paid action pending settlement |
| 241008 | committed_choice | Bool | Choice fixed at attempt time, not changed by outcome |
| 241009 | site_slot | Int | Fixed environmental table slot, unavailable to controllers |

See [choice contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/choice_v1.md).

## Evolving-minds D0 development v1

Profile `evolving-minds-d0-development-v1` shares ENERGY, owner, age,
birth identity, signal and world/choice keys, with a separate genotype grammar.
Here opcode 240002 means State=0, Actuator=1, Wire=2, Halt=3. Maintenance
240010 is the price derived from all founder-owned material, including slots
and instructions. This profile implements bounded development, without births
or mutation; prior profile meanings remain scoped to their own contracts.

| Key | Name | Type | Meaning |
| --- | --- | --- | --- |
| 242001 | cursor | Int | Next instruction position, distinct from stable instruction ID |
| 242002 | execution_status | Int | Ready=0, waiting=1, halted=2, permanent failure=3 |
| 242003 | instruction_position | Int | Contiguous genotype order |
| 242004 | instruction_id | Int | Unique immutable u32 ID, independent of arena allocation |
| 242005 | destination_slot | Int | Instruction destination or material/map local slot, 0–7 |
| 242006 | source_slot | Int | Wire source slot, 0–7; zero for other instructions |
| 242007 | initial_signal | Bool | State initialization parameter; false for other instructions |
| 242008 | slot_used | Bool | Slot map descriptor occupancy |
| 242009 | has_input | Bool | State/actuator has its one allowed writer |
| 242010 | next_signal | Bool | Buffer read from pre-commit source state |
| 242011 | has_actuator | Bool | Root's one-actuator allocation flag |

State and slot descriptor labels are `0xE1010000` and `0xE1010001`.
Owned-part, slot-map and adjacent-instruction relations are `0xE1010064`–
`0xE1010066`; source/target use the existing labelled port relations. Slot 0
contains the cue sensor, slot 1 the independent coin sensor; slots 2–7 begin
vacant. See [D0 contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_development_v1.md).

## Evolving-minds faithful D0 family v1

Separate profile `evolving-minds-d0-family-v1` reuses D0 instruction and port
semantics, with fixed material prices, native faithful birth and paid turnover.
PARENT_ID (240008) is immutable ancestry; OCCUPIED (240006) and counter
NEXT_ID (240009) retain their scoped meanings. All offspring runtime fields
reset while instruction IDs/parameters copy unchanged. ROOT becomes DEAD
during energy-settled disassembly; no tombstone persists at a completed tick.

| Key | Name | Type | Meaning |
| --- | --- | --- | --- |
| 242012 | genome_length | Int | Immutable instruction count, 1–8; independently checked against genotype |
| 242013 | internal_parts | Int | Owned internal count; native construction increments and cleanup decrements |

MAINTENANCE (240010) is the sum of fixed prices: root 1; slot descriptor
1/16; instruction 1/8; sensor/state/actuator/connection 1/4. It cannot evolve.
See [family contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_family_v1.md).

### Existing WL invariant

**Design rule:** a node's WL hash depends ONLY on its `Label`, its `energy`
`f32` field, and its neighbourhood structure. The per-key `properties` map is
**never** hashed. Consequence: setting *any* property — energy (key 1), a genome
trait (keys 10–13), the marker (key 20) — leaves the node's WL class unchanged.

**Why:** carriers are hypergraph nodes and store their continuous, jittered
genome as properties. If WL hashed properties, every carrier (and every energy
level of every carrier) would be its own WL class; the class registry would
explode and `EI_micro → log2(M)` with M growing as the population — the scope
doc's §9 Q2 catastrophe. Excluding the whole properties map (rather than a
denylist of specific keys) makes the guarantee un-rottable: a new property key
is automatically WL-invisible, no hasher edit required.

Enforced by `hg_core::hash::tests::wl_hash_invariant_under_property_change`
(and `wl_hash_still_sees_energy_field`, which pins the intended asymmetry: the
energy *field* — as opposed to property key 1 — remains in WL, since it is also
part of `canonical_hash` and the EI energy buckets). Implementation:
`crates/hg-core/src/hash.rs`, both `compute_wl_hash` and `compute_wl_class_map`.

**S0.1 determinism (confirmed 2026-07-10):** WL hashing uses `FxHasher` seeded
with the compile-time constant `WL_SEED = 0x9E3779B97F4A7C15` — never
`DefaultHasher`/`RandomState`. Hashes are therefore identical across processes,
so cross-run WL comparisons in the characterization report are valid. The plan's
worry that the shipped hasher might have diverged to a random seed does not
hold.

## Evolving-minds one-shot source mutation v1

| Key | Name | Type | Meaning |
| --- | --- | --- | --- |
| 242014 | request_id | Int | Native request counter frontier / immutable escrow attempt ID |
| 242015 | transaction_status | Int | Escrow pending 0 / committed 1 / rejected 2 |

The separate `evolving-minds-d0-mutation-v1` profile adds an escrow node labelled `0xE1020000` and parent/target relations `0xE1020064`/`0xE1020065`. Its Float energy participates in conservation until native settlement and zero-energy disposal. A fixed request charge 1 precedes one recorded proposal; rejected syntax refunds the unused child/copy reserve. Counter property 242014 advances on admitted requests; birth IDs advance only on committed children. Faithful-family v1 counter schema and checkpoint format are unchanged.


## Evolving-minds lifecycle controls and arbitration v1

| Key | Name | Type | Meaning |
| --- | --- | --- | --- |
| 242016 | communication_enabled | Bool | CONNECTION signal-prepare gate, set by native development/control laws in lifecycle-controls v1 |
| 242017 | offered | Bool | SITE target-offer mask in the separate four-pass arbitration diagnostic |
| 242018 | local_next | Int | SITE birth-ID frontier, initially 3 + physical slot, advanced by 8 in the local arbitration prototype |

A disabled connection retains its ordinary ownership, ports, genotype and maintenance burden. Native controls wash out actuator signal/buffer after sensing and before prepare; an explicit zero external control charge applies equally across arms. Offered/local-next properties apply only to the arbitration profile. The main faithful-family and one-shot mutation profiles retain their global counters. See [prospective contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_lifecycle_fairness_v1.md).


## Evolving-minds parallel ecology admission v1

The separate `evolving-minds-d0-admission-v1` profile reuses SITE `local_next` (242018, Int) with a new declared update contract: initially `3 + logical slot`, advanced by eight on every native identity pass, once per tick. Request IDs (242014) and successful child birth IDs copy this frontier. Invalid settlement leaves identity holes. Logical slot is physical slot minus the fixed initialization rotation modulo eight. No COUNTER node exists in this profile; the earlier family/mutation and arbitration meanings remain fixed.

Escrow properties/relations and fixed material prices retain their mutation-v1 meanings. Multiple escrows may coexist for disjoint admitted offers and must all settle/dispose before a completed tick. The read-only external scheduler records semantic offers and random permutation draws; it never writes material. See [admission contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_admission_v1.md).


## Evolving-minds live-state storage v1

The opt-in `evolving-minds-d0-streaming-v1` storage profile introduces no graph labels, properties, native rules, physiology prices or scheduling laws. It wraps admission-v1 with a separately versioned live checkpoint and immutable committed evidence segments. SITE local tick IDs and escrow semantics retain the admission-v1 meanings above. Founder/depth metadata and evidence cursors are host persistence records, not organism-accessible graph properties. See [storage contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_streaming_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_streaming_findings.md).


## Evolving-minds paid source-neighborhood diagnostic v1

`evolving-minds-d0-neighborhood-v1` introduces no graph labels/properties or material prices. It reuses mutation-v1 escrow and the global birth/request counters. A supplied post-first-request native reserve guard requires parent birth ID >=2, excluding the original founder from further requests; descendants copy faithfully. The first request is an enumerated source edit with explicit provenance and no stochastic draws. Diagnostic lifetime/proposal records are host evidence, not organism-accessible properties. This eligibility intervention is separate from admission-v1 and does not establish reproductive fairness. Old checkpoint export rejects this unsupported diagnostic state. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_neighborhood_v1.md) and [decision](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_h1_h2_decision_20261003.md).


## Evolving-minds parent-funded reserve diagnostic v1

`evolving-minds-d0-funding-v1` introduces no graph labels/properties. It separately derives native energy eligibility/debit, escrow deposit/copy guard and newborn reserve for fixed endowments12/24, charging the entire transfer to the parent. Mutation-v1 request/copy/maintenance costs and death/refund meanings remain fixed; invalid syntax refunds actual escrow. Host funding/proposal/lifetime records are evidence only. The neighborhood-v1 supplied founder exclusion and faithful descendants are retained for this bounded diagnostic; old checkpoint export rejects it. This is not funded semantic admission or a scientific fairness claim. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_funding_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_funding_findings_20261003.md).


## Evolving-minds configurable funded admission and streaming v1

`evolving-minds-d0-funded-admission-v1` and `evolving-minds-d0-funded-streaming-v1` introduce no graph labels or property keys. SITE local tick IDs and semantic scheduling retain admission-v1 meanings. Parent energy eligibility/debit and escrow/copy/refund amounts use the explicitly configured endowment12/24, with no diagnostic founder exclusion. Founder programs, kernels, world seed, homes, rotation/allocation order, horizon and scheduler are host configuration/evidence records, bound into validation and persistence; the legacy coordinate adapter recipe is not the funded physiology authority. New live envelope `HGEMFS01` carries the complete configuration; old `HGEMS001` remains unchanged. Founder/depth metadata and evidence cursors remain host-only records. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_funded_admission_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_funded_admission_findings_20261003.md).

## Evolving-minds retained first-request comparator v1

`evolving-minds-d0-retained-proposal-v1` and `evolving-minds-d0-retained-streaming-v1` introduce no graph labels or property keys. Policy `founder-first-proposal-v1` records each initial founder's first admitted paid proposal in bounded host state, with original request/genome/edit/draw/status provenance and unsettled receipt fields. Later founder requests reuse the proposal under fresh native identities/settlement; descendants copy their inherited genotype faithfully. Samples are bounded to the two founders and survive their deaths; they are not organism-accessible properties. Live envelope `HGEMRP01` adds this policy/sample state to the existing funded live state. Paid eligibility, escrow, upkeep, semantic scheduling and SITE tick IDs keep their funded-admission meanings; `HGEMFS01`/`HGEMS001` layouts remain unchanged. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_retained_proposal_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_retained_proposal_findings_20261004.md).

## Evolving-minds matched development pilot v1

`evolving-minds-d0-matched-development-v1` orchestrates fixed five-arm development cases on existing funded/retained native profiles. It introduces no graph labels, property keys, prices, grammar or scheduler laws. World/arm/configuration and ecological/resource/power decisions are host evidence records. Funded `HGEMFS01` and retained `HGEMRP01` envelopes retain their meanings. Completed native execution and independent reconstruction are separate from failed development feasibility and scientific admission. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_matched_development_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_matched_development_findings_20261004.md).

## Evolving-minds failure diagnosis and shared audit v1

`evolving-minds-d0-failure-audit-optimization-v1` is archive-only analysis, adding no native graph labels/properties, prices, genotype operators or checkpoint formats. Request-route categories, AST derivation receipts, regenerated boundary comparisons and measured audit projections are host evidence records. It executes zero native advances and preserves the failed pilot's ecological criteria and scientific boundary. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_failure_audit_optimization_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_failure_audit_optimization_findings_20261004.md).

## Evolving-minds paid supplied-edit handoff v1

`evolving-minds-d0-paid-handoff-v1` / `evolving-minds-d0-handoff-streaming-v1` add no graph labels or property keys. Immutable host `HandoffConfig` binds `paid-beneficial-edit-v1`, Useful/Unchanged/Invalid/Unreachable supplied founder proposals and Faithful/SourceKernel descendants. Founder edits occur only after native paid reservation; parents keep their original genomes. Live envelope `HGEMHO01` adds that bounded configuration to funded live state. Requests, copying, escrow, development, upkeep, semantic scheduling and refunds retain funded meanings. Old envelope layouts and retained policy stay unchanged. Handoff-only CleanStop is declared at24; older profiles retain their tick8 stop. The audit continuation changes only a resource dictionary key and introduces no native semantics. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_paid_handoff_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_paid_handoff_findings_20261004.md).

## Evolving-minds descendant route diagnosis v1

`evolving-minds-d0-descendant-route-diagnosis-v1` is archive-only analysis, adding no graph labels/properties, native operators, prices or checkpoint formats. Canonical genotype-class hashes, incoming request routes, observed boundary intervals, installation cohorts, root/class censoring and explicitly nonadditive descendant subtree summaries are host evidence records. Class recurrence through independent mutation is distinct from reproductive propagation. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_descendant_route_diagnosis_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_descendant_route_findings_20261004.md).

## Evolving-minds paid descendant source neighborhood v1

`evolving-minds-d0-paid-descendant-neighborhood-v1` / `evolving-minds-d0-paid-descendant-streaming-v1` add no graph labels or property keys. Host `DescendantConfig` binds `first-paid-descendant-by24-v1` and unchanged or one of35 fixed source substitutions around authored Immediate. After native paid reservation, each founder family targets its first descendant request by24, choosing the smallest semantic parent birth ID within simultaneous requests; no late replacement or viability filter is allowed. Founders supply the prior useful edit, parents remain unchanged and all other copies are faithful. `DescendantState` retains at most two original pre-settlement receipts after organism death; these are inaccessible to organisms. Envelope `HGEMDN01` wraps funded live state plus configuration/targets, preserving old envelope layouts. Header configuration, externally bound commits and independent full-prefix replay establish first-request/cache provenance and detect omissions; resident cache validation alone does not prove historical firstness. CleanStop for this profile is24. Prices, grammar, copying, B24 child funding, refunds, development, upkeep and semantic scheduling retain funded meanings. See [contract](/Users/kalewicker/Documents/hypersim/experiments/evolving_minds/contracts/d0_paid_descendant_neighborhood_v1.md) and [findings](/Users/kalewicker/Documents/hypersim/docs/evolving_minds_paid_descendant_neighborhood_findings_20261005.md).

## Constructive chemistry CC-0 reservation — proposed CC-1 v1

Reserved 2026-10-05; no executable chemistry feature or graph profile exists yet. The [compatibility specification](/Users/kalewicker/Documents/hypersim/docs/constructive_chemistry_cc0_compatibility_spec_v1.md) and [frozen fixture contract](/Users/kalewicker/Documents/hypersim/experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json) define the next implementation. Existing labels/properties/codec meanings are unchanged.

Node labels SITE/BEAD/BOND are `0xE2000000..0xE2000002`. Binary relation labels ADJACENT/PLACED/BOND_A/BOND_B are `0xE2000064..0xE2000067`. All are below the reserved ERE metadata allocation band. Ordinary edge endpoints remain unordered; distinct incidence labels carry roles.

| Key | Name | Type | Proposed profile meaning |
|---|---|---|---|
| 243001 | semantic_id | Int | Globally unique positive i64 identity on each SITE/BEAD/BOND |
| 243002 | site_index | Int | SITE coordinate0/1/2, separate from revision/time |
| 243003 | species | Int | BEAD type A0/B1/C2/D3/F4 |
| 243004 | activated | Bool | BEAD activation; true allowed only on F |
| 243005 | bond_kind | Int | BOND A–B kind0 |
| 243006 | port_a | Int | BOND A endpoint port0 |
| 243007 | port_b | Int | BOND B endpoint port0 |

Chemical material and integer stored energy are derived from this exact schema. Legacy node energy must be positive zero bits and property1 is absent. Reservoirs, heat, revision, semantic frontier and accepted records are private profile state with checked arithmetic; cell/owner/genome/fitness labels have no physical role. The reservation is not evidence of conservation, atomic commit, chemical kinetics or protocell reproduction.
