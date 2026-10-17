# V7 / L-track — presence versus motility: the question is outside the instrument's domain,
# and the instrument dilemma splits in two

Reported against the V7 plan. Harnesses: `l1_diffusion_substrate.rs`, `l1_presence_vs_motility.rs`,
`l2_instrument_validation.rs`. Pre-registrations committed before the corresponding numbers:
L2.1 criteria `847b36c`, L1.1 mechanism `01bf010`. Seed 12345 throughout.

---

## The single sentence

**A carrier that is present but does not move, replicate, or turn over is *not* undetectable — it is
*unmeasurable*: it leaves the graph literally constant, so every structural instrument sees an
identity transition matrix and returns the same number for the true partition and every permutation
of it. What the WL/CE instrument detects is sustained structural *activity*, and arm D is maximally
metabolically active while structurally silent — invisible, not inert.**

---

## L1.1/L1.2 — the specimen's substrate, and the firewall

The static substrate cannot sustain a stationary carrier, and the arithmetic says why: under
`MaxParallel` a site is either harvested or refilled, never both, so a stationary carrier needs
`f >= 1/3` harvest steps while influx alone permits `f <= 0.254`. That inequality **predicts the
G-track token's death at step 120** (−0.125/step from ~15.0) — the mechanism was derived, not fitted.

Churn cannot supply the specimen either: it fires on *unoccupied* nodes and deletes them, so
"refreshing the carrier's site" would delete the site the carrier is attached to. The mechanism is
therefore **energy transport, not node turnover** — a conservative gradient rule `diffuse`
(donor ≥ 2.0, recipient < 1.0, moves exactly 1.0, substrate-label-only, naming no carrier property).
This deviation from the plan's "churn refreshes occupied sites" is recorded in the L2.1 doc, and
stationarity was consequently measured at **both** tiers rather than reinterpreted.

**Firewall (L1.2) holds, strongly:**

| check | result |
|---|---|
| operating point, 0 carriers, with vs without `diffuse` | **per-step energy vectors identical** — the rule never fires there |
| gradient fixture, total energy | drift **0.00e0** — neither source nor sink |
| `n(t)` / edge count | flat 36 / 85 |
| transport character | one-directional, self-limiting (tops a depleted node up; does not equalise) |

`diffuse` is gradient-driven: no gradient, no transport. Inert at the operating point is the honest
characterisation, and it is the strongest possible no-back-reaction statement.

## L1.3 — the ladder, with realized demography

| arm | replicate | relocate | diffuse | peak | last-alive | births | deaths | **relocations** |
|---|---|---|---|---|---|---|---|---|
| A replicating | yes | – | – | 36 | 6000 | 3389 | 3355 | 0 |
| B non-repl motile (static) | – | yes | – | 6 | 6000 | 6 | 0 | **248** |
| B′ non-repl motile (diffuse) | – | yes | yes | 6 | 6000 | 6 | 0 | **6** |
| **D non-repl STATIONARY (diffuse)** | – | – | yes | 6 | **6000** | 6 | **0** | **0** |
| C token | – | – | – | 6 | 120 | 6 | 6 | 0 |

**The specimen is valid**: D is present for the full window with zero deaths and **zero
relocations**, and B′ still relocates on the same substrate, so the D-vs-B′ contrast is
non-degenerate by construction rather than by luck.

## L1.4 — BLOCKED, and the reason is the finding

D scores `p = 1.000` on the WL partition. That number is **arithmetic, not measurement**:

| arm | k_c/M | EI_micro | EI_macro | null min / max | null sd | degenerate? |
|---|---|---|---|---|---|---|
| A | 4/200 | 5.6088 | 0.9157 | 0.0061 / 0.0970 | 1.79e-2 | no |
| C | 2/30 | 4.8141 | 0.9652 | 0.1997 / 0.2077 | 2.27e-3 | no |
| B′ | 2/40 | 5.1060 | 1.0000 | 0.1283 / 0.1958 | 2.30e-2 | no |
| **D** | 2/28 | 4.8074 | **1.0000** | **0.2080 / 0.2080** | **0.00e0** | **YES** |

Arm D's realized null is a **single point**. On such a null *no* partition could ever be detected,
so D's non-detection carries no information about presence or motility. Per the pre-committed L2.1
STOP condition this is reported as **UNMEASURABLE**, not as a negative — and specifically **not** as
the plan's "motility was the operative variable" branch, which the data do not license.

**Diagnosis, measured rather than inferred:**

| arm | WL-frozen fraction | steps with ALife-energy change | relocations |
|---|---|---|---|
| A | 0.3801 | 5999 | 0 |
| C | 0.9998 | 152 | 0 |
| B′ | 0.9998 | 5999 | 6 |
| **D** | **1.0000** | **5999 / 5999** | **0** |

**Arm D is not inert — it is invisible.** Its graph never changes (WL-frozen 1.0000 ⟹ the TPM is the
identity ⟹ `EI_macro = 1.0000` exactly), while its ALife energy changes on *every single step*. WL
hashes `label + node_data.energy` (an `f32` **field**) + structure; the ALife energy economy lives in
**property key 1**, which the Q2 invariant deliberately excludes. Everything arm D does happens in
the one channel WL cannot see.

**This generalises beyond WL, and provably so.** If the graph is constant, *every* structural class
function is constant, so *every* structural partition yields an identity TPM. L2.2's remaining
candidates (carrier-intrinsic attachment degree; the label-anchored/structural-resolution hybrid) are
therefore excluded by argument, not by omission — they are structural, and no structural instrument
can see arm D. They were not run empirically; the search terminated on candidate 0 passing.

## The complete 2×2 — what WL-detectability actually tracks

| arm | structural activity | sustained across window | WL-detectable |
|---|---|---|---|
| A | yes (3389 births / 3355 deaths) | yes | **yes** (p < .005) |
| B | yes (248 relocations) | yes | **yes** (p < .005) |
| B′ | yes (**only 6** relocations) | yes | **yes** (p = 0.010) |
| C | yes (6 deaths) but only to step 120 | **no** | no (p = 0.930) |
| D | **none** (graph constant) | yes | **undefined** (vacuous) |

**Both are necessary.** C has activity without duration and fails; D has duration without activity
and is unmeasurable. The instrument is strikingly *sensitive* — B′ detects on 6 structural events in
6000 steps — but its domain excludes zero.

This narrows the K-track verdict without overturning K1/K2/K3, which stand. "Sustained structural
presence" must be read as **sustained structural presence *with activity***: every arm that ever
detected had structural activity, and the first arm to separate presence from activity is exactly the
arm the instrument cannot see. Whether presence *per se* carries signal is not answered here, and
cannot be answered by any structural partition.

## L2 — the instrument dilemma splits in two

**Criterion (a), biology-discriminating: WL PASSES.** Detects A (p < .005, k_c = 4, support 6.47e7);
dead token `|z_C| = 0.83 < 2.0`, `p_C = 0.930`. Energy-band remains excluded by construction (K3:
z_C = 9.59, ~98% definitional).

**Criterion (b) Tier 1 (gates L1): WL PASSES** — TWTF 0.000 on every arm's own substrate.

**Criterion (b) Tier 2: WL PASSES, which updates H0.** On the **bounded** H1 churn substrate WL is
stationary with churn demonstrably active — bare: 1679 fresh node identities (0.280/step), `n_end`
36, 39 classes, TWTF 0.000; with carriers: 956 fresh identities, `n_end` 40, 231 classes, TWTF 0.000.
H0's ephemeralisation verdict (TWTF 0.515 / 0.984) was measured on **growing** builtins
(`growth_treadmill`, `triangle_subdivide`), so it was a statement about substrate **growth**, not
about node turnover as such. Bounded turnover keeps the WL class registry recurrent.

So **the dilemma as posed dissolves**: WL *is* both biology-discriminating and stable under bounded
churn. But a **different and sharper gap replaces it**:

> Neither instrument can see metabolic activity that is not merely being a carrier. WL excludes
> properties by design (the Q2 invariant, adopted because per-carrier genome properties would mint a
> class per carrier and drive `EI_micro = log2(M)`). The energy-band partition sees properties but is
> ~98% definitional. Arm D's entire dynamics fall in that gap.

That tension is structural, not incidental: including properties in WL explodes the class registry;
excluding them makes metabolism invisible. Energy-band is one coarse-grained middle and it is
dominated by the definitional signal.

## A gap in a standing criterion

The F0 support bar `C(M,k) > 100` **passed** on arm D (`3.78e2`) while the realized null was
degenerate. **Combinatorial support does not imply realized null spread.** Any future scoring should
report the realized null range/sd alongside support, and treat a zero-spread null as unmeasurable.
This check was added mid-track after D's `z` came back `n/a` — it was not pre-registered, and is
reported as a criteria gap rather than folded in silently.

---

## L3 — which branch, and the report consequence

The results select a **fourth** branch the plan did not enumerate, and it is closest to its branch 3.

- **Not branch 1** (D detectable ⟹ presence load-bearing): D was not detectable.
- **Not branch 2** (D undetectable ⟹ motility load-bearing): the test was vacuous, so there is no
  evidence D lacks a signal — only that WL cannot look. Asserting motility here would be exactly the
  overclaim V7 rule 2 and the STOP condition were written to prevent.
- **Branch 3, sharpened**: the presence-vs-motility question is **not measurable by any structural
  partition** on this substrate family — not for want of a partition, but because presence without
  structural activity is the null input for a structural measure. This bounds everything downstream:
  any causal-emergence claim resting on a structural partition is a claim about structural activity.

**Report updates:** §5.8/§9/§10 — state that WL-detectability tracks *sustained structural activity*
(the 2×2 above), that the K-track verdict reads as presence-with-activity, and that H0's scope
narrows to *growing* substrates while bounded churn is now instrumentable with WL.

**Genuinely unblocked by this track:** the churn-substrate re-run of the confound split, which the
K-track listed as unblocked-but-unrun. It can now use **WL** — the biology-discriminating partition —
rather than the definitional energy-band one, because Tier 2 passed with carriers present.

## Scope and honest caveats

- **Single seed (12345)** throughout. B′'s detection is **marginal** — p = 0.010, i.e. 2/200 nulls
  beat it, on 6 structural events — and should not be leaned on without a multi-seed replication.
- L2.2 candidates 1 and 2 were **not run**; they are excluded by the constant-graph argument above,
  which is sound but is an argument, not a measurement.
- The `null_distinct` diagnostic wobbles run to run (floating-point summation order under HashMap row
  iteration); the degeneracy call is keyed on `null_sd` and `max − min`, which are stable at exactly 0.
- `diffuse` is a new substrate mechanic, characterised bare, but it is a mechanic the earlier tracks
  did not have — every L-track statement is about *this* biology plus `diffuse`.
- Magnitude remains out of scope (F2.4); everything here is detectability.
