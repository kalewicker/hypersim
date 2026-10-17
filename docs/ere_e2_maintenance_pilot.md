# E2 maintenance-and-reuse pilot v1

Preregistered 2026-09-26 before implementation tests or new research rollouts.
User-authorized follow-up to the E2 grammar pilot: improve maintenance measurement
and test a joint objective. This is a new experiment, not a retry of failed
historical receipts. Full training remains outside this task.

## Hypothesis and intervention

The previous reuse-only objective retained positive fitness while seeded encoded
families were invalid. Keep the same 40-logit, four-family E2 grammar and actual
unprotected A3 rewrite semantics; change the objective and instrumentation only.
No hidden meta exclusion, label compartment, family repair, cached-body fallback,
energy replenishment, catalogue tuning or runtime-budget increase is allowed.
Protected A2 is a measurement control, not evidence of learned self-maintenance.

Opt-in instrumentation records whole-family validity **after every advancing
world step**, including dormant proposals and the last step after the last
compile epoch. The four original seeded anchors remain the denominator: losing
an anchor, invalidating its body or creating a replacement clone cannot shrink
it or substitute another family. For each family retain checks, valid steps,
final validity, first invalid step, invalid/recovery transitions and final failure
reason. A transient invalid family that later recovers still reduces maintenance.
Record paid seeded-editor rule events by installed rule ID range, separately from
ordinary meta-touching rewrites, and retain complete ERE window rows and per-class
reuse maps so the objective can be reconstructed without rerunning physics.

Let R be the unchanged priority-normalized structural proposal reuse score from
the previous pilot. Let T = valid seeded-family post-step observations divided
by (four families x actual completed steps); let F = final valid seeded families
divided by four. New joint score **J = R x T x F**. R, T, F and J are all reported.
A complete cap-free rollout with damaged families is a measured low/zero score,
not an invalid optimizer evaluation. A cap or short rollout is invalid and gets
null fitness; withhold both members of an invalid antithetic pair, never rank it
as zero. J cannot establish self-maintenance by itself; admission also requires
every seeded family valid on every observed step and no quarantine or cleanup.

## Frozen resource envelope and design

Protocol `E2-maintenance-reuse-pilot-v1`; seed `0xE352390001`, four common
seeds seed+0..3. Same 4x4 periodic binary PureCrystal substrate, four families,
50,000 work/50,000 results per step, rule-region ceiling 500, MaxParallel max
16 accepted matches, thread 1, epoch 10, metrics every 100 steps, initial editor
energy 32 per family, EI disabled. Recheck the explicit 400k/50k classification
and task-specific downstream admission receipts without modifying them.

- Twelve matched A1 frozen/A2 protected/A3 unprotected rollouts, 1,000 steps each.
- Four separately counted A3 baseline repeats and two forced A3 controls:
  structurally active and initially dormant priority-only, 1,000 steps each.
- Five real ES probe generations x eight candidates x 1,000 steps, sigma 1,
  alpha 0.05, using J. A generation with no valid pairs receives no update.

Total hard ceiling **58 attempts / 58,000 requested steps**. No retries, seed
shopping, replacement candidates or optional scientific rollouts. Catalogue and
instrument unit tests are separately accounted regression work. Every attempt,
error and available completed-step count is saved. The original reserved full
20 x 16 x 10,000 campaign and held-out evaluations remain at zero.

## Fixed checks and interpretation

- P1: forced executable family signature outside the initialization's 64-common-
  emission-seed distribution; at least one. This is a sampled signature check.
- P2: genotype-distribution total variation >=0.05 across 64 common emission
  seeds after probes **and all 20 antithetic pairs valid**. Parameter L2 diagnostic.
- P3: forced structural versus priority/dormant **joint-score** difference greater
  than twice the four-baseline population SD and greater than 1e-12.
- P4: joint-score baseline mean's distance from [0,1] endpoints greater than 0.02.
- Controls: all 12 advancing/cap-free/complete with reconstructible measurements.
- A3 maintenance: all four controls have T=F=1, no cleanup loss/quarantine, and
  actual structural world reuse in at least one control. Surviving anchors alone
  do not pass. A2 intact/firing editor is an instrumentation/control check only.

All must pass to recommend a future full-campaign decision; passing still does
not authorize that campaign. Four-seed comparisons are descriptive, not a strong
generalization claim. Report remaining blockers even if the optimizer moves or
J improves. This experiment tests the joint objective, not a new repair mechanism;
failure may show that the unchanged grammar lacks a viable maintenance route.

## Outputs

Fresh directory `output/ere_e2_maintenance_pilot_20260926`. Manifest and actual
effective configuration before admission/evaluation, emission distributions and
examples, per-attempt full ERE rows and errors, raw controls, scores/components,
probe parameters, prechecks, independently reconstructed receipt audit, status,
checksums and Markdown findings. Preserve all previous results.

## Results

The single preregistered pilot completed. **Instrumentation and joint objective
are implemented; full training remains blocked.** No retries, grammar retuning,
budget increases or held-out evaluations followed these findings.

| Check | Measurement | Result |
|---|---|---|
| P1 expressibility | One forced executable family signature outside sampled initial support | Passed |
| P2 traversal | TV 0.078125 >= 0.05; parameter L2 0.1478160954; only 16/20 valid pairs | Failed validity requirement |
| P3 joint discrimination | Difference 0.0677211854 > twice baseline SD 0.0614414122 | Passed |
| P4 not saturated | Initial joint-score mean/endpoint margin 0.0287097343 > 0.02 | Passed |
| Control execution | All 12 complete, cap-free, 1,000 advancing steps | Passed |
| Protected instrument control | Every family valid throughout; 128 paid edits/run; no ordinary meta rewrites or quarantines | Passed |
| Unprotected maintenance | Body validity lost in all four A3 controls; no observed recovery | Failed |

### Paired controls

Seed offsets are relative to `0xE352390001`. R is structural reuse, T is the
time-valid fraction, F the final-valid fraction, J = R x T x F. Every control
has zero cleanup loss and surviving anchors. Scores are rounded here; raw JSON
retains full precision. Paid seeded edits are scaffolding, not discovery.

| Offset | Arm | R | T | F | J | Structural fires | Paid editor events | Ordinary meta rewrites | Quarantine decisions |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | A1 | 0 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| 0 | A2 | 0.270582 | 1 | 1 | 0.270582 | 7,920 | 128 | 0 | 0 |
| 0 | A3 | 0.165193 | 0.508 | 0.5 | 0.041959 | 240 | 128 | 7,872 | 195 |
| 1 | A1 | 0 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| 1 | A2 | 0 | 1 | 1 | 0 | 0 | 128 | 0 | 0 |
| 1 | A3 | 0.132079 | 0.005 | 0 | 0 | 80 | 128 | 80 | 394 |
| 2 | A1 | 0.291739 | 1 | 1 | 0.291739 | 15,980 | 0 | 0 | 0 |
| 2 | A2 | 0.142221 | 1 | 1 | 0.142221 | 112 | 128 | 0 | 0 |
| 2 | A3 | 0.291520 | 0.5 | 0.5 | 0.072880 | 15,864 | 128 | 8 | 198 |
| 3 | A1 | 0 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| 3 | A2 | 0 | 1 | 1 | 0 | 0 | 128 | 0 | 0 |
| 3 | A3 | 0 | 0.508 | 0.5 | 0 | 0 | 128 | 7,872 | 195 |

Fresh seeds differ from the prior pilot, so fewer cap errors or larger TV cannot
be attributed to the new objective as a controlled before/after improvement.
Four matched seeds are descriptive evidence only. Frozen seed 2 already has
high reuse; editor engagement does not imply learning.

**10 of 16 original A3 families became invalid**: six after step 1, two after
step 11 and two after step 33. None recovered. Eight final failures are `Sides`,
two are `Variables`. All 16 A2 families remained valid throughout. Quarantines
are repeated compile decisions, not distinct destroyed families; per-family
counters now measure the latter validity losses explicitly.

The joint objective closes one blind spot: seed 1's positive R=0.132079 yields
J=0 because no complete family remains. Seed 2's high reuse still yields positive
J with half its families invalid, so multiplication alone is **not** a maintenance
gate. The separate all-family/all-step requirement rejects it correctly.

No complete optimizer probe combined positive J with T=F=1. Largest observed
probe J was 0.1010917624 at evaluation 31, with R=0.4023552734, T=0.5025, F=0.5.
That is an in-probe observation on a reused seed, not a trained champion or
held-out result. This sample does not prove the grammar has no viable genotype.

### Budget and errors

58 attempts = 12 controls + four separately counted baseline repeats + two
forced controls + 40 ES probes. **54 full rollouts plus 32 partial steps = 54,032
actual advancing steps**, within the 58,000 requested ceiling. Baseline repeats
exactly match corresponding A3 controls. Valid pairs by generation: **1,4,3,4,4**.

Four probes hit the unchanged matcher ceiling: evaluation 18 after 10 steps,
22 after 10, 25 after 12, and 34 before any advancing step. No partial candidate
pool was applied. Errors have null observations/fitness, not zero. Each affected
antithetic pair was withheld, including four complete partner evaluations. No
retry or replacement occurred. Five probe generations were run; full campaign
generations/evaluation steps and held-out steps remain zero. The full 3.2M-step
allocation is untouched.

Opt-in `EreHalt` errors retain exact completed-step counts. Halted receipts do
not retain partial ERE histories or final graph snapshots; their histories are
explicitly empty. Complete attempts retain all ten ERE window rows and exact
per-class fire maps, closing the previous score-reconstruction gap.

### Architecture and extension map

- [Runtime instrumentation](../crates/hg-sim-core/src/ere.rs): opt-in
  `EreConfig.measure_family_maintenance`, `measure_maintenance`, and event
  attribution by the known static-editor rule-ID suffix. Checks read the current
  whole graph family after every advancing step, not the cached compiled table.
  Both dormant and active bodies must be valid; strict/partial families cannot
  masquerade as intact E2 families. Original seeded anchors never change and
  counters accumulate across windows.
- [Additive metrics](../crates/hg-schema/src/ere.rs): `EreFamilyMaintenance`,
  optional `seeded_family_maintenance`, `seeded_editor_rewrite_events`, and
  `ordinary_meta_rewrite_events`. The flag defaults off and false is omitted from
  config serialization. Existing CSV/bincode and feature-off contracts remain.
- [Joint-objective pilot](../crates/hg-train/src/ere/maintenance.rs): `summarize`
  reconstructs R/T/F/J from ERE rows; `run_pilot` handles the frozen budget,
  controls and ES probes. It still calls the unchanged `e2::emit` grammar and
  shared production runner. No engine semantics changed.
- CLI: `ere-train pilot-maintenance BASE_CONFIG OUTPUT_DIR CLASSIFICATION_DIR
  DOWNSTREAM_GATE_DIR`. It refuses existing output directories; admission
  precedes evaluations and the full campaign is never called. Command completion
  is not scientific success; inspect `status.json`.

### Verification and provenance

[Independent read-only audit](../scripts/verify_ere_maintenance_pilot.jq) passed:
all scores reconstructed from maps and validity counters, paid/ordinary event
accounting, P1 support counts, P2 TV/valid pairs, P3/P4 statistics, exact step
accounting and blocked decision. It also checks full window horizons, effective
runtime profiles and identical seed material/configurations across matched arms.
It audits measured validity counters, not saved per-step graph snapshots (not
recorded), and does not rerun physics or the optimizer.

Three corrupted in-memory controls (fire map, family observation count, editor
event count) were rejected without editing source receipts. **91 targeted tests
passed**: sim-core ERE 44, training ERE 11, meta encoding/families 18, schema four,
feature-off pipeline/snapshot/matcher 14. Tests cover transient dormant-body
damage/recovery, anchor loss, fixed denominators, all-terminal-body loss, explicit
editor/ordinary event engagement and instrument on/off graph/count/RNG identity.
`git diff --check` passed. Previous pilot and admission/gate receipts checked
before/after are unchanged.

Preregistration SHA-256 before findings:
`d83d75548d7471aee4f6e5b937c07df2988e5feee9cf6ad8568ae64273f14164`.
Executed binary SHA-256:
`cbb795de8be0b3acf10f033aab8b13fdde662fce3ef6baebfe51ee9a380d91bb`.
[Provenance and raw-receipt hashes](../output/ere_e2_maintenance_pilot_20260926/provenance.json).

### Next decision

Do not expand training or raise caps on this evidence. Measurements now reveal
body loss accurately, but no observed probe maintained every family with useful
reuse. The next substantive intervention should be a **small schema-preserving
meta-rewrite vocabulary**, with explicit energy costs and damage/repair traces,
to give encoded families a genuine maintenance route. It needs a new semantics/
grammar protocol and gate checks, not hidden protection, automatic reader repair
or relabeling these failed controls. It was not implemented/evaluated here.

Subsequent work: [schema-preserving vocabulary v1](ere_meta_vocabulary_v1.md)
implements paid activation, deactivation and slot transposition and reports its
separate 12-rollout diagnostic. That follow-up does not change this pilot's raw
receipts or failed maintenance/stateability verdict.

### Artifacts and read-only reproduction

- [Manifest](../output/ere_e2_maintenance_pilot_20260926/manifest.json)
- [Admission](../output/ere_e2_maintenance_pilot_20260926/admission.json)
- [All evaluations and completed-step counts](../output/ere_e2_maintenance_pilot_20260926/evaluations.json)
- [Matched controls](../output/ere_e2_maintenance_pilot_20260926/controls.json)
- [Emission distributions](../output/ere_e2_maintenance_pilot_20260926/emission_distributions.json)
- [Stateability](../output/ere_e2_maintenance_pilot_20260926/prechecks.json)
- [Blocked status](../output/ere_e2_maintenance_pilot_20260926/status.json)
- [Arithmetic audit](../output/ere_e2_maintenance_pilot_20260926/receipt_verification.json)

```sh
jq -s \
  --slurpfile evaluations output/ere_e2_maintenance_pilot_20260926/evaluations.json \
  --slurpfile manifest output/ere_e2_maintenance_pilot_20260926/manifest.json \
  --slurpfile controls output/ere_e2_maintenance_pilot_20260926/controls.json \
  --slurpfile distributions output/ere_e2_maintenance_pilot_20260926/emission_distributions.json \
  --slurpfile prechecks output/ere_e2_maintenance_pilot_20260926/prechecks.json \
  --slurpfile status output/ere_e2_maintenance_pilot_20260926/status.json \
  -f scripts/verify_ere_maintenance_pilot.jq \
  output/ere_e2_maintenance_pilot_20260926/rollouts/*.json
```
