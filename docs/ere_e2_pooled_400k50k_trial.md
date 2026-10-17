# E2 pooled 400k/50k classification trial

Preregistered 2026-09-26 before this trial's replay or full assay. User authorized
one 400,000-work / 50,000-result pooled classification trial. Protocol:
`E2-alternatives-pooled-400k50k-v1`.

## Frozen scope and acceptance

This is a diagnostic-guided prospective resource-envelope expansion, not a new
genotype, matching algorithm, mutation kernel, biological improvement, independent
replication, or retroactive pass for the earlier failed receipts. Split and pooled
200k/20k defaults and all production/training settings remain unchanged.

- Global per-step matching limits: 400,000 enumeration-work units and 50,000
  returned bindings; pooled deterministic installed-rule order, unchanged matcher.
- Same 168 E2 parent states, 10,000 ordered mutation draws, seed `0xE152310001`,
  100-step trajectories, 13 operators and target-role sampling, fixtures, graph
  caps (2,000 nodes / 4,000 edges), neutral/freeze/explosion definitions, and
  structural, active-structural and activation strata as E2.
- Same acceptance: zero unmeasured cases; valid >=40%, NearNeutral >=10%,
  catastrophic <=25%; the existing independent semantic-stratum checks,
  ceiling-adjacent coverage, and 50 successful neutral soundness checks.
- First replay exactly the eight remaining sample indices: 1136, 2518, 3183,
  4554, 6859, 7127, 8380, 8679. Regenerate the original draws and reproduce their
  original split failures. Compare every completed step's full graph hash,
  accepted/attempted counts and RNG state against the unchanged diagnostic-only
  split oracle (20,000,000 work / 2,000,000 results). Retain graph caps and horizon.
- Halt on correctness disagreement or any cap violation. Otherwise run exactly
  one full classification assay, even if some replay cases still cannot complete.
  No further limit increases, resampling, or semantic tuning in this trial.
- Compare the full receipt to the old pooled assay: document every classification
  delta and require all previously measured outcomes/counters to remain unchanged.
- Record elapsed wall time and aggregate/peak matcher usage across traced parent
  and child trajectories (including neutral soundness checks). These counters are
  instrumentation, not additional acceptance thresholds or process-memory metrics.

## Receipt isolation and downstream boundary

Write new artifacts to `output/ere_e2p_400k50k_20260926` and
`output/ere_e2p_400k50k_probe_20260926`; preserve historical receipts. This
classification-only protocol does not satisfy the existing pooled-v1 research or
training unlock. Full expressivity, identity and protection gates remain a
follow-up decision after assessing this trial. No optimizer, proposal-grammar
change, stateability probe or self-maintenance experiment is launched here;
the reserved 20 x 16 x 10,000 training budget is untouched.

## Reproduction

```sh
cargo test -p hg-sim-core --features hg-meta --test ere_pooled --test ere_e2 --test ere_native
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- replay-e2p-400k50k output/ere_e2p_400k50k_probe_20260926 output/ere_e2_diagnosis_20260925/classification_audit.json
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- gr1-e2p-400k50k output/ere_e2p_400k50k_20260926
jq -n --slurpfile old output/ere_e2p_20260926/exp_r01.json --slurpfile trial output/ere_e2p_400k50k_20260926/exp_r01.json --slurpfile audit output/ere_e2_diagnosis_20260925/classification_audit.json --slurpfile replay output/ere_e2p_400k50k_probe_20260926/pooled_replay.json -f scripts/verify_ere_400k50k.jq
```

## Results

**Classification passed at the newly preregistered envelope.** The original
split E2 and pooled 200k/20k receipts remain failed and unchanged.

### Eight-case paired replay

All eight original genotype draws and split failure steps were reproduced.
All eight pooled trajectories completed 100 steps. Every completed step matched
the diagnostic oracle's full canonical graph hash, accepted/attempted counts and
engine RNG probe. Aggregate matching limits were respected. Replay elapsed time
was **8.521 seconds**, excluding compilation.

| Original sample index | Edited role | New classification | Peak work | Peak results |
|---|---|---|---:|---:|
| 1136 | Core | NearNeutral | 191,734 | 20,994 |
| 2518 | Core | NearNeutral | 191,734 | 20,204 |
| 3183 | Proposal | NearNeutral | 233,242 | 194 |
| 4554 | Core | NearNeutral | 323,093 | 40,239 |
| 6859 | Core | NearNeutral | 171,934 | 20,994 |
| 7127 | Core | NearNeutral | 357,406 | 1,194 |
| 8380 | Core | NearNeutral | 323,093 | 40,239 |
| 8679 | Core | NearNeutral | 233,242 | 392 |

NearNeutral means the executable family changed but the measured 100-step graph
hash/rewrite-count trajectory matched its parent. It is not byte-neutral and
does not assert equivalence on other substrates or beyond that horizon.

### Full assay

The single full trial completed in **146.304 seconds**, excluding compilation.
Rates use all 10,000 draws, including quarantined cases, as the denominator.

| Outcome | Pooled 200k/20k | Pooled 400k/50k |
|---|---:|---:|
| Valid | 8,391 / 83.91% | 8,391 / 83.91% |
| Neutral | 3,739 | 3,739 |
| NearNeutral | 2,387 / 23.87% | **2,395 / 23.95%** |
| Divergent | 1,042 | 1,042 |
| Catastrophic | 1,215 / 12.15% | 1,215 / 12.15% |
| Quarantined | 1,609 | 1,609 |
| Unmeasured | 8 | **0** |
| Neutral soundness checks | 50/50 | 50/50 |
| Classification acceptance | Failed | **Passed at new limits** |

All previously unmeasured draws were active-parent AddPatternEdge edits. Seven
targeted Core and one Proposal. The exact full JSON comparison, normalizing only
trial metadata and acceptance and applying these eight expected counter changes,
matched every remaining operator, seed-shape and semantic-stratum field. The
read-only verifier independently recomputed the numerical/stratum acceptance
conditions and checked replay correctness and strict aggregate caps. This is
paired completion evidence, not independent replication or biological progress.

| Stratum | Samples | Valid | NearNeutral | Catastrophic | Unmeasured |
|---|---:|---:|---:|---:|---:|
| Structural | 8,463 | 6,854 / 80.99% | 1,920 / 22.69% | 1,086 / 12.83% | 0 |
| Active-parent structural | 4,250 | 3,484 / 81.98% | 1,669 / 39.27% | 441 / 10.38% | 0 |
| Activation/deactivation | 1,537 | 1,537 / 100% | 475 / 30.90%, descriptive only | 129 / 8.39% | 0 |

Activation evidence remains 751 actual mode changes and 276 trajectory-changing
activations/deactivations. All 13 operators, both target roles, 1,439
ceiling-adjacent samples and 1,111 valid ceiling-adjacent cases remain unchanged.

### Resource usage and verification

Across 4,870 traced trajectories (168 parents, 4,652 nonneutral valid children,
and 50 neutral soundness children), all **487,000 attempted steps completed**.
Peak per-step use was **357,406 work / 40,239 bindings**; totals were
1,068,607,078 work units and 96,254,365 bindings. There were zero matcher caps,
graph caps or aggregate-budget violations. These are matcher-accounting metrics,
not process memory, CPU instructions or a throughput comparison against another
build. No peak-memory benchmark or independent timing replicate was run.

**46 targeted default-build tests passed**: ERE simulation 32 (native 16,
E2 seven, pooled nine including three new trial checks), pooled engine 11, and
ERE training three. **10 additional nonparallel pooled-engine tests passed**.
Checks include short-assay equality, distinct resource-envelope metadata,
research-unlock rejection, overwrite prevention, preregistered target validation,
and the existing matching/RNG/cap protections. `git diff --check` passed.
SHA-256 checks confirmed all four historical baseline/diagnostic receipts stayed
unchanged. The trial executable SHA-256 was
`40ad638478d4cf3d6e4624c5d046c36e8124d7be49b417300184aa511c4a35e5`.

### Artifacts and next boundary

- [Full classification receipt](../output/ere_e2p_400k50k_20260926/exp_r01.json)
- [Paired replay receipt](../output/ere_e2p_400k50k_probe_20260926/pooled_replay.json)
- [Trial report](../output/ere_e2p_400k50k_20260926/report.md)
- [Verification record](../output/ere_e2p_400k50k_20260926/verification.json)
- [Research status](../output/ere_e2p_400k50k_20260926/research_status.json)
- [Read-only paired verifier](../scripts/verify_ere_400k50k.jq)

No full downstream pooled gates or optimizer runs were launched in this
classification-only trial. The next decision is to verify full pooled
expressivity (G-R2), identity (G-R3), and protection/validator/stationarity fixtures
(G-R0/4/5/6), with an explicitly admitted resource-envelope protocol. Existing
research/training admission intentionally still rejects this distinct receipt:
it cannot be substituted into the original pooled-v1 campaign. Production
proposal variation/activation and stateability/self-maintenance controls also
remain prerequisites before meaningful E2 training. The reserved
20 x 16 x 10,000 optimizer budget has incurred zero evaluations.

Subsequent user-authorized verification on 2026-09-26 completed the full pooled
G-R2/G-R3/G-R0/4/5/6 gates. See
[the explicit gate association and scope limits](ere_e2_pooled_full_gates.md).
The classification-only record above remains the status at the end of this
trial; no historical receipt or training admission policy was rewritten.
