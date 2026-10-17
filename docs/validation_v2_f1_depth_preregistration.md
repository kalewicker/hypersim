# F1 — WL depth-selection pre-registration

**Committed before any CE coefficient was computed at any depth other than 1.**
This document exists so the depth choice is auditable: plan rule 6 makes WL depth a
knob with large CE leverage, and selecting the depth that maximises CE is the same
forbidden move as tuning influx to open a band. The rule below is decided entirely
from the F0.2 table, which contains **no CE values**.

Harness: `crates/hg-sim-core/tests/f0_partition_resolution.rs`.
Plan: `docs/validation_v2_fix_plan.md` §F0–F1.

---

## What F0 established (no CE involved)

### F0.1 — configured depth in the CE pipeline

| consumer | WL radius |
|---|---|
| `experiment6_ce_across_onset.rs:102` | `1` (hardcoded literal) |
| `experiment6b_birthdeath_ce.rs:95` | `1` (hardcoded literal) |
| `v3_ce_validation.rs:101` | `1` (hardcoded literal) |
| `runner.rs:1045` | `1` (hardcoded literal) |
| `hg-causal/src/observer.rs` | `WL_RADIUS = 1` |
| `hg-eval/src/thresholds.rs` | `WL_RADIUS = 1` |
| `hg-selfmod/src/runner.rs` | `cfg.wl_radius`, **default 3**, configurable |

The CE stack is internally consistent at depth 1 — there is no mismatch *within* it.
What is worth surfacing is that the depth is a **magic number duplicated across four
call sites**, never surfaced as a parameter or shared constant, while a sibling
subsystem independently chose 3.

### F0.2 — carrier class count vs depth

Seed 12345, influx 0.34, bootstrap threshold 3.0, 6000 steps; class inventory sampled
every 20 steps (300 snapshots).

| d | total M | carrier classes | substrate classes | group sizes (sub, carr) |
|---|---|---|---|---|
| **1** | 9 | **1** | 8 | **(8, 1)** |
| **2** | 145 | **4** | 141 | (141, 4) |
| 3 | 1427 | 71 | 1356 | (1356, 71) |
| 4 | 3834 | 1021 | 2813 | (2813, 1021) |
| 5 | 6503 | 2356 | 4147 | (4147, 2356) |

Depth 1 reproducing M = 9 exactly matches the `M = 9` reported for the 6a model in
`validation_v1_findings.md`, confirming this diagnostic is measuring the same partition
the CE pipeline built.

**Why depth 1 could not have split carriers.** `compute_wl_hash`'s `base_hash` is
`WL_SEED + label + energy field`, and ALife energy lives in *property key 1*, which WL
deliberately excludes (the Q2 invariant, so per-carrier genome traits cannot mint a class
each). Every substrate site therefore has an identical base hash. A carrier has exactly
one neighbour — its occupancy site — so at radius 1 every carrier hashes to
`f(carrier_base, {site_base})`, **necessarily identical for every carrier regardless of
which site it occupies**. Sites split at radius 1 only because their neighbour-multiset
*sizes* differ (degree / occupancy). The site's own neighbourhood, the thing that
actually varies, first enters the carrier's hash at radius 2. The (9, 1) partition was
structurally forced by the depth, not a fact about the biology.

### F0.3 — carrier site-occupancy across substrate classes

Carriers occupy **4 of the 8** depth-1 substrate classes over the run
(44.5 % / 44.4 % / 5.5 % / 5.6 %, n = 10 109 observations). Occupancy is *spread*, not
concentrated, so there was genuine site heterogeneity available for carriers to inherit —
they simply could not express it at depth 1.

---

## The selection rule (pre-committed, priority order)

1. **Choose the minimum depth at which the carrier group resolves into more than one
   class.** This is the coarsest description that can represent the distinction under
   test, and it is decided by the F0.2 table alone.
2. **Additionally report the WL convergence depth** — the depth at which total class
   count `M` stabilises — as a secondary reference, and run F2 at both if they differ.

Explicitly forbidden: selecting depth by inspecting CE, by p-value, or by null margin.

## Applying the rule

**Rule 1 selects depth 2.** Carriers first resolve into more than one class (4 classes)
at d = 2.

**Rule 2 selects nothing.** `M` does not stabilise anywhere in 1..5 — it grows
monotonically and steeply (9 → 145 → 1427 → 3834 → 6503), with carrier classes reaching
2356 at d = 5 against a population of only ~35 live carriers. That is the open-registry
class-explosion regime the WL Q2 comment warns about — each carrier approaching its own
class — **not** a convergence point. There is therefore no second depth to run F2 at, and
this is reported as an absence rather than substituted with the largest tested depth.

> Note this cuts *against* convenience rather than for it. Deeper partitions are not
> being avoided because they would look bad; per plan rule 6 the expected effect of
> greater depth is **larger `M` → larger `EI_micro` → lower CE**, since CE is
> `EI_macro / EI_micro`. Depth 2 is selected because it is the minimum resolving depth,
> and it is also the depth least likely to depress CE. That coincidence is stated here
> explicitly so it cannot later be mistaken for a favourable choice made after seeing a
> coefficient.

### Committed: **F2 runs at WL depth 2.**

---

## What F2 will report, and the pre-committed interpretation

F2 repeats the V3 battery at depth 2 on the **6b vacuum-augmented model** — the only
model whose `EI_macro` is not structurally pinned (6a reduces exactly to
`CE = 1/EI_micro` and cannot falsify anything, so it is reported but not tested).

- **F2.2 must report the null's support size**, i.e. how many distinct partitions the
  label permutation can actually produce. V1's fatal weakness was support ≈ 10. At group
  sizes (141, 4) the permutation null ranges over `C(145, 4) ≈ 1.77 × 10⁷` distinct
  partitions — roughly six orders of magnitude more resolution. The exact figure will be
  computed from the realised TPM classes at F2 time.
- **p < 0.05 with support ≫ 10** ⟹ the boundary carries partition-specific causal signal,
  and the V1 negative was a resolution artifact.
- **p ≥ 0.05 with adequate support** ⟹ the negative is now *properly powered* and is a
  substantially stronger result than V1's.

**CE at depth 2 has not been computed at the time of writing.** Both outcomes are
acceptable; neither licenses a further change of depth.
