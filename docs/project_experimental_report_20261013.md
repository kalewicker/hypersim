# Hypersim: experimental programme and closeout report

**Storage update — 2026-10-14:** Generated evidence was archived and fully
restore-verified before local cleanup. Original artifact links below require
restoration from the flash-drive archives. The scientific outcomes and dated
verification records remain historical evidence; see the
[archive/restoration guide](project_cleanup_20261014.md).

**Reporting date:** 2026-10-13, America/Phoenix.  
**Status:** retrospective research synthesis and programme closure.  
**Scope:** the Hypersim repository, its preserved LK workspace, Evolving Minds,
constructive chemistry and the final cumulative-evolution experiment. Separate
Mind Seed and FL-Decipher projects are outside this report.

## Abstract

Hypersim investigated whether a deterministic hypergraph-rewriting substrate
could support increasingly rich dynamics, replication, selection, learned
control, mutable executable rules and cumulative functional evolution. The
project produced a reusable simulation and measurement system and several
bounded positive results: paid replication and turnover; conditional selection;
locality restoration and finite coexistence; learned candidate selection;
learned selection and composition of authored paid edits; exact paid repair and
prevention; and durable inheritance of specific externally selected rule edits.
These results did not combine into demonstrated autonomous cumulative evolution.

The final prospectively fixed campaign tested 47 matched blocks across four
arms, retaining all 188 worlds and 1,880 sampling slots. All 111 unique diagnostic
assays completed, with 13,178 native rewrites and a passing independent audit
and sealed-archive recheck. No endogenous or external-editing world met the full
criterion: 0/47 in each arm, against a primary endogenous threshold of 16/47.
One endogenous event pair passed the functional intervention checks but lacked
the required paid transmissions and active descendant generations. The outcome
was a valid negative for the tested world, not an operational failure.

The programme is closed at this boundary. Earlier positive capabilities and
negative results remain separate evidence; they are neither erased by the final
negative nor pooled into a stronger positive. No universal impossibility of
hypergraph evolution is established. No additional campaign, longer horizon,
relaxed criterion or exhaustive reachability project is commissioned by this
report.

## Contents

- [Evidence guide](#reading-and-evidence-guide)
- [Research questions](#1-research-questions-and-standards-of-evidence)
- [Architecture](#2-architecture-and-experimental-boundaries)
- [Result map](#3-programme-level-result-map)
- [Substrate](#4-fixed-rule-substrate-characterization-ae)
- [ALife and causal emergence](#5-artificial-life-selection-and-causal-emergence)
- [Locality](#6-locality-and-leakage-from-growth-to-measurement-back-action)
- [Cognition](#7-cognition-genuine-learning-with-an-objective-mismatch)
- [Self-modification](#8-self-modifying-rule-ecology-v3v24)
- [ERE](#9-ere-endogenous-rule-representations-repair-and-learning)
- [Evolving Minds and chemistry](#10-evolving-minds-and-constructive-chemistry)
- [Final campaign](#11-the-final-cumulative-evolution-experiment)
- [Interpretation](#12-cross-project-interpretation)
- [Closure](#13-closure-preservation-and-documentation-corrections)
- [Preservation and reproducibility](#14-preservation-and-reproducibility-handoff)

## Reading and evidence guide

This report explains the scientific trajectory and final interpretation. The
[documentation and evidence catalogue](project_experimental_inventory_20261013.md)
indexes the first-party documents and bounded on-disk metadata inventory; its
[JSON companion](project_experimental_inventory_20261013.json) records paths,
SHA-256 identities and raw status fields. The
[architecture map](project_architecture_and_experiment_status.md) retains detailed
implementation history. The [final campaign findings](cumulative_scientific_campaign_findings_20261013.md)
provide its exact resources, seal and diagnostic counts.

The review reads current source structure, findings, contracts and saved metadata.
It does **not** freshly rerun every historical experiment or certify every old
archive. Historical audit/replay claims below are attributed to their linked
records. The final campaign's independent audit and recheck were completed in
the immediately preceding execution on this date. This documentation work adds
zero native simulation, training or optimizer updates.

The catalogue deliberately avoids a recursive scan of large run payloads. It
includes named metadata to depth three below six explicit evidence roots, and
excludes build, vendor and frozen source/evidence trees from the document index.
Historical LK run archives were not all imported with its source; an absent
local payload is an availability limitation, never a scientific zero. The
[LK integration record](lk_workspace_integration.md) identifies those archives
and known historical test limitations. A source document is not an independently
reproduced result merely because it is present in this checkout.

## 1. Research questions and standards of evidence

The project progressively distinguished six questions that initially overlapped:

1. Can a fixed rule set sustain diverse, recurring substrate activity?
2. Can rule-defined organisms reproduce, turn over and undergo non-degenerate selection?
3. Do coarse causal descriptions add information beyond their micro descriptions?
4. Can an optimizer learn useful control over native rewrite opportunities?
5. Can executable rules be changed, paid for, maintained and inherited?
6. Can endogenous changes produce successive useful innovations that persist
   through descendants and causally depend on earlier innovations?

Success at an earlier question does not answer the later ones. A high novelty
count need not reflect function; an improved objective need not reflect greater
complexity; a surviving root need not preserve its program; and a durable edited
lineage need not have discovered or chosen its edit autonomously.

The strongest later protocols froze inputs, denominators, source identities,
resource ceilings, control arms and acceptance rules before native execution.
They retained failed routes, caps, untriggered births, dead samples and missing
measurements. Earlier exploratory studies have weaker or different designs and
are labelled accordingly. There is no project-wide pooled significance test,
common independent replicate denominator, or retrospective universal effect size.

| Evidence category | What it establishes | What it does not establish |
|---|---|---|
| Engineering fixture | A specified mechanism and its controls execute correctly | Spontaneous discovery, generality or ecological success |
| Accounting/integrity audit | Saved inputs, events, payments and outputs reconcile | Independent physical replay unless explicitly performed |
| Exact replay | A specified execution reproduces at its declared identity boundary | Independent replication on new worlds |
| Authored capacity witness | A useful state or trajectory can be constructed | An evolutionary process can reliably find it |
| Fixed-checkpoint intervention | Conditional causal effect at those saved states | Fresh-world population generality |
| Fresh fixed panel | Evidence across the specified new independent units | Universal architecture capacity or impossibility |
| Complete negative | The admitted measurement fails unchanged criteria | Missing data, a software failure, or proof of impossibility |

## 2. Architecture and experimental boundaries

The root Cargo workspace contains 21 packages: graph and schema foundations,
rule compilation and execution, observation, evaluation, causal analysis,
ALife, alphabet analysis, cognition/training, self-modification, meta-rule
representations, output/reporting, app interfaces and the GUI/CLI entry points.
The [user guide](user_guide.md), [rewrite contract](rewrite_contract.md) and
[rule grammar](rule_grammar.md) describe the ordinary execution interface.

| Layer | Main components | Experimental responsibility |
|---|---|---|
| State | `hg-core`, `hg-schema` | Generational graph identity, typed properties, snapshots, stable result shapes |
| Execution | `hg-rules`, `hg-engine`, `hg-parallel` | Compile rules, enumerate bindings, resolve conflicts, apply changes deterministically |
| Extension seams | `hg-observe` | Read-only rewrite observations and candidate-selection policy interface |
| Measurement | `hg-eval`, `hg-causal`, `hg-alphabet` | Calibration, regimes, recurring context sets, causal partitions and locality instruments |
| Ecology and control | `hg-alife`, `hg-cognition`, `hg-train` | Bookkeeping, policies, learning and campaign evaluation |
| Mutable rules | `hg-selfmod`, `hg-meta` | Reified executable material, codecs, mutation, admission and economies |
| Applications | `hg-sim-core`, `hg-output`, `hg-report`, `hg-app-api`, `hg-gui`, two binaries | Ordinary orchestration, durable evidence and user interfaces |

The ordinary application path is configuration → compiled rules → graph state →
candidate bindings → policy selection → conflict resolution → application →
observers → metrics and durable outputs. A selection policy does not bypass
conflict resolution. Optional research features retain explicit activation
boundaries. `hg-sim-core` orchestrates ordinary application runs; research
library drivers such as `hg-selfmod` are documented exceptions to an overly
literal reading of the old “only engine importer” slogan.

`experiments/lk` is an isolated, divergent Cargo workspace, not an extra set of
members in the root workspace. Its historical self-modification, ERE and
cognition results must be interpreted using its own engine, schemas and rules.
Use `scripts/lk.sh` for that workspace. `experiments/cumulative` is another
isolated workspace that deliberately borrows preserved LK graph/compiler/engine
components. It adds its own frozen world law and campaign machinery. These
implementations share ancestry, not an assertion of identical scientific physics.

Evolving Minds adds paid developmental organisms and explicit lineage/lifecycle
interfaces in separately versioned profiles. Constructive chemistry adds a
default-off checked staging/atomic publication path; its scripted CC-1 fixture
does not supply a kinetic ecology. Changes to source profiles, costs, encodings,
resource laws or scheduling define different experimental conditions.

### Reproducibility and custody

Ordinary `manifest.json` and training `campaign_manifest.json` files are lifecycle
authorities. Later specialized harnesses also use versioned `status.json` and
`run_status.json`; their associated contracts determine their meaning. A
`complete` status alone is never a scientific pass. Reports, audit receipts,
provenance and immutable seals supply the interpretation.

Exact continuation required more than a graph hash: allocator free-list state,
RNG state, rule identity/order, ledgers, pending work and evidence cursors all
became relevant. The final campaign used durable intent-before-action records,
single-use native claims, conservative reservations, compact evidence and
independent reconstruction. Unknown work was not eligible for refund or retry.
Build caches are regenerable; sealed outputs, frozen sources, raw failure
receipts and historical checkpoints are research evidence.

## 3. Programme-level result map

| Programme | Strongest supported result | Limit and closeout disposition |
|---|---|---|
| Substrate A–E | Rich but finite recurrent context sets in tested fixed-rule systems | No accumulating repertoire; stop equivalent fixed-rule sweeps |
| ALife 5–5e | Replication, turnover and conditional non-degenerate selection | Inert threshold drift sink persists; no universal adaptive ecology |
| CE and V-track | Calibrated structural/metabolic instruments and scoped partition results | No general claim that replication produces causal emergence; V-track closed |
| Locality/LK | Removal attractor, lifetime-mediated coexistence and quantified measurement back-action | Scaffold-dependent; closure defect remains unmeasured under admitted conditions |
| Cognition C–E | Bounded trained selection beats controls on four substrates | Objective rewards repetitive simplification; not increasing complexity |
| Selfmod V3–V24 | Specific edited families can inherit and maintain finite function | Selected edits/reversions, not autonomous cumulative innovation |
| ERE | Native mutable rules, bounded repair/prevention and several learned edit-selection mechanisms | Broad sustained autonomous rule ecology remains unaccepted; resource-learning campaign negative |
| Evolving Minds | Paid development, reproduction, exact recovery and inherited variants | Reproducing tested edits were behaviorally neutral; adaptation unestablished |
| Constructive chemistry | Scripted conservation and atomic rejection/publication | Kinetics, reproduction and evolution not tested |
| Final cumulative campaign | Complete audited discriminating experiment | Valid negative; tested configuration retired |

## 4. Fixed-rule substrate characterization: A–E

The alphabet programme separated raw symbols from recurring symbols, effective
alphabet size, radius refinement and novelty over time. Bounded substrates had
small effective alphabets, approximately 1.7–4.1 in the early cases, while large
raw alphabets in growing rules often consisted of nonrecurring contexts.

| Track | Principal result | Qualification |
|---|---|---|
| A | Time-plateaued novelty and small effective recurrent repertoires | Radius saturation and temporal plateau are distinct |
| B | Roughly linear effective-alphabet scaling across the 1–4-rule series | Four points with contention are not a universal scaling law |
| C | `swap` and `cyclic3` establish an intermediate “partial” class | Both settle into fixed recurrent sets; diversity depends on activity shares |
| D | Scaling and priority reversal preserve the bounded conclusion | Universal `influx` defeats the intended contention-free construction |
| E | `rebalance` retains a byte-identical recurrent alphabet over 16× horizons | Rapid contraction toward a conserved degree sequence explains this family |

The practical result is a baseline: fixed authored rules can sustain activity
and structural richness without accumulating a functional repertoire. This
motivated rule modification; it did not prove the architecture universally
incapable of richer dynamics. The planned equivalent F-track was stopped.

Sources: [substrate report](../hypersim_substrate_characterization_report.md),
[A](atrack_findings.md), [B](btrack_findings.md), [C](ctrack_findings.md),
[D](dtrack_findings.md), [E](etrack_findings.md).

## 5. Artificial life, selection and causal emergence

### 5.1 Replication and selection: Experiments 5–5e

Experiment 5 established spontaneous carrier replication and turnover with
accounting and reproducibility in the tested ecology. Its original selection
claim was narrowed: node-funded reproduction drove thresholds toward a
degenerate always-satisfied regime rather than a robust interior optimum.

Experiment 5b made maintenance genome-dependent, but selection toward negligible
maintenance produced effectively immortal carriers and lost turnover. Parent
funding in 5c created a real interior replication-threshold optimum for
behaviorally relevant lineages. A proportional reproductive tax in 5d improved
selection in energy-rich lineages while their thresholds still affected action.
Reproductive senescence in 5e improved marginal cases but did not remove the
deep inert sink. Within the tested unclamped trait model, a threshold that no
longer controls behavior is insulated from those selection levers.

These are genuine mechanistic distinctions. “Selection failed everywhere” is
false; “robust open-ended adaptation was achieved” is also unsupported.
Sources: [5 scorecard](experiment5_scorecard.md), [5b](experiment5b_findings.md),
[5c](experiment5c_findings.md), [5d](experiment5d_findings.md),
[5e](experiment5e_findings.md).

### 5.2 Causal-emergence experiments and the validation sequence

Experiment 6a's survivor-only partition was identity-pinned and its causal
emergence magnitude was superseded. Birth/death-aware 6b retained a
non-degenerate boundary but its CE declined from roughly 0.25 at cadence 1 to
0.019 at cadence 64. Population-count 6c had richer microdynamics, but no tested
coarsening/cadence exceeded CE=1; its reported maximum was 0.5625.
These negatives apply to those observables and partitions.

The V-track then tested instrument engagement, WL depth, nulls, stationarity,
selection power, sterile tokens, moving carriers, bounded churn and metabolic
partitions. Early statements about replication specificity were progressively
narrowed to sustained presence or structural activity. The metabolic channel
distinguished active from inert carriers but not the desired biological
subclasses. Its coarseness was intrinsic to the three-valued energy-change
observable. Sorted aggregation fixed a cross-process precision defect.

The V-track is complete. Structural and metabolic measures are complementary
questions, not competing estimates of one universal “life” quantity. Its
original node-funded selection negatives do not erase the parent-funded 5c–5e
mechanisms. Sources: [6a](experiment6_findings.md), [6b](experiment6b_findings.md),
[6c](experiment6c_findings.md), [V-track closeout](validation_vtrack_closeout_findings.md).

## 6. Locality and leakage: from growth to measurement back-action

In the energy-metered LK-R family, subdivision generates filamentary structure
and long-range shortcuts, while fusion removes emergent links. Populated
relaxation produced the exact measured decay form `L(t)=L0(2/3)^t`, with rate
`ln(3/2)` and a structurally restored clean endpoint. Energy-threshold removal
was bistable because it sampled a near-degenerate conduction plateau.

Later LK work moved beyond that early balance failure. Lifetime gating produced
bounded coexistence with `L* = beta * k * p` over the tested lifetimes and energy
budgets. Crucially, an earlier clock implemented as a high-priority rewrite
contended with the process it measured. Replacing it with passive age changed
the measured prefactor from 1.24–2.00 to 1.03–1.18 and removed a supposed
standing-population cap of 12. Production serialization was real; the claimed
population ceiling was measurement back-action.

Closure defect remained unmeasured at its admitted scientific scope. Later
sample counts met the floor, but stationarity/mask requirements still failed;
an appealing plateau was not retrospectively selected to rescue the estimate.
The v10 guard-shadow audit also corrected the causal explanation of skeleton
protection: guard-family evaluation order differed from assumed TOML order,
and degree invariance carried the non-removal guarantee. The findings remained
conditional on the authored scaffold and finite seeded energy.

The usable closeout is an attractor/coexistence and instrumentation result,
not a measured universal locality law or a closure-defect success. Sources:
[root locality findings](leakage_experiment_findings.md),
[later LK synthesis](../experiments/lk/docs/attractor_writeup.md),
[later findings and guard correction](../experiments/lk/docs/leakage_experiment_findings.md),
[back-action study](../experiments/lk/docs/back_action_v8.md).

## 7. Cognition: genuine learning with an objective mismatch

The early root-workspace experiments found that StaticBias champions did not
beat their untrained baselines on saturated objectives. Shaping changed dynamics
and causal metrics, but a changed policy class was not evidence of learning.
Four saved Stage-2 campaigns finished generation 39; their historical reports
also lacked the later-required budget-matched random arm.

The isolated LK D-phase revised those instruments and established bounded
selection-learning with a per-candidate `MlpScorer` under F5 constructor reuse.
The E-phase tested initialization directions and substrate transfer: champions
beat budget-matched random and all 30 sampled untrained directions on four
substrates, exceeding the best sampled direction by 4.9%–10.0%.

That positive result does not mean complexity increased. Subsequent measurements
found smaller graphs, lower micro effective information, fewer tasks and
shallower lattices. Reuse rewarded repetitive execution; some improved ratios
rose through denominator collapse. The proposed F6 corrective objective was
not opened because its motivating causal chain failed. The adjacent-seed
“perturbation” separation statistic was retired: adjacent ChaCha seeds produce
different random streams, not the controlled physical perturbation the
statistic required.

Thus the project demonstrated modest useful optimization of a specified
objective, while falsifying the interpretation that this objective reliably
selects richer computational organization. Sources:
[early stages](cognition_stage2_findings.md),
[D-phase closeout](../experiments/lk/docs/cognition_dphase_closeout_findings.md),
[E-phase closeout](../experiments/lk/docs/cognition_ephase_closeout_findings.md).

## 8. Self-modifying rule ecology: V3–V24

This line reified executable rules and combined mutation, reproduction, capital,
rent and competitive scheduling. Its strongest later results concern specific
externally selected edits and paid descendants. They must not be described as
autonomous discovery or pooled with the final endogenous-evolution campaign.

| Versions | Question and measured result | Interpretation retained |
|---|---|---|
| V3–V4 | Native decode and authored cycles work; evolution did not reach persistent novel sets | Early reproduction-causality interpretation was subsequently corrected |
| V5 | No-reproduction controls show some clonal reproduction is protective | Earlier claim that reproduction itself caused the deaths is refuted |
| V6–V9 | Canonical truncation, rule-id ordering, historical merit and pricing distort invasion | Entry, persistence and productive fitness require distinct calibrated controls |
| V10 | Corrected random/no-price control survives 7/8 at 20k; tested nonzero-mutation arms collapse | Collapse is observed; categorical “no viable variation” interpretation is later qualified |
| V11 | 180 mutation draws; 25 distinct variants pass short functional criterion on at least 6/8 worlds | Viable bounded variants exist without reproductive establishment |
| V12–V14 | Named prune fails full establishment under ordinary fission; fission-off helps; cap-eight parent replacement gives prune 7/8 versus faithful 1/8 | Conditional checkpoint interventions identify reproduction-policy effects |
| V15 | Fresh prune replication gives 5/8 against a 6/8 threshold | Primary replication fails despite survival and late reproduction |
| V16 | Longer continuations support retirement-cycle timing behind financing failures | Dependent diagnostic; V15 remains failed |
| V17–V18 | Named prune passes durability 8/8 in both fresh panels; two V18 alternatives pass 0/8 | Specific finite-horizon edited lineage is robust at this operating point |
| V19 | Intended second-edit genotype was not realized; assertion stops assay | Operational/intervention failure, no durability verdict |
| V19b | Further selected edit and faithful controls both pass 8/8 | A second edit can persist; no demonstrated advantage |
| V20 | Third removal gives 0/8 versus faithful 6/8; one world never reaches fork | Specific negative; missing route stays in denominator |
| V21 | Restoring a label constraint gives edited 8/8 and faithful 8/8 | Durable reversion to a known genotype, not a novel third innovation |
| V22 | Same-world continuation to 50k gives edited 8/8, faithful 7/8 | Longitudinal evidence, not fresh replication |
| V23 | Fresh serial panel gives edited 7/8, faithful 6/8 at 50k | Both qualify; one inadmissible world retained; no superiority claim |
| V23 supplement | First reached pair replays all 296 receipts; financing reconstruction covers 180k rows | Closes a verification gap without replacing the original first-seed replay |
| V24 | Four 1,000-step intervals; candidate-fate reconstruction and 60 corruption rejections | Observational conflict attribution, not scheduler intervention benefit |

The last faithful-family losses occurred with zero capital and creation
allowance 200. Saved-interval attribution identified older relatives and another
culture member as competitors for every considered candidate during the final
income gaps. This narrows an explanation but does not prove that changing the
scheduler would rescue those families or make one genotype universally superior.

The development history repeatedly distinguishes a real failure from an overly
strong explanation of it. V10's collapse was real; V11 showed that “nothing
viable exists” was too broad. V15's financing criterion failed; later durability
results did not rewrite its threshold. The final disposition preserves these
positive and negative results and leaves shadow-scheduler or fourth-edit work
unperformed.

Primary records: [V5 correction](../experiments/lk/docs/selfmod_v5_findings.md),
[V6 instrument](../experiments/lk/docs/selfmod_v6_findings.md),
[V10](../experiments/lk/docs/selfmod_v10_findings.md),
[V11](../experiments/lk/docs/selfmod_v11_findings.md),
[V15](../experiments/lk/docs/selfmod_v15_replication_findings.md),
[V17](../experiments/lk/docs/selfmod_v17_durability_findings.md),
[V19 failure](../experiments/lk/docs/selfmod_v19_second_edit_findings.md),
[V19b](../experiments/lk/docs/selfmod_v19b_second_edit_findings.md),
[V20](../experiments/lk/docs/selfmod_v20_third_edit_findings.md),
[V21](../experiments/lk/docs/selfmod_v21_reversible_edit_findings.md),
[V22](../experiments/lk/docs/selfmod_v22_maintenance_findings.md),
[V23](../experiments/lk/docs/selfmod_v23_fresh50k_findings.md),
[reached replay](../experiments/lk/docs/selfmod_v23_reached_replay_findings.md),
[V24](../experiments/lk/docs/selfmod_v24_conflict_attribution_findings.md).
The catalogue includes every intermediate preregistration and findings document.

## 9. ERE: endogenous rule representations, repair and learning

### 9.1 Encoding, exposure, growth and coherent closure

ERE moved rule material into an exposed graph and tested whether paid native
editing could generate, install, use and maintain executable classes. Early
grammar and maintenance pilots exposed quarantine, body destruction, limited
coverage and edits that canceled before live compilation. One-use controls
established that a paid change could cross the compile boundary and change
ordinary dynamics, while exposed unprotected bodies still failed maintenance.

Witness-backed repair then separated paid exact recovery from unsafe decoding.
An initial repair version admitted role-reversing matches and was superseded.
Typed repair fixed that boundary, but a sealed-payload extension exposed another
failure: composite witness/payload erasure admitted changed bodies. A separately
versioned complete-ownership codec retained the whole body obligation and passed
728 engineering cases, including quarantine of the composite erasures. These
were explicit material redundancies, not host-side regeneration of arbitrary
missing programs or autonomous ecological self-maintenance.

The coherent G-R1 closure campaign later completed 10,000 mutation draws, 168
parent baselines and 4,886 selected child trajectories, with zero unmeasured
outcomes and 27 rejected tamper cases. It passed its specific admission gate;
it did not independently admit unrestricted training or sustained evolution.

The divergent LK grammar-expansion line also characterized explosive mutation
growth and fee containment. Its G-GE1 hot fraction was 0.1419 in the original
reported panel, placing it in the preregistered descope band. Fee effects were
seed-sensitive; the later C2 grid remained partial with three completed seeds,
a compute kill and an unrun seed. The similarly named EXP-R09 F5-prime optimizer
is a different experiment and did not establish improvement over initialization.
ERE's causal measurement work additionally showed how meta-band inclusion
inflated apparent micro-EI and how changing rule sets defeated finite-cadence
stationarity requirements. These findings cannot be merged into one “R09” result.

Sources: [failure register](ere_failure_register.md) (historical predictions,
including an early reproduction explanation later corrected by selfmod V5),
[typed repair](ere_witness_repair_v2_findings.md),
[unsafe composite boundary](ere_sealed_repair_findings.md),
[complete witness](ere_complete_witness_findings.md),
[coherent closure](ere_coherent_closure_campaign_findings_20260929.md),
[GE1](../experiments/lk/docs/gge1_findings.md),
[fee-loop/partial grid](../experiments/lk/docs/ere_r5_findings.md),
[F5-prime](../experiments/lk/docs/exp_r09_f5prime_findings.md),
[measurement limits](../experiments/lk/docs/ere_r6_findings.md).

### 9.2 Broad grammar learning and direction diagnostics

| Study | Completed denominator / key result | Scientific status |
|---|---|---|
| Short training | 320 training and 768 held-out evaluations; mean gain +0.05839, sign p=.109375 | Improvement unconfirmed |
| Fixed-champion confirmation | 6,144 evaluations, 44 cap nulls | All-valid rule prevents official full-panel means/p-value; no confirmation |
| Feasibility-aware training | 320 training and 6,144 held-out evaluations; utility gain +.004825, p=.2983 | Improvement criterion fails |
| Budget-matched minibatching | 9,472 evaluations, eight replicate comparisons; effect −.007155, p=.63671875 | Batching unsupported at fixed budget |
| Fixed update-direction assay | 7,168 evaluations; alignment p=.14453125; forward effect −.008832 | Useful-direction conjunction fails |
| Categorical versus ES | 12,288 native evaluations; original frozen audit fails; corrected historical accounting passes | Positive directional endpoints do not overcome negative weighted-validity guard |
| Feasibility-aware categorical development | 2,176 evaluations, four retained cap/nulls; both moving proposals have negative validation effects | Integration passes; no improvement claim |

These are separate protocols, not repeated samples from one null hypothesis.
The categorical accounting correction changed accumulation order in the
independent reference to match the measured implementation; it preserved the
original failed auditor and did no new native work. Its weighted validity
effect, −.0000141485847120, still failed the unchanged ≥−1e-12 guard.

Sources: [short training](ere_complete_short_training_findings_20260930.md),
[confirmation](ere_complete_champion_confirmation_findings_20261001.md),
[feasibility](ere_complete_feasibility_training_findings_20261001.md),
[minibatching](ere_complete_minibatch_trial_findings_20261001.md),
[direction](ere_complete_direction_assay_findings_20261001.md),
[original categorical trial](ere_categorical_comparison_scientific_findings_20261003.md),
[historical correction](ere_categorical_comparison_historical_audit_findings_20261003.md),
[categorical feasibility pilot](ere_categorical_feasible_development_findings_20261003.md).

### 9.3 Narrow learned mechanisms and maintenance

Smaller tasks did yield positive learned behavior. Three paid-choice learners
improved expected native novelty reward by approximately .158–.165. Shared
two-edit policies improved all three optimizer-held-out diagonal pairs in each
of three policy seeds. Graph-aware learners using authored connectivity
features exceeded both trained blind controls and the balanced graph-blind
ceiling, while retaining the weaker third seed.

Learning those relations from a raw-adjacency encoder was harder. The initial
sampled native trial failed all three full/contrast criteria. A separate fixed
100-update exact-gradient diagnostic on saved native rewards succeeded; its
12-update endpoint remained negative. The fixed update100 encoder subsequently
passed native controller-transfer checks on 396 cells/9,504 advances. These
were exposed fixtures with existing deterministic native seeds, not independent
unseen ecological worlds.

Paid repair restored newly born rules after damage in all nine learned worlds
of a 63-cell maintenance panel, but first genotype/installed lifetimes of 11/10
steps failed the fixed 16-step continuity requirement. An authored paid
interceptor then preserved useful classes in 9/9 learned worlds versus 1/9
same-weights masked and 2/9 trained-blind controls in a 72-cell transfer panel.
The learner chose birth edits; the defense timing, mechanism and fuel remained
authored. It did not learn maintenance from feedback.

Sources: [choice](ere_rule_choice_learning_findings_20261004.md),
[composition](ere_rule_composition_learning_findings_20261004.md),
[relational learning](ere_rule_relational_learning_findings_20261004.md),
[raw encoder negative](ere_graph_encoder_learning_findings_20261007.md),
[exact-gradient diagnostic](ere_graph_encoder_exact_gradient_findings_20261007.md),
[native transfer](ere_graph_encoder_native_confirmation_findings_20261007.md),
[maintenance](ere_rule_maintenance_findings_20261007.md),
[prevention transfer](ere_rule_prevention_transfer_findings_20261007.md).

### 9.4 Resource allocation and the last unlaunched adapter

A resource-choice fixture established informational headroom: state-dependent
utility 12.5 exceeded constant harvest 11 and constant defense 8. The bounded
four-seed, two-arm, 100-update campaign then completed all 7,440 trajectories,
238,080 advances, 7,440 Seal initializations and 800 updates. Full confirmation
averaged 10.9609375 versus CuesMasked 10.8125. The positive difference did not
beat constant harvest or the 11.5 threshold; 0/4 Full replicates beat both
controls. Only one of three local criteria passed.

A saved-receipt diagnostic found representational capacity but interfering
feature coordinates. A default-off contrast adapter using `[1,f,b,f*b]` passed
rank, sign and derivative checks with zero new native or optimizer work. It was
not a new learning result. Its signed inputs require a versioned funding-explicit
runner; the old runner's `input[1]` funding assumption is incompatible. That
runner/campaign remains unlaunched under this closeout.

Sources: [allocation](ere_defense_allocation_findings_20261007.md),
[resource campaign](ere_resource_learning_campaign_findings_20261008.md),
[gradient diagnosis](ere_resource_gradient_diagnostic_findings_20261009.md),
[contrast preflight](ere_resource_contrast_preflight_findings_20261009.md),
[profile admission history](ere_complete_profile_gate_status.md).

## 10. Evolving Minds and constructive chemistry

### 10.1 Paid developmental organisms

Evolving Minds implemented paid construction, development, action, reproduction,
mutation, death and exact continuation with semantic birth identities. Its
controls separated request charges, copying, escrow/refunds, child endowments
and dissipation. Complete-tick and live recovery were engineering achievements,
not evolutionary passes.

The initial five-instruction offspring could not complete development on the
12-unit newborn reserve. A separately declared parent-funded profile enabled
24-unit children while charging the full endowment. Supplying a useful founder
edit then exposed descendant variation in four fixed worlds; SourceKernel
descendant harvest was lower than faithful copying in every world. Route
reconstruction found 31 descendant proposals: 12 syntax rejections, five paid
unreachable births, ten unchanged reachable children that reproduced, and four
reachable edited children with no paid reproductive request by the horizon.
Right-censoring was retained.

A later fixed neighborhood calibration exposed 140 substitutions plus four
unchanged targets on two worlds. All early targets were reached; 84 edits were
syntax-rejected, 40 paid but developmentally unreachable, and 16 reachable
edited targets remained. Eight reproducing State-input edits had the same
observed behavior as matched controls despite different internal signals.
Their genotype bytes propagated through three paid transmissions. This is
finite inherited neutrality, not adaptive improvement. H3a/evolutionary and
fairness admission remained unestablished.

Sources: [plan review](evolving_minds_plan_review_2026-10-01.md),
[H1/H2 decision](evolving_minds_h1_h2_decision_20261003.md),
[funded admission](evolving_minds_funded_admission_findings_20261003.md),
[paid handoff](evolving_minds_paid_handoff_findings_20261004.md),
[route diagnosis](evolving_minds_descendant_route_findings_20261004.md),
[neighborhood](evolving_minds_paid_descendant_neighborhood_findings_20261005.md),
[behavior/propagation](evolving_minds_descendant_behavior_findings_20261005.md).

### 10.2 Constructive chemistry

CC-0 fixed compatibility and physical accounting. CC-1 passed 41 scripted cases
and 82 worker variants for a finite three-site A–B bond system. Forming a bond
consumed activated fuel, stored two quanta in the bond and dissipated two as
heat. Conservation included reservoirs and carrier material. Exact checked
staging preserved graph, allocator, ledger and frontier state on rejection;
prepared commits were bound to world identity and exact before-state.

This established bounded atomic chemistry engineering. No stochastic kinetic
clock, reproducing chemical ecology, durable CC-2 continuation or evolutionary
capacity was established. CC-2 remains proposed, not implicitly authorized by
the closeout. Sources: [CC-0](constructive_chemistry_cc0_findings_20261005.md),
[CC-1](constructive_chemistry_cc1_findings_20261005.md).

## 11. The final cumulative-evolution experiment

### 11.1 Purpose and consolidation

The final experiment consolidated useful components from the earlier lines into
one falsifiable question: can endogenous paid editing acquire successive useful
organizations whose benefits depend on earlier acquisitions and persist through
paid descendants? It replaced proxy novelty with native functional interventions
and a fixed inheritance requirement. It retained the substrate and borrowed
components while defining a separate explicit finite world law.

Readiness progressed through authored capacity/accessibility, shared-resource
population dynamics, lineage interventions, mutable editor regulation, durable
journaling, prospective sizing, measurement, compact storage, funded dispatch,
native admission, controller integration and exact launch bindings. Failed early
fixtures and source/packaging corrections remain in their original archives.
These stages are engineering prerequisites, not independent scientific replicates.
The [original proposal](final_cumulative_complexity_experiment_proposal_20261009.md)
and [launch review](cumulative_scientific_launch_review_20261011.md) retain their
historical dates and original authorization state.

### 11.2 Frozen design

| Design element | Frozen setting |
|---|---|
| Independent unit | 47 matched seed blocks |
| Arms | Heritable endogenous editing, external editing, nonheritable editing, frozen |
| Complete denominator | 188 worlds; five censuses × two slots = 1,880 slots |
| World | Two stage-0 founders, capital 40, shared reservoir 512, two parcels/tick, eight sites, 16 ticks |
| Bounds | At most 32 retained cells and eight genes; finite stage repertoire |
| Censuses | Ticks 0, 4, 8, 12, 16, retaining living and dead sampled cells |
| Diagnostic context | Four cycles, environment 512, fixed seed 2026100903, capital 40, editing frozen and copy funded |
| Pair definition | Two distinct retained nonfounder stage acquisitions, up to three pairs per sample |
| Inheritance | At least two paid transmissions and two distinct active descendant generations for each acquisition |
| Functional evidence | Paid acquisition gain, removals, padded interaction controls, sham and restoration |
| Primary inference | Exact prevalence null .2, alpha .025; at least 16/47 endogenous successes |
| Secondary inference | Paired exact alpha .025 plus observed advantage at least .2; primary must pass |
| Invalid/incomplete cases | Qualifying mechanical controls invalidate; unknown work/measurement makes the campaign inconclusive |
| Ceilings | 2M rewrites, three hours funded unit time, six hours workflow, 8 GiB storage |

Thirteen assay roles tested before/after acquisitions, intact programs, ordinary
and padded removals, a single removal, sham and restoration. The later
acquisition had to yield a paid gain with the earlier organization present but
not without it. Exact role/program duplicates could reuse the frozen diagnostic
assay; distinct sampling slots and worlds retained their full denominators.
Assay count is therefore not independent replicate count.

### 11.3 Authorization, execution and verification

The user authorized one campaign on October 13. Its reviewed directory retained
the October 11 suffix because the authorization binds the exact path. The frozen
driver/binary/source closure passed preflight; no rebuild, retry, seed
replacement, extended horizon or altered acceptance rule was used.

| Accounting | Result |
|---|---:|
| Worlds / slots | 188 / 1,880 |
| Unique assays | 111 |
| Complete native units | 189 (188 worlds and one assay batch) |
| World rewrites | 12,238 |
| Assay rewrites | 940 |
| Total, exact lower = upper | 13,178 |
| Funded execution/archive/audit unit time | 440.141 seconds |
| Peak reserved admission storage charge | 243,630,080 bytes |
| Completed archive including inventory | 24,832,427 file bytes |
| Sealed payloads / pinned sources | 9,306 / 219 |
| Independent campaign audit | Pass |
| External archive recheck | Pass, exit 0, zero native work |

The filesystem authorization-copy-to-seal interval was approximately 656.28
seconds; this is observational timing, not a substitute for controller budget
accounting. All authorized ceilings were respected.

### 11.4 Scientific result and failed gates

| Arm | Qualifying worlds | Living / dead slots | Pair observations | Distinct world/event pairs |
|---|---:|---:|---:|---:|
| Endogenous | 0/47 | 351 / 119 | 6 | 3 |
| External | 0/47 | 373 / 97 | 0 | 0 |
| Nonheritable | 0/47 | 352 / 118 | 3 | 2 |
| Frozen | 0/47 | 428 / 42 | 0 | 0 |

All nine observed pairs had funded complete measurements and nonempty ablations.
There were no unknown worlds and no qualifying mechanical controls. Both exact
p-values were 1.0; the frozen report returned **`valid_negative`**.

The endogenous block-002 pair appeared at three censuses. It passed functional
checks, with paid acquisition margins +5 and +13, but had transmissions [1,0]
and active descendant-generation sets [[1],[]]. Repeated census observations
do not multiply its innovation count. Block 037 additionally failed an early
acquisition margin; block 044 failed margins and required functional controls.
All endogenous candidates failed inherited persistence. This is a descriptive
gate diagnosis, not proof of the causal intervention needed to repair it.

The result supports retiring this configuration. It neither demonstrates that
useful edits are impossible nor proves that more time would rescue them. The
limited world horizon, finite repertoire, resources and diagnostic environment
are part of the tested hypothesis, not post-hoc excuses to change its outcome.

### 11.5 Primary evidence identity

- [Manifest](../output/cumulative_scientific_campaign_20261011/manifest.json)
- [Full measured report](../output/cumulative_scientific_campaign_20261011/report.json)
- [Independent audit](../output/cumulative_scientific_campaign_20261011/audit.json)
- [Authorization](../output/cumulative_scientific_campaign_authorization_20261013.json)
- [Preflight receipt](../output/cumulative_scientific_launch_preflight_20261013.json)
- [External recheck receipt](../output/cumulative_scientific_campaign_20261013_recheck.json)
- [Archive inventory](../output/cumulative_scientific_campaign_20261011/output_sha256.json)

Inventory SHA-256:
`7d49f529fa597678086fa23c354dbf9897d90dc285fb75158656993406f17f52`.
The inherited measurement helper's `development_only` label and false nested
authorization/engineering flags are preserved legacy wrapper fields. Actual
authorization is recorded in the outer campaign manifest/controller; the
versioned scientific report supplies the decision. Scientific acceptance and
evolution-demonstrated flags remain false, consistent with the negative.

## 12. Cross-project interpretation

### What the overlapping experiments actually taught

The programmes repeatedly encountered the same **interfaces**, rather than
establishing one universal failure mechanism:

| Interface | Recurring lesson |
|---|---|
| Representation → execution | Syntactic validity, decoder completeness and live installation are separate |
| Execution → useful behavior | A new class, firing count or larger graph need not improve function |
| Useful behavior → fitness | Contention, rent, copying, retirement and funding determine descendants |
| Fitness → durable innovation | Survival or faithful copying alone does not show successive useful inherited acquisitions |
| Measurement → interpretation | An engaged proxy can measure the wrong quantity; ratios and censored denominators need explicit interpretation |
| Engineering → science | Accounting, replay and fault refusal make a claim testable but cannot make its scientific criterion pass |

This explains both the apparent repetition and the real progress. Instrument
repairs prevented incorrect conclusions; controlled capacities accumulated.
However, repeatedly making a component testable did not establish the final
autonomous evolutionary conjunction. The final campaign supplied a stopping
boundary instead of another component milestone.

### Claims supported at closeout

Hypersim is a capable deterministic workbench for these bounded experiments.
Its implemented worlds can support replication, conditional selection, useful
learned control, paid executable edits, repair, prevention and finite inherited
function under the respective authored conditions. Several narrow negative
results are also established under their registered criteria.

### Claims not supported

The project has not demonstrated open-ended evolution, generally increasing
complexity, an autonomous self-improving rule ecology, a universal causal
emergence property of life, or a universal Hypersim complexity ceiling. It has
not shown that combining all successful mechanisms would solve the failed
conjunction. There is no theorem over every possible graph, rule family,
resource law, scheduler and initialization expressible in Hypersim.

An exhaustive bounded reachability analysis could, in principle, prove a result
for a precisely fixed transition system or find a replayable capacity witness.
It would not prove a universal ceiling, and a solver-chosen path would not be
spontaneous evolution. That option was discussed and **not undertaken**.

## 13. Closure, preservation and documentation corrections

The user accepted wrapping up after the final valid negative and requested this
project-wide record. This closes the current experimental programme as an
allocation decision, without rewriting any prior scientific verdict or deleting
its infrastructure. Pending proposals remain historical possibilities, not an
active backlog or permission to run them.

| Pending or misleading continuation | Closeout treatment |
|---|---|
| Final cumulative longer runs or new seeds | Not launched; configuration retired at the registered endpoint |
| V24 shadow resolution / further selected edits | Unperformed proposals; observational findings retained |
| ERE contrast runner and new learning campaign | Engineering adapter retained; no launch |
| Evolving Minds H3a or expanded ecology | No evolutionary admission; no launch |
| Constructive chemistry CC-2 | Specification direction only; no kinetic/evolutionary result |
| Locality closure-defect rescue or broad generality campaign | Unresolved; do not adopt outcome-selected strata |
| Cognition F6 / validation V10 / equivalent alphabet sweeps | Closed/no-go under their recorded decisions |
| Universal ceiling proof/search | Discussed, not implemented or measured |

Several historical navigation statements needed explicit correction:

- The root README described ALife/cognition as “reserved”; those packages now
  exist. It also described the ordinary app's engine boundary too absolutely.
- The architecture map's original 292-manifest census is a dated September 4
  audit, not a current whole-project count. Its early selfmod and cognition
  tables precede materially different LK results.
- Older selfmod V8–V10 documents retain preregistration-era “in progress”
  headings alongside later results. Read their results and subsequent
  findings; do not infer current execution solely from those headings.
- Early reproduction-instability explanations, adjacent-seed separation claims,
  population-cap interpretations and repair-safety claims were corrected by
  later controls. Their original records remain part of the research history.
- “Next” sections in older documents describe then-current decisions. This
  closeout takes precedence for programme allocation, without modifying their
  frozen contracts or authorizing their proposed follow-ups.

The root README and current architecture/status entry point now link this report
and the complete catalogue. Historical findings, contracts, manifests and sealed
archives remain untouched. No source-engine or scientific-protocol change is
part of this documentation closeout.

## 14. Preservation and reproducibility handoff

Retain the experiment findings with their preregistrations, versioned configs,
source/binary pins, failed attempts, corrections, seed lists, checkpoints,
allocator state, journals, audit receipts and seals. Preserve raw and compact
evidence according to each archive's verified storage protocol; do not remove
failed variants merely because a successor passed. Copies, audit supplements,
replays and continuations remain separately identified.

For the final campaign, a read-only recheck is:

```sh
python3 -B output/cumulative_scientific_campaign_20261011/source_snapshots/experiments/cumulative/scripts/recheck_scientific_campaign.py \
  output/cumulative_scientific_campaign_20261011 \
  7d49f529fa597678086fa23c354dbf9897d90dc285fb75158656993406f17f52
```

Run from the repository root. This performs integrity checks and independent
reconstruction, not native execution. Historical LK checks require the exact
matching archived sources and possibly external historical run files; see the
integration guide rather than invoking a blanket workspace test suite, since
some tests launch substantial experiments.

The documentation catalogue can be regenerated with
`python3 -B scripts/build_project_closeout_inventory.py`. Its hashes identify
the documentation/metadata snapshot at generation time, not an immutable seal
of all historical payloads. The accompanying documentation validation receipt
records link checks and agreement with the saved final-campaign counts. It does
not promote this retrospective synthesis into an independent new experiment.

### Documentation verification and known availability gap

The closeout inventory covers 505 first-party Markdown files, 301 immediate
evidence directories and 2,843 named metadata files. Of the metadata files,
2,835 were parsed and hashed; eight larger than the inventory's 4-MiB read bound
were recorded by path/size only. Those omissions are explicit in the JSON
inventory and are not claimed as fresh audit coverage.

All file links in the new report, catalogue and current navigation pages resolve.
The historical Markdown scan found one missing local target:
`experiments/lk/docs/gge1_findings.md` points to
`../runs/ge/gge1/gge1_report.txt`. The original report is present in the
[external preserved LK archive](/Users/kalewicker/Documents/hypersim-lk/runs/ge/gge1/gge1_report.txt).
Its existence was checked; its full experiment was not rerun. The old link and
source finding are preserved, and this note supplies the location correction.
Bare code-formatted paths, external web URLs and historical claims are not
implicitly verified by a Markdown link check.

The final campaign's complete 9,306-entry inventory and every payload hash were
rechecked during documentation validation and remained unchanged. Its saved
independent scientific audit/recheck receipts and reported counts also agree.
Run `python3 -B scripts/validate_project_closeout.py` after regenerating the
catalogue to reproduce these documentation/integrity checks with zero native work.
