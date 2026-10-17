# Ordered-sum supplemental audit: tests pass, final live-source check blocked

Date: 2026-10-03. Supplemental profile
`E2-categorical-comparison-postmeasurement-ordered-audit-v1`.

## Outcome

Implemented and separately froze a receipt-only accounting correction. All nine
regression groups pass, including exact reconstruction of all 32 categorical
64-emission raw updates. All fifteen detached helper-stage semantic controls
reject altered receipts. No strict f32 comparison or numerical tolerance was
relaxed. No physics, RNG sampling, grammar compilation or training occurred.

The revision-2 full supplemental invocation did not produce an accepted result:
after the original complete receipt/math audit call returned, its final live
measurement-source check detected two files changed in the shared checkout.
The runner preserves a failed manifest and subprocess traceback; there is no
supplemental `audit.json`. Do not call this a passing full audit or training
admission. The original scientific archive and frozen audit failure remain intact.

## Exact correction

Four new supplemental sources are registered independently of the 481 native
measurement inputs:

- `scripts/audit_ere_categorical_comparison_ordered.py`
- `scripts/test_audit_ere_categorical_comparison_ordered.py`
- `scripts/verify_ere_categorical_comparison_ordered.py`
- `docs/ere_categorical_comparison_ordered_audit_v1.md`

The checker loads research helpers from the failed science archive's pinned
source snapshots. It replaces precisely three `math.fsum` callees in memory:
softmax normalization, leave-one-out baseline, and batch utility mean. An explicit
left-to-right f64 loop supplies the replacement. Restoring those three names
must yield the original two function ASTs exactly. The original adapter source
hash is `2c5e2d64fcff7c7600b46151c1e7eb857673f155c0f7d84e1fd79c949d47db28`.
The human mechanical diff and machine correction receipt are frozen separately.
No measured source, native binary, config, seed, receipt, objective, proposal
radius, denominator, sign test, validity threshold or old checker file changed.

The known original compensated-sum mismatch still reproduces in a regression;
the ordered reference accepts its exact f32 bits. Other groups exercise ordered
cancellation, mixed-cap/all-cap/equal-utility complete denominators and strict
signed-zero/one-ulp refusal. Detached controls cover alpha, sample count, utility
mean, gradient, baseline, advantage, score gradient, sample outcome, one-ulp raw
update, matched proposal, utility/validity effects, weight, full repeated-emission
equality and forged scientific support. These are copied in-memory helper-stage
controls, not fifteen full resealed campaign audits. The unchanged guard is also
tested with positive synthetic utility endpoints and a negative validity effect.

## Preserved verification lineage

| Archive | Entries | Status / scope | Root |
| --- | ---: | --- | --- |
| `ere_categorical_comparison_scientific_20261003` | 12,883 | Native complete; original frozen audit failed | `387a6e32a152250a081b246e53e5a69adbd273c4f465f726594a256ca3600fc1` |
| `ere_categorical_comparison_ordered_audit_20261003` | 10 | Test packaging failed; no full audit invoked | `85cd6236489bf9feee0a8b864bd0f0292c210af5f3fb3350ac9d97cce61598bc` |
| `ere_categorical_comparison_ordered_audit_v2_20261003` | 12 | Nine groups/15 controls pass; final live-source check fails | `d69addbb3e6e681c3929a4329316386610308a0ef80316411f701e024f69003d` |

All three archive seals verify. All four live supplemental source hashes match
the revision-2 registration. Revision 1's eight successful groups and helper
reference failure remain preserved. Revision 2 corrects the test's two references
to `matched` through the pinned original module, adds explicit prior-failure
registration and documentation, and freezes fresh sources before checking.
The arithmetic checker is byte-identical between these verification revisions.

Revision 2 ran the frozen regressions in 13.258 seconds and the full verification
process for 191.173498 seconds overall, below its registered ceilings. Its
`audit_process.json` preserves the full-check traceback. The original audit call
returned before the final `verify_input()` assertion (traceback at wrapper line
135); therefore the receipt/math call was reached and completed. Its local result
was not emitted or saved because the provenance guard stopped the wrapper. This
control-flow observation is not a substitute for an accepted persisted verdict.

## External checkout drift

The initial registered checks and the supplemental check's first `verify_input()`
accepted the measured live hashes. The final check rejected two changed files:

| File | Measurement SHA-256 | Current observed SHA-256 |
| --- | --- | --- |
| `Cargo.lock` | `fab09392e413bddf671a471ec07af2a6f4be2580c5c8b0bd4b85644d1e1c05c3` | `064685f2264a3716f644950c6b805a313c6730180eee47e4f2f8a45ffadab3cc` |
| `crates/hg-sim-core/Cargo.toml` | `1dbafee44dfa664ba3e54f4cb9224c40298da27aef0428ee40de920367495da8` | `0c8d028d3868615ce26fd7742ac9aa6e9fea73d10c84c70ab202a2eddf83a2bc` |

The manifest diff changes `evolving-minds = []` to `evolving-minds =
["dep:sha2"]` and adds optional `sha2` version 0.10. The lockfile adds that
dependency and version 0.10.9. This task did not edit either file or run Cargo.
Their changed provenance indicates other concurrent workspace work; its author
and intent are not inferred. These changes were not reverted, copied into the
sealed measurement archive, or silently exempted from the registered check.
Current hashes are a point-in-time observation and may change with other work.

The original sealed 481 source snapshots and native binary remain intact.
Current-checkout equality is no longer established; historical measurement
identity and current implementation identity must remain separate claims.

## Next decision

The current invocation is finished and failed; it will not be patched or retried.
Recommended next direction is an explicitly registered **historical/archive-only
accounting audit** that requires all original snapshot/root/binary pins, records
live drift separately, and makes no current-code/admission or replay claim.
This changes the registered live-source acceptance policy and requires user
direction before a new invocation. Do not restore unrelated checkout changes to
make this receipt pass. Freeze any revised policy/checker as a new archive and
retain both failed verification receipts above.

The native summary's scientific verdict remains negative: weighted validity
−0.0000141485847120 fails the unchanged ≥−1e-12 guard, despite positive native
utility/alignment sign tests. No follow-on feasibility experiment or training is
authorized by this correction. See the original
[scientific findings](ere_categorical_comparison_scientific_findings_20261003.md).
