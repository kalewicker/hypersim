# selfmod v7 — the tie-break fix

## §0 What v7 is

v6's routing note (selfmod_v6_findings.md) identified
`Binding::canonical_key`'s `rule_id` term as an instrument bias: within a
priority class, every binding of the lower-`rule_id` rule precedes every
binding of the higher one, so coexisting equals never share contested nodes.
`rule_id` is decode order = `(class_hash, K_SEQ)` — content of the rule, not
merit of it. This contaminates every multi-rule result since v3, and makes
G5.3 structurally unable to measure fitness: a neutral clone (identical spec,
later K_SEQ) can never win a contested node, so "share ≥ 2× seed share" is
impossible by construction rather than informative about fitness.

Routing offered three options: fix hg-engine in place (ground rule 1 forbids),
implement conflict resolution locally in hg-selfmod, or break ties randomly.
v7 does the second and third: **a local resolver with selectable tie-break**
(`crates/hg-selfmod/src/resolve.rs`), hg-engine untouched.

## §1 The instrument, fixed and verified

- `TieBreak::RuleId` — byte-equivalent restatement of the engine order.
  Verified two ways: unit test `test_ruleid_mode_matches_engine` (same
  accepted set as `hg_engine::resolve_conflicts` on a mixed pool), and a
  full-run check: phaseB seed=42 churn3 flux 2000 steps produces
  **byte-identical** `run.csv`/`summary.txt` under the new binary in RuleId
  mode vs the committed v6 binary. G4.0 (v3 gates) passes unchanged.
- `TieBreak::Merit` — equal-priority bindings ordered by cumulative
  productive firings (RuleRoot-keyed, survives re-decode; newborns start 0).
- `TieBreak::Random` — equal-priority bindings ordered by a per-step keyed
  draw (`seed ^ step·φ`, fresh stream per step, `subsample()`'s RNG
  discipline: switching tie-break mode shifts zero draws of the main stream).

10 unit tests cover all three modes (lockout reproduced, work-sharing,
priority dominance, creation bindings, determinism, keyed-unbiasedness).

## §2 Pre-registration (written before the battery ran)

Predictions recorded in the working notes before any battery output was
read: null fails under ruleid (structural); null fails under merit
(newborn disadvantage — hazard #1 from resolve.rs's header); null PASSES
under random; guard passes under random. **Both random predictions were
wrong**, §3 says how.

## §3 Results

### §3.1 The battery (P0c/G5.3, 8 seeds, steps=2000, invasion@500)

| invader | tie_break | coeff | invades | grew | R3 reduced | verdict |
|---|---|---|---|---|---|---|
| clone | ruleid | 0 | **0/8** | 0/8 | 0/8 | structural, as known |
| clone | merit | 0 | **0/8** | 0/8 | 0/8 | newborn disadvantage realized |
| clone | random | 0 | **0/8** | 0/8 | 2/8 | see §3.3 |
| guard θ=.1 | ruleid | 0 | **8/8** | 8/8 | 8/8 | ARTIFACT, see §3.2 |
| guard θ=.1 | merit | 0 | **0/8** | 0/8 | 0/8 | incumbent lockout |
| guard θ=.1 | random | 0 | **4/8** | 4/8 | 7/8 | first real signal, §3.3 |
| clone | random | .0005 | 1/8 | 1/8 | 4/8 | confounded by collapse |
| guard θ=.1 | random | .0005 | 3/8 | 3/8 | 4/8 | ditto |

### §3.2 The v5/v6 "invasion" was queue position, not fitness

`orderprobe`: `guard energy>0.1` has class_hash 11474396747993513293 <
R3's 15603166327473374834, so under `ruleid` the invader had FIRST claim on
every contested node. The battery shows what that means: under ruleid the
guard "invades" 8/8 and reduces R3 8/8 — including seeds where its guard
selectivity buys it nothing. It wins because it sits ahead of R3 in the
queue, full stop. Every multi-rule reading since v3 inherits this artifact;
G6.2's substitution scoring partially masked it (substitution removes the
queue interaction), which is why the construction gate passed while the
coresidence gate failed.

Equally diagnostic: under ruleid AND merit, the **neutral clone is a pure
spectator** — R3's per-seed copy counts are identical with and without the
clone present (e.g. 4/4, 5/5, 3/3 …). A co-resident rule that cannot
perturb the system even marginally is not competing; the gate was measuring
nothing.

### §3.3 Random restores competition — and falsifies the null's premise

Under `random`, R3's trajectories genuinely differ between invader and
control arms (real contention), and the guard-vs-clone contrast appears:
**guard establishes 4/8, clone 0/8**, same instrument, same conditions,
only the efficiency edge differing. That contrast is the fitness signal
G5.3 was built to detect — smaller than the old 6/8 bar, but it is signal
rather than artifact.

Why the neutral clone fails even under a fair coin: the gate demands a
single copy grow to ≥2× its seed share within 1500 steps. For a TRUE
neutral, absorption at zero is the dominant outcome from n=1 — expecting
6/8 growth from neutrality was never a calibrated null. v5/v6 read the
null's failure as pathology; part of it is the criterion demanding growth
where neutral theory predicts extinction. **The null needs recalibration
(establishment probability vs the seed-share baseline), not just the
resolver.** Recorded as v8 work.

### §3.4 Survival side-effect: merit stabilizes, random destabilizes

Bare coexistence (churn3, flux, proportional income, coeff=5e-4, 2000
steps, 8 seeds):

| tie_break | survived | final rule copies (per seed 1,2,3,7,42,99,123,777) |
|---|---|---|
| ruleid | 8/8 | 12,10,9,9,10,11,9,9 (mean 9.9) |
| merit | **8/8** | 12,**12**,**12**,**13**,10,11,**12**,**12** (mean 11.75) |
| random | **3/8** | 10,12,0,9,0,0,0,0 |

Merit dominates ruleid per-seed (never fewer lineages, +2 on five seeds):
work-sharing keeps weak-but-viable rules above the starvation floor.
Keyed-random collapses outright on 5/8 seeds — fair shares are thin shares,
and thin shares starve. At coeff=0 the instability persists (battery control
arms: 5/8 dead by step 2000), so this is not a pricing artifact.

### §3.5 Verdict on the routing options

- **Cumulative merit**: refuted as the invasion fix — its own documented
  newborn-disadvantage hazard fires (a seeded variant can never outrank an
  incumbent's history), and for identical specs it degenerates toward
  decode order. But it is strictly the best coexistence regulator measured
  (§3.4): more lineages than ruleid on every seed, no collapses.
- **Keyed-random**: the only mode in which contested nodes are actually
  contested and fitness-like differences convert into establishment
  differences (4/8 vs 0/8). Its cost is population stability.
- The two failures have opposite signs: merit freezes the hierarchy,
  random dissolves it. Neither endpoint is the right instrument alone.

## §4 Routing for v8

1. **Recalibrate the null before touching the resolver again.** Estimate
   p_establish(clone) under keyed-random over many seeds; compare the
   guard's p_establish against it binomially. The claim to test becomes
   "p_est(guard) > p_est(neutral)", not "share doubles in 6/8 seeds".
2. **Stabilize randomness with merit, not instead of it.** The natural
   hybrid — randomize WITHIN near-equal merit, letting accumulated history
   matter only in the small — keeps newborns competitive (they tie at 0
   with other low-merit rules and randomize among them) while preserving
   merit's demonstrated stability. Predicted (pre-registered here): hybrid
   survives 8/8 like merit, keeps the guard-vs-clone contrast like random.
3. **Diagnose the random-collapse mechanism** (income variance → starvation
   cascade?) before trusting long-horizon random-arm runs; if the hybrid of
   (2) removes the collapses, that is evidence for the mechanism.
4. Only then re-run G6.2 substitution scoring under the chosen resolver and
   revisit v6's cap×budget conditioning with an instrument that no longer
   hands the queue to hash luck.

## Provenance

- Resolver + tests: `crates/hg-selfmod/src/resolve.rs` (10 tests, all pass);
  wiring in `runner.rs` (`tie_break` config, `merit_by_root` accumulation),
  CLI `--tie_break={ruleid,merit,random}`.
- Byte-equivalence: new binary vs committed v6 binary, phaseB seed=42 churn3
  flux 2000 steps → identical run.csv + summary.txt; G4.0 gate suite passes.
- Battery: `p0c1 --invader_kind={clone,guard} --invader_theta=0.1
  --tie_break={ruleid,merit,random} [--match_cost_coeff=0.0005]
  --steps=2000 --invasion_step=500`, seeds P0A_SEEDS[0..8].
- Survival sweep: phaseB churn3 flux proportional coeff=5e-4 mu=0, 8 seeds ×
  3 modes.
