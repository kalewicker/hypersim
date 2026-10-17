# Experiment 6b — pre-registration: birth/death-aware CE (fixing the identity macro)

**Committed 2026-07-15, BEFORE building or running the birth/death-aware measurement.**
Motivated by `experiment6_findings.md` §1: Experiment 6a's carrier/substrate macro TPM is
**exactly the identity**, so `EI_macro` is pinned to `log₂(K)` and CE reduces to `1/EI_micro`,
carrying no information about replicator turnover. The forbidden move (Finding F1) still
binds: **do not tune anything to raise CE.**

## 1. Diagnosis → the fix

6a records node-class transitions only for nodes that **survive** window-to-window. A
surviving node never crosses the carrier↔substrate boundary (a substrate node never becomes
a carrier; a carrier stays a carrier until it dies), so the 2×2 macro TPM is the identity.
But the carrier/substrate boundary **is** crossed — by node **creation and deletion**: a
`replicate` births a carrier (∅→carrier) and a `die_and_recycle` removes one (carrier→∅).
6a drops exactly these events.

**Fix (6b):** augment the micro dynamics with a **vacuum state ∅**. Per observation:
- surviving node: record `class_prev → class_now` (as 6a);
- **died** node (present at t, absent at t+Δ): record `class_prev → ∅`;
- **born** node (absent at t, present at t+Δ): record `∅ → class_now`.

∅ is grouped with substrate in the macro partition (absence-of-carrier = substrate state).
Now births contribute substrate→carrier macro mass and deaths contribute carrier→substrate
macro mass: the macro TPM acquires real off-diagonal flow and `EI_macro` is **no longer
pinned** at `log₂(K)`. This is the *same node-class micro space* as 6a plus the transitions
6a dropped — the minimal faithful change, not a new state space bolted on.

Self-contained in a harness (`experiment6b_birthdeath_ce.rs`) using the exposed ei.rs
primitives (`TpmAccumulator`, `ei_from_tpm`, `coarse_grain`); the core `CausalObserver` TPM
is **untouched**, so all existing EI measurements are byte-unchanged (exit-criterion 7).

## 2. Fixed parameters (committed)
Same regime as 6a: seed 12345, 6×6 static crystal, turnover config
(`influx_rate=0.34`, `bootstrap_replication_threshold=3.0`, `initial_substrate_energy=5.0`).
Micro = node WL class (radius 1) + ∅ sentinel. Macro partition = carrier (`motif`≠0) vs
{substrate ∪ ∅}. Paired bare control (alife off). **Cadence is SWEPT** (Δ ∈ {1, 4, 16, 64}):
the birth/death flow relative to persistence is a function of Δ, so Δ is part of the
measurement and the sweep is the test of P2. `min_transitions` gate = 1000.

## 3. Predictions (committed)

- **P1 (un-pinning — the headline of 6b).** With ∅ transitions included, `EI_macro_live`
  is **strictly less than `log₂(K)`** (= 1 for K=2), UNLIKE 6a where it equalled 1 exactly.
  The macro TPM has nonzero off-diagonal (birth/death) mass. Success criterion for the fix:
  `log₂(K) − EI_macro_live > 0` beyond FP tolerance.
- **P2 (turnover dependence).** `EI_macro_live` **decreases as cadence Δ increases** — a
  longer window exposes more birth/death per persistence, so the macro becomes less
  identity-like. This demonstrates `EI_macro` now *encodes turnover*, the thing 6a's pinned
  value could not.
- **P3 (bridge preserved).** `CE_bare = 0` (no carriers ⟹ no ∅ carrier-crossings ⟹ K=1
  degenerate) and `CE_live > 0` at every cadence.
- **P4 (Finding F1).** Report `CE_live` honestly. We do **not** predict or target CE > 1;
  a lower CE than 6a is an acceptable, expected outcome (a stochastic macro has less causal
  determinism than the frozen-label identity — that is the point, not a regression).

## 4. Falsification / honest-negative

- **F-A (fix fails):** if `EI_macro_live ≈ log₂(K)` (within FP) at **all** cadences — i.e.
  birth/death mass is negligible against persistence even at Δ=64 — then node-transition CE
  fundamentally cannot capture replicator turnover, and the honest conclusion is that a
  **population-level observable** (carrier-count dynamics), not a node-class TPM, is needed.
  Report it; do not escalate cadence indefinitely to manufacture a departure.
- **F-B (bare artifact):** `CE_bare > 0` ⟹ the vacuum bookkeeping is mis-specified (a bare
  static substrate has no births/deaths). Diagnose before any live claim.
- **F-C (gate):** below 1000 transitions ⟹ no claim.
- **F-D (determinism):** transition counts must be bit-reproducible at each cadence.

## 5. Decision table

| outcome | interpretation |
|---|---|
| EI_macro < log₂K, decreasing in Δ, CE_bare=0 | **Fix works.** The macro encodes turnover; CE is now a meaningful (birth/death-aware) number. Report its value and its cadence-dependence; still likely < 1 (F1). |
| EI_macro ≈ log₂K at all Δ | **F-A honest negative.** Node-class CE can't see turnover; recommend a population-level observable as the next pre-registration. |
| CE_bare > 0 | **F-B artifact.** Fix the vacuum accounting. |
