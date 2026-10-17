# selfmod v3 findings — VF2 port + enriched-vocabulary selection test

**Setup.** `hg-selfmod` crate on the real engine: rules reified in a separate RuleStore,
decoded each step to `CompiledRule`s, matched/applied by hg-engine's own
`find_bindings`/`resolve_conflicts`/`apply_binding` (engine untouched). Full pattern class
within caps (LHS ≤ 4 vars / 4 edges, arity ≤ 3, ≤ 3 creates; non-injective matching;
disconnected patterns allowed), 15 structural meta-operators, conserved-token economy
ported from v2 with **effect-gated reward** (a firing pays only if it changed the store;
delete-and-recreate-identical-edge counts as sterile). Determinism: single ChaCha8 stream,
path-independent draw counts in mutation, canonical rule enumeration by
(class_hash, creation seq); a Phase A seed-7 rerun was **bit-identical** [Measured].
Conservation `free + Σ tokens == total` asserted every step, both phases — never tripped.

## Gates

- **G0 [Measured] PASS** — 10 fixtures spanning the enriched class round-trip
  (`decode(encode(r)) == r`, equal `class_hash`); 5 deliberately malformed subgraphs
  decode to `None` without panic.
- **G1 [Measured] PASS** — reification neutrality on the real engine: decode-each-step vs
  compile-once produce bit-identical world `canonical_hash` trajectories, 100 steps ×
  seeds {7, 42, 999}, fixture set including arity-3 hyperedge creation/consumption.
  F1 carries to the engine port.
- **G2 [Measured] PASS** — the enriched class *contains* an activity-sustaining ruleset:
  a hand-built 3-rule churn cycle (A—B ⇒ A—C,C—B; C—B ⇒ delete C, B→D; A,D ⇒ A—B, D→B)
  sustains productive rewrites in every 100-step window for 10k steps, canonical hash
  changing in 20/20 late windows, world stable at (176 nodes, 200 edges).

## Excursions (logged per ground rule 6)

1. **Edge explosion (branch (d) event).** `add_rhs_edge` mutants immediately discovered
   unbounded parallel-edge growth (the engine store does not dedupe identical edges;
   the node clamp does not bind edge-only growth): 18k+ edges by step 2000. Fix: an
   `edge_ceiling` (1000) gating rules whose static net edge delta is positive — the edge
   analogue of v2's F0 node clamp. v2 never met this because parameter-only mutation
   cannot invent edge-spawning rules.
2. **Match-cost budget.** Per-rule enumeration estimate (edge candidates × endpoint
   permutations × label selectivity; isolated vars × label-filtered node count) with
   `match_enum_budget = 5e4`; over-budget rules are skipped that step (deterministic,
   counted). Required because `find_bindings` materializes all bindings before our
   canonical-order truncation to `match_cap_per_rule = 64`.

## E0 — Phase A on the real engine [Measured]

7/8 seeds to budget (20k steps, genesis on, birth_cost 50): survivor final-third mean
rule_count **35.7, σ = 0.52** against N\* = total/birth_cost = 40 — the v2 bounded,
seed-independent attractor reproduces on the real engine (v2: 44.9 ± 0.9 vs N\* = 50).
Seed 42 collapsed inside the first 100 steps: a startup-transient gambler's-ruin from
initial population 4, made absorbing by the runner's terminate-at-0-rules check (kept for
v2 comparability) even though genesis makes 0 non-absorbing. [Inferred] termination-rule
artifact, not a regulation failure; regulation is validated. Economy port good.

## E1 — Phase B, enriched vocabulary (the experiment) [Measured]

**Universal death spiral, indistinguishable from v2:** 8/8 collapse by step 80–90, peak
population 4, `frac_fired → 0` by step ~15 (world at node ceiling, seed rules' matchable
structure exhausted), **0 births, M_rules = 4** — mutate-at-birth never executed, so the
enriched vocabulary was never sampled.

**Pre-committed branch (b).** G2 passed, so sustaining rulesets exist but are not
reachable — and the failure is sharper than "connectivity": at rent 1 / reward 2 /
threshold 100 / e0 60, a rule needs ~40 productive steps to fund one fission, and the
seed ruleset freezes its own world in ~15. **Selection's variation channel is dead on
arrival: the seed dynamics' time-to-freeze is shorter than the metabolism's
time-to-first-reproduction.** [Measured mechanism, per-step traces]

### E1b — branch-(b) follow-up: G2 triple seeded alongside the v2 seed rules [Measured]

(Logged choice: the plan's "seed one G2 rule" was widened to the full triple — the
sustaining unit G2 itself defines is the 3-rule cycle; one member alone cannot sustain.)

**8/8 collapse.** The churn runs beautifully (~34 productive applies/step, steps 1–25)
until the v2 **G rule — an unbounded node pump — drags the world to the 500-node
ceiling**, where the carrying-capacity clamp gates the cycle's node-creating member (R1).
The world ratchets to 512 nodes (a MaxParallel overshoot above the ceiling with nothing
left to prune), permanently freezing R1 off; freeze by ~35, all starved by ~140.
[Inferred] **The G2 attractor's basin is not stable against coresident growth-pressure
rules under an all-or-nothing carrying-capacity clamp** — death by clamp interaction, not
by vocabulary or selection.

### E1c — isolation: G2 triple only [Measured]

| seed | outcome | births | M_rules end |
|---|---|---|---|
| 1 | collapse @200 | 7 | 9 |
| 2 | **budget (20k)** | 5 | 7 |
| 3 | collapse @100 | 0 | 3 |
| 7 | collapse @400 | 9 | 9 |
| 42 | **budget (20k)** | 0 | 3 |
| 99 | **budget (20k)** | 0 | 3 |
| 123 | collapse @200 | 4 | 6 |
| 777 | collapse @200 | 7 | 6 |

Three regimes:
- **Degenerate persistence (42, 99):** the triple fires forever (late `frac_fired`
  0.33–0.67, ~1.5 productive applies/step) but per-rule duty cycle equilibrates below the
  reproduction threshold — 0 births in 20k steps. Seed 42's canonical hash is **constant**
  across the last 20 windows (a perfect content limit cycle); seed 99's keeps changing.
  `M_data` and `M_rules` slopes in the final third: **0.0000**.
- **Variation-induced collapse (1, 7, 123, 777):** reproduction DOES fire (4–9 births,
  new rule classes appear), and then the population collapses within 100–400 steps —
  mutant offspring perturb the world's label/structure composition until the parent
  cycle's material loop breaks (seed-1 trace: fission at ~40, world composition drifts,
  activity dies by ~130, mass starvation at ~140). Under a conserved zero-sum pool,
  **variation is a net destabilizer of the autocatalytic set that funds it**.
- **Evolved stabilization (2):** 5 births, 4 deaths; a 4-rule population containing
  evolved classes survives to budget with a live, changing world (20/20 distinct late
  hashes, ~3 productive applies/step) — the one case where selection digested variation.
  Its late `M_data`/`M_rules` slopes are still **flat**.

`sterile_firings = 0` in every run — the effect-gated reward boundary was never probed;
populations never lived long enough or grew varied enough to find weed rules.

## Verdict — pre-committed branch (c), with (b) as the entry barrier

Reading E1/E1b/E1c against the pre-committed interpretations:

- E1 lands on **(b)**: sustaining rulesets exist (G2) but are unreachable — refined to a
  *timescale race*: time-to-freeze (~15 steps) ≪ time-to-first-fission (~40+ steps).
- E1c (basin test) lands on **(c) flip-flop attractor / degenerate persistence**:
  survival happens (3/8), by trivial churn; `M_data`/`M_rules` flat in the late window in
  every survivor. Per the plan's own reading: **positive for "metabolic selection can
  sustain a live world given a sustaining seed"; negative for open-endedness.**
- The v2→v3 diagnosis ("the death spiral was a vocabulary limitation") is therefore
  **partially refuted**: enriching the vocabulary did not change the Phase B outcome
  (E1 ≡ v2 bit-for-bit in shape), because the binding constraint was never the pattern
  class — it is (i) the seed dynamics' freeze rate vs the metabolism's reproduction
  latency, and (ii) once past that barrier, the fragility of the funding autocatalytic
  set to its own variation.

## What v3 redirects toward

1. **The reproduction latency, not the pattern class, is the critical knob.** Candidates
   for v3.1, in increasing invasiveness: lower `repro_threshold`/raise `fire_reward`
   (close the timescale race directly); contingency B2 (maintenance reward — pay rules
   whose motif persists, decoupling survival from instantaneous rewriting); heritable
   token endowments (child gets a fraction of parent wealth, buffering lineages through
   composition shifts).
2. **Clamp design matters as much as economy design.** The all-or-nothing ceiling turned
   a coresident grower into a churn-killer (E1b). A soft clamp (probabilistic gating ∝
   headroom) or deletion-pressure regulation would remove the ratchet-above-ceiling
   absorbing state.
3. **Variation-rate control.** Mutate-at-birth with these operators is a large step in a
   zero-sum world; E1c's collapse mode motivates either milder operators at birth or a
   mutation probability < 1, so lineages can establish before exploring.

E2 (novelty characterization) was warranted only under (a) or partial (c): on the three
survivors it is trivial — M-slopes 0, LHS sizes static (mean 2.25 on seed 2), sterile
firings 0. Fission genealogy (parent RuleId at birth) is not yet logged by the runner;
with ≤ 9 births per run there was nothing for it to discriminate. Noted as a v3.1 metric.

## Promotion assessment

The decode phase itself earned its acceptance test: G1 holds bit-identically on the real
engine across the full pattern class, decode → compile-native costs nothing observable at
these scales, and the junk-DNA discipline (malformed ⇒ inert, counted) worked as designed.
On the science, however, no v3 outcome requires the decode phase to live inside
`hg-engine::step()` — the library-driver pattern (find_bindings / resolve_conflicts /
apply_binding) gave everything the experiment needed with zero engine changes.
**Recommendation: do not feature-gate decode into `hg-engine::step()` yet.** Promotion
becomes worth its maintenance cost only at the full-reflection milestone (meta-rules
matched by the same engine over a co-resident store), which additionally requires:
(i) observer masks or a namespace field so hg-observe metrics can exclude rule-subgraphs,
(ii) a decode cache invalidated by rewrite events touching rule roots, and (iii) a
resolution for the clamp/economy interactions above — otherwise the engine would be
hosting a dynamics we already know starves.
