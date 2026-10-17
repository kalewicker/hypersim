# Valid-production optimizer signal panel v1

Prospectively fixed on 2026-09-29 before new panel physics.
Profile: `E2-complete-valid-optimizer-signal-v1`.
The user requested this panel after the validated grammar build. This authorizes
one bounded optimizer diagnostic, not the reserved full training campaign.

## Frozen identities and schedule

- Accepted closure prerequisite: `ere_complete_coherent_closure_full_20260929`,
  root seal `fa3725f7cf5a03b207635d4c121d47280d38b4a05ae018ec3606e00086a4b60a`.
- Grammar: `E2-complete-valid-production-grammar-v1`, fixed 168 parents,
  13 operators, 183 logits and 18,956 legal productions. Engineering archive
  `ere_valid_production_grammar_20260929`, root seal
  `ccc22798899e426f93cd2801020fef0e93391e8790a23a214d22b625caa58891`,
  catalogue SHA256
  `ca02b55e5939ead818cdfce321c74b2f13c1ac7bde3c500061ca3ff3c402c8a6`.
- Rebuild the grammar once before physics and require exact catalogue hash
  equality. Verify the original sealed grammar archive and unchanged native,
  codec, coherent-kernel, grammar, fixture, parent-bank and optimizer sources.
  Freeze panel sources, binary, contract, auditor and prerequisite receipts
  before rebuilding/measurement. Compiler-domain refusals are not sampled
  evaluation failures; no fitness enters grammar compilation.
- Matcher `repeated-self-pair-feasibility-v1`; metadata invisible, isolated
  allocator, MaxParallel with at most one rewrite per step, pooled 400k/50k.
  Fixed ordinary world and native reuse objective remain the original panel's.
- Fresh master seed `0xE552390001`. Initial/common emission panel: master+1000
  through master+1063. Baseline emissions: master+2000 through master+2007.
  Physics seeds: 509, 613, 719, 827. Generation g/pair p emission seed:
  master+4000+4g+p, shared by plus/minus. Both arms use physics 509 and 613.
- 32 baseline attempts (8 genotypes x 4 physics seeds), 8 control attempts
  (active/dormant x 4 seeds), 80 ES attempts (5 generations x 4 pairs x 2
  arms x 2 seeds): exactly 120 scheduled attempts, horizon 100, ceiling
  12,000 attempted Engine steps. Sigma=1, alpha=.05, zero initial logits,
  master-seeded ChaCha8 perturbations and average ranks for tied fitness.
- Active control: grammar parent 4 / Activate / Proposal / production 0.
  Dormant control: grammar parent 5 / Deactivate / Proposal / production 0.
  Emission seed master+3000. Parent 5 has the same Core/Proposal bodies as
  parent 4, but starts active, so Deactivate is a changed legal production
  rather than the old grammar's dormant no-op. Final controls must differ
  only in activation; verify equal decoded alleles and typed ordinary world.

## Unchanged objective and acceptance

R=ln(1+max(fires-1,0))/ln(100). Only active Proposal events whose
priority-normalized canonical class differs from Core earn credit. Native
rewrite IDs supply counts; complete family validity is measured each step.
This frozen-family objective does not certify novelty, paid repair or learned
maintenance. Successful full-horizon zero-reuse genotypes score zero.

Retain all scheduled attempts. A rejected emission, matching/store cap,
invalid post-step family or nonadvancing step has null score, not parent/zero
fitness. Withhold both ES arms if either arm has any null evaluation. No
outcome-dependent retry, repair, replacement, seed change or branch selection.
Grammar/type mismatches and campaign resource violations stop the campaign
incomplete; no substitute run is launched.

P1: active forced signature is absent from the initial 64-emission panel.
P2: maximum common-seed executable-distribution TV across actual ES updates
is at least .05 **and all 20 antithetic pairs are valid**.
P3: mean paired active-minus-dormant gap exceeds twice the population SD of
all 32 baseline scores and 1e-12; a null baseline invalidates this check.
P4: baseline mean is more than .02 from both zero and one; null baselines
invalidate this check. Report within-genotype physics SD and between-genotype
SD separately, with fixed denominators. All four must pass; thresholds are
unchanged from the failed panel.

Wall ceiling 900 seconds including grammar rebuild and measurement after
source freezing; output ceiling 512 MiB. No larger optimizer/training campaign.
Archive every native attempt, production membership, perturbation, paired
score, theta update and distribution sample. The independent checker verifies
accounting, grammar membership, provenance, masks and decisions, with real
temporary-copy tamper tests and a completion seal. It is not a second native
physics replay or full G-R1 semantic assay. Even a pass retains
`training_admitted=false`, the 50k/50k runtime bridge and other admission limits.

## Implementation scope

Use a separate `signal-valid-complete` CLI and panel module. Do not change
the old `signal-complete` generator, its criteria, seeds or archived failure.
Verify all original optimizer, grammar and closure archive seals before and
after work. Run only this registered fresh panel, once, after targeted harness
tests. Report failures or unmeasured checks without changing their criteria.
