# Parallel birth admission in the full D0 ecology

Implemented 2026-10-03 (America/Phoenix) as the separate opt-in `evolving-minds-d0-admission-v1` engineering profile. The profile removes global bookkeeping contention, applies a supplied semantic lottery to contested offers, and supports concurrent paid mutation settlement across complete lifecycle ticks. Scientific launch, H1/H2 passage, training, evolution and broad reproductive fairness remain false.

## What changed

The [prospective contract](../experiments/evolving_minds/contracts/d0_admission_v1.md) fixes the profile, twelve cells, horizons, seeds, controls and limits. [Native laws](../rules/evolving_minds/admission_v1.toml) derive from the frozen mutation/family laws with the original prices. [Admission runtime](../crates/hg-sim-core/src/evolving_minds/admission.rs) performs read-only offer enumeration and sampling; [family orchestration](../crates/hg-sim-core/src/evolving_minds/family.rs) executes all costs, construction and settlement through the existing engine. No engine, graph-store or dependency change is required. Earlier family/mutation/lifecycle profiles and their checkpoint readers retain their meanings.

There is no global counter in the new profile. Each site's birth/request frontier starts at 3 plus its logical slot. One mandatory native pass advances every site's frontier by eight each tick. Selected requests copy that frontier, and successful children use the same ID. Disjoint target/tick pairs therefore have unique IDs without binding shared bookkeeping. Rejected proposals leave holes; birth IDs are monotone in birth time, not contiguous. This differs from the earlier local arbitration prototype, whose frontier advanced only upon a birth.

After mandatory maintenance, the scheduler enumerates every eligible parent and empty neighboring target. It sorts semantic offers by parent birth ID, logical home/target and genome length, shuffles with a separately seeded SplitMix64/Fisher-Yates stream using rejection sampling, and greedily selects offers that share neither a parent nor a home/target site. Only selected pairs receive specialized native reserve rules. The engine still enforces node-disjoint application. The observer checks that every selected request was admitted and that no other request appeared.

This lottery is explicitly supplied environmental reproduction scheduling. It is not organism cognition or an evolved policy. It removes native handle and local rule-ID ordering from offer selection. Greedy selection does not sample a uniform maximum matching or equalize different eligibility, offer counts, construction costs or resource access.

Each admission pays charge 1 and reserves child energy 12 plus its length-dependent copy fee. A request-local stream makes one source proposal, with no repair, redraw or fallback. Native request-specific settlement rules either create the declared child with reset runtime fields or refund the unused reserve after invalid syntax. Zero-energy escrows dispose natively. Multiple different proposals cannot accidentally apply to one another's escrow. In this new profile, `settlement_rule` records the specialized rule name; its exact effects are reconstructed from frozen templates and proposal metadata. Older mutation records retain their complete TOML meaning.

The extra identity pass makes 46 native passes per tick. `HGEMA001` checkpoints preserve recipe, offer/lottery/proposal history, graph, allocator, engine state and ledgers at completed ticks. Native replay rejects forged histories, altered fixed identities, partial boundaries, trailing data and incompatible profiles. Full-history checkpoint storage remains bounded; streaming storage is the next implementation package.

## Fixed full-lifecycle panel

Twelve cells run 64 ticks: three recipes × physical rotations 0/3 × forward/reversed initial allocation. Each begins with two common-seed founders of reserve 64 and eight sites of reserve 32, for total initial energy 384. The world has the existing informative seed-17 schedule. Descendants inherit their founder's declared mutation kernel. Every cell retains the full horizon, including deaths and extinct lineages.

The following are the rotation-0 forward endpoints; the checker verifies each cell individually and tests allocation invariance across all six pairs.

| Recipe | Requests | Births | Invalid rejections | Deaths | Live at tick 64 | Maximum depth | Harvested | Dissipated | Final energy |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Faithful, disjoint founders | 30 | 30 | 0 | 24 | 8 | 8 | 3112 | 1944.375 | 655.625 |
| Mixed genomes, competing founders | 30 | 30 | 0 | 24 | 8 | 8 | 2784 | 1816.875 | 695.125 |
| Invalid/source-kernel settlement | 50 | 15 | 35 | 14 | 3 | 5 | 600 | 576.125 | 591.875 |

Every energy residual is zero. In the faithful fixture, two admissions commit in the same tick on ticks 3, 4, 17, 21, 34, 38, 51 and 55. The mixed-settlement fixture has nine ticks with multiple admissions; ticks 4, 6, 9, 11 and 15 contain both a valid and an invalid proposal. All requests settle once and all escrows disappear before completed ticks. Nine paid development failures in that fixture retain their costs and lineage provenance.

The mixed-genome fixture uses cue (three instructions) and delayed-cue (five instructions) founders. Both fund children, but it produces no simultaneous mixed-length admissions within this fixed ecology. That negative coverage result is preserved. A separate controlled native test develops both founders before opening offers and verifies two simultaneous reservations and copies with lengths three/five, distinct IDs, child reset and the exact charge `2 + copy_cost(3) + copy_cost(5)`. This establishes the settlement mechanism without treating that fixture as ecological viability evidence.

All allocation pairs have identical semantic offers, raw draws, admissions, proposals, lineage and physiology rows. Matcher work and opaque handles remain diagnostics. One/four-thread tests also match exact native journals for the same allocation. The faithful cue fixture has equivalent semantic trajectories under site rotation; source-kernel variants retain physical-site coin schedules and do not assume rotation equivalence.

These endpoints do not compare scheduler productivity with the older single-founder family profile: the founder count, request charge and reproductive schedule differ. They do not establish useful inherited variation, adaptation or whole-population equal opportunity.

## Lottery calibration and verification

The fixed read-only calibration contains 1,024 cases: 32 seeds × eight site rotations × two allocation presentations × disjoint/competing offer pairs. Every disjoint pair admits both parents; every competing pair admits one. A given seed produces the same semantic selected set across all presentations and rotations. Across the 32 distinct competing seed cases, parent 1 wins 18 and parent 2 wins 14. These are reported counts, with no statistical fairness gate. Additional tests exercise shared-home/target conflicts and multiple offers from one parent.

The [independent checker](../scripts/verify_evolving_minds_admission.py) reconstructs all 35,328 native advances across twelve cells, full material grammar, mandatory phases, balances, resets, candidate inventories, lottery draws, one-shot mutation draws, specialized native effects, rows and ancestry. It checks frozen source/law bindings and rejects unsupported research claims. Python validates checkpoint envelopes/checksums; native replay validates opaque contents. All 105 final frozen source entries match the executed source inventory.

Twenty selected Rust tests passed: three admission integration tests, two new controlled/checkpoint unit tests, and fifteen existing family/mutation/lifecycle regression tests. The fifteen adversarial/read-only checker tests cover omitted offers/requests/identity steps, forged lottery selection/draws/refunds/IDs, free requests, missing concurrent native effects, wrong frontiers, corrupted checkpoints and false fairness claims. Feature-enabled and feature-disabled checks passed. Fresh-process tick-8 continuation matches all canonical evidence through tick 64. Every earlier sealed Evolving Minds archive remains intact.

Evidence: [final audit](../output/evolving_minds_admission_verified_20261003/verification.json), [checks](../output/evolving_minds_admission_verified_20261003/checks.json), [continuation and invariance](../output/evolving_minds_admission_verified_20261003/continuation_verification.json), and [archive seal](../output/evolving_minds_admission_verified_20261003/output_sha256.json). The [development archive](../output/evolving_minds_admission_development_20261003/verification.json) keeps its original frozen checker and separate positive audit. Final verification additionally checks faithful site-rotation equivalence and forged native checkpoint histories.

Preflight failures were test/compilation diagnostics: an assertion initially compared deliberately different allocation-recipe metadata, and another incorrectly required simultaneous admissions in the mixed-length ecology. The assertions were corrected to the declared scope, with the original ecology and denominator retained. Mixed-length concurrency received the controlled native test described above. Native parsing/build issues were corrected before the corresponding archive execution. These diagnostics are recorded in the checks receipt; no failed ecology was replaced or tuned to a desired endpoint.

## Next implementation package

Implement separately versioned live-state checkpoints with append-only event/accounting segments, instead of embedding and replaying the entire history. Preserve exact native allocator/RNG and admission state, link continuation manifests by committed offsets/hashes, and verify interrupted-write/output-budget recovery into a fresh directory. Measure execution, storage and independent audit costs under frozen resource limits before extending horizons or launching evolutionary trials. Viable mutation-neighborhood calibration and broad reproductive-fairness assessment remain prerequisites for scientific selection claims.


Storage follow-up (2026-10-03): package 5 is now implemented in a separate streaming-v1 profile. See [live checkpoint and segment findings](evolving_minds_streaming_findings.md). The admission-v1 full-history archive and checkpoint format remain intact.
