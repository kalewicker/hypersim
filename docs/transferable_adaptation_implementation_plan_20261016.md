# Transferable useful adaptation: implementation plan

Date: 2026-10-16. Status updated 2026-10-17: development implementation complete
through mutation stress; stopped at its failed retention gate. No scientific
campaign launched.

Implementation update: [Gate0 is implemented](transferable_adaptation_gate0_implementation_20261016.md)
and [passes its development assay](transferable_adaptation_gate0_findings_20261016.md).
The [Gate1 temporal development preflight](transferable_adaptation_gate1_findings_20261017.md)
also passes. The [mutation-stress panel](transferable_adaptation_mutation_stress_findings_20261017.md)
is implemented and independently audited: faithful controls pass, but both mutation
kernels fail retention. This version stops; held-out transfer and discovery
remain unlaunched. The prospective plan below is retained to distinguish its
conditional later work from completed development evidence.

The user requested this plan after the closed cumulative-evolution programme.
The historical negative result and its criteria remain unchanged. This is a
separately versioned experiment, provisionally named **TA-1**. The immediate
deliverable is a bounded inheritance-and-transfer assay, followed conditionally
by discovery. Cumulative reuse is a later gate, not a prerequisite for a useful
TA-1 result. This document is not a launch contract or campaign authorization.

## 1. Question and claims

Can a heritable program change improve paid reproductive performance, survive
descendants, and retain its benefit in environments not used to choose it?

Separate four claims:

1. **Authored capability:** a supplied beneficial edit works and is inherited.
2. **Transfer:** its frozen genotype works in held-out environmental conditions.
3. **Discovery:** paid stochastic variation produces a qualifying change without
   an operator supplying its identity or choosing it using held-out results.
4. **Cumulative reuse:** an acquired capability causally supports a later one.

TA-1 targets claims 1–3. None establishes open-ended complexity, general transfer
to arbitrary tasks, or portability into arbitrary genetic backgrounds. Native
mutation at reproduction is not evidence that an organism evolved its own
mutation policy; self-modification of that policy is outside this experiment.

## 2. Existing evidence and implementation choice

Use Evolving Minds' paid developmental lifecycle rather than merge the LK,
cumulative, and ERE runners. Existing source supports paid requests, copying,
offspring endowment, development, death, ancestry, and independent replay.

- [Paid handoff findings](evolving_minds_paid_handoff_findings_20261004.md):
  an authored g103 source2-to-0 edit reaches offspring; continued SourceKernel
  variation reduces descendant harvest in all four diagnostic worlds.
- [Descendant neighborhood findings](evolving_minds_paid_descendant_neighborhood_findings_20261005.md):
  syntax/reachability failures and inherited behavioral neutrality are distinct.
- [Final cumulative findings](cumulative_scientific_campaign_findings_20261013.md):
  promising functional effects did not meet paid inheritance criteria.

Start with the existing information-dependent resource-choice task. The authored
Immediate versus Delayed program is a positive control, not a new discovery.
Do not add resource-conversion chemistry merely to implement the earlier
illustrative example. A richer task becomes necessary only for cumulative reuse.

Current constraints matter: funded configuration validates exactly 64 ticks;
the streaming cursor assumes 46 native advances per tick and a 64-tick ceiling;
handoff configuration accepts only Faithful or SourceKernel; source substitutions
operate on an authored five-instruction program. Extend through new profiles and
codecs, not by changing the meaning of old contracts or checkpoint bytes.

## 3. Experimental structure

### Gate 0 — instrumentation and attainable positive control

Implement a new TA profile with explicit environment, founding-genotype,
mutation-policy, candidate-selection, and evaluation configurations. Use exposed
development seeds only. Preserve the original B24 parent-funded endowment,
request/copy prices, upkeep, and semantic scheduler initially.

Demonstrate that the known useful edit has a measurable resource advantage AND
a reproductive advantage over its ancestor. Count completed offspring that
subsequently perform at least one paid task as the primary reproductive endpoint;
report funded requests, completed births, survival and descendant net resource
gain separately. Define ledger inclusion boundaries before execution to avoid
double-counting transfers between parents and children as new yield.

Require the useful function along a lineage through three paid parent-to-child
transmissions, with functional activity in each descendant generation. Birth,
genotype identity, and depth alone do not satisfy this requirement. Test faithful
inheritance first; mutation stress is a separate comparison.

Retain a 64-tick baseline. If it cannot express three generations, record the
reason and stop this version. A longer horizon requires a new prospective
contract and compatible codec before fresh exposure; never extend a failed run.
No regeneration of missing targets or selective rescue endowments.

### Gate 1 — authored transfer and mutation stress

Freeze two panels before evaluation:

- **Frozen transfer panel:** ancestor, authored useful genotype, exact edit
  reversion, sham intervention, and restoration; faithful copying in all arms.
- **Mutation stress panel:** the useful genotype with Faithful, original
  SourceKernel, and a new syntax-preserving source-substitution kernel.

For the new kernel, enumerate legal alternatives from grammar alone and sample
once from that set. Do not filter by resource yield, material reachability,
reproduction, future behavior, or whether the known useful site survives. An
empty set produces a declared no-edit outcome. Record candidate-set identity,
probabilities, random draws, and rejection/no-edit counts. This changes the
proposal distribution, so it is an explicit experimental intervention, not a
repair applied invisibly to the original kernel. Host grammar checking is
declared scaffolding; no claim that its computational cost is natively paid.

Environmental panels must distinguish ordinary seed generalization from a real
distribution shift. Keep cue-to-resource meaning and prices fixed, but vary the
temporal correlation of resource availability and spatial placement/access.
Use balanced schedules with equal total opportunities wherever possible. Rotating
all coordinates together is only a symmetry control, not transfer evidence.
Specify native schedule injection and independent schedule reconstruction; never
alter receipt outputs after a world runs. If access geometry cannot be varied
without changing physiology, restrict v1 to temporal transfer and label it so.

Include a cue-withheld or cue-permuted diagnostic. A capability need not succeed
after its informative relationship is destroyed; this diagnoses what transfers.
It is not pooled with the primary environment family.

### Gate 2 — discover, freeze, transplant, evaluate

Only after Gates 0–1 pass, start independent discovery populations from the
ancestor, with no supplied useful edit. The default discovery kernel is the new
syntax-preserving operator; retain original SourceKernel and faithful controls
to identify the effect of that scaffold. All populations receive identical
environment distributions, prices, and fixed resource/time budgets.

Before any run, define one deterministic candidate extraction rule using only
discovery receipts: select a changed genotype by its number of paid, functionally
active descendant transmissions, then completed active offspring, then a stable
genotype-hash tie break. Exact counting windows and lineage deduplication must be
specified in the contract. Freeze at most one candidate per discovery replicate;
no changed candidate is a retained discovery failure, not a replacement request.

Transfer genotype bytes only into standardized assay founders. Reset resource
balances, developmental state, internal signals, ancestry, location and world
state to matched initial conditions. Charge the same declared initialization in
every assay arm. This laboratory transplant does not count as a paid inherited
generation; subsequent native births must pay their own costs.

Hold mutation off throughout transfer evaluation. Compare the candidate against
its founding ancestor, sham, exact reversion(s), and restoration. For multiedit
candidates, freeze the bounded ablation procedure in advance, retaining possible
epistasis rather than choosing the most flattering single edit. Record both
whole-genotype benefit and the narrower attribution supported by interventions.

Held-out schedules, seeds and parameter cells cannot be used to rank candidates,
retune kernels, or revise thresholds. Do not silently reuse old reserved worlds;
allocate a new namespace and keep an exposure ledger. Freeze the complete panel
before opening it, and report every replicate even when discovery fails.

### Gate 3 — conditional cumulative reuse

Do not implement this until discovery transfers. The current narrow grammar
does not itself establish room for a second useful capability. First specify a
new task with a concrete intermediate product or memory state that capability A
produces and capability B can use. Demonstrate feasibility with authored A, B,
and A+B controls under full accounting, then register a separate experiment.

Compare second-acquisition success from A-bearing descendants against matched
ancestors lacking A under equal budgets. Require retained A function, new B
function, paid inheritance, and loss/restoration of the combined benefit when A
is ablated/restored. Compare maintained mutation with faithful inheritance.
Authored composition establishes assay feasibility only. Failure of TA-1 ends
this branch without building Gate 3.

## 4. Acceptance, denominators, and stopping

The final preregistration must bind a conjunctive success predicate: positive
cost-accounted benefit, positive active-offspring advantage, three paid active
descendant generations, and causal reversion/restoration on the declared transfer
family. The practical effect margin, environment coverage requirement, confidence
procedure, replicate count and caps are mandatory contract fields, not defaults
to choose after results. Authored feasibility and discovered adaptation receive
separate status flags.

Use independent discovery populations as the statistical unit for Gate 2.
Descendants, repeated censuses, and multiple assay environments are nested
observations, not additional independent discoveries. Predefine paired contrasts
and multiplicity handling. Estimate sample size from a declared useful effect and
development-only variance, or exact conservative bounds; publish a sensitivity
analysis. Do not copy the old 47-world denominator without a new rationale.

Report invalid, unreachable, no-request, extinction and horizon-censored outcomes
in the original denominator. Infrastructure failure means unresolved evidence,
not a fabricated biological zero; it prevents a positive conclusion until the
registered handling rule is satisfied. No outcome-driven retries or replacements.

Stop at the first failed gate:

| Failure | Supported conclusion |
|---|---|
| Authored edit has resource benefit but no offspring advantage | Ecological coupling is inadequate in this profile |
| Faithful inheritance fails | Lifecycle/funding/persistence is inadequate |
| Faithful passes, mutation fails | The tested variation process disrupts retention |
| Authored transfer fails | Capability is too context-specific for this declared family |
| Authored passes, discovery fails | Discovery failed within this grammar and budget |
| Discovery transfers, reuse fails | Transferable adaptation established; accumulation not established |

None establishes a universal Hypersim ceiling. Record failure before considering
any separately versioned follow-up; this plan contains no automatic retry branch.

## 5. Code work packages

All new filenames below are proposed, not existing tools.

| Package | Existing integration point | Deliverable |
|---|---|---|
| TA contract/config | `evolving_minds/funded_admission.rs`, `handoff.rs` | New `transfer.rs` config, versioned profiles, strict validation and statuses |
| Environment family | `choice.rs`, `family.rs`, funded law generation | Explicit frozen schedules and matching independent reference |
| Variation | `mutation.rs`, `d0.rs` | Separate grammar-only kernel and complete proposal receipts |
| Genotype assays | `handoff.rs`, `descendant.rs` | Arbitrary validated genotype initialization and exact interventions |
| Lifecycle evidence | `stream.rs`, `stream_output.rs`, `source_inventory.rs` | New live codec, ancestry/function accounting, frozen source provenance |
| Execution | `src/bin/evolving_minds.rs`, new `transfer_output.rs` | Explicit dry-run/register/execute modes; fresh output directories only |
| Independent audit | `scripts/evolving_minds_shared_replay.py`, handoff and descendant verifiers | New `verify_evolving_minds_transfer.py` with physical replay and denominator checks |
| Analysis | New `scripts/analyze_evolving_minds_transfer.py` | Fixed candidate extraction, paired endpoints, uncertainty and gate report |

Rust paths in the table are relative to `crates/hg-sim-core/src/`.
Add modules under the existing `evolving-minds` feature. Preserve old enum wire
representations and profile meanings; use separate policy types where needed.
The independent checker must derive outcomes from native transitions and ledgers,
not simply trust the runner's gate flags or proposal metadata.

## 6. Tests and evidence

- Config rejects incomplete contracts, overlapping discovery/evaluation identities,
  unsupported profiles and unauthorized launch states.
- Kernel tests verify candidate sets, sampling law, empty sets, and no outcome
  filtering; neutral, invalid and unreachable controls retain their meanings.
- Ledger/lineage tests catch free endowments, transfer double counting, extinct
  lineage loss and counting a transplanted founder as inherited offspring.
- Transfer tests catch copied hidden state, changing genotype during evaluation,
  training/test overlap and candidate substitution after results.
- Causal controls include exact reversion, sham and restoration; preserve costs
  for size-changing ablations or report their confounding explicitly.
- Fresh-process recovery must reproduce uninterrupted results. Worker 1/4 and
  allocation-order controls must preserve semantic outcomes. Tamper tests cover
  schedules, genotype bytes, mutation receipts, omitted worlds and source hashes.
- Run relevant existing funded/handoff/descendant/streaming regressions plus new
  tests. Candidate extraction tests use synthetic receipts with known answers.

Target Rust command during implementation:
`cargo test -p hg-sim-core --features evolving-minds --test evolving_minds`.
Run Python verifier regression modules with bytecode writing disabled. Tests use
development fixtures only; a test command must never launch the held-out panel.

## 7. Resources, artifacts, and execution sequence

Keep archival evidence on its current storage; implementing TA-1 does not require
restoring all historical outputs. See the [restoration guide](project_cleanup_20261014.md).
Build targets were removed, so a first build has nontrivial time/disk cost.

Initial engineering ceilings: 256 native nodes, 768 edges, 2 MiB checkpoints,
32 MiB per segment, 512 MiB total fixture evidence, 3,600 seconds native execution
and a separate 3,600-second audit budget. These are proposed conservative caps
informed by the old panels, not measured TA performance. Measure on development
fixtures and freeze caps for each version before native execution. Reserve build
space separately. Resource exhaustion retains a failed/incomplete receipt.

Before scientific launch, a dry run must enumerate every cell and segment and
estimate native advances, audit work, storage and total time. Scientific budgets
remain unset until sample size and environment panels are fixed; launch must
refuse unset values. Do not authorize a campaign by interpreting this plan as one.

Each evidence directory contains contract and case list, exposure ledger, source
and binary hashes, manifests, native receipts, proposal and ancestry records,
candidate-selection trace, frozen genotypes, audit results, analysis and a seal.
Publish separate execution, audit, authored-transfer, discovery-transfer and
cumulative-reuse statuses. Default all scientific success flags to false.

Implementation order:

1. Write versioned TA schema, environment specification, endpoint definitions,
   and a development-only Gate 0 contract. Freeze explicit effect/generation rules.
2. Implement configuration, schedule adapter and faithful authored assay with
   replay. Complete Gate 0 engineering verification before mutation changes.
3. Add grammar-only variation and the Gate 1 causal/transfer panels. Freeze their
   complete acceptance rules before any new evaluation exposure.
4. If these pass, implement Gate 2 extraction and genotype-only transplantation;
   prepare a reviewable preregistration, power calculation and budget for launch.
5. Execute only a separately authorized scientific campaign, seal the result,
   and stop or proceed according to its fixed gate. Gate 3 needs its own plan.

Gate1 mutation stress is now complete and failed its fixed retention gate.
There is no automatic next implementation or launch step for this version.
Held-out evaluation and discovery remain unlaunched; any follow-up needs a
separately versioned prospective contract.
