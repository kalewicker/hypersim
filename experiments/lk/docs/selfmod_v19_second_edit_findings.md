# Selfmod v19 — interrupted second-edit assay

**The assay stopped on a genotype mismatch; there is no second-edit durability
result.** All eight fresh worlds completed the 10,000-step establishment phase
and passed the fixed readiness criterion. At seed 313's first qualifying second
birth, the fixed native mutation parameters removed the variable-1 constraint
instead of the preregistered variable-0 constraint. The exact genotype assertion
stopped the run before any post-second-birth treatment continuations began.

The failed scientific archive remains unchanged at
`output/selfmod_v19_second_edit_20260928`, with authoritative `status.json`
state `failed` and zero treatment trajectories completed. This is an
intervention-realization failure, not evidence that the intended second edit
loses financing, productivity or durability. No full-assay pass/fail fraction,
paired treatment comparison, training admission or further-edit viability
claim can be inferred. Historical v15, v17 and v18 verdicts are unchanged.

## What completed

The fixed eight-seed panel was 311, 313, 317, 331, 337, 347, 349 and 353.
An independent read-only reconstruction verified every initial prefix, paired
paid initial birth, full establishment trajectory, checkpoints, allocator and
readiness criterion. It also verified the completed second-search receipts and
saved valid paid forks; this is explicitly an interrupted-stage verification,
not the complete assay audit.

| Seed | First R1 prefix steps | Readiness | Second-search steps executed | Saved second paid forks |
|---|---:|---|---:|---|
| 311 | 20 | Pass | 29 | Faithful and exact edited child |
| 313 | 30 | Pass | 17 | Faithful; edited genotype rejected |
| 317 | 14 | Pass | 0 | Search not run |
| 331 | 16 | Pass | 0 | Search not run |
| 337 | 17 | Pass | 0 | Search not run |
| 347 | 14 | Pass | 0 | Search not run |
| 349 | 22 | Pass | 0 | Search not run |
| 353 | 23 | Pass | 0 | Search not run |

Actual scientific advancement was **80,202 steps**: 156 initial prefix,
80,000 establishment and 46 second-search steps. Treatment continuation steps
were **zero**. All eight seeds remain consumed by this scientific attempt and
must not be reused in a fresh replacement panel. Both second paid phases at
311 were materialized, but neither received a treatment continuation. At 313,
the post-object fork and faithful paid phase were saved; the incorrect edited
child was not saved as an admitted branch. Remaining readiness verdicts were
reconstructed from complete raw establishment output after interruption.

Interrupted-stage verification is separately archived at
`output/selfmod_v19_interruption_verification_20260928/interrupted_run_verification.json`.
It authenticates the frozen source/binary/contracts and references, independently
reconstructs all completed stages and sets `full_assay_complete=false` and
`assay_verdict=null`. Original failed status/raw records were not repaired.

## Cause and concrete correction

Native operator 3 (`DropPnodeConstraint`) removes
`children(root,HAS_PNODE)[parameter0 % number_of_constraints]`. `children()`
sorts allocated slotmap node IDs rather than semantic variable IDs. The fixed
parameter 0 is even, so it chooses index 0 in a two-constraint child. The saved
native faithful material orders are:

| Seed | Native ordered constraint IDs and variables | Fixed draw removes |
|---|---|---|
| 311 | 717259538442:var0, 760209211433:var1 | var0, as specified |
| 313 | 493921239082:var1, 579820584969:var0 | var1, mismatch |

The intended labels were `[null,1]`; the rejected child decoded as `[0,null]`.
Required LHS edge and pruned RHS structure were otherwise unchanged. Allocation
reuse across generations makes a fixed raw draw insufficient to define a
semantic constraint removal. This is separate from the native fission/kernel
and from the descendant-family scoring machinery.

An opt-in engineering adapter now resolves the unique actual-child constraint
with `K_VAR_ID=0`, replaces only intervention parameter 0 with its native
index, then calls the unchanged native operator exactly once. It records both
parameter vectors and the complete constraint order. Missing targets or
incorrect composite genotypes stop execution; it does not repair a failed edit.
The existing fixed-parameter path remains available unchanged, and the failed
frozen v19 binary/source are preserved. This adapter is a changed intervention
definition, requiring a separately registered new assay.

## Engineering evidence and limits

The original engineering preflight at seeds 7/42 passed 37 library tests,
six unchanged durability test methods and five ancestry methods; full readiness,
four 2,000-step second branches, exact split continuations and frozen v18 parity
passed. Nine receipt corruption classes and five missing/readiness/denominator
cases were rejected. Its 56,200 advancing steps were excluded from science.
Both native constraint orders happened to be `[var0,var1]`, so this preflight
did not expose the allocation-order assumption.

The corrected engineering archive is
`output/selfmod_v19_semantic_target_engineering_20260928`. **39 library tests**
now pass, including explicit `[var0,var1]` and `[var1,var0]` material-order
fixtures and rejection of an absent semantic target. The six unchanged
criterion methods and five ancestry methods still pass.

Four saved post-object forks, at 7, 42, 311 and 313, were restored for
engineering-only paid-phase checks. These advance **zero object steps**.
The independent checker verifies exact faithful state/allocator parity with
saved native evidence, exact edited composite/hash 4713340462273116244,
unchanged native payment, donor retirement, allowance, siblings and RNG,
and removal of exactly the var0 material node and its role edge. All other
live rule material is unchanged. Both native constraint orders pass. Hashes
confirm the complete failed science and original preflight archives were not
modified. The corrected frozen source extraction compiles offline/locked.

These checks establish the targeting correction at paid births, not durability
of its descendants. No corrected scientific continuation or automatic fresh
panel was launched, and no failed archive was overwritten.

## Proposed next assay

The reviewable proposal is `selfmod_v19b_second_edit_preregistration.md` with
`selfmod_v19b_second_edit_contract.json`. It changes only the semantic targeting
definition, uses fresh seeds **359, 367, 373, 379, 383, 389, 397 and 401**, and
retains readiness, physical conditions, native financing, paired faithful
second-child control, immutable ancestry, 10,000-step horizon and 6/8 criterion.
The proposed fresh inventory includes the failed v19 panel and has no overlap.

Before launching, complete a revised full engineering preflight and semantic
metadata corruption audit, freeze its own instrument, and compile its extracted
source. The v19 preregistration explicitly requires preserving failures and
**no automatic scientific retry**. The revised fresh assay therefore awaits
approval; its contract is marked proposed and not authorized. No root ERE
work, broad mutation campaign or training run is part of this work.
