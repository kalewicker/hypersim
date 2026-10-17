# Q7-2 — Tick-Blocking Audit (directional contention)

**Verdict: BLOCKING CONFIRMED — concurrency failure #7.** And it is
larger in scope than F-LK22 predicted: `lk_tick` blocks not only
*removal* but *production*, so the "production self-throttling" of the
A4 result (Q7-4) has the same single mechanism. The countdown clock,
introduced as a passive measurement device, is the dominant scheduler
actor in the run.

Per v7 §1.2, both directions are audited explicitly; neither is inferred
from the other.

---

## The mechanism

`lk_tick` has a **single-node LHS**, so a tick binding claims exactly the
node it decrements, at **priority 2**. Every other LK rule claims more
nodes at lower priority: `lk_metered_subdivide` {x,y} at 1,
`lk_transport` {x,y} at 0, `lk_fuse` {x,w,y} at −1. Under node-disjoint
MaxParallel resolution, the tick therefore wins every contest it enters.

A0 measured the converse direction — can a tick *lose* a node to an edge
rule? — and found ρ = 1.000, never. **That is evidence for blocking, not
against it:** the tick never yields, which is precisely why everything
else yields to the tick.

## Check 1 — residence exceeds designed lifetime

Little's law: mean residence = L\*/p, which equals k exactly when nothing
delays removal. Measured excess tracks the blocked fraction:

| cell | k | L\* | p | residence L\*/p | residence/k | blocked share |
|---|---:|---:|---:|---:|---:|---:|
| swap transfer k=30 | 30 | 37.7 | 0.956 | 39.4 | **1.31** | 19% |
| swap transfer k=100 | 100 | 127.0 | 0.805 | 157.8 | **1.58** | 84% |
| swap transfer k=300 | 300 | 241.6 | 0.439 | 550.3 | **1.83** | 95% |
| pump vanish k=10 | 10 | 12.4 | 0.541 | 22.9 | **2.29** | 77% |
| pump vanish k=100 | 100 | 12.2 | 0.059 | 206.8 | **2.07** | 93% |

Residence excess rises monotonically with the blocked share across the
swap series. **The paper's L\*/k prefactor is therefore re-attributed
from era-averaging to blocking.**

### Check 1b — the same number, derived independently

`residence = L\*/p` is Little's law applied to quantities that were both
fitted over the era, so on its own it cannot distinguish "blocking" from
"whatever else makes L\* large". An independent estimate comes from the
*expired share* alone: a node spends k steps counting down and then b
steps expired-but-blocked, so the fraction of the standing population
that is expired is f = b/(k+b), giving residence/k = 1/(1−f) with no
reference to L\* or p at all.

| cell | residence/k from L\*/p | residence/k from expired share | agreement |
|---|---:|---:|---:|
| swap transfer k=30 | 1.31 | 1.35 | 2.8% |
| swap transfer k=100 | 1.58 | 1.59 | 0.6% |
| swap transfer k=300 | 1.83 | 1.79 | 2.7% |
| pump vanish k=10 | 2.29 | 2.33 | 1.5% |
| pump vanish k=30 | 2.10 | 2.08 | 0.6% |
| pump vanish k=100 | 2.07 | 2.08 | 0.8% |
| pump vanish k=300 | 2.02 | 2.00 | 0.8% |

The two agree to within 0.6–2.8% across every cell. **Blocking accounts
for the entire residence excess quantitatively**, leaving no residual for
era-averaging to explain — which is what the paper currently credits.

## Check 2 — eligible-but-unfired population

Per era window (`crates/hg-eval/tests/q7_2_blocking.rs`):
*expired* = gen>0 ∧ countdown == 0; *eligible* = expired ∧ degree == 2;
*BLOCKED* = eligible ∧ some neighbour still counting down (so the tick
takes it); *FREE* = eligible with no tick-eligible neighbour.

| cell | filament | expired | eligible | BLOCKED | FREE | free@end | blocked/eligible |
|---|---:|---:|---:|---:|---:|---:|---:|
| pump vanish k=10 | 12.4 | 7.0 | 7.0 | 5.4 | 1.6 | 1.6 | **77%** |
| pump vanish k=30 | 12.2 | 6.4 | 6.4 | 5.8 | 0.5 | 0.5 | **92%** |
| pump vanish k=100 | 12.2 | 6.3 | 6.3 | 5.9 | 0.5 | 0.5 | **93%** |
| pump vanish k=300 | 12.1 | 6.1 | 6.1 | 6.0 | 0.1 | 0.1 | **99%** |
| pump transfer k=100 | 34.6 | 6.5 | 6.5 | 5.9 | 0.5 | 0.5 | 92% |
| swap vanish k=100 | 36.0 | 18.3 | 18.3 | 12.7 | 5.7 | 5.0 | 69% |
| swap transfer k=30 | 37.7 | 9.8 | 9.8 | 1.9 | 7.9 | 7.8 | **19%** |
| swap transfer k=100 | 127.0 | 47.4 | 47.4 | 40.0 | 7.4 | 5.2 | 84% |
| swap transfer k=300 | 241.6 | 107.1 | 107.1 | 101.5 | 5.5 | 3.2 | **95%** |

Three signatures, all as predicted:

1. **Roughly half the standing population is expired but still present**
   (50–57% pump, 26–51% swap). These nodes are removal-ready by age and
   by shape and are simply not being removed.
2. **77–99% of removal-ready nodes are blocked by a ticking neighbour**,
   and the blocked share rises with k — longer lifetimes mean neighbours
   spend longer tick-eligible.
3. **Removal happens at chain ends.** The FREE population is almost
   entirely `free@end` (a gen-0 anchor neighbour, which has no countdown
   key and so is never tick-eligible): 1.6/1.6, 0.5/0.5, 7.8/7.9.

The k = 30 swap cell is the control that proves the mechanism rather than
merely fitting it: there chains are single links, so *both* neighbours
are gen-0 anchors, blocking is only 19%, and residence/k is 1.31 — the
closest to the unblocked prediction anywhere in the sweep.

## Check 3 — direct contention instrument

A directional counter was added to conflict resolution
(`resolve_conflicts_counted`, engine-side, pinned byte-identical to the
legacy resolver by `counted_resolution_matches_legacy`). It records
`(loser rule, winner rule, refusals)` and is emitted into `guards.csv`
as `contention:<winner>` rows. 3000-step cells:

**pump, k = 100:**

| loser | winner | refusals |
|---|---|---:|
| `lk_fuse` | **`lk_tick`** | **34,206** |
| `lk_transport` | **`lk_tick`** | 17,520 |
| `lk_metered_subdivide` | **`lk_tick`** | **16,962** |
| `lk_transport` | `lk_metered_subdivide` | 1,086 |
| `lk_fuse` | `lk_metered_subdivide` | 684 |

**swap, k = 100:** `lk_transport` loses to `lk_tick` 69,566×,
`lk_fuse` 47,079×, `lk_metered_subdivide` 25,524×.

## What this changes

1. **Removal is blocked** — F-LK22 confirmed, the paper's methods
   appendix goes to seven items, and the L\*/k prefactor is
   re-attributed. Coexistence still exists; its quantitative law has a
   scheduler term.
2. **Production is blocked too** — not anticipated by F-LK22. Subdivide
   loses to tick 16,962 times in 3000 pump steps, because the seed's
   filament neighbours are tick-eligible whenever they are counting down.
   This *is* the "production self-throttling" mechanism Q7-4 demanded,
   and it explains why p falls with k: larger k ⟹ neighbours tick-eligible
   for longer ⟹ more refused subdivisions.
3. **The pump cap is fully explained.** Q7-1 measured exactly 6.0 chains
   × ~2.0 links. Each chain holds one ticking node and one
   expired-but-blocked node; the same clock gates both the fuse that
   would shorten the chain and the subdivision that would lengthen it, so
   the population sits at the structural fixed point 6 × 2 = 12
   regardless of k. This replaces the paper's "production
   self-throttling" sentence, which named an effect without a mechanism.
4. **Measurement back-action.** The countdown mechanism was introduced to
   *measure* a balance and instead participates in it — the gate variable
   is not inert with respect to the dynamics it gates. This is the most
   general lesson of the round and belongs in the methods appendix.

## Not fixed in v7 (hazard §8.2)

A tick guard `countdown > 0 ∧ degree ≠ 2` would exempt interior chain
nodes from ticking and remove the blocking, but it changes lifetime
semantics (interior nodes would stop aging) and would invalidate the A4
sweep it is meant to explain. **Recorded as a v8 design option; v7
measures and reports.** A cheaper alternative also deferred: give
`lk_tick` the *lowest* priority so it yields instead of winning, which
preserves lifetime semantics for unblocked nodes but makes ρ < 1 and so
trades one bias for another — it would need its own AG-style gate.
