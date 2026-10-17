# ERE Determinism Hazards D1/D2 (R0 spec, RuleSpec implementation)

**Status:** specification + implementation. ERE plan v1 §2.4 named these
hazards because both would otherwise be discovered as live bugs.

## D1 — canonical enumeration order

**Rule.** Decoded-rule order (hence `rule_id` assignment) must be a pure
function of rule *content*, never of store allocation history (slotmap slot
order, `HashMap` iteration, insertion order, slot reuse).

**Why it is load-bearing.** `Binding::canonical_key` is
`(Reverse(priority), rule_id, sorted bound nodes)`
(`crates/hg-observe/src/binding.rs:68-74`), and `resolve_conflicts` sorts by
it (`crates/hg-engine/src/conflict.rs:48-51`). At a priority tie, `rule_id`
decides the winner. Two runs that install the same rule set in different
orders but assign different ids diverge at the first tie — silently.

**As implemented** (`decode_all`, `decode.rs`): canonical order is
`(class_hash, K_SEQ)` — content hash first, install counter only to break
ties between byte-identical specs (clones). `class_hash` is **frozen**: it
ignores `distinct` (E1d) and any future metadata, so pre-extension artifacts
keep byte-identical order. Two specs differing only in distinct pairs share
a class_hash and fall back to `K_SEQ` order — deterministic per history,
arbitrary across histories; the real instrument (`rcf_hash`) separates them.
Consequences, owned:

* Distinct contents: order is fully insertion-invariant (class_hash is a
  pure function of spec content). Insertion order, `K_SEQ` base, and slot
  reuse cannot move them. Pinned by `gate_d1` parts 1–2.
* Identical contents (clones, e.g. fission children): ordered by birth
  (`K_SEQ`). Relative birth order is preserved under any permutation of the
  installation sequence, and clones are interchangeable (same spec, same
  compiled name `r{hash:016x}`), so the observable spec sequence per
  `rule_id` is still insertion-invariant. Pinned by `gate_d1` part 3.
* `children()` / `rule_roots()` sort by slotmap ffi key (allocation order).
  This is safe *only* because every consumer normalizes afterwards:
  `decode_all` re-sorts, `class_hash`/`rcf_bytes` sort outer edge lists,
  and the object step sorts bindings by `canonical_key`. No new consumer of
  `children()`/`rule_roots()` may treat their order as meaningful without
  its own normalization — review flag.
* Future `hg-meta` port: keep `(content_hash, install_counter)` shape; if
  the content hash becomes RCF (colliding rename-variants), rename-variant
  ties fall back to install order — deterministic for a fixed history,
  arbitrary across histories. Rename-variants are behaviorally equivalent,
  so this is acceptable but must be re-pinned, not assumed.

**Fixture (G-D1).** Two stores, same spec multiset, different insertion
orders + different `K_SEQ` base + a delete/re-add cycle (slot reuse):
identical `rcf_bytes` sequence per `rule_id`, and 10 `object_step`s on
identical worlds with same-seed RNGs give identical `canonical_hash`
trajectories — over a world engineered for a priority tie (two
same-priority, same-pattern rules relabeling one node to different labels).

## D2 — no RNG in the compile path

**Rule.** Decode, RCF normalization, and every tie-break inside them draw
nothing from any RNG stream. All tie-breaks are lexicographic (RCF takes the
minimal byte string; ordering sorts hashes numerically). Enforced
structurally: `decode_spec_counted`, `decode_all`, `rcf_bytes`, `rcf_hash`
take no RNG parameter — a future caller cannot silently thread one in
without changing signatures (review flag).

**Fixture (G-D2).**

* Stream invariance: fixed-seed stream, one draw, full adversarial decode +
  20 object steps, second draw — both draws equal a fresh same-seed stream's
  first two draws (zero consumption pinned). `RuleSpec` has no jitter
  encoding, so the object step also draws nothing today; if jitter-like
  effects ever land in `RuleSpec`, this test fails loudly and forces an
  explicit stream-discipline update — that failure is the test working.
* Golden RCF values (algorithm + hasher + fixture pin; update deliberately,
  never silently):

| Spec | `rcf_hash` |
|---|---|
| churnK-like (`gates::rcf_churnk_like`) | `0x9b5814caf8c1197b` |
| RCF rich base (`gates::rcf_rich_base`) | `0x731e9bcaa2ee5ecf` |

## Constants

| Constant | Value | Used by |
|---|---|---|
| `D1_STEPS` | `10` object steps, hash compared every step | `gate_d1` dynamics |
| `D2_STEPS` | `20` object steps inside the stream window | `gate_d2` |
| `D2_STREAM_SEED` | `0xD1D2` | `gate_d2` |
| `RCF_BRUTE_FORCE_MAX_VARS` | `8` | `schema.rs` (factorial cost guard) |
