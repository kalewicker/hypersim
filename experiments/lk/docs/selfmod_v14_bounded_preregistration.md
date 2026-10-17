# Selfmod v14 — bounded funded reproduction

Frozen before implementation and new output, 2026-09-27. Conditional mechanism
test on the validated v13 panel, inside isolated LK; no root ERE changes.

## Design

Reuse the exact v13 precision-repaired inputs for seeds 11, 17, 23, 31, 47,
61, 83, 101. Require complete status, passed audit and replay. Start faithful
and named R1 prune from their identical fully paid initial-birth snapshots.
Keep all existing residents, allowance, world, financial parameters and RNG.
No new prefix. Faithful/prune crossed with native ON,
OFF, vacancy8 and replace8: eight conditions per seed, 64 continuations total.

Population bound 8 is fixed prospectively as two thirds of income pool 12 /
rent 1. It exceeds all existing post-birth populations of 5–7. Do not tune it
after results or drop initial residents. A bound below available per-step
income leaves rent headroom; it is not a proof of sustainable competition.

- ON: native full funded fission, identical to v13 ON.
- OFF: skip subsequent funded fission, identical to v13 OFF.
- vacancy8: normal funded births in native sorted root order only while fewer
  than eight live copies exist. Reopen vacancies only through native starvation.
- replace8: same normal births below eight. At eight, each eligible parent
  makes its native faithful child, pays the exact normal 50 transfer from
  capital at least 100, then that parent alone retires. Recycle its remaining
  capital to ledger free pool through an explicit recorded withdrawal and
  delete its rule graph. No parent refund, child top-up, repair, viability
  screen, extra RNG draw or new creation allowance. Newborn is not eligible
  again in the same phase. Birth then retirement allows a temporary ninth
  copy within the atomic meta phase, never more than eight at a phase boundary.
  Parenthood carries member/class/ancestry through retirement; no unrelated
  victim or selective protection is used. Refill surviving allowances after
  all births/retirements in native order. Any free-pool overflow is dissipated
  by ordinary next-step influx. Death/retirement tokens are never counted as
  earned lineage income.

All future offspring faithful. Normal rent 1, income pool 12, influx 20 /
free cap 500, full birth threshold 100 / transfer 50; native budget inheritance
max 200/refill 1, random tie break, match cost 0, canonical match cap 64,
enumeration budget 50,000, world ceilings 500/1,000 and hard node stop 20,000;
mu/abio/p_dup/p_del zero. Native RunConfig stays identical; cap8 is an explicit
admission policy, not a secretly altered financial parameter.

Each continuation lasts exactly 2,000 object steps, including after biological
extinction. Runtime/hard-stop failures stay incomplete. No seed replacement,
horizon extension, policy routing or further scientific campaign in this run.
64 x 2,000 = 128,000 advancing steps: 64,000 repeated ON/OFF controls and
64,000 new bounded conditions. Controls must reproduce v13 byte-for-byte and
are not pooled as independent replications. No new prefix steps.
Preflight: saved corrected v13 engineering checkpoints 7/42, 64 steps per
condition, 16 continuations = 1,024 excluded engineering steps. Verification:
separate-process seed 11 vacancy8 and replace8 for each genotype, 8,000 excluded
steps. Cap and retirement class/stock semantics tested before scientific run.

## Primary criteria

Retain the v12/v13 legacy establishment verdict unchanged for comparison:
executable child, full uncensored run, lineage alive at end and productive
in every final 250-step window; late lineage income at least rent over final
1,000 steps; every seed member alive and productive each late window.
No enumeration skips anywhere; invalid/censored rates null.

The PRIMARY JOINT success for continuing funded reproduction additionally
requires: (1) late lineage income >= late rent + all late selected-lineage
capital recycled on retirement; (2) at least one actual funded birth anywhere
in the culture in EVERY final 250-step window; (3) at least one actual funded
birth from the selected-child lineage in the final 1,000 steps. Apply these
criteria to all policies, including ON/OFF, without relaxing boundary cases.
Replicated joint success requires >=6/8 seeds per genotype/policy. Survival or
initial/transient births alone cannot count as ongoing reproduction.

Retirement withdrawal is an external loss to the tracked lineage. Its funds
join the free pool, with no assignment of later pooled income to their source.
Record lineage capital at the step-1,001 start and final phase boundary;
independently reconcile capital change = income - rent - retirement returns
- residual starvation returns (within-lineage birth transfers cancel).
The >=0 late-flow criterion may reject an otherwise stable replacement cycle
when boundaries fall at different points in its 50-token funding cycle.
Report these phase-sensitive deficits, stock floors/ranges, productive/birth
windows and bounded capital oscillation explicitly rather than retuning.
A negative finite-window result is not proof of permanent biological failure.

Report legacy establishment and joint success separately; per-seed gains/losses,
cycle and lineage survival, per-copy productivity, budgets/gates, births each
late window, eligible/admitted/blocked counts, recycled retirement capital,
financial reconciliation, direct-root deaths, capital floors and population
bounds. Parent retirement is distinct from starvation, not a failure of the
native child/parent finance ledger. Reuse makes this conditional evidence, not
fresh-seed replication or open-ended evolution.

## Validation and provenance

Freeze archive of ALL compile-time protocol dependencies, binary, protocol,
control/checkpoint inputs and hashes before evaluation. Exclusive fresh output,
streamed raw receipt, saved births/final states, failure status and full input
provenance; preserve historical outputs/import provenance unchanged.
Native funded fission keeps its ordinary path when no bound is supplied.
Bounded policies share the native mutation/transfer/inheritance body. Test
native parity, admission below/at capacity, normal donor payments, parent
retirement conservation, transient cap9/boundary cap8, immutable genealogy
under slot reuse, stable RNG counts (14 words/birth; none for suppression or
retirement), checkpoint replay and null censored verdicts.
Independent auditor reconstructs every raw root's money, gate, genealogy,
creation charging/refill, eligibility/order/cap, retirement and capital return,
late lineage stock-flow, both verdicts and continuing-birth windows. Reject
corrupted costs, retirements, victims, stocks, RNG counts, cap admissions and
reproduction verdicts. Exact ON/OFF control parity and four bounded replays
are required before validated interpretation. Native matcher/classification,
mutator and numerical ChaCha values are tested/replayed, not reimplemented.
