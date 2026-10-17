# Cumulative evolution: first integrated readiness fixture

2026-10-09. The implementation starts the accepted final-experiment proposal at
`docs/final_cumulative_complexity_experiment_proposal_20261009.md`.

**The integrated cell-capacity fixture passes. The full readiness gate remains
incomplete. No evolutionary campaign has run.**

The separately frozen shared-population continuation is documented in
`docs/cumulative_shared_population_readiness_findings_20261009.md`. Its finite
shared economy and zero-environment births do not change this earlier isolated
fixture's law, archive or interpretation.

## Implementation and scope

`experiments/cumulative` is its own Cargo workspace. It imports the preserved
LK graph, rule compiler, matcher, apply path and observer types through local
dependencies. It changes neither LK nor the main workspace, and adds no path
dependencies or members to their manifests. Its separate lockfile pins the
new build; the patched LK allocator supplies exact free-list continuation.

Wallet, editable gene nodes, an editor node, and resource parcels share each
cell's hypergraph. Ownership edges bind every executable operation to that
wallet and its local material. A graph gene encodes a legal adjacent-stage
reaction. Those reactions compile to native rules, consume actual matching
parcels, and relabel them using the existing engine apply implementation.

The editor uses the same native matcher/apply path: duplicate a bound gene,
copy its stage, jitter that stage by -1/0/+1, assign a fresh serial, and debit
two wallet units. Editor mode is read from the graph and copied to offspring.
The current repertoire contains reaction duplication and stage variation;
broader program edits and mutable editor regulation remain to be implemented.
No utility table or target outcome selects autonomous edit bindings.

The fixture's authored accessibility probe instead selects two declared native
bindings and uses a deterministic editor mode. This shows that paid legal edits
can construct the capacity witness; it supplies no autonomous-discovery claim.
The stochastic arm controls are separate and use no selected edit outcome.

All admitted bindings are enumerated. Bound violations reject instead of
silently truncating. Metabolic ties use stable logical gene serials; the fixture
tests reversed decode order and a different node-allocation history. This
establishes invariance for isolated cells, not fairness in a competing population.

## Frozen law and capacity result

The prospective engineering contract is `experiments/cumulative/contract.json`.
It fixes one finite 512-unit test environment per isolated cell, initial founder
capital 40, parcels worth 16, adjacent-stage reactions, settlement values
0/4/8/14/16, and all inputs/controls before the sealed run.

Storage costs one plus gene count per cycle. Each reaction costs one; editing
costs two. Copying costs two plus gene count plus one editor, and reproduction
transfers eight existing capital units to the child. Unused parcel energy and
all costs dissipate. Every boundary closes an exact integer energy balance.

| Fixed capacity input | Net capital per cycle | Net over 12 cycles |
|---|---:|---:|
| Founder `[0]` | 1 | 12 |
| First addition `[0,1]` | 3 | 36 |
| Second addition `[0,1,2]` | 7 | 84 |
| Earlier addition removed `[0,2]` | 0 | 0 |
| Later addition removed `[0,1]` | 3 | 36 |
| Same-program sham | 7 | 84 |
| Restored program | 7 | 84 |
| Redundant extra gene `[0,1,2,0]` | 6 | 72 |

The intermediate reaction is causally necessary for the later pathway's benefit
under this law. Redundant graph growth adds storage cost. These are authored,
finite capacity/ablation witnesses, not evolved complexity or global minimality.

Three actual paid parent-child transmissions preserve the multistage program.
Four arm fixtures check heritable endogenous edits, external birth-time edits,
nontransmitted within-lifetime edits, and the frozen founder. The nonheritable
arm pays the parent's actual copying charge even though its child resets to the
registered founder program. Arm cost/timing differences remain explicit.

Every child receives a separately accounted finite **diagnostic environment**.
This is an isolated inheritance assay; it is not a shared-resource population
or replenishment/reproduction ecology. No aggregate population resource claim
is drawn from these transmissions.

## Verification and immutable evidence

Completion authority: `output/cumulative_readiness_20261009/manifest.json`.
The sealed archive contains 111 entries and 98 pinned first-party source files.
SHA256 of `output_sha256.json`:

`8c753671f1bca20029ca8d1f83142d42d7d5eac2027a37366264c5afaf4fa791`

All 13 case inputs, negative controls, full native/host transition receipts,
three transmissions, checkpoint, parity branches and endpoints are retained.
The independent Python auditor reconstructs typed graph ownership, admissible
bindings, actual changes, expenses, payouts, complete horizons, inheritance,
energy balances and fixed denominators. It checks native continuation equality,
but does not independently replay ChaCha8 words or slotmap allocation. Those
limitations are explicit false fields in the audit.

Seven Rust test groups pass, covering useful pathways, native paid editing,
unfunded/capped refusals without changes, arm inheritance/costs, corrupted
checkpoints, exact RNG/allocator continuation after recycling, and the full
fixture. Eight independent Python groups pass, rejecting altered payments,
outcomes, ownership, edit bindings, opportunities, ancestry, transmission,
denominators, work, resume/order parity, contract and admission fields.

The package records **1,045 native rewrites**: 544 in focused Rust tests and
501 in the sealed fixture. Its run used 10.671 seconds and 10,949,388 pre-seal
payload bytes, within 4,000 rewrites / 1,200 workflow seconds / 128 MiB.
The native fixture also fits its separate 300-second limit. These counts cover
the sealed workflow; initial disposable development tests are not counted as
scientific work or silently folded into this package's measurements.

All pinned sources were unchanged before/after execution. Known-root custody,
frozen audit and corruption tests pass again without native work. A relocated
copy reproduces those checks using only its own frozen scripts, contract and
inputs; its supplemental receipt is
`output/cumulative_readiness_20261009_relocated_recheck.json`.

Replay the verification without native work:

```sh
python3 -B experiments/cumulative/scripts/recheck.py output/cumulative_readiness_20261009 8c753671f1bca20029ca8d1f83142d42d7d5eac2027a37366264c5afaf4fa791
```

To execute the engineering fixture again, use a new output directory and the
wrapper in `experiments/cumulative/README.md`. Historical archives stay immutable.

## Remaining work inside the same readiness gate

Implement shared-resource population competition and reproductive settlement,
broaden the editable repertoire/editor regulation, freeze prospective lineage
sampling and intervention rules, and measure complete campaign costs. Independent
world count and scientific acceptance thresholds require a sensitivity/power
calculation. Those are the next requirements of this single experiment, not
additional scientific claims from this fixture.

The executable currently exposes only the bounded fixture command. All full
readiness, scientific acceptance, evolved-complexity and campaign-launch fields
are false. The readiness implementation must stop at its agreed boundary if
these remaining requirements cannot be made testable; the positive capacity
witness does not authorize an expanding sequence of evolutionary campaigns.
