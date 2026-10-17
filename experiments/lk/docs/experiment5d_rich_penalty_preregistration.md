# Experiment 5d — pre-registration: an energy-proportional reproductive tax

**Committed 2026-07-15, BEFORE running.** Follows Experiment 5c (parent-funded replication),
which produced the first *partial* non-degenerate criterion-5 selection: `replication_threshold`
settled at an interior optimum near the child cost for the majority of lineages, but a residual
degenerate basin remained (some lineages still collapsed to the floor). This experiment tries to
**close that basin**. Forbidden move (scope §8.1) still binds: **do not tune to manufacture
universal convergence.**

## 1. Diagnosis → the mechanic

The residual basin is not "low threshold is fit for the rich" — it is **neutral-trait drift**.
A carrier that stays reliably rich (solitary on a well-fed site, pinned near satiation) is always
above its threshold, so the threshold never gates anything and drifts freely to the floor. 5c's
*fixed* per-birth cost cannot bite such a carrier: it pays the fixed cost and re-harvests.

**Fix (5d):** add an **energy-proportional reproductive tax**. On each birth, after paying the
fixed child cost, the parent loses fraction `tau` of its remaining energy to entropy
(`parent.energy ×= 1 − tau`, via `scale_from`). Now the *absolute* cost of reproducing scales
with the parent's wealth, so reproducing while overflowing is genuinely wasteful — the threshold
stops being neutral for the rich (waiting to reproduce conserves energy that a low threshold
squanders). The fixed cost is kept, so 5c's poor-carrier survival penalty still holds.

Config: `repro_energy_tax = Some(tau)` with `parent_funded_replication = true`. `None` ⟹ exactly
5c. Off by default ⟹ node-funded (exit-criterion 7). The tax is a declared proportional entropy
sink; `scale_from` can only *remove* energy, so it cannot create any.

## 2. Fixed parameters
Turnover regime, child cost 10, 10 000 steps, 6 seeds. **Bootstrap `replication_threshold` = 6**
(seeded *below* the child-cost optimum — the case with the biggest residual basin in 5c: 3/6
collapsed at boot 6). **Sweep `tau` ∈ {0.0, 0.3, 0.6}** (0.0 reproduces the 5c baseline in-run).
Interior band = [7, 13] around the child cost; collapsed = final threshold < 3.

## 3. Predictions (committed)

- **P1 — the tax closes the basin.** As `tau` increases, the fraction of lineages collapsing to
  the floor **decreases** and the fraction settling in the interior band **increases**: reproducing
  when rich is now costly, so even reliably-rich lineages are selected toward the interior optimum.
  At `tau = 0` we reproduce 5c (≈3/6 collapsed at boot 6); at `tau = 0.6` the basin should be
  largely closed (collapsed → 0–1).
- **P2 — the optimum shifts up.** The proportional tax penalizes reproducing at high energy, so
  the interior optimum moves toward higher thresholds (reproduce later, when the proportional loss
  is a smaller share of a leaner reserve) — the median final threshold rises with `tau`.
- **P3 — too much tax → extinction.** A large `tau` drains carriers faster than harvest can
  refill, so establishment/turnover degrades at high `tau`. Report the establishment/turnover map;
  there may be a `tau` window where the basin closes *and* the population persists.

## 4. Falsification / honest outcomes

- **F-A — basin does not close:** if collapsed fraction is flat (or rises) across `tau`, the
  proportional tax does not fix the neutral-drift, and the honest report is that closing the basin
  needs a *non-energy* cost (per-reproduction mortality / aging via `lifetime`), a separate
  pre-registration.
- **F-B — universal extinction:** if even `tau = 0.3` collapses the population, the tax is too
  harsh at these constants; report, do not tune the constants to rescue it.
- **F-C — conservation/determinism:** total energy must never spuriously increase (the tax is a
  sink), and runs must be reproducible at a fixed seed.

## 5. Decision table

| outcome | interpretation |
|---|---|
| collapsed↓ and interior↑ with `tau`, population persists | **P1–P3 confirmed:** the rich-biting tax closes the basin — universal (or near-universal) non-degenerate selection. The headline. |
| basin closes only as the population goes extinct | partial: no `tau` window with both; report the trade-off. |
| collapsed flat across `tau` | **F-A:** energy-proportional cost insufficient; a non-energy cost is needed next. |
