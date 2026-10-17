# Complete-profile coherent kernel: implementation and development findings

Date: 2026-09-28. Profile `E2-complete-coherent-kernel-v1`.
**Implemented and development-tested; no scientific G-R1 acceptance or training
admission.** This is the follow-up to the verified raw-kernel numerical failure,
not a repair or promotion of that receipt.

The prospective development contract is
[`ere_complete_coherent_kernel_v1.md`](ere_complete_coherent_kernel_v1.md).
Family version 10, the complete reader/compiler, paid vocabulary, runtime
budgets, production mutator and admission implementation are unchanged.

## What was built

[`../crates/hg-sim-core/tests/support/coherent.rs`](../crates/hg-sim-core/tests/support/coherent.rs)
provides an isolated deterministic host-assay transaction. Its 13 requests
cover slots, variables, patterns, labels, priority and activation. Each request
names an allele and exact node/edge handles; there is no RNG, fitness function,
target sampler, retry loop or Engine step in the kernel.

[`../crates/hg-sim-core/tests/ere_complete_coherent.rs`](../crates/hg-sim-core/tests/ere_complete_coherent.rs)
contains 19 bounded integration tests. Cargo discovers this target automatically
when `hg-meta` is enabled. No Cargo manifest, dependency or old support file was
edited. The kernel is deliberately test/assay infrastructure, not a registered
production optimizer or ecological instruction.

### API and extension routing

| Entry point | Responsibility |
| --- | --- |
| `targets(store, root, role, limits)` | Read-only supported-presentation discovery; returns original anchor, sides, physical patterns, variables and selected body set |
| `slots(store, pattern)` | Ordered binary slot handles, sorted by role then stable generational ID |
| `apply(store, root, role, request, limits)` | Stage one exact request, check original validators and noninterference, commit only an Applied result |
| `Tx::owned` / `delete_owned` | Create/delete body declarations and ownership with exact root-count deltas; refuse implicit destructive cascades |
| `Tx::relation` / `unlink` | Add/remove one binary payload, its ordered witness and exactly two endpoint obligation deltas |
| `Tx::compact` / `prune` / `drop_pattern` | Explicit slot-role compaction and the contract's bounded semantic deletion cascades |
| `Tx::execute` | The complete request-to-edit mapping; add new scientific semantics here only under a new prospective contract |
| `outside` and final checks | Preserve world, unrelated families, unselected allele material, original anchors/sides, root data and store clock |

`Role` is Core or Proposal. Activation is a family operation even though the
caller still supplies a role. A `Scope` describes physical graph material,
not a re-encoded candidate: a pattern shared by LHS and RHS remains one object,
and its edit affects both sides. Existing anchors and sides retain their IDs.

### Transaction and metadata semantics

1. Complete-decode and compile the original family, read-only. Require supported
   explicit order hints, binary unsealed payloads and bounded store size.
   Refuse ordinary foreign attachments to the selected body.
2. Clone the whole store, preserving generational arenas and indexes. Apply
   the request only to that stage, recording primitive operations and old values.
3. Update records locally as part of the requested edit. A new owned body
   declares its schema and all 26 obligations, including zeroes. A relation
   changes only the outgoing/incoming declarations for its role. Root ownership
   changes only when a body is created or deleted. There is no metadata recount,
   bulk refresh, codec regeneration or repair preamble.
4. Apply declared cascades only: compact removed slots; prune variables with no
   surviving slot references and their unconsumed literal/guard bodies; drop
   patterns under arity two only for RemoveVar. Never strip a surviving RHS-only
   variable's guard to rescue an otherwise invalid child.
5. Complete-decode and compile the stage with the unchanged reader. Check
   unselected candidate equality, requested mode changes, exact unaffected typed
   material, root data, clock, original anchor/side identities, and membership of
   every fresh node in the selected allele. Commit only after these checks.

An unsupported or invalid request remains a failed attempt. Original caller
bytes, including allocator/free-list and index state, remain unchanged for
every non-Applied status. The returned stage is not substituted with the parent.

### Returned evidence and statuses

`Attempt` contains `status`, a JSON `receipt`, the full staged store, and optional
before/after decoded `RuleProgram`s. Receipt fields include profile, role,
request, reason/validator, committed flag, typed original/staged inventories,
primitive edit transcript, zero advancing Engine steps, and explicit false
`codec_reencoded`, `metadata_recounted` and `training_admitted` flags.

| Status | Interpretation |
| --- | --- |
| Applied | Stage passes complete/compile and noninterference checks; caller replaced atomically |
| Inapplicable | Valid input but absent/present label makes the requested label operation inapplicable; original and stage agree |
| RejectedInput | Starting family fails original complete/compile preconditions; no edit or healing |
| RejectedPolicy | Unsupported handles, presentation, sealed material, foreign attachment, cap or noninterference refusal |
| RejectedPostcondition | Edited stage fails original complete/compile validation; invalid stage and transcript retained, caller rolled back |

Applied is **not** a classification or novelty claim. Same-endpoint retargeting
can change edge IDs while preserving semantics; same-value activation or label
writes can be semantic no-ops. Dormant Proposal changes can alter genotype
without changing executable identity. A shared literal's value change affects
all its consuming variables; label deletion retains it until its final consumer
disappears. Future sampling/accounting must expose these cases explicitly.

## Development verification

The final release target passes **19/19 tests**, with no ignored tests or
scientific entrypoint. It also passes in the debug build. Coverage includes:

- Successful independently specified semantics for all 13 operators and both
  allele roles; shared patterns, multiplicity and repeated endpoints.
- Slot compaction, variable/literal/guard deletion, pattern under-arity pruning,
  shared-literal lifecycle, exact outgoing/incoming deltas and explicit-zero
  declarations on fresh bodies.
- Invalid arity, empty LHS and guarded RHS-only stages retained as failures;
  no stripping guards, silent parent replacement or resampling.
- i32 priority, variable/arity, region and 2,000-node/4,000-edge store boundaries;
  stale and other-allele targets; unsupported optional-hint presentations.
- Actual native paid CORE Seal during fixture setup, followed by host-kernel
  refusal with unchanged protection and fuel. This is not a repair trajectory.
- Byte-exact rollback, deterministic receipts and replay, world/other-family
  noninterference, original root/pattern handles and unchanged store clock.
- Read-only eligibility of all 168 existing parent genotypes. No mutations,
  sampled rates, fitness or rollouts were measured for this bank check.
- 168 deliberate damage variants on an already structurally edited family:
  all 13 roles' payload/witness loss and duplication (52), all 26 required keys
  plus schema loss/duplication/mistyping/mismatch (108), root owned-count damage
  (4), ownership loss/duplication and body/root deletion (4). Every variant
  remains invalid; the kernel returns RejectedInput without changing it.

One deterministic Engine test uses four cases: original, coherent Core edit,
dormant Proposal edit, and activated edited Proposal. Each runs four steps in
both complete-source and direct-static legs: **32 advancing development steps
per suite invocation**, with pooled 400,000/50,000 matching, no binding ceilings
and zero source quarantine. Each pair agrees on exact typed projected world,
canonical hash and RNG probe. The static leg retains identical inert meta
allocation so arena/free-list differences cannot masquerade as semantic
differences in edge handles. The Core edit changes this motif's trajectory;
the dormant edit leaves executable bytes and trajectory unchanged. This is
an effect/dormancy fixture, not a NearNeutral rate, long-horizon catastrophe
test, learned behavior or admission receipt.

Regression checks also pass: old raw G-R1 target **6 passed / 2 ignored**;
unchanged training guards **2 passed**; old checker regressions with its
existing complete development archive **12 passed**, including all four archive
checks. No old assay was invoked or rewritten. The two ignored raw harness
entries remain the opt-in development archive and separately authorized
scientific campaign.

Reproduction commands from repository root:

```sh
cargo test --release --locked -p hg-sim-core --features hg-meta --test ere_complete_coherent -- --test-threads=1
cargo test --locked -p hg-sim-core --features hg-meta --test ere_complete_coherent -- --test-threads=1
cargo test --release --locked -p hg-sim-core --features hg-meta --test ere_complete_gr1 -- --test-threads=1
cargo test --locked -p hg-train --features ere --test ere_complete_witness
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_verify_ere_complete_gr1.py output/ere_complete_gr1_smoke_complete_20260927
```

## Preserved baseline and scientific limitations

The original full archive `output/ere_complete_gr1_full_20260927` remains
byte-sealed: all **194 frozen inputs**, **205 sealed outputs**, and **177 pinned
live sources** verify after the kernel implementation. Exact output membership
also verifies, including nested basis checksum seals. Its complete numerical
failure and closed training flags are unchanged. This preservation check is
hash verification, not a newly executed full semantic audit.

No new scientific archive, stochastic selector, frozen coherent-kernel auditor,
rate pilot, 100-step assay, optimizer or production admission path was created
or run. Unit-test stages/receipts are in memory; this document records development
results and replay commands, not independently audited persisted child evidence.
The prospective contract preceded implementation; it is not a frozen scientific
precommitment or acceptance design.

The host transaction provides atomic powers unavailable to a sequence of
ecological rewrites. It does not prove affordable native coherent edits,
maintenance under continuing damage, whole-family recovery, unbiased sampling,
population navigability or optimizer signal. Rejected attempts must stay in
future attempted-operator denominators, never become valid Neutral rollbacks.

## Next bounded step

Build a separately named prospective coherent-assay design, deterministic
selector/manifest producer and independent frozen auditor. They should freeze
choices before applying this API; independently replay primitive edits and
ownership/obligation changes; retain failed stages; distinguish operator labels
from actual logical topology changes; separate active/dormant/no-op cases; and
preserve the raw-kernel counterfactual and full damage controls.

After bounded archive/tamper verification, request separate authorization for
any new scientific trial with its exact sampling, gates and resource ceilings.
Do not change the failed raw contract, silently relax its 40%/10% structural
lines, launch a retry or spend the reserved optimizer budget. Scientific G-R1
and complete-profile training remain closed.

## Subsequent harness task (2026-09-28)

The next user-authorized task implemented a separately contracted coherent
harness and frozen auditor. See `ere_complete_coherent_assay_implementation.md`
and `ere_complete_coherent_assay_v1.md` / `.json`. Its clean eight-step receipt
passes independent evidence replay and sealing, not scientific acceptance.
It records two allocation-sensitive Neutral soundness failures; the kernel
remains pinned and unchanged. A bounded identity/allocator follow-up is
recommended before any full scientific trial. Training stays closed.
