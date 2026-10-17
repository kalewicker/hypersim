# Complete-profile coherent mutation kernel: prospective development contract

Date: 2026-09-28. Kernel profile `E2-complete-coherent-kernel-v1`, retaining
`E2-witness-complete-v1` / family 10 and the unchanged complete reader.
This is a new kernel, not a repair, rerun or promotion of the failed raw G-R1
receipt. The present task authorizes isolated implementation and bounded
development tests, not a new scientific campaign or training.

## Question and preserved counterfactual

Can explicitly typed local structural edits preserve complete ownership and
incidence while changing a rule's topology and executable semantics? The
verified raw baseline is `output/ere_complete_gr1_full_20260927`: all 5,845
logical-topology-changing draws were quarantined. That archive and its immutable
design remain unchanged. The new mechanism must not weaken the reader that
successfully rejects completeness damage.

## Implementation boundary

Implement an assay-only host transaction in
`crates/hg-sim-core/tests/support/coherent.rs`, exercised by
`crates/hg-sim-core/tests/ere_complete_coherent.rs`. It does not register a
production mutator, native ecological instruction, paid repair operation,
optimizer adapter or admission profile. It has no stochastic sampler or
scientific acceptance path in version 1 of this development task. Requests
name an operator, allele and exact typed handles; no fitness or future world
trace is consulted to select or retry a request.

Start only from complete-decodable, compilable family-10 material with explicit
writer order hints and unsealed binary payloads. Refuse damaged input before
editing; refuse sealed payloads rather than bypassing native protection or
spending/ignoring fuel. Unsupported presentation or foreign attachment is an
explicit refusal, not a request to reconstruct the material. Complete decode
and compilation are read-only pre/postconditions, never a re-encoding step.

Apply one request to a cloned store, recording every primitive node/edge/property
edit. Commit the staged store only if it passes the original complete reader
and compiler and preserves the unselected allele and activation state (except
explicit mode operators). Rejected staged material and its edit transcript
remain available in the returned receipt. A failed attempt is not silently
replaced with the parent and called Neutral. No resampling, codec regeneration,
whole-region recount, validator relaxation or bulk metadata refresh is allowed.

## Local complete-graph primitives

- Add a body: declare its actual schema label, initialize all 26 required
  incidences to zero, add one root ownership edge and increment owned count.
- Add a supported binary payload `(role,a,b)`: add its exact
  `(root,a,b)` role witness; increment only `a`'s outgoing and `b`'s incoming
  declared counter for that role. Endpoint order and repetition matter.
- Remove a payload: remove its uniquely corresponding witness and decrement
  those exact two counters before deleting body endpoints.
- Change a payload: explicitly remove the old pair and add the new pair;
  never retarget a witness while leaving its obligation behind.
- Remove a body: first remove each declared consuming relation coherently;
  delete ownership and decrement root count. Refuse deletion if foreign or
  unexpected incidence remains. Do not let native cascade silently erase
  unrelated material.

The original root, allele anchors and sides are never rebuilt. All unaffected
handles and data survive. Intermediate staged states may be incomplete, but
no Engine/source compile boundary observes them: this is an atomic host assay
operation, not evidence that a sequence of ecological rewrites has these powers.

## Operators and exact semantic cascades

| Operator | Declared coherent action |
| --- | --- |
| AddSlot | Append a supported slot to the chosen physical pattern with the chosen existing variable; arity 8 is an explicit policy rejection |
| RemoveSlot | Remove the selected pair, reindex higher slot roles coherently, then prune variables that lose their last slot reference; a resulting arity below 2 is a postcondition rejection |
| RetargetSlot | Replace the selected pair with the same role and chosen variable; prune an old variable only if it has no remaining slots; identical endpoint replacement is an honest ID-only edit |
| AddVar | Create an owned, unlabelled variable with first free hint 0..7 and append its slot to the chosen physical pattern; variable/arity caps are policy rejections |
| RemoveVar | Remove every selected-allele slot referring to that variable, reindex survivors, and remove patterns left with fewer than two slots; prune now-unused variable subtrees; empty LHS or remaining illegal guards reject the staged result |
| AddPatternEdge | Add an owned pattern to the chosen side with first free side-order hint and two supplied variable endpoints, allowing repetition; preserve existing shared patterns and multiplicity |
| RemovePatternEdge | Remove all memberships and slots of the chosen physical pattern, then ownership; prune now-unused variables; deleting a shared pattern affects both sides, and empty LHS rejects the stage |
| AddLabel | Add a declared owned literal and coherent label relation if absent; an existing label is explicitly Inapplicable |
| RemoveLabel | Remove the chosen variable's label pair; delete the literal only after its last consuming label relation disappears; absent label is Inapplicable |
| ChangeLabel | Change the existing literal's typed Int value 0..2; absent label is Inapplicable |
| ChangePriority | Increment only the chosen allele's priority within i32 bounds |
| Activate / Deactivate | Set family activation to Int(1) / Int(0), retaining complete genotype validity |

Pruning is not general repair: it is a declared deletion cascade restricted to
variables in the selected allele that the request leaves with no slot references,
and their unconsumed literal/guard bodies. Surviving variables are not relabelled,
renumbered or stripped of guards to make a failed mutation pass. If a guarded
variable remains RHS-only, the unchanged Guards validator rejects it.

Pattern slot roles remain contiguous after removal. Variable hints may have gaps;
read-order sorting supplies the ordinary candidate indices without rewriting
unaffected hints. A shared physical pattern's edit affects both memberships;
unsharing or silently rebuilding it is outside this kernel.

## Receipts and honest failure accounting

Return explicit `Applied`, `Inapplicable`, `RejectedInput`, `RejectedPolicy` or
`RejectedPostcondition` status, reason/validator, original and staged typed
inventories, primitive transcript and decoded programs when available. Caller
state stays byte-identical on every non-Applied result. World state, store clock
and RNG are not advanced by the transaction. Sampling, if added later, must be
outside this deterministic API and must archive choices before applying it.

A future campaign must keep all attempted draws. Input/policy/postcondition
rejections count as unsuccessful attempts in attempted-operator denominators,
not valid Neutral parents after rollback. Report eligibility, applicability,
actual topology/executable changes, rejected stages, dormant edits and honest
no-ops separately. The old 40% Valid / 10% NearNeutral / 25% catastrophe lines
cannot be retrospectively relaxed; any new assay and its true-topology strata,
sampling distribution, counterfactuals and ceilings need their own prospective
contract before measurement. This development contract declares no new pass.

## Development checks

Exercise all 13 operators, both allele roles, shared LHS/RHS patterns, repeated
endpoints, exact count deltas, created/deleted bodies, slot compaction, orphan
pruning, i32/arity/variable/region boundaries, invalid semantic postconditions,
foreign handles/attachments, sealed input and deterministic replay. Compare
known successful cases to independently specified candidate semantics and to
fresh codec output only as a test oracle, never as the committed mutant.

Verify unchanged world/unselected allele/root identity and atomic rollback;
probe loss/duplication/mistyping of metadata on edited families and ensure it
still fails the unchanged complete reader and is not healed by this kernel.
Use the existing raw harness tests and prior sealed scientific receipt as
regression evidence, without modifying either. Short actual complete-source
Engine tests may verify semantic effects/dormancy, but are development controls,
not a full G-R1 or optimizer campaign. New frozen sampler/assay auditing and
scientific resource approval remain subsequent work.
