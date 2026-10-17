# Two-paid-edit composition: implementation and findings

Date: 2026-10-04. Native profile: `E2-rule-composition-v1`.
Status: **native complete; corrected archive-only engineering accounting passed**.
The original frozen audit failed and remains failed. No native cell was retried.
No policy training, generalization evaluation, or scientific admission occurred.

## Outcome

Two distinct paid literal edits are necessary for the newly created proposal to
fire on ordinary structure. Either edit alone, a wrong label combination, no edit,
one fuel, frozen compilation, unpaid/disabled editing, and initialized reuse earn
zero novelty credit. Applying the two correct edits in reverse order produces
the same result without early credit.

All 396 cells completed all 24 steps: **9,504 native attempts/advances**, with no
caps, failures, partial native work or retries in the reconstructed receipts.
The full panel spans nine context pairs, all 16 live action combinations, two
new seeds, and six controls per context/seed. It records 522 paid edits / new
genotype classes and 324 credited ordinary firings.

| Condition | Cells | Paid edits | New genotype classes | Credited ordinary fires | Reward per cell |
| --- | ---: | ---: | ---: | ---: | ---: |
| Correct live pair | 18 | 36 | 36 | 162 | 9/24 |
| Other live pairs: no/partial/wrong edits | 270 | 396 | 396 | 0 | 0 |
| Frozen paid | 18 | 36 | 36 | 0 | 0 |
| Live unpaid | 18 | 0 | 0 | 0 | 0 |
| Live disabled | 18 | 0 | 0 | 0 | 0 |
| Initialized complete proposal | 18 | 0 | 0 | 0 | 0 |
| One fuel | 18 | 18 | 18 | 0 | 0 |
| Correct reverse-order pair | 18 | 36 | 36 | 162 | 9/24 |

The preseeded proposal runs 24 world rewrites per cell, including 12 initialized
proposal firings, but receives no novelty credit. All ordinary node identities/
data and physics RNG probes remain unchanged. Matching limits remain pooled
50k work / 50k results, with no cap flags.

## Timeline and class identity

For successful forward and reverse pairs:

- Step1: the first paid rewrite creates a partial class. It is installed at
  boundary2, but cannot match both ordinary label constraints and never fires.
- Step5: the second paid rewrite creates the complete class. The partial
  genotype retires at step5; its installed table entry retires at boundary6.
- Step7: the complete class first fires. Nine credited firings occur at
  7,9,11,13,15,17,19,21,23. Reward is 9/24 = 0.375.

Partial genotype spell is `[1,5)`; installed spell `[2,6)`. Complete genotype
starts5 and installation starts6; both are right-censored at boundary24, not
observed deaths. The measured edit interval is4, and the completed partial-class
lifetime is also4. No population half-life or sustained H-R1 claim is inferred.
Cases with just one edit have no measured replacement interval.

## Extension map

- [Native fixture](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere/rule_composition.rs):
  typed two-literal targets, registered pair contexts/actions, seven arms,
  paid edits at steps1/5, native epoch compilation and full graph/event recorder.
  Both action choices are supplied before execution; timing and editor semantics
  are seeded. This is composition, not learned online scheduling.
- [Shared action/reward interface](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/rule_composition.rs):
  twelve logits shared across both positions via fixed category normalization,
  factorized sampling, joint likelihood and score gradient, strict receipt-based
  reward. There are no pair-specific lookup rows or optimizer updates. Joint
  likelihood overflow and malformed/incomplete/capped inputs refuse.
- [Corrected independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_rule_composition_v2.py):
  reconstructs two target identities from cross-side pattern topology, then
  checks raw/compiled editors, complete typed node/edge ledger, budgets, epoch
  tables, canonical/execution identities, lifetimes, reward and shared-policy
  arithmetic. It does not rely on Rust's cached summary to compute credit.
- [Archive-only audit runner](/Users/kalewicker/Documents/hypersim/scripts/audit_ere_rule_composition_v2.py):
  verifies immutable native evidence and proof-only checker changes, runs Python
  accounting/tamper tests, and seals a separate supplement. It cannot launch
  Cargo tests or native physics.
- [Original native runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_rule_composition.py)
  retains its original failing checker. Do **not** use it to repeat this panel:
  verify these saved receipts with the corrected auditor. A future native
  campaign needs a prospective version using the corrected target proof.

The existing fixture, one-edit learner/reward, canonical helpers, compiler,
vocabulary and prior auditor bodies are unchanged. Shared historical active
source drift is limited to the two additive module exports. All 212 original
measurement pins still match live sources; 202 previous active learning-trial
pins are unchanged, with only those export extensions differing. Evolving Minds
remains feature-disabled and outside measurement scope, checked before/after
the supplemental audit. Unrelated changes were not reverted.

## Original checker failure and exact correction

Native tests passed all outcomes, but the first frozen independent audit failed
at its second-target proof. It incorrectly treated `E_SLOT_BASE+1` as global
variable1 across every left pattern. That role is **endpoint slot1 within each
pattern**: the left star contains `(v0,v1)` and `(v0,v2)`, so it legitimately
points to two different variables.

V2 identifies v0 as the common left-slot0 endpoint, v1 as the unique intersection
of left-slot1 and right-slot0 endpoints, and v2 as the remaining left-slot1
endpoint. It verifies both full side motifs, typed variable/literal labels and
the two E_HASLABEL target links, including the untouched third literal12.
Allocation ID, collection order and presentation hints do not determine identity.
The supplement verifies that no other native-auditor predicate changed. All
original contexts, controls, thresholds and expected rewards remain intact.

The original [development manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_composition_development_20261004/manifest.json)
remains `status:failed`, including its generic `native_work_unknown:true`. Its
native child manifest is `native_complete`, with all 396 outcomes and successful
Rust assertions. Corrected accounting establishes the known 9,504 completed
steps without rewriting that historical lifecycle status. The correction used
**zero new native steps** and is explicitly postmeasurement accounting, not
retroactive success of the original frozen checker.

## Verification / provenance

Two native Rust groups pass, covering the full fixed panel, partial/complete
lifetimes, shared policy gradients by finite differences, sampling and refusals.
The corrected supplement passes five Python groups, including a slot-identity
regression, 24 detached native tamper rejections and four policy corruptions.
A separate feature-off Cargo check passes. Native Rust execution: 42.40 seconds;
original runner including failed audit: 45.810 seconds; supplement: 19.443 seconds.
Storage: about 69 MiB native archive and 260 KiB supplement.

- Original native archive: 617 seal entries, root
  `8dff01746ecc81db6e3c3e1c3036e0de9cdf8af4050857d1150a8cbbe11ede71`.
- Corrected supplement: 11 seal entries, root
  `6c1e8fb8a746c60919ab220f75e5cd46198b762a329a981c8c1c47fdd775163c`.
- The previous 758-entry learning, 283-entry adapter and 187-entry lifecycle
  archives, plus the previous failed source preflight, retain their exact seals.
- Frozen corrected accounting reruns from the supplemental source snapshot.
  This is independent receipt accounting, not independent Engine/RNG replay.

See the [supplemental manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_composition_archive_audit_v2_20261004/manifest.json)
and [corrected full accounting](/Users/kalewicker/Documents/hypersim/output/ere_rule_composition_archive_audit_v2_20261004/audit.json).
To verify without rerunning physics:

```sh
python3 -B output/ere_rule_composition_archive_audit_v2_20261004/source_snapshots/scripts/verify_ere_rule_composition_v2.py output/ere_rule_composition_development_20261004/native
```

## Next boundary

The task and shared policy interface are ready for a separately preregistered
small learning trial: update on the six off-diagonal pairs, then evaluate the
three diagonal combinations withheld from optimizer updates. Freeze budget,
draw streams, complete denominators, endpoint comparison and causal controls
before launch. Reward must still come from native post-birth ordinary events.

All nine pairs are already exposed in this engineering panel. “Held out” would
mean withheld from optimization, not unseen by developers or a novel category.
Category normalization and edit templates are seeded; transferable composition,
autonomous grammar discovery, sustained survival, ecological persistence and
scientific/full-training admission remain unestablished. No such learning trial
was launched by this step.
