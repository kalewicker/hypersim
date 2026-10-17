# Constructive chemistry and higher-order rewriting: Hypersim adaptation

**Revised technical plan — 2026-10-05, America/Phoenix.** This improves the supplied six-stage proposal and adapts it to the current Rust workspace. CC-1 checked staging, chemistry authority and scripted fixtures are implemented. Later kinetics, durable continuation, construction and scientific stages remain proposed. The [integrated roadmap](evolving_minds_constructive_chemistry_integrated_plan_20261005.md) owns milestone ordering and gate IDs. Existing D0 and ERE results are engineering precedents, not evidence that this chemistry already exists.

**CC-0 complete — 2026-10-05:** The [compatibility specification](constructive_chemistry_cc0_compatibility_spec_v1.md) now fixes the proposed schema/commit seam and [CC-1 contract](../experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json):41 cases/82 worker variants, checked closed/open material-energy ledgers and scripted validation without kinetic time. [CC-0 review findings](constructive_chemistry_cc0_findings_20261005.md) distinguish that design milestone from runtime evidence.

**CC-1 complete — 2026-10-05:** [Verified atomic chemistry findings](constructive_chemistry_cc1_findings_20261005.md) report all41 cases/82 worker variants, exact graph/allocator/ledger rejection identity, balanced finite ledgers, independent semantic replay and existing-profile regressions. The profile is default-off and scripted; physical kinetics and durable continuation remain CC-2.

## 1. Physical design retained from the source

Use one material world under fixed laws for each autonomous run. Represent a fixed spatial substrate S, evolving material graph M, and append-only causal evidence C. Space constrains encounters and transport; material implements catalysts, templates, barriers and later operators. C records history and must not decide physical eligibility.

Retain four layers, activating them separately:

1. **Fixed physics:** material types, locality, valence, energy accounting, reaction actions and scheduling.
2. **Constructive chemistry:** ordinary material assemblies catalyze reactions, copy templates and build boundaries.
3. **Embodied effective operators:** bounded material motifs determine local recognition and one allowed physical action under a fixed interpreter.
4. **Material descriptions and interpreters:** descriptions acquire effects only through physical interaction with an interpreter/adaptor assembly.

The first reproducer is explicitly authored. Its components must still operate through generic laws applicable to neighboring structures. No whole-genome hash identifies a privileged seed, no host `replicate_cell` clones an organism, and no observer label grants reproduction or permeability.

## 2. Compatibility audit and software route

| Hypersim component | What can be reused | Required extension or restriction |
|---|---|---|
| [`hg-core` graph](../crates/hg-core/src/graph.rs) | Generational node/edge identities and node properties | Define material types, port roles and persistent semantic IDs. Ordinary unordered hyperedges are not directed bonds with arbitrary edge properties |
| [`hg-rules` effects](../crates/hg-rules/src/effects.rs) | Typed compiled rule representation and matching vocabulary | Existing integer addition/subtraction saturates; chemical ledgers require checked arithmetic and atomic failure |
| [`hg-engine`](../crates/hg-engine/src/engine.rs) | Candidate enumeration, execution infrastructure and opt-in integration point | Current application is in-place and records skipped property effects; it is not a general validate-before-write chemistry transaction |
| [`conflict resolver`](../crates/hg-engine/src/conflict.rs) | Current binding/conflict semantics | Extend or prove adequacy for chemical reads, writes, neighborhood/absence tests and cleanup effects; node overlap alone is not the new transaction contract |
| [`RewriteEvent`](../crates/hg-observe/src/event.rs) | Binding/topology changes, cleanup neighbors and existing vanished-energy field | Add chemistry-specific property deltas, material/energy ledger, physical-time/RNG and causal read provenance. Existing vanished-energy accounting is not this chemistry's conservation proof |
| [`GraphRuleSource`](../crates/hg-meta/src/source.rs) | Source identity and rule lifecycle patterns | Global epoch compilation cannot establish local embodiment or immediate disappearance of a destroyed catalyst |
| [`meta schema`](../crates/hg-meta/src/schema.rs) | Bounded validation ideas | Existing encoding excludes several needed effects/constraints; use a separately specified chemical operator schema |
| [`hg-sim-core` manifest](../crates/hg-sim-core/Cargo.toml) | Feature-gated orchestration and profile-specific binaries | Implemented default-off `constructive-chemistry` feature and independent CC-1 audit; new checkpoint identity remains CC-2 |
| Existing Evolving Minds streaming/auditors | Design patterns for allocator continuation, committed segments and source freezes | New chemical event/time/ledger schemas; no claim of codec compatibility |

Current CC-1 APIs are in `constructive_chemistry/mod.rs` and `fixture.rs`, with generic staging in `hg-engine/src/checked.rs`. The broader layout below remains proposed:

```text
crates/hg-sim-core/src/constructive_chemistry/
  config.rs       schema.rs       runtime.rs
  chemistry.rs    geometry.rs     kinetics.rs
  checkpoint.rs   output.rs       observers.rs
  operators.rs    # only when HO-1 is contracted
configs/constructive_chemistry/
rules/constructive_chemistry/
experiments/constructive_chemistry/contracts/
scripts/          # independent chemical reference/audit tools
```

CC-1 now places generic checked staging in `hg-engine` under its default-off feature; chemistry owns whole-world commit authority. Keep chemistry-specific energy and geometry predicates in the profile behind a narrow interface. A standalone `hg-chem` crate is optional later; do not create a second engine or copy the isolated `experiments/lk` workspace into root crates.

## 3. Representation and invariants

### 3.1 Material and space

Begin with a tiny explicitly enumerated local patch. A periodic triangular lattice with six neighbors remains a later candidate, not an already calibrated default. Freeze coordinates, adjacency, orientation conventions, boundary conditions and capacity before a fixture. Reject lattice sizes with accidental duplicate neighbor identities under the selected topology.

Use A/B/C/D as provisional conserved material classes, with declared ports/valence, activation state and orientation. Bonds requiring direction, state or energy should be represented by role-labelled incidence structures and state-bearing relation nodes consistent with Hypersim's unordered ordinary-edge semantics. Register labels/property keys without collisions. Separate administrative relation nodes from physical inventory explicitly: administrative metadata must not hide uncharged catalyst capacity or memory.

Use semantic bead/event identities independently of arena handles. Checkpoints preserve generational handles and allocator continuation as well as semantic IDs. Do not encode a raw handle index into a property and assume it survives deletion/reuse. A copied physical bead is constructed from material with a new identity; it is not free duplication of the original ID.

### 3.2 Matter and energy

For each accepted internal event require exact species/material-vector balance, `ΔN = 0`. Reservoir exchanges carry explicit signed material entries. Bond formation, activation and transport must satisfy a declared local state-energy function and reservoir/work/heat ledger. Account for F*→F energy once; do not both credit activation energy and separately create an unrelated fuel budget.

Use checked integer/fixed-point arithmetic for conserved inventories and the selected energy proxy. Overflow, missing properties, invalid types, impossible valence, occupancy violations and insufficient energy reject the whole event. Explain what the energy proxy models; an arbitrary instruction fee is not a thermodynamic derivation. Physical-time sampling may use floating-point arithmetic under a separately documented numerical contract.

Conservation includes free substrates, polymers, boundaries, products, debris and reservoir transfers. Death does not erase material. Dissolution and recycling use the same declared reaction laws. Observer bookkeeping has no authority to repair an imbalance.

### 3.3 Compartments and topology

Transport eligibility is determined by local material barriers and encounters, never a cached `Cell` owner tag. Define local crossing/repair/scission rules before calling an assembly a membrane. On a torus there is no distinguished global outside: classify contractible enclosures, winding boundaries, open boundaries and ambiguous structures explicitly. Use an independent periodic topology observer to validate interpretation; do not insert a global enclosure oracle into a purportedly local reaction.

A fission event label follows actual material separation and the predeclared observer criterion. It cannot trigger genome copying, resource resetting or deletion of inconvenient fragments. Record lineage ambiguity, fusion, multiple contributors and unassigned debris rather than forcing every event into a binary tree.

## 4. Atomic rewrite contract — CC-1

Introduce a proposed checked patch representation carrying source rule/action identity, bound semantic and generational IDs, complete read dependencies, proposed writes, material/energy changes and local geometry requirements. This is an API design target, not the existing `Engine::step` interface.

Read dependencies include relevant incident edges, port occupancy, absence predicates, material state and cleanup effects. Versioning only nodes already matched misses phantom insertions that invalidate a free-port or empty-site test. Include the appropriate neighborhood/version guards.

Evaluate the complete candidate on staged state before publication. Revalidate at commit, including the source catalyst, checked arithmetic and every invariant. Use a design that guarantees rejection leaves graph, allocator, ledger and accepted-event history unchanged; prove this with fault injection. Rejected-attempt RNG/time changes, if the chosen kinetic model defines them, are explicit and separately logged. Do not apply structure first and silently skip a failed chemical property effect.

The minimum fixture contains two eligible reactions sharing one fuel unit. Only one may spend it. Add independent reactions, stale occupancy, deleted catalyst, generation-reused handle, dangling cleanup, negative inventory and overflow cases. Compare with a small independent reference implementation. Test the enabled and default-off paths; preserve existing profile behavior.

## 5. Kinetics, determinism and limits — CC-2

Choose one reference stochastic law before optimization. Recommended initial specification: a **serial continuous-time reaction process** that enumerates all eligible channels on the bounded state, assigns declared nonnegative local hazards, draws the waiting time from the total hazard, selects a channel proportionally, commits one event, and recomputes affected eligibility. Freeze multiplicity/symmetry counting so equivalent bindings do not accidentally multiply reaction rates. Zero total hazard is an explicit absorbing state.

Define units, numerical treatment of zero/extreme draws, hazard updates, tie handling and the relationship between physical time, accepted events and computational attempts. If an attempted-reaction/uniformization model is chosen instead, it needs its own rejection/time law and reference fixtures; it cannot be silently substituted. Hypersim's sequential weighted mode alone does not establish either process.

Specify RNG algorithm/version and stream keys based on semantic event/attempt identities, not allocator order or worker completion. Persist physical time, counters, generator state, pending scheduler state if any, graph/allocator, reservoir ledgers and all state required to reconstruct eligibility. Same-build replay compares exact events/state; cross-backend floating-point or distributional equivalence is a different claim requiring an explicit tolerance/statistical contract.

Start with parallel candidate enumeration only if it agrees with the serial reference and deterministic commit. Batch commits require a separate proof/test that hazards, conflicts and time semantics remain equivalent. Do not parallelize by observer organism boundaries. Metal remains deferred until a measured bottleneck justifies it and a semantic oracle exists.

Distinguish **physical bounds** (radius, action vocabulary, valence, operator size) from **engineering caps** (memory, candidates, events, logs). A binding engineering cap halts and marks the trajectory incomplete; it cannot quietly drop reactions. Increasing a physical bound changes the law version. Benchmark execution, checkpoint size, event storage and independent audit before choosing world dimensions or population sizes; the source's 128×128/100-founder/48-unit defaults are uncalibrated proposals.

## 6. Provenance and persistence from the first fixture

Persist complete compact accepted-event records sufficient to reconstruct material state, property changes, physical time, ledger entries, source motifs and causal dependencies. Preserve RNG/scheduler/rejection information required by the chosen process. Every used precondition needs attribution, including read-only catalysts and absence/occupancy dependencies; a product-only genealogy is not full causal evidence.

Use chunked append-only segments with inventory/hash commits, source/binary/law/config hashes, monotone semantic event IDs and versioned checkpoints. Define crash boundaries and recovery publication rules; an uncommitted tail cannot become accepted history. Record cache derivation versions and either restore or rebuild caches without changing the next event. Verify fresh-process continuation across a segment boundary, interrupted writes and deletion/reuse of arena handles.

Rich snapshots and viewer samples may be secondary products. Do not choose complete logging windows after seeing interesting outcomes. If later scale forces a reduced-evidence profile, preregister its claim limitations; it cannot inherit exact causal reconstruction claims by name.

Observers derive cells, genomes, births, deaths, generations and functional classes. Store operational definitions and versions. Separate genotype birth, operator installation, first ordinary firing, class persistence and root survival. A successful decoder or recurring class hash does not establish selection.

## 7. Stage 1: construct and qualify an authored reproducer

Use the roadmap's CC-3 through CC-6 sequence:

- **Copying:** generic local templating from paid substrates, with explicit initiation, extension, termination, strand separation and fidelity. Test altered templates and sequence neighbors before a population campaign. A host copy loop cannot hide construction work.
- **Boundary:** material assembly, selective transport, growth, repair and scission under local geometry. Test leakage, missing components, open/winding configurations and debris.
- **Coupling:** maintenance, copying, boundary growth and segregation in the same frozen world. Supply an authored ancestor and label all external assistance. Remove fuel, template, catalyst or boundary; use matched rescue and sham controls.
- **Qualification:** repeat unmutated reproduction through a prospectively fixed independent-world panel and horizon. Define offspring viability as its own maintenance/copying/reproduction over a specified subsequent window; separate viable, nonviable and right-censored offspring. Do not define viability merely as whatever survived a convenient endpoint.

The source's example 8/10-world, 20-transition and 100-fission criteria are not adopted as calibrated thresholds. First measure attainable times and variation on exposed development fixtures, then freeze biologically meaningful thresholds, world count, power assumptions and a fixed physical-time horizon. Fissions within one genealogy are not independent replicates. Event ceilings are safety limits, not an alternative outcome-dependent stopping rule. No resetting, culling, survivor replacement, hidden warmup or manual rescue in the autonomous condition.

Before mutation campaigns, enumerate a bounded predeclared catalyst/template neighborhood or report its sampling coverage. Include faithful, neutral, impaired and altered-function controls with all unreachable/nonviable routes retained. Failure to find useful accessible variation is evidence about this world, not grounds to skip to operators.

## 8. Stages 2–4: function, niches and cumulative opportunities

**EC-2:** Introduce physical copying errors under a new contract. Measure inherited function on a fixed assay panel rather than clustering only hashes. Test coexistence using frequency/invasion and resource controls; transient heterogeneity is not stable diversification. Mutation events, rejected constructions and extinct lineages retain denominator membership.

**EC-3:** Test products that alter opportunities for later organisms. Predeclare candidate discovery and confirmation rules. Use producer removal with products retained, product removal with producers retained, matched material/energy replacement, product rescue and spatial controls. Record autonomous versus supplied product origins. Candidate interventions must not add free fuel or change geometry without an appropriate sham.

**EC-4:** Test cumulative chains of causally enabling changes. Freeze the required chain depth and replication criterion after calibration but before confirmation. Use time-indexed events and interventions at successive links; class-level cycles cannot be interpreted as unbounded dependency depth. Discovery can expand an assay registry, but confirmatory comparisons require a frozen registry and ascertainment procedure.

Accessible-opportunity assays state proposal distributions, effort, inclusion rules and uncertainty. A finite sampled frontier is a lower-bound observation, not proof of all reachable chemistry or absence of alternatives. Successful operator translation alone does not satisfy a multi-link ecological chain gate.

## 9. Higher-order branches — HO-1 and HO-2

For HO-1, begin with one bounded physical action per operator: for example bond formation/breaking, activation transfer, local transport or conformational change. Freeze the permitted vocabulary, material encoding, recognition radius, arity and edit budget. Small arities such as 1–3 are design candidates to calibrate, not implemented limits. Exclude arbitrary RHS graph creation, nonlocal IDs, free beads, host calls and unbounded recursion within one event.

A proposed `OperatorIR` records full semantic source structure plus hash, anchor, port/recognition constraints, action, locality/edit bounds, energy requirements and kinetic parameters derived under fixed laws. Hashes are indexes, not sole equality proofs. Cache keys include compiler/schema versions and source identities; validity covers movement, mutation, destruction, neighborhood dependencies and handle reuse. Revalidate the physically present source at commit. Material presence makes an operator **eligible**; it does not guarantee a stochastic firing.

A shared executable table is permitted as a derived optimization only when the same bounded interpreter and local eligibility are preserved. Existing epoch compilation is not sufficient: an operator must not keep acting after its material source disappears. Test destruction, relocation, partial assembly and competing substrate cases before any evolutionary claim. Generic signatures may accelerate matching, followed by exact checks; expensive eligible reactions cannot simply be skipped.

For HO-2, material descriptions have no effect without an interacting interpreter/adaptor. Test crossed description/interpreter combinations, loss and rescue, charged product construction, inherited adaptor variation and downstream product function. A fixed host lookup mapping each genome ID to a hand-written product is not an evolvable construction language. Engine code may implement fixed primitive decoding laws; the varying translation relation must be carried by material configuration.

Operator-producing operators and modifier assemblies remain multiple paid local events, not recursive execution outside time and accounting. Freeze each expanded action repertoire in a new model version. If a valid universal operator or generalist dominates, retain it as a result; only actual invariant violations are bugs to repair within the stated semantics.

## 10. Stage 5: higher-level reproduction — COL-1

Collectives require recurrent group reproduction, transmitted group organization and a group-level genealogy that handles splitting, merging and distributed contributions. Adhesion or a spatial cluster alone is insufficient. Test partner removal/rescue, spatial scramble, equal-material controls, cheaters and conflict between member and group reproduction. Separate selection among groups from shared environmental sorting and within-group selection through a prospective analysis.

Distributed construction languages are a conditional application, not a prerequisite for every collective test. A failed collective branch does not invalidate a successful simpler chemistry or justify a group fitness term that manufactures the desired outcome.

## 11. Stage 6: finite scaling and mechanism ablation — SC-1

Begin engineering scaling early, then scale each scientifically admitted claim layer under fixed laws. Vary space, supplied resources and observation horizon in a predeclared factorial or bounded panel with independent-world replication. Report all caps, extinction, audit coverage, storage and wall-clock costs. Do not vary operator radius or instruction vocabulary and call that a pure hardware scaling result.

Use a claim matrix for inherited function, ecological dependence, innovation chains, operator construction, translation and collective reproduction. Ablations require matched physical interventions and their own validity checks. Repertoire, dependency depth and closure measures carry assay/coverage definitions. Finite growth or lack of observed saturation cannot establish indefinite open-ended evolution.

## 12. First implementation contract and completion criteria

After EM-16, CC-0 should freeze only the CC-1 primitive fixture first: material/port schema, checked energy function, two reactions competing for one fuel unit, disjoint controls, exact independent reference and failure cases. No large world or evolutionary run is needed.

CC-1 is complete only when accepted changes balance, rejected changes leave physical state unchanged, stale/invalid candidates cannot commit, allocation and event provenance are explicit, and existing-profile regression checks pass. CC-2 then adds physical kinetics and durable replay before any protocell demonstration. Select campaign budgets and quantitative scientific thresholds only when their prerequisites have measured receipts.

The source attachment's SHA-256 and this adaptation's review scope are recorded in the [integrated plan](evolving_minds_constructive_chemistry_integrated_plan_20261005.md). The original attachment is preserved unchanged. This document introduces no executable feature, scientific acceptance or new exposure by itself.
