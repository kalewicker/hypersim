# Feasibility-aware short training: prospective contract

Registered 2026-10-01, before any physics under this profile. User request:
"ok let's try that training", accepting the previous feasibility-aware proposal.
Profile: `E2-complete-feasibility-training-v1`; objective version:
`feasible-first-reuse-v1`. One fresh invocation, no outcome-dependent retries,
replacement, culling, early statistical stopping or generation selection.

## Fixed scope and inherited evidence

Train 20 generations × 16 candidates × 100 native steps from 183 zero logits.
Use the parent's typed grammar, ordinary fixed world, metadata-isolated execution,
coherent kernel, native reuse reward and opt-in pooled 50,000/50,000 matcher.
No Engine, native evaluator, matcher, grammar, optimizer core, reward or cap
change is permitted. This route is separately user-authorized; it does not open
the legacy admission guards or authorize the reserved 10,000-step campaign.

Parent receipt: `output/ere_complete_short_training_continuation_20260930`.
Require root seal
`dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132`,
all 1,690 sealed entries with exact coverage, the parent's original frozen audit
and precheck pass, and all 230 native/Cargo/optimizer source pins equal live.
The preceding champion confirmation remains unsuccessful and immutable. Its
512 draws and the parent's 64 pilot draws are not reused or pooled here.
No fresh precheck physics: the inherited evidence covers this unchanged native
100-step scope; the new adapter is verified with development tests instead.

## Constraint-aware optimization, not fabricated native fitness

Native valid scores remain measured reuse in [0,1]. Native cap scores remain
null, with their full terminal receipt. For optimizer ordering only, a feasible
candidate ranks above every resource-capped candidate, then feasible candidates
rank by their native score cast to f32, preserving the previous ES ranking.
Capped candidates tie with each other. Dense ordinal codes preserve these keys
without adding an offset in f32 that could collapse near-ties. Feed all sixteen
finite ordinal scores into the existing tied-rank ES, eight antithetic pairs,
sigma 1 and alpha f32 .05 (JSON widened value 0.05000000074505806).

Both-capped arms have equal shaped ranks and zero pair difference. A pair with
one cap receives directional pressure toward its feasible arm. All-feasible
ordering, tie treatment and updates are identical to legacy ES on the same
scores/noise. This does not change the global rank-shaped ES estimator: pair
differences and the denominator of eight pairs remain unchanged. A generation
with all caps has a zero update but is retained; no valid-pair masking or
minimum-valid-pair retry is applied under this new objective version.

For descriptive and held-out constraint utility, define U=1+native reuse for
valid outcomes and U=0 for measured resource caps. This utility is not native
reuse fitness. Invalid-family, nonadvancing, timeout/error/panic or unsupported
outcomes cannot silently become constraint zero: save the returned receipt if
available, stop and retain the remaining NotRun schedule. No unknown work is
assigned a score or utility.

## Prospective schedule and budgets

Master seed `0xE656000001` = 989285318657. Fresh optimizer RNG master+10000.
Training emission seed master+5000+8*generation+pair, shared across +/− arms.
Training physics master+100000+(generation+pair)%4, shared across arms.
Generation/pair indices are zero-based; + arm runs before − arm. Champion is
the final generation-20 parameter vector, fixed before held-out physics.

Held-out: 512 emission draws × four physics seeds × three arms (champion,
zero-logit untrained, fresh random standard-normal 183-vector). Emission seed
master+2000+draw; random-parameter RNG master+3000+draw. Each random vector and
arm genotype is reused across the four physics seeds master+200000..200003.
Order: draw, then champion/untrained/random, then physics seed. No held-out
information enters training. Distribution-TV diagnostics use 64 fresh common
emission seeds master+1000..1063 and perform no Engine work.

Total: 320 training + 6,144 held-out = 6,464 evaluations, at most 646,400 native
attempts/steps. Ceiling: 3,600 seconds including setup/final checks, cooperative
30-second cell deadline, 2 GiB output, 128 MiB receipt reserve and 16 MiB final
reserve. One grammar rebuild. Existing node/edge caps 2,000/4,000 remain fixed.

## Registered analysis

The new primary comparison is paired **constraint utility** champion−untrained
over 512 genotype means (four physics seeds averaged per genotype/arm). Require
every scheduled outcome to be known valid or measured resource-cap, completed
20-generation training and full held-out schedule. New objective improvement
requires positive primary utility mean, exact one-sided paired sign p<.05,
and champion held-out feasible count at least untrained feasible count. This
last condition is a point-count guard, not statistical proof of noninferiority.
Ties are excluded only from the binomial denominator. Champion−random is secondary.
Utility averaging can trade reward against cap frequency across draws: the
per-candidate ordering is feasible-first, not a lexicographic ordering of
aggregate feasibility rates. Report feasibility rates separately, each /2,048.

Separately preserve the earlier **native reuse** analysis: full 512-draw means,
differences and sign p only when all 6,144 native evaluations are valid, otherwise
null; no complete-case substitution. Positive primary native mean plus p<.05
and completion is required for native reuse improvement. The new utility
criterion cannot be called a pass of that earlier criterion. Report native and
utility wins/losses/ties, exact fixed denominators, resource caps and all missing
outcomes separately. No pooling with older panels; no post-hoc seed substitution.

## Durability, audit and verification

Create all 6,464 NotRun slots before setup, freeze all native pins, the full
learning crate, protocol/config/checker/test dependencies and executed binary
before physics. Check live frozen identities after compilation, every training
generation, each 64 held-out draws and at completion; reverify parent at completion.
Save in-flight marker before native execution, terminal row atomically before
counter commits. A hard kill can leave unknown in-flight work; no per-step crash
journal or OS-preemptive deadline guarantee is claimed.

Development tests must cover one-capped and both-capped arms, all-capped and
all-tied generations, near-tie all-feasible rank/update compatibility, unsupported
outcome refusal, fixed schedule/null denominators, reconstruction of prior real
ES updates and detached metadata/native-receipt tampering. Build complete-feature
release CLI and check legacy feature isolation before launch.

The frozen streaming auditor verifies exact seals/source/binary/parent identities,
schedule, membership, native accounting/caps, optimizer score conversion, f32
candidate/update arithmetic, final champion and independent statistics. It uses
the exact pinned parent's native-accounting helpers. Run detached tamper controls,
save audit and reseal, then reverify. No postmeasurement checker edits may be
presented as a frozen-checker pass. Accounting reconstruction is not independent
physics/RNG replay or compiler-to-binary proof. No learned repair/maintenance,
online metadata evolution or long-horizon scaling claim follows from this run.
