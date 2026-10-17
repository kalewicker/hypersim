# Paid rule-edit choice adapter: implementation and findings

Date: 2026-10-04. Status: **engineering complete**, not learned improvement,
scientific acceptance, sustained H-R1 novelty, or training admission.

The action/reward boundary recommended by the lifecycle fixture is implemented
and independently verified. A single prospectively fixed panel completed all
48 cells and 1,152 native attempts/advances, with no caps, failed cells, unknown
work, retries, or policy updates. Every productive paid edit receives credit;
initial-rule reuse, unpaid/disabled actions, frozen compilation, and wrong-label
edits receive none.

## Extension map

- [Native interface](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/src/ere/rule_choice.rs):
  registered contexts, actions, arms, seeds, and `run(arm, context, action, seed,
  deadline)`. Only this layer imports Engine. It constructs the fixture, installs
  one selected seeded native editor, records typed property/edge deltas and
  pre-step rule tables, and executes the fixed 24-step horizon. The host does
  not edit the graph during execution. Two disabled editor slots retain the
  existing recorder/table layout; they never execute.
- [Learning interface](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/rule_choice.rs):
  `probabilities`, `sample`, `log_probability`, `score_gradient`, and `reward`.
  Twelve f64 logits represent four choices in each of three contexts. Draws
  are explicit, so policy sampling cannot consume the physics RNG. Stable
  softmax and log-space likelihood avoid ordinary overflow/underflow hazards;
  malformed or overflowing requests refuse. No optimizer is implemented here.
- `reward` reconstructs credit from the timeline and checked initial canonical
  identities, rather than using the action's expected correctness or cached
  summary. It checks the fixed horizon, paid provenance, structural novelty,
  actual pre-step installation, and strictly post-birth ordinary firing. Reward
  is credited firings / 24. It refuses incomplete/capped/unrecognized receipts.
  This is a fixed-profile interface validator, **not** a general receipt
  authenticator: the independent auditor must also check the full native ledger
  and the archive must retain source/binary identity before research use.
- [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_rule_choice.py):
  independently reconstructs canonical/execution identities, typed ownership,
  editor raw/compiled semantics, complete node/edge changes, native pooled
  budgets, epoch installation, lifetimes, reward, and policy arithmetic. It
  reuses the previous Python primitive encoder/ledger helpers, not Rust reward
  output. This is receipt accounting, not independent Engine/RNG physics replay.
- [Runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_rule_choice_adapter.py):
  requires a fresh output path, freezes all 224 implementation inputs before
  native work, records the executable and tests, audits all cells, checks
  feature-off compilation and historical preservation, then seals the archive.
  A failure is preserved and marked failed; it is not silently retried.
- [Contract](/Users/kalewicker/Documents/hypersim/docs/ere_rule_choice_adapter_v1.md)
  and [configuration](/Users/kalewicker/Documents/hypersim/configs/ere_rule_choice_adapter_v1.json)
  freeze the exact action/context/control schedule and budget. Any added context,
  seed, horizon, optimizer, or reward objective needs a new version and contract.

The original lifecycle module, tests, configuration, preregistration, runner,
and auditor remain byte-identical. The new native fixture reuses its canonical
and class-lifetime helpers but keeps its recorder in a versioned module, rather
than changing the source of historical receipts. Shared source edits are only
the additive `rule_choice` exports in sim-core and hg-train.

## Fixed panel results

Contexts are ordinary labels 10, 20, and 30, with secondary labels 11/12. Live
actions are no-edit, write10, write20, write30; both new seeds cover every pair.
Each non-live control uses the matching edit. All rows below include both seeds.

| Condition | Cells | Paid edits | New genotype classes | Credited ordinary fires | Reward per cell |
| --- | ---: | ---: | ---: | ---: | ---: |
| Matching live paid edit | 6 | 6 | 6 | 66 | 11/24 |
| Nonmatching live paid edit | 12 | 12 | 12 | 0 | 0 |
| Live no-edit choice | 6 | 0 | 0 | 0 | 0 |
| Frozen paid edit | 6 | 6 | 6 | 0 | 0 |
| Live unpaid | 6 | 0 | 0 | 0 | 0 |
| Live disabled | 6 | 0 | 0 | 0 | 0 |
| Initialized proposal lure | 6 | 0 | 0 | 0 | 0 |

Matching edits create the class at step1, install it at boundary2, and first
fire at step3. Eleven credited firings occur at steps3,5,...,23. Nonmatching
edits are genuine structural births but cannot match the ordinary context.
Frozen edits change the genotype without installing the changed rule. The
initialized lure fires 24 world rewrites per cell, including 12 proposal
firings, but none receive novelty credit. Ordinary node identities/data and
physics RNG remain unchanged throughout.

All new genotype spells are right-censored at boundary24. The positive class
has genotype spell `[1,24)` and installed spell `[2,24)` with censoring retained,
not observed deaths. No replacement interval or population half-life was
measured. No sustained-lifetime claim is made.

## Verification and provenance

- Two Rust test groups pass: policy numerics/sampling/finite-difference score
  gradients/refusals, and complete native-panel/receipt-reward integration.
  The latter also rejects six detached interface corruptions for each of 48
  receipts, without launching additional native cells.
- Four Python groups pass, including every registered context/action/control,
  24 detached native-receipt tamper rejections, and four policy tamper rejections.
- Feature-off `hg-train` Cargo check passes. Whole runner: 19.609 seconds;
  Rust test execution: 6.33 seconds. Archive occupies about 18 MiB.
- All 224 frozen inputs match live sources. The exact 283-entry output seal
  verifies, and the frozen archived auditor independently reruns successfully.
  Root seal SHA256:
  `2d8e6c7378c9f73a3cdfee9c791c636981d8a4ddd70a1166408f8a6a4a95b5d2`.
- Executed binary SHA256:
  `e84179c22ccb6d6b9adf64fb27c18d7c340a05c889a906498c272081262da251`.
- The original lifecycle archive still verifies all 187 entries under its
  unchanged seal `0cdba8b56ae0d490c87f0ea6d6172fbce1ed27c81c18f24b37d9840e398b8bd5`.
  Its nine directly relevant fixture/helper files still match live sources.
  Of its 161 historical source pins, only `crates/hg-sim-core/src/ere.rs` now
  differs, by the additive new export. This is disclosed checkout drift;
  the old archive remains historical evidence, not a current-source certificate.

See the [authoritative lifecycle manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_choice_adapter_development_20261004/manifest.json)
and [independent panel accounting](/Users/kalewicker/Documents/hypersim/output/ere_rule_choice_adapter_development_20261004/audit.json).
To verify saved receipts without new native execution:

```sh
python3 -B output/ere_rule_choice_adapter_development_20261004/source_snapshots/scripts/verify_ere_rule_choice.py output/ere_rule_choice_adapter_development_20261004/native
```

## Interpretation and next boundary

This removes the specific interface ambiguity: a learner can now select among
paid edits, and a measurable reward distinguishes productive post-initialization
rule use from initialized proposal reuse. There is no need for another broad
gate sweep before testing that narrow learning mechanism.

Next: separately preregister a tiny, context-balanced learning experiment with
uniform initialization, fixed updates/draws/horizons, disjoint policy-sampling
streams, complete denominators, and fixed before/after comparisons plus controls.
Use native receipts rather than the known reward table for training. Its purpose
would be to test whether the selector increases productive paid-edit probability,
not to claim autonomous discovery, unseen-context generalization, sustained H-R1
acceptance, or full-project training admission. The three exposed contexts form
a deliberately simple contextual bandit with seeded editor semantics.

No such training campaign was launched. All historical scientific verdicts and
admission guards remain unchanged.
