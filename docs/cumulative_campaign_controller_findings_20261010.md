# Durable campaign controller rehearsal — 2026-10-10

The durable scheduling and accounting layer now passes a zero-native rehearsal.
It preserves the frozen campaign denominator, publishes intents before dispatch,
binds receipts to exact request identities, freezes one global assay registry, and
closes interrupted work without retrying or turning missing evidence into failure.
This is a controller engineering result; a live scientific executor is not yet
integrated.

## Implemented boundary

`experiments/cumulative/scripts/campaign_controller.py` implements an immutable,
hash-linked event ledger with exclusive writer locking and synchronized publication.
Its sequence is fixed worlds, complete measurement/registry freeze, contiguous
assay batches of at most 128, then a content-bound decision or operational closure.

Each intent reserves rewrite and native-time allowances and checks workflow and
storage charges before it can be recorded. The registry barrier requires funding
for the entire assay denominator before the first batch. Exact completed receipts
release unused reservations. Unknown receipts retain their full reservations and
stop scheduling. Reopening a pending intent never redispatches it automatically.
Operational closure retains pending worlds as unknown and untouched worlds as
not started, without publishing a negative or positive decision.

The controller only accepts `saved_development` and `synthetic_full_scale` purposes.
It contains no native execution function. Timing and storage values are supplied
rehearsal inputs; passing these checks does **not** establish live clock enforcement,
actual campaign disk usage or campaign completion time.

## Evidence

| Route | Denominator | Result |
|---|---|---|
| Saved development | Four worlds, 40 census slots, nine assays | Independently recovered native journals bind to controller receipts; 335 historical rewrites accounted, zero new rewrites |
| Full-size synthetic | 188 worlds, 1,880 slots, 78,960 assay requests | 617 contiguous batches and 1,612 durable events complete |
| Interrupted synthetic | Same 188-world panel | Reopened pending first world closes as unknown; remaining 187 stay not started; full 250-rewrite reservation retained |

The synthetic maximum-denominator route accounts for 1,623,474 hypothetical
rewrites, matching the frozen conservative campaign envelope. These numbers are
simulated ledger charges, not executed native work. Its 805 milliseconds of
accounted time and 3,144,140,800-byte storage charge are explicit test inputs,
not new timing or compression measurements. The actual rehearsal executes
**zero native rewrites** and takes 11.51 seconds of workflow time.

Eight test groups reject scientific purpose/flags, changed request identities or
ordering, all four resource-limit violations, pending redispatch, unknown-work
refunds, changed/overrun receipts, premature or duplicate registry freezes,
insufficient full-registry funding, incomplete decision closure and torn/changed
event chains. A separate auditor reconstructs the ledger without importing the
controller reducer and independently rechecks the saved native evidence.

Package: `output/cumulative_campaign_controller_20261010`.

Seal: `67b3c53a77fa692f504218e09a18753fb865ed46cd3dbe597440cfc48188dd0b`.

The package contains 4,065 sealed entries, 31,864,657 payload bytes and 174 frozen
source files. All sixteen preceding archive roots and 168 previous source pins
remain unchanged. The complete measurement parent, including its preserved report
assembly failure, is retained inside the package. Original and relocated known-root
audits pass with zero native work; external receipts are
`output/cumulative_campaign_controller_20261010_recheck.json` and
`output/cumulative_campaign_controller_20261010_relocated_recheck.json`.

## Draft final scope and remaining execution boundary

`final_campaign_draft.json` binds the original 47-block/188-world panel, prospective
candidate and statistical rules, and the proposed 2,000,000-rewrite, 8 GiB,
three-native-hour and six-workflow-hour ceilings. These remain **unapproved scientific
budgets**, not extensions of the zero-native rehearsal allowance.

The proposed claim is cumulative functional evolution within the fixed world and
diagnostic environment, with a separate paired test for endogenous editing advantage.
The fixed assay uses environment 512, four cycles and seed `2026100903`. No
environmental generalization or open-ended evolution claim is included. This narrow
scope requires no new assay environment or additional scientific panel.

The next bounded implementation is a live dispatch bridge and a versioned native
admission profile restricted to exact frozen identities. It must supply real clock
and spool admission values, verify native journals before admitting receipts, and
exercise the controller end to end on development evidence. The original development
native profile remains unchanged. Only after that integrated check should the final
scientific contract and budget be presented for launch authorization. All 188
scientific worlds and 1,880 sampling slots remain unstarted; all scientific admission
and acceptance flags remain false.
