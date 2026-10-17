# Selfmod v11 — single-mutation spectrum findings

Completed 2026-09-26. Scientific output:
`experiments/lk/output/selfmod_v11_spectrum_20260926`.
The independent accounting/gate/verdict audit passed.

**Changed, executable variants can maintain bounded function without
reproduction.** Of 180 recorded mutation draws, 111 changed executable class
(66 distinct member/class variants), 43 were executable but unchanged, and
26 were malformed. Forty-four draws, representing **25 distinct variants
across eight operators**, passed the preregistered functional criterion on
at least six of eight world seeds.

This qualifies v10's categorical interpretation that there is nothing viable
for selection to select. Its observed collapse under repeated mutation remains
unchanged: v11 tests a different, 256-step system with fission omitted. It
neither establishes long-term evolutionary viability nor identifies the cause
of v10's collapse. The next useful experiment is controlled establishment of
known functional children before introducing a probation mechanism.

## Design and completed accounting

The contract is `docs/selfmod_v11_spectrum_preregistration.md`. The panel was
frozen before scientific output: all 15 existing operators, all three `churn3`
members, four keyed parameter blocks per operator/member, eight paired
world/scheduler seeds (1,2,3,7,42,99,123,777). Each operator is applied exactly
once through its existing implementation. Inapplicable and invalid edits
remain in the panel; there is no replacement sampling or repair.

Two measurements separate opportunity from persistent function:

- **Isolated opportunities:** one object step per focal program on common
  parent reference worlds at steps0/64/128/192, with fresh creation allowance200.
  Each test starts from an unmodified reference; its effects do not carry over.
- **Replacement:** substitute one mutated member into the original three-rule
  population. Use ordinary influx, rent/starvation, decoding, object effects,
  proportional income and creation refill, explicitly omitting funded fission
  and all further mutation for256 steps.

The operating point is corrected keyed-random scheduling, match cost0,
capital60 per member, rent1, pool12, influx20/free cap500, initial world200
nodes/300 edges, creation budget200/refill1, canonical match cap64,
enumeration estimate ceiling50,000, world ceilings500 nodes/1,000 edges.
These are declared constraints, not parameters retuned after outcomes.

| Measurement | Completed evidence |
|---|---:|
| Parent/normalized faithful-clone control rollouts | 16/16 ×256 steps |
| Mutation replacement rollouts | 1,440/1,440 ×256 steps |
| Parent/clone isolated one-step tests | 192 |
| Mutant isolated attempts | 5,760 |
| Mutant isolated measured opportunities | 4,844 |
| Mutant isolated enumeration-unmeasured opportunities | 84 |
| Mutant isolated malformed/skipped opportunities | 832 |
| Actual advancing steps across all measurements | **377,856** |
| Incomplete replacement/control rollouts | **0** |

The maximum378,688 advancing steps is reduced solely by832 malformed isolated
attempts, which advance zero steps. The84 enumeration-unmeasured attempts
still execute an object phase and advance its step counter. Twenty-four
replacement trajectories also encounter enumeration skips; their productivity
rates are null and they cannot qualify. All are retained in the evidence.

All eight parent controls keep all three members alive and late-productive;
every normalized clone matches its parent exactly in initial material, fates,
capital, resource balances, world hashes and RNG probes. Every parent role has
productive reference opportunities. No baseline seed was excluded.

## Primary operator results

A functional cell requires a changed executable class, complete256 steps,
no enumeration skips, all original member roots alive at the end, focal
productivity in the last128 steps, and activity in every final32-step cycle
window. A replicated lead requires that same candidate to qualify on≥6/8 seeds.
Passing does not require outperforming the parent.

Each row contains12 recorded draws (three members ×four parameters). Lead
draws can repeat an identical decoded variant; the25 distinct variants above
are deduplicated by member and canonical class hash. Deduplication and ranking
are post-result descriptive analyses, not changes to the primary criterion.

| Operator | Changed executable draws | Malformed draws | Functional cells /96 | Replicated lead draws |
|---|---:|---:|---:|---:|
| jitter_guard | 0 | 0 | 0 | 0 |
| mutate_label | 10 | 0 | 0 | 0 |
| add_pnode_constraint | 11 | 0 | 0 | 0 |
| drop_pnode_constraint | 12 | 0 | 42 | 5 |
| add_lhs_var | 12 | 0 | 32 | 4 |
| drop_lhs_var | 0 | 12 | 0 | 0 |
| add_pedge | 12 | 0 | 0 | 0 |
| drop_pedge | 8 | 0 | 64 | 8 |
| add_rhs_create | 12 | 0 | 96 | 12 |
| drop_rhs_create | 0 | 4 | 0 | 0 |
| add_rhs_edge | 10 | 2 | 80 | 10 |
| drop_rhs_edge | 8 | 0 | 16 | 2 |
| toggle_delete | 4 | 8 | 16 | 2 |
| grow_arity | 12 | 0 | 8 | 1 |
| shrink_arity | 0 | 0 | 0 | 0 |
| **Total** | **111** | **26** | **354 /1,440** | **44** |

Interpretation follows the raw distinction, not the operator's boolean:

- Guard jitter and arity shrink are inapplicable to these guardless, binary
  parents. Their productive outcomes are inherited parent behavior, not novelty.
- Changing labels or overwriting label constraints can fire on reference
  worlds but does not sustain the tested replacement. Single-step productivity
  alone would have given a misleading continuation signal.
- All sampled LHS-variable removals are malformed. Removing R1's RHS create
  also leaves dangling references. Toggling deletion or adding RHS edges can
  invalidate references. These are representation failures, not measured
  sterile variants.
- Added pattern edges produce no isolated productive opportunity in this
  panel. Extra variables produce both functional candidates and the24
  enumeration-censored replacement cells. Those limits are not counted as
  biological failures or repaired by raising the cap.
- Additions of RHS material often preserve function, but also consume resources
  or add material not shown to be useful. Passing the viability threshold is
  not evidence of a beneficial mutation.

## The edits worth taking forward

The following ordering is an interpretation of completed v11 results, not a
preregistered optimization or a new experiment. Member indices in raw files
are0/1/2, corresponding to R1/R2/R3.

**First: prune R1's RHS edge between var0 and the created node.** Representative
`op11_member0_draw1` (draw3 is the same class) retains the required LHS edge,
creation of label2, and the RHS edge between the created node and var1. It removes
only the other emitted edge. This is a small, interpretable change that leaves
the three-member productive cycle working on every tested seed.

- Functional on8/8 seeds; all three members have productive firings in the
  last128 steps on all eight, beyond the primary focal-only requirement.
- Mean focal full-horizon rate: **1.277344**, versus parent **0.888184**
  productive firings per step; mean paired difference **+0.389160**.
- Mean paired late-half difference: **+0.551758** productive firings per step.
- Final world nodes160–175. Resource gates occur in two of eight trajectories;
  enumeration skips occur in none. The gate-bound seeds are retained.

The edit reduces emitted material per firing; native creation-spend receipts
record its actual cost, including effects of other rules. The observed rate
gain is not itself a proof of higher reproductive fitness.

**Second: remove R2's LHS pattern edge.** Representative
`op07_member1_draw0` (all four draws are the same class) retains both label
constraints, deletion of label2, and relabeling1→3. It broadens matching while
preserving the deletion/relabel role. It passes8/8; mean focal rate rises
from1.028320 to1.074707, with a late-half paired difference+0.353516. All three
members are late-productive on all seeds; final nodes147–163. Every trajectory
has resource gates elsewhere in the cycle. More matches should not be assumed
to mean greater efficiency: use the recorded fate breakdown in the birth test.

**Keep deletion removal as a stress comparison, not the first lead.**
`op12_member1_draw0` removes R2's deletion and passes8/8 with a modest rate gain,
but leaves374–431 final world nodes versus the two smaller-world examples
above. That accumulation makes a256-step pass a weak basis for long-horizon
confidence. Similarly, extra RHS creates/edges should not be prioritized just
because they produce many passing draws.

## Routing and limits

Proceed next with a **controlled child-establishment assay** using the R1 edge
prune and R2 pattern-edge removal as named variants, alongside faithful-child
and unscreened-mutation controls. Preserve the actual parent/child capital
transfer, creation budgets and native scheduling. Measure parent injury,
child productive income, sibling competition, and whole-cycle persistence,
with token-flow auditing. Pre-register the birth placement, independent seed
panel, horizon and stop limits before launching it.

That experiment separates intrinsic replacement function from costs and
competition at birth. If functional children still destabilize the population,
then a narrowly specified probation/capital-commitment experiment becomes
motivated. Do not yet rerun the broad mutation-rate sweep or replace all15
mutators. No establishment, probation, training or evolving-population probe
was launched in v11.

Limitations are material. The horizon is256; four parameter draws per member
are descriptive coverage. Of351 qualifying cells belonging to replicated
leads,294 encounter creation/world gates somewhere in the cycle. The operating
point is constrained, and a pass is not unconstrained viability. Canonical class
ordering can also change resolver rule indices after mutation, so paired
replacement trajectories include scheduling responses as well as program
effects. The isolated tests corroborate direct function on common reference
worlds. The primary criterion keeps all roots alive but does not require every
partner to be late-productive; the two prioritized variants do meet that
stronger descriptive check. Class novelty is not proof of useful new behavior.

## Engineering validation and reproduction

All23 `hg-selfmod` library tests passed, including four new tests covering
recorded-mutation/production RNG and material parity, exact faithful clones,
enumeration censoring, and malformed-root rent/starvation. The existing mutator,
object-phase semantics and production loop remain unchanged; existing source
changes add serialization, diagnostic access and module registration.

Two separate engineering preflights each completed90 mutation cells and four
control rollouts at32 steps, with3,360 advancing steps each. The second adds
raw label-count and creation-spend logging for independent reconstruction.
All137 science-bearing JSON receipts match between the preflights after
removing those added fields. Neither preflight enlarges the scientific panel.

The auditor reconstructs every scientific row's candidate identity, rent,
income rounding/order, capital, creation budgets, flux/dissipation ledger,
resource and enumeration-estimate gates, panel counts and verdicts. It also
checks reference pairing, exact clone controls, frozen binary/archive/source
hashes, and rejects seven deliberately corrupted receipts. It does not
independently reimplement native matching, decoding, effect classification or
class hashing; those remain part of the tested native instrument.

From `experiments/lk`, a fresh reproduction directory can be generated with:

```sh
python3 scripts/run_selfmod_spectrum.py output/NEW_V11_RUN
python3 scripts/audit_selfmod_spectrum.py output/NEW_V11_RUN
python3 scripts/summarize_selfmod_spectrum.py output/NEW_V11_RUN
```

The runner builds offline/locked, archives the source and executable before
evaluation, and refuses an existing output directory. Raw evidence includes
`manifest.json`, `instrument_provenance.json`, `source.tar.gz`, the executable,
`controls/`, `mutations/`, `cells/`, `summary.json`, `status.json`, and `run.log`.
`audit.json` contains reconstructed verdicts and evidence hashes;
`interpretation.json` holds explicitly post-result deduplication and ranking,
linked to the auditor and summarizer hashes. `post_analysis/` preserves the
summarizer added after the scientific instrument was frozen.

All changes and runs are confined to the isolated LK workspace. Original
`experiments/lk/provenance.json` remains the historical import record; the new
instrument's hashes are separate. Root ERE work was not modified by this assay.
