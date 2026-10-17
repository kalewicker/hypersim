# Experiment 6c — pre-registration: population-level causal emergence

**Committed 2026-07-15, BEFORE building or running.** Follows the Experiment-6b caveat: the
carrier/substrate CE was measured over a *node-class* TPM (6a identity-pinned; 6b vacuum-augmented
gave the *sign* — replication onset is non-degenerate but causally NON-emergent, CE ≪ 1). 6b
flagged that a **population-level observable** (carrier-count dynamics) is what a question about the
*magnitude* of replicator causal power needs. This experiment measures CE over the carrier-count
dynamics directly. Forbidden move (Finding F1 / scope §8.1) still binds: **CE is biased downward,
so CE > 1 clears a conservative bar; do NOT tune anything to produce CE > 1.**

## 1. The measurement

- **Micro state = carrier count `N(t)`** (number of label-100 nodes), sampled every cadence `Δ`
  steps. The micro TPM records `N(t) → N(t+Δ)` over the whole run (so it spans the establishment
  transient `1 → ~36` and the steady turnover fluctuation near carrying capacity).
- **Macro coarse-graining = `N` binned into `K` equal-width bands** over the observed range. CE =
  `EI_macro / EI_micro` (guarded: `EI_micro = 0 ⟹ CE = 0`).
- **Paired bare control** (alife off): `N = 0` for all time ⟹ one micro state ⟹ `EI_micro = 0` ⟹
  `CE = 0` — the degenerate baseline the biology must lift.
- Swept: cadence `Δ ∈ {1, 4, 16}` and bands `K ∈ {2, 4}` (both are part of the measurement's
  definition and are reported, not optimised).
- Self-contained harness using the exposed `TpmAccumulator` / `ei_from_tpm` / `coarse_grain`
  (carrier count `N` mapped to a synthetic class id); the core observer is untouched, so
  exit-criterion 7 holds by construction.

Regime: pinned turnover seed 12345, influx 0.34, bootstrap threshold 3, 20 000 steps (long enough
to accumulate ≥ 1000 count-transitions past the transient).

## 2. What CE means here (the honest framing)

At the population level the natural macro-causal structure is a **carrying-capacity attractor**:
the micro count fluctuates stochastically (birth/death ±1) but the coarse "near-capacity" macro
state is highly persistent. So a coarse-graining that lumps near-capacity counts *could* be more
deterministic than the fluctuating micro — which is what CE > 1 would report. Equally, if the
count dynamics is essentially a pinned value with symmetric noise, coarse-graining buys nothing and
CE < 1. Both are meaningful; we measure which.

## 3. Predictions (committed)

- **P1 — the bridge holds.** `CE_bare = 0` (N ≡ 0 ⟹ one state ⟹ EI = 0) and `CE_live > 0` at some
  cadence/K — the population-count dynamics is non-degenerate only because replication exists.
- **P2 — magnitude, reported not targeted.** We report `EI_micro`, `EI_macro`, and `CE_live` for
  every (Δ, K). We predict `CE_live` is O(1) and quite possibly **> the node-level 6b value**
  (0.02–0.37), because the carrying-capacity attractor is a genuine population-level macro-causal
  structure that the carrier/substrate node partition could not express. We do **not** predict,
  target, or tune toward CE > 1; if it appears, it is a datum to scrutinise, not celebrate.
- **P3 — cadence/K dependence is informative.** `CE` varies with `Δ` (how much turnover a step
  resolves) and `K` (coarse vs fine bands); the trend is part of the result. A carrying-capacity
  attractor should make coarser `K` (fewer bands) relatively more deterministic.

## 4. Falsification / honest outcomes

- **F-A — degenerate micro:** if `EI_micro ≈ 0` (the count is essentially pinned with no
  resolvable dynamics past the transient), the population-count observable carries no causal
  structure to coarse-grain, and CE is undefined/0 — report that the steady-state count is
  causally trivial (all the dynamics is in *who* lives, not *how many*).
- **F-B — gate not cleared:** < 1000 count-transitions ⟹ no claim (raise steps, not lower the
  gate).
- **F-C — bare not degenerate:** `CE_bare > 0` ⟹ the count bookkeeping is wrong (a bare substrate
  has N ≡ 0). Diagnose first.
- **F-D — determinism:** `CE_live` reproducible at the pinned seed.

## 5. Decision table

| outcome | interpretation |
|---|---|
| CE_bare = 0, 0 < CE_live < 1 | population-count dynamics is non-degenerate but sub-emergent — a magnitude for replicator causal power, below the conservative bar (consistent with 6b's sign). |
| CE_bare = 0, CE_live > 1 | **population-level causal emergence** (carrying-capacity attractor) clearing the conservative bar — a strong, surprising positive; verify determinism + that nothing was tuned. |
| EI_micro ≈ 0 | **F-A:** steady-state count is causally trivial; the replicator dynamics lives at the individual/lineage level, not the count. |
