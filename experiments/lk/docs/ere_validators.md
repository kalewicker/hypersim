# ERE Validators V1–V9 — Phase R0 deliverable

**Status:** implemented in `crates/hg-meta/src/validate.rs`; 12 unit tests
(one per class + ordering + boundary + counters). Engagement-counter keys
are stable strings for metrics rows; zeros are reported, never omitted
(G-R4).

## PROVENANCE / SPEC GAP

The plan's §5.4 — the section that was supposed to define V1..V9 — is
absent from the v1 document body. These nine validators are instantiated
from E1's actual failure modes and the plan's cross-references (SB-R1,
SB-R2, FM-R3, §11 epoch stratification). They await ratification by the
plan owner; amendments should be made HERE and in `validate.rs` together,
never in code alone.

Also recorded: the reader (`schema.rs`) rejects walk-level structural
failures via typed `ReadRejection` — those are "no rule could be formed",
distinct from quarantine ("a candidate formed but is barred"). The split
keeps read totality clean and makes G-R4 accounting unambiguous: only
validator hits land in quarantine counters.

## The nine validators

Deterministic order V1→V9; first failure wins, so quarantine attribution
is stable across runs and builds.

| Id | Predicate (fails when…) | Protects | Counter key | Fixture test |
|---|---|---|---|---|
| V1 | LHS pattern-edge list empty | Creation rules cannot be encoded endogenously (SB-R2); an encoded rule with no LHS is an unguarded emission, not a rewrite | `v1_lhs_empty` | `v1_fires_on_empty_lhs` |
| V2 | any pattern edge has arity 0 post-compaction | Arity-0 edges match nothing but inflate match cost — pure FM-R3 bloat | `v2_arity_zero` | `v2_fires_on_arity_zero_edge` |
| V3 | any pattern edge arity > MAX_ARITY (=8) after slot compaction | Expressibility cap (SB-R1): over-cap edges cannot round-trip to `Rule` at compile time | `v3_arity_overflow` | `v3_fires_over_max_arity` |
| V4 | any var label constraint ≥ META_BASE | Ordinary rules must not pattern-match the meta band itself — v1 enforces the epoch/Tarski stratification statically (§11) | `v4_meta_label` | `v4_fires_on_meta_band_label` |
| V5 | RULE_ROOT priority property present but non-Int or out of i32 range | The reader would silently coerce to 0, changing semantics; quarantine keeps the breaking mutation visible in accounting | `v5_priority_malformed` | `v5_fires_on_malformed_priority` |
| V6 | same var appears twice within ONE pattern edge | hg-engine matches endpoints as unordered multisets with distinctness semantics; a repeated var is dead-on-arrival or degenerate | `v6_repeated_var_in_edge` | `v6_fires_on_repeated_var_within_one_edge` |
| V7 | region node footprint > REGION_NODE_BUDGET (=256) | FM-R3 bloat ceiling; binding/non-binding status logged by compile layer every run | `v7_node_budget` | `v7_fires_over_node_budget`, `budgets_exactly_at_ceiling_are_valid` |
| V8 | meta-band nodes adjacent to the region but unreachable from its root within r_meta (orphan fragment — dangling-cleanup amputation detector) | Plan §2.5 handle-identity principle: a partially-eaten region must not pass as a rule | `v8_orphan_fragment` | two-stage: fires in compile layer (Phase R3), reserved here |
| V9 | region edge footprint > REGION_EDGE_BUDGET (=512) | FM-R3 bloat ceiling, edge side | `v9_edge_budget` | `v9_fires_over_edge_budget` |

Budgets are INCLUSIVE at the ceiling (strictly-greater trips the validator)
— pinned by `budgets_exactly_at_ceiling_are_valid`.

## Known coarsening

V8 is a two-stage validator: the offline harness (no snapshot at validate
time) reports Valid; the compile layer engages it. As of R4, walk-level
read rejections in `GraphRuleSource` count under V8 — EXCEPT
`ArityOverflow`, which maps to V3 (its predicate surfacing at the walk).
The original "adjacent-but-unreachable fragment" audit remains reserved;
the counter is live either way.

## E1-G (landed R4, commit 9b0e1e5)

Guards are part of the encoding: `VAR --E_GUARDS--> GUARD{key,op,val}`.
Reader rejects `GuardEndpointNotGuard` and `GuardMalformed` (op outside
0..=2 per the selfmod v2 convention 0=Gt/1=Lt/2=Eq, key out of u32,
missing threshold). Guard nodes count toward the V7/V9 footprints.
`candidate_to_rule` translates guards to engine `NodePropertyGuard`s.

## Ordering determinism

`first_failure_wins_in_v_order` pins: a candidate violating multiple
classes reports the LOWEST-numbered one, so quarantine histograms are
comparable across builds regardless of internal evaluation order.

## SB-GE3(b) ratification (2026-09-04) — var-guard consistency fixed at the kernel, no V10

G-GE1 recorded 166 uncompilable draws, all on `remove_slot` (134) /
`retarget_slot` (32): slot edits evicted a var's last LHS wiring while
its E1-G / E1-V entries survived, passing validation and failing engine
compile (`InvalidGuard("... not in LHS")`, `hg-rules/src/compiled.rs`).
Decision, per the HERE-and-code-together convention: fix at the mutation
kernel (`mutate.rs::drop_lhs_orphaned_guards`, called by both slot ops —
same drop semantics `remove_var` already applies, no renumbering since
slot edits keep indices stable; E2-lite effects untouched, they resolve
against RHS indices), NOT a new V10 validator. Rationale: the kernel can
no longer produce the class, so a validator for it would be a dead
instrument (FM-GE2) — pinned instead by three kernel unit tests
(`mutate::tests`, incl. a 32-seed property sweep that no slot-op output
carries guard refs off the LHS). `validate.rs` unchanged by design.
