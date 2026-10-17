# Complete coherent full assay: preparation and launch status

Date: 2026-09-29. Profile `E2-complete-coherent-draw-assay-v1`.

## Outcome

The prospective full contract, opt-in producer, independent auditor and
preflight tests are implemented. All 14 release Rust checks, 15 native-fixture
Python checks and 42 Python integration/differential checks pass.
The full scientific campaign has not been authorized or launched. Neither
a new development campaign nor training ran during this preparation.
The planned output directory does not exist.

[Preregistration](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_draw_assay_v1.md)
and [exact parameters and evidence pins](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_draw_assay_v1.json).
[Machine-readable preparation receipt](/Users/kalewicker/Documents/hypersim/docs/ere_complete_coherent_draw_assay_preflight_20260929.json).

## Implementation and extension points

| Component | Responsibility |
| --- | --- |
| [Rust harness](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_complete_coherent_draw_assay.rs) | Fresh-directory launch guard; source/binary/material freeze; native controls and staged trajectories; independent audits, budgets and failure preservation |
| [Independent auditor](/Users/kalewicker/Documents/hypersim/scripts/verify_ere_complete_coherent_draw_assay.py) | Frozen contract/authorization/population integrity; full material/trace/classification replay; pilot-prefix regression; exclusive receipts and output seal |
| [Native-fixture tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_complete_coherent_draw_assay.py) | Operator/role transformations, decoder agreement, exact identity and rollback/denominator predicates |
| [Integration and full-mode tests](/Users/kalewicker/Documents/hypersim/scripts/test_verify_ere_coherent_draw_assay_integration.py) | Frozen differential evidence; semantic preservation/ownership; 10,000-row population and numerical predicates; authorization, parity, caps and integrity |

This is a new isolated assay profile; no production Engine/mutator/allocator
code was changed. Earlier profiles and archives remain untouched.

Twenty-four native scientific functions and the complete native Tally
implementation are text-identical to the validated pilot. Sixteen selected
Python semantic functions/classes are AST-identical to the winning draw-local
checker. Native mutation, matching, child selection, exact typed identity,
classification precedence and all thresholds remain unchanged.

The campaign selects 10,000 draws/100 steps in scientific mode, using the same
entire 168-parent population and seed. Development mode retains the
64-draw/100-step envelope solely for fixture testing. Each full invocation
requires exact approval tokens before creating output; it freezes a launch
attestation binding the caps and both contract hashes. The auditor requires
explicit matching mode and verifies that attestation and its frozen inclusion.

Freezing now independently checks exact parent/draw/control counts, draw
indices/operator/role/RNG-position envelope and control attack order.
Full-size compressed freezing remains a prerequisite of the future launch;
it has not been executed as a campaign during this preparation.

Full parity checks all fixed parent/control streams and the exact first
64 requests/classifications plus their 55 complete child traces against
the latest sealed pilot. The rest of the scientific sample is fully replayed
and its entire summary independently recomputed. No equality is required
between the 10,000-draw summary and a 64-draw diagnostic summary.

## Verification evidence and boundaries

The release suite passes 14 checks, with both campaign entrypoints ignored.
The native selector check constructs all 10,000 requests without Engine steps,
verifies the pilot's first 64 exactly, and covers all 13 operators, three roles
(Core/Proposal/Family) and both activation states. Its local monotonic wall
clock is 122.825000375 seconds.

A temporary new-profile fixture freezes current sources/contracts and then
replays the preserved pilot's controls, all 168 parent traces, 55 child traces
and classifications through the complete new auditor. Audit and measurement
checks pass, 27 tamper categories reject, and gate/training stay false.
The fixture's 22,300 attempted steps are **prior recorded evidence**, not newly
executed campaign steps. Temporary fixture material is not scientific output.

The Python suite checks a synthetic 10,000-row numerical-pass fixture and
rejects missing samples, soundness failures, Unmeasured, insufficient pattern/
activation effects, missing role/state/cap coverage and excessive catastrophe.
Synthetic predicate success is a unit test, not an observed scientific result.
It also checks omitted/duplicate frozen rows, bad roles/word positions,
missing/changed approval, typed canonical-byte prefix changes, historical/mode
refusal, disabled assertions, bounded exclusive writes and nested basis seals.

All 17 relevant historical receipt seals verify, including the latest pilot's
311 outputs and every nested basis receipt. Exact source and basis pins match
the live material. The final Rust rebuild rechecks the updated contract hashes,
native fixtures, current source freeze and complete auditor replay; the unchanged
10,000-selector test had already passed in the preceding release suite.
A new test-helper regex initially missed generic Rust function names; its
recognition was fixed and the final 42-test suite passes. No scientific
intervention or historical evidence was changed to resolve that test issue.

No full numerical outcome or training admission is asserted by these tests.

## Resource sizing correction

Retain the existing proposed caps: one thread, 1,016,800 attempted/advancing
steps, 3,600 total wall seconds, 900 shared audit seconds, 4 GiB output,
1 GiB auditor RSS, pooled 400k/50k matching and 2,000/4,000 store caps.
The freeze audit has a 120-second sublimit charged to the shared allowance.
No automatic retry or limit increase is implemented.

The prior pilot-based scenario was ~722 audit seconds and ~3,429 total seconds.
The total used the pilot's small mostly fixed setup/freezing proxy; it did not
scale the full request-generation/compression/read cost. The 123-second
zero-Engine-step selector preflight shows that cost is material.

Simply adding this local preflight gives ~3,552 seconds and ~48 seconds nominal
headroom. This is a scheduling illustration, not a phase-disjoint new estimate:
it overlaps some prior setup cost, includes fixture/prefix checks, and omits
full manifest compression/read, larger source/provenance integrity and process
variability. The complete full-size freeze cost and future child-cost mix remain
unmeasured. Completion within the fixed wall allowance is uncertain.
The new contract makes the sizing gap explicit rather than claiming
3,429 seconds is a fully accounted full-run forecast.

## Launch decision and reproduction

Preparation is complete; explicit approval for one scientific invocation is
the remaining boundary. This does not include training, retries or larger
resources. On cap/error retain partial material with failed/unmeasured status
and gate/training false. A completed numerical negative remains a scientific
negative; no criteria change is automatic.

After that approval, the exact launch command is:

```sh
env ERE_COMPLETE_COHERENT_DRAW_ASSAY_OUTPUT=/Users/kalewicker/Documents/hypersim/output/ere_complete_coherent_draw_full_20260929 \
  ERE_COMPLETE_COHERENT_DRAW_ASSAY_APPROVED_STEPS=1016800 \
  ERE_COMPLETE_COHERENT_DRAW_ASSAY_APPROVED_CAPS='3600s,4GiB,900s,1GiB,400k/50k' \
  ERE_COMPLETE_COHERENT_DRAW_ASSAY_APPROVED_CONTRACT=80c8cbc072a393d8bddd921b5c4dbf88ac60e8e4b1ebb50dfb7a0683aa967dba \
  cargo test --release --locked -p hg-sim-core --features hg-meta \
  --test ere_complete_coherent_draw_assay complete_coherent_draw_frozen_assay \
  -- --ignored --exact --nocapture --test-threads=1
```

Do not execute that command merely to test the approval guard: the full-mode
unit tests already cover refusal before output creation.

Implementation checks:

```sh
cargo test --release --locked -p hg-sim-core --features hg-meta \
  --test ere_complete_coherent_draw_assay -- --test-threads=1 --nocapture
python3 -B scripts/test_verify_ere_coherent_draw_assay_integration.py
```

Once a future full receipt is sealed, the frozen auditor can replay it
read-only with `--scientific --stage full --verify-only` and an explicit bounded
verification allowance. Verification time is separate from campaign accounting
and cannot repair/reseal a failed archive.

Contract Markdown SHA256: `8a0e97586c4c7f84c28c105b246ffaeb8688618ae659c9ea3716ed5b7bba1d46`.
Contract JSON SHA256: `80c8cbc072a393d8bddd921b5c4dbf88ac60e8e4b1ebb50dfb7a0683aa967dba`.
