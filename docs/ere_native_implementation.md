# Native ERE implementation and gate status

This document describes the co-resident E1 implementation, not the older
`hg-selfmod` separate-store prototype. The original prototype and its recorded
31/45 extended-AST expressivity result remain intact and must not be relabeled
as results for native E1.

## Current outcome

Follow-up: the versioned E1.1 shared-pattern redesign is documented in
`docs/ere_e11_redesign.md`. Its initial full gate also failed (Catastrophic
29.93%); it is opt-in and does not alter native E1's configuration or receipts.
Native E1's results and the original blocked budget record below remain valid.

The architecture, validation infrastructure, epoch integration, four-arm
orchestration, measurement instrumentation and gated training entry point are
implemented. The scientific track is **stopped at G-R1**, not completed:
native E1's Catastrophic fraction exceeds the pre-existing 25% limit. No
EXP-R04–R10 research campaign or optimizer generation has been launched.
The implementation tests below are mechanism checks, not a waiver of G-R1.

| Gate | Verified result | Scope |
|---|---|---|
| G-R0 | PASS, 40 pair checks plus encoding tests | Exact capped native E1 RCF |
| G-R1 / EXP-R01 | **FAIL**, 10,000 mutations, 28 seeds, 100 steps | Valid 81.84%, NearNeutral 13.44%, Catastrophic 30.27% |
| G-R2 / EXP-R02 | PASS, 2/45 expressible (4.44%) | Both expressible shipped rules identical for 1,000 steps; 2/41 excluding four scheduled creation rules (4.88%) |
| G-R3 / EXP-R03 | PASS, 100,000 steps | Stable active identity fixture, every full-graph hash and canonical per-step CSV equal |
| G-R4 | PASS, counters `[1; 9]` | Each native validator engaged independently |
| G-R5 | PASS, 10,000 protected steps | Anchors survive, zero cleanup loss; seeded editor engages 32 times |
| G-R6 | PASS positive-change fixture | Rule change invalidates both EI/CE variants |

Artifacts: `output/ere_native_v2_20260925/exp_r01.json`, `exp_r02.json`,
`exp_r03.json`, `fixture_gates.json`. `output/ere_native_v1_20260925` is a
development-only characterization: it lacked the final freeze/explosion and
ceiling-adjacent checks. Its apparent pass is **not an admissible gate receipt**.
The final protocol corrects these instrumentation omissions, preserves the
original numerical acceptance thresholds and RNG seed, and records a failure;
no seed-shopping or threshold relaxation was used to obtain a pass.

### Experiment ledger for this implementation

| Experiment | State | Evidence or blocker |
|---|---|---|
| EXP-R01 | Executed, failed | Full native-v2 mutation report above; development-v1 receipt is inadmissible |
| EXP-R02 | Executed, passed within the reported envelope | All 45 shipped rules classified; two supported rules round-trip identically |
| EXP-R03 | Executed, passed | 100,000 active steps, exact identity receipt |
| EXP-R04 | Not launched | Four-arm orchestration implemented; blocked by G-R1 |
| EXP-R05 | Not launched | Protection engagement tested separately; no unprotected survival research result |
| EXP-R06 | Not launched | RCF novelty/nontrivial reuse instruments implemented; no research novelty claim |
| EXP-R07 | Not launched | Dual novelty/TPM instruments available; no growth-curve verdict |
| EXP-R08 | Not launched | Six epoch settings implemented; no epoch viability or CI verdict |
| EXP-R09 | Budget committed, blocked before prechecks | 20×16×10,000 optimizer budget; zero optimizer, precheck or held-out evaluation steps |
| EXP-R10 | Not launched | No EI/CE research finding; only stationarity/CI mechanism fixtures passed |

The blocked campaign record is `output/ere_training_20260925/manifest.json`,
`status.json` and `report.md`. It does not contain a trained champion. Prechecks
and held-out controls are listed separately in the manifest; they have not run.
The original project-wide experiment inventory remains in
`docs/project_architecture_and_experiment_status.md`; these native findings do
not overwrite or reinterpret historical prototype/cognition/LK runs.

This is not a claim that every phase is scientifically complete. R2's exit gate
failed, so downstream research exits remain unmet. Dual biological/geometry
instruments and fully endogenously encoded editor rules remain explicit
limitations, not measured zeros or inferred successes.

## Architecture and extension points

The explicit E2 semantic extension is now implemented in
[`ere_e2_semantic_redesign.md`](ere_e2_semantic_redesign.md). It adds core/proposal
families and candidate-level attribution without changing engine physics. Its
full assay meets numerical rate thresholds but remains blocked by 14 unmeasured
matcher-ceiling cases; it does not supersede this native E1 failed receipt or
authorize research/training. Use `read_program` for versioned family decoding;
the older `read` remains the single strict-component reader.

```text
hg-core snapshot
    -> hg-meta bounded region reader -> validated CandidateRule -> exact RCF
    -> hg-rules RuleSource / RuleSetDelta
    -> hg-engine compile_at_boundary -> unchanged matcher/conflict/apply
    -> hg-sim-core ERE telemetry -> dual substrate / stationary TPM records
    -> hg-train G_theta -> same run_inner rollout (no separate physics)
```

`hg-meta` depends only on `hg-core`, `hg-rules` and utility crates. It has no
engine, observer, evaluator or causal dependency. Enable it with
`hg-sim-core`'s `hg-meta` feature; enable training with `hg-train`'s `ere`
feature. Existing simulation paths install no source, protection or match
budget unless ERE is explicitly configured. `SimConfig.ere = None` is omitted
from serialization, preserving the ordinary config hash.

### Encoding and totality

`crates/hg-meta/src/schema.rs` defines the plan's `META_BASE = 0xFFF00000`
label vocabulary. `region.rs` seeds one root, two sides, pattern-edge nodes,
shared variable nodes, optional label literals and property guards. An LHS-only
variable is deleted; a both-sides variable survives; an RHS-only variable is
created. No integer endpoint references or variable-name table provide
cross-side identity. Slots are binary edges with labels `E_SLOT_BASE + k`;
remaining slots compact in sorted-label order after an edit.

`K_ORDER = 6` is an explicit auxiliary presentation-order extension. It is not
a variable reference or semantic membership field: variable identity remains
topological. It preserves the legacy matcher's orientation/tie presentation in
TOML round trips, is excluded from RCF, and is included indirectly in the full
executable ordering key. When writer hints are absent, exhaustive canonical
presentation eliminates allocation history. Duplicate/out-of-range hints are
quarantined; gaps after deletion are allowed.

The reader walks only meta-labeled relations, with radius, node, edge and
incidence-work limits. A relation has at most three physical endpoints;
unknown or malformed structures quarantine rather than panic. All supported
guard operators retain `PropertyValue`'s variant-mismatch-fails semantics.

### RCF and rule-id determinism

Native RCF enumerates variable permutations for at most eight variables and
chooses minimal content bytes. This intentionally replaces v1's incomplete
WL proposal with an exact, factorially bounded form. There is **no greedy
fallback above eight**: such regions quarantine. Unsupported features must be
added to validation, canonical bytes, conversion and fixtures together.

Canonical bytes include priority, node constraints, both side edge lists,
ordered slots and sorted guards with tagged property values. They exclude
allocation handles and seed presentation metadata. The novelty registry stores
complete bytes, so a 64-bit hash collision cannot inflate novelty. Public reuse
maps use fixed `FxHasher` RCF hashes; a collision remains a stated limitation of
those aggregate class identifiers. Unlike the prototype's `class_hash`, the
native decoder's rule-id ordering compares **full RCF bytes and full executable
bytes**, never an incomplete legacy hash or installation counter.

`GraphRuleSource` compares `(root handle, executable bytes)` for installation
and retirement. Equivalent replacement handles still register lifecycle
changes; only an executable-table change increments the engine revision.
The gen-0 anchor vector is captured once. It is never reconstructed from edges
or reset after damage or replacement.

### Epoch integration and protection

`hg-rules::RuleSource` receives a snapshot and step, never an RNG.
`EpochPolicy::Never` compiles the graph once; `Fixed(NonZeroU64)` compiles at
step zero and divisible step boundaries. Duplicate compile calls at one step
are suppressed. `StaticRuleSource` leaves the installed TOML table unchanged.

`Engine::compile_at_boundary` runs before candidate collection. Changes replace
the whole table atomically, clear old per-rule guard counters, and reapply ERE
match budgets. A deterministic RNG state probe and a consuming sequential
positive control verify D2. Both parallel and sequential matching use the same
ordinary source-independent matcher. Two meta-band conflict fixtures verify
that disjoint bindings' effects cannot remove another binding's dependencies.

ERE ordinary rules have `meta_visible = false` and an implicit node-label
ceiling checked for every binding, including isolated variables. A3 removes
this protection globally. Legacy static rules have no implicit ceiling unless
ERE is active. Match work and result budgets are opt-in; a truncated pool is
**not applied**, because partial enumeration would bias selection. Binding
ceilings halt and record the run, not silently clip a successful result.

### Specification resolutions and limits

1. **A0's initial graph includes inert encoded regions.** All four arms must
   share an initial graph; otherwise A0/A1 full-graph hashes cannot be equal by
   construction. A0's authoritative table comes from TOML, not from those
   regions. A1 compiles them once. Source table presentation is canonicalized
   for both controls.
2. **Seeded meta editors are explicit static scaffolding.** E1 cannot emit
   edge labels or property effects; therefore it cannot endogenously encode
   the editor needed to rewrite labeled encoding relations or pay energy.
   Editors use the ordinary engine, are appended after encoded rules, and are
   excluded from endogenous RCF novelty and reuse claims. The supplied editor
   restores `E_BECOMES`, pays one unit of seed energy and increments priority.
   This is a point-source control, not evidence that a useful rule evolved.
   Full endogenous editor representation requires a separately characterized
   extension beyond v1's E1 envelope.
3. **A2 editor admission protects anchors and topology.** Editors cannot delete
   unconstrained/anchor/side/var nodes, relabel survivors, or create a meta edge
   incident to unprotected data. Otherwise an ordinary data deletion could
   destroy a meta edge by cleanup even though its binding contains no meta
   node. Matching depth/arity/count limits also apply to admitted editors.
4. **ERE metrics are an additive structured attachment, not new legacy CSV
   columns.** `MetricsRow.ere` is skipped in existing serde/bincode output.
   Native `EreMetricsRow` fields retain `Option<T>` and serialize to
   `ere_metrics.jsonl`, with `null` meaning unmeasured, plus `ere_report.md`.
   This preserves old CSV and checkpoint bytes rather than silently changing
   headers or attempting to flatten arrays/maps into CSV cells.

E1 explicitly excludes schedules, edge-label constraints, selectable
distinctness, survivor relabels, property writes, derived/property-to-property
comparisons and non-unit weight/disabled-rule controls. The converter refuses
them; it never silently drops them. The small native envelope is not the older
prototype's 68.9% E1d/E2 envelope.

### Instrumentation

`EreRuntime` is installed into the existing `run_inner` loop. It captures the
pre-step store, consumes the ordinary engine's rewrite events, and records
RCF classes, window installation/retirement/quarantine counts, all nine
validator counters, novel class arrivals, cumulative reuse, non-meta firing
reuse, anchor survival and meta cleanup loss. Half-life is right-censored when
the anchor population has not fallen to half; `None` is not zero. NonTrivial
requires a real firing on non-meta structure, not just encoding reproduction.
Trivial copying is marked only when an actual new encoded root reproduces the
firing rule's RCF; arbitrary meta activity is not called replication.

Meta exclusion removes nodes and incident/meta-labeled edges **before** WL,
PI and TPM construction. Included and excluded windows have independent
`WindowComputer` state and actual window durations. EI uses per-step surviving
generational-handle transitions. A measurement window pins the executable
table revision; any change suppresses both EI/CE variants. The legacy cumulative
causal observer is refused for ERE runs so it cannot report cross-rule TPMs.
EI is disabled in training rollouts, whose objectives need reuse and PI only.
`EreSubstrateMetrics` retains optional values even for firing/match counts:
the included count is measured, world-only accepted events are counted, and
projected attempted matches are `null` with an explicit reason. Structural and
energetic CE are reported individually; WL class counts are cumulative per
projection. The quarantine alarm divides by all validated/quarantined region
decisions, not just newly installed rules.

Temporal CI estimates use circular moving-block bootstrap, not IID transition
resampling. They are conditional estimator intervals within a stationary
window, not across-seed hypothesis confidence intervals. Short epochs/windows
or inadequate transitions are unmeasured with a reason. Biological constructor
registries, geometry and attempts separated by substrate subset are not
invented from rewrite totals; extending those dual instruments is separate
work. Native arm summaries should be read with these instrumentation limits.

## Constants and protocol traceability

| Constant | Default/value | Interpretation/source |
|---|---:|---|
| `META_BASE` | `0xFFF00000` | Plan §3.2 label band |
| physical relation arity | at most 3 | E1 relations binary, E_BECOMES ternary |
| `max_arity`, `max_vars` | 8, 8 | Plan arity default; exact-RCF factorial bound |
| `max_pattern_edges` | 16 per side | Native bounded-reader operational cap |
| region nodes/edges/radius | 128 / 256 / 5 | Root→side→pattern→var→literal/guard path; generous bounded region budget |
| incidence work | `max_region_edges × 16` | Bounds duplicate-incidence scans; endpoint cap is separately enforced |
| global meta-region ceiling | 500 nodes | Plan FM-R3 operational ceiling, binding logged |
| matcher work/results | 50,000 each, split across active rules | Operational hard ceiling; partial pool never applied |
| G-R1 | 10,000 samples, ≥20 seeds, 100 steps | Plan §6; 28 seeds here, 11 uniformly sampled graph-edit operators |
| G-R1 thresholds | Valid ≥.40, NearNeutral ≥.10, Catastrophic ≤.25 | Unchanged from attached plan |
| freeze | 50 consecutive zero-fire steps | Plan metrics freeze definition |
| explosion | >4× initial nodes or edges | Existing characterization convention; measured on fixed background |
| mutation background | 64 nodes, 96 binary edges, label 0, Int key-1=100 | Identical fixture for parent and mutant, including constraints that may fail |
| soundness spotchecks | first 50 Neutral samples | Checks RCF equality against actual 100-step behavior |
| characterization work/results ceilings | 200,000 / 20,000 | Resource-bound classifications are unmeasured and prevent gate pass |
| hard characterization graph ceilings | 2,000 nodes / 4,000 edges | Resource guard distinct from the 4× scientific explosion definition |
| G-R2 | 1,000 steps per expressible rule | Plan §6; max one ordinary rewrite per step, active fixture required |
| G-R3 | 100,000 steps | Plan §6; active stable reference, full hashes and per-step CSV |
| G-R5 verification | 10,000 steps | Native protected fixture duration, with actual editor engagement |
| epochs | ∞, 10,000, 1,000, 100, 10, 1 | Plan EXP-R08 |
| EI | ≥1,000 transitions, ≤512 classes | Transition floor and dense-TPM operational ceiling |
| CI | 16-step blocks, 64 replicates, empirical 95% | Moving-block estimator; ≥32 and ≤10,000 steps per window |
| usable EI CI width | .25 bits | Native sweep reporting threshold, recorded in manifest |
| seeded meta editor | 32 units, cost 1/edit, priority 100 | Declared seed energy budget, finite point-source control |
| quarantine alarm | ≥.90 | Plan FM-R7 |
| grammar | 64 logits: four groups ×16 productions | Plan 50–200 parameter range |
| training | 20 generations ×16 candidates ×10,000 steps | User-authorized budget, 3.2 million optimizer evaluation steps |
| prechecks | 64 common grammar seeds; 4 objective seeds; 5×8 optimizer probes at ≤1,000 steps | Expressibility, actual traversal, discrimination, saturation |
| controls | 8 held-out common seeds ×3 arms | Champion/untrained/random-resample; separate from optimizer budget |

## Running and extending

### Verification completed

| Check | Result |
|---|---|
| Native encoding tests (`hg-meta`) | 5 passed, including 40 RCF pair checks and randomized edge deletions |
| Native simulation integration (`ere_native`) | 14 passed, including real pipeline/rollout equality, G-R6, resource ceilings and receipt-threshold recomputation |
| Training grammar/budget gating | 2 passed; not a trained-objective performance result |
| Engine/rules/output/schema tests | 35 / 24 / 5 / 4 passed |
| Feature-off matcher hash / pipeline regressions | 1 / 11 passed |
| Causal / evaluator suites | 29 / 48 passed; five existing evaluator probes ignored |
| All native-feature simulation test targets | Compiled successfully (`--no-run`); not a claim that all research tests executed |
| `git diff --check` | Passed |

These 178 executed tests are separate from the full characterization receipts
above. No unit-test pass is being substituted for the failed research gate.
Existing unrelated edits and the older separate-store implementation were
preserved; no commit or pull request was created.

```sh
cargo test -p hg-meta
cargo test -p hg-sim-core --features hg-meta --test ere_native
cargo test -p hg-train --features ere --test ere

# Use a new output directory; existing experiment records are not overwritten.
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- gr1 OUTPUT_DIR
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- gr2 OUTPUT_DIR
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- gr3 OUTPUT_DIR
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- fixtures OUTPUT_DIR

# These refuse to proceed unless the full prerequisite receipts pass.
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- arms ARM_OUTPUT configs/ere_native.toml GATE_DIR
cargo run -p hg-sim-core --release --features hg-meta --bin ere -- sweep SWEEP_OUTPUT configs/ere_native.toml GATE_DIR
cargo run -p hg-train --release --features ere --bin ere-train -- configs/ere_native.toml TRAIN_OUTPUT GATE_DIR
```

Training precommits the budget/base config before probes. P1 compares canonical
emitted graph signatures, not names or serialization order. P2 runs the actual
OpenAI-ES optimizer on the real F5′ objective, then measures emitted-graph total
variation; L2 movement is diagnostic, following the existing cognition v2
implementation. P3 compares extreme objectives against seed variability; P4
requires margin from the normalized objective's bounds. Failure halts before
main optimizer generations. F5′ uses real non-meta class reuse; F4′ is reported
as secondary, and F1–F3 are absent. Controls share held-out seeds; the primary
report is paired champion minus untrained, not champion minus random alone.

To add a grammar feature: extend CandidateRule, both writer and total reader,
the appropriate validator, RCF bytes and executable ordering, converters,
fixtures and envelope characterization together. Re-run G-R0, D1/D2, G-R1 and
G-R2; do not reuse an old acceptance receipt after changing the encoding or
mutation kernel. To add measurement fields: keep unmeasured distinct from zero,
define both meta variants and revision invalidation, then add an engagement
fixture before reporting a mechanism verdict.

The next scientific decision is an explicitly versioned E1 redesign addressing
the 30.27% catastrophic surface. Changing seeds or thresholds until a passing
number appears, or narrowing the kernel under the wrong Valid-fraction branch,
would not complete the original plan.
