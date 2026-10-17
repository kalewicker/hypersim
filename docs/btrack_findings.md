# B-track findings — what bounds the alphabet, and what the A-track could not resolve

**Substrate-track measurement. Zero biology. Characterised as physics.**

- Pre-registration: [`docs/btrack_preregistration.md`](btrack_preregistration.md), committed at
  **`6bb672d`** before any B-track cardinality existed.
- Instrument: the A-track's `hg-alphabet`, **unchanged** — proven, not asserted, by
  `tests/b0_instrument_pin.rs` (below).
- Raw output: [`runs/btrack/b1_rule_count_series.txt`](../runs/btrack/b1_rule_count_series.txt),
  [`runs/btrack/b2_growth_horizon.txt`](../runs/btrack/b2_growth_horizon.txt).

---

## The single sentence

**The alphabet is bounded by the poverty of the rule files tested, not — on this evidence — by the
architecture.** Effective alphabet size scales **LINEARLY** with rule count (b = +0.937, r² = 0.886,
3.67× across four rules, and the two horizons agree). The A-track's conclusion narrows to *"each
rule contributes few contexts"*, which is a local claim about rule files, and **the architectural
argument does not follow from it.** Making the endogenous-rule-encoding argument on the A-track's
evidence is therefore not available.

---

## B0 — instrument identity, enforced

Binding rule 2 requires the instrument unchanged, or B-track numbers cannot be compared to A-track
numbers. `tests/b0_instrument_pin.rs` re-runs A-track specs and asserts their **published** values:

| pinned | published | reproduced |
|---|---|---|
| `glider` S1 sweep | `[1, 3, 3, 3, 3]`, R\* = 1 | ✓ |
| `bounded_churn` S1 sweep | `[2, 28, 61, 95, 106]`, R\* = NONE | ✓ |
| `bounded_churn` events / edges | 39 320 / 24 | ✓ |
| **post-burn-in headline** | `\|A\|` = 12, A_eff = 3.57, 0 singletons, top-3 95.2%, 24 164 events | ✓ |

The B-track added an `Init` variant, an `event_cap` budget, a `rule_only_symbol` accessor and report
formatting; the pin proves none of them reached into the measurement. `event_cap` is separately
asserted inert when unset and exact when set.

**The pin caught one thing — a claim of mine, not a drift.** The A-track prose called `glider`'s
`A_eff = 3.00` *"the theoretical maximum evenness for its size"*. The exact value is **2.997**: the
distribution is near-uniform (top-1 mass 35.4% against 33.3%), not uniform, and "3.00" was two-
decimal display rounding. Corrected in `docs/atrack_findings.md`; nothing depends on it.

---

## B1 — alphabet scaling in rule count (the confound)

### B1.1 — the committed composition, and why the series is 1,2,3,4

Substrate fixed across every row: `CompositeChurnRing` — a property-seeded 6×6 triangle-lattice
crystal (2-edges) plus a **disjoint 24-node glider ring** (3-edges), **n = 60 flat throughout**,
`MaxParallel`, all priorities flattened to 0 (the edit `rules/wn_three.toml` already makes, for the
same reason: a standing priority difference would confound "which rule fired" with "which rule the
resolver preferred").

Nested: `{glider} ⊂ {+influx} ⊂ {+churn} ⊂ {+transfer}`. Row 3 contains `bounded_churn` entire, as
the plan required.

**The 5-rule row does not exist, and the reason is measured.** A liveness probe run *before* the
pre-registration, reporting fire counts only (mirroring `docs/policy_config.md` §2.5):

| candidate 5th rule | firings in 2000 steps | why |
|---|---|---|
| `lk_transport` | **0** | needs a 50-unit gradient; `influx` caps fill at 20 |
| `excite` | **0** | needs a label-1 node on a 2-edge; the only mark is on the 3-edge ring. Seeding one makes it live but *monotone* — it floods the crystal in ~10 steps and extinguishes `influx` and `churn` |
| `quench` | — | would remove the ring's only mark and extinguish `glider`, the series base |
| all others live | — | every one **mints nodes**, which would make row 5 a *growing* substrate while rows 1–4 are flat, so rule count would no longer be the only varying quantity |

**The shipped zoo contains exactly four rules that are simultaneously live, perpetual and
node-count-neutral on any one initial condition.** That poverty is itself a B-track datum: it is a
real limit on how far the rule-count axis can be pushed with existing rules, and it is the reason
the plan's 5-rule row was not deliverable rather than not attempted.

Two further facts the probe established, both reported rather than smoothed over:

- **`influx` alone is self-terminating** — it fills every node to the cap and quiesces at step 23.
  The minimal perpetual dynamic on this substrate is the `influx`+`churn` *pair*, which is why
  `bounded_churn` is a two-rule file. No single crystal rule could have served as the series base.
- **The two components are disjoint**, so ring and crystal rules cannot interact. Any super-linear
  composition effect must come from the three crystal-resident rules, so this design **understates**
  super-linearity and does not manufacture sub-linearity.

### B1.2 — the rows (all VALID: `A_rule_only` equals the rule count in every row)

**PRIMARY — equal event budget, 20 000 accepted rewrites per row.** `|A| ≤ n_events` is a hard
bound, so events are the binding resource; the rows differ ~17× in event rate, so fixing steps and
fixing events are different experiments.

| k | rules | `A_rule_only` | per-rule event share | \|A\| | **A_eff** | singletons | recurrent | \|A\|/n_ev | R\* |
|---|---|---|---|---|---|---|---|---|---|
| 1 | glider | 1 ✓ | glider 100% | 3 | **3.00** | 0 | 3 | 0.000 | 1 |
| 2 | +influx | 2 ✓ | glider 95.7, influx 4.3 | 16 | **3.74** | 0 | 16 | 0.001 | NONE |
| 3 | +churn | 3 ✓ | glider 8.6, influx 87.2, churn 4.2 | 66 | **7.26** | 23 | 43 | 0.003 | NONE |
| 4 | +transfer | 4 ✓ | glider 5.6, influx 87.6, churn 1.7, transfer 5.0 | 97 | **11.01** | 35 | 62 | 0.005 | NONE |

**Amendment 1 (adopted prospectively) suspends nothing here**: every row sits at
`|A|/n_events ≤ 0.005`, three orders of magnitude below the 0.5 bar. These rows are
substrate-limited, not budget-limited — unlike four of the A-track's eight.

**SECONDARY — equal step budget, 2000 steps per row.**

| k | events | \|A\| | **A_eff** | singletons | recurrent |
|---|---|---|---|---|---|
| 1 | 2 000 | 3 | 3.00 | 0 | 3 |
| 2 | 2 855 | 16 | 7.53 | 0 | 16 |
| 3 | 23 032 | 66 | 7.02 | 23 | 43 |
| 4 | 34 806 | 97 | 9.86 | 35 | 62 |

The k = 2 row is where the horizons visibly disagree — `A_eff` 3.74 at equal events versus 7.53 at
equal steps — and the per-rule shares say why: `influx` self-terminates at step 23, so at a 20 000
*event* budget it supplies 4.3% of events, while at a 2000-*step* budget it supplies 29.9%. Fixing
only one horizon would have reported one of these as *the* answer.

### B1.3 — the verdict the data selects

| series | verdict | fit |
|---|---|---|
| **PRIMARY, equal events** | **LINEAR** | b = **+0.937**, r² = 0.886, ratio **3.67×** |
| SECONDARY, equal steps | **LINEAR** | b = +0.802, r² = 0.877, ratio 3.29× |
| *diagnostic — raw \|A\|, NOT the verdict* | *SUPER-LINEAR* | *b = +2.608, r² = 0.988, ratio **32.33×*** |

**INDICATIVE, NOT DECISIVE — the series is four points.** Both horizons select the same branch, which
is the strongest thing four points can say.

The pre-committed reading of LINEAR, verbatim: *"alphabet is roughly the rule count times a small
constant context factor. The A-track's conclusion narrows to 'each rule contributes few contexts,'
which is a much weaker and more local claim, and the architectural argument does not follow."*

**Reported per binding rule 1: this weakens the A-track's headline, and that is the finding.**

### The raw-versus-effective split is the sharpest thing in this track

Raw `|A|` grows **32×** across four rules (b = +2.608, r² = 0.988 — a *better* fit than the
effective-size series). Effective size grows **3.67×**. Both are real and they say different things:

> **Rules genuinely compose to produce contexts none of them produces alone** — that is the raw
> super-linearity, and it falsifies the strong form of the fixed-alphabet reading for composed
> systems. **But the mass does not follow the count**: the new contexts are thin. Singletons appear
> only when the third and fourth rules arrive (0, 0, 23, 35), and top-3 mass falls 100% → 95.7% →
> 72.8% → 60.8%.

The pre-registration fits `A_eff` because the A-track showed raw count is the wrong number in five of
eight rows, and that choice was made before these numbers existed. It is the right choice and it is
also the *conservative* one here: the raw series would have returned SUPER-LINEAR, which is an even
stronger verdict against the A-track's reading.

---

## B2 — de-confounding non-recurrence from run budget

### B2.1 — a structural fact that bounds what this test can show

**Every growing rule in the zoo mints exactly one node per firing.** Asserted arithmetically, not
hoped for (`tests/b2_growth_horizon.rs`):

| rule | n | events | minted/event |
|---|---|---|---|
| `growth_treadmill` | 36 → 96 | 60 | **1.0000** |
| `triangle_subdivide` | 36 → 180 | 144 | **1.0000** |
| `binary_edge` | 36 → 240 | 204 | **1.0000** |
| `tetrahedral` | 4 → 44 | 40 | **1.0000** |

So `n_nodes = n_0 + n_events` identically and `n_events / n_nodes → 1`. **The plan's stated target —
a horizon where `n_events ≫ n_nodes` — is unreachable by construction, not by compute**, for every
growing rule in the zoo. The decision quantity therefore has to be `|A|/n_events` directly rather
than that proxy.

### B2.2 — the horizon ladder

`growth_treadmill`, the A-track spec exactly, varying only the step cap:

| steps | n_nodes | events | \|A\| | A_eff | singletons | **recurrent** | **\|A\|/n_events** | wall |
|---|---|---|---|---|---|---|---|---|
| 400 (A-track rung) | 436 | 400 | 400 | 400.00 | 400 | **0** | **1.0000** | 0.0 s |
| 1600 | 1 636 | 1 600 | 1 600 | 1600.00 | 1 600 | **0** | **1.0000** | 0.3 s |
| 6400 | 6 436 | 6 400 | 6 400 | 6400.00 | 6 400 | **0** | **1.0000** | 5.3 s |

A **16× horizon increase moved nothing at all** — not the ratio, not the singleton fraction, not the
recurrent count. Compute was never the constraint (5.3 s at the top rung).

### B2.3 — which pre-committed reading fires

**The third one: THE CEILING CANNOT BE ESCAPED ⟹ BUDGET-LIMITED ⟹ the branch stays OPEN rather than
rejected, and the A-track's A5 must be amended to say so.** It has been — see the amendment block in
`docs/atrack_findings.md` A5.

The plan's criticism was correct on its own terms. The A-track rejected its "growing alphabet" branch
on a recurrence column that was confounded with horizon, and the identical logic that made
amendment 1 suspend the *saturation* verdict suspends the *recurrence* verdict too.

### B2.4 — the sub-test, and why the conclusion survives anyway

The three readings above cannot separate *"the budget was too small"* from *"every event is
genuinely unique"*, and on this rule the ceiling is unescapable for the structural reason in §B2.1 —
so by the letter, B2 could only ever return BUDGET-LIMITED. That is why B2.4 was pre-registered
alongside them, before any B2 number existed:

> **A budget artifact must dilute as the budget grows. A structural property must not.**

**Result: `SCALE-INVARIANT` — span 16.0×, `|A|/n_events` 1.0000 → 1.0000, fall −0.0000.**

So the A-track's *conclusion* survives, on **a different and better argument than the one it gave**:
not "zero recurrence was observed" (which was confounded), but "a 16× budget increase changed the
ratio by zero, and a budget artifact could not have done that". This is reported as a distinct
outcome, never as one of the three pre-committed readings.

**Net position on the growing rules:** the A-track's reasoning is withdrawn; its conclusion stands on
new evidence; and the pre-committed branch status is `OPEN`, not `REJECTED`, because the letter of
B2.3 governs the branch and B2.4 is a separate instrument.

---

## B3 — `triangle_subdivide` reclassified

Done in `docs/atrack_findings.md` A5. The A5 table now has four classes, and
`triangle_subdivide` has its own:

- **definitions converge** (`tetrahedral`, `growth_treadmill`, `@noise`): `|A| = n_events` on S1, S2
  **and** S3 — including the radius-free degree signature, which does not depend on the context
  encoding at all. Non-recurrence is strongly supported.
- **definitions diverge** (`triangle_subdivide`): S1/S2 at 91% singletons, but **S3 gives 1602
  degree-signatures with zero singletons**. Its degree patterns recur perfectly; only its WL contexts
  do not. The evidence is about **the context encoding on a growing graph**, not about the dynamics.

This is the §6.1 second-witness discipline paying out, and grouping it inside a "92% singletons" cell
lost the finding.

---

## B4 — the `{{FILL}}` slot audit

**Correcting the plan's framing first, in the plan's own spirit:** the slots are an explicit,
documented convention (§0), each carries provenance, and §7 already holds a standing punch-list.
This is the `unmeasured`-vs-`zero` discipline working. The finding is narrower — how many remain
open, and which are closable.

### B4.1 — classification of all 31 hits

| class | count | which |
|---|---|---|
| **not a data slot** (the convention's own syntax and punch-list instructions) | 4 | lines 23, 27, 580, 628 |
| **(b) closable from an existing run on disk — CLOSED by this track** | 5 | 3 × `d_eff`, 2 × `regime` (§3.1) |
| **(c) requires a new run** | 21 | constructor ecology (6), EI/CE dumps (7), verifier agreement (2), hashes (2), glider fidelity (2), soak, seeds |
| **(d) obsolete** | 0 | — |
| **held, pending a resolved failure** | 1 | workspace test count (§B4.4) |

### B4.2 — what was closed, against which artifact

Resolved from `output/gui_*` runs — **artifacts on disk, never session recollection**:

| slot | filled | source |
|---|---|---|
| `triangle_subdivide` d_eff | **2.08** (w7; 1.87–2.40 over w4–7) | `gui_triangle_subdivide_42_20260709T092214` |
| `triangle_subdivide` regime | `GenerativeCandidate` → `Monoculture` | same |
| `binary_edge` d_eff | **0.855** (stable w28–31, n ≈ 44–49 k) | `gui_binary_edge_42_20260708T204034` |
| `binary_edge` regime | `GrowthTreadmill` | same |
| `growth_treadmill` d_eff | **5.20** (w7; 4.87–5.20) | `gui_growth_treadmill_42_20260708T214551` |

**Both `prediction-only` slots came in BELOW their design predictions** — `triangle_subdivide`
predicted 2.5–3.5, measured 2.08; `binary_edge` predicted 1–2, measured 0.855 and stably so at
n ≈ 49 000, so not a small-graph transient. §0's convention says a disagreement with the anchor is
itself a finding, so it is recorded as an open discrepancy in a Prediction/Result/Resolution block
rather than explained away. **What the numbers do settle** is the load-bearing discrimination claim:
0.855 versus 2.08 cleanly separates tree from sheet, and that survives both predictions being wrong.

`white_noise` d_eff and regime **stay open**: no `white_noise` run exists on disk.

### B4.3 — the naming collision, written into F3

`M(t)` is a **node-state** partition; the A-track alphabet is an **event** partition. Both bear on
linear-vs-accelerating novelty; they are not the same measurement. F3 called `M(t)` *the* Hash
Chemistry quantity, and the article is now corrected to *a*, with the distinction stated at the point
a reader would trip.

**This has a negative consequence the plan expected to go the other way.** B4.2 anticipated that the
A-track could close F3's headline pair (crystal ≈15, noise 304) and the cross-check slot. **It
cannot.** The A-track's numbers are *per-step distinct counts at WL radius 2*; F3's are *append-only
registry sizes at the causal layer's own depth*. They are close enough in magnitude to be tempting —
the A-track's `growth_treadmill` runs 15 → 16 against the `session≈15` anchor — and that resemblance
is precisely the trap. Filling a slot with a related-but-different quantity destroys the marker that
says *unmeasured*, which is worse than leaving it open. **Both slots stay open.**

### B4.4 — one slot held open, and a measurement trap found while trying to close it

The workspace test-count slot (`session=137`) looked like the easiest possible close: run
`cargo test --workspace` and read the number. **The obvious way to produce it is wrong.**

A default `cargo test --workspace` reported **481 passed, 1 failed, 27 ignored**. That is not the
workspace's test count: **cargo stops after the first failing test binary**, so every binary after
it never ran and never contributed to the total. The same command with `--no-fail-fast` reports
**496 passed, 10 failed, 36 ignored** — the default run undercounted the failures tenfold, the
passes by 15 and the ignored by 9.

> **A test count harvested from a default `cargo test` run is a number about where cargo stopped,
> not about the workspace.** It is the same shape as the `|A| ≤ n_events` ceiling: a quantity that
> silently reports the budget instead of the thing. Any future attempt to close this slot must use
> `--no-fail-fast` and must state the failure count beside the pass count.

The failures are **pre-existing and out of the substrate track's scope** — all 10 are in
`crates/hg-train` (the cognition/NN track): `c5_stage2` (1), `campaign_artifacts` (6),
`d2_readout_r2` (2), `fl1_learnable_signal` (1). `hg-train` does not depend on `hg-alphabet`, and
the B-track diff touches no file it compiles. `hg-alphabet`'s own suite is green.

The slot **stays open** regardless: a test count published while ten tests are red would remove the
marker that says *unmeasured*, which is the one thing §0's convention exists to prevent.

---

## B5 — folding the A-track into the substrate characterisation report

Done: report §2.5 (with the `M(t)` distinction), **§2.6** (the event-alphabet definition), **§3.1b**
(the measured per-rule alphabet zoo table with effective size and recurrence, the refinement sweep,
the novelty curves on both abscissae, the radius/time dissociation, the seventh guard instance, and
the `bounded_churn` connectivity-thinning byproduct — 36 nodes / 24 edges after 2400 steps).

---

## The interpretation for the zoo, independent of B1–B3

**Plateau in time and non-saturation in radius are not in tension — they answer different
questions.** Plateau in time means the system reached an **attractor**: the realised symbol set stops
growing. Non-saturation in radius means that attractor has **structure at every scale examined**:
finer resolution keeps splitting its symbols.

This is why the ALife result's form does not transfer. There, three values were the **whole channel**
— refinement had nothing to split. Here refinement keeps finding more and the realised set stops
growing anyway. **The bound is on exploration, not on resolution.** That is cleaner than the
A-track's "generalises in conclusion but not in mechanism", and it belongs in the rule zoo on its own
line.

B1 sharpens it further: exploration is bounded **per rule**, and adding rules buys proportionally
more exploration. Nothing measured here bounds the alphabet at the architecture level.

---

## What this track does not license

- **The endogenous-rule-encoding argument is not available on this evidence.** B1 resolved the
  rule-count confound *against* it: alphabet scales with rule count, so "the alphabet cannot grow
  within the current architecture" is not what was measured. Endogenous encoding remains *one* way
  to get more rules; this track gives no reason to call it the indicated one.
- **Four points are not a scaling law.** Every quotation of the B1 verdict carries
  `INDICATIVE, NOT DECISIVE`, in the code as well as the prose.
- **The disjoint-component caveat stands.** Ring and crystal rules cannot interact, so the super-
  linear raw result was measured with one hand tied — which strengthens it and weakens nothing.

## Open items

1. **Ten workspace tests are failing**, all in `crates/hg-train` (cognition/NN track):
   `a_single_rule_stage2_campaign_is_refused`; the six `campaign_artifacts` tests;
   `gliders_r2_readout_is_not_constant` and `r2s_extra_blocks_are_what_carry_the_signal_on_glider`;
   `cma_es_solves_fl1_within_thirty_generations`. Pre-existing on `v-track`, surfaced (not caused)
   by B4.4, and outside the substrate track. `hg-alphabet` is green. The test-count slot stays open
   until they do.
2. **The `d_eff` discrepancy** — two design predictions missed, recorded and undiagnosed.
3. **A 5-rule row** needs a rule the zoo does not have: live, perpetual and node-count-neutral. The
   cheapest way to extend the B1 series is to write one, and that is a rule-zoo task.
4. **`white_noise` d_eff / regime, F3's headline pair, and the F3 cross-check** all need runs that do
   not exist on disk.
