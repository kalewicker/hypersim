# Native executor integration — 2026-10-11

The new versioned entry point consumes the unchanged scientific admission
validator and contains a working, durably journaled native unit loop. Its fixed
development routes pass independent auditing. Scientific execution remains
unconditionally disabled: the campaign controller has not yet been connected to
this runner, and the proposed scientific resource envelope is not approved.
This is an engineering result, not evidence of evolved complexity.

## Changes and bounded results

`experiments/cumulative/src/bin/cumulative-scientific-executor.rs` provides
check-only admission, explicit scientific execution refusal, and six compiled
development routes. Development callers select a route name; they cannot supply
another seed, world, registry, budget or fault. The loop uses the existing world
and assay implementations. Each native action follows a synced intent, and each
result retains its complete checkpoint or assay trace and native work count.
The writer requires a fresh directory and offers no resume or retry operation.

- All 188 scientific world requests pass the shared validator and independent
  oracle. All 188 execution attempts, including an explicit forged approval
  assertion used only as a negative test, fail with zero native work and no journal.
- Four fixed development worlds use 275 rewrites. Every result row and checkpoint
  exactly matches the authenticated earlier adapter archive.
- Thirteen authored intervention controls use 152 rewrites. Their scores remain
  `[0, 7, 7, 22, 22, -5, 7, -10, -5, 22, 22, 2, -10]`.
- An interruption immediately after the first intent preserves a 12-rewrite
  reservation. Its journal is unknown, despite the process reporting zero work.
  The audit therefore retains a total interval of **427–439 rewrites**, below
  the frozen 1,500-rewrite ceiling. The route was not retried.
- Reusing an existing journal is refused before native work. Three Rust admission
  tests pass. Eight corruptions with recomputed hash chains are rejected by the
  independent semantic auditor, covering identity, funding, counters, checkpoints,
  missing intents, premature completion and records after completion.

Native command time is 5.53 seconds under 180 seconds; the complete bounded workflow
takes 18.21 seconds under 1,200 seconds. These are development measurements, not
certified scientific campaign runtime bounds. All 188 preceding source pins are
unchanged. The admission parent and historical adapter reference are checked
against their known archive roots; no historical evidence was removed.

## Evidence and relocation

Package: `output/cumulative_scientific_executor_20261011`.

Seal: `757c831eb022cecc2458b411808184c5ae5354d936fc451c6f8b85396c59e057`.

The package freezes 196 source files, its executable, requests, command receipts,
native journals, authenticated historical reference files and independent audit.
Use its frozen `scripts/recheck_scientific_executor.py` with the seal above.
Original and relocated checks run only Python auditing and corruption tests;
they do not rerun the native experiment.

The first relocation invocation used macOS's `/var` alias and was correctly
refused by the existing symlink-ancestor guard. That refusal is retained outside
the sealed package as `_relocated_alias_refusal.json`. The subsequent check uses
the resolved `/private/var` path; no source, evidence, or path-safety rule changes
were needed.

## Remaining launch work

The native unit loop and development journal interpretation now exist. The
scientific execution branch deliberately remains locked; it is not yet a usable
scientific campaign adapter. The next implementation must connect this runner to
the durable campaign controller, enforce actual aggregate time/storage/work
limits, and bind scientific request identities to independently audited journals.
It must retain all 188 worlds and 1,880 sampling slots, all unknown outcomes and
the original frozen acceptance criteria. Then finalize the scoped contract and
obtain explicit authorization for one fixed launch. No scientific world was
initialized or advanced, and all scientific/readiness/launch flags remain false.
