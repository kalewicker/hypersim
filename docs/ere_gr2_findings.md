# EXP-R02 / G-R2 findings — expressivity envelope of the RuleSpec encoding

**Run.** `hg-selfmod gate_gr2` (release), 45 shipped rules × 1000-step
behavioral comparisons on auto-synthesized fixtures. Protocol:
`docs/ere_gr2_protocol.md`.

## The envelope: 2/45 (0.044)

| Rule | Verdict |
|---|---|
| `crystal_battery.toml :: crystal_grow` | Identical, 1000 steps, engaged |
| `wolfram_analog.toml :: wolfram_like` | Identical, 1000 steps, engaged |

Non-creation companion (SB-R2): 2/41 (0.049). **G-R2 verdict: PASS** — both
expressible rules behaviorally identical (hash + firing counts every step +
final RNG stream); low coverage is the reported limit, not a failure. Zero
divergences, zero unmeasured (every auto-fixture engaged by construction).

## Blocking reasons (first-wins ranked)

VarsDistinct (E1d) 21, property-effects (E1a) 14, schedule (SB-R2) 4,
degree-compare 2, quantity-compare 1, weight 1.

## Solely-blocked-by (extension prioritization)

E1d alone 20, E1a alone 3, schedule alone 2, weight alone 1; multi-blocked
17. **Lifting VarsDistinct alone would take the envelope from 2 to ~22**
(plus whatever multi-blocked rules list it second — the 17 need full-set
analysis per candidate extension). E1d is the binding constraint on the
encoding, and it blocks every classic fixture the program's earlier results
were built on (`triangle_subdivide`, `glider`, `binary_edge`, `cyclic3`,
`tetrahedral`, `growth_treadmill`, `swap`, … — pinned by test).

## Reading (pre-committed: low coverage is a stated limit)

1. Every downstream novelty claim carries the number **0.044** (0.049
   ex-creation): "novel" means novel within the 4% of rule-space the
   encoding can say. Anything stronger is uninterpretable.
2. Caps never bind (no TooManyVars/Edges/Arity/Creates anywhere): the
   envelope is limited by encoding *features*, not size caps. Raising caps
   buys nothing.
3. The E1a+E1d pair accounts for 35/43 exclusions. The LK track's
   energy-metered rules (the program's most interesting dynamics) are
   out-of-envelope by construction — energy-metered *meta*-rules (E1-G's
   purpose) would hit the same wall until E2 effects land.
4. No `Vacuous`, `BadVar`, `FixtureUnsupported`, or guard-op exclusions
   fired: the shipped library is clean of degenerate cases and the
   auto-fixture synthesizer covered every guard shape encountered.

## Routing

The next encoding work, in order: (1) VarsDistinct control (E1d — 20 rules
solely), (2) property effects E2 (3 solely + 14 jointly), (3) creation
schedules E1.1/SB-R2 (2 solely). Until then, R5/R6 operate inside the 4%
envelope with the number stated.

## Re-run after E1d (VarsDistinct lifted)

**Run.** Same gate post-extension (release): converter expands each
`VarsDistinct` guard to unordered pairs carried as `DISTINCT` nodes;
harness upgrades — encoding-reach caps (8/16/8/8, E1's `MAX_ARITY` scale),
match budget uncapped in-harness (production throttle must not censor
semantics — the pre-fix run had triangle spuriously Unmeasured because its
unlabeled 3-edge pattern costs ~8×10⁶ against a 5×10⁴ budget), FM-R3
bloat-halt (2000 nodes / 4000 edges, ≥10 steps), 200-world differential sweep.

| | Pre-E1d | Post-E1d |
|---|---|---|
| Expressible | 2/45 (0.044) | **23/45 (0.511)**; non-creation 23/41 (0.561) |
| Identical (1000-step traj + 200/200 sweep) | 2 | 23 (all full 1000 steps, no bloat-halt fired) |
| Divergent / Unmeasured | 0 | 0 |
| Solely-blocked-by | E1d 20, E1a 3, … | E1a 3, schedule 2, weight 1, quantity 1, degree 1; multi 14 |

**G-R2 verdict: PASS** (unchanged criterion). The binding constraint moves to
property effects E2 as predicted. Caps never bind under encoding-reach caps
except one `TooManyLhsEdges` (tetrahedral's K4 now converts — 6 binary edges
fit in 16). Every downstream novelty claim now carries **0.511** (0.561
ex-creation).

## Re-run after E2 (property effects + full guard ops)

**Run.** Same gate post-extension (release): `EFFECT` nodes carry the full
8-op set (decoded under V9, order preserved); guard ops completed to the full
`PropertyOp` set (the energy idiom is `gte` — without it the flagship rules
stay out); encoding-reach caps already in place.

| | Post-E1d | Post-E2 |
|---|---|---|
| Expressible | 23/45 (0.511) | **31/45 (0.689)**; non-creation 31/41 (0.756) |
| Identical (1000-step traj + 200/200 sweep) | 23 | 31 (no bloat-halt fired) |
| Divergent / Unmeasured | 0 | 0 |
| Solely-blocked-by | E1a 3, sched 2, … | property-compare 5, schedule 4, degree 2, quantity 1, weight 1; multi 1 |

**G-R2 verdict: PASS.** Two harness bugs caught by the gate during this
build, both fixed and documented in protocol §3: (1) the production match
budget censored triangle's unlabeled pattern into spurious Unmeasured —
budgets don't belong in semantics comparisons; (2) float synthesis `1e18 −
1.0 == 1e18` silently unsatisfying a strict `Lt` — synthesis is now
precision-robust (`above`/`below`/`ane` with sign-aware fallbacks), with
Bool `Gt true` / `Lt false` as the only honest unsatisfiables. Remaining
blockers are all *read* expressivity (compares, schedules) plus one weight
and one vacuous rule — the write side is done. Novelty claims now carry
**0.689** (0.756 ex-creation).
