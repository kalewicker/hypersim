# Cumulative campaign adapter and capacity benchmark — 2026-10-10

The bounded development adapter passes. It supports the proposed world dimensions,
all thirteen intervention roles, a complete global assay registry split into native
batches, and authenticated one-hop continuation. The measured evidence format does
**not support the proposed 8 GiB campaign allowance**. Whole-campaign readiness,
scientific acceptance, evolution demonstrated and launch authorization remain false.

This is the next component of the existing experiment, not a new evolutionary track.
No scientific world was advanced, no criterion was relaxed, and no development
outcome was used to select or replace a scientific seed.

## Implemented behavior

`experiments/cumulative/src/bin/cumulative-campaign.rs` is a separate development
entry point. Its request profile permits only development seed `2026101001` and
rejects scientific launch requests. World specifications retain the existing
finite economy, scheduler, editing, copying and checkpoint implementation.

The adapter validates the **entire registry** before executing any entry, including
entries outside the requested batch. Global IDs are contiguous and retained in
every result. The registry admits up to 78,960 entries; each invocation selects
at most 128. The executed fixture has 142 requests split into 128 and 14. It tests
the batch boundary, not execution or performance of the full 78,960-entry registry.

`padded_remove_later` and `padded_remove_both` preserve their semantic names and
stage vectors in the global request. Their native call uses the existing `intact`
tag with local ID zero: tags do not alter the unchanged frozen chemistry. The
independent auditor verifies both the original request and this explicit mapping.

Every action retains a synced intent and a committed result. Continuation references
the complete retained source journal by hash; it does not duplicate every old
receipt in the new journal. Before dispatch, the publisher independently audits
that source. Pending actions, changed identities and invalid registries are refused
before native work. The archive retains both halves of each continuation and can
be checked after relocation without reading the absolute paths in old requests.

Only an additive module export changes an existing source file. All previous
Cell, population, lineage, regulation, world and journal bodies remain byte-for-byte
preserved, as do all eleven earlier sealed archives. No dependency was added.

## Fixed development panel

- Four matched worlds, one for each arm, use two `[0]` founders, capital 40,
  reservoir 512, two parcels per tick, eight sites and sixteen ticks.
- Eight repeated one-tick stress trials use a prepared accounting-valid state at
  tick 15: 32 retained cells, eight living founders, eight genes per cell and eight
  funded resource parcels. Each trial executes 48 native rewrites. The 24 dead
  children and earlier birth records are **synthetic preparation**, not observed
  paid descent, a reachable-pedigree proof or evolutionary evidence.
- Thirteen authored intervention controls plus 129 fixed eight-gene requests test
  all roles and the batch boundary. Duplicate stress requests remain deliberately
  in the denominator; they are not additional scientific replicates.
- World and assay committed-boundary continuations reproduce the complete
  reference results exactly. An interrupted world retains its pending reservation;
  continuation is refused. Invalid final registry rows, registry hash changes,
  changed source identities, a reserved scientific seed and a zero-work cap are
  tested explicitly.

The authored thirteen-role control is funded throughout. Its later gain is 20
with the earlier component and 0 without it; the difference of differences is 20.
These are intervention-capacity checks on supplied programs. They establish no
newly discovered lineage, transmission history or autonomous cumulative evolution.
The four world trajectories were not classified as scientific successes or failures.

## Accounting and verification

| Check | Result |
|---|---:|
| Rust regression groups | 38 passed |
| New zero-native corruption checks | 11 rejected |
| Previous exact decision-rule groups | 10 passed |
| Declared routes | 25 |
| Global assay slots executed | 142 |
| Native test work | 1,765 |
| Four development worlds | 275 |
| Repeated maximum-dimension ticks | 384 |
| Complete 142-slot assay registry | 2,732 |
| World continuation pair | 72 |
| Thirteen-control assay continuation pair | 152 |
| Uncommitted interruption allowance | 0–12 |
| Total retained native work | **5,380–5,392**, below 6,000 |
| Aggregate native command time, including tests | 44.619 seconds, below 300 |
| Publication workflow | 67.287 seconds, below 1,200 |
| Archive payload | 42,570,825 bytes, below 128 MiB |

The uncertainty is deliberate and remains in the durable work ledger even though
the fault-injection process returned. It is not converted into a completed action
or retried. The previous complete durability archive is independently re-audited.
Original and relocated known-root rechecks run the frozen auditor and corruption
checks with zero native rewrites; their receipts remain outside the seal.

## Campaign resource consequence

The preregistered *proposal for evaluation* was 2,000,000 native rewrites, 8 GiB
output, three native-command hours and six workflow hours. It was not approval to
launch a scientific campaign.

The largest observed world journal record is 1,286,247 bytes. The largest observed
eight-gene assay record is 114,262 bytes. Applying those observed maxima to the
fixed denominator, with conservative allowances for intent/result records and
headers, projects **26,890,574,044 bytes (25.04 GiB)**. Even scaling assay results
alone at the observed maximum gives 8.40 GiB before any worlds or other evidence.
Thus the earlier 3.48 GiB small-fixture extrapolation was insufficient for capacity
planning. The new estimate is conservative observed sizing, **not a certified bound**
on all reachable records and not a claim that every actual request will be maximal.

Measured development command times project to roughly **157 minutes**. This also
is not a guaranteed limit or a complete campaign verification-time benchmark.
The independently derived 1,623,474-rewrite reservation remains unchanged and
fits the proposed two-million-rewrite ceiling; storage feasibility does not pass.
No larger allowance is silently adopted in response to this result.

## Evidence and next boundary

Archive: `output/cumulative_campaign_adapter_readiness_20261010`.
Seal: `827890af64baf8cf8c25d67056a5006294c6c95fcddd888d03870c6d6287f794`.
There are 924 sealed entries, 149 first-party source pins, the preserved locked
SHA dependency sources, the native binary, every input, journal and command receipt.

Recheck receipts:

- `output/cumulative_campaign_adapter_readiness_20261010_recheck.json`
- `output/cumulative_campaign_adapter_readiness_20261010_relocated_recheck.json`

With the known seal independently checked, the frozen zero-native commands are:

```sh
python3 -B output/cumulative_campaign_adapter_readiness_20261010/source_snapshots/experiments/cumulative/scripts/audit_campaign_adapter.py output/cumulative_campaign_adapter_readiness_20261010 --sealed
python3 -B output/cumulative_campaign_adapter_readiness_20261010/source_snapshots/experiments/cumulative/scripts/test_campaign_adapter.py output/cumulative_campaign_adapter_readiness_20261010
```

The proposed 8 GiB envelope remains unsupported. Before any scientific execution,
resolve evidence storage by validating a lossless compact representation against
these saved receipts or explicitly adopting a larger fixed budget. That work can
use existing evidence and should not require another seed pilot. If the chosen
fixed storage envelope cannot be met, close the attempt rather than dropping
requests, receipts or failed worlds.

The remaining integration is the complete campaign census/version-lineage to
thirteen-role request mapping and final scientific contract. Current frozen
intervention chemistry is deterministic: changing its unused seed is not an
independent environmental confirmation. The 47 untouched seed blocks can provide
prospective world replication; no separate environmental robustness claim is
admitted by this fixture. That scope must be explicit in the final contract, and
any genuinely different confirmation environment needs its own fixed definition
before results. This component does not remove that final preregistration gate.
