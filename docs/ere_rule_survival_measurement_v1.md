# Rule survival measurement v1

Date2026-10-07. Profile `E2-rule-survival-measurement-v1`.
This is an archive-only engineering measurement layer for the next maintenance
experiment. It evaluates already exposed receipts and introduces no scientific
acceptance threshold, new physics or optimizer update.

## Measurement contract

The input is a full, independently audited native timeline, its initial
candidate catalogue, a specified target RCF and actual region-touch event steps.
The caller must bind event steps to the native ledger: clock ticks and edits
outside the family are excluded. Multiple actual touches at one step remain
multiple events. A first-spell survival estimate never merges later recoveries.
Targets present at initialization, or structurally equal after removing
priority, are refused as novelty. A target that never appears stays in the
fixed cohort as a missing birth; it is never assigned a fabricated zero lifetime
or dropped from the success denominator. No ordinary firing means ineligible.

Report genotype class spells, pre-step installed class spells and generational
root survival separately. Installed attribution uses the native pre-step table,
including its existing final-row convention; there is no extra final compile.
An observed end gives duration=end-start. An open spell has only the lower bound
horizon-start, and its death age remains null. A root that dies cannot be
replaced by a later generational handle for survival credit.

For a fixed cohort, half-life is the first age at which at least half the cohort
has ended its first spell. Compute conservative bounds by order statistics:
rank=ceil(N/2), lower bounds sorted at rank, and completed death ages plus
infinite unresolved censored ages sorted at rank. Return a point only if finite
lower and upper bounds agree. Otherwise return null plus bounds. No independent
censoring assumption, survival extrapolation, horizon-as-death or midpoint
estimate is used. If any fixed member has no birth, the cohort half-life is
unmeasured and the missing-birth count is explicit.

The original literal lifetime ratio is reported only if both genotype and
installed half-lives are observed, all fixed members are novel/nontrivial, and
every member has a measured region-touch interval. Both point estimates must
strictly exceed the mean actual interval; equality fails. Missing intervals
are null, never infinity. This numerical flag cannot grant scientific acceptance.

A separately supplied, positive reference exposure interval provides an
additional robustness comparison using the same lifetimes. Increasing repair
event counts may lower the all-touch mean, but cannot lower this reference.
The original literal flag and stronger comparison remain separate; neither
retroactively changes the previous experiments' criteria or verdicts.

## Archive-only engineering panel

Recheck exactly24 saved receipts:18 learned-graph decisions from the new native
confirmation (nine worlds ×two existing physics seeds), plus the three
live-paid and three live-replacement historical lifecycle cells. Bind copied
bytes to the two original root seals and independently audit each full ledger.
Use the prospective confirmation's specified target proposal for each world;
the historical target is the exact proposal with label10. Analyze each physics
seed's nine-world cohort separately, and historical arms separately. Physics
seeds are controls, not independent biological replicates.

The historical replacement case has first genotype and installed lifetimes8,
edit interval8 and an open root lifetime; equality must fail. The historical
one-shot case has lower bounds23/22 and no replacement interval. Fresh learned
decisions have two paid edits, but no ongoing exposure sequence or observed
class death. Their finite observation cannot establish an observed half-life.
These known cases verify the instrument; this panel is retrospective engineering.

## Implementation and safeguards

`scripts/ere_rule_survival_metrics.py` provides `presence_spells`, `duration`,
`half_life_bounds`, `cell` and `cohort`. It is a pure measurement library;
ledger/canonical verification remains the caller's responsibility.
The archive runner freezes this library, tests, contract and its existing
native-verifier dependencies, copies hash-bound inputs, preserves old archives,
stores every cell/cohort and seals success or failure. Tests cover actual8/8
and one-shot records, missing births, censor ambiguity, reappearance, root/class
distinction, initialization/priority exclusions and touch/timeline refusals.
All outputs retain scientific_acceptance, sustained_claim and training_admitted
false. No existing native kernel or archived summary is modified.
