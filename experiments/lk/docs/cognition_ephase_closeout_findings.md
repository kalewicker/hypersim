# E-phase closeout — plan v3, complete

Synthesis of `docs/cognition_e0_quantizer_audit.md`,
`cognition_e2_n1_rerun_findings.md`, `cognition_e4_stage8_findings.md`,
`cognition_e3_stage0b2_stage5prime_findings.md` and
`cognition_e5_stage4prime_stage6_findings.md`. Every phase of
`hypersim_nn_experiments_plan_v3.md` §10 has run.

---

## 1. Phase status

| phase | exit | met? |
|---|---|---|
| E0 | every quantizer has a declared observable range checked against its design range | **yes** — all six were mis-calibrated; the two that *saturate* were fixed |
| E1 | Gumbel basin not reproducible under N-1; `conservation_soak` aborts at the static screen; a single-draw gate refuses a verdict | **yes**, all three as assertions |
| E2 | margin survives; the calibrated-vs-mis-scaled gap narrows as predicted | **yes**, both clauses |
| E4 | ≥1 substrate other than `tri_two` carrying the margin | **yes**, three |
| E3 | separation classes with spreads; emergence prediction adjudicated; F6 go/no-go | **yes** — and the separation half is satisfied by an ill-posed statistic (§3) |
| E5 | Stage 4′, then Stage 6; F6 campaign if opened | **yes**; F6 not opened |

---

## 2. What the programme can now claim

> On F5 (constructor reuse), a trained `MlpScorer` beats budget-matched
> random and beats **30/30** untrained directions of its own class on
> **four** substrates spanning three rule pairs and two initial conditions,
> exceeding the best of thirty random directions by **4.9%–10.0%**.

Reliable learning of modest value. The single-substrate caveat that would
have been the headline is gone (E4).

**And what that champion actually does**, on four independent measurements
across three phases:

| measurement | champion vs random | where |
|---|---|---|
| `n_nodes`, `active_persist` | smaller graph, less persistent structure | E4 §5 |
| `ei_micro` | **down** 3.2–7.6 sd | E3 §4.1 |
| lattice depth, task count | **down** 3.1–14.5 sd | E5 §1.1 |
| composition rate, `ce_structural` | **up**, both by denominator collapse | E5 §1.3, E3 §4.4 |

**F5's champions make the substrate do less.** Reuse depth rewards the same
constructor firing repeatedly, and a smaller, more repetitive substrate
supplies exactly that. This is a coherent account of the programme's only
positive learning result, and it is not the account D6 or E2 implied when
they reported it.

---

## 3. The three pre-registered predictions, adjudicated

| prediction | source | outcome |
|---|---|---|
| the margin survives N-1 and the calibrated-vs-mis-scaled gap narrows | plan §4 | **held**, both clauses (E2 §6) |
| champion has higher `ei_micro`, lower `ce_structural` | plan §7.2 | **failed in the confident term** — `ei_micro` *falls* 3.2–7.6 sd; on `tri_two` neither term moves, which is §7.2's own outcome 3 (E3 §4) |
| policy-shaped dynamics populate the lattice at a higher rate | v2 Stage 4 | **rate yes, absolutes inverted** — the champion builds a shallower lattice with fewer tasks (E5 §1.2) |

Pre-registration earned its keep on the second one. §7.2 named three
outcomes in advance and the measurement landed on one of them for `tri_two`
and outside all three for the authored substrates; without the
pre-registration the `ei_micro` fall would have been available for reading
as a success.

### 3.1 F6: NO-GO

Not on cost — the cost checks were never reached. §7.1's chain is F5 →
higher `ei_micro` → (C-4) → lower `ce_structural`, and **the first link is
falsified**. `ce_structural` never falls on any substrate, so there is no
measured suppression of emergence for a corrective objective to correct.
F6 does not enter `fitness_formulas.md`.

Additionally, §7.4's floor is specified against the *noise* route to a higher
ratio; the route measured here is **size**, which a Frozen-gate threshold on
absolute structure metrics does not block.

---

## 4. Stage 0b is retired, not repaired

E3 §2 is the phase's most consequential negative. D4 diagnosed Stage 0b's
defect as an unstable denominator; the fix was built to specification and
made the symptom worse (`triangle_subdivide` 5/8 → **8/8** hard-bound
violations). The actual defect is upstream:

> `(seed, seed + 1)` is called "the perturbed pair", but the seed reaches
> the run only through the policy RNG, and ChaCha8 at 4242 and 4243 gives
> uncorrelated streams. It is an independent draw. Numerator and denominator
> estimate the same quantity.

Measured: P(adjacent < far) = 0.63–0.67 ± 0.12, where the premise needs ~1.0.
**There is no perturbation, so there is nothing to normalize** — and that
accounts for every symptom D4 recorded, including `binary_edge`'s ratio of
15.818.

Stage 0b has **no admissible classes on any rule set**, including
`triangle_subdivide`'s unanimous `Chaotic ×8`, which is eight impossibilities
in a row rather than a class. The C0–C5 separation table stays demoted, with
the reason changed from "pending the §2.5 redesign" to **pending a
perturbation**. E3 §2.3 records what a real one would require.

---

## 5. The recurring defect shape, fourth through sixth instances

The D-phase closeout named it: **a proxy standing in for a fact it cannot
observe**. The E-phase produced a variant sharp enough to name separately:

> **A summary statistic quoted without the spread of the thing it
> summarizes.**

| instance | what it looked like | what it was |
|---|---|---|
| E2 §2.3 | κ = 11.15, "curvature-dominated" | mean-of-ratios at n = 4, dominated by one near-zero denominator; the aggregate is 0.61 |
| E2 §5.1 | 1.85 direction-sd, then 1.24 on more data | a parametric z on a left-skewed distribution, measuring the wrong tail; the answer is 0/30 |
| E3 §2.1 | P(adjacent < far) = 0.67 | 144 comparisons from 12 + 12 runs; ±0.12, i.e. 1.4 sd from a coin flip |
| E3 §4 (first pass) | "failed in both terms 4/4" | 210 correlated windows treated as 210 observations; at n = 5 seeds, `ce_structural` moves on one substrate |

And a second shape, twice in two phases:

> **A ratio rising because its denominator fell.** `ce_structural` (E3 §4.4)
> and composition rate (E5 §1.3). In both cases the ratio was the quantity
> that would have been the headline.

Both shapes have the same remedy and it is cheap: report the denominator,
and report the spread of the unit that is actually independent.

---

## 6. What plan v3 fixed that stays fixed

- **N-1 score normalization** (E1/E2). The policy is now a function of θ's
  *direction alone*, exactly — `‖θ‖ ∈ {1, 7, 42, 420}` produce byte-identical
  selections. `MLP_INIT_L2` is no longer a tuning parameter. The ε floor is
  a declared residual basin at `‖θ‖ ≲ 1`, not a silent one at `‖θ‖ ≲ 3`.
- **The static rule-dependency screen** (E1) turned substrate authoring from
  trial-and-error into a design criterion: D5's nine candidates all aborted;
  two authored rule sets gave **6/6** pairings passing P-1 and 5/6 clearing
  P-1/P-3/P-4 (E4 §1).
- **Cyclic dependency also makes a substrate policy-proof** (E4 §4.2) — no
  genome direction produces a dead run, which removes the dead-direction
  tail that made `tri_two`'s class control unreadable. Not designed for, and
  it should hold for any substrate authored the same way.
- **`genome_l2`** (E2 §6, CSV schema 4) makes the self-annealing that N-1
  creates visible; it is invisible in the fitness column by construction.
- **Gate discipline** — `GATE_UNDERDRAWN` / `GATE_UNSTABLE`, ≥5 draws.

---

## 7. Carried forward, unresolved

1. **The ≥5-draw machinery is built but not wired into P-1…P-4** in the
   campaign runner. E4 ran P-1 at five draws by hand; the runner still takes
   one. E3 §2 and E2 §5 are both arguments for finishing this.
2. **Stage 0b needs a perturbation** (§4), or retiring outright. It gates
   nothing currently, which is why this is not urgent.
3. **The size mechanism behind the `ei_micro` fall** (E3 §4.5) is a
   suggestion at n = 4, not a result. A budget or horizon sweep would test it.
4. **`relay_wn` and `fan_tri`/F3** abort `OBJECTIVE_BLIND` at five seeds.
   Whether more seeds would state them is unknown and cheap to find out.
5. **F4 remains a diagnostic with a documented name mismatch resolved**
   (`f4_monotone_drift`) and no role in any campaign.
6. **`crates/hg-selfmod/`** does not compile, so `cargo test --workspace`
   still fails. Plan v3 risk 6 asked for this before E1 and it was not done;
   `hg-train`, `hg-cognition` and `hg-sim-core` are tested explicitly instead.

---

## 8. Suite at close

`hg-train` **102 passed**, `hg-cognition` **75 passed**, `hg-sim-core` lib
clean, all test binaries building. Long-running phase tests are `#[ignore]`d
and run explicitly, as throughout the programme.
