# V4 / H-track — H0 findings: the churn rung is a dead end for CE; hand off to H4

Reported against `~/Downloads/validation_v4_churn_substrate_plan.md` and the committed
criterion `docs/validation_v4_h0_stationarity_criterion.md` (H0.1, commit `24456eb`, written
before any measurement). Harness: `crates/hg-sim-core/tests/h0_stationarity.rs`. Same F8 WL/TPM
machinery, no pipeline change. **No churn substrate was built** — H0 answered the gate on the
existing code, exactly as the track is designed to.

---

## The single sentence (H0's mandated answer)

**CE over a fixed carrier/substrate partition is *not* well-defined once the substrate moves — so
the churn rung is a dead end for the CE instrument, and the track hands off to H4 (a
non-partition-relative measure).**

## Headline

| run | steps | n_start→end | M | TWTF (primary) | persist | T/M | verdict |
|---|---|---|---|---|---|---|---|
| **H0.2 static W=120** | 120 | 36→72 | 121 | **0.005** | 0.899 | 64.8 | floor |
| **H0.2 static W=6000** | 6000 | 36→70 | 200 | **0.000** | 0.804 | 2106 | floor |
| H0.3 growth_treadmill | 120 | 36→190 | 730 | **0.515** | 0.619 | 20.4 | **STOP** |
| H0.3 triangle_subdivide | 120 | 36→1476 | 1562 | **0.984** | **0.000** | 58.5 | **STOP** |

TWTF = transition-weighted transient fraction (share of TPM transitions whose source WL class is
live in < R=3 steps). Committed decision rule: transient-majority (TWTF > 0.5) ⟹ STOP → H4. Both
motion sources cross it; the static floor is ~0.005.

---

## H0.2 — the static floor is essentially perfect, and the metric is not fooled by growth

On the trusted F8 static run the partition is about as stationary as it can be: TWTF 0.005 (short
window) → 0.000 (full window), `M` plateaus at 200 with `dM/dt` 0.13 → 0.00 (state space closes),
`T/M` = 2106, step-to-step live-class persistence ~0.80–0.90.

Crucially, **the static run itself grows** — 36 → 72 nodes over W=120 as carriers are born — **and
TWTF stays 0.005.** So node growth *per se* does not inflate TWTF: newborn carriers persist and
recur, so they are not transient classes. TWTF measures class *ephemerality*, which is the thing
that actually contaminates the TPM, not node count. This is what makes it a fair metric to carry
into the (unbounded) motion proxies.

## H0.3 — any substrate motion ephemeralizes the partition, past tolerance immediately

- **triangle_subdivide: TWTF 0.984, persistence 0.000.** Subdividing edges changes every node's
  neighbourhood every step, so every node draws a *new* WL class every step — no class survives
  one step to the next. A fixed partition over such a state space is meaningless; the TPM is
  essentially all singleton transient rows. This is not a window or growth artifact — it is
  per-step reclassification.
- **growth_treadmill: TWTF 0.515, persistence 0.619.** Even this mild motion (+~1.3 nodes/step)
  puts a *majority* of transitions on transient classes — ~100× the static floor.

**Why this generalizes to a *bounded* churn substrate (the honest reading of unbounded proxies).**
The proxies grow, and one could worry TWTF is inflated only by end-of-window growth. Three things
rule that out as the explanation:
1. The static run grows too (36→72) at TWTF 0.005 — growth alone does not inflate TWTF.
2. triangle_subdivide's failure is persistence = 0.000 (per-step reclassification), which is
   independent of node count.
3. **The mechanism is structural and applies to *any* churn substrate.** A bounded-churn substrate
   is *defined* by fast node-identity turnover at flat `n(t)` (H1.1): old label-0 nodes retired,
   fresh ones minted. A node that is minted and retired within a few steps **is** a transient WL
   class by construction — and the *faster* the identity turnover, the more ephemeral the classes,
   the higher TWTF. Worse, H2.1 *needs* fast turnover: a non-replicating population is "sustained
   by substrate turnover" only if fresh nodes appear fast enough to recolonise as carriers die. So
   the very mechanism the churn rung requires to sustain the population (fast substrate turnover)
   is the mechanism that ephemeralizes the partition and breaks CE. Low turnover keeps the
   partition stationary but does not sustain the population (back to the static problem); high
   turnover sustains it but destroys stationarity. This is the S6.1 squeezed-window pathology
   again, now on the *measurement* rather than the biology.

## H0.4 — windowing cannot rescue it (lose-lose)

A shorter scoring window is the plan's rescue for churn that only bites over long windows. It does
not apply here, and the measurement confirms it:

| window | TWTF | T/M |
|---|---|---|
| W=20 | 0.331 | **5.7** |
| W=40 | 0.336 | 10.9 |
| W=60 | 0.384 | 14.4 |
| W=120 | 0.515 | 20.4 |

Shortening the window keeps TWTF high (~0.33 at W=20 — still ~66× the static floor) because the
churn is *per-step*, and a shorter window makes the R=3 persistence bar *harder* to clear, not
easier. Meanwhile `T/M` collapses toward the under-sampled regime (5.7 at W=20, < the T/M < 10
STOP line). **There is no window that is simultaneously partition-stationary and adequately
resolved.** H0.4 is foreclosed.

---

## H0 decision: **FAILS → H4.** The churn substrate is not built.

Per the committed rule (transient-majority ⟹ STOP) and the plan's forbidden-response list
("Building the churn substrate before H0 clears — the entire track structure exists to prevent
exactly this"), H1–H3 are not attempted. Substrate motion breaks the instrument that would measure
the split it enables. This is the dead end the two-phase gate existed to detect cheaply, and it was
detected on paper + ~4 minutes of measurement over existing code, before any substrate was built.

**What H0 settles.** CE over a fixed WL carrier/substrate partition is well-defined only while the
substrate is static (the regime of every result in the report). It is *not* well-defined under
substrate motion, at any window that preserves adequate sampling. The churn substrate therefore
cannot be the instrument for splitting replication from turnover-duration.

**What this does *not* change.** The G-track's sustained-turnover verdict stands as the current
frontier: replication is load-bearing for detectability, confounded with duration, and the
confound is now shown to be **not cheaply resolvable by the obvious next instrument.** The
biology-vs-construction question, the magnitude question (F2.4), and the temporal degeneracy
remain open — the churn substrate would have addressed all three *if* CE survived motion, and it
does not.

---

## H4 — the handoff (design task, replaces H1–H3)

The question "does sustained turnover produce macro causal structure" now needs a measure that
does **not** depend on a fixed carrier/substrate partition holding still. Scoped candidates and the
one property H0 proved is required — *well-definedness under a moving, ephemeral class set*:

1. **Dynamical-independence / ΦID-style causal decoupling.** Measures whether a coarse variable
   evolves with its own closed dynamics, without committing to a fixed micro partition across the
   window. Well-definedness question to resolve first: these still assume a stable macro variable;
   under substrate churn the macro variable (carrier population) *is* stable even though the micro
   classes are not — which is promising, but the micro-side estimator must tolerate an
   ever-changing class set. This is the most aligned candidate and should be scoped first.
2. **Closure-defect estimator over a moving partition.** Directly quantify how much a coarse
   dynamics fails to be Markov-closed, computed incrementally so it never requires a single fixed
   `M` across the window. Must be shown to converge under the per-step reclassification H0.3
   measured (triangle-like persistence 0.000 is the stress test).
3. **Label-anchored (not WL-anchored) partition.** The carrier/substrate split does *not* actually
   need WL classes — it needs the carrier-vs-substrate *label*, which is stable under churn. A
   measure that coarse-grains by label directly (two macro states) and estimates information flow
   between them sidesteps the WL-class ephemerality entirely. The cost is losing the site-induced
   sub-structure (the 4 carrier classes) that F2's resolution depended on — so its power must be
   checked against F8 before trusting it under motion. This is the cheapest to prototype and is a
   natural H4.0.

H4 is a design task, not an experiment; it is out of scope for this track's build budget and is
recorded here as the forward path.

---

## The single sentence to state plainly (end of H0)

**Whether CE over a fixed carrier/substrate partition remains well-defined once the substrate
moves: it does not — substrate motion turns the WL class set ephemeral (transient-majority TWTF at
even the mildest motion, per-step reclassification at strong motion), no window is both stationary
and sampled, so the churn rung is a dead end for CE and the program continues via a
non-partition-relative measure (H4), not by building the substrate.**
