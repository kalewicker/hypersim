# Selfmod v12 — controlled child establishment

Completed 2026-09-26. Final validated panel:
`experiments/lk/output/selfmod_v12_birth_v2_20260926`.
Protocol: `docs/selfmod_v12_birth_preregistration.md`.

**The R1 edge prune can survive competitive birth, but does not meet replicated
self-sustaining establishment.** Descendants remain alive on 6/8 seeds versus
3/8 for faithful offspring. The full preregistered criterion passes on only
1/8 pruned seeds versus 2/8 faithful seeds; neither reaches the 6/8 threshold.
All eight faithful and all eight pruned branches retain productive cycles.

Do not promote the v11 replacement-rate gain to reproductive fitness or adopt
a probation mechanism from this result. Competitive ancestry survival, net
income and population persistence are different measurements here.

## What was tested

Eight new world/scheduler seeds:11,17,23,31,47,61,83,101. Run a shared native
faithful prefix to its first funded R1 birth, then fork immediately before
fission into four branches:

- **Faithful:** ordinary duplicate.
- **Prune:** operator 11 with the exact v11 `op11_member0_draw1` parameter block,
  removing RHS edge [0,2] and retaining [2,1].
- **Unscreened:** apply once the ordinary operator/parameter block already
  consumed at that child's birth, without extra RNG consumption or resampling.
- **Refund:** remove that child and return its exact 50 tokens to its parent,
  retaining native RNG consumption and all future native births.

Every subsequent birth is faithful. All branches retain normal full fission,
threshold 100/transfer 50, rent 1, proportional pool 12, influx 20/free cap 500,
corrected random scheduling, match cost0 and the declared resource ceilings.
Each runs 2,000 object steps after the selected birth. Genealogy is sideband
recording; no new lineage property is inserted into the genome or scheduler.

All eight seeds trigger naturally at absolute steps 15–37. Parent tokens at
the trigger are 100–103: after the paid birth 50–53, versus 100–103 in refund.
The child receives exactly 50 tokens, and inherits creation allowance 138–176
before ordinary refill. No fresh allowance or endowment is injected.

The final panel completes **32/32 continuations, 64,000 continuation steps,
173 shared-prefix steps: 64,173 scientific advancing steps**. There are no
untriggered seeds, runtime stops or enumeration-unmeasured trajectories.
Separate exact cross-process verification adds 4,000 steps, excluded from the
scientific panel. Two engineering preflights each complete eight 64-step
continuations plus 61 prefix steps, 573 steps each, also excluded.

## Primary results

Bounded establishment requires an executable child, complete uncensored run,
living selected-child descendants at the end, lineage productivity in each
final 250-step window, lineage income ≥ rent over the final 1,000 steps, and
living/productive copies of every seed member in every late window.
Refund has no child-establishment verdict.

| Branch | Descendants alive at end | Full establishment | Productive cycle persists |
|---|---:|---:|---:|
| Faithful | 3/8 | **2/8** | 8/8 |
| R1 edge prune | **6/8** | **1/8** | 8/8 |
| One unscreened mutation | 0/8 | 0/8 | 6/8 |
| Refund/cancel selected birth | N/A | N/A | 8/8 |

The pruned lineage persists and fires in all late windows on six seeds, but
five of those six draw down capital over the late interval. Their late
income-minus-rent totals are −1,−55,−63,−154 and−66 tokens. These are retained
failures under the frozen ≥0 criterion, not a threshold adjusted after output.
The −1 case is borderline; even relaxing only that boundary would not reach
the required6/8 replication threshold. This does not establish a statistical
ranking between pruning and faithful offspring from eight seeds.

| Seed | Faithful established | Prune descendants at end | Prune late income−rent | Prune established |
|---|---|---:|---:|---|
| 11 | No | 1 | −1 | No |
| 17 | Yes | 1 | −55 | No |
| 23 | No | 5 | −63 | No |
| 31 | No | 1 | −154 | No |
| 47 | Yes | 3 | +31 | **Yes** |
| 61 | No | 4 | −66 | No |
| 83 | No | 0 | −18 | No |
| 101 | No | 0 | 0 | No |

Faithful establishment passes seeds 17/47. Its other surviving lineage,
seed 61, also fails late financing (−21). Thus a child lineage can disappear
or lose capital even with an unchanged genotype and a persistent culture.

Original parent roots remain alive in 3/8 faithful, 1/8 prune, 5/8 unscreened and
2/8 refund branches. Parent-root loss does not imply loss of its other
descendants or loss of the cycle. On pruned seeds 23/61, the original R1 class
has no living copies at the end, while the changed R1 lineage remains: genetic
replacement can occur without satisfying the late-financing criterion.
Class identity and ancestry are tracked separately; faithful descendants
remain the original class even when other R1 ancestries disappear.

## Why the v11 gain does not carry over directly

Mean fixed-horizon **lineage** productivity is 0.594750 for pruning versus
0.441750 for faithful children. That includes differences in survival and
number of descendants. The mean of seed-wise productive rates **per live
copy-step** is almost identical:0.331152 prune versus0.333630 faithful.
These descriptive means are not statistical fitness estimates.

No creation, world or enumeration gate binds in any faithful, prune or refund
continuation. The inherited allowance is therefore not an observed limiting
gate in this panel. Pruning's v11 advantage came from a three-copy replacement
assay where resource gates often bound. With native copying, productive work
is shared across more copies and each copy refills its own allowance. The
absence of binding creation gates is consistent with that cost advantage
becoming less relevant; this is an inference, not a separate causal experiment.

The refund branch demonstrates exact capital accounting and provides a
no-selected-child comparison. It does not demonstrate a rescue: faithful and
prune cycles already persist on 8/8. Native fission can try again on the next
step after refund. Cancellation also changes copy count, allocation history
and competition, so it cannot identify a pure 50-token price effect by itself.

The eight unscreened draws are descriptive, not a mutation-quality estimate:
two retain the parent's class, five are other changed executable variants,
and one is malformed. Seed 11's shrink is a no-op and seed 23's label edit keeps
the same class; both reproduce their faithful continuation. Seed 83's ordinary
RHS-edge removal produces the named prune and reproduces that continuation.
No such draw was replaced. The invalid child on 101 pays rent and dies at
continuation step 51; its productivity rates are null. All selected unscreened
lineages disappear, and whole-cycle persistence fails on 47/61. Do not call
these failures enumeration artifacts: there are no enumeration skips.

## What is worth continuing

Keep R1 pruning as a **functional competitive control**, not a validated
self-financing adaptive operator. It preserves the cycle and often retains
descendants, but the advantage seen in replacement is not a clear per-copy
advantage at birth. Do not launch the broad mutation sweep or install probation
on the basis of this panel.

The next useful isolation is the **same funded-birth checkpoint with subsequent
fission enabled versus disabled**, for faithful and pruned offspring. Keep the
initial transfer, inherited budgets, residents, prices and matching identical.
This separates ongoing copy turnover/ancestry drift from resident-child income
competition; it does not tune the economy. Freeze its horizon, seeds and
financial criteria before running it. The R2 pattern-edge removal remains
another named v11 lead, but test it competitively before adopting it: its match
access change differs from R1's material-cost change.

No follow-up campaign, probation policy, new mutation sweep or ERE work was
launched as part of v12. Conclusions are limited to one selected R1 birth,
eight seeds, this initial culture, native faithful descendants and 2,000 steps.
This is not repeated mutation, long-run open-ended evolution or proof of the
mechanism behind v10's collapse.

## Validation, repair and evidence

All 28 `hg-selfmod` library tests pass. They include recorded/native funded
fission material/money/RNG parity, paired birth costs and allowances, named
genotype, restored checkpoint continuation, faithful class preservation after
slot reuse, invalid/censored rates and disk float-bit preservation. Production
fission uses the same body with an observational callback for diagnostic birth
events; transfer order, mutation draws and ordinary defaults are unchanged.

The independent audit reconstructs every raw row's rent/starvation, productive
income and integer remainders, token flux/dissipation, creation spending/refill,
native birth eligibility/order, inherited budgets, genealogy, gate estimates,
RNG draw counts, fixed denominators and primary verdicts. It rejects ten
deliberately corrupted receipts. Native matching, effect classification,
mutator semantics, class hashing and ChaCha numerical values are tested and
replayed rather than independently reimplemented in Python.

Two engineering issues were exposed and retained transparently:

1. Deleting the refunded child allows slot reuse to reverse raw RHS-edge list
   order in a later faithful clone. The program's canonical class is unchanged.
   The auditor compares canonical programs while preserving raw order.
2. The first complete invocation passed the accounting audit, but its disk
   checkpoint parser changed some seeded Float properties by a rounding bit.
   The first faithful replay had the same aggregate verdict but differing raw
   snapshots/world hashes. That invocation remains at
   `output/selfmod_v12_birth_20260926`, with `validation_failure.json` and its
   attempted replay. It is not the final validated panel.

The repair enables `serde_json`'s `float_roundtrip` feature and adds a disk
checkpoint regression test. The identical protocol was then frozen and run
in the fresh `selfmod_v12_birth_v2_20260926` directory. **All 154 scientific
receipt files are byte-identical across the two complete invocations**;
`repair_parity.json` records the comparison. No result, seed, horizon or
threshold changed, and duplicates are not pooled into a larger panel.

The repaired binary exactly reproduces seed 11's faithful and prune branches
in separate processes: birth state, all 2,000 raw rows, verdict and final full
snapshot match byte-for-byte. `replay_verification.json` records hashes and
the separate 4,000 verification steps. The final frozen source archive also
passes an offline locked Cargo check after extraction to a fresh checkout.

Key evidence in the final directory:

- `manifest.json`, `preregistration.md`, `instrument_provenance.json`,
  `source.tar.gz`, `selfmod-birth`, `v11_selected_mutation.json`;
- `seed_*/initial.json`, shared `prefix.jsonl`, `fork.json`;
- each branch's `birth.json`, every row in `rows.jsonl`, `final.json`,
  `score.json`; `summary.json` and completed `status.json`;
- `audit.json`, `replay_verification.json`, `repair_parity.json`;
- `interpretation.json` and its archived post-result script in `post_analysis/`.
  A superseded descriptive ancestry-label report is preserved explicitly;
  the current report distinguishes class copies from ancestry copies. No
  primary receipt or verdict changed during that labeling correction.

From `experiments/lk`, reproduction uses fresh output directories:

```sh
python3 scripts/run_selfmod_birth.py output/NEW_BIRTH_RUN
python3 scripts/verify_selfmod_birth_replay.py output/NEW_BIRTH_RUN
python3 scripts/audit_selfmod_birth.py output/NEW_BIRTH_RUN
python3 scripts/summarize_selfmod_birth.py output/NEW_BIRTH_RUN
```

Existing outputs refuse overwrite. The import's original `provenance.json`
is unchanged; new instrument provenance is separate. Work remains entirely
inside the isolated LK workspace, with root ERE files untouched by this assay.
