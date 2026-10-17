# Witness-backed damage prevention and paid repair: implementation and results

**Safety correction:** these are archived version1 observations, not a safe
production recommendation. After the bounded assay, an exhaustive-binding
engineering test found legal endpoint permutations that invalidate a family.
The fixed-seed receipts remain unchanged and their arithmetic still verifies,
but the compiler needs typed endpoint guards. Corrected version2 has separate
tags, preregistration and receipts; see [version2 findings](ere_witness_repair_v2_findings.md).

Executed 2026-09-26, protocol `E2-witness-repair-v1`.
**The combined sealing/repair control passed under A3 exposure in all four
predeclared seed groups.** This is engineered graph-resident redundancy with
finite native repair, not learned maintenance or full training admission.

[The frozen contract](ere_witness_repair_v1.md) and its
[executed snapshot](../output/ere_witness_repair_20260926/preregistration.md)
remain unchanged. This report is separate from the preregistration.

## Result and ablations

All24 attempts completed100 steps: **2,400 actual /2,400 requested steps**.
No scientific retries, tuning, cap failures or missing completed-step captures.
Maximum observed matcher work15,816/results1,584 stayed below50k/50k.
All conditions contained identical initial family/witness/instruction material;
only exposure arm and initial Seal/Repair fuel varied. Every condition included
one paid WitnessActivate, including the “neither” maintenance controls.

The four common seeds,984927043585–984927043588, produced the same dynamics.
RNG probes changed neither during compilation nor during stepping: this is a
deterministic fixture replicated across seeds, not four stochastic samples.

| Condition, per run | Valid-time T | Final valid | Measured recoveries | Seal / Repair / Activate | Proposal world fires | Joint J |
|---|---|---|---|---|---|---|
| A3 neither | .5000 | 2/4 | 0 | 0 /0 /1 | 0 | 0 |
| A3 Seal only | .5000 | 2/4 | 0 | 32 /0 /1 | 0 | 0 |
| A3 Repair only | .2750 | 1/4 | 3 | 0 /54 /1 | 640 | .018528 |
| **A3 both** | **.8925** | **4/4** | **5** | **32 /32 /1** | **640** | **.240528** |
| A2 neither | 1 | 4/4 | 0 | 0 /0 /1 | 720 | .274412 |
| A2 both | 1 | 4/4 | 0 | 32 /0 /1 | 720 | .274412 |

Each A3 combined run recovered all four families by completed step21 and kept
them valid through step100. Across four runs, all16 original families ended
valid, with20 observed invalid-to-valid transitions (one family recovered twice
per run) and2,560 actual proposal world firings. Each run spent65 fuel units:
32 Seal +32 Repair +1 WitnessActivate, from97 initially available. Remaining
fuel was32, held by the two Repair instructions (15 and17). All instruction
nodes remained valid; there was no replenishment or static editor.

Prevention-only did not rescue families already damaged at step1. Repair-only
was also insufficient: family0 recovered and stayed valid; families2/3 recovered
temporarily then became invalid; family1 never recovered. It finished with
10 repair fuel units unused, so its failure is not merely exhaustion. Its three
final invalid families failed `Members`. Local exact-endpoint repair does not
cover all possible structural damage. Saved aggregate observations do not
identify a particular missing/rewired edge or causally attribute each failure;
do not infer that specific mechanism without graph/event evidence.

These ablations establish the predeclared narrow combined-control outcome, not
that either component independently guarantees maintenance or that the combined
scheme is globally optimal.

## Recovery and live compilation timeline

All four A3 combined runs shared this measured timeline:

| Completed step | Whole-family state | Installed world table / paid activity |
|---|---|---|
| 1 | Families0/1 invalid;2/3 valid | WitnessActivate pays1 and activates family0 locally; whole validity can still fail after ordinary world exposure |
| 11 | Two valid families | Compilation excludes invalid whole families; only two core candidates are installed |
| 14–16 | Further damage; family1 briefly recovers at15; all four invalid at16 | Local native maintenance continues even while a whole family is quarantined |
| 17 | Families0/1 valid again | Paid Repair traces coincide with their measured recoveries; family0 retains the activated proposal |
| **21** | **All four valid** | Compile at store step20 installs the recovered family0 proposal; first world proposal fires; final paid repairs recover2/3 |
| 22–30 | All four remain valid | Installed table still reflects the compile at20 (two cores +one proposal) |
| 31–100 | All four remain valid | Next compilation installs all four cores +one proposal; no further paid maintenance firing after21 |

Whole-family validity is measured after each advancing step; the installed table
is captured as actually used for that step. Thus four valid families at21 does
not falsely imply that all four were already in the table used at21.
Family valid-step counts were84,85,93,95:357/400, giving T=.8925.
Initial invalid steps were1,1,14,16; recovery counts1,2,1,1.

## Actual representation and native mechanism

The opt-in profile uses family root version5 and instruction version6, not a
silent change to old version3 families/version4 instructions. The serialized
outer encoding selection remains `AlternativeFamiliesV3`, with the explicit
new vocabulary protocol selecting the witnessed representation in installation.
Old vocabulary configurations reject the new operations and vice versa; older
training routes refuse any vocabulary profile rather than accepting old gates.

For each original binary schema relation, seed one typed witness
`[root,a,b]` identifying its role. A live payload is either its original
typed binary edge `[a,b]` or a typed sealed ternary edge `[a,b,root]`.
The catalogue's64 binary relations add64 witness edges. Every rollout's meta
region contains69 nodes and136 edges: unchanged family nodes plus five
instructions and explicit redundancy. This material overhead is counted.

The bounded reader requires exactly one present correctly typed payload for
every witness. It projects a present sealed ternary payload to the logical
binary relation in an isolated temporary decoding graph, then applies the
whole E2 reader. This is interpretation of a new representation, not filling
a missing edge, copying a cached family or mutating the live graph. Missing,
label-damaged, duplicate or ambiguous payloads remain invalid.

The finite native macros are:

- `WitnessActivate`: a graft witness and a dormant version5 root guard permit
  mode activation; pays1; priority110.
- `Seal`: consumes an intact binary payload and its witness, recreates the
  witness plus a typed sealed ternary payload; pays1; priority100.
- `Repair`: consumes a label0 binary payload with the witness's exact original
  endpoints, recreates the witness plus a correctly typed sealed ternary
  payload; pays1; priority100.

Seal and Repair each expand into13 fixed schema-role productions: core, graft,
membership, eight slot roles, label and guard relations. Together with
WitnessActivate, there are27 native productions. Expansion is fixed typed
syntax, not inferred executable code. Cached native guards enforce explicit
opcode/version, an isolated instruction node, Float fuel in1..32 and the
version5 family root. All use unchanged native matcher/conflict/apply semantics.
No new nodes or fuel are created. Multiple instruction copies provide separate
finite pools; normal shared-node conflicts limit simultaneous edits.

The binary world productions cannot match ternary witnesses or sealed payloads.
**This is engineered arity protection against this declared binary grammar,
not arbitrary damage immunity.** Ternary world rules can threaten this scaffold.
Deletion, endpoint changes, node/property loss and witness damage are outside
the repair primitive's guarantee. There is no runtime host repair, matcher
shield, cached world-table fallback or infinite energy source.

## Extension map

| Source | Responsibility / safe extension seam |
|---|---|
| `crates/hg-meta/src/repair.rs` | Witness seeding, bounded strict decoding, fixed role catalogue and native production expansion. New roles/damage classes require explicit representation/version controls. |
| `crates/hg-meta/src/vocabulary.rs` | New opcodes3/4/5 and version6; old opcodes0/1/2 retain version4 and their identities. `compile_operations` permits explicit macro expansion; old single-rule `compile_operation` rejects new macros. |
| `crates/hg-meta/src/program.rs` | Explicit version5 reader dispatch, preserving old dispatch. |
| `crates/hg-meta/src/source.rs` | Correct per-production operation attribution and table identity; profile-specific opcode admission; repair productions survive whole-family quarantine through independently valid instruction material. |
| `crates/hg-sim-core/src/ere.rs` | Opt-in protocol, witness writer selection, all-seed validation before store writes, expanded instruction-bank bound8 and total-fuel bound128. Existing maintenance/paid trace instruments remain in use. |
| `crates/hg-train/src/ere/repair.rs` | Fixed six-condition configure/evaluate/ledger/contrast driver;100 steps; fresh-directory-only; manifests and source/binary snapshots before evaluation. |
| `crates/hg-train/src/bin/ere-train.rs` | `repair-assay BASE_CONFIG FRESH_OUTPUT` diagnostic entry point, no optimizer. |
| `scripts/verify_ere_repair.jq` | Independent profile, fuel, per-step validity/recovery, score, contrast and budget reconstruction from raw receipts; not independent graph replay. |

World-rule semantics, engine matching/conflicts/apply, ordinary production
defaults, legacy instruction semantics and training-admission thresholds were
not changed. Witness seeding and native actions are explicit opt-in semantics,
so prior gate receipts do not establish this profile's G-R/stateability status.

## Every attempt

Every row completed100/100 requested steps without error; every receipt has
the entire effective config, seeded catalogue, complete windows and step probes.

| Receipt | Engine seed | Condition | T | Final valid | Recoveries | Paid total | Proposal fires | J |
|---|---|---|---|---|---|---|---|---|
| [000](../output/ere_witness_repair_20260926/rollouts/evaluation_000.json) | 984927043585 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 | 0.000000 |
| [001](../output/ere_witness_repair_20260926/rollouts/evaluation_001.json) | 984927043585 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 | 0.000000 |
| [002](../output/ere_witness_repair_20260926/rollouts/evaluation_002.json) | 984927043585 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 | 0.018528 |
| [003](../output/ere_witness_repair_20260926/rollouts/evaluation_003.json) | 984927043585 | A3_both | 0.8925 | 4/4 | 5 | 65 | 640 | 0.240528 |
| [004](../output/ere_witness_repair_20260926/rollouts/evaluation_004.json) | 984927043585 | A2_neither | 1 | 4/4 | 0 | 1 | 720 | 0.274412 |
| [005](../output/ere_witness_repair_20260926/rollouts/evaluation_005.json) | 984927043585 | A2_both | 1 | 4/4 | 0 | 33 | 720 | 0.274412 |
| [006](../output/ere_witness_repair_20260926/rollouts/evaluation_006.json) | 984927043586 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 | 0.000000 |
| [007](../output/ere_witness_repair_20260926/rollouts/evaluation_007.json) | 984927043586 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 | 0.000000 |
| [008](../output/ere_witness_repair_20260926/rollouts/evaluation_008.json) | 984927043586 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 | 0.018528 |
| [009](../output/ere_witness_repair_20260926/rollouts/evaluation_009.json) | 984927043586 | A3_both | 0.8925 | 4/4 | 5 | 65 | 640 | 0.240528 |
| [010](../output/ere_witness_repair_20260926/rollouts/evaluation_010.json) | 984927043586 | A2_neither | 1 | 4/4 | 0 | 1 | 720 | 0.274412 |
| [011](../output/ere_witness_repair_20260926/rollouts/evaluation_011.json) | 984927043586 | A2_both | 1 | 4/4 | 0 | 33 | 720 | 0.274412 |
| [012](../output/ere_witness_repair_20260926/rollouts/evaluation_012.json) | 984927043587 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 | 0.000000 |
| [013](../output/ere_witness_repair_20260926/rollouts/evaluation_013.json) | 984927043587 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 | 0.000000 |
| [014](../output/ere_witness_repair_20260926/rollouts/evaluation_014.json) | 984927043587 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 | 0.018528 |
| [015](../output/ere_witness_repair_20260926/rollouts/evaluation_015.json) | 984927043587 | A3_both | 0.8925 | 4/4 | 5 | 65 | 640 | 0.240528 |
| [016](../output/ere_witness_repair_20260926/rollouts/evaluation_016.json) | 984927043587 | A2_neither | 1 | 4/4 | 0 | 1 | 720 | 0.274412 |
| [017](../output/ere_witness_repair_20260926/rollouts/evaluation_017.json) | 984927043587 | A2_both | 1 | 4/4 | 0 | 33 | 720 | 0.274412 |
| [018](../output/ere_witness_repair_20260926/rollouts/evaluation_018.json) | 984927043588 | A3_neither | 0.5 | 2/4 | 0 | 1 | 0 | 0.000000 |
| [019](../output/ere_witness_repair_20260926/rollouts/evaluation_019.json) | 984927043588 | A3_seal | 0.5 | 2/4 | 0 | 33 | 0 | 0.000000 |
| [020](../output/ere_witness_repair_20260926/rollouts/evaluation_020.json) | 984927043588 | A3_repair | 0.275 | 1/4 | 3 | 55 | 640 | 0.018528 |
| [021](../output/ere_witness_repair_20260926/rollouts/evaluation_021.json) | 984927043588 | A3_both | 0.8925 | 4/4 | 5 | 65 | 640 | 0.240528 |
| [022](../output/ere_witness_repair_20260926/rollouts/evaluation_022.json) | 984927043588 | A2_neither | 1 | 4/4 | 0 | 1 | 720 | 0.274412 |
| [023](../output/ere_witness_repair_20260926/rollouts/evaluation_023.json) | 984927043588 | A2_both | 1 | 4/4 | 0 | 33 | 720 | 0.274412 |

[Derived summary](../output/ere_witness_repair_20260926/derived_summary.json)
retains every original family's first invalid step, valid time, recovery count,
final validator, exact scores, fuel and operation totals.
[Contrast decisions](../output/ere_witness_repair_20260926/contrasts.json) passed
all seven checks per seed; [raw status](../output/ere_witness_repair_20260926/status.json)
records complete budget accounting.

R uses the existing priority-normalized structural-class reuse score;
T=valid family-steps/400, F=final valid families/4, J=R*T*F. Repair-only and
combined both have R=.2694990723045588; combined J=.2405279220318187.
This is a fixed hand-built catalogue; no discovery, initial-genotype novelty,
learned adaptation, EI result or evolutionary fitness result is claimed.

## Verification and provenance

**110 targeted tests passed**:21 training/driver,55 ERE simulation,20 meta
library and14 feature-off regressions. Feature-off check and release build
passed. New isolated tests cover each binary relation's paid repair, exhausted/
malformed instruction guards, no-match prevention after sealing, missing payload/
damaged witness rejection and native no-RNG controls. New driver tests cover
short normal/instrumented parity, caps, missing observations, corrupt fuel,
fresh output and cross-protocol rejection. These engineering controls are
separate from the24 research attempts; no full-workspace test claim.

The independent audit passed seven receipt-check groups and rejected three
in-memory corruption cases: a false fuel debit, falsified recovery count and
tampered family validity. Raw receipts were not rewritten. Selected prior
persistence/vocabulary/maintenance receipts and the old executed binary
retained identical SHA256 checksums before and after this work.

Executed binary SHA256:
`717ec0e199166dd75330dd3431d940d73a1196889aee8cb3098861891344f617`.
Frozen contract SHA256:
`9174fe7c1228efb9bbed54c6e4af9d6dc77db7e5228f01c7fa4cbeda13a2be58`.
All seven saved compile-time source snapshots matched the executed source at
the time of the v1 assay. Current source now implements corrected v2;
these are targeted snapshots, not a complete dependency archive.

Supporting evidence: [tests](../output/ere_witness_repair_20260926/verification_tests.md),
[audit](../output/ere_witness_repair_20260926/receipt_verification.json),
[corruption controls](../output/ere_witness_repair_20260926/corruption_controls.json),
[provenance](../output/ere_witness_repair_20260926/provenance.json) and
[checksums](../output/ere_witness_repair_20260926/checksums.sha256).
Hashes and aggregate step traces do not replace complete graph replay or
event-level causal damage attribution.

## Decision and next boundary

The bounded engineered prevention-plus-paid-repair mechanism is now implemented
and verified under A3 for this fixture. It repairs initial damage, converts the
supported binary relations into a protected form, and then needs no further
paid maintenance after21. That is stronger than the preceding protected-only
persistence control, but **not continual general-purpose self-repair**.

Full semantic gates/stateability and full training remain pending: zero optimizer
or held-out steps were run. The next useful check is a new preregistered
falsification campaign against witness/sealed ternary damage and structural
endpoint/deletion damage, with finite-energy and full-graph checkpoints.
This additional campaign was not run or authorized as part of the current
2,400-step control. Do not extend the successful control post hoc or overwrite it.

Subsequent engineering counterexample: `every_seal_binding_preserves_the_decoded_family`
failed with `Err(Members)` on a legal native binding. The engine's unordered
endpoint matching admitted reversed schema roles. This invalidates an
all-bindings preservation claim despite the narrow positive fixed-seed result.
The v2 correction adds typed endpoint labels to every production and splits
left/right membership; it is a new protocol, not a rewritten v1 outcome.
