# hypersim — V2 / F-track: fix plan following the V1 validation findings

**Status of the prior track.** `docs/validation_v1_plan.md` + `validation_v1_findings.md`
completed: V0 5/5, V1 no fork, V2 4/4, V3 boundary not survived, V4 3/3. The instrument is
sound, the pipeline reproduces bit-identically from a clean room, and the biological
transition is structurally real. What failed is that **CE across the carrier/substrate
partition does not resolve that transition.**

**Why this track exists.** The V1 findings took the pre-registered fork branch labelled
*"boundary spurious."* That label is wrong, and the correction is the whole point of this
plan. V2.2 established the `NonTrivial` 0→>0 crossing coincides with a real depth-2
sustained self-copying event, and V0.5 established the classifier can withhold the label. The
boundary is real. What is degenerate is the **macro description**: the partition has group
sizes (9, 1) — a single carrier WL class against nine substrate-side classes — so the macro
is 2×2 with one carrier row, and the permutation null has support of roughly ten options.

A test with that little resolution reporting p = 0.495 is not evidence of no signal. It is a
test that could not have returned a positive. **Before Experiment 6 is abandoned or
re-scoped, the resolution problem has to be diagnosed**, because the leading candidate cause
is a configuration value, not a fact about the biology.

Nothing in this track adds a mechanic, changes the economy, or alters a rule.

---

## Rules for the implementer (binding — carried forward and extended)

Rules 1–5 from `validation_v1_plan.md` remain in force. Three additions specific to this
track:

6. **WL depth is now a knob with large CE leverage. Treat it as pre-registered, not
   tuned.** Raising WL iteration depth raises the micro class count `M`, which raises
   `EI_micro`, which — since CE is `EI_macro / EI_micro` — will most likely **lower** CE. That
   is expected and acceptable. The goal of F0/F1 is *resolution*, not magnitude. Selecting
   the depth that maximises CE is the same forbidden move as tuning influx to open a band, and
   is explicitly prohibited. The depth-selection rule is committed in F1 **before** CE is
   computed at any depth.
7. **Report the diagnostic even when it kills the fix.** If F0 shows carriers cannot be
   split at any depth, that is a finding that strengthens the V1 negative and must be reported
   as such — not worked around by reshaping the partition by hand.
8. **The canonicalisation requirement is now an invariant, not a note.** V3.1 found that
   `TpmAccumulator` assigned dense class indices in `HashMap` iteration order, which is
   `RandomState`-seeded per process, silently corrupting every statistic computed over
   partition *indices* while leaving CE itself invariant. Any new code that ranks, permutes,
   samples, or indexes micro classes must canonicalise on WL hash first, and F3 makes that
   structurally enforced rather than remembered.

---

## F0 — Partition resolution diagnostic (branch point; run before anything else)

The substrate resolves into **8 distinct WL classes**, which can only arise from geometric
heterogeneity in the 6×6 arity-3 crystal (boundary versus interior sites). So sites are
demonstrably *not* all alike. A carrier attached to a corner site and one attached to an
interior site should therefore receive different WL hashes — but only at sufficient
iteration depth. At depth 1 a carrier's neighbourhood is exactly `{site}`, identical for
every carrier; the site's own neighbourhood, which is what differs, only enters the carrier's
hash at depth 2.

Two candidate causes, with different fixes. F0 distinguishes them.

### F0.1 — Report the configured WL iteration depth
Report the exact iteration count used to build the micro partition in the CE pipeline
(`TpmAccumulator` / the WL hashing call feeding it), and whether it differs from the depth
used elsewhere (observers, `hg-observe` metrics, the motif classifier). A depth mismatch
between the partition builder and the rest of the stack is itself worth surfacing.
*Cost:* minutes. *This is the single highest-value item in the track.*

### F0.2 — Carrier class count as a function of depth
On a retained snapshot from the CE run (seed 12345, influx 0.34, bootstrap threshold 3.0),
recompute WL classes offline at depth `d = 1..5` and report, per depth:

| d | total classes M | carrier-group classes | substrate-group classes | group sizes |
|---|---|---|---|---|

*Gate:* carrier-group classes > 1 at some `d ≤ 5`.
*If yes:* the (9, 1) partition was a depth artifact. Proceed to F1.
*If no:* the collapse is not a depth problem — proceed to F0.3 before concluding anything.

### F0.3 — Carrier site-occupancy across substrate classes
If F0.2 does not split carriers, the alternative explanation is that carriers only ever
occupy sites belonging to a **single** substrate WL class, so there is no heterogeneity for
them to inherit. Report the distribution of carrier-occupied sites across the 8 substrate
classes, over the whole run.
*Interpretation:* concentrated on one class ⟹ dispersal is confined to a structurally
homogeneous region, which is a **finding about the dispersal mechanic** (birth-dispersal to
adjacent nodes may never reach boundary sites), not about WL. Spread across classes but
carriers still share one hash ⟹ a genuine hashing/classifier issue; stop and surface it.

### F0 decision tree

- **Split at depth `d`** ⟹ F1 → F2. The V1 headline is provisional and may reverse.
- **No split, occupancy concentrated** ⟹ report as a dispersal finding; the V1 negative
  stands but its scope narrows further (to "carriers confined to one site class"), and the
  forward path is the occupancy question, not CE.
- **No split, occupancy spread** ⟹ classifier/hashing investigation; do not proceed to F2.

---

## F1 — Depth-selection pre-registration (commit before computing any CE)

Write the selection rule into the repo **before** running F2. Pre-committed rule, in
priority order:

1. Choose the **minimum depth at which the carrier group resolves into more than one class.**
   This is principled (it is the coarsest description that can represent the distinction under
   test) and it is decided by F0.2's table alone, which contains no CE values.
2. Additionally report the **WL convergence depth** — the depth at which total class count
   `M` stabilises — as a secondary reference point, and run F2 at both if they differ.

Explicitly forbidden: selecting depth by inspecting CE, by p-value, or by null margin.
Record the chosen depth and the rule that chose it in the findings before F2 executes.

---

## F2 — Re-run the V3 battery at the selected depth

Same tests, same pre-registrations as `validation_v1_plan.md` §V3, on the enriched partition.

### F2.1 — Reproducibility (V3.1 repeat)
Two processes, bit-identical CE / EI / K / M / transition counts. Non-negotiable; the
canonicalisation fix from V3.1 must hold at the new depth.

### F2.2 — Boundary reality (V3.2 repeat, now with real support)
Label-permutation null preserving group sizes, ≥200 replicates, on the 6b vacuum-augmented
model (the only model whose `EI_macro` is not structurally pinned — 6a reduces exactly to
`CE = 1/EI_micro` and cannot falsify anything, so it is reported but not tested).
Report: real CE, null mean / p95 / max, count null ≥ real, p, and — new and important —
**the size of the null's support**, i.e. how many distinct partitions the permutation can
actually produce. The V1 result's weakness was support of ~10; state the new figure
explicitly so the test's resolution is legible.
*Pre-committed interpretation:* p < 0.05 with support ≫ 10 ⟹ the boundary carries
partition-specific causal signal and the V1 negative was a resolution artifact. p ≥ 0.05 with
adequate support ⟹ the negative is now **properly powered** and is a much stronger result
than V1's.

### F2.3 — Coarse-graining sensitivity (V3.3 repeat, corrected)
Repeat the perturbation sweep, but **separate degenerate from non-degenerate
reassignments.** V1's headline range `[0.0000, 0.3668]` includes the reassignment that moves
the sole carrier class into the substrate group, yielding an empty carrier group, `K = 1`,
and `CE = 0` by construction. That endpoint is a degenerate partition, not evidence of
sensitivity, and quoting the full range against V3.4's ±0.07 registry effect compares two
different kinds of perturbation. Report non-degenerate range separately, and keep the
temporal-shift robustness check (V1: CE moved 0.249748 → 0.250273 across a +10 window shift —
strikingly stable, and the dissociation between temporal robustness and partition fragility
is a real result worth preserving).

### F2.4 — F1-bias bound (V3.4 repeat)
Repeat the registry-closure sweep. V1 measured ~+0.07, non-monotonic. State the result as a
**bound**: the maximum plausible F1 correction versus the distance from measured CE to 1.0.
In V1 that gap was 0.25 → 1.0 against a +0.07 lever, i.e. an order of magnitude short, which
closes F1 as an available rescue for this experiment. Restate the bound at the new depth.

---

## F3 — Make the canonicalisation invariant structural

The V3.1 methodological finding is the most transferable output of the V-track and must not
be able to regress.

- **F3.1 — Monotonicity assertion.** Assert that the class-index → WL-hash map is sorted by
  WL hash at construction. A debug assertion inside `TpmAccumulator` is preferable to a test,
  because it fires for every future caller rather than only the ones someone remembered to
  test.
- **F3.2 — Two-process statistic test.** A CI test that computes an index-dependent
  statistic (the permutation null count is the natural choice) in two processes and asserts
  identity. CE alone is invariant to index permutation and therefore cannot catch this — the
  test must use a statistic that ranks or permutes.
- **F3.3 — Audit existing call sites.** Grep for every consumer of dense class indices and
  confirm each is downstream of canonicalisation. Report the list.

*Gate:* all three. *Failure:* bug, blocking for any future CE or partition analysis.

---

## F4 — Criterion 5: is `satiation` underpowered or null?

V2.1 concluded criterion 5 is met only via the degenerate route. For
`replication_threshold` that is certain — node-funded replication means nothing opposes it,
it moved 100% down, and its effect size sits *inside* the drift band (1.22× against a 4.47
envelope). For `satiation` the conclusion is less safe: mean Δ +4.28, |Δ| 7.56, **1.69×** the
drift envelope (larger than `replication_threshold`'s), and **83% directional**. That is the
one trait carrying a genuine trade-off (more buffer means longer survival but costs more
energy to fill), it moved in the pre-registered direction, and the statistics rest on a
regime that establishes in only 6/10 seeds.

**F4.1** — Replicate at higher seed count, targeting **≥30 established-turnover runs**
(expect ~45–55 attempts at 57–71% establishment). Report per-seed satiation Δ against the
matched inert-trait drift null.

**F4.2** — Pre-committed interpretation, written before the runs:
- Sign test on Δ direction significant **and** |Δ|/drift-envelope > 1.5 ⟹ criterion 5 has a
  **non-degenerate component**; the write-up's criterion-5 verdict changes.
- Either fails ⟹ the V1 verdict stands, now properly powered.

**F4.3 — Record the structural reason criterion 5 was near-unsatisfiable.** Deviation #4
(fixed harvest/maintenance quanta rather than genome-scaled) makes `harvest_rate` and
`maintenance_cost` inert — no rule reads them. Of four genome traits, two can only drift, and
one of the remaining two (`replication_threshold`) is unopposed under node-funded
replication. Only `satiation` can express a trade-off. **The deviation that made the economy
tractable is what made selection undetectable.** This belongs beside the criterion-5 result
as a pre-committed-deviation consequence, not discovered later.

---

## F5 — Corrections to `validation_v1_findings.md`

Textual, no re-running required:

1. **Re-label the fork branch.** "Boundary spurious" contradicts V2.2 in the same document.
   Correct framing: *the boundary is real; the macro is underspecified.* The remedies differ —
   spurious boundary means abandon, underspecified macro means enrich the macro — and the
   whole of F0 follows from the second reading.
2. **Qualify the V3.3 range.** Note the `0.0000` endpoint is the `K = 1` degenerate partition
   and report the non-degenerate range separately (see F2.3).
3. **State the F1 bound quantitatively.** Convert "the caveat should stand as a caveat" into
   the stronger and more useful claim: registry closure moves CE by ~+0.07 against a required
   0.25 → 1.0, so the downward-bias argument **cannot** be invoked as a reason the real effect
   is masked here.
4. **Flag the experiment substitution.** Experiment 6 was specified as CE *across the
   replication-onset boundary* — a temporal comparison. What was computed is CE of a
   carrier/substrate **spatial partition during sustained turnover**. V3.2 correctly reports
   the temporal control cannot be constructed (longest birth-free stretch 58 steps against 800
   required), but the deeper point is that on a **static** substrate the temporal experiment is
   *degenerate*, not merely unconstructible: V4.2 confirms zero back-reaction, (36, 85)
   unchanged at every window, so pre-boundary micro dynamics are empty and a before/after
   comparison is trivially "nothing versus something." Both framings are blocked by the static
   substrate, and the substitution of one for the other should be explicit.
5. **Own the superseded reasoning.** The 1.47× heterogeneity requirement and the route-A/route-B
   framing built on it were conditioned on the sweep regime that had no band. Turnover was
   unlocked by **bootstrap threshold 15 → 3**, at a measured satiation spread of only 1.161×.
   The heterogeneity-rescue hypothesis was never the operative mechanism; record this as a
   prediction failure rather than quietly dropping it.

---

## F6 — Pre-registrations for the record

- **`harvest_rate` out-of-range drift.** V1.3 measured `harvest_rate` at −13.948 from a seed
  of 1.00 — confirming that the S2.1 unclamped-trait choice does drive traits far outside
  nominal range. Harmless while the trait is inert. **If deviation #4 is ever reversed and
  `harvest_rate` becomes active, the population begins from wildly out-of-range values**, and
  any resulting dynamics are an artifact of accumulated drift on a previously-inert trait, not
  a fresh economy. Register this now so it cannot be discovered as a surprise.
- **Turnover fragility.** The regime establishes in 57–71% of seeds. Any future claim resting
  on it must report establishment rate alongside, and single-seed results are not
  generalisable.

---

## Forbidden responses (explicit)

- Selecting WL depth by its effect on CE, p-value, or null margin.
- Pursuing CE > 1 via registry closure (V1: ~+0.07 available) or partition reassignment
  (V1: ~±0.15 available non-degenerately). A CE > 1 obtained that way is worth less than the
  reproducible CE ≈ 0.25 already in hand.
- Reversing deviation #4, adding a trait, or coupling an inert trait in order to make
  criterion 5 pass non-degenerately.
- Hand-reshaping the carrier/substrate partition to produce more carrier classes if F0 shows
  they do not arise naturally.
- Re-running Experiment 6 proper, the tri-ecology gate, or the discrete-substrate novelty
  question before F2 resolves. These remain held.

---

## Report back

- **F0:** configured WL depth; the depth→class-count table; carrier site-occupancy
  distribution; which branch of the decision tree the data selects.
- **F1:** the chosen depth and the rule that chose it, recorded before any CE was computed.
- **F2:** CE, two-process identity, permutation p **with null support size**, non-degenerate
  sensitivity range, and the restated F1 bound.
- **F3:** assertion in place, two-process statistic test green, call-site audit list.
- **F4:** establishment rate, N established seeds, satiation sign test and effect size against
  drift, and whether the criterion-5 verdict changes.

The single sentence to state plainly at the end: **whether the carrier group resolves into
more than one class, and therefore whether the V1 negative was a measurement or an
artifact of the partition's resolution.**
