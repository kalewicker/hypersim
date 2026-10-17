# F6 — pre-registrations for the record

Two facts established during the V- and F-tracks that are **harmless now and load-bearing
later**. Both are registered here so they cannot be rediscovered as a surprise, and so any
future work that trips over them is answerable to a record that predates it.

Source: `docs/validation_v2_fix_plan.md` §F6. Measurements from
`docs/validation_v1_findings.md` and `docs/validation_v2_findings.md`.

---

## F6.1 — `harvest_rate` has drifted far outside its nominal range, and that is a trap

**Measured.** V1.3 recorded population-mean `harvest_rate` at **−13.948**, from a bootstrap
seed of **1.00**, after ~500 generations. Nominal range (`genome.rs::TRAIT_BOUNDS`) is
`[0.0, 2.0]`. `maintenance_cost` likewise sits at 0.258 from a seed of 1.0, and
`replication_threshold` at −3.72 from 3.0.

**Why it happened, by design.** S2.1 chose option (b): jitter is **UNCLAMPED**. The stated
rationale was that the engine stays ALife-agnostic and the biology must be robust to
out-of-range traits (a negative `harvest_rate` simply never harvests). `TRAIT_BOUNDS` is
genome *semantics* for sampling and human reasoning — explicitly **not** an enforced
invariant.

**Why it is harmless today.** Deviation #4 keeps harvest and metabolise on **fixed quanta**,
so `harvest_rate` (key 10) and `maintenance_cost` (key 12) are read by no rule. A trait no
rule reads can drift anywhere without consequence — this is exactly what makes them a valid
matched drift null (V2.1, F4).

**The registered hazard.** *If deviation #4 is ever reversed* — i.e. if
`genome_scaled_metabolism` is switched on for a run whose population descends from one
evolved under fixed quanta — **the population begins from wildly out-of-range values**. A
carrier with `harvest_rate = −13.9` does not harvest; it starves regardless of the economy.
Any dynamics observed under such a switch are an artifact of **accumulated drift on a
previously-inert trait**, not a property of the genome-scaled economy.

**Committed consequence.** A genome-scaled experiment must either (a) start from a fresh
bootstrap population rather than an evolved one, or (b) report the inherited trait
distribution alongside its results and treat the first generations as contaminated. Observing
"genome-scaled metabolism causes immediate collapse" without doing one of these is a
misattribution, and this note pre-dates any such run.

> Experiment 5b did activate these traits and reported turnover collapsing in 5/6 seeds. That
> experiment bootstrapped fresh (`maintenance_cost` set to 0.5 at bootstrap so the initial
> economy was unchanged), so it is **not** subject to this hazard. The registration is for
> future runs that might not take that care.

---

## F6.2 — Turnover fragility: establishment rate must be reported with any claim

**Measured, and revised downward.** The record has carried **57–71 %** establishment since
V2.1 and `alife_band_reality`. F4 measured **32 % (30/94)** on an unbiased sequential seed
sweep, and an earlier 60-attempt slice of the same sweep gave **23 % (14/60)**.

The 57–71 % figure came from **small curated seed sets** — n = 7 in `alife_band_reality`,
n = 10 in `alife_criterion5_selection_vs_drift`. Those sets are not wrong, they are just
small and were not drawn adversarially. **The turnover regime is materially more fragile than
the record has stated.**

| source | seeds | establishment |
|---|---|---|
| `alife_band_reality` (influx 0.28) | 7 | 71 % |
| `alife_band_reality` (influx 0.34 / 0.40) | 7 | 57 % |
| `alife_criterion5_selection_vs_drift` | 10 | 60 % |
| **F4 sequential sweep (influx 0.34)** | **94** | **32 %** |

**Committed consequences.**

1. Any future claim resting on the turnover regime **must report the establishment rate
   alongside it**, and must state the seed set and how it was chosen.
2. **Single-seed results are not generalisable** and must not be presented as properties of
   the regime. This includes the pinned establishing seed 12345 used throughout Experiment 6
   and the F-track — every CE number in this project is a single-seed measurement on a regime
   that fails to establish roughly two thirds of the time.
3. A claim that depends on the regime being *typical* rather than *possible* needs a
   multi-seed replication at n ≥ 30 established, which at 32 % means budgeting ~95 attempts.

> This is the single largest outstanding caveat on the F2 boundary result. F2 establishes
> that the carrier/substrate boundary is detectable **at seed 12345**; it does not establish
> that this holds across the regime. Multi-seed F2 replication is the obvious next
> measurement and is **not** claimed here.
