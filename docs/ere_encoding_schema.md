# ERE Encoding Schema (R0 spec)

**Status:** specification + current-status mapping. No engine changes.
**Target:** E1 per the ERE track plan v1 §3. **Actual today:** the
`hg-selfmod` v3/v4 encoding (`crates/hg-selfmod/src/schema.rs`). This document
specifies E1 and records exactly where the current code differs, so every
downstream novelty claim can state which encoding it was measured under.

## 1. E1 target schema (unchanged from plan v1 §3)

### 1.1 Label bands

```rust
pub const META_BASE: u32 = 0xFFF0_0000;   // labels >= META_BASE are meta
pub const L_RULE_ROOT: u32   = META_BASE + 0x01;
pub const L_SIDE_L: u32      = META_BASE + 0x02;
pub const L_SIDE_R: u32      = META_BASE + 0x03;
pub const L_PAT_EDGE: u32    = META_BASE + 0x04;
pub const L_VAR: u32         = META_BASE + 0x05;
pub const L_LABEL_LIT: u32   = META_BASE + 0x06;
pub const L_GUARD: u32       = META_BASE + 0x07;   // E1-G
pub const L_SCHEDULE: u32    = META_BASE + 0x08;   // E1.1 (SB-R2)

// edge labels
pub const E_BECOMES: u32     = META_BASE + 0x40;
pub const E_MEMBER: u32      = META_BASE + 0x41;
pub const E_SLOT_BASE: u32   = META_BASE + 0x50;   // + k, k < MAX_ARITY
pub const E_HASLABEL: u32    = META_BASE + 0x70;
pub const E_GUARDS: u32      = META_BASE + 0x71;   // E1-G
```

`MAX_ARITY = 8` (config constant). Slot position is carried in the **edge
label** (`E_SLOT_BASE + k`), not tuple position, so deleting one slot edge of
three leaves a valid arity-2 pattern edge rather than an arity-3 edge with a
hole. The compiler compacts surviving slots by sorted label.

### 1.2 Structure

```text
RULE_ROOT --E_BECOMES--> (root, side_l, side_r)      arity 3, positional
side_*    --E_MEMBER-->  (side, pat_edge)            one per pattern edge
pat_edge  --E_SLOT_k-->  (pat_edge, var)             one per slot
var       --E_HASLABEL-> (var, label_lit)            optional; absent = free
label_lit                 carries Int property key 0 = the label value
RULE_ROOT                 carries Int property key 2 = priority
```

Design principle (§3.1): the encoding is **homomorphic to the semantics**.
A var node wired to LHS edges, RHS edges, or both directly yields the
delete/survive/create classification. No name table. Single-edit mutations
(attach/detach a var from a side) map to single semantic changes
(preserve ↔ delete ↔ create).

### 1.3 E1-G guards

`GUARD` node carries `Int(key)`, `Int(op_code)`, `PropertyValue` threshold,
wired to a var by `E_GUARDS`. Ops reuse `Guard::NodePropertyOp` and its
mismatched-variant-fails semantics unchanged. Required before arm A3;
not required for A0–A2.

### 1.4 Expressivity envelope (stated limits)

E1 cannot express (measured by G-R2, not claimed):

* E1a. ~~No property effects (E2). Rules read properties (E1-G) but not write.~~
  **LIFTED** for the RuleSpec encoding: `EFFECT` nodes carry the full 8-op set
  (set/copy_from/scale_from/add/subtract/add_from_property/
  subtract_from_property/jitter) with target/source refs and op payloads,
  decoded under V9 (declaration order preserved — sequential semantics) and
  compiled to `PropertyEffectDecl`s for `compile_rule` resolution. No mutation
  operator touches effects yet (frozen trait, same doctrine as E1d).
* E1b. No creation-rule schedules (SB-R2; E1.1 adds `L_SCHEDULE`).
* E1c. No edge-label constraints in patterns.
* E1d. ~~No `VarsDistinct` control (compiler decision, not encoded).~~
  **LIFTED** for the RuleSpec encoding: `DISTINCT` nodes (`K_VAR_ID`,
  `K_VAR_ID2`, normalized `a < b`) carry expanded unordered pairs, decoded
  under V6 and compiled to one 2-var `VarsDistinct` guard per pair (which
  expands to exactly that pair — no engine changes). E1 target unchanged.
  No mutation operator touches pairs yet (frozen trait: fission copies them
  verbatim; priority-unevolvable analogue — a deliberate follow-up, not
  drift).
* E1e. Arity capped at `MAX_ARITY = 8`.

## 2. Current implementation status (`hg-selfmod` v3/v4)

Rules live in a **separate** `HypergraphStore` (not the world store), decoded
each step via `decode_spec` → `RuleSpec` → `spec_to_compiled`
(`crates/hg-selfmod/src/decode.rs:17,144`). Engine untouched.

| E1 concept | `hg-selfmod` actual (`schema.rs`) | Note |
|---|---|---|
| `META_BASE` band | `RULE_ROOT=1000`, `PNODE=1003`, `GUARD=1005`, `RELABEL=1006`, `CREATE=1007`, `REDGE=1008`, `PEDGE=1009`, `DELETE=1010`; roles `HAS_*=2002..2009`, `ROLE_BASE=2000` | Numeric band differs; separateness is by store, not label. Migration to `META_BASE` deferred to `hg-meta`. |
| Homomorphic var node | Split vocabulary: `PNODE` (label constraint) + `PEDGE`/`REDGE` (endpoint refs by `K_EP0+i` property, not topology) + `DELETE`/`CREATE`/`RELABEL` nodes | AST-shaped, not homomorphic. G-R1 navigability results on this encoding do **not** transfer to E1 without re-measurement. |
| `E_SLOT_k` in edge label | `K_ARITY` + `K_EP0+i` properties on edge-nodes | Same arity-hole avoidance intent, different mechanism. |
| `MAX_ARITY = 8` | `Caps.max_arity`, default **3** (`schema.rs:75-79`) | Experiment parameter, not schema limit. Defaults: `lhs_max_nodes=4`, `lhs_max_edges=4`, `rhs_max_create=3`, `n_labels=4`. |
| Priority | `K_PRIORITY=1` on root | Same. |
| Guards (E1-G) | `GUARD` with `K_GVAR/K_GKEY/K_GOP/K_GVAL`, full op set 0=Gt,1=Lt,2=Eq,3=Lte,4=Gte,5=Ne | Complete for property guards (no quantity compares — still compiler-side). |
| Schedules (SB-R2) | Not encoded; `spec_to_compiled` never emits `schedule` | Same limit as E1, owned. |
| Edge labels (E1c) | Not encoded | Same limit as E1, owned. |
| Property effects (E1a) | Not encoded | Same limit as E1, owned. |

Property keys (rule space): `K_PRIORITY=1`, `K_NVARS=2`, `K_VAR_ID=3`,
`K_LABEL_CONSTRAINT=4`, `K_GVAR=5`, `K_GKEY=6`, `K_GOP=7`, `K_GVAL=8`,
`K_NEW_LABEL=9`, `K_CREATE_INDEX=10`, `K_CREATE_LABEL=11`, `K_ARITY=12`,
`K_EP0=20` (+i), `K_RESOURCE=200`, `K_SEQ=201`, `K_CREATION_BUDGET=202`,
`K_DEPTH=203`. World energy key `K_ENERGY=100`.

## 3. Numeric constants (traceability for fixtures)

| Constant | Value | Defined |
|---|---|---|
| `META_BASE` (target) | `0xFFF0_0000` | §1.1 |
| `MAX_ARITY` (target) | `8` | §1.1 |
| `Caps` defaults | `4,4,3,3,4` (nodes,edges,arity,creates,labels) | `schema.rs:75-79` |
| Guard ops | `0=Gt,1=Lt,2=Eq` | `schema.rs:87-99` |
| RCF domain separator | `0x5243_4631` ("RCF1") | `ere_canonical_form.md` |
| Legacy `class_hash` separator | `0x5e1f_a0d5` | `schema.rs:125` (compat only, never novelty) |
