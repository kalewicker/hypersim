# V5 / J-track — J2 findings: a churn-stable instrument exists (metabolic, not geometric)

Reported against `~/Downloads/validation_v5_nonpartition_plan.md`, the committed criteria
`docs/validation_v5_j0_ablation_definition.md` (J0.1, `165c925`) and
`docs/validation_v5_j2_variable_persistence_criterion.md` (J2.1, `57ff48f`). Harnesses:
`j2_variable_persistence.rs`, `j2_churn_invariant_resolution.rs`, `j2_energyband_stationarity.rs`.

J0 sent the track to J2 (carrier WL structure is substrate-borrowed). J2's gate was expected to
close the program's instrument search as a significant negative. **It did not — J2.1's STOP does not
fire.** A churn-stable resolution exists; it is metabolic, not geometric.

---

## The single sentence

**The carrier/substrate boundary has a second, carrier-intrinsic signature — the carrier's metabolic
(energy-band) dynamics — that WL excluded and J0's structural ablation never tested; unlike the
geometric WL resolution it recovers F8 detectability on static data *and* stays stationary under
substrate motion, so the sustained-turnover confound-split that the WL/CE instrument could not
support is instrumentable after all, by a metabolic-state partition.**

## Headline

| | WL partition (structural) | (label, energy-band) partition (metabolic) |
|---|---|---|
| recovers F8 on static (seed 12345) | 0.1633, **0/200**, k=4, support 6.5×10⁷ | **0.2022, 0/200**, k=16, support **2.2×10⁹** |
| survives J0 site-ablation | **no** (collapses to 1 class) | n/a — not geometry-based |
| TWTF under motion (H0 gate) | 0.005 → **0.515 / 0.984** (FAILS) | 0.010 → **0.001 / 0.000** (CLEARS) |

---

## J2.2 — micro-variable persistence (and an honest correction)

The cheap probe measured whether any fixed micro-variable set persists under the motion builtins.
Result and honest correction: the available builtins **grow (add nodes) without deleting**, so
original node identities *persist* (37–48 survive the window) — they cannot exhibit the
identity-churn a real deleting churn substrate would, the same gap the route-A probe found ("no
builtin churns-without-growing"). So J2.2 cannot demonstrate node-identity churn. What it does show:
the **carrier** population turns over as individuals (≤1 persists, static and motion alike), so
carrier micro-variables are never a persistent fixed set. F8 handled this not with persistent
carrier variables but with a **class resolution** — which shifted the decisive question from
variable persistence to *resolution*: is there a churn-invariant carrier resolution above the F0
bar?

## The crux — energy-band resolution (J2.1 requirement 2, rule 4)

J0 ruled out carrier-intrinsic *WL* structure. It did not, and by the Q2 invariant could not, test
the carrier's **energy** (property key 1): WL-excluded, carrier-owned (not site-owned), and — the
key property — a **bounded, recurrent value-set** rather than an unbounded structural hash. Banding
energy by `floor(energy)` gives a small class set (M≈34) that survives coarse-graining.

On the static F8 run, the `(label, energy-band)` partition **recovers F8 detectability**: real CE
0.2022 ≫ null mean 0.0277, **0/200** (p < 0.005), k_carrier = 16, null support **2.2×10⁹** — while
the WL partition reproduces its anchor (0.1633, 0/200, k=4) in the same run. The boundary is
detected via the carrier's distinct **energy-transition kernel** (harvest-to-satiation then
metabolise, versus substrate influx/harvest-drain), a genuinely carrier-intrinsic causal signature,
not borrowed geometry. This satisfies rule 4 (recover F8) for a churn-invariant resolution.

## Re-clearing the H0 TWTF gate for the new partition (mandatory, not assumed)

The plan forbids assuming the stationarity gate transfers. Measured head-to-head under the same
motion builtins as H0.3, with the WL column reproducing H0 exactly (0.005 / 0.515 / 0.984) as a
harness check:

| motion | WL TWTF | energy-band TWTF |
|---|---|---|
| static | 0.005 | 0.010 |
| growth_treadmill | **0.515** | **0.001** |
| triangle_subdivide | **0.984** | **0.000** |

The energy-band partition **re-clears the gate**: its TWTF stays ~0 under motion because a fresh
churned node (energy 0) falls into an **existing** band (band 0) — bands do not proliferate the way
WL structural hashes do. This is precisely the property H0 found WL lacked.

**Honest caveat on resolution under real churn.** Under triangle_subdivide's *explosion* the
energy-band partition stays stationary but collapses to M=2 — the biology dies in the blow-up, not a
partition failure. So "energy-band stays *resolved* (k_carrier ≥ 2, adequate support) under real
**bounded** churn" is *encouraged* (growth_treadmill keeps M=31) and strongly supported by the
static F8-recovery (k=16, support 2.2×10⁹), but is **not proven** — proving it needs the bounded
churn substrate that does not yet exist. Stationarity is proven; sustained resolution under bounded
churn is the first thing a J2.3/H1 build must confirm.

---

## J3 — the report-level consequence (sharper than either branch anticipated)

J0 measured that the boundary's **structural** detectability is substrate-geometric (borrowed, dies
under churn). J2 measured that the boundary *also* has a **metabolic** detectability that is
carrier-intrinsic and churn-stable. So the honest §9/§10 statement is two-sided:

- The **geometric** signature (the 4 WL classes, the F8 result as originally framed) is a property of
  the frozen crystal and does **not** survive detachment from it (H churn, J0 ablation).
- The **metabolic** signature (energy-transition kernel) is a property of the biology itself,
  survives churn, and independently recovers detectability. This is a *stronger* biology-side reading
  than the program had: the carrier subsystem has a substrate-independent causal signature after all
  — just not the one the detectability result was originally built on.

## Verdict and what is now unblocked (not yet built)

**J2.1 STOP does not fire.** Contrary to the expectation set by the H-track dead end, an instrument
survives: a metabolic-state (energy-band) partition, F8-recovering and churn-stationary. The
sustained-turnover confound-split (H2) — build a bounded churn substrate, sustain a *non-replicating*
population by substrate turnover, and score the three-way detectability with the energy-band
partition — is **instrumentable** and no longer blocked by the CE-well-definedness wall.

It is **not yet built**, and that is the honest boundary of this track. The remaining work is
substantial and gated in order:
1. **Confirm sustained resolution:** that the energy-band partition keeps k_carrier ≥ 2 with adequate
   support under a *bounded* churn substrate (untested here; the unbounded proxies cannot show it).
2. **Build the bounded churn substrate as physics first** (H1 firewall: characterise `n(t)` flat +
   identity turnover on its own terms, before biology) — the route-A/H1 finding stands that no
   existing builtin provides bounded churn, so this is a new rewrite-family sub-investigation.
3. **Run H2** with the energy-band partition: sustained non-replicating class detectable or not →
   the replication-vs-duration split the G-track left open.

These are a build effort of their own and are recorded here as the now-open forward path, to be
scoped with the user rather than started under this track's cheap-gate budget.

## The single sentence J0/J2 answer, together

**Carriers have no *structural* structure of their own once the substrate's geometry is removed (J0),
but they do have a *metabolic* one (J2) — an energy-band causal signature that is carrier-intrinsic,
recovers the boundary on static data, and stays stationary under churn where the geometric partition
does not — so the sustained-turnover boundary can be instrumented by a partition after all, just not
the one it was discovered with.**
