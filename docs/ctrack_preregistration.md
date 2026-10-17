# C-track pre-registration — the gap rules, their mechanisms, and their predicted behaviour

**Committed before any C-track rule is measured.** Substrate-track design task, zero biology.

Inherits the A-track instrument (`a43ca3d`) and the B-track's adopted amendment 1 (`6bb672d`)
unchanged. `tests/b0_instrument_pin.rs` extends to this track verbatim and still passes.

---

## 0. C0 is closed — the gate is open

C0 required the B1 series re-read on share-independent measures before C1 could start. Done, and
recorded in `docs/ctrack_findings.md` C0. Summary of what it changed:

- The confound is **real and now exactly quantified**: `A_eff = 2^H(w) × 2^(Σ_r w_r H_r)`, so the
  share perplexity `2^H(w)` is a separable factor, measured at 1.146 / 1.436 on the two k = 2 rows
  and ≈ 1.003 at k = 3 and 4.
- **B1's LINEAR verdict survives** on `A_eff`, on the share-corrected `A_eff / 2^H(w)`, and on the
  fully share-independent `Σ_r within-rule A_eff` — on both horizons.
- One refinement to the plan's diagnosis, reported rather than smoothed: the share factor is **not
  the whole** k = 2 discrepancy. `A_eff` differs 2.01× across horizons and `2^H(w)` accounts for only
  1.25× of it, because the corrected quantity is still *weighted* by the shares. The **only** fully
  share-independent effective-size witness is `Σ_r within-rule A_eff`, and on it the two horizons
  agree to **2%** (11.28 vs 11.07) where `A_eff` disagreed by a factor of two.
- **The two share-independent witnesses disagree with each other**, and this is carried forward:
  `Σ within-rule A_eff` is LINEAR but weakly (r² = 0.589, and non-monotone in k), while `recurrent`
  is SUPER-LINEAR and cleanly so (b = +2.243, r² = 0.992, 20.67×).

---

## 1. C1 — the two mechanisms

Two families are built. They are second witnesses for each other, and they are **predicted to
behave differently** — two mechanisms predicted to do the same thing would be one attempt run twice.

The target, restated so it can be designed toward rather than searched for:

> **Node-count-neutral, with fresh context minted at a rate below one per firing.**

### 1.1 `swap` — topology churn at a fixed node set (`rules/swap.toml`)

**Mechanism.** Match two node-disjoint 2-edges `{a,b}` and `{c,d}`; replace them with `{a,d}` and
`{c,b}`. This is the canonical Markov move on the space of graphs with a **fixed degree sequence**.
Node count, edge count and every individual node's degree are all exactly preserved. Fresh contexts
arise from wiring configurations the walk has not yet visited; old contexts recur when it returns to
a configuration class. The configuration space is large but finite, so novelty should be high early
and recurrence should accumulate as coverage grows.

**Parameters: none.** No rate, no threshold, no period. Binding rule 3 forbids sweeping parameters
until the criterion is met; this rule has nothing to sweep, which is the cleanest possible
compliance and is the main reason it is the primary mechanism.

**Committed predictions:**

| quantity | prediction |
|---|---|
| `minted/event` | **0.0000 exactly** — stronger than "< 1"; nothing is created or destroyed |
| `n(t)`, `e(t)` | both **exactly flat, forever** |
| S3 degree signature | **DEGENERATE at \|A\| = 1** — degrees never change, so the radius-free witness must see nothing |
| `\|A\|/n_events` | **well below 1** |
| recurrent count | **GROWS with horizon** |
| `A_eff` | **> 4**, i.e. not a flat rule in disguise |
| C3.2 verdict | **gap occupied** |

**Known and accepted:** if `{a,d}` already exists the swap creates a parallel edge. That is a
legitimate point in the configuration space of multigraphs with this degree sequence, and the degree
sequence is still exactly preserved. It is recorded, not guarded away. A double-edge swap can also
disconnect a graph; C2's edge and connectivity trajectory is where that shows up.

**Substrate:** `PureCrystal(6×6, arity 3)` — 36 nodes of 2-edges, the A-track's
`triangle_subdivide` substrate, bare and already characterised.

### 1.2 `cyclic3` — state churn at a fixed topology (`rules/cyclic3.toml`)

**Mechanism.** A node in state `L` advances to `L+1 (mod 3)` iff it has a neighbour already in state
`L+1`. The Fisch–Gravner–Griffeath cyclic cellular automaton, which on a lattice from random initial
states self-organises into rotating spiral waves that persist indefinitely. Topology is never
touched; the only thing that moves is WL-visible node state (WL hashes the *label*, and the
properties map is excluded by the Q2 invariant, so state churn has to go through labels).

**Period = 3, recorded not tuned.** Two states cannot make a cyclic CA — with `L+1 = L−1` the
advance condition is symmetric, waves have no direction, and the rule degenerates to alternation.
Three is the minimum period at which the excitable cycle exists, and it also matches the 3-cycles of
the triangle lattice it runs on.

**The ALife-artifact risk, stated before measuring.** The ALife metabolic channel had exactly three
values and that turned out to *be* the channel — a property of the encoding, not the dynamics. A
3-state rule is exactly that shape. The cycle length is therefore reported beside every number, and
the alphabet is read against the ceiling the state count imposes: at radius 1 a context is
(own state, multiset of neighbour states), so `|A| ≤ 3 × (multisets of 3 states over degree ≤ 6)` —
a few hundred at most, **by construction and independently of the dynamics**.

**Committed predictions:**

| quantity | prediction |
|---|---|
| `minted/event` | **0.0000 exactly** |
| `n(t)`, `e(t)` | both **exactly flat, forever** |
| alphabet | **PLATEAUS** |
| recurrent count | **FLAT with horizon** |
| `A_eff` | modest |
| C3.2 verdict | **"flat rule in disguise" — predicted to FAIL to occupy the gap** |

**Substrate:** `LabelledCrystal(6×6, arity 3, states = 3, seed 42)` — a crystal with labels drawn
uniformly from `{0,1,2}`. Required by mechanism: on a uniform crystal no node has a neighbour one
state ahead and nothing fires.

### 1.3 The balanced mint-and-retire family is NOT built, and why

C1.3 proposed minting a node every *k* firings and retiring one, so `minted/event = 1/k` explicitly.
**That family is already instantiated in the zoo by `bounded_churn`**, and already measured: its
`churn` rule mints one and retires one per firing, and `churn` is 1.7–4.2% of ruleset events, so the
ruleset-level `minted/event` is already ≈ **0.04** — well below 1. Its measured behaviour is the
family's answer: `A_eff` 3.57 post-burn-in, zero singletons, **PLATEAU on both abscissae**, recurrent
count flat.

Building a second one would vary only *k*, which is exactly the parameter sweep binding rule 3
forbids. The family's result is therefore **reported from `bounded_churn`** rather than
re-manufactured, and the two rules actually built attack the gap through mechanisms `bounded_churn`
does not use.

---

## 2. C2 — the bare characterisation gate

For each rule, before any alphabet comparison and before any other track may use it:

- **C2.1** `n(t)` and `e(t)` trajectories (flatness is a *design claim* and is asserted, not
  assumed), `d_eff`, regime, and a **long-run edge trajectory** — B-track found `bounded_churn`
  thins to 36 nodes / 24 edges after 2400 steps, so "node count flat" does not imply "structure
  preserved". Connectivity (component count) is reported too.
- **C2.2** liveness and perpetuality probe, **fire counts only**, exactly as B1.1 did. Three of four
  zoo candidates died here.
- **C2.3** zoo entry.

**Gate:** a rule that is not live-and-perpetual at flat `n` is not a C-track rule, however good its
alphabet looks over a short window.

---

## 3. C3 — the measurement

**Horizons: 2000 steps and 8000 steps (4×).** 2000 is the A-track's flat-rule horizon (`glider`
2000, `bounded_churn` 2400); B2.2 showed a 16× increase moving nothing, so two rungs suffice to
detect dilution.

Per rule and per rung: `|A|`, `A_eff`, **within-rule `A_eff`** (C0), singletons, **recurrent**,
`|A|/n_events`, the refinement sweep, `R*` **subject to amendment 1**, and novelty curves on **both**
abscissae.

### 3.2 The gap criterion, pre-committed

| condition | reading |
|---|---|
| `\|A\|/n_events` well below 1 **and** recurrent count **grows** with horizon | **GAP OCCUPIED** |
| `\|A\|/n_events` below 1 but recurrent count **flat** | **PARTIAL** — bounded structure, no accumulation; a third class, not the target |
| `\|A\|/n_events` → 1 | **GROWING RULE IN DISGUISE**; B2.1 generalises further than the zoo |
| alphabet plateaus at `A_eff` ≲ 4 | **FLAT RULE IN DISGUISE**; bimodality holds |

Thresholds fixed here: "well below 1" is `< 0.5` (amendment 1's bar, reused). "Grows" is
`recurrent(8000) / recurrent(2000) ≥ 1.5`; "flat" is `< 1.5`. `→ 1` is `≥ 0.99`. "Plateaus at
`A_eff ≲ 4`" is `A_eff(8000) ≤ 4.0`.

**Evaluation order, committed before any C3 number exists** — an unstated precedence is a free
parameter, and this criterion has four readings that can co-fire:

1. `|A|/n_events ≥ 0.99` → **GROWING RULE IN DISGUISE** (the ceiling clause wins even if recurrence
   grew, because a rule riding the ceiling has no recurrence to speak of);
2. else `A_eff ≤ 4.0` → **FLAT RULE IN DISGUISE** (wins even if the ceiling is low);
3. else `|A|/n_events < 0.5` **and** recurrence growth `≥ 1.5` → **GAP OCCUPIED**;
4. else `|A|/n_events < 0.5` → **PARTIAL**;
5. else → **INDETERMINATE**.

**Firing test (binding rule 5):** `tests/c3_firing.rs` feeds the classifier synthetic rows with
known answers, including one for **each** of the four readings. A criterion that cannot return
GAP OCCUPIED cannot support "not occupied", and one that cannot return FLAT RULE IN DISGUISE cannot
support "occupied".

### 3.3 / 3.4

Scale-invariance across the two rungs via B2.4's `classify_ceiling`, span and fall reported. And the
B1 series extended to **k = 5** with the surviving C-track rule appended, on the same
`CompositeChurnRing` substrate with the same flattened priorities and both horizons — B-track open
item 3 named exactly this.

---

## 4. What this pre-registration does not decide

Which C4 branch the data selects. And nothing here reopens the endogenous-rule-encoding argument,
which B1 resolved against; C-track evidence is not admissible for it either way.
