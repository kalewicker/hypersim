# Categorical comparison: memory-bounded auditor findings

Date: 2026-10-02. Result: **supplemental engineering verification passes**.
The completed native development run, original checker, native harness, driver,
preregistered numerical criteria and all 472 historical source bindings remain
unchanged. No scientific comparison or additional native work was launched.

## Measured result

Each resource worker ran once in its own Python process. Peak RSS includes
interpreter/import overhead; macOS `resource.ru_maxrss` was recorded in bytes.

| Fixed worker | Peak process RSS, MiB | Wall seconds | Scope |
| --- | ---: | ---: | --- |
| Original auditor, development | 333.922 | 10.311 | Full saved 192-receipt accounting |
| Bounded auditor, development | 132.375 | 10.753 | Exactly the same accounting result |
| Bounded auditor, preflight | 91.922 | 3.823 | Preserved 192-slot zero-work input |
| Bounded storage, 12,288 bindings | 89.703 | 75.805 | Synthetic reuse of historical receipts |

Measured development RSS falls **60.36%**, from about 334 to 132 MiB. Audit time
increases about 4.3% (0.44 seconds); the implementation trades rereading receipts
for not retaining their parsed trace dictionaries. Both preregistered engineering
resource checks pass: lower development RSS and synthetic stress below 512 MiB.

The stress workload cycled through the same 192 saved receipt byte streams 64
times, checking their original schedules and full traces, then reading every
bound row again. It retained **2,345,546 bytes (2.24 MiB)** of ID/hash bindings,
cached zero receipt objects, and performed 24,576 reads totaling 3,109,204,480
bytes. These are historical receipt checks, not new Engine evaluations. The
12,288 synthetic identities are not scientific seeds or a scientific schedule.

This stress case establishes bounded trace retention, not full scientific-audit
RSS or runtime: it does not reproduce the proposed estimation/validation phase
mix, eight blocks of support arithmetic, new outcome distribution, or cap tail.
The prior roughly 12-GiB estimate for the eager full-scale parsed-row heap was an
extrapolation, not an attempted allocation. Host installed/free memory remains
unverified. The full run's unchanged 2-GiB output and 900-second auditor limits
are not certified by this engineering workload.

## Implementation and equivalence

[`audit_ere_categorical_comparison_bounded.py`](../scripts/audit_ere_categorical_comparison_bounded.py)
introduces `BoundedRows` and a statically copied audit body. A fail-closed AST
check verifies exact identity with the pinned original audit body except for two
storage expressions. All native/math/source/gate helpers remain the original
implementation; no dynamically generated checker or module monkeypatch is used.

The first pass reads one receipt, performs all original schedule, full catalogue
membership, native credit, 100-step trace, pooled-budget, cap/null/status, score,
utility and parameter checks, then discards it. Only its ID and SHA-256 remain.
Later accesses hash and parse the same byte buffer, rejecting changes since
validation. Duplicate JSON keys, unsafe IDs and symlinks also reject. The mapping
has no cache; transient rows remain local to the unchanged estimator or four-row
physics panel. IDs/hashes grow with N, but stored step traces do not.

No compact score projection or emission digest replaces a calculation. The
validation code still compares complete emission dictionaries across all four
physics repetitions. Native cap denominators, incomplete/NotRun treatment,
fixed schedules, f32 raw updates/matched reversals, LOO baselines, full-support
weights, ordered effects and sign-test decisions are unchanged.

The bounded development result exactly equals the original worker and saved
development audit dictionary, excluding only its separately stored six-control
metadata. The bounded preflight result exactly equals the ordinary saved
preflight dictionary. Numerical tolerances were not loosened. These are receipt
and mathematical checks, not independent physics, grammar or RNG replay.

## Regressions and negative controls

The final verifier passes **27 regression groups**:

- Seven new bounded-store/auditor groups: full reload equivalence, strict JSON
  and paths, post-validation mutation refusal, cap/partial/NotRun semantics,
  native trace/credit/utility tampering, full-emission equality and AST pinning.
- Seven unchanged original harness groups, including independent Python/Rust
  full-support and f32 math cross-checks. The offline Rust fixture performs no
  native steps or grammar compilation.
- Eight unchanged planning groups in their hermetic historical adapter-stage
  fixture; fixture provenance is recorded. This is not current-stage admission.
- Three receipt-shape groups and two verifier import/symbol-table guards.

**Fourteen detached mutations reject:** the original six controls against each
input mode (false support, false training admission, changed attempt count,
schedule seed, full-training authorization and build-feature declaration), plus
development trace-type and utility mutations. All controls edit disposable
copies, never original inputs. Unit tests additionally reject changed credit,
bad world hashes and changes between native validation and later reads.

## Preserved engineering failures and corrections

No failed manifest or frozen checker was overwritten. All four resource-worker
outputs came from the first receipt and were reused without rerunning, replacing
or selecting measurements. Only the verification/packaging layer continued.

| Receipt | Authoritative state | Reason / correction |
| --- | --- | --- |
| `ere_categorical_comparison_bounded_auditor_20261002` | Failed engineering | Four workers finished; assembly incorrectly expected development-only control metadata in preflight `audit.json` |
| `ere_categorical_comparison_bounded_auditor_v2_20261002` | Failed engineering | New seven regressions passed; old tests could not resolve repository-relative fixtures from a snapshot directory |
| `ere_categorical_comparison_bounded_auditor_v3_20261002` | Failed engineering | Copied verifier lacked standard-library imports; stopped on `NameError: math` before regressions |
| `ere_categorical_comparison_bounded_auditor_v4_20261002` | Complete engineering | Explicit imports and guards; correct receipt shapes and source-stage tests; 27 groups and 14 controls pass |

A separate diagnostic planning test also correctly rejected current post-harness
registration, because it expects the earlier adapter-only stage. It was not
patched into accepting current code: the unchanged eight tests were executed in
a disposable source fixture copied from all 467 frozen adapter sources plus the
four planning sources and the minimal historical lineage inputs. Original
harness tests ran from their expected repository location with all live inputs
hash-bound before and after. No assertion or scientific threshold changed.

The current verification entry point is
[`verify_ere_categorical_comparison_bounded_v4.py`](../scripts/verify_ere_categorical_comparison_bounded_v4.py).
Earlier verifier versions remain frozen failure evidence, not recommended
entry points. The supplemental read-only re-audit can be invoked as:

```sh
python3 -B scripts/audit_ere_categorical_comparison_bounded.py \
  output/ere_categorical_comparison_development_20261002
```

## Sealed evidence

Final receipt:
[`manifest`](../output/ere_categorical_comparison_bounded_auditor_v4_20261002/manifest.json),
[`verification`](../output/ere_categorical_comparison_bounded_auditor_v4_20261002/verification.json),
[`preserved measurements`](../output/ere_categorical_comparison_bounded_auditor_v4_20261002/preserved_measurements.json),
[`controls`](../output/ere_categorical_comparison_bounded_auditor_v4_20261002/controls.json).
It seals **29 entries and twelve engineering sources**; all 472 historical live
bindings and all twelve engineering live bindings independently verify.

```text
Final engineering root:
5c5cbc9f902fa6dde7505a69f8df0fb8b32881ebbb6a3ac15a553c2fd3091671
Frozen registration root:
2a0ca4af5b359177f03f168a5317d0eec5c5ca9748653c02d146cf9cca50c9ee

Preserved engineering failure roots (v1, v2, v3):
6b98587e9b151ec738bf32210ba0d60e7b657c27d87e50757c1e4c68a9b970d8
f6c2a73caf4fb2013cfd43225024cb983b1836b68e64e37aecae60862749bd39
5409a2c0a8a81bd92a996651b4a2f0778a11cbc560ac3c0595a3f564fa946697

Unchanged native development / preflight / numerical planning roots:
3a9d48f7059dd910a8e008a566237cbf03935790629902f1b9305d496ecf3682
bf2daa0d582fa05bc2bc9d8ea6c75bdc97991a1abd1dd48dea6be31b0da395b8
dcf4ec69f2ea647f1f7eeda9185ae526788a553639fd490395da7f98933b5d57
```

An additional hashlib-only check independently verified exact inventory coverage
and every hash in all seven listed archives; the failed receipts remain failed.

## Next: explicit launch integration, then approval

The trace-retention issue is addressed, but **the native launcher still invokes
the original v1 auditor**. This supplemental verification does not silently
replace its checker or relax its source-identical development prerequisite.
The full 12,288-evaluation comparison remains unlaunched and training admission
remains false; this turn ran zero new native evaluations, steps or compilations.

Next implement a prospective versioned launch bridge that explicitly binds this
storage-only accounting replacement to the unchanged native development evidence
and exact permitted source/dispatch changes, retaining all scientific criteria.
Also fix the launcher's mode scope: its present seed scan checks both development
and scientific domains, so the completed development seeds block preparation of
disjoint science. Check the selected mode against all consumed historical seeds;
do not waive collisions, reuse development as science, or repeat development to
manufacture source identity. Verify that bridge with zero-work launch refusals
and detached controls. Only then request explicit approval for the full trial.
