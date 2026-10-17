# Valid-production optimizer signal panel

Panel date: 2026-09-29; accounting audit completed 2026-09-30.
Profile `E2-complete-valid-optimizer-signal-v1`.

## Outcome

The single registered native invocation completes all **120/120 rollouts**
at 100 steps and all **20/20 ES pairs**. P1–P4 all pass under the unchanged
criteria. The corrected postmeasurement accounting checker agrees and rejects
all 19 tamper categories. The originally frozen checker did not pass: its
variable-shadowing exception and the narrow correction are disclosed below.
No physics retry, replacement, seed change or acceptance-threshold change
occurred. This is a successful bounded stateability diagnostic, not training
admission, independent physics replay, learned maintenance or a new G-R1 result.

Authoritative artifacts:

- [Manifest](/Users/kalewicker/Documents/hypersim/output/ere_valid_optimizer_signal_panel_20260929/manifest.json): `status:complete`, `measurement_complete:true`, `training_admitted:false`.
- [Native summary](/Users/kalewicker/Documents/hypersim/output/ere_valid_optimizer_signal_panel_20260929/summary.json): `precheck_pass:true`.
- [Corrected accounting audit](/Users/kalewicker/Documents/hypersim/output/ere_valid_optimizer_signal_panel_20260929/audit.json): `audit_pass:true`, 19 tamper rejections.
- [Correction provenance](/Users/kalewicker/Documents/hypersim/output/ere_valid_optimizer_signal_panel_20260929/postmeasurement_audit/correction.json): original failure and unchanged measurement hashes.
- [Completion seal](/Users/kalewicker/Documents/hypersim/output/ere_valid_optimizer_signal_panel_20260929/output_sha256.json): all 454 entries verify; SHA256 `8166cb77242f4e141f204b0c7b1d8e6c40d04f366fa9ed0143231255de72b205`.

## Fixed-denominator results

| Precheck | Registered requirement | Measured result | Decision |
| --- | --- | --- | --- |
| P1 | Active control signature absent from initial 64 emissions | Absent; all 64 are valid, with 58 distinct signatures | Pass |
| P2 | Actual common-seed distribution TV ≥ .05 and 20/20 valid pairs | Maximum TV .109375 = 7/64; 20/20 valid pairs | Pass |
| P3 | Active-minus-dormant gap > twice all-32 baseline population SD and > 1e-12 | Gap 1.0; SD .33071891388307384; twice SD .6614378277661477 | Pass |
| P4 | Baseline mean strictly between .02 and .98, no nulls | Mean .125; 32/32 valid scores | Pass |

All eight active/dormant controls finish; the four active scores are 1 and
four dormant scores are 0. All 32 baseline and 80 ES attempts finish, with
12,000 attempted and advancing Engine steps overall. No emitted genotype is
rejected, no score is null, and no runtime cap is hit.

| Baseline | Parent | Operator | Role / production index | Scores at physics seeds 509 / 613 / 719 / 827 |
| --- | --- | --- | --- | --- |
| 0 | 16 | Activate | Proposal / 0 | 1 / 1 / 1 / 1 |
| 1 | 150 | RemoveLabel | Core / 0 | 0 / 0 / 0 / 0 |
| 2 | 144 | RetargetSlot | Proposal / 3 | 0 / 0 / 0 / 0 |
| 3 | 163 | ChangePriority | Proposal / 0 | 0 / 0 / 0 / 0 |
| 4 | 61 | AddVar | Core / 2 | 0 / 0 / 0 / 0 |
| 5 | 129 | AddLabel | Core / 4 | 0 / 0 / 0 / 0 |
| 6 | 55 | Deactivate | Proposal / 0 | 0 / 0 / 0 / 0 |
| 7 | 157 | ChangePriority | Proposal / 0 | 0 / 0 / 0 / 0 |

Within-genotype physics SD is 0; between-genotype SD is .33071891388307384.
These four physics seeds give identical scores within each registered baseline
in this narrow MaxParallel fixture. That is not evidence of general stochastic
robustness, ecological diversity or successful maintenance. Most sampled
baseline genotypes have no credited Proposal reuse.

| Generation (zero-based) | Valid pairs | TV from initial common-seed panel | Cumulative theta L2 norm |
| --- | --- | --- | --- |
| 0 | 4/4 | 0 | 0 |
| 1 | 4/4 | .015625 | .126001703 |
| 2 | 4/4 | .03125 | .180042523 |
| 3 | 4/4 | .03125 | .193737315 |
| 4 | 4/4 | .109375 | .249161846 |

Generation 0 has a flat objective and correctly produces no update. Later
registered pairs generate nonzero tied-rank updates. Across all 80 ES scores,
62 are zero, eight are one, two are .8961958447491268, six are
.8406206186877936 and two are .8360489289678587. Distribution movement under
actual optimizer updates passes P2; this does not establish improved held-out
fitness, a learned champion, novelty or adaptation.

Compared with the preserved earlier raw-coherent panel, valid rollouts improve
from 94/120 to 120/120 and valid pairs from 14/20 to 20/20. The old maximum TV
was .0625 but its pair-validity requirement failed; its null baseline statistics
made P3/P4 fail closed. Those historical decisions remain unchanged. The new
panel uses a prospectively different grammar and fresh seeds, so this is not a
controlled causal estimate of the grammar's improvement.

## Audit correction, resources and preservation

The frozen checker failed with `KeyError: 'parents'` after its first rollout:
the catalogue variable `c` was overwritten by per-emission class data. An ES
coefficient later reused the same name. The postmeasurement correction changes
only the catalogue binding and its three membership-use sites to `catalogue`.
An exact mechanical diff was verified; reward, rank, membership, denominator,
threshold and sealing logic were not changed.

The original checker stays in `source_snapshots/scripts/`; the corrected checker
and a new read-only whole-archive regression are archived separately in
`postmeasurement_audit/`. The correction receipt pins both checker hashes,
the updated regression and the original manifest, summary and rollouts. All
435 original frozen source hashes still verify. There are exactly two declared
live/frozen source differences: the accounting checker and its regression file;
the other 433 sources match. The original six helper tests passed before
measurement; all seven tests, including the new 504-membership-call regression,
pass after correction. This must not be described as original frozen-checker
passage or a checker correction fixed before measurement.

The corrected accounting audit reconstructs all 120 scores, 20 paired updates
and 384 distribution emissions. Tamper cases cover score, fires, seed,
duplicate ID, step, credit, cap, pair, theta, epsilon, distribution, summary,
world, membership, mask, fallback, retry, catalogue and control. All 19 are
rejected on temporary copies; production measurements are unchanged.

Native elapsed time is 174.923 seconds against 900, including 167.087 seconds
for the exact-hash grammar rebuild. The remaining emission/evaluation/optimizer
work is approximately 7.836 seconds. Final sealed output is 45,199,579 bytes
against 512 MiB. Peak per-step matcher work/results are 11,192 / 1,194 within
the registered 400k/50k envelope. Although these observed peaks are below
50k/50k, this is not a separately measured runtime-budget bridge.

All 454 new output entries, including nested prerequisite seals and declared
postmeasurement correction artifacts, verify with exact file-set coverage.
The historical optimizer's 441, grammar's 438 and accepted closure's 364 seal
entries also verify unchanged. Only the new archive's root seal excludes itself.

## Next boundary

Build and test the complete-profile runtime/admission bridge before proposing
full training. Bind the exact grammar, coherent kernel, isolated allocator,
matcher work model and receipt hashes; keep the runtime ceiling at 50k/50k.
Its contract must explicitly address the disclosed postmeasurement checker
correction rather than asserting an unmodified frozen-auditor pass. Verify
missing, stale, tampered or incompatible receipts refuse admission with zero
optimizer/precheck/held-out steps, then prospectively register any needed fresh
runtime confirmation. No such additional physics or full 20×16×10,000-step
training campaign was launched in this task. Legacy admission remains closed
for complete-profile vocabulary-bearing configurations.

## Design and implementation

The user requested a fresh panel after building the validated production
grammar. The immutable prospective
[contract](/Users/kalewicker/Documents/hypersim/docs/ere_valid_optimizer_signal_panel_v1.md)
has SHA256 `010264bc0fbb63459d501af379988e8f7c4a96d1c208584ff8a653b557688c08`.
This is a bounded frozen-family optimizer diagnostic, not full training,
learned maintenance, paid repair or a new G-R1 assay.

The schedule retains the original four criteria, 183-logit layout, zero initial
theta, sigma 1, alpha .05, five generations / four antithetic pairs, 100-step
horizon, fixed ordinary world and native structural Proposal reuse objective.
Only the validated grammar, fresh registered seeds and legal dormant-control
construction change. The new master seed is `0xE552390001`; physics seeds are
509, 613, 719, 827. The 32 baseline, eight control and 80 ES attempts retain
all 120 scheduled members and the 12,000 attempted-step ceiling. No fitness
or result selected these seeds or the grammar's legal productions.

The active control is parent 4 + Activate. Its dormant counterpart is parent
5 + Deactivate, where parent 5 has identical Core/Proposal alleles and begins
active. This makes both mode edits legal changed productions rather than
retaining a forbidden no-op. The producer verifies matched final alleles and
typed ordinary worlds, with only activation different, before physics.

The new
[harness](/Users/kalewicker/Documents/hypersim/crates/hg-train/src/ere/valid_signal.rs)
is separate from the original `signal-complete` path. It verifies both sealed
prerequisites, freezes current sources/binary, rebuilds the grammar once and
requires exact registered catalogue SHA256 equality. It then uses the unchanged
sim-core native evaluation backend and tied-rank ES. All pair members share
emission and physics seeds; a null on either side withholds the complete pair.
No retry, replacement, parent fallback, repair or outcome-dependent tuning is
permitted. Wall cap is 900 seconds including grammar rebuild after source
freezing; output cap is 512 MiB. Runtime matching remains this diagnostic's
400k/50k opt-in feasibility envelope, not the 50k/50k production bridge.

The new
[checker](/Users/kalewicker/Documents/hypersim/scripts/audit_ere_valid_optimizer_signal.py)
independently reconstructs reward credit, scores, caps, denominators, pair
withholding, float32-aware rank updates, distribution TV and all four decisions.
It also checks grammar production membership and masks for every measured
genotype and all 384 common-seed distribution emissions. It is an accounting/
membership audit, not independent semantic decode or native physics replay.
Only the root completion seal is excluded from sealing; frozen nested seals
remain covered. The previous failed panel and accepted grammar/closure archives
are preserved as separate evidence with their original decisions.

## Entry points

```sh
cargo run --release -p hg-train --features ere-complete-signal --bin ere-train -- signal-valid-complete FRESH_OUTPUT_DIR output/ere_complete_coherent_closure_full_20260929 output/ere_valid_production_grammar_20260929
python3 scripts/audit_ere_valid_optimizer_signal.py FRESH_OUTPUT_DIR --tamper-tests --seal
```

The 92 training unit tests, 11 existing ERE integration tests, two complete-
profile compatibility/admission tests and six Python helper regressions pass
before measurement. New guard checks reject missing grammar/closure evidence
before creating output or executing physics. Existing complete-profile full
training remains closed with zero optimizer/precheck/held-out steps.
