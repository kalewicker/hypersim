# Selfmod v17 — prospective funded-reproduction durability

Frozen before implementation and fresh evaluation, 2026-09-27. This new
criterion is motivated by v16's endpoint-phase diagnostic and is fixed before
observing a new panel. It does not revise v15's failed 5/8 combined-financing
replication, turn dependent v16 windows into replicates, or establish novelty.

## Fresh panel, initialization and policies

Fixed fresh seeds in order: 211, 223, 227, 229, 233, 239, 241, 251. They have
no overlap with prior selfmod manifest seed/world-seed lists. Never replace
untriggered, inadmissible, censored or failed seeds or shrink the eight-seed
denominator. All eight genotype/policy arms are new paired trajectories:
faithful/prune crossed with native ON, OFF, vacancy8 and replace8.

Initialize the exact v15 operating point, with cfg.steps=10000 as the scoring
horizon only. Ordinary churn3, one copy/member and 60 capital, world 200/300,
random tie breaking, match cost 0, threshold 100/transfer 50, rent 1, pool 12,
proportional income, influx 20/free cap 500, creation budget init/max 200 and
refill 1, match cap 64/enumeration ceiling 50000, world ceilings 500/1000,
hard node stop 20000, mu/abio/p_dup/p_del zero. Other original defaults unchanged.
Stop the shared faithful prefix at its first naturally funded R1 birth, at
most 256 object steps. Replay the complete paid birth phase for faithful and
the exact named v11 R1 prune, preserving siblings, transfers and allowance.
All later children remain faithful in their genotype, with unchanged v14
policy/retirement functions. No resets, resident culls, subsidy or intervention.

Save and restore exact allocator order starting at fresh initialization and
the prefix fork; preserve world/rules, full ledger/config, immutable genealogy,
absolute clock, sequence and ChaCha stream/position. Thus the historical v15
missing-allocator limitation is not inherited by this new experiment.

If no R1 trigger within 256, record untriggered and no continuations. If the
complete initial birth exceeds eight residents, run ON/OFF but mark both
bounded treatments inadmissible without culling. Retain those cases as no
demonstrated success in the fixed denominator. Continue valid trajectories
after extinction for exactly 10000 additional steps; preserve runtime/hard-stop
failures, never count incomplete or enumeration-censored runs as successful.

Maximum science: 64 x 10000 = 640000 continuation steps + at most 2048 shared
prefix steps = 642048. Up to four independent processes concurrently, no shared
mutable simulation state/RNG. Preserve all historical raw evidence.

## New primary durability criterion

The first 2000 continuation steps are prospectively fixed warmup. Score the
remaining eight disjoint 1000-step windows, 2001–3000 through 9001–10000,
each with four disjoint 250-step subwindows. Require ALL of:

1. Executable, uncancelled selected child; full uncensored 10000-step run.
2. At every scored phase boundary (including step 2000), at least one living
   selected-family copy and family capital >= rent x current living family
   copies. This floor derives from the next scheduled family rent obligation,
   not the observed minimum capital in a historical panel. It is aggregate
   one-step rent coverage, not a guarantee that each individual can pay or that
   the family survives arbitrary future horizons. Ordinary starvation is not
   hidden; record copy losses and minimum capital/headroom.
3. All three culture members alive at every scored boundary, and each member
   and the selected family productive in EVERY scored 250-step subwindow.
4. An actual culture funded birth in EVERY scored 250-step subwindow and an
   actual selected-family funded birth in EVERY scored 1000-step window.
5. Every birth passes the normal threshold/payment/inherited-budget audit;
   every retirement returns only the paying donor's remaining capital; all
   money/allowance/genealogy/RNG/cap receipts and each window's exact identity
   `end-start = income-rent-retirement returns-starvation returns` reconcile.

There is NO nonnegative endpoint-change requirement in this NEW primary
criterion. Withdrawals remain fully counted and recorded. Pass >=6/8 fresh
prune_replace8 seeds to meet its declared durability replication threshold;
apply/report the same criterion for every other arm. No pooling old panels.
Report matched gains/losses and missing reasons; no significance claim from
eight worlds, no independent-sample claim for serial windows.

Retain the UNCHANGED v15 establishment/combined endpoint functions as secondary
metrics for all five disjoint 2000-step segments (1–2000 through 8001–10000),
with each segment's final 1000/four 250 windows. Scoring uses a local 2000-step
view only; it never changes physical RunConfig or state. Report stock traces,
floors/headroom, living-copy counts, post-retirement stocks and descriptive
mean slopes alongside both criteria. No floor/window/threshold tuning after
fresh output and no automatic broad mutation campaign.

## Validation and evidence

Engineering: seeds 7/42, all 16 fresh arms, cfg horizon 10000, 2000-step
continuations. Compare uninterrupted execution against two separate restored
1000-step processes, including concatenated rows, intermediate/final snapshots
and allocator sidecars; verify exact faithful fresh prefix/birth pairing.
64000 continuation engineering steps plus shared prefixes, excluded from science.
Require library tests and synthetic criterion cases: negative endpoint change
with maintained stock and continuing funded births may pass the new criterion;
crossed rent floor, extinction, missing family/culture birth, inactive member,
enumeration censoring and false verdict must fail. Audit prefix selection,
missing/over-cap routing and fixed denominator, including corruption rejection.

Archive source/vendor/contract, binary, seed inventory, named v11 evidence,
validated v16 reference and engineering receipts before scientific execution.
Independently reconstruct every prefix/trigger, paired paid birth, continuation
receipt, window, stock floor/headroom and primary/secondary verdict. Preserve
exclusive raw output, explicit complete/failed status and full checkpoints.
Native matching/effect classification/hash/ChaCha and exact allocator reuse
are tested/replayed rather than independently reimplemented.

Verification: first triggered cap-admissible seed in fixed order, regardless
of outcome. Separate-process regenerate initialization, prefix fork and both
paid births including allocator orders. Then exactly replay all four bounded
10000-step continuations, all ten checkpoints, final states and allocator
sidecars: 40000 plus <=256 prefix verification steps, excluded from science.
If none admissible, explicitly report verification unavailable, no panel change.
Fresh archive must compile offline/locked. This tests the initial named variant
as a durable funded reproduction platform, not repeated self-modification.
