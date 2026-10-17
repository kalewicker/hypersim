# Complete coherent draw-local full trial: findings

Date: 2026-09-29. Profile `E2-complete-coherent-draw-assay-v1`.
[Preserved status](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_draw_full_20260929/run_status.json).

## Verdict

The single explicitly approved scientific invocation processed all 10,000
primary draws. It ended **failed/unmeasured**, with gate and training false,
after 3,305.631 seconds (55 minutes 5.631 seconds) of producer wall;
outer invocation wall was 3,306.11 seconds.

There are two independent blockers:

1. The frozen full auditor exceeded its 1 GiB RSS ceiling in
   `check_native_timing`, which materializes all classification records.
   No completed full-audit receipt or root completion seal was written.
2. Native draw 9730 hit the 400,000-work pooled matching ceiling at step 92.
   Its trajectory has only 91 measured/advancing steps and remains Unmeasured.
   That alone violates zero-Unmeasured acceptance, independently of the
   auditor bug. It cannot be dropped, resampled or relabeled as a completed
   NearNeutral/Catastrophic result.

This is neither scientific acceptance nor a completed, fully audited kernel
negative. Retain the native observations as non-admissible evidence.
No trial retry, additional Engine run, criterion/cap change, repair of the
failed archive or training occurred after failure.

## Authorization and immutable provenance

The user approved one 10,000-draw × 100-step trial under the unchanged caps.
[Separate launch authorization](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_draw_full_authorization_20260929.md)
records “I approve it”. Prospective documents and preparation receipts remain
unchanged historical records of their earlier pending-approval state.

Frozen contract Markdown SHA256:
`8a0e97586c4c7f84c28c105b246ffaeb8688618ae659c9ea3716ed5b7bba1d46`.
JSON SHA256:
`80c8cbc072a393d8bddd921b5c4dbf88ac60e8e4b1ebb50dfb7a0683aa967dba`.

Exactly one release/locked invocation ran the ignored
`complete_coherent_draw_frozen_assay` test with one test thread and exact
step/cap/contract approval tokens. Cargo exits 101 after the frozen auditor
fails; its generic “to rerun” suggestion was not followed.
[Captured final execution output](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_draw_full_execution_20260929.txt).

## Persisted native observations

These counts describe the native material; a full audit-pass receipt is absent.

| Quantity | Native result |
| --- | ---: |
| Primary draws | 10,000, all classified |
| Damage controls / matched clean siblings | 5,376 / 5,376, native and frozen stage audit pass |
| Relation roles / obligation keys | 13 / 26, explicit zero included |
| Parent baselines | 168, all 100-step complete |
| Selected children | 4,886 |
| Complete children | 4,885, each 100 steps |
| Incomplete child | `child_9730`, matching cap, 91 measured steps |
| Engine attempts / advances | 505,392 / 505,391 |
| Neutral soundness checks / failures | 50 / 0 |
| Actual mode changes / trajectory-changing mode edits | 824 / 298 |

Outcomes retain every attempted draw:

| Outcome | Count |
| --- | ---: |
| Neutral | 3,832 |
| NearNeutral | 2,373 |
| Catastrophic | 1,297 |
| Divergent | 1,165 |
| Rejected | 1,332 |
| Unmeasured | 1 |

Valid is 8,668/10,000, including the one valid-but-Unmeasured draw.
Its inclusion in validity and all applicable denominators is unchanged.
Do not interpret 99.99% measured primary coverage as satisfying the
zero-Unmeasured requirement.

[Native classification summary](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_draw_full_20260929/classification_summary.json)
has `numerical_pass:false` and `parent_baselines_complete:true`.
All 10,000 classification rows and completed gzip streams are persisted.
The failure status's `primary_mutations:"partial"` is a coarse file-existence
fallback from the producer error branch; the persisted manifest, summary and
row count establish that all requests were processed. We did not rewrite that
status label or invent a completed audit.

### Native threshold inventory

Native validity, required NearNeutral fractions, catastrophe fractions,
sample minima, operator/role/state/cap coverage and mode/soundness counts
meet their respective lines. Zero Unmeasured does not.
The table uses attempted-draw denominators, including rejected/incomplete draws.
These diagnostic ratios are not a substitute for completed independent audit.

| Stratum | Samples | Valid | NearNeutral | Catastrophic | Unmeasured |
| --- | ---: | ---: | ---: | ---: | ---: |
| overall | 10,000 | 86.68% | 23.73% | 12.97% | 1 |
| structural | 8,376 | 84.10% | 22.05% | 13.88% | 1 |
| active_parent_structural | 4,159 | 84.44% | 37.68% | 10.96% | 1 |
| topology_operator | 5,290 | 74.82% | 16.77% | 17.84% | 1 |
| active_parent_topology_operator | 2,571 | 74.83% | 34.50% | 14.04% | 1 |
| actual_pattern_change | 4,812 | 76.52% | 18.43% | 19.62% | 1 |
| active_parent_actual_pattern_change | 2,345 | 76.63% | 37.83% | 15.39% | 1 |
| activation | 1,624 | 100.00% | not required | 8.25% | 0 |

Sample minima remain 2,000 active structural and active topology,
1,000 activation and actual pattern-change, and 500 active pattern-change.
All 13 operators, both Core/Proposal roles, both activation states and both
at-cap/off-cap populations are present. Mode-change minima 100/20 and
Neutral-check minimum 50 with zero failures are met.
The one Unmeasured draw appears in every applicable structural/topology/pattern
stratum and blocks their predicates as well as overall acceptance.

## Matching-cap case: exact scope

Index 9730, parent 139, shape 5, active parent, off-cap parent,
Proposal `AddPatternEdge`; coherent transaction status Applied, admitted and
decoded, executable RCF changed. This is not an invalid transaction or a
Neutral soundness counterexample.

The request selects side `NodeId(30v1)` and repeats variable node
`NodeId(42v1)` as both endpoints. In the frozen parent's typed metadata,
the side is LHS (`L_SIDE_L = META_BASE + 2`) and the variable has
`K_VAR_ID=3`: the additional Proposal pattern edge is `[3,3]`.
The original Proposal LHS is `[[0,1,2]]`, with priority 1 and a var0
label/energy guard. The Core has the same original pattern with priority 0.

The terminal record says `complete:false`, `reason:"matching_cap"`,
`measured_steps:91`, `exploding:false`, `frozen:false`.
At attempted step 92, store step remains 91:

| Recorded item | Value |
| --- | ---: |
| Ordinary world nodes / edges | 159 / 279 |
| Pooled work / results | 400,000 / 372 |
| Core work / results | 8,462 / 372 |
| Proposal work allowance / consumption | 391,538 / 391,538 |
| Proposal results / truncated | 0 / true |
| Matching ceiling / graph-store cap | true / false |

Thus work, not the 50,000-result or 2,000/4,000-store cap, binds.
The repeated-variable LHS requirement is a concrete bounded matcher-analysis
target; these observations do not by themselves prove a particular optimization
or the counterfactual 100-step outcome. No capped-cell replay was performed.

## Auditor failure and verification boundary

[Exact frozen timing checker](/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_draw_full_20260929/source_snapshots/scripts/verify_ere_complete_coherent_draw_assay.py:1505)
contains:

```python
classifications = list(stream(out / "classifications.jsonl.gz"))
```

The timing check only needs ordered non-null trace IDs, but retains the
complete classification, material, request/receipt, counterfactual and compiled
records. This is a full-scale memory bug missed by the pilot-sized regression
and resource evidence. The first actual row is 98,511 canonical wire bytes
and about 980,431 bytes of Python object storage in a read-only illustrative
size measurement; that is not peak RSS or a rigorous all-row memory bound.

The actual frozen process fails at `stream -> guard` with
`AssertionError: auditor RSS cap`. Exact peak RSS is not persisted:
the full audit exits before its success receipt records resource statistics.
The 1 GiB guard was enforced, not raised or bypassed.

The traceback reaches timing after the frozen control flow has replayed
parents/children/classifications, recomputed the summary and checked pilot
parity. It therefore indicates earlier checks completed in that process.
This control-flow inference is not a durable full-audit-pass receipt.
The post-timing 27-category tamper suite and final sealing were not reached.
Do not transfer the pilot's 27-category pass to this full invocation.

A streaming ID/order coverage check is the smallest proposed engineering
correction: retain every timing/count/order assertion without retaining all
row payloads. That correction has **not** been implemented or rerun here.
Even a corrected auditor cannot erase the separate Unmeasured matching case.

## Resources and archive preservation

| Resource | Observed | Approved cap |
| --- | ---: | ---: |
| Producer wall to failure | 3,305.631 s | 3,600 s |
| Outer invocation wall | 3,306.11 s | Separate wrapper observation |
| Attempted / advancing Engine steps | 505,392 / 505,391 | 1,016,800 each |
| Output | 262,464,467 bytes (250.31 MiB) | 4 GiB |
| Full auditor RSS | Exceeded guard; exact peak unavailable | 1 GiB |

No total-wall, output or campaign-step limit caused termination.
The per-step matching work cap and full-auditor RSS cap bind independently.
Shared auditor duration is not durably recorded on this error path, so no
exact shared-time or peak-memory measurement is fabricated.

Native phase clocks are attested diagnostics: controls 15.787 s,
parent phase 73.618 s, mutation phase 2,381.802 s, with children 2,182.913 s
and nontrace remainder 198.889 s. Child/nontrace are subsets, not additional
wall terms. The capped child took 1.112 s for 92 attempted steps.
No completed full-auditor phase receipt exists.

All 328 frozen inputs, including authorization, contracts, sources, binary and
sample manifests, verify against their hashes. The snapshot contains 207
source files. All 17 relevant earlier receipt seals verify unchanged.
The failed archive contains 340 files. Its failed status and native evidence
are preserved without a completion seal or any resealing.

[External failure-preservation inventory](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_draw_full_20260929_preservation.json)
captures SHA256 for every one of those 340 files. It is explicitly **not**
a completion seal or acceptance receipt and lives outside the failed archive.

At postflight, 203/207 frozen source files match the live checkout. Four
live files differ: `crates/hg-core/src/arena.rs`, `crates/hg-core/src/graph.rs`,
`crates/hg-sim-core/Cargo.toml` and `crates/hg-sim-core/src/lib.rs`.
Those live changes were not made or reverted by this task. The frozen inputs
themselves remain intact, and the run used its copied binary and frozen checker.
Reproduction/extension must identify the frozen revision or prospectively pin
a deliberately chosen new revision; do not claim all live sources still match.

[Machine-readable findings](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_draw_full_trial_findings_20260929.json)
preserve counts, thresholds, capped-case evidence and source-drift hashes.

## Next boundary

First propose and validate the streaming timing-check correction with real
full-size payload/ownership/order/memory regressions. Separately diagnose the
`[3,3]` Proposal matching-cap case in a bounded engineering harness under the
existing 400k/50k envelope. Preserve exact matching/identity/RNG semantics and
all scientific denominators. Neither automatically increasing the budget nor
excluding that draw is an acceptable repair of this trial.

Any subsequent scientific run/revalidation needs a separately specified,
approved scope and fresh provenance. The failed frozen archive remains failed;
training requires the remaining gates and separate admission/authorization.

