# Durable journal and continuation readiness — 2026-10-09

The bounded durability fixture passes its engineering boundary. All 14 registered
routes are retained: complete worlds and assays, committed interruptions and safe
continuations, pending and torn results, an unchanged-budget stop, a separately
charged checkpoint replay, and three refused unsafe continuations. Thirty-eight
Rust and 54 Python test groups pass, including preserved regressions. The original
and relocated frozen rechecks pass with zero native work.

Full readiness, scientific acceptance, evolution and campaign-launch flags remain
false. This implements the next engineering component of the single final design
in `docs/final_cumulative_complexity_experiment_proposal_20261009.md`; it adds no
scientific worlds, seeds, horizons, innovations or outcome-dependent retries.

## Durable action boundaries

The additive module
`experiments/cumulative/src/population/regulation/world/journal.rs` leaves the
existing Cell, population, lineage, editor and shared-world bodies unchanged.
It uses the same 12 development worlds, 96 census slots and 33 frozen assay
requests as the authenticated integrated parent. Complete trajectories, version
measurement and assay results match that parent exactly.

Before each native tick or assay, a journal intent reserves a deterministic
upper bound on native rewrites. After execution, one result record commits both
the actual native work counter and the full row/allocator checkpoint, or the
complete standardized assay. Exact record bytes form a SHA256 chain. Publication
creates a fresh staging file, writes JSON plus a newline, synchronizes it, links
it to a fresh sequence filename without overwrite, synchronizes the directory,
removes its own staging link, and synchronizes the directory again.

Every initial state and ancestral row remains in the journal. An explicit safe
continuation verifies the source chain and inputs, copies its complete prefix
into a fresh journal, restores the last committed checkpoint, and executes only
the remaining fixed horizon. The original source stays unchanged. Only one
continuation hop is admitted. Changing seed, input, source identity or original
biological work budget rejects before native work. The invocation's separate
execution allowance is drawn from the remaining aggregate budget; it does not
increase the source's biological budget.

An unpaired intent retains a funded interval of unknown work. Recovery never
infers its missing result, counts a torn stage as committed work, releases that
reservation, or retries that action. A stage left after publication must exactly
match the last committed record; the auditor can recognize it, while the native
entry point conservatively refuses any staged source. This recognition is also
covered by an adversarial audit test.

The independent Python auditor reconstructs the journal sequence, native law,
world scheduling, energy ledger, wire checkpoint projection, complete ancestry,
fixed assay identity mapping, native work intervals and aggregate reservations.
Adversarial tests rebuild hash chains after semantic corruption, so successful
rejection is not just detection of a stale hash.

## Fixed route results

| Route | Saved status | Native work in this invocation |
|---|---|---:|
| Complete 12-world denominator | Complete | 151 |
| World interruption after committed tick 1 | Partial, safe boundary | 6 |
| Explicit continuation of that boundary | Complete, identical full denominator | 145 additional |
| World interruption before result commit | Partial, unresumable | 4–9 |
| Torn world result stage | Partial, unresumable; raw stage retained | 4–9 |
| Original world budget of one rewrite | Budget stop before native work | 0 |
| Version-witness replay from tick 2 through tick 4 | Complete, exact suffix/checkpoint | 6 additional |
| Complete 33-assay denominator | Complete | 324 |
| Assay interruption after committed request 2 | Partial, safe boundary | 24 |
| Explicit continuation of that assay boundary | Complete, identical full denominator | 300 additional |
| Assay interruption before result commit | Partial, unresumable | 16–36 |
| Continuation from uncertain world | Refused before publication/work | 0 |
| Continuation from torn world | Refused before publication/work | 0 |
| Continuation from uncertain assay | Refused before publication/work | 0 |

Each partial world report retains all 96 slots, distinguishing committed censuses,
not-started cases, unreached censuses and outside-horizon slots. Partial assay
reports retain all 33 request slots. A complete package means the declared
engineering route panel finished and was verified; its interrupted routes remain
partial. They are never reclassified as complete scientific runs.

Safe world continuation costs 6 + 145 = 151, and safe assay continuation costs
24 + 300 = 324. The prefix is not executed twice. The diagnostic replay costs six
additional rewrites and retains the complete source ancestry; it is never a
replacement world, discovery candidate or favorable checkpoint selection.
The unchanged native panel still contains zero retained two-innovation pairs
and zero discovery candidate passes.

## Immutable evidence and verification

The successful package is
`output/cumulative_durable_journal_readiness_v3_20261009`, with seal root
`8ae372d4e9f71cb34441453d2333dd524b0431b025028f20c7226c19f1709fe1`.
It contains 803 sealed entries, 137 first-party/borrowed source pins and 136
additional registry source pins. The seven new SHA dependency packages are also
retained as cached package archives authenticated by their Cargo.lock checksums;
extracted source bytes were checked against those packages before execution.

The measured publication workflow takes 71.674 seconds, including 29.069 native
command seconds, and retains 23,963,746 payload bytes. Its aggregate native work
is **2,745–2,775**, comprising 1,765 known test rewrites and independently audited
route work/reservations. The deliberate missing results leave 30 rewrites of
uncertainty, so `native_work_unknown` remains true. Reporting an exact aggregate
would discard the intended failure evidence. The upper bound fits the registered
6,000 rewrites, 300 native seconds, 1,200 workflow seconds and 128 MiB. Prepublication
native development checks separately used 348 test rewrites and six committed
world rewrites; even including those, the observed upper bound is 3,129.

`inputs/`, `commands/`, `journals/` and `recovery/` retain every route. `audit.json`
retains the independent reconstruction; `request_freeze.json` authenticates inputs
fixed before all native routes. `parent_equivalence.json` records canonical hashes
of the authenticated parent worlds, measurement and assay results. The executable,
source closure, registry closure, contract, work counters and preservation receipts
are covered by `output_sha256.json`.

Two earlier publication attempts failed during dependency evidence collection,
before any native tests or routes ran. They remain sealed as failures:

- `output/cumulative_durable_journal_readiness_20261009`, root
  `3f135cabdd1fa18fdb72a5b757ff26616ef9e1c91fc108293a23555385297786`:
  full Cargo metadata attempted an unavailable unpack outside the workspace.
- `output/cumulative_durable_journal_readiness_v2_20261009`, root
  `89cd6ffa5b241e0025a96b9719f8cc4698d2ebc242db74e63edde868f3550fcf`:
  the evidence collector assumed checksum sidecars that this cache does not have.

The successful collector inventories root metadata without unrelated dependency
resolution, verifies the seven cached package archives directly, and compares all
extracted package files. No biological inputs, routes, thresholds or budgets
changed between attempts. The six earlier historical roots and both failed
publication roots verify before and after the successful workflow.

The frozen recheck verifies the complete archive, source snapshots, executable
identity and admission flags, then repeats the independent audit and ten
adversarial groups. It resolves source journals within the relocated package,
without executing the native binary or relying on live source files. Receipts:

- `output/cumulative_durable_journal_readiness_v3_20261009_recheck.json`
- `output/cumulative_durable_journal_readiness_v3_20261009_relocated_recheck.json`

```sh
python3 -B experiments/cumulative/scripts/recheck_durability.py output/cumulative_durable_journal_readiness_v3_20261009 8ae372d4e9f71cb34441453d2333dd524b0431b025028f20c7226c19f1709fe1
```

## Remaining gate

This is evidence for the registered interruption/publication model on bounded
development worlds and assays. It is not hardware power-loss, arbitrary filesystem
failure or adversarial-storage certification. Independent RNG and allocator replay
remain false; exact native continuation and independent prefix accounting are
separate claims. Whole scientific campaign routes remain unadmitted.

The next boundary is whole-campaign compute/storage feasibility and an explicit
sensitivity/power justification for the independent-world count. Then freeze the
prospective sampling, success thresholds, world panel and operational limits
together. Retain the accepted readiness cap and close this configuration if these
requirements cannot fit it. This result does not authorize a larger search or a
scientific launch.

The subsequent zero-native planning result is now sealed in
`docs/cumulative_campaign_planning_findings_20261009.md`. It proposes a powered
world count and comparable prospective functional/interaction criteria, while
keeping campaign feasibility false. A scientific budget has never been approved;
the readiness-fixture allowance must not be silently reinterpreted as that budget.
