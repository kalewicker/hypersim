# E-track findings — the grammar hole is closed, and the qualifier comes off

Pre-registration: `docs/etrack_preregistration.md` (`17b0c02`), committed before any E-track alphabet
number existed. Instrument `hg-alphabet` **unchanged**; the extension touches `hg-rules` and
`hg-engine` only. `tests/b0_instrument_pin.rs` 3/3 with the extension compiled in.

**The single sentence.**

**A rule that changes degrees and balances itself — the one shape the grammar could not express —
saturates like everything else.** Its alphabet is byte-identical at 1×, 4× and 16×. **The
architectural claim publishes without "for rules expressible in the current grammar."** Hard stop:
the arc closes.

And it saturates for a reason sharper than the verdict: **self-balancing converges to conservation.**
`rebalance` changed the degree sequence on **10 of 8000 steps and 0 in the final 10%** — it *becomes*
the `swap` control after about ten steps.

---

## E1 — the primitive

### E1.1 — what was built, and why it is a primitive rather than a fix

`node_quantity_compares`: **var-to-var comparison of a derived node quantity**.

```toml
[[rules.lhs.node_quantity_compares]]
var_a = "c"; quantity = "degree"; op = "gt"; var_b = "a"; offset = 0   # optional
```

`quantity` is an **enum** (`NodeQuantity::Degree` today), so a second quantity is a new *variant*, not
a new guard form — evaluation, counters, TOML surface and error handling are written once for all
quantities. Only `Degree` is implemented, because only `Degree` has a use; adding unused variants to
look general would be its own kind of overfitting (plan rule 3).

Degree semantics are **inherited, not redefined**. `check_degree_compares` and
`check_quantity_compares` now call one shared `distinct_degree` helper, so the two guards cannot drift
apart on what a degree is (distinct incident hyperedges; a node appearing twice in one edge counts
that edge once).

### Where it evaluates, and what it costs

**A post-binding filter.** It cannot be evaluated until *both* vars are bound, so it runs where
`node_property_compares` runs — in `bind_free_vars`, once every variable has a node — and is
**appended last** so no existing guard's short-circuit position moves (guard counters are
position-sensitive by design).

Cost is `O(deg(a) + deg(b))` per **completed candidate**, and it **does not shrink the candidate set
enumeration builds**. That is not a footnote: it means a rule with a quadratic LHS stays quadratic no
matter how selective its guard is, which is exactly `swap`'s scaling limit (E4.2) and is why arm G is
measured on the 6×6 crystal and nowhere larger.

**Determinism:** guards are a `Vec` in declaration order; the dedup is a `SmallVec` in incidence-index
order, identical to the existing guard. No RNG, no hashing, `i64` comparison — no float
reproducibility surface of the kind the V-track's C3 finding exposed.

### E1.2 — the gate, 5/5

| check | result |
|---|---|
| **Firing test — the guard REJECTS** | On the path `x — a — b` (degrees 1, 2, 1): `degree(a) > degree(b)` evaluated 4 candidates, **kept 2, rejected 2**. The inverted `<` form rejected 2 from the other side. |
| **Partition** | `kept(>) + kept(<) == evals` exactly — the two orientations partition the evaluated set, so the guard is demonstrably reading the degrees. |
| **Offset correctness** | `a > b` and `a >= b + 1` select **identically** (2 each); `a > b + 5` rejects **everything**. So the offset is neither dropped nor applied to the wrong side. |
| **Parse errors** | An unknown quantity and an undeclared var are both **parse errors**, not guards that quietly never fire. |
| **Inertness** | 7 zoo rules measured; **none carries a quantity guard**, asserted rather than assumed. B0 pin 3/3 unchanged. |
| **Order gate** | **n = 3 processes bit-identical** on a rule that *uses* the guard (3 202 events, \|A\|@R2 = 151, `A_eff` compared on its raw bits). |

No E-track alphabet number was quoted before the inertness assertion passed.

---

## E2 — `rebalance`, and the third pre-registered outcome

**The rule:** match two node-disjoint 2-edges `{a,b}` and `{c,d}` with `degree(c) > degree(a)`; move
the incidence at `c` to `a`. Identical LHS/RHS structure to `rewire` and `rewire_bounded`; the only
difference from `rewire_bounded` is that its two substrate-derived constants are replaced by the one
comparison that expresses the mechanism. **Zero parameters**, so there was nothing to sweep.

### E2.2 — bare characterisation

| arm | n flat | e flat | degree seq | **seq-changes / 8000 (final 10%)** | events | final 10% | deg range end | gate |
|---|---|---|---|---|---|---|---|---|
| `rewire_bounded` (control) | ✓ | ✓ | CHANGED | **7998 (800)** | 56 093 | 5 600 | 2..6 | PASS |
| `rebalance` (arm G) | ✓ | ✓ | CHANGED | **10 (0)** | 64 002 | 6 400 | 3..6 | PASS |

**The third pre-registered outcome fired, and the contrast is 7998 against 10.** `rebalance` is live
and perpetual — 64 002 firings, 6 400 of them in the final tenth — but its degree sequence stops
moving after ~10 steps. Every subsequent firing merely permutes *which node holds which degree*.

This was pre-registered, with the reasoning, before it was seen: 170 incidences over 36 nodes is mean
4.72, so a perfectly regular graph is unreachable and the guard remains satisfiable by *some* pair —
but once degrees contract to a near-regular sequence, every firing swaps adjacent degrees and
**preserves the sequence**. The degree range contracted 2..6 → 3..6 and then froze.

**So the treatment collapses onto its own control.** The rule the grammar could not express turns out
to *become* a conservation rule under its own dynamics. Neither a gate kill nor a clean occupancy —
reported as its own branch, exactly as committed.

---

## E3 — the measurement: the five-mechanism table

D2.4 horizons (2000 / 8000 / 32000), R = 2, headline `recurrent`. All four arms re-measured here, so
nothing is quoted.

| arm | \|A\| 1× / 4× / 16× | **recurrent** | A_eff 1× → 16× | singletons | S3 \|A\| | byte-identical @16× | C3.2 |
|---|---|---|---|---|---|---|---|
| control `swap` | 17 / 17 / 17 | **17 / 17 / 17** | 16.67 flat | 0 | 4 | **YES** | PARTIAL |
| arm E′ `rewire_bounded` | 1079 / 1079 / 1079 | **172 / 172 / 172** | 171.88 → 118.46 | 907 | 122 | **YES** | PARTIAL |
| arm F `swap`@labelled | 18 / 18 / 18 | **18 / 18 / 18** | 18.00 flat | 0 | 4 | **YES** | PARTIAL |
| **arm G `rebalance`** | **151 / 151 / 151** | **32 / 32 / 32** | 25.80 → 24.40 | 119 | 24 | **YES** | PARTIAL |

Byte-identity is checked on the **symbol set**, not on `|A|` — N1.3's lesson is that scalars agreeing
across runs is not evidence the structure agrees. All four sets are identical across the 16× span.

**Pre-committed branch: SATURATES.** Five independent mechanisms — conservation (`swap`), attractor
formation (`cyclic3`), constant-bounded degree change (E′), substrate labelling (F), and now
self-balancing degree change (G) — all saturate. **The qualifier comes off. Hard stop; no E5/F-track.**

### What the table says beyond the verdict

1. **Richness spans 17 → 1079 symbols and changes nothing.** Four arms differing by ~60× in alphabet
   size and ~10× in recurrent count are equally frozen. This is the D-track's "richness and
   accumulation are independent" finding at a fourth point.
2. **Arm G sits between its two neighbours in exactly the way the mechanism predicts.** It carries 32
   recurrent symbols against `swap`'s 17 and E′'s 172 — richer than the pure conservation control
   because of its ~10-step transient, poorer than the constant-bounded rule because it *stops*
   changing the sequence. The alphabet size is a fossil of a transient.
3. **A fourth witness for demoting `A_eff`.** Arm G's `A_eff` slides 25.80 → 24.40 on a **frozen**
   symbol set, the same singleton-dilution artefact seen in `cyclic3` and arm E′. `recurrent` was flat
   on every arm at every rung.
4. **A knife-edge in my own instrument, again.** C3.3 returned INSUFFICIENT LADDER for arm G at
   span = 16.0× — it needs `span >= MIN_SPAN` and the ratio lands exactly on the boundary in floating
   point for some arms and just under (15.9×) for E′. Reported, not adjusted.

---

## E4 — the two record items

**E4.1 — vacuous pre-registration has a third shape.** Two instances were on record, both **per-clause**
(a truncation criterion that removed the phenomenon; a correlation clause whose variable could not
vary). The D-track produced a new one: D1.4's four branches were each *individually satisfiable*, but
all four shared the premise "shares even out at low demand", which `influx`'s arity-1 universal match
makes **structurally unreachable** at any scale (demand fraction *rises* 98.3% → 99.9% across 16×).
**Adopted prospectively: a per-clause vacuity check is insufficient; a premise shared across branches
must itself be checked for satisfiability, and measured first if it is a quantity.** Folded into the
report as Finding F4.

**E4.2 — `swap`'s intractability is a zoo property, not a run failure.** Two independent 2-edges in
the LHS ⟹ candidate enumeration quadratic in edge count: ~7 000 pairs/step at 6×6 versus ~2.7 M at
24×24; not runnable at n = 960 (10 min CPU, ~10 GB, no 50 steps). Recorded in the zoo entry as a
**scaling limit**, because it bounds what multi-node-LHS rules the zoo can hold at scale — and it
interacts directly with E1.1: a post-binding filter shrinks what enumeration *produced*, never what
enumeration *builds*.

---

## What this track deliberately did not do

No E5, no F-track (rule 6, hard stop). No retroactive application of the `recurrent`-as-headline
recommendation. No tolerance band to dodge the E2.3 outcome. `cyclic3`'s knife-edge verdict remains
untouched. The `rebalance` rule acquired no constants when the third outcome appeared — per rule 4,
that outcome is **a finding about the mechanism**, and it is reported as one.

## Still open

- Arm G's transient is ~10 steps on *this* substrate; whether a substrate whose incidence count makes
  near-regularity harder to reach would extend it is untested, and the quadratic LHS makes larger
  substrates expensive.
- The `NodeQuantity` enum has one variant. A second quantity (component size, incident-label count)
  would be a different lever on the same question and is not explored.
- C3.3's `MIN_SPAN` boundary now sits exactly on the measured span for several arms.
