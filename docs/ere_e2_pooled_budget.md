# E2 pooled matching: frozen implementation and verification plan

Preregistered 2026-09-26, before pooled results. This is a matching-resource
allocation trial, not a new rule encoding, mutation kernel, or relaxed gate.
Protocol: `E2-alternatives-pooled-v1`; historical split-policy E2 receipts remain
unchanged and cannot authorize a pooled campaign.

## Hypothesis and policy

The existing engine divides total work/results equally by table length. This
can halt an expensive alternative while another rule has unused allocation.
Explicit `MatchBudgetPolicy::Pooled` will enumerate enabled ordinary rules in
their installed, deterministic rule-id order, passing remaining global work
and result capacity to each matcher. Debit actual enumeration work and returned
bindings, not the whole allowance. Limits reset each step. Stop at the first
truncation; apply **none** of an incomplete candidate pool, including scheduled
creation bindings. Creation bindings retain their existing schedule semantics
and do not consume ordinary matching allocation. No candidate is skipped because
of priority or fitness. Matching uses no RNG, and completed pools retain their
canonical sort, existing selection, conflict resolution and apply semantics.

The default remains Split, including parallel matching and historical config
hashes. Pooled matching intentionally uses deterministic serial enumeration even
in parallel builds; no thread race determines budget ownership. Matcher exact
boundary behavior is unchanged and conservative: exhausting capacity before
proving enumeration complete remains truncation. Zero remaining capacity must
never be clamped to one.

## Verification and decision rule

1. Test pooled borrowing, aggregate work/result caps, zero/exact boundaries,
   disabled and scheduled rules, per-step reset, epoch table replacement,
   RNG preservation, binding-pool equality on small fixtures, and identical
   trajectories for every step mode when both policies complete. Test parallel
   and nonparallel builds and thread-count determinism.
2. Replay the 14 recorded E2 cap cases using their exact original mutation draws.
   Reproduce each split-policy failure. Compare pooled completed prefixes and
   RNG states to an explicit **diagnostic-only** oracle budget of 20,000,000
   work / 2,000,000 results per step, retaining the same graph-size caps and
   100-step horizon. Oracle limits do not become gate limits. Incomplete oracle
   runs support no full classification claim. Record all outcomes, not just
   successfully rescued cases.
3. If pooled prefixes differ from the oracle, stop and fix correctness. If
   pooled trajectories still hit limits, record the limitation; do not increase
   global limits or iterate policy changes in this trial. In either case, run
   at most one full pooled G-R1 characterization if implementation correctness
   is verified, so the entire kernel rather than only the 14 selected cases is
   measured. Zero unmeasured cases is still required for acceptance.

Full G-R1 keeps exactly the E2 parent catalog (168 states), 10,000 mutations,
100-step trajectories, seed `0xE152310001`, uniform operations/target roles,
200,000 global work / 20,000 results, 2,000-node/4,000-edge hard caps, freeze and
explosion definitions, 50 neutral soundness checks, all numerical thresholds
and independent active-structural/activation strata. No genotype repairs or
resampling. Record fresh policy-bound receipts; do not overwrite split receipts.

If full G-R1 passes, verify pooled G-R0/2/3/4/5/6 and receipt-policy isolation.
This task does **not** launch the optimizer, change the proposal grammar, or
claim self-maintenance. The 20×16×10,000 optimizer budget stays reserved. Prepared
proposal generation and genuine activation in production remain a separate
design prerequisite before an E2 learning campaign.

## Implementation and results

Implemented and verified. **Full G-R1 still does not pass: 8 cases remain
unmeasured at the unchanged global limits.** No optimizer, stateability probes,
four-arm research, epoch sweep, or further budget-policy trial was launched.

### Architecture and use

`hg-rules::MatchBudgetPolicy` defines Split (default) and Pooled. The engine's
existing `set_match_budget(work, results)` retains Split. Opt into pooling with
`Engine::set_match_budget_policy(work, results, MatchBudgetPolicy::Pooled)`.
`Engine::match_budget_usage()` exposes fresh per-step totals and rule-level
allowances, work, result counts and truncation flags. These are separate from
cumulative guard-engagement counters, so a previous halted attempt does not
consume a future pooled step's quota or leave its truncation flag sticky.

The pooled collector clones each current compiled rule to supply remaining
limits to the **unchanged** `find_bindings_counted` matcher. It debits the
returned `GuardStats.enumeration_work` and binding count, preserving all matcher
orientation, guard, injectivity and traversal behavior. Completed pools are
canonically sorted as before. Exhaustion stops collection and prevents all
application, sampling and policy invocation for that step. Epoch replacement
and static table replacement reapply the policy; no RNG is drawn by compilation.

Pooled matching is serial even when the engine's parallel feature is enabled.
This is an intentional deterministic budget-ownership choice, not a claim of
parallel speedup. Creation rules retain their existing schedule semantics;
disabled and scheduled rules consume no ordinary enumeration quota.

Production opt-in is `ere.match_budget_policy = "Pooled"` in
`configs/ere_e2_pooled.toml`. E2's schema stays `AlternativeFamiliesV3`; allocation
policy is **not** a new genotype version. Omitted/Split configuration fields
retain the old serialized form and config hash. Pooled ERE currently requires
the explicit E2 encoding. Research and training validate both protocol and
policy before executing any evaluation. Split and pooled receipts cannot be
mixed or substituted, and the diagnostic replay is never a gate receipt.

### Paired 14-case replay

Artifact: `output/ere_e2p_probe_20260926/pooled_replay.json`.

All original genotype bodies and split failure steps were reproduced exactly.
All 14 diagnostic oracle trajectories completed 100 steps. Every completed
pooled and split prefix matched the oracle's per-step full graph hash, accepted
and attempted match counts, and engine RNG probe. Every pooled step respected
the global 200,000-work/20,000-result caps. Every truncated step accepted zero
rewrites and left the RNG unchanged.

Pooling completed **6/14** formerly unmeasured trajectories: samples 2248, 3080,
5713, 6130, 7709 and 8649 (zero-based original assay indices). This removed an
allocation artifact for one core edit and five proposal edits. The other **8**
progressed farther but reached a genuine total-budget ceiling: five result-count
ceilings and three work ceilings, at attempted steps 57–100.

| Sample index | Edited role | Split completed steps | Pooled completed steps | Pooled result |
|---|---|---:|---:|---|
| 1136 | Core | 71 | 97 | Global result cap |
| 2248 | Proposal | 96 | 100 | Completed |
| 2518 | Core | 71 | 99 | Global result cap |
| 3080 | Proposal | 96 | 100 | Completed |
| 3183 | Proposal | 33 | 85 | Global work cap |
| 4554 | Core | 49 | 70 | Global result cap |
| 5713 | Core | 78 | 100 | Completed |
| 6130 | Proposal | 96 | 100 | Completed |
| 6859 | Core | 71 | 97 | Global result cap |
| 7127 | Core | 29 | 56 | Global work cap |
| 7709 | Proposal | 96 | 100 | Completed |
| 8380 | Core | 49 | 70 | Global result cap |
| 8649 | Proposal | 96 | 100 | Completed |
| 8679 | Core | 33 | 85 | Global work cap |

This is not just conservative exact-boundary handling. The oracle measured
100-step trajectories needing up to **357,406 work units** and **40,239 bindings**
in a step. Each remaining case exceeds at least one global limit. Those larger
oracle allowances are explicitly diagnostic and do not authorize a gate pass.

### Full 10,000-sample characterization

Receipt: `output/ere_e2p_20260926/exp_r01.json`. Same E2 genotype draws, parent
states, mutation seed and classification definitions; only allocation changed.
Rates include quarantined samples in their denominators.

| Outcome | Original split E2 | Pooled E2 |
|---|---:|---:|
| Valid | 8,391 / 83.91% | 8,391 / 83.91% |
| Neutral | 3,739 | 3,739 |
| NearNeutral | 2,386 / 23.86% | 2,387 / 23.87% |
| Divergent | 1,037 | 1,042 |
| Quarantined | 1,609 | 1,609 |
| Catastrophic | 1,215 / 12.15% | 1,215 / 12.15% |
| Unmeasured | 14 | **8** |
| Neutral soundness | 50/50 | 50/50 |
| Acceptance | Not passed | **Not passed** |

The six newly completed classifications are one NearNeutral and five Divergent.
A full JSON comparison, removing only protocol/policy metadata and applying
those six expected counter changes, matches every other aggregate, operator,
seed-shape and semantic-stratum field exactly. This is a **paired measurement
completion improvement**, not a biological improvement or independent sample
replication. No previously measured outcome was changed.

| Stratum | Samples | Valid | NearNeutral | Catastrophic | Unmeasured |
|---|---:|---:|---:|---:|---:|
| Structural | 8,463 | 80.99% | 22.59% | 12.83% | **8** |
| Active-parent structural | 4,250 | 81.98% | 39.08% | 10.38% | **8** |
| Activation/deactivation | 1,537 | 100% | 30.90%, reported only | 8.39% | 0 |

Remaining cases comprise seven core edits and one proposal edit, all active
AddPatternEdge draws. All 13 operators and both roles still engage, with the
same 751 actual mode changes, 276 trajectory changes, 1,439 ceiling-adjacent
samples and 1,111 valid ceiling-adjacent cases. No unmeasured sample was counted
as neutral, divergent, frozen or exploding.

### Verification and experiment status

**143 targeted default-build tests passed**, including 11 new engine checks,
6 new pooled integration checks and a new training receipt-isolation test.
Counts: engine 46; hg-meta 18; hg-rules 24; hg-output 5; hg-schema 4; ERE
simulation 29; ERE training 3; feature-off pipeline/snapshot/hash regressions 14.
An additional **12 targeted nonparallel-build checks** passed (10 pooled engine
unit tests, matcher adjacency and ERE conflict checks). A broad nonparallel
test invocation encountered the pre-existing unconditional `rayon` reference
in `phase5_engine.rs`; this unrelated test-harness limitation was not modified.

Mechanism coverage includes full ordered binding-pool equality (including
self-edge noninjectivity and equal-priority ties), nonuniform-weight RNG and
trajectory equality for all three step modes, strict aggregate caps, no-RNG/no-
application on truncation, borrowing in both rule orders, zero capacity, exact
boundaries, disabled/scheduled rules, per-step reset, static and epoch table
replacement, and one-versus-four-thread reproducibility. Production output and
rollout histories match with a live E2 editor. Policy/version receipt isolation
is enforced by both research and training entry points.

Full pooled G-R2/G-R3/protection-fixture receipts were **not launched after the
failed G-R1**, as preregistered. Short pooled identity and positive fixtures
passed as integration mechanisms, not substitute full receipts. Historical
split E2 full receipts remain valid evidence for that old policy only.
EXP-R04–R10 remain blocked; no new research or training result is claimed.
`output/ere_e2p_20260926/research_status.json` records the reserved
20×16×10,000 budget and zero optimizer/precheck/held-out evaluation steps.

### Reproduction

Use fresh output directories. The full gate saves a failed receipt before
exiting nonzero; a targeted replay never unlocks research.

```sh
cargo test -p hg-engine
cargo test -p hg-engine --no-default-features --lib --test matcher_adjacency --test ere_conflicts
cargo test -p hg-sim-core --features hg-meta --test ere_pooled --test ere_e2 --test ere_native
cargo test -p hg-meta -p hg-rules -p hg-output -p hg-schema
cargo test -p hg-train --features ere --test ere
cargo test -p hg-sim-core --test phase10_pipeline --test snapshot_wire --test matcher_hash_regression
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- replay-e2p PROBE_DIR output/ere_e2_diagnosis_20260925/classification_audit.json
cargo run --release -p hg-sim-core --features hg-meta --bin ere -- gr1-e2p GATE_DIR
```

The versioned `gr2-e2p`, `gr3-e2p`, and `fixtures-e2p` commands are implemented
for a future admissible run, but this trial has not supplied those full receipts.

### Next decision, not authorized work

Pooling should remain an opt-in resource policy: it removes artificial equal-
share limits and preserves completed execution, but cannot make an exhaustive
binding pool fit when the pool itself exceeds the global ceiling. A subsequent
preregistered trial would need to choose between larger classification resources
and a more efficient/exact matching strategy. Faster traversal alone cannot
solve the five cases with too many returned bindings; changing storage or
selection strategy also requires a precise new definition of its result budget.
No such change, larger budget, new kernel, or optimizer run was performed here.
Proposal variation/activation in the production grammar remains a separate
prerequisite before meaningful E2 training.

Follow-up authorized 2026-09-26: the separately preregistered
[400k/50k resource-envelope trial](ere_e2_pooled_400k50k_trial.md) completed all
eight remaining classifications and passed G-R1 at its new limits. This original
200k/20k receipt remains failed and unchanged; downstream gates/training were
not launched by that classification-only follow-up.
