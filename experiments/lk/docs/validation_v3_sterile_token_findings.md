# V3 / G-track — findings: the sterile-token control

Reported against `Downloads/validation_v3_sterile_token_plan.md` and the frozen
pre-registration `docs/validation_v3_sterile_token_preregistration.md` (committed
`f03c1b9`, before any token CE). Harnesses: `g_sterile_token_probe.rs` (G1, CE-free),
`g3_sterile_token.rs` (G3 comparison), `g3_sterile_token_robustness.rs` (confound
supplement). No `src` change, no pipeline change: the F8 scoring (WL depth 2,
`TpmAccumulator`, `canonical_order`, size-preserving permutation null) is reused
verbatim, and the carrier arm reproduces the F8 anchor in every harness (seed 12345:
CE 0.1633, k=4, M=200, 0/200).

---

## The single sentence

**Removing replication removes the boundary's detectability — but because replication
is the sole sustainer of turnover on a static substrate, this does not separate
replication-specific structure from turnover *duration*; the sterile token
self-extinguishes in ~120 steps, and a churn substrate is required to cut the two
apart.**

## Headline

| | real carrier (F8) | sterile token (frozen G2) |
|---|---|---|
| rule set | full `alife_base.toml` | full **minus `replicate`** |
| bootstrap | 1 marker | 36 markers (full grid; matches k=4, peak=36) |
| seeds detected (permutation p < 0.05) | **30/30** | **0/30** |
| real CE vs its null mean | 0.16 **≫** 0.04–0.07 | 0.218 **≈** 0.2205 (real ≤ null) |
| per-seed p | < 0.005 (median) | ~0.96 |
| null support at its resolution | 6.4 × 10⁷ | **8 855** (adequate; ≫ F0's fatal ~10) |
| last step with a live population | 6000 | **120** |
| classes touch this share of transitions | **48.7 %** | **2.0 %** |

By the pre-committed decision rule (token p ≥ 0.05 while carrier p < 0.05), this is the
**"replication is load-bearing"** branch. But the mechanism is turnover duration (D1),
so the honest name is a **sustained-turnover boundary**, not a bare "replication
boundary" and not the deflationary "matter-turnover boundary" — see the verdict.

---

## G3.1/G3.2 — the head-to-head

On the pre-registered seed set 1..30, the carrier arm (= F8) separates on every seed;
the sterile token separates on none. The token's real CE (~0.218) is not merely
*below* significance — it sits essentially *on top of* its null mean (Δ ≤ 0.001,
p ~ 0.96), i.e. the token/substrate partition carries no more causal-structural
information than an arbitrary same-sized partition. Null support at the token's
resolution is **8 855** (C(23,4)) — three orders of magnitude above the ~10-member
null that made the V1/F0 test unable to return a positive, so this is a real null, not
an under-resolution artifact (the forbidden-response guard is satisfied).

The token's real CE (0.218) is *higher* than the carrier's (0.16). This is F2.4 in
action: CE **magnitude** is registry/M-dominated (token M=23 ≪ carrier M=200), and is
not a measure of emergence. Detectability is the *separation from the null*, not the
coefficient's value — and on that axis the token is flat.

---

## D1 — the load-bearing confound (why this is *sustained*-turnover, not
## replication-per-se)

The token is **not** a matched-duration control. Head-to-head at seed 12345:

| arm | last-alive step | carrier-touching / total transitions | fraction |
|---|---|---|---|
| carrier | 6000 | 205 338 / 421 302 | **0.487** |
| token | 120 | 4 320 / 220 284 | **0.020** |

The carrier sustains turnover for the whole run and its classes participate in nearly
half of all transitions; the sterile token blooms once and self-extinguishes at step
120, participating in 2 %. Extinction is **structural, not tuned**: a token is pinned to
its single birth-site, influx (0.34) < maintenance (0.5), so once it drains that site
it starves, and with no replication it cannot colonise a fresh one.

Therefore the token's non-detection is **confounded with turnover duration**, and on a
*static* substrate replication is the only thing that sustains a population — so
"replication is load-bearing" and "sustained turnover is load-bearing" **cannot be
separated here.** This is the same entanglement the plan reserved for the churn-substrate
rung (§G4), reached from a second direction.

What D1 *does* let us rule out: the deflationary **"matter-turnover boundary."** That
reading required birth-and-death-with-site-structure to be *sufficient* for
detectability. The token **has** all three — it is born, it dies, and it resolves into
4 site-induced WL classes — and it still does not separate. So bare matter-turnover is
**insufficient**; the turnover must *persist*. The boundary is a sustained-turnover
boundary.

---

## D2 — robustness (the null result is not an N=36 determinism artifact)

At the pre-registered full-grid bloom (N=36) the token run is **seed-independent** (no
replication RNG on a fully-marked grid), so the 30 token rows in G3 are one
deterministic measurement repeated — the token's cross-seed sign test is therefore
**not** 30 independent trials and is disregarded. What carries the result is the
*within-run* permutation null (200 draws, support 8 855), which is valid, plus the
following robustness sweep where marker placement (N < 36) **is** seed-dependent:

| N | M range | k range | real CE ≈ null mean | detected? | support |
|---|---|---|---|---|---|
| 4  | 21–27 | 2–3 | 0.207–0.225 ≈ null (Δ ≤ 0.003) | **no** ×3 | 2.1×10²–2.6×10³ |
| 9  | 35–38 | 2–3 | 0.187–0.192 ≈ null | **no** ×3 | 6.3×10²–8.4×10³ |
| 16 | 39–45 | 3–4 | 0.178–0.186 ≈ null | **no** ×3 | 9.1×10³–1.5×10⁵ |
| 25 | 42–45 | 4 | 0.178–0.182 ≈ null | **no** ×3 | 1.1×10⁵–1.5×10⁵ |
| 36 | 23 | 4 | 0.218 ≈ null | **no** ×3 | 8.9×10³ |

The token fails to separate at every bloom size, **including well-resolved cases**
(N=16/25, support ~10⁵, comparable to real carrier seeds), and with genuine seed
variation at N < 36. Real CE ≈ null mean to ±0.001–0.003 throughout. The non-detection
is a property of the sterile dynamics, not of the specific bloom.

---

## G3.3 — the direct-contrast null: **not constructible, fell back to G3.2 (as the
## plan permits)**

A single run containing both a replicating carrier class and a sterile token class, with
labels permuted between them, would need a second node label and a duplicated rule set
(a `bootstrap₂`, `harvest₂`, … for the token label) — because every rule in
`alife_base.toml` matches on label 100. That is (a) new physics beyond "remove one rule",
violating rule 3, and (b) worse, the two populations would then compete for the same
substrate energy, so each partition's causal structure would be perturbed by the other's
presence — confounding the very contrast it aims to sharpen. The model does not admit
both classes cleanly, so per G3.3 we fall back to the paired per-seed comparison. That
fallback is already strong: both arms run the identical scoring in one harness (carrier
validated against F8), and each seed pairs the arms on the same substrate geometry.

---

## G4 — what this track settles and does not

**Settles.** On the static substrate, within the vacuum model, **replication is
load-bearing for the boundary's detectability**: with it, the carrier/substrate boundary
separates on 30/30 seeds; without it, the sterile token separates on 0/30, robustly
across bloom sizes and resolutions, with an adequate null. And it **refutes the
deflationary reading**: birth-and-death-with-site-structure is *not* sufficient — the
token has it and stays flat — so the boundary is a *sustained*-turnover boundary.

**Does not settle — replication-per-se vs turnover-duration (the D1 confound).** The
sterile token self-extinguishes at step 120 (2 % of transitions), so "the boundary needs
replication" and "the boundary needs a population that persists" are inseparable on a
static substrate, where replication is the sole sustainer. This experiment cannot award
the boundary to replication *specifically* over duration.

**Does not settle — the biology-vs-construction question in full (the plan's own G4).**
Both real carriers and sterile tokens are still the only ∅-touching classes; this track
removes replication, not the ∅-touching construction. H_construction is narrowed but not
refuted.

**Does not settle — magnitude** (F2.4 stands: CE value is registry/M-dominated; the
token's higher CE is exactly this artifact) — **or the static-substrate temporal
degeneracy** (§8 / F5-4).

**The one measurement that cuts all three at once: a churn substrate.** A substrate whose
nodes are themselves born and die would let a *non-replicating* population persist via
substrate turnover — separating duration from replication (resolving D1), giving a
∅-touching class that is *not* the biological one (resolving H_construction), and
unfreezing the temporal experiment. It needs new physics and is the next rung; per rule
2 it is **not** invoked to rescue anything here.

---

## The single sentence to state plainly (as the plan requires)

**Removing replication, and nothing else, removes the boundary's detectability — but not
cleanly enough to name it "replication": on a static substrate replication is the only
thing that keeps a population alive, so what the boundary demonstrably needs is *sustained*
turnover, and separating "replication" from "persistence" is the job of the churn-substrate
rung.**
