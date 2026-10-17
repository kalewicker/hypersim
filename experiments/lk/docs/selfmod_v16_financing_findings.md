# Selfmod v16 — extended financing diagnostic

Completed 2026-09-27 in isolated LK. Evidence:
`output/selfmod_v16_financing_20260927`. Contract:
`docs/selfmod_v16_financing_preregistration.md`, including its engineering
amendment made before diagnostic output.

**The restored extensions support retirement-cycle timing as the explanation
for the three v15 pruned financing failures, rather than sustained capital
erosion over this horizon.** All eight pruned replacement families and whole
cycles persist through continuation step 10,000. Every family has a funded
birth in every new 1,000-step window; every culture has births in every
250-step subwindow. All eight have both positive and negative fixed-window
balances. The original v15 primary result remains **5/8, below its 6/8
replication threshold**. This diagnostic is dependent continuation evidence,
not another independent seed panel or a retroactive change of criterion.

## Extension and the checkpoint limitation

Extend every v15 seed (131/137/149/157/163/173/181/191) under all eight
faithful/prune × ON/OFF/vacancy8/replace8 treatments. Restore its exact
validated final snapshot at step 2,000 and advance 8,000 further steps.
All 64 trajectories complete: **512,000 new diagnostic steps**, no new
prefix, omitted seeds, replenishment intervention or future mutation.
Ordinary influx, rent, paid transfer, retirement withdrawals and creation
allowance retain their original behavior. The full old RunConfig is preserved;
the new loop supplies its own extension duration without changing physics.

A beyond-horizon engineering test discovered that upstream SlotMap 1.1.1
serde omits vacant-slot allocation order. Restoring an old checkpoint can
therefore change later reused IDs and full states even when early biological
rows agree. The original v15 in-memory free-list order was never saved and
cannot be recovered. V16 begins from each exact **restored v15 snapshot**,
with canonical upstream restoration order explicitly saved. It cannot establish
full-state parity with a hypothetical uninterrupted v15 run.

V16 checkpoints additionally save node/edge free-list order for world and
rules. A local vendored SlotMap adds validated, opt-in export/restore methods;
default allocation and serialization remain unchanged. The graph API restores
orders atomically without altering versions, keys or data. From the restored
starting state onward, split and uninterrupted v16 execution match exactly,
including allocator sidecars. This limitation applies uniformly to the panel.

## Capital, births and retirement phase

Each row independently reconciles selected-family capital with earned income,
rent, retired donor capital and residual starvation returns. Eight disjoint
1,000-step windows record end balances, means, minima, copy counts and birth
windows. Family capital is also reconstructed immediately after every
selected-family retirement, including intermediate admissions within a phase.

| Seed | Capital at 2,000 → 10,000 | Minimum extension capital | Capital-mean slope per 1,000 steps | Family births during extension |
|---|---:|---:|---:|---:|
| 131 | 100 → 183 | 86 | +8.448 | 142 |
| 137 | 121 → 177 | 87 | +5.673 | 222 |
| 149 | 53 → 55 | 5 | +1.132 | 42 |
| **157** | **76 → 89** | **46** | **−0.010** | **157** |
| 163 | 135 → 224 | 86 | +6.043 | 180 |
| 173 | 174 → 118 | 86 | −0.157 | 202 |
| **181** | **62 → 154** | **43** | **+10.594** | **178** |
| **191** | **72 → 244** | **45** | **+9.629** | **158** |

Slopes are descriptive ordinary least squares slopes of the eight window
means, with no p-values or claim that serial windows are independent.
Total capital must be read alongside family copy counts. Five seeds gain
copies during the extension, explaining much of their capital growth.

Seed 157, previously failing by −19 tokens, keeps one family copy throughout.
Its window means are 74.027, 73.298, 74.518, 74.118, 74.190, 73.495, 73.657
and 74.073. Every family retirement leaves exactly 50 tokens in the new child.
Its 1,000-step balances alternate:
`−17, +38, −10, −23, −14, +28, −12, +23`.
The near-flat means, constant retirement-phase stock and recurring paid births
support a funding-cycle explanation for negative endpoint differences.

Seed 181, previously −21, increases from one to two family copies, while 191,
previously −26, increases from one to three. Both capital means and
retirement-phase stocks rise with that expansion. Their final capital exceeds
the starting capital by 92 and 172 tokens, respectively. Each continues
funding births in all eight windows; neither shows sustained erosion here.

Seed 173 illustrates why net endpoint loss alone is misleading. It loses
56 tokens between the extension's two endpoints, yet keeps two family copies
throughout, has window means between 138.813 and 143.458, and retirement-phase
means between 118.667 and 120.160. Its mean slope is only −0.157 tokens per
1,000 steps. It has 202 paid family births and never falls below 86 tokens.
All four strict segment financing verdicts fail despite these stable stocks.

For a single-copy family at capacity, a paid replacement transfers 50 tokens
to the child and withdraws the parent's remainder back to the free pool.
Family capital returns to 50 at that event and accumulates between births.
The accounting is correct: adjusted surplus equals the change in family
capital. Its sign can depend on which phase is sampled, even for repeated
funded turnover. Removing withdrawals from accounting would obscure this
mechanism, rather than solve the endpoint problem.

The smallest observed pruned stock is 5 tokens on seed 149. Survival through
10,000 steps is evidence over this horizon, not a guarantee of indefinite
viability or a uniform safety margin.

![Pruned replacement capital: window means, retirement-phase means and fixed endpoints](../output/selfmod_v16_financing_20260927/post_analysis/prune_replacement_financing.png)

## Unchanged success functions across fixed segments

Apply the original 2,000-step success functions to each disjoint extension
segment, using its final 1,000 steps and four 250-step late windows. These
are descriptive dependent segment verdicts, never pooled replications.

| Policy | Faithful combined counts, by segment | Prune combined counts, by segment | Faithful families alive at 10,000 | Prune families alive at 10,000 |
|---|---|---|---:|---:|
| ON | 3, 2, 2, 2 | 2, 1, 1, 4 | 2/8 | 6/8 |
| OFF | 0, 0, 0, 0 | 0, 0, 0, 0 | 7/8 | 8/8 |
| Vacancy8 | 0, 0, 0, 1 | 0, 0, 2, 0 | 5/8 | 8/8 |
| Replace8 | 1, 2, 1, 2 | **5, 1, 6, 4** | **3/8** | **8/8** |

Segments are 2,001–4,000; 4,001–6,000; 6,001–8,000; 8,001–10,000.
Every pruned replacement segment has legacy establishment in 8/8 seeds.
For the three old failures, combined segment verdicts are identical:
pass, fail, pass, pass. Their second-segment late deficits are −23, −24 and
−57, respectively. Those failures remain recorded, alongside later recovery.

Pruned replacement has **4,309 culture births and 1,281 family births** in
the extension. Faithful replacement has 1,689 and 131; three faithful families
survive, and only those three reproduce in every 1,000-step window. Its whole
cycles persist in 5/8. The paired genotype difference in sustained turnover
therefore persists, while strict financing pass counts fluctuate with endpoints.

Pruned vacancy8 eventually has 289 culture births as ordinary deaths reopen
slots, but only three selected-family births across the entire panel. No
selected family reproduces in every 1,000-step window. Its occasional segment
successes do not imply continuous selected-family turnover. OFF accumulates
capital but has no later births. No enumeration censoring occurs in any arm.

## Experimental status and next useful step

Keep the v15 primary failure intact. The new evidence supports **productive,
funded bounded turnover with stable or expanding pruned stocks over this
restored extension**, and shows that a nonnegative fixed-window change in
capital is phase sensitive. It does not establish open-ended evolution or
prove viability at arbitrary horizons.

Before a broad mutation campaign, freeze a separate prospective durability
criterion that retains exact withdrawal accounting but evaluates recurring
fully paid births, persistent productive families/cycles and maintained stock
floors across multiple fixed windows. Validate that criterion on a fresh panel,
reporting the old endpoint criterion alongside it. Do not choose a floor or
window after observing the new panel, and do not relabel v15 as replicated.
No additional scientific campaign was launched here.

## Validation and evidence

- All **35 LK `hg-selfmod` library tests** pass, including exact restored
  continuation beyond the old horizon and allocator-order rejection for
  missing, duplicate, sentinel, occupied and out-of-range slots.
- All 16 engineering arms match uninterrupted 128 steps against two restored
  64-step processes: raw rows, full final snapshots and allocator orders.
  **4,096 engineering steps**, excluded from the diagnostic count.
- Independent audit validates every new receipt, input hash, inherited config,
  continuous clock, stock-flow identity, policy/cap/retired donor, allowance,
  genealogy, RNG consumption, window and unchanged segment verdict. It rejects
  eight deliberately corrupted receipt/financing cases. Allocation-order
  validity is checked; exact native allocation, matching, mutation/hash and
  ChaCha values are tested/replayed rather than independently reimplemented.
- Four separate-process seed-131 bounded extensions replay all 8,000 steps
  exactly, including eight checkpoints and their allocator sidecars: **84
  receipt files**, **32,000 excluded verification steps**.
- The frozen source archive, including vendor source and upstream provenance,
  builds after fresh extraction with offline locked Cargo. All scientific
  source hashes remain unchanged; audited v15 evidence remains unchanged.

Artifacts include frozen contract/source/binary and instrument/input
provenance, original v15 verdicts and validated finals, every new row and
checkpoint, `audit.json`, `replay_verification.json`, library/archive check
receipts, `interpretation.json`, and archived post-analysis/plot scripts with
PNG/SVG figure hashes. Plotting dependencies are isolated in a temporary uv
cache. Root ERE sources and experiments were not edited by this diagnostic.

From `experiments/lk`, the pipeline creates exclusive receipts:

```sh
python3 scripts/run_selfmod_financing.py output/NEW_DIAGNOSTIC
python3 scripts/audit_selfmod_financing.py output/NEW_DIAGNOSTIC
python3 scripts/verify_selfmod_financing_replay.py output/NEW_DIAGNOSTIC
uv run --no-project --with matplotlib python scripts/summarize_selfmod_financing.py output/NEW_DIAGNOSTIC
```

The default panel remains deliberately fixed; a repeated extension is not
new-seed evidence. The frozen `selfmod-finance` executable also supports
`FINAL POLICY OFFSET STEPS FRESH_OUT` for receipt replay. Preserve its adjacent
`final_allocator.json` sidecar when resuming a v16 checkpoint.
