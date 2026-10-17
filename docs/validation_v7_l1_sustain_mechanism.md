# V7 / L1.1 — how a *stationary* carrier is sustained (mechanism committed before build)

**Committed before the mechanism is built or any arm-D number exists.** Per V7 rule 4, whatever is
specified here is a substrate/economy change and must be re-characterised bare as physics (L1.2)
before biology uses it.

## Why the static substrate cannot supply the specimen (the arithmetic)

Under `MaxParallel` a node takes part in at most one rewrite per step, so the carrier's site can
either be harvested, or receive influx — never both on the same step. Let `f` be the fraction of
steps on which `harvest` fires.

- Carrier: `+1.0` on harvest steps, `−0.5` (metabolise) otherwise ⟹ net `1.5f − 0.5`.
  **Sustained requires `f >= 1/3`.**
- Site: `−1.0` on harvest steps, `+R` on the steps it receives instead ⟹ `f <= R/(1+R)`.

With influx alone (`R = 0.34`): `f <= 0.254 < 1/3`. The stationary carrier starves. This is not a
tuning accident — it *predicts the G-track token's observed death at step 120*: after the initial
site charge (5.0) is eaten, the carrier loses `0.5` per 4-step influx cycle and gains `1.0` per
harvest, i.e. `−0.125/step` from ~15.0 ⟹ ~120 steps. The mechanism is understood, not assumed.

To sustain a stationary carrier the site must receive at a rate `R >= 0.5` per receiving step.

## Why not churn

The V7 plan suggests churn refreshing occupied sites. It cannot: the H1 churn rule fires on
**unoccupied** nodes and *deletes* the old node. Refreshing the carrier's own site by churn would
mean deleting the site the carrier is attached to — destroying the specimen rather than sustaining
it. Churn mints fresh *free* nodes, which is exactly the H2.1 finding: a stationary carrier cannot
reach them.

So the mechanism is **energy transport, not node turnover**. This is a deliberate deviation from the
plan's phrasing and is recorded as such in the L2.1 criteria (which measures stationarity on both
this substrate and under churn).

## The mechanism: `diffuse` — gradient-driven, conservative energy transport

A substrate-only rule. It refers to **no carrier property and no carrier label**; it is transport on
the label-0 substrate, and the carrier is sustained only as a downstream consequence of its site
being fed.

```toml
name = "diffuse", priority = 1
LHS  nodes ["hi", "lo"], edges [["hi","lo"]], labels hi = 0, lo = 0, vars_distinct
     guard  hi.energy >= 2.0      # donor has surplus
     guard  lo.energy <  1.0      # recipient is depleted
RHS  same nodes, edge re-declared
     hi.energy subtract 1.0
     lo.energy add      1.0
```

Committed properties of the design:

- **Conservative.** Exactly `1.0` moves; the rule is neither a source nor a sink. The sole unmatched
  source remains `influx` and the sole sink remains `metabolise` — the S3 energy accounting is
  unchanged, and the ledger identity must still balance to 0.
- **One-directional, non-oscillating.** The guards (`donor >= 2.0`, `recipient < 1.0`) enforce a
  strict gradient, so two nodes cannot trade a quantum back and forth forever.
- **Delivers `R = 1.0`** per receiving step ⟹ `f <= 0.5`, comfortably above the `1/3` sustain
  threshold. Predicted carrier balance `+0.25/step` up to satiation, then an
  eat/metabolise limit cycle at satiation. **This prediction is committed here and checked against
  the realized trajectory.**
- **Priority 1** — above `influx` (0), so a depleted site is fed the full quantum rather than 0.34;
  at parity with `harvest`, which wins ties by rule id (harvest is loaded first), so the carrier eats
  when it can and is refilled when it cannot. Below `relocate` (2), so a *motile* carrier still
  prefers to move — which keeps the D-vs-B contrast non-degenerate rather than silently collapsing
  arm B into arm D.

## Predicted (and therefore falsifiable) behaviour of the bare substrate

At the L1 operating point (`initial_substrate_energy = 5.0`, influx 0.34, **zero carriers**) every
node starts above the depletion threshold and influx drives all of them to the cap, so **`diffuse`
never fires** and the bare substrate is expected to be *identical* to the plain static substrate.
That is a strong firewall statement — adding the rule perturbs bare physics not at all — but it also
means the operating point alone does not exercise the rule. L1.2 therefore characterises it twice:

1. **at the operating point, bare** — expect 0 firings, node/edge/energy trajectories identical to
   static (this is the no-back-reaction check);
2. **on a gradient fixture** (one node loaded, the rest empty, influx off) — expect exact
   conservation of total energy, spreading transport, flat `n(t)`, unchanged degree, and reported
   partition stationarity.

`diffuse` is a *gradient-driven* rule: no gradient, no transport. Reporting it as inert at the
operating point is the honest characterisation, not a gap.

## The arms L1.3 will build

| arm | replicate | relocate | diffuse | expected |
|---|---|---|---|---|
| A | yes | no | no | replicating baseline (F8) |
| B | no | **yes** | no | K-track arm B: motile, sustained, 248 relocations |
| **B′** | no | **yes** | **yes** | motile *on D's substrate* — the clean same-substrate control |
| **D** | no | **no** | **yes** | **the specimen: present, stationary, non-replicating** |
| C | no | no | no | the token, dead at 120 |

- **D vs B′** isolates **motility** cleanly: same substrate, same rule set, differing in `relocate`
  alone. This is the decisive contrast, and it is only decisive if B′ actually still relocates —
  which the priority choice above is designed to ensure and which is **measured, not assumed**
  (realized relocation counts are reported for every arm).
- **D vs C** isolates **presence**, but *not* as a single-variable contrast: sustaining a stationary
  carrier necessarily requires the energy-delivery change, so D and C differ in `diffuse` as well as
  in duration. This confound is stated here in advance rather than discovered later; it is the reason
  D-vs-B′ carries the verdict.

All arms use N = 6 planted markers (except A, N = 1, the F8 configuration), seed 12345, 6000 steps —
inherited from the K-track; re-selecting them is the forbidden move.

## Pre-committed reading (L1.4)

Read **only** on a partition that passed L2.1 Tier 1; otherwise the result is reported as BLOCKED.

- **D detectable** (and C not) ⟹ **presence, not motility, is load-bearing.** A carrier is detectable
  merely by being structurally present across time — independent of replication, turnover, *and*
  motion. This is the strongest defensible form of the verdict.
- **D undetectable while B/B′ are** ⟹ **motility was the operative variable.** The property is
  structural *activity from motion*; B's earlier detectability is reattributed to its relocations and
  the verdict narrows toward construction. Per V7 rule 2 this is a finding: motion is **not** to be
  added back, and "present" is **not** to be redefined.
- Report D's **structural and metabolic** z-scores against A, B, B′ and C, so the always-on
  definitional signal (K3: dead token z_eb = 9.59) is separated from whatever structural signal D
  carries.
