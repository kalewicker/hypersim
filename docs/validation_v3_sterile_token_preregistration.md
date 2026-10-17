# V3 / G-track — G1 structural results + G2 pre-registration (committed BEFORE any CE)

Plan: `Downloads/validation_v3_sterile_token_plan.md` (the sterile-token control).
This document is committed **before any CE or permutation p-value is computed on the
token** (G2, rule 1). It records: (G1) the token's structure — WL class count, group
sizes, transition count — measured by a CE-free probe; and (G2) the matched
configuration and the unavoidable mismatches, frozen here so no downstream choice can
be outcome-driven.

Harnesses:
- G0 anchor: `crates/hg-sim-core/tests/f7_multiseed_boundary.rs` (reproduced, below).
- G1 probe (CE-free): `crates/hg-sim-core/tests/g_sterile_token_probe.rs`.
- G3 comparison (computes CE): to be added **after** this commit.

---

## G0 — Fidelity anchor (reproduced on current code)

`f7_multiseed_boundary` passes unchanged on the current tree: seed 12345 at WL depth 2
gives **CE = 0.1633, k_carrier = 4, M = 200, null≥real 0/200**; all 30 pre-registered
seeds p < 0.05; cross-seed sign test **p = 1.86 × 10⁻⁹**; no reversals. The baseline the
reversal was established on has not drifted. The token work is added to this exact state.

---

## The construction (no `src` change, no pipeline change — rule 3)

The sterile token is built **entirely in the test harness**, so the F8 scoring pipeline
(WL depth 2, `TpmAccumulator`, `canonical_order`, size-preserving permutation null) is
reused byte-for-byte:

- **Remove replication:** the compiled rule named `replicate` is filtered out of the
  rule set before the engine runs. Everything else in `alife_base.toml` — `bootstrap_seed`,
  `harvest`, `metabolise`, `die_and_recycle`, `influx` — is identical to the real carrier.
  The token therefore has the carrier's full energy economy and ∅-transitions (birth via
  bootstrap, death via `die_and_recycle`) and differs from a real carrier in exactly one
  rule (G1.1, the axis under test).
- **Bootstrap bloom:** `N` `carrier_marker`s are planted with the *same* deterministic
  ChaCha8 picker the production single-marker path uses (`seed ^ 0x00CA_111E`, same node
  order), so **N = 1 reproduces the production marker exactly**. The carrier arm (N = 1,
  replication on) reproduces F8's structural facts through this harness — verified
  (M = 200, k = 4) — proving the setup is faithful before any token number is trusted.

Bootstrap is the token's only birth source (no replication), so the token blooms once and
cannot re-seed: a pure bloom-and-decay transient.

---

## G1.2 / G1.3 — token structure across bloom sizes (CE-free, seed 12345)

Carrier arm for reference: **M = 200, k_carrier = 4, transitions = 421 302, peak pop = 36.**

| n_markers | M  | k_tok | transitions | peak pop | peak carrier-class sizes | trajectory (step:pop) |
|---|---|---|---|---|---|---|
| 1  | 15 | **1** | 216 084 | 1  | [1]            | 1:1 … 50:1 200:0 |
| 4  | 26 | 2 | 216 444 | 4  | [3,1]          | 1:4 … 50:4 200:0 |
| 9  | 29 | 2 | 217 044 | 9  | [6,3]          | 1:9 … 50:9 200:0 |
| 16 | 40 | 3 | 217 884 | 16 | [8,7,1]        | 1:16 … 50:16 200:0 |
| 25 | 40 | 3 | 218 964 | 25 | [12,11,2]      | 1:25 … 50:25 200:0 |
| **36** | **23** | **4** | **220 284** | **36** | **[16,16,2,2]** | 1:36 … 50:36 200:0 |

Findings:

1. **G1.2 gate (carrier classes > 1): met for all N ≥ 4; the *matched* value is N = 36,
   which resolves into exactly 4 token WL classes — identical to the real carrier's 4**,
   via the same site-induced mechanism (corner/edge/interior sites of the 6×6 crystal;
   the [16,16,2,2] sizes are the site-type multiplicities). N = 1 fails the gate (a single
   token is one class) and is excluded, as the plan requires.
2. **G1.3 gate (transitions in F8's power range): met at every N** (≈216–220k). The count
   is dominated by substrate self-loops (≈36 nodes × 6000 steps), so it clears the F8
   extinct-seed floor (~220k) structurally. The token's 220k matches the **extinct-seed**
   carrier transitions (~220k), which is the correct comparison arm since the token also
   goes extinct.
3. **Extinction is structural, not tuned.** Every bloom decays to zero by ~step 150: a
   token is pinned to its single birth-site, influx (0.34) < maintenance (0.5), so once it
   drains that site it starves, and without replication it cannot colonise a fresh one.
   This is the peel-one-property result made concrete — removing replication removes
   persistence as a side effect (the plan's "persistence was removed by the extinct seeds"
   rung, here re-derived mechanistically).

---

## G2 — the matched configuration (FROZEN before CE)

**Matched config:** F8 config verbatim, plus `N = 36` planted markers, minus the
`replicate` rule. WL depth 2, STEPS = 6000, N_PERM = 200, seeds 1..30 — **all identical to
F8**. No scoring window; the token is scored over the same 6000-step run as the carrier
(introducing a window would be a forbidden pipeline change).

**Why N = 36 (justified by matching, not by outcome — rule 1):**

- Matches the real carrier's **WL class count** (k = 4 = 4) — the axis G1.2 names.
- Matches the real carrier's **peak population** (36 = 36).
- It is the **maximal** bloom (every substrate node marked), not a tuned interior value,
  so it gives the token its **greatest** structural richness and its **best** chance of
  registering as detectable. For a control whose job is to potentially *deflate* the
  carrier claim, maximal token power is the conservative choice: if a token this
  well-matched still cannot be told from an arbitrary partition, the deflation is strong;
  if it separates as cleanly as the carrier, "replication boundary" dies honestly.

**The unavoidable mismatches (reported, not engineered around — rule 4, G2):**

1. **Population trajectory.** A replicating carrier *sustains*; a sterile token *decays* to
   extinction by ~step 150. The two cannot be matched on trajectory. Per G2.1 both arms are
   scored over their transient, and the token's transient is shorter. (This is *why* seed
   replication could not settle the question — the construction is what is held constant.)
2. **Total WL resolution M.** Token M = 23 vs carrier M = 200, because replication grows
   the grid (children on fresh nodes → more distinct WL classes) and a sterile population
   cannot. Consequently the token's permutation-null support is **C(23,4) = 8 855** vs the
   carrier's C(200,4) ≈ 6.4 × 10⁷. Both are genuine nulls — 8 855 is three orders of
   magnitude above the ~10-member null that made the F0/V1 test unable to return a positive
   — but the token's is smaller, and **the token's null support at its own resolution will
   be reported alongside every token p-value** (G3.1, and the explicit forbidden-response:
   no separation reported without it).

**Explicitly forbidden (rule 1, and restated so the commit binds it):** choosing N, the run
length, or any scoring window by its effect on the token's p-value or CE. N = 36 is frozen
here by the matching argument above, before the G3 harness that computes CE exists.

---

## Pre-committed interpretation (written before running G3)

- **Token p ≥ 0.05 across seeds while carrier p < 0.05** ⟹ replication-specific structure
  exists; **"replication boundary" survives** (H_bio for the replication axis). Strongest
  positive the program has produced.
- **Token p < 0.05 comparably to the carrier** ⟹ replication is not what the boundary
  detects; the object is birth-and-death-with-site-structure. Rename to **"matter-turnover
  boundary."** Publishable deflation; H_construction not refuted.
- **Token intermediate / seed-dependent** ⟹ report the distribution; claim partial
  replication-specific structure, quantified.

The single sentence G4 must answer: **whether removing replication, and nothing else,
removes the boundary's detectability.**
