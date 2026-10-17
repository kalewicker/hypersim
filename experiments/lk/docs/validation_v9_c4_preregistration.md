# V-track closeout / C4 — confirmatory arm-C sweep, and the last guard gap

**Committed before the C4 sweep runs and before any C4 number exists.**

Two items, independent of C1–C3.

---

## C4.1 — converting the arm-C empty-fraction sweep to confirmatory provenance

### What is already established, and with what provenance

Arm C (the dead token) does **not** detect on the structural channel: z = −0.83, p = 0.930 at seed
12345, and **0/29** on the Phase-2 per-arm multi-seed sweep — that part is already confirmatory
(N4.1).

The standing challenge to it is different: arm C's carriers die at ~step 120, so over a 6000-step
window the carrier group is **empty for ~98% of the scored steps**. Is the non-detection therefore an
**empty-group artifact** — a partition well-formedness problem — rather than a fact about arm C?

M2 answered this by sweeping the scoring window so that the empty fraction ranged 8% → 98% and
finding the score **identical throughout**. That is strong, and it is **exploratory**: the windows
were chosen after seeing that the pre-registered truncation had failed. C4.1 re-runs the sweep with
the grid and the reading fixed in advance, on a **fresh seed set**.

### Design, fixed now

- **Seeds:** ascending from **201**, until **10 included** or 40 attempted. Disjoint from 12345 (the
  fidelity anchor) and from 1–30 (Phase 2).
- **Window grid, fixed now:** `W ∈ {130, 150, 200, 300, 600, 1200, 3000, 6000}` steps, scored from
  step 1. Realised empty fraction is *reported per cell*, never chosen.
- **Channel:** structural (WL depth 2), the channel on which arm C's non-detection was claimed.
- **Cell inclusion**, both clauses required, so that an uninformative window cannot be read either
  way (the M2.2 lesson — a truncated window whose own null is degenerate resolves nothing):
  1. **positive control:** arm A detects at that same `W` and seed. If arm A has no power at `W`,
     the cell has no power and nothing is concluded from arm C there.
  2. **null-spread guard** on arm C's null at that cell (`sd > 0`, `max − min > 0`, ≥ 10 distinct).

### The pre-committed reading

Let `f` = fraction of included cells in which arm C **detects** (p < 0.05), and `ρ` = Spearman rank
correlation between a cell's realised empty fraction and arm C's z, pooled over included cells.

- **`f ≤ 0.05` and `|ρ| < 0.5`** ⟹ arm C's non-detection is **flat in the empty fraction**. The
  empty-group artifact is rejected under pre-registration, and M2's exploratory finding is
  **converted to confirmatory**.
- **`f > 0.05` concentrated at LOW empty fraction, or `ρ` strongly negative** ⟹ the artifact reading
  is supported: C's non-detection is a well-formedness problem, "sustained structural activity"
  weakens to "a populated carrier group is necessary", and that must be reported as the finding.
- **Fewer than 5 included cells** ⟹ **vacuous**; report as such and change nothing.

`ρ` is directional by construction: an artifact makes z *degrade* as the group empties, so the
artifact branch owns the negative sign. This is fixed before any cell is scored.

---

## C4.2 — Gap 2: a firing test for M4.4's null-validity check

The M4.3 guard audit logged two gaps. **Gap 1 is closed** (N4.2: the null-spread inclusion clause
fired 29 times on arm D's structural channel — the positive firing case that was missing).

**Gap 2 remains:** M4.4's null-validity check — *the two inert traits must be ~50/50 in sign, else no
selection verdict is read* — passed on the real run because the null happened to be clean (43%/53%).
There is no constructed case in which it **rejects**, so by V8 rule 5 it is an untested guard, and
§6.1's pattern is precisely a guard that passes while the condition it stands for is unexamined.

**The fix:** the check is a pure function of the displacement vectors, so it is driven into rejection
on **constructed input**:

1. a synthetic displacement set with an inert trait sign-skewed to 80% down ⟹ the check must
   **reject** (`null_clean == false`) and the verdict must be **withheld**;
2. a synthetic set at the 40%/60% boundary ⟹ **accept**, so the test pins the threshold from both
   sides rather than only showing that some extreme input fails;
3. the **real** run's inert balance ⟹ still **accept**, so the fix does not silently invert the
   published reading.

Asserted in `m4_4_selection_power.rs`, beside the guard itself.

---

## Report back

- **C4.1:** included cells, realised empty fractions, arm C's z and p per cell, `f`, `ρ`, and which
  branch of the pre-committed reading the data selects.
- **C4.2:** the guard rejecting constructed input, accepting at the boundary, and accepting the real
  run.
