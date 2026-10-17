# V8 / M3.1 — the inert-carrier control and the expectation table (COMMITTED BEFORE ANY INSTRUMENT WORK)

**Committed before any metabolic-dynamics measure is designed, built, or evaluated**, per V8 rule 4
and the forbidden-response list. The arc's repeated lesson is that instruments must be validated
against a case where they *should* score zero, and that control must exist before the instrument
does, or the instrument gets tuned to the answer.

## The diagnosis this responds to

The gap is **architectural, not instrumental**. The Q2 invariant excludes ALife property keys from
the WL base hash — adopted for a good reason (per-carrier genome properties would mint a WL class
per carrier and drive `EI_micro = log2(M)`) — so the channel the biology actually lives in is
invisible to every structural measure. Arm D is the demonstration: structurally constant, while its
ALife energy changes on 5999 of 5999 steps.

This is the layer firewall's **first demonstrated cost**. The separation that kept the substrate
track clean and made bugs attributable is the same separation that makes arm D invisible.

**The fix is not to breach it.** Hashing ALife energy into WL (forbidden, rule 3) would likely
restore arm D's visibility while converting WL into something close to the energy-band partition —
detecting *carrier-ness* rather than *what carriers do*, i.e. reproducing exactly the ~98%
definitional deflation this track is trying to escape.

## Level versus dynamics — the distinction the instrument must preserve

- The energy-band partition detects that carriers occupy a **distinct energy band**. That is a
  *level*: always-on, definitional, present even in a dead token (K3: z = 9.59 for arm C vs 9.81 for
  arm A — ~98% of arm A's metabolic signal is definitional).
- Arm D exhibits energy **changing** on every step. That is a *dynamics*.

The instrument the program lacks is keyed on the **transition structure** of the ALife energy
channel, not on its distribution.

## The inert carrier

An **inert carrier** is:

- **carrier-labelled** (`Label(100)`, so every structural and label-anchored measure counts it a carrier);
- **sitting in the carrier energy band** (initial energy set to the normal bootstrap value, so the
  energy-band partition classes it with live carriers);
- **subject to no economy rules whatsoever** — it does not harvest, metabolise, replicate, relocate,
  or die. Its ALife energy never changes.

It is a carrier in every definitional sense and it does nothing. Implementation is by **rule-set
composition, not by a new mechanic**: the carrier is planted and the economy rules that would touch
it are withheld, so nothing new is added to the biology and the firewall is untouched.

## THE EXPECTATION TABLE (committed before building)

| arm | energy-band (level) | metabolic-dynamics (target) |
|---|---|---|
| **inert carrier** | **high** (definitional) | **≈ 0** |
| C token | high (z = 9.59) | low-to-moderate (dies at 120) |
| D stationary | high | **≫ 0** (5999/5999 steps) |
| A replicating | high (z = 9.81) | high |

**A candidate measure is biology-discriminating in the way WL is and the energy-band partition is
not IF AND ONLY IF it scores `inert ≈ 0` while scoring `D ≫ 0`.**

This is the whole gate. A measure that scores the inert carrier high is measuring carrier-ness
again and is rejected regardless of how well it separates anything else.

## Operational thresholds (fixed here)

- **`inert ≈ 0`**: the inert carrier's metabolic-dynamics score must be within the range attributable
  to a genuinely static channel — concretely, its per-carrier energy-change rate is exactly 0 and any
  transition-structure score built on it must be **degenerate or zero**, never "detecting".
- **`D ≫ 0`**: arm D must score at least an order of magnitude above the inert carrier on the same
  measure, and its permutation test (where one applies) must satisfy the **M2.3 null-spread guard**
  (`sd > 0`, `max − min > 0`, ≥ 10 distinct values) — a degenerate null is not a score.
- **Separation ratio** `D / inert` is reported for every candidate; a candidate that cannot make the
  denominator small is reported as failing, not rescaled.

## Candidates to evaluate (M3.2), cheapest first

1. **Energy-transition TPM** — build the TPM over *transitions in energy state* (binned deltas, or
   sign-of-change: `{decreasing, flat, increasing}`) rather than over energy levels. An inert carrier
   has constant energy and therefore an entirely degenerate transition structure (all mass on
   `flat→flat`); arm D does not.
2. **Per-carrier metabolic activity rate as a partition feature** — carriers resolved by *how much*
   their energy channel moves, not by *where it sits*.
3. **Closure-defect estimator on the energy channel** — the project's own construct, and the natural
   fit given M3.3.

For each candidate, both are reported:
- does it separate **inert** from **D** (the M3.1 gate);
- is it **stationary under bounded churn** (the M1 criterion applied to the new measure) — because an
  instrument that discriminates but is not churn-stable simply reproduces the dilemma M1 just
  dissolved.

## M3.3 — the closure-defect connection (recorded here, before building)

Arm D is a **maximal closure defect**: the structural (macro) description is not informationally
closed over the micro state, with essentially **100% of the causal work** — who lives, who dies, who
can move — occurring in a channel the macro description cannot see. It is the sharpest concrete
instance of the closure-defect construct the project has produced, and it arrived as a *byproduct of
a measurement failure* rather than by design.

This is a reason to prefer the closure-defect estimator among the candidates: arm D is a ready-made
extreme test case for it, with a known-correct answer (maximal defect) and a known-correct control
(the inert carrier, which should show **no** defect — nothing is happening in either channel).

Belongs in the LK-line notes as well as here.

## Forbidden

- Building or evaluating any candidate before this file is committed.
- Adjusting the expectation table, the thresholds, or the inert-carrier definition after seeing a
  candidate's numbers.
- Hashing ALife property keys into WL (rule 3).
- Reporting a candidate as successful on the strength of separating A from C while scoring the inert
  carrier high — that is the definitional signal again.
