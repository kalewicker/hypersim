# B-track pre-registration — rule-count scaling, horizon de-confounding

**Committed before any B-track alphabet cardinality is computed.** Substrate track, zero biology.

Companion to [`docs/atrack_a1_preregistration.md`](atrack_a1_preregistration.md) (`a43ca3d`), whose
signature definitions, radius selection rule, composition formulas and growth classifier are
**inherited unchanged**. This file adds only: the composition scheme, the rule-count series, the
horizons, and the new verdict thresholds.

---

## 0. Instrument identity (binding rule 2)

The B-track compares its numbers against A-track numbers, so the instrument must be provably the
same object, not merely believed to be.

- `signature.rs`, `alphabet.rs`, `saturation.rs`, `growth.rs` are **unmodified**. S1/S2/S3, the
  `ε = 0.05` radius rule, the sorted containers, the entropy/effective-size formulas and the regime
  classifier are byte-identical to the A-track.
- What the B-track adds is confined to: a new `Init` variant (an initial condition, not a
  measurement), new `RunSpec`s, a `rule_only_symbol(rule_id)` accessor that reads existing counts,
  and report formatting. None of these can change a computed value.
- **This is enforced, not asserted.** `tests/b0_instrument_pin.rs` re-runs the A-track's
  `bounded_churn` and `glider` specs and asserts their **published numbers reproduce exactly**
  (glider `|A| = 3`, `A_eff = 3.00`, `R* = 1`; bounded_churn post-burn-in `|A| = 12`,
  `A_eff = 3.5701…`, 0 singletons, 24 164 events). Any instrument drift fails there first.
- The n = 3 process order gate is unchanged and continues to run.

## 0.1 Amendment 1, adopted prospectively (binding rule 4)

> **`R*` is reported only when `|A(R_MAX)| / n_events < 0.5`. Otherwise the run is `BUDGET-LIMITED`
> and no saturation verdict issues.**

The A-track stated this amendment and correctly declined to retro-apply it. This track adopts it
from the start, for every row it reports. It is **not** applied backwards to A-track rows.

---

## 1. B1 — the rule-count series

### 1.1 The substrate, fixed across the whole series

`CompositeChurnRing(w=6, h=6, arity=3, fill=5.0, ring=24)`: a property-seeded 6×6 triangle-lattice
crystal (2-edges) **plus a disjoint 24-node glider ring** (3-edges). **n = 60, flat by construction
for every rule in the series.** Step mode `MaxParallel`, seed 1, as in the A-track's bounded-churn
run.

**Why a composite.** `generate_crystal` emits only 2-node edges at every arity, so `glider` — the
zoo's only perpetual, node-count-neutral, single-rule dynamic — cannot fire on a crystal at all,
while the crystal rules need 2-edges the ring does not have. No single shipped initial condition
hosts a size-bounded rule-count series.

**Stated limitation:** the two components are disjoint, so ring rules and crystal rules cannot
interact. Any super-linear composition effect must therefore come from the three crystal-resident
rules; `influx` is the only rule spanning both components. A super-linear result would be
*understated* by this design, and a sub-linear one is not explained by it.

### 1.2 The series is 1, 2, 3, 4 — not 1, 2, 3, 5, and here is the measured reason

Nested by construction (each set is a subset of the next):

| count | rules | rule sources |
|---|---|---|
| **1** | `glider` | `rules/glider.toml` |
| **2** | `glider`, `influx` | + `rules/bounded_churn.toml` |
| **3** | `glider`, `influx`, `churn` | + `rules/bounded_churn.toml` — **contains `bounded_churn` entire** |
| **4** | `glider`, `influx`, `churn`, `transfer` | + `rules/conservation_soak.toml` |

The plan asked for a 5-rule row. **The shipped zoo does not contain a fifth rule that is
simultaneously live, perpetual and node-count-neutral on any one initial condition**, and this was
established by a liveness probe run *before* this pre-registration and reporting fire counts only
(`tests/b1_liveness_probe.rs`, mirroring `docs/policy_config.md` §2.5):

- `lk_transport` — **0 firings in 2000 steps**. Its guard needs a 50-unit gradient; `influx` caps
  fill at 20, so the gradient is unreachable.
- `excite` — **0 firings in 2000 steps**. Needs a label-1 node incident to a 2-edge; the only
  label-1 node is the ring's mark, and the ring has 3-edges only. Seeding a label-1 crystal node
  makes it live but *monotone*: it floods the crystal to label 1 within ~10 steps and extinguishes
  `influx` and `churn`, which is a worse defect than absence.
- `quench` — would remove the ring's only mark and extinguish `glider`, the series base.
- Every remaining live rule (`binary_edge`, `triangle_subdivide`, `growth_treadmill`,
  `wolfram_like`, `crystal_grow`, `crystal_defect`, `lk_metered_subdivide`, `inject_*`) **mints
  nodes**, which would make the 5-rule row a *growing* substrate while rows 1–4 are flat. Rule count
  would no longer be the only varying quantity, which is the one thing §1.1 must hold.

**That poverty is itself a B-track datum** and is reported as such: the constraint on how far the
rule-count axis can be pushed with existing rules is a property of the zoo.

### 1.3 Conflict resolution, committed

`MaxParallel` fires node-disjoint bindings concurrently. Conflict resolution sorts by
`(Reverse(priority), rule_id, sorted bound node ids)`, so a standing priority difference would carry
a systematic preference among rules and confound "which rule fired" with "which rule the resolver
preferred". **All priorities are flattened to 0** in the composed files — the same edit, for the
same stated reason, that `rules/wn_three.toml` already makes and documents. It is the only edit made
to any rule.

### 1.4 Horizons — both, because the binding resource is events, not steps

`|A| ≤ n_events` is a hard bound, so a row with more events can carry a larger alphabet for no
structural reason. Rows differ enormously in event rate (measured in the liveness probe: 1 event per
step at count 1, ~17 per step at count 4). Fixing steps and fixing events are therefore different
experiments and **both are run**:

- **PRIMARY — equal event budget: 20 000 accepted rewrites per row**, whichever step count that
  takes. This holds the binding resource constant, so rule count is the only varying quantity in the
  sense that matters for `|A|`.
- **SECONDARY — equal step budget: 2000 steps per row.** Reported beside it. Where the two disagree,
  the disagreement is the finding, as it was for `binary_edge` in the A-track.

A row that cannot reach 20 000 events (quiescence) is reported with the events it did reach and
flagged.

### 1.5 Measured per row (B1.2)

`|A|`, `A_eff`, singletons, **recurrent** (`|A| − singletons`), `|A|/n_events`, the full refinement
sweep, `R*` **subject to amendment 1**, and both novelty curves. Plus:

- **`A_rule_only` must equal the rule count.** If it does not, the composition did not fire all its
  rules and **the row is void** and is reported as void, not as a small alphabet.
- **per-rule event share**, so a row dominated by one rule cannot be read as a balanced composition.

### 1.6 Scaling verdicts (B1.3) — committed thresholds

Fit `A_eff` (not raw `|A|`) against rule count `k ∈ {1,2,3,4}` at the primary horizon. Let `b` be
the log–log slope of `A_eff(k)`, with `r²` reported beside it, and let
`ratio = A_eff(4) / A_eff(1)`.

| condition | verdict | consequence |
|---|---|---|
| `b < 0.5` **and** `ratio < 2.0` | **SUB-LINEAR / SATURATING** | strongly supports the fixed-alphabet reading; adding rules demonstrably does not buy alphabet |
| `0.5 ≤ b ≤ 1.5` | **LINEAR** | alphabet ≈ rule count × a small constant; the A-track's conclusion narrows to "each rule contributes few contexts" and the architectural argument does not follow |
| `b > 1.5` | **SUPER-LINEAR** | rules compose to produce contexts none produces alone; the fixed-alphabet reading is **falsified** for composed systems |

Reported with fit quality, never as a bare label. **The series is four points and will be labelled
`INDICATIVE, NOT DECISIVE` wherever the verdict is quoted.**

**Firing test (binding rule 5):** `tests/b1_firing.rs` feeds the classifier synthetic
`A_eff(k)` series with known answers — a constant series (must return SUB-LINEAR), `A_eff = 3k`
(must return LINEAR), and `A_eff = k³` (must return SUPER-LINEAR). A scaling verdict that cannot
return all three cannot support a conclusion in either direction.

---

## 2. B2 — de-confounding non-recurrence from run budget

### 2.1 The horizon ladder

`growth_treadmill` on `PureCrystal(6×6, arity 3, seed_label1)`, `MaxParallel`, seed 1 — the exact
A-track spec, varying only the step cap:

**400 (the A-track rung) → 1600 → 6400 steps.**

A rung exceeding **30 minutes** of wall clock is reported as *not reached*, with the rungs that were.

**A structural fact recorded in advance, because it bounds what this test can show:** every growing
rule in the zoo mints exactly **one node per firing**, so `n_nodes = n_0 + n_events` identically and
`n_events / n_nodes → 1`. The plan's stated target — a horizon where `n_events ≫ n_nodes` — is
therefore **unreachable by construction, not by compute**, for this rule and for every other growing
rule in the zoo. This is asserted arithmetically in `tests/b2_growth_horizon.rs` rather than hoped
for. The decision quantity is consequently `|A|/n_events` directly, not the `n_events/n_nodes` proxy.

### 2.2 Pre-committed readings (B2.3, verbatim from the plan)

- **Recurrence appears at longer horizon** ⟹ the A-track's serial-number reading is a budget
  artifact; the growing rules have genuine alphabets and A5's branch rejection is **withdrawn**.
- **Non-recurrence persists with `|A|/n_events` well below the ceiling** ⟹ the serial-number reading
  is **earned** and A5's rejection stands.
- **The ceiling cannot be escaped** ⟹ **BUDGET-LIMITED**; the branch stays **open rather than
  rejected**, and the A-track's A5 must be amended to say so.

"Well below the ceiling" is fixed here as `|A|/n_events < 0.5`, the same bar as amendment 1.

### 2.3 B2.4 — the scale-invariance sub-test (new; pre-registered here, before any B2 number)

The three readings above cannot distinguish *"the budget was too small"* from *"every event is
genuinely unique"*, and on this rule the ceiling may be unescapable for the structural reason in
§2.1. One more thing is measurable, and it separates them:

> **A budget artifact must dilute as the budget grows. A structural property must not.**

Pre-committed reading, applied to `|A|/n_events` across a ladder spanning **≥ 16×** in `n_events`:

- **`|A|/n_events` falls by more than 0.05 across the ladder** ⟹ the ceiling is a **BUDGET ARTIFACT**;
  a longer horizon would resolve it and the recurrence verdict stays suspended.
- **`|A|/n_events` stays ≥ 0.99 across the whole ladder** ⟹ the ceiling is **SCALE-INVARIANT**: not
  a budget artifact but a property of the rule, since a 16× budget increase changed nothing. The
  serial-number reading is then supported *by a different argument than the one A5 gave*, and this
  is reported as a distinct outcome from the three readings above, never as one of them.
- **Anything between** ⟹ **INDETERMINATE**, reported as such.

**Firing test (binding rule 5):** `tests/b1_firing.rs` also feeds this criterion two synthetic
ladders — one whose ratio dilutes (must return BUDGET ARTIFACT) and one pinned at 1.000 (must return
SCALE-INVARIANT). A criterion that cannot return "budget artifact" cannot earn "scale-invariant".

---

## 3. What this pre-registration does not decide

Nothing about which B1 scaling verdict or B2 reading the data will select, and nothing about the
endogenous-rule-encoding argument, which the plan forbids making before B1 resolves.

B3 (reclassify `triangle_subdivide`), B4 (`{{FILL}}` audit) and B5 (fold into the substrate report)
require no new measurement and are not pre-registered here; B3 and B4 are corrections to existing
text against existing artifacts.
