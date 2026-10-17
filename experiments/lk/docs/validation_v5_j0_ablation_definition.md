# V5 / J-track — J0.1: the site-ablation definition (committed BEFORE running J0.2)

Plan: `~/Downloads/validation_v5_nonpartition_plan.md`. J0 is the gate: one ablation on existing
F8 data decides whether the sustained-turnover boundary can be instrumented by a *partition* at all
(J1 hybrid) or must be pursued by a measure over causal dependencies directly (J2). This document is
the J0.1 deliverable — the precise ablation definition and its pre-committed reading — written
**before** the ablation is run (J0.2). Per the forbidden-response list, neither J1 nor J2 is scoped
until J0.2's result selects the branch.

---

## What the ablation must do

The carrier WL hash at depth 2 mixes two things:
- **carrier-intrinsic content** — the carrier's own base hash and its own connectivity;
- **borrowed site geometry** — via depth-2 refinement, the carrier's hash absorbs its occupancy
  site's neighbourhood (how many lattice neighbours the site has: corner vs edge vs interior). This
  is the signal that split carriers into **4 classes** at depth 2 (F0/F2), and it is a property of
  the **frozen crystal's geometry**, which a churning substrate would dissolve.

The ablation **holds the site contribution constant across all carriers**, so that any residual
carrier class distinction is carrier-*intrinsic* and therefore (unlike site geometry) potentially
churn-stable.

## The precise ablation (how the WL computation is modified)

WL refinement (`hash.rs::compute_wl_class_map`) is: round 0 assigns each node a base hash
`FxHash(WL_SEED, label.0, energy_field.to_bits())`; each of `radius` refinement rounds replaces a
node's hash with `FxHash(WL_SEED, prev_self, sorted multiset of neighbour prev-hashes)`.

**Ablated computation:** run the identical refinement to depth 2, but **force every non-carrier
(substrate, label ≠ 100) node's hash to a single fixed constant `SITE_CONST` at round 0 and after
every refinement round**, before carriers aggregate their neighbours. Carrier nodes keep their real
base hash and their real incident edges throughout.

Because a carrier's neighbours are its substrate site(s), every carrier now aggregates a neighbour
multiset consisting only of `SITE_CONST` values — identical for any two carriers of equal degree,
and carrying **no** corner-vs-interior site-geometry signal. This is exactly "every carrier sees an
identical synthetic site neighbourhood" (J0.1). Any remaining split among carriers is therefore
driven only by carrier-owned attributes.

Equivalently but less transparently, this is the partition induced by each carrier's base hash
propagated through its own degree structure; forcing the substrate constant at *every* round (not
just round 0) is required, else substrate nodes re-diverge and re-inject geometry at depth 2.

## Which carrier attributes remain LIVE under the ablation (so the result is interpretable)

Everything the WL hash can see *except* substrate geometry:

1. **Energy field** (`node_data.energy`, folded into the base hash). **Code fact, verified before
   committing:** `NodeData::new` hard-sets `energy: 0.0` and **no production code anywhere writes
   the energy field** (workspace grep: the only write is a hash.rs *test* fixture). ALife energy
   lives entirely in **property key 1**, which WL excludes (Q2). So the energy field is **uniformly
   0.0** across all carriers — live in principle, dead in practice.
2. **Own degree / incident-edge structure** — the number and arrangement of a carrier's own edges.
   By construction a carrier has exactly **one** occupancy edge (bootstrap/replicate attach one
   `[site, carrier]` / `[target, child]` edge; nothing adds more), so degree is expected uniform =
   1 — but if any carrier ever differs, that is a genuine intrinsic discriminator and the ablation
   will show it.
3. **Carrier–carrier adjacency** — live if two carriers were ever directly edge-linked. They are
   not (occupancy edges connect carrier↔site only), but the ablation would expose it if present.

## Which attributes are EXCLUDED (never in WL, by the Q2 invariant — cannot be anchored on)

The entire **properties map**: property key 1 (ALife energy), genome traits (keys 10 harvest_rate,
11 replication_threshold, 12 maintenance_cost, 13 satiation), carrier_marker (20), occupied (22).
None of these can carry a WL split, so none is available to the ablation — this is why the ablation
result is a clean read on *WL-visible* intrinsic structure, and per rule 2 we do **not** go looking
for a properties-derived or site-derived feature to rescue a split if none survives.

## Accumulation and baseline (how J0.2 will report)

Mirroring F8: run seed 12345 with the F8 config + biology; at each step compute both the **normal**
carrier WL classes (baseline — must reproduce **4**, doubling as a fidelity anchor) and the
**ablated** carrier WL classes; accumulate the set of distinct classes of each kind over the whole
run; report both counts and, for the ablated set, group sizes.

## Pre-committed reading (J0.3, committed here before J0.2)

- **Ablated carrier classes > 1** ⟹ intrinsic, churn-survivable structure exists ⟹ **J1** (hybrid
  partition); record which attribute (energy field / degree / adjacency) carries it.
- **Ablated carrier classes = 1** ⟹ the 4-class resolution was **substrate-borrowed** ⟹ **J2**
  (non-partition measure). This is a substantive finding: the boundary's detectable structure was a
  property of the frozen crystal's geometry.
- **Borderline (2 classes with tiny support)** ⟹ report the support; treat as J2 unless the intrinsic
  split robustly clears the F0 resolution bar (≫ ~10) on its own.

**Committed prediction (from the code facts above, to be measured not assumed):** energy field
uniformly 0.0 and carrier degree uniformly 1 predict **collapse to 1 class ⟹ borrowed ⟹ J2**. If the
measurement contradicts this (a residual split), the driving attribute is the finding and the branch
flips to J1. I am not free to re-read a collapse as "proceed."

## The single sentence J0 must answer

**Whether carriers have any structure of their own once the substrate's geometry is removed — and
therefore whether the sustained-turnover boundary can be instrumented by a partition at all (J1), or
must be pursued, if at all, by a measure over causal dependencies directly (J2).**
