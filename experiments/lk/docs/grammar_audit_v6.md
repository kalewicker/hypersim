# Grammar Audit — v6 Phase A0 (single-node LHS feasibility)

**Gate outcome: PASS.** A single-node decrement rule (`lk_tick`) is
expressible and executable with **zero engine or matcher changes**. The
countdown mechanism (§4) proceeds; the §4.3 fallback is not needed.

Evidence: `crates/hg-engine/tests/tick_cost_a0.rs` (both tests), read
against `matcher.rs`, `conflict.rs`, `parser.rs`, `effects.rs`,
`engine.rs`.

---

## Q1 — Does the LHS grammar admit a single-node pattern (no edge)?

**Yes, natively.** Two independent layers already support it:

- **Parser** (`parser.rs`): `is_creation = lhs.nodes.is_empty() &&
  lhs.edges.is_empty()`. A pattern with `nodes = ["w"]` and no edges is
  *not* creation (nodes non-empty), so it needs no `[rules.schedule]`
  and passes `validate_pattern` (one node, zero edges → no undeclared-
  variable check to fail).
- **Matcher** (`matcher.rs`): with `pattern.edges.len() == 0`,
  `match_edge` immediately delegates to `bind_free_vars`, whose
  "isolated var" branch (lines 299–308) binds the single variable to
  every live node under the label/distinctness constraints, then runs
  the property guards. This is a pre-existing path (isolated pattern
  vars), not new code.

**No minimal engine change required.** The adjacency-guided enumeration
is untouched: it only activates when a pattern edge has a bound anchor;
a no-edge pattern never enters that path. Matcher enumeration and
conflict semantics are not modified.

## Q2 — Do single-node bindings conflict under MaxParallel?

**No — all fire every step, verified against `conflict.rs`.** Each
`lk_tick` binding has `bound_nodes = [w]` for a distinct node `w`.
`resolve_conflicts` accepts a binding iff none of its bound nodes were
claimed by an earlier accepted binding; distinct single-node bindings
are pairwise node-disjoint, so **every live countdown node decrements
every step** — a genuine parallel clock. Confirmed empirically:
`a0_countdown_roundtrip_and_decrement` steps a 4-node ring at
countdown 50 and every node reads 49 after one step.

**Recorded interaction (not a blocker, feeds AG2 / §4.2):** a `lk_tick`
binding on node `w` *can* conflict with an EDGE-rule binding that claims
`w` (e.g. `lk_transport` on an edge incident to `w`), since that binding
also claims `w`. Conflict resolution then awards `w` to one rule by
priority + canonical order, so a node being transported may skip a tick
that step, lengthening its effective lifetime. This is exactly the
mechanism that §4.2's `L* = k` vs `2k/3` alternatives discriminate, and
AG2 measures it. It is the fifth appearance of the concurrency-semantics
class — noted, not assumed.

## Q3 — Can an effect decrement toward a floor without going negative?

**Yes, with the guard the design already carries.** `EffectOpDecl::
Subtract` on an `Int` is saturating `i64` subtraction (`effects.rs`),
which does not stop at 0 by itself. But `lk_tick`'s guard
`countdown > 0` (`property = 7, op = "gt", value = 0`) means the rule
only fires while countdown ≥ 1, so the last decrement is 1 → 0 and it
never fires on a 0. No separate floor clamp is needed; the guard is the
floor. The fuse guard `countdown == 0` then admits the node on the
following step (tick and fuse never contend for the same node the same
step — mutually exclusive guards on the countdown value).

## Q4 — Cost: N single-node bindings per step at 10⁴–10⁵ nodes

**Measured (`a0_tick_cost`), release build, single thread:**

| nodes | live frac | bindings | µs/step | ns/node |
|------:|----------:|---------:|--------:|--------:|
| 10,000 | 0.1 | 1,000 | 140 | 14.0 |
| 10,000 | 1.0 | 10,000 | 609 | 60.9 |
| 50,000 | 0.5 | 25,000 | 1,605 | 32.1 |
| 100,000 | 0.1 | 10,000 | 1,282 | 12.8 |
| 100,000 | 1.0 | 100,000 | 5,683 | 56.8 |

The scan is O(N) at ~13 ns/node (guard-rejected) to ~57 ns/node (guard-
passed) — the isolated-var path collects and iterates all `node_ids`
each step. **Verdict: acceptable at the EXP-L17 scale.** Those runs are
a 32² torus (1024 base nodes) grown by subdivision to E₀/(q+c)
midpoints: ~2.7k nodes at E₀=1e5, ~7.7k at E₀=4e5 — well under 10⁴. At
that scale the tick costs ~40–450 µs/step, comparable to but not
dominating a full family step (the exp_l14 timing is sub-ms/step at
~5k nodes). The §4.3 fallback (piggyback on transport's edge LHS) is
**not** triggered. (Should a future run push toward 10⁵ nodes the tick
would reach ~1–6 ms/step and a property index or the fallback would be
worth revisiting — recorded for scope, not needed now.)

## Q5 — Free property key + snapshot round-trip

**Key 7 (`countdown`, Int)** — from the unassigned 7..=63 block
(`property_registry.md`); registry updated. Round-trip verified in
`a0_countdown_roundtrip_and_decrement`: the store bincode-serializes and
deserializes with the Int countdown intact on every node, through the
exact codec `SnapshotWriter` uses. (Consistent with the v5 relaxation-IC
path, which already round-tripped Float keys 1 and 6; `PropertyValue`
Int and Float share the one serde derive.)

---

## Chosen implementation path

- New rule `rules/lk_tick.toml`: single-node LHS `["w"]`, guard
  `countdown > 0` (key 7), RHS `["w"]` with `subtract 1` on key 7.
  Appended to the loaded set (same pattern as `lk_fuse`), config-gated.
- Subdivision endows the child: `w.countdown set k` on the subdivide
  RHS, `k` from config (`countdown_k`). Injected in `load_rules` like
  the f_cap guard, so an unthrottled/legacy load is byte-unchanged.
- Fuse gate switches from `f_cap` (energy) to `countdown == 0`
  (key 7) when `gate = "countdown"`; the guard is config-injected.
- Priority: `lk_tick` above fuse, below/beside transport+subdivide —
  fixed in A2 with the AG2 lifetime-exactness gate as the arbiter of the
  exact ordering (the Q2 tick-vs-transport interaction is measured
  there, not guessed here).

No matcher enumeration or conflict-semantics change. A0 gate: **PASS**.
