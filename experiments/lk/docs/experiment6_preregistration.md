# Experiment 6 — pre-registration: causal emergence across the replication-onset boundary

**Committed 2026-07-15, BEFORE building the carrier/substrate partition or reading any
CE number.** This is the "separate experiment doc" the ALife plan's S7 points to. It
commits the measurement, the predictions, and the falsification conditions *first*; the
result is reported against them in `experiment6_findings.md`, not tuned to them.

The forbidden move (scope §8.1, restated for CE): **do not tune anything to produce
CE > 1.** That is the same move as tuning influx to make the turnover band appear.

---

## 0. What Experiment 5 hands to Experiment 6 (and its caveats)

From `experiment5_scorecard.md` + the CE-bridge validation (2026-07-15):
- **Criterion 6 (SOUND):** `non_trivial_replicator_count` 0 → >0 is a structural depth-K
  sustained-self-copying event, not a label/WL artifact (negative control withholds it
  from trivial copiers). So there IS a genuine replication-onset boundary to measure across.
- **Criterion 7 (SOUND):** determinism holds cross-process; energy books balance at every
  sweep point; the substrate subgraph equals a bare control at every influx (no
  back-reaction). So the carrier/substrate partition is over a **clean, static substrate**.
- **Turnover was unblocked on a STATIC substrate** (asymmetric `replication_threshold`
  lever), *not* route A (churn) or route B (aging). So the partition is over the **same
  micro state space** — exactly the condition §142 of the scope doc requires. The
  coarse-graining assumptions do **not** change.

Two caveats carried forward, neither of which blocks the CE measurement:
- **Criterion 5 is degenerate-only** (selection is the free node-funded collapse, drift-
  dominated in magnitude). CE needs the partition and the onset boundary, both clean — it
  does **not** need non-degenerate selection. But the biological claim is bounded: Exp 5
  produced a stable, replicating, turning-over population *without* open-ended selection.
- **The turnover band is seed-fragile** (≈57–71% establishment). Experiment 6 therefore
  **pins seed 12345** (which establishes) and reports establishment explicitly; it does not
  average over seeds that never entered the regime.

---

## 1. The measurement

### 1.1 Micro state space
The micro dynamics are the **WL-class transition matrix (TPM)** already built by
`CausalObserver` / `TpmAccumulator`: each node's local WL class (radius `WL_RADIUS`) is the
micro state; surviving-node class→class transitions between observation windows populate the
row-stochastic TPM. `EI_micro = H_int − H_noise` over that matrix (`ei_from_tpm`).

WL sees label + structure but **not** the energy property (S0 decision), so a static
substrate node has a **fixed** WL class (self-transitions), while a carrier's class changes
as occupancy edges are created/destroyed by birth/death/dispersal. The carrier dynamics are
therefore the live part of the micro TPM.

### 1.2 Macro partition — carrier / substrate (the "real partition")
The macro coarse-graining groups each WL class by its **`motif_class`**: substrate
(`motif_class == 0`) vs carrier (`motif_class != 0`, i.e. `CARRIER_LABEL`), assigned by the
same `HypergraphMotifClassifier` used since S2. Compact indices: if a group is absent (e.g.
**pre-replication: no carrier classes exist**), the partition collapses to K = 1 →
`EI_macro = 0` → **CE = 0** by construction. That degenerate baseline is the whole point:
CE must go **0 → nonzero by biology**, not by a bolted-on second system.

`CE = EI_macro / EI_micro` under this partition (guarded: `EI_micro = 0 ⟹ CE = 0`).

This partition is NEW (S7.2). The pre-existing `functional_partition` groups by the
constructor registry (fidelity/cycles) which is **empty on ALife runs** — it is not the
Experiment-6 partition and is not used here.

### 1.3 The boundary and the comparison (PRIMARY design)
**Paired, same-seed, bare-vs-live** — the S0.2 control discipline applied to CE:

| arm | rules | carrier classes | expected partition |
|---|---|---|---|
| **bare** (pre-onset) | substrate only, alife OFF | none ever | K = 1 → CE = 0 (degenerate baseline) |
| **live** (post-onset) | substrate + biology, alife ON, turnover regime | present, churning | K = 2 → CE = a real number |

Same seed (12345), same substrate, same cadence and `min_transitions` gate. The
replication-onset boundary is the presence/absence of the `replicate` process; CE is measured
under the identical carrier/substrate partition in both arms. **Report both `EI_micro` and
`EI_macro`, the cadence, and the transition count** — a bare CE number is not a statement
(the cadence is part of the measurement's definition).

### 1.4 Secondary (within-run temporal) view
In the live run, also report CE over an **early window (around onset, few carriers)** vs a
**late window (established turnover)**, to show the 0→nonzero transition happening *inside a
single run* as replication sustains — a weaker but assumption-free corroboration of the
paired-control result.

### 1.5 Fixed measurement parameters (committed)
- Seed **12345**; 6×6 static PureCrystal (36 sites); MaxParallel.
- Turnover regime: `influx_rate = 0.34`, `bootstrap_replication_threshold = 3.0`
  (`configs/experiment5.toml`), `initial_substrate_energy = 5.0`, `seed_carrier_marker`.
- `causal.enabled = true`, carrier/substrate partition ON; other CE strategies may run but
  are not the Experiment-6 claim.
- `ei_minimum_transitions = 1000` (Empirical certainty gate — no CE claim below it).
- Steps ≥ 15 000 so the late window clears the transition gate with an established population.

---

## 2. Predictions (committed)

**P1 (the bridge — primary).** Under the carrier/substrate partition, **CE_bare = 0**
(degenerate: no carrier classes ⟹ K = 1) and **CE_live > 0** (biology makes the partition a
genuine 2-group coarse-graining with real macro causal structure). The headline of
Experiment 6 is this **0 → >0** crossing, mirroring criterion 6's replicator crossing.

**P2 (Finding F1 — direction, not magnitude).** **CE_live < 1** is expected and is **not a
failure.** CE is biased *downward* on open registries, so CE > 1 would clear a conservative
bar (a strong positive) but CE < 1 falsifies nothing. We predict CE_live ∈ (0, 1). We do
**not** predict, target, or tune toward CE > 1.

**P3 (temporal).** In the live run, CE rises from ≈0 in the pre-onset/early window to its
live value once turnover is established (secondary corroboration of P1).

**P4 (micro sanity).** `EI_micro` is dominated by carrier-class churn; on the bare arm the
static substrate gives a low, saturating `EI_micro` (self-transitions), consistent with the
S5-era "bare-substrate CE < 1 baseline."

---

## 3. Falsification / honest-negative conditions (committed)

- **F-A. CE_live ≈ 0 (P1 fails).** If the carrier/substrate coarse-graining captures no macro
  causal structure (EI_macro ≈ 0 even with carriers churning), then replication onset does
  **not** produce causal emergence under this partition. That is a **real negative result**,
  reported as such — Experiment 6 would conclude "no CE across the boundary under the
  carrier/substrate partition," not be quietly repaired.
- **F-B. CE_bare > 0.** If the bare arm yields CE > 0 under this partition, the partition is
  not actually degenerate pre-onset (a bug or a mis-specified `motif_class`), and the 0→>0
  crossing is an artifact. Must be diagnosed before any live claim.
- **F-C. Transition gate not cleared.** If the live late window has < 1000 transitions, no CE
  claim is made (certainty table). Report "insufficient data," do not lower the gate to
  manufacture a number.
- **F-D. Determinism breaks.** If CE_live is not reproducible across processes at the pinned
  seed, the measurement is void (criterion 7 is a hard gate).
- **The forbidden repair:** should CE_live come out ≈ 0 or < 1, we do **not** switch to a
  partition, cadence, or regime chosen to raise it. Any post-hoc partition change is a new
  pre-registration, reported separately.

---

## 4. What a result means (decision table)

| outcome | interpretation |
|---|---|
| CE_bare = 0, 0 < CE_live < 1 | **Expected success.** Replication onset produces macro causal emergence under the carrier/substrate partition; the conservative (downward-biased) bar puts it below 1. The 0→>0 bridge holds. |
| CE_bare = 0, CE_live > 1 | **Strong success (surprising).** Clears the conservative bar. Report with extra determinism + accounting scrutiny; do NOT celebrate a tuned number — confirm nothing was tuned. |
| CE_bare = 0, CE_live ≈ 0 | **Honest negative (F-A).** The partition captures no causal emergence; report it. Candidate next directions (each a fresh pre-registration): a finer functional partition, or the non-degenerate-selection variant (§1 option 2). |
| CE_bare > 0 | **Artifact (F-B).** Diagnose the partition before any claim. |

---

## 5. Provenance

Depends on: `experiment5_scorecard.md` (Exp-5 achieved + validation), scope doc §142
(carrier/substrate is the Exp-6 partition), `crates/hg-causal/src/ei.rs` (EI/CE calculator,
pre-existing), Finding F1 (CE downward bias). The carrier/substrate partition itself is built
in S7.2 and documented in `docs/metric_formulas.md`.
