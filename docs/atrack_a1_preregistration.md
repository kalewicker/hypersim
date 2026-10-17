# A-track / A1 — event-signature definition and the radius selection rule

**Committed before any alphabet cardinality is computed.** This file is the pre-registration the
A-track plan's binding rule 2 requires: the resolution knob is selected by a structural property
(refinement saturation), never by the alphabet size it yields, never by whether it supports the
fixed-alphabet hypothesis, and never by comparison against the ALife three-symbol result.

Scope: **substrate-track measurement, zero biology.** The measurement crate `hg-alphabet` depends on
`hg-core`, `hg-rules`, `hg-engine`, `hg-observe` and on nothing else. `hg-alife` is not in the
dependency graph and no `alife` feature is enabled anywhere in this track.

---

## 0. What is being measured, and what is deliberately not

The quantity is the **realised event space**: the distinct *(rule, local structural context)*
signatures a run actually produces. It is not "which rule fired" — that is bounded by the rules
file by construction, cannot come back large, and is the A-track plan's named trap (the arm-C ρ
clause recurring: a clause that cannot fail is not evidence).

The vacuous quantity is nevertheless **computed and reported alongside** as `A_rule_only`, so the
contrast between "number of rules" and "number of realised event types" is visible in the same
table rather than asserted.

---

## 1. The signature definitions (A1.1)

Let

- `G_t` = the graph state at the **start** of step *t*. This is exactly the graph every binding in
  step *t* was matched against: `Engine::collect_candidates` matches all rules against the
  pre-step snapshot before any rewrite in that step is applied. "Pre-rewrite graph" therefore has
  an unambiguous referent, and it is the state that *caused* the match.
- `G_{t+1}` = the graph state after the whole of step *t*.
- `wl_R(v; G)` = the project's Weisfeiler–Lehman class of node *v* in graph *G* at radius *R*,
  computed by `hg_core::compute_wl_class_map` — the same machinery as the rest of the project, so
  the signature inherits its determinism guarantees (sorted neighbour hashes, nonzero seed) and the
  Q2 invariant (properties excluded from WL identity).
- An **event** is one accepted rewrite: one element of the step's `RewriteEvent` vector.

Radii swept: **R ∈ {0, 1, 2, 3, 4}**, `R_MAX = 4`.

### S1 — context signature (PRIMARY)

```
S1_R(e) = H( rule_id(e), [ wl_R(v_0; G_t), wl_R(v_1; G_t), … ] )
```

over the event's bound nodes **in LHS-variable order**. A creation rule (schedule-driven, zero
bound nodes) contributes the empty list. Var order is canonical by construction — the compiled
pattern fixes variable indices — so it is a sequence, not a multiset, and is hashed in order.

S1 is the primary definition because it reads only `G_t`, which is precisely the state the matcher
saw.

### S2 — structural-delta signature (cheaper variant)

```
S2_R(e) = H( rule_id(e),
             [ wl_R(v_i; G_t) ]                      in LHS-var order,
             sorted[ wl_R(u; G_{t+1}) : u surviving bound node ],
             sorted[ wl_R(w; G_{t+1}) : w created by e ] )
```

The post-state collections are unordered and are **sorted before hashing** (binding rule 5).

### S3 — degree signature (cheaper variant, radius-free)

```
S3(e) = H( rule_id(e), sorted[ ( deg(u; G_t), deg(u; G_{t+1}) ) : u ∈ touched(e) ] )
```

`touched(e)` = bound nodes ∪ nodes created ∪ `cleanup_neighbors` (the engine's one documented
locality-of-effect exception, v4 §8.1). Degree = number of incident edges; a node absent from a
graph takes the sentinel `−1`.

This records the (before, after) **pair**, which strictly refines the plan's "multiset of degree
changes" — the pair determines the change. The refinement is the conservative choice for an
instrument that is required to be *able* to return "large", and it is stated here rather than
discovered later.

### Stated limitation of S2 and S3

Within one step, accepted rewrites are applied sequentially, so `G_{t+1}` is the joint post-state
of the whole step, not of `e` alone. Conflict resolution guarantees the bound-node sets are
disjoint, but radius-R balls may still overlap. S1 is unaffected (it reads `G_t` only). S2 and S3
therefore measure "this rewrite's site after the step", and under `MaxParallel` with many
simultaneous firings that can include neighbours' effects. This is why they are the *cheaper
variants* and S1 is primary; the three converging or not is the §6.1 second-witness discipline
applied to a definition rather than to a metric.

### Hash

`H` = `FxHasher` seeded with the project's `WL_SEED` (`0x9E37_79B9_7F4A_7C15`), order-sensitive,
with explicit length prefixes between sub-sequences so concatenation cannot alias.

---

## 2. The radius selection rule (A1.2) — committed here, before any cardinality exists

`|A(R)|` denotes the alphabet cardinality of signature variant S1 (and, computed the same way, S2)
at radius R over a given run.

**Monotonicity (asserted, not assumed).** WL colours at round R+1 determine the colours at round R
(the previous colour is folded into every refinement step), so `wl_{R+1}` refines `wl_R` and
`|A(R)|` is non-decreasing in R. A measured violation is an **instrument fault**, not a finding,
and the harness asserts it.

**Marginal refinement gain**

```
g(R) = ( |A(R+1)| − |A(R)| ) / |A(R)| ,   for R ∈ {0, 1, 2, 3}
```

**Selection rule**

> **R\* = the smallest R ∈ {1, 2, 3} with g(R) < ε, where ε = 0.05.**
>
> If no such R exists, the alphabet **does not saturate within the swept range**, no R\* is
> reported, and the rule has *rejected*.

Fixed choices, stated so they cannot drift: `ε = 0.05`, `R_MAX = 4`.

- **R = 0 is excluded from eligibility.** It is the no-context degenerate baseline (the node's own
  label and energy field with no neighbourhood at all), kept only as a control so that the
  contribution of context is visible. Selecting it would be selecting "almost the vacuous
  definition".
- **R\* = 4 is unreachable by construction**, because verifying saturation at R requires measuring
  R+1 and the sweep stops at 4. A run whose alphabet is still refining at R = 3 is reported as
  *not saturating within the swept range*, which is the honest reading, not as R\* = 4.
- R\* is selected **per (rule-set, signature variant)**: whether a substrate's event alphabet
  saturates is a property of that substrate, which is exactly what A5's "rule-dependent" branch
  is about.
- The **full R-sweep table is reported for every rule regardless of R\***, so no cherry-picking of
  a radius is possible after the fact.

**Explicitly forbidden**, restated: selecting R by the alphabet size it yields, by whether the
result supports the fixed-alphabet hypothesis, or by comparison against the ALife alphabet.

---

## 3. Composition and effective size (A3.3) — committed formulas

Over the symbol frequency distribution `{p_i}` of a run's alphabet:

- `H = − Σ p_i log₂ p_i`, summed in **sorted symbol order** (the counts live in a `BTreeMap`, so
  iteration is by key; this is the C3 sorted-summation gate applied at construction rather than
  retrofitted).
- **effective alphabet size** `A_eff = 2^H` (perplexity). Reported *beside* the raw count, never
  instead of it, and the raw count is never reported alone.
- `top1_mass`, `top3_mass` — the fraction of events carried by the 1 and 3 most frequent symbols.
- `singletons` — symbols observed exactly once. This is the `K_MIN`/no-singletons discipline from
  the V-track closeout: an alphabet whose symbols are mostly singletons is a different object from
  one whose symbols carry realised mass.

---

## 4. The novelty curve and its growth regime (A4) — committed before any curve is fitted

`N(x)` = cumulative distinct symbols after abscissa `x`. Two abscissae, **both reported**:

- **vs step** — the quantity directly comparable to the linear-novelty result in the literature.
- **vs cumulative accepted rewrites (events)** — the confound-controlled companion. On a growing
  substrate the number of events per step grows with the substrate, so a curve that accelerates
  against *step* may be flat per *event*. Reporting only one of these would be a choice made after
  seeing the data.

The verdict sentence quotes the **vs-step** regime (comparability), with the vs-event regime stated
immediately beside it.

**Fit.** On the final decade of the abscissa (`x ≥ x_max / 10`), sampled at 200 log-spaced points:

- log–log OLS: `N ≈ a · x^b` → exponent `b` and `r²_loglog`;
- linear OLS on the raw axes → slope and `r²_linear`;
- `final_decade_new_frac` = `( N(x_max) − N(x_max/10) ) / N(x_max)`.

**Regime labels** (evaluated in this order):

| condition | label |
|---|---|
| fewer than 8 usable points in the window | `Insufficient` |
| `final_decade_new_frac < 0.05` **and** `b < 0.15` | `Plateau` |
| `b < 0.85` | `Sublinear` |
| `0.85 ≤ b ≤ 1.15` | `Linear` |
| `b > 1.15` | `Accelerating` |

A label is **never reported without `b`, both r² values, and `final_decade_new_frac`** (plan A4.2:
"report it with the fit quality, not just a label"). If the alphabet is exactly constant across the
window, `b = 0` and `r²_loglog = 1.0` by convention (a constant is a perfect fit to a constant).

**Bound worth stating in advance:** `N(e) ≤ e`, since one event can contribute at most one new
symbol. `Accelerating` against events is therefore possible only while `N` is catching up to the
diagonal, and is impossible asymptotically. No such bound applies against *step*.

---

## 5. Calibration gates (A2) — committed bars

### A2.1 — synthetic firing test (binding rule 6: every criterion ships with a case where it rejects)

The saturation rule of §2 and the regime classifier of §4 are exercised on synthetic streams whose
answers are known by construction:

| stream | construction | required verdict |
|---|---|---|
| `fixed` | `\|A(R)\| = 6` for all R | saturates, `R* = 1` |
| `growing` | `\|A(R)\| = 3·2^R` | **no R\* — the criterion REJECTS** |
| `late` | `\|A(R)\| = [4, 40, 41, 41, 41]` | `R* = 1` (R = 0's large gain must not fool it) |
| `plateau_curve` | constant after an initial rise | `Plateau` |
| `linear_curve` | one new symbol every k events | `Linear` |
| `accel_curve` | `N ∝ x²` within its window | `Accelerating` |

The `growing` row is the firing case: a saturation criterion that cannot reject cannot support a
verdict.

### A2.2 — known-poor versus known-rich rule pair

| rule | expectation | why |
|---|---|---|
| `glider` | tiny, saturates immediately | flat node count, mints no fresh nodes, reuses a, b, c |
| `triangle_subdivide` | large, plausibly growing | explodes 36 → 396+, minting fresh label-0 structure |

**The gate, committed:** at **every** R ∈ {1, 2, 3, 4},

```
|A_S1(R)| for triangle_subdivide  ≥  4 × |A_S1(R)| for glider
```

and the two must not receive the same growth-regime label. 4× is a mechanism-derived *minimum*
separation (one substrate mints fresh structure at every firing, the other permutes a fixed ring),
not a bar fitted to observed values.

**If the pair does not separate, the instrument is not measuring context diversity, nothing
downstream is interpretable, and the track stops to diagnose** — no A3 number is reported.

---

## 6. Instrument discipline carried over

- **Engagement counter (instrument registry Rule 1).** Every per-rule alphabet carries
  `n_events`, the number of times the instrument actually did work. `n_events = 0` emits
  `INSTRUMENT_NOT_ENGAGED` and the row is written `unmeasured`, never `0`.
- **Predicted ranges (Rule 2), all hard:** `1 ≤ |A| ≤ n_events`; `1 ≤ A_eff ≤ |A|`;
  `|A(R)|` non-decreasing in R; `|A_S1(R)| ≥ A_rule_only` (S1 refines rule-only by construction);
  `R* ∈ {1, 2, 3}` or absent.
- **Reproducibility (binding rule 5).** Alphabet enumeration is set-and-hash work, which is where
  index-order contamination lives. All symbol containers are `BTreeMap`/sorted `Vec`; every
  entropy and every ranking is computed in sorted key order. The order gate is checked at
  **n = 3 separate OS processes**, per §6.2's worked counter-example where processes 0 and 1 agree
  bit-for-bit and only process 2 diverges.
- **Burn-in on the churn substrate.** The M1 correction requires churn to be scored after burn-in
  (~215 of 231 WL classes there are startup transients). The churn run therefore reports its
  alphabet **twice**: whole-run, and restricted to events at step ≥ 900 (the N0-v2 derived churn
  burn-in). Both numbers are published; the post-burn-in one is the one the verdict uses.

---

## 7. Run budgets (committed before the runs)

Each rule is run to a step cap **or** a node cap, whichever binds first, and the stop reason is
recorded with the result. Quiescence (no accepted rewrite for 8 consecutive steps) also stops a run
and is recorded.

| run | initial condition | step mode | step cap | node cap |
|---|---|---|---|---|
| `glider` | `GliderRing(24)` | SequentialRandom | 2000 | — |
| `triangle_subdivide` | `PureCrystal 6×6 arity 3` | MaxParallel | 400 | 20000 |
| `binary_edge` | `PureCrystal 6×6 arity 2` | MaxParallel | 400 | 20000 |
| `tetrahedral` | `K4 seed` | MaxParallel | 400 | 20000 |
| `growth_treadmill` | `PureCrystal 6×6 arity 3`, one node label 1 | MaxParallel | 400 | 20000 |
| `growth_treadmill @ noise` | `WhiteNoise`, one node label 1 | MaxParallel | 400 | 20000 |
| `bounded_churn` | `PureCrystal 6×6 arity 3`, fill 5.0 | MaxParallel | 2400 | — |

A node cap that binds is not a failure — it is recorded as the stop reason, and the novelty curve
is fitted on the range actually reached.

---

## 8. What this pre-registration does not decide

The A5 interpretation branches are already written in the plan and are not restated or amended
here. This file fixes the *instrument*: the definitions, the knob, the bars, and the reporting
obligations — nothing about which branch the data will select.
