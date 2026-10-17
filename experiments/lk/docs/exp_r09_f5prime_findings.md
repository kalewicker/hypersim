# EXP-R09 findings — the ERE outer loop under F5′ (ERE plan §10, §12)

> NOTE — name collision owned: `docs/ere_r09_findings.md` is the GE-track
> fee-gated loop. THIS file is the ERE-plan EXP-R09 (evolutionary outer
> loop, G_θ under F5′, champion/untrained/random). Different experiment,
> different plan section, similar name — hence the `f5prime` suffix.

Driver (to be built): `crates/hg-meta/src/outer.rs`.
Run with `cargo test --release -p hg-meta --lib outer`.

## PRE-REGISTRATION (2026-09-04, before any EXP-R09 number exists)

**Question (§12).** Does the outer loop beat initialization under F5′?
**Pre-committed null:** NO — expected given F1–F3 history; the run
discriminates F5′ from diversity objectives (it does not redeem them).

**What is optimized.** G_θ, a harness-scale generative grammar emitting
seed rule regions. θ ∈ R^13, five softmax groups (all E1-expressible,
substrate alphabet {0,1,2,3} read off `seed_world_n`):

| group | choices | dims |
|---|---|---|
| n_seeds | {1,2,3} | 3 |
| n_lhs_edges | {1,2} | 2 |
| n_rhs_edges | {1,2} | 2 |
| n_vars | {2,3} | 2 |
| label per var | {0,1,2,3} | 4 |

Fixed OFF the θ-parametrization (descoped, stated): arity (=2, the only
arity the substrate speaks — a 3-slot pattern edge can never match an
arity-2 substrate edge), guards, var_guards, effects, priority.
θ=0 is uniform — the neutral start (StaticBias-at-zero discipline).
Sampling is i.i.d. per seed from per-group softmax; candidates are
validated (`validate`) with bounded resampling (16 tries/seed); residual
invalid slots fall back to a fixed known-valid template AND THE FALLBACK
FRACTION IS REPORTED (a high fallback rate aborts via P1′ — the grammar
is unstatable, not the loop untuned).

**Inner loop.** A2-protected (label-mirror guards on), full 10-op kernel,
NO fee economy (fees are GE's instrument; ERE plan v1 has no economy —
kept pure). 30 epochs × 10 MaxParallel steps; burn-in = first 3 epochs
unscored. Region-node cap 384 (M1-Pro constraint); over-cap epochs skip
the install and log bound (a result under a binding ceiling is a result
about the ceiling, FM-R3).

**F5′ — constructor reuse over rule classes.** Per scored epoch: drain
`SharedFireCounter` → rule_id → root (consultation order) → read region
→ RCF hash; accumulate fires per RCF class. NonTrivial gate (FM-R4):
a class counts only if its candidate is not an exact identity
(lhs ≡ rhs as shape multisets — the GE seed-rule-2 copy trap that fired
4000× as a no-op; SB-R5 corollary: same-shape sides sharing vars cannot
differ in labels, so the shape comparison is exact, and slot permutation
sorts out per SB-R4). F5′ = Σ_c fires_c² / max(1, Σ_c fires_c)
(concentration, the F5 §4B.1 form with fidelity ≡ 1). Zero scored fires
with live regions = genuine 0.0; extinction before scored epochs =
0.0 WITH the extinction counted separately (stated, not silently
averaged). EXP-R08's empty intersection does NOT bind here: F5′ needs
no stationarity window (stated exemption — firing reuse is measurable
under rule change; EI is not).

**Optimizer.** Antithetic OpenAI-ES with rank shaping (the
`hg-train::optim::OpenAiEs` form, inlined — hg-train's dev-dependency
weight is unjustified for 40 lines; behavior pinned by a synthetic
quadratic unit test). σ=0.5, α=0.1. Budget pre-committed: G=3
generations × 4 pairs (8 evals/gen, shared eval seeds per pair so the
seed-lottery component cancels in the antithetic difference) = 24
training evals. Controls × 4 fresh seeds each: champion (best θ),
untrapped θ=0, random-resample (fresh θ ∼ N(0,1)/√d per eval).
Primary statistic: champion − untrained mean ± SE.

**Prechecks P1′–P4′ (run first; any abort ENDS the campaign and the
abort IS the finding, §10.4):**

| # | checks | bar |
|---|---|---|
| P1′ expressiveness | θ=0 vs θ_far (±2.5 spread) seed populations differ; valid-seed rate | TV ≥ 0.05 (TRAVERSAL_TV_MIN mirror), fallback < 50% |
| P2′ traversal | σ-perturbations move the induced allocation | mean TV ≥ 0.05 over 8 draws |
| P3′ discrimination | F5′ varies over {θ=0, θ_far, 2 random θ} × 2 seeds | range > 0, not all zero |
| P4′ non-saturation | caps non-binding on probe majority; F5′ below ceiling | bound epochs < 50% of probe epochs |

**Predictions.**

1. P3′ passes weakly: F5′ varies seed-lottery-first, θ-second
   (SB-GE7(b) seed-contingency). A P3′ ABORT is the live alternative —
   F5′ seed-locked (children dormant, GE finding #2) makes the outer
   loop unstateable at this scale, recorded as such.
2. Campaign null CONFIRMED: champion − untrained ≈ 0 within SE
   (F1–F3 history + EXP-R07 linear null).
3. Goodhart watch: if champion wins, expect top-class share → 1.0
   (single-churn concentration) — flagged as gaming, not celebrated
   (risk-register row + GE §1 B1/B3 warning: selection concentrating
   mass is the blob dynamic in evolutionary clothes).
4. Fallback rate < 20% (grammar mostly-valid by construction).

**Decision outputs.** (i) champion−untrained ± SE — the EXP-R09 answer.
(ii) P1′–P4′ verdicts. (iii) Goodhart panel: top-class share, bound
fraction, fallback fraction, extinction counts.

## EXECUTION LOG (post-registration, pre-verdict)

**SB-R6 (opened by the P3′ battery, 2026-09-08).** `K_LABEL_MIRROR`
(protect.rs) was property key 0 — the same key as schema
`K_LABEL_VALUE` (the LABEL_LIT value key). The old comment claimed
reuse was "unambiguous" because the lit key "lives on LABEL_LIT nodes
only"; false — `sync_label_mirror` writes EVERY node including
LABEL_LITs, so the first sync overwrote each HASLABEL value with the
lit node's own label (`L_LABEL_LIT`) and V4-quarantined every labeled
region at the next consult. Battery symptom: F5′ all-zero over 8
probes, every run extinct at epoch 1 — the §10.4 abort firing as
designed, on an instrument bug rather than the objective. Fixed: mirror
key 0 → 3 (free: substrate energy is 1, priority is 2, guard/effect
blocks are 10+; reader ignores non-`K_LABEL_VALUE` props on lits).
GE-track reframe owned: the GE loop survived only via its label-free
identity copy rule (no lits to clobber) — its 4002 free firings and
~0.67 quarantine shares are partly this clobber, not pure economics;
GE numbers stand, their interpretation is discounted pending a GE
re-run under the fixed key.

**Budget alignment (review catch, same session).** The campaign as
built ran 4 evals/pair (2 seeds × ±) = 48 training evals vs the
pre-committed 24 (8/gen). Fixed to ONE shared eval seed per pair
before any post-fix number existed — the 44-eval battery total in the
driver docs is now exact (8 probes + 24 training + 12 controls).

**Review verdict on the driver (2026-09-08).** Correct: θ dims/groups,
softmax/TV math, scratch-install→read-back seed validation with
reported fallback, SB-R4/R5 `nontrivial()` gate, exact
`seed_world_n` substrate duplicate, A2 protection wiring, F5′ formula
with burn-in/cap/extinction accounting, dropped-attribution tripwire,
inlined ES form pinned by the quadratic test, P1′/P4′ bars, controls
design, `#[cfg(test)]` gating, order-preserving `compile_rules`
(rule_id = index, attribution sound). Noted-not-changed: engine RNG
seed fixed at 42 across evals (substrate/mutation RNGs vary per eval;
deliberately left — varying it now adds an unplanned degree of
freedom). Temporary funnel diagnostic removed before the re-run.

## VERDICT (2026-09-08, post-fix battery, release, deterministic ×2 identical)

**P1′ PASS.** TV(zero, far) = 0.5117 ≥ 0.05; fallback 1/24 (4.2%).
**P2′ PASS.** Mean σ-traversal TV = 0.1198 ≥ 0.05 over 8 draws.
**P3′ PASS.** F5′ range [0.0000, 18680.6836] over 4 θ × 2 seeds —
blindness lifted by the SB-R6 fix alone (pre-fix: all-zero, extinct
epoch 1). Variation is seed-lottery-first as predicted: far-θ gives
651.2 (EB01) vs 18680.7 (EB02); rand1 gives 1075.4 vs 96.6; zero-θ
gives 840.0 vs 1000.0 (top = 1.0 both — single-class churn at θ=0).
**P4′ PASS.** Bound 0.000 of probe epochs (cap never binds), extinct
0/8, dropped 0 (attribution sound).

**Campaign (G=3 × 4 pairs = 24 training evals, per pre-reg).**
per-gen plus/minus means: (1.25/4.00) → (186.45/388.15) →
(516.76/216.29), best pair-fitness 850.0. Controls × 4 fresh seeds:
champion 139.74 ± 131.52, untrained 467.22 ± 283.61, random
28524.53 ± 28511.20. **Champion − untrained = −327.47 ± 312.63 —
the pre-committed null is CONFIRMED** (prediction 2; interval covers
0, champion numerically worse).

**Predictions scorecard.** (1) P3′ weak pass, seed-first ✓. (2) null
✓. (3) Goodhart watch moot — champion did not win; but the wild
signature is on display: random-ED02 concentrates 122,185 fires at
top-share 0.966 (F5′ = 114,058), i.e. exactly the single-churn gaming
row the risk register warns against, arrived at by luck, not
selection. (4) fallback 4.2% < 20% ✓.

**EXP-R09 answer.** The outer loop does not beat initialization under
F5′ at this budget/scale (G_θ, 24 training evals, 24-node substrate,
30-epoch inner). F5′ discriminates (seed-first) without saturating —
the instrument is stateable; the optimizer finds nothing to grip.
Matches F1–F3 history and the EXP-R07 linear null. No follow-up
campaign at this scale is warranted; F5′ stays available as a
diagnostic (it caught SB-R6), not as an objective.
