# V5 / J-track — J2.1: the variable-persistence criterion (committed BEFORE J2.2)

Plan: `~/Downloads/validation_v5_nonpartition_plan.md`. J0 selected J2 (the carrier resolution was
substrate-borrowed; no hybrid partition survives churn). J2 is the non-partition fallback: a measure
over the causal dependency structure directly (dynamical independence / ΦID-style causal decoupling,
or a closure-defect estimator). Mirroring H0.1 and J0.1, this document commits the measure's
well-definedness criterion and its STOP condition **before** the cheap probe (J2.2) and before any
implementation (J2.3). Per rule 3 and the forbidden-response list, no measure is built until this
gate and the cheap probe clear.

---

## What a causal-dependency measure requires

A ΦID/dynamical-independence/closure-defect measure decomposes the dependencies among a set of
**variables** observed over a window. For the decomposition to be defined, those variables must:

1. **Persist across the window** — have stable identity/membership, so a joint distribution and a
   transition structure can be estimated over *the same* variables. (This is the one-level-up form
   of H0's partition-stationarity requirement: CE needed a stable class set; a dependency measure
   needs a stable *variable* set.)
2. **Resolve the object above the F0 bar** — the variable set must distinguish the carrier subsystem
   finely enough to carry the boundary. F0/V1 established that a resolution of ~one carrier state
   (the (9,1) partition, null support ~10) **cannot return a positive** (p = 0.495). A measure whose
   churn-invariant variables collapse the carrier to a single aggregate inherits that failure — it is
   well-defined but powerless. Rule 4 (recover F8) is therefore folded into this criterion, not left
   to J2.3.

## The two horns (and why J0 + H0 already frame the answer)

A hypergraph's natural micro variables are its **nodes**. Under substrate churn, node identities
turn over — H0 measured the consequence (WL classes ephemeralise, TWTF 0.5–0.98; and node identities
themselves are minted/retired). So node-level variables fail requirement 1: **no fixed micro-variable
set persists across a churning window.**

The escape (plan J2.1) is a **churn-invariant variable abstraction** — variables defined by *role*
or *aggregate* rather than identity: label-anchored carrier/substrate counts, occupancy/energy
fields, site-role distributions. These persist under churn (you can always count carriers). But J0
showed the carrier has **no churn-survivable internal resolution** — every finer WL distinction was
site-geometric. So the churn-invariant carrier abstraction bottoms out at the **label** (carrier vs
substrate), i.e. the aggregate. That is exactly the (9,1) resolution F0 proved cannot detect
(requirement 2 fails).

So the two horns:
- **Resolved measures** (WL-class-resolved, the resolution F8 needed) — die under churn (H0/J0):
  fail requirement 1.
- **Churn-invariant measures** (label/aggregate-resolved) — survive churn but sit at the F0-fatal
  resolution: fail requirement 2.

## Pre-committed STOP condition (committed here, before J2.2)

**STOP → close the J2 branch (and the track's instrument search) as a significant negative if:**
no variable set exists that is *both* (a) churn-invariant (persists under substrate motion,
requirement 1) *and* (b) resolved above the F0 bar for the carrier subsystem (requirement 2). In
that case the sustained-turnover question is **not instrumentable by any available causal-emergence
measure on this substrate family**, because resolution and churn-invariance are in direct tension —
the same tension H0 found (turnover that sustains a non-replicating population is the turnover that
ephemeralises the partition), now shown to be a property of the *object*, not just the CE measure.

**PROCEED to J2.2/J2.3 only if** a candidate churn-invariant variable set with above-F0 carrier
resolution is identified on paper — then probe it cheaply (J2.2) and, if it survives, build and
F8-check it (J2.3).

## What J2.2 will measure (the cheap horn-1 check)

Requirement 2's failure for aggregates is **already established** by F0/V1 (the (9,1) partition's
p = 0.495 at null support ~10 is the aggregate-resolution result, measured). So J2.2 only needs to
confirm horn 1 empirically: under the existing motion builtins, does any **fixed micro-variable set
persist** across the window, or do node identities churn so that only aggregates survive? Cheapest
decisive check: node-identity survival across the window, mirroring H0.3's use of the same builtins.

## The single sentence J2.1 commits to

**A causal-dependency measure needs variables that both persist under churn and resolve the carrier
above the F0 bar; if no variable set does both — resolved ones dying under churn, churn-invariant
ones sitting at the F0-fatal aggregate resolution — the sustained-turnover question is not
instrumentable by any available causal-emergence measure here, and that negative closes the branch.**
