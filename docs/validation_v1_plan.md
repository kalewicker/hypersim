# hypersim — Pre-Experiment Validation Plan (V-track)

The v4 plan and the hg-alife stages S0–S7 are code-complete. S7 emits the
headline the whole program was built to produce: a causal-emergence coefficient
across the replication-onset boundary. **This document is the gate between "the
code runs and the criteria pass" and "a result I can report."** Nothing here
adds a mechanic or tunes a parameter. It validates that the instrument is sound,
that the seven-criteria verdict means what it says, and that the CE number is a
measurement rather than an artifact — then states, per outcome, whether the next
move is an experiment or a write-up of a negative result.

Ordering is by blast radius, not by the stage each test was first raised in.
Duplicates raised at multiple stages (the NonTrivial negative control, the
reproducibility spine, the no-`alife` bit-identity check) are deduplicated into
the phase where they are load-bearing. Each phase gates the next: a corrupted
instrument (V0) makes every V2/V3 number meaningless, so V0 runs first.

---

## Rules for the implementer (binding — read before writing any test)

This is a diagnostic track. Several tests are *expected* to fail in ways that
are **findings, not bugs**, and the whole value of the track evaporates if a
failing assertion gets "fixed" to go green. Concretely:

1. **Distinguish a bug in the instrument from a finding about the system.** A
   read-only observer that perturbs the run is a bug — fix it. A trait
   distribution that collapses degenerately is a finding — report it, do not
   suppress it. Each test below states which of the two a failure is. If a test
   fails in a way not covered by its pre-committed interpretation, **stop and
   surface it**; do not improvise a fix.
2. **Never tune to pass.** Do not lower a threshold, add a cap, adjust influx,
   or reshape a partition to make a criterion pass or to push CE > 1. This is
   the scope §8.1 forbidden-response column, and it applies to validation as
   hard as it applied to S4. A CE > 1 obtained by tuning is worth less than a
   reproducible CE < 1.
3. **Report the realized quantity, not the configured one.** Realized influx is
   `rate × count(sites below cap)`, not `rate`. Realized band boundaries, realized
   entropy loss, realized per-trait variance — measure what happened, not what
   was set.
4. **A test that cannot be run is itself a result.** If a fixture or control
   cannot be constructed (as happened with the route-A churn substrate), that
   absence is a finding to report, not a blocker to engineer around silently.
5. **No new run directories reuse an old one.** The contamination firewall
   between substrate and cognition/ALife run dirs is a standing invariant; every
   run below goes to a fresh, labelled directory, one CSV per run.

---

## V0 — Instrument soundness (gates everything downstream)

S5's observer is the measuring device for every criterion and every CE input. A
silently wrong observer does not fail loudly; it emits plausible numbers. These
five prove the device before any number it produces is trusted. Cheapest and
most fundamental first.

### V0.1 — Read-only proof
Run the ecology with `AlifeObserver` attached and detached, same seed, same
steps. Assert **bit-identical `canonical_hash` and bit-identical total-energy
trace**. Not the fixture — a live multi-carrier run.
*Gate:* identical. *Failure:* **bug.** Observation is mutating state; every S5–S7
number is measuring a different system than the one reported. Blocking.

### V0.2 — Population conservation
Every step, assert `carriers(t) == carriers(t-1) + births(t) - deaths(t)` against
the graph's actual carrier-labelled node count. Run across a full
saturation run, not a fixture.
*Gate:* holds every step. *Failure:* **bug.** An event is missed or
double-counted; lineage and clade counts are corrupt.

### V0.3 — NodeId reuse under SlotMap (the one I'd bet on)
`SlotMap` recycles keys. If `LineageTree` maps a bare `NodeId` and a dead
carrier's slot is reused by a later birth, the new carrier silently inherits the
dead one's ancestry — and this only manifests *once turnover exists*, i.e. exactly
when the numbers start mattering. Test: force a death, then force a birth that
reuses the slot; assert the new carrier is either a fresh key or that
`LineageTree` keys on the **version-tagged** SlotMap key, not the raw index.
Confirm the `AHashMap` key type carries the version.
*Gate:* no ancestry bleed across a reused slot. *Failure:* **bug**, and a
subtle one — high-turnover clade counts are garbage while low-turnover fixtures
pass. Blocking for any lineage claim.

### V0.4 — Events-alone reconstruction vs ground truth
The S5 gate ("reconstructable from events") passes trivially on a hand-traced
3-generation fixture. The real test: on a live run, at step N compare the
observer's reconstructed live-carrier set to the actual carrier-labelled nodes in
the graph. Assert **set equality**, including birth-child-id and death-deleted-id
attribution.
*Gate:* set equality at several N across a run. *Failure:* **bug** in event
attribution (child id misread from created-nodes, death not attributed).

### V0.5 — NonTrivial negative control (has this *ever* run?)
The criterion-6 headline depends on the classifier being able to **withhold**
`NonTrivial`. Hand-build two carriers: one a trivial copier (no structural
elaboration), one genuinely non-trivial. Assert the classifier assigns
`NonTrivial` to the second and **not** the first.
*Gate:* separation. *Failure:* **bug** if it labels everything `NonTrivial` —
then the 0→>0 crossing is a label artifact and the entire CE boundary is
spurious. This is the single most important instrument test for the S7 claim;
if it has never run, run it before anything else in this track.

---

## V1 — What the frozen run already says (cheap; may fork the plan)

These are the diagnostic reads S5 made measurable but that were never taken.
They run on the **existing frozen saturation run** — no new runs, no mechanics —
and one of them can invalidate several turns of prior reasoning. Run V1.1 first.

### V1.1 — `replication_threshold` vs `satiation` (run first)
With fixed quanta and a satiation gate, a carrier's steady-state energy is
**pinned at `satiation`** (it stops harvesting there). If
`replication_threshold > satiation`, reproduction is **structurally impossible at
any influx rate** — the carrier can never climb to the birth floor. Compare the
two values in the live population (S5 gives per-trait mean directly).
*Gate:* threshold ≤ satiation across the population. *Failure interpretation:* if
threshold > satiation, **this is the barrier** — a misconfigured genome, not an
economy failure, and the influx sweep, the `child_node_cost` refutation, and the
synchronized-death argument must all be re-read as measuring an artifact. Highest
expected value in the track; two numbers you already have.

### V1.2 — Satiation distribution (the heterogeneity budget)
Report mean, variance, and max/min of `satiation` across the live population.
Carriers pinned at different energies hit zero at different times, so the
satiation spread is an **endogenous death-desynchronization budget** independent
of the substrate.
*Gate:* report the ratio. *Interpretation:* max/min ≳ **1.47×** (= 0.49/0.333,
the squeezed turnover window) ⟹ the desynchronization budget is already present
in the genome and route A's motivation is undercut. Spread ≈ 0 ⟹ ask why (jitter
too weak, or selection already collapsed it — the latter is itself a criterion-5
signal).

### V1.3 — Trait shift vs drift vs degenerate collapse (criterion 5 substance)
Compare per-trait distributions at freeze against seed. The frozen run had
births, so selection has had generations. Criterion 5 ("distributions shift") is
satisfied three ways and only one is the finding.
*Interpretation, pre-committed:* a `replication_threshold` collapse toward its
floor is **free and degenerate** (node-funded replication → parent pays nothing →
nothing opposes it). A directional `satiation` move is **selective** (more buffer
= longer survival but costs more energy to fill — a real trade-off). If the only
trait that moved is the one under no trade-off, criterion 5 passed on the
artifact and must be reported as such, not as selection.

### V1.4 — Clade structure on real data
`clade_count`, `clade_survival_curve`, and `export_newick` on the frozen
population. Zero mortality ⟹ no clade has died, so the survival curve is
degenerate-but-informative: it says whether the population descends from one
lineage or the seed's early radiation left structure. Also the first exercise of
`export_newick` on real data rather than a fixture.
*Gate:* Newick parses in a standard tool; clade counts consistent with V0.2/V0.3.

---

## V2 — Verdict integrity (is the seven-criteria pass trustworthy)

### V2.1 — Criterion 5: drift-null distinguishability
Beyond V1.3's attribution: run a pure-drift null (jitter present, differential
survival removed) and confirm the observed trait trajectory is **statistically
distinguishable** from it. If it is not, the result is drift, not selection,
regardless of whether a mean moved.
*Failure:* **finding** — selection is not demonstrated; report criterion 5 as
unmet on its own terms.

### V2.2 — Criterion 6: the crossing is a real event
Confirm the `NonTrivial` 0→>0 crossing corresponds to an actual structural
elaboration in the run, not the first carrier acquiring a distinct WL class for
an uninteresting reason. (Bridges to V3.2's spurious-boundary control.)
*Failure:* **bug/finding** — a crossing without a structural event means the CE
boundary is spurious.

### V2.3 — Band reality (only if the sweep found a band)
Low late-window variance is necessary, not sufficient. Three checks:
- **Trend, not just variance:** a slowly climbing ratio has low windowed variance
  and is not stable. Test for trend.
- **Band vs single value:** is the stable region an interval or one influx value?
  A razor-thin band is the synchronized-death discontinuity barely avoided —
  report it as *fragile*, not as a robust carrying capacity.
- **Seed robustness:** 3–5 seeds at the band edges. Each run is deterministic;
  the question is whether the *boundary* moves across seeds. If it swings wildly,
  the band claim is soft.

### V2.4 — Criterion 7: the reproducibility spine
The terminal cash-out of S0.1's WL-hasher determinism. The sweep is
cross-run comparison by construction, so this is where soft determinism corrupts
the *claim*, not one datum.
- Re-run the **whole sweep in a second process**; assert identical **band
  boundaries** (not merely identical single runs).
- **Energy books balance to declared entropy loss at every sweep point**,
  including the **death-residual term** (deleting a carrier at negative energy is
  an energy source scaling with overshoot depth). Balancing at only one influx
  rate makes the band uninterpretable.
- Paired bare-substrate control present for every sweep point; substrate-only
  metrics match their control.
*Gate:* all hold. *Failure:* **bug** (determinism/accounting) — blocking for the
band claim and for S7.

---

## V3 — CE validation (is the coefficient real)

The CE number is the most falsifiable object in the project. It is partition-
relative and cross-scale, so it fails quietly: a spurious boundary or a soft hash
yields a *well-defined* number that is not measuring what it claims.

### V3.1 — Reproducibility (two processes, identical CE)
Compute CE in two separate processes; assert identical to float epsilon. Micro
and macro state distributions must be sampled identically across runs, which
requires the coarse-graining hashes to be fixed-seed — everything upstream can be
deterministic and CE can still be non-reproducible if the partition hashing is
not. *Failure:* **bug**, blocking.

### V3.2 — Boundary reality (spurious-boundary control)
Compute CE across a deliberately **arbitrary** boundary (partition the run at a
step with no replication event). Assert the real replication-onset boundary
scores **higher** CE than the arbitrary one.
*Gate:* real > spurious, with margin. *Failure:* **finding/bug** — if an
arbitrary cut scores as high as the onset, CE is not detecting the emergence
being attributed to it, and the S7 headline is void.

### V3.3 — Coarse-graining sensitivity
Perturb the partition: reassign a handful of boundary nodes; shift the onset step
by ±a few. Assert CE is stable under small perturbations.
*Failure:* **finding** — a CE that swings with the exact partition definition is
measuring the partition choice, not the system. Report the sensitivity
envelope alongside the coefficient regardless.

### V3.4 — Bias direction (empirical F1 confirmation)
Finding F1 ("CE biased downward on open registries") has been carried as a
caveat since the characterization report; S7 is the first time it gates a
headline, so confirm it rather than cite it. Run CE on a **closed-registry
control** (fixed, small WL class set) and confirm the coefficient moves in the
predicted direction relative to the open-registry run.
*Gate:* moves as F1 predicts. *Failure:* **finding** — the caveat is wrong and
every CE interpretation inherits the error. This is the cash-out F1 was written
for.

---

## V4 — Full-stack integrity (the pipeline as one system)

Every gate tested its own stage; nothing has tested S0→S7 as one pipeline, and
the contamination firewall deserves one whole-system check now that all seven
stages have landed.

### V4.1 — No-`alife` build bit-identical to pristine substrate
Build without the `alife` feature and diff a full run against a pre-alife-crate
commit. Feature-gate leakage accumulates across stages; this proves the ALife
layer is still genuinely optional and the substrate track is uncontaminated.
Pair with the compile-time half: `cargo tree` shows **no `hg-engine` edge from
`hg-alife`** (a compile failure if anyone added it is the desired behavior).
*Gate:* bit-identical + no forbidden dependency edge.

### V4.2 — Paired control for the experiment run
The experiment run that produces the reported CE gets a same-seed bare-substrate
control; substrate-only `d_eff`, node/edge counts, and the substrate-subgraph
regime classifier match the control (a carrier bloom must not read as substrate
`Exploding`). This is what licenses attributing anything to biology.

### V4.3 — Clean-room reproduction
Fresh checkout, fresh run directory, the full sequence to the CE number from
nothing. The difference between "reproducible on my machine" and reproducible.
*Gate:* the CE number reproduces from a clean checkout to float epsilon.

---

## The fork: what the outcomes mean for "experiments next"

The next move is not knowable until the track runs, because it depends on
(a) what S7 returned, (b) what substrate turnover ran on, and (c) whether V0.5 /
V3.2 show the boundary is real. Pre-committed branches:

- **CE > 1, robust (V3.1–V3.4 clean), static substrate, boundary real** ⟹ causal
  emergence across a biological transition on a hypergraph substrate: the
  headline the program aimed at. Next experiment is Experiment 6 proper toward
  the tri-ecology gate, and the discrete-substrate risk becomes testable — does
  *this* substrate show accelerating rather than Hash-Chemistry-linear novelty?
- **CE < 1, reproducible** ⟹ not a failure. Macro is not informationally closed
  over micro at this boundary — a real claim about where emergence *doesn't*
  happen, publishable as a conservative negative result (F1 makes it
  conservative). Next work is *why*: which coarse-graining, which boundary.
- **Boundary spurious (V0.5 or V3.2 fails)** ⟹ S7 computed a clean number across
  nothing. Report the stack up to the blocker; do not run experiments on a
  boundary that isn't real.
- **Turnover required route A (churn substrate) or route B (aging)** ⟹ the CE
  partition is over a *moving* substrate, or death is *decoupled from the energy
  economy* so "replication onset" is no longer the natural boundary. Either
  changes what S7 measured; re-examine V3.2/V3.3 under the actual substrate
  before treating the coefficient as the intended result.
- **V1.1 shows `replication_threshold > satiation`** ⟹ the last several turns
  analyzed a system that structurally cannot reproduce at equilibrium; fix the
  genome config, re-run the influx sweep, and re-enter this track from V1.

Only along the branch V3 actually selects do the standalone
substrate-characterization write-ups (the CE result, the sweep band or its proven
absence, the "no bounded-churn regime in the builtin rule zoo" finding, the
synchronized-death discontinuity) become the first publishable output — and their
interpretations were pre-committed across the prior turns, which is what keeps
them clean.

---

## Report back at each phase

- **V0:** the read-only diff result; the SlotMap-version key confirmation; whether
  the NonTrivial negative control had ever run and its result.
- **V1:** the `replication_threshold`-vs-`satiation` comparison (the one that can
  fork the plan); the satiation max/min ratio against 1.47×; which trait moved at
  freeze and whether its motion carries a trade-off.
- **V2:** drift-null distinguishability; band trend/fragility/seed-robustness;
  the two-process band-boundary identity and the per-sweep-point energy balance
  including the death-residual term.
- **V3:** the CE coefficient, its two-process reproducibility, real-vs-spurious
  boundary margin, partition-sensitivity envelope, and the closed-registry
  bias-direction confirmation.
- **V4:** no-`alife` bit-identity + `cargo tree` edge absence; experiment-run
  control match; clean-room CE reproduction.

The headline to state plainly, once and unhedged: the CE coefficient, the
substrate it ran on, whether the boundary survived V3.2, and which fork branch
the project is therefore on.
