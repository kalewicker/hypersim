# Experiment 6b — findings: birth/death-aware CE fixes the identity macro

Reported against `experiment6b_preregistration.md` (committed before any number was seen).
Harness: `crates/hg-sim-core/tests/experiment6b_birthdeath_ce.rs`. Seed 12345, turnover
regime, 8000 steps. Self-contained (own `TpmAccumulator`); the core `CausalObserver` is
untouched, so exit-criterion 7 is preserved by construction. **Nothing tuned to raise CE.**

## The fix and the result

6a's carrier/substrate macro TPM was exactly the identity because surviving nodes never
cross the carrier↔substrate boundary, pinning `EI_macro = log₂(K)`. 6b adds a **vacuum state
∅** so the dropped boundary crossings return: **birth = ∅→carrier, death = carrier→∅**.

Cadence sweep (Δ = steps between observations; Δ is part of the measurement):

| Δ | CE_live | EI_micro | EI_macro | log₂K − EI_macro | births | deaths | bare CE |
|---|---|---|---|---|---|---|---|
| 1  | 0.2500 | 2.7235 | 0.6808 | **+0.3192** | 4522 | 4488 | 0 |
| 4  | 0.2446 | 2.3317 | 0.5703 | +0.4297 | 4522 | 4488 | 0 |
| 16 | 0.1324 | 2.3726 | 0.3140 | +0.6860 | 4521 | 4488 | 0 |
| 64 | 0.0190 | 2.8578 | 0.0544 | **+0.9456** | 4246 | 4247 | 0 |

- **P1 (un-pinning) CONFIRMED.** `EI_macro` is no longer `log₂(K) = 1`; already at Δ=1 it is
  0.68. The macro TPM has real off-diagonal (birth/death) mass. The identity artifact is gone.
- **P2 (turnover dependence) CONFIRMED.** The gap `log₂K − EI_macro` grows monotonically with
  cadence (0.32 → 0.43 → 0.69 → 0.95): a longer window exposes more birth/death per
  persistence, driving the macro further from the identity. `EI_macro` now *encodes turnover*
  — the quantity 6a's pinned value could not.
- **P3 (bridge preserved) CONFIRMED.** Bare CE = 0 at every cadence (a static substrate has no
  births/deaths ⟹ no carrier crossings ⟹ K = 1 degenerate); live CE > 0.
- **F-D CONFIRMED.** Birth/death/transition counts are bit-reproducible; CE reproducible to FP
  precision (`experiment6b_is_deterministic`).

## What it means (the honest scientific conclusion)

6a reported CE = 0.37 with `EI_macro = 1.0`. That macro value was an **artifact of dropping
birth/death** — it measured only that node labels are frozen, not any replicator dynamics.
With the boundary crossings restored, the picture inverts:

**Once the replicator's actual dynamics (birth/death) are in the macro, the carrier/substrate
coarse-graining has LOW and cadence-decreasing causal power** — CE falls from 0.25 (Δ=1) to
0.019 (Δ=64), and `EI_macro → 0` as more turnover is resolved. The carrier/substrate macro is
a **stochastic occupancy process**, not a causally-emergent deterministic one. There is **no
causal emergence** under this partition (CE ≪ 1, and falling), and 6a's 0.37 was inflated by
the identity artifact.

This is the stronger, more honest result the identity-macro caveat was pointing at: replication
onset makes the partition *non-degenerate* (the 0→>0 taxonomy bridge of 6a still holds), but it
does **not** confer macro-level causal emergence. Under Finding F1 (CE biased downward) a CE
this far below 1 is a clean, un-tuned negative on emergence, not a masked positive.

## Caveats

1. **Cadence-dependence is the result, not a single CE.** CE is meaningless without Δ (the
   pre-registration committed the sweep, not a point value). The reportable finding is the
   *trend* (un-pinned, monotone), not "CE = 0.25."
2. **Aliasing at large Δ.** Carriers born *and* dead within one window are invisible to a
   snapshot-diff (Δ=64 undercounts turnover: 4246 vs 4522 births at Δ=1). So the largest-Δ CE
   is a lower bound on resolvable turnover; the trend across Δ is robust regardless.
3. **Still node-class based.** This captures birth/death as ∅ transitions but remains a
   node-class TPM. A fully turnover-native measurement would use a **population-level
   observable** (carrier-count / occupancy-configuration dynamics); if a future question needs
   the *magnitude* of replicator causal power rather than its *sign*, that is the next
   pre-registration. 6b already answers the sign: not emergent.

## Verdict

The identity-macro limitation of 6a is **fixed**: including birth/death un-pins `EI_macro` and
makes it encode turnover (P1, P2), the paired bridge is preserved (P3), and the measurement is
deterministic (F-D). The corrected Experiment-6 conclusion: **replication onset yields a
non-degenerate but causally NON-emergent (CE ≪ 1) carrier/substrate macro** — a clean negative
on causal emergence, reported without tuning, superseding 6a's identity-inflated CE.
