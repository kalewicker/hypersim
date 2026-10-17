# Experiment 6c — findings: population-level causal emergence

Reported against `experiment6c_population_ce_preregistration.md` (committed before running).
Harness: `crates/hg-sim-core/tests/experiment6c_population_ce.rs`. Self-contained (own
`TpmAccumulator`); no source changes, so exit-criterion 7 holds by construction. **Nothing tuned;
CE > 1 was never targeted (Finding F1).**

## Result (carrier-count dynamics, seed 12345, turnover regime, 20 000 steps)

| Δ | K | CE_live | EI_micro | EI_macro | micro states | transitions | bare CE |
|---|---|---|---|---|---|---|---|
| 1  | 2 | 0.1428 | 3.2850 | 0.4692 | 28 | 19 999 | 0 |
| 1  | 4 | 0.3648 | 3.2850 | 1.1983 | 28 | 19 999 | 0 |
| 4  | 2 | 0.1724 | 1.6716 | 0.2881 | 18 | 4 999  | 0 |
| 4  | 4 | 0.5625 | 1.6716 | 0.9403 | 18 | 4 999  | 0 |
| 16 | 2 | 0.0002 | 0.9504 | 0.0002 | 15 | 1 249  | 0 |
| 16 | 4 | 0.0019 | 0.9504 | 0.0018 | 15 | 1 249  | 0 |

Bare (alife off): `N ≡ 0` ⟹ one micro state ⟹ CE = 0 at every (Δ, K) — F-C clean, the degenerate
baseline the biology lifts. Determinism (F-D): count series and CE bit-reproducible.

## Interpretation

- **P1 (bridge): CONFIRMED.** `CE_bare = 0`, `CE_live > 0`. The population-count dynamics is
  non-degenerate only because replication exists.
- **F-A REFUTED — the count is NOT causally trivial.** `EI_micro = 3.29` over **28 distinct count
  states** at Δ=1 — *richer* than the node-level micro of 6a/6b (~2.0–2.7). So the replicator
  dynamics is genuinely, richly present at the population-count level (the establishment growth
  `1 → ~36` plus steady turnover fluctuation give real structure); it is not all hidden in *who*
  lives.
- **P2 (magnitude): NO causal emergence.** `CE_live < 1` at **every** cadence and every K
  (maximum 0.56 at Δ=4, K=4). Coarse-graining the count into bands *always loses* effective
  information — no macro band description is more causal than the fine count. The pre-registered
  carrying-capacity-attractor route to CE > 1 **did not materialise**: the count near capacity
  fluctuates symmetrically enough that lumping states buys no determinism.
- **P3 (cadence/K): informative.** `EI_micro` falls with cadence (3.29 → 1.67 → 0.95: longer
  windows resolve fewer transitions) and CE collapses to ~0 at Δ=16 (the count is nearly pinned,
  so coarse-graining captures nothing). CE rises with K only because a finer partition approaches
  the micro description (CE → 1 as K → #states) — it never crosses 1, i.e. every coarse-graining
  is strictly information-losing.

## The complete Experiment-6 picture (6a → 6b → 6c)

| observable | micro state | result |
|---|---|---|
| 6a carrier/substrate | surviving-node WL class | EI_macro identity-pinned (log₂K); CE = 1/EI_micro (artifact) |
| 6b carrier/substrate + ∅ | node class incl. birth/death | EI_macro un-pinned; CE ≪ 1, falling with cadence — non-emergent |
| **6c population** | **carrier count N(t)** | **rich micro (EI 3.29, 28 states) but CE < 1 at every Δ,K — non-emergent** |

**Consolidated Experiment-6 conclusion:** replication onset makes the system's coarse descriptions
non-degenerate — the **0 → >0 bridge holds at every level** (carrier/substrate and population) — but
it produces **no causal emergence**: `CE < 1` across *all three* observables and every cadence/
partition tested. The replicator is real and, at the population level, richly structured; yet at no
level does a coarse (macro) description gain causal power over the fine (micro) one. This is a
consistent, un-tuned **negative on causal emergence**, exactly the conservative direction Finding F1
predicts CE should sit — CE > 1 was never sought or produced.

## Reusable deliverable

The population-CE harness is a permanent, self-contained measurement (exposed `TpmAccumulator` /
`ei_from_tpm` / `coarse_grain`; carrier count mapped to a synthetic class id). No source changes;
`property_effects_regression` untouched. The Experiment-6 line (causal emergence across the
replication-onset boundary) is now answered at both the node and population levels: **non-degenerate
bridge, no emergence.**
