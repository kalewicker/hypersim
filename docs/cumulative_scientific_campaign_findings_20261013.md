# Fixed cumulative-evolution campaign — 2026-10-13

**The authorized campaign completed and returned a valid negative under the
unchanged criteria.** All 188 worlds, 1,880 sampling slots and 111 unique assay
requests completed. The independent campaign audit passed. No seed was replaced,
no native unit was retried, and no threshold was relaxed.

## Scientific result

The fixed denominator is 47 matched blocks with endogenous, external,
nonheritable and frozen arms. Endogenous qualification was **0/47**, against the
preregistered primary threshold of 16/47. External qualification was also **0/47**.
Neither mechanical control qualified. There were no unknown worlds or missing
sampling slots. The frozen decision is `valid_negative`; prevalence and paired
advantage p-values are both 1.0. Cumulative evolution and a self-modification
advantage were not demonstrated in this fixed world and diagnostic environment.

The negative is a completed scientific result, not an operational failure. It
does not establish that every possible Hypersim architecture or environment is
incapable of cumulative evolution.

## Where the observed candidates stopped

This is a descriptive reading of the sealed measurement report, not a new test or
an authorization to alter the campaign. Repeated observations of the same event
pair are not independent innovations or replicates.

| Arm | Living / dead sampling slots | Candidate-pair observations | Distinct world/event pairs | Qualified worlds |
|---|---:|---:|---:|---:|
| Endogenous | 351 / 119 | 6 | 3 | 0/47 |
| External | 373 / 97 | 0 | 0 | 0/47 |
| Nonheritable | 352 / 118 | 3 | 2 | 0/47 |
| Frozen | 428 / 42 | 0 | 0 | 0/47 |

All nine pair observations had complete, funded intervention measurements and
nonempty ablations. None met the required two paid transmissions and two distinct
active descendant generations for **each** acquisition.

- **Endogenous block 002:** the same pair appeared at censuses 4, 8 and 16.
  Its paid acquisition margins were +5 and +13; intervention, interaction,
  sham and restoration checks passed. But its transmission counts were [1, 0]
  and its active descendant-generation sets were [[1], []]. It therefore failed
  the inherited persistence requirements. This is a functionally promising
  observed pair, not accepted cumulative evolution.
- **Endogenous block 037:** one observation had paid acquisition margins
  [-14, +13] and no paid transmissions or active descendant generations.
- **Endogenous block 044:** two observations of one pair had paid margins
  [-14, -7], failed required removal/interaction checks and had no paid
  transmissions or active descendant generations.
- The nonheritable observations in blocks 037 and 044 also lacked paid
  transmissions and active descendant generations. External and frozen samples
  contained no retained candidate pairs under the frozen extraction rule.

The strongest observed pair thus failed on inherited persistence, while other
endogenous pairs also failed functional acquisition criteria. These observations
identify failed gates; they do not causally establish which change would repair
them. The stopping rule is to close this configuration with its negative result.
There is no automatic longer run, additional seed panel, relaxed inheritance
criterion or new architecture campaign.

## Execution and evidence

The user explicitly authorized the one reviewed campaign on 2026-10-13. Its
directory retains the reviewed `20261011` suffix because the approval binds that
exact path. The frozen driver and executable were used without rebuilding.

- Campaign: `output/cumulative_scientific_campaign_20261011/`.
- Authorization: `output/cumulative_scientific_campaign_authorization_20261013.json`.
- Preflight: `output/cumulative_scientific_launch_preflight_20261013.json`.
- Status authority: campaign `manifest.json`, with `status: complete` and
  `independent_campaign_audit_pass: true`.
- Scientific decision and all measured pairs: campaign `report.json`.
- Independent audit: campaign `audit_command.json` (exit 0) and `audit.json`.
- External sealed-archive recheck:
  `output/cumulative_scientific_campaign_20261013_recheck.json`.

All 189 native units have complete audited receipts and exactly one intent claim:
188 worlds plus one batch containing 111 globally registered assay requests.
World rewrites were 12,238; assay rewrites were 940; total native rewrites were
**13,178**, with equal lower and upper accounting bounds and no unknown work.
Funded unit time was **440.141 seconds**. The authorization-copy-to-final-seal
file timestamps span approximately **656.28 seconds**; this is a filesystem timing
observation, not a replacement for controller budget accounting.

Peak admission storage charge was **243,630,080 bytes** (232.34 MiB), including
reservations. The completed archive occupies **24,832,427 file bytes**, including
its inventory. All are within the authorized 2,000,000-rewrite, 3-hour funded
unit-time, 6-hour workflow and 8-GiB storage ceilings.

The archive contains 219 pinned sources and 9,306 sealed payload entries. Its
`output_sha256.json` SHA-256 is:

`7d49f529fa597678086fa23c354dbf9897d90dc285fb75158656993406f17f52`

The external rechecker verifies the complete inventory and hashes, independently
reconstructs the native evidence, measurement and inference, and requires exact
agreement with the completed manifest and audit. It performs zero native work.
It passed with exit code 0 in 38.24 seconds, with identical counts and no stderr.

The reused nested measurement helper retains its legacy `development_only`
scope label and the inference helper retains false engineering/authorization
flags. These are preserved verbatim. The actual authorization is in the outer
campaign authorization/controller/manifest, and the scientific decision is in
the versioned outer report. Scientific acceptance and evolution-demonstrated
flags remain false, consistent with the negative result.
