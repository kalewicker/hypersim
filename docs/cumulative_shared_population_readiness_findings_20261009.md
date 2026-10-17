# Cumulative evolution: shared-population readiness continuation

2026-10-09. This continues the accepted final-experiment proposal and the first
integrated cell-capacity fixture. **The shared-population engineering fixture
passes. The full readiness gate remains incomplete; no evolutionary campaign
or scientific acceptance is claimed.**

The separately sealed lineage/intervention measurement continuation is now
documented in `docs/cumulative_lineage_intervention_readiness_findings_20261009.md`.
It audits these archived inputs without rerunning or changing them.

## What this boundary establishes

`experiments/cumulative/src/population.rs` adds a well-mixed population of local
hypergraph compartments, sharing one finite resource reservoir and a finite
number of physical sites. Metabolic reactions and paid stochastic gene editing
use the preserved native LK matcher/apply path. Wallets, editor nodes, executable
genes and supplied parcels remain graph-resident within each cell. The reservoir,
site allocation, edit cadence and reproduction requests are fixed host world law;
this is not a single spatial hypergraph chemistry or evolved editor regulation.

Every tick charges every pre-existing live cell one plus its current gene count,
including cells that receive no parcel. A cell unable to pay the whole storage
charge dies, dissipates its remaining capital and releases its site. Its graph,
ledger and genealogy remain in the archive. No dead or unsuccessful route is
culled from the denominator.

One grant debits 16 units from the shared reservoir. A paid birth dissipates
`3 + current parent gene count` copying units and transfers eight existing wallet
units to the child. The child receives **zero environmental resources**. The
parent must retain a 24-unit reserve after the payment. Only unoccupied physical
sites admit births; refused births cost nothing and retain their reason. Newborns
first participate in resource and birth competition on the next world tick.

Each boundary verifies both cell accounting and the world conservation identity:

`reservoir + sum(all retained cells' capital + dissipated) = initial world energy`.

Transfers are counted in the parent's local ledger and the child's endowment,
without counting them twice in the world total. No resource arrives at birth or
after the initial reservoir. Initial founder capital is an explicit world input.

Resource and birth candidates are fully enumerated and ranked by SplitMix64
scores keyed by world seed, tick, phase and immutable logical cell identity.
The Python auditor reconstructs all scores, memberships, winners and refusals
independently. Reversed input/rule order produces identical full trajectories;
an allocation-offset probe preserves registered outcome projections. These are
bounded probes, not a proof of arbitrary identity invariance.

The endogenous and nonheritable arms perform one blind paid native edit at world
ticks 0, 3, 6, ... when funded. The external arm performs one blind paid edit in
each newborn; the child pays. The frozen arm makes no edits. Nonheritable births
transmit the original founder program, while paying the current parent's copying
charge. This verifies the declared controls, including their different timing
and cost exposure; it does not establish a scientifically matched arm comparison.

## Fixed cases and retained outcomes

`experiments/cumulative/population_contract.json` fixes all nine development
cases, inputs, horizons, controls and limits before the sealed workflow. One
development seed, `2026100902`, is used. Every case completes its full horizon.

| Registered case | Ticks | Paid births | Deaths | Final living cells |
|---|---:|---:|---:|---:|
| Two founder programs, frozen | 8 | 2 | 1 | 3 |
| Authored founder and multistage capacity program | 8 | 1 | 0 | 3 |
| Endogenous edits | 8 | 1 | 1 | 2 |
| External birth edits | 8 | 2 | 2 | 2 |
| Nonheritable edits | 8 | 1 | 1 | 2 |
| No supply / death control | 4 | 0 | 1 | 0 |
| Unfunded copying control | 2 | 0 | 0 | 1 |
| Occupied-site control | 2 | 0 | 0 | 1 |
| Finite reservoir control | 3 | 2 | 0 | 4 |

Every final reservoir is zero. The finite-reservoir control cannot mint additional
parcels despite requesting two per tick. The occupied-site case records funded
birth refusals; the unfunded-copy case records funding refusals. The no-supply
case retains the dead cell and subsequent empty world ticks. These establish
observable accounting and ecological failure modes.

The authored mixed case is a capacity diagnostic. Short birth/survival counts
from one development world neither rank the four arms nor demonstrate discovered
innovations, cumulative evolution or increasing functional complexity. No case
was replaced, its horizon shortened, or its acceptance threshold relaxed.

## Verification and custody

Completion authority: `output/cumulative_shared_population_readiness_20261009/manifest.json`.
The archive seals **122 entries and 105 source files**. SHA256 of
`output_sha256.json`:

`885748da9a20e93a5ef251352540dd5c3280df5932e04745cc664ff46bcd4c2f`

Thirteen Rust test groups pass: the original seven capacity regressions and six
population groups. Sixteen Python groups pass: eight shared-population auditor
groups and the original eight auditor regressions. Corruption probes reject free
newborn resources, bank changes, altered scores/winner memberships, inheritance
and genealogy changes, omitted deaths, unpaid storage, missing cases/refusals,
checkpoint graph/index corruption, changed continuations and admission flags.

The independent auditor reconstructs native integer law and admissible graph
bindings, reservoir flows, ancestry and complete scheduler opportunities. It
checks checkpoint graph wire data against published state, incidence/label
indices and allocator free-slot membership. Restoring the serialized population
after births and edits gives exact native continuation. Independent ChaCha edit
word replay and allocator allocation-order replay remain explicitly false.

The sealed workflow records **927 native rewrites**: 544 original Rust regression
rewrites, 224 population-test rewrites, and 159 fixture rewrites. It uses
13.125 workflow seconds and 12,295,127 pre-seal payload bytes, within the frozen
6,000-rewrite / 1,200-second / 128-MiB limits and the separate 300-second fixture
limit. These measurements cover this sealed workflow, not disposable development
tests or a projected scientific campaign.

All 105 pinned sources are unchanged during execution. The original capacity
archive verifies before and after against its known root
`8c753671f1bca20029ca8d1f83142d42d7d5eac2027a37366264c5afaf4fa791`.
The original cell implementation is byte-identical apart from the new population
module export; the launcher adds dispatch and the README documents the new
boundary. All other original pinned bodies, borrowed LK sources and both existing
workspace manifests remain unchanged.

Known-root rechecks of both archives and a relocated copy of the new frozen
package pass with zero native work. The relocated receipt is
`output/cumulative_shared_population_readiness_20261009_relocated_recheck.json`.
Use a fresh output directory for any further engineering execution:

```sh
python3 -B experiments/cumulative/scripts/run_population.py output/cumulative_shared_population_NEW
python3 -B experiments/cumulative/scripts/recheck_population.py output/cumulative_shared_population_readiness_20261009 885748da9a20e93a5ef251352540dd5c3280df5932e04745cc664ff46bcd4c2f
```

## Remaining readiness boundary

The population now has finite shared accounting, paid reproduction, retained
death/refusals and audited competition. The remaining gate must still establish
a bounded program/editor repertoire appropriate to the proposed self-modification
claim, prospective lineage candidate sampling and causal intervention assays,
and a measured whole-campaign budget and replicate-count justification. The
current repertoire only duplicates reactions and varies their stage; editor
mode and cadence do not evolve in this population fixture.

The next evidence-producing boundary is the lineage/intervention measurement
fixture: distinguish authored inherited additions from discovery, reconstruct
program innovations through paid births, and recover known necessity under
matched removal/sham/restoration controls. Freeze its inputs and cap before
execution. If the remaining integrated gate cannot become testable within its
accepted budget, report that failure and close this configuration. Do not turn
these short diagnostics into an adaptive search campaign or another mutation
sweep. Scientific inputs and thresholds must be frozen together after readiness.
