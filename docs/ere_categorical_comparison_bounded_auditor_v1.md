# Categorical comparison: memory-bounded supplemental auditor contract

Registered 2026-10-02, before the isolated resource comparison below. One initial
functional re-audit had already passed; no resource measurements had been taken.
Purpose: engineering storage equivalence, not scientific evidence or launch.

## Immutable inputs and unchanged scope

The native development input is
`output/ere_categorical_comparison_development_20261002`, root
`3a9d48f7059dd910a8e008a566237cbf03935790629902f1b9305d496ecf3682`
(692 sealed entries; 192 real receipts). The zero-work input is
`output/ere_categorical_comparison_preflight_20261002`, root
`bf2daa0d582fa05bc2bc9d8ea6c75bdc97991a1abd1dd48dea6be31b0da395b8`
(679 entries). The v1 auditor hash is
`f8e9222864c5b9ed0ef115282a4887d4c742b876af0d0d671ae211fe7c12914c`.
Preserve these archives, all 472 development source bindings, the original
checker/driver/native harness, and the numerical preregistration without edits.

New sources are the bounded supplemental auditor, its tests, the separate
engineering measurement driver, and this contract. Snapshot/hash them before
measurement in a new direct child of `output`; refuse existing destinations.
Seal that registration before any measured worker. Preserve failure outputs;
do not retry a failed measurement in the same receipt or silently correct it.

## Storage design

The new auditor statically duplicates the v1 audit body. A fail-closed AST check
allows exactly two expression substitutions: an eager dictionary becomes
`BoundedRows`, and the first cell load becomes its `read_unregistered` call.
All other audit-body statements and all math/native/source/gate helpers remain
the pinned v1 implementation. There is no dynamically generated audit code.

Each first-pass row receives the original full schedule, catalogue membership,
credit, trace, pooled-allowance, status/null/cap, utility and parameter checks.
Assignment then discards the complete receipt and retains only its ID/hash.
Later accesses hash and parse one byte buffer, refusing any mutation since that
validation. Duplicate JSON keys, unsafe IDs and symlinks are rejected. No lazy
receipt cache, projected score, weakened emission equality, or validation via
hash alone replaces a native or mathematical check. Four-physics validation
still compares the full emission objects directly. IDs/hashes are O(N), while
live native receipt retention is bounded by caller-local objects (a four-row
panel plus transient/last-loop rows), not all N step traces. Catalogue,
direction-support and summary allocations are unchanged. This is a retention
bound, not a universal hard RSS limit for arbitrary input sizes.

## Fixed engineering checks

Run once per registered worker, in separate processes with a 900-second limit:

1. Original frozen v1 audit of all 192 development receipts.
2. Bounded audit of exactly the same sealed development input.
3. Bounded audit of the preserved zero-work preflight.
4. Bounded storage stress at 12,288 bindings: reuse the 192 saved receipt byte
   streams in fixed cyclic order 64 times, validate each original schedule/trace,
   then reload and check all bindings. This is a synthetic storage workload,
   not a scientific schedule, new rollout, seed domain, or independent evidence.
   No 12-GiB eager synthetic allocation is attempted.

Acceptance: development audit dictionaries exactly equal (excluding the saved
six-control list absent from ordinary audit returns); preflight audit dictionary
exactly equals its saved accounting result; seven new regression groups pass;
the original six detached archive mutations and additional native-trace/utility
mutations are rejected by the bounded path; all 472 live source bindings and
input root seals remain unchanged. No numerical tolerances are loosened.

For resource acceptance, bounded development peak process RSS must be less than
original development RSS. Synthetic 12,288-binding peak RSS must be below
512 MiB. Report wall seconds and platform-normalized `resource.ru_maxrss` for
each isolated worker; this includes interpreter/import overhead, not just the
row store. Report retained-binding deep object bytes separately. No host memory
query, new native evaluations, grammar compilation or training is permitted.

## Deliberate launch boundary

This is a separately registered **supplemental re-auditor**, not a retrospective
replacement checker or new native admission receipt. The scientific launcher
still uses v1, including its source-identical development requirement. No
monkeypatch, source-list exclusion, development repetition, reseal or implicit
source-equivalence waiver is introduced. A future prospective versioned launch
binding must explicitly register the accounting replacement and unchanged
native sources; it must also resolve the launcher's seed scan checking the
already-consumed development domain when preparing a disjoint scientific mode.
These integration facts are reported, not bypassed here. The eight-block
12,288-evaluation comparison remains unlaunched and requires explicit approval.
