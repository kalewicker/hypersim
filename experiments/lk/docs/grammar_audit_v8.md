# Grammar Audit — v8 Phase B0 (age guard feasibility)

**Gate outcome: PASS, and cheaper than the plan budgeted for.** The
passive clock needs no new property key, no stamping effect, and no
per-step rule. The birth step is **already in the store**: every node's
creation step is written to `NodeData::generation` at birth and never
written again. v8 §1.1's preferred design order (a) — "derive the
quantity from state already present" — is available, so the round takes
it rather than order (c).

The one addition is a guard *kind* (`node_age_compares`), evaluated in
the same post-candidate filter as the existing property and degree
guards. B1 proceeds; none of the §3.4 fallbacks is needed.

Evidence: `crates/hg-engine/tests/age_grammar_b0.rs`, read against
`matcher.rs`, `conflict.rs`, `snapshot.rs`, `graph.rs`, `engine.rs`.

---

## Q1 — Can a guard compare a node property against the step counter?

**The guard evaluator already holds the step, and the birth step is
already on the node — so the comparison needs neither new plumbing nor a
new property.**

Two facts, both pre-existing:

- **The step is in scope.** Guards are evaluated against a
  `GraphSnapshot`, and `GraphSnapshot::step()` (`snapshot.rs:53`)
  forwards `HypergraphStore::step()`. No signature changes anywhere:
  `find_bindings(snapshot, rule, rule_id)` is unchanged.
- **The birth step is already recorded.** `NodeData::new(label,
  generation)` is called with `store.step()` at every creation site —
  `engine.rs:367` (RHS node creation, the only path that creates nodes
  during a run) and `generators.rs` (IC construction). The field is
  never written again by any rule; `property_effects` write the
  properties map, not fields.

So the minimal schema addition is a guard kind, not a property:

```toml
[[rules.lhs.node_age_compares]]
var   = "w"
op    = "gte"
value = 100        # (step − birth_step(w)) >= 100
```

Implemented as `NodeAgeCompare` → `CompiledAgeCompare` →
`check_age_compares`, mirroring the degree-compare guard exactly: Int
arithmetic, saturating subtraction, missing node rejects, no RNG drawn.

**Why this is the better mechanism, not merely the cheaper one.** A
`birth_step` property would have required a new effect op ("set this key
to the current step") — the first effect whose value is not a literal or
another property, and therefore new apply-path machinery with its own
canonical-hash regression. Reading a field that is already written
correctly for every node, by construction, removes that surface
entirely.

## Q2 — Does the addition touch matcher enumeration or conflict resolution?

**No, on both counts. The stop condition does not fire.**

- **Enumeration.** Candidate generation happens in `match_edge` /
  `unify_positions` / the isolated-var loop in `bind_free_vars`. The age
  guard is evaluated only at `var == pattern.n_vars`, i.e. against
  *completed* candidate bindings, alongside `check_property_guards`,
  `check_property_compares` and `check_degree_compares`. It is a filter,
  never a generator: it cannot change which candidates are visited, only
  which survive. Adjacency-guided enumeration is untouched.
- **Conflict resolution.** `resolve_conflicts_counted` consumes
  `Binding`s and reads only `canonical_key()` and the claimed node/edge
  sets. It never inspects guards. Unchanged, not even recompiled against
  a new type.

Rules that declare no `node_age_compares` get an empty vector and a
`return true` on the first line of the check — the pre-feature behaviour
byte-for-byte, the same discipline the degree and compare guards ship
with.

## Q3 — Property key reservation

**No key is claimed, and key 7 is not reused.** `countdown` (key 7)
stays exactly as v6 defined it, so archived ticked runs remain readable
and the B2 comparison arms load unchanged. `property_registry.md` records
the birth step as a **field**, not a key, so that a future author does
not "reserve" key 8 for something that was never a property.

Round-trip verified (`b0_creation_step_is_recorded_and_round_trips`): the
creation step survives the bincode serialization the relaxation-IC path
uses, and distinguishes cohorts created at different steps.

## Q4 — Semantics: absent stamps, and the snapshot-load hazard

**Hazard §8.1 does not fire.** The plan worried that `birth_step` is
absolute, so a snapshot loaded into a run whose counter restarts at 0
makes every stamped node instantly ancient. It does not restart:
`HypergraphStore::step` is a serialized field, `load_relaxation_ic`
deserializes the whole store, and the runner's own `for step in
0..max_steps` is a loop counter that never touches it — the engine
advances the store's counter with `store.set_step(store.step() + 1)`.
Birth steps and the current step therefore stay in **one frame** across a
load. No epoch offset is needed, and none is added.

**The real semantic change is the opposite of the one the plan
anticipated, and it is a loss.** Under the countdown gate, a gen-0
lattice node carried no key 7, and a missing property fails a guard — so
the fuse's countdown guard was, incidentally, a *second lattice
protection*. An age guard gives no such thing: every node has a birth
step, and IC nodes read as born at step 0, i.e. **maximally old**. They
pass the age guard at every k.

The lattice stays non-removable via lk_fuse's own guards — `generation`
(key 6) ≥ 1.0 and `degree == 2` — which are unchanged and were always
the load-bearing ones. This is asserted rather than assumed, in two
places: `b0_lattice_nodes_read_as_ancient_so_other_guards_must_protect_them`
(unit) and `bg_lattice_stays_non_removable_though_it_passes_the_age_guard`
(a full run, checking that the gen-0 population never shrinks *and* that
the generation guard is the one doing the rejecting).

## Q5 — Cost delta

**Measured, and it does not offset §5's budget: the saving is 0.7–2.1%.**

`b0_tick_cost_delta_at_l17_scale`, matched state (the same store, the
same standing population, the only difference being whether `lk_tick` is
in the ruleset), 1024-node lattice, medians over 9 interleaved repeats of
40 iterations each — release build:

| filaments | bindings (tick/none) | µs enumerate | µs resolve | µs total | saving |
|---|---|---|---|---|---|
| 100  | 310/211   | 1808 → 1802 | 28 → 21  | 1836 → 1823 | 0.7% |
| 300  | 912/615   | 2321 → 2305 | 108 → 83 | 2429 → 2388 | 1.7% |
| 1000 | 3020/2030 | 3917 → 3905 | 439 → 360 | 4356 → 4265 | 2.1% |

The direction the plan predicted holds in both halves, but the
magnitudes are lopsided. **Conflict resolution** does get materially
cheaper (18–25% of a small number) because the tick contributes ~1000
extra bindings to sort and screen. **Matcher enumeration barely moves**
(0.3%): the tick's isolated-var scan is O(N) with a trivial constant,
while `lk_transport`'s and `lk_fuse`'s multi-edge enumeration dominates
the step by two orders of magnitude. Removing ~990 bindings/step buys
~12 µs against a ~3900 µs step.

Two consequences, both recorded so later phases cannot lean on a saving
that is not there:

1. **B3's pilot must price the passive clock on its own measured
   wall-clock.** The clock removal contributes ~1–2%, not a factor. If
   EXP-L7R becomes affordable it will be because the coexistence graph is
   bounded, not because the tick is gone.
2. **This sharpens F-LK25 rather than softening it.** The instrument was
   *cheap* — under 2% of step time — and still dominated the scheduler,
   beating `lk_fuse` 34,206× and `lk_metered_subdivide` 16,962×.
   Back-action is not a cost phenomenon and cannot be screened for by
   profiling. A rule that claims bindings competes with the physics no
   matter how little work it does.

An end-to-end arm (`b0_tick_cost_delta_full_step`, 200 consecutive steps)
is also present, but it is a run-budget number and not an instrument-cost
number: the two arms' graphs diverge after the first step, by design.
The matched-state table above is the measurement of record.

## Measurement footprint of the new instrument (v8 §1.1, stated in-plan)

The age guard's binding-claim footprint is **empty**. It adds no rule, so
it enumerates nothing, claims no nodes, and appears in no conflict. This
is why BG6 is structural rather than statistical: `INSTRUMENT_RULES` (the
runner's list of rules that exist to measure) has nothing to match under
`gate = "age"`, so `contention:instrument_losses` is 0 by construction —
and is still written to `guards.csv` every window, as an explicit zero,
because a missing row and a zero row must not be the same observation.

---

## Implementation path (recorded, per the exit condition)

1. `hg-rules`: `NodeAgeCompare` (guards.rs), `Rule::lhs_node_age_compares`
   (rule.rs, `#[serde(default)]`), `RawNodeAgeCompare` +
   `[[rules.lhs.node_age_compares]]` parsing with the `deny_unknown_fields`
   discipline (parser.rs), `CompiledAgeCompare` +
   `MatchPattern::age_compares` (compiled.rs).
2. `hg-engine`: `GuardStats::age_compares` and `check_age_compares`
   (matcher.rs), wired as the last of the four post-candidate filters.
3. `hg-sim-core`: `gate = "age"` injection in `load_rules` — append one
   age guard to `lk_fuse`, append **no** tick rule, leave subdivide
   exactly as parsed; `lifetime_k` config field falling back to
   `countdown_k`; `INSTRUMENT_RULES` + the `instrument_losses` row.
4. `hg-output`: manifest gains `lifetime_k` and `clock_mechanism`
   (`"tick_rule"` | `"birth_stamp"`).
5. `hg-causal`: `micro_features_ignore_birth_step_and_age` — §3.3's
   exclusion, pinned on its own channel rather than inherited (the v6
   guarantee covered *properties*; the birth step is a field).
