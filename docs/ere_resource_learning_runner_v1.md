# Bounded on-policy learning runner and zero-native preflight v1

Keep the sealed learning proposal unchanged: four fixed learner seeds, full/
cues-masked arms, 100 eight-state batch updates, neutral initialization, fixed
score-function credit, denominator8, norm clipping1 and parameter bounds8.
Only final endpoints are confirmed; all 7,440 slots remain fixed. This contract
finalizes the previously open engineering choices, not scientific criteria.

Portable SplitMix64 supplies one top-53-bit draw per training state, including
unpaid states; restart each arm at the same learner seed. Freeze the entire
schedule before native work and SHA256 each seed's 800 little-endian f64 draws.
Confirmation uses 16 midpoint draws per state; constant baselines use neutral
heads and draws0.25/0.75. No native physics RNG change or sampled-seed replacement.

`hg-train::ere::resource_learning` invokes only the verified sim-core timing
bridge. Credit is counted from actual native target-RCF ordinary events at
17–32 plus final payoff-node Int credits; no saved counterfactual utility table
is accessed by the campaign. All eight cells in a training batch use the same
pre-update parameters. Unpaid actor credit is zero irrespective of reward,
but its draw, sampled action, physical result and batch slot are retained.

Write each full native/inference/credit receipt as deterministic compact gzip
(mtime0, pinned existing flate2 1.1.9, fast compression). Do not replace physics
evidence with a summary. Independently audit one cell at a time, retaining a
bounded 16-cell historical reference cache and eight-sample batch. Auditing
checks native equivalence/flags/timing, sampled policy inputs, utility and every
gradient/clip/update against its own arithmetic; historical receipts are only
verification references, never training rewards.

Limits: 238,080 advances, 7,440 paid nonadvancing Seal, 800 optimizer updates,
7,200 seconds for the native process, 7,200 for full postflight auditing, 1 GiB
total campaign output. Before each native call reserve2 MiB for one compressed
receipt plus ledger/manifest/update overhead; reject a compressed cell above
1 MiB. Verify actual directory bytes, cap/truncation and source identity. Fresh
outputs only; no automatic resume, native retry or completion of failed partial
batches. A negative learning criterion does not halt or resample the panel.

Append/sync a start/complete/failure/update ledger and atomically refresh the
work manifest. Native failures record known advances/Seal from sim-core; a
running start without completion is marked work-unknown on process interruption.
Only persist an optimizer update after all8 samples complete; partial batches
retain their cell evidence without an update. No reconstruction of a successful
campaign from dropped or replaced cells. Capture both native-complete and audit
status separately if postflight fails.

The preflight runs zero native advances and zero campaign optimizer updates.
It checks all 7,440 schedule slots, paired portable draw hashes, independent
credit/update arithmetic on eight sealed saved receipts, explicit norm/parameter
projection probes and refusal cases. These one-batch calculations are arithmetic
fixtures, not a cached-reward learning campaign or fitted endpoint. Deterministic
gzip must reproduce and independently decompress; storage ratios are measured
only on saved inputs. Test auditor corruption refusals and both feature-off
checks. Preserve all old archive roots and old bodies, except registered
additive train export, optional pinned compressor/bin manifest additions and
one Cargo.lock dependency entry. No engine/kernel/adapter changes.

Before sealing the preflight, pin the campaign executable and test executable,
actual Cargo/source closure, proposal/contract, copied historical input seal maps,
test receipts, arithmetic outputs, draw/schedule hashes and preservation. Recheck
from frozen source/tests/binaries and copied inputs in a relocated archive,
without live-source fallback or physics. Exercise live tests before freezing to
catch implementation defects without creating unnecessary failed archives.

Launch is explicitly separate: the future wrapper requires `--launch` plus
`--acknowledge-native-advances 238080` and a human authorization note. Neither
this contract nor a passing preflight is launch approval. Scientific/sustained/
training-admission/independent-physics-replay flags remainfalse even if the future
local authored-task learning predicate passes. Fresh seed-wise results, baseline
comparisons and every failed criterion must be reported with denominator4.
