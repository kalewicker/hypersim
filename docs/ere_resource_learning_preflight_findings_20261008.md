# Bounded learning runner/auditor preflight — 2026-10-08

## Outcome and experiment state

The bounded on-policy learning runner, independent auditor and explicit launch
guard are implemented. The zero-native engineering preflight passes. No
training campaign, native advances or campaign optimizer updates were run.
The single saved-receipt batch calculation is an arithmetic fixture, not an
optimized endpoint or cached-reward training run.

`output/ere_resource_learning_preflight_20261008` is complete and sealed:
732 entries, 673 source pins, root
`72e7635f031dfa46cc42520cf1089b50fa0e0e774224c0b554c342cd119df319`.
The workflow took 8.649 seconds; preseal output is 54,147,336 bytes. Live checks
were performed in zero-native scratch before freezing the final package; no
failed immutable package or corrected criterion was needed for this milestone.

The future 100-update comparison remains proposed/unlaunched. Original ERE
scientific acceptance, sustained claim, training admission and independent
physics replay all remain false. Passing this preflight is not learning evidence.

## Extension map

- `crates/hg-train/src/ere/resource_learning.rs`: portable draw stream, full
  fixed schedule, actual native utility extraction, eight-cell batch arithmetic,
  parameter/norm projection, deterministic gzip, synchronized append ledger,
  atomic work manifest and explicitly acknowledged native campaign execution.
  The campaign calls the existing sim-core timed bridge; it never imports an
  engine or loads a saved reward table. Parameters are unchanged within each
  batch; no update is committed until its eighth cell completes.
- `crates/hg-train/src/bin/ere-resource-learning.rs`: `schedule` is zero-native;
  `campaign OUTPUT ACK_NATIVE_ADVANCES BOOTSTRAP_BYTES` is a separate branch
  requiring exact budget acknowledgement and actual bootstrap-byte agreement.
- `scripts/run_ere_resource_learning.py`: defaults to zero-native preflight.
  A separate launch branch requires `--launch`, a passing authenticated sealed
  preflight, an explicit human authorization note and the exact advance budget.
  It reuses the preflight's pinned campaign executable rather than rebuilding
  or changing endpoints. Fresh outputs only; failed/partial work is retained.
- `scripts/verify_ere_resource_learning.py`: independent SplitMix64 draws,
  native event/property credit, sampled policy/timing/native-table equivalence,
  every contribution/mean/clip/update, fixed endpoint confirmation, paired
  four-replicate criteria and resource/ledger accounting. Reads compressed
  campaign receipts one at a time with a fixed 16-cell reference cache; no
  eager whole-campaign graph load.
- `scripts/recheck_ere_resource_learning.py`: self-contained frozen schedule,
  gzip and arithmetic reproduction plus Python tests. `--campaign` audits only
  saved future campaign evidence; neither branch launches physics or fitting.

The old native kernel, timed bridge, resource adapter and learning proposal
remain byte-identical. Of the prior 664 pinned bodies, 661 are unchanged. The
three registered exceptions are the additive train-module export, an optional
pinned existing `flate2=1.1.9` dependency/bin entry, and its one Cargo.lock
dependency entry. No dependency version or physics/engine rule changed. Both
feature-off checks pass; compression is only enabled with ERE.

## Frozen schedule and draw evidence

All 7,440 trajectory slots are precommitted before any native work:

| Phase | Trajectories | Advances | Seal | Optimizer updates |
| --- | ---: | ---: | ---: | ---: |
| Four learner seeds × two arms × 100 updates × eight states | 6,400 | 204,800 | 6,400 | 800 |
| Final-endpoint confirmation, 16 midpoint draws per state | 1,024 | 32,768 | 1,024 | 0 |
| Two constant-action baselines across eight states | 16 | 512 | 16 | 0 |
| Maximum total if separately launched | 7,440 | 238,080 | 7,440 | 800 |

SplitMix64 uses top 53 bits divided by 2^53. Each arm restarts the same stream
for its paired learner seed; unpaid states still consume a draw. Each seed's
800-draw SHA256 covers concatenated little-endian f64 bytes:

| Learner seed | Draw SHA256 |
| --- | --- |
| 993748721101 | `e5ef322f42d1bc688e2e98626b6bf74f6690bfdc1b7abe0beebf1ae7a5a9e814` |
| 993748721102 | `fb6e183945d24ed26416a2b7da6f89561c0156b2165c138f497b5728511cd39e` |
| 993748721103 | `9b64d3ac71f514abcf4dc75d7a16cd37c361f17c70a46b5ce6bdc1d8ec3a513c` |
| 993748721104 | `f906455263c64b5c626de7ed971d98e8a5d0fcd81dbca4a8a944d55193b58c7d` |

These are learner replicates in one authored deterministic physical world,
not independent physical seeds, unseen tasks or a generalization panel.

## Credit, tests and storage

The frozen proposal's funded credit is
`((actual utility - 9.5)/20) * grad(log P(sampled action))`.
Unpaid actor contribution is zero even when native ordinary utility is eight;
its outcome and slot remain retained. Divide the eight contributions by eight,
clip aggregate norm at one, ascend with rate one, clip parameters to [-8,8].
There is no fitted critic, teacher action, entropy bonus, tuning, curriculum,
best-checkpoint selection or seed replacement.

Preflight verifies:

- All 7,440 Rust schedule slots against independently generated Python slots,
  including paired draws, original float round-trips and exact phase counts.
- Actual credit from eight sealed saved timed-bridge receipts and the resulting
  single diagnostic batch, independently recomputed in Python.
- Explicit norm `[3,4,0,0]` and parameter `[9,-9,3,0]` projection probes;
  incomplete, reordered, invalid-domain and nonfinite batch refusals.
- Deterministic gzip reproduction and independent decompression back to full
  original receipts, not reduced summaries.
- Six Python groups, including corrupt schedules/draws, credit, gradients,
  parameter results, policy/timing metadata and rejection of a partial campaign
  falsely marked complete. The campaign's full audit requires all 7,440 cells,
  all 800 update receipts, exact ledger order and all four replicates.
- Two normal zero-native Rust groups plus the explicit saved-credit fixture;
  both `hg-train` and `hg-sim-core` feature-off checks; unchanged historical
  archive seals and the registered source-body exceptions.
- Frozen recheck of binaries, schedule, arithmetic, gzip bytes and audit/tests,
  both in place and from a relocated complete archive copy, with no new
  native/optimizer work.

Eight original pretty-JSON receipts occupy 17,490,888 bytes; compact gzip uses
472,274 bytes (2.7001%). Extrapolating those fixtures alone yields 439,214,820
bytes of cell storage for 7,440 trajectories. This is a development estimate,
not a guaranteed full-run size or resource pilot: future parameter strings and
both action branches may vary. The hard 1 GiB campaign cap, per-cell reserve,
actual file-byte checks and failure retention remain authoritative.

The campaign executable SHA256 is
`297c712d2828b95a1d9d34ac48126153483eeb80bb375c61087976ccf29bf12a`;
test executable SHA256 is
`f281d296bd501e8e0600d4fc70b052a9eb6ead5c4260e6a10b26b0e3e412d933`.
Evidence includes `manifest.json`, `audit.json`, `schedule.json`,
`draw_hashes.json`, `arithmetic.json`, compressed/original saved inputs,
binary/build/test receipts, source snapshots and before/after preservation.

```sh
python3 -B scripts/recheck_ere_resource_learning.py output/ere_resource_learning_preflight_20261008
```

The helper dispatches to the archive's own frozen code. A copied archive can
invoke its own `source_snapshots/scripts/recheck_ere_resource_learning.py`
using the copy's absolute path, without live source or historical inputs.
Authenticate the archive against the known seal above; self-consistency alone
does not establish authenticity. This reproduces inference/arithmetic/storage,
not independent physics or new-toolchain compilation.

## Ready next step and limits

The next evidence-producing step is explicit approval to launch the frozen
100-update full-versus-cues-masked comparison. Its maximum is 238,080 advances,
7,440 Seal and 800 optimizer updates, with a two-hour native deadline, separate
two-hour postflight audit deadline and 1 GiB total output cap. It can stop early
on an engineering/resource failure without replacing cells or completing a
partial batch. It does not stop early merely because learning appears negative.

The unchanged local learning predicate is pooled full funded mean ≥11.5,
positive pooled full-minus-masked difference, and at least three of four full
replicates strictly beating both constant harvest11 and their paired masked
endpoint. Every replicate and failed criterion must be reported. No outcomes
exist yet; there is no promise this 100-update budget will meet that predicate.
Even passage would be externally learned allocation on an exposed authored
task, not endogenous cognition or the original plan's remaining scientific gates.
