# Isolated coherent assay: implementation and all-parent smoke

Date: 2026-09-28. Profile: `E2-complete-coherent-isolated-assay-v1`.
Status: **bounded all-parent development smoke complete and independently
verified**. No full scientific campaign or training admission has occurred.

## Implementation and version boundary

The prospective contract is `ere_complete_coherent_isolated_assay_v1.md` and
its companion JSON. The isolated producer is
`crates/hg-sim-core/tests/ere_complete_coherent_isolated_assay.rs`; its standalone
checker and regression fixtures are
`scripts/verify_ere_complete_coherent_isolated.py` and
`scripts/test_verify_ere_complete_coherent_isolated.py`.

This is a separately named profile, not an amendment to the measured coherent
v1 contract. It constructs the same world semantics directly in
`HypergraphStore::new_metadata_isolated()` before complete-family encoding. Both
ordinary node and edge identities are retained in exact trajectory comparisons.
This avoids allocator interference without normalizing handles or endpoint order.
The manifest, parent records and trace starts record `metadata-isolated-v1`.

The same 168 genotypes, ChaCha8 seed, parent/operator/role sampling, selector
cardinalities and ordinal/RNG-consumption schedule are retained. A regression
checks all 64 selections and transaction statuses against the old sealed smoke.
Handles necessarily differ between allocation modes; cross-profile byte identity
is not asserted. The original raw counterfactual still runs once on its separate
clone, with no filtering or resampling. Coherent refusals retain their real staged
material and remain Rejected rather than being promoted through rollback.

The original complete decoder, coherent transaction, compiler, Engine, matching
policy, production constructors and training guard are unchanged by this task.
No dependency or core allocator change was needed beyond the prior identity fix.
The old producer/checker/contracts were preserved. The new profile explicitly
pins the current lock, arena and graph implementation, rather than bypassing the
old profile's correct dependency-pin refusal. Its historical basis also includes
the old two-failure smoke and the separate allocator diagnostic.

### Independent allocation checks

The checker adds a standalone `Allocation` model of ordinary/metadata lanes,
next indices, generational recycling, LIFO free lists and ordered incidence.
Parent allocation shape and initial generations are checked. Every primitive
birth must use the expected handle, not merely an unused index or larger version.
Deletion returns a slot to its birth domain even after relabeling.

During native Engine replay the checker reconstructs ordinary allocation and
the ordered deletion/cleanup history, including repeated-endpoint incidence,
and verifies exact expected ordinary birth handles. Metadata is static during
these trajectories. The checker never calls Rust, re-encodes a candidate, or
repairs material. As before, it does not independently implement ChaCha sampling
or the complete Engine's matching/conflict selection; native bindings and RNG/hash
witnesses are checked alongside independent semantic/event replay.

The extra two tamper categories reject a wrong allocation domain and a skipped,
noncolliding allocation index. These have deterministic fixtures independent of
whether the primary smoke happens to select a suitable edit. Existing 25 tamper
categories are retained. Additional predicate tests ensure that same-hash endpoint
reversal and changed ordinary edge generation still fail exact identity.

## Fresh sealed receipt

Archive: `output/ere_complete_coherent_isolated_smoke_20260928`.

- All **168 eight-step parent baselines** complete.
- All **5,376 damage controls** pass native and frozen checks before trajectories.
  All 13 relation roles and 26 obligation keys, including explicit zero, are covered.
- All **64 attempted draws** remain in their declared denominators.
- **55 child traces** complete; **1,784 attempted and advancing Engine steps**.
- **Zero Unmeasured draws**; measurement completeness is true.
- **22 Neutral soundness checks, zero failures**.
- **27 tamper categories rejected** by the full frozen audit.
- **222 frozen inputs, 184 matching live source files, 233 sealed outputs** verify.
  A separate read-only frozen full replay also passes without resealing or writes.
- 168 genotype parents represent 112 executable RCF/presentation classes; no
  assumption of 168 distinct executable classes is made.

The pre-seal assay time is 111.046 seconds; pre-seal shared audit charge is
87.355 seconds, within 300/120-second development caps. The full invocation took
113.17 seconds. Final archive size is 8,982,779 bytes, below 512 MiB. The frozen
full audit measured 244,285,440 bytes peak RSS, below 1 GiB. Peak per-step matching
work/results are 8,646/896, below the unchanged 400k/50k limits. No matching or
graph ceiling interrupted a trace. Freeze setup uses its separately bounded
setup allowance; subsequent freeze/controls/full/seal audits share one budget.

## Results and comparison with the old smoke

Primary outcomes: **22 Neutral, 12 NearNeutral, 21 Divergent, 9 Rejected**.
Transactions: 44 Applied, 11 Inapplicable, 9 RejectedPostcondition. The separate
raw counterfactual remains 30 decodable / 34 invalid; it is not another trajectory
campaign or a replacement primary denominator. Six actual mode changes include
four trajectory-changing edits.

| Draw / parent | Old coherent-v1 smoke | New isolated smoke |
| --- | --- | --- |
| 19 / 138, dormant Proposal AddVar | Neutral identity; soundness failure | Neutral; exact paired world/RNG, no failure |
| 27 / 30, dormant Proposal AddVar | Neutral identity; soundness failure | Neutral; exact paired world/RNG, no failure |
| 15 / 129, Proposal AddPatternEdge | Divergent, allocation-sensitive world handles | NearNeutral under exact node/edge identity |
| 20 / 39, Proposal AddLabel | Divergent, allocation-sensitive world handles | NearNeutral under exact node/edge identity |

These are the only outcome/soundness-flag changes among the retained 64 draws.
Draw 32 remains Divergent: the equal-hash ordered-endpoint difference is not
ignored. Nothing is retroactively reclassified in the old archives.

The short-run fractions below are descriptive only. NearNeutral excludes Neutral;
Valid includes both Applied and Inapplicable children. No catastrophe was observed
within eight steps, which cannot test the 50-step freeze criterion.

| Stratum | Attempts | Valid | NearNeutral | Valid fraction | NearNeutral fraction |
| --- | ---: | ---: | ---: | ---: | ---: |
| Overall | 64 | 55 | 12 | 85.94% | 18.75% |
| Original structural operators | 52 | 43 | 10 | 82.69% | 19.23% |
| Active-parent structural | 22 | 18 | 8 | 81.82% | 36.36% |
| Topology operators | 30 | 21 | 4 | 70.00% | 13.33% |
| Active-parent topology operators | 14 | 10 | 4 | 71.43% | 28.57% |
| Actual pattern change | 29 | 20 | 4 | 68.97% | 13.79% |
| Active-parent actual pattern change | 13 | 9 | 4 | 69.23% | 30.77% |

Although these point fractions meet the declared percentage lines, the scientific
sample/horizon requirements are not met. Overall needs 10,000 draws; active
structural/topology need 2,000 samples each; actual changes need 1,000 and active
actual changes 500. Neutral soundness needs 50 checks, not 22. Activation has
12 samples, six mode changes and four trajectory changes, below 1,000/100/20.

All 13 operators appear; both roles, activation states and at-cap/off-cap parents
appear globally. Proposal AddSlot and Proposal ChangePriority are absent from
this primary short sample. The separate 96-request native fixture matrix covers
every operator and both applicable roles, but does not fill scientific primary
sample coverage. Do not filter or add success-selected draws to close these gaps.

`numerical_pass=false`, `gate_pass=false`, `training_admitted=false` and
`scientific_run_launched=false` are preserved. This is a clean engineering smoke,
not a passing scientific G-R1 result or proof of a mutation-rate improvement.

## Tests and historical preservation

New harness release and debug suites: **8 passed / 2 ignored**. The development
entry point was separately invoked once; the full scientific entry point was
not invoked. The independent native fixture checker passes **13 Python tests**.
Regression suites also pass: coherent kernel 19; raw G-R1 6 (2 ignored); identity
harness 3 (1 ignored); complete-profile training guard 2.

Read-only checks after the smoke and re-audit confirm the exact archive membership
and hashes for the historical coherent-v3 smoke (218 sealed files), allocator
diagnostic (77) and raw full trial (205), as well as the new archive (233).
The old coherent contract hashes and coherent transaction hash are unchanged.
Failed development archives and failed scientific results remain preserved.

## Reproduction and extension points

Use the archive's frozen checker for a read-only replay:

```sh
python3 -B output/ere_complete_coherent_isolated_smoke_20260928/source_snapshots/scripts/verify_ere_complete_coherent_isolated.py output/ere_complete_coherent_isolated_smoke_20260928 --development --stage full --verify-only --seconds 120
```

Run non-campaign regressions:

```sh
cargo test --release --locked -p hg-sim-core --features hg-meta --test ere_complete_coherent_isolated_assay -- --test-threads=1
cargo test --locked -p hg-sim-core --features hg-meta --test ere_complete_coherent_isolated_assay -- --test-threads=1
```

Key extension boundaries are `world()`/`parents()` for isolated construction,
`raw_edit()`/`draw()` for exact selector-to-request routing, `typed_world()` and
`neutral_failure()` for identity, and `Allocation`/`Builder`/`replay_step()` for
independent material and handle checks. `freeze()` retains source/basis pins;
`campaign()` enforces the controls-before-trajectories barrier and shared caps.
Future material changes require a fresh prospective profile/source freeze, not
rewriting this measured receipt or replacing the old contracts.

## Remaining work

Metadata scan work remains paid and is bounded independently of typed-world
identity. No neutrality claim is made at a binding matching ceiling. No production
runner was switched to isolated mode and no legacy checkpoint was migrated.

The next useful bounded step is a prospectively specified **100-step resource and
long-horizon pilot**, to test freeze behavior and estimate whether a full assay
fits its unchanged caps. This smoke does not establish full-campaign cost: audit
already used 87.355 of its 120 development seconds. No full-run budget is silently
increased and no extrapolated runtime is presented as a guarantee.

The proposed full envelope remains 10,000 draws/100 steps, maximum 1,016,800 Engine
steps, 3,600-second assay, 4 GiB output, shared 900-second auditor, 1 GiB auditor
RSS, one thread, 2,000 nodes/4,000 edges and pooled 400k/50k classification matching.
It needs separate explicit authorization and all preregistered controls, sample
coverage, strict soundness and numerical requirements. Even a full pass would
still require optimizer prechecks and separate training authorization.
