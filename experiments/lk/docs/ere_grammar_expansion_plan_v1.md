# Grammar Expansion Without Blobs — Sub-track Plan v1 (GE)

**Status:** specification + pre-registration. No code written against this
document yet.
**Scope:** how the rule grammar (TOML-side and endogenous-side alike) may be
extended past the shapes characterized by the A–E alphabet arc without
re-triggering the growth pathologies on record. Defines two gates (**G-GE1**
growth-bias audit, **G-GE2** blob-panel engagement), four failure modes
(**FM-GE1..4**), and standing policies that bind every future grammar change.
**Prefix discipline:** `G-R*` through `G-R6` are taken by the ERE plan; this
sub-track uses `G-GE#` / `FM-GE#` / `SB-GE#` so quarantine histograms and
greps never collide.
**Relation to the main plan:** this document constrains *how R5's mutation
kernel and any future expressivity addition may ship*. It modifies nothing in
plan v1; where the two touch (R5 stateability prechecks, §6 G-R1 thresholds),
it defers.

---

## 0. Version history

| Version | Change |
|---|---|
| v1 | Initial specification. Blob incident register, conservation-coupling constraint, G-GE1/G-GE2 pre-registrations, standing policies. |
| v1.1 | Reconciled against repo state: R5 envelope widening (E1-V, E2-lite, six-op E1-G) already landed; G-R2 executed at rule granularity (20/41 = 48.8%); EXP-R06/R07 complete (234 shapes, LINEAR 8/8 seeds). Scope reworded to bind both the landed widenings' characterization and future additions. |
| v1.2 | G-GE1 executed (`88ac32a`): DESCOPE at f_hot = 0.1419, eight operators REQUIRES-COUPLING, SB-GE3 recorded. §4.3 amended with the branch taken — DESCOPE-via-fees (GE-C1(b)), not exclusion; §10 added: EXP-R09 kernel amendment (fee semantics, control arms, pre-registration). SB-GE4 owned (fixture-specificity of the hot-rate table). |
| v1.3 | SB-GE3(b) landed (kernel drops LHS-orphaned guards in slot ops; no V10 per FM-GE2, ratified in `docs/ere_validators.md`); G-GE1 re-run: DESCOPE f_hot = 0.1449, same eight descoped, non-slot rows byte-identical. SB-GE1 closed as formality (`late` all-zero ×4 runs). SB-GE2 standing watch continues (0.005 below PASS). |

---

## 1. Motivation — what the arc established, and what it costs

The A–E tracks proved the architectural claim: every statically expressible
rule shape saturates into a small recurrent event alphabet, including the one
shape the grammar could not previously say (`rebalance`, byte-identical
alphabet at 1×/4×/16×, converging to conservation on 10-of-8000 steps). The
grammar's output space is closed, therefore open it — that is the license for
ERE and for further expressivity work.

The license comes with a liability the arc equally well documented: the
substrate has *demonstrated*, repeatedly, what happens when expressive power
arrives without its own braking. This document exists so the next grammar
change buys alphabet breadth without buying back those incidents.

**Where the track actually stands (v1.1 reconciliation).** Expansion is no
longer hypothetical: E1-V (`vars_distinct`), E2-lite (effects), and the six-op
E1-G guard vocabulary have landed as envelope widenings; G-R2 executed at
RULE granularity — 20 of 41 shipped rules round-trip, **expressible fraction
48.8%**; and EXP-R06/R07 established the kernel-reach baseline (479 RCF
classes / 234 distinct shapes from 2 seeds; new-shape yield LINEAR,
flat-to-slightly-declining quartiles, 0/8 accelerating across seeds). The
outer loop (EXP-R09) is queued against that null. This sub-track therefore
binds two things: characterizing the widenings already shipped against the
incident register, and gating every future verb the same way.

A direct connection between EXP-R07 and blob risk deserves stating: the R6
regime **installs every valid child** (no selection pressure yet), so region
count and total region footprint grow monotonically at the valid rate. That
is controlled growth by construction today, but it means the population-side
blob channel is armed the moment R5 adds fitness ranking — selection
concentrating mass on whatever region shape reproduces fastest is precisely
the B1/B3 dynamic wearing evolutionary clothes. The panel of §5 reads on both
sides of the boundary (meta-included metrics), which is why it must be
primary *before* EXP-R09 opens, not after.

**Target signature, stated once so every threshold below has a referent:**
`bounded_churn` post-burn-in — 12 raw symbols, A_eff 3.57, zero singletons,
top-3 mass 95.2%, PLATEAU novelty on both abscissae, flat node count. The
opposite of a blob is not stasis; it is sustained turnover inside a fixed
recurrent alphabet.

---

## 2. Incident register — the blobs on record

Every expansion proposal is checked against this table. Each species was
caught by a different instrument, which is why §5 demands a panel rather than
one metric.

| # | Incident | Species | Mechanism | Containment on record | Source |
|---|---|---|---|---|---|
| B1 | `binary_edge`: 36 → 27 628 nodes in 22 steps | Size (node) | node-minting × MaxParallel | none in zoo; classified Exploding | atrack_findings A4.1 |
| B2 | `triangle_subdivide`: 36 → 396+, fresh label-0 structure minted | Size (node) + structure | subdivision with net-positive create side | none; excluded from live series | experiment5_preregistration §7 |
| B3 | selfmod v3 `add_rhs_edge` mutants: 18k+ edges by step 2000 | Size (edge-only) | store does not dedupe identical edges; node clamp cannot bind edge-only growth | `edge_ceiling = 1000` gating positive-net-delta rules | selfmod_v3_findings Excursion 1 |
| B4 | `excite` floods crystal to label-1 in ~10 steps | State undifferentiation | monotone label propagation; size fine, diversity dead | avoided only by not seeding; extinguishes influx/churn | btrack_findings B1.1 |
| B5 | LK metered rules: serial point-source pump | Concentration | metering blocked size-blob, bred single-site pump | instrument exists (`meta_rewrite_site_entropy`); no cure | plan v1 §8 M2; attractor_writeup |
| B6 | `growth_treadmill` / `tetrahedral`: 100% singletons, A_eff = \|A\| | Alphabet pseudo-blob | maximal raw novelty, zero recurrence — serial numbers | measurement trap, not defect; A_eff neutralizes | atrack_findings A3.3 |

Two structural facts fall out, and they organize everything downstream:

1. **Size-blob containment has never been load-bearing.** Every size incident
   was caught after the fact or by exclusion from the zoo; the only armed
   containments (B3's edge ceiling, the v2/v3 node clamp) were retrofits.
2. **Metering trades species rather than curing.** B5 shows pressure against
   size re-emerges as concentration. Any expansion that "can't blob because
   it's metered" is claiming a trade, not an immunity, and must ship the
   concentration instrument.

---

## 3. The design constraint: no unconstrained creation verb

> **GE-C1 (binding on all future grammar changes).** Every new expressive
> power that can increase substrate extent (nodes, edges, region footprint) or
> collapse substrate state (monotone label/property propagation) ships with a
> paired brake of one of exactly three kinds:
>
> - **(a) Conservation coupling** — creation is structurally conditioned on a
>   counted consumption elsewhere (var-to-var quantity guards, degree-balance
>   constraints, E1-G guards over conserved tokens). Precedent: `rebalance`
>   converged to conservation unprompted; effect-gated rewards starve sterile
>   churn.
> - **(b) Fee-and-threshold economics** — the verb fires only against a
>   conserved budget sized so that unbounded compounding is arithmetically
>   impossible (income = min(reward, free) discipline; asymmetric bootstrap
>   thresholds per Experiment 5).
> - **(c) Armed ceiling + attribution** — hard cap, logged binding/non-binding
>   every run, with the FM-R3 rule in force: a result obtained under a binding
>   ceiling is a result about the ceiling. Precedent: `edge_ceiling`, V7/V9
>   budgets.
>
> Option (c) alone is permitted only for verbs whose EXPANSIONARY rate under
> G-GE1 measures ≤ 0.05; anything hotter must pair (a) or (b).

The rebalance result is why (a) is listed first and preferred: the arc's
sharpest fact is that self-balancing shapes carry their own brake *by
construction*, not by parameter tuning.

---

## 4. G-GE1 — mutation-kernel growth-bias audit (pre-registration)

**Committed before any number exists.** This gate is the weekend-class,
highest-information-per-compute move, in the plan's own tradition (§1.4):
if the R5 kernel is systematically expansionary, runtime clamps cannot save
the campaign, and that is discoverable offline.

### 4.1 Instrument identity

G-GE1 is **G-R1's harness plus four columns**, run on the *same* sampling
distribution (uniform parent over the seed set, uniform operator via
`hg-meta/src/mutate.rs::mutate`). Nothing in the existing G-R1 classification
or thresholds changes; the added columns are purely additive instrumentation,
so the published G-R1 result remains authoritative and untouched.

New columns per draw:

| Column | Definition |
|---|---|
| `static_dn` | net RHS − LHS node count of the compiled child |
| `static_de` | net RHS − LHS edge count (parallel instances counted, since the store does not dedupe — B3) |
| `dyn_n_traj` | node count trajectory firing the child on its parent's fixture graph, W = 100 steps (same window as G-R1's Catastrophic class) |
| `dyn_class` | see 4.2 |

Engine access follows the G-R1/G-R2 precedent (harness lives under `tests/`
with the engine as a dev-dependency; `hg-meta` the library keeps its
core+rules-only discipline).

### 4.2 Classification (pre-committed)

Per valid mutant, from `dyn_n_traj` (and `dyn_e_traj`, recorded identically):

| Class | Definition |
|---|---|
| `BALANCED` | returns to ±5% of baseline node count within W |
| `SATURATING` | grows then plateaus: last 20 steps within ±1% of step-W−20 value |
| `EXPANSIONARY` | neither of the above — monotone growth at window end |
| `REGIME_EXPLODING` | enters the evaluator's `Exploding` regime within W |

Per operator o: `expansion_rate(o)` = (EXPANSIONARY ∪ REGIME_EXPLODING) / valid
draws of o. Envelope statistic: `f_hot` = same fraction over the whole valid
ensemble.

### 4.3 Committed thresholds and interpretations

| Outcome | Verdict | Action |
|---|---|---|
| `f_hot ≤ 0.15` and max_o `expansion_rate(o) ≤ 0.10` | **PASS** | R5 proceeds as planned |
| `f_hot ≤ 0.35` but some operator hot | **DESCOPE** | hot operators excluded from the R5 kernel (or gated behind GE-C1(b) fees); descoping recorded as a finding about the encoding, not a parameter choice |
| `f_hot > 0.35` | **REDESIGN** | kernel redesigned before R5 opens; mirrors G-R1's own ladder logic (below Valid = 0.10 redesign, 0.10–0.40 descope) |

> **Amendment v1.2 — the branch taken.** G-GE1 executed at DESCOPE
>(f_hot = 0.1419; findings in `docs/gge1_findings.md`, commit `88ac32a`).
> The descope is implemented as **GE-C1(b) fee-gating**, not exclusion.
> Rationale: the hot-rate table is fixture-relative (measured on one
> 8-node world), while unrestrained firing under MaxParallel — the thing
> that actually went REGIME_EXPLODING — is substrate-independent;
> exclusion would bake one world's ranking into the kernel permanently and
> collapse the search space below EXP-R07's demonstrated reach, whereas a
> fee prices the phenomenon itself and keeps FM-GE3's pre-registered
> failure mode live. Semantics and control arms are specified in §10.

Additionally, independent of the envelope verdict: any operator with
`expansion_rate(o) > 0.10` is flagged **REQUIRES-COUPLING** and may not ship
in any future kernel without a GE-C1(a)/(b) pairing. Operators with
`expansion_rate(o) ≤ 0.05` may ship under GE-C1(c).

### 4.4 Fixtures and scale

Seed set: the G-R1 set (≥20 rules) extended with the live members of the
shipped library identified by B-track B1.1 (the simultaneously live,
perpetual, node-neutral quartet plus their parents). Draws: 10⁴, keyed
ChaCha stream derived from run seed + draw index (never the engine stream),
identical derivation to `epoch.rs`. Raw output archived under `runs/ge/gge1/`;
one CSV + one findings document; manifest.json per the campaign-output
discipline.

### 4.5 What would falsify the framing

If `f_hot` lands low but R5 campaigns still blob, the kernel distribution was
not the operative variable (composition effects across epochs are), and
G-GE1 must be re-scoped from per-draw to per-epoch-sequence statistics. That
outcome is a finding about where growth bias lives, reported as such.

---

## 5. G-GE2 — the blob panel as primary instrumentation (pre-registration)

For every R5+ run and every future grammar-expansion characterization run, the
following panel is computed, reported, and treated as **primary outcome**, not
diagnostics. Each row cites the incident it exists to catch.

| Instrument | Catches | Committed form |
|---|---|---|
| A_eff trajectory (+ raw \|A\| second witness) | B6 serial-number alphabets; raw-vs-effective split per B-track | S1 signatures at the rule's pre-registered radius; both reported, verdicts on A_eff |
| Recurrent-symbol fraction & singleton share | B6; distinguishes bounded_churn from treadmill | post-burn-in only |
| Novelty curve vs STEP **and** vs EVENT | binary_edge illusion (growth that was event-rate, not alphabet) | growth classifier with fit quality; both abscissae mandatory |
| Burn-in window before scoring | 80%-transient inflation of churn's whole-run count | burn-in = first 10% of steps, M1-mandated |
| `meta_rewrite_site_entropy` | B5 concentration/pump | pre-committed interpretation inherited verbatim from plan §8 M2: entropy → 0 with high meta_rewrite_events = pump recurring at meta level, a finding about energy metering, not encoding |
| Dual meta-excluded/meta-included metrics | engagement failure (region growth masquerading as substrate novelty) | plan §8 M1, both sets always reported |
| Regime classifier incl. `Exploding` early-abort | B1/B2 | abort wired into campaign loops (stage-0 gate exists; reuse, do not reimplement) |
| Edge-count trajectory alongside node-count | B3 edge-only growth invisible to node panels | static_de/dyn columns of G-GE1 carried into run metrics |

Engagement proof (G-GE2 exit): each panel member demonstrated firing on at
least one synthetic or archived case — the A-track calibration standard. An
instrument that cannot reject cannot support a verdict. Archived incidents
B1–B3 serve as natural firing fixtures where replayable.

EXIT DECLARED 2026-09-04: proof table + first panel light in
`docs/gge2_findings.md` (R09-lineage emission ×2 byte-identical);
wiring in `docs/gge2_audit.md` slice 3.

---

## 6. Standing policies (bind every future grammar change)

1. **Ceilings log binding/non-binding every run.** A binding ceiling during a
   reported result voids the result's downstream claims (FM-R3, now applied at
   the meta level). Zeros are reported, never omitted — same convention as the
   V1–V9 counters.
2. **Parallel-edge semantics decided before any edge-minting verb ships.**
   Either the store learns to dedupe identical edges or the `edge_ceiling`
   analogue arms permanently; shipping an edge-minting verb against the
   current store is how B3 recurs.
3. **No damping argument without checking resolver semantics first.**
   MaxParallel conflict resolution is NODE-disjoint (LK finding, third
   instance of the concurrency-assumption error class); any claim that
   parallelism slows a blob must cite the disjointness semantics actually in
   force.
4. **Both abscissae, always.** No novelty or alphabet claim from step-indexed
   curves alone.
5. **Newborn rules start throttled.** Meta-level analogue of Experiment 5's
   asymmetric bootstrap: a freshly installed mutant region does not fire at
   full reward until it survives k epochs or pays an install fee from conserved
   tokens. Prevents the excite failure mode (B4) wearing endogenous clothes —
   a hot newborn minter flooding the graph inside one epoch.

---

## 7. Failure modes (pre-registered)

| Id | Failure mode | Pre-committed interpretation if observed |
|---|---|---|
| FM-GE1 | Kernel passes G-GE1 offline; campaigns still blob | composition effect across epochs; re-scope to sequence statistics (see 4.5) — a finding about where growth bias lives |
| FM-GE2 | Panel instruments never fire across a full R5 campaign | either the couplings work (report as success with the engagement fixtures as evidence) or the panel is dead weight (report honestly which, citing firing fixtures) |
| FM-GE3 | Fees/throttles (GE-C1b, policy 5) suppress novelty instead of blobs | report the novelty suppression curve as the result; descope fee magnitude, do not silently tune |
| FM-GE4 | Coupled verbs (GE-C1a) prove unsearchable — mutants break coupling too often | mirror G-R1's ladder: coupling redesigned, not training loop tuned |

---

## 8. Owned gaps in this document

Recorded per the SB convention, amendments land HERE and in code together.

- **SB-GE1 (CLOSED as formality 2026-09-04).** §4's dynamic probe window (W = 100) is inherited from G-R1's
  Catastrophic class without independent justification. A slow blob that
  outlives the window reads as SATURATING. Mitigation: SATURATING verdicts on
  boundary (plateau detected late) are flagged for a one-off 16× horizon
  confirmation, mirroring the C-track's 16× discipline. Four gate runs,
  `late` all-zero every time — mitigation never triggered; horizon stands
  for this kernel, reopened by any future nonzero `late`.
- **SB-GE2.** The seed-set extension in 4.4 mixes rule provenances (G-R1 set +
  zoo-live quartet + parents); the growth-bias estimate is therefore
  population-dependent in a way G-R1's neutrality estimate was not. Accepted
  for v1 because R5's actual parents will come from exactly this kind of mix;
  revisit if G-GE1's verdict sits within 0.05 of a threshold boundary.
- **SB-GE4 (opened by execution).** The hot-rate table is measured on ONE
  fixture world (`seed_world_fixed(8)`, labels 0..3); the operator ranking is
  therefore substrate-relative — `clear_label` is hottest there BECAUSE
  labels do the constraining there. Consequence adopted in v1.2: the descope
  must not implement as permanent exclusion (which would freeze this table
  into the kernel), but as pricing of the fixture-independent phenomenon
  (unrestrained firing), per §10. Any future exclusion decision requires the
  audit re-run on at least one more substrate first.
- **SB-GE5 (opened by §10.1 wiring, commit `b10441e`).** The engine
  `RuleSelectionPolicy` seam is consulted once per step, with a snapshot of
  the per-region ledger state. The actual `try_fire` debit (per §10.1) is
  end-of-epoch — the policy cannot see a bankrupt region become bankrupt
  *mid-window*. The integration test pins this: a 50-step A1 run with a
  bankrupt ledger withholds everything; a 50-step A1 run with a solvent
  ledger produces the same fire count as no policy at all. Both are
  correct given the forward/debit split. Consequence: any future change to
  the engine's step loop that re-consults mid-step must either re-walk the
  policy or move the debit to the same boundary. The `epoch.rs`
  epoch_transition is the natural place to take the per-region debit; that
  is R5+ wiring, not R3. Amendment belongs HERE and in
  `epoch.rs`/`runner.rs` together.
- **SB-GE5 (RESOLVED 2026-09-03).** `epoch_transition_with_economy` +
  `EpochEconomy` + `EconomyAudit` + `FireCounter` landed in
  `crates/hg-meta/src/epoch.rs` (6 tests): end-of-epoch debit in bulk
  `try_fire` semantics, newborn ledgers opened post-aging at age 0,
  income/aging for survivors, extinct dropped with issuance tightened,
  per-epoch audit (`balanced` = bounds + Σ totals == issuance).
  Attribution flows through the observer chain because
  `StepStats.accepted_rewrites` is a step total — the engine was NOT
  changed. `epoch_transition` (all four existing callers) delegates with
  no economy and is behavior-identical; FEE_ZERO rides along inert
  (pinned by test). Driver work DONE 2026-09-03 (EXP-R09 runs 1–2):
  `rule_id → region root` mapped through consultation order
  (`map_fired_to_roots`), counts fed in; see SB-GE7 for the verdict.
- **SB-GE6 (opened AND closed 2026-09-03 by EXP-R04's `exp_r04_a2`
  failure).** The A2 protection gate injected its label-mirror guards
  under RAW var indices while `candidate_to_rule` interns vars in
  first-encounter order — the reader's RHS-before-LHS content sort returns
  rule 0 as `lhs [[0,2]]`, interning original var 2 as `v1`, so the gate's
  guard on `v2` named a var absent from `lhs.nodes` and `compile_rules`
  rejected the seed read-back (`InvalidGuard("guard var 'v2' not in
  LHS")`). Fix: `compile::interned_names` shared by compiler and gate;
  raw-index naming must never recur at any compile-adjacent site. Pinned
  by `seed_world_readback_compiles_under_both_arms`; EXP-R04 A2 passes
  40/40 (29 regions alive). Sibling note: this was NOT the install/reader
  round-trip — install→read is semantics-preserving up to renaming; the
  bug was purely the gate's naming assumption. G-GE1 unaffected (probe
  uses bare `candidate_to_rule`; third re-run byte-identical).
- **SB-GE7 (opened 2026-09-03 by EXP-R09 runs 1–2,
  `docs/ere_r09_findings.md`).** Outer-loop blob regime undemonstrated:
  A0 saturates (max_n 100; 4002 node-neutral seed-rule firings, children
  dormant) — the fee was never binding, so A1-vs-A0 suppression is
  unmeasurable in this harness although the throttle itself works (firing
  4002 → 7, audits 40/40 balanced). The A0 gate-assert is converted to a
  reported control verdict (run-1 firing on record); no harness tuning to
  chase the control. Next in order: (a) mechanism test — a known-exploding
  G-GE1 pair installed as a live region, A0-vs-A1 windows; (b)
  emergence-first loop conditions. A2 interim: 31 installs, 14 shapes,
  saturating — exclusion safe and fertile at this scale. Run 1 also owned
  a driver bug: `XorShift(seed | 1)` collided A0/A1 seeds (`0x90A0|1 ==
  0x90A1|1`); run 2 uses odd distinct seeds.
- **SB-GE7(a) (RESOLVED 2026-09-03 — redesign (i) landed,
  `docs/ere_r09_findings.md`).** Mechanism test had FALSIFIED the
  snapshot-per-window throttle (A1 still RegimeExploding, window-ends
  [70, 70, 70, 120] — whole-window gating, nothing priced at the
  margin). Fix: `FeePolicy` projects within-window spend (forward while
  one more firing fits the window-start balance; `forwarded` counts +
  `withheld_total`; one policy object per window). Boundary debit now
  reconciles actuals (paid == accepted by construction). Falsifier
  GREEN: A1 Saturating max_n 24, firing 200 → 4, window-ends
  [21, 22, 23, 24]. R09 re-run: A1 paid 5/refused 0 exact, trajectory
  unchanged (SB-GE7 outer-loop verdict stands). Amended: SB-GE5 split
  doc, solvent integration pin (now caps at endowment).
- **SB-GE7(b) (CLOSED 2026-09-04 as a failed attempt,
  `docs/ere_r09_findings.md`).** One-shot A0-only emergence search
  (`LoopConfig` + `run_arm_with`, `sb_ge7b_emergence_search`): all four
  cells Saturating, no emergence. C1 seed-contingency (accepted 2 vs
  slice-3 4002 — firing volume is a kernel-seed lottery); C2 novelty
  went the wrong way (4 shapes, recurrence 0.983); C3 compounded
  nothing; C4 went extinct at epoch 19 — the unprotected band is
  non-viable, not explosive (surface and persistence are the same
  knob; direction closed permanently). Dormancy is structural:
  variation quality, not parameters, is the exposed layer. R5 proceeds
  on the reference loop panel-primary with fee-efficacy unmeasured
  (mech falsifier stands as the fee-efficacy evidence).

---

## 9. Execution order

1. **G-GE1** (offline, CPU-hours): extend G-R1 harness, run 10⁴ draws, publish
   `runs/ge/gge1/` + findings doc. Cheapest gate first; blocks R5 opening.
   Blocks the **outer loop (EXP-R09)** specifically — the envelope-widening
   commits already landed are characterized through this gate too, since their
   operators enter the same kernel surface.
2. **G-GE2** (fixture work): wire the panel into the runner's metrics rows,
   demonstrate firing per instrument, archive proofs.
3. Then, and only then: R5 outer loop with the (possibly descoped) kernel,
   panel primary, policies 1–5 in force. Its committed null to beat is the
   EXP-R07 linear kernel-reach curve; any blob-panel excursion during R5 is
   reported against the incident register (§2) as a species recurrence or a
   new species, never averaged away. (RAN 2026-09-04 —
   `docs/ere_r5_findings.md`: priced kernel linear at ~40% of null,
   FM-GE3 silent, exclusion saturates, no excursions.)

Ordering rationale is the plan's own: the highest-information checks are the
cheapest, and a bad kernel discovered after a 40-run sweep is months wasted.

---

## 10. EXP-R09 kernel amendment — fee-gated firing economy (pre-registration)

**Committed before any EXP-R09 number exists.** This section implements the
§4.3 DESCOPE branch as GE-C1(b) fee-gating. It binds the OUTER LOOP only:
G-GE1's offline harness never draws fees, so its published numbers stay
frozen.

### 10.1 The translation step (why fees attach to firings, not to sampling)

The eight hot OPERATORS act at epoch boundaries; what went REGIME_EXPLODING
in G-GE1 was never the mutation itself but **the child region's subsequent
firings** under MaxParallel. So the fee attaches to firing economics, not to
sampling:

- All ten operators remain in the sampling kernel unchanged. The G-GE1
  distribution stays reproducible; nothing about *which* mutants exist is
  altered.
- Every region installed through a hot operator joins the conserved-token
  economy (selfmod v3/v4 machinery: income = min(reward, free),
  effect-gated so sterile delete-and-recreate pays nothing).
- A firing whose region cannot pay is REFUSED whole — the rule goes dormant
  that step; no partial execution, no debt. The region survives; bankruptcy
  shows up as dormancy, not death.
- Per-epoch conservation `free + Σ tokens == total` asserted every epoch,
  residual = 0, same audit discipline as selfmod v4.

### 10.2 Newborn throttle (standing policy 5, made concrete)

A freshly installed mutant region starts with endowment `E_0` tokens and
does not reach full income until it has survived `k_throttle` epochs
(pre-committed starting point, subject to the sweep: E_0 = 3 × mean
per-epoch firing cost of its parent class, k_throttle = 2). This is the
meta-level analogue of Experiment 5's asymmetric bootstrap threshold:
expensive to start, cheap to continue — turnover without guaranteed growth.

### 10.3 Control arms (all pre-registered; none optional)

| Arm | Configuration | Purpose | Committed reading |
|---|---|---|---|
| **A0 fee-zero** | fee = 0, throttle off, all 10 ops live | validates the economy is load-bearing | MUST reproduce the G-GE1 blob regime: hot-operator descendants explode again. If it doesn't, the fee was never binding and EXP-R09's main arms are uninterpretable — stop and report |
| **A1 main** | fees + throttle, all 10 ops live | the experiment | primary; panel of §5 read as primary outcome |
| **A2 exclusion-shadow** | fees OFF, the 8 hot operators REMOVED from sampling | the road not taken, run once for the record | expected: no blobs, reduced shape reach vs EXP-R07 null. If A1's novelty ≥ A2's, fee-gating dominates exclusion on both axes — record and move on |

### 10.4 Parameter discipline (the v7–v9 lesson)

Fees introduce two knobs into a program with documented pricing-side-effect
history (`match_cost_coeff` starved establishment across two rounds).
Therefore: the arm grid MUST include the zero point (A0 IS that point);
before any A1 result is discussed, the burned side effects are re-checked —
(a) establishment probability of NEW regions under fees, (b) whether fees
suppress novelty rather than blobs (FM-GE3's detector: if quarantine/blob
counts fall AND new-shape yield falls proportionally, the finding is FM-GE3
and the suppression curve is reported, not tuned away).

### 10.5 What would falsify the fee approach

### 10.6 Forward/debit split (the SB-GE5 contract)

The engine consults `RuleSelectionPolicy::select` once per step, with the
**start-of-window** snapshot of every region's ledger. The actual
`RegionLedger::try_fire` debit (cost) is deferred to the end-of-epoch
boundary. Consequences that fall out of this:

- A region that goes bankrupt mid-window keeps its bindings forwarded
  within that window (the policy doesn't see the change).
- Bankruptcy becomes visible from the *next* window onward.
- Per-epoch residual `free + Σ tokens == total` is asserted exactly at the
  epoch boundary, so the conservation invariant and the per-binding
  refuse happen at the same granularity.

The integration test (`crates/hg-meta/src/fee_integration_tests.rs`,
commit `b10441e`) pins both halves: A0 = no policy at the engine level,
A1 with bankrupt ledger = n=0 of n=many firings, A1 with solvent ledger
= identical to baseline within the forward-window. None of this requires
`hg-engine` source changes — the existing policy seam is sufficient.

If A1 still produces REGIME_EXPLODING panel excursions with residuals at
zero — i.e. blobs survive full pricing — then unrestrained firing was not
the operative channel and the §4.3 amendment is wrong. That outcome reopens
REDESIGN: the failure would be reported against incident register §2 as a
new species (priced-but-exploding), and exclusion becomes the default
descope for the offending lineage.
