# Experiment 5c — pre-registration: parent-funded replication (§1 option 2)

**Committed 2026-07-15, BEFORE running the parent-funded economy.** This is the
pre-registered §1 option-2 route: it **reopens the node-funded decision** (scope §1, §7.3),
argued on its merits below, and is the last candidate route to *non-degenerate* criterion-5
selection after Experiment 5b closed the active-traits route. Forbidden move (scope §8.1)
binds: **do not hand-inject a selective force to make the run "look like" open-ended
evolution, and do not tune to produce an interior optimum.**

## 1. The change, and why it is a principled mechanic (not a knob)

Node-funded `replicate`: the *target site* pays the child cost; the parent pays nothing. So
`replication_threshold` has **no opposing force** — a lower threshold is strictly fitter
(more offspring, zero self-cost), and selection drives it to the floor (the criterion-5
degeneracy, confirmed twice: repl_threshold in 5-validation, maintenance_cost in 5b).

Parent-funded `replicate` (config `parent_funded_replication = true`): the **parent pays the
child cost from its own energy**; the target only supplies the unoccupied dispersal location.
This is a real biophysical mechanic — an organism invests *its own* energy in offspring — not
a fitness term bolted onto the genome. Its consequence is a genuine **survival trade-off**:
to survive reproducing, a carrier must hold energy ≥ child cost; a carrier whose
`replication_threshold` is far below the child cost reproduces while too poor to pay and dies.
So `replication_threshold` acquires an **interior optimum** near the child cost, from *both*
sides: too low ⟹ die on reproduction; too high ⟹ reproduce too rarely.

**Conservation is untouched:** total energy per birth is still −2 (parent −cost, child
+0.8·cost, 0.2 entropy) exactly as node-funded — only the payer moves from target to parent.
The ledger identity is preserved by construction. Off by default ⟹ node-funded, every
calibrated test byte-unchanged (exit-criterion 7).

## 2. Fixed parameters
Turnover regime (influx 0.34, child cost 10, 10 000 steps). Because a founder with
`replication_threshold` ≪ child cost dies on its first birth, **bootstrap threshold is swept**
∈ {6, 10, 15, 20} (via `bootstrap_replication_threshold`), from below and above the
hypothesized interior optimum (~10). Seeds {12345, 1, 7, 99999, 2024, 42}. Drift reference:
the still-inert traits `harvest_rate`/`maintenance_cost` (fixed economy) — sign-mixed, |Δ|
envelope ≈ 4.5.

## 3. Predictions (committed)

- **P1 — interior optimum (the non-degenerate signature).** Under parent-funding,
  `replication_threshold` does **NOT** collapse to the floor. It **converges toward an
  interior value near the child cost (~10) from BOTH directions**: runs seeded above ~10 drift
  down toward it, runs seeded below (but survivable) rise toward it. Convergence-from-both-sides
  to a stable interior value is selection resolving a trade-off — qualitatively unlike the
  monotone floor-collapse of node-funding.
- **P2 — establishment threshold.** Runs seeded with `replication_threshold` far below the
  child cost fail to establish (founders die on first reproduction). Establishment improves as
  bootstrap threshold approaches/exceeds the child cost. Report the establishment map.
- **P3 — turnover persists.** Unlike 5b (where maintenance collapse froze the population),
  parent-funded turnover should persist: reproduction now costs energy, so the population does
  not go immortal; deaths continue.
- **P4 — criterion-5 verdict.** If P1 holds, this is the project's **first non-degenerate
  criterion-5 selection**: a trait under a real trade-off shifting to an interior optimum,
  distinguishable from drift. Report the mean and the convergence, not a single number.

## 4. Falsification / honest outcomes

- **F-A — still degenerate / no interior optimum:** if `replication_threshold` collapses to the
  floor anyway, or diverges to the ceiling, or shows no reproducible convergence, then
  parent-funding does **not** produce non-degenerate selection — report it (the degeneracy is
  deeper than the funding source).
- **F-B — universal extinction:** if no bootstrap threshold establishes, the trade-off is too
  harsh at these constants; report and note the child-cost/energy scale, do not tune to rescue.
- **F-C — determinism / conservation:** the run must be reproducible and the ledger must still
  balance to −2 per birth (parent-funding must not have leaked energy).

## 5. Decision table

| outcome | interpretation |
|---|---|
| repl_threshold converges to interior ~child cost from both sides, turnover persists | **P1–P4 confirmed:** first non-degenerate criterion-5 selection. The headline. |
| repl_threshold collapses to floor / ceiling regardless | **F-A:** parent-funding insufficient; degeneracy is not (only) about the payer. |
| no seed establishes | **F-B:** trade-off too harsh at these constants; report, don't tune. |
