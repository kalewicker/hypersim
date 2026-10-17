# ERE Failure Register (R0 spec)

**Status:** pre-registered interpretations. Each failure mode carries its
meaning *before* the run, so a null remains a finding. Thresholds and
formulas trace to `docs/ere_metrics.md` and `docs/metric_formulas.md`.

## FM-R1 Autophagy

Rule region consumed within *k* steps in A3 (unprotected self-maintenance).
**Predicted default.** Measure survival half-life by handle identity
(gen-0 anchor set) as a function of protection level and region size.
*Interpretation:* the half-life curve **is** the result. A3 collapsing is not
a failed experiment unless the half-life is unmeasured. Current evidence
(`hg-selfmod` v4, separate store): clonal reproducers die 4/6 at μ=0 —
reproduction destabilizes independent of variation; expect faster autophagy
co-resident.

## FM-R2 Freeze

Compiled rule set has no applicable matches (modal attractor — most rule
sets are dead). *Interpretation:* report the frozen fraction across the
mutation ensemble as a property of the encoding's search space (partly
pre-measured by G-R1's `Catastrophic` class). Freeze sustained ≥50 steps with
`productive_applied == 0` defines `time_to_freeze` (as in `runner.rs:677-685`).

## FM-R3 Bloat

Rule region grows without bound; match cost explodes. Hard ceilings on
`rule_region_nodes` and matches/step (see `ere_metrics.md`); the ceiling is
logged **binding** or **non-binding** every run. *Interpretation:* a result
produced under a binding ceiling is a result about the ceiling. Current
backstops: `world_ceiling=500`, `edge_ceiling=1000`, `rule_cap=400`,
`match_cap_per_rule=64`, `match_enum_budget=5.0e4` (`runner.rs:92-133`).

## FM-R4 Trivial quine

A rule whose only effect is copying its own encoding. High fidelity, zero
computation; mimics success (rising reuse + novelty). *Interpretation:*
gated out by `RuleReplicatorKind::NonTrivial`, which additionally requires
≥1 firing whose binding contains no node with label ≥ `META_BASE` (target)
— current code approximates via the productive/sterile effect gate
(`runner.rs:410-460`, G4.2). A quine scores `Trivial`, never `NonTrivial`.

## FM-R5 Meta point-source pump

Meta-rewrites concentrate at a single site (LK-track pathology one level
up). Detector: `meta_rewrite_site_entropy` collapsing toward 0 while
`meta_rewrite_events` stays high. *Interpretation:* finding about energy
metering, not about rule encoding.

## FM-R6 Novelty inflation

Relabeled duplicates counted as novel. Mitigated by RCF (G-R0).
*Pre-committed interpretation:* if G-R0 passes and novelty growth is still
linear rather than accelerating, that is **not** an instrumentation failure —
it is the Hash Chemistry result reproducing on a hypergraph substrate, and
the most important negative finding this track could produce. Report as such.
(V4 note: legacy `class_hash` novelty is inflation-suspect; all v4 novelty
slopes must be re-read under RCF before citing.)

## FM-R7 Quarantine capture

Most of the rule region quarantined; system runs on 1–2 surviving rules
while the region churns. Detector:
`rules_quarantined / (installed + quarantined)` with pre-committed alarm at
**0.9**. *Interpretation:* without this counter the run looks healthy from
outside. All validator counters present including zeros every run.
