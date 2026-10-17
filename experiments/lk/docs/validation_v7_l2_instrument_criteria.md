# V7 / L2.1 — what an instrument must satisfy to measure the presence-vs-motility question

**Committed before any L2 candidate is evaluated and before any L1 arm is built or scored.**
Per V7 rule 3, L1's detectability number is uninterpretable unless it is read on a partition that
passes the criteria written here.

## The dilemma this must resolve

The arc's two partitions deflated in opposite directions:

| partition | discriminates biology? | stationary under node turnover? |
|---|---|---|
| WL depth 2 (structural) | **yes** — dead token C scores z = −0.83, p = 0.93 | **no** — H0: TWTF 0.515 / 0.984 under motion |
| (label, energy-band) (metabolic) | **no** — dead token C scores z = 9.59, ≈ arm A's 9.81 | **yes** — TWTF 0.000 under motion |

Neither is usable alone: WL cannot be trusted on a moving substrate, and the energy-band partition
is ~98% definitional (K3), so a "detection" on it says only that carriers are a distinct kind of
object. Per V7's forbidden list, the energy-band partition is **not** treated as
biology-discriminating anywhere in this track; it is reported only as the always-on calibrator.

## The two criteria

A candidate instrument **qualifies** iff it satisfies both:

**(a) BIOLOGY-DISCRIMINATING.** On the static F8 data, it must separate a live sustained carrier
from the dead token:
- arm A detectable: `p_A < 0.05` with `k_carrier >= 2`, `k_carrier < M`, null support `C(M,k) > 100`
  (the standing F0 bar), and
- arm C **not** detectable: `p_C >= 0.05` **and** `|z_C| < 2.0`, where `z = (real_CE − null_mean)/null_sd`
  is the K3 effect size.

The `|z_C| < 2.0` clause is what rules out the energy-band partition (z_C = 9.59) and admits WL
(z_C = −0.83). Both clauses are required: a partition that fails to detect *anything* trivially
satisfies the C clause and is excluded by the A clause.

**(b) STATIONARY.** On the substrate whose data it will score, the H0.1 criterion, verbatim:
transition-weighted transient fraction `TWTF <= 0.5` at `R_PERSIST = 3`, together with adequate
sampling and resolution (`k_carrier >= 2`, support > 100) on that substrate.

## Two tiers, stated explicitly as a deviation

The V7 plan writes criterion (b) as "stationary **under churn**". L1's specimen (see
`validation_v7_l1_sustain_mechanism.md`) is sustained by **energy diffusion on a fixed node set**,
not by node churn, because churn cannot refresh an *occupied* site without deleting the carrier's
own site. So the literal phrasing does not match the substrate L1 actually uses. Rather than
quietly reinterpret the criterion, both tiers are measured and reported:

- **Tier 1 — gates L1 (rule 3).** Stationary on the **substrate L1's arms actually inhabit**. This
  is the requirement that makes L1's number interpretable, and it is the tier that decides whether
  L1 reports a verdict or reports BLOCKED.
- **Tier 2 — the arc-level dilemma.** Stationary under **node-turnover churn** (the H0.3 proxies and
  the H1 bounded-churn substrate). This is the general open problem the V7 plan names, and it is
  reported whether or not Tier 1 passes.

**Passing Tier 1 does not resolve Tier 2, and will not be reported as if it did.** If WL qualifies
on the diffusion substrate but still fails under churn, the finding is "L1 is measurable; the
general instrument dilemma stands unresolved" — not "the dilemma is solved".

## Candidate order (cheapest first, per L2.2)

0. **WL depth 2 on L1's own substrate.** Already known biology-discriminating; its stationarity on a
   fixed-node-set/moving-energy substrate is *not* known, because WL's base hash includes the energy
   field, so diffusion can proliferate classes even with no node turnover. Cheap to measure, and it
   is the candidate that decides whether any new instrument is needed at all.
1. **Carrier-intrinsic structural features** — the carrier's own attachment degree, and lineage/age
   structure — scored for C-vs-A discrimination on static data and for churn-invariance.
2. **Hybrid**: label-anchored boundary (churn-stable) with carrier-internal resolution by an
   intrinsic structural feature, re-posed from the J-track with the added requirement `|z_C| < 2`.

## STOP condition (pre-committed)

- **No candidate passes Tier 1** ⟹ L1 is **not measurable**. The four-way comparison is still built
  and its demography reported, but **no detectability verdict is read**, and the presence-vs-motility
  question is recorded as BLOCKED pending a non-partition measure (L2.3), whose own
  variable-persistence gate (V5 J2.1) must be answered on paper before any implementation.
- **No candidate passes Tier 2** ⟹ reported as a standing limitation of the arc, bounding every
  downstream churn-substrate ambition. This alone does not block L1.

## Forbidden

- Reading an L1 verdict on a partition that failed Tier 1.
- Weakening `|z_C| < 2.0`, the TWTF threshold, or the F0 support bar after seeing a candidate's numbers.
- Treating the energy-band partition as biology-discriminating.
- Presenting a Tier-1 pass as a resolution of the instrument dilemma.
