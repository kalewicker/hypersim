# Selfmod v12 — controlled R1 child establishment

Design recorded before implementation and scientific output, 2026-09-26.
Follow-up to v11. Test the named R1 RHS-edge prune against faithful offspring
at a natural funded birth. This is not a broad mutation-rate sweep or probation
policy. Work is confined to the isolated LK workspace; historical v11 artifacts
and import provenance remain unchanged.

## Frozen panel and intervention

Eight independent world/scheduler seeds: **11,17,23,31,47,61,83,101** (none in
v11). Start ordinary `churn3` with corrected random tie breaking, match cost0,
one copy/member, capital60, full fission threshold100/transfer50, proportional
income pool12, influx20/free cap500, creation allowance200/refill1, canonical
match cap64, enumeration estimate ceiling50,000, world200 nodes/300 edges,
world ceilings500 nodes/1,000 edges and hard node stop20,000. All other
`v6_base_cfg` defaults remain. Mutation/abiogenesis/stochastic duplication and
deletion probabilities are zero; ordinary funded fission remains enabled.

Run a shared faithful prefix until the **first naturally funded member0 (R1)
birth**, searching at most256 object steps. Checkpoint after object effects and
income, immediately before that meta phase. Replay the complete native funded
fission phase from that checkpoint for four branches; other births in the same
phase are unchanged. Intervene on only the first R1 child (native root order):

1. **faithful:** native child unchanged.
2. **prune:** apply existing operator11 once with the exact saved v11
   `op11_member0_draw1` parameters:
   `[14984134727853237840,12375471953779355968,14176329756113589063,
   15783146528319595889,2475482604415574477]`.
   Require the decoded result to equal R1 with only RHS edge[0,2] removed,
   retaining[2,1]. No repair or parameter replacement.
3. **unscreened:** apply the ordinary operator/parameter block consumed for
   that same native child's birth. The native mu0 path already consumes the
   coin plus six-u64 block; record it through a shadow RNG, apply the saved
   operator once, and consume no extra run RNG draws. Retain no-ops and invalid
   outcomes; this is only eight unscreened draws, not an operator distribution.
4. **refund:** execute the native birth, return its exact50 tokens to its
   parent, and delete only that child graph. Keep the native RNG consumption.
   This deliberately cancels one birth, not all future reproduction; it is a
   capital/competition counterfactual, not an adoption proposal or free subsidy.

Child creation allowance and metadata are inherited by native duplication;
**do not reset allowance or inject capital**. Apply the intervention before
the ordinary creation refill. All initial birth transfers, siblings, world
stores and post-birth RNG states must pair exactly, except the named edit or
the refunded child. No continuation outcome affects checkpoint selection.

After the birth phase, run **2,000 additional object steps per branch**, with
native rent, matching/effect gating, income, full funded fission, faithful
future offspring and creation refill in their normal order. No subsequent
forced edit, viability filter, horizon extension, seed replacement or cap change.
Continue logging through biological extinction; only runtime failure or hard
explosion makes a trajectory incomplete. Missing triggers are explicit
untriggered seeds, not zero child productivity or replacements.

32 requested continuations ×2,000 =64,000 object steps, plus shared prefixes
at most2,048: **maximum66,048 scientific advancing steps**. Birth-phase forks
advance no object steps. Separate engineering preflight: seeds7/42, prefix≤256,
64-step continuations, eight branches. Cross-process verification may replay
faithful/prune for scientific seed11 once each (4,000 verification steps),
reported separately and excluded from scientific denominators.

## Measurement and criteria

Record every rule's permanent root identity, parent, seed member and whether
it descends from the selected child, independently of class identity. Track
descendants through actual native birth events. No metadata added to genomes
may change scheduling. Retain per-root fates, rent, income, capital before/after,
creation spending/allowance, decoding, gate estimates/flags, every birth and
transfer, world counts/hashes, reproducible RNG seed/stream/position and token
ledger. Retain the exact common checkpoint and all initial branch material.

Primary **bounded establishment**, for faithful/prune/unscreened:

- initial selected child executable; complete2,000 steps; no enumeration
  estimate skips anywhere in the continuation;
- at least one selected-child descendant alive at the end;
- that lineage productive in **every final250-step window**;
- lineage income≥lineage rent over the final1,000 steps (birth transfers
  within the lineage are internal, not income);
- every seed-member lineage has living copies at the end and some productivity
  in every final250-step window.

Refund has no child-establishment verdict (null); evaluate whole-cycle
persistence under the same latter condition. Replication threshold≥6/8 seeds
for each child arm. Report paired seed outcomes; do not require faithful
controls to survive to retain the seed. Also report direct child/parent death,
first child productive firing, lineage copy-time and fixed2,000-step rates,
late productivity/income/rent, ancestry shares and parent-relative capital.
Invalid child rates are null; enumeration-censored trajectory productivity
rates are null. Zero firings in measured valid opportunities remain zeros.
Creation/world gates and rule-cap binding are declared resource constraints;
record them and limit all conclusions to this operating point.

Predictions: initial paid-parent capital is identical in the three paid arms
and50 lower than refund. Pruning may improve establishment or merely compete
with its parent. Viable replacement in v11 may fail at birth with inherited
allowance or resident competition. Neither direction is assumed in routing.

If pruning establishes with persistent cycles, next consider a bounded
known-variant reproductive campaign. If viable children fail while refund
helps, design a separately preregistered commitment/probation test. If faithful
and prune both fail similarly, inspect native birth costs, inherited resource
budgets and competition before retuning. No routed experiment is automatic.

## Evidence and validation

Freeze source, protocol, executable and hashes before scientific evaluation;
refuse directory overwrite and preserve failed/partial output. A streamed raw
receipt includes all rows, birth genealogies, shared-prefix/checkpoint evidence,
branch initial states, aggregate verdicts and status. Independently reconstruct
token flows, income rounding, inherited creation budgets, genealogy, gates,
counts and verdicts. Test recorded/native fission parity (material, output and
RNG), named child genotype, exact paired birth finances, checkpoint restoration,
and malformed/censored classifications. Engineering fixes precede the frozen
scientific run; do not revise thresholds after seeing results.
