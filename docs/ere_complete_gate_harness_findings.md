# Complete-profile deterministic gate harness: findings

2026-09-26, `E2-complete-deterministic-gates-v1`, complete family10/instruction11.

## Latest status: authorized fresh run passed

The subsequent user-authorized fresh run passed all four bounded native gates,
the standalone frozen archive/semantic audit and16 tamper controls. Its clean
receipt is in `output/ere_complete_gates_fresh_20260926`; see
[fresh-run findings](/Users/kalewicker/Documents/hypersim/docs/ere_complete_gate_fresh_run_findings.md).
Training remains closed, and G-R1/G-R3/G-R5 plus optimizer prechecks remain
pending. The original packaging-failed archive is preserved unchanged.

The sections below retain the initial harness task's outcome and failure history;
statements about needing a fresh run describe that earlier task, not current status.

## Original outcome and important qualification

The harness is implemented. One frozen native invocation passed G-R0, G-R4,
G-R6 and the within-envelope G-R2 comparisons in22.65s. The frozen independent
semantic audit also passed, including16 negative tamper controls. However,
**the standalone archive audit failed**: freezing imported a Python companion
and generated an unlisted bytecode cache inside the source snapshot tree.
That cache is itself present in the before-measurement hash manifest, and every
frozen-input hash still verifies. This is a packaging failure, not a native
semantic failure, but the original receipt is **not an admissible clean gate
receipt**. No data, manifests, frozen sources or checker were repaired to claim
acceptance. No replacement run was launched.

The current harness fixes the packaging invocation with `python3 -B`. A new
regression checks that freezing produces no unlisted bytecode files. The driver
now also requires the standalone frozen auditor to succeed before the assay
test can report success; native success alone is insufficient. These are
post-run harness changes, not changes to measured dynamics or acceptance
criteria. A separately authorized fresh frozen invocation is still needed to
obtain a clean archive receipt. Training remains closed.

## Exact native and semantic results

| Gate | Fresh measured evidence | Qualification |
| --- | --- | --- |
| G-R0 |20 active families;20 variable-renaming/edge-order equivalences;20 priority distinctions;40 two-family installation-order controls; one paid Seal representation control | Full canonical and execution bytes and inventories, not only hashes; shared archive packaging failure prevents acceptance |
| G-R4 |31 rejection cases plus clean two-class control; all nine first-failure validators; ownership/schema/obligation/orphan/topology controls;13 role-specific double erasures | Actual source primed before damage, family retired, zero cached fallback, repeat quarantines; synthetic validator inputs, not paid ecology |
| G-R6 |224 native steps across stable, paid activation, metadata-only Seal and recovered stationary windows | Actual EI enabled; both projections measured when stable and withheld on executable change; engineered control, not adaptation |
| G-R2 | All45 rules in27 shipped TOMLs retained; two supported rules each compared for1,000 steps per leg | Only2/45 coverage;43 explicit exclusions; no claim of general grammar expressivity |

G-R2 has two production engines per supported rule:4,000 advancing engine
steps, plus224 G-R6 steps =4,224 advancing steps in the bounded native assay.
G-R0's paid Seal is a separately declared normal matcher/apply preamble with
no advancing engine step. Validator mutations and compile-only reads do not
count as simulation steps. Development probes are separate engineering runs,
not independent scientific replicates or substitute cells.

### G-R0

Every family member is encoded/decoded under a distinct allocation prefix,
including removed/reused slots and unrelated live background. The decoded
core/proposal/active state must match its input exactly. Renaming changes
execution presentation where appropriate but not canonical identity; priority
distinctions change canonical identity. Two-family installation order cannot
change full compiled tables/classes or consume RNG. Reader/compile calls must
not mutate the store. Paid Seal changes live graph representation and consumes
one fuel while preserving the exact decoded family and executable bytes.

Python independently decodes valid complete inventories and reconstructs
candidate/program canonical and execution byte encodings. Ownership and role
obligations are checked separately from executable identity; required metadata
loss is never declared harmless because some executable bytes survive.

### G-R4

The active eight-variable fixture contains all13 role labels (including every
slot), two distinct priority classes and a property guard. First-failure
controls are Region(local region cap), Sides(reversed becomes endpoints),
Members(missing ownership), Slots(local arity cap), Variables(bad order hint),
Labels(bad literal type), Guards(bad guard opcode), Bounds(local pattern cap),
and Compilability(bad priority type).

Additional controls cover duplicate/reversed ownership, wrong/missing schema
declarations, wrong/missing/duplicate obligations, an owned but projected-orphan
variable, unexpected topology, and exact payload+witness double erasure for
each role. The actual GraphRuleSource first installs the valid family, then
retires it, emits zero world rules/classes and quarantines exactly once at the
declared validator; a repeated read stays quarantined without repeated
retirement. Graph state is unchanged by all reads/compiles. Cap changes and
host mutations are explicitly synthetic test inputs at store step0.

### G-R6

Each fixture starts with a labelled three-node path, identity core, dormant
path-to-star priority1 proposal, family fuel0, epoch1, A3,
MaxParallel/pairings16, pooled50,000 work/results and native EI enabled with
minimum one transition. No static editor, evaluator or legacy causal observer
changes the family. One instruction is installed with the declared finite fuel.

| Control/window | World rewrites | Paid operations | Stationary | Included micro/macro EI | Excluded micro/macro EI |
| --- | ---: | ---: | --- | --- | --- |
| WitnessActivate fuel0,0–64 |64 |0 |Yes |3.807355 /2.321928 |1.584963 /0 |
| WitnessActivate fuel1,0–64 |2 |1 |No |Withheld /withheld |Withheld /withheld |
| Activation recovery,64–96 |0 |0 |Yes |3.807355 /2.321928 |1.584963 /0 |
| Seal fuel1,0–64 |64 |1 |Yes |3.867811 /2.046439 |1.584963 /0 |

Activation pays at step1 and changes the installed executable table at step2.
The mixed window's EI and related CE fields are null in both projections,
with the explicit changed-rule-table reason, not disabled-EI nulls or zeros.
Metadata-only Seal pays once but changes no executable table and does not
suppress measurement. The activation recovery window has **no world
rewrites**: it verifies restored stationarity measurement, not renewed activity.
Zero macro EI on the three-node excluded substrate is a real measured result.

Raw per-step inventories/events/RNG/table revisions/fuel/match use and raw TPM
hash registries/counts are retained. Python independently recomputes micro EI
with live-source-row normalization; macro EI presence/finite values and
withholding are checked, but coarse partitions are not independently rebuilt.
Peak matching work/results are73/1(stable),136/2(activation),472/27(Seal), all
well below the declared limits. A paired local probe/no-probe test confirms
that the new read-only TPM export changes neither trajectories nor windows.

### G-R2

`crystal_grow` and `wolfram_like` each perform1,000 accepted rewrites per leg.
Every actual step matches between direct and graph-decoded execution in full
and projected graph hashes, complete StepStats, RNG and match-budget usage.
Both initial stores contain identical inert complete metadata. Final complete
families remain valid. No cap, shortened horizon or inactive run is accepted.

The separate G-R2 matching profile is pooled2,000,000-work/100,000-results,
MaxParallel/pairings1, seed42, caps2,000 nodes/4,000 edges. It does not change
normal runtime limits. Crystal peaks at1,029 nodes/2,099 edges and
91,993 work/11,994 results; Wolfram at1,026 nodes/1,073 edges and1,059 work/2
results. Unsupported rules are retained with first blocking reasons:
35 distinctness, four schedules, three property-effects and one enabled/weight.
Both the shipped TOML and parsed rule declarations are retained; Python checks
source-driven exclusions and reconstructs supported candidates independently.

## Provenance, implementation and failure history

The original archive contains the contract, harness/helpers, selected native
sources, every shipped TOML, frozen auditor/companion and exact executed binary.
Its before-measurement hash manifest has105 files, including the accidental
Python cache. The standalone auditor detects the source-inventory mismatch
before semantic checking. Separately calling its unchanged frozen semantic
functions verifies the native data and16 in-memory tamper rejections. That
separate report explicitly records `standalone_archive_audit_pass=false` and
`overall_admitted_receipt=false`; it does not bypass the failure for admission.

Production additions are only `TpmAccumulator::diagnostic_counts()` and
`EreRuntime::diagnostic_tpms()`: read-only count export, no new serialized fields,
graph mutation or RNG use. The test harness is an opt-in integration test; there
is no production admission/CLI/optimizer modification. Frozen source hashes:

- Contract: `7402409998f851e28e29cd87ec661bef8ab4039763076523a1ac04dbcc28c28c`.
- Original harness: `83226f24cafb0fa789c9cc301fd063c6e4b50fc0ffeab1ae5d088ecf1bad18d8`.
- Unchanged frozen checker: `92678b21663ae29d2710ac972670362dd2ee1ea8e225145973eb9a380172859a`.
- Executed binary: `bf915900a27ee34200982f95b1775fa24300c850804133cf9b7067efe8350232`.

Before freeze, construction fixes addressed repeated-array JSON syntax,
RewriteEvent field names, borrowing runtime config, Rule-vs-JSON TOML
serialization, and comparing serialized TOML hash-map presentation in a
behavioral parity test. The first persisted development preflight exposed
untagged guard values in candidate JSON; explicit Int/Float/Bool receipt tags
fixed independent decoding. Both preflight directories are retained and never
promoted to gate receipts. An initial relative-output invocation failed before
creating receipts because Cargo runs the test from its crate directory; use
absolute output paths. The packaging fix and audit-required completion were
made after the original measurement; the original binary/receipts are preserved.

Selected core/engine/rules/meta/causal/eval regressions, complete-profile training
rejection, continuation/labelled local tests, workspace/feature-disabled checks,
new harness formatting/Python syntax and diff checks passed. All11 existing
witness/repair/persistence/complete/label/checkpoint checksum manifests verified
unchanged. Old frozen matrices and isolated LK tests were not rerun.

## Use and next boundary

Run local implementation/packaging tests without new scientific receipts:

```sh
cargo test --release -p hg-sim-core --features hg-meta --test ere_complete_gates
```

For a separately authorized fresh assay, choose a never-used absolute directory
under the repository's output folder:

```sh
ERE_COMPLETE_GATES_OUTPUT=/absolute/path/to/fresh-output cargo test --release -p hg-sim-core --features hg-meta --test ere_complete_gates complete_gates_frozen_assay -- --ignored --nocapture
```

The corrected driver freezes before measurements, saves all raw gate files
before assertions and runs its archived checker. Existing directories are
refused; short preflights cannot pass full gates. The original standalone audit
must still fail on its preserved packaging mismatch:

```sh
python3 -B output/ere_complete_gates_20260926/source_snapshots/scripts/verify_ere_complete_gates.py output/ere_complete_gates_20260926 --tamper-tests
```

After a clean archive receipt, the next work is preregistered full complete-
profile G-R3/G-R5 horizons and a separately specified G-R1 mutation envelope.
Optimizer distribution/semantic-effect/signal-noise checks and explicit new-
profile training admission also remain pending. No optimizer/precheck/held-out
campaign ran and the reserved20×16×10,000-step research budget is untouched.

Artifacts:

- [Native-only summary](/Users/kalewicker/Documents/hypersim/output/ere_complete_gates_20260926/summary.json)
- [Semantic audit and non-admissible archive status](/Users/kalewicker/Documents/hypersim/output/ere_complete_gates_20260926/semantic_audit.json)
- [Before-measurement hashes](/Users/kalewicker/Documents/hypersim/output/ere_complete_gates_20260926/frozen_inputs_sha256.json)
- [Verification and failure history](/Users/kalewicker/Documents/hypersim/output/ere_complete_gates_20260926/verification_tests.md)
- [Current harness](/Users/kalewicker/Documents/hypersim/crates/hg-sim-core/tests/ere_complete_gates.rs)
- [Readiness map](/Users/kalewicker/Documents/hypersim/docs/ere_complete_profile_gate_status.md)
