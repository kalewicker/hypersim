# ERE Validators (R0 spec)

**Status:** specification. Each validator Vk has: predicate, fields read,
fixtures that exercise it, engagement counter name. Compile never errors:
failure → quarantine (counted), never panic, never silent drop.

Quarantine registry: `quarantine_by_validator: [usize; 9]`, one counter per
Vk, present on every run including zeros (a zero is a measurement). Decoder
mapping: `decode_spec` (`crates/hg-selfmod/src/decode.rs:17-139`) returns
`None` = quarantined; `decode_all` (`decode.rs:218-234`) aggregates
`decode_failures` today, per-Vk counters land with the `hg-meta` port
(step 3 of the finish plan; G-R4 gates on them).

## V1 — var-count well-formedness

* **Predicate:** `1 ≤ K_NVARS ≤ lhs_max_nodes` (default cap 4).
* **Reads:** root `K_NVARS` (`decode.rs:18-23`).
* **Fixtures:** `n_vars=0` → quarantine; `n_vars=1` minimal ok;
  `n_vars=caps+1` → quarantine.
* **Counter:** `quarantine_by_validator[V1]`.

## V2 — LHS pattern-edge well-formedness

* **Predicate:** every `PEDGE` has `2 ≤ K_ARITY ≤ max_arity` (default 3) and
  every endpoint ref `K_EP0+i` satisfies `0 ≤ ref < n_vars`.
* **Reads:** each `HAS_PEDGE` child `K_ARITY`, `K_EP0..` (`decode.rs:38-57`).
* **Fixtures:** arity-5-beyond-cap → quarantine; dangling ref (≥ n_vars,
  negative, or missing key) → quarantine; arity-3 ok.
* **Counter:** `quarantine_by_validator[V2]`.

## V3 — LHS edge-count cap

* **Predicate:** `|pedges| ≤ lhs_max_edges` (default 4).
* **Reads:** count of `HAS_PEDGE` children (`decode.rs:58-60`).
* **Fixtures:** exactly-at-cap ok; cap+1 → quarantine.
* **Counter:** `quarantine_by_validator[V3]`.

## V4 — guard well-formedness

* **Predicate:** every `GUARD` has `0 ≤ K_GVAR < n_vars`,
  `K_GOP ∈ {0,1,2}` (Gt/Lt/Eq), `K_GKEY` present, `K_GVAL` present with a
  compilable `PropertyOp`.
* **Reads:** each `HAS_GUARD` child `K_GVAR/K_GKEY/K_GOP/K_GVAL`
  (`decode.rs:62-72`).
* **Fixtures:** guard var out of range → quarantine; bad op → quarantine;
  missing val → quarantine; Int + Float guards ok.
* **Counter:** `quarantine_by_validator[V4]`.

## V5 — delete-list well-formedness

* **Predicate:** every `DELETE` has `0 ≤ K_VAR_ID < n_vars`. Post-check:
  sort + dedup (decoder normalizes; duplicates are not quarantine).
* **Reads:** each `HAS_DELETE` child `K_VAR_ID` (`decode.rs:74-83`).
* **Fixtures:** delete of var ≥ n_vars → quarantine; duplicate deletes ok
  (deduped); empty deletes ok.
* **Counter:** `quarantine_by_validator[V5]`.

## V6 — relabel + distinct-pair well-formedness

* **Predicate:** every `RELABEL` has `0 ≤ K_VAR_ID < n_vars`. Relabel of a
  deleted var is **dropped as inert** (deletion wins), not quarantine —
  recon note (a), pinned by G0b. One relabel per var survives (first in
  sorted order). Every `DISTINCT` (E1d) node has both `K_VAR_ID` and
  `K_VAR_ID2` present with `0 ≤ each < n_vars`; self-pairs (`a == b`) are
  **dropped as inert** (mirroring `compile_rule`'s own skip), not quarantine.
* **Reads:** each `HAS_RELABEL` child `K_VAR_ID/K_NEW_LABEL`
  (`decode.rs`); each `HAS_DISTINCT` child `K_VAR_ID/K_VAR_ID2`.
* **Fixtures:** relabel of var ≥ n_vars → quarantine; relabel-of-deleted →
  compiles without that relabel; duplicate relabels → one survives;
  distinct var out of range → quarantine; distinct self-pair → dropped.
* **Counter:** `quarantine_by_validator[V6]` (two adversarial arms: M6
  relabel-range, M6b distinct-range).

## V7 — create-list well-formedness

* **Predicate:** `|creates| ≤ rhs_max_create` (default 3) and
  `K_CREATE_INDEX` values are exactly `0..len` (no gaps, no duplicates).
* **Reads:** each `HAS_CREATE` child `K_CREATE_INDEX/K_CREATE_LABEL`
  (`decode.rs:101-115`).
* **Fixtures:** index gap (0,2) → quarantine; over-cap → quarantine;
  max-creates ok.
* **Counter:** `quarantine_by_validator[V7]`.

## V8 — RHS edge well-formedness

* **Predicate:** every `REDGE` has `2 ≤ arity ≤ max_arity`, every ref
  `0 ≤ ref < n_vars + |creates|`, and **no ref to a deleted var**.
* **Reads:** each `HAS_REDGE` child `K_ARITY`, `K_EP0..` (`decode.rs:117-126`).
* **Fixtures:** ref-to-deleted-var → quarantine; dangling create ref →
  quarantine; edge-only creation (no pedges + redge) ok and productive.
* **Counter:** `quarantine_by_validator[V8]`.

## V9 — compilability + non-vacuity

* **Predicate:** (a) the spec is **non-vacuous**: at least one of `pedges`,
  `deletes`, `relabels`, `creates`, `redges`, `effects` is non-empty — a rule
  with none of these can never change the store (guards and label
  constraints alone do not act) and is pure match-volume load; (b) every
  `EFFECT` (E2) node has `K_ETARGET`/`K_EKEY`/`K_EOP` present, op in 0..=7,
  refs in range, and op-required params present (source pair for
  from-variants, value for set/add/subtract, Float value for scale factor
  and jitter magnitude); (c) the decoded `RuleSpec` compiles via
  `spec_to_compiled` and `hg_rules::compile_rule` (effect targets on deleted
  vars fail here as undeclared RHS names).
* **Reads:** whole spec; each `HAS_EFFECT` child; engine `compile_rule`
  result (`decode.rs`, `decode_all`).
* **Fixtures:** vacuous rule → quarantine; effect target out of range →
  quarantine; guard-only rule → quarantine; every contentful G0/G-R0 fixture
  compiles. The engine-reject sub-case is a regression hook — reachable in
  principle (deleted-var effect targets), counter present.
* **Counter:** `quarantine_by_validator[V9]` (two adversarial arms: M9
  vacuous, M9b effect-range).

## Adversarial fixture (G-R4)

One deliberately malformed region per validator (11 cases, `gr4_malformed` in
`gates.rs`; V6 covered twice, V9 twice): M1 n_vars=0 (V1), M2
pedge-arity-over-cap (V2), M3 five pedges-over-cap (V3), M4 guard-var-range
(V4), M5 delete-var-range (V5), M6 relabel-var-range (V6), M6b
distinct-var-range (V6, E1d), M7 create-index-gap (V7), M8 redge-to-deleted
(V8), M9 vacuous rule (V9), M9b effect-target-range (V9, E2). Each must decode
to `None` incrementing exactly its own counter and no other; the aggregate
report over the adversarial store must read `total=11`,
`by_validator=[1,1,1,1,1,2,1,1,2]`; the clean seed set must read `total=0`,
`by_validator=[0;9]`.
