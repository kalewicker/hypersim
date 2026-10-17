# G0.1 — The guard-shadow audit (v10 §2.1, F-LK33)

*LK track, v10 / Phase G0. Executed 2026-08-05.*
*Driver: `crates/hg-sim-core/tests/g0_guard_shadow.rs`.*
*Artifact: `runs/v10_g0/guard_shadow_audit.log` (both tests, exit 0).*

---

## 0. What was asked, and the answer in one line

v10 §1.4 makes guard **pairs** auditable. `lk_fuse` carries one — a
generation guard (`w.generation ≥ 1`) and a degree guard (`degree(w) == 2`)
— and it had never been audited as a pair. F-LK33 predicted the generation
guard is *shadowed* by the degree guard, so the v4 §1.4 non-removal
guarantee rests on degree, not generation.

**F-LK33's conclusion is confirmed. Its stated mechanism is exactly
backwards, and the inversion is itself the finding.**

---

## 1. The order claim: REFUTED

`bind_free_vars` chains the guard families with `&&`
(`crates/hg-engine/src/matcher.rs:288-291`):

```
property_guards → property_compares → degree_compares → age_compares
```

The generation guard is a `node_property_guard`, so it runs **first**. The
degree guard only ever sees bindings that generation has already admitted.
F-LK33's premise — "the degree guard rejects first, so generation cannot be
reached" — is the wrong way round, and the archived counters say so at
every window of every run.

Across **182 runs** carrying `lk_fuse` guard rows:

| quantity | value |
|---|---:|
| generation-guard rejects | **179,783,270,400** |
| degree-guard evaluations | 825,405,448 |
| degree-guard **rejects** | **0** |
| runs where the degree guard is never evaluated at all | **65** |
| runs where generation rejects/window varies | **0** |

The 65 unreachable runs are the `f_cap` arms: the throttle is injected as
`prop1`, a second *property* guard, so it short-circuits ahead of degree in
the same family loop. In those runs the degree guard is not merely
non-rejecting — it is never asked.

**So the inert instrument of the pair is the DEGREE guard**, which is the
precise inverse of the prediction. By the track's own engagement rule
(v6 §1.1), that has to be recorded as such.

This is the same failure family the catalogue already holds — eight
concurrency failures, three instrument-inertness cases, one back-action
case, all of them *assuming when things happen without checking*. F-LK33
was written from the rule file, where the degree guard is declared above
the generation guard. Declaration order is not evaluation order. The
catalogue entry generalises: **the guard grammar's family loop, not the
TOML, fixes precedence.**

## 2. The reachability claim: CONFIRMED

Whether the generation guard is *load-bearing* is a different question, and
the short-circuit counters cannot answer it — they only ever report the
first rejector. It is a counterfactual: of the bindings generation rejects,
would degree have rejected them anyway? It reduces to one predicate:

> **Does any gen-0 node ever reach degree 2?**

Measured on 19,323 persisted snapshots, with degree computed by the
guard's own predicate (distinct incident hyperedges, v4 §6.2):

| population | run dirs | snapshots | gen-0 degree spectrum | **gen-0 at degree 2** |
|---|---:|---:|---|---:|
| clean-lattice | 209 | 18,891 | `{6: 18,862,080}` | **0** |
| planted-handle | 12 | 432 | `{6: 295,464, 7: 8,328, 8: 336}` | **0** |

Never. Not once, in 19.2M stamped gen-0 node-observations. Deleting the
generation guard would admit **nothing new**: it is semantically redundant,
and the non-removal guarantee is the degree guard's.

The mechanism is that gen-0 degree is a **topological invariant** of the
rule family, and the counters witness it independently: generation's
rejects-per-window are *exactly* constant in all 182 runs, to the unit.
That constancy is only possible if the gen-0 skeleton's local edge
structure never changes — which it cannot, because no rule can change it:

- `lk_metered_subdivide` replaces edge (a,b) with (a,m),(m,b): both
  endpoints keep their edge count.
- `lk_fuse` removes midpoint w and its two edges, then adds (x,y): x and y
  each lose one and gain one.
- `lk_transport` is topology-inert.

Nothing creates gen-0 structure and nothing lowers a gen-0 degree, so every
gen-0 node sits at its birth degree (6 on the torus; 7–8 at planted handle
endpoints) forever.

## 3. The third result, unasked for

The emergent population's degree spectrum is `{2: 14,252,898}` — **every
gen>0 node, in every snapshot, is at exactly degree 2**. No branch point
has ever formed. This is why the degree guard's reject count is zero rather
than merely small, and it is a sharper statement of §2's "1-D filaments"
than the paper currently makes: filamentary morphology is not a description
of the observed runs, it is enforced by the fact that subdivision only ever
inserts degree-2 midpoints and fusion only ever removes them.

So **both guards are individually redundant** — but not symmetrically, and
the asymmetry is what matters downstream:

| guard | redundant because | strength |
|---|---|---|
| generation | gen-0 can never reach degree 2 | **necessary** — degree is a topological invariant of the family |
| degree | no gen>0 node has ever left degree 2 | **contingent** — no process has yet created a branch point |

The generation guard's redundancy is provable. The degree guard's is
merely measured, and a rule family that could create a branch point would
break it.

## 4. Scope, and the false positive that produced it

The first version of this audit read every snapshot in `runs/` and fired
§2.1's alternative on 3,501,576 nodes. The attribution was exactly three
families, and the cause was an instrument limit, not erosion:

| family | nodes carrying the generation property |
|---|---|
| `expL_v2_archive` | **0 of 57,600** |
| `exp_l10` | **0 of 3,986,529** |
| `exp_l12` | 4,170,411 of 4,325,458 (partially stamped) |

These are v3-era production-only runs that predate the generation stamp
(neither has an `lk_fuse` rule at all). A missing property *fails* the
guard `gte 1.0`, so for the fuse rule "absent" and "gen-0" are the same
thing — but for this audit they are not: an unstamped snapshot reports its
entire emergent filament as skeleton, and a filament midpoint is degree 2
by construction. All 3,387,489 offenders in `exp_l10` were unstamped
filament. In the stamped fraction the count was zero everywhere.

The predicate is therefore evaluated only where the property exists; the
unstamped population is **reported as the audit's blind spot** rather than
dropped, its degree spectrum is printed, and the family list is asserted so
the scope statement cannot silently go stale.

Recorded because the near-miss is the point: the first read of this audit
said *the lattice is eroding*, and the difference between that and the
truth was one un-checked instrument precondition.

## 5. Consequences

1. **Paper §1 corrected.** The scaffolding bullet named
   "generation-guarded fusion" as what makes the skeleton non-removable.
   It is the degree guard. Reworded, with the generation guard's actual
   (redundant) role stated.
2. **`rules/lk_fuse.toml` corrected.** Its v8 comment read "The generation
   guard below is what keeps the lattice non-removable; do not weaken it."
   That is wrong in both halves — it is not what keeps the lattice
   non-removable, and weakening it alone would change nothing.
3. **G3 (skeleton erosion) is confirmed as a bigger change than it looks,
   now with proof rather than inference.** v10 §5 reasoned that relaxing
   generation does nothing while degree stands. Correct — and §2 above
   upgrades it from an argument about a torus to a topological invariant of
   the family. Eroding the skeleton requires relaxing **degree**, i.e. a
   general vertex contraction, which is a different operation from a
   chain-link unzip. **Default disposition stands: defer.**
4. **New standing rule (proposed for v11 §1):** *declaration order is not
   evaluation order.* Any guard-pair claim must cite the family loop in
   `bind_free_vars`, not the rule file.
5. `guards.csv` needs no `shadowed_by` column (v10 App. B). Precedence is
   already recoverable from the counters — each family's `evals` equals the
   previous family's `evals − rejects` — and the audit above was carried
   out with no schema change. Recording the *cascade identity* in the
   schema doc is the cheaper fix.

## 6. What this does not show

- It does not show the generation guard is useless **prospectively**. Under
  G1 influx or a G2 scheduler change the invariant argument in §2 still
  holds (neither axis adds a rule that alters degree), but a future family
  that contracts vertices or plants structure mid-run would need re-auditing.
- The degree guard's redundancy is contingent (§3) and should not be
  relied on at all.
- The blind spot in §4 is real: for `expL_v2_archive` and `exp_l10` this
  audit has **no** measurement, only the observation that neither run
  carries a removal rule, so neither can erode anything.
