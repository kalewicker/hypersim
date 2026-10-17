# G-R2 Expressivity Protocol (EXP-R02)

**Status:** specification + implementation (`crates/hg-selfmod/src/gr2.rs`).
Gate: `hg-selfmod gate_gr2`.

## 1. Question

What fraction of the shipped rule library can the rule encoding express at
all? That fraction *is* the expressivity envelope and appears in every
downstream novelty claim — a novelty result without it cannot be
distinguished from the encoding having made the "novel" rule reachable by
construction.

## 2. Conversion (TOML → RuleSpec)

`rule_to_spec` (pure function of rule + caps, no RNG/store). Variable order
follows `lhs.nodes`, creates follow `rhs.nodes` order — the same conventions
as `compile_rule`, so any behavioral difference is the encoding's, not the
indexing's. First blocking reason wins, ordered fundamental-first:

`Schedule` (SB-R2) → `LhsEdgeLabels` (E1c) → `RhsEdgeLabels` →
`PropertyCompare` → `DegreeCompare` → `QuantityCompare` → `Weight` →
`GuardOp` (defensive only — all 6 ops convert) → caps (`TooManyVars`,
`TooManyLhsEdges`, `Arity`, `TooManyCreates`) → `Vacuous` (mirrors V9).
`VarsDistinct` (E1d, lifted) expands to pairs; property effects (E2, lifted)
map op-for-op to `EffectSpec`.

Caps are **encoding-reach** (`gr2_caps`: 8/16/8/8, E1's `MAX_ARITY` scale),
not evolution experiment caps — the envelope measures the encoding.

Notes: explicit-but-default `Label(0)` RHS edge labels are expressible;
undeclared LHS label names are ignored exactly as `compile_rule` ignores
them; relabel-to-same is preserved (both sides no-op-fire identically).

## 3. Behavioral identity

Each expressible rule: baseline = `compile_rule` direct; roundtrip =
convert → `install_rule` → `decode_all` (must yield exactly one rule) →
compiled. Two legs:

1. **Trajectory** up to `GR2_STEPS = 1000` on the cloned auto-fixture with
   same-seed streams and unlimited budgets, uncapped match enumeration
   (harness economics must not censor matching — a production throttle has
   no place in a semantics comparison). FM-R3 bloat-halt at 2000 nodes /
   4000 edges (both arms must halt together; a halt-step mismatch is
   divergence). Minimum 10 compared steps for a Measured verdict.
2. **Differential sweep**: one step on each of 200 random worlds (12–28
   nodes, labels 0–3, seed match injected so all engage), all must match
   exactly (hash + firing counts).

Compared everywhere: `canonical_hash`, applied count, productive count;
plus final RNG draws after the trajectory. First divergence (either leg)
fails the rule loudly with its location.

Auto-fixture (per rule, deterministic from `GR2_FIXTURE_SEED`): background
soup (64 nodes / 96 edges / 6 labels), then one node per LHS var carrying its
label constraint and guard-satisfying properties (strict-side ±1 around
Int/Float thresholds via precision-robust `above`/`below`/`ane`, exact for
Eq, flipped for Bool Ne; unsatisfiable combos — Bool `Gt true` / `Lt false`
under the engine's total Bool order — refuse → `FixtureUnsupported`), then
the LHS edges. The seed match guarantees step-0 engagement by construction;
a rule that still never fires reports **Unmeasured** (a measurement of the
fixture, never counted as identical).

## 4. Verdict

**PASS** iff every expressible rule is behaviorally identical with
engagement. Low coverage is a stated limit (PASS with the fraction reported),
never a failure. Any `DivergentAt` / `Unmeasured` / roundtrip-decode failure
is a converter or harness bug (FAIL). Companion fraction over non-creation
rules (excluding `Schedule`) is pre-committed here as the SB-R2 companion —
not cherry-picking, because creation rules are a different kind by owned
spec limitation. The report additionally ranks **solely-blocked-by** counts
(rules whose full reason set from `inexpressibility_reasons` is a singleton)
to prioritize encoding extensions; multi-blocked rules need full-set analysis
per candidate extension.

## 5. Constants

| Constant | Value |
|---|---|
| `GR2_STEPS` | `1000` |
| `GR2_MIN_TRAJ_STEPS` | `10` |
| `GR2_BLOAT_NODES` / `EDGES` | `2000` / `4000` |
| `GR2_SWEEP_WORLDS` | `200` |
| `GR2_SOUP_NODES` / `EDGES` / `LABELS` | `64` / `96` / `6` |
| `GR2_FIXTURE_SEED` / `TRAJ_SEED` | `0x6232_…0001` / `…0002` |
