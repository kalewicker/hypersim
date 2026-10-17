# Direct categorical adapter: implementation and engineering verification

Date: 2026-10-01. Profile `E2-direct-categorical-adapter-v1`.

## Outcome

The adapter is implemented and engineering-verified. It reconstructs the actual
masked hierarchical selection law, estimates a raw constraint-utility gradient
with an emission-level leave-one-out baseline, and returns a proposed f32 vector.
It is separate from the existing ES optimizer and has no simulator, sampler,
admission or training loop. Native physics, grammar, reward and `optim.rs` remain
unchanged. **No training campaign or fresh native experiment was launched.**

Nine Rust adapter tests and nine Python regression groups pass. The separately
sealed engineering verification covers all 18,956 legal productions, all 183
coordinates for two features in two contexts (**732 finite-difference checks**),
256 byte-identical historical baseline cells and all five historical cap cells.
Maximum finite-difference error is **4.134129844013401e−11**, below the fixed
2e−9 engineering tolerance. All **18 detached result-tamper controls reject**.
This establishes arithmetic and receipt reconstruction, not better native
directions, reduced variance, learned improvement or training admission.

The first packaging attempt failed before the adapter executed because its
copied executable lost permission bits. That failed archive remains sealed and
unchanged. The corrected copy preserves executable mode and has a regression
test; verification ran in a new v2 output directory, not over the failed receipt.

## Extension map

| File / API | Responsibility and boundary |
| --- | --- |
| `crates/hg-train/src/ere/categorical.rs` | Pure model, selection scores, full-support engineering expectations and LOO estimator |
| `Model::from_catalogue` | Validates parent/operator/role/production indexing and legal support/count agreement; no compilation |
| `Model::from_grammar` | Extracts supports from an already existing native Grammar; does not create one |
| `Model::score` | Reconstructs conditional probabilities, full log leaf probability and all 183 score derivatives at supplied f32 theta |
| `Model::selection_from_emission` | Checks the native selection envelope, role, legal masks and production count; not physical or RNG replay |
| `Model::expectation` | Requires complete unique production coverage and finite supplied feature values; useful for known engineering oracles |
| `estimate` | Requires the complete allocation and distinct emission seeds/IDs; computes utility, LOO terms, gradient and arithmetic proposal |
| `crates/hg-train/src/bin/ere-categorical-adapter.rs` | Pinned-catalogue, engineering-only offline JSON entry point; exclusive result creation |
| `scripts/verify_ere_categorical_adapter.py` | Creates fresh frozen source/binary/receipt snapshots, invokes only the offline adapter, audits and seals |
| `scripts/audit_ere_categorical_adapter.py` | Independently reconstructs arithmetic, checks f32 bits and full-support finite differences, validates archived native accounting and rejects detached corruptions |
| `scripts/test_audit_ere_categorical_adapter.py` | Mathematical, fixture, CLI refusal and executable-packaging regression groups |

The only edits to existing learning files are a feature-gated module export in
`crates/hg-train/src/ere.rs` and a required-feature binary registration in
`crates/hg-train/Cargo.toml`. The adapter stays behind `ere-complete-signal`; both
the default library and the older `ere`-only library still compile. No existing
optimizer or historical runner is routed through the adapter automatically.

Among the historical direction assay's 459 live input paths, exactly these two
registration files now differ. All historical snapshots and archive seals remain
unchanged; the 467 new engineering source snapshots bind the new implementation.
Do not claim the old complete live-source set still matches the current checkout.
Source-locked native launch routes must register the exact new learning-layer
identity before a future trial; this build does not relax their guards or reuse
historical admission as permission for a changed invocation.

## Mathematics and input discipline

For a legal leaf `x=(parent,operator,role,production)`:

```text
Pθ(x) = Pθ(parent)
      × Pθ(operator | parent's legal operators)
      × Pθ(role | parent/operator's legal roles)
      / production_count(parent,operator,role)

score_parent[j]   = 1[j=parent]   − Pθ(j)
score_operator[j] = 1[j=operator] − Pθ(j | parent), within legal mask
score_role[j]     = 1[j=role]     − Pθ(j | parent,operator), within legal mask
```

Unsupported score coordinates are zero, as are singleton-mask derivatives.
There is no score coordinate for the uniformly chosen production. Productions
inside one group therefore remain indistinguishable to this parameterization;
this implementation does not claim to solve that remaining restriction.

Each `Sample` retains ID, emission seed, physics seed, exact selection, nullable
native score and reason. It represents one distinct emission and one already
audited native outcome. The existing utility function is reused unchanged:
valid score [0,1] → 1+score; measured cap with null score → zero utility;
unsupported null, crash, missing/invalid score or inconsistent reason → refusal.

For complete allocation `n>=2`, baseline `b_i` is the mean of all other
allocated utilities. The gradient is `sum((U_i-b_i)*score_i)/n`. This includes
every cap, never changes the denominator and does not rank-shape outcomes.
Unlike centering on the mean containing the sample itself, LOO does not introduce
the `(n−1)/n` shrinkage for independent emissions. Distinct seed identifiers
alone are not an independence proof; future campaigns must establish their
actual on-policy sampling and replication design.

Parameter values and alpha are f32 inputs. Probabilities, log probabilities,
advantages and gradient accumulation are f64. For each coordinate:

```text
proposed_theta[j] = f32(f64(theta[j]) + f64(alpha) * gradient[j])
```

That is one final f32 cast, with no normalization, clipping, projection,
favorable branch selection or automatic application. A negative/nonfinite alpha,
nonfinite theta, illegal production, selected categorical probability that
underflows to zero, or overflowing proposal fails closed. A zero alpha is useful
for engineering checks, not an alternative setting chosen from native outcomes.
All-cap or equal-utility batches produce exactly zero gradients.

The caller must establish that native receipts are physically/accountingly
valid and were sampled at the supplied theta. The API does not certify admission,
RNG draws, source identity or on-policy sampling from a `Sample` descriptor.
The engineering harness checks those recorded theta/selection/accounting bindings
against the immutable historical archive. General future callers need their own
prospective receipt-bound execution and audit path.

## Verification and measured engineering scope

| Check | Result |
| --- | --- |
| Rust adapter unit tests | 9/9 pass: masks/production normalization, all-coordinate log derivatives, oracle derivatives/gauge, enumerated iid parent-pair LOO expectation, caps/sign/f32 proposal, unsupported outcomes/duplicates, extrema/coverage, catalogue/emission masks and overflow |
| Python regression groups | 9/9 pass, including independent arithmetic, 18 detached corruption controls, eleven invalid CLI input cases, existing-result refusal and copied-executable mode/bytes |
| Full-support native-catalogue oracle | 18,956 leaves, both activation/eligibility features, both old fixed contexts; mass/gauge checks pass |
| Full-catalogue central differences | 732/732 coordinates pass; maximum error 4.13413e−11 at fixed step 1e−4 |
| Archived native accounting | 261/261 cells checked; all source receipt bytes equal their historical originals |
| Historical arithmetic proposals | 16 base-only batches × 16 emissions, no new evaluations or application |
| Synthetic arithmetic proposals | Four labeled fixtures: all cap, equal valid, mixed cap and ordered valid |
| Detached result corruption | All 18 reject, including masks, production count, scores, gradients, LOO terms, f32 updates, cap imputation/drop, seed, denominator, missing sample, oracle and false native/admission claims |
| Default and older `ere` feature compilation | Library checks pass |
| Existing Rust optimizer / direction / feasibility arithmetic | 13/13, 4/4 and 3/3 tests pass respectively |
| Existing Python direction regressions | 7/7 pass |
| Existing complete-witness integration regressions | 2/2 pass; legacy training/maintenance refusals remain intact |

The complete-witness compatibility tests include their existing short native
diagnostic fixtures. Those are separate regression tests, not the adapter's
engineering archive, fresh scientific measurements or training. The adapter and
its oracle/archived-receipt invocation itself execute **zero native Engine steps,
zero grammar compilations and zero training updates**.

The independent checker differentiates a directly summed finite expectation at
all 183 coordinates rather than only checking a Rust formula against a copy of
itself. It additionally reconstructs selection-score/LOO arithmetic in Python
and compares proposed f32 bit patterns after Rust JSON serialization. The original
activation/eligibility features remain engineering oracles, never native reward.
The checker validates saved physical accounting but neither independently
replays the Engine nor reproduces the native RNG stream.

There is no comparison of these historical proposed vectors' native outcomes:
they were not evaluated. No improvement statistic, champion or new scientific
acceptance flag is produced. Engineering success does not reverse the negative
direction-assay verdict.

## Preserved failure and new immutable receipt

The original engineering packaging archive is
`output/ere_categorical_adapter_engineering_20261001`, status
`failed_engineering`, 732 sealed entries. `result.json` is absent. The source
executable had mode 0755 but `shutil.copyfile` produced a 0644 snapshot, so process
creation raised `PermissionError` before adapter execution. The wrapper recorded
the failure and sealed its evidence. Fix: preserve mode using `shutil.copy2`;
the new regression executes a temporary frozen copy and checks identical bytes
and permissions. The failed archive was not chmodded, repaired or resealed.

Successful archive:
[`output/ere_categorical_adapter_engineering_v2_20261001`](../output/ere_categorical_adapter_engineering_v2_20261001).
Durable [manifest](../output/ere_categorical_adapter_engineering_v2_20261001/manifest.json)
reports `complete`, `engineering_arithmetic_only`, `training_admitted:false`.
The [audit](../output/ere_categorical_adapter_engineering_v2_20261001/audit.json)
and [result](../output/ere_categorical_adapter_engineering_v2_20261001/result.json)
contain the exact checks and full per-sample/proposal arithmetic. There are
467 frozen source snapshots and 734 exact sealed entries. Engineering invocation
and audit elapsed 42.47 seconds; final archive size 58,677,916 bytes.

| Artifact | Root seal SHA256 |
| --- | --- |
| Historical direction input, 7,686 entries | `ba88c696014fa42ca612f4c20baadb1818cc6c5757fd41594c87a1e4bc1a185a` |
| Preserved failed engineering attempt, 732 entries | `05c3cd5a6a4f3f4dfa207af97839ce8a2670687f173e4582d31c2533bd4535c5` |
| Successful engineering v2, 734 entries | `b889b006889ec2b2d2d978b87b8f444c49e694b04aecc9441cc1ba1adaf7cb5c` |

Executed offline binary SHA256:
`1ad51d3c0edbd52993d66ed1c4172c8e5c46cbc248f65816753017c203a40d49`.
Result SHA256:
`0e0d311e30ae20c764bc06f97fa7c45df4e49cd7c3ee87e6557295a1dc12dd40`.
Audit SHA256:
`5f5821616af77b7ba822a4c17698a254308e7c52681dd55e52d4c2ba063e169e`.
The original direction and historical helper archives verified unchanged;
the v2 source set matches the live engineering inputs and its exact seal verifies.

## How to use and reproduce

API sketch, after the caller validates its native receipts and on-policy design:

```rust
use hg_train::ere::categorical::{estimate, Model};
let model = Model::from_catalogue(&catalogue)?;
let score_receipt = model.score(&theta, selection)?;
let proposal = estimate(&model, &theta, &samples, 64, 0.05_f32)?;
// Persist/audit proposal; do not treat this call as training admission.
```

Build and verify from the repository root:

```sh
cargo build -p hg-train --features ere-complete-signal --bin ere-categorical-adapter
cargo test -p hg-train --features ere-complete-signal --lib categorical::tests
python3 -B -m unittest discover -s scripts -p test_audit_ere_categorical_adapter.py -v
python3 -B scripts/verify_ere_categorical_adapter.py \
  target/debug/ere-categorical-adapter output/NEW_FRESH_ENGINEERING_DIRECTORY
python3 -B scripts/audit_ere_categorical_adapter.py \
  output/ere_categorical_adapter_engineering_v2_20261001 --tamper-tests
```

Do not rerun the wrapper over either recorded directory. Both the wrapper and
the offline CLI reject existing outputs; the CLI also uses exclusive creation
to protect against an overwrite race. The saved Python checker/import hashes
must still match when using the live checker to re-audit. This is not a portable
standalone physics replay package.

## Next step

The adapter milestone is complete. Next is a **separately preregistered native
direction comparison** against ES, with fresh on-policy seeds and explicit
receipt/source registration. Freeze estimator allocation, physics grouping,
baseline, update scale, validation arms, independent block denominator,
resource limits and cap/partial handling before launch. Keep the same native
constraint utility and fixed-world horizon. The suggested 64 evaluations per
estimate remains a design proposal, not a campaign authorization. Do not run
training until native usefulness/reproducibility is supported by that fresh test.
