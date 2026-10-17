# Selfmod v14 — funded parent replacement at eight copies

Completed 2026-09-27. Validated output:
`output/selfmod_v14_bounded_20260927`, inside the isolated LK workspace.
Protocol: `docs/selfmod_v14_bounded_preregistration.md`.

**The R1-pruned lineage passes the preregistered combined financing and
continuing-reproduction criterion in 7/8 seeds under an eight-copy cap with
funded parent replacement.** It meets the frozen 6/8 threshold. Faithful
children pass in 1/8. A vacancy-only cap preserves pruned establishment but
fails to sustain selected-lineage reproduction: combined success is 0/8.
This supports advancing the replacement policy to independent replication;
it is conditional evidence from reused v12/v13 checkpoints, not new-seed
validation or open-ended evolution.

## Policy and matched comparison

Use the same eight validated post-birth checkpoints: seeds 11, 17, 23, 31,
47, 61, 83, 101, with faithful or the named v11 R1 RHS-edge prune. Initial
births and siblings are already fully funded and identical to v13. Keep the
resident copies, child allowance, world, RunConfig and post-birth RNG state.
Compare four subsequent reproduction policies:

- **ON:** original native full funded fission.
- **OFF:** no subsequent funded fission, as in v13.
- **vacancy8:** native funded births in sorted root order while fewer than
  eight copies exist. Vacancies reopen only through ordinary starvation.
- **replace8:** the same births below eight. At capacity, an eligible parent
  makes its native faithful child and transfers the normal 50 tokens, then
  that parent retires. Its remaining capital returns to the ledger free pool
  and is recorded as a withdrawal from its lineage. No refund or child top-up.

The cap is prospectively fixed at eight, two thirds of the income-pool/rent
capacity (12/1), leaving rent headroom. It does not delete initial residents
or change ordinary financial/resource parameters. At capacity the atomic
birth briefly creates a ninth copy before its parent retires; every completed
bounded phase has at most eight. Newborns cannot reproduce again in that
phase. Native mutation draw blocks, budget inheritance and refill remain.
All future children are faithful; there is no viability filter or new allowance.

Parent replacement preserves that parent's member/class/ancestry through its
child. It does not protect a selected family or an unrelated member. Ordinary
starvation can still remove a member or a whole selected lineage. Recycled
capital is not earned income; ordinary next-step influx handles pool overflow.

The native/default funded-fission path remains the original policy. The
experimental bounded call shares its transfer, duplication, metadata and
mutation body. It is used by the new `selfmod-bounded` assay, not installed as
a new default for other simulations or the root ERE workspace.

## Frozen criteria and results

Legacy establishment is unchanged from v12/v13: executable child, complete
uncensored 2,000-step run, selected descendants alive at the end and productive
in every final 250-step window, lineage income at least rent during the final
1,000 steps, and all three seed members alive/productive in those windows.

The new **combined success** additionally requires:

1. Late lineage income covers rent plus all capital withdrawn on retirement.
2. The culture has an actual funded birth in every final 250-step window.
3. The selected-child lineage has an actual funded birth in the final 1,000
   steps.

Residual starvation returns also enter the stock-flow reconciliation; with
integer capital and rent 1, these residual returns are zero in this panel.
The boundary identity is independently checked:

`final lineage capital − start-of-late-interval capital = income − rent − retirement returns − starvation returns`.

| Policy | Faithful legacy establishment | Prune legacy establishment | Faithful combined success | Prune combined success |
|---|---:|---:|---:|---:|
| Native ON | 2/8 | 1/8 | 2/8 | 0/8 |
| OFF | 7/8 | 8/8 | 0/8 | 0/8 |
| Vacancy cap 8 | 5/8 | 8/8 | 0/8 | 0/8 |
| Funded parent replacement, cap 8 | 6/8 | 8/8 | 1/8 | **7/8** |

All 64 trajectories complete 2,000 steps. There are **128,000 advancing
steps**, zero new prefix steps and no enumeration skips. Exactly half are
rerun ON/OFF controls, all byte-identical to their v13 birth, raw rows, score
and final state; the other 64,000 steps are the new bounded conditions.
Repeated controls are not independent observations or pooled replications.

The pruned replacement culture persists in 8/8, with selected descendants
alive in 8/8 and actual births in every late window in 8/8. Every selected
pruned family reproduces late. The faithful replacement culture persists in
7/8 and selected descendants survive in 6/8. It has births in every late
window and selected-lineage late reproduction in only 5/8 each.

The successful combined pruned seeds are 11, 17, 23, 31, 47, 61 and 101.
Faithful replacement passes only 61. Neither surviving capital nor gross
income above rent can replace the combined criterion.

## Actual reproduction and financial accounting

| Bounded condition | Subsequent births across eight seeds | Parent retirements | Late births in selected families | Maximum phase-boundary copies |
|---|---:|---:|---:|---:|
| Faithful vacancy8 | 31 | 0 | 0 | 8 |
| Prune vacancy8 | 17 | 0 | 0 | 8 |
| Faithful replace8 | 687 | 644 | 31 | 8 |
| Prune replace8 | 1,107 | 1,080 | 162 | 8 |

Replacement is therefore ongoing funded turnover, not a relabeled OFF
condition or just the initial expansion. It admits every eligible old parent;
no extra RNG draw chooses a victim. Vacancy8 blocks 106,570 of 106,601
faithful and 117,609 of 117,626 pruned eligible opportunities. Ordinary
starvation permits some late vacancy births elsewhere in the cultures, but
no selected family reproduces late in either vacancy arm. The stable-cap
establishment result does not imply continuing selected-lineage reproduction.

The pruned replacement culture has 499 births over the final 1,000-step
interval across the eight seeds, including 162 in the selected families.
The per-seed selected-family late birth count ranges from 9 to 25.

| Seed | Prune late income−rent | Prune late retirement returns | Adjusted late surplus | Late selected-family births | Combined success |
|---|---:|---:|---:|---:|---|
| 11 | +1,123 | 1,105 | +18 | 21 | Yes |
| 17 | +1,158 | 1,145 | +13 | 23 | Yes |
| 23 | +1,351 | 1,261 | +90 | 25 | Yes |
| 31 | +1,287 | 1,272 | +15 | 25 | Yes |
| 47 | +972 | 929 | +43 | 19 | Yes |
| 61 | +1,231 | 1,231 | 0 | 24 | Yes |
| 83 | +440 | 456 | **−16** | 9 | **No** |
| 101 | +854 | 729 | +125 | 16 | Yes |

Seed 83 meets every other requirement, including continuing reproduction,
but its lineage capital falls from 98 to 82 over the late interval after
paying retirement withdrawals. Its minimum late boundary capital is 39.
Retain this failure under the frozen nonnegative criterion. Different phases
of a bounded funding/retirement cycle can affect endpoint capital, as declared
before evaluation; this run does not distinguish that phase effect from
longer-term erosion. Do not call the finite-window deficit extinction or
silently relax the threshold. Seed 61's exactly zero adjusted surplus passes
because the contract uses greater than or equal to zero.

For faithful replacement, five seeds have negative adjusted late surplus.
Four still pass legacy establishment but fail full financing: 11 (−13),
31 (−26), 47 (−34) and 83 (−3). Seed 17 has no late lineage birth and a culture
birth-window gap despite legacy establishment. Selected families disappear
on 23/101; the whole cycle also fails on 23. These failures have different
mechanisms and should not be collapsed into a mutation-validity score.

## Resource constraints and interpretation

Creation allowance binds in 6/8 faithful and 7/8 pruned replacement runs;
no node/edge-ceiling gates or enumeration skips occur. Mean seed-wise
productive firings per live lineage-copy step are 0.523220 faithful replacement
and 0.890004 pruned replacement. The vacancy means are 0.509254 and 0.822813.
These are descriptive finite-horizon rates, not statistical fitness estimates.

At this operating point, the named pruned genotype can support a productive
cycle, repeatedly fund descendants and cover observed rent/retirement returns
under the replacement policy. Its advantage remains sensitive to population
and resource allocation: unrestricted reproduction still fails the combined
criterion, and a vacancy-only bound stops its selected families reproducing.
The successful intervention combines a population bound, parent retirement,
capital recycling, changed root identities and competition. It does not
identify the cap or material-cost reduction as the sole cause.

The same initial checkpoint seeds were used to motivate the experiment.
The 7/8 result clears the declared continuation threshold within that panel,
but cannot establish general performance across fresh worlds. Parent retirement
is a policy-induced loss, not starvation; a direct parent's disappearance
must not be used as evidence of offspring failure. All future offspring here
remain the same genotype: this is a reproduction platform for self-modification
experiments, not repeated mutation, selection among novel children, or
open-ended evolution.

## What to continue

Advance **cap8 with normal funded parent replacement and the named R1 prune**
to a separately preregistered fresh-seed replication, retaining the full
withdrawal accounting and continuing-birth requirements. Preserve native,
OFF and vacancy controls. If it replicates, use this as the controlled
reproductive setting for named competing variants or bounded mutation tests.
Do not adopt the policy as an engine default or start a broad mutation campaign
from this conditional result alone. No follow-up scientific campaign was
launched as part of v14.

## Validation and evidence

All 33 `hg-selfmod` library tests pass. New tests cover native parity below
capacity, vacancy admission/blocking, paid parent replacement at capacity,
exact inherited budget and RNG word counts, no repeated newborn birth within
a phase, cap-boundary enforcement, immutable genealogy under slot reuse and
restored replacement continuation. Existing malformed/censored-rate, native
fission and disk float precision checks also pass.

The independent auditor reconstructs every raw row's money, rent/starvation,
proportional income and integer rounding, creation spending/refill, immutable
genealogy/classes, eligibility/admission order, cap, exact retired donor and
returned capital, birth-only RNG consumption, both verdicts and late lineage
stock-flow. It rejects 22 deliberately corrupted receipts/verdicts. Native
matching/effect classification, mutator semantics, class hashing and ChaCha
numerical values are tested/replayed rather than independently reimplemented.

All four seed 11 bounded continuations reproduce in separate processes:
faithful/prune crossed with vacancy8/replace8. Birth, all 2,000 rows, legacy
score, reproduction/financial verdict and full final snapshot match exactly.
Their **8,000 verification steps** are excluded from scientific denominators.
The engineering preflight uses saved seeds 7/42 and 16 64-step continuations:
1,024 additional excluded steps. The scientific frozen source archive builds
with an offline locked Cargo check after fresh extraction. No scientific
repair, seed replacement, criterion change or output overwrite was needed.

Key output artifacts:

- `manifest.json`, `preregistration.md`, `source.tar.gz`, frozen executable,
  `instrument_provenance.json`, `input_provenance.json` and copied v13 inputs;
- all 64 branch `birth.json`, streamed `rows.jsonl`, legacy `score.json`,
  `reproduction.json` and `final.json`;
- completed status/summary, `audit.json`, `replay_verification.json`;
- post-result `interpretation.json` and archived summarizer in `post_analysis/`.

Implementation is in `crates/hg-selfmod/src/birth_bounded.rs`, the experimental
bounded economy entry point and `src/bin/selfmod-bounded.rs`. Historical
artifacts and original import `provenance.json` remain intact. Earlier runners
now include the newly compiled protocol dependency in future archives.

From `experiments/lk`, reproduce into a fresh directory:

```sh
python3 scripts/run_selfmod_bounded.py output/NEW_BOUNDED_RUN
python3 scripts/audit_selfmod_bounded.py output/NEW_BOUNDED_RUN
python3 scripts/verify_selfmod_bounded_replay.py output/NEW_BOUNDED_RUN
python3 scripts/summarize_selfmod_bounded.py output/NEW_BOUNDED_RUN
```
