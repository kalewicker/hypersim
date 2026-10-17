# Bounded in-run rule-birth/lifecycle fixture v1

Frozen design before native fixture tests, 2026-10-03. Profile
`E2-rule-lifecycle-fixture-v1`. This is scripted engineering, not a learner,
scientific H-R1 acceptance, or a new admission gate. The authoritative schedule
is `configs/ere_rule_lifecycle_fixture_v1.json`: three fixed seeds, six arms,
24 requested advancing steps per cell, 18 cells / 432 maximum native attempts.
No retries, replacements, candidate search, horizon changes or threshold tuning.
One native fixture invocation, 60-second native wall ceiling; test compilation
is separate from that ceiling. All terminal receipts are retained before the
panel's positive/negative assertions. Unknown/partial work fails closed.

## Representation and causal intervention

Construct one labelled ordinary triple a/b/c with labels10/11/12 and two
undirected label-zero path edges (a,b),(b,c). Seed a complete version10 family:
core path->star with those labels, proposal star->path with labels99/11/12,
priorities0/1, both active. All initial encoded candidate RCF bytes, including
any dormant allele, are excluded from novel birth. Also compare zero-priority
RCF, so priority-only changes cannot earn structural novelty.

The existing Activate/repair vocabulary cannot create an absent candidate body;
Transpose swaps slot order, while native world edges are unordered. Therefore
this fixture does **not** reuse activation or endpoint reordering as novelty.
Use the existing `GraphRuleSource.editors` seam for an explicitly seeded, typed,
paid literal-edit motif. This is a diagnostic static-editor extension, not a new
graph instruction opcode, arbitrary mutation language, or vocabulary admission.

At initialization only, mark the proposal's unique label99 literal as the edit
target. A meta-band editor token holds a native Int clock and Float fuel. The
paid native rule matches the complete root's ordered E_OWNS relation to that
label-literal, checks its target marker, label/value and family version, changes
the literal99->10, debits exactly one Float fuel, and advances the same clock.
The replacement arm changes10->99 at clock8 and99->10 at clock16, paying each
time. A lower-priority native tick advances the token clock otherwise; it
conflicts with edits on the token, preventing double ticks. World and meta edits
may occur in the same step because they bind disjoint nodes.

All editor/clock bodies are explicitly seeded and excluded from world novelty.
The token is scaffolding, not an evolved or encoded instruction. No host writes,
direct apply_binding calls, metadata recounting, fuel renewal, or world reset
are permitted after the initial store is constructed. Both world productions
use the existing matcher/apply pipeline and exclude meta nodes.

## Arms and fixed expectations

| Arm | Source | Fuel | Edit enablement | Purpose |
| --- | --- | ---: | --- | --- |
| live_paid | Fixed(2) | 1 | birth at clock0 | novel genotype, later installation, ordinary use |
| live_replacement | Fixed(2) | 3 | edits at clocks0/8/16 | class retirement and reappearance with the same root alive |
| frozen_paid | Never | 1 | birth at clock0 | genotype birth without executable admission or new-class use |
| live_unpaid | Fixed(2) | 0 | birth rule present | no paid edit, installation or new-class firing |
| live_disabled | Fixed(2) | 1 | edit rules disabled | distinguish clock/world activity from actual editing |
| preseeded_lure | Fixed(2) | 1 | proposal label10 already seeded | ordinary proposal firing must not count as novel birth |

Live paid: new RCF appears after completed step1; first executable admission is
the pre-step boundary2 (step3). First ordinary firing is step3, then every other
step through23 (11 novel-world events). Its genotype lifetime is right-censored
at24, lower bound23 completed-step units; installed lifetime lower bound22.
One paid edit gives no inter-arrival estimate; sustained-vs-replacement remains
unmeasured, not infinite or a pass.

Replacement: genotype spells [1,9) and [17,24], installed spells [2,10) and
[18,24], second spells right-censored. Exactly one globally novel class, two
presence spells, and seven novel-world events at3/5/7/9/19/21/23. Step9 uses the
pre-step installed class even though the post-step genotype has already changed.
Do not misattribute it using the post-step table. Edit intervals are8/8; the
completed class lifetime8 does not strictly exceed that replacement timescale.
Root survival through24 must not substitute for class survival.

Frozen: genotype birth still occurs and fuel is spent, but original installed
rules stay frozen and the new class never fires. Unpaid/disabled: no new class.
Preseeded: 12 proposal-world fires but zero novel classes, because the full
initial RCF set contains that rule. All arms preserve the root, the three world
node identities/data, and valid complete metadata at every completed boundary.

## Measurement and verification

Record exact RCF/execution/zero-priority RCF bytes, genotype and installed class
spells separately, initial class exclusion, raw pre-step rule-id attribution,
native bindings and node/edge deltas, ordinary topology, clock/fuel values,
match-budget accounting, compile boundaries and RNG probes. A genuine world
event must bind only ordinary nodes and produce the declared path/star switch;
mere edge-ID churn or a meta event cannot earn nontrivial firing credit.

Full initial/final inventories plus per-step state/event records bind graph
changes and payment. Independent Python checks reconstruct the tiny candidate
RCF/execution bytes, timeline, topology and class lifetimes, including censoring
and the absence of a sustained claim. This is receipt reconstruction, not an
independent Engine, general CandidateRule interpreter or RNG replay.

Existing anchor half-life and aggregate reuse counters are not changed or reused
as novel-class lifetimes. New measurement lives only in this diagnostic module;
the old evaluator, matcher, source/compiler, encoding, vocabulary, optimizer,
gates and sealed outputs retain their previous meanings. The sole shared source
change is an additive `ere::lifecycle_fixture` export. Snapshot the new stage and
test binary, run focused tests, save verification controls and seal a fresh
engineering output. Larger horizons, training and a scientific campaign remain
unlaunched and require a separate specification and compute authorization.
