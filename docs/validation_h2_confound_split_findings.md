# H2 — the replication-vs-duration confound, resolved: it was DURATION

Reported against the V4 H-track plan (H1–H2) and the V5 J-track forward path. This completes the
three forward-path steps opened at the end of the J-track: (1) confirm the energy-band partition
stays resolved under bounded churn; (2) build the bounded churn substrate as physics; (3) run the
H2 confound-split. Harnesses: `h1_churn_substrate.rs`, `h2_step1_energyband_on_churn.rs`,
`h2_step3_sustain_trajectory.rs`, `h2_step3_split.rs`. Seed 12345 throughout.

---

## The single sentence

**Once a non-replicating population is *sustained* — which requires movement to fresh energy
(motility), not replication and not substrate churn — the carrier/substrate boundary is detectable
on the very WL partition that the G-track's dying sterile token failed, so the G-track's
replication-vs-duration confound resolves to DURATION: the boundary is sustained turnover, and
replication-per-se is not required.**

---

## Step 2 (H1) — a bounded-churn substrate was built and characterised as physics

`h1_churn_substrate.rs`. A balanced energy-gated churn rule: a label-0 node fills via influx to the
cap, then a path-preserving churn (`[a,old,b] → [a,fresh,b]`) deletes the full, unoccupied `old` and
mints a fresh empty node re-bridging `a`–`b`. Net-zero node count (n(t) exactly flat by
construction); disjoint guards (influx energy<cap, churn energy≥cap) so no rule conflicts; an
occupied-guard so it never deletes a carrier's site. Characterised bare (0 carriers, firewall; rate
= the influx parameter, not tuned to biology):

| influx | n | identity turnover | mean degree | energy-band M/TWTF | E-spread |
|---|---|---|---|---|---|
| 0.34 | flat 36 | full (0% original survive, 0.35 fresh/step) | 4.72 → 1.33 | 21 / 0.000 | ~19 |
| ≥1.0 | flat 36 | **none** (synchronised freeze) | 4.72 | 1 / 0.000 | 0 |

So a bounded-churn substrate exists (flat n, genuine identity turnover, strong energy
heterogeneity). Characterised limitations, honestly: mean degree erodes to a sparse chain (~1.3) —
fixed-arity churn cannot reconnect a high-degree node's every edge — and high influx synchronises
all fills to the cap on one step and deadlocks. The energy-band partition stays stable and bounded
(M=21, TWTF 0.000) through the churn, confirming the J2 prediction on a *real* bounded substrate.

## Step 1 — the energy-band partition stays resolved under bounded churn

`h2_step1_energyband_on_churn.rs`. With the F8 **replicating** population, the substrate saturates
and stays below the cap, so the churn rule never fires — static and churn energy-band detectability
are byte-identical (M=34, k_carrier=16, 0/200, support 2.2×10⁹). Churn only bites when free nodes
exist. The bare characterisation (above) already showed the energy-band classes stay bounded and
stationary under active churn, so the J2 instrument survives real bounded churn, not just the static
substrate and the unbounded proxies. **The one thing triangle's explosion could not show (does the
energy-band partition stay resolved under bounded churn) is now shown.**

## Step 3 / H2.1 — what actually *sustains* a non-replicating population? (a reframing)

`h2_step3_sustain_trajectory.rs`. The plan hoped substrate churn would sustain a non-replicating
population by "fresh nodes recolonised". It does not:

| arm | peak | last-alive |
|---|---|---|
| replicating (F8), static | 36 | 6000 |
| sterile non-motile, static | 1 | **120** (the G-track token) |
| sterile non-motile, **churn** | 1 | **108** |
| sterile **MOTILE**, static | 1 | **6000** |
| sterile MOTILE, churn | 1 | 6000 |

**Churn mints fresh *free* sites, but a non-replicating, non-motile carrier has no way to reach
them** — recolonising requires *movement*. The missing mechanism is **motility** (the S6.3-deferred
mechanic), and it sustains a non-replicating population *on the plain static substrate*: a single
grazing carrier that relocates to an adjacent unoccupied higher-energy site when hungry lives the
full run. So the confound-split is achievable on static — where the WL and energy-band partitions
both work cleanly — without needing the churn substrate at all. A `relocate` rule was added: the
same individual moves its occupancy to a fresh site; no child, no heredity — movement, not
replication.

## Step 3 / H2.2 — the confound-split, both partitions

`h2_step3_split.rs`, pre-committed interpretation committed before running. Each arm scored with the
**WL** partition (the G-track's instrument — the one whose confound we resolve) and the energy-band
partition. Arm B (sustained non-replicating) is a motile-sterile bloom below carrying capacity.

| arm | sustained? | WL: k / p / support | e-band: k / p / support |
|---|---|---|---|
| A replicating (F8) | yes (6000) | 4 / **<.005** / 6.5×10⁷ | 16 / <.005 / 2.2×10⁹ |
| B non-repl **SUSTAINED** (motile, N=6) | yes (6000) | 4 / **<.005** / 2.0×10⁷ | 15 / <.005 / 5.6×10⁹ |
| C non-repl **NOT** sustained (token, N=6) | no (120) | 2 / **0.930** / 4.4×10² | 16 / <.005 / 2.2×10¹⁰ |

Density sweep (motile-sterile): N = 1, 6, 12, 24 all sustain to 6000 and are WL-detectable; N = 36
saturates (no free site to relocate to) → dies at 120 → WL not detectable. Motility sustains only
below carrying capacity.

**The decisive comparison is B vs C on the WL partition.** Both are non-replicating, same 6 markers;
they differ *only* in duration — B is sustained by motility, C is the G-track token that dies at 120.
B is WL-detectable, C is not. So the G-track token's WL non-detection was **duration** (a short bloom
with too few transitions), not replication. Restore duration *without* replication and the WL
boundary returns. And A vs B (differ in replication, both sustained) are *both* WL-detectable, so
replication adds nothing to WL-detectability beyond sustaining the population.

A separate finding: the **energy-band** metabolic partition detects *all three* arms, including the
dead token — a carrier is metabolically distinct from substrate whether or not it is sustained. So
the energy-band signal is always-on and cannot by itself discriminate this question; the WL
partition is the one that carries the replication/duration distinction, and it says **duration**.

---

## Verdict

**The replication-vs-duration confound the G-track left open is resolved: it was DURATION.** The
"sustained-turnover boundary" name is confirmed and sharpened — what the boundary needs is a
*sustained* carrier population (enough duration and transition share), and replication is merely one
way to sustain one. Motility sustains an equally-detectable non-replicating population, and a
non-sustained non-replicating population is undetectable exactly as the G-track found. Replication-
per-se is **not** load-bearing for the boundary's detectability.

## Scope and honest caveats

- **Single seed (12345).** This is the confound-split at the F8 anchor seed; a multi-seed replication
  (as F8→F7 did) would harden it. The B-vs-C contrast is clean and large, but n=1 seed.
- **Motility is a new mechanic**, added to sustain the non-replicating population. It is a movement
  rule (same individual relocates), deliberately not replication (no child, no heredity), so it is a
  fair way to restore duration without restoring replication. But it is a mechanic the G-track did
  not have, and the result is a statement about *this* biology plus motility.
- **The churn substrate (H1) turned out unnecessary for the split**, but is a genuine deliverable: a
  characterised bounded-churn substrate now exists (with its sparsity/synchronisation limitations
  recorded), and it independently confirmed the energy-band partition survives bounded churn.
- **Magnitude remains out of scope** (F2.4: CE value is registry-dominated); this is a detectability
  result, as all the V-track results are.
- The energy-band "always-on" metabolic detectability is itself a finding worth follow-up: it means
  the carrier/substrate boundary has a metabolic signature independent of duration and replication,
  which is a *different* object from the WL structural/turnover boundary.
