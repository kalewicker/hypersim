# Selfmod v13 — subsequent fission limits child establishment

Completed 2026-09-27. Final validated panel:
`output/selfmod_v13_turnover_v2_20260927` in the isolated LK workspace.
Protocol: `docs/selfmod_v13_turnover_preregistration.md`.

**Disabling subsequent funded fission restores bounded establishment on this
saved checkpoint panel: faithful offspring pass 7/8 seeds and the R1 prune
passes 8/8, versus 2/8 and 1/8 with native fission enabled.** Both OFF arms
reach the unchanged 6/8 threshold. This is a conditional mechanism result on
the same eight v12 seeds, not independent new-seed validation.

## Comparison and primary result

Reuse the precision-repaired v12 checkpoints for seeds 11, 17, 23, 31, 47,
61, 83 and 101. Replay each complete initial birth phase for faithful and
named R1-pruned children, including all siblings and the normal 50-token
parent-to-child transfer. Require identical saved v12 birth material, money,
allowance and RNG state. Each genotype then branches into:

- **ON:** native full funded fission, with faithful subsequent offspring.
- **OFF:** skip only the subsequent funded-fission call, starting on the next
  object step. Keep native influx, rent/starvation, matching/effects, income,
  creation charging and refill. Do not change the threshold, prices or caps;
  do not refund the initial birth or reset the child's allowance.

Run each branch for exactly 2,000 additional object steps. There are 32/32
complete continuations and **64,000 advancing steps**, with no new prefix.
Half are rerun ON controls and half are new OFF conditions. All 16 ON
trajectories reproduce v12 birth, raw rows, score and final state byte-for-byte.
The repeated controls are not pooled as independent observations.

Bounded establishment retains the executable-child requirement, complete
uncensored run, living selected-child descendants at the end, productivity
in every final 250-step window, income at least rent in the final 1,000 steps,
and a living/productive cycle of all three seed members. All 32 runs have
zero enumeration skips. In OFF, the selected child's lineage has no new
births, so its survival is direct child survival.

| Condition | Lineage alive at end | Full establishment | Whole cycle persistent |
|---|---:|---:|---:|
| Faithful, fission ON | 3/8 | 2/8 | 8/8 |
| Faithful, fission OFF | 8/8 | **7/8** | 8/8 |
| Prune, fission ON | 6/8 | 1/8 | 8/8 |
| Prune, fission OFF | 8/8 | **8/8** | 8/8 |

Faithful OFF gains establishment on seeds 11, 23, 31, 61, 83 and 101, retains
it on 17, and loses it on 47. That seed's child survives and is productive in
all late windows, but earns 40 tokens less than rent in the late interval.
Do not conceal this loss or relax the financial threshold.

Prune OFF gains establishment on all seven seeds that failed with fission ON
and retains it on 47. Every prune OFF lineage earns more than its late rent.

| Seed | Faithful ON established | Faithful OFF late income−rent | Faithful OFF established | Prune OFF late income−rent | Prune OFF established |
|---|---|---:|---|---:|---|
| 11 | No | +148 | Yes | +1,223 | Yes |
| 17 | Yes | +108 | Yes | +1,199 | Yes |
| 23 | No | +46 | Yes | +862 | Yes |
| 31 | No | +503 | Yes | +906 | Yes |
| 47 | Yes | −40 | No | +1,302 | Yes |
| 61 | No | +611 | Yes | +1,229 | Yes |
| 83 | No | +1,200 | Yes | +2,085 | Yes |
| 101 | No | +62 | Yes | +1,208 | Yes |

Every selected child and its direct parent remains alive in both OFF arms.
With fission ON, direct child survival is 2/8 faithful and 4/8 prune; direct
parent survival is 3/8 and 1/8. ON lineage survival includes descendants and
must not be substituted for these direct-root measurements.

## Mechanism evidence and limits

The ON arms each produce 256 subsequent births across the eight seeds.
OFF produces none, despite eligible parents occurring in 15,885 faithful
and 15,878 prune continuation steps across the panel. The intervention
therefore suppresses actual opportunities; it does not happen to select
branches without qualified parents.

The native prefixes and initial birth have already left 5–7 resident copies,
which remain in OFF at the end. ON populations end at 11–13 copies for
faithful and 10–13 for prune. Every copy pays rent while competing for the
same income pool of 12. The improved financing with OFF is consistent with
lower population competition and the removal of subsequent reproductive
commitments. It does not separately identify birth-transfer costs, population
size, creation allowance allocation, scheduling competition or ancestry drift.
Birth transfers are internal token movements, not token destruction.

Creation allowance binds in all eight OFF trajectories for each genotype,
while no resource gate binds in either ON arm. Across the whole continuations,
the OFF faithful cultures have 18,200 creation-blocked rule opportunities,
versus 3,586 for OFF prune cultures. These are whole-culture counts, not just
the selected child. There are no node/edge-ceiling gates or enumeration skips.

The mean of seed-wise productive firings per live lineage-copy step is:

| Condition | Mean productive firings per live copy-step |
|---|---:|
| Faithful ON | 0.333630 |
| Prune ON | 0.331152 |
| Faithful OFF | 0.538125 |
| Prune OFF | 0.766750 |

Pruning's material-cost advantage becomes visible again in the budget-limited
OFF regime. The higher productivity, fewer allowance gates and positive late
income are consistent with the v11 replacement result. This is evidence of
context-dependent performance, not proof that allowance cost alone causes
all differences or that pruning improves reproductive fitness under ON.
The 7/8 versus 8/8 comparison does not establish a statistical ranking from
eight selected seeds.

The original funded birth and inherited allowance are compatible with
establishment when subsequent reproduction is disabled. Continuing native
reproduction is therefore a limiting mechanism at this operating point;
the v12 failures should not be treated as showing intrinsically nonfunctional
children. No probation/adoption filter is warranted by this comparison alone.
Neither OFF arm tests an evolving population: all subsequent reproduction
is deliberately absent.

## What to continue

Keep the R1 prune as a functional, resource-efficient comparison variant.
Its establishment is supported in a fixed resident population, while its
establishment under native reproduction remains unreplicated. Preserve both
facts in the experimental status.

Prioritize a separately preregistered bounded reproduction/turnover policy
that retains the normal funded birth and explicitly tracks its capital and
competition consequences. The useful target is self-financing establishment
with ongoing reproduction, rather than survival sustained by permanently
stopping births. A fresh-seed replication is also needed before generalizing
this conditional result. No new policy or follow-up campaign was implemented
or launched as part of v13; root ERE work is untouched by this assay.

## Validation and retained engineering history

All 30 `hg-selfmod` library tests pass. The new switch tests compare exact
paid-birth snapshots and first object phases, force eligible parents, verify
OFF capital preservation and native allowance refill, forbid births and RNG
consumption, and replay restored OFF states. Existing native fission parity,
class/genealogy, malformed/censored-rate and disk float-bit tests also pass.

The independent auditor reconstructs every raw row's money, rent/starvation,
proportional income and integer remainder, inherited allowance/spending/refill,
genealogy, gate, native ON eligibility, explicit OFF suppression, RNG birth
draw count, final stocks and unchanged establishment verdict. It rejects 16
deliberately corrupted receipts, including an injected OFF birth and a missing
eligible ON birth. Native matcher/effect classification, class hashing and
ChaCha numerical values are tested/replayed, not independently reimplemented.

Seed 11 faithful OFF and prune OFF reproduce in separate processes: birth,
all 2,000 raw rows, score and full final snapshot match byte-for-byte. Their
4,000 verification steps are excluded from the scientific panel. The final
frozen source archive builds with an offline locked Cargo check after fresh
extraction, and has the same source hash map as the corrected preflight.

A test-fixture setup initially raised parent capital without recording the
added initial funding; it failed the conservation assertion and was corrected
before either preflight. No assay financing was changed.

The first preflight and complete invocation omitted the compiled v11 contract
from their source archives. Both outputs are retained with `validation_note.json`.
The archive inventory was corrected, then the same instrument and protocol
were frozen and run in fresh `*_v2_20260927` directories. All 32 preflight
receipt files and all 128 full-panel receipt files match their prior invocation
byte-for-byte, and the full-panel binary is unchanged. This is an archive
repair, not a changed result. The final scientific denominator remains 32
continuations/64,000 steps; the prior identical invocation's 64,000 steps are
excluded and never pooled. The two engineering preflights add 512 steps each,
also excluded. Historical v11/v12 receipts and import provenance are preserved.

Evidence in the final output directory:

- frozen protocol, source archive and executable, instrument/input provenance;
- immutable copied v12 inputs, all 32 branch birth/row/score/final receipts;
- completed status and summary, `audit.json`, `replay_verification.json`;
- `archive_repair_parity.json`, `interpretation.json`, archived post-analysis
  script and exact OFF verification receipts.

From `experiments/lk`, reproduce into a fresh directory:

```sh
python3 scripts/run_selfmod_turnover.py output/NEW_TURNOVER_RUN
python3 scripts/audit_selfmod_turnover.py output/NEW_TURNOVER_RUN
python3 scripts/verify_selfmod_turnover_replay.py output/NEW_TURNOVER_RUN
python3 scripts/summarize_selfmod_turnover.py output/NEW_TURNOVER_RUN
```

The runner freezes the corrected validated v12 input panel. Existing scientific
outputs refuse overwrite. All conclusions are limited to the declared horizon,
initial culture, eight reused checkpoints and this economy.
