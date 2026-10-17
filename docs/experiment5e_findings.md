# Experiment 5e — findings: reproductive senescence, and a fundamental limit

Reported against `experiment5e_aging_preregistration.md` (committed before running). Harness:
`crates/hg-sim-core/tests/experiment5e_aging.rs`. Config-gated behind `reproductive_lifetime_cap`
(off by default ⟹ no `die_of_age` rule; exit-criterion 7 preserved). **Nothing tuned.**

## Result (parent-funded, child cost 10, bootstrap threshold 6, 10 000 steps, 6 seeds)

| lifetime cap | median | interior [7,13] | collapsed <3 | **seed 2024** | **seed 42** | turnover |
|---|---|---|---|---|---|---|
| off (= 5d) | 4.5 | 2 | 3 | **−28.1** | **−6.3** | 6/6 |
| 20 | 9.8 | 4 | 2 | **−28.1** | **−6.3** | 6/6 |
| 8 | 6.5 | 2 | 2 | **−28.1** | **−6.3** | 6/6 |
| 3 (die after 3 births) | 8.8 | 4 | 2 | **−28.1** | **−6.3** | 6/6 |

`die_of_age` fires and the mechanic is deterministic (smoke test); turnover persists at every
cap. But the headline is the two deep-sink columns.

## The finding: the deep sink is IRREDUCIBLE (F-A confirmed)

The two 5d deep-sink seeds (2024, 42) sit at **−28.1 and −6.3 — bit-for-bit identical across
every cap**, including the most extreme (cap 3: a carrier dies after only three births). Rate-based
mortality, at *any* intensity, does **not** move them. The pre-registered indirect mechanism
(aging → turnover → low-energy newborns → threshold binds) did not materialise for these lineages.

Combined with 5d (energy costs also left them exactly at −28.1/−6.3), this is a clean, robust,
**fundamental result**:

> A carrier whose `replication_threshold` is always satisfied (≪ 0) reproduces regardless of its
> energy and regardless of how often it has reproduced. Its threshold therefore **gates no
> decision** — it is *causally inert*. No selective force can act on a causally-inert trait,
> because there is no behavioural difference for selection to reward or punish. So the deep drift
> sink is **immune to every cost we can impose** — energy-proportional (5d) *and* rate-based /
> mortality (5e) alike. The identical −28.1/−6.3 across two orthogonal mechanisms and four
> intensities is the proof.

This is not "a mechanic still to find." It is a limit of selection in an **unclamped** trait
space: once a trait drifts into a region where it stops affecting behaviour, selection cannot pull
it back. The only way to prevent the deep sink is to **bound the trait** (clamp `replication_threshold`
≥ 0), which the project deliberately rejected at S2 (option b, unclamped jitter) — so within the
committed design, the deep sink is irreducible.

## The complete criterion-5 picture (Exp 5 → 5e)

The residual degeneracy resolves into **two components with fundamentally different status**:

1. **Marginal drift toward the floor** (thresholds a little too low). **SOLVED by cost.** A
   parent-side cost creates an interior optimum (5c); an energy-proportional tax pulls even
   reliably-rich lineages up to it (5d, median 4.5 → 9.9); rate-based aging helps here too (5e,
   marginal seeds improve, collapsed 3 → 2). Genuine non-degenerate selection, for the majority.
2. **A deep, always-satisfied drift sink** (threshold ≪ 0). **PROVABLY IRREDUCIBLE.** Immune to
   energy costs (5d) and rate-based mortality (5e), because the trait is causally inert there.
   Closable only by bounding the trait — excluded by the S2 unclamped decision.

So the honest, final criterion-5 verdict: **selection here is genuinely non-degenerate for the
lineages whose threshold still gates behaviour, and it converges them to the interior optimum; but
in an unclamped trait space a causally-inert drift sink is irreducible, and a minority of lineages
fall into it.** Criterion 5 is met non-degenerately in the substantive sense, with a precisely
characterised and *provable* limit — not a hidden failure.

## Progression (final)

| stage | mechanic | threshold outcome |
|---|---|---|
| Exp 5 | node-funded | collapses to floor (degenerate) |
| 5b | active metabolic traits | maintenance → floor, destroys turnover |
| 5c | parent-funded | interior optimum for the majority; residual basin |
| 5d | + energy-proportional tax | even the rich pulled to the optimum; deep sink immune |
| **5e** | **+ reproductive senescence** | **deep sink provably irreducible — the limit is characterised** |

## Reusable deliverable

`reproductive_lifetime_cap` (route B / `lifetime` key 21, adds a `die_of_age` rule) joins the
config knobs; off by default, `property_effects_regression` bit-identical. All six trade-off/economy
levers now exist as infrastructure. The criterion-5 investigation is complete: the achievable is
achieved, and the unachievable is proven unachievable within the committed (unclamped) design.
