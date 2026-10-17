# Selfmod v16 — continuous financing diagnostic

Frozen before implementation and diagnostic execution, 2026-09-27.
Engineering amendment before ANY diagnostic output: a new beyond-horizon
restoration test found SlotMap 1.1.1's upstream serde omits free-list order.
Long split runs can change vacant-slot reuse and full snapshots even while
raw biological receipts agree. An opt-in v16 allocator sidecar now preserves
the exact node/edge free-list order and versions across all v16 checkpoints.
The local vendored dependency adds only validated export/restore APIs; default
serialization, allocation and old assays remain unchanged. Archive vendor
source/provenance and test rejection of malformed orders. All split/replay
checks also require allocator sidecar parity.

Historical v15 snapshots do NOT contain original in-memory free-list order;
it cannot be recovered. V16 therefore begins from each exact *restored v15
snapshot*, whose canonical upstream restoration order is explicit in an
initial sidecar. It cannot claim full-state parity with a hypothetical
uninterrupted v15 run. This limitation applies to the whole fixed panel,
not just failed seeds, and does not revise the v15 primary verdict.

Extend every completed v15 trajectory, all eight fixed seeds
131/137/149/157/163/173/181/191 and all eight genotype/policy arms. No selection
by the v15 verdict, missing-arm substitution, tuning or new mutation. Preserve
v15's 5/8 primary pruned-replacement verdict and 6/8 replication threshold;
this is a dependent longitudinal diagnostic, not another independent seed panel.

Restore each exact validated v15 final snapshot at continuation step 2000.
Keep its full RunConfig (including the old scoring horizon), world, rules,
capital/free ledger, creation allowance, genealogy, sequence, absolute clock,
ChaCha stream/position and probe. Advance 8000 more native object/birth steps
to continuation step 10000 under its original ON/OFF/vacancy8/replace8 policy.
Use the unchanged v14 phase functions. No reset, top-up, refill intervention,
new initial birth or capital injection beyond ordinary configured influx.
Continue after biological extinction. Stop only on the inherited hard stop or
runtime failure; preserve incomplete raw data and never count them as success.
Save every row, initial/final state, and a checkpoint each 1000 extension steps.

64 trajectories x 8000 = 512000 NEW diagnostic advancing steps; no new prefix.
Prior v15 steps are reference observations, not rerun, pooled or counted again.
Execute at most four independent processes concurrently; scheduling does not
share mutable simulation state or RNG. Keep all prior evidence immutable.

## Fixed measurements

For each of the eight disjoint 1000-step extension windows (2001–3000 through
9001–10000), reconstruct selected-family income, rent, capital withdrawn on
parent retirement and residual starvation, start/end/min/max/mean capital,
mean live family copies, culture/family funded births and productive firings
in all four 250-step subwindows, member persistence, all resource gates and
enumeration censoring. Enforce exactly:

`end capital - start capital = income - rent - retirement returns - starvation returns`.

Capture selected-family capital immediately after every family retirement,
with step and live family-copy count. Summarize that phase-conditioned stock
in each fixed window (count, minimum, maximum and mean); do not pick endpoints
or exclude events using the observed outcomes. Report both increasing/decreasing
fixed-window stock balances and phase-conditioned balances. Record total
capital alongside copy count; population changes can affect its interpretation.

Across the eight new windows report their capital means and an ordinary least
squares descriptive slope per 1000 steps, net change from 2000 to 10000,
all-window minimum, final selected-family/culture survival, number of positive,
zero and negative window balances, and continuing-birth gaps. Serial windows
are not independent samples; no p-values or pooled replication percentages.
An oscillating stock with recurring births and no sustained decline supports
a phase explanation; declining window means/phase-conditioned stocks or
extinction supports erosion. Report mixed/insufficient evidence candidly;
do not force either explanation from a sign or slope alone.

Additionally apply the UNCHANGED v15 2000-step establishment and combined
functions to each disjoint extension segment 2001–4000, 4001–6000,
6001–8000 and 8001–10000. Its last 1000 steps and four final 250-step windows
remain exact. These are descriptive segment verdicts on dependent extensions,
not a replacement primary criterion or independent replication. Preserve each
seed/arm and explicitly highlight the prior financing failures 157/181/191
only in interpretation after analyzing the entire fixed panel.

## Engineering, audit and replay

Before science, extend all 16 validated v15 engineering final states (seeds
7/42, all arms) 128 steps. Compare uninterrupted execution against two
separate restored 64-step processes, including concatenated byte-identical
rows and full final snapshots. 4096 engineering steps, excluded. Require
library tests including restoration across a split beyond cfg.steps.

Archive validated v15 receipts/input hashes, original verdicts, source,
binary and this contract before execution in a fresh exclusive output.
Independent audit reconstructs every new row's money, policy births, donor
retirement, allowance, genealogy, RNG counts and snapshots; reconstruct
all stock-flow windows and unchanged segment verdicts. Reject corrupted
start-state, offset, income/retirement, cap/RNG and financing metrics.
Native matcher/effect classifier/hash/ChaCha values are tested/replayed,
not independently reimplemented.

Separate-process replay of all four bounded genotype/policy extensions for
seed 131, fixed by seed order independently of outcome: 32000 verification
steps, excluded. Compare initial state, all 8000 rows, eight checkpoints and
final state byte-for-byte. Check a fresh extracted archive compiles offline
and locked. No automatic follow-up experiment or broad mutation campaign.
