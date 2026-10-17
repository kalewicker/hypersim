# Fixed-champion confirmation findings

Date: 2026-10-01. Profile: `E2-complete-fixed-champion-confirmation-v1`.

The larger fresh held-out panel completed its entire 6,144-evaluation schedule,
but **did not establish trained improvement**. There were 6,100 valid evaluations
and 44 resource-cap nulls. No evaluation crashed, remained Incomplete or NotRun,
or left unknown partial work. The prospectively frozen accounting/provenance
audit passes, including 23 tamper controls. An audit pass is not a scientific
confirmation pass.

Under the registered all-valid rule, full-panel arm means, paired mean differences
and the official sign-test p-value remain null. The saved champion was not changed,
selected again or retrained. Neither the earlier 64-draw pilot nor this panel is
being pooled or repaired after seeing outcomes.

## Design and provenance

The prospective design is
[`ere_complete_champion_confirmation_v1.md`](ere_complete_champion_confirmation_v1.md),
with machine-readable settings in
[`ere_complete_champion_confirmation_v1.json`](../configs/ere_complete_champion_confirmation_v1.json).
The measured archive is
[`output/ere_complete_champion_confirmation_20261001`](../output/ere_complete_champion_confirmation_20261001).

The champion is the final-generation parameter vector from the completed
20-generation, 16-candidate, 100-step training continuation. That earlier pilot
had a positive held-out primary point estimate, but p=0.109375 failed its
registered improvement criterion. This follow-up freezes the same champion
before fresh held-out physics; it does not select a more favorable generation.

| Identity | Value |
| --- | --- |
| Parent archive | `output/ere_complete_short_training_continuation_20260930` |
| Parent root seal | `dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132` |
| Champion file SHA256 | `14cf0b13540167b3a3b20082db71ddd9bbd761a5f5b0d81083dd6d719e24988f` |
| Registered master seed | `0xE655000001` = 989268541441 |
| Physics seeds | 989268641441, 989268641442, 989268641443, 989268641444 |
| Grammar catalogue SHA256 | `ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6` |

Exactly 512 emission draws are evaluated on four physics seeds in each of three
arms: frozen champion, zero-logit untrained initialization, and fresh random
standard-normal parameters. Each draw's emission seed is master+2000+draw;
its random-parameter seed is master+3000+draw. Each random 183-parameter vector
is reused across its four physics replicas. Emission randomness is shared across
arms, with arm-dependent distributions; physics seeds are also shared. The fixed
order is draw, then champion/untrained/random, then physics seed.

Every requested evaluation has a 100-step horizon. The ordinary world, native
reward, typed grammar, coherent kernel, metadata isolation, opt-in matcher,
MaxParallel/one-rewrite policy and pooled work/results limits of 50,000/50,000
are unchanged from the parent. Graph caps remain 2,000 nodes / 4,000 edges.
There is no new training or precheck physics, interim statistical decision,
outcome-dependent retry, replacement, culling or branch selection.

The maximum registered native budget is 614,400 steps. Primary statistics use
512 paired genotype means, each averaging its four physics seeds. The primary
comparison is champion minus untrained; champion minus random is secondary.
Confirmation requires every scheduled evaluation valid, a positive primary
mean and exact one-sided paired sign p<.05. Ties are excluded only from the
binomial test denominator, not from the fixed 512-draw mean denominator.

## Completed work and resource outcomes

| Arm | Scheduled / terminal receipts | Valid | Resource-cap null |
| --- | ---: | ---: | ---: |
| Champion | 2,048 / 2,048 | 2,036 | 12 |
| Untrained | 2,048 / 2,048 | 2,040 | 8 |
| Random resample | 2,048 / 2,048 | 2,024 | 24 |
| Total | 6,144 / 6,144 | 6,100 | 44 |

Recorded native attempts: **611,564**. Advancing steps: **611,520**. Each of the
44 null evaluations has one terminal non-advancing capped attempt. Unknown
partial-work cells: zero. All caps are delayed, after at least one successful
step; each terminal capped receipt has truncation true, zero advance and an
empty rule-event list. The earlier observer-capture reliability fix therefore
handles these real campaign cap paths without crediting stale events or panicking.

The 44 nulls are 11 draw/arm genotypes, each capped on all four physics seeds:

| Draw (zero-based) | Arm | Attempt on which the cap occurs |
| --- | --- | ---: |
| 33 | Champion | 81 |
| 73 | Random | 2 |
| 103 | Random | 2 |
| 178 | Random | 82 |
| 263 | Champion and untrained | 47 |
| 464 | Random | 51 |
| 502 | Champion, untrained and random | 2 |
| 504 | Random | 73 |

Thus the sampled resource tail is genotype-associated, not isolated to one
physics replica. This narrow fixed-world observation does not establish general
determinism or stochastic robustness. All ordinary-world identities recorded
in this panel are `c73b4b7afbe84357`.

Native elapsed campaign time, including setup and final identity checks, is
442.534755958 seconds; the single grammar rebuild takes 153.531432459 seconds.
These exclude the separate final audit/tamper suite. The largest recorded native
evaluation duration is 0.183213 seconds, below the 30-second cooperative cell
ceiling. Peak matcher work/results are 50,000 / 6,845; peak total graph
nodes/edges are 291 / 516. The work cap is reached despite staying well below
graph-size and result-count caps. Final sealed archive size is 799,495,851 bytes,
below the 2-GiB output ceiling. The campaign is below its 3,600-second ceiling.

## Statistical result and interpretation

Authoritative fields in [`summary.json`](../output/ere_complete_champion_confirmation_20261001/summary.json):

- `measurement_complete:true`, `all_valid:false`, `fixed_denominator:512`.
- `improvement_supported:false`.
- All three full-panel arm means, both full paired mean differences and
  `one_sided_paired_sign_p` are null.
- 509 complete primary pairs: 20 wins, 13 losses and 476 ties.
- 504 complete secondary pairs.

Three primary pairs are unavailable because draws 33, 263 and 502 contain
capped champion and/or untrained evaluations. They stay in the fixed schedule;
no complete-case mean is substituted for the registered full-panel estimate.

A separate, **post-hoc diagnostic bound**, not the registered p-value, asks
whether the three missing primary directions could possibly rescue the sign
criterion while leaving all observed directions fixed. Assigning all three
missing pairs wins is most favorable: 23 wins versus 13 losses, yielding
`sum(comb(36,k), k=23..36) / 2^36 = 0.06624908198136836`.
Even that bound exceeds .05. This does not impute fitness or change official
null fields. It shows that eliminating the cap-related missing directions alone
cannot make this unchanged set of observed primary directions pass the sign
threshold. It says nothing about the magnitude or confidence interval of a
full-panel mean difference.

The panel identifies two separate limitations: incomplete feasibility under the
resource envelope, and a weak/sparse directional advantage for the frozen
champion. Most observed primary pairs are ties. Increasing held-out coverage
has not confirmed the earlier pilot's positive point estimate.

## Implementation and verification

The feature-gated route is:

```sh
target/release/ere-train confirm-short-complete \
  output/ere_complete_short_training_continuation_20260930 \
  output/ere_complete_champion_confirmation_20261001
```

This invocation has already run; its output directory must not be reused.
`crates/hg-train/src/ere/confirmation.rs` verifies all 1,690 parent seal entries,
the exact champion, parent admission/audit state and all 230 native/Cargo/optimizer
pins before setup. It freezes 444 source/protocol inputs and the executed binary,
creates the entire fixed schedule as NotRun before setup, verifies the parent
with its original frozen checker, and rebuilds the pinned catalogue once.
Live identities are checked after compilation, every 64 draws and at completion.

Each native evaluation uses the existing evaluator and terminal receipt wrapper.
The runner writes an in-flight marker before evaluation, saves the terminal row
atomically, then commits known counters. Resource caps return null and allow the
remaining fixed schedule to run. Returned errors or panics become Incomplete
with unknown partial work and stop execution. There is no per-step crash journal
or claim of OS-preemptive timeout enforcement.

The streaming Python checker reuses the exact parent's frozen native-accounting
checker and its pinned runtime helper. It verifies archive/source/binary hashes,
schedule, champion identity, catalogue membership, replica genotype consistency,
native reward/event/cap allowances, counts and independently recomputed statistics.
It processes one draw at a time. Its 23 controls use detached metadata/receipt
copies and small temporary digest/seal fixtures; original evidence is untouched.

The successful final invocation is:

```sh
python3 -B output/ere_complete_champion_confirmation_20261001/source_snapshots/scripts/audit_ere_complete_confirmation.py \
  output/ere_complete_champion_confirmation_20261001 --tamper-tests --seal
```

Final root seal:
`822bea3a828ce6e59324dd13e5c133eba4ffb9a75ab0c8a1102c2346fe70c0bb`.
All 6,598 sealed entries verify with exact archive file coverage. All 444 live
source/protocol hashes still match the prospective freeze. The parent training
archive, initial failed short-training archive and both failed scaling archives
remain byte-identical under their original complete seals. No native evaluator,
grammar, matcher, optimizer, reward or old evidence was changed for this panel.

Development verification before launch: 124 library tests pass, with one
release-only full-budget delayed-cap regression ignored in debug; 11 ERE and
two complete-witness integration tests pass; four Python confirmation regression
groups pass. The complete-feature release CLI builds and the legacy ERE-only
CLI check passes. The full-budget release cap regression was verified in the
preceding reliability task, not newly rerun as a scientific cell here.

The audit reconstructs provenance, membership and accounting. It does not
independently replay native physics/RNG streams or prove compiler-to-binary
correspondence. This fixed-world, frozen-metadata, 100-step panel establishes
neither learned repair/maintenance, novelty, online metadata evolution,
growing-world robustness nor 1,000/10,000-step training feasibility.

## Extension points and recommended next experiment

- `crates/hg-train/src/ere/confirmation.rs`: fixed-champion receipt guard,
  schedule, durable execution and full-denominator statistics.
- `scripts/audit_ere_complete_confirmation.py` and its test file: independent
  accounting/statistics and detached tamper controls.
- `crates/hg-train/src/ere/short_training.rs`: training admission, antithetic
  evaluation schedule and conversion of native results into optimizer pairs.
- `crates/hg-train/src/optim.rs`: rank-shaped ES updates and null-pair treatment.
- `crates/hg-sim-core/src/ere/signal.rs`: native reward, horizon and honest
  resource-cap null semantics; retain the observer reset.

The next useful experiment is **feasibility-aware optimization**, not another
unchanged champion confirmation or an engineering-only gate. Currently a capped
evaluation has null native fitness and its antithetic pair is excluded from ES
updates; this provides no direct optimizer preference for avoiding that cap.
A separately registered version should retain those native null receipts while
giving the optimizer an explicit constraint-aware ordering (feasible outcomes
ahead of capped outcomes, then reuse reward among feasible outcomes), with a
declared policy for both-capped pairs. Do not relabel capped native fitness as
a valid measured reuse score.

First test that conversion and update logic on bounded fixtures, including one
and two capped arms, ties and all-feasible backward compatibility. Then use a
fresh, prospectively fixed short-training budget and fresh held-out seeds,
reporting feasibility and reuse separately with unchanged full denominators.
The strong tie rate also warrants checking whether the reward offers sufficient
resolution for optimization before merely increasing compute. Any reward change
needs a separate objective version and cannot be compared as if this panel had
used it. No additional training or scientific run is authorized or launched by
this recommendation.
