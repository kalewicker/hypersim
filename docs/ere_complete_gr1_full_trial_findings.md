# Full complete-profile G-R1 mutation trial

Trial date: 2026-09-27; report finalized 2026-09-28. Profile
`E2-complete-mutation-classification-v1`,
encoding `E2-witness-complete-v1`, family 10.

Status: **complete, independently audited numerical failure**. All 10,000 draws
are classified, measurement is complete, native and frozen stage/full audits pass,
and the output archive is sealed. G-R1 fails its mandatory structural and
active-parent structural lines. `gate_pass=false`, `training_admitted=false`.
This is a verified scientific result, not a capped/incomplete run or null result.

## Authorization and unchanged contract

The user authorized one full invocation after development verification, replying
“ok let's do the full trial”. The dated authorization is
`ere_complete_gr1_full_authorization_20260927.md`. The output directory is
`output/ere_complete_gr1_full_20260927`. This is the first full invocation of
this complete-profile G-R1 design, not a scientific retry.

The immutable `ere_complete_gr1_v1.md` and `.json` remain unchanged. Their
historical design-time authorization/status fields are not the current run
status. The separate authorization and current manifest record the launch.
No source, seed, operator, threshold, denominator, horizon, matching/store limit
or admission criterion was changed during this trial. No repair, resampling,
automatic retry, prior-archive promotion or training invocation occurred.

The release harness uses 10,000 unrepaired draws from the frozen 168-parent bank,
100-step horizons, all 5,376 separate damage controls and the first 50 eligible
Neutral soundness checks. Maximum attempted/advancing steps are 1,016,800;
the wall/output/shared-auditor/RSS ceilings are 3,600 seconds / 4 GiB /
900 seconds / 1 GiB. Matching is classification-only pooled 400k/50k.
Production runtime remains pooled 50k/50k. Build and freeze are outside the assay
wall timer; frozen files and the exact binary still count toward output bytes.

Preflight reverified the development archive's 194 frozen inputs, 205 sealed
outputs and 177 unchanged live source files. Six ordinary Rust tests passed;
the smoke and full entry points are ignored unless explicitly selected.

## Verified classifications

All 10,000 draws are retained. All 168 parent baselines complete their horizons;
there are zero Unmeasured draws. Valid includes every outcome except Quarantined,
and rates use all draws in the respective stratum, not only Valid draws.

| Exclusive outcome | Draws | Fraction of all 10,000 |
| --- | ---: | ---: |
| Quarantined | 5,845 | 58.45% |
| Neutral | 2,739 | 27.39% |
| NearNeutral | 1,057 | 10.57% |
| Catastrophic | 178 | 1.78% |
| Divergent | 181 | 1.81% |
| Unmeasured | 0 | 0.00% |
| Valid, overlapping subtotal | 4,155 | 41.55% |

The 178 Catastrophic draws comprise 155 frozen and 23 exploding trajectories,
with zero overlap. Catastrophic is their union, counted once only after a
measured child differs from its parent under the declared precedence.
Neutral is executable-RCF identity, with completed baseline evidence;
NearNeutral is RCF-distinct but identical measured world hash/rewrite-count
vectors, also checked against typed world state. These are fixed-substrate,
100-step assay outcomes, not optimizer fitness or learned adaptation.

## Exact acceptance checks

The following percentages are display values only. Acceptance uses the
preregistered integer predicates `5*Valid >= 2*N`, `10*NearNeutral >= N`
where required, and `4*Catastrophic <= N`.

| Cohort | N | Valid | NearNeutral | Catastrophic | Numerical lines |
| --- | ---: | ---: | ---: | ---: | --- |
| Overall | 10,000 | 4,155 (41.55%) | 1,057 (10.57%) | 178 (1.78%) | All three pass |
| Structural, operators 0..10 | 8,442 | 2,597 (30.76%) | 534 (6.33%) | 69 (0.82%) | Valid and NearNeutral fail |
| Active-parent structural | 4,265 | 1,315 (30.83%) | 352 (8.25%) | 48 (1.13%) | Valid and NearNeutral fail |
| Activation/deactivation | 1,558 | 1,558 (100.00%) | 523 (33.57%, descriptive) | 109 (7.00%) | Required Valid/catastrophe lines pass |

The failing exact comparisons are:

- Structural Valid: `5*2597 = 12985 < 2*8442 = 16884`.
- Structural NearNeutral: `10*534 = 5340 < 8442`.
- Active-parent structural Valid: `5*1315 = 6575 < 2*4265 = 8530`.
- Active-parent structural NearNeutral: `10*352 = 3520 < 4265`.

The active-structural cohort exceeds its 2,000-draw minimum; activation exceeds
its 1,000-draw minimum. There are 769 actual mode changes and 246 measured
trajectory-changing mode edits, exceeding 100 and 20. All 50 selected Neutral
soundness comparisons pass with zero failures. Both target roles, both parent
activation states, all 13 operators, and both at-cap/off-cap strata occur.

At-cap means either allele has LHS or RHS arity 8; it is not an observed resource
cap. The at-cap stratum has 1,463 draws, 609 Valid, 147 NearNeutral and four
Catastrophic. Off-cap has 8,537 draws, 3,546 Valid, 910 NearNeutral and 174
Catastrophic. These are descriptive splits without additional thresholds.

## Operator-level evidence

The table is a disjoint primary partition. Damage controls never enter it.

| Operator | N | Quarantined | Neutral | NearNeutral | Catastrophic | Divergent |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| AddSlot | 798 | 798 | 0 | 0 | 0 | 0 |
| RemoveSlot | 784 | 784 | 0 | 0 | 0 | 0 |
| RetargetSlot | 792 | 512 | 280 | 0 | 0 | 0 |
| AddVar | 749 | 749 | 0 | 0 | 0 | 0 |
| RemoveVar | 735 | 735 | 0 | 0 | 0 | 0 |
| AddPatternEdge | 696 | 696 | 0 | 0 | 0 | 0 |
| RemovePatternEdge | 756 | 756 | 0 | 0 | 0 | 0 |
| AddLabel | 809 | 649 | 160 | 0 | 0 | 0 |
| RemoveLabel | 787 | 166 | 621 | 0 | 0 | 0 |
| ChangeLabel | 772 | 0 | 697 | 36 | 34 | 5 |
| ChangePriority | 764 | 0 | 192 | 498 | 35 | 39 |
| Activate | 770 | 0 | 388 | 267 | 57 | 58 |
| Deactivate | 788 | 0 | 401 | 256 | 52 | 79 |

The primary invalidity reports are 5,047 Members and 798 Bounds failures;
no different validator names are inferred from an operator's intent. Both roles
are affected: Core has 4,292 draws / 1,317 Valid / 2,975 Quarantined; Proposal
has 4,150 draws / 1,280 Valid / 2,870 Quarantined. Dormant material is not exempt
from complete decoding, and no invalid dormant edit was silently discarded.

## Completeness controls and evidence integrity

All 5,376 damage attacks and matched clean siblings pass native checks and the
frozen control-stage audit. They cover all 13 relation roles, all 26 obligation
keys and explicit-zero obligations. Present invalid roots install zero world
classes; root deletion/relabel shows retirement rather than a fabricated
quarantine count or cached installation. This matrix uses zero advancing Engine
steps and is separate from primary mutation rates.

The stage barrier records successful native/frozen controls, zero advancing
steps and mutations not yet started at that boundary. The frozen final auditor
reconstructs all classifications, world transitions, strata and integer
acceptance checks; it agrees with the native summary and rejects all 18 distinct
tamper categories. Its `audit_pass=true` is an evidence-validity result, not a
scientific numerical pass. Native and audited `measurement_complete=true`,
`numerical_pass=false`, `gate_pass=false` and `training_admitted=false` agree.

All 194 frozen SHA256 entries, 177 live source comparisons and 205 sealed output
hashes verify. The output inventory is exact, including the nested basis checksum;
there are no bytecode-cache files. The 168 genotype parents have 112 unique
executable RCF identities and 112 unique executable presentation identities;
no duplicates were dropped. Prior archives and all immutable source/basis pins
remain unchanged.

## Actual resources

| Resource | Actual | Approved ceiling |
| --- | ---: | ---: |
| Attempted / advancing Engine steps | 163,400 / 163,400 | 1,016,800 |
| Assay elapsed, pre-seal status | 1,462.30 s | 3,600 s |
| Shared control/full auditor time, pre-seal status | 787.01 s | 900 s |
| Frozen full auditor peak RSS | 242,712,576 bytes | 1,073,741,824 bytes |
| Final on-disk archive including seal | 37,818,812 bytes | 4,294,967,296 bytes |
| Peak matching work / results per step | 18,396 / 2,388 | 400,000 / 50,000 |
| Total matching work / results | 308,446,700 / 31,947,601 | Per-step limits apply |

The invocation completed in 1,496.39 s including freeze. All 168 parent traces
and 1,466 child traces complete 100 steps each: 16,800 + 146,600 = 163,400 steps.
The children comprise all 1,416 RCF-distinct Valid draws plus 50 Neutral
soundness tripwires. Remaining eligible Neutral draws use the completed parent
cache rather than incurring an unnecessary child replay. Quarantined draws have
read/compile evidence but no advancing trajectory. This is the declared budget
accounting, not dropped samples or shortened horizons.

A read-only post-run extraction reconciles all 1,634 unique trace endings,
all complete at 100 measured steps with no cap reason, and all 10,000
classification rows. The 12 checker regression groups also pass when run from
the full archive's frozen scripts against the existing development receipt;
that suite's development-archive fixture is not misrepresented as a full-data
audit. The full-data verification is the separately completed frozen full auditor.

Status time/byte/auditor counters are recorded before the final seal phase;
the producer checks all ceilings again after sealing. The successful process
exit and sealed receipt confirm that the final seal also fits. No matcher/store,
wall/output or shared-auditor cap forced Unmeasured or stopped verification.
The separate damage matrix still consumes zero advancing Engine steps.

## Interpretation and scope

The verified result is a restricted-kernel/redesign branch: the aggregate lines
pass, but structural navigability does not meet the preregistered requirements.
It is not a borderline pass and must not be rescued by removing operators,
dropping invalid draws, refreshing metadata, relaxing thresholds or changing
the denominator after measurement.

The logical-topology partition makes the limitation particularly clear: every
one of the 5,845 topology-changing draws is Quarantined, and all 4,155 Valid
draws preserve this label/incidence topology. Property, priority and activation
changes can still alter execution without changing topology. The partition is
the frozen assay's definition, not a claim that genotype or behavior is unchanged.
There are 1,382 inapplicable draws and 2,229 raw-unchanged draws, all honestly
retained as Neutral; applicable but behaviorally neutral edits are also retained.

The separate protection matrix and structural rejection rates address different
questions. Strict complete ownership/incidence checks reject incomplete edits;
the declared raw kernel deliberately does not update their obligations. Thus
protection can work while this particular unrepaired mutation kernel remains
poorly navigable. This is not evidence that all possible complete-profile
mutation kernels are unsearchable, nor proof of broad expressivity, maintenance,
learned novelty, optimizer signal/noise or independent replication.

Baseline behavior is another scope limit: 68 parent traces satisfy the frozen
predicate and four satisfy the exploding predicate, while still completing
their declared horizons. Equivalent child/parent inactivity or growth can
therefore be Neutral/NearNeutral by precedence. No evidence here substitutes
for a new optimizer semantic-effect, genotype-distribution or signal/noise
precheck. A low catastrophe fraction is not proof that every parent is active
or ecologically suitable.

The appropriate next design is a separately named, prospective coherent-edit
kernel: typed structural
operators that update their own ownership/incidence/witness records together,
with explicit raw-kernel counterfactuals and damage-rejection controls retained.
That would be a new scientific question and needs its own design and approval;
it is not a repair or rerun of this receipt. No such kernel was implemented here.

Training admission remains closed. The reserved optimizer budget was not used.
