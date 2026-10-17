# ERE Rule Canonical Form (R0 spec)

**Status:** specification + implementation note. RCF is a Phase-0 item: no
novelty number is interpretable without it.

## 1. Why RCF comes first

Without a normal form modulo variable renaming and construction order, "a new
rule appeared" counts relabeled duplicates as novelty. A system that shuffles
var identities then emits smooth linear novelty growth that means nothing —
indistinguishable from the Hash Chemistry result this program tracks as a
standing challenge. Building the novelty instrument after the campaign
guarantees an uninterpretable campaign.

RCF is `canonical_hash` one level up, built the same way (sort by logical
content, deterministic hasher, no RNG).

## 2. Algorithm

### 2.1 Plan v1 §4.2 (E1 target, WL-style)

1. Extract the rule region to an in-memory `CandidateRule`.
2. Drop node identity. Vars become slots; pattern edges become
   `(side, [slot_idx…])` with slots ordered by `E_SLOT_k`.
3. WL-style refinement over the rule's own bipartite var/pattern-edge graph,
   initial colors from `(side_membership_multiset, label_constraint, degree)`.
4. Order vars by refined color, tie-break lexicographically on the sorted
   multiset of incident `(side, pat_edge_color, slot_k)` triples, then
   remaining-tie index in stable RNG-free order.
5. Emit bytes: priority, LHS edges sorted, RHS edges sorted, guards sorted.
   Hash with fixed-seed `FxHasher`.

### 2.2 As-implemented (`RuleSpec::rcf_bytes` / `rcf_hash`, `schema.rs`)

The current encoding is `RuleSpec` (not E1 regions), so RCF is implemented
over `RuleSpec` by **brute-force canonical labeling**, which is complete for
the capped sizes (`n_vars ≤ lhs_max_nodes`, default 4):

1. Enumerate every bijection `p` over `0..n_vars` in lexicographic order
   (deterministic, RNG-free; fast path for `n_vars ≤ 1`).
2. For each `p`, build the renamed candidate:
   * `var_labels`, `deletes`, `relabels`, guard `var`s: remapped by `p`.
   * `pedges`: refs remapped by `p`, **inner order preserved** (slots are
     positional — a slot permutation is a semantic change, §3).
  * `redges`: refs `< n_vars` remapped by `p`; refs `≥ n_vars` are create
    indices and are **not** remapped; inner order preserved.
  * `creates`: unchanged (index-ordered by construction).
  * `distinct` (E1d): vars remapped by `p`, each pair normalized `a < b`
    (renaming can flip the order), outer sorted, deduped.
3. Serialize each candidate in normalized form (outer edge lists sorted,
   guards sorted by `(var,key,op,val)`, deletes/relabels sorted) and keep the
   **lexicographically minimal** byte string. That minimum is `rcf_bytes`.
4. `rcf_hash = FxHasher(domain_separator 0x5243_4631 || rcf_bytes)`.
   `FxHasher::default` is deterministic (no per-process seed), same hasher
   family as `canonical_hash`.

Byte layout (all integers little-endian, lengths as u64 prefix):

```text
"RCF1" | priority i32 | n_vars u8
| var_labels (per var: 0x00 free | 0x01 + u32 label)
| pedges (count, then per edge: arity u8 + refs u8…)
| guards (count, then per guard: var u8, key u32, op u8, val tag+payload)
| deletes (count + sorted u8…)
| relabels (count + sorted (var u8, label u32)…)
| creates (count + u32…)
| redges (count, then per edge: arity u8 + refs u8…)
| distinct (E1d: count + sorted (a u8, b u8), a < b)
| effects (E2: count, then per effect IN DECLARATION ORDER: target u8,
  key u32, op u8 + op payload — order significant, never sorted)
```

`PropertyValue` payload: tag `0x01=Int(i64)` / `0x02=Float(bits u64)` /
`0x03=Bool(u8)`; `Float` hashed/compared by bits (canonical, no `==` fuzz).

### 2.3 Deviation from the plan, owned

Plan v1 specifies WL refinement, which is **incomplete** (SB-R3): it can
collide non-isomorphic rules and must under-report novelty. The brute-force
implementation is **complete** for all `n_vars` it enumerates: two specs have
equal `rcf_hash` iff they are identical up to var renaming (modulo hash
collision at 2⁻⁶⁴, negligible and symmetric). Completeness reports exact
novelty — also in the safe direction (it never inflates). The WL known-limit
case class (§3.4) is therefore **empty** for this implementation; if a future
`hg-meta` port reintroduces WL for performance at large `n_vars`, SB-R3
re-applies and must be re-stated in every novelty result.

Cost: `n_vars!` candidates per hash. At default caps (`n_vars ≤ 4`, 24
candidates) this is negligible next to one matcher call. If caps grow beyond
8 (40320 candidates), RCF must move behind a cache or back to WL + documented
incompleteness. The cap value is logged with every RCF-gated result.

## 3. Fixture set (G-R0)

Hand-built pairs over `RuleSpec` (see `gates::gate_rcf` / `rcf` tests).
Counts: **must-collide 23**, **must-distinct 20**, known-limit 0
(§2.3). Exact list in `docs/ere_canonical_form.md` appendix as implemented
in code; summary of classes:

### 3.1 Must collide (RCF-equal)

* C1 var renaming (swap, 3-cycle, reversal on 4 vars; with labels, guards,
  deletes, relabels, creates+redges carried along) — ≥8 cases.
* C2 pattern-edge outer reordering (LHS and RHS independently) — ≥2 cases.
* C3 guard/relabel/delete-list reordering — ≥3 cases.
* C4 isolated-var renaming (var in no edge, only label/guard) — ≥2 cases.
* C5 whole-region translation: same spec installed in two stores with
  different `K_SEQ` / insertion orders → equal RCF — ≥2 cases.
* C6 nested combined rename + reorder — ≥3 cases.
* C7 distinct pairs carried through renaming (E1d: reversal, 3-cycle,
  hand-checked) — 2 cases.
* C8 effects carried through renaming (E2: scale/subtract chain incl. create
  ref, hand-checked) — 1 case.

### 3.2 Must not collide (RCF-distinct)

* D1 slot permutation within one pattern edge (same labels) — ≥2 cases.
* D2 one var moved LHS-only → both-sides (add redge ref) — ≥1 case.
* D3 add one label constraint to a free var — ≥1 case.
* D4 remove one label constraint — ≥1 case.
* D5 change priority, all else equal — ≥1 case.
* D6 add one disconnected pattern edge — ≥1 case.
* D7 add one guard / change one guard op / change one guard threshold —
  ≥3 cases.
* D8 delete one var's survival status (add to `deletes`) — ≥1 case.
* D9 change one create label — ≥1 case.
* D10 distinct-pair add / change / drop (E1d) — 3 cases.
* D11 effect add / order-swap / op-change (E2 — order significant) — 3 cases.

### 3.3 Known-limit cases

None (complete implementation, §2.3). If WL returns, documented
WL-indistinguishable pairs go here, asserted to collide.

## 4. Gate G-R0

**Predicate:** every §3.1 pair RCF-equal AND every §3.2 pair RCF-distinct,
via `gate_rcf()` (`crates/hg-selfmod/src/gates.rs`). **Pass:** all hold,
no panic. **Fail:** any mismatch. **Cost:** seconds. **Blocks:** everything
(G-R1, all novelty claims, EXP-R06/R07). The legacy `class_hash` is compat
only and must never gate novelty.
