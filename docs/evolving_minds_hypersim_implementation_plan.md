# Evolving adaptive organisms in Hypersim

## Adapted implementation and research plan

**Created:** 2026-09-29. **Reviewed and revised:** 2026-10-05 (America/Phoenix).
**Status:** Research roadmap. Following the user's separate request to start implementation, the first bounded engineering fixture is implemented; see [implementation and verification](evolving_minds_engineering_implementation.md). Full H1/H2 acceptance and scientific evolution remain pending. This document alone does not authorize assays or training.  
**Source plan:** [Evolving Minds from Scratch in a Causal-Hypergraph World](/Users/kalewicker/Downloads/evolving_minds_causal_hypergraph_implementation_plan.md).  
**Target:** The current Rust Hypersim workspace at `/Users/kalewicker/Documents/hypersim`.

**2026-10-05 roadmap integration:** The [integrated Evolving Minds / constructive chemistry roadmap](evolving_minds_constructive_chemistry_integrated_plan_20261005.md) now governs future cross-profile ordering, with a [revised chemistry and higher-order technical plan](constructive_chemistry_higher_order_hypersim_plan_20261005.md). The existing contracts and completed packages below remain intact. EM-16 is now complete; CC-0 is now complete and the immediate next step is CC-1 atomic chemistry implementation. Chemistry starts as a separately versioned proposed profile, not a reinterpretation of D0 evidence.

**Engineering progress:** Paid relay development/lifecycle, exact completed-tick continuation, and isolated relay/ownership probes have persisted engineering receipts. The **authored choice-interface calibration is implemented** in `evolving-minds-choice-v1`; see [choice findings](evolving_minds_choice_findings.md). **Minimal D0 execution is implemented** in `evolving-minds-d0-development-v1`; see [D0 findings](evolving_minds_d0_findings.md). **Paid faithful D0 inheritance and turnover now have a separately priced engineering implementation** in `evolving-minds-d0-family-v1`, with full genotype copy, offspring reset and exact continuation; see [family findings](evolving_minds_family_findings.md). **One-shot source mutation and native invalid-birth escrow settlement are implemented** in `evolving-minds-d0-mutation-v1`; see [mutation findings](evolving_minds_mutation_findings.md). **Lawful full lifecycle communication controls are implemented** in `evolving-minds-d0-lifecycle-controls-v1`. A separate 64-cell birth-arbitration panel measures global-counter serialization and allocation-order bias, with a local-counter prototype; see [lifecycle and arbitration findings](evolving_minds_lifecycle_fairness_findings.md). **Full-ecology parallel admission and semantic contested-site selection are implemented** in the separate `evolving-minds-d0-admission-v1` profile; see [admission findings](evolving_minds_admission_findings.md). Broad reproductive fairness remains unaccepted. **Live-state checkpoints, append-only segmented evidence and a fixed interruption/resource pilot are implemented** in `evolving-minds-d0-streaming-v1`; see [storage findings](evolving_minds_streaming_findings.md). **The H1/H2 review and exhaustive paid source-neighborhood calibration are complete for one exposed world**; see [decision and calibration](evolving_minds_h1_h2_decision_20261003.md). The original B12 review found five-instruction newborns underfunded and no reproducing edited first child. Subsequent B24 funding and configurable admission implementations close the construction/storage prerequisites; broader opportunity/power calibration still blocks the [conditional H3a design](evolving_minds_h3a_conditional_proposal_20261003.md). See the [2026-10-01 review](evolving_minds_plan_review_2026-10-01.md).

**2026-10-05 current decision:** The [descendant behavior/class-propagation audit](evolving_minds_descendant_behavior_findings_20261005.md) completes EM-16 across all144 targets/140 substitutions. All658 states and726 lifetime records agree exactly with the prior reconstruction;9765 root/tick observations reconcile. The eight reproducing State-input edits match unchanged target-root behavior over their full observed lifetimes, despite changed internal signals. Each has three same-class paid transmission generations, still present at64 after its target dies at33; the final descendants remain right-censored. Actuator-input edits diverge and have no inherited target descendants.24 focused/metadata tests pass; all49 prior archives/11883files are preserved; zero native advances. This is finite neutrality and faithful propagation, with science/power/adaptive improvement unestablished. [CC-0 specification](constructive_chemistry_cc0_compatibility_spec_v1.md) and the fixed41-case contract are now complete. Next: CC-1 default-off schema and checked atomic-reaction implementation. Held-out worlds remain unexposed.


**Current versus proposed:** `Idle`/`BuildRelay`, nine native phases, fixed prices, lineage, full traces and the two-phase probes exist. Meaningful paid choices, generic D0 construction/variation and faithful multigeneration genotype copying now have bounded engineering implementations. One-shot source mutation, native reserve/commit/reject escrow and semantic-target/allocation continuation controls now have bounded engineering evidence. Lawful communication sham/freeze/rescue/release controls now run across full lifecycle ticks, including descendants and extinct tails. The original arbitration prototype admits both disjoint offers but retains handle-order sensitivity for competing offers. A separate full-ecology profile now admits disjoint requests concurrently, supplies a semantic lottery and settles valid/invalid escrows natively through full ticks. Broader mutation operators, per-world opportunity/power calibration, other lifecycle interventions, broad reproductive fairness and long-horizon storage scaling remain proposed. No H1/H2 research gate is marked complete by the small fixtures.

## 1. Recommended direction

Build a feature-gated experiment on Hypersim's existing graph store, typed rewrite language, matcher, conflict resolver, and artifact pipeline. Start with a small graph-native resource ecology and inherited developmental programs. Establish useful behavior and a viable reproduction economy before adding lifetime plasticity, generated rule syntax, or a large spatial world.

The first scientific question is:

> Can inherited developmental variation produce useful behavior and funded descendants beyond its frozen-founder and random-program controls? If so, how does its efficiency and search performance compare with direct encoding on the same substrate?

Memory, prediction, functional specialization, and continued innovation remain separate empirical questions. A failure at any stage is an interpretable result; there is no assumed progression toward a mind.

The original plan is a research proposal to adapt, not an instruction to execute its runs or replace Hypersim's architecture. This adaptation changes the implementation route and experimental sequencing while retaining its central interest in evolution acting on development and learning.

## 2. Changes to the original plan

| Original sections / assumption | Adaptation | Improvement |
| --- | --- | --- |
| §§4, 33–34: new C++ graph arena and simulator | Reuse `hg-core`, `hg-rules`, `hg-engine`, and `hg-sim-core` | Avoid duplicate graph identity, matching, and persistence implementations |
| §§4, 10: directed, weighted, delayed hyperedges | Represent causal connections as state-bearing nodes with explicitly labelled endpoint roles | Fits the current node-property model without silently changing ordinary edge semantics |
| §§5, 20, 47: 128×128 world and hundreds of organisms immediately | Begin with a small adjacency graph and one resource mechanism; add complexity in controlled increments | Separate substrate, ecology, and evolutionary failure causes |
| §§7–9: freely evolvable rewrite programs and generators | Begin with bounded developmental templates; introduce a new validated program schema later | Existing ERE encoding does not cover the required property effects and ownership guards |
| §§11–13, 18: metabolic coefficients can mutate | Keep physical prices and conservation laws fixed; evolve costly capacities and choices | Prevent lineages from selecting maintenance cost toward zero or creating energy |
| §§23, 46: lineage and interventions appear relatively late | Implement ancestry, full-state forks, ledgers, and intervention calibration before scientific evolution | Every interesting behavior can be traced and tested from its first appearance |
| §§23, 60: direct neural versus unrestricted hypergraph comparison | First compare direct and developmental encodings of the same graph dynamics | Separate encoding effects from changes in computational substrate |
| §§28–29: predictive state and counterfactual futures | Require observable evidence, held-out anticipation, and controlled state interventions | Remove clock, reflex, cue leakage, and inaccessible-future confounds |
| §§36–37: organism parallelism and Metal | Preserve shared-store conflicts; profile CPU execution and parallelize independent worlds first | Shared resources and rewrites make organisms interdependent |
| §§38–43: replay and SQLite | Extend existing CSV/JSONL/snapshot/manifest output; add complete checkpoints | Reuse Hypersim's durable evidence contracts; a database can remain a derived index |
| §§45, 47–49: fixed seed counts and million-birth runs | Calibrate effect sizes, run costs, censoring, and seed requirements prospectively | Avoid arbitrary expensive runs and unsupported confidence claims |
| §56: change ecology when the desired behavior fails | Diagnose and publish failure, then preregister a separate revised intervention | Prevent adaptive tuning from turning a negative result into apparent success |
| §§49, 58: remove caps and infer open-endedness | Retain host safety ceilings and test novelty across budgets and ecological opportunity | Finite sustained innovation is measurable; indefinite open-endedness is not established by a finite run |

## 3. What Hypersim already supplies—and what it does not

This is a targeted review of the source and documents linked below, not a fresh audit of every experiment archive. Current source and later findings take precedence over historical implementation plans.

### 3.1 Reusable components

| Component | Current capability | Use here |
| --- | --- | --- |
| `hg-core` | Generational node/edge handles, labels, sparse typed node properties, snapshots, deterministic hashes; opt-in metadata-isolated allocation | Shared world, organism graphs, genotype material, and stable identity |
| `hg-rules` / `hg-engine` | Pattern matching, labelled relations, guards, property effects, survivor relabels, deterministic conflict resolution | Environment, metabolism, development, signal propagation, and reproduction |
| `hg-observe` | Read-only observers, rewrite events, candidate-selection policy vocabulary | Accounting and experiment instruments; optional external-policy controls |
| `hg-alife` | Carrier genomes, motif classification, lineage and ecology observers; rule-defined biological dynamics | Reuse concepts and extend bookkeeping for multi-node organisms |
| `hg-cognition` | Candidate features and host rule-selection policies | Comparator arms and calibration tools |
| `hg-meta` / ERE integration | Bounded graph-resident rule encoding, validators, epoch compilation, quarantine, execution identity | Reuse infrastructure patterns after checking the encoding envelope |
| `hg-output`, `hg-schema`, `hg-report` | Generic output, stable results, manifests, reports, opaque snapshot payloads | Persist engineering and scientific evidence |
| `hg-sim-core` | Composition and execution; rule-source integration | Own the new runner and all engine-facing orchestration |
| `hg-app-api` / `hg-gui` | DTOs and live views | Later inspection of world, organism, and development snapshots |

Sources: [workspace manifest](/Users/kalewicker/Documents/hypersim/Cargo.toml), [architecture map](/Users/kalewicker/Documents/hypersim/docs/project_architecture_and_experiment_status.md), [graph store](/Users/kalewicker/Documents/hypersim/crates/hg-core/src/graph.rs), [ALife module](/Users/kalewicker/Documents/hypersim/crates/hg-alife/src/lib.rs), and [cognition module](/Users/kalewicker/Documents/hypersim/crates/hg-cognition/src/lib.rs).

### 3.2 Important gaps

1. **Ordinary edges have labels and endpoints, not weight, delay, or plasticity properties.** Raw/world matching treats endpoints as unordered. An opt-in compiled ordered-edge constraint exists for codec relations; it is not a general directed-edge API.
2. **Current cognition is an external selection policy.** It can inspect the supplied graph snapshot and select candidates. That is different from an organism computing from its own local sensory boundary. It cannot supply the main condition's internal cognition by assumption.
3. **The ALife genome is a small trait record, not a developmental program.** Its observer identifies a created carrier and bound parent under named replication rules. Multi-node birth, genome copying, construction, and death need explicit new contracts.
4. **Graph-resident rule compilation is global today.** `GraphRuleSource` scans rule roots and installs a shared executable table. Organism-specific rules need ownership restrictions and a validated dispatch design.
5. **ERE is not an unrestricted developmental compiler.** `CandidateRule::from_compiled` excludes property effects, property-to-property comparisons, distinctness, edge labels, schedules, survivor relabels, and ordered-edge constraints. Complete-witness encoding does not remove that world-rule envelope.
6. **Checkpoint scope is profile-specific.** Evolving Minds v1 restores graph and allocator order, engine RNG/table, clocks, ledger, lineage and accumulated evidence at completed ticks. It does not support arbitrary observers/policies, partial-phase restart or ordinary manifest continuation. A graph snapshot alone is not a resumable experiment.
7. **The original activation and Hebbian equations are not directly available.** Effects support constant scaling and property addition/subtraction, but not arbitrary property multiplication, aggregation, nonlinearities, or general delay buffers.

Sources: [matcher](/Users/kalewicker/Documents/hypersim/crates/hg-engine/src/matcher.rs), [effect vocabulary](/Users/kalewicker/Documents/hypersim/crates/hg-rules/src/effects.rs), [meta schema](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/schema.rs), [rule source](/Users/kalewicker/Documents/hypersim/crates/hg-meta/src/source.rs), and [checkpoint findings](/Users/kalewicker/Documents/hypersim/docs/ere_checkpoint_continuation_findings.md).

### 3.3 Findings that should change the design

- **ALife costs need real trade-offs.** Experiment 5b selected reduced maintenance and lost turnover in five of six tested seeds. Later variants characterize an always-satisfied threshold drift sink. Do not reopen that design by adding another mortality or cost knob; construct a new behaviorally consequential resource trade-off. See [5b findings](/Users/kalewicker/Documents/hypersim/docs/experiment5b_findings.md), [5d findings](/Users/kalewicker/Documents/hypersim/docs/experiment5d_findings.md), and [portfolio review](/Users/kalewicker/Documents/hypersim/docs/experimental_status_review_2026-09-26.md).
- **An optimizer win can be a policy-class effect.** Root cognition experiments require champion-versus-untrained comparisons. The isolated LK continuation reports modest learning on its reuse objective, while finding smaller, more repetitive graphs and weaker random-baseline evidence on one substrate. Neither reuse nor CE is a proxy for richer cognition. See [root cognition findings](/Users/kalewicker/Documents/hypersim/docs/cognition_stage45_findings.md), [LK closeout](/Users/kalewicker/Documents/hypersim/experiments/lk/docs/cognition_ephase_closeout_findings.md), and [LK E4 limits](/Users/kalewicker/Documents/hypersim/experiments/lk/docs/cognition_e4_stage8_findings.md).
- **Reification and coherent edits are not training admission.** The full root complete-profile G-R1 trial was measured and audited but failed mandatory structural strata. The 2026-09-28 64-draw pilot is historical development evidence, with numerical, scientific-launch, gate, and training flags false in that receipt. It is not a statement of the latest ERE readiness; consult the separately maintained profile status for later work. Preserve profile-specific distinctions. See [G-R1 failure](/Users/kalewicker/Documents/hypersim/docs/ere_complete_gr1_full_trial_findings.md), [current profile status](/Users/kalewicker/Documents/hypersim/docs/ere_complete_profile_gate_status.md), and [pilot status](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_draw_resource_pilot_20260928/run_status.json).
- **LK has a relevant bounded inheritance result.** V19b reports durable faithful inheritance after one externally selected further edit in eight fresh worlds. It does not show autonomous edit choice, indefinitely useful modifications, or superiority over faithful inheritance. Reuse its semantic-targeting and ancestry lessons, not its conclusion as a cognitive gate. See [v19b findings](/Users/kalewicker/Documents/hypersim/experiments/lk/docs/selfmod_v19b_second_edit_findings.md).

## 4. Architecture and repository layout

### 4.1 Main integration route

The opt-in `evolving-minds` feature and experiment-specific orchestration in `crates/hg-sim-core/src/evolving_minds/` are implemented for engineering v1. Extend them through a separately versioned developmental/ecology profile. Use separate rule/config/contract files and fresh output directories. The feature must default off, and the enabled-but-unconfigured path must preserve existing behavior.

Keep the first implementation small. Extract an engine-free `hg-development` crate only when the genome schema, compiler, and read-only analyses are substantial enough to justify it. Such a crate may depend on core/rule/observation/schema types; it must not become a second simulation engine. Engine composition remains in `hg-sim-core`. The existing separately driven `hg-selfmod` experiments remain separate.

```text
CLI / optional GUI
        |
hg-sim-core::evolving_minds             implemented engineering runner
        |
        +-- hg-engine / hg-rules        existing execution contract
        +-- hg-core                    shared world and organism store
        +-- hg-alife / observers        ancestry, ecology, accounting
        +-- developmental compiler     bounded templates first
        +-- hg-output / hg-report      durable evidence
```

### 4.2 Proposed additions

The original engineering v1 modules include `config.rs`, `runtime.rs`, `output.rs`, `interventions.rs`, and `source_inventory.rs`, with laws in `rules/evolving_minds/engineering_v1.toml`. Subsequent D0, family, admission, streaming, funding, handoff and descendant modules/profiles are indexed by the completed-package findings in section 16; the original list is not a current exhaustive inventory. The following remains a proposed decomposition, not a list of all existing runnable modules; extract modules only when new behavior needs them:

```text
crates/hg-sim-core/src/evolving_minds/
  config.rs       experiment identity, budgets, timing, arm definitions
  runner.rs       composition, phase boundaries, stop/resume behavior
  genome.rs       versioned inherited program and mutation records
  compile.rs      bounded template expansion; later validated AST compiler
  checkpoint.rs  full experiment-state persistence
  intervention.rs checkpoint forks and recorded interventions
  observer.rs    read-only construction, phenotype, ancestry, and ledger data

rules/evolving_minds/
  world_v1.toml
  physiology_v1.toml
  signals_v1.toml
  development_v1.toml

configs/evolving_minds/
  development_smoke.toml
  ecology_calibration.toml
  developmental_comparison.toml

experiments/evolving_minds/
  contracts/     versioned prospective Markdown and machine contracts
  fixtures/      hand-traced positive, negative, and confound controls

scripts/verify_evolving_minds.py        independent artifact checker
```

Do not add duplicate packages from `experiments/lk` to the root workspace or make them root path dependencies. Its preserved workspace and `scripts/lk.sh` remain an independent evidence/tool source. See [LK integration boundary](/Users/kalewicker/Documents/hypersim/docs/lk_workspace_integration.md).

### 4.3 Rules own the physics

Observers measure; policies select; neither directly mutates organism state. Environmental transfers, costs, development, plasticity, and reproduction must apply through validated rewrites. Any initialization or analysis intervention performed by the runner is explicitly tagged and excluded from autonomous behavior claims.

An expensive continuous neural side engine is optional future research, not a prerequisite. If eventually added, it requires explicit update, conflict, locality, serialization, and determinism semantics. It cannot silently write properties around the rewrite contract.

## 5. Shared graph representation

### 5.1 Organisms, world, and ownership

World sites and organisms live in one `HypergraphStore`. Each organism has a persistent birth identity, an anchor, genotype material, owned internal nodes, and explicit environment-facing ports. Host ancestry records use stable birth IDs; generational graph handles remain implementation identities.

The engineering profile has reserved labels/properties in `docs/property_registry.md`. Extend that registry for new program and lifecycle state; keep archived profile semantics fixed. Labels distinguish physical roles such as site, port, relation, and genotype material; none means brain, planner, or memory module.

Every internal rewrite must prove that its relevant nodes belong to the same organism. Environment interaction requires a specified port/site relation. A generated program cannot match arbitrary foreign internal nodes merely because their labels agree.

A first correctness fixture may bind a shared organism anchor, accepting serialization within that organism. Scaling requires a separately verified ownership scheme using local membership and immutable ownership values; it must prevent cross-organism matches without making every internal operation contend on one global root. Ownership is a boundary condition, not an evolved cognitive subsystem.

### 5.2 Directed influence as graph structure

Represent a connection as a node `c` and role-labelled incidence relations:

```text
source node -- SOURCE relation -- connection c -- TARGET relation -- target node
                                     |
                           strength / local state / parameters
```

For multiple inputs or outputs, use input/output role or slot nodes. Source and target membership is specified by topology and labels, not by interpreting an ordinary edge's endpoint order as direction. Weight, timing state, and plasticity coefficients live on `c` or related state nodes.

Persistent connection state must not be keyed only by `EdgeId`: the rewrite contract deletes matched edges and creates RHS edges afresh, even when topology is unchanged. Preserve the connection node when preserving its learned state.

Developmental rules must state which variables survive. Declare distinctness when different logical elements must be different graph nodes; matching is non-injective by default. Account for incident-edge cleanup when deleting a node.

Sources: [rewrite contract](/Users/kalewicker/Documents/hypersim/docs/rewrite_contract.md), [pattern compiler](/Users/kalewicker/Documents/hypersim/crates/hg-rules/src/compiled.rs).

### 5.3 State and identity

Begin with a small bounded signal alphabet, threshold states, and local propagation. General persistent state is allowed, but do not seed a task-specific remembered-cue circuit. Signal tokens and generic state transitions are physical primitives; task-specific storage mechanisms must evolve.

Use the existing typed property representation and tagged persistence boundary. Choose one authoritative metabolic representation for this profile—initially the ALife-compatible Float property convention—and do not confuse it with `NodeData.energy`, which is a separate `f32` field. Validate required types before execution; a skipped cost effect is an accounting failure, not a free operation.

Maintain three identities:

- exact runtime state for replay, including handles, clocks, queues, and RNG;
- executable program identity for behavioral equivalence and rule novelty;
- inherited genotype identity, including dormant material and canonical serialized bytes.

Also report a phenotype descriptor independent of incidental allocation. Canonical graph hashes alone are neither full graph-isomorphism certificates nor evidence of behavioral novelty. WL hashing excludes the properties map; activation- or weight-based analysis therefore needs an explicit state observable. See [property registry](/Users/kalewicker/Documents/hypersim/docs/property_registry.md).

## 6. Developmental genome and mutation

### 6.1 Three levels of capability

| Level | Inherited content | What it can test |
| --- | --- | --- |
| D0: bounded template program | Ordered bounded instructions, template IDs, parameters, local tags, growth limits | Evolution of development within a fixed interpreter vocabulary |
| D1: validated rule program | Typed patterns, local guards, node/property effects, explicit ownership and resource obligations | Evolution of rule syntax inside a declared language envelope |
| D2: rule-generating program | Programs that construct or modify D1 rule material | Evolution of developmental rule generation and later lifetime self-modification |

Start at D0. Fixed interpreter templates are an explicit inductive bias, and conclusions must say so. Generated phenotype nodes should arise from paid development, not a host loader instantiating an already complete controller and calling it development.

D0 avoids claiming that today's ERE can encode physiology it cannot represent. D1 needs a prospective schema/compiler profile covering the required effects and ownership semantics. It may reuse `hg-meta` region walking, validation, quarantine, and epoch-source designs, but must not widen an existing ERE profile or inherit its gate receipts by name.

### 6.2 Compilation and dispatch

Before any D1 implementation, enumerate every required operation against raw TOML and ERE capability. For each unsupported construct, either compose it from existing primitives, propose a small typed extension with tests, or defer it explicitly.

Organism-specific compilation must preserve a fixed physical rule table and scope every generated executable to its owner. Initially, owner-specific compiled instances are acceptable at small scale; measure their table/matching cost. Later sharing of semantically identical templates needs owner-safe dispatch and sampling-equivalence tests.

Bound program length, pattern size/radius, expanded rule count, construction work, and compiler work. Validation is deterministic and consumes no engine RNG. Failed programs have a recorded rejection or nonviable-birth outcome; no silent resampling, repair, or fallback controller.

### 6.3 Heredity and mutation

An offspring inherits program material and permitted parameters, then develops from a small common seed. Lifetime signal state and learned changes reset at birth in the main condition. Any inheritance of acquired state is a separately named experiment.

For D0, begin with parameter change, instruction duplication/deletion, and template substitution. Fixed prices, conservation, owner checks, conflict semantics, and host ceilings are outside the mutation vocabulary. Delay mutation-rate evolution, sexual recombination, and unbounded recursive generators.

For every mutation record: birth-attempt ID, parent and proposed child genotype bytes/hashes, operator/schema versions, stream identity and draw index, operation, sampled parameters, semantic target, changed material, validity, and cost. Resolve targets by unique semantic identity; reject missing or duplicate targets. Preserve both native material order and translated indices when allocation order affects an existing operator.

Where complete ownership records accompany an edit, use a new coherent transaction profile and retain independent raw-damage rejection controls. A host-maintained coherent mutation at birth is an experimental mechanism; it is not an organism autonomously inventing or repairing a rule.

### 6.4 Concrete D0 implementation contract

Define the choice interface in §7.1 before freezing instruction vocabulary. Start with an ordered program of at most eight instructions, bounded role/slot references and typed finite parameters. Use generic material/state creation and role-labelled wiring templates sufficient to express the authored calibration controls. Keep `BuildRelay` as a v1 regression fixture; avoid making whole useful behaviors single evolvable opcodes.

For the first D0 revision:

- Genotype graph material is authoritative; canonical bytes are a checked projection. Any decoded host cache is read-only derived state keyed by genotype/schema identity. Do not execute from a host-only program while claiming graph-resident heredity.
- A per-organism development cursor, local slot map and construction status are runtime state, not inherited learned state. IDs for mutation targets are stable within the declared genotype; graph allocation indices are never semantic targets.
- Specify instruction preconditions, successful effects, fixed costs, invalid-reference handling and insufficient-energy behavior. Each tick allows a bounded number of attempts, including failed attempts. Waiting, failed development and halting must be distinguishable. No uncharged infinite retry, implicit repair or hidden host-created phenotype.
- Initially reject multiple writers to one signal slot. A newly constructed connection cannot propagate until the declared next signal boundary. Document whether signal steps use native sequential semantics or explicit previous/next buffers; do not assume a synchronous network. Recurrence must have an explicit delay and work bound.
- Replace the v1 validator's exact relay shape with grammar invariants: ownership, legal ports and roles, reference validity, material-derived upkeep, bounded expansion and conservation. Test multiple valid phenotypes plus cross-owner and duplicate-role rejection. A larger genotype must not be forced back into the one-relay shape.
- Require at least two programs with different valid phenotypes and observable actions, exact unmutated copying across paid births, and reset of runtime state. Schema serialization alone does not finish D0.

Begin mutation calibration with one declared substitution or bounded parameter change per eligible birth attempt, followed by a separate addition of insertion/deletion. Freeze operator probabilities and the no-mutation probability. Sample once; invalid proposals remain recorded outcomes. Exhaustively inspect one-step neighbors of tiny calibration programs where feasible, and use a declared sampled panel otherwise. Record valid, neutral, behavior-changing and viable offspring fractions. If no useful phenotype is reachable from the chosen seed under the operator set, revise the seed/language contract before evolutionary runs.

## 7. World, timing, and ecological selection

### 7.1 Minimal world

Start with a small fixed adjacency graph: a ring, patch network, or modest lattice. Sites expose local resource and port contact. Position for rendering is display data; organisms do not read global coordinates or population counts.

The v1 replenished-resource fixture is useful for accounting but insufficient for informative selection: `sense_available` tests site reserve ≥8, and `action_intake` independently imposes the same resource guard. Failed requests have no separate cost. Thus a true actuator can exploit the physical guard without using the resource signal; an always-on comparator has not yet been measured. Relay-versus-idle and signal clamps do not establish the value of sensing.

Before enlarging the interpreter, freeze a small local choice interface and an authored calibration panel. A suitable candidate is two mutually exclusive resource-handling actions with opposing payoffs under locally observable resource states. An attempt must commit the same declared action/time budget and fixed attempt cost before success is resolved; a wrong choice must have an opportunity cost or declared physical loss. Preserve guards against overdraft, but do not let an unsuccessful guarded match provide a free oracle or a free attempt at the other action. Resource transfer remains native fixed physics. Movement is optional unless needed to create the trade-off.

Require inactive, always-on/constant-choice, random-choice and information-using reactive controls with matched material/energy accounting. Cross informative versus permuted/withheld cues; name the permitted observations and decision interval. Evaluate both resource states and a separately frozen test schedule. Before an evolutionary run, require the informed control to exceed the strongest uninformed control by a prospectively chosen meaningful margin and retain paid descendant viability over multiple generations. If no difference exists, revise the task physics; adding genome complexity is not a remedy. These are engineering task checks, not yet evidence of learned cognition.

Build further complexity in this order:

1. Preserve the existing one-resource lifecycle fixture as a regression profile.
2. Add the smallest resource-choice profile above, with optional paid movement only when causally needed.
3. One short-lived cue and delayed local environmental consequence.
4. Within-lifetime changes in cue relationships and stochastic timing.
5. Additional resources, hazards, signal channels, and organism interactions.

Replenishment and hazards are declared environmental physics. Cue reliability is an experimental parameter, not a reward message. The main ecology has no tournament, intelligence score, novelty bonus, or hidden reproductive subsidy.

Competition for bound resource sites uses the engine's shared-node conflict rule. Record access/win distributions and starvation. Deterministic priority or handle order can create monopolies; diagnose them before interpreting phenotype differences as selection. Any new arbitration law is a versioned change to experimental physics.

### 7.2 Explicit timescales

Define an environmental tick separately from an engine microstep. Fast computation, developmental instruction execution, structural adaptation, and reproduction may use different numbers of microsteps, all declared in the contract.

Use an explicit proposed phase schedule:

```text
environment update
  -> local sensing
  -> bounded internal signal microsteps
  -> permitted local adaptation/development
  -> action/resource contention
  -> obligatory maintenance and lifecycle settlement
  -> observation/checkpoint boundary
```

The v1 runner implements Environment → Sense → Signal → Development → Action → Maintenance → Birth → Death → Cleanup. Any new microsteps or reordered phases define a new profile; preserve this fixture as a regression rather than silently changing its clocks. Within a phase, retain native matching, conflict resolution, and sequential application. Test phase order and persist both clocks, phase position, rule-table state, and pending actions. Do not introduce cross-phase simultaneity claims or a global phase value that organisms can read as a cue shortcut.

Record eligibility at each phase: which newborns can sense, act, pay upkeep and die, and whether a parent that reaches the age limit can reproduce first. V1 permits birth before age-based death and newborn upkeep begins next tick; those are declared laws, not universal biological assumptions.

Each live node/organism must pay maintenance exactly once per declared tick. A busy computation must not starve its own cost rule, and a high-priority biological rule must not prevent aging indefinitely. If the scheduler cannot establish these invariants, the ecology is not ready for evolution.

### 7.3 Primitive interfaces

Initially permit only local signal sensing, move-to-adjacent-site, intake, and reproduction requests. Development may change internal routing and, later, the number/placement of paid ports. Interfaces enforce access limits even when the genome is malformed.

Defer attack, attachment, complex morphology, and terrain modification. Actions such as go-to-food, remember-cue, predict-hazard, or plan-path are absent. An internal-state reset during analysis is an intervention, never a normal organism action.

## 8. Energy and lifecycle contracts

### 8.1 Fixed physical prices

Computation, construction, ports, movement, plasticity, genotype storage, and offspring provisioning have nonnegative profile-fixed prices. Organisms can evolve usage and efficiency through defined mechanisms; they cannot mutate the price of existing physics downward.

Use a ledger with declared external supply and sinks:

```text
change in total accounted energy = external influx - declared dissipation
```

Transfers between world, organism reserves, construction escrow, offspring, and recyclable material cancel in the total. Define exactly whether construction stores recoverable energy or dissipates it. Do not count it twice on death. Mutation/compiler work must have separately recorded computational cost; any biological energy charge is specified rather than implied by host time.

For initial fixtures, use exactly representable quantized Float amounts where practical. For later fractional operations, predeclare a numerical error bound and independent ledger reconstruction. Avoid both bit-equality claims unsupported by arithmetic order and tolerances loose enough to hide a free-energy bug.

### 8.2 Birth as a paid process

Reproduction requests reserve parental resources, construct/copy the bounded genotype and common seed, and commit a new birth only when required payments and validity checks finish. At this small scale, a root-bound transaction may serialize the construction; optimize only after correctness.

Specify what happens if construction is interrupted: declared loss or refund of unused escrow, never an unrecorded free partial child. No graph-owning organism is counted as a completed offspring before commit. Record both attempted and committed births, gestation failures, and all parents.

Use an explicit transaction: admit request and reserve resources → sample mutation once → copy/validate the proposed genotype → construct the paid common seed → commit birth, or record failure and settle escrow. Validation failure, insufficient reserve, no vacant site and host truncation are distinct outcomes. An unsuccessful proposal is neither silently redrawn nor replaced with a viable parent clone. Define fixed request/copy charges and refund rules before running; child-size-dependent storage/construction costs follow actual copied material. Lifetime state never leaks through the copy path.

V1 matches one global `COUNTER` in every birth, which serializes births to at most one per tick even at independent sites. Before population comparisons, decouple stable identity bookkeeping from biological conflicts using a versioned native request/commit design, or explicitly retain and test that global capacity law. Do not resolve this by unrecorded host births. Add two spatially disjoint paid-birth fixtures and competing-request fixtures; permute site/allocation labels and swap equivalent founders to detect priority or handle-order advantages. Exact replay remains required, while distributional/behavioral equivalence is a separate fairness check.

Admission v1 now removes the shared counter in a separate full-ecology profile, using native per-site tick IDs and a recorded external lottery over semantic offers. It verifies disjoint and valid/invalid concurrent settlement, allocation/thread invariance and checkpoint continuation; mixed-length concurrency has a separate controlled native fixture. This closes the bounded bookkeeping/handle-order mechanism, while broad reproductive-opportunity calibration remains pending. See [admission findings](evolving_minds_admission_findings.md).

The genotype controls developmental choices. Mandatory construction and metabolic charges are fixed laws. A low reproductive threshold still cannot bypass provisioning, maturity requirements, or resource guards.

### 8.3 Death and cleanup

Death retires the organism, owned internal graph, pending signal material, unfinished construction, and genotype material according to the declared recycling law. Neighboring world nodes and other organisms survive. Shared external signals/artifacts have explicit independent ownership and expiry rules.

Verify lineage continuity through node retirement/reuse, missing parents, and interrupted births. Host birth identities never reuse an organism's identity even when graph slots recycle.

### 8.4 Boundedness and failure

Economic costs should bound typical growth. Independent host ceilings still protect work, memory, table size, output, and wall time. A hard ceiling ends or censors a run with a recorded reason; it must not silently cull the population or prune large organisms as if that were natural selection.

Report extinction, cap hits, malformed births, and incomplete cells in the original denominator. Biological invalidity and computationally unmeasured outcomes are different categories. No automatic reseeding after extinction.

## 9. Fast dynamics and lifetime learning

### 9.1 Initial dynamics

Begin with local bounded state transitions and discrete signal propagation implemented by named rules. Hand-trace one reactive circuit, one persistent-state circuit, and one recurrent circuit as instrument-positive controls. Keep those authored circuits out of evolutionary starting seeds.

Delays first arise from generic propagation chains or bounded physical timing state. If a delay primitive is introduced, distinguish supplied timing capacity from an evolved task-specific memory mechanism.

The original continuous equation is deferred. If weighted continuous computation becomes necessary, specify a typed finite operation vocabulary: multiply, accumulation, bound/nonlinearity, numerical domain, missing-property behavior, and ordering. Add only operations required by a demonstrated bottleneck. Compare the resulting substrate as a new condition.

### 9.2 Local plasticity

First support bounded local strengthening/weakening or tag/state adaptation from activity available at the matched connection. No global backpropagation, global evaluator signal, or hidden future-event access is supplied to an organism.

Implement structural plasticity separately from parameter plasticity. Add it only after parameter adaptation is causally established. Growth/pruning is paid and owner-scoped; freeze controls affect only the mechanism under test.

The Hebbian expression `Δw = η·pre·post` is a candidate future rule, not existing API behavior. Multiplication and stability bounds need implementation and calibration. Homeostasis and modulation are generic paid primitives if introduced; organism-generated modulation must be distinguished from environment-injected reward.

Research precedent exists for evolving local learning rules rather than final weights, but those systems use explicitly neural architectures and external evolutionary objectives. That supports a comparator, not a claim that Hypersim will produce equivalent results. See [Najarro and Risi, 2020](https://proceedings.neurips.cc/paper_files/paper/2020/file/ee23e7ad9b473ad072d57aaa9b2a5222-Paper.pdf).

## 10. Staged implementation and research gates

Use Hypersim-specific milestone names to distinguish these from the original plan's stages and existing cognition stages. Gates require preregistered numerical criteria before scientific execution; the criteria below define the evidence required, not invented passing thresholds.

| Milestone | Implementation / question | Required exit evidence | Stop interpretation |
| --- | --- | --- | --- |
| H0: contracts and feasibility | Operation coverage, ownership model, fixed prices, clocks, artifact schema, baseline inventory | Hand-traced design covers each operation or marks its extension/deferment; costs fit a small pilot budget | Missing primitive or unclear causal access is a design gap |
| H1: engineering substrate | Representation, phase runner, accounting, birth/death, mutation, complete checkpoints, calibrated interventions | Positive/negative fixtures, exact discrete replay, full-state resume, ledger checks, no-feature regressions | Engineering defect; no biological or cognitive conclusion |
| H2: viable ecology | Fixed simple organisms in the new economy; control that behavior changes resource yield | Sustained paid turnover, viable descendants, behavior headroom, independent accounting, uncensored measured outcomes | Ecology or phenotype interface is uninformative; design a separate revision |
| H3a: inherited developmental adaptation | D0 variation under paid ecological reproduction | Improvement beyond frozen-founder and unevolved-program controls; useful descendants in held-out worlds | No demonstrated adaptation in this language/ecology |
| H3b: encoding comparison | Direct and developmental programs on the same substrate | Matched initial phenotype/control envelope, declared mutation neighborhoods and budget sensitivities | Scoped encoding/search result, not a universal superiority claim |
| H4: lifetime adaptation | Parameter plasticity; cue relationships change within life | Experience-dependent improvement, freeze/reset/yoked controls, replication on held-out schedules | Reflex advantage, drift, or no useful learning in this condition |
| H5: memory and anticipation | Delayed hidden-cue tasks and candidate internal-state interventions | Task-valid delay, selective disruption, matched lesions, clock/external-trace controls, held-out anticipation | Persistent activity without demonstrated memory or prediction |
| H6: functional specialization | Same graph substrate for internal processes; discovery of candidate functional regions | Repeated selective causal contributions to sensing/action/metabolism across independent evolutionary worlds | Region density/modularity alone is descriptive |
| H7: rule syntax and generators | D1 then D2; generated rule material and scoped compilation | New profile's validity, differential execution, mutation, locality, cost, replay, and novelty checks | Decoder or generator capability not ready; preserve failed evidence |
| H8: lifetime rule creation | Experience changes developmental rule material | Executable semantic change, useful causal effect, rule-freeze control, paid generation, restored replay | Host edit, dormant-material novelty, or unhelpful self-modification |
| H9: ecological diversification | Interaction, morphology, persistent artifacts; then social/planning tasks | New causal mechanisms and viable lineages across budgets/contexts; transfer and interaction controls | Treadmill, plateau, external-memory dependence, or artifact accumulation |

H7 has an engineering dependency branch that may be explored with small fixtures before H6, if needed for D1. Its scientific comparison remains separately gated. There is no requirement to manufacture brain/body separation before testing a better developmental encoding.

For each milestone, publish the contract, calibration, implementation receipt, scientific findings, and a go/no-go decision. Engineering completion, scientific measurement completeness, numerical passage, and training admission are separate fields. Prior authorization for another experiment does not authorize this one.

## 11. Fair comparisons and causal evidence

### 11.1 Developmental encoding

Use the developmental, frozen and random controls at H3a; add the direct arm for H3b. Do not make a full encoding tournament a prerequisite for first testing whether the declared developmental variation is useful:

| Arm | Encoding / mechanism | Role |
| --- | --- | --- |
| Direct | Adult phenotype explicitly encoded | Same-substrate encoding comparator |
| Developmental | D0 program builds the phenotype | Main treatment |
| Frozen developmental genotype | Development runs; evolutionary mutation disabled | Separate authored scaffold from evolutionary change |
| Unevolved genotype sample | Same interpreter and declared proposal distribution, sampled without evolution | Separate search progress from capacity of the representation |
| Simple authored reactive organism | Paid local physical behavior | Establish interface usefulness and baseline headroom |

Record invalid/extinct random samples in the original denominator; a viable-conditioned view is supplementary and reports its acceptance rate. Viability rejection sampling cannot silently grant the random control extra search.

All arms share the same signal physics, ports, world law, reproduction resources, and evaluation scenarios. Main ecological reproduction is spatial/resource driven. Directly encoded organisms also pay physical adult construction costs; development pays actual instruction/execution overhead. Report those decompositions.

No single constraint makes the comparison fair in every sense. Compare matched lifetime computation/resource budgets first, and separately report genotype length, mutation opportunities, adult size, and development time. Add sensitivity analyses for matched genome size or adult capacity. Do not quietly grant development free expansion or restrict the direct arm to a smaller computational substrate.

For H3b, first pair encodings that construct the same authored adult topology and initial dynamic state. Match their construction payments while reporting instruction overhead separately. Then compare ecological evolution under explicit mutation kernels: equal mutation probability does not imply equal phenotype step size or equal access to useful neighbors. Report these neighborhoods and separate performance, robustness and compression hypotheses. Choose one primary endpoint and budget matching rule per comparison, with other matchings as named sensitivities.

A neural baseline is optional and separately labelled because it changes the substrate. Any `hg-train` optimizer-assisted development experiment is a distinct external-search condition; it cannot stand in for ecologically selected development.

### 11.2 Lifetime adaptation

The original four Signal/Action probes are diagnostic tests at tick 2. Foreign-owner material is intentionally rejected by ordinary lifecycle validation; it is not a lawful ecological lesion. The separate lifecycle-controls v1 profile now supplies a legal communication-disabled state, a native intervention boundary, washout and full 64-tick sham/freeze/rescue/release continuations, including descendants and extinct tails; see [verified scope and limits](evolving_minds_lifecycle_fairness_findings.md). Other lesion mechanisms and lifetime-adaptation claims still require their own declared duration, edit scope and energy consequences. Preserve matched-cost and natural-cost controls as separately interpreted arms.

Fork the same developed individual before informative exposure. Compare learning enabled, plasticity frozen from exposure, learned-state reset afterward, and irrelevant/yoked exposure. Add an inherited nonplastic baseline and fresh individuals from the same genotype.

Freeze-from-exposure tests acquisition; freeze-after-exposure tests expression and ongoing adaptation. A learned behavior may survive a later freeze, so the original plan's requirement that every freeze erase improvement is too strong. Charge equal dummy costs in a mechanism-isolation control, and also report the natural energy savings of disabling learning.

Measure response to reversals in the same living individual, avoiding survivor replacement as apparent learning. Keep nonstationarity inside the useful lifetime horizon, with nonzero observation/action opportunities and measured recovery time.

### 11.3 Memory and prediction

Require a task in which instantaneous allowed observations do not determine the useful action, while past observations carry information about it. Confirm this with a memoryless/reactive comparator and a calibrated memory-positive control.

Control residual cues, periodic clocks, organism age, global phase, lingering chemical traces, and host-policy snapshot leakage. Vary cue-to-consequence delays and episode phases. Separate internal memory from external environmental memory by crossing internal-state reset with external-trace removal.

For a candidate region, use a sham intervention and lesions matched for size, degree, role, baseline activity, and energy burden. Check immediate sensing, mobility, resource uptake, and viability separately. A lesion that kills an organism cannot establish selective predictive function.

Prediction requires held-out anticipation beyond current sensory information and a causal manipulation of the relevant internal state. Decode candidate state on one panel and test its contribution on another. Counterfactual state clamps should represent plausible observed states where possible; report off-manifold interventions separately.

Correct the original planning counterfactual: changing an inaccessible hidden future after an identical history creates no information for the organism. Test alternative futures through different observable evidence, or compare paired stochastic future realizations for policy robustness. Successful action sequences alone do not establish planning.

### 11.4 Analysis discipline

- The independent experimental unit is an evolutionary world/run, not nodes, descendants, windows, or many replay forks from one champion.
- Archive selected lineages prospectively; retain extinct and failed worlds. Hold selection/discovery data separate from confirmatory interventions.
- Report raw resource yield, funded descendants, survival, adaptation time, and absolute graph/rule counts alongside normalized rates.
- Treat complexity, EI/CE, motif counts, and novelty archives as diagnostics. Predefine observables, partitions, cadence, engagement counts, nulls, and unmeasured states.
- Genome compression uses canonical encoded bytes and a declared phenotype description. Include interpreter/schema identity; node-count/genome-length ratios can be inflated by redundant duplication.
- Use paired effects and uncertainty across worlds. Define the primary endpoint and observation horizon before selection: for example funded descendant output per initial founder over a fixed ecological horizon, retaining extinct founders as zero where appropriate and cap-censored worlds as unmeasured/censored rather than zero. Endpoint choice must fit the claim; density/carrying-capacity saturation can erase meaningful differences. Estimate confirmatory seed requirements from a separate pilot and meaningful minimum effects; do not assume ten or thirty seeds guarantee power.
- Freeze seed lists, endpoints, exclusions, censoring policy, and multiple-comparison handling before the corresponding scientific run. Do not tune against held-out worlds.

## 12. Replay, checkpoints, and durable evidence

### 12.1 Artifact contract

Use a fresh directory per run under a proposed `output/evolving_minds/<run_id>/` namespace. Write the normal run CSV/report/manifest and versioned auxiliary artifacts through existing generic output interfaces.

```text
manifest.json                   lifecycle, provenance, feature/profile identity
metrics.csv / report.md          ordinary run-level record
organisms.csv                    stable birth IDs, parents, lifecycle outcomes
genomes/                         immutable versioned genotype material
mutations.jsonl                  requests, semantic targets, outcomes
development.jsonl                selected construction provenance
energy.csv                       declared inputs, transfers, sinks, residuals
interventions.jsonl              checkpoints, treatments, pairing, results
snapshots/                       sampled world and organism material
checkpoints/                     complete resumable experiment state
verification.json                independently checked claims and coverage
output_sha256.json               sealed evidence inventory
```

This is a proposed schema. Prefer experiment-specific rows/files until a field is a stable cross-project metric. Schema additions require versioned readers and report propagation; CSV readers use headers, not positional column numbers. New genotype/mutation/intervention artifacts should use one explicit tagged property schema across graph state and edit records; the v1 graph-versus-edit scalar discrepancy already caused a checker failure. Preserve old readers rather than silently reinterpreting archived values. Absent measurements remain absent, not numerical zero.

Record the source plan hash, contract, actual config, rules, compiler schema, feature set, target/toolchain, root seed, streams, dirty working-tree/source hashes, stop reason, work counters, and final clocks. The current checkout has existing uncommitted changes, so commit SHA alone is insufficient provenance.

A SQLite index can be built from these artifacts later. It is not a new authority or a prerequisite for H1. Ordinary evolution here is a scientific run with paid ecological births; an optional external-search campaign follows the existing campaign artifact contract. In-memory rollouts are not citable results. See [campaign output](/Users/kalewicker/Documents/hypersim/docs/campaign_output.md).

### 12.2 Complete checkpoints

Persist the graph store and allocator free-list order/frontiers/mode, engine RNG/state, executable table and revision, compiler caches/source history, organism birth IDs and lineage, pending birth escrow, signal queues, environmental stochastic state, both clocks/phases, observers' partial windows, and output continuation positions. Unsupported state must cause an explicit checkpoint rejection.

V1 checkpoints embed every historical full graph frame and metrics row. Retain that transparent format for small fixtures; before long runs, introduce a separately versioned checkpoint containing live execution state and durable event-log offsets/hashes. Stream append-only events/accounting and periodic snapshots instead of retaining/copying the entire graph history. Fresh-directory continuation should form a manifest-linked segment chain with no duplicated or missing committed events; mid-write failure recovery must preserve the interrupted evidence and recover only the committed prefix into a fresh segment under a declared protocol. Independent replay must match the full-trace fixture before the new format is admitted.

Checkpoint at declared quiescent boundaries. Verify uninterrupted versus resumed trajectories at birth, death, mutation, cue appearance, plasticity update, compilation epoch, and partial measurement windows, including fresh-process resume. A snapshot plus reseeded RNG is insufficient.

Use one root seed and explicit documented stream derivation. A new environmental stream may be needed to pair worlds across different organism RNG consumption; persist it and declare the change from the legacy stream policy. Do not derive streams from worker scheduling, pointer addresses, or thread IDs. Common random numbers pair environmental randomness, not divergent endogenous events by fiat.

Keep legacy no-feature execution unchanged. Exact discrete state/replay checks are required within the declared same-build/platform envelope. Floating metrics use the project's documented tolerance. Do not promise cross-platform bit-identical continuous dynamics without a separate demonstration. See [reproducibility policy](/Users/kalewicker/Documents/hypersim/docs/reproducibility_policy.md).

The opt-in streaming-v1 profile implements this boundary for the fixed admission-v1 ecology: `HGEMS001` live snapshots, every-eight-tick SHA-256-bound commit markers, immutable source/child segment links, rejection of damaged published commits and independent reconstruction of complete evidence. Earlier full-history profiles retain their checkpoint formats. The 64-tick pilot closes bounded implementation coverage; future horizons still require resource calibration. See [storage findings](evolving_minds_streaming_findings.md).

### 12.3 Independent verification

The checker reconstructs ancestry, construction/payment totals, ownership, mutation targets, genotype identity, intervention pairing, measured denominators, and termination conditions from persisted evidence. Check event/ledger completeness, not only final values. Include deliberate corruption controls for omitted costs, foreign writes, duplicate IDs, stale references, wrong schema, reordered mutations, and altered seeds.

A hash seal demonstrates archive integrity, not correctness of the producer. Verification must independently recompute key conclusions. Sampled dynamic logs are acceptable only when the retained checkpoints, stochastic state, and full relevant event/accounting material suffice to reconstruct a selected lineage exactly.

## 13. Resource budget and performance plan

Start with provisional engineering bounds, not the original scientific population targets:

| Parameter | Initial proposal | Purpose |
| --- | --- | --- |
| World sites | 16–64 | Hand inspection and bounded shared-resource conflicts |
| Initial organisms | 1–8 | Verify development, descent, and interactions |
| Common seed | Approximately 4–8 physical/port nodes, excluding genotype material | Keep founder behavior simple; count bookkeeping separately |
| Internal phenotype envelope | Approximately 32 nodes / 64 relations per organism | Bounded construction and intervention fixtures |
| D0 program | Approximately 4–8 instructions | Traceable development; exact instruction sizes declared in H0 |
| Smoke horizon | Approximately 128–1,024 environmental ticks | Exercise all implemented mechanisms, not cognition claims |
| Scientific horizon / worlds | Determined after ecology and resource calibration | Cover turnover, delay, reversal, and independent variation |

These are design starting points. Accept or revise them prospectively after measuring viability and cost; do not change them midway through a scored run. Count connection/role/genotype nodes and compiler material separately so nominal phenotype size does not hide actual graph work.

Record attempted and accepted matches, enumeration work, truncation, conflict rejection by process, compiler/validator cost, memory, output, births, advances, and elapsed time. Measure complete execution and audit overhead rather than extrapolating solely from fast graph updates.

Keep ceilings for matching, expansion, live material, wall time, memory, and output. A selective post-binding guard does not necessarily reduce candidate enumeration. Every proposed optimization must preserve semantic outputs and censoring policy in differential tests.

Parallelize independent replicate worlds first. Retain canonical candidate order and native application for shared-store interactions. Use the existing candidate-collection parallel path where valid. Consider local indexing/cache improvements after profiling; defer GPU/Metal and dense continuous fields until they address a measured bottleneck and have a declared numerical/replay envelope.

## 14. Later extensions and claim limits

**Functional specialization:** ports, genotype interpretation, and metabolic laws are supplied constraints. H6 can ask whether internal computation specializes within them. Stronger body/mind emergence needs internal processes represented in the same material and selective interventions, rather than a predefined neural controller bolted onto a metabolic body.

**Morphology:** add paid physical segments, local transport, contacts, and actuators only after point/patch organisms are interpretable. Count morphology's passive storage and filtering as possible computation; do not assign all effects to an internal neural region.

**Social interaction:** add communication channels without meanings and endogenous ecological roles. Test sender/receiver benefit, cue removal, partner replacement, and held-out partners before claiming communication or social inference.

**Persistent artifacts:** distinguish resource modification, external memory, and socially transmitted information. Artifact persistence alone is not culture; require informative transmission and a receiver contribution under controls.

**Planning and learning-to-learn:** use unseen configurations/task families, faster adaptation under matched initial competence, and selective disruption of candidate computation. Improved energy accumulation, longer action chains, or a lookup strategy are insufficient alone.

**Continued innovation:** predefine executable novelty, causal usefulness, persistence, and viable mutational access. Compare fixed versus expanded ecological opportunities and multiple run/resource budgets. Report plateau and recurrent novelty separately from accumulation.

Taylor's proposed open-ended-evolution requirements include viable reproduction, accessible viable mutations, increased offspring complexity, diversity of interactions, and continued evolutionary drive. They motivate testing these bottlenecks independently; they do not guarantee open-endedness. See [Taylor, 2015](https://arxiv.org/abs/1507.07403). Finite observations support continued innovation over the tested horizon, not indefinite cognitive evolution.

## 15. Failure register and response

| Failure | First diagnosis | Permitted next step |
| --- | --- | --- |
| No viable births / extinction | Funding, construction, maintenance cadence, resource access | Archive the result; separately revise the ecology if warranted |
| Saturated immortal population | Turnover law, free efficiency direction, occupancy access | Establish a real fixed-cost trade-off in a new profile |
| Development adds redundant material | Authored template bias, mutation access, marginal behavioral contribution | Measure useful structure and encoding sensitivity |
| Plasticity never useful | Cue information, within-life timing, baseline headroom, activity engagement | Publish scoped negative; design a new task only prospectively |
| Apparent memory survives all internal resets | External traces, clock, age, host snapshot access | Test leakage/external-memory controls before making the claim |
| Candidate-region lesion harms everything | Viability, metabolism, sensing, action confounds | Add matched causal controls; withhold predictive-region claim |
| Generated rule affects another organism | Owner restrictions or compiler dispatch | Treat as correctness failure; block scientific use |
| Cheap genotype wins by deleting costs | Price exposure, skipped effects, phase starvation | Fix the physical invariant; no evolutionary success claim |
| Novelty is dormant genotype or handle order | Executable bytes, semantic targets, typed-world identity | Correct the instrument/profile and preserve prior evidence |
| Cap or enumeration truncation | Work accounting and resource envelope | Mark censoring; no silent extra budget or replacement worlds |
| CE/reuse rises as graphs shrink | Denominator effects and raw structure/performance | Report both; do not infer cognitive progress |
| A checkpoint resumes differently | Missing RNG, source, observer, phase, or pending-state data | Engineering failure; full-state resume gates long runs |

Neither failure nor promising behavior triggers an automatic rerun, training launch, ecological enrichment, or criterion relaxation.

## 16. Revised immediate implementation package (2026-10-01)

Do not expand straight from the one-opcode fixture into a broad language. Finish the following bounded work packages; each delivers a contract, focused implementation, independent receipt and explicit limitations. They refine H0–H2 without declaring those gates passed.

| Order | Package | Concrete acceptance evidence |
| --- | --- | --- |
| 1 | Choice interface and authored baselines — implemented for choice v1 | Fixed paid attempts; five controls × three cue modes × two schedules; informative yield margin and viable paid descendants; [receipt](evolving_minds_choice_findings.md). Broad ecology readiness remains pending |
| 2 | Minimal D0 semantics and execution — implemented for D0 development v1 | Bounded instructions/slots/cursor; five valid programs and three phenotype sizes; fixed costs; general ownership/material validator; buffered signal timing; exact replay; [receipt](evolving_minds_d0_findings.md). Full D0 inheritance closure remains pending |
| 3 | Reproduction economy, paid copying and one-shot source mutation/escrow — bounded implementations | [Family receipts](evolving_minds_family_findings.md) and [mutation receipts](evolving_minds_mutation_findings.md): paid copying/reset/turnover; recorded single proposals; native refund/commit/disposal; semantic targets; reversed allocation continuation; exhaustive source-edit grammar neighborhoods. Viable/useful neighborhood fractions and broader operators remain uncalibrated |
| 4 | Full lifecycle controls and bounded parallel admission implemented; broad fairness pending | [Verified findings](evolving_minds_lifecycle_fairness_findings.md): lawful sham/freeze/rescue/release over 64 full ticks; exact release continuation; 64 fixed arbitration cells. Local prototype admits both disjoint births; competing winners change under allocation reversal in 6/8 rotations for both policies. Earlier family/mutation profiles retain their global counter. [Admission v1](evolving_minds_admission_findings.md) adds a separate twelve-cell full ecology with semantic lottery, concurrent escrow, allocation/thread invariance and exact continuation. Broad fairness remains pending |
| 5 | Evidence storage and resource pilot — implemented for streaming v1 | [Storage receipts](evolving_minds_streaming_findings.md): live-state checkpoints, append-only committed segments, three baselines and six fixed interruption/recovery chains; fresh-process exact resume; independent reconstruction and bounded resource measurements. Long-horizon scaling remains unmeasured |
| 6 | H1/H2 review and paid variation calibration — bounded evidence complete; H3a design conditional | [Decision/calibration](evolving_minds_h1_h2_decision_20261003.md): all 56 source edits plus two unchanged controls, complete paid lifetimes and fixed denominators. Current ecology is no-go for H3a. [Conditional proposal](evolving_minds_h3a_conditional_proposal_20261003.md) fixes candidate arms, endpoints, world seeds, criteria and limits; funding/profile/opportunity/resource prerequisites remain |
| 7 | Parent-funded reserve calibration — bounded implementation and audit complete | [Funding findings](evolving_minds_funding_findings_20261003.md): 116 fixed cells at budgets 12/24; complete native reconstruction; B12 byte equivalence. B24 completes five-instruction construction and exposes one reproducing edit on seed 17. Configurable admission/live recovery is completed by package8; broader opportunity/power calibration remains required |
| 8 | Configurable funded semantic admission and recovery — bounded implementation and audit complete | [Funded admission findings](evolving_minds_funded_admission_findings_20261003.md): ten baselines, six interruption/recovery chains, 150 independently reconstructed published states, exact funded and legacy fresh-process continuation. Immediate B24 sustains depth5; both exposed source-kernel worlds become extinct. Opportunity/power calibration and sampled-variant comparator semantics remain prerequisites |
| 9 | Descendant endpoint and archive-only opportunity/power accounting — implemented and checked | [Accounting findings](evolving_minds_opportunity_accounting_findings_20261004.md): ten full baselines /640 ticks /29440 reconstructed passes /107 organism records; ancestry-based founder exclusion, full native lifetime/cost reconciliation, graph-derived opportunities and exact conditional sign-tail sensitivity. Zero new native execution/worlds. Retained first-request comparator semantics clarified; native comparator and matched new-world pilot remain pending |
| 10 | Retained first-admitted-request comparator and live recovery — implemented and checked | [Retained findings](evolving_minds_retained_proposal_findings_20261004.md): six fixed baselines, seven complete chains/eight segments, 72 independently reconstructed published states including fresh-process continuation; HGEMRP01 preserves bounded original samples after founder death. Repeated invalid proposals stay charged/invalid; descendants copy faithfully. Old funded continuation verified exactly. Eleven native and36 Python tests pass; all39 earlier archives preserved. Matched development opportunity/control/resource/power calibration remains next |
| 11 | Matched five-arm development calibration — complete, independently verified, negative | [Development findings](evolving_minds_matched_development_findings_20261004.md):20 fixed cells /1280ticks /58880native advances /180 published states /140 full organism records. Immediate/coin and faithful construction checks pass; world101 action-funding exposure and both inherited-improvement screens fail. Inherited−faithful differences0/-56/-96/-64; inherited−retained all0. Conservative audit projection7642.513s fails7200s ceiling. Ten decision tests pass; all42 earlier archives preserved. No threshold relaxation or held-out launch; next is archive-only diagnosis and audit optimization |
| 12 | Archive-only route diagnosis and shared replay optimization — complete and checked | [Diagnosis/optimization](evolving_minds_failure_audit_optimization_findings_20261004.md):all146 request routes reconciled; zero descendant reproduction in the eight inherited/retained cells and no authored immediate proposal among27 actual SourceKernel samples. All180 boundary states and complete lifetime/retained/final-row results agree with original audit. Reference work323840→58880passes, measured audit503.531→79.831s; unchanged resource projection now passes. Eleven tests pass; all43 earlier archives preserved; zero native execution. Ecological/power/launch status stays negative/pending. Next is a supplied paid handoff diagnostic |
| 13 | Paid supplied-edit handoff and descendant variation exposure — complete and independently checked | [Handoff findings](evolving_minds_paid_handoff_findings_20261004.md):12 fixed64tick cells /13segments /35328native advances /109 published states /113 organism records. Every faithful founder family reaches paid descendant reproduction; SourceKernel descendants make31 requests but yield less in all four worlds (mean difference-536). Unchanged/invalid/unreachable and fresh-process recovery controls pass. Original checker packaging failure is preserved; separate AST-bound continuation changes one metadata key and no physical predicate.16 native/43 Python tests pass; all44 previous archives remain unchanged. Next is archive-only descendant mutation-route/lifetime diagnosis, not scientific launch |
| 14 | Archive-only descendant mutation routes, root lifetimes and class persistence — complete and independently checked | [Route findings](evolving_minds_descendant_route_findings_20261004.md): all117 requests/113 lifetimes and109 states agree exactly with the sealed audit.31 descendant SourceKernel routes retain12 rejected,5 unreachable,10 unchanged and4 edited proposals; unchanged children all reproduce, edited children have zero observed reproduction and two censored roots. Class recurrence is distinguished from inheritance/root survival.19 tests pass; all46 prior archives preserved; zero native advances. Next is prospectively fixed exhaustive paid descendant-neighborhood calibration before operator changes or scientific launch |
| 15 | Prospectively fixed exhaustive paid descendant source neighborhood — complete and independently checked | [Calibration findings](evolving_minds_paid_descendant_neighborhood_findings_20261005.md):72 main cells on601/607, all35 substitutions plus unchanged;144 targets at17; one separate fresh-process recovery.658 states, full lifetimes and bounded two-receipt persistence match exactly.84 rejections/40 unreachable births/16 completed edits;8 edited roots reproduce only under two State-input changes, while Coin/Delayed actuator edits remain unfunded. No target censoring;6 new/16 legacy native and51 Python tests pass; all47 prior archives preserved. Next is archive-only behavioral equivalence and inherited-class propagation, with science and power unadmitted |
| 16 | Archive-only finite behavior and inherited-class propagation — complete and independently checked | [Behavior findings](evolving_minds_descendant_behavior_findings_20261005.md): all73 cases/658 states/726 lifetimes and9765 root/tick observations reconcile exactly. All140 substitutions plus4 unchanged remain retained. Eight State-input edits match full unchanged target behavior with differing internal state; each has three faithful transmission generations, alive-class presence at64 and explicit descendant censoring. Actuator edits have no target propagation.24 focused/metadata tests pass; all49 earlier archives/11883files preserved; zero native advances. CC-0 is now complete; next is CC-1 implementation; adaptive/scientific claims remain unadmitted |



Admission v1 now supplies the bounded full-ecology reproduction mechanism with separate semantic scheduling, local tick IDs, concurrent one-shot mutation settlement and conservation. Its fixed twelve-cell panel preserves the absence of simultaneous admissions in the mixed-length ecology; a controlled native fixture verifies mixed-length concurrent settlement separately. Package 5 now supplies separately versioned live checkpoints, append-only committed evidence, fresh-directory continuation and a fixed interrupted-write/output-budget pilot; see [storage findings](evolving_minds_streaming_findings.md). The H1/H2 review and paid inherited-neighborhood calibration now expose an offspring-funding barrier for five-instruction genotypes and no reproducing edited first child on seed 17. See [decision](evolving_minds_h1_h2_decision_20261003.md). The [parent-funded calibration](evolving_minds_funding_findings_20261003.md) now verifies B24 construction and one viable inherited edit within a supplied diagnostic. The [configurable funded admission package](evolving_minds_funded_admission_findings_20261003.md) now implements matched five-instruction founders, separately versioned live checkpoints, exact recovery and a fixed resource panel. The [archive-only opportunity accounting package](evolving_minds_opportunity_accounting_findings_20261004.md) now independently instruments descendant-only yield, full lifetimes/payments and conditional sign-count power for all ten exposed controls. The [retained first-admitted-request comparator](evolving_minds_retained_proposal_findings_20261004.md) now supplies the third arm, bounded original proposal persistence, repeated invalid/unreachable settlement and exact new/legacy process continuation. The [matched development pilot](evolving_minds_matched_development_findings_20261004.md) now completed all20 cells with independent accounting: ecological feasibility and the conservative campaign audit projection failed. Worlds101/103/107/109 are now exposed development evidence; held-out worlds remain unexposed. The [archive-only decomposition/shared replay package](evolving_minds_failure_audit_optimization_findings_20261004.md) now retains every checkpoint check with exact original-result agreement and a passing optimized resource projection. It finds zero descendant reproductive requests in all inherited/retained cells and no sampled authored immediate genotype. The [paid supplied-edit handoff assay](evolving_minds_paid_handoff_findings_20261004.md) now completes that boundary on fixed fresh worlds401/403/407/409. Every faithful founder family reaches paid descendant reproduction, while continued variation yields less in all four worlds. Its109-state audit passes through an explicitly recorded metadata-only repair continuation; the original failed checker and all negative routes remain sealed. The [archive-only descendant route diagnosis](evolving_minds_descendant_route_findings_20261004.md) now completes that reconstruction: every sampled parent is the authored Immediate genotype, unchanged children reproduce and the four reachable edits show no observed reproduction, with two censored tails. Genotype-class recurrence does not imply reproductive propagation. The [paid descendant-neighborhood fixture](evolving_minds_paid_descendant_neighborhood_findings_20261005.md) now completes that calibration on fixed fresh601/607:all144 targets are paid at17, all658 states pass, and all target lifetimes end before64. Eight edited roots reproduce through two State-input changes; Coin/Delayed actuator-input edits complete but never become funded parents. All35 substitutions, syntax rejections and unreachable/self-wire births remain in the fixed denominator. The [EM-16 audit](evolving_minds_descendant_behavior_findings_20261005.md) now closes that boundary: the eight fertile State-input edits are behaviorally equal on observed target lifetimes and propagate faithfully for three generations; internal state differences do not establish adaptive improvement. CC-0 has now frozen the chemistry specification; next is CC-1 implementation. Any native revision must be prospectively declared with fresh fixed diagnostic worlds; the [H3a proposal](evolving_minds_h3a_conditional_proposal_20261003.md) remains conditional and unlaunched. Broad reproductive opportunity and viable mutation neighborhoods remain to be calibrated before selection claims.

A task-interface specification comes first; its native authored controls and the minimal D0 implementation may be developed incrementally against the same frozen interface. Mutation calibration and deterministic fixtures are engineering work. Evolutionary trials remain separately scoped scientific execution. H3b is a later comparison once H3a has a useful mechanism to compare; it is not necessary to implement all D1/D2, plasticity or memory features beforehand.

The D0 development fixture is intentionally funded and is not a reproduction economy: its smallest active circuit has upkeep 16 and maximum harvest 8 per tick. Family v1 prospectively declared a separate material price table (direct-circuit upkeep 2.875), paid copying/offspring budget and full turnover. Its bounded informative fixture now demonstrates paid descendants; the original construction receipt remains intact. This engineering viability observation does not establish adaptation or scientific acceptance.

The first useful research outcome is a viable, replayable lineage whose inherited variation changes resource choices and funded descent. A larger graph, successful parser or increased raw birth count alone does not close that question.

## 17. Acceptance checklist for this plan's implementation

This is a broad research acceptance checklist, not an implementation inventory. Checked items refer only to their stated evidenced scope; unchecked items may have bounded engineering coverage in section 16 without broad research acceptance. Chemistry has separate gates in the integrated roadmap.

- [x] Existing graph identity and rewrite semantics are reused by engineering v1.
- [ ] Proposed APIs/features are distinguished from available ones.
- [ ] Developmental language coverage and ERE exclusions are explicit.
- [ ] Generated rules are owner-scoped and cannot replace physical laws.
- [ ] Obligatory costs and age cannot be skipped through conflict starvation.
- [ ] Birth/death, incomplete construction, and recycling have balanced ledgers.
- [ ] Direct/developmental arms share computational physics and declared budgets.
- [x] Engineering v1 uses native graph rules; the plan distinguishes internal computation from external selection-policy control.
- [ ] State, genome, executable-rule, and phenotype identities are separately recorded.
- [ ] Complete replay/resume and causal intervention controls precede scientific evolution.
- [ ] Failed, extinct, malformed, and censored worlds retain their original denominators.
- [ ] Complexity/EI/CE/reuse remain diagnostic rather than reproductive objectives.
- [ ] Long assays, optimizer training, and revised interventions require their own reviewed contracts and resource authorization.
- [x] Existing LK/ERE artifacts and profile-specific admission boundaries are preserved in the implemented Evolving Minds fixtures.

## 18. Evidence basis and verification scope

This adaptation examined the attached plan, workspace manifests, current graph/matcher/effect/compiler/policy/ALife/source/checkpoint code, and the linked architecture, reproducibility, output, cognition, ecology, ERE, and LK findings. Selected current root ERE manifests and the draw-local pilot status were read to distinguish complete execution from scientific or training passage. Historical archives were not independently rerun or re-audited for this planning task.

The 2026-10-01 review read the implemented configuration, lifecycle validation, checkpoint and intervention code, native sensing/intake/birth laws, and persisted findings. All 29 files in the final lifecycle archive seal and all 19 in the verified causal-control archive seal matched their recorded SHA-256 digests. This was an integrity check, not a fresh scientific assay or a repeat of every test. At that 2026-10-01 review, D0/ecology/storage changes remained proposals; subsequent implementation and evidence are recorded in section 16. No executable code, fixed contract or archived receipt was changed during this review.

The 2026-10-03 implementation completion verified lawful full lifecycle communication forks and the separate 64-cell arbitration diagnostic, including exact fresh-process Release continuation. That completion checked all 24 then-sealed Evolving Minds archives, including 17 earlier seals, against their recorded inventories and hashes. [Findings and receipts](evolving_minds_lifecycle_fairness_findings.md) distinguish the local-counter prototype from pending full-ecology fairness. Research acceptance flags remain false.

The subsequent 2026-10-03 admission milestone verified twelve full lifecycle cells, 1,024 read-only lottery cases and exact native/fresh-process continuation. Twenty selected Rust tests and fifteen adversarial checker tests passed; all 29 currently sealed Evolving Minds archives retain their hashes and inventories. [Admission findings and receipts](evolving_minds_admission_findings.md) preserve the fixed ecology, its negative mixed-length concurrency coverage and the separate controlled mechanism test. Scientific and broad reproductive-fairness acceptance remain false.


## 19. Constructive chemistry and higher-order integration — 2026-10-05

The [integrated roadmap](evolving_minds_constructive_chemistry_integrated_plan_20261005.md) adds conserved material chemistry, physical copying and compartment reproduction, then conditional niche construction, embodied operators, interpreters and collective reproduction. The [technical adaptation](constructive_chemistry_higher_order_hypersim_plan_20261005.md) identifies the required atomic-commit, kinetic, topology and provenance extensions to current Hypersim. These are proposed capabilities.

Retain D0 owner guards and paid instruction economics in their existing profiles. Chemical permeability and reproduction must instead follow local material laws; observer cell labels cannot become hidden physical permissions. Existing ERE rule installation is not sufficient evidence of a locally embodied catalyst. No historical codecs, contracts, source kernels, output archives, held-out worlds or scientific admission flags are changed by this planning integration.

EM-16 is complete; see the [behavior and propagation findings](evolving_minds_descendant_behavior_findings_20261005.md). CC-0 has now frozen the [chemistry specification](constructive_chemistry_cc0_compatibility_spec_v1.md) and41-case contract. The next bounded action is CC-1 checked local reactions before larger worlds. H3a remains conditional; later chemistry, higher-order and cognitive claims have separate prospective gates.


**CC-0 completion:** The [chemistry specification findings](constructive_chemistry_cc0_findings_20261005.md) validate a reserved schema, conservative whole-state transaction guard, exact integer conservation model and fixed prospective CC-1 fixture. This introduces no executable chemistry feature, native advances or additional scientific admission.
