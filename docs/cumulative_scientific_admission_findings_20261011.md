# Frozen scientific admission checks — 2026-10-11

A new Rust admission validator now checks the exact frozen scientific panel and
global assay request schema without constructing worlds, creating journals or
executing native transitions. All **421 predeclared checks** pass against an
independent Python validator: **194 accepted and 227 rejected**. Scientific execution
remains disabled and unimplemented in this entry point.

## Implemented checks

The auto-discovered binary `cumulative-scientific-admission` has one interface:
`--check REQUEST`. Its shared validator is
`experiments/cumulative/src/bin/support/scientific_admission.rs`. Existing native
modules, Cargo manifests and scientific criteria remain unchanged.

The binary embeds `scientific_panel.json`, `scientific_admission_contract.json`
and the existing prospective planning contract. Their hashes bind the original
47 blocks / 188 world identities and all world fields, including seed, arm,
founder programs, resource settings and horizon. A request must be an exact panel
member, with its exact conservative arm reservation. It cannot replace a seed,
alter a world specification or introduce unknown fields.

Assay admission validates the complete registry, including rows outside the selected
batch. It enforces contiguous global IDs, all thirteen permitted roles, ordered
stage vectors, registry hashes, at most 78,960 requests and fixed aligned batches
of at most 128. The required assay reservation is exactly twenty rewrites per
selected request. Both world and assay checks enforce the proposed fixed funding
and journal limits, typed integers and explicit null continuation/fault fields.
Duplicate JSON keys are rejected at every nesting level, avoiding parser ambiguity.

These are structural checks against a proposed budget. `launch_authorized` must be
false; execution mode and claimed launch authorization are rejected. The check
does not constitute approval of the proposed 2,000,000-rewrite scientific budget.

## Frozen test evidence

- All 188 exact scientific world requests pass; a changed-seed counterpart for
  every world fails.
- Six valid assay cases cover every role, full and tail batches, the maximum
  registry size and the complete permitted stage domain.
- Further refusals cover changed world fields, missing identities, changed funding,
  wrong hashes, duplicate keys, booleans/floats in integer fields, invalid stages,
  empty or oversized programs, malformed batch boundaries, invalid final registry
  rows, excess registry length, excess encoded bytes and execution requests.
- Three Rust test groups verify compiled provenance and strict JSON parsing.

The native results match the independent Python oracle exactly, including admitted
world indices, hashes, batch ranges and reservations. Every process reports zero
native rewrites. The isolated working directory remains empty. No scientific world
was initialized or advanced, and no new native journal was created.

## Archive and relocation

Package: `output/cumulative_scientific_admission_20261011`.

Seal: `4f3e56d081c30a7e5aa7098ab962314ae08631b791758f31fb8ee601ca3f9ba4`.

The package contains 3,710 sealed entries, 77,198,752 payload bytes and 188 frozen
source files. The bounded workflow takes 22.74 seconds. All eighteen preceding
archive roots and 180 preceding source pins remain unchanged. The sealed live
development parent is included and independently rechecked.

The negative input `requests/encoded_byte_cap.json` is deliberately 32 MiB plus one
byte, consisting entirely of spaces. The generic native-evidence inventory helper
caps every file at 32 MiB, so **use `scripts/recheck_admission_boundary.py` for this
particular package**, rather than `recheck_scientific_admission.py` or the generic
`source_inventory`/`pack` helper. This supplemental read-only checker permits the
extra byte only for that exact named, sized and content-checked negative input.
Every file still must match the known seal, all other per-file limits remain
unchanged, and the total fixture stays below 128 MiB. The original sealed package
and its frozen source are not edited to accommodate the recheck.

Original and relocated rechecks pass using the frozen independent auditors plus
the supplemental checker. Its source SHA-256 is
`cb670993283b2c63cb92c13751f632a89b10d3cb128a677e4fcf8584696bd7b9`;
both receipts record this provenance. Receipts and a retained copy of that checker
are outside the sealed package under
`output/cumulative_scientific_admission_20261011_recheck.json`,
`output/cumulative_scientific_admission_20261011_relocated_recheck.json` and
`output/cumulative_scientific_admission_20261011_boundary_checker.py`.
Neither recheck runs the native checker or advances a world.

## Remaining execution boundary

The native **admission validator** is implemented and verified. A versioned
scientific execution entry point still needs to consume this shared validator,
connect to the controller and have its scientific journal interpretation audited.
The existing development executor continues to reject scientific seeds. This
distinction remains explicit in `remaining_launch_requirements.json`:
`native_admission_validator_implemented: true`,
`scientific_execution_adapter_implemented: false`.

After that integration, finalize the scoped scientific contract and obtain explicit
authorization for its resource ceilings and one fixed launch. All readiness,
scientific acceptance, evolution and launch flags remain false.
