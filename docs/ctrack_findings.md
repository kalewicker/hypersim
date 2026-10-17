# C-track findings — writing rules into the zoo's measured gap

**Substrate-track design task. Zero biology. Characterised as physics before anything else uses it.**

- Pre-registration: [`docs/ctrack_preregistration.md`](ctrack_preregistration.md); the C3.2 criterion
  and its evaluation precedence committed at **`61fc0ae`**, before any C3 number existed.
- Instrument: `hg-alphabet`, unchanged. `tests/b0_instrument_pin.rs` extends here verbatim and passes.
- Raw output: `runs/ctrack/`.

---

## The single sentence

**A node-count-neutral rule can mint fresh context, but neither mechanism could mint it slowly
enough for recurrence to keep accumulating — both saturated into a fixed recurrent set and stayed
there over a 16× horizon — so the gap is not occupied; but the zoo's bimodality is nonetheless
refuted, because both rules landed in the *third* class the plan anticipated, and one of them
carries a 17-symbol fully-recurrent alphabet four times the effective size of anything previously in
the flat class.**

---

## C0 — the `A_eff` share confound (blocking gate, closed)

`A_eff` decomposes **exactly**:

```
H_global = H(w) + Σ_r w_r H_r
A_eff    = 2^H(w)  ×  2^(Σ_r w_r H_r)
         = share perplexity  ×  weighted geometric mean of within-rule A_eff
```

asserted arithmetically for every B1 row (`c0_share_decomposition_is_exact`). `2^H(w)` is the
contamination and is now a measured number. On a single-rule file it is **exactly 1**, which is why
the A-track was never affected — asserted as its own firing test.

### The defect, confirmed

At k = 2 the two horizons give **identical `|A|` = 16 and identical recurrent = 16** with `A_eff`
**3.74 vs 7.53**:

| horizon | `A_eff` | `2^H(w)` | corrected | **within-rule sum** |
|---|---|---|---|---|
| equal events | 3.74 | 1.146 | 3.26 | **11.28** |
| equal steps | 7.53 | 1.436 | 5.25 | **11.07** |

**One correction to the plan's diagnosis, reported rather than smoothed.** The share factor is *not*
the whole discrepancy: `A_eff` differs by 2.01× and `2^H(w)` accounts for only 1.25× of it, because
the "corrected" quantity is still *weighted* by the shares. The only **fully** share-independent
effective-size witness is `Σ_r within-rule A_eff`, in which the shares do not appear at all — and on
it the two horizons agree to **2%** where `A_eff` disagreed twofold.

### C0.2 — does B1's LINEAR verdict survive? Yes, on the metric it was registered on

| measure | PRIMARY (equal events) | SECONDARY (equal steps) |
|---|---|---|
| `A_eff` (B1's metric, contaminated) | LINEAR b=+0.937 r²=0.886 | LINEAR b=+0.802 r²=0.877 |
| corrected `A_eff / 2^H(w)` | LINEAR b=+0.947 r²=0.820 | LINEAR b=+0.832 r²=0.993 |
| **`Σ` within-rule `A_eff`** (share-independent) | LINEAR b=+0.759 **r²=0.589** | LINEAR b=+0.717 **r²=0.560** |
| **recurrent** (share-independent) | **SUPER-LINEAR** b=+2.243 **r²=0.992** | **SUPER-LINEAR** b=+2.243 r²=0.992 |

**The two share-independent witnesses disagree with each other**, and that is carried forward rather
than averaged away. `Σ within-rule A_eff` is LINEAR but weakly — r² = 0.589, and **non-monotone in
k** (3.00 → 11.28 → 6.91 → 9.85), because adding `churn` changes what `influx` sees: `glider`'s
within-rule `A_eff` drops 2.98 → 1.36 and `influx`'s 8.30 → 4.32 when churn arrives. `recurrent` is
the cleanest signal in the whole programme (r² = 0.992) and it says SUPER-LINEAR — consistent with
B1's own raw-`|A|` diagnostic (SUPER-LINEAR, b = +2.608).

**B1's headline stands on its pre-registered metric.** The reading that survives all four measures is
the one B1 already gave: rule count buys context *types* faster than linearly, and mass-weighted
effective size only linearly.

*Gate open.*

---

## C1 — the two mechanisms (predictions committed at `fb4a0d5`)

| | `swap` (`rules/swap.toml`) | `cyclic3` (`rules/cyclic3.toml`) |
|---|---|---|
| mechanism | degree-preserving double-edge swap: `{a,b},{c,d} → {a,d},{c,b}` — the canonical Markov move on graphs with a fixed degree sequence | Fisch–Gravner–Griffeath cyclic CA: state `L → L+1 (mod 3)` iff a neighbour is already at `L+1` |
| churn in | topology | state |
| parameters | **none** — no rate, no threshold, no period | period 3, derived from mechanism (2 states cannot make a directed wave) |
| predicted `minted/event` | 0.0000 exactly | 0.0000 exactly |
| predicted verdict | **GAP OCCUPIED** | **FLAT RULE IN DISGUISE** (predicted to fail) |

Two mechanisms *predicted to behave differently* are second witnesses; two predicted to agree would
be one attempt run twice.

**C1.3 was deliberately not built.** `bounded_churn` already instantiates the balanced
mint-and-retire family — its `churn` rule mints one and retires one, and `churn` is 1.7–4.2% of
ruleset events, so ruleset-level `minted/event` is already ≈ **0.04**. Its answer is already
measured: `A_eff` 3.57, zero singletons, PLATEAU. A second one would vary only *k*, which is exactly
the sweep binding rule 3 forbids.

---

## C2 — bare characterisation (the firewall gate, and it fired)

| arm | n | e | degree seq | components | `minted/event` | events / 8000 steps | final 10% | **GATE** |
|---|---|---|---|---|---|---|---|---|
| `swap` @ PureCrystal 6×6 | 36 flat | 85 flat | **preserved exactly** | 1 → 1 | **0.0000** | 72 000 | 7 200 | **PASS** |
| `cyclic3` @ 6×6 open — *pre-registered* | 36 flat | 85 flat | preserved | 1 → 1 | 0.0000 | **59** | **0** | **FAIL** |
| `cyclic3` @ 24×24 torus — *deviation* | 576 flat | 1728 flat | preserved | 1 → 1 | 0.0000 | 24 771 | 2 400 | PASS |

**The gate killed `cyclic3` on its pre-registered substrate** — 59 firings in 8000 steps, then
frozen, self-terminated exactly as `influx` did in B1.1. That failure is pinned as a test so it
cannot quietly start passing.

**Diagnosed to mechanism, then deviated from once, and the deviation is recorded as one.** Two
defects in the original substrate choice, both identifiable without reference to any alphabet
number: an **open boundary is a sink** (waves propagate off the edge and the lattice drains), and
**6×6 has no room for a spiral core**. The correction is to the *substrate*; the rule itself has no
parameters and none were touched. 24×24 periodic was run **once**. The pre-registered result stands
as the published C2 outcome for that arm and the torus arm is always labelled as the deviation.

Every structural prediction held on every arm: `minted/event` exactly 0.0000, `n(t)` and `e(t)`
exactly flat, degree sequence preserved. The long-run edge trajectory the `bounded_churn` byproduct
motivated shows **no thinning at all** for either rule — `swap` holds 85 edges and 1 component for
8000 steps, where `bounded_churn` had decayed to 24 edges by 2400.

---

## C3 — the measurement

### `swap` — topology churn, zero parameters

| steps | events | `\|A\|` | `A_eff` | singletons | **recurrent** | `\|A\|/n_events` |
|---|---|---|---|---|---|---|
| 2 000 *(pre-reg)* | 18 000 | **17** | 16.67 | 0 | **17** | 0.0009 |
| 8 000 *(pre-reg)* | 72 000 | **17** | 16.67 | 0 | **17** | 0.0002 |
| 32 000 *(power)* | 288 000 | **17** | 16.67 | 0 | **17** | 0.0001 |

Refinement sweep 1 → 11 → 17 → 18 → 18, **R\* = 3** (amendment 1 satisfied: ceiling 0.0001 ≪ 0.5).
PLATEAU on both abscissae. **`|A|` is byte-identical at 1×, 4× and 16× the horizon.**

> **C3.2 VERDICT: PARTIAL** — bounded structure, no accumulation. `|A|/n_events` = 0.0002, far below
> the ceiling, but recurrence does not grow (17 → 17, ×1.00).

**Prediction miss #1 — the primary mechanism failed, and the manner of failure is the finding.**
`swap` was predicted GAP OCCUPIED. It saturates completely by step 2000 and never moves again.

*Diagnosis.* The substrate is **unlabelled** — every node is label 0 with energy 0 — so every WL base
hash is identical and WL classes are determined **purely by degree structure**. `swap` conserves the
degree sequence *exactly*. At R = 1 a node's class is therefore exactly its degree, which is
**frozen per node for all time**; at R = 2 the class is (degree, neighbour-degree multiset), which
can move but ranges over a tiny space. **The very property chosen for parameter-freedom — exact
conservation — is what caps the alphabet.** This is a defect in the rule/substrate pairing that I
can name, so it is weak evidence for any architectural claim.

**Prediction miss #2.** S3 was predicted **degenerate at `|A| = 1`** because degrees never change.
Measured **`|A|` = 4, `A_eff` = 3.57**. The reasoning was simply wrong: S3 records the *sorted
multiset of (pre-degree, post-degree) pairs*, so it encodes **which** degrees the matched quadruple
had, not merely that they were unchanged. Degree-invariance makes each pair `(d, d)`; it does not
make the multiset constant.

### `cyclic3` on the torus — state churn, cycle length **3**

| steps | events | `\|A\|` | `A_eff` | within-sum | singletons | **recurrent** | `\|A\|/n_events` |
|---|---|---|---|---|---|---|---|
| 2 000 *(pre-reg)* | 6 771 | **780** | 8.22 | 6.18 | 767 | **13** | 0.1152 |
| 8 000 *(pre-reg)* | 24 771 | **780** | 4.13 | 4.82 | 767 | **13** | 0.0315 |
| 32 000 *(power)* | 96 771 | **780** | 3.29 | 4.46 | 767 | **13** | 0.0081 |

> **C3.2 VERDICT: PARTIAL.** And it is **knife-edge**: the criterion's flat bar is `A_eff ≤ 4.0` and
> the pre-registered high rung gives **4.13**. At the power rung `A_eff` = 3.29, at which the same
> criterion returns **FLAT RULE IN DISGUISE**. The pre-registered pair governs; both are reported.

*Diagnosis.* 767 of the 780 symbols are **startup transients** from the random initial labelling,
seen once and never again. Only **13** recur — and a rotating spiral is a **periodic attractor**, so
its recurrent context set is finite and small *by construction*. The prediction (fail to occupy the
gap) was correct; the exact class was off by one branch.

**The C1.2 caution was warranted but is not what happened.** The worry was that a 3-state channel
would reproduce the ALife three-symbol result by construction. It did not: the raw alphabet is 780,
far above the naive `3 × (neighbour multisets)` intuition. The bound came from **attractor
formation**, not from the state count.

### C3.3 — scale-invariance: the criterion could not fire, and the raw fact needs no criterion

| rule | span | `\|A\|/n_events` | verdict |
|---|---|---|---|
| `swap` | 16.0× | 0.0009 → 0.0001 | `INDETERMINATE` |
| `cyclic3` | 14.3× (events) | 0.1152 → 0.0081 | `INSUFFICIENT LADDER` |

**An inconsistency in my own C-track pre-registration, reported as one:** C3.3 reuses B2.4's
instrument, whose committed minimum span is **16×**, while the pre-registered 2000/8000 pair spans
only **4×** — so on the pre-registered rungs it could return nothing but `INSUFFICIENT LADDER`. The
32 000-step rung was added **for power, not for a verdict**; it changed no threshold, and the C3.2
verdict is still read from the pre-registered pair alone. Even then B2.4 cannot fire here: it defines
`SCALE-INVARIANT` as the ratio *pinned at ≥ 0.99*, which is a test built for rules riding the
ceiling, and these sit four orders of magnitude below it.

**The underlying observation is unambiguous without any criterion: `|A|` is literally identical at
every rung — 17, 17, 17 and 780, 780, 780.** There is no dilution to detect because the alphabet is
fully saturated.

### C3.4 — the k = 5 row is VOID, and B-track open item 3 is NOT closed

| k | events | `\|A\|` | `A_eff` | recurrent | `A_rule_only` |
|---|---|---|---|---|---|
| 1–4 | 20 000 each | 3 / 16 / 66 / 97 | 3.00 / 3.74 / 7.26 / 11.01 | 3 / 16 / 43 / 62 | 1 / 2 / 3 / 4 ✓ |
| **5** | 20 000 | 97 | 11.01 | 62 | **4 — VOID** |

`swap` fires **zero** times in the k = 5 composition, so by B1.2's own rule the row is void and is
reported as void rather than read as a small alphabet.

**The cause is compositional, not a defect in `swap`** (`tests/c3_4_composition_probe.rs`):

| composition | `swap` fires | others |
|---|---|---|
| `swap` alone | 4 500 | — |
| `glider` + `swap` | 4 500 | glider 500 |
| k = 4 + `swap` (swap is `rule_id` 4) | **0** | influx 8242, churn 166, transfer 429 |
| `swap` + k = 4 (swap is `rule_id` 0) | 4 500 | influx **315**, churn **0**, transfer **0** |

**`swap` binds 4 nodes per firing and the composite crystal has exactly 36.** Alone it fires 9 times
per step — 9 × 4 = 36, the whole crystal — so under `MaxParallel` node-disjointness `swap` and the
crystal-resident rules are **mutually exclusive**, and `rule_id` order (the conflict resolver's
tie-break) decides which side wins outright. Starvation runs in *both* directions, which is what
rules out "swap is simply weak".

*One zero in that table is not starvation and is labelled so:* in `glider + transfer + swap`,
`transfer` fires 0 despite a lower `rule_id`, because its guard needs property 1 > 10 and without
`influx` the fill never leaves 5.0 — no candidates at all. A zero does not explain itself, which is
why the arms are run side by side.

**So `swap` satisfies every B1.1 criterion for a fifth rule — live, perpetual, node-count-neutral —
and still cannot be composed with the k = 4 set on this substrate.** B-track open item 3 stays open,
with its difficulty now understood: the blocker is not the zoo's supply of rules but node-disjoint
conflict resolution on a 36-node substrate.

---

## C4 — which pre-committed branch the data selects

**"Gap not occupied, across two or more independent mechanisms."** Both mechanisms returned PARTIAL.
Per the plan this is the branch that would make an *architectural* claim — and it must be earned by
attempts rather than inferred from absence, so here is exactly how much it is worth.

**The manner of failure, which rule 4 says is the finding:**

- **`swap`** — saturates at 17 symbols by step 2000 and is byte-identical at 16× the horizon.
  *Attributable to a design choice I can name*: on an unlabelled substrate WL classes are pure
  degree structure, and the rule conserves the degree sequence exactly. **This weakens the
  architectural reading substantially.** The obvious untried follow-up is a node-count-neutral rule
  that *changes* degrees, or the same rule on a labelled substrate.
- **`cyclic3`** — 780 symbols of which 767 are startup transients; the recurrent set is 13 and fixed.
  *Attributable to attractor formation*: a rotating spiral is periodic, so its recurrent context set
  is finite by construction. This is the more general failure mode.

**What both share** is the shape the B-track already named: **the bound is on exploration, not on
resolution.** The dynamics reach an attractor, and an attractor's recurrent context set is finite and
small. The C-track's contribution is that this now holds for rules *purpose-built to avoid it*.

**Honest verdict: the bimodality is refuted, but the architectural claim is not earned.** Two things
cut against the strong reading:

1. **The zoo was not bimodal — it is at least trimodal.** `swap` is node-count-flat, edge-count-flat,
   degree-preserving, with **17 recurrent symbols, zero singletons, `A_eff` = 16.67** — roughly **4×
   the effective size of anything previously in the flat class** (1.69–4.13). PARTIAL is a real third
   class and it is now populated. "There is nothing in between" was false.
2. **One of the two failures is over-determined by my own design.** Rule 4 says a failure to occupy
   the gap is the more interesting result *if* it is earned; `swap`'s is not fully earned, because
   the mechanism conserves precisely the invariant the measurement reads.

So: **gap not occupied by these two mechanisms; bimodality refuted; architecture not implicated.**

---

## Instrument findings this track produced

**`A_eff` is a fragile headline metric, for two independent reasons now.**

1. **C0 — rule-share contamination.** Over a multi-rule file it moves with the share distribution
   independently of context diversity; the factor is exactly `2^H(w)`.
2. **C3 — horizon dependence under transient singletons.** `cyclic3`'s alphabet is *fixed* at 780
   symbols, yet `A_eff` falls 8.22 → 4.13 → 3.29 across the ladder, purely because the 767
   never-repeated symbols' mass dilutes as events accumulate. **Any `A_eff` threshold is therefore
   horizon-sensitive**, and this one straddles the C3.2 flat bar: PARTIAL at the pre-registered rung,
   FLAT RULE IN DISGUISE at the longer one.

**`recurrent` count is immune to both** — share-independent by construction, and unchanged across a
16× horizon in both C-track rules (17 → 17 → 17; 13 → 13 → 13). It was already the cleanest signal in
B1 (r² = 0.992 against rule count). **Recommendation for any successor track: report `recurrent` as
the headline and `A_eff` as the companion, not the reverse.** Not applied retroactively; A- and
B-track verdicts stand on their pre-registered metrics.

---

## Open items

1. **The untried third mechanism**: node-count-neutral but degree-*changing*, or `swap` on a labelled
   substrate. This is the direct follow-up to `swap`'s named failure and it is cheap.
2. **B-track open item 3 stays open** — a fifth composable rule needs to survive node-disjoint
   conflict resolution against `influx`, not merely exist.
3. **`cyclic3`'s knife-edge verdict** would resolve with a recurrent-count-based criterion, which
   C3's own instrument finding recommends but which is not applied here because it was not
   pre-registered.
4. The **ten pre-existing `hg-train` failures** from B4.4 remain; `hg-alphabet` is green.
