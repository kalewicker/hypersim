# V9 / N-track — the metabolic instrument is validated, and it answers a *different* question
# than the structural one

Harnesses: `n0_burnin_criterion.rs`, `n1_metabolic_instrument.rs`, `n1_validation.rs`,
`n1_reproducibility.rs`, `n2_two_channel.rs`, `n4_3_retention_on_churn.rs`.
Pre-registrations committed before their numbers: N0 v1 `d853f94` (**failed**), N0 v2 `18dcd28`,
N3 scope `6613973`, Phase 1 `19ebd70`, N2.1 expectation table `1f8e64d`.

---

## The two sentences the plan asked for

**End of Phase 1:** the closure-defect estimator is **not** a validated statistic — it *degenerates*
as a within-channel boundary measure — but **CE on the metabolic channel is**: null-tested with the
spread guard applied from birth, 30/30 multi-seed, two-process reproducible, and stationary under
bounded churn post-burn-in.

**End of Phase 2:** the channel that carries the causal work **does not agree** with the structural
channel about what property is load-bearing — but not by reversing it. **The metabolic channel
excludes nothing in the ladder**: every live arm detects, and the only thing it separates is *active*
from *inert*. And the structurally-invisible arm D is **not** metabolically the loudest on the
validated statistic (4th of 5) — it is loudest only on a raw activity rate that is not a null-tested
boundary statistic.

---

## N0 — the burn-in criterion, and a criterion of mine that failed

**v1 failed and is in the record.** I committed absolute thresholds (block Jaccard ≥ 0.90 at
`L_blk = 100`). They rejected churn-with-carriers — and then, on the calibration case v1 had omitted,
**they also rejected the F8 static baseline** (F8 runs 0.55–0.76, never 0.90). A criterion that
rejects the run every published structural result rests on is measuring block size, not stationarity,
so v1's churn rejection meant nothing.

Root cause: I used **absolute** thresholds where this program's own precedent (H0.1) is
**baseline-relative** — *"whatever the F8 static run exhibits defines the tolerated floor."*

**v2** derives `J_ref = 0.5484` and `D_ref = 0.3333` **from arm A**, so the numbers are outputs, not
parameters I could shop toward a verdict.

| run | v2 `T_burn` | v1 |
|---|---|---|
| static bare | 0 | 0 |
| static F8 (A) | **0 (accepted)** | **rejected** |
| CHURN bare | 100 | 100 |
| CHURN + carriers | **900** | rejected |
| growth_treadmill | none | none |
| triangle_subdivide | none | none |

**N0.2:** the derived 900 is **below** the M-track's post-hoc 1000, so that choice was *conservative*
— the M1 correction stands, now on a derived rather than post-hoc basis, which is what N0 existed to
establish. **N0.3 firing test passes**: both growing proxies and the pre-settling window are rejected.

## Phase 1 — one instrument validates; the project's own construct does not

**Depth `R = 2`**, fixed by the pre-registered arm-A-only rule and frozen.

**The closure defect degenerates**, demonstrated numerically rather than asserted: `CD ≡ H(micro) −
H(macro)` **exactly, in all six arms**, because the macro group is a deterministic *function* of the
micro class, so `H(micro, macro) = H(micro)`. Since `H(micro)` is permutation-invariant, the
permutation test on `CD` is exactly a test on `−H(macro)`. It survives only in its M3 **cross-channel**
form (macro = WL structural, micro = energy — two genuinely different labelings). Its direction was
**not** flipped post-hoc to rescue it.

**CE on the metabolic channel passes every gate:**

| gate | result |
|---|---|
| N1.1 null + spread guard from birth | all live arms non-degenerate (188–200 distinct); **inert correctly degenerate** |
| N1.2 multi-seed (arm A, seeds 1–30, disjoint) | **30/30**, sign test p = 1.9 × 10⁻⁹, no reversals ⟹ regime property |
| N1.3 inert zero-point as a measured band | **unresolvable on 10/10 seeds** (`k = 1`, `M = 3`), activity band exactly 0 |
| N1.3 two-process reproducibility | scalars bit-identical; sorted-order registry bit-identical |
| N1.4 stationarity under bounded churn | TWTF 0.000, block-Jaccard 0.957 vs baseline 0.975 |

**N1.3 fired, and caught a real hazard.** The index-dependent digest differs across processes: the
class *set* and every row sum agree, but the **registry indices** they land on differ, because the
per-step node maps were iterated in `std::HashMap` order and `RandomState` is seeded per process.

- **Scalars are unaffected.** CE, EI and the permutation p-value are order-invariant, and F3's
  `canonical_order()` already shuffles the null in hash-sorted order for exactly this reason. **No
  published result is retracted.**
- **But any future statistic reading the TPM by index would be non-reproducible** — precisely the §6
  failure. The instrument now records in sorted `NodeId` order, asserted bit-identical.

**Two N1.4 caveats, stated not glossed:** (a) the metabolic partition is stationary on the *growing*
proxies too, because its state space is **bounded by construction**; that is a real advantage of this
channel, but it makes witness 2 a weaker check here than it was on WL. (b) `triangle_subdivide`
collapses to `M = 1`, so its "stationary" is **trivial** and is not a pass.

**N1.5 convergent witnesses:** all three rank arm C **last** (unanimous), but they **disagree at the
top** — CE and energy-transition entropy say arm A; the raw activity rate says arm D. Both runners-up
needed **window-normalisation** to place the dead token below arm D; that repair is *principled* (an
arm that stops early should contribute less over a fixed window) but was identified **after** seeing
the table, so it is labelled post-hoc.

## Phase 2 — the two-channel table

| arm | structural (WL d2) | metabolic (Δ-history d2) |
|---|---|---|
| A replicating | detect, z = 6.56 | detect, **z = 20.16** |
| B motile static | detect, z = 2.28 | detect, z = 14.72 |
| B′ motile diffuse | detect, z = 1.18 (p = .010) | detect, z = 17.96 |
| C token | **no** (z = −0.83, p = 0.930) | **detect**, z = 9.12 |
| D stationary | **degenerate/vacuous** (sd = 0) | **detect**, z = 11.72 |
| inert control | no (degenerate) | **no (unresolvable)** |

**Per-arm multi-seed:** every arm **30/30 on the metabolic channel** (sign p = 1.9 × 10⁻⁹). Arm C:
**0/29** structural. Arm D: **0 included** structurally — **29 of 30 seeds degenerate-null**.

### The finding that matters most, and it is a narrowing

**The metabolic channel excludes nothing in the ladder.** Every live arm detects, on 30/30 seeds. The
only line it draws is between an **active carrier and an inert one**. So the metabolic re-run does
**not** relocate the ladder's property verdicts — it cannot, because it does not discriminate among
replication, turnover, motility or presence at all.

- **Structural channel:** requires sustained structural *activity* — C and D fail.
- **Metabolic channel:** requires only that a carrier be metabolically active — nothing in the ladder
  is load-bearing, and the inert control is the sole negative.

This is the pre-committed "metabolic channel gives a different answer" branch, and the honest reading
is that the structural findings are **relocated, not retracted**: they remain real facts about the
structural channel, while the metabolic channel supplies a *coarser* account (carrier-vs-substrate
detectability) rather than a competing property verdict. Neither is adjusted to reconcile them.

### The headline question, answered without flattery

**"Is the structurally invisible arm D the loudest metabolic signal?"** — **statistic-dependent:**

- On the **Phase-1-validated boundary statistic (CE): NO.** A (20.16) > B′ (17.96) > B (14.72) >
  **D (11.72)** > C (9.12).
- On a **raw per-carrier activity rate: YES.** D (0.9998) > B′ > B > A > C.

They answer different questions — CE asks whether the *partition* is informative about the dynamics;
the activity rate asks how much an average carrier's energy moves. **The quotable form of the shadow
result is true of the activity rate and false of the validated statistic**, and that distinction is
the finding.

### What *is* directly confirmed

**Arm D is "invisible, not inert"** — now demonstrated rather than inferred. Structurally its null is
degenerate on 29/30 seeds (the graph is constant, so the test is vacuous); metabolically it detects
with a non-degenerate null on 30/30. The L-track diagnosis is confirmed on an instrument that can see
the channel D was active in.

**The metabolic CE instrument is not the energy-band deflation repeating itself.** The energy-band
partition scored the *dead token* as high as arm A (~98% definitional). Here the **inert carrier —
definitionally a carrier, sitting in the carrier band — is unresolvable on 10/10 seeds**, and the dead
token ranks *last* of the five live arms. That is the discriminating evidence the energy-band
partition could not produce.

## N4 — supporting items

- **N4.1 (confirmatory arm C).** Phase 2's per-arm multi-seed **confirms C's structural non-detection
  at regime level on a fresh disjoint seed set (0/29)**, converting the single-seed exploratory
  finding to confirmatory. The *empty-fraction sweep itself* was not re-run under pre-registration, so
  that specific exploratory provenance stands.
- **N4.2 (guard gaps).** **Gap 1 is now closed**: the null-spread inclusion clause, which had never
  fired on a sustained seed, fired **29 times** on arm D's structural channel in Phase 2 — exactly the
  positive firing case that was missing. **Gap 2 remains open** (M4.4's null-validity check still has
  no dedicated firing case).
- **N4.3 (retention on churn).** Retention **rises under churn** for the motile arm (B: 3.86% →
  7.12%) and stays ~0 for A (0.14% → 0.13%) and D (0.00% both). So **the exact "~0%" is partly a
  property of the static substrate**, though retention stays ≤ 7%, so the qualitative shadow result
  survives while the precise figure does not. Per rule 2 this is **class-level information
  retention**, never causal independence: the channels are coupled by construction (energy crossing
  zero *causes* death, a structural event), and the coupling washes out because WL classes pool
  carriers at different energies.

## N3 — scope, written before Phase 2 ran (`6613973`)

Selection is settled **degenerate-only at power** (M4.4, 30 seeds). The system has **metabolism but no
meaningful evolution**. A metabolic boundary is a legitimate object and this track established one; an
**evolutionary** boundary is not available on this substrate and no metabolic result can supply it.
The tri-ecology gate's premise is **contradicted, not merely unmet**, and must be re-examined on its
own terms rather than treated as waiting downstream.

## Standing caveats

- The Phase-2 expectation table was **not blind** — N1.1 requires per-arm nulls while N2.1 requires a
  table committed before scoring, and both cannot hold. Every row is flagged `[SEEN]`/`[NEW]`; the
  genuinely predictive content was the per-arm multi-seed sweep.
- N0 v2's `J_ref`/`D_ref` are derived from a **single** reference run (arm A, seed 12345).
- The metabolic micro-state bucketing (delta magnitude classes) is a design choice made once and
  frozen; it was not swept.
- Magnitude remains out of scope (F2.4). Everything here is detectability.
