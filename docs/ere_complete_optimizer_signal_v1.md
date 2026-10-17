# Complete-profile optimizer signal diagnostic v1

Preregistered before new diagnostic physics on 2026-09-29. This is an
engineering/optimizer precheck, not a replacement G-R1 assay or training admission.

## Frozen design

- Profile: `E2-complete-optimizer-signal-v1`; complete family version 10.
- Prerequisite: accepted `ere_complete_coherent_closure_full_20260929` receipt,
  with sealed manifest, classification, run status and full audit checked before launch.
- Use the coherent atomic kernel that passed closure; copied without transaction
  changes into sim-core, and regression-compared against the assay helper.
- Matcher: opt-in `repeated-self-pair-feasibility-v1`, pooled 400,000 work /
  50,000 results, MaxParallel, maximum one rewrite per step, metadata invisible.
- Fixed common ordinary world: 64 label-0 nodes (property 1 = Int(100)),
  96 binary ring edges, one ternary edge and one arity-8 repeated edge.
  Metadata-isolated allocator; no genotype-dependent ordinary fixtures.
- Grammar: 183 finite logits: parent (168), operator (13), role (2).
  Independent stable softmax draws; selectors uniformly drawn from kernel target
  lists using the same ChaCha8 emission RNG. One edit, no retries or repair.
  Activation operators ignore role. Rejected staged material is archived and
  receives null fitness, never its parent's fitness.
  After all three categorical draws, selectors always consume draws for
  pattern, variable, second variable, side, slot of the selected pattern and
  label value, including fields unused by the selected operator.
- Master seed `0xE452390001`. Emission comparison panel: master+1000 through
  master+1063. Baseline emission seeds: master+2000 through master+2007.
  Physics seeds: 101, 211, 307, 401. Forced controls: parent 4 + Activate and
  parent 4 + Deactivate, Proposal role, selector seed master+3000.
- Baseline: 8 emissions x 4 physics seeds = 32 attempts. Forced controls:
  2 x 4 = 8. ES: 5 generations x 4 antithetic pairs x 2 arms x 2 physics
  seeds (101,211) = 80 attempts. Exactly 120 scheduled rollouts, horizon 100;
  at most 12,000 attempted Engine steps. All attempts remain in denominators.
  Generation g, pair p emission seed is master+4000+4g+p, shared by both arms.
- ES sigma=1, alpha=.05, zero initial logits, master-seeded ChaCha8 perturbations.
  Both arms withheld if either arm has any null fitness. Equal scores receive
  average ranks; flat-fitness movement is an implementation failure.
- Reuse objective R: only active Proposal rewrites whose priority-normalized
  canonical bytes differ from Core count. R=ln(1+max(fires-1,0))/ln(100).
  A successful full-horizon genotype with no such rewrites has score zero.
  Frozen metadata means maintenance is measured but not learned here; no paid
  editors, novelty or damage-resilience claims.
- Matching caps, >2000 nodes, >4000 edges, invalid post-step complete family or
  nonadvancing steps terminate that rollout with null fitness. No replacements.
  Campaign wall ceiling 900s; output ceiling 512MiB. Stop entire campaign on
  resource ceiling and leave incomplete status; do not replace failed attempts.

## Checks and decision

P1: forced active control has an executable signature absent from the initial
64-emission panel. P2: maximum common-seed empirical executable-distribution TV
from initial theta across actual ES updates is at least .05, and all 20 pairs
are valid. P3: mean paired active-minus-dormant score exceeds twice population
SD of all 32 initial baseline scores and 1e-12; baseline must be complete.
Report within-genotype physics SD and between-genotype SD separately.
P4: mean initial baseline score is more than .02 from both 0 and 1.

All four must pass to support this specific bounded frozen-family optimizer
precheck. Any failure is evidence to diagnose, not permission to tune or rerun
within this campaign. Even success leaves `training_admitted=false`: complete
profile runtime/admission compatibility and longer-horizon evidence are separate.
Archive exact emissions/receipts, step rule IDs, cap/validity accounting, scores,
all perturbations and theta updates. Independently recompute objective, pair
withholding, tied ranks, updates, distributions, denominators and checks.
