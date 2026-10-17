# E2 alternative-rule families: preregistration and implementation record

Status: implemented and tested. **G-R1 is not passed: 14 resource-bounded
classifications remain unmeasured**, despite meeting every numerical rate and
activation-engagement threshold. Training and downstream research remain
blocked. This changes the semantic envelope; it is not another E1 storage
layout or a reinterpretation of old runs. The frozen preregistration below is
retained unchanged, followed by the implementation, results and limitations.

## Frozen hypothesis and semantics

A graph-encoded family explicitly contains a core rule and one proposal. A
typed state is Dormant (only core is executable) or Active (both are executable
alternatives). Activating commits the proposal into the executable set;
deactivation removes it from that set, not from the graph. Identical executable
alternatives are deduplicated within a family, not assigned twice the weight.
Different alternatives use ordinary unit-weight compiled rules and normal
engine matching/conflict resolution. No fitness-dependent decoder, repair,
implicit fallback, schedule or custom matcher is introduced.

The core is real, visible genotype, not an immortal interpreter built-in. The
structural mutation assay chooses either core or proposal uniformly, including
while active. If either encoded body is malformed the entire family quarantines,
even when that body is dormant. Deleting either mandatory family link is invalid.
No incomplete family is silently rescued by retaining an old compiled table.

## Representation and identity

Family root: existing `L_RULE_ROOT`, `K_ENCODING = Int(3)` and typed
`K_GRAFT_MODE = 9`: Int(0) dormant / Int(1) active. New `L_ALLELE_ROOT` labels
component roots. Binary `E_CORE = META_BASE+0x42` and
`E_GRAFT = META_BASE+0x43` attach one distinct component each. Components use the
E1.1 shared-pattern body, with exactly the old strict rule semantics. Whole-family
node/edge/radius/incidence budgets apply before reading its components. Candidate
component walks exclude only the explicitly validated family links.

Executable program identity is a canonical set of ordered candidate content;
dormant proposal edits do not count as novel executable semantics. Rule novelty,
reuse and nontrivial reuse remain per executable candidate RCF, not per family.
Thus a core firing cannot credit an unused novel proposal. Family handles remain
the generation-zero survival anchors. Lifecycle counters count family installs;
rule-id attribution covers every expanded executable allele. TPM revision tracks
the actual expanded table, including activation changes.

Legacy E1 and E1.1 APIs/configurations remain available. E2 is explicit opt-in;
its receipts cannot unlock a different version's research or training.

## Precommitted characterization

Retain the 28 original cores, fixture graphs, RNG seed `0xE152310001`,
100-step trajectories, 10,000 samples, all resource limits, ≥50 neutral soundness
checks, and the arity-eight ceiling-adjacent cores. Prepare three proposals per
core, each with priority core+1:

1. Identical topology, changed priority: always-applicable control.
2. LHS variable 0 constrained to label 1: context-absent control on the original
   fixture (the decoder does not know this and still installs it when active).
3. Extra RHS binary self-edge `[0,0]`: potentially trajectory-changing activation.

Each proposal is tested in both Dormant and Active states: 168 parent states.
Sample states and the 13 operations uniformly. Eleven operations are exactly
the previous structural operations, applied to a uniformly chosen core/proposal
body. The two new operations are Activate (set mode 1) and Deactivate (set mode
0); already-satisfied draws are recorded as no-ops, not resampled.

Accept only if:

- Overall Valid ≥40%, NearNeutral ≥10%, Catastrophic ≤25%, no unmeasured cases.
- The **structural-only** stratum meets the same three thresholds.
- The **active-parent structural-only** stratum meets the same thresholds and
  has at least 2,000 samples. Adding harmless mode operations cannot dilute this
  gate, and dormant edits alone cannot unlock it.
- Activation/deactivation stratum has ≥1,000 samples, Valid ≥40%, Catastrophic
  ≤25%, ≥100 real state changes and ≥20 trajectory-changing outcomes. NearNeutral
  is reported here, not required: activation is deliberately allowed to change
  behavior. Frozen and exploding cases remain failures, not reclassified.
- Every stratum reports quarantine/neutral/near/divergence/freeze/explosion/
  unmeasured counts. Ceiling-adjacent counts and both target roles are reported.

This is a new semantic assay with explicit additional controls and operations,
not a directly paired estimate of improvement over E1's 10,000 samples. Retain
old failed receipts. Do not modify these settings after observing the result.
Training remains the existing 20×16×10,000 budget, and starts only after full
E2 gates and the existing four stateability prechecks pass. This request does not
pre-authorize a new larger optimizer or another mutation-kernel trial.

## Required mechanism checks

Dormant/active round trips; exact duplicate deduplication; activation of an
unmatched proposal leaves a matchable core; activation of a matching changed
proposal affects trajectories; core edits are not hidden; malformed dormant
proposals quarantine; nine independent validators engage; unknown family mode
and cross-family sharing quarantine; order/allocation-independent identities;
no-RNG compile; 100,000-step A0/A1 identity; protected handle survival/cleanup;
activation invalidates a TPM window; reuse attributed to the branch that fired.

## Implementation and extension guide

The implementation is opt-in through `EncodingVersion::AlternativeFamiliesV3`
and `configs/ere_e2.toml`. Default/native E1 and E1.1 continue to use their own
encodings and gate receipts. Legacy metrics CSV and bincode formats are unchanged.

| Layer | E2 responsibility / extension point |
|---|---|
| `hg-meta/src/schema.rs` | Versioned labels, typed activation mode and unchanged component bounds |
| `hg-meta/src/program.rs` | `encode_family`, `family_parts`, `read_program`, executable alternatives and canonical identity |
| `hg-meta/src/region.rs` | Bounded whole-family traversal; validated component traversal excluding family links |
| `hg-meta/src/source.rs` | Atomically validate/compile each family, expand and content-sort rule table, attribute each rule id to its own RCF |
| `hg-rules/src/source.rs` | `validated_regions` counts accepted genotype regions separately from expanded rules |
| `hg-sim-core/src/ere.rs` | Per-candidate novelty/reuse, family anchors, correct quarantine denominator, table-revision stationarity |
| `hg-sim-core/src/ere/semantic.rs` | Frozen 168-state mutation assay, independent strata, receipt validation and targeted resource-ceiling diagnostic |
| `hg-sim-core/src/ere/experiments.rs` | Versioned gates, paid seeded editor, protected fixtures and positive activation stationarity control |
| `hg-train/src/ere.rs` | Existing grammar/ES path; E2 receipts checked before stateability probes or optimizer evaluations |

`read_program` is the public decoding entry point for mixed supported versions;
the older `read` remains the single-component E1/E1.1 reader. For a prepared E2
genotype, call `encode_family(store, core, proposal, active, limits)` with two
explicit `CandidateRule`s. The standard simulation seed writer instead mirrors
each existing TOML rule into a **dormant identical proposal**. It does not secretly
initialize the experiment's three prepared proposal controls in production.

To extend the semantic envelope, update representation, whole-family validation,
candidate compilation, executable identity, per-rule attribution and mechanism
tests together. Do not add fields only to the writer or identify an expanded
table by its family root. A dormant body must still validate; a missing or bad
body rejects the family and retires its old table contribution. The encoder
prevalidates a temporary whole family, including radius and aggregate caps,
before modifying the caller's store. Shared nodes between components or families,
unknown family scaffolding, duplicate links and unknown activation modes reject.

Identity is content-derived. Active candidates are sorted by full RCF bytes and
full execution bytes; exact executable duplicates are deduplicated only inside
one family. Separate families still represent separate rule occurrences. A
family's executable RCF ignores its dormant proposal, while engine table identity
retains presentation-sensitive execution bytes. A priority-changing proposal is
a distinct candidate; a core firing cannot credit that proposal's novelty/reuse.

The versioned seeded editor is explicit static scaffolding: it consumes one
family energy unit, activates the proposal and increments its priority. Its own
rule id is not an endogenous candidate. It demonstrates that ordinary engine
rewrites can reach the genotype, not evolved computation or an endogenously
encoded editor. A2 protects meta nodes; A3 uses the same engine without that
protection. Activation of a changed proposal invalidates both TPM measurements
for the affected window. Activating an exact duplicate does not change the
executable table or falsely invalidate stationarity.

## Full characterization result — 2026-09-25

Receipts: `output/ere_e2_20260925`. All denominators below include quarantined
samples; catastrophic means frozen plus exploding. Neutral and NearNeutral are
separate categories. Classification limits and acceptance thresholds were not
changed after this result.

| Stratum | Samples | Valid | Neutral | NearNeutral | Divergent | Quarantined | Frozen | Exploding | Unmeasured |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Overall | 10,000 | 8,391 | 3,739 | 2,386 | 1,037 | 1,609 | 1,117 | 98 | **14** |
| Structural only | 8,463 | 6,854 | 2,953 | 1,911 | 890 | 1,609 | 1,007 | 79 | **14** |
| Active-parent structural | 4,250 | 3,484 | 835 | 1,660 | 534 | 766 | 377 | 64 | **14** |
| Activation/deactivation | 1,537 | 1,537 | 786 | 475 | 147 | 0 | 110 | 19 | 0 |
| Core structural target | 4,263 | 3,439 | 845 | 1,162 | 555 | 824 | 849 | 20 | 8 |
| Proposal structural target | 4,200 | 3,415 | 2,108 | 749 | 335 | 785 | 158 | 59 | 6 |

| Required rate | Overall | Structural only | Active-parent structural | Activation/deactivation |
|---|---:|---:|---:|---:|
| Valid ≥40% | 83.91% | 80.99% | 81.98% | 100% |
| NearNeutral ≥10% except mode stratum | 23.86% | 22.58% | 39.06% | 30.90%, reported only |
| Catastrophic ≤25% | 12.15% | 12.83% | 10.38% | 8.39% |
| No unmeasured cases | **Fail** | **Fail** | **Fail** | Pass |

All 13 operators engaged. The mode stratum contained **751 actual state changes**
and **276 trajectory-changing outcomes**, exceeding the preregistered 100 and 20.
Of those trajectory changes, 147 were divergent and 129 catastrophic; the latter
remain failures. All 50 neutral trajectory soundness checks passed. The
arity-eight stratum had 1,439 samples, 1,111 valid, 146 catastrophic and zero
unmeasured. Full operator and seven core-shape breakdowns are in `exp_r01.json`.

The rates meet the preregistered numerical criteria even for active structural
edits. This supports the *narrow* mechanism hypothesis that an explicit
alternative can preserve applicability after some valid core edits. It does not
establish fully measured searchability or statistical superiority to E1/E1.1:
the parent catalog and mutation kernel changed, so the assays are not paired.
Even the conservative bound assigning all 14 unresolved cases to catastrophic
would be 12.29% overall and 10.71% active-structural, but that bound is **not a
classification or permission to bypass the zero-unmeasured gate**.

## Resource-ceiling diagnosis

`output/ere_e2_diagnosis_20260925/classification_audit.json` is a post-result
**diagnostic replay, not a gate receipt or independent replication**. It replays
the identical 10,000 genotype draws and traces the 415 valid, RCF-changing,
active-parent AddPatternEdge mutants. It recovers all 14 unresolved cases with
the same 8 core / 6 proposal split. No new samples, seeds, repairs or higher
limits are used.

Every case hits the matcher budget, not the 2,000-node / 4,000-edge trajectory
size caps: **10 binding-result ceilings, 4 enumeration-work ceilings**, at
attempted steps 30–97. The unchanged engine divides the global assay budget of
200,000 work units / 20,000 results equally between executable rules. An active
two-rule family therefore receives 100,000 work units / 10,000 results per rule.
Added LHS edges can generate a large joint binding enumeration as the substrate
grows, including in a lower-priority alternative whose matches would not win
the current rewrite. Having a surviving core does not remove this matching cost.
Each cap step accepted **zero rewrites**; the engine never applies a partial pool.

The diagnostic contains each sample/state index, edited role, proposal variant,
first cap step, graph size, candidate bodies and per-rule work/result counters.
The next research question is whether better matching or budget allocation can
complete these trajectories while retaining a defined, precommitted global
budget. This implementation does not silently raise that budget, skip an
expensive candidate or claim that the 14 cases were harmless.

## Gate and experiment ledger

| Gate / experiment | State | Evidence |
|---|---|---|
| G-R0 | Pass | 20 isomorphism pairs and 20 priority-distinct pairs |
| G-R1 / EXP-R01 | **Not passed** | Full assay above, 14 unresolved classifications |
| G-R2 / EXP-R02 | Pass within envelope | 2/45 shipped rules supported, both exact 1,000-step round trips; no new strict-component expressivity |
| G-R3 / EXP-R03 | Pass | 100,000 active identity steps, every full hash and canonical per-step CSV equal for A0/A1 |
| G-R4 | Pass | Nine validators independently engaged, counters `[1;9]` |
| G-R5 | Pass | 10,000 protected steps, complete anchor survival, no cleanup loss, 32 paid seeded editor events |
| G-R6 | Pass | Activating a changed proposal invalidates both TPM variants |
| EXP-R04–R08, R10 | Not launched | G-R1 blocks four-arm/epoch/novelty/EI research |
| EXP-R09 | Budget committed, blocked | Same 20×16×10,000 budget; zero stateability-probe, optimizer or held-out evaluation steps |

The blocked campaign record is `output/ere_e2_training_20260925/manifest.json`,
`status.json`, and `report.md`. There is no trained E2 champion. Historical E1
and E1.1 failures remain in their original artifact directories.

Mechanism verification includes 8 new family tests and 7 E2 integration/gate
tests, plus existing regression coverage. The candidate-level attribution test
first activates an unmatched novel proposal and verifies that ten core fires
give that proposal **no nontrivial reuse**; removing its label constraint then
allows its own fire to receive credit. Production output and no-output rollout
use the same run loop and produce identical histories with a live E2 editor.

Verification passed **124 targeted tests**: hg-meta 18; hg-engine 35; hg-rules
24; hg-output 5; hg-schema 4; ERE simulation integration 23; ERE training 2;
and feature-off pipeline/snapshot regression 13. This is targeted coverage, not
a claim that every long-running project-wide experiment was rerun.

## Reproduction

Follow-up on 2026-09-26: explicit pooled allocation is implemented and measured
in [`ere_e2_pooled_budget.md`](ere_e2_pooled_budget.md). It rescues 6/14 bounded
cases under the same global limits, but its full policy-bound G-R1 still fails
with 8 unmeasured cases. This does not alter the split-policy receipts above.

From the repository root, use a **fresh output directory** to retain old receipts:

```sh
cargo test -p hg-meta
cargo test -p hg-sim-core --features hg-meta --test ere_e2 --test ere_native
cargo test -p hg-engine -p hg-rules -p hg-output -p hg-schema
cargo test -p hg-train --features ere --test ere
cargo test -p hg-sim-core --test phase10_pipeline --test snapshot_wire
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- gr1-e2 OUTPUT_DIR
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- gr2-e2 OUTPUT_DIR
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- gr3-e2 OUTPUT_DIR
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- fixtures-e2 OUTPUT_DIR
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- audit-e2 DIAGNOSTIC_DIR
```

`gr1-e2` currently exits nonzero after saving its truthful failed report.
`audit-e2` creates only a diagnostic artifact, never authorizing a campaign.
The training command below commits the budget/configuration and refuses before
any evaluation when the gate receipts have not passed:

```sh
cargo run --release -p hg-train --features ere --bin ere-train -- configs/ere_e2.toml CAMPAIGN_DIR OUTPUT_DIR
```

Limitations remain explicit: one mandatory proposal per family, narrow strict
component expressivity (2/45 library rules), identical dormant standard seeds,
static seeded editors rather than fully endogenous property-effect rules,
possible collisions in public 64-bit reuse-map identifiers (full bytes are used
for novelty/table ordering), and 14 incomplete mutation classifications. No
claim of open-ended evolution, self-maintenance without protection, outer-loop
improvement, or positive EI/causal-emergence research follows from these fixtures.
