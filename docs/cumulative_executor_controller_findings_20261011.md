# Executor/controller integration — 2026-10-11

The new native executor now completes the development campaign through the
durable controller, bounded evidence spool, independent journal audit, census
measurement, assay registry and report closure. Scientific execution remains
disabled. This verifies engineering integration; it supplies no new scientific
evidence that the system evolves complexity.

## Implementation

The previous executor's thirteen authored controls differ from the nine requests
produced by campaign measurement. A separately versioned executable,
`cumulative-controlled-executor`, freezes the nine-request development route while
preserving all earlier source pins and archives. Its physical world and assay
implementations, shared admission validator and journal writer are unchanged.
It accepts only compiled development route names, not arbitrary scientific worlds.

`scripts/controlled_executor_dispatch.py` binds each exact route and executable
hash to the controller's next request. It checks rewrite funding, actual monotonic
time and storage before publishing a durable intent and invoking the child.
The storage charge includes files outside the spool in the fixture workspace.
Native execution, compaction and independent semantic auditing all count against
the unit's time grant. Only a complete verified journal permits an exact completion
receipt and refund of unused work/time. Reopening a controller cannot reset its
clock epoch or redispatch unresolved work.

## Results

- Four fixed development worlds complete with 72, 66, 73 and 64 rewrites. Every
  result and final allocator checkpoint equals the authenticated reference.
- All 40 census slots are retained. The measured nine-request registry is frozen
  before assays and matches the compiled development route exactly. Assays use
  60 rewrites; measurement, registry and independently reconstructed report equal
  the prior sealed report. Zero retained acquisition pairs occur.
- Completed work totals **335 rewrites**. Native execution, compaction and auditing
  take **6.195 seconds**. Peak whole-workspace admission is **192.25 MiB**, below
  256 MiB. The bounded workflow takes 16.33 seconds.
- A separate preregistered interruption stops after its first durable native intent.
  The native journal bounds that action at 0–12 rewrites. The controller retains
  its entire **250-rewrite world reservation**, records no successful receipt,
  refuses another dispatch and closes operationally. The other three registered
  worlds remain `not_started`. No incomplete result becomes a scientific negative.
- Aggregate controller accounting is therefore **335–585 rewrites**, below the
  1,500-rewrite fixture ceiling. This is conservative reservation accounting,
  not a claim that 585 native rewrites occurred.
- Seven boundary-test groups and three Rust admission tests pass. All 188
  scientific execution attempts are refused before native work or journal creation.

The seven boundary groups cover workflow exhaustion, native-time exhaustion,
rewrite exhaustion, spool capacity, storage outside the spool, altered route/binary
identity, and durable failure with retry/restart refusal. They use mocked invocation
and add no native work. An initial test assertion tried to replay a deliberately
invalid zero-budget owner; that test-only assertion was corrected before freezing
or running the native fixture. The published tests use an exhausted native-time
counter and verify the untouched event directory directly.

## Evidence

Package: `output/cumulative_executor_controller_20261011`.

Seal: `2424277b247b5345f4dd7c8af6e49132cb6c03ee1f45490c58f328d371142358`.

The package contains the two authenticated parent archives, 203 frozen source
files, the executable, controller ledgers, compact native evidence, child receipts,
measurement, report and independent audit. All 196 preceding source pins remain
unchanged; no historical evidence was removed. Original and relocated checks use
the frozen `scripts/recheck_executor_controller.py`; both are read-only with zero
native execution. Their receipts are outside the sealed package.

## Remaining boundary

The controller-to-new-executor development path is complete. Scientific dispatch
is still unimplemented: it must consume the exact admitted panel requests and
measured scientific assay batches instead of compiled development route names,
bind those identities and explicit launch authorization into native journals, and
receive equivalent independent auditing. This should reuse the tested controller,
storage and measurement laws without changing chemistry, seeds, denominators or
acceptance thresholds. The proposed scientific resource ceilings still require
explicit authorization after that executable path and final contract are reviewable.
No scientific world was initialized or advanced; readiness, scientific acceptance,
evolution and launch flags remain false.
