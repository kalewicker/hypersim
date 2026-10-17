# E1.1 shared-pattern redesign preregistration

Status: implemented and verified; **G-R1 still fails**. This is a new encoding,
not a retroactive correction of native E1's failed receipt. Research and training
remain blocked. The preregistered mechanism/protocol below are retained, followed
by the results and explicit limitations.

## Verified outcome

| Metric / check | Native E1 | E1.1 verified | Interpretation |
|---|---:|---:|---|
| Valid | 81.84% | 80.78% | Both meet ≥40% |
| NearNeutral | 13.44% | 13.25% | Both meet ≥10% |
| Catastrophic | 30.27% | **29.93%** | Both fail ≤25%; no demonstrated improvement |
| Frozen catastrophic cases | 2,974 | 2,934 | Dominant failure mode persists |
| Exploding catastrophic cases | 53 | 59 | Small minority of failures |
| Unmeasured classifications | 0 | 0 | No classification ceiling bound |
| Neutral soundness spot checks | 50/50 | 50/50 | No observed soundness failure |
| Ceiling-adjacent Valid | 1,107/1,428 (77.52%) | 1,084/1,455 (74.50%) | Separately reported, no cap exclusion |
| Ceiling-adjacent Catastrophic | 551/1,428 (38.59%) | 501/1,455 (34.43%) | Same high-arity limitation |
| Expressivity / 1,000-step round trips | 2/45 | 2/45 | Envelope unchanged; no new expressivity claim |
| Interpreter identity | 100,000 steps, pass | 100,000 steps, pass | Every full hash and canonical per-step CSV equal |
| Quarantine engagement | `[1;9]` | `[1;9]` | Each validator independently engaged |
| Protection / meta editor | 10,000 steps, 32 edits | 10,000 steps, 32 edits | Full anchor survival, no cleanup loss |
| G-R6 positive-change fixture | Pass | Pass | Both TPM variants suppressed after table change |

Verified receipts are in `output/ere_e11_verified_20260925`. Its mutation JSON
is byte-identical to the initial E1.1 report, including every operator and seed
shape breakdown. This verification replay supplies no additional independent
samples or improved significance. The original native diagnostic replay also
reproduced the original native aggregate counts exactly.

EXP-R04 through EXP-R10 have not launched. The new E1.1 budget record in
`output/ere_e11_training_20260925` contains the same 20×16×10,000 budget and
machine-readable zero optimizer/precheck/held-out evaluation counts. Neither
this record nor the prior native record contains a trained champion.

## Mechanism and scope

Native E1 stores two separate pattern nodes even when an ordered edge appears
identically on both sides. E1.1 will instead share that pattern node, with one
membership edge to each side. A local slot or pattern-node edit then affects
both occurrences through topology, without a reader repair rule. This extends
the existing shared-variable principle to identical patterns. Duplicate edge
occurrences are paired one-to-one, never merged within a side. Unpaired LHS/RHS
patterns remain independent; edge multiplicity and all compiled semantics are
unchanged in unmutated round trips.

The region root carries an explicit version property. Legacy encoding remains
the default and readable. Unknown version values quarantine. Shared patterns
carry separate side presentation hints to preserve legacy matcher order;
hints and the encoding version are not part of semantic RCF identity.

This proposal does not reject rules based on their fitness, add fallback rules,
repair invalid mutations, introduce a phenotype-dependent validator, or change
the matcher/rewrite physics. The hypothesis may fail.

## Frozen comparison protocol

- Keep the same 28-rule seed bank, all 11 uniformly sampled mutation operators,
  RNG seed `0xE152310001`, 10,000 samples, and 100-step fixtures.
- Keep Valid ≥40%, NearNeutral ≥10%, Catastrophic ≤25%, all nine validators,
  resource ceilings, explosion criterion and freeze criterion unchanged.
- First replay native E1 with additive operator/seed-shape/cause counters.
  Its original counts must reproduce exactly before interpreting the counters.
- Characterize E1.1 once under these fixed settings. Different encoded pattern
  counts affect random index draws: this is not a per-sample paired comparison.
- Report both overall and ceiling-adjacent fractions and all failure causes;
  a failed redesign is preserved, not followed by threshold or seed shopping.
- Do not reuse native E1 receipts. E1.1 needs new G-R0/1/2/3/4/5/6 checks and
  a gate receipt bound to its encoding before any research or optimizer run.

The previously committed training budget remains 20×16×10,000 optimizer steps.
Training stays blocked until the new prerequisites pass. Characterization and
mechanism checks are not optimizer evaluations.

## Validation obligations

Round-trip exact execution order with duplicate/shared/unpaired edges; shared
slot edits visible on both sides; side-detachment produces an ordinary
independent pattern; duplicate membership on the same side quarantines; legacy
regions cannot acquire shared-pattern interpretation without a version change;
unknown versions and malformed hints quarantine. RCF agrees across encoding
versions and allocation histories for the same semantic rule. Recheck all nine
validator engagements, protection, no-RNG compilation, and 100,000-step identity.

## Native E1 diagnostic replay

`output/ere_v2_diagnosis_20260925/exp_r01.json` reproduced all original aggregate
counts exactly: 8,184 Valid, 2,052 Neutral, 1,344 NearNeutral, 1,761 Divergent,
1,816 Quarantined and 3,027 Catastrophic, with no unmeasured cases. New counters
partition the catastrophic class into 2,974 freezes and 53 explosions.

| Operator | Samples | Frozen | Exploding |
|---|---:|---:|---:|
| AddPatternEdge | 931 | 518 | 53 |
| AddVar | 922 | 492 | 0 |
| AddLabel | 936 | 454 | 0 |
| AddSlot | 871 | 444 | 0 |
| RetargetSlot | 886 | 312 | 0 |
| RemovePatternEdge | 896 | 308 | 0 |
| RemoveSlot | 908 | 161 | 0 |
| RemoveVar | 911 | 145 | 0 |
| ChangeLabel | 963 | 140 | 0 |
| ChangePriority / RemoveLabel | 1,776 | 0 | 0 |

These counts support a limited claim: freezes dominate, with many changes to
matching structure. They do not establish that asymmetric storage caused each
freeze, or that sharing will help. Labels and genuinely asymmetric patterns
remain exposed. The fixed full E1.1 run tests that hypothesis without altering
the fixtures to make newly introduced labels or arities match.

## Initial E1.1 result and parser correction

The initial full run in `output/ere_e11_20260925` measured Valid 80.78%,
NearNeutral 13.25% and Catastrophic 29.93%: **G-R1 still fails**. It contains
2,934 freezes and 59 explosions. The small 0.34 percentage-point difference
from native E1 does not establish an improvement, particularly without paired
sample trajectories. Sharing identical patterns is insufficient.

The adversarial fixture also caught a parser bug: presentation-hint range was
incorrectly tied to the currently configured pattern-count ceiling. Setting the
ceiling to zero to engage V8 instead engaged V3 before reaching V8. Hints are
nonsemantic positions and can retain gaps after deletions; their numeric range
must be independent of the current edge-count ceiling. They now accept typed
nonnegative u32 positions, still rejecting duplicates and malformed values.
The original failed fixture is preserved, not overwritten.

A clean verification replay uses `output/ere_e11_verified_20260925`. The hint
fix cannot change this mutation sample's semantics: the frozen seed/operator
kernel emits only small original hints or no hint, never an out-of-range hint.
The replay must reproduce every mutation count exactly; it is a regression
check of this parser correction, not another parameter/seed trial.

## Code and extension contract

`hg-meta::encode` keeps legacy output. `encode_versioned` opts into
`EncodingVersion::SharedPatternsV2`. A root's `K_ENCODING = 8` property is Int(2)
for E1.1; missing/Int(1) uses the old schema, and other values quarantine. One
pattern can belong once to each side in E1.1, but duplicate same-side membership
always fails. Side order uses `K_ORDER = 6` and `K_ORDER_R = 7`. Pairing is
one-to-one and preserves duplicates, exact ordered endpoints, and each side's
presentation. Unmutated executable bytes and semantic RCF are unchanged.

`EreConfig.encoding_version` is explicit and omitted when legacy, preserving
existing serialized configurations/hashes. `configs/ere_e11.toml` opts in.
The production runtime, mutation assay, round-trip assay, identity harness and
adversarial/protection fixtures all use the selected writer. Gate receipts are
checked against that encoding in both arm/sweep and training entry points;
new versioned receipts cannot unlock the old configuration, or vice versa.
Legacy receipts without a version marker remain readable only as legacy.

The assay now records `by_operator` and `by_seed_shape` breakdowns with valid,
neutral, near-neutral, divergent, quarantine, freeze, explosion and unmeasured
counts. Adding these counters did not alter the native RNG stream or outcomes.

This turn verified 108 tests: 10 encoding tests, 16 native integration tests,
2 training tests, 35 engine, 24 rules, 5 output, 4 schema, and 12 feature-off
matcher/pipeline tests. All native-feature simulation test targets compiled.
`git diff --check` passed. Existing unrelated changes and historical receipts
were preserved; no commit or pull request was made.

```sh
cargo test -p hg-meta
cargo test -p hg-sim-core --features hg-meta --test ere_native
cargo test -p hg-train --features ere --test ere

# Use new directories: all receipts refuse overwrite.
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- gr1-e11 NEW_GATE_DIR
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- gr2-e11 NEW_GATE_DIR
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- gr3-e11 NEW_GATE_DIR
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- fixtures-e11 NEW_GATE_DIR

# Currently refuses G-R1; records the budget but executes zero training steps.
cargo run -p hg-train --features ere --bin ere-train -- configs/ere_e11.toml NEW_TRAIN_DIR NEW_GATE_DIR
```

## Scientific finding and next design boundary

Sharing identical patterns is a compact, semantics-preserving representation,
but is insufficient to make this mutation surface viable. In E1.1, AddVar,
AddPatternEdge and AddSlot still produce 549, 569 and 478 catastrophic cases
respectively; their 1,596 cases account for more than half of all catastrophes.
Many valid edits change the structure a rule must match. Shared patterns do not
provide alternative matching paths when the changed pattern ceases to occur.
These counters identify where failures concentrate, not their full per-sample
causal attribution; an unmatched label versus exhausted rewrite supply is not
yet distinguished for every freeze.

A stronger follow-up would require an explicit semantic design, such as
optional structural grafts with separately represented activation/commitment.
That is not merely an E1 storage optimization: it changes the match/activation
envelope and must characterize activation mutations as well as dormant ones.
Reporting only harmless dormant edits would game the gate. Such an E2 design,
its RCF, actual mutation kernel and new preregistration need to be specified
before implementation; none is silently introduced here and no pass is promised.

Do not enable a data-dependent decoder veto, ignore freeze outcomes, add an
undeclared fallback, narrow operators under the wrong Valid-fraction branch,
or spend the training budget to compensate for this failed gate.
