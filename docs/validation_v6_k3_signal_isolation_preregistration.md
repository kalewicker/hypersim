# V6 / K3 — isolating the presence-specific signal from the always-on metabolic one

**Committed before any K3 number is computed.** K1 (commit `815d2fd`) renamed the positive property
from *turnover* to *presence*, so this file says "presence-specific" where the V6 plan says
"turnover-specific"; the quantity is the same one the plan defines.

## The problem K3 addresses

The energy-band (metabolic) partition detects **every** arm of the H2 ABC experiment, including
arm C, the token that dies at step 120. So part of carrier detectability is **definitional**: a
carrier is a metabolically distinct kind of object — it holds energy in a different band than
substrate — whether or not it is sustained, replicating, or even alive for long. If that always-on
component is not separated out, "the boundary is detectable" conflates *does something interesting*
with *is a distinct kind of object*.

## Effect size (fixed here, before computing)

CE **magnitude** is registry-dominated (F2.4) and is not comparable across arms with different `M`.
So the effect size is the standardized distance of the real partition from its own permutation
null:

```
z = (real_CE − null_mean) / null_sd
```

over the same 200 size-preserving permutations already used everywhere in this project (no
pipeline change; `null_sd` is simply also recorded now). `z` is dimensionless — "how many null
standard deviations above an arbitrary same-sized partition" — so it is comparable across arms and
across partitions as a *detectability* measure. It is **not** a magnitude claim and is not to be
read as one. The raw margin `Δ = real_CE − null_mean`, `p`, `k_carrier`, `M` and the null support
`C(M,k)` are reported beside every `z`.

## K3.1 — the always-on signal's magnitude

Arm **C** (non-replicating, not sustained, dead at 120) is metabolically detectable and WL-
undetectable, so **C isolates exactly what the metabolic partition contributes that is not
presence**. Report `z_eb(C)`, `Δ_eb(C)`, `p_eb(C)`, `k_carrier`, support. That number *is* the
magnitude of the always-on definitional signal.

## K3.2 — the presence-specific component

The subtraction is done **within a partition**, never across two, because two partitions' CE values
live on different registries. Arm C is the calibrator: it is the arm that has the always-on
metabolic distinctness and nothing else.

```
S(arm) = z_wl(arm) − z_wl(C)        for arm ∈ {A, B}
```

`z_wl(C)` is the WL-partition signal attributable to bare metabolic/definitional distinctness with
no sustained presence. What exceeds it is the presence-specific component. The cross-partition
comparison `z_wl(arm)` vs `z_eb(arm)` is reported per arm as context, but **no verdict is read from
a cross-partition difference.**

## K3.3 — PRE-COMMITTED READING

- **SURVIVES** — `S(A) > 0` and `S(B) > 0`, **and** A and B are WL-detectable (`p_wl < 0.05`)
  **while C is not** (`p_wl(C) >= 0.05`) ⟹ there is a real presence signal distinct from mere
  metabolic distinctness. The logic is direct: C carries a *large* always-on metabolic signal and
  *no* WL signal, so the WL boundary cannot be the metabolic boundary in disguise. This is the
  object worth carrying to any future causal-emergence claim.

- **VANISHES** — `S(A) <= 0` or `S(B) <= 0`, **or** C is itself WL-detectable ⟹ A/B's WL
  detectability is not distinguishable from the always-on metabolic level. The "presence boundary"
  was substantially the metabolic boundary in disguise: a strong deflation and a partial
  vindication of the construction hypothesis. Reported as such, with no attempt to rescue it.

- Any mixed outcome (e.g. `S(A) > 0` but `S(B) <= 0`) is reported as **partial** with the arm-level
  numbers stated, not resolved by picking the favourable arm.

## Forbidden

- Reading a verdict from a cross-partition CE difference (different registries).
- Reporting `z` or CE as a magnitude claim.
- Re-defining the effect size after seeing the numbers.
