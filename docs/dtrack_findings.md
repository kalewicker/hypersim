# D-track findings — node-budget contention, and the conservation control

Pre-registration: `docs/dtrack_preregistration.md` (`2901f56`, plus the D2.3a deviation recorded
before arm E′ was measured). Instrument `hg-alphabet` **unchanged**; `tests/b0_instrument_pin.rs`
3/3 before any number here was produced. `cargo clippy -p hg-alphabet --all-targets` clean;
`hg-alphabet` suite **49 passed, 0 failed, 15 ignored**.

**The two sentences.**

**D1 — B1's rule-count verdict SURVIVES.** It survives a 16× substrate scale-up and a total
inversion of the conflict-resolution order, both of which moved the event shares violently. But it
was never tested on a substrate where the node budget is not binding, **because no such substrate
exists in this family** — the contention is scale-invariant.

**D2 — a node-count-neutral rule that does NOT conserve what the instrument reads STILL saturates.**
Its alphabet is ~10× richer than `swap`'s and just as frozen. **`swap`'s saturation was not a defect
of its design**, and the architectural claim the C-track declined to make now has real support.

---

## D1.1 — the node-demand table

`tests/d1_1_node_demand.rs`. Each rule alone on `CompositeChurnRing`, n = 60, 500 steps. Fire counts
only; arity read off the compiled rule.

| rule | LHS arity | firings/step early / mean / peak | demand/step early / peak | demand/n early / peak |
|---|---|---|---|---|
| `glider` | 3 | 1.00 / 1.00 / 1 | 3.0 / 3 | 5.0% / 5.0% |
| `influx` | 1 | 17.70 / 1.77 / 59 | 17.7 / 59 | 29.5% / **98.3%** |
| `churn` | 3 | 0 / 0 / 0 | 0 / 0 | 0% / 0% |
| `transfer` | 2 | 0 / 0 / 0 | 0 / 0 | 0% / 0% |
| `swap` | 4 | 9.00 / 9.00 / 9 | 36.0 / 36 | 60.0% / 60.0% |

**Summed peak over the k = 4 set: 62 nodes/step = 103.3% of n. Contention is BINDING.**

**The two zeros are GUARD zeros, not starvation zeros.** Fill is seeded at 5.0; `churn` needs ≥ 20
and `transfer` needs > 10; only `influx` raises fill. They have no candidate sites when alone.
Re-measured in their minimal live configuration (itself contended, so **lower bounds**):
`influx`+`churn` → 254 firings, peak demand 36 (60% of n); `influx`+`transfer` → 3747 firings, peak
demand 36 (60% of n). Best summed estimate: **134 nodes/step = 223% of n.**

**This changed the plan's premise.** The four rules are not independent competitors: `churn` and
`transfer` are **downstream of `influx`** by guard, and `churn` feeds back into `influx` by resetting
fresh nodes to fill 0. B1's shares have a dependency-chain explanation as well as a contention one,
and removing contention alone cannot separate them.

## D1.2 — the pre-registered target is UNREACHABLE

`tests/d1_2_scaling_probe.rs`. D1.2 asks for summed demand ≤ 10% of n. **No substrate in this family
can deliver it.** `influx` has LHS arity 1 and a guard every node satisfies, so it proposes `n`
mutually disjoint bindings: its demand is **extensive** and its *fraction* is scale-invariant.

| mult | n | `influx` peak firings/step | peak demand / n |
|---|---|---|---|
| 1 | 60 | 59 | 98.3% |
| 2 | 240 | 239 | 99.6% |
| 3 | 540 | 539 | 99.8% |
| 4 | 960 | 959 | **99.9%** |

The fraction **rises** across a 16× span, and the k = 4 shares get **less** even, not more
(`glider` 5.4% → 0.4%, `influx` 88.3% → 93.9%).

**So B1's contention is not node scarcity — it is universal coverage plus `rule_id` order.** `influx`
claims every node; under the canonical key `(Reverse(priority), rule_id, sorted bound nodes)` with
priorities flattened, `churn` (id 2) and `transfer` (id 3) get what is left, which on any step where
`influx` has candidates is nothing. Substrate size cannot change that.

Two arms were therefore run: **B**, the specified scaled substrate (n = 960), reported as failing its
own requirement; and **C**, a substituted control that varies contention alone — **`rule_id`
reversal**, holding rules, substrate, instrument and horizons exactly fixed. Component disjointness
carries forward unchanged (two components under scaling), so B1's caveat holds verbatim.

## D1.3 / D1.4 — the three-arm series

`tests/d1_3_contention_series.rs`, both horizons. Arm A reproduces B1 exactly (asserted).

### PRIMARY, equal events (20 000 per row)

| arm | k | \|A\| | A_eff | 2^H(w) | recur | shares |
|---|---|---|---|---|---|---|
| **A** baseline | 1–4 | 3 / 16 / 66 / 97 | 3.00 / 3.74 / 7.26 / 11.01 | 1.000–1.003 | 3 / 16 / 43 / 62 | k=4: glider 5.6 influx 87.6 churn 1.7 transfer 5.0 |
| **B** scaled | 1–4 | 3 / 16 / 59 / 72 | 3.00 / 4.15 / 5.35 / 6.00 | 1.000–1.513 | 3 / 16 / 47 / 61 | k=4: glider 0.1 influx 97.5 churn 1.9 transfer 0.4 |
| **C** reversed | 1–4 | 3 / 16 / 68 / **VOID** | 3.00 / 3.79 / 6.06 / — | 1.000–1.167 | 3 / 16 / 40 / — | k=4: transfer 92.1 **churn 0.0** influx 2.9 glider 5.1 |

### The verdicts, both horizons

| arm | headline `recurrent` | companion `A_eff` |
|---|---|---|
| A baseline (events) | SUPER-LINEAR b=+2.243 r²=0.992 | LINEAR b=+0.937 r²=0.886 |
| A baseline (steps) | SUPER-LINEAR b=+2.243 r²=0.992 | LINEAR b=+0.802 r²=0.877 |
| B scaled (events) | SUPER-LINEAR b=+2.259 r²=0.985 | LINEAR b=**+0.510** r²=0.996 |
| B scaled (steps) | SUPER-LINEAR b=+2.937 r²=0.980 | LINEAR b=+0.588 r²=0.812 |
| C reversed (events) | SUPER-LINEAR b=+2.364 r²=1.000 | LINEAR b=+0.607 r²=0.888 |
| C reversed (steps) | SUPER-LINEAR b=+2.364 r²=1.000 | LINEAR b=+0.628 r²=0.513 |

**Every verdict is unchanged on both metrics, on both horizons, on both arms.**

### Which pre-committed branch fires — and the honest answer is *none cleanly*

The four D1.4 branches all condition on "shares even out", which **never happened and could not
happen**. The outcome the data selects is closest to branch 3 (*domination is a real property of the
rules, not contention*), but branch 3's own premise — "even at low demand" — was never satisfied, so
it is weaker evidence than the plan intended. Branch 4 fires as expected: `recurrent` and `A_eff`
disagree (SUPER-LINEAR vs LINEAR) and the headline is read from `recurrent`.

What **is** established, and it is not nothing: the verdicts are invariant under a 16× size change
*and* under a complete inversion of who wins conflicts — and arm C moved the shares far more than
relieving contention would have (`influx` 87.6% → 2.9%, `transfer` 5.0% → 92.1%). **B1's verdict is
robust to the contention structure.** The C-track premise is secure.

### Two things that cut against the clean reading

1. **Arm B's `A_eff` verdict is knife-edge.** b = +0.510 against a SUB-LINEAR threshold of b < 0.5,
   and ratio = 2.00 against a threshold of ratio < 2.0. **Both clauses miss by ~1%.** The verdict is
   LINEAR as printed, but a reader should know it is one part in a hundred from flipping.
2. **Arm C's k = 4 row is VOID** — `churn` fired **0.0%**. Reversal makes `transfer` `rule_id` 0, and
   `transfer` binds 2 nodes on every 2-edge with fill > 10, so it claims the crystal and starves
   `churn` completely. This is the *same* mechanism as the forward case with a different winner, which
   is the cleanest single demonstration that the domination is an **ordering** effect. It costs arm C
   its fourth point; its fits run on 3.

**Prediction scorecard:** P1 ✓ (shares got less even), P2 ✓ (A_eff LINEAR on both arms), P3 ✓
(`recurrent` SUPER-LINEAR on both), P4 ✓ (reversal moved shares — beyond expectation, to a void row),
P5 ✓.

## D1.5 — the k = 5 row: VOID again, as predicted

`tests/d1_5_k5_on_scaled.rs`. **Two recorded reductions, both forced:** 50 steps, and `SCALE = 2`
(n = 240) rather than D1.2a's `SCALE = 4`. `swap`'s LHS is two independent 2-edges, so candidate
enumeration is quadratic in edge count; at n = 960 it is **not runnable at all** — three attempts
abandoned, the last after 10 min CPU and ~10 GB resident without completing 50 steps. No alphabet
number is read at this horizon or quoted against a B1 row.

- `swap` **alone** at n = 240: 36.0 firings/step × 4 = **144 nodes/step = 100.0% of the 144-node
  crystal** — the identical fraction it had at n = 36. The extensivity claim is confirmed to the
  decimal.
- k = 5 as composed: `swap` fired **0**. `A_rule_only` = 4. **ROW VOID.**
- Control with `swap` promoted to `rule_id` 0: `swap` 1800, `churn` **0**, `transfer` **0**,
  `influx` 1395. **Starvation runs both ways**, so this is not "`swap` is weak".

**B-track open item 3 does NOT close**, exactly as pre-registered. And there is now a *second*,
independent reason a bigger substrate was never going to close it: `swap` is computationally
intractable there.

---

## D2.1 — the reframing

`swap` is a **conservation control**, not a failed gap attempt. It establishes that when a rule
conserves exactly what the instrument reads, the alphabet is bounded and saturates completely.
`swap`'s failure is **structural**; `cyclic3`'s is **dynamical** (attractor formation). C4 grouped
them; they are not the same shape.

**Promoted zoo entry.** `swap` is simultaneously the **richest flat alphabet measured** (17
recurrent, zero singletons, `A_eff` 16.67 — ~4× anything previously flat at 1.69–4.13) **and the only
structurally non-degrading flat substrate**: 85 edges, 1 component, degree sequence and degree range
2..6 all held for 8000 steps, where `bounded_churn` had decayed to 24 edges by 2400. It is a better
downstream substrate than `bounded_churn`.

## D2.2 / D2.3 — the matched treatment, and a gate that fired again

| arm | n flat | e flat | minted/event | degree seq | predicted | events | final 10% | **gate** |
|---|---|---|---|---|---|---|---|---|
| `swap` (control) | ✓ | ✓ | 0.0000 | preserved | preserved | 72 000 | 7 200 | **PASS** |
| `rewire` (arm E) | ✓ | ✓ | 0.0000 | CHANGED | CHANGES | 560 | **0** | **FAIL** |
| `rewire_bounded` (E′) | ✓ | ✓ | 0.0000 | CHANGED | CHANGES | 56 093 | 5 600 | **PASS** |
| `swap`@labelled (arm F) | ✓ | ✓ | 0.0000 | preserved | preserved | 72 000 | 7 200 | **PASS** |

**Arm E failed the firewall gate** — 560 firings in 8000 steps, none in the final 10%. The mechanism
is in the trajectory, not inferred: **max degree ran 6 → 85 within 1000 steps.** One node joined
every edge, after which no two node-disjoint 2-edges exist. `rewire` moves incidence in one direction
always, so it is rich-get-richer with an absorbing state — **it eats its own precondition.** It gets
no D2.4 row, the same disposition `cyclic3` received at the C2 gate. This is the gate's second kill.

**The generalisation, and a grammar limit.** At fixed `n` and fixed edge count, changing the degree
sequence means moving incidences, and incidences are conserved; any *monotone* direction has an
absorbing state. Reversing direction requires comparing two matched nodes' degrees — and the rule
grammar **cannot express that**: `NodeDegreeCompare` is `degree(var) <op> (value + offset)`, against
a **constant**, with no var-to-var form (`crates/hg-rules/src/guards.rs:40`). This is an
**instrument/grammar limit**, not a fact about the substrate.

**Arm E′ — the one recorded deviation.** Bound the movement at both ends with constants **read off
the substrate**: receiver below the initial max degree (6), donor above the initial min (2). A
counting argument committed before measurement — 170 incidences over 36 nodes, all-at-6 needs 216 and
all-at-≤2 allows 72, so both guards are always satisfiable — ruled out arm E's failure mode without
proving perpetuality. **It passed:** 56 093 firings, perpetual, degree range held at exactly 2..6, one
component, `n` and `e` flat.

## D2.4 — all three arms saturate

`tests/d2_4_matched_treatment.rs`, C-track horizons 2000 / 8000 / 32000, R = 2.

| arm | \|A\| 1× / 4× / 16× | `recurrent` 1× / 4× / 16× | A_eff 1× / 4× / 16× | singletons | S3 \|A\| | C3.2 |
|---|---|---|---|---|---|---|
| control `swap` | 17 / 17 / 17 | **17 / 17 / 17** | 16.67 / 16.67 / 16.67 | 0 | 4 | PARTIAL |
| arm E′ `rewire_bounded` | 1079 / 1079 / 1079 | **172 / 172 / 172** | 171.88 / 129.60 / 118.46 | 907 | 122 | PARTIAL |
| arm F `swap`@labelled | 18 / 18 / 18 | **18 / 18 / 18** | 18.00 / 18.00 / 18.00 | 0 | 4 | PARTIAL |

**Every alphabet is byte-identical across a 16× horizon span. Nothing accumulates.**

**Both D2.2 predictions MISSED.** Both arms were predicted GAP OCCUPIED; both returned PARTIAL. Per
the pre-committed reading, that is the outcome that matters:

> *Degree-changing arm also saturates ⟹ the architectural claim gets **real support for the first
> time**, because the named defect in `swap`'s design has been removed and the result is unchanged.*

**The named defect is removed and the result is unchanged.** Arm E′ demonstrably does not conserve
what WL reads — its S3 degree alphabet is **122 symbols against the control's 4** — and it still
freezes.

### Four things the table says that the verdict alone does not

1. **Richness and accumulation are independent.** Arm E′ carries 1079 symbols and 172 recurrent —
   **10× the control** — and is exactly as frozen. A bigger alphabet is not a growing one.
2. **Labelling the substrate is nearly inert: 17 → 18 symbols, one single symbol.** Arm F isolates the
   substrate half of the C-track diagnosis, and it moves almost nothing. Between the two arms, the
   **rule-invariant half was the larger lever, and neither was binding.**
3. **A third independent witness for demoting `A_eff`.** Arm E′'s `A_eff` slides 171.88 → 129.60 →
   118.46 while `|A|` and `recurrent` are frozen — pure singleton dilution, the C0 horizon-dependence
   finding reproducing on a rule that did not exist when it was found. `recurrent` was flat on all
   three arms at all three rungs.
4. **A knife-edge in my own instrument.** C3.3 returned INSUFFICIENT LADDER for arm E′ at
   **span = 15.9×** against a committed `MIN_SPAN` of 16.0 — the event counts are not exactly
   proportional to steps because the first steps differ. The control and arm F cleared it at 16.0.
   Reported, not adjusted.

---

## What is still open

- **B-track open item 3** stays open: no fifth composable rule, for two independent reasons now.
- **`cyclic3`'s knife-edge verdict** (C-track open item 3) is untouched, as the plan required.
- **A perpetually degree-changing rule is not expressible** in the current grammar without
  substrate-derived constants. Arm E′ is a bounded proxy, and its bounds keep degrees inside the
  substrate's own initial range — which is a real limitation on how far it departs from `swap`.
- **The contention question cannot be settled by scaling** in this rule family. Settling it properly
  needs a rule set with no universally-matching arity-1 rule.
- Arm B's `A_eff` LINEAR verdict is within ~1% of flipping SUB-LINEAR on both clauses.
