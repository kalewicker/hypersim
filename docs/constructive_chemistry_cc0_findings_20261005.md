# CC-0 chemistry compatibility specification — 2026-10-05

CC-0 is complete as a design/specification milestone. It freezes the [compatibility specification](constructive_chemistry_cc0_compatibility_spec_v1.md), reserves graph labels/properties, and supplies a [machine-readable CC-1 contract](../experiments/constructive_chemistry/contracts/cc1_atomic_fixture_v1.json). Chemical runtime execution and conservation/atomicity acceptance remain pending CC-1.

## Concrete decisions

- Reuse the current graph store and its exact in-memory clone. Add a proposed default-off checked-staging seam in `hg-engine`; keep physical validation and whole-world publication in `hg-sim-core`.
- Preserve all existing rewrite semantics. Chemical failures must reject the entire staged graph/ledger/frontier/accepted-record change; effect skipping and saturating integer arithmetic cannot supply this guarantee.
- Start with three fixed nonperiodic sites and a valence-one A–B assembly action. Activated F carries4 stored quanta; a bond stores2 and heat receives2. Conserved species include the fuel carrier. Finite activated/inactive reservoirs supply separately labelled boundary transfers.
- Use an opaque prepared commit guarded by world identity, global revision and complete exact-state encoding. Any intervening accepted commit invalidates old preparations, including disjoint ones; reprepare them. Node creation generation is not a mutation version.
- Compare rejection states including allocator free-list order, not only graph summaries. Keep all raw evidence; reversed disjoint orders compare a declared physical projection, while identical serial order under1/4 workers requires exact state/evidence equality.
- Keep CC-1 scripted, with no stochastic clock or RNG. A fixed proposal test does not claim complete chemical-channel enumeration. CC-2 owns reaction-rate semantics and durable continuation.

The schema reserves node labels `0xE2000000..0xE2000002`, relation labels `0xE2000064..0xE2000067`, and typed keys243001..243007. Ordinary hyperedge endpoints remain unordered; incidence labels carry endpoint roles. No owner/cell/genome labels gate physical actions.

## Validation scope

The [contract validator](../scripts/validate_constructive_chemistry_cc1_contract.py) checks41 unique cases/82 worker variants and14 base fixtures. Coverage comprises9 commit cases,13 preparation cases,2 generic-staging cases,5 staging faults,5 invariant-only negative cases and7 malformed-input admissions. It verifies valid initial material/energy baselines, semantic identity, placement, valence, exact integer encoding, overflow-test baselines, stated resource limits and explicit false admission flags.

The namespace scan checked1192 production source/config/rule files and found no collisions. Current manifests confirm neither proposed feature exists yet. These checks validate the contract and initial-state definitions; they do not execute reactions, build new Rust code or establish rejection correctness. No native advances, new world/seed exposure, training or scientific campaign occurred.

The [sealed CC-0 archive](../output/constructive_chemistry_cc0_20261005) contains the specification, frozen contract, validator, source snapshot, namespace/accounting receipt, roadmap snapshot and preservation receipt. The validator reproduces the complete original receipt against its frozen workspace. All51 pre-existing sealed Evolving Minds archives and11939files retain their exact inventories and SHA-256 hashes. Native source, existing laws/codecs/kernels and archived experiment contracts are unchanged.

## Next implementation boundary

Implement **CC-1** in the root workspace: default-off feature/schema, generic checked graph staging, private whole-state prepare/commit, exact closed/open ledgers, all fixed adverse cases and independent semantic reference. The contract requires every case, including failures and injected staging faults, plus appropriate shared-core/engine and existing-profile regressions. Keep scientific_acceptance, training_admitted, evolution_enabled, sustained_claim, fairness, power and launch flags explicitly false.
