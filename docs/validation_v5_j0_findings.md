# V5 / J-track — J0 findings: the carrier resolution was substrate-borrowed → J2

Reported against `~/Downloads/validation_v5_nonpartition_plan.md` and the committed ablation
definition `docs/validation_v5_j0_ablation_definition.md` (J0.1, commit `165c925`, before J0.2 ran).
Harness: `crates/hg-sim-core/tests/j0_site_ablation.rs`. Run on the retained F8 static run, seed
12345, WL depth 2. Per the forbidden-response list, neither J1 nor J2 was scoped until this gate
selected the branch.

---

## The single sentence (J0's mandated answer)

**Carriers have no structure of their own once the substrate's geometry is removed — the 4-class
resolution the detectability result rests on was entirely substrate-geometric — so the
sustained-turnover boundary cannot be instrumented by a churn-stable partition, and the fallback
must be a measure over causal dependencies directly (J2), not a hybrid partition (J1).**

## Headline

| carrier WL classes (seed 12345, depth 2) | count |
|---|---|
| **with** site geometry (production WL — fidelity anchor) | **4** (= F8) |
| with site geometry (local WL replication — must match anchor) | 4 |
| **without** site geometry (site held constant — the gate) | **1** |

Corroborating intrinsic attributes observed across all carriers over the whole run:
- distinct **energy-field** values: **1** (all `0.0`) — the field is never written in production; ALife
  energy is property key 1, which WL excludes (Q2).
- **max carrier degree: 1** — every carrier has exactly one occupancy edge.

## Verdict: **BORROWED structure → J2** (pre-committed reading, `ablated == 1`)

The carrier's entire resolvable WL structure was the corner-vs-interior neighbourhood of its
occupancy **site** — a property of the frozen crystal's geometry. Factor the site out and every
carrier is identical: same label (100), same energy field (0.0), same degree (1), no
carrier–carrier adjacency. There is no carrier-intrinsic, churn-survivable WL structure. This is
the code-level prediction from J0.1, now measured, not assumed.

Consequences the plan pre-committed to:
- A **hybrid partition (J1) is not viable**: its whole premise was a carrier-intrinsic attribute to
  resolve on, and there is none. Not scoped.
- A **label anchor would reinstate F0's fatal (9,1) under-resolution** — carriers as one class — which
  V1 showed cannot return a positive. So a partition-based fallback is closed.
- The forward path is **J2**, a non-partition measure over causal dependencies directly.

---

## J3 — the report-level consequence (write into §9/§10), independent of any fallback

**The detectability result's dependence on the frozen substrate's geometry is now measured, not
assumed.** The sustained-turnover boundary is **geometry-dependent**: its very *detectability*
lived in the static crystal's corner-vs-interior site structure, which the carriers borrowed
through depth-2 WL refinement. This sharpens the central result in a specific way:

- The F8 claim stands exactly as stated — on the **static** substrate the carrier/substrate boundary
  is robustly detectable (30/30 seeds). J0 does not weaken that.
- But the boundary's detectable structure is **not substrate-independent**. Remove the geometry and
  the carrier partition collapses to a single class. So the boundary is a property of *biology on
  this particular frozen geometry*, not of the biology alone. Combined with the G-track
  (sustained-turnover, replication load-bearing but duration-confounded) and the H-track (CE breaks
  under motion), the picture is: **the detectable object is a replicating population resolved
  through a static substrate's geometry, and every attempt to detach it from that static geometry —
  churn (H), intrinsic carrier structure (J0) — finds the detectability does not survive the
  detachment.** That is the honest scope of the result.

This J3 statement holds regardless of whether J2 ever yields a working instrument.

---

## What this sets up for J2 (not yet scoped beyond its gate)

J0 does more than select J2 — it constrains it. The only churn-*invariant* carrier descriptor that
survives is the **label** (an aggregate: carrier vs substrate), because every finer WL distinction
was site-geometric and dies with the geometry. A non-partition measure therefore has only
label-anchored **aggregates** (carrier count, occupancy/energy fields) as churn-stable variables —
which is exactly the aggregate/(9,1) resolution F0 diagnosed as fatal for *detection power*. So J2
enters with a well-definedness question (can a causal-dependency measure be posed over churn-
invariant variables at all — J2.1) already shadowed by a power question (even if posable, can an
aggregate-only measure recover F8 — rule 4). Both are addressed cheaply, and before any build, in
the J2.1 criterion + J2.2 probe that follow.
