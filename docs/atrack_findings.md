# A-track findings — the substrate's realised event alphabet

**Substrate-track measurement, characterised as physics, zero biology.** Belongs in the substrate
characterisation report and the rule zoo, beside the existing regime classifications.

- Pre-registration: [`docs/atrack_a1_preregistration.md`](atrack_a1_preregistration.md), committed
  at **`a43ca3d`**, before any alphabet cardinality was computed.
- Instrument: `crates/hg-alphabet` (depends on `hg-core`, `hg-rules`, `hg-engine`, `hg-observe`; the
  firewall is asserted, with a firing test, in `tests/a0_firewall.rs`).
- Harnesses: `a0_firewall.rs`, `a1_definitions.rs`, `a2_1_synthetic_firing.rs`,
  `a2_2_calibration_pair.rs`, `a_order_gate.rs`, `a3_a4_zoo_measurement.rs`.

---

## The single sentence

**The substrate's realised event alphabet is small and closed wherever the substrate's size is
bounded — 1.7 to 4.1 effective symbols, plateauing novelty — while the rules that appear to have
unbounded alphabets buy their symbol count entirely in symbols that never recur. A fixed symbol set
bounds what this substrate can build; growing the alphabet is the architectural question.** The
ALife three-symbol result generalises in its *conclusion* but not in its *mechanism*: on the
substrate, refinement never stops splitting symbols, and the alphabet is small anyway. Full statement
in A5.

---

## A0 — what existing data already answers, and the scoped gap

### A0.1 — WL class count over time for each builtin

**The project already records a quantity with this name, and it is a different object.**

`wl_class_count` = `M(t)` (`docs/metric_formulas.md` §238, `crates/hg-sim-core/src/runner.rs:2067`,
which fills it from `TpmAccumulator::class_count()`) is the **node-state** WL class registry: a
partition of the *live node set*. The substrate characterisation report (§2.5, Finding F3) already
names it "the Hash Chemistry linear-vs-accelerating novelty-growth quantity". The A-track's
quantity is the **event** alphabet — distinct *(rule, local context)* signatures — which is not a
partition of nodes and is not derivable from one.

**Coverage on disk.** 20+ runs carry a populated `wl_class_count` column. Every one of them is an
LK/leakage-track run (`exp_l7r`, `exp_l9`, `exp_l9b`, `exp_l10`, `exp_l11`, `exp_l12`,
`q0_retrodiction`) on `lk_*` rulesets, and every one is **flat at 2–4 classes for the entire run**
— e.g. `exp_l7r/*` at 2 for all 24 windows, `exp_l10/g1_mouth_s42` at 4 for all 25. **No run on
disk carries `wl_class_count` for any canonical builtin** (`triangle_subdivide`, `binary_edge`,
`tetrahedral`, `glider`, `growth_treadmill`) **or for the bounded-churn substrate.**

**The two headline figures in the report are unverified slots.** "Crystal saturates at ≈15 classes,
noise climbs to 304" are `{{FILL: … | session≈15 | src: run csv}}` placeholders, explicitly marked
pending verification against a run's `metrics.csv` / `ei_dump.json`. They are session recollections,
not data read off disk.

So A0.1 is answered *partially, for a different object, on rules that are not the zoo, with its two
headline numbers unverified.* That is not a free answer to the alphabet question, and it is also not
nothing: the flat 2–4 on LK substrates is a genuine prior that node-side class counts saturate hard.

**Filled for free by this track.** Because the harness computes WL class maps at every step anyway,
the A0.1 gap is closed as a byproduct: the node-WL-class count `M(t)` at R = 2 is reported per step
for every rule in the zoo, in the run blocks below, and labelled as the node-state quantity so the
two objects are never conflated.

### A0.2 — what existing data does not cover (the scoped gap)

1. **The event alphabet, at all.** Nothing on disk counts *(rule, context)* signatures. The novelty
   archive (`docs/metric_formulas.md` §Novelty archive) counts *type-bin* signatures — node WL
   hashes again, the same node-side object.
2. **Any node-class trajectory for the canonical builtins**, verified against a run.
3. **Any refinement sweep in radius.** Every existing WL number in the project is at one fixed
   depth. There is no existing evidence about whether *any* of these quantities saturate under
   refinement — which is the A-track's central instrument question and the ALife result's actual
   form ("3 → 121 codes split zero classes").
4. **Composition of any novelty measure.** `new_signatures` counts first-sightings; nothing records
   the frequency distribution, so nothing on disk can say whether a large count is a large alphabet
   or a handful of symbols plus a long tail of singletons.
5. **The bounded-churn substrate as a zoo entry.** It existed only inside test harnesses
   (`crates/hg-sim-core/tests/h1_churn_substrate.rs`, `tests/n_shared/churn.toml`). This track
   promotes it to `rules/bounded_churn.toml`, which is where the plan says its characterisation
   belongs.

**Gate:** A1–A4 proceed, scoped to items 1–5.

---

## A1 — the committed definitions

Full text in the pre-registration; summarised here so the findings are readable alone.

An **event** is one accepted rewrite. `G_t` is the graph at the *start* of step `t` — which is
exactly what `Engine::collect_candidates` matched every binding against, so "pre-rewrite graph" has
an unambiguous referent rather than a chosen one.

| variant | definition | role |
|---|---|---|
| **S1 context** | `H(rule_id, [wl_R(v_i; G_t)]` in LHS-var order`)` | **primary** — reads only the state the matcher saw |
| **S2 structural-delta** | S1's input, then the sorted post-state classes of survivors and of created nodes | cheaper variant |
| **S3 degree** | `H(rule_id, sorted[(deg_pre, deg_post)]` over touched nodes`)` | cheaper variant, radius-free |
| `A_rule_only` | `H(rule_id)` | **the vacuous definition**, computed only as the contrast |

Radius selection rule, committed at `a43ca3d`: **R\* = smallest R ∈ {1,2,3} with
`g(R) = (|A(R+1)|−|A(R)|)/|A(R)| < 0.05`**; if none exists, the alphabet *does not saturate within
the swept range* and the rule has rejected. R = 0 is the no-context baseline and is not eligible;
R = 4 is unreachable because verifying saturation at R needs R+1 measured.

### Invariants asserted, not assumed (`tests/a1_definitions.rs`)

- the progressive WL sweep is **bit-identical** to calling the shipped `compute_wl_class_map` once
  per radius (the sweep is an optimisation, and an optimisation that moves a published number is
  not one);
- `|A(R)|` is **non-decreasing in R** — a violation is an instrument fault, not a finding;
- `|A_S2(R)| ≥ |A_S1(R)|`, since S2 hashes S1's whole input as a prefix;
- `1 ≤ |A| ≤ n_events`, `1 ≤ A_eff ≤ |A|`, `|A_S1(R)| ≥ A_rule_only`.

---

## A2 — calibration

### A2.1 — the synthetic firing test: the criteria can reject

| construction | required verdict | result |
|---|---|---|
| `\|A(R)\| = 6` flat | saturates, R\* = 1 | ✓ |
| **`\|A(R)\| = 3·2^R`** | **no R\* — REJECTS** | ✓ **fires** |
| `[4, 40, 41, 41, 41]` | R\* = 1 (R = 0's large gain must not fool it) | ✓ |
| `[2, 8, 32, 128, 512]` | no R\*, and never R\* = 4 | ✓ |
| constant novelty curve | `PLATEAU` | ✓ |
| one new symbol per 10 events | `LINEAR`, r² > 0.99 | ✓ |
| **`N(e) ∝ e²`** | **`ACCELERATING`** | ✓ **fires** |
| 6 events | `INSUFFICIENT` (refuses to guess) | ✓ |

Both criteria have a case where they reject. A saturation criterion that cannot reject cannot
support a verdict, and a growth classifier that can only say PLATEAU is measuring its own
definition.

### A2.2 — the calibration gate: known-poor vs known-rich

Committed bar: `|A_S1(R)|` for `triangle_subdivide` ≥ **4×** `glider`'s at **every** R ∈ {1,2,3,4},
and different growth-regime labels.

| R | `glider` \|A\| | `triangle_subdivide` \|A\| | ratio | bar (4×) |
|---|---|---|---|---|
| 1 | 3 | 3200 | 1067× | pass |
| 2 | 3 | 4400 | 1467× | pass |
| 3 | 3 | 4400 | 1467× | pass |
| 4 | 3 | 4799 | 1600× | pass |

Regimes: `glider` **PLATEAU** (b = +0.000, r² = 1.000), `triangle_subdivide` **LINEAR**
(b = +0.991, r²_loglog = 1.000). **GATE: PASS**, by three orders of magnitude. The instrument
measures context diversity, not its own definition.

The trap is also closed empirically rather than by assertion: `A_rule_only = 1` for
`triangle_subdivide` (one rule in the file, as the vacuous definition must give), against a realised
event space of 4400. A measurement that could not tell those apart would not be evidence.

### Reproducibility order gate — n = 3 processes (`tests/a_order_gate.rs`)

All of `card`, `H_sorted`, `symdigest` (the *sorted symbol list*, per N1.3: scalars agreeing is not
evidence the structure agrees), the growth exponent `b`, and `n_events` are **bit-identical across
three separate OS processes**, 30 fields per variant-radius.

The raw diagnostic **fires**: the same entropies summed over a std `HashMap` (per-process
`RandomState` seed, non-associative float addition) **disagree across processes**. So the C3 hazard
is live in this environment, and this crate's reproducibility is a consequence of every container
being a `BTreeMap` or sorted `Vec` — not of luck.

### A defect Rule 2's reasoning caught during construction

The growth classifier reported `r²_loglog = 0.000` for a *perfect* plateau. Cause: a zero-variance
test written as `Σ(y − ȳ)² == 0` leaks a ~1e−28 denominator, because the floating-point mean of `n`
identical values is not bit-identical to that value (`Σ ln 3 / 61 ≠ ln 3`) — while the same curve in
linear space *was* exact (`183.0/61 = 3.0`), so the defect appeared in one space and not the other.
A fit quality of 0 on a flawless fit is mechanism-impossible, which is what made it visible.
Constancy is now detected from the data (`y_max == y_min`).

---

## A3 — the measurement

Raw output archived at [`runs/atrack/a3_a4_zoo_measurement.txt`](../runs/atrack/a3_a4_zoo_measurement.txt).
S1 (primary) at each rule's own pre-registered radius:

| rule | R\* | \|A\| | A_eff | singletons | **recurrent** | \|A\|/n_events | n_events | stop |
|---|---|---|---|---|---|---|---|---|
| `glider` | 1 | **3** | **3.00** | 0 | 3 | 0.002 | 2000 | step cap |
| `triangle_subdivide` | 2 ⚠dip | 4400 | 4276.3 | 4000 | 400 | **0.917** | 4800 | step cap |
| `binary_edge` | none | **56** | **1.69** | 0 | 56 | 0.002 | 27592 | node cap |
| `tetrahedral` | 1 ⚠ceiling | 120 | 120.0 | 120 | **0** | **1.000** | 120 | step cap |
| `growth_treadmill` | 1 ⚠ceiling | 400 | 400.0 | 400 | **0** | **1.000** | 400 | step cap |
| `growth_treadmill@noise` | 1 ⚠ceiling | 3200 | 3200.0 | 3200 | **0** | **1.000** | 3200 | step cap |
| `bounded_churn` (whole run) | none | **61** | **4.13** | 28 | 33 | 0.002 | 39320 | step cap |
| `bounded_churn` (post-burn-in) | none | **12** | **3.57** | 0 | 12 | 0.000 | 24164 | step cap |

`recurrent` = `|A| − singletons`, read directly off the pre-registered singleton column (§3 of the
pre-registration commits it precisely because "an alphabet whose symbols are mostly singletons is a
different object from one whose symbols carry realised mass").

`A_rule_only` — the vacuous definition — is **1** for every single-rule set and **2** for
`bounded_churn`. It cannot come back large, which is why it is the contrast and not the measurement.

### A3.2 — the refinement sweep, and where it saturates

| rule | \|A(0)\| | \|A(1)\| | \|A(2)\| | \|A(3)\| | \|A(4)\| | verdict |
|---|---|---|---|---|---|---|
| `glider` | 1 | 3 | 3 | 3 | 3 | saturates at R = 1, exactly |
| `triangle_subdivide` | 1 | 3200 | 4400 | 4400 | 4799 | R\* = 2 is a **dip**: refinement resumes, g(3) = 0.091 |
| `binary_edge` | 1 | 10 | 56 | 116 | 179 | **never saturates** (g = 4.60 / 1.07 / 0.54) |
| `tetrahedral` | 1 | 120 | 120 | 120 | 120 | flat at the ceiling, not at a plateau |
| `growth_treadmill` | 1 | 400 | 400 | 400 | 400 | flat at the ceiling |
| `growth_treadmill@noise` | 1 | 3200 | 3200 | 3200 | 3200 | flat at the ceiling |
| `bounded_churn` | 2 | 28 | 61 | 95 | 106 | **never saturates** (g = 1.18 / 0.56 / 0.12) |
| `bounded_churn` post-burn-in | 2 | 5 | 12 | 19 | 23 | **never saturates** (g = 1.40 / 0.58 / 0.21) |

The ALife analogue of "3 → 121 codes split zero classes" **does not reproduce on the substrate.**
Refinement keeps splitting symbols on the two substrates where the measurement is not budget-limited
(`binary_edge`, `bounded_churn`). This is the sharpest single structural difference from the ALife
result, and it is a negative for the naive form of the hypothesis.

### A3.3 — composition: the raw count is the wrong number in five of eight rows

- `binary_edge`: **56 raw symbols, effective size 1.69**, one symbol carrying **90.7%** of all
  events. This is precisely the case the plan named — "a 500-symbol alphabet with 99% mass on three
  symbols is a three-symbol alphabet in practice". Reporting 56 alone would have been misleading.
- `bounded_churn` post-burn-in: 12 raw, **A_eff 3.57**, top-1 45.2%, top-3 **95.2%**, **zero
  singletons**. A small, fully recurrent alphabet with realised mass in every symbol.
- `glider`: 3 raw, **A_eff 3.00**, zero singletons — a genuine three-symbol alphabet. *(B-track
  correction: the exact value is **2.997**, and "at the theoretical maximum evenness for its size",
  as this line originally read, was an artifact of printing A_eff at two decimal places. The
  distribution is near-uniform — top-1 mass 35.4% against 33.3% for a perfect three-way split — not
  uniform. Nothing else in the A-track depends on it.)*
- `tetrahedral`, `growth_treadmill`, `growth_treadmill@noise`: **100% singletons**. Not one symbol
  ever recurs. `A_eff = |A|` exactly, which is what a uniform distribution over serial numbers
  looks like.

### The three definitions as a second witness (§6.1 discipline applied to a definition)

| rule | S1 | S2 | S3 (degree) | reading |
|---|---|---|---|---|
| `glider` | 3 | 3 | 1 (A_eff 1.00) | converge: tiny |
| `binary_edge` | 56 (A_eff 1.7) | 56 (A_eff 1.7) | 8 (A_eff 1.16) | converge: tiny effective |
| `bounded_churn` | 61 (A_eff 4.1) | 76 (A_eff 4.1) | 36 (A_eff 1.47) | converge: small |
| `tetrahedral` / `growth_treadmill` / `@noise` | = n_events | = n_events | = n_events | converge: serial number |
| `triangle_subdivide` | 4400, 91% singletons | 4400 | **1602, 0 singletons, A_eff 1514** | **diverge** |

`triangle_subdivide` is the one row where the definitions disagree, and informatively: its *degree*
signatures recur (0 of 1602 are singletons) while its *context* signatures do not (4000 of 4400 are).
Its structural contexts are all different; the degree patterns they induce are not.

**S2 ≡ S1 on six of seven rules** (and 61 → 76 on `bounded_churn`). Since S2 hashes S1's whole input
as a prefix, equality means the post-rewrite local state is *fully determined* by the matched
pre-rewrite context. That is a substantive result about these rules, not a null: on this zoo, the
rewrite adds no information the match did not already contain.

---

## A4 — the novelty curves

### A4.1 / A4.2 — growth regime, both abscissae, with fit quality

| rule | vs STEP | vs EVENT |
|---|---|---|
| `glider` | PLATEAU b=+0.000 r²=1.000 last-decade-new 0.0% | PLATEAU b=+0.000 r²=1.000 0.0% |
| `triangle_subdivide` | LINEAR b=+0.991 r²=1.000 89.5% | LINEAR b=+1.000 r²=1.000 90.0% |
| `binary_edge` | SUBLINEAR b=+0.290 r²=0.812 46.4% | **PLATEAU** b=+0.000 r²=1.000 0.0% |
| `tetrahedral` | LINEAR b=+0.975 r²=1.000 89.2% | LINEAR b=+1.000 r²=1.000 90.0% |
| `growth_treadmill` | LINEAR b=+0.991 r²=1.000 89.5% | LINEAR b=+1.000 r²=1.000 89.8% |
| `growth_treadmill@noise` | LINEAR b=+0.991 r²=1.000 89.5% | LINEAR b=+1.000 r²=1.000 89.7% |
| `bounded_churn` | PLATEAU b=+0.000 r²=1.000 0.0% | PLATEAU b=+0.000 r²=1.000 0.0% |
| `bounded_churn` post-burn-in | PLATEAU b=+0.000 r²=1.000 0.0% | PLATEAU b=+0.000 r²=1.000 0.0% |

**Pre-registering both abscissae paid immediately.** `binary_edge` is SUBLINEAR against step and
**PLATEAU against event**: its alphabet had stopped growing entirely, and the apparent rise against
step was the event *rate* rising as the substrate exploded (36 → 27 628 nodes in 22 steps). One
abscissa alone would have reported growth that is not there.

### A4.3 — the bounded-churn substrate, reported separately

The zoo's only substrate with sustained identity turnover at flat size — A4.3 designates it the best
case for a growing alphabet — and it is the flattest thing measured. After the M1-mandated burn-in
(step ≥ 900): **12 raw symbols, effective size 3.57, zero singletons, top-3 mass 95.2%, PLATEAU on
both abscissae, over 24 164 post-burn-in events.** `n(t)` is flat at 36 by construction throughout,
and the node-WL class count `M(t)` runs 9 → 5 (peak 15).

**Burn-in was load-bearing, exactly as M1 said.** Whole-run: 61 symbols, 28 of them singletons.
Post-burn-in: 12 symbols, **0** singletons. Forty-nine of the whole-run symbols — 80% of the count —
were startup transients that never recurred. Scoring churn without burn-in would have inflated the
alphabet fivefold and manufactured a singleton tail.

**Substrate byproduct worth recording in the zoo:** over 2400 steps the churn substrate thins its
connectivity — final state 36 nodes / **24 edges**, down from the 6×6 arity-3 crystal's initial
density. Node count is flat by construction, but the "keeps two edges so connectivity does not
collapse to dust" property of the H1 rule degrades over long runs. That is a characterisation fact
this measurement produced incidentally and it bears on any future use of the substrate.

### Comparison to linear-novelty results in the literature — stated, with the differences

**No substrate in this zoo produced an ACCELERATING alphabet on either abscissa.** The outcomes are
PLATEAU, SUBLINEAR and LINEAR only. The classifier *can* return ACCELERATING — `tests/a2_1` proves
it on a quadratic curve — so this is a measured absence, not a classifier that cannot say the word.

This is **consistent with, and analogous to, Hash Chemistry's linear-rather-than-accelerating novelty
growth. It is not a replication**, and the differences are load-bearing:

1. Different substrate family — hypergraph rewriting under a fixed rule file, versus their chemistry.
2. Different quantity — realised *(rule, context)* event signatures, versus their novelty measure.
3. **Every LINEAR row here is pinned to a hard ceiling.** `N(e) ≤ e` (one event contributes at most
   one new symbol), and those rows sit at `|A|/n_events` of 0.917–1.000. They are linear because
   they are on the diagonal, which is the *maximum* the quantity can be, not because a growth law
   was fitted to free data.
4. Run budgets here are short (120–2400 steps); a linear-vs-accelerating claim over long horizons
   is not something this measurement is powered for.

---

## Instrument defects this run exposed (both reported, neither retro-applied)

### 1. `R* = 1` can be the ceiling masquerading as saturation — a seventh instance of the pattern

For `tetrahedral`, `growth_treadmill` and `growth_treadmill@noise` the committed rule returned
`R* = 1` with `g(1) = g(2) = g(3) = 0.000`. That looks like textbook saturation. It is not: those
runs sit at `|A|/n_events = 1.000`, so `|A|` **cannot** grow with R — every event already has its
own symbol and refinement has nothing left to split. The gain is zero for an arithmetic reason, not
a structural one.

This is the project's documented failure mode — a guard passing for the wrong reason — occurring for
the seventh recorded time, and it was caught by the `|A|/n_events` ceiling diagnostic added during
construction, not by the pre-registered rule, which has no ceiling clause.

**Proposed amendment, for a future pre-registration and NOT applied here:** `R*` may be reported
only when `|A(R_MAX)| / n_events < 0.5`; otherwise the run is `BUDGET-LIMITED` and no saturation
verdict is issued. Applying it retroactively would change three rows, so it is stated rather than
used, and both readings appear in the table above (⚠ceiling marks the affected rows).

### 2. "Smallest R with `g(R) < ε`" latches onto dips

`triangle_subdivide`: g(1) = 0.375, **g(2) = 0.000**, g(3) = 0.091. The rule takes the smallest
passing R and returns `R* = 2`, but refinement *resumes* above it. The `refines_above_r_star`
diagnostic reports this wherever `R*` is reported; the committed rule was not changed mid-run.

### 3. The `r²` defect

Recorded in A2 above and in `docs/instrument_registry.md` §2.8: a perfect plateau reported
`r²_loglog = 0.000` because a variance-based constancy test leaks a ~1e−28 denominator in log space.
Fixed before any A3 number was produced.

---

## A5 — which pre-committed branch the data selects

**The data selects the `rule-dependent` branch — the plan's "most informative outcome" — and the
rule-zoo property it exposes is sharper than "some rules are richer than others".**

The split is not arbitrary. It tracks **whether the substrate's node count is bounded**:

| class | rules | alphabet |
|---|---|---|
| **bounded `n`** | `glider` (n = 24 flat), `bounded_churn` (n = 36 flat) | **A_eff 3.00 and 3.57**, zero singletons, PLATEAU on both abscissae, `\|A\|/n_events ≤ 0.002` |
| **growing `n`, contexts recur** | `binary_edge` | **A_eff 1.69**, 90.7% of mass on one symbol, PLATEAU vs event |
| **growing `n`, all three definitions agree on non-recurrence** | `tetrahedral`, `growth_treadmill`, `growth_treadmill@noise` | `\|A\| = n_events` on S1, S2 **and** S3; **0 recurrent symbols**; LINEAR *by construction* |
| **growing `n`, the definitions DIVERGE** | `triangle_subdivide` | S1/S2 91% singletons, but **S3 gives 1602 degree-signatures with 0 singletons** — its degree patterns fully recur; only its *context* signatures do not |

> **B3 correction (B-track).** `triangle_subdivide` was originally grouped with the three
> all-definitions-agree rules at "92% singletons", which buried the one row where the second-witness
> discipline actually paid out. The contrast is the finding: where three definitions **converge** on
> non-recurrence — including the radius-free degree signature, which does not depend on the context
> encoding at all — the non-recurrence is strongly supported. Where they **diverge**, as here, the
> evidence is about **the context encoding on a growing graph**, not about the dynamics.
> `triangle_subdivide`'s degree patterns repeat perfectly; only the WL contexts they sit in do not.

Reading each branch of the pre-committed interpretation against this:

- **"Small alphabet, early saturation, plateauing novelty curve"** — holds, *conditioned on bounded
  size*, and holds strongly: on every substrate where the measurement is not budget-limited, the
  effective alphabet is between **1.69 and 4.13 symbols** and the novelty curve plateaus. The
  bounded-churn substrate — the zoo's designated best case for a growing alphabet — lands at **3.57
  effective symbols with zero singletons**. That is an independent substrate-side arrival at the
  same order of magnitude as the ALife economy's three values, by a completely different mechanism,
  and it was not obtainable from any existing data.
- **"Growing alphabet ⟹ the ALife result was local to the economy"** — the growing-`n` rules do emit
  unboundedly many distinct symbols, but **not one of those symbols ever recurs** (0 recurrent of
  120, 400 and 3200). A symbol seen exactly once carries no recurrence, so nothing can be composed
  out of it; by the no-singletons discipline the V-track closeout already established, that is a
  serial number, not an alphabet. This branch is **not** selected, but the observation is real and
  is the honest counterweight to the first bullet.

  > **AMENDED BY THE B-TRACK (B2).** *The rejection above was not earned by the data it cited.*
  > Those rows sat at `|A|/n_events = 1.000` on runs of 120, 400 and 3200 events over a substrate
  > growing from 36 nodes — and **nothing can recur when there are barely more events than nodes**.
  > Zero recurrence at that budget is what would have been measured whether or not recurrence exists
  > at longer horizons. It is the same ceiling artifact as the seventh guard instance, appearing in
  > the *recurrence* column instead of the radius column, and amendment 1's own logic suspends the
  > verdict.
  >
  > By the letter of the B-track's pre-committed reading the branch is **BUDGET-LIMITED and stays
  > OPEN rather than rejected**: a 16× horizon ladder (400 → 6400 events) could not escape the
  > ceiling, because every growing rule in the zoo mints exactly one node per firing, so
  > `n_events ≈ n_nodes` **by construction**.
  >
  > The conclusion nevertheless survives on a *different and better* argument than the one given
  > here: `|A|/n_events` stayed pinned at **1.0000 across the whole 16× ladder**, and a budget
  > artifact must dilute as the budget grows. See `docs/btrack_findings.md` B2.
- **"Large but saturating ⟹ resolution is not the binding constraint"** — not selected. Nothing
  large *and* saturating was measured.
- **A finding the plan did not anticipate: saturation in radius and saturation in time dissociate.**
  `binary_edge` and `bounded_churn` **never saturate under refinement** (R\* = NONE at ε = 0.05) yet
  **plateau in time on both abscissae**. Finer resolution keeps splitting their symbols while the
  realised symbol set stops growing. The ALife result's *form* — "refining 3 → 121 codes splits zero
  classes" — therefore **does not transfer to the substrate**: here refinement does keep splitting,
  and the alphabet is small anyway. The bound on realised novelty is not a resolution bound.

### What this licenses, and what it does not

It **licenses the argument** that within the current rule family, expressive novelty is bounded
wherever substrate size is bounded — and that growing the substrate is not a way around it, because
the growing rules buy their symbol count with non-recurring symbols. That is a mechanism for
linear-not-accelerating novelty, and it is the argument for endogenous rule encoding — the only
roadmap item in which the *alphabet itself* can grow.

It **does not license building endogenous rule encoding now**. The plan is explicit that a
small-alphabet result licenses the *argument*, which is a separate design decision. Three specific
gaps stand between this and that decision:

1. **Three rows are budget-limited** (`|A|/n_events` = 1.000) and one is at 0.917. For those, `|A|`
   measures the run length. A longer-horizon measurement with the ceiling clause of amendment 1 is
   the obvious next step, and it is cheap.
2. **The recurrence reading is derived, not pre-registered as a verdict quantity.** Singletons were
   pre-registered as a reported column with this exact rationale, but "recurrent alphabet size" as a
   *decision* variable should be pre-registered before it carries a verdict.
3. **`R* = NONE` on the two best-measured substrates** means the pre-registered knob did not
   converge where it mattered most. The R-sweep is reported in full so nothing is hidden, but a
   selection rule that rejects on the load-bearing rows is a weak instrument, and the falsifiable
   follow-on ("no parameter tuning within the current rule family produces open-ended novelty")
   deserves a knob that resolves.

---

## The single sentence

**The substrate's realised event alphabet is small and closed wherever the substrate's size is
bounded — 1.7 to 4.1 effective symbols, zero-to-few singletons, plateauing on both abscissae,
including on the bounded-churn substrate that was the best case for the opposite result — while the
rules that appear to have unbounded alphabets buy their symbol count entirely in symbols that never
recur; so a fixed symbol set does bound what this substrate can build, growing the alphabet is the
architectural question, and the ALife three-symbol result generalises in its conclusion but not in
its mechanism, because on the substrate refinement never stops splitting symbols and the alphabet is
small regardless.**
