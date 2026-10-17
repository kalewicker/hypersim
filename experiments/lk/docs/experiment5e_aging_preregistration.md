# Experiment 5e — pre-registration: reproductive senescence (rate-based aging)

**Committed 2026-07-15, BEFORE running.** Follows Experiment 5d, which proved energy costs have
a limit: they rescue *near-boundary* lineages toward the interior optimum but are **provably
immune** to a deep drift sink — a lineage with an always-satisfied (≪ 0) threshold reproduces
regardless of energy, so its threshold is causally inert and unselectable by any energy cost.
This experiment attacks the deep sink with a cost tied to reproduction **rate**, not energy.
Forbidden move (scope §8.1) still binds: **do not tune to force convergence.**

## 1. The mechanic

`reproductive_lifetime_cap = Some(cap)` (route B, pre-scoped `lifetime` key 21): each birth ages
the parent (`lifetime += 1`), and a carrier dies once `lifetime ≥ cap` — a death route decoupled
from energy. A chronic always-reproducer exhausts its cap fast and dies no matter how well-fed it
is. Added as a `die_of_age` rule (cloned from `die_and_recycle` with a lifetime guard). Off by
default ⟹ energy-death only (exit-criterion 7). Tested **on top of parent-funded replication**
(the 5c/5d base where the deep sink appears).

## 2. Why it might close the deep sink (the mechanism under test)

Directly, aging does NOT distinguish threshold for a *well-fed* carrier: a +10 and a −28 carrier
both reproduce at the target-availability rate when well-fed, so both age equally — the sink
would seem immune to aging too. **But aging acts indirectly:** more death ⟹ more freed sites ⟹
more re-colonisation ⟹ more newborns, each born at low energy (the child endowment ≈ 8). A
low-energy newborn is exactly the state where the threshold BINDS: with parent-funding, a newborn
with a ≪ 0 threshold reproduces immediately, pays the child cost, and dies (energy death), while a
newborn with a threshold near the child cost waits until it harvests up. So aging-induced turnover
manufactures the low-energy states that make the threshold causally active — the condition the
well-fed deep sink lacked.

## 3. Fixed parameters
Parent-funded, child cost 10, bootstrap `replication_threshold` = 6, 10 000 steps, 6 seeds
(including the two 5d deep-sink seeds: 2024 → −28.1, 42 → −6.3). **Sweep `cap` ∈ {no-aging, 20, 8, 3}**
(smaller cap = more aging = more turnover). Interior band [7, 13]; collapsed < 3. Compare the
collapsed fraction and the two deep-sink seeds' final thresholds to 5d (no-aging: 3/6 collapsed,
seeds 2024/42 at −28.1/−6.3).

## 4. Predictions (committed)

- **P1 — aging drains the deep sink.** As `cap` decreases (more turnover), the collapsed fraction
  falls **below** the energy-cost floor (5d's 2/6), and specifically the two deep-sink seeds
  (2024, 42) rise out of ≪ 0 toward the interior — because the churn forces their lineages through
  low-energy newborn states where a ≪ 0 threshold is lethal. This is the effect no energy cost
  could produce.
- **P2 — a turnover window.** Too-small `cap` over-kills (carriers die before leaving a
  reproducing descendant) ⟹ extinction; too-large `cap` ⟹ negligible extra turnover ⟹ no effect.
  There is an intermediate `cap` where the sink drains and the population persists.
- **P3 — determinism/conservation.** Runs reproducible; energy never spuriously increases
  (`die_of_age` recycles the same fixed amount as `die_and_recycle`, the rest is declared death
  entropy).

## 5. Falsification / honest outcomes

- **F-A — sink survives aging:** if the deep-sink seeds stay ≪ 0 at every survivable `cap`, then
  the deep sink resists even rate-based mortality, and the honest conclusion is that an
  always-satisfied threshold is **irreducibly unselectable** without bounding the trait — a
  fundamental limit, not a mechanic still to find. (This is a real and publishable negative.)
- **F-B — only closes via extinction:** if the sink drains only where the population dies out,
  report the trade-off; there is no working window.
- **F-C — mechanism bug:** carriers not aging, `die_of_age` never firing, or energy created ⟹
  diagnose before any claim.

## 6. Decision table

| outcome | interpretation |
|---|---|
| collapsed↓ below 5d, deep-sink seeds rise to interior, population persists | **P1–P2 confirmed:** rate-based aging closes the deep sink energy costs couldn't — universal (or near) non-degenerate convergence. Headline. |
| deep-sink seeds stay ≪ 0 at all survivable caps | **F-A:** the deep sink is irreducibly unselectable (causally-inert trait) — a fundamental limit. Also a real finding. |
| sink drains only at caps that cause extinction | **F-B:** no working window; report trade-off. |
