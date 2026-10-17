# Shared two-paid-edit learning: implementation and findings

Date: 2026-10-04. Profile: `E2-rule-composition-learning-v1`.
Status: **complete; independent accounting passed; fixed learning criterion passed
for all three seeds**. This is bounded mechanism-learning evidence, not full
training/scientific admission or independent physics replay.

## Outcome

The shared twelve-logit policy learns from native novelty reward on the six
off-diagonal pairs and improves on all three diagonal pairs withheld from
optimizer updates. No reserved-pair receipt enters an update. There are no
pair-specific parameters, seed selection, retries, incomplete horizons, caps,
skipped updates or source waivers in this completed trial.

| Policy seed | Initial reserved expected reward | Final reserved expected reward | Gain | Fixed criterion |
| --- | ---: | ---: | ---: | --- |
| 993748717001 | 0.023437500 | 0.076855757 | 0.053418257 | pass |
| 993748717002 | 0.023437500 | 0.188151806 | 0.164714306 | pass |
| 993748717003 | 0.023437500 | 0.075459913 | 0.052022413 | pass |

The preregistered minimum gain is 0.01 **per seed**, with every reserved pair's
correct joint-action probability above 1/16. All nine seed/pair comparisons pass:

| Reserved pair | Initial joint probability | Seed001 final | Seed002 final | Seed003 final |
| --- | ---: | ---: | ---: | ---: |
| [10,11] | 0.062500000 | 0.265640776 | 0.582317309 | 0.179769443 |
| [20,21] | 0.062500000 | 0.104928297 | 0.470885779 | 0.159072630 |
| [30,31] | 0.062500000 | 0.244276981 | 0.452011358 | 0.264837228 |

Endpoint values are exact policy expectations over all 16 native action pairs
per reserved context, enumerated separately before/after updates. They are
not sampled final-policy reward averages. Native action outcomes are unchanged;
the increase comes from learned probabilities, not an easier final world.

## Fixed budget and controls

The complete denominator is **1,206 cells × 24 = 28,944 native attempts/advances**:

- 864 training cells: three seeds × 12 batches × six development pairs × two
  samples ×two paired policies (live paid and initialized reuse).
- 288 endpoint cells: three seeds × two boundaries × three reserved pairs × 16
  action pairs. All outcomes remain in the exhaustive expectation.
- 54 correct-action controls: three seeds × three reserved pairs × six arms.

All 72 policy updates complete (36 live, 36 initialized-reuse). Reuse logits remain
exactly zero. The two slot draws are shared by the paired policies, but sampled
actions reflect their different logits. Adjacent live/reuse cells also share
the same physics seed. Policy SplitMix64 streams are independent of physics.
No native RNG probe changes during any cell.

All frozen-paid, unpaid, disabled, initialized-reuse and one-fuel controls earn
zero novelty reward. Reverse-order correct controls earn 9/24, after the second
edit and live-table installation, with no early credit. The independent native
ledger audit verifies every partial/complete class, retirement/installation,
typed target, fuel payment, rewrite attribution and pooled 50k/50k budget.

Initialized-reuse training has zero reward on all 432 cells. Its unchanged
policy, evaluated on the *live-paid* reserved action panel, has expected
reward 0.0234375; this is a policy comparator, not novelty credit awarded to
initialized reuse. The frozen-uniform policy gives the same expectation at
baseline and final. Both causal native-arm and policy comparators are retained.

Training live cells receive 59 rewarding outcomes (12/35/12 per policy seed),
each with nine credited firings. Across the entire trial there are 774 credited
ordinary firings and 1,168 paid edits. Zero-reward, partial and wrong-action
outcomes remain in every update batch denominator. Rate 4, baseline 0, denominator 12
and 12 iterations were fixed before launch; none were adjusted after observing
the results. Variation between seeds is reported, not selected away.

## Implementation and extension points

- [Learner](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/rule_composition_learning.rs):
  fixed development/reserved arrays; strict balanced update; exact reserved
  endpoint; saved pre-update receipts; paired streams; immutable batch logits;
  full schedule and completion/learning decisions. Invalid rewards are refused,
  never zero-imputed. An incomplete paired batch skips both updates and invalidates
  the campaign rather than shrinking its denominator.
- [Prospective contract](/Users/kalewicker/Documents/hypersim/docs/ere_rule_composition_learning_v1.md)
  and [config](/Users/kalewicker/Documents/hypersim/configs/ere_rule_composition_learning_v1.json):
  fixed seeds, split, update, time/step limits, controls and all-seed criteria.
- [Runner](/Users/kalewicker/Documents/hypersim/scripts/run_ere_rule_composition_learning.py):
  fresh-only archive, 223 active source pins/snapshots, prelaunch synthetic tests,
  one exact native Cargo test, corrected independent audit, tamper tests,
  feature-off check, before/after scope/preservation checks and final seal.
- [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_rule_composition_learning.py):
  reconstructs every receipt using the unchanged corrected composition V2,
  then cell schedule, paired seeds, exact draws/states, factorized probabilities,
  joint likelihood/gradients, math.fsum updates, reserved expectations and verdict.
- [Rust tests](/Users/kalewicker/Documents/hypersim/crates/hg-train/tests/ere_rule_composition_learning.rs)
  and [Python tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_rule_composition_learning.py):
  synthetic zero updates, exact positive direction, leakage/refusals, endpoint
  arithmetic, detached sampling/update/native/control/endpoint corruption and
  complete measured accounting without requiring a favorable result.

Only the new module/export, contract, runner/auditor and tests were added. Existing
native fixture, action interface/reward, one-edit learner, compiler, vocabulary
and corrected auditor bodies are unchanged. A future trial must introduce a new
prospective version, not edit this sealed numerical contract or reuse its output.

## Verification and preservation

Two Rust groups pass (prelaunch update/refusals and exact native trial). Five
Python groups pass before launch with two postlaunch groups correctly skipped;
all seven pass afterward. Tests reject 42 primary detached corruptions (20
transition, 10 endpoint/comparator, 12 native), plus four measured corruptions
and separate leakage/denominator refusals. The feature-off Cargo check passes.

The archive has **1,445 seal entries**, all verified, and all 223 active source
pins still match live sources. A separate rerun of the frozen learning auditor
uses the archive's own source snapshots without launching native physics.
Native trial execution: 136.13 seconds. Full runner: 196.862 seconds. Archive
storage: 202,338,112 bytes (about 193 MiB).

Root seal: `e6b6ef1faa22f7c2a1af0787154fc7b13a3811bc038e4f4ac9b35babc5ba262f`.
Executed binary: `b6441469ca2e8c491e6dc6d42b0593ee278f0aa812f989b9216122f68ed4c5c9`.

The original 617-entry composition attempt still has its failed audit status;
its corrected 11-entry archive-only supplement is unchanged. Of the original 212
active measurement pins, 211 remain identical and one differs solely by the
additive hg-train learner export. All seven corrected supplement pins match.
Prior 758/283/187-entry learning/adapter/lifecycle archives and the failed source
preflight retain their exact seals. Evolving Minds remains disabled, separately
scoped and unreverted. No historical archive was altered or failed panel retried.

See the [sealed manifest](/Users/kalewicker/Documents/hypersim/output/ere_rule_composition_learning_trial_20261004/manifest.json)
and [independent audit](/Users/kalewicker/Documents/hypersim/output/ere_rule_composition_learning_trial_20261004/audit.json).
To verify receipt accounting only:

```sh
python3 -B output/ere_rule_composition_learning_trial_20261004/source_snapshots/scripts/verify_ere_rule_composition_learning.py output/ere_rule_composition_learning_trial_20261004/native
```

## Interpretation and next boundary

This passes the proposed update-held-out composition test. But the representation
already supplies the factorization: slot1 category normalization reuses slot0's
category lookup row. It demonstrates native reward-driven learning and transfer
through a seeded shared interface; it does **not** establish learned relations,
an unseen category, endogenous vocabulary discovery or an autonomous grammar.
Both edits are selected before execution and their steps 1/5 timing is seeded.
Complete-class lifetimes remain right-censored at 24; no sustained H-R1,
population half-life or ecological persistence claim is made.

The next useful step is a **relation-sensitive native task and graph-conditioned
policy**, keeping this factorized policy as the comparator. Construct matched
worlds with the same per-slot category cues but different native graph relations
that require different useful edit pairs. First prospectively verify that the
task actually distinguishes them under unchanged paid-edit provenance/reward;
then authorize a small interaction-capable learner trial. That would test a
capability the current category lookup cannot represent, rather than merely
scale this successful fixture. No such extension or further campaign is launched
here. Scientific/full-training admission remains false.
