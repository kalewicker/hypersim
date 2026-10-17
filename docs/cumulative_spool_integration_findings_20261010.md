# Bounded evidence spool integration — 2026-10-10

**The development runner now uses the validated compact format through a bounded
storage spool.** Four injected archive/release interruptions recover safely; nine
unsafe/capped cases fail closed; a saved maximum-dimension journal round-trips
exactly. The native interrupted-and-restored route reproduces the complete reference
world byte-for-byte. Scientific readiness and launch authorization remain false.

## Implemented handoff

`experiments/cumulative/scripts/evidence_spool.py` adds an owned, single-writer
spool with an exclusive mutation lock and an append-only, hash-chained event log.
Its lifecycle is:

`reserved → sealed → verified → release_intent → released`

Reservation happens before the native child is launched. Each writer receives a
31 MiB journal allowance, leaving room within the 32 MiB raw-unit cap for a seal.
The admission reserves raw storage, compact staging, per-file allocation/metadata
and control overhead. Retained files and unfinished reservations remain charged;
an unfinished writer cannot be silently redispatched after reopening.

When the child exits, including a declared interrupted exit, its closed journal is
sealed and packed with the unchanged lossless codec. Every object and original
hash is verified before the durable release intent. Cleanup is restricted to the
spool's own raw directory and rechecks every remaining file against the compact
copy. Unknown files, symlinks or changed bytes stop cleanup. Historical evidence
is never a cleanup target.

A separate bounded restore lease exposes only a verified journal unit to the
existing native continuation reader. Its bundle seal stays outside the native
journal namespace. Torn staging and unpaired native intents are refused before
restoration. The restore is checked and released after use. Interrupted extraction
or partial compact publication stays charged and fails closed; this fixture does
not implement automatic repair of those ambiguous states.

The wrapper is development-only. It checks the executable hash, preserves the
request identity, applies the storage allowance before dispatch, retains the child
exit/timeout receipt and never retries the child. Aggregate native-work funding
and the independent semantic audit remain the caller's responsibilities; the
fixture launcher supplies both. No native chemistry, old journal body, seed,
criterion or previous source file changed.

## Fixed verification panel

| Check | Result |
|---|---|
| Interrupt after raw seal | Reopened, packed, verified and safely released |
| Interrupt after compact publication | Adopted verified completed store; no overwrite |
| Interrupt after verification receipt | Reverified before release |
| Interrupt during raw cleanup | Verified remaining files; finished exact cleanup |
| Saved maximum-dimension journal | Exact compact/restore round-trip |
| Insufficient storage admission | Refused before a work reservation |
| Unfinished writer reservation | Retained and charged; no redispatch |
| Pending native action | Archived intact; continuation restore refused |
| Torn native result | Staging bytes retained; continuation restore refused |
| Partial compact publication | Raw retained; no silent overwrite/retry |
| Corrupt compact object | Raw retained; release refused |
| Foreign raw file | Cleanup refused |
| Symlink raw file | Sealing/release refused; only the test link removed |
| Modified restored file | Cleanup refused; changed copy retained |

These fourteen saved-evidence checks execute zero native rewrites. Their intentionally
failed states remain in the sealed fixture, with capacity reports. The four injected
archive interruptions are process-boundary tests, not a universal proof against
every filesystem or power-loss failure.

The subsequent native panel uses the unchanged development seed `2026101001` and
matched endogenous world from the adapter fixture, with a fixed aggregate ceiling
of **250 rewrites and 60 command seconds**:

| Native unit | Committed work |
|---|---:|
| Complete reference | 72 |
| Interrupted prefix | 6 |
| Continuation from compacted and restored prefix | 66 |
| **Total new native work** | **144** |

All sixteen world rows and the final allocator/checkpoint state match exactly
between the complete route and the continued route. The compact source retains
the original journal head. All three native raw journals and the temporary restore
are released only after verification; their compact copies remain available for
independent audits. No scientific seed was executed.

## Limits and custody

The greatest recorded admission charge is **123,899,904 bytes (118.16 MiB)**,
below the fixture spool's fixed **256 MiB** cap. That charge includes the full
reservation, not just the smaller observed compressed output. The overall fixture
still has a separate 128 MiB output cap. Native dispatch plus archival takes
3.361 seconds; the full workflow takes 7.399 seconds, below its 1,200-second cap.

All thirteen earlier sealed roots and all 156 prior source pins remain unchanged.
The existing adapter binary is copied and hash-checked rather than rebuilt. Its
native source bodies remain unchanged; request receipts additionally bind the
current spool/source snapshot. The new package has 160 source pins and 468 sealed
entries. Original and relocated known-root checks repeat the independent audit
with zero new native work.

## Evidence and remaining work

Package: `output/cumulative_spool_integration_20261010`.
Seal: `a4460392316a6689cac371d474e04ba1e3290cdd5a8cb68160bf4dc8efe728be`.

- `contract.json`: fixed saved-evidence cases, native routes and limits.
- `cases/`, `fault_tests.json`: retained successful and failed storage transitions.
- `native_spool/`: ownership/event ledger, request/exit receipts and compact units.
- `native_inputs/`, `native_routes.json`, `audit.json`: funded native inputs and
  independent transition-law/continuation verification.
- `source_snapshots/`, `preservation.json`: immutable code and historical custody.

External recheck receipts are
`output/cumulative_spool_integration_20261010_recheck.json` and
`output/cumulative_spool_integration_20261010_relocated_recheck.json`.
The frozen `scripts/audit_spool.py` runs against either location after its known
seal and complete inventory have been checked.

The lossless storage format, bounded spool and development native handoff now pass.
The remaining scientific integration is the complete census/version-lineage to
thirteen-role assay mapping and the final campaign contract. The previous observed
2.93 GiB storage projection remains an estimate under explicit reserves, not a
universal completion guarantee. A full campaign must retain every planned world,
stop at its actual funding/storage limits and report unknown routes without
replacing seeds or weakening the criterion.
