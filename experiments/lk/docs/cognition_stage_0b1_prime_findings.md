# Stage 0b′ and Stage 1′ — the separation and memory verdicts under r2

Phase D4 of `hypersim_nn_experiments_plan_v2.md`, §9. Produced by
`crates/hg-train/tests/d4_stage0b_stage1_r2.rs`.

> **Stage 1′ confirms its pre-registered null.** MC moves 0.0390 → 0.0364
> under a readout three times as wide, against a dimension-matched ESN. No
> delay ≥ 1 is decodable under either readout. The Stage 1 null is real.
>
> **Stage 0b′ cannot report a class, and the reason is not the readout.**
> The separation ratio exceeds its own hard bound on a **majority of seed
> triples** on `growth_treadmill` (5/8) and `triangle_subdivide` (5/8),
> under r1 **and** r2. Every Stage 0b class ever published — including the
> `triangle_subdivide` "Viable, ratio 0.752" that plan v2 §9 asks to
> re-confirm — is **one draw from a distribution spanning two classes**.

---

## 1. Stage 1′ — memory capacity under r2

`rules/stage1_reservoir.toml`, `Empty` IC, 4096 steps, window 32, `k = 4`,
max delay 12.

| readout | state dim | MC | ESN MC (matched dim) | input `r²` at d=0 | `r²` at d=1 | max decodable delay |
|---|---|---|---|---|---|---|
| r1 | 16 | **0.0390** | 5.8237 | 0.9261 | 0.0170 | **none** |
| r2 | **48** | **0.0364** | 7.2609 | **1.0000** | −0.0264 | **none** |

### 1.1 The decision rule, resolved

Plan v2 §7: *"re-run under r2. If MC stays at ~0.039 the verdict is real
and the substrate's intrinsic timescale is genuinely shorter than one
window. If it moves materially, the factor of 65 was readout."*

**MC stays.** 0.0390 → 0.0364, a 7% move and in the *wrong direction* for
the readout hypothesis — a richer projection found slightly less decodable
memory, not more. Tripling the state dimension bought nothing.

So the pre-registered failure reading applies as written: **the substrate's
intrinsic timescale is shorter than one rewrite window**, at every window
tested, and the Stage 1 null is a substrate result rather than a projection
artifact. It is publishable as *memory capacity of hypergraph rewriting
systems is bounded below one rewrite window*.

### 1.2 The sharpest number is the one that did move

`input_coupling_r2` at delay 0 goes **0.9261 → 1.0000**.

Under r2 the readout recovers the injected symbol **perfectly** — and there
is still nothing at any delay ≥ 1. That is much stronger than r1's version
of the same signature. The competing hypothesis was "the readout resolves
the symbol but not its consequences"; a readout that resolves the symbol
*exactly* and finds no trace of it one window later is not the readout
failing.

This is the one place where the r1 number could not discriminate on its own
(§7 says so explicitly, and it was correct to say so). r2 discriminates.

### 1.3 The ESN comparison, now actually dimension-matched

v1 compared the substrate's 16-bucket readout against a 16-unit ESN, so it
*was* dimension-matched at 16 — the plan's "the v1 comparison was not
dimension-matched" is not accurate as stated. What was never matched is
**information content per dimension**: an ESN unit is a real-valued state
driven directly by the input, a WL bucket is a lossy hash of the graph.

The fix implemented here is that the ESN is now sized from the **probe
rows** rather than from `buckets`, so the match holds under any readout. It
changes nothing under r1 (the two coincide) and matters under r2, where
using `buckets` would have pitted a 48-dimensional substrate readout
against a 16-unit ESN and called the ratio a substrate property.

| readout | substrate / ESN |
|---|---|
| r1 (dim 16) | 0.0067 — a factor of **149** |
| r2 (dim 48) | 0.0050 — a factor of **200** |

**Correction to the plan's arithmetic:** §7 quotes "the factor of 65". The
measured r1 ratio is `5.8237 / 0.0390 = 149`, not 65. Nothing downstream
depended on 65, but the figure should not be cited.

---

## 2. Stage 0b′ — the separation ratio is not a valid statistic

### 2.1 What a single triple says

At the C0–C5 configuration exactly — base seed 4242, `budget = 4` on all
five rule sets:

| rule set | readout | plateau | baseline | ratio | class |
|---|---|---|---|---|---|
| `glider` | r1 / r2 | 0.0000 | 0.0000 | — | Undetermined / Undetermined |
| `growth_treadmill` | r1 | 0.0042 | 0.0025 | **1.699** ⚠ | Chaotic |
| `growth_treadmill` | r2 | 0.0011 | 0.0017 | 0.624 | Viable |
| `triangle_subdivide` | r1 | 0.0096 | 0.0128 | **0.752** | **Viable** |
| `triangle_subdivide` | r2 | 0.0013 | 0.0019 | 0.696 | Viable |
| `binary_edge` | r1 / r2 | 0.0000 | 0.0000 | 0.995 / 0.994 | Undetermined |
| `fl1_two_rule` | r1 | 0.0101 | 0.0093 | **1.095** ⚠ | Chaotic |
| `fl1_two_rule` | r2 | 0.0017 | 0.0018 | 0.935 | Chaotic |

r1 reproduces the published `triangle_subdivide` number **exactly** —
0.752, Viable — which is what licenses reading any r1/r2 difference as a
readout effect. (Getting there required fixing a confound of my own: the
first draft of this pass varied the budget per fixture, which mixes the
readout contrast with a budget contrast. C0–C5 used `k = 4` throughout and
so does this.)

Read alone, this table says: two r1 ratios exceed their hard bound, r2
repairs both, and `growth_treadmill` flips Chaotic → Viable.

**All three of those readings are wrong.**

### 2.2 What eight triples say

`stage0b` derives its perturbed partner *and* its independent reference
from one base seed, so **a single call is a single seed triple, and nothing
in C0–C5 ran more than one.** Over 8 triples:

| rule set | readout | ratios > 1 | median | max | classes over 8 triples |
|---|---|---|---|---|---|
| `glider` | r1 | 0/0 | — | — | Undetermined ×8 |
| `glider` | r2 | 0/0 | — | — | Undetermined ×8 |
| `growth_treadmill` | r1 | **5/8** | 1.310 | 2.086 | Chaotic ×7, Viable ×1 |
| `growth_treadmill` | r2 | **5/8** | 1.321 | **4.238** | Chaotic ×6, Viable ×2 |
| `triangle_subdivide` | r1 | **5/8** | 1.240 | 1.703 | Chaotic ×6, **Viable ×2** |
| `triangle_subdivide` | r2 | **4/8** | 1.072 | 1.540 | Chaotic ×7, Viable ×1 |
| `binary_edge` | r1 | 3/7 | 1.000 | **15.818** | Undetermined ×8 |
| `binary_edge` | r2 | 3/7 | 1.000 | 15.839 | Undetermined ×8 |
| `fl1_two_rule` | r1 | 3/8 | 0.840 | 1.735 | Chaotic ×5, Viable ×3 |
| `fl1_two_rule` | r2 | 2/8 | 0.797 | 1.535 | Chaotic ×3, **Viable ×5** |

Three corrections to §2.1, in order of how much they matter:

**1. The `growth_treadmill` flip is noise.** r2 classifies it Viable on
**2 of 8** triples. The single-triple result I was about to report was one
draw.

**2. r2 does not repair the ratio.** It exceeds the bound on 5/8 triples
for `growth_treadmill` and 4/8 for `triangle_subdivide`, reaching **4.238**
— worse than anything r1 produced on that rule set. This was never a
readout problem.

**3. Every published Stage 0b class is a single draw from a
two-class distribution.** `triangle_subdivide`'s "Viable, ratio 0.752" —
the one the plan asks to re-confirm as a consistency check — comes back
**Viable on 2 of 8** and Chaotic on 6. `fl1_two_rule` is Chaotic ×5 under
r1 and Viable ×5 under r2, which is a coin flip dressed as a verdict.

### 2.3 The diagnosis: the denominator, not the numerator

A ratio above 1 means the **perturbed pair diverged further than two
independent seeds**. No real distance can do that, so the fault is in the
normalizer (`docs/instrument_registry.md` §2.7).

The independent-seed baseline is a **single pair**. On these substrates it
is not a stable upper bound: it is one sample of a quantity with enough
spread that it lands below the perturbed pair on roughly half of draws.
`binary_edge`'s max of **15.818** is the clearest case — there the baseline
occasionally collapses to near zero and the ratio explodes, which is bug
4's denominator-collapse signature in a different measurement.

This is a defect in Stage 0b's **design**, present since C0–C5 and
untouched by the readout work. r2 was the right experiment to run and it
answered its own question — it just also revealed that the question was
downstream of a broken statistic.

### 2.4 Consequence for the exit criterion

Plan v2 §9's Stage 0b′ exit is *"every rule set has a class under r2;
r1/r2 disagreements documented."*

**Not met, and it should not be met by this instrument.** Reporting three
classes from single triples would satisfy the letter of it and would be
three coin flips. What is delivered instead:

- **`glider`** — excluded, with a proof. Its trajectory is a sequence of
  isomorphic graphs, so every isomorphism-invariant readout is constant on
  it by construction (`docs/policy_config.md` §4B.6). It produces no ratio
  at all under either readout (`0/0`), consistently across all 8 triples,
  which is exactly what that proof predicts.
- **`binary_edge`** — `Undetermined` ×8 under both readouts, and now with a
  reason: its *independent-seed baseline* is ~0. Two independent seeds do
  not diverge in the readout either, so there is no scale to measure the
  perturbed pair against. That is a joint statement about substrate and
  readout, not a substrate verdict.
- **`growth_treadmill`, `triangle_subdivide`, `fl1_two_rule`** — **no
  admissible class.** The statistic their class is computed from is invalid
  on a majority (or near-majority) of draws.

### 2.5 What would fix it

Not a readout change. The baseline needs to stop being a single pair:

1. **Average the independent-seed reference over several disjoint pairs**
   before dividing, so the denominator is an estimate rather than a sample.
2. **Report the ratio with its spread across triples**, and refuse a class
   when the spread crosses a class boundary — the same null-spread
   discipline `f4_null_sd` and the V-track's arm D already impose
   elsewhere.
3. **Keep the hard-bound check live**, so a ratio above 1 is an abort
   rather than a `Chaotic` label. It fires on real data at a rate that
   makes it the most productive guard in the programme so far.

None of that is expensive — the whole 8-triple sweep above runs in 47
seconds. It is a Stage 0b redesign, not a new experiment, and it is
recorded here rather than done because it is outside D4's scope.

---

## 3. What this does and does not change

**Unchanged.** Stage 0a, Stages 2, 4 and 5 do not read Stage 0b's classes.
The `triangle_subdivide` "Viable" label was used as *narrative* context for
choosing fixtures, never as an input to a fitness, a campaign, or an exit
criterion. Nothing published elsewhere depends on it.

**Changed.** `docs/cognition_stage0_findings.md` §2's separation table is
superseded: its classes are single-triple draws and its two ratios above 1
were admitted where the hard bound should have rejected them. The Stage 0b
section of the C0–C5 record should be read as *unmeasured* pending the §2.5
redesign — a demotion in the sense of `docs/instrument_registry.md` §4, and
not a retraction: the runs happened and the numbers are what they are; what
is missing is a statistic that can turn them into a class.

**A note on how this was found.** The single-triple table in §2.1 is
publishable-looking, internally consistent, and tells a satisfying story:
"r2 repairs two faulty ratios and reclassifies a rule set." It is also
wrong in every particular. What caught it was running the same measurement
eight times — which is the cheapest possible check and the one C0–C5 never
ran on this diagnostic.
