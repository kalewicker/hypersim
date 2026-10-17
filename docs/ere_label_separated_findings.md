# Label-separated ecology trial findings

## Result

The separately frozen `E2-complete-label-separated-v1` trial passed its bounded
criteria. All 48 cells completed 64 steps (3,072 total), with no capped or
aborted cells and no retries. It recorded **8,640 distinct-node world rewrites**
and 2,160 clock-free topology-changing step observations. Every world rewrite
switched a real three-node motif between a path and a star. There were zero
aliased world bindings, self-loops, cross-motif edges or valid changed-family
admissions.

All 48 paid repairs restored the exact original active whole family for one
fuel, one step after a newly damaging CORE lesion. Its two world classes were
retired during quarantine, and their topology-changing activity resumed only
after the repaired family was recompiled. Protection blocked all targeted
damage without repair payment. Fuel exhaustion stopped recovery; late ownership
loss remained invalid even with four repair fuel remaining.

This resolves the previous experiment's aliased-binding limitation. It does
**not** demonstrate a rich ecology, growth, adaptation or self-sustaining
maintenance: the substrate is four engineered, synchronized two-state motif
oscillators. No optimizer, held-out evaluation or training campaign was launched.

The one frozen invocation finished in 77.05 seconds, exit 0. Seeds
984927043621–624 give identical outcomes for each condition, and the engine RNG
never advances. These are deterministic reproducibility controls, not four
independent stochastic samples.

## Every condition

Each row includes all four seeds, with one receipt per seed. All cells requested
and completed 64 steps. A2 is protected; A3 is exposed. Ownership exposure
means a scheduled native hazard at step 40, not actual damage under A2.
"Exact-valid" means full original core/proposal/mode equality, not anchor survival.

| Arm | Initial fuel | Ownership exposure | Runs | Exact-valid steps/run | Repairs/run | Final fuel | Topology-changing steps/run | World fires/run | Core/proposal fires/run |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| A2 | 0 | No | 4 | 64 | 0 | 0 | 64 | 256 | 128 / 128 |
| A2 | 0 | Yes | 4 | 64 | 0 | 0 | 64 | 256 | 128 / 128 |
| A2 | 2 | No | 4 | 64 | 0 | 2 | 64 | 256 | 128 / 128 |
| A2 | 2 | Yes | 4 | 64 | 0 | 2 | 64 | 256 | 128 / 128 |
| A2 | 8 | No | 4 | 64 | 0 | 8 | 64 | 256 | 128 / 128 |
| A2 | 8 | Yes | 4 | 64 | 0 | 8 | 64 | 256 | 128 / 128 |
| A3 | 0 | No | 4 | 7 | 0 | 0 | 8 | 32 | 16 / 16 |
| A3 | 0 | Yes | 4 | 7 | 0 | 0 | 8 | 32 | 16 / 16 |
| A3 | 2 | No | 4 | 21 | 2 | 0 | 22 | 88 | 44 / 44 |
| A3 | 2 | Yes | 4 | 21 | 2 | 0 | 22 | 88 | 44 / 44 |
| A3 | 8 | No | 4 | 60 | 4 | 4 | 60 | 240 | 120 / 120 |
| A3 | 8 | Yes | 4 | 35 | 4 | 4 | 36 | 144 | 72 / 72 |

Final family validity is true in all 24 A2 cells and the four A3/fuel8/no-ownership
cells; false in the other 20. This includes the expected negative controls, not
only recovered trajectories.

Receipt order is seed, arm (A2/A3), fuel (0/2/8), ownership (false/true): seed621
has attempts000–011, 622 has012–023, 623 has024–035, 624 has036–047. Every
effective config, raw completed-step trace, native event/binding, error and
summary is retained. No local diagnostic is substituted for a frozen cell.

Totals: 3,072 completed steps; 2,140 exact-valid family observations; 48 paid,
non-advancing Seal preambles; 48 paid repairs; 96 exposed CORE erasures and
12 exposed ownership erasures. Of the CORE exposures, 64 caused new damage,
48 recovered, and 16 stayed damaged through insufficient fuel. Ordinary world
fires split evenly: 4,320 core and 4,320 proposal events. All exclude the 3,072
native clock events, hazards and paid operations.

## Why this trial is different

Only the new diagnostic harness, independent auditor and documentation were
added. Production codec, matcher, source/runtime, shared runner API and
training admission are unchanged from the preceding experiment.

At step-zero initialization, the existing 16 crystal substrate nodes are kept
but the ordinary edges are replaced with four disjoint three-node paths. Their
role labels are 10/11/12; the four remaining ordinary nodes are isolated label99.
The original genesis and constructed substrate are archived separately. After
this declared initialization, no host callback changes the live substrate.

World productions use labels 10/11/12 on their three variables. A single node
cannot satisfy different label constraints, so distinctness follows from the
existing grammar rather than a matcher change or selection override. Patterns
stay inside each disjoint triple. Clock, inert nodes and meta schema nodes
cannot match either world production.

Both complementary productions start active: the core switches path-to-star,
and the proposal switches star-to-path. This is deliberately seeded activity,
not runtime activation or learned novelty. The active family and less dense
labelled substrate are additional fixture changes; this is not a one-factor
causal comparison of performance against the predecessor. Counts or runtime
differences must not be attributed solely to eliminating aliasing.

The old native environment is unchanged: CORE exposures at 8/16/24/32, optional
ownership loss at40, and one clock increment per advancing step. Hazard/repair
conflicts prevent simultaneous damage and repair. The same A2 meta-node ceiling
protects the same hazard catalogue. All dynamics use the existing production
loop, ordinary matcher/conflict/apply path and pooled 50,000 work/results budgets.
Max observed work was 1,487; max results6; no ceilings bound. Pairing allowance
16 and single-thread execution were fixed, not tuned against outcomes.

## Actual topology and class pause/resume

For each accepted world binding, the audit checks three distinct role-labelled
nodes in one declared motif, both matched/deleted original edge IDs, both
created edge IDs and their actual endpoints. One logical relation remains
unchanged; the other moves between the middle/end nodes and the root/end nodes.
Ignoring allocation IDs and endpoint presentation order, that is a real
adjacency change rather than edge replacement masquerading as topology change.
Each compiled-valid step changes all four motifs; no motif gets two firings.

During a compiled-invalid step, both world classes are absent and the motifs
retain the exact same edge IDs/endpoints, not merely the same reported phase.
Both native core/proposal role counts and production reuse metrics agree with
the reconstruction; ambiguous-role reuse is empty and runtime activation is
zero. Every cell engages both roles before its first possible lesion.

Compilation precedes the step. At lesion8 the original valid table still fires
and the family becomes invalid afterwards. Before9 both world classes retire;
repair during9 restores the exact active family, but world activity remains
paused. Before10 the two original classes reinstall and the motifs resume from
their saved phase. No-fuel cells never resume; fuel-two cells resume twice,
then stop after lesion24. The extra compiled-valid boundary for unrecovered
cells explains, for example, seven valid family observations but eight
topology-changing steps in no-fuel cells. It is not a changed-body admission.

In fuel8/ownership cells, all four CORE repairs have completed by33. Erasure
of the root-to-core E_OWNS relation at40 invalidates the owned-node count/
obligations; both classes retire before41. There are no repair payments after
this loss, and four fuel remain through64. Ownership loss is detected, not
reconstructed from a hidden original-program oracle.

## Audit, verification and provenance

The independent Python auditor passed all 48 scientific receipts and 3,072
step timelines without revising its frozen checks after measurement. It checks
every world edge/phase/change, role attribution, native conflict disjointness,
metadata/fuel/event accounting, compile/step RNG, exact installed tables,
quarantine timing and unchanged node data/labels. It checks ownership and 26
per-node role obligations in 576 selected post-preamble inventories, plus48
unsealed baselines and48 original substrate-genesis inventories. All48 initial
HGSSv2 full stores roundtrip in Rust, and their frame headers/lengths pass Python
checks. The original and constructed substrate delta is verified as declared.

Sixteen in-memory tamper controls reject missing steps, false recovery, fuel
renewal, cached admission, hidden ceilings, omitted hazards, changed programs,
visibility bypass, missing payment/ownership, aliased world bindings, false
topology changes, missing world edges, wrong phases/role attribution and wrong
substrate labels. Scientific receipts are never edited for an audit pass.

Limitations: Python does not implement the full CandidateRule or bincode
decoder; exact whole-family equality also relies on the archived Rust reader
and comparison with its initial decoded program. An initial store roundtrip
is not checkpoint/resume equivalence for engine, rule source and runtime
ledgers. Generic meta-excluded graph/activity aggregates still include the
ordinary clock; clock-free topology and role activity here use separate raw
world inventories and rule-ID attribution.

Before measurement the driver archived its compiled-in contract, harness/two
helpers, independent auditor/dependency, 16 selected production/lockfile sources,
planned configs and exact executed test binary. All selected production files,
harness and checker match current sources. It also archives the second local
12-cell preflight's raw receipts/checkpoints in `development_preflight`; these
remain separate engineering evidence, not 12 additional scientific replicates.
The first local 12-cell invocation had no persisted receipts. Both local
invocations passed (17.69s/18.61s); all 12 saved local receipts were independently
checked before the frozen run. No failed local or frozen invocation occurred.

Selected verification passed 134 distinct Rust tests: 129 existing regressions
plus four new local test functions and the frozen assay. Feature/workspace
checks, new-harness formatting, Python compilation and diff checks passed. All
eight preceding witness/sealed/complete/repeated-damage evidence checksum
manifests verified unchanged. This is not a claim that every workspace test or
the independent inherited LK tests were run.

Contract SHA-256:
`b98a5cca3a005448650e05dbc28e08f3b2fe4dcdba8f63291ec0c3f90dc6ba79`.
Archived harness SHA-256:
`aa92be82b0c2cf97fd3fb638f88025aacbc758ec608d9c34c958a04c52221055`.
Frozen binary SHA-256:
`cc72873c9b2bd165f750168599fec4c2de4f7bfb324e46a4e2c8edb805d7d256`.

Evidence:

- [Contract](/Users/kalewicker/Documents/hypersim/output/ere_label_separated_20260926/preregistration.md)
- [All-attempt summary](/Users/kalewicker/Documents/hypersim/output/ere_label_separated_20260926/summary.json)
- [Independent audit](/Users/kalewicker/Documents/hypersim/output/ere_label_separated_20260926/receipt_verification.json)
- [Verification commands](/Users/kalewicker/Documents/hypersim/output/ere_label_separated_20260926/verification_tests.md)
- [Checksums](/Users/kalewicker/Documents/hypersim/output/ere_label_separated_20260926/checksums.sha256)

## Reproduction and next boundary

Use a fresh directory; neither local exports nor the frozen run overwrite
previous evidence. The development-preflight archive is optional:

```sh
cargo test -p hg-sim-core --features hg-meta --test ere_label_separated
ERE_LABEL_OUTPUT=/private/tmp/ere-label-new-run cargo test -p hg-sim-core --features hg-meta --test ere_label_separated label_separated_frozen_assay -- --ignored --exact --nocapture
python3 -B scripts/verify_ere_label_separated.py /private/tmp/ere-label-new-run --test-rejections --write-checksums
```

The next useful evidence is checkpoint/continuation equivalence: preserve
graph/fuel, engine RNG, installed classes and rule-source/runtime role and
maintenance ledgers across interruption, including damaged and exhausted
boundaries. New-profile full semantic gates must also be verified before any
training admission. Neither follow-up was launched here. More complex ecology
or ownership repair would need a new preregistered design/authorization rather
than extrapolation from these finite engineered oscillators.
