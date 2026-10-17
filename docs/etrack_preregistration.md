# E-track pre-registration — closing the grammar hole

Plan: `substrate_grammar_etrack_plan.md`. Depends on A/B/C/D findings. Instrument `hg-alphabet`
**unchanged**; the extension touches `hg-rules` and `hg-engine` only. `tests/b0_instrument_pin.rs`
must pass 3/3 before any E-track number.

**Hard stop (plan rule 6):** if the new arm saturates, the arc closes. No E5/F-track.

---

## E1.1 — the primitive, specified before implementation

### What it is

A **var-to-var comparison of a node quantity** — deliberately *not* a degree-difference guard for one
rule (plan rule 3). The shape mirrors the existing `NodePropertyCompare`, which already compares
`var_a[key_a] <op> var_b[key_b] (+ offset)` across two bound vars; this is the same idea with the
compared quantity drawn from a small enumeration of *derived* node quantities rather than from the
property map.

```toml
[[rules.lhs.node_quantity_compares]]
var_a    = "a"
quantity = "degree"
op       = "gt"
var_b    = "c"
offset   = 0          # optional: var_a.q <op> (var_b.q + offset)
```

```rust
enum NodeQuantity { Degree }               // extension point: one variant today

struct NodeQuantityCompare {
    var_a: String,
    quantity: NodeQuantity,
    op: PropertyOp,
    var_b: String,
    offset: Option<i64>,
}
```

**Generality.** `quantity` is an enum, so a second quantity (incident-edge-label count, component
size, …) is a new *variant*, not a new guard form. The guard's evaluation, counters, TOML surface and
error handling are written once for all quantities. Only `Degree` is implemented here, because only
`Degree` has a use; adding an unused variant to look general would be its own form of tuning.

**Degree semantics are inherited verbatim, not redefined:** count of **DISTINCT incident hyperedges**
(a node appearing twice in one edge counts that edge once), exactly as `NodeDegreeCompare` pins it.
The two guards must never disagree about what a degree is, so they share the definition.

**Offset** is `i64` and added to **side B**, saturating — the same discipline as
`NodeDegreeCompare` and `NodePropertyCompare`. This is what makes strict and non-strict forms both
expressible (`degree(a) > degree(b)`, `degree(a) >= degree(b) + 1`).

### Where it evaluates, and what that costs

**It is a post-binding filter.** The existing var-to-constant degree guard could in principle prune
during enumeration; a var-to-var guard cannot be evaluated until *both* vars are bound, so it runs at
the same point as `check_property_compares` — in `bind_free_vars`, once `var == pattern.n_vars` and
every variable has a node. Order at that point is: property guards → property compares → degree
compares → **quantity compares**, appended last so no existing guard's short-circuit position moves
(guard counters are position-sensitive by design).

**Cost.** `O(deg(a) + deg(b))` per *completed candidate binding*, from the `SmallVec` dedup that
enforces distinct-edge semantics. It does **not** reduce the candidate set that enumeration builds,
so a rule with a quadratic LHS stays quadratic and merely filters more of what it enumerated. That
interacts directly with E4.2: `swap`'s two-independent-2-edge LHS enumerates ~edge² candidates per
step and is already intractable at n = 960. **A self-balancing rule with the same LHS shape inherits
that limit**, so the E2 rule is measured on the 6×6 crystal at the D2.4 horizons and nowhere larger.

### Determinism

No new iteration over unordered containers. Guards are a `Vec` in **declaration order**; the dedup
uses a `SmallVec` in incidence-index order, identical to `check_degree_compares`. No RNG is drawn. No
hashing, no float arithmetic — the comparison is `i64`, so there is no ulp-level reproducibility
surface of the kind the V-track's C3 finding exposed.

### Failure semantics

Mirrors the guard discipline exactly: an unbound var, a var not in the LHS (parse error), or a node
missing from the snapshot ⟹ **guard false**, never a panic and never a coercion.

## E1.2 — the gate, committed

1. **Firing test** — a case where the guard **rejects**, not merely one where it passes. Eight
   documented instances of guards passing for the wrong reason.
2. **Inertness, asserted not assumed** — every rule in the zoo produces **byte-identical** output with
   the extension compiled in. This is the `event_cap` precedent from B0: a feature that is supposed to
   be inert when unused must be shown inert.
3. **Order gate** — n = 3 processes, bit-identical, on a rule that *uses* the new guard.

*No E2 rule is measured until all three hold. No E-track alphabet number may be quoted before (2).*

---

## E2.1 — the rule, and the committed predictions

**`rebalance`** — the rule arm E′ approximated with substrate-derived constants, now written natively:

> Match two node-disjoint 2-edges `{a, b}` and `{c, d}` with `degree(c) > degree(a)`. Move the
> incidence at `c` to `a`: rewrite to `{a, b}` and `{a, d}`.

So `deg(a) += 1` and `deg(c) -= 1`, and the move only ever runs from the **higher-degree** node to the
**lower-degree** one. Identical LHS/RHS structure to `rewire` and `rewire_bounded`; the *only*
difference from `rewire_bounded` is that the two substrate-derived constants (receiver < 6, donor > 2)
are replaced by the one comparison that expresses the mechanism directly. **Zero constants**, so
there is nothing to sweep (plan rule 4).

### Committed predictions

| claim | prediction |
|---|---|
| `minted/event` | **0.0000 exactly** |
| `n(t)`, `e(t)` | both exactly flat (36 / 85) |
| degree sequence | **CHANGES** — asserted; it is the design claim |
| arm-E absorbing state (one node joins every edge) | **cannot occur** — the direction reverses the moment the comparison flips |
| degree **range** | **CONTRACTS toward uniformity.** Every firing moves a unit from a higher-degree node to a strictly lower-degree one, which is a majorisation-decreasing step. The range cannot widen. |
| liveness / perpetuality | **This is the risk, and I am naming it as the likely outcome rather than the unlikely one.** See below. |
| C3.2 verdict | **PARTIAL**, i.e. saturating — *if* it survives the gate |

### The E2.3 failure mode, and why I expect it

170 incidences over 36 nodes is mean degree 4.72, which is **not an integer**, so a perfectly regular
graph is unreachable and the guard `degree(c) > degree(a)` can always be satisfied *by some pair of
nodes*. That is a counting argument, and it is the same shape as the one committed for arm E′.

**But it is weaker than it looks, and the difference is the whole risk.** The guard must be satisfied
by a pair that also sits on two node-disjoint 2-edges with the right orientation. Once degrees have
contracted to `{4, 5}`, every firing swaps a 4 and a 5 — which **preserves the degree sequence**, the
very thing that separates this arm from `swap`. So the plausible outcome is not a clean quiesce but
something subtler: **the rule stays live while becoming asymptotically sequence-preserving**, and the
treatment collapses onto its own control.

That is neither a gate kill nor a clean occupancy, and it is pre-registered here so it cannot be
discovered and then described as expected. **If it happens it is reported as a third outcome**, and
the honest reading is that self-balancing degree dynamics converge to conservation — which would make
the structural claim *stronger*, not weaker, exactly as the plan says of the uniformity kill.

Per plan rule 4 and the forbidden-response list: if the rule needs a constant to stay live, **that is
a finding about the mechanism, not a parameter to search**, and no tolerance band will be added to
avoid a uniformity kill.

## E2.2 — bare characterisation (firewall gate)

Per the D2.3 template, and the gate is real — it has killed two arms (`cyclic3` at 59 firings,
`rewire` at 560). Report `n(t)`, `e(t)`, **degree sequence and its RANGE over time**, component count,
long-run edge trajectory (the `bounded_churn` thinning precedent), and the liveness/perpetuality probe
reporting fire counts only, with a final-10% window.

*Gate:* live and perpetual at flat `n`, or the arm is a reported gate kill and gets no E3 row.

## E3 — the measurement

Same instrument, same S1/S2/S3, same R = 2, **the D2.4 horizons (2000 / 8000 / 32000)** so the
comparison against `swap`, arm E′ and arm F is direct. Headline `recurrent`; companion `A_eff`.
Amendment 1 applies. The alphabet is reported at all three rungs so byte-identity across 16× is
directly checkable.

### Pre-committed readings

- **Saturates** ⟹ **the qualifier comes off.** Five independent mechanisms, including the one the
  grammar previously forbade, all saturate. **Hard stop; the arc closes.**
- **Accumulates** ⟹ **the D-track's architectural claim is falsified** and rested on a grammar
  limitation. Reported at full strength.
- **Gate kill by degree uniformity** ⟹ reported as a gate kill, with the stronger structural claim:
  both monotone *and* self-balancing degree dynamics have absorbing states at fixed `n` and `e`.
- **Live but asymptotically sequence-preserving** (the third outcome above) ⟹ reported as its own
  branch, with the degree-range trajectory as the evidence.
