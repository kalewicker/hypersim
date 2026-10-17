# D-track pre-registration — node-budget contention, and the conservation control

Plan: `substrate_contention_control_dtrack_plan.md`. Depends on A/B/C findings.
Instrument: `hg-alphabet`, **unchanged**; `tests/b0_instrument_pin.rs` passes (3/3) before any
D-track number below was produced.

Binding rules carried from the plan: instrument unchanged (1); **`recurrent` is the headline and
`A_eff` the companion**, not applied retroactively (2); do not tune to a criterion (3); firewall —
characterise bare before any alphabet comparison (4); amendment 1 prospectively (5); every new
criterion ships with a firing test (6).

---

## D1.1 — the node-demand table (MEASURED, `tests/d1_1_node_demand.rs`)

Each rule run **alone** on `CompositeChurnRing(6×6 arity 3, fill 5.0, ring 24)`, n = 60, 500 steps,
`MaxParallel`. Fire counts only; no alphabet was read. Arity is read off the compiled rule.

| rule | LHS arity | firings/step (early / mean / peak) | node-demand/step (early / peak) | demand / n (early / peak) |
|---|---|---|---|---|
| `glider` | 3 | 1.00 / 1.00 / 1 | 3.0 / 3 | 5.0% / 5.0% |
| `influx` | 1 | 17.70 / 1.77 / 59 | 17.7 / 59 | 29.5% / **98.3%** |
| `churn` | 3 | 0 / 0 / 0 | 0 / 0 | 0% / 0% |
| `transfer` | 2 | 0 / 0 / 0 | 0 / 0 | 0% / 0% |
| `swap` | 4 | 9.00 / 9.00 / 9 | 36.0 / 36 | 60.0% / 60.0% |

**Summed over the k = 4 set: peak 62 nodes/step = 103.3% of n.** Contention is **binding**.

### The two zeros are GUARD zeros, not starvation zeros

`churn` and `transfer` fire **zero times alone**. C3.4's rule applies — a zero does not explain
itself — and the mechanism is checkable without any alphabet: the substrate seeds fill at 5.0,
`churn` needs `old` at fill ≥ 20 and `transfer` needs `src` at fill > 10, and the only rule that
raises fill is `influx`. They have no candidate sites at all when alone.

Measured in their minimal live configuration (`influx` + rule), which is itself contended, so these
are **lower bounds**:

| pair | firings / 500 steps | peak/step | peak demand | / n |
|---|---|---|---|---|
| `influx` + `churn` | 254 | 12 | 36 | 60.0% |
| `influx` + `transfer` | 3747 | 18 | 36 | 60.0% |

Summed peak on the best available estimate is then 3 + 59 + 36 + 36 = **134 nodes/step = 223% of n**.

**This changes the diagnosis the plan started from.** B1's rules are not four independent competitors
for a node budget; `churn` and `transfer` are **downstream of `influx`** by guard, and `churn` feeds
back into `influx` by resetting fresh nodes to fill 0. The shares 5.6 / 87.6 / 1.7 / 5.0 have a
dependency-chain explanation as well as a contention one, and D1 cannot separate them by removing
contention alone.

---

## D1.2 — the large substrate: THE PRE-REGISTERED TARGET IS UNREACHABLE

D1.2 requires node count chosen so summed demand at k = 4 is **≤ 10% of n**, computed from D1.1's
numbers. **It cannot be met by any substrate in this family, and the reason is structural.**

`influx` has **LHS arity 1** and a guard every node of a freshly built substrate satisfies. It
proposes one binding per node — `n` bindings, all mutually node-disjoint, all accepted. Its demand is
**extensive**: it grows exactly as fast as the budget it is measured against.

Measured (`tests/d1_2_scaling_probe.rs`), `influx` alone across the family:

| mult | n | peak firings/step | peak demand / n |
|---|---|---|---|
| 1 | 60 | 59 | 98.3% |
| 2 | 240 | 239 | 99.6% |
| 3 | 540 | 539 | 99.8% |
| 4 | 960 | 959 | **99.9%** |

The fraction **rises** across a 16× span in `n`. Scaling does not relieve the node budget; it cannot,
because the quantity is scale-invariant. The same probe shows the k = 4 shares becoming **less** even
under scaling, not more: `glider` 5.4% → 0.4%, `influx` 88.3% → 93.9%.

**So the contention B1 exhibits is not node scarcity.** It is **universal coverage plus `rule_id`
order**: `influx` claims every node, and under the canonical key `(Reverse(priority), rule_id, sorted
bound nodes)` with all priorities flattened to 0, the rules with higher `rule_id` — `churn` (2) and
`transfer` (3) — get whatever is left, which on any step where `influx` has candidates is nothing.
A larger substrate cannot change that.

### D1.2a — the scaled substrate, committed anyway

**`CompositeChurnRing { w: 24, h: 24, arity: 3, fill: 5.0, ring: 384 }`, n = 960** (crystal 576 +
ring 384, preserving B1's 3 : 2 component ratio; a 16× scale-up).

It is run **despite failing its own requirement**, because "does substrate size move the verdict" is
a real question and this answers it. Its demand fraction is reported with every row, and no row from
it may be described as low-contention.

**Component disjointness carries forward unchanged**: the scaled substrate still has exactly two
components (one crystal, one ring), so B1's caveat — ring rules and crystal rules cannot interact, so
any super-linear composition effect must come from the crystal-resident rules — holds verbatim.

### D1.2b — the substituted control: `rule_id` permutation

Since scaling cannot vary contention, the control that does is the one lever that changes who wins
the tie-break while holding rules, substrate, instrument and horizons **exactly** fixed: **reverse
the `rule_id` assignment** (`transfer`, `churn`, `influx`, `glider`). The rule *set* at each k is
unchanged and the nesting is unchanged; only the arbitrary conflict-resolution order differs.

This is a **tighter** control than the plan's substrate swap, because it varies contention alone and
confounds nothing with size. C3.4 already showed reordering moves shares violently (`influx`
8242 → 315 when `swap` was promoted to `rule_id` 0). It is an addition to the plan, recorded as one.

Nesting under reversal is by *suffix*: k = 1 is `{glider}`, k = 2 is `{influx, glider}`, k = 3 is
`{churn, influx, glider}`, k = 4 is `{transfer, churn, influx, glider}` — the same sets B1 used, so
the series is still a refinement and every row is comparable to B1's.

---

## D1.3 / D1.4 — committed predictions

Written before any D-track alphabet number existed.

- **P1.** On the scaled substrate the shares get **less** even, so D1.4's third branch (*shares do not
  even out ⟹ domination is a real property of the rules, not contention*) is the one that fires —
  **with the caveat that demand never became low**, so it is weaker evidence than the plan intended.
- **P2.** `A_eff` stays **LINEAR** on both new arms. B1 measured b = +0.937 against a LINEAR band of
  0.5 ≤ b ≤ 1.5, which is wide, and C0 already showed the verdict survives three different
  share-independent recomputations.
- **P3.** `recurrent` stays **SUPER-LINEAR** on both arms (C0 measured b = +2.243, r² = 0.992).
- **P4.** Reversing `rule_id` order moves the shares substantially at k ≥ 3 — specifically `churn`
  and `transfer` gain and `influx` loses, since the starvation is an ordering effect.
- **P5.** If P2 and P3 both hold on the reversed arm, **B1's verdict is robust to the contention
  structure** and the C-track premise is secure. That is the reading; it is not assumed.

A row with `A_rule_only ≠ k` is **VOID** and reported as void (B1.2's rule), excluded from every fit.

## D1.5 — the k = 5 row

**Predicted VOID again.** `swap` binds 4 nodes and matches every 2-edge, so its demand is *also*
extensive: on the 576-node crystal it is ~576 nodes/step = 60% of n, the same fraction as at n = 36.
The blocker D1.5 expected scaling to remove by construction is **not removed**, because the blocker
was never absolute node scarcity. `swap` sits at `rule_id` 4, last, so it is cut first.

**B-track open item 3 is therefore predicted to stay open.** If the row *does* compose, the
prediction is wrong and that is reported as a miss.

---

## D2 — the conservation control and its matched treatment

### D2.1 — reframing (documentation, no new run)

`swap` is a **conservation control**, not a failed gap attempt: it establishes that when a rule
conserves exactly what the instrument reads, the alphabet is bounded and saturates completely (17
symbols by step 2000, byte-identical at 4× and 16×). `swap`'s failure is **structural**; `cyclic3`'s
is **dynamical** (attractor formation). C4 grouped them; they are not the same shape.

### D2.2 — the matched treatment, two arms

Both arms differ from `swap` in **exactly one** property, and the two isolate the two halves of the
C-track diagnosis ("WL classes are pure degree structure *because the substrate is unlabelled*").

**Arm E — `rewire`, node-count-neutral and DEGREE-CHANGING.** Route chosen: *edge-count-neutral,
degree-changing*, i.e. move one incidence from one node to another. Mechanism: match a 2-edge
`{a, b}` and a second 2-edge `{c, d}` sharing nothing, and rewrite to `{a, b}` and `{a, d}` — the
incidence at `c` moves to `a`, so `deg(a)` rises by 1 and `deg(c)` falls by 1. `n` is exactly flat,
edge count is exactly flat, the degree *sequence* is not conserved. **Zero tunable parameters**, like
`swap`. Chosen over the add/remove-pair route because a rule that adds and removes edges in the same
step has two failure modes (thinning and runaway) that would need a parameter to balance, and rule 3
forbids acquiring a parameter that would then be swept.

**Arm F — `swap` on a LABELLED substrate.** The rule is untouched; the substrate carries labels, so
WL classes are no longer pure degree structure. `LabelledCrystal { w: 6, h: 6, arity: 3, states: 3,
seed: 42, periodic: false }` — the substrate `cyclic3` was pre-registered on, reused rather than
invented so that no new free choice enters. This arm isolates the *substrate* half; arm E isolates
the *rule* half.

**Committed predictions per arm:**

| | `minted/event` | degree sequence | C3.2 verdict | parameters |
|---|---|---|---|---|
| arm E `rewire` | **0.0000** | **CHANGES** (asserted, it is the design claim) | GAP OCCUPIED | none |
| arm F `swap`@labelled | **0.0000** | preserved exactly (the rule is unchanged) | GAP OCCUPIED | none |

Both are predicted to occupy the gap, because in both the named defect in `swap`'s design has been
removed. **If either saturates instead, the architectural claim gets real support for the first
time** — that is the outcome the C-track correctly declined to claim, and it would be the more
interesting result.

Arm F carries a *known risk* recorded in advance: a labelled substrate with no rule that changes
labels means the labels are **static**, so they may enrich the alphabet by a constant factor without
making it accumulate. That would be a real answer (labelling is not the binding constraint), not a
failure of the arm.

### D2.3a — arm E FAILED the gate; the one recorded deviation (arm E′)

**Measured, `tests/d2_3_bare_characterisation.rs`.** `rewire` self-terminated: **560 firings in 8000
steps, 0 in the final 10%**. `n` and `e` were exactly flat and the degree sequence did change as
designed, but the rule is not perpetual, so **it is not a D2 rule** and gets no D2.4 row — the same
disposition `cyclic3` received at the C2 gate.

The mechanism is in the trajectory, not inferred: **max degree ran 6 → 85 within 1000 steps.** A
single node joined every one of the 85 edges, after which no two node-disjoint 2-edges exist and the
LHS cannot match. `rewire` moves incidence in one direction always, so it is a rich-get-richer
process with an absorbing state — **it eats its own precondition**.

**This generalises, and the generalisation is the finding.** At fixed `n` and fixed edge count,
"change the degree sequence" means moving incidences, and incidences are conserved; any *monotone*
direction of movement has an absorbing state. To be perpetual the direction must reverse, which
requires comparing two matched nodes' degrees — and **the rule grammar cannot express that**:
`NodeDegreeCompare` is `degree(var) <op> (value + offset)`, a comparison against a **constant**, with
no var-to-var form (`crates/hg-rules/src/guards.rs:40`). So the natural fix is unavailable, and this
is an **instrument/grammar limit**, not a fact about the substrate.

**Arm E′ — `rewire_bounded`**, the one recorded post-hoc deviation, correcting the named defect with
the only means the grammar allows. A receiver may gain only below the substrate's initial **maximum**
degree; a donor may lose only above its initial **minimum**. On `PureCrystal(6×6 arity 3)` those are
**6 and 2, read off the substrate, not chosen** — the same discipline as N0's "derive the bars from
arm A". Neither may be swept (rule 3).

*Counting argument, committed before measurement:* 36 nodes, 85 2-edges = 170 incidences, mean 4.72.
All-at-6 needs 216 and all-at-≤2 allows at most 72, so in **every** reachable state some node has
degree < 6 and some has degree > 2. Both guards are always satisfiable, which rules out the failure
mode that killed arm E. It does **not** prove perpetuality — the two edges must also be node-disjoint,
which is structural and is precisely what the gate measures.

**Committed prediction for arm E′:** passes the gate (live and perpetual, `n` and `e` flat, degree
sequence changing), and then **GAP OCCUPIED** at D2.4, for the same reason arm E was predicted to.
If it self-terminates too, the route "node-count-neutral but degree-changing" is closed by mechanism
rather than by one bad rule, and that is reported as the result.

### D2.3 — firewall gate

Per C2's template, and the gate is real — it killed `cyclic3` at 59 firings in 8000 steps. Report
`n(t)`, `e(t)`, degree sequence over time, component count, **long-run edge trajectory** (the
`bounded_churn` thinning precedent), and the liveness/perpetuality probe (fire counts only). *Gate:*
live and perpetual at flat `n`, or it is not a D2 rule.

### D2.4 — the measurement

Same instrument, same S1/S2/S3, same R = 2, same horizons as the C-track (`C_LADDER` 2000/8000 plus
the 32000 power rung) so the saturation comparison against `swap` is direct. Headline `recurrent`,
companion `A_eff` with `2^H(w)` separated.

**This track does not** revive the endogenous-rule-encoding argument, apply the `recurrent`-headline
recommendation retroactively, or resolve `cyclic3`'s knife-edge verdict.
