# E2-aware grammar and bounded pilot

Preregistered 2026-09-26 before pilot/control/stateability results. Scope:
implement an explicit family grammar and admission for the verified resource
profile; run bounded controls and stateability probes, never automatically run
the reserved 20 x 16 x 10,000 full training campaign.

## Grammar and attribution

Four families; each has independent categorical core (four choices), proposal
(four choices), and activation (two choices) distributions: 40 real logits.
The grammar emits actual validated core/proposal graph families through the
shared production rollout, not a separate simulator or an edited inner genome.
Core motifs are identity binary, identity path, identity ternary, identity star.
Proposals are a priority-only core copy (negative/scaffolding control),
path-to-star rewiring, path-to-self-loop-plus-shortcut rewiring, and label-1
constrained path-to-star (unmatched-on-label-0 control). Rewirings preserve their
three variables and two-edge counts. All catalogue pairs/modes are validated;
malformed families reject, never repair/resample or silently fall back to core.

Production configurations omitting explicit families remain unchanged. Static
A0 and frozen A1 execute the same initially enabled family alternatives; their
inert graph seed regions are identical. A2 protects meta structure; A3 permits
ordinary matching into it. All arms share the same seeded families, world seed,
initial substrate and energy. A1 has no editor; A2/A3 receive the same declared,
paid activation/priority editor. Its events themselves earn no proposal credit.

Seeded executable candidates belong to the initial novelty baseline, including
active proposals. Dormant proposals count as newly executable only upon genuine
activation. Record actual runtime activation transitions and proposal world-fire
counts, separately from cumulative novelty. Only unambiguously attributed,
world-touching **structurally different** proposal classes enter the pilot's
primary score. Priority-normalized RCF comparison excludes priority-only copies;
classes simultaneously serving as a core and proposal receive no proposal credit.
Structural reuse buckets also normalize priority, so seeded-editor priority
increments cannot manufacture extra rewarded classes for the same body.
Seeded proposal activity is not endogenous discovery or evolved self-maintenance.

## Fixed profile and admission

Explicit admission rechecks raw G-R1 400k/50k classification and full pooled
G-R0/2/3/4/5/6 receipts using their existing task-specific budgets. It rejects
wrong protocols/policies, incomplete horizons, missing semantic strata, vacuous
identity, missing validator controls or an unsuppressed moving-window EI.
Old failed receipts and legacy campaign admission remain unchanged.

Runtime pilot work/results remain **50,000/50,000 per step**, aggregate rule
region ceiling 500, four families, MaxParallel with at most 16 accepted matches,
4x4 periodic binary PureCrystal substrate, fixed epoch 10, seed energy 32 per
family, metrics every 100 steps, EI disabled. No budget escalation, seed shopping,
rerun-to-pass or catalogue retuning based on results.

## Pilot budget and checks

Seed `0xE252390001`; four common seeds (seed + 0..3). Twelve matched control
rollouts: frozen A1, protected A2 and unprotected A3 for each seed, 1,000 steps
each. Report per-seed progression, caps, quarantine, anchor survival, cleanup,
proposal reuse, activation, editor events and novel classes. Comparisons are
paired/descriptive; four seeds do not support a strong generalization claim.

Stateability uses the same A3 objective and parameterized grammar:

- P1: 64 common emission seeds, initialization versus a forced grammar; at least
  one executable class outside the initialized signature distribution.
- P2: five real OpenAI-ES probe generations x eight candidates x 1,000 steps;
  sigma 1, alpha 0.05; emitted-graph total variation >=0.05 over 64 common seeds.
  Parameter L2 is diagnostic only; this is a probe, not the full campaign.
- P3: forced structurally active versus dormant priority-only configurations,
  score difference > twice the four-seed initialization score SD and >1e-12.
- P4: initial score's distance from either endpoint [0,1] >0.02.

Primary score: sum over structural proposal classes of log(1 + max(fires-1,0)),
divided by four x log(1 + steps x max_pairings / four), clipped to [0,1].
F4/PI is diagnostic, never used to select a passing outcome. A capped or short
rollout is invalid, not a zero fitness; invalid paired optimizer evaluations
are withheld. Four A3 baseline rollouts plus two forced rollouts add 6,000 steps.

Total ceiling: **58 rollouts / 58,000 simulation steps** (12,000 controls +
46,000 prechecks). Catalogue/unit tests and 64-seed grammar-distribution samples
are separately reported, not hidden optimizer evaluations. Record every attempted
evaluation, including errors. Failed checks or unprotected losses block full
training; this task never starts the 3.2M-step campaign, even if all checks pass.

## Outputs

New artifacts: `output/ere_e2_grammar_pilot_20260926`, with manifest written
before admission/probes, raw controls and evaluation log, precheck status,
emitted family examples, explicit resource-profile verification and Markdown
report. Preserve all historical results and user changes.

## Results

The single preregistered pilot completed on 2026-09-26. **Implementation is
complete; full training is blocked by the pilot.** No retries, seed selection,
threshold changes, runtime-budget increases, full campaign or held-out
evaluations followed these findings.

| Check | Measurement | Outcome |
|---|---|---|
| P1 class expressibility | One forced executable family signature outside the initialization's 64-seed sample | Passed |
| P2 optimizer traversal | Emission TV 0.0625 >= 0.05; parameter L2 0.1270107703; only 12/20 valid antithetic pairs | Failed validity requirement |
| P3 objective discrimination | Forced structural/dormant difference 0.2708847415 > twice baseline SD 0.2469550489 | Passed |
| P4 not saturated | Baseline mean/endpoint margin 0.1162887189 > 0.02 | Passed |
| Matched controls | All 12 completed 1,000 steps without binding caps | Complete/valid |
| Unprotected intact | All four A3 controls had quarantine decisions, though anchors survived and cleanup loss was zero | Failed |

P2's validity condition was in the executable pilot before the run: all 20
pairs must be valid, not just a nonzero parameter update or sufficient TV.
The five probe generations are real optimizer evaluations, **not** the reserved
20-generation campaign. Generation two had zero usable pairs and therefore no
update. P1 is a sampled family-signature support check, not an exhaustive proof
that a particular individual allele is absent from every initial draw.

### Matched controls

Seed offsets are relative to `0xE252390001`. Every row has final step 1,000,
anchor survival 1.0, zero cleanup edges lost and no binding ceiling. Scores are
rounded here; full-precision results and final graph hashes remain in the JSON.

| Seed offset | Arm | Structural reuse score | Structural world fires | Activation transitions | Quarantine decisions | Meta-touching rewrites | Novel executable classes |
|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | A1 frozen | 0 | 0 | 0 | 0 | 0 | 0 |
| 0 | A2 protected | 0.291291 | 15,744 | 3 | 0 | 128 | 8 |
| 0 | A3 unprotected | 0.291291 | 15,744 | 2 | 99 | 176 | 8 |
| 1 | A1 frozen | 0 | 0 | 0 | 0 | 0 | 0 |
| 1 | A2 protected | 0 | 0 | 2 | 0 | 128 | 8 |
| 1 | A3 unprotected | 0 | 0 | 1 | 195 | 8,000 | 7 |
| 2 | A1 frozen | 0 | 0 | 0 | 0 | 0 | 0 |
| 2 | A2 protected | 0 | 0 | 3 | 0 | 128 | 16 |
| 2 | A3 unprotected | 0 | 0 | 0 | 396 | 168 | 0 |
| 3 | A1 frozen | 0.270885 | 8,000 | 0 | 0 | 0 | 0 |
| 3 | A2 protected | 0.132079 | 80 | 2 | 0 | 128 | 12 |
| 3 | A3 unprotected | 0.173864 | 320 | 0 | 294 | 8,000 | 7 |

Quarantine decisions are summed across compile epochs: a broken family can be
counted repeatedly. They are **not** counts of distinct destroyed families.
Anchor survival does not imply valid encoded bodies. Protected runs accumulated
no quarantine decisions; unprotected runs did. Four common seeds are descriptive
evidence, not a population-level causal/generalization result. The protected
editor does not uniformly improve structural reuse relative to frozen seeds.

The zero structural scores in seeds 1 and 2 are measured zeros, not missing
rollouts. For example seed 1 has 8,000 proposal fires but none qualifying as
structurally distinct reuse. The forced priority/dormant negative control also
records 7,920 proposal fires after editor activations, yet structural score zero.
Priority changes, activation counts and novelty alone therefore cannot earn this
objective. Conversely, seed 0's A3 score equals A2's despite 99 quarantine
decisions: **the reuse objective does not establish encoded-family maintenance**.

### Budget, errors and verification

| Section | Attempts | Complete, cap-free rollouts | Requested-step ceiling |
|---|---:|---:|---:|
| A1/A2/A3 matched controls | 12 | 12 | 12,000 |
| A3 baseline repeats | 4 | 4 | 4,000 |
| Forced positive/negative controls | 2 | 2 | 2,000 |
| Optimizer probes: five generations x eight candidates | 40 | 31 | 40,000 |
| Total pilot | 58 | 49 | 58,000 |
| Full optimizer campaign | 0 | 0 | 0 |
| Held-out evaluations | 0 | 0 | 0 |

The nine invalid evaluations (zero-based indices 26, 27, 29, 31, 32, 38, 40, 50,
52) all returned `ERE match ceiling is binding; partial candidate pool was not
applied`. Both limits remained 50k. The error does not separately identify which
budget bound first. These evaluations have null observations and no zero fitness.
Eight antithetic pairs were invalid; seven otherwise-complete partner evaluations
were consequently withheld too. Valid pairs by generation: **4, 0, 2, 4, 2**.
No failed candidate was retried or replaced.

Known completed work is 49,000 simulation steps. Failed rollouts may have made
partial progress, but their errors do not retain the final step; actual total
executed work is not exactly measured and is bounded above by the 58,000 requested
steps. Baselines were deliberately re-executed and counted, not reused for free;
their observations exactly match their paired A3 controls. Catalogue/unit tests
are separate regression work, not hidden research evaluations.

[Read-only receipt audit](../scripts/verify_ere_e2_pilot.jq) rechecks all attempt
indices/sections/seeds, controls versus raw log, baseline repeat identity, usable
pair counts, parameter L2, P3/P4 statistics and the blocked decision. Its
[receipt](../output/ere_e2_grammar_pilot_20260926/receipt_verification.json) verifies
accounting, **not** a research pass. It does not rerun physics or the optimizer;
P1 signatures and P2 TV are accepted as recorded. The pilot saved aggregate
reuse/score observations, not per-class fire maps or complete metrics histories,
so exact independent primary-score reconstruction from these receipts is not
possible. Future instrumentation should retain those maps.

Three in-memory audit negative controls (wrong attempt count, false P2 pass,
duplicate evaluation index) were all rejected without editing source receipts.
The final targeted regression suite passed **86 tests**: sim-core ERE 42,
training ERE eight, meta encoding/families 18, schema four, and feature-off
pipeline/snapshot/matcher compatibility 14. `git diff --check` passed.

### Measurement-label erratum

The pilot executable serialized `meta_editor_events`, but the underlying
`EreMetricsRow.meta_rewrite_events` counts **all** meta-touching/creating rewrites,
including ordinary A3 rewrites. This is the column labeled accurately above.
The original raw JSON is preserved byte-for-byte. A2 excludes ordinary rewrites
from meta structure, so its 128 events are seeded paid-editor events. A3's exact
paid-editor count is **not separately measured**; its 8,000 meta rewrites must not
be described as 8,000 paid edits or endogenous discoveries. The declared initial
editor energy permits at most 128 paid events across four families.

Source now writes the accurate field `meta_rewrite_events` and accepts the old
name only as a deserialization alias. A regression test covers this correction.
The already-running executable was not replaced, results were not rewritten,
and no science was rerun. Primary scores, quarantine counters, thresholds and
the blocked decision do not depend on this label.

### Implementation and extension map

- [Family grammar and pilot](../crates/hg-train/src/ere/e2.rs): `emit`, `forced`,
  `configure`, `observe`, `run_pilot`. To add an allele, update categorical group
  dimensions, parameter count and exhaustive catalogue tests together; register
  a new protocol/manifest before evaluating it. Forced helpers are controls,
  not optimized solutions.
- [Runtime family installation and attribution](../crates/hg-sim-core/src/ere.rs):
  `EreSeedFamily`, optional `EreConfig.seed_families`, `EreRuntime::install`,
  `refresh_family_roles`, `after_step`, `close_window`. Validate both bodies and
  aggregate bounds before seed writes; preserve the initial executable novelty
  baseline, A0/A1 equivalence and unambiguous role-credit rules.
- [Gate-specific admission](../crates/hg-sim-core/src/ere/admission.rs):
  `require_pooled_400k50k_gates` rereads the classification and downstream raw
  receipts and applies their separate resource identities. It is an explicit
  profile, not a relabeling of pooled-v1 or a global budget increase. The legacy
  campaign guard remains unchanged.
- [ERE metrics](../crates/hg-schema/src/ere.rs): optional proposal reuse,
  structural reuse, ambiguous-role reuse and runtime activation fields. Existing
  configurations without explicit families retain their defaults and legacy
  CSV/bincode contracts.
- [Runtime regression tests](../crates/hg-sim-core/tests/ere_grammar.rs) cover
  seeded novelty exclusion, dormant activation, unmatched/priority-only controls,
  normalized reward buckets, ambiguous roles, static/frozen identity and strict
  profile admission. [Training tests](../crates/hg-train/tests/ere.rs) cover all
  32 core/proposal/mode catalogue combinations, shared production execution,
  budget/admission guard and the label compatibility correction.

The CLI is `ere-train pilot-e2 BASE_CONFIG OUTPUT_DIR CLASSIFICATION_DIR
DOWNSTREAM_GATE_DIR`. It refuses an existing output directory, never calls the
full campaign, and reports scientific failure in `status.json`; a successfully
completed pilot command is not a claim that its checks passed. The saved manifest
contains a base-config template; `configure` overrides its rules, seed, horizon,
arm, editor and `measure_ei=false` for every rollout. EI was disabled, not zero.

### Provenance and follow-up

Preregistration SHA-256 before appending findings:
`c3df38917f0e5806a17a19e1081a08bcdd64d6f82e7d9d5e5f6db835e744c001`.
Executed release binary SHA-256:
`beeab3f02d217e3b78121e59f415a24e0034e84f57ec92a35e327b1d72797fd2`.
Historical classification and G-R2/G-R3/fixture receipt hashes were checked before
and after the pilot and are unchanged. Raw pilot receipts, checksums and source
label erratum are linked in the output directory's report/provenance.

The next decision is a **new preregistered maintenance-focused redesign**, not
the full 3.2M-step optimizer run. First retain per-family validity, per-class
fire maps and a separately identified paid-editor count; then test whether
unprotected rewrite semantics can maintain complete encoded bodies under the
existing caps. A future objective could jointly require valid-family maintenance
and structural world reuse, but changing it now would be post-hoc. Protection is
currently a diagnostic control, not the desired A3 scientific result. Any grammar,
maintenance objective or budget change requires new receipts rather than a retry
of this pilot. The original 20 x 16 x 10,000 full-training allocation is untouched.

Subsequent authorized follow-up: the [maintenance-and-reuse pilot](ere_e2_maintenance_pilot.md)
added opt-in per-step family validity, exact paid-editor attribution and a
reconstructible joint objective. Its new 58-attempt experiment also blocked
full training; it does not replace or amend these original raw results.

### Artifacts

- [Frozen manifest](../output/ere_e2_grammar_pilot_20260926/manifest.json)
- [Explicit admission](../output/ere_e2_grammar_pilot_20260926/admission.json)
- [Actual family examples](../output/ere_e2_grammar_pilot_20260926/emitted_examples.json)
- [Every evaluation, including errors](../output/ere_e2_grammar_pilot_20260926/evaluations.json)
- [Matched controls](../output/ere_e2_grammar_pilot_20260926/controls.json)
- [Stateability checks](../output/ere_e2_grammar_pilot_20260926/prechecks.json)
- [Probe parameters](../output/ere_e2_grammar_pilot_20260926/probe_theta.json)
- [Full-training blocked status](../output/ere_e2_grammar_pilot_20260926/status.json)
- [Raw-receipt checksums and provenance](../output/ere_e2_grammar_pilot_20260926/provenance.json)

To reproduce only the read-only accounting audit (no experiment rerun):

```sh
jq -n \
  --slurpfile evaluations output/ere_e2_grammar_pilot_20260926/evaluations.json \
  --slurpfile controls output/ere_e2_grammar_pilot_20260926/controls.json \
  --slurpfile prechecks output/ere_e2_grammar_pilot_20260926/prechecks.json \
  --slurpfile status output/ere_e2_grammar_pilot_20260926/status.json \
  --slurpfile manifest output/ere_e2_grammar_pilot_20260926/manifest.json \
  --slurpfile theta output/ere_e2_grammar_pilot_20260926/probe_theta.json \
  -f scripts/verify_ere_e2_pilot.jq
```
