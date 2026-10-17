# Evolving Minds H0/H1 engineering fixture v1

Date: 2026-09-29 (America/Phoenix). Profile `evolving-minds-engineering-v1`.
The user requested starting implementation of the adapted plan. This contract
defines bounded engineering verification, not an evolutionary assay or training.

## Frozen physical laws

- One shared store; a ring of 3–64 labelled resource sites, one common founder
  seed (anchor, sensor, instruction), and an explicit birth-ID counter.
- The authored instruction is exactly one of `Idle` or `BuildRelay`. No mutation,
  optimized policy, evolved genotype, memory claim, or general D0 language exists.
- Labels `0xE1000000..0xE1000007`, relations `0xE1000064..0xE100006B`, and
  property keys `240001..240010` are profile-local. Key 1 Float is metabolic
  energy. `NodeData.energy` is not metabolic reserve and stays zero.
- Sites initially hold 32 energy each; founder holds 24. Influx adds 8 at sites
  with reserve <=56 (cap 64). No external refill of organism reserves.
- BuildRelay costs 2 energy, creates a connection and actuator, and preserves
  the common seed. SOURCE(sensor,connection) and TARGET(connection,actuator)
  define directed influence through role-labelled incidence, not endpoint order.
- Sense reads the occupied site's energy>=8. Signal copies that Boolean through
  the owned relay. An active actuator transfers 8 from its site when anchor
  energy<=40; acquisition is conservative. Idle has no action circuit.
- Maintenance costs 1 per owned node per environmental tick: seed 3, relay 5.
  It is obligatory, ages the anchor by 1, and drains to zero if underfunded.
  Genome mutations cannot change prices; this profile has no mutation path.
- Reproduction requires developed parent, age>=2, reserve>=32 and a vacant
  adjacent site. Transfer 16 to child; charge 2 construction dissipation.
  Child gets a faithful copy of the single instruction and a fresh common seed.
  Child birth IDs come from a matched counter. That deliberately serializes
  births to at most one per tick in this correctness fixture.
- Death occurs at age>=16 or reserve<=0. Remaining reserve transfers to the
  occupied site; all owned internals are deleted; anchor becomes a zero-energy
  tombstone then is removed in cleanup. Recycling can temporarily raise a site
  above 64; influx remains disabled until it falls below its guard.
- Phases: environment, sense, signal, development, action, maintenance, birth,
  death, cleanup. Each is one real MaxParallel engine step with an explicit
  static rule table. Newborns start age0 and pay maintenance from the next tick.
  Development after signal means a new relay first acts on the following tick.
- All intra-phase application/conflicts use the existing engine. No observer
  writes graph state. Every relevant owned-node binding has owner checks.

## Accounting and correctness gates

Per phase and tick: total reserve change equals site influx minus development,
offspring-construction and maintenance dissipation. Float amounts are integer
quanta, exactly representable at these scales. Transfer and recycling have no
sink. Required property mismatch/skip, negative reserve, malformed ownership,
duplicate birth ID, invalid occupancy or lineage, material/matching truncation
are errors; no fallback/reseeding/culling.

The JSONL trace includes initial material and every phase's full tagged graph
and native events; CSV includes completed-tick counters and energy totals.
Checkpoints contain graph/allocator, engine state/RNG/table, both clocks,
configuration, instruction, ledger/lineage and accumulated trace/rows. Only
completed environmental-tick boundaries are supported; partial-phase or
arbitrary-observer/policy restart is rejected rather than promised.

The independent Python checker recomputes costs, influx, transfers, age,
birth/death identities, ownership/topology, maintenance engagement, faithful
inheritance and coverage from the trace. It does not reuse producer ledger sums.
It is a semantic/accounting checker, not a replacement full matcher replay.

## Limits and interpretation

Default: 16 sites, 64 ticks, 256 nodes, 512 edges, 100000 match work and 10000
match results per phase, 64 MiB per run output. Advancing CLI runs have a
60-second wall ceiling checked between ticks; compilation/serialization/audit
are measured separately. Trace retention is capped at one quarter of output
allowance to reserve checkpoint/trace duplication space. Config rejects >1024 ticks,
>64 sites or output outside 1–256 MiB. Safety caps end the run as interrupted with a
failure reason; they do not alter ecological dynamics. Exclusive output creation.

Active-vs-Idle is a supplied mechanism positive/negative control, not evidence
of evolution or a fair direct-vs-developmental encoding comparison. Site access
remains deterministic, genotype length one, there is no movement, resource
trade-off, lifetime plasticity, generated syntax or signal-cost primitive.
Tick maintenance accounts for material upkeep; signal operations have no separate
event charge in v1. Therefore H3 and broader H2 ecology-readiness remain closed.

All run statuses explicitly keep `scientific_run_launched`, `gate_pass`,
`training_admitted` and `evolution_enabled` false even when engineering checks
pass. Existing ERE/LK gates, contracts, artifacts and runners are untouched.
