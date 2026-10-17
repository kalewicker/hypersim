# Q7-1 — Census-Zero Resolution (chain census over EXP-L17)

**Verdict: MIXED across k, and every census zero is `null (out-of-range)`
— a detection-threshold zero, not a sampling artifact.** Additionally,
two facts the paper's §5 asserts are wrong and are corrected here: the
detour formula is regime-dependent, and the swap population is not
"~15 chains" but many mostly-single-link chains with a heavy tail.

Data: persisted `runs/exp_l17/` (no new simulation). Instruments:
`crates/hg-eval/tests/q7_1_chain_census.rs` (exact chain census + exact
unsampled wormhole count) and `q7_1_detour_domain.rs` (detour vs chain
length). Means over the production era (n_subdivisions rising).

---

## 1. Distributions

Chain length = node count of a maximal degree-2, generation>0 path.
`wh_est` is the shipped sampled census (256-edge sample);
`wh_EXACT` is every filament-incident edge measured with the real detour.

| cell | chains n | p50 | p95 | max | L\* | links/chain | wh_est | wh_EXACT |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| swap vanish k=10 | 2.7 | 1.0 | 1.0 | 1.0 | 2.7 | 1.00 | 0.0 | **0.0** |
| swap vanish k=30 | 6.4 | 1.0 | 1.1 | 1.1 | 6.5 | 1.01 | 0.0 | **0.0** |
| swap vanish k=100 | 16.6 | 1.1 | 6.2 | 7.9 | 36.0 | 2.16 | 0.0 | **0.0** |
| swap vanish k=300 | 23.1 | 1.5 | 14.4 | 20.0 | 81.8 | 3.54 | 38.7 | **33.8** |
| swap transfer k=10 | 12.8 | 1.0 | 1.0 | 1.0 | 12.8 | 1.00 | 0.0 | **0.0** |
| swap transfer k=30 | 35.1 | 1.0 | 1.3 | 2.4 | 37.7 | 1.07 | 0.0 | **0.0** |
| swap transfer k=100 | 56.6 | 1.0 | 7.0 | 23.9 | 127.0 | 2.24 | 58.8 | **54.4** |
| swap transfer k=300 | 55.5 | 1.1 | 13.1 | 80.9 | 241.6 | 4.35 | 190.2 | **165.1** |
| pump vanish k=10 | **6.0** | 2.0 | 2.5 | 2.5 | 12.4 | 2.09 | 0.0 | **0.0** |
| pump vanish k=100 | **6.0** | 2.0 | 2.2 | 2.2 | 12.2 | 2.03 | 0.0 | **0.0** |
| pump transfer k=100 | 28.5 | 1.0 | 2.0 | 2.1 | 34.6 | 1.21 | 0.0 | **0.0** |
| ctrl nofuse (no removal) | 6.0 | 143.8 | 144.1 | 144.1 | 862.4 | 143.73 | 861.4 | **856.0** |

## 2. The zeros are not sampling

The exact, unsampled instrument **agrees with the sampler in every
cell**, including reading exactly 0.0 wherever the sampler reads 0.0.
Where the population is in range the two track closely (33.8 vs 38.7;
54.4 vs 58.8; 165.1 vs 190.2; 856.0 vs 861.4 — the sampler runs a few
percent high, consistent with scaling a small sample). **The plan's
"sampling artifact" row is refuted by direct measurement.** The
tracked/exact instrument is nonetheless adopted as the instrument of
record for standing-census claims, since it carries no sampling variance.

## 3. The detection domain is REGIME-DEPENDENT (correction to §5)

The track has used detour = 2L+2 since F-LK14. Measuring detour directly
against chain length shows that formula is **pump-specific**:

| regime | chain L | measured mean detour | implied law |
|---|---:|---:|---|
| pump | 2 | 6.0 | 2L+2 |
| pump | 3 | 7.0 | ~2L+2 |
| swap | 1 | 3.7 | L+3 |
| swap | 2 | 5.4 | L+3 |
| swap | 3 | 5.0 | ~L+3 |
| swap | 25, 40 | ∞ (all edges) | ≫ R_max |

The reason is structural and was already known in a different form:
F-LK14 derived 2L+2 because *in the pump* the seed's own bulk edges are
consumed into chains, so a return path must descend a **second** chain.
In the swap regime production is distributed and the ambient lattice
around a filament is intact, so the return path is one walk back along
the chain plus a short lattice hop — L+3.

Consequently the visibility threshold at R_max = 12 is:

- **pump:** 2L+2 > 12 → **L ≥ 6**
- **swap:** L+3 > 12 → **L ≥ 10**

Applying the correct threshold, every reading in §1 is consistent:
swap k=100/vanish has max chain 7.9 (< 10) and reads zero; k=300/vanish
reaches 20 and reads 33.8; transfer k=100 reaches 23.9 and reads 54.4.
There is no anomaly left to explain.

## 4. Morphology: not 15 chains

The F-LK21 ambiguity was computed against "the ~15 chains EXP-L12
measured". That figure came from a **no-removal** configuration and does
not transfer. Under removal the swap population is organized as **many
more, much shorter** structures whose count grows with k (2.7 → 6.4 →
16.6 → 23.1 vanish; 12.8 → 35.1 → 56.6 → 55.5 transfer), with a **median
chain length of 1** at every k and a heavy tail (max 80.9 at k=300).
So the coexisting population is predominantly *single-link refinements*,
with a minority of long filaments carrying all the wormhole signal:
only 7–14% of chains are in range even in the cells that register.

The pump, by contrast, holds **exactly 6.0 chains** at every k with
~2 links each — the "6 chains × 2 links" arithmetic the v7 §4 cap check
asks about, confirmed to the decimal.

## 5. Crossover

The k at which chains cross into the detection domain (L ≥ 10, swap):

- **swap / transfer:** between k = 30 (max 2.4) and k = 100 (max 23.9).
- **swap / vanish:** between k = 100 (max 7.9) and k = 300 (max 20.0).
- **pump:** never, at any k tested — max chain length is ~2 regardless.

## 6. Consequences

1. **Paper §5's explanation is corrected**, not discarded: the zeros are
   threshold zeros, but the threshold is L ≥ 10 in the regime where
   coexistence lives, not L ≥ 6, and the population that is invisible is
   invisible because it is *made of single links*, not because it is
   ~2 links per chain across 15 chains.
2. **The title claim is supported only at high k.** For k ≤ 30 (and for
   the pump at every k) the coexistence regime is entirely local by the
   track's own definition — sub-threshold refinement, hazard §8.1's
   "clean result", to be written as one. At k = 100–300 in swap there is
   a genuine standing wormhole population (54–165 edges, exact).
3. **Instrument-range preconditions are now a schema field**, per v7
   §1.1: `LeakageRow.census_in_range` is computed exactly each window as
   the detour of an edge on the longest filament chain (detour grows
   with chain length, so if that edge is not a wormhole none is). This
   deliberately encodes no detour formula, so it is correct in both
   regimes. A census zero with `census_in_range = false` is
   `null (out-of-range)` and is not a population claim.
4. **Q7-5's instrument of record:** the exact filament-edge count above,
   with the standing-population criterion met by the gen>0 census; the
   best-behaved cells for EXP-L7R are swap × transfer at k = 100–300,
   which are the only cells that are simultaneously in range and
   long-lived.
