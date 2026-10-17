# Selfmod v11 — single-mutation spectrum

Frozen design, 2026-09-26, before implementation or assay output. Follow-up to
v10 routing item 1. This is a diagnostic of the existing 15 mutation operators,
not a new evolution campaign, probation mechanism, ERE experiment, or engine
integration. Historical v10 claims remain historical and unchanged.

## Question and measurements

Which single edits of the three `churn3` seed members produce genuinely changed,
executable and persistently productive variants under the corrected instrument?
Distinguish inapplicability, encoded-only/unchanged executable material,
malformed material, no matching opportunities, sterile firings, loss of cycle
function, starvation, and resource-censored execution.

A member of an autocatalytic cycle is not expected to sustain itself alone.
Therefore use two complementary measurements:

1. **Isolated opportunities:** run the focal parent, normalized faithful clone,
   or mutant alone for one object step on identical unmodified reference worlds
   from steps 0/64/128/192 of the corresponding parent control. Never carry its
   effects into the next opportunity. Use actual `object_step` effect gating,
   ordinary matching/conflicts/apply, not a synthetic matching score. Each test
   has a fresh creation allowance200; it does not inherit a reference member's
   depleted allowance. World ceilings still apply. Invalid focal programs have
   null productivity rates in both measurements.
2. **Cycle replacement:** replace precisely one seed member with the single
   mutated graph, preserving the other two. Run 256 steps with normal influx,
   rent/starvation, decoding, object dynamics, proportional income and creation
   budget refill, but deliberately **without funded fission** or further
   mutation. Malformed roots remain present, pay rent and may starve. This
   isolates variant function from repeated variation and fission capital loss.

The latter is a new no-reproduction diagnostic; it is not a replication of
v10's evolving population or an invasion/establishment test. A variant that
works here is a candidate for a subsequent reproduction experiment, not proof
of evolutionary viability. No endowment or budget is selected after outcomes.

## Frozen panel and resources

- All 15 existing operators, in their existing order; all three seed members.
- Four parameter seeds: `0x5E110001`, `0x5E110003`, `0x5E110005`, `0x5E110007`.
  A keyed ChaCha8 stream per operator/member/parameter seed draws the ordinary
  six-u64 operator-and-parameter block. Ignore the random operator selector and
  apply the named operator exactly once with the five saved ordinary parameters.
  No rejection sampling, viability filtering or replacement of inapplicable draws.
- Eight paired world/scheduler seeds: 1,2,3,7,42,99,123,777. Each of the 180
  operator/member/parameter candidates is evaluated on every world seed:
  **1,440 mutation replacement cells**.
- Eight parent and eight normalized faithful-clone controls: **16 control
  rollouts**. Duplication is decoded and canonically reinstalled so allocation
  position is not mistaken for a clone phenotype. Parent and clone must have
  identical decoded seed programs and exact trajectories on every world seed.
- Operating config: `v6_base_cfg(seed,"churn3",256)`, keyed-random tie breaking,
  `match_cost_coeff=0`, default canonical per-rule truncation, three initial
  copies (one per member), mutation/abiogenesis/stochastic birth/death off.
  Fission is omitted explicitly by the diagnostic loop. Initial rule capital60,
  rent1, proportional pool12, flux influx20/free cap500; world200 nodes/300
  binary edges/4 labels; enumeration estimate ceiling50,000, match cap64,
  world ceilings500 nodes/1,000 edges, initial/max creation budget200/refill1.
- Each reference world is archived once. Parent/clone isolated controls:
  3 members x 8 worlds x 4 opportunities x 2 = **192 one-step tests**.
  Mutant isolated tests: 1,440 x 4 = **5,760 one-step attempts**. Malformed or
  uncompilable rules have explicit skipped/unmeasured isolated observations,
  not zero-productivity measurements.
- Maximum advancing steps: 1,456 x 256 + 192 + 5,760 = **378,688**. Invalid
  isolated attempts consume zero object steps. Every actual count is reported.
  No full optimizer, held-out learning or subsequent evolution probe is run.

Creation/world gates and ordinary truncation are explicit features of the
operating point, not invisible noise. Record them per step. An enumeration
estimate skip is unmeasured for productivity on that opportunity/trajectory;
never treat it as a measured sterile/null result. A hard explosion stop or
runtime error makes a cell incomplete and prevents a cap-free viability verdict.
No retry, seed replacement, horizon extension or post-result cap increase.

## Criteria and interpretation

Precondition: every parent/clone pair is byte-equivalent in its science-bearing
JSON, all flow and candidate-fate identities close, and each parent seed member
has a productive opportunity in at least one reference world. Retain baseline
failures on individual seeds; do not select a surviving subset.

For each mutation distinguish `operator_reported_change`, actual encoded-store
change, decoding, compilation, and changed class identity. Same executable
class is not viable novel variation even if the operator reported a change.

**Bounded functional candidate**, per cell: changed executable class; complete
256-step replacement run; no enumeration-ceiling skips; focal root and all
three original member roots remain live at the end; focal productive firings
in the final128 steps; every final32-step window has some cycle productivity.
This is function under the declared resource gates. Also report whether
creation/world gates bound; do not generalize a gated result to unconstrained
dynamics. Report full-horizon and late-half rates, initial/final capital,
income/rent, focus death time, fixed denominators and parent-paired deltas.

An operator is a **replicated lead** if the same changed candidate satisfies
the bounded functional criterion on at least6/8 paired world seeds. Isolated
parent-relative rates corroborate direct function; they do not supersede the
cycle test. Four parameter draws per parent are descriptive coverage, not an
exhaustive operator spectrum or a statistical generalization over all edits.

Predictions: decoder failures concentrate in reference-invalidating edits;
some true no-ops occur because these parents have no guards or only binary
edges; loss of cycle function need not imply the mutant cannot fire alone.
Whether any changed variant is a replicated lead is deliberately open. A
positive lead refutes the categorical reading that this operator set has no
viable variation at all; zero leads only gives a negative on this sampled panel.

Routing: leads -> bounded reproduction/probation design using the measured
operators; isolated-positive/cycle-negative -> assay complementary role
preservation; mostly malformed/unmeasured -> representation or instrument work
before economics. Do not automatically run any routed follow-up here.

## Evidence and engineering work

Archive the contract, config, initial/reference world stores, parent/clone and
mutated graph material, every parameter block, decoded programs, complete raw
per-step fates/capital/ledger/resource flags, aggregate summaries, errors and
the exact executable/source hashes in a fresh output directory. Refuse overwrite.
Record status before evaluating scientific cells, including partial progress on
failure. A separate auditor reconstructs all counts and verdicts from raw rows.

Engineering preflight is separate: seeds7/42, one parameter seed, 32 steps,
all operators/members (90 mutation cells, four controls, opportunity cuts0/8/16/24).
Its outputs cannot enlarge the scientific panel. Preflight may expose harness
bugs; preserve each invocation and fix the harness before freezing the binary.
Keep all existing mutator semantics, RNG draw counts, production phase defaults,
and `experiments/lk/provenance.json` unchanged. New hashes describe the extension.
