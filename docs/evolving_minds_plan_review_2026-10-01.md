# Evolving Minds plan review

**Implementation follow-up (2026-10-01):** The first resource-choice calibration is now implemented; see [findings](evolving_minds_choice_findings.md). The recommendations below motivated that separate profile; remaining packages are still open.

Reviewed 2026-10-01 (America/Phoenix). This review updates the [implementation
plan](evolving_minds_hypersim_implementation_plan.md); it changes documentation
only. Fixed v1 laws, contracts, implementations and archived receipts are preserved.

## Recommendation

Keep the native Hypersim architecture, fixed physical prices, paid lifecycle,
lineage and independent verification. Change the immediate sequence: define and
calibrate a useful resource-choice interface before committing to a larger
developmental language. Then implement the smallest D0 vocabulary needed to
express and vary those controls.

The existing fixtures establish paid construction, reproduction, accounting,
exact completed-tick continuation and observability of a supplied relay. They
provide a sound engineering base. They do not establish that environmental
information improves decisions or that evolution has useful choices to explore.

## Findings and improvements

### 1. Test the value of information, not just the presence of an actuator

**Evidence:** `sense_available` and `action_intake` both require site reserve
of at least 8. The latter also requires the actuator to be true and the organism
to have reserve at most 40. Failed action matching has no separate attempt cost.
See [native laws](../rules/evolving_minds/engineering_v1.toml).

**Implication:** the current comparison removes all intake capability in the
idle control. It cannot distinguish useful sensing from an always-enabled
actuator whose requests are filtered by the physical guards. The expectation
that an always-on control could exploit this is a code-based inference, not a
newly measured outcome. Turning off the signal demonstrates a necessary path
in this authored controller, not the advantage of information over constant action.

**Improvement:** add constant-choice/always-on, random and reactive controls to
a small task with incompatible actions and an actual cost of choosing poorly.
Charge/commit an attempt before outcome resolution; prohibit free retries via
failed matches. Cross informative and permuted/withheld cues. Require a useful
effect against the strongest uninformed control and viable paid descendants
before launching evolution. Keep movement optional until it serves that test.

### 2. Resolve reproductive scheduling before interpreting selection

**Evidence:** every `birth_faithful` binding contains the same global `COUNTER`.
Native conflict resolution therefore allows at most one such birth per tick,
including requests at disjoint sites. Canonical candidate order can affect access.

**Improvement:** test two spatially disjoint births and competing births, swap
equivalent founders, and permute allocation/site labels. Use a versioned native
request/commit scheme to decouple identity bookkeeping from biological
contention, or explicitly retain the global capacity law and measure its bias.
Do not interpret repeatability as proof of unbiased competition. V1 stays fixed
as a regression fixture.

### 3. Make D0 a complete execution and heredity contract

**Evidence:** current `Instruction` is only `Idle` or `BuildRelay`, while
`validate_material()` checks an exact seed/relay topology and corresponding
3/5 upkeep. Adding instruction serialization alone will not admit varied
phenotypes. See [configuration](../crates/hg-sim-core/src/evolving_minds/config.rs)
and [runtime](../crates/hg-sim-core/src/evolving_minds/runtime.rs).

**Improvement:** specify bounded instructions, legal slots, cursor/halting,
energy starvation, failed-attempt work and signal timing together. Keep genotype
graph material authoritative, with checked canonical projections. Generalize
validation to ownership, role, grammar and material-cost invariants. Require
two distinct valid phenotypes and actions plus paid multigeneration copying.
Start with one simple mutation class; add insertion/deletion only after its
copying and failure semantics are verified. Measure local mutation neighborhoods
so the intended behavior is actually accessible from the proposed starting seed.

### 4. Specify failed reproduction as carefully as successful birth

**Gap:** the plan requests mutation records but leaves the order of sampling,
payment, validation, child creation and failure settlement too open.

**Improvement:** define a single-attempt transaction: reserve → sample once →
copy/validate → construct → commit or settle failure. Record stable attempt ID,
RNG stream/draw, parent/proposal bytes, semantic target, charges/refund and
outcome. Distinguish invalid genotype, insufficient reserve, occupied target and
host censoring. No silent redraws, repaired programs or fallback parent clones.
Reset runtime/learned state at birth and charge storage for actual genotype size.

### 5. Extend causal controls through lawful lifecycle continuations

**Evidence:** current probes deliberately stop after Signal and Action at
engine steps 19/20 while environmental tick remains 2. Foreign ownership fails
normal validation. See [probe implementation](../crates/hg-sim-core/src/evolving_minds/interventions.rs)
and [findings](evolving_minds_causal_controls_findings.md).

**Improvement:** keep those malformed-state probes as guard tests. Add a legal
disabled-communication state or paid lesion law for complete lifecycle forks.
Declare intervention timing/duration, washout and cost controls, then measure
resource yield and funded descendants over the same horizon. A two-phase result
does not establish effects on long-run fitness or acquired memory.

### 6. Separate useful adaptation from encoding superiority

**Gap:** H3 combined successful adaptation, useful development, and superiority
to direct encoding in one gate. That obscures what a failure would mean and
expands the first scientific comparison unnecessarily.

**Improvement:** H3a asks whether inherited variation improves on frozen-founder
and unevolved-program controls. H3b compares encodings after a useful mechanism
exists. Begin H3b with equal adult phenotypes on the same physics, expose real
construction/interpreter costs, and characterize mutation neighborhoods. Keep
all invalid random-program proposals in the main denominator; viable-conditioned
reports are secondary. Select a primary endpoint, horizon and budget matching
rule prospectively rather than choosing the most favorable normalization later.

### 7. Bound evidence costs before long experiments

**Evidence:** every v1 phase stores a full graph frame, and checkpoints serialize
all past frames and rows again. The 64-tick active receipt occupies about 12.5 MB.
Long horizons or larger organisms will increase evidence costs substantially;
no scaling factor or long-run throughput was measured in this review.

**Improvement:** preserve full traces for tiny reference fixtures. Before long
runs, introduce a versioned live-state checkpoint with event-log offsets/hashes,
streamed complete accounting/events and periodic snapshots. Prove equality
against the reference mode, including fresh-process resume, partial-write failure
and segment boundaries. Measure peak memory, serialized bytes and audit time as
well as engine time. Output savings must not remove reconstructable evidence.

## Revised order and acceptance

1. Freeze the choice interface and run authored information-value controls.
2. Implement its minimal D0 language with paid development and varied phenotypes.
3. Verify paid copying, single-attempt mutation and failed-birth settlement.
4. Verify lawful lifecycle interventions and reproductive contention/fairness.
5. Validate scalable evidence persistence and a bounded resource pilot.
6. Review H1/H2, then specify the smallest H3a evolutionary experiment.

Contract design and small implementation fixtures can advance together; the
ordering does not require a large new ecology framework. D1/D2, plasticity,
continuous neural operations, GUI expansion and broad morphology remain later
extensions tied to demonstrated needs. This review does not call for replacing
the existing engine or importing a second simulation model.

## Verification scope

Read the current plan, implementation/findings, native rules, configuration,
material/checkpoint runtime and intervention code. Rechecked all 29 file hashes
in `output/evolving_minds_engineering_final_20260929/output_sha256.json` and all
19 in `output/evolving_minds_causal_controls_verified_20260929/output_sha256.json`:
zero mismatches. Read the persisted outcomes rather than rerunning experiments.
Historical source-match counts describe their original verification times;
archive integrity does not claim all live source still matches those snapshots.

No new simulator tests, scientific runs or training were needed for this
documentation review. Proposed always-on, mutation, fairness and long-horizon
tests remain future work. The revised roadmap distinguishes that work from the
implemented v1 evidence.
