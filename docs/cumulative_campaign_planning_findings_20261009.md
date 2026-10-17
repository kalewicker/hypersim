# Campaign sizing and prospective acceptance — 2026-10-09

The zero-native planning boundary passes its engineering audit. Under the stated
target sensitivity model, the smallest panel with at least 80% joint power is
**47 independent seed blocks, comprising 188 separate arm worlds and 1,880 fixed
sampling slots**. Exact integer calculations agree with an independently
implemented multinomial convolution across all 256 scenario/count cells. Ten
synthetic decision-rule groups pass; original and relocated frozen rechecks pass.

Whole-campaign feasibility does **not** pass. A scientific budget has never been
approved, the complete native campaign/measurement adapter is absent, and time
and storage for the full population and intervention denominator remain
unverified. This is a planning result, not a scientific preregistration, launch,
evolutionary result or a new negative scientific campaign. All four scientific
admission flags remain false. No native execution or scientific seed pilot ran.

## Statistical design and assumptions

Each seed block contains four separate worlds: heritable endogenous editing,
external blind birth mutation, nonheritable endogenous editing and frozen
founders. Worlds share their block seed and physical law; diverging populations
retain their own keyed opportunities. Blocks are the modeled independent units.
Lineages, census slots and observation windows never increase the replicate count.

The proposed worlds have two [0] founders with capital 40, a finite initial
reservoir of 512, two parcels per tick, eight sites and a 16-tick horizon. These
parameters fit the current kernel's declared domains. No further resource inflow,
newborn environmental credit, authored probes or seeded useful pathways are
proposed. Enabled/mask policies implement the existing arm semantics; the
nonheritable arm resets the complete founder program and policy at birth.

The prospective sampler uses two keyed slots at each of censuses 0,4,8,12,16,
including retained dead cells and empty slots. Every retained novel-acquisition
pair receives the fixed intervention suite. Every planned world remains in the
world denominator, including extinct, capped and unmeasured worlds. A missing
result prevents scientific acceptance; it is not silently converted into a
negative world.

Two decision tests use alpha 0.025 each, summing to 0.05:

1. A one-sided exact binomial test of endogenous qualifying-world prevalence
   above 20%. The 20% floor is an explicit proposed programme decision threshold
   for repeatability, not an estimated mutation baseline or false-positive rate.
2. Conditional on that criterion, a one-sided exact discordant-pair test of
   endogenous versus external qualification, plus an observed advantage of at
   least 20 percentage points. A prevalence pass alone never establishes an
   endogenous-mechanism advantage.

The prevalence calculation uses the binomial distribution directly. The paired
calculation conditions on discordant blocks and uses their exact binomial sign
probability; the one-sided tail is specified explicitly rather than calling a
two-sided library default. See the [NIST binomial reference](https://www.itl.nist.gov/div898/handbook/eda/section3/eda366i.htm)
and [statsmodels exact McNemar documentation](https://www.statsmodels.org/dev/generated/statsmodels.stats.contingency_tables.mcnemar.html).
The implementation uses Python integer arithmetic and no statistical dependency.

The target model assigns block probabilities 0.10 to both arms qualifying, 0.40
to endogenous only, 0.10 to external only and 0.40 to neither. Thus endogenous
prevalence is 0.50, external prevalence 0.20, and the population advantage 0.30.
These are sensitivity assumptions, **not estimates from the 12 heterogeneous
readiness worlds**. The design does not establish that these rates are plausible
for a 16-tick evolutionary horizon. Independence and identical block distributions
are modeling assumptions; deterministic seed generation does not prove them.

Joint power is the probability that both the prevalence and advantage decisions
pass in the same panel. It is calculated from the complete multinomial outcome
space, not obtained by multiplying marginal powers or by calculating only the
probability of seeing one innovation. The first target-model count reaching 80%
is 47; at that count, the prevalence decision requires at least 16 qualifying
endogenous worlds.

| Sensitivity model | Endogenous / external prevalence | True advantage | Joint power at 47 blocks |
|---|---:|---:|---:|
| Strong | 60% / 15% | 45 points | 99.61% |
| Target | 50% / 20% | 30 points | 80.84% |
| Small advantage | 30% / 20% | 10 points | 8.33% |
| Rare | 10% / 5% | 5 points | 0.000712% |

The latter models demonstrate the design's limits. Failure of the primary rule
would not show that rare cumulative evolution is impossible. One verified useful
lineage or a smaller advantage can remain a secondary observation while failing
the proposed programme-level acceptance criterion. Every count from 1 through
64 is retained for each scenario; no scientific outcomes determined the selection.

The proposed seed list is fixed now using the first 64 bits of SHA256 of the
contract's domain and decimal block index. All 47 seeds are distinct and avoid
the registered development seeds. A collision would reject the panel rather
than replace a seed. No proposed seed has been executed by this planning workflow.

## Comparable functional criteria and causal dependence

The existing development report's `discovery_candidate_pass` requires endogenous
origins. Applying that field unchanged to external controls would guarantee
that they fail regardless of function. The prospective helper separates the
common functional rule from the expected arm origin. Two external acquisitions
can satisfy the same cumulative functional criterion; their external origins
remain explicit and are not promoted to endogenous discovery.

The shared rule retains the original requirements: two successive novel
acquisitions after the founder, authenticated paid descent and version identity,
at least two paid transmissions and useful activity in at least two distinct
descendant generations for each acquisition, strictly positive incremental
standardized score after acquisition cost, funded comparisons, group removal,
storage-matched earlier ablation, separate sham and restoration. Seeded founder
material, authored insertions, reacquisition, neutral copies and repeated counts
of the same generation do not supply the two innovations. Empty ablations remain
inadmissible; no founder gene is added to make an empty program legal.

The old score rule establishes that both components contribute. That alone does
not establish that the later innovation builds on the earlier one. Two prospective
roles, `padded_remove_later` and `padded_remove_both`, complete a storage/copy-matched
interaction comparison:

- Later gain with the earlier mechanism present:
  `S(intact) - S(padded_remove_later)`.
- Later gain with the earlier mechanism absent:
  `S(padded_remove_earlier) - S(padded_remove_both)`.

The first gain must exceed the two-unit acquisition cost by at least one score
unit; the second gain, after that cost, must be nonpositive. The difference of
these gains must be positive. Padding uses the existing legal inactive stage -1
and preserves positions, gene count and copying/storage load. All comparisons
must finish and fund their paid copy; an unfunded assay remains an explicit
inadmissible comparison.

Synthetic tests show that two separately helpful components without this
interaction fail, and that matched endogenous and external synthetic examples
can both pass. Those tests are decision-rule checks, not new native positive
controls. The helper consumes independently validated provenance and assay links;
it does not replace the native lineage/request auditor. The current native
request schema does not admit the two extra role names, so their adapter and
native/independent verification remain required before preregistration.

Nonheritable and frozen arms are mechanism/accounting controls. A qualifying
lineage in either invalidates that measurement or implementation before scientific
interpretation. They are not extra statistically independent comparisons with
which to enlarge the sample size. Endogenous editing has a fixed opportunity
every three ticks; external mutation has one opportunity per actual paid birth.
Both pay the existing two-unit edit cost. Their realized opportunity counts and
costs need not match. Any eventual advantage is scoped to these registered arm
semantics, not a perfectly rate-matched test or a universal advantage of self-editing.

## Resource envelope

The prior 6,000-rewrite/300-native-second/128-MiB limits belong to bounded readiness
fixtures. They were never an approved scientific campaign budget. The following
calculation compares against them without silently expanding them or declaring
the scientific configuration impossible because it exceeds a fixture allowance.

A world processes at most 32 parcels. Each parcel performs at most four metabolic
rewrites plus settlement. At most eight living cells can edit at each of six
cadences; dead retained cells do not execute edits. External mutation can execute
at most once for each of 30 possible new retained cells. This gives conservative
actual-work bounds of 208 endogenous, 190 external, 208 nonheritable and 160 frozen
rewrites per block arm. All 47 blocks therefore reserve at most 36,002 actual world
rewrites. A further 8,272 rewrites conservatively cover one unpaired journal
reservation per world. Unknown reservations are never reclaimed or retried.

With founder stage 0 and executable stages 0,1,2,3, a selected path can have at
most three first-ever novel stages, hence at most three retained acquisition
pairs. Ten slots per world, three endpoint roles and 13 roles for each pair give
an uncached upper request denominator of 420 per world, or **78,960** across the
panel. Four assay cycles allow at most 20 native rewrites each.

| Component | Conservative reservation |
|---|---:|
| World execution | 36,002 rewrites |
| Retained uncertainty allowance | 8,272 rewrites |
| Complete uncached assay denominator | 1,579,200 rewrites |
| Total sufficient reservation | **1,623,474 rewrites** |

This is a sufficient conservative reservation, not a lower bound on actual work
or a claim that all 78,960 requests will be unique. Role/program caching can
reduce actual work. Funding cannot depend on the old panel having no useful
pairs, selecting only favorable candidates, or dropping excess requests. The
current single-request assay cap of 128 is not sufficient to admit this complete
upper denominator without a prospectively specified global registry/chunk mapping.

Authenticated saved complete journals retain 3,776,523 world bytes over 12 cases
and 57 ticks, and 1,477,510 assay bytes over 33 requests. Scaling these observations
to the proposed denominator gives approximately 190 MiB for world journals and
3.29 GiB for uncached assays: about **3.48 GiB combined** before extra provenance,
verification and failed-route overhead. Saved command times scale to about 188
world seconds and 3,522 assay seconds, about **62 minutes combined**.

These are observed-scale projections, not certified future time/storage bounds.
The saved worlds do not match the proposed maximum population/program dimensions,
and the retained full assay denominator may contain larger programs. The report
marks these limitations explicitly. A larger fixed budget must be chosen and
verified against a bounded full-dimension engineering benchmark before a scientific
launch could be justified. Increasing the number of unexamined scientific seeds
to obtain that benchmark is not permitted.

## Sealed evidence and verification

The successful package is `output/cumulative_campaign_planning_v2_20261009`,
with seal root
`a82f42baf4d6f12ee3c7f576b6a47ca1d208df5a3f386de74ca185bc172a9dc9`.
It has 368 sealed entries, 143 source pins and 7,012,055 payload bytes. The
publication workflow takes 6.354 seconds and executes zero native rewrites.

`contract.json` freezes the planning assumptions and proposed rule.
`plan.json` retains all 256 exact rational sensitivity cells, the entire proposed
seed/arm panel, resource calculations and unresolved requirements. `audit.json`
records the independent multinomial and structural resource checks. `evidence/`
contains complete saved world/assay journals, input/command receipts and an
original-parent seal with an authenticated copy map. Frozen rechecks need no live
source, Cargo build, native binary or original archive directory.

The first publication attempt exposed a command-receipt filename collision that
overwrote its generated `plan.json` before auditing. It remains a failed,
zero-native package at `output/cumulative_campaign_planning_20261009`, root
`bd30202bf798fc8f3416865a3b6288dea2c819dd955faac064cbda44b2f66b35`.
The successful launcher separates command receipts in `commands/`; statistical
assumptions, seeds, criteria and resource formulas were unchanged. All nine
previous roots and this retained failure verify before and after publication.

Known-root and relocated rechecks repeat the frozen independent audit and ten
synthetic decision-rule groups with zero native work. Receipts are:

- `output/cumulative_campaign_planning_v2_20261009_recheck.json`
- `output/cumulative_campaign_planning_v2_20261009_relocated_recheck.json`

```sh
python3 -B experiments/cumulative/scripts/recheck_campaign.py output/cumulative_campaign_planning_v2_20261009 a82f42baf4d6f12ee3c7f576b6a47ca1d208df5a3f386de74ca185bc172a9dc9
```

## Decision required next

Choose whether to fund and verify a larger fixed scientific envelope or close
this configuration at the existing limits. No campaign budget has been adopted
by this report. If continuing, finish the bounded generalized runner, complete
interaction-role mapping, maximum-dimension evidence/time benchmark and independent
confirmation-environment policy, then freeze the full scientific contract together.
Do not weaken the criterion, use fewer blocks because they fit, or add seeds after
outcomes. If the funded envelope cannot pass, retain the failed/inconclusive route
and close the configuration. This planning result supplies no general ceiling on
Hypersim or on hypergraph evolution.
