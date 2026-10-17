# ERE Phase R6 — Measurement Pass (first results)

Status: EXP-R06, EXP-R07 (multi-seed), EXP-R08, EXP-R10 complete.
EXP-R09 belongs to the R5 outer loop.

## EXP-R08 — the epoch sweep

**Setup.** E realized as epochs-per-estimation-window over
{∞, 50, 20, 10, 5, 1}, arms A2 + A3, 400 transitions per cell.
Rule-set change detected by RCF-multiset signature of all regions;
`tpm_rule_set_stable` = fraction of windows with zero changes inside
(the G-R6 assertion).

**Result (both arms identical — region evolution is arm-independent):**

| E | rule_change_rate | tpm_rule_set_stable |
|---|---|---|
| ∞ | 0.765 | **1.000** |
| 50 | 0.797 | 0.000 |
| 20 | 0.770 | 0.000 |
| 10 | 0.750 | 0.000 |
| 5 | 0.762 | 0.000 |
| 1 | 0.830 | 0.170 |

**Verdict: the intersection is EMPTY** — per the plan's pre-committed
interpretation, endogenous rule change at this kernel rate and
causal-emergence measurement are not simultaneously available at this
scale. Rule-change rate sits at 75–83% at every finite cadence; no
window with E ≤ 50 is ever stationary. E* can only exist if the kernel
rate is throttled (selection pressure, acceptance thresholds) or the
TPM is defined over a coarser partition. That agenda-setting null is
the substantive result.

## EXP-R10 — EI/CE, meta band excluded vs included

**Setup.** WL node classes (radius 2) → per-node class transitions →
TPM, run twice per arm over identical dynamics: substrate-only
(label < META_BASE) vs all nodes. G-R6 honored by construction
(E = ∞ frozen rule set across estimation). Arms: A1-frozen, A3-bare.

**Result (400 epochs, 4788 transitions each):**

| arm | EI_micro excl | EI_micro incl | CE*(excl) | CE*(incl) |
|---|---|---|---|---|
| A1 frozen | 3.5850 | 7.6511 | 0 | 0 |
| A3 bare | 3.5850 | 8.0715 | 0 | 0 |

**M1 divergence measured: |EI_excl − EI_incl| ≈ 4.1–4.5 bits** — including
the meta band roughly *doubles* apparent micro-EI. Region nodes are few but
structurally distinctive; their class transitions carry high surprisal.
Every downstream EI/CE claim must state its filter — the two numbers are not
comparable across filters.

Secondary: A3 > A1 by 0.42 bits on the included filter — the first
engine-consumed difference this harness has resolved (bare recompile shifts
substrate-visible structure slightly vs frozen compilation).

CE = 0 across all coarse-graining strategies at this scale: no partition of
the class space closes over this substrate yet. Consistent with the small
world and the young run; to be revisited at campaign scale.

---

# Earlier results (kernel reach & novelty)

## EXP-R07 — novelty growth curve, meta-excluded

**Setup.** One 600-epoch run, keyed kernel stream. Each epoch: mutate one
region (single-edit kernel), validate, install-or-quarantine; then read back
every region and register it twice:

- **RCF hash** — the plan's novelty class (labels included)
- **SHAPE key** — RCF of the *label-stripped* candidate. This implements
  FM-R6's mitigation directly: a relabel of an already-seen rule is not
  novel.

**Result (seed 0x600D):**

| metric | value |
|---|---|
| kernel events | mutated 592 / installed 505 / quarantined 87 |
| distinct RCF classes | 479 |
| distinct shapes (nontrivial-installed) | **234** |
| cumulative curve (start/q1/mid/q3/end) | 2 / 63 / 122 / 182 / 234 |
| per-quartile new shapes | **61 · 59 · 60 · 52** |

## Verdict

**~LINEAR.** Per-quartile yields are flat-to-slightly-declining — no
acceleration anywhere in the curve.

Under the plan's **pre-committed interpretation (FM-R6)**: this is not an
instrumentation failure. **The Hash Chemistry result reproduces on a
hypergraph substrate.** The mutation kernel's reachable set is genuinely
large (234 distinct rule shapes from 2 seeds), but its discovery rate does
not compound: each single edit draws from a roughly fixed-size effective
neighborhood of the current population, so new-shape yield per epoch stays
constant until saturation drift begins.

This is the most important negative finding the track could produce at this
stage, and it is recorded as such.

## EXP-R06 — do RCF-novel non-trivial rules appear?

**Yes, in abundance**: 479 RCF classes / 234 distinct shapes from 2 seed
rules under a single-edit kernel with ~15% quarantine pressure. The reachable
set is not the bottleneck; compounding discovery is.

## Caveats (owned)

1. ~~**Single seed.**~~ **Retired**: confirmed across 8 seeds
   (0x600D, 1, 2, 3, 7, 42, 99, 777). Shapes_end 130–236, per-quartile
   yields flat-to-slightly-declining on every seed; 0/8 accelerating,
   0/8 saturating. Linear is the stable cross-seed shape.
2. **No selection pressure.** The epoch loop installs every valid child;
   there is no fitness ranking to concentrate the population. FM-R6's
   interpretation is about exactly this regime (kernel-only reach); whether
   *selection* changes the curve shape belongs to R5's outer loop.
3. **Engine consumption is not measured here.** Regions evolve identically
   across arms by construction; what the engine does with novel children is
   EXP-R08's question.

## Queue

- EXP-R08 (epoch sweep) and EXP-R10 (EI/CE, band in/out) need `hg-eval`
  TPM machinery — G-R6 stationarity assertion included.
- R5 outer loop (EXP-R09) now has its baseline: linear kernel-reach curves
  are the null it must beat.
