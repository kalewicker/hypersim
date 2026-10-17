# Categorical versus ES scientific trial: native complete, frozen audit failed

Date: 2026-10-03. Profile `E2-categorical-versus-es-direction-v1`.

## Outcome and evidence boundary

The user approved the next fixed 12,288-evaluation scientific comparison after
the verified launch bridge. One invocation completed all native work. Its
prospectively frozen accounting auditor failed on categorical raw-update f32
bits. The failed checker result is preserved; no checker/source correction,
physics retry, replacement, extra seed, or training update occurred.

**This is native-complete, audit-failed evidence, not an audited scientific
success.** The native summary separately reports
`support_categorical_direction:false`: all three sign-test endpoints pass in
that summary, but the unchanged weighted-validity point guard fails. Even a
future successful receipt-only correction must retain that negative verdict
unless independent reconstruction reveals a genuine accounting error. Do not
relax the guard or count this as admission to training.

Archive: [`output/ere_categorical_comparison_scientific_20261003`](../output/ere_categorical_comparison_scientific_20261003).
Its final SHA-256 root, including the preserved failure receipt, is
`387a6e32a152250a081b246e53e5a69adbd273c4f465f726594a256ca3600fc1`.
All **12,883 sealed entries** verify. Final archive size is **1,643,691,002 bytes**
(including the seal), below the registered 2 GiB limit. Integrity verification
is not semantic-audit acceptance.

## Authorization, frozen inputs, and single launch

The numerical config remains
`configs/ere_categorical_comparison_v1.json`, SHA-256
`b26d94f1d889018ff4baa79f81f68a82c1d8c838a0375e080e3de99bb81d4328`.
The nine-entry preregistration root remains
`dcf4ec69f2ea647f1f7eeda9185ae526788a553639fd490395da7f98933b5d57`.

Before launch, the development, bounded-auditor verification, bridge preflight,
and bridge verification seals were rechecked at their registered roots:

| Receipt | Entries | Root |
| --- | ---: | --- |
| Native development, 2026-10-02 | 692 | `3a9d48f7059dd910a8e008a566237cbf03935790629902f1b9305d496ecf3682` |
| Bounded auditor v4, 2026-10-02 | 29 | `5c5cbc9f902fa6dde7505a69f8df0fb8b32881ebbb6a3ac15a553c2fd3091671` |
| Full-layout bridge preflight, 2026-10-02 | 12,786 | `2b47e9b8d6895a3833c9f2a2ba2b3e960a83ccb7468124899ab16706c42259fb` |
| Bridge verification, 2026-10-02 | 5 | `c6f7623fd8434690a915aa6962fb204aea1ce07c477f33013990db3e3dfd9979` |

All 481 registered live source hashes and the native binary exactly matched
the frozen bridge preflight before launch. The source relation remains 470
historical paths identical, two exact additive exports, and nine registered
additions. The binary SHA-256 is
`491e1cf14d62d7b8c02b28b0cc04a1fd6e40ab2b8c5b94f9b622b88187b64938`.
The final archive's 481 source snapshots still match live files, and its copied
binary matches that identity. No implementation source was edited in this turn.

Selected-mode seed inventory found seven prior explicit schedule archives and
4,434 conservatively reserved seed identities, with no scientific collision.
The frozen launcher checked the inventory again immediately before native
execution. The authorized archive now reserves its entire scientific schedule;
it must not be reused as if it were an unauthorized zero-work preflight.
The inventory is scoped to explicit root-level schedule lists, not all implicit
RNG draws or an atomic cross-process reservation service.

The single command used the frozen preflight launcher:

```sh
python3 -B output/ere_categorical_comparison_launch_bridge_preflight_20261002/source_snapshots/scripts/run_ere_categorical_comparison_bridge.py \
  --output output/ere_categorical_comparison_scientific_20261003 \
  --mode scientific --execute \
  --development-receipt output/ere_categorical_comparison_development_20261002 \
  --bridge-preflight output/ere_categorical_comparison_launch_bridge_preflight_20261002
```

The launcher ultimately exited 1 because its frozen audit failed. That does not
erase or reclassify the already-complete native manifest. Distinguish the
native manifest's `status:complete` from the failed end-to-end audit/launcher.

## Fixed work and resources

Two fixed length-183 contexts, eight paired estimator blocks, two methods and
two independent estimates per method/context were retained. Estimation finished
before any validation. Every proposal used the registered group-centering and
f32-rounded L2 radius 0.1. Validation sampled base-context leaves only, with four
physics evaluations per leaf and saved importance weights for the fixed virtual
forward/reverse proposals. No optimizer iteration followed these estimates.

| Work | Complete native receipts | Measured cap/null receipts | Total |
| --- | ---: | ---: | ---: |
| Estimation | 4,083 | 13 | 4,096 |
| Base-only validation | 8,176 | 16 | 8,192 |
| Entire fixed schedule | 12,259 | 29 | 12,288 |

The 13 estimation caps comprise six categorical and seven ES evaluations.
The 16 validation caps are four leaves capped at each of their four physics
seeds. Every cap retains native score null and the prescribed utility/validity
zero contribution; no cap is a valid native reuse score or a dropped sample.
Unknown partial work, InFlight unknowns, and NotRun slots are zero.

The manifest records:

- 1,227,940 attempted steps and 1,227,911 advancing steps, below the fixed
  1,228,800-attempt maximum; the horizon remains 100.
- 64 started/completed direction estimates and 6,144 conditional leaf checks.
- One grammar compilation, taking 161.754553 seconds.
- Campaign elapsed time 796.175440 seconds including setup, below 3,600 seconds.
- Native receipt-byte counter 1,526,604,953; final archive size is separately
  measured above and includes other files/seal.
- Zero optimizer generations and training updates;
  `training_admitted:false` and `full_training_authorized:false`.

These counters were read from the manifest and compared with a read-only scan
of the saved terminal receipt statuses. The frozen audit failed later, so they
do not constitute a complete independent semantic-audit verdict. The 30-second
cooperative cell limit, pooled work/results 50,000/50,000 and node/edge limits
2,000/4,000 were not changed. No measured auditor RSS result is claimed for this
full run; the earlier synthetic storage verification is not a substitute.

## Native summary, not a completed independent audit

The following values come from the sealed **native** `summary.json`. All eight
blocks are retained. The statistical unit is the paired estimator block, not
contexts, estimates, leaves, physics seeds or 12,288 evaluations. Each listed
sign test uses the unchanged ±1e-12 deadband and one-sided exact rule.

| Confirmatory endpoint | Native mean | Wins / losses / ties | Native sign p |
| --- | ---: | --- | ---: |
| Categorical A/B alignment | 0.247887332592 | 8 / 0 / 0 | 0.00390625 |
| Categorical forward utility effect | 0.001354910626 | 8 / 0 / 0 | 0.00390625 |
| Categorical minus ES utility effect | 0.001061014933 | 8 / 0 / 0 | 0.00390625 |

Native categorical context effects are +0.001893518076 (zero context) and
+0.000816303175 (alternating-quarter context). Both are positive. However,
the aggregate categorical weighted-validity effect is
**−0.0000141485847120**, below the required −1e-12 guard. The native conjunction
therefore fails, despite its positive sign-test endpoints.

| Block (zero-based) | Categorical alignment | Categorical utility effect | Categorical minus ES |
| --- | ---: | ---: | ---: |
| 0 | 0.150442634 | 0.001739096 | 0.001533054 |
| 1 | 0.209724558 | 0.001656219 | 0.000459753 |
| 2 | 0.391962114 | 0.001563484 | 0.001504893 |
| 3 | 0.213145523 | 0.001864849 | 0.001853355 |
| 4 | 0.243113253 | 0.000402626 | 0.000244142 |
| 5 | 0.348042848 | 0.001568110 | 0.001015887 |
| 6 | 0.041671569 | 0.001767015 | 0.001869106 |
| 7 | 0.384996161 | 0.000277887 | 0.000007930 |

These are importance-weighted categorical-model effects, not empirical native
proposed-arm means, full expected-reward enumeration, a pure equal-coupling
variance comparison, or evidence of multi-generation learning. Because base
validation includes caps, a full-panel native reuse mean cannot be treated as
an all-valid result. Positive utility endpoints must not supersede the failed
validity guard, nor the accounting failure.

### Descriptive location of the validity loss

A read-only computation from saved validation terms reproduces the negative
native validity effect. All four capped leaves receive a categorical mean
forward weight greater than one:

| Block | Context | Draw | Mean A/B categorical weight | Contribution to aggregate validity effect |
| --- | --- | ---: | ---: | ---: |
| 1 | Alternating-quarter | 95 | 1.008797823520 | −0.000004295812266 |
| 5 | Alternating-quarter | 75 | 1.000390279835 | −0.000000190566325 |
| 7 | Zero | 26 | 1.009032330499 | −0.000004410317627 |
| 7 | Zero | 59 | 1.010755867636 | −0.000005251888494 |

Each leaf's validity is zero. Valid leaves contribute zero to the validity
control-variate contrast. Thus these four terms sum to −0.0000141485847120.
This locates the saved point-effect mechanism; it is not independently replayed
physics or statistical proof that true proposal feasibility decreased.

## Frozen audit failure and bounded diagnosis

[`failed_audit.json`](../output/ere_categorical_comparison_scientific_20261003/failed_audit.json)
preserves auditor exit 1 and its traceback. No `audit.json` exists. The failure
is in original helper `audit_ere_categorical_adapter.check_batch`, called while
reconstructing the categorical raw update for `r1-c0-categorical-e1.json`.
It rejects `theta_after` with `AssertionError: f32 vector bits`.

The frozen bridge auditor reached estimate reconstruction after its initial
full cell-validation pass. It did **not** finish all estimates, support,
validation reconstruction or summary verification. Passing earlier checks is
not a final audit pass.

The root arithmetic disagreement is visible in unchanged sources:

- Rust `categorical.rs` uses ordered f64 sums for softmax normalization,
  leave-one-out utility baselines and utility means.
- Python `audit_ere_categorical_adapter.py` uses `math.fsum` for those sums,
  while requiring exact f32 update bits. The prospective comparison contract
  specifies left-to-right reference accumulation.

A receipt-only scan of all 32 categorical estimates using the original helper
finds just that estimate's raw-update bit mismatch, in 113 coordinates. Maximum
gradient difference there is 2.2551405187698492e-17. Example coordinate 0:
native gradient 6.708500942254059e-19 versus checker −2.236166980751353e-19;
native f32 update 3.3542506649974854e-20 (little-endian bytes `67661e1f`) versus
checker −1.1180834742198051e-20 (`3333539e`). Numerically tiny differences around
zero still violate the frozen bit-exact contract.

A separate **in-memory diagnostic**, using ordered sums without editing any
source or receipt, reproduces all 32 saved categorical f32 updates exactly and
all saved LOO baselines with maximum difference zero. This isolates a reference
accumulation mismatch. It is not a registered corrected auditor, a full audit,
a tolerance waiver, or a replacement for the preserved failed result. No full
audit was retried and no diagnostic output was written into the sealed archive.

## Next boundary

The next task should be a **separately registered receipt-only accounting
correction**, not another scientific seed panel:

1. Pin this failed archive and its original checker/source hashes unchanged.
2. Specify the exact ordered-accumulation correction, with mechanical diff and
   cancellation-heavy/mixed-cap/64-emission regression cases. Keep f32 equality
   and all scientific criteria unchanged; do not substitute a broad tolerance.
3. Freeze a new supplemental checker before running it against these receipts;
   require complete accounting and detached semantic-tamper rejection. Store
   the new verdict outside this archive, explicitly postmeasurement.
4. Retain the original frozen-audit failure. If complete reconstructed results
   agree with native, report promising local utility/alignment evidence with
   failed validity conjunction, not categorical support or training admission.

Only after that evidence repair should a new prospective experiment be chosen.
A feasibility-aware/constrained direction design is a candidate because the
saved proposals increase weight on measured capped leaves. It requires its own
engineering review, preregistration and fresh seeds; neither changing this
trial's validity threshold nor selecting a smaller step from these held-out
outcomes is an admissible way to pass this trial. No follow-on work was launched.
