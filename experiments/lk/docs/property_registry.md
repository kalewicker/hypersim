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
| 7 | `countdown` | Int | LK v6 §4 removal clock: subdivision endows the child `set k`; `lk_tick` (single-node LHS) `subtract 1` while guard `countdown > 0`; fuse admits at `countdown == 0`. The first LK gate variable with dispersion by construction. Excluded by name from CD micro features (v6 §1.3). |
| 8..=63 | — | — | Unassigned; free for experiment-local use. |

## Disambiguation: names bound to more than one concept (v9 §1.4)

**Rule:** introducing a name already bound to a different concept in this
codebase requires an entry here. The entries below are the ones that
existed when the rule was written; "generation" is why the rule exists.

### "generation" — three distinct things, one of them now renamed

| what | where | type | meaning |
|---|---|---|---|
| slot generation | `slotmap` `NodeId`/`EdgeId` | internal | the ABA counter that makes a stale id detectably stale. Never read by rules. |
| **property key 6**, `generation` | node properties | Float | LK filament depth: IC nodes 0.0, each subdivision child parent + 1. What `lk_fuse`'s lattice protection guards on. |
| `NodeData::birth_step` | node field | u64 | the step the node was created. What the v8 age guard reads. **Named `generation` until v9 C2.1.** |

The v8 age guard reads the **field**; `lk_fuse`'s lattice protection reads
**key 6**. A guard written against the wrong one compiles, runs, and
silently means something else — which is why the field was renamed rather
than merely documented. `#[serde(rename = "generation")]` preserves the
persisted name; archived-snapshot readability is the acceptance check
(`archived_snapshot_round_trips_after_birth_step_rename`, hg-engine).

Also note `generation_tag` (property key 4, Int) — an author-assigned
cohort id, unrelated to all three.

## Birth step is a FIELD, not a key (v8 B0, 2026-07-30)

The v8 passive clock (`[lk_rules.fuse] gate = "age"`) reads a node's
creation step and needs **no property key**. `NodeData::generation` — the
store's structural field, distinct from property key 6 — is populated
with `store.step()` at every creation site (`engine.rs` RHS node
creation; `generators.rs` at IC time) and is never written again; no
`property_effects` op can touch it. The `node_age_compares` guard
evaluates `(step − NodeData::generation) <op> value`, taking the step
from the snapshot the matcher already holds.

**No key was reserved for `birth_step`, and key 8 remains free.** A
future author looking for the birth step should not go looking for a key.

Key 7 (`countdown`) is retained unchanged and NOT reused, so archived
ticked runs stay readable and the EXP-L18 comparison arms load as
written.

Naming hazard, stated once: the field `NodeData::generation` (creation
step, u64) and property key 6 `generation` (LK subdivision generation,
Float) are different quantities with the same word. The age guard reads
the field; lk_fuse's lattice protection reads the key.

WL invariance holds on this channel too, but for a different reason than
the property-map exclusion below: the WL base hash covers label + energy
field + structure and deliberately omits `generation` (see `hash.rs`).
Pinned by `micro_features_ignore_birth_step_and_age` (hg-causal), which
exists because the v6 clock-exclusion guarantee was about properties and
does not extend to a field on its own.

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
