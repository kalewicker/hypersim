//! Q7-1 (v7 §2): census-zero resolution. Chain-length distributions over
//! the persisted EXP-L17 cells, to decide whether the zero wormhole
//! census in the coexistence regime is a DETECTION-THRESHOLD zero or a
//! SAMPLING zero.
//!
//! The two instruments differ in kind:
//!   * `wormhole_count_est` = wormhole_frac × edge_count, where the frac
//!     comes from a RANDOM SAMPLE of `detour_sample_edges` (default 256)
//!     edges. It can read zero because no filament edge was sampled.
//!   * an interior edge of a chain of n filament nodes has detour
//!     2n+2 (the F-LK14 arithmetic), so it registers as a wormhole only
//!     when 2n+2 > R_max = 12, i.e. n ≥ 6. It can read zero because
//!     every chain is shorter than that.
//!
//! The chain census is exact and unsampled, so it separates them: if
//! p95 ≥ 6 the population IS in range and a zero is a sampling artifact;
//! if p95 < 6 the population is genuinely below the detection domain.
//!
//! Q7_RUN_DIRS = colon-separated run dirs containing snapshots/.
//! Reports means over the PRODUCTION ERA (read from leakage.csv), the
//! same segment the standing census and p use.

use hg_core::PropertyValue;

const GENERATION: u32 = 6;
/// Detour cap the runs were configured with (`[leakage] detour_r_max`
/// default 12). A chain of n filament nodes has interior detour 2n+2.
const R_MAX: usize = 12;

fn visible_min_chain() -> u32 {
    // smallest n with 2n+2 > R_MAX
    ((R_MAX as u32) / 2).max(1) // 2n+2 > 12 → n > 5 → n ≥ 6
}

#[test]
#[ignore]
fn q7_1_chain_census() {
    let dirs = std::env::var("Q7_RUN_DIRS").expect("Q7_RUN_DIRS=dir1:dir2:…");
    let vis = visible_min_chain();
    println!("(visible when chain length ≥ {vis}: detour 2n+2 > R_max={R_MAX})");
    println!(
        "cell | era_w | chains n | len p50 | len p95 | len max | gen>0 L* | links/chain | \
         %chains≥{vis} | wh_est(sampled) | wh_EXACT | VERDICT"
    );
    for dir in dirs.split(':').filter(|s| !s.is_empty()) {
        let path = std::path::Path::new(dir);
        let snaps = hg_eval::cone::load_snapshots(path.join("snapshots").as_path())
            .unwrap_or_else(|e| panic!("{dir}: {e}"));
        if snaps.is_empty() {
            println!("{dir} | (no snapshots)");
            continue;
        }
        // Production era from leakage.csv (n_subdivisions rising), plus
        // the per-window wormhole_count_est for cross-reference.
        let leak = std::fs::read_to_string(path.join("leakage.csv")).unwrap_or_default();
        let lrows: Vec<(u64, f64, f64)> = leak
            .lines()
            .skip(2)
            .filter_map(|l| {
                let f: Vec<&str> = l.split(',').collect();
                Some((
                    f.first()?.parse().ok()?,
                    f.get(15)?.parse().ok()?,
                    f.get(10)?.parse().ok().unwrap_or(0.0),
                ))
            })
            .collect();
        let era_steps: std::collections::HashSet<u64> = (1..lrows.len())
            .filter(|&i| lrows[i].1 > lrows[i - 1].1)
            .map(|i| lrows[i].0)
            .collect();
        let wh_csv: f64 = {
            let v: Vec<f64> = lrows
                .iter()
                .filter(|(s, _, _)| era_steps.is_empty() || era_steps.contains(s))
                .map(|&(_, _, w)| w)
                .collect();
            v.iter().sum::<f64>() / v.len().max(1) as f64
        };

        let mut acc: Vec<(u32, f32, f32, u32, usize, f64)> = Vec::new();
        for (step, store) in &snaps {
            if !era_steps.is_empty() && !era_steps.contains(step) {
                continue;
            }
            let c = hg_eval::chains::chain_census(store);
            let pop = store
                .nodes()
                .filter(|(_, d)| {
                    matches!(d.get_property(GENERATION), Some(PropertyValue::Float(g)) if g > 0.0)
                })
                .count();
            // Fraction of chains at or above the detection threshold,
            // recovered from the exact per-chain lengths.
            let lens = chain_lengths(store);
            let frac_vis = if lens.is_empty() {
                0.0
            } else {
                lens.iter().filter(|&&l| l >= vis).count() as f64 / lens.len() as f64
            };
            acc.push((
                c.chains_count,
                c.chain_length_p50,
                c.chain_length_p95,
                c.chain_length_max,
                pop,
                frac_vis,
            ));
        }
        if acc.is_empty() {
            println!("{dir} | (no era windows)");
            continue;
        }
        let n = acc.len() as f64;
        let m = |f: &dyn Fn(&(u32, f32, f32, u32, usize, f64)) -> f64| -> f64 {
            acc.iter().map(f).sum::<f64>() / n
        };
        let chains = m(&|a| a.0 as f64);
        let p50 = m(&|a| a.1 as f64);
        let p95 = m(&|a| a.2 as f64);
        let lmax = m(&|a| a.3 as f64);
        let pop = m(&|a| a.4 as f64);
        let fvis = m(&|a| a.5);
        let verdict = if p95 >= vis as f64 {
            "IN-RANGE (zero ⇒ sampling)"
        } else if lmax >= vis as f64 {
            "MIXED (tail in range)"
        } else {
            "OUT-OF-RANGE (sub-threshold)"
        };
        // EXACT standing wormhole count on the filament subgraph — the
        // instrument of record per v7 §2 when a cell is in range. Every
        // edge incident to a gen>0 node, measured with the real detour
        // (no sampling). Subsampled over era windows for cost.
        let era_snaps: Vec<&(u64, hg_core::HypergraphStore)> = snaps
            .iter()
            .filter(|(s, _)| era_steps.is_empty() || era_steps.contains(s))
            .collect();
        let stride = (era_snaps.len() / 8).max(1);
        let exact: Vec<f64> = era_snaps
            .iter()
            .step_by(stride)
            .map(|(_, store)| {
                let snap = store.snapshot();
                let filament: std::collections::HashSet<hg_core::NodeId> = store
                    .nodes()
                    .filter(|(_, d)| {
                        matches!(d.get_property(GENERATION), Some(PropertyValue::Float(g)) if g > 0.0)
                    })
                    .map(|(id, _)| id)
                    .collect();
                let mut edges: std::collections::HashSet<hg_core::EdgeId> =
                    std::collections::HashSet::new();
                for &n in &filament {
                    for e in store.edges_incident_to(n) {
                        edges.insert(*e);
                    }
                }
                edges
                    .iter()
                    .filter(|&&e| {
                        let d = hg_observe::edge_detour(&snap, e, R_MAX);
                        !d.is_finite() || d as usize > R_MAX
                    })
                    .count() as f64
            })
            .collect();
        let exact_mean = exact.iter().sum::<f64>() / exact.len().max(1) as f64;
        let name = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| dir.into());
        println!(
            "{name} | {} | {chains:.1} | {p50:.1} | {p95:.1} | {lmax:.1} | {pop:.1} | {:.2} | \
             {:.0}% | {wh_csv:.1} | {exact_mean:.1} | {verdict}",
            acc.len(),
            if chains > 0.0 { pop / chains } else { 0.0 },
            fvis * 100.0,
        );
    }
}

/// Exact per-chain lengths (node counts of maximal degree-2 gen>0 paths)
/// — the same construction as `chain_census`, exposed as the raw list so
/// the in-range fraction can be computed.
fn chain_lengths(store: &hg_core::HypergraphStore) -> Vec<u32> {
    use std::collections::{HashMap, HashSet};
    let mut degree: HashMap<hg_core::NodeId, u32> = HashMap::new();
    let mut adjacency: HashMap<hg_core::NodeId, Vec<hg_core::NodeId>> = HashMap::new();
    for (_, edge) in store.edges() {
        let mut ends: Vec<hg_core::NodeId> = edge.nodes.to_vec();
        ends.sort_unstable();
        ends.dedup();
        for &u in &ends {
            *degree.entry(u).or_default() += 1;
            for &v in &ends {
                if v != u {
                    adjacency.entry(u).or_default().push(v);
                }
            }
        }
    }
    let eligible: HashSet<hg_core::NodeId> = store
        .nodes()
        .filter(|(id, data)| {
            degree.get(id).copied() == Some(2)
                && matches!(data.get_property(GENERATION), Some(PropertyValue::Float(g)) if g > 0.0)
        })
        .map(|(id, _)| id)
        .collect();
    let mut seen: HashSet<hg_core::NodeId> = HashSet::new();
    let mut out = Vec::new();
    for &start in &eligible {
        if seen.contains(&start) {
            continue;
        }
        let mut stack = vec![start];
        seen.insert(start);
        let mut size = 0u32;
        while let Some(u) = stack.pop() {
            size += 1;
            for v in adjacency.get(&u).into_iter().flatten() {
                if eligible.contains(v) && seen.insert(*v) {
                    stack.push(*v);
                }
            }
        }
        out.push(size);
    }
    out
}
