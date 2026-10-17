# Complete-witness repeated-damage experiment findings

## Result

The fixed `E2-complete-repeated-damage-v1` diagnostic passed its bounded
mechanism criteria: all48 cells completed their64 requested steps; all48 paid
repairs restored the exact original complete family one step after a newly
damaging CORE lesion; no valid changed-body admission occurred. Protection
prevented all targeted meta attacks. Finite fuel exhaustion stopped recovery.
Ownership erasure remained detector-only, including when four repair fuel
remained. No training, optimizer or held-out evaluations were launched.

This is **not** a general ecological-maintenance pass. Every ordinary world
binding aliased variables under the existing non-injective matcher. The assay
demonstrates repeat repair alongside a syntactically non-identity world
production, not a rich three-distinct-node ecology, sustained metabolism or
learned adaptation.

The single frozen invocation completed in101.79 seconds, exit0. All48 attempts
are retained in `output/ere_repeated_damage_20260926/attempt_000` through
`attempt_047`; there were no scientific retries, replacement seeds, aborted
cells or matcher/region ceilings. The four seeds984927043601–604 give identical
outcomes for each condition; the engine RNG never advanced. They are
deterministic reproducibility controls, not four independent stochastic samples.

## All conditions

Every table row covers all four seeds, exactly one receipt per seed. All runs
requested and completed64 steps. The late-ownership column means exposure at
step40, not guaranteed damage under protection. A2 is protected; A3 is exposed.
All valid steps decoded the exact original whole family, including its dormant
proposal, rather than merely retaining an anchor.

| Arm | Initial repair fuel | Late ownership exposure | Runs | Exact-valid steps per run | Paid repairs per run | Final fuel | Final family valid | Ordinary world fires per run |
| --- | ---: | --- | ---: | ---: | ---: | ---: | --- | ---: |
| A2 | 0 | No | 4 | 64 | 0 | 0 | Yes | 512 |
| A2 | 0 | Yes | 4 | 64 | 0 | 0 | Yes | 512 |
| A2 | 2 | No | 4 | 64 | 0 | 2 | Yes | 512 |
| A2 | 2 | Yes | 4 | 64 | 0 | 2 | Yes | 512 |
| A2 | 8 | No | 4 | 64 | 0 | 8 | Yes | 512 |
| A2 | 8 | Yes | 4 | 64 | 0 | 8 | Yes | 512 |
| A3 | 0 | No | 4 | 7 | 0 | 0 | No | 64 |
| A3 | 0 | Yes | 4 | 7 | 0 | 0 | No | 64 |
| A3 | 2 | No | 4 | 21 | 2 | 0 | No | 176 |
| A3 | 2 | Yes | 4 | 21 | 2 | 0 | No | 176 |
| A3 | 8 | No | 4 | 60 | 4 | 4 | Yes | 480 |
| A3 | 8 | Yes | 4 | 35 | 4 | 4 | No | 288 |

Receipt ordering is seed, arm(A2 then A3), fuel(0/2/8), ownership(false/true).
Seed601 corresponds to attempts000–011;602 to012–023;603 to024–035;604 to036–047.
The per-attempt rows, effective configs and errors are in `summary.json`, and
each receipt contains the full completed timeline, raw events and windows.
The table is not a selection of successful cells: the expected unrecovered
no-fuel, exhausted-fuel and ownership-loss outcomes are included.

Total:3,072 completed steps;2,140 exact-valid family observations;48 native paid
Seal preambles(non-advancing) and48 paid RepairSealed events;96 CORE exposures
and12 ownership-erasure events in exposed arms. The CORE exposures include
re-erasure of already damaged payloads:64 were newly damaging lesions,48 were
repaired, and16 had no remaining fuel. All24 protected cells spent no repair
fuel. Final family validity was28/48, a descriptive count rather than an
unconditional survival success rate.

## Timing, authority and compiled-table interpretation

Four CORE exposures occur at completed steps8/16/24/32. Sufficient-fuel cells
pay at9/17/25/33; fuel-two cells pay only at9/17. Zero-fuel cells remain invalid
from8. The primitive never creates fuel or authorizes witness/ownership repair.

Epoch-one compilation happens **before** an advancing step. A lesion at8 is
observed invalid after8, its world class retires before9, paid repair occurs
during9, and the class reinstalls before10. World firing at8 does not mean an
invalid changed program was admitted: the table was compiled from the original
valid graph before that step. The audit independently checks this lag at every
step, together with the exact installed candidate bytes and final decode.

In sufficient-fuel ownership cells the family has already recovered all four
CORE lesions by33. At40, a separate native rule erases its root-to-core E_OWNS
label. The owned-node count/record obligations no longer validate, the world
class retires before41, and the family stays invalid through64 with fuel4.
No repair payment or reconstruction occurs after ownership loss. The other
ownership cells were already invalid through insufficient fuel; their late
lesion is not credited as causing a second initial failure. Lost authority
is correctly detected, not solved by the existing maintenance vocabulary.

## What was implemented

`runner::run_ere_environment_diagnostic` is a new explicit diagnostic-only API.
It uses the same `run_inner`, `EreRuntime`, matcher, pooled budgets, conflict
resolver and apply path as production runs. Its initialization closure runs
once at step zero; it must not advance the store or engine RNG. The closure
seeds markers/clock, applies one real native Seal binding, and registers a
read-only observer. There is no host-side mutation callback during stepping.

`EreRuntime::install_diagnostic_environment` accepts a bounded exogenous suffix
only for complete-profile A2/A3 diagnostics. It checks arm visibility/label
ceiling, variable/arity/pattern limits, a maximum of eight rules and no creation
schedules. The suffix is a native clock plus four ordered label-erasure rules,
and optionally one ordered ownership-erasure rule. Hazard priorities beat the
clock; sharing the clock prevents double ticks, and sharing the family
prevents hazard/repair from firing simultaneously. A2 has the same catalogue
but the engine's existing meta-node ceiling prevents its hazards from matching.

The suffix is externally seeded experimental machinery, **not** an evolved
world class or an encoded instruction. Its events are counted as seeded-editor
events. No macro, ownership decoder, ordinary world grammar, matcher, fuel law,
normal run configuration or optimizer admission was changed. Complete-profile
vocabulary preflight still rejects `meta_rules_toml` static editors; this
separately named diagnostic API cannot be invoked by ordinary training configs.

The harness is `crates/hg-sim-core/tests/ere_repeated_damage.rs`; its frozen
research test is ignored by default, so routine tests do not launch campaigns.
An empty diagnostic setup matches normal instrumented-loop hashes, windows and
step observations. Other profiles, step-advancing initialization and A2
visibility bypasses are rejected. A budget-one test retains a partial ceiling
receipt, with zero completed steps, paid runtime edits or RNG advance.

## World-ecology limitation

The core is a three-variable, label-zero path-to-star production; the proposal
is a distinct dormant star-to-path production. The crystal starts with16
ordinary nodes. However the ordinary matcher is intentionally non-injective,
and **all17,280 world-rule firings bind fewer than three distinct nodes**.
For example, variables0 and2 may bind the same node. Such a production can
create self-loops rather than enact the intended three-distinct-node rewiring.
The final selected inventories contain eight ordinary self-loops in40 cells
and zero in the eight no-fuel A3 cells, versus zero initially. This confirms
world graph effects but does not establish a complex or productive ecology.

Counts above reconstruct actual ordinary world events using raw rule IDs/names
and the installed world table. They exclude the3,072 native clock events and
all exogenous hazards/paid operations. The ordinary label-777 clock is included
in generic meta-excluded graph hashes/activity metrics: those aggregates should
not be mistaken for clock-free ecological engagement or optimizer fitness.
No proposal activation or proposal-only firing was attempted or claimed.

## Audit, provenance and verification

The independent Python auditor checks all48 receipts and3,072 step timelines:
compile/step RNG, ceilings, installed world tables, native conflict disjointness,
hazard timing/bindings, unit fuel payments, maintenance counters, exact-program
observations, initial Seal graph delta, and unchanged nodes/properties apart
from the declared markers/clock/fuel effects. It independently validates
ownership/role-obligation material and native logical graph deltas in576
post-preamble selected inventories, plus48 unsealed baselines. All48 initial
HGSSv2 full-store roundtrips pass Rust restoration checks and independent frame
header/length checks. It does not decode bincode or independently implement the
entire CandidateRule decoder; step exactness also relies on the archived Rust
reader and original-program comparison.

Ten in-memory tampering controls reject missing steps, false recovery, renewed
fuel, cached world admission, hidden ceilings, omitted exposures, changed
programs, visibility bypass, unpaid traces and missing ownership. Raw receipts
are never edited to obtain an audit pass.

The contract, harness/two helpers,16 selected production/lockfile sources, initial
auditor/helper and exact executed binary were archived **before** measurement.
The binary and archived harness/sources remain the experiment's implementation
identity, not whichever executable Cargo later rebuilds. All current selected
production sources and harness match those snapshots. These selected files are
not a complete standalone build or continuation/stateability package.

The independent auditor was corrected after seeing raw evidence: standalone
PropertyValue clock JSON is scalar while NodeData inventory values are tagged;
native world bindings may alias within one binding, but accepted binding node
sets must be disjoint; planned/actual rule TOML key order from AHashMap must be
compared after parsing, not as presentation strings. Parsed rules are separately
checked against the fixed path/star/labels/priority design, so this does not
permit semantic config drift. The initially archived checker is preserved in
`source_snapshots`; the final checker is separately archived at the output root.
No experiment rule, contract, receipt, seed or outcome was changed or re-run for
these auditor corrections. The initial checker failures are recorded in the
verification notes.

Selected Rust verification:194 existing regression tests plus four new local
test functions and the frozen assay passed(199 distinct tests). The local
12-cell preflight control test was run three times while developing the
harness; these development trajectories are not substituted for or pooled into
the48 frozen cells. The first build failed on using a nonexistent
`SimConfig::default`; it was corrected before measurement. Workspace feature
check, feature-off sim-core check, new-test formatting, Python compilation and
`git diff --check` passed. One later test-selection command named a nonexistent
hg-train `ere_pooled` target and ran no tests; it was not an experiment attempt.
All seven older witness/sealed/complete evidence checksum manifests verified
unchanged. Detailed commands are retained in `verification_tests.md`.

Contract SHA-256:
`ef7996387796a60aceeed0ba8b4c51e2ce449308d00729a2cdc67f7899ba13ec`.
Archived harness SHA-256:
`6858ba03b7aa5657a028151e32d46a2aefc613df31ab42e1c3482a8fc9029699`.
Executed frozen binary SHA-256:
`75879466f76f8e7580a394530e492944d8c05128c40960829dbc61fe3f1a65a4`.

Evidence:

- [Frozen contract](/Users/kalewicker/Documents/hypersim/output/ere_repeated_damage_20260926/preregistration.md)
- [All-attempt summary](/Users/kalewicker/Documents/hypersim/output/ere_repeated_damage_20260926/summary.json)
- [Independent audit](/Users/kalewicker/Documents/hypersim/output/ere_repeated_damage_20260926/receipt_verification.json)
- [Verification notes](/Users/kalewicker/Documents/hypersim/output/ere_repeated_damage_20260926/verification_tests.md)
- [Checksums](/Users/kalewicker/Documents/hypersim/output/ere_repeated_damage_20260926/checksums.sha256)

## Reproduce and next step

Use a fresh output directory; the harness refuses overwrites:

```sh
cargo test -p hg-sim-core --features hg-meta --test ere_repeated_damage
ERE_REPEATED_OUTPUT=/private/tmp/ere-repeated-new-run cargo test -p hg-sim-core --features hg-meta --test ere_repeated_damage repeated_damage_frozen_assay -- --ignored --exact --nocapture
python3 scripts/verify_ere_repeated_damage.py /private/tmp/ere-repeated-new-run --test-rejections --write-checksums
```

The next ecology experiment should force distinct world-role bindings, for
example using different node labels already supported by the world grammar
and a separately declared labelled substrate fixture. It should measure actual
world topology changes/role engagement, not only firing counts, while retaining
matched finite-fuel/protection controls. This requires a new frozen design and
fresh output, not editing or reinterpreting this completed experiment. No such
follow-up was launched. New-profile full semantic gates and stateability still
need verification before training admission. Ownership repair would be a
separate semantic extension with its own native authorization/safety contract,
not an automatic fix inferred from this detector-only result.
