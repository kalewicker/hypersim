# Live development controller integration — 2026-10-10

The controller now completes a bounded live development flow: frozen world
requests, actual clock/storage admission, durable intents, native execution,
independent compact-journal audits, complete census/registry construction, assay
dispatch and a content-bound final report. The scientific native profile remains
unimplemented, and scientific execution is still blocked.

## Fixed execution and results

The contract fixes four matched worlds in endogenous, external, nonheritable and
frozen order, all using development seed `2026101001`. They run exactly once in
this integration fixture. The complete measurement then determines nine endpoint
requests, which run once in a single batch. No scientific world is executed.

| Native unit | Committed rewrites |
|---|---:|
| Endogenous world | 72 |
| External world | 66 |
| Nonheritable world | 73 |
| Frozen world | 64 |
| Complete nine-assay batch | 60 |
| **Total new native work** | **335** |

All new world rows and final allocator/checkpoint states exactly match their
previously sealed development counterparts. Independent reconstruction retains
all 40 census slots and reproduces the complete measurement, registry and report.
There are zero retained acquisition pairs; none of these repeated development
checks is an independent scientific replicate or evidence of evolved complexity.

The fixed development ceilings are 1,200 native rewrites, 180 native seconds,
30 seconds per native unit, 1,200 workflow seconds, a 256 MiB spool and a separate
128 MiB fixture output cap. The 335 rewrites use 6.401 measured seconds including
native dispatch and archival. The full workflow takes 17.72 seconds. Peak actual
spool admission charge is 124,518,400 bytes (**118.75 MiB**). This milestone uses
instrumented monotonic times and the actual spool ledger, rather than synthetic
resource inputs. The auditor checks those receipts and bounds; it is not an
independent hardware timing measurement.

## Dispatch and failure behavior

`experiments/cumulative/scripts/live_campaign_dispatch.py` verifies the exact
frozen request and binary, checks the actual spool capacity, and publishes a durable
controller intent before invoking the native child. The child's timeout is bounded
by the per-unit allowance, remaining aggregate native time and remaining workflow
time. A controller receipt is admitted only after the native journal is compacted,
independently audited and confirmed complete within its funded allowance.

Changed seeds, purpose, requests, funding or executable hashes fail before dispatch.
Clock exhaustion and insufficient storage fail before a controller intent. A child
failure leaves the funded intent unresolved; a second dispatch is refused. A reopened
controller cannot acquire a fresh clock epoch and resume execution. Its evidence
can be audited and the funded attempt closed separately, preserving uncertainty.

Five dispatch test groups and eight unchanged controller test groups pass. They
include an executor spy proving the intent is durable when invocation begins,
fault injection before native work, restart refusal, actual clock/storage rejection,
and preserved pending reservations. All five live native units additionally pass
independent controller/spool/request/journal correspondence checks.

## Sealed evidence

Package: `output/cumulative_live_controller_20261010`.

Seal: `4a7ef1f96d6898bc3990d81af31441854098f844e888cc51c39c02e083ce3f23`.

- `controller/`: immutable world order, intents, receipts, registry barrier and closure.
- `native_spool/`: actual admission charges, native requests, exits and compact journals.
- `measurement.json`, `registry.json`, `report.json`: complete measured denominator.
- `native_routes.json`, `audit.json`: native/controller bindings and independent results.
- `measurement_parent/`: exact sealed reference, including its preserved assembly failure.
- `remaining_launch_requirements.json`: scientific native admission and authorization gaps.

The package freezes 180 source files and seals 2,666 entries / 22,891,124 payload
bytes. All seventeen preceding roots and 174 preceding source pins remain unchanged.
The native binary and chemistry were not changed or rebuilt. Original and relocated
known-root audits pass with zero additional native work. Their external receipts
are `output/cumulative_live_controller_20261010_recheck.json` and
`output/cumulative_live_controller_20261010_relocated_recheck.json`.

## Remaining boundary

Live **development** dispatch is integrated. The next implementation must add
versioned native admission restricted to the exact fixed 47-block scientific
panel and independently audit that profile without starting scientific worlds.
The existing adapter only accepts the development seed. After that admission check,
freeze the final scoped scientific contract and present its proposed budget and
single launch for explicit authorization. The current development allowance does
not authorize the proposed scientific budget. All scientific readiness, acceptance,
evolution and launch flags remain false.
