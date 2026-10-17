# E0 — the quantizer audit

Phase E0 of `hypersim_nn_experiments_plan_v3.md`, §6.1 and §10. Produced by
`crates/hg-train/tests/e0_quantizer_audit.rs`.

> **All six fixed scales in the learning layer are mis-calibrated against the
> substrates they run on, and three are seriously so.** The policy's
> `priority` input is **identically zero on both D6 substrates** — the same
> `used = 0.000` score F4's grid earned. Its `candidate count` input is
> **saturated at 1.0 on 78.9% of steps** on `tri_two`, where the median
> observable is 9198 against a design range of 4096.
>
> D6's champion learned through these. That does not invalidate the margin —
> every arm shared the same inputs — but it means the effective input space
> is materially smaller than the reported feature dimension, and plan v3
> §6.1 was right that this is catchable before a run.

---

## 1. What counts as a quantizer

`grep` finds 31 `floor`/`round`/`ceil`/`clamp` sites across the workspace.
Most are not instances of this defect class, and saying which and why is part
of the deliverable — a registry that lists everything is as unusable as one
that lists nothing.

The defect class is **a fixed scale applied to a continuous or unbounded
observable, where the scale's design range is a constant and the
observable's actual range is a substrate property that nothing verifies.**

Excluded, with reasons:

| excluded | why |
|---|---|
| `jsd.clamp(0,1)` (`hg-eval/window.rs`), `interestingness.clamp(0,1)` (`classify.rs`) | saturation guards on mathematically-bounded quantities. The bound is a theorem, so there is no observable-range question |
| rank/quantile computations (`hg-observe/locality.rs`, `hg-eval/chains.rs`, `hg-sim-core/twin.rs`, F4's `quantile_symbols`) | data-adaptive by construction, immune to this defect — which is exactly why F4's fix was to become one |
| `hg-gui/*` | display, not measurement |
| CMA-ES's `clamp(-1,1)` on an exponent, population sizing | not observables |

What remains is six scales: four in the **policy's own input**
(`hg-cognition/features.rs`) and two in the **probe readout**
(`hg-cognition/readout.rs`).

`used` below is the fraction of the scale's `[0, 1]` output range the
substrate actually occupies. **F4's grid scored 0.000 by this metric**, which
is what makes it comparable.

---

## 2. The policy's input scales

384 steps, `k = 4`, budget-matched random, measured on the raw observables
the feature extractor consumes rather than on its scaled output — reading the
output would only report what the scale already decided.

| observable | design | `tri_two` obs range | used | sat | `wn_three` obs range | used | sat |
|---|---|---|---|---|---|---|---|
| priority (tanh) | `PRIORITY_SCALE = 8` | 0 – 0 | **0.000** | **100%** | 0 – 0 | **0.000** | **100%** |
| arity (linear) | `ARITY_SCALE = 8` | 2 – 3 | 0.125 | 0% | 2 – 3 | 0.125 | 0% |
| node degree (log) | `DEG_SCALE = 64` | 2 – **69** | 0.737 | 0.3% | 1 – **130** | 0.834 | 6.0% |
| candidate count (log) | `CAND_SCALE = 4096` | 1152 – **16974** | **0.152** | **78.9%** | 150 – **9888** | 0.397 | 49.0% |

### 2.1 `priority` is a dead input

`tanh(0/8) = 0` on every candidate of every step of both substrates, because
`rules/tri_two.toml` and `rules/wn_three.toml` both flatten every priority to
`0`. That flattening is *correct* and deliberate — both files document it, so
that "the policy chose rule X" is not confounded with "the conflict resolver
preferred rule X".

But the consequence was never followed through: the feature computed from
that field is then a **constant**, and a constant input to a linear layer
contributes exactly its bias. The MLP's nominal `feat_dim` is
`n_rules + wl_buckets + 7`; its effective dimension on both D6 substrates is
one less.

This is F4's defect with the sign flipped. F4's quantizer was too coarse to
see a moving observable; here the observable genuinely does not move, and
nothing in the instrument says so. Either way a feature reports a constant
and the report calls it a feature.

### 2.2 `candidate count` is saturated where it matters most

`CAND_SCALE = 4096` against a `tri_two` median of **9198** and a max of
16974. The feature is pinned at 1.0 for **78.9% of steps** — so for four
steps in five, the policy cannot tell a 5000-candidate pool from a
17000-candidate one.

This is the most consequential of the four, because candidate count is the
one feature that carries *scarcity* — and C-1 established that scarcity is
the precondition for selection to have causal power at all. The policy's
access to the variable its own relevance depends on is pinned for the
majority of the run.

`wn_three` at 49.0% is not better in kind, only in degree.

### 2.3 `node degree` is clipped at the top

`DEG_SCALE = 64`, observed max **69** on `tri_two` and **130** on
`wn_three`. Only 0.3% / 6.0% of values clip, so the feature is mostly
informative — but `wn_three` runs a hub structure whose top decile the
feature cannot distinguish. Under a `binary_edge`-dominated pool that hub is
not incidental.

### 2.4 `arity` is compressed, not broken

`ARITY_SCALE = 8` against arities of 2 and 3. The feature takes exactly two
values, `0.25` and `0.375`, occupying 12.5% of its range. Nothing is lost —
two values map to two values — but the network must resolve a
one-eighth-scale difference against inputs that use their full range. Worth
declaring; not worth an urgent fix.

---

## 3. The readout's scales

| substrate | observable | design | obs range | used | sat |
|---|---|---|---|---|---|
| `tri_two` | mean degree | `DEG_SCALE = 64` | 4.42 – 5.61 | **0.048** | 0% |
| `tri_two` | graph size | `SIZE_SCALE = 65536` | 83 – 1539 | 0.262 | 0% |
| `wn_three` | mean degree | `DEG_SCALE = 64` | 2.75 – 3.48 | **0.043** | 0% |
| `wn_three` | graph size | `SIZE_SCALE = 65536` | 37 – 1388 | 0.324 | 0% |

Neither saturates, and both are compressed. The mean-degree columns are the
notable ones: **the entire substrate's mean-degree variation occupies under
5% of the feature's dynamic range.**

That connects to a D4 result. Stage 0b′ concluded "r2 does not repair the
ratio" and correctly located the fault in the denominator rather than the
readout. The audit adds a second fact in the same direction: r2's degree
moments barely move, so r2 was a three-times-wider readout carrying little
additional variation — which is also consistent with Stage 1′'s memory
capacity not moving (0.0390 → 0.0364) under it. **The r2 null is not
explained by the compression, but the compression is a reason not to have
expected r2 to help much.**

---

## 4. What this does and does not change

**Does not change.** D6's margin. Every arm — random, untrained, champion —
consumed the same features through the same scales, so a mis-scaled input is
not a confound between arms. The +382.02 / +341.09 stand.

**Does change.** What the result is a result *about*. §3 of plan v3 says the
champion "learns something a uniform-over-rules policy cannot express". True,
and now more precisely: it learns it from an input space that is **one
feature dead, one feature pinned for 79% of steps, one clipped at the top
decile, and one compressed eightfold**. The learnable signal available to a
correctly-scaled policy is strictly larger than what D6 searched.

**Decided and built (§4A).** Rescaling changes the policy's input space,
making any pre-fix champion incomparable with any post-fix one — the
identical argument plan v3 §4 makes for N-1 requiring a D6 re-run. **E2 is
already a D6 re-run**, so E1/E2 is the only window where the fix is free.
Taken.

---

## 4A. The fix — `feature_set = "v2"`

Built, not deferred. v2 changes **only the two scales that saturate** and
leaves the two that merely compress:

```
degree          →  rel = deg / mean_degree,          then rel / (1 + rel)
candidate count →  rel = n_candidates / node_count,  then rel / (1 + rel)
```

`x / (1 + x)` is strictly monotone on `[0, ∞)` into `[0, 1)`, so it **cannot
reach either endpoint** — no substrate, at any magnitude, can drive either
feature to a constant. `mean_degree` is `edge_count / node_count`, O(1) from
the snapshot, so there is no extra pass over the candidate pool. Measured:

| substrate | observable | obs range | v1 sat | **v2 sat** | v2 used |
|---|---|---|---|---|---|
| `tri_two` | node degree | 2 – 69 | 0.3% | **0.0%** | 0.494 |
| `tri_two` | candidate count | 1152 – 16974 | **78.9%** | **0.0%** | 0.489 |
| `wn_three` | node degree | 1 – 130 | 6.0% | **0.0%** | 0.622 |
| `wn_three` | candidate count | 150 – 9888 | **49.0%** | **0.0%** | 0.779 |

`used` rose too — `tri_two`'s candidate count from 0.152 to 0.489 — but that
is a **side effect and explicitly not the target**. Choosing constants to
maximize observed spread would be fitting the instrument to the observations,
which `instrument_registry.md` §1.1 rejects. The claim v2 makes is only that
saturation is now structurally impossible, and the assertion in
`v2_encodings_do_not_saturate` is on the saturation column, not on `used`.

### 4A.1 What v2 deliberately leaves alone

- **`arity`** (used 0.125, 0% saturated) — two observable values map to two
  distinct feature values. No information is lost, so there is nothing to
  fix. Widening it would be tuning.
- **`priority`** (constant `0`) — the observable does not move, and rescaling
  a constant yields a constant. This is a **substrate fact**, not a scale
  fault: both rule files flatten priorities deliberately, and correctly. The
  honest fix is *reporting* that a feature is dead, not re-encoding it.
- **Both readout scales** (used 0.048–0.324, 0% saturated) — compressed,
  lossless, untouched.

### 4A.2 Compatibility

`feat_dim` is unchanged at `n_rules + wl_buckets + 7`, so genome length and
every campaign mechanism carry over — only the values differ. **v1 remains
the default**, because a champion genome trained under one normalization is
meaningless under another; v2 is opt-in via `feature_set`, and E2's re-run is
where it is first used. `FeatureExtractor::new` and `MlpScorer::new` keep v1
semantics so no existing call site silently changes meaning.

---

## 5. Declared observable ranges

Per §6.1, for `instrument_registry.md` v4. These are the measured ranges on
the two substrates the programme owns; a new substrate (§8) re-opens each
row, which is the point of declaring them.

| quantizer | consumes | design range | measured range | blind spot |
|---|---|---|---|---|
| `features.rs` priority | rule priority, `i32` | unbounded via `tanh(x/8)` | `{0}` on all owned substrates | cannot report that its observable is constant |
| `features.rs` arity | bound node count | `[0, 8]` | `[2, 3]` | nothing — under-uses range, loses no information |
| `features.rs` degree | node incident-edge count | `[0, 64]` | `[1, 130]` | values above 64 are indistinguishable |
| `features.rs` candidate count | pool size per step | `[0, 4096]` | `[150, 16974]` | pools above 4096 are indistinguishable; 49–79% of steps |
| `readout.rs` degree | window mean degree | `[0, 64]` | `[2.75, 5.61]` | nothing — under-uses range |
| `readout.rs` size | window node count | `[0, 65536]` | `[37, 1539]` | nothing — under-uses range |

**The distinction the blind-spot column has to carry:** under-using a range
loses no information (arity, both readout scales), while saturating one
destroys distinctions the substrate actually makes (candidate count, degree)
and a constant observable destroys the feature entirely (priority). Only the
second and third kinds are defects. A registry that reported "used < 1" as
uniformly bad would flag three sound instruments and rank them alongside the
dead one.

---

## 6. Raised for the plan, not decided here

1. **`priority` on authored substrates (§8).** If new substrates carry
   non-flat priorities, the feature becomes live and D6's comparability
   breaks in a way no re-run fixes — the input space would differ *between*
   substrates. Worth deciding before authoring, not after.
2. **`CAND_SCALE` is now moot** — v2 removes the constant rather than
   choosing one, which was the only option that survives a 60× difference in
   observed medians between two substrates. Recorded because the reasoning
   generalizes: when a scale constant cannot serve two known substrates, it
   will not serve an unknown third, and the fix is to remove the constant.

---

## 7. Provenance

| what | where |
|---|---|
| §2's policy input table | `e0_quantizer_audit.rs::observable_ranges_against_design_ranges` |
| §3's readout table | `…::readout_observable_ranges` |
| §4A's v2 table | `…::v2_encodings_do_not_saturate` — asserts 0% saturation, not a `used` figure |
| §4A's encoding, pinned | `hg_cognition::features::tests::v2_cannot_saturate_the_two_scales_v1_pins` — also asserts v1 really does collide degree 69 with 130 |
| the metric itself | `…::the_audit_metric_catches_the_f4_grid` — asserts the v2 F4 grid scores `used = 0.000` on the real JSD band, and that quantile cutting scores > 0.99 on the same data |

The third is a live (non-`#[ignore]`d) test. If the audit's metric ever stops
scoring F4's grid at zero, the metric has stopped measuring the thing the
audit exists to catch.
