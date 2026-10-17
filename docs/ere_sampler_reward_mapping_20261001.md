# Sampler-to-reward mapping: read-only diagnostic

Date: 2026-10-01. Diagnostic profile `E2-sampler-reward-map-readonly-v1`.
Input: the completed, negative `E2-complete-direction-reliability-v1` assay.

## Outcome

The mapping is complete. **No native simulator, reward, grammar, optimizer,
prospective contract or historical archive was changed.** This work adds an
offline analyzer, seven passing test groups, this report and a separately sealed
diagnostic archive. It ran zero Engine steps, zero grammar compilations and zero
training updates; analysis elapsed time was 63.54 seconds.

The evidence points to a difficult signal/estimator interface, not an established
sign error:

- The 183 parameters control three hierarchical categorical choices. There are
  no logits for choosing a particular production inside a legal group.
- Actual forward steps move the leaf-selection distribution by only 0.4364%
  mean total variation; executable-signature movement is 0.3191%.
- Sharing the same first uniform draw switches the selected parent with mean
  probability 3.1492%, about 11.10 times parent distribution TV. Ordered CDF
  coupling can produce substantial paired churn even when marginal movement is
  small. That does not make the marginals incorrect.
- Mean credit-eligible mass increases by only 0.01722 percentage points, with
  18 positive and 14 negative directions. This is a proxy calculation, not
  native reward improvement.
- Only 5/512 saved forward draw comparisons change utility, all losses. Each
  switches from an active parent to an adjacent inactive parent. Other choices
  sometimes change too; activation is not isolated as the sole causal effect.
- Even among 2,068 credit-eligible validation evaluations, 1,156 earn zero
  reuse: 952 have no credited fires and 204 have only one.

Recommendation: build and algebraically verify a **direct categorical-gradient
adapter**, then preregister a small native, budget-matched direction comparison.
Keep the physical objective unchanged. Do not launch longer training, reverse
the optimizer, tune alpha on these outcomes, or turn activation into reward.
No new experiment or adapter implementation is authorized by this report.

## Evidence and unchanged scientific result

The [direction assay findings](ere_complete_direction_assay_findings_20261001.md)
remain the authority for the scientific verdict: eight fixed estimator blocks,
7,168 terminal evaluations, 32 one-step estimates, five estimation caps and
5,120 valid validation evaluations. Alignment mean .1918566 has sign p=.14453125;
forward-minus-base utility is −.0088317614, zero wins/five losses/three ties,
improvement sign p=1.0. `support_direction:false` remains unchanged.

This analysis reads every saved cell and rechecks its schedule, catalogue
membership, native receipt accounting, proposal credit, score and constraint
utility using the hash-pinned historical checker stack. It does not rerun the
Engine or independently reconstruct physical trajectories or RNG draws.
The old audit's forty tamper controls belong to that old audit, not to a newly
claimed scientific acceptance test. No new p-values or acceptance criteria are
introduced here. Counts of directions and draws are descriptive; they do not
replace the registered eight-block statistical denominator.

## Architecture: where the parameters enter

| Component | Role in this mapping | Source |
| --- | --- | --- |
| Parent bank | 28 core seeds × three proposal variants × inactive/active = 168 parents | `crates/hg-sim-core/src/ere/semantic.rs`, `parent_bank` |
| Valid-production grammar | Enumerates legal, applied, non-no-op productions; masks unavailable operators/roles | `crates/hg-sim-core/src/ere/valid_grammar.rs`, `compile`, `emit`, `emit_selected` |
| Coherent kernel | Applies exactly the selected typed edit; structural edits retain activation, activation operators change it | `crates/hg-sim-core/src/ere/coherent.rs` |
| Program decoding | Core is executable; proposal is executable only when active; candidate execution is sorted/deduplicated | `crates/hg-meta/src/program.rs`, `candidates`, `execution_bytes` |
| Canonical identity | Encodes rule structure and priority; structural comparison ignores priority | `crates/hg-meta/src/schema.rs`; `signal.rs`, `structural` |
| Native evaluation | Fixed world, actual proposal event credit, horizon/caps and reuse reward | `crates/hg-sim-core/src/ere/signal.rs`, `evaluate_with_horizon` |
| Existing optimizer | Rank-shaped antithetic ES over all 183 logits | `crates/hg-train/src/optim.rs` |

The ordering of the parent bank matters to shared-uniform coupling. Each variant
is stored inactive first, active second: even indices are inactive and odd
indices active. Adjacent indices can additionally cross a proposal-variant or
core-seed boundary. The variants are: priority-only difference, a proposal
label changed to 1, and a proposal RHS self-edge addition. These are not 168
interchangeable activation settings.

### Exact hierarchical probability law

| Zero-based parameter range | Number | Controls |
| --- | ---: | --- |
| 0–167 | 168 | Softmax over all parents |
| 168–180 | 13 | Shared operator logits, softmax masked separately for each parent |
| 181–182 | 2 | Shared Core/Proposal role logits, softmax masked separately for each parent/operator |
| None | 0 | Production index: uniform within the chosen legal parent/operator/role group |

Operators by local index: AddSlot 0, RemoveSlot 1, RetargetSlot 2, AddVar 3,
RemoveVar 4, AddPatternEdge 5, RemovePatternEdge 6, AddLabel 7, RemoveLabel 8,
ChangeLabel 9, ChangePriority 10, Activate 11, Deactivate 12.

For a legal leaf `x=(p,o,r,k)`, with legal supports `O(p)` and `R(p,o)` and
production count `N(p,o,r)`:

```text
Pθ(x) = softmax(θparent)[p]
      × masked_softmax(θoperator, O(p))[o]
      × masked_softmax(θrole, R(p,o))[r]
      / N(p,o,r)
```

The same operator logit affects many parents, but its probability is not globally
fixed: the denominator depends on each parent's legal mask. Roles behave
similarly; a singleton role support has probability one, irrespective of its
logit. Adding a constant to every logit in any one of the three parameter groups
does not change this probability law.

The native sampler consumes an inverse-CDF softmax draw at each categorical
stage, including singleton masks, then a uniform integer production draw.
It does not reject/resample, silently substitute parent fitness or fall back to
another parent. Production selection is not directly trainable: a beneficial
and harmful production in the same group share the same parameter-controlled
group probability. The optimizer can change how often that group is reached,
not their relative probabilities within it.

### Full persisted support

| Catalogue property | Count |
| --- | ---: |
| Parents | 168 |
| Nonempty parent/operator groups | 1,744 |
| Nonempty parent/operator/role groups | 3,232 |
| Legal production leaves | 18,956 |
| Active leaves | 9,478 |
| Credit-eligible leaves | 9,282 |
| Distinct executable signatures | 10,433 |
| Operator groups with one / two legal roles | 256 / 1,488 |

Leaf counts are not probability weights: selecting a group with two productions
and one with twenty does not assign equal mass to each individual production.
Coalescing equal executable signatures exposes selection changes that are
execution-invisible. It is **not** a proof of native-reward equivalence: the
designated core/proposal identities can affect which events receive credit.

## Architecture: when an emission earns reward

The complete grammar guarantees an admitted edit, not useful native execution.
Native reward has additional conditions:

1. The final program's proposal must be active.
2. Its canonical structure must differ from the core after zeroing priority.
   A priority-only change is not sufficient structural novelty for credit.
3. The executable candidate corresponding to the proposal must actually fire.
   Other core activity is not proposal reuse.
4. The rollout must complete all 100 steps without a resource cap or other
   terminal failure. Only then is native reuse a valid scalar.

For credited fire count `F` in a valid 100-step rollout:

```text
native_reuse = log1p(max(F − 1, 0)) / log(100)
constraint_utility = 1 + native_reuse
measured resource cap: native_reuse = null, constraint_utility = 0
```

Thus zero and one credited fire both earn zero reuse. An inactive but otherwise
valid evaluation earns utility one, not cap utility zero. Active, eligible and
repeatedly firing are three different properties. Proposal priority, guards,
pattern/label compatibility, core competition and trajectory evolution matter
after activation. The ordinary fixed world begins with Label 0 nodes; merely
activating a proposal with a changed Label 1 constraint does not establish that
it matches. This mapping does not assign a measured cause to every no-fire case.

The offline eligibility calculation parses the saved program RCF, zeroes the
four priority bytes in each canonical candidate and checks for two distinct
structural bodies. For this two-allele grammar that identifies active structural
novelty. It was checked against activation classes and credited-rule IDs in
all 7,168 receipts, including capped cells.

### Observed reward bottlenecks

| Valid-cell outcome | Estimation | Validation |
| --- | ---: | ---: |
| Valid cells | 2,043 | 5,120 |
| Positive reuse | 407 | 912 |
| Zero reuse: inactive | 1,126 | 2,848 |
| Zero reuse: active but structurally identical | 106 | 204 |
| Zero reuse: eligible but no credited fires | 355 | 952 |
| Zero reuse: eligible but exactly one credited fire | 49 | 204 |
| Total zero reuse | 1,636 | 4,208 |

The five estimation caps are excluded only from this **valid-score breakdown**,
not from the scientific schedule or constraint-utility means. Among all 2,048
estimation cells there are 922 active and 816 eligible emissions; among all
5,120 validation cells, 2,272 active and 2,068 eligible. Four capped estimation
cells already have at least two credited fires, but their native score stays
null. Fire count alone does not erase a cap.

Only 17.8125% of validation cells earn positive reuse. Of the eligible validation
cells, 1,156/2,068 still earn zero. These cell counts include the prescribed
repeated physics evaluations and shared-arm comparisons; they are not independent
Bernoulli samples or new significance tests.

## Exact sampler features at the two fixed contexts

Computed from every legal leaf under the hierarchical law, with f64 numerical
arithmetic. These are probability calculations, not new RNG samples. They
model the softmax law, not a bit-exact replay of finite-precision RNG thresholds.

| Feature | Zero logits | Alternating +.25/−.25 logits |
| --- | ---: | ---: |
| Initially active parent mass | .5000000000 | .3775406688 |
| Finally active emission mass | .5000000000 | .3797212069 |
| Credit-eligible emission mass | .4417199039 | .3296265281 |
| Core role marginal | .4260058064 | .3241059049 |
| Proposal role marginal | .5739941936 | .6758940951 |

The alternating context assigns +.25 to even, inactive parents and −.25 to odd,
active parents. It also biases operators and roles according to their index
parity. This asymmetry follows from the prospectively fixed context; it is not
a reason to discard it or replace it with a favorable context after measurement.

### What the saved forward updates actually change

Summary over all 32 saved forward vectors, with each actual f32 vector converted
to f64 for the calculation. TV is half the L1 distance between distributions.
Percentages below represent probability mass; signed changes are percentage
points. No directions are selected or dropped.

| Diagnostic | Mean | Minimum | Maximum |
| --- | ---: | ---: | ---: |
| Finally active mass change, pp | +.0178110 | −.0527775 | +.1352904 |
| Credit-eligible mass change, pp | +.0172204 | −.0335333 | +.1272720 |
| Parent distribution TV, % | .283820 | .136405 | .444809 |
| Same-uniform first-parent switch probability, % | 3.149217 | .911873 | 7.076201 |
| Full leaf-selection TV, % | .436422 | .172603 | .688650 |
| Executable-signature TV, % | .319145 | .113475 | .552772 |
| Operator marginal TV, % | .270034 | .090062 | .427685 |
| Role marginal TV, % | .154191 | .010042 | .493765 |

Active probability rises in 16 directions and falls in 16. Eligibility rises in
18 and falls in 14. Zero-context mean eligible change is +.0116910 pp; alternating
context mean is +.0227497 pp. A positive average proxy movement can coexist with
negative sampled native reward. No exact expected native utility is inferred.

Mean squared-step energy fractions are parent 91.6630%, operator 7.3407%, role
.9963%. For the **eligibility proxy only**, the exact gradient's group L2 norms
are parent .029399/operator .052740/role .000265 at zero, and
.027262/.046921/.000526 at the alternating context. The operator proxy gradient
has larger norm despite much less step energy. This motivates testing a
parameter-aware estimator; it does not prove operator-only optimization is
better for native utility. Raw logit norms include gauge directions and are not
comparable to probability movement without the softmax mapping.

### Why shared seeds can churn more than the distribution moves

For ordered parent probabilities `a` and `b`, let `Ia(i)` and `Ib(i)` be the
CDF intervals that select parent `i`. Under the same first uniform draw:

```text
P(parent changes) = 1 − Σi length(Ia(i) ∩ Ib(i))
TV(a,b) = ½ Σi |a(i) − b(i)|
```

CDF boundary shifts can move draws between neighboring parents even when
both parents' individual marginal changes are small. For example, distributions
(.5,.5,0) and (0,.5,.5) have TV .5 but switch on every shared uniform draw.
The current assay's mean CDF switch probability is 11.10 times mean parent TV.
This is an exact first-stage coupling calculation, **not** the complete emitter's
switch probability, a native reward variance estimate, or evidence that every
alternative coupling would have lower reward variance.

Because inactive/active parents alternate, local CDF boundary crossings readily
change activation. Marginally, probability may improve slightly while a small
finite paired panel happens to encounter unfavorable crossings. The receipts
identify such crossings here, but do not establish that this explains all
estimation noise or that the expected native objective improved.

## Every observed forward reward loss

All four physics scores agree within every saved validation draw/arm. The table
therefore describes physics-averaged draws, not four independent loss events.
All five losses occur in context 1, remain valid, and change active to inactive;
post-update credited fires are zero and native reuse is zero.

| Block / draw / arm | Parent | Operator | Role | Production index | Base fires | Reuse change |
| --- | --- | --- | --- | --- | ---: | ---: |
| 1 / 3 / forward B | 115→116 | 8→8 | Core | 0→0 | 48 | .8406206→0 |
| 4 / 8 / forward A | 139→140 | 0→0 | Core | 7→7 | 100 | 1→0 |
| 5 / 6 / forward A | 111→112 | 9→7 | Proposal | 0→1 | 48 | .8406206→0 |
| 6 / 0 / forward A | 131→130 | 9→9 | Core | 0→0 | 100 | 1→0 |
| 7 / 1 / forward B | 119→120 | 7→5 | Proposal | 2→18 | 48 | .8406206→0 |

These correspond to IDs `validate-r{block}-c1-d{draw}-{arm}-p0`; the JSON
diagnostic records exact base/post IDs, credit IDs and selections for all
35 changed-signature or changed-utility comparisons across forward and reverse
controls. The reverse control's one gain, at block 4/context 0/draw 0/reverse B,
switches parent 42→41, inactive→active, reuse 0→1. It is not a rationale for
selecting reverse B or negating the optimizer.

Of 512 forward comparisons, 18 change executable signature and five change
utility. The zero context has seven signature changes and no utility changes;
the alternating context has eleven signature changes and five losses. This
explains the *recorded* negative effect exactly. It does not identify the
population gradient or causal contribution of a single changed field.

## Coverage and what remains unknown

The complete archive observes 1,930 distinct leaves of 18,956, including 1,925
with valid scores. Their total selection mass under the two baseline contexts
is .3107952 and .3280287, respectively. These masses use the union of leaves
observed anywhere in estimation or validation, not a fresh probability sample
or complete valid-reward coverage. No valid leaf exhibits score variation in
its recorded evaluations; unobserved physics and trajectories remain unknown.

Most baseline probability mass has no saved per-leaf native outcome. Missing
leaves are not assigned zero, an activation score or their neighbors' fitness.
Consequently `exact_native_expected_reward_available:false` is explicit in
the diagnostic. Executable-signature coalescence likewise does not substitute
for a credit-aware native outcome table. No causal attribution to priority,
label, operator or role is measured by this analysis alone.

## Recommended next bounded milestone

The next implementation should target the estimator interface, not relax the
gates or quietly change the reward. For a sampled legal leaf `(p,o,r,k)`, the
three score-function components are directly available:

```text
parent j:   1[j=p] − Pθ(parent=j)
operator j: 1[j=o] − Pθ(operator=j | p), within O(p); zero outside it
role j:     1[j=r] − Pθ(role=j | p,o), within R(p,o); zero outside it
production: no parameter, hence no score component
```

Differentiating the finite sum `Eθ[U]=Σx Pθ(x) E[U|x]` gives
`∇Eθ[U]=Eθ[U ∇log Pθ(x)]`, provided the fixed evaluator's conditional outcome
law does not depend on logits beyond selection. This targets the original
constraint utility directly rather than a rank-shaped, sigma-one parameter
perturbation objective. It is a mathematical candidate, **not** a measured
variance reduction or improvement result. Rare native reuse can still make it
noisy, and production-level credit remains unparameterized.

Suggested sequence, requiring a separate implementation request and frozen
prospective specification before new measurements:

1. Implement the selection-score adapter in isolation. Check masks, singleton
   zero derivatives, group gauge sums, sign, cap utility inclusion and f32
   receipt reconstruction. Use the fully known activation/eligibility features
   only as engineering oracles: compare enumeration-derived gradients with
   finite differences and estimator expectations. Do not use these features
   as the native training objective or select settings on historical reward.
2. Freeze a small comparison with the existing ES estimator. A proposed native
   estimator budget is **64 evaluations per estimate**, matching the old
   16 candidates × four emissions, now sampling at the fixed base context.
   Freeze independent identities, baseline construction (for example a
   leave-one-out baseline), update scale, validation allocation, cap treatment,
   block denominator, resource ceilings and failure conditions before launch.
   The direct method's draw grouping and physics replication must be explicit;
   equal native-cell counts alone do not make every variance comparison fair.
3. Test native direction reproducibility and held-out utility, not just agreement
   with the eligibility oracle. Keep all allocated blocks and caps; do not
   increase budgets or change alpha after inspecting outcomes. Only a passing
   native result would justify proposing another training campaign.

A distribution-preserving alternative coupling is a secondary methodological
option, not an automatic fix. It would need marginal-law proofs, deterministic
identity/receipt changes and fresh comparison. This report recommends the
selection-gradient adapter first because it makes the objective-to-parameter
mapping explicit without changing the grammar or physical reward.

## Reproduction, verification and provenance

Run from the repository with the unchanged archived checker stack available:

```sh
python3 -B -m unittest discover -s scripts -p test_analyze_ere_sampler_reward.py -v
python3 -B scripts/analyze_ere_sampler_reward.py \
  output/ere_complete_direction_assay_20261001 \
  output/NEW_FRESH_MAPPING_DIRECTORY
```

The analyzer refuses an existing output directory before reading the archive.
It verifies exact seal coverage and source hashes, derives the full legal model,
checks every saved cell and verifies the input/helper seals again at completion.
The new artifact's four sealed entries are the analysis, manifest and two source
snapshots; the root seal file itself is additional. This is a reproducible
accounting/probability diagnostic, not a standalone independent physics auditor.

Seven test groups cover catalogue classes/support, structural priority handling
and malformed packets, masked softmax/gauge normalization, analytic gradients
versus central differences, CDF churn versus TV, signature coalescence/zero-step
movement and eligible-but-zero-reward examples. The finite-difference test uses
step 1e−4 to balance subtraction roundoff against quadratic truncation error;
a step-size check gave maximum error below 5e−11 at the tested coordinates.
No scientific threshold was edited to accommodate this numerical unit test.

All 459 historical frozen inputs still match their live counterparts. The sealed
executed binary hash verifies. The input archive and both helper archives
verified unchanged before and after analysis:

| Artifact | Sealed entries | Root seal SHA256 |
| --- | ---: | --- |
| Direction assay input | 7,686 | `ba88c696014fa42ca612f4c20baadb1818cc6c5757fd41594c87a1e4bc1a185a` |
| Original short-training continuation helper | 1,690 | `dc2088d51cad77130a30f49b01dd4ef8f808d1a79cb66ec223f17964b3660132` |
| Runtime signal helper | 460 | `0099ecae4b7487e9ac975f73d813e7cbde0de430470dc66198bbafd7df16488c` |
| New diagnostic | 4 | `ce9617430cf6ec76aef7ca71b910e958f38645e8775ead3cd4ee35942e9b97ee` |

Native input binary SHA256:
`9657c3513f8c08d8a683e3790141157632cf61bc062837f65095ef98581538fb`.
Diagnostic analysis SHA256:
`363912cc636ece61bdfcae2192321ff5ce89fbb030573e00a9a3d372d19287f4`.
Source hashes and exact per-direction/cell details are in the new
[manifest](../output/ere_sampler_reward_mapping_20261001/manifest.json) and
[analysis JSON](../output/ere_sampler_reward_mapping_20261001/analysis.json).
The new archive was independently reverified after creation. Historical failures
and criteria remain untouched; no inference here admits longer-horizon training
or establishes learned maintenance.
