# V6 / K-track — hardening the verdict: it is sustained PRESENCE, it holds across seeds,
# and it survives subtracting the always-on signal

Reported against the V6 hardening plan. Harnesses: `k1_turnover_vs_persistence.rs`,
`k2_multiseed_arm_b.rs`, `k3_signal_isolation.rs`. Pre-registrations committed before the
corresponding numbers existed: K1 `7da6c2e`, K2 `38944b1`, K3 (before any K3 number).

---

## The single sentence

**The load-bearing property is sustained *presence*, not sustained turnover — arm B has literally
zero deaths in 6000 steps; it holds across the regime (30/30 disjoint seeds, sign test p = 1.9×10⁻⁹);
and it survives subtracting the always-on signal, because the dead token carries a large metabolic
signal and *no* WL signal, so the WL boundary cannot be the metabolic boundary in disguise.**

---

## K1 — arm B is PERSISTENCE, not turnover (the gate moved the verdict)

Ran first, as required. All quantities are realized (carrier `NodeId` set differences step to
step), never configured rates.

| arm | peak | last-alive | mean pop | births | deaths | mean completed lifetime | censored | per-capita death rate | **relocations** |
|---|---|---|---|---|---|---|---|---|---|
| A replicating | 36 | 6000 | 33.66 | 3389 | 3355 | 59.9 | 1.0% | 0.01661 | 0 |
| B non-repl SUSTAINED | 6 | 6000 | 6.00 | 6 | **0** | n/a | **100%** | **0.00000** | **248** |
| C non-repl token | 6 | 120 | 0.12 | 6 | 6 | 120.0 | 0% | 0.00833 | 0 |

The pre-committed PERSISTENCE branch fires on all three of its clauses at once, at the extreme:
`r_B = 0` (needs ≤ 0.1·r_A), no completed lifetimes at all, `censored_frac_B = 1.0` (needs ≥ 0.5).
Arm B's six founders are alive at step 6000. They are not stationary, though — 248 realized
relocations — so B is **persistence of individuals, not persistence of position**: the same
carriers graze indefinitely, which is exactly why they never starve the way C does.

So the ABC design's assumption was wrong in a way that matters: A and B are at *opposite* demographic
extremes (A replaces its population ~94 times over; B never replaces it once) and **both are
WL-detectable**. Neither replication nor turnover is required. What both share, and C lacks, is
**presence across the scoring window**.

**TWTF did not discriminate**, and this is reported rather than quietly dropped: it reads 0.000 for
the static floor, arm A and arm B alike (whole-graph and carrier-restricted, at W=120 and W=6000).
TWTF measures *class* ephemerality, and arm A's carrier classes are persistent even while arm A's
individuals turn over completely. The plan expected TWTF to be the turnover-vs-persistence
discriminator; it is not one, because class-level stationarity and individual-level demography are
different things. The demographic counts carried the verdict alone.

## K2 — the detectability is a REGIME PROPERTY

Seeds 1–30, disjoint from 12345, F8 discipline verbatim; fidelity anchor reproduced H2.2's arm B
exactly (k_carrier = 4, 0/200, support 1.97×10⁷).

- attempts 30, **sustained 30 (100%)**, **included 30 (100%)** — establishment rate reported per F6.2
- **primary: 30/30 detectable at p < 0.05 on the WL partition** (every seed 0/200, i.e. p < .005) ⟹ `f = 1.000`
- secondary sign test: 30/30 real CE above own null mean, exact binomial **p = 1.9×10⁻⁹**
- **reversals: none**
- CE range 0.1414–0.1490 against null means 0.1253–0.1341; supports 8.8×10⁶–2.3×10⁷; k_carrier = 4 on every seed

Pre-committed threshold `f ≥ 0.90` ⟹ **regime property**. The sustained-presence verdict may be
stated flat rather than as a seed-12345 draw.

**K1 also generalises**: 30/30 included seeds recorded **zero carrier deaths** over 6000 steps
(mean 273 relocations). Persistence is regime-level, not a seed artifact.

**Honest caveat on how strong this test is.** Arm B's regime is far more deterministic than F8's:
every seed yields exactly pop = 6, zero deaths, and an identical transition count (251 958), because
6 planted markers plus motility on a 36-node substrate always sustain. So the seeds vary marker
placement and RNG tie-breaking, not survival. This is a weaker stress test than F8's, where
establishment itself varied (~57–71%). It rules out a seed-12345 fluke; it does not probe a fragile
regime, because arm B's regime is not fragile.

## K3 — the presence-specific signal SURVIVES the subtraction

Effect size fixed before computing: `z = (real_CE − null_mean)/null_sd` over the same 200
permutations — a **detectability** measure, not a magnitude claim (F2.4 stands).

| arm | WL: k_c/M | WL Δ | **z_wl** | WL p | e-band: k_c/M | e-band Δ | **z_eb** | e-band p |
|---|---|---|---|---|---|---|---|---|
| A replicating | 4/200 | +0.1173 | **6.56** | <.005 | 16/34 | +0.1745 | 9.81 | <.005 |
| B non-repl SUSTAINED | 4/149 | +0.0115 | **2.28** | <.005 | 15/36 | +0.2282 | 12.72 | <.005 |
| C non-repl token | 2/30 | −0.0019 | **−0.83** | 0.930 | 16/38 | +0.1874 | 9.59 | <.005 |

(Arm A's WL row reproduces F2 exactly: CE 0.1633, 0/200, support 6.47×10⁷.)

**K3.1 — the always-on signal's magnitude: `z_eb(C) = 9.59`.** Arm C is dead from step 120 and has
neither replication nor sustained presence, so that entire signal is definitional — a carrier simply
holds energy in a different band than substrate.

**K3.2 — presence-specific component**, subtracted within the WL partition with C as calibrator:
`S(A) = 6.56 − (−0.83) = +7.38`, `S(B) = 2.28 − (−0.83) = +3.11`. Both positive, A and B WL-detectable,
C not.

**K3.3 — SURVIVES.** The argument is direct and does not lean on the arithmetic: arm C carries a
*large* always-on metabolic signal (z = 9.59) and *no* WL signal (z = −0.83, p = 0.93). If the WL
boundary were the metabolic boundary in disguise, C would have to show it. It does not. Sustained
presence adds WL structure that metabolic distinctness alone does not produce. That is the object
worth carrying to any future causal-emergence claim.

**But K3 also deflates the metabolic instrument, and that belongs in the report.** On the energy-band
partition the dead token scores z = 9.59 against arm A's 9.81 — the always-on definitional signal
accounts for essentially *all* of arm A's metabolic detectability. The energy-band partition (the J2
deliverable that unblocked this whole arc) recovers F8 detectability and is churn-stable, but it
discriminates almost nothing about the biology: it mostly reports that carriers are carriers. Any
future use of it must subtract a C-like control, or it will conflate the definitional with the
interesting.

Note also that `z_wl(B) = 2.28` is modest in absolute terms while its permutation p is < .005 (0/200):
the null is tight and non-Gaussian in the tail. `p` remains the pre-registered detectability
criterion inherited from the F-track; `z` is the K3 effect size and is used only for the subtraction.

---

## K4 — report-level consequence

- **§5.7 verdict.** Replace "sustained turnover" with **"sustained presence"**. The corrected
  statement: *a carrier population that is present across the scoring window is detectable on the WL
  partition; neither replication (A vs B) nor population turnover (A vs B again, at opposite extremes)
  is required; a population that is not sustained (C) is not detectable.* Upgrade from single-seed to
  **regime property** (30/30, p = 1.9×10⁻⁹), with the caveat above about arm B's regime being
  deterministic rather than fragile.
- **§9/§10.** State the K3 result both ways: the presence-specific component **survives** subtraction
  (C has max metabolic signal and zero WL signal), *and* the energy-band partition is **almost
  entirely** always-on (z 9.59 dead vs 9.81 alive), so metabolic-partition detectability must never be
  reported without a C-like calibrator.
- The central claim **narrows** in one respect (turnover → presence) and **strengthens** in two
  (single-seed → regime; detectability → detectability beyond the definitional signal). The narrowing
  is written, not smoothed over.

## Scope and standing caveats

- Motility remains a mechanic the G-track did not have; every K-track statement is about *this*
  biology plus `relocate`.
- Magnitude stays out of scope (F2.4, registry-dominated). Every number here is detectability.
- K3 is single-seed (12345). K2 hardened arm B's *detectability* across seeds but did not repeat the
  K3 subtraction per seed; that is the cheapest available next hardening.
- The later track the plan gates on this one — re-running the split on the churn substrate to remove
  the motility caveat — is now unblocked but still unrun.
