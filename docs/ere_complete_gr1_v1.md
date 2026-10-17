# Complete-profile G-R1 mutation classification: prospective contract

Date: 2026-09-27. Design version 1. Assay profile
`E2-complete-mutation-classification-v1`; encoding protocol
`E2-witness-complete-v1`, family 10. Status: preregistered design, producer and
checker not implemented, scientific run not authorized or launched. This task
authorizes writing the design, not spending the proposed compute or admitting
training. The accompanying `ere_complete_gr1_v1.json` fixes machine-readable
counts, thresholds, resources and provenance. Preserve both files after freeze;
record any subsequent authorization separately.

## 1. Question and boundaries

Does a declared, unrepaired graph-edit kernel have a measurable and navigable
mutation-to-semantics map on the complete ownership/witness representation?
Separately, does the complete reader reject deliberate completeness damage
without silently installing an incomplete family?

Historical G-R1 assays do not answer this question: their E2-v3 family uses no
complete ownership/incidence obligations. `family_parts` expects version 3 and
must not be invoked directly on version-10 material. A writer-generated valid
replacement is not a raw graph mutation. Never rewrite the mutant through the
codec, refresh obligations, supply lost witnesses, prune invalid draws, or invoke
paid repair to improve classification. Such a kernel would require a separate
prospective design and could not replace this result.

The passed full G-R3/G-R5 receipt is a basis, not evidence of mutation searchability.
This assay measures seeded parents on a fixed substrate for 100 steps; it is not
evolution, learned novelty, broad expressivity, ecological maintenance, independent
replication of the old sample stream, or optimizer signal/noise verification.

## 2. Stages and fixed parent bank

1. Freeze exact design, parent/world inventories, draw manifest, code, lockfile,
   executable and pinned prior receipts before scientific classification.
2. Run the complete damage-control matrix and audit it from the frozen archive.
   Start mutation trajectories only if native controls and the frozen stage
   auditor pass. Control failure leaves the mutation stage not started.
3. Classify exactly 10,000 unrepaired mutation draws, with cached parent traces
   and the neutral soundness checks below; no resampling or replacements.
4. Run the frozen standalone full auditor, seal all outputs and report acceptance
   independently from execution completion. A completed numerical failure is a
   scientific result; an incomplete/capped measurement is not a null result.

Use the exact 168 ordered `semantic::parent_bank()` states: seven structural
shapes × four label/guard variants × three proposal variants × two activation
states. The pinned builders and lockfile hashes are in the JSON. Enumerate their
full candidate descriptions, executable RCF/execution bytes and complete graph
inventories before steps. Genotype duplicates or executable clones are not
dropped: dormant and activated states are deliberate strata. Report the number
of unique genotype and executable identities separately; do not claim 168
distinct executable semantics.

Every parent must encode and decode through `encode_complete_family` and
`read_complete_program` with the fixed MetaLimits. If any parent fails the new
aggregate region limits, retain the failed preflight and stop; do not silently
filter the bank or increase limits. This design does not assume that successful
encoding of the old representation proves feasibility of the new one.

For each parent, use `experiments::mutation_fixture(parent.core)`: its native
LHS motif plus 64 label-0 nodes carrying property 1 = Int(100), and 96 label-0
ring edges with endpoints `i mod 64`, `(i+1) mod 64`. Allocate this world before
encoding the family so paired ordinary IDs are identical. Initial world node/
edge counts include the motif, not just the background. Both parent and child
begin on this parent's world, never on a child-specific matching fixture.

## 3. Primary mutation kernel: 10,000 draws

Uniform parent index 0..167, then uniform operator index 0..12, from a new
ChaCha8 stream seeded `0xE352310001` with the pinned rand/lockfile implementation.
For operators 0..10 draw Core/Proposal uniformly; activation operators target
the family root. Save parent, operator, role, all selector draws and engine/RNG
receipts. This is a new complete-profile stream, not an exact historical replay.

Use the historical eleven operator meanings, but select allele bodies by typed
parent payload links, not by traversing ownership edges. Pre-edit eligible lists
are sorted by stable node/edge handle (index then generation); slots additionally
sort by role. A choice uses `gen_range(0..max(1,len))`, including empty lists.
For structural operators choose variable, pattern, slot in that order, as in the
old mutator; abort if variable or pattern is absent. Then use the branch-local
selectors of the pinned old mutator on these sorted lists. Freeze the complete
draw/edit manifest before any trajectory; the producer must not choose targets
using a mutant's classification or future behavior.

| Index | Operator | One unrepaired invocation |
| --- | --- | --- |
| 0 | AddSlot | Add the first free payload slot role 0..8 on the chosen pattern, to the chosen variable; role 8 deliberately tests the arity boundary |
| 1 | RemoveSlot | Delete the chosen payload slot edge |
| 2 | RetargetSlot | Replace that payload edge with its same role and chosen variable endpoint, even if logically unchanged |
| 3 | AddVar | If fewer than eight eligible variables, create a variable with the first unused order hint 0..7 and add the first free payload slot 0..8 when available |
| 4 | RemoveVar | Delete the chosen variable and its incident edges through normal store semantics |
| 5 | AddPatternEdge | Choose an allele side; add a pattern node, one membership payload and slot-0/slot-1 payloads to selected variables |
| 6 | RemovePatternEdge | Delete the chosen pattern and its incident edges |
| 7 | AddLabel | If the chosen variable has no label payload, create a literal with Int label uniformly 0..2 and add its label payload |
| 8 | RemoveLabel | Choose the variable's attached literal and delete it with incident edges |
| 9 | ChangeLabel | Choose that literal and replace its value with Int uniformly 0..2 |
| 10 | ChangePriority | Increment the target allele's Int priority by one |
| 11 | Activate | Set family activation property 9 to Int(1) |
| 12 | Deactivate | Set family activation property 9 to Int(0) |

None of these edits changes ownership, witnesses, required counts or schema
declarations to match its desired topology. New body nodes intentionally lack
these records. Payload deletions leave surviving witnesses/obligations in place.
Native cascade deletion is retained, not compensated. Missing branch targets
are honest no-ops, with draws and applicability recorded. This transport is an
assay-only adapter, not a claim that a complete-profile evolutionary kernel is
already shipped. No production mutator is replaced by this design.

Report each draw's raw inventory change, logical topology change, metadata
change, applicability and executable change independently. An ID-only edge
replacement can remain logically neutral. Valid property edits may change
semantics without changing incidence. Strict completeness may make structural
edits frequently invalid; that is an encoding/kernel finding, not permission to
repair the trial. Dormant Proposal validity is still mandatory even when its
executable effect is neutral.

## 4. Separate completeness-damage controls: 168 × 32 = 5,376

For each parent start each attack from a fresh complete encoding, with a matched
clean sibling. This matrix is outside the 10,000-draw rate denominator; it cannot
inflate neutrality or dilute the primary invalid fraction. All rows and any
failed/unmeasured row remain in the matrix's own denominator. No advancing engine
steps, instructions, fuel, Seal preamble or repair is used in this stage.

Thirty attacks must fail direct complete decoding and install zero child world
classes. Two root-loss attacks must retire the anchored material; they are
absence/undiscovery controls, not automatic quarantine-count positives:

- Owned count: delete, mistype to Float, increment Int, duplicate key.
- Body schema declaration: delete, mistype to Float, change to wrong label,
  duplicate key.
- Required incidence obligation: delete, mistype to Float, increment Int,
  duplicate key, including explicit-zero obligations.
- Ownership edge: delete, reverse, duplicate, retarget to a foreign unowned body
  clone. The clone retains the selected body's properties but has no payload or
  witness links; original payload/witness material remains.
- Witness edge: delete, relabel to 0, reverse its two role endpoints, duplicate.
- Payload edge: delete, relabel to 0, reverse its two endpoints, duplicate.
- Body node: delete with native incident-edge cascade; relabel to 0.
- Owned orphan: add a correctly declared body clone and ownership edge, increment
  owned count, but add no consuming schema links. This deliberately coordinated
  edit tests orphan consumption, not an immutable-metadata guarantee.
- Double erasure: relabel a paired payload and witness to 0.
- Wrong encoding version: replace root encoding 10 with Int(3); activation
  mistype: replace root activation with Float. Both preserve the root handle.
- Root deletion with its incident-edge cascade; root relabel to 0.

Choose the body from the sorted owned body list at `parent_index mod body_len`.
Choose obligation key `0x220 + (parent_index mod 26)`; require aggregate coverage
of all 26 keys and at least one explicit-zero obligation. For relation attacks,
select the first supported role in the cyclic WITNESS_ROLES order starting at
`parent_index mod 13`, then its first eligible payload/witness pair by handle;
require aggregate coverage of all 13 roles. Reversed-endpoint attacks select a
non-self pair; retain an explicit inapplicable result if none exists and fail
matrix completeness rather than substitute another parent. Duplicate required
properties append a second record rather than overwriting the first.

Prime a complete-enabled GraphRuleSource on each clean sibling, apply the attack,
then request the next due epoch without advancing the Engine. Record direct
validator, discovered root, anchored root survival, classes before/after,
retirements and full before/after inventories. Root deletion/relabel must show
retirement and no cached installation. Other attacks must preserve the root,
show invalidity and quarantine, and leave no installed classes. Record actual
validator names, rather than assuming every attack reaches Members first.
Compile and read must mutate neither store nor RNG. Clean siblings must decode
exactly and install their expected, deduplicated world classes.

Literal/priority/activation value substitution is not expected to be caught as
completeness damage when topology and types remain valid; the primary cohort
characterizes that behavior. This is not protection against coordinated rewriting
of all metadata, and the matrix must not make that claim.

## 5. Execution and classification precedence

Use the actual Engine with complete-enabled GraphRuleSource, no static editor,
no instructions, no evaluator, no EI/legacy causal observer, meta visibility
false, epoch Never after the initial compile. Record complete decoding and the
initial native rule table; a static-only reconstruction cannot stand in for the
complete source path. Meta graph state is held fixed during the world trace.
Match mode MaxParallel, max pairings 1, engine seed 42, one thread. Parent and
child use identical world and RNG starts. Only the execution rules are measured;
source lifecycle differences and genotype differences are separately reported.

Classification uses the following exclusive precedence:

1. **Quarantined:** present root fails complete decoding or compilation; record
   validator and zero classes. No trajectory is needed. An absent root would be
   a separate invalid/absent outcome, never Neutral; the primary kernel does not
   delete the root, so such a primary outcome is a manifest/implementation failure.
2. **Unmeasured:** valid material, but its required parent/child trace encounters
   matcher/store/resource limits, missing evidence, or the required soundness
   comparison cannot complete. Keep it in Valid and all draw denominators; it
   blocks acceptance. Do not call a capped trajectory dead or explosive.
3. **Neutral:** valid and executable RCF-identical, with a completed parent
   baseline. Trace the first 50 such draws in sample order as soundness tripwires;
   all must match their parents. Remaining neutrals may reference the completed
   parent trace without advancing a child. Report execution bytes separately.
4. **NearNeutral:** valid, RCF-distinct, complete 100-step world hash and accepted
   world-rewrite-count vector identical to parent.
5. **Catastrophic:** valid, measured, trajectory differs and has any contiguous
   50-step window with zero accepted world rewrites, or world nodes or edges at
   any measured step exceed four times their initial count. Report frozen,
   exploding, and overlapping predicates separately but count the union once.
6. **Divergent:** valid, measured, differing trace and neither catastrophe predicate.

World hash/inventory excludes family/witness/ownership metadata; otherwise every
raw edit would trivially destroy near-neutrality. Preserve ordinary labels,
property variants, endpoint multiplicity/order, births and energies. Record full
hash separately, along with RNG before/after, native bindings/deltas, attempted/
accepted counters and work/results. Hash equality is checked with archived typed
world state/deltas, not treated as a collision-proof substitute for raw evidence.
Soundness checks additionally require paired RNG equality. Do not require equal
matching cost to declare equivalent world behavior; report cost differences.

All measured traces run the full 100 steps, including explosive ones, unless a
hard cap forces Unmeasured. No classification-exact early-out is assumed. Parent
cache has exactly 168 entries, including failed baselines. Never compare two
failed/empty traces as equal. If fewer than 50 eligible neutrals occur, soundness
coverage is incomplete and the assay cannot pass.

## 6. Numerical acceptance and reporting strata

Use exact integer comparisons, no rounded percentages. Let N be all draws in a
stratum; Valid includes every non-quarantined draw, including Unmeasured. Require
`5*Valid >= 2*N`, `10*NearNeutral >= N` where specified, and
`4*Catastrophic <= N`. Unmeasured must be zero everywhere. These preserve the old
40% Valid / 10% NearNeutral / 25% catastrophe lines; metadata attacks do not
silently lower the primary thresholds.

- Overall primary cohort: all three lines, exactly 10,000 draws.
- Structural cohort (operators 0..10): all three lines.
- Active-parent structural cohort: all three lines and at least 2,000 draws.
- Activation/deactivation cohort: Valid/catastrophe lines, at least 1,000 draws,
  at least 100 actual mode changes and 20 measured trajectory-changing mode edits.
  NearNeutral is descriptive, not a required activation threshold.
- At least 50 completed Neutral soundness checks with zero failures; both target
  roles, all 13 operators and both activation states must be represented.
- At-cap/off-cap split uses any parent Core or Proposal LHS/RHS arity == 8.
  Require both strata to occur; report fractions without a second threshold.
  This differs explicitly from the old Core-only split.
- Report by operator, target role, parent shape, parent activation, logical
  change/no-op, validator, genotype/executable identity and at-cap status.
  All disjoint partitions must reconcile; overlapping strata must be labeled.
- All 5,376 damage controls and clean siblings, coverage checks, native source
  retirement/quarantine checks and the frozen stage audit must pass independently.
- Final frozen audit must reconstruct classifications/counts/thresholds and pass
  all declared tamper controls. All source/input/output hashes must verify.

Valid below 10% indicates an unsearchable kernel/encoding pairing; do not tune
the optimizer. Valid 10–40%, or failure of another threshold, is a measured
restricted-kernel/redesign branch, not a borderline pass. No rerun to shop the
seed, no rounding a miss upward and no dropping attacks or dormant edits after
seeing outcomes. A future coherent-edit kernel or smaller operator set must
retain this archive and have its own name/design and authorization.

## 7. Proposed resource envelope, not yet approved

Classification-only pooled per-step budget: **400,000 work / 50,000 results**,
not the passed runtime's 50,000/50,000 policy. The old full classification had
peak work 357,406; that motivates a separate proposed envelope, not evidence that
the new assay will fit. No enlarged oracle, dynamic per-sample cap or cap replay
is included. Binding a cap yields Unmeasured and blocks acceptance.

| Resource | Proposed hard ceiling |
| --- | ---: |
| Primary draws / parent baselines / horizon | 10,000 / 168 / 100 |
| Advancing Engine steps | 1,016,800 = (10,000 + 168) × 100 maximum |
| Damage controls | 5,376, zero advancing steps |
| Full-store nodes / edges | 2,000 / 4,000 |
| Meta arity / vars / pattern edges | 8 / 8 / 16 |
| Region nodes / edges / radius | 128 / 256 / 5 |
| Assay wall time, including audits but excluding build/freeze | 3,600 s |
| Total archive output, including frozen source/binary | 4 GiB |
| Stage and final audit wall time combined | 900 s |
| Auditor resident memory | 1 GiB |
| Threads | 1 |

Neutral soundness child steps are already included in the 10,000-child upper
bound; no separate replay budget is hidden. Read/compile/control work, matching
cost and byte/time counters must be separately logged even though they do not
advance Engine steps. Development controls require a separately labeled bounded
smoke receipt, gate flags false, and cannot substitute for the scientific bank,
draws or horizons. Full-run approval must name these ceilings and fresh output
directory; this document grants none.

## 8. Producer, auditor and implementation acceptance

Implement isolated `crates/hg-sim-core/tests/ere_complete_gr1.rs`,
`scripts/verify_ere_complete_gr1.py` and its regression tests. Keep historical
assays, CLI behavior, mutator, defaults, admission guard and archives unchanged.
Before classification, freeze the 10,000-draw/edit manifest, 168 parent/world
inventories, 5,376 attack manifest rows, selected dependency sources, exact test
binary, design files and basis receipts. Refuse overwriting an existing directory.
Use streaming compressed canonical JSONL with per-case initial/final inventories,
raw native deltas and per-step headers; no unbounded in-memory whole-trial list.

The Python auditor must independently reconstruct typed complete material,
candidate/program canonical bytes, edit effects, projected world sequences,
classification precedence, strata and exact acceptance. It must reject unsupported
material explicitly, not fall back to Rust's top-level pass flag. RNG/hash native
generation and execution remain Rust witnesses, not a second independent engine.
The Rust producer must regenerate the frozen draw stream and check exact manifest
equality; Python must verify declared selector-to-delta rules and complete matrix
coverage. Do not claim independent Python ChaCha replay unless actually implemented
and tested. Cache keys must include full candidate data, not just a short hash.

Required development tests: deterministic draw/edit reproduction; absent-target
no-op accounting; unchanged metadata on raw structural edits; dormant invalidity;
logical versus ID-only replacement; ordered payload/witness endpoints; complete
read/compile nonmutation; root absence/retirement versus quarantine; paired world
starts; projected world versus full hash; 49/50-zero-window and exact 4× boundary;
failed baselines; budget/output/time/short-horizon refusals; stream/checksum schema;
instrumented versus plain Engine RNG/behavior parity; legacy admission rejection.

At least these 18 distinct in-memory tamper categories must be rejected: omitted
draw, duplicate draw, wrong parent/operator/role selector, hidden graph edit,
automatic metadata repair, dropped invalid dormant body, false no-op, swapped
payload endpoint, cached missing root, forged validator/class table, wrong RCF,
full-meta hash substituted for world hash, altered world delta/hash sequence,
hidden cap/unmeasured outcome, premature trace completion, changed catastrophe
boundary, falsified stratum/denominator, forged pass/training flag. Record each
actual rejection rather than counting a test invocation as evidence.

Scientific directory proposed: `output/ere_complete_gr1_full_20260927` (or a
separately recorded fresh date suffix if implementation happens later). Emit
manifest, frozen checksum inventory, control-stage audit, per-draw/parent streams,
classification and resource reports, final audit, run status and sealed output
inventory. Persist partial results and failure reason before stopping. Never
rerun automatically, delete a failed archive, relax limits, refill fuel or launch
training. A passing G-R1 leaves optimizer distribution/effect/noise prechecks and
explicit complete-profile admission still pending.
