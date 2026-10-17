//! Shared BFS-ball computation (LK plan §2, required refactor): one BFS
//! pass per source yields BOTH the ball-growth profile `n_of_r` (consumed
//! by the d_eff estimator in `hg-eval::geometry`) and the boundary-cut
//! profile `c_of_r` (consumed by the `LocalityObserver`). BFS treats an
//! N-ary hyperedge as an endpoint clique — the same adjacency as
//! `hg_core::compute_wl_hash`'s ball walk and the previous d_eff code.

use hg_core::{EdgeId, GraphSnapshot, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Per-source ball profile. Both vectors have length `r_max + 1`.
///
/// `n_of_r[r]`: nodes within graph distance r of the source (cumulative;
/// `n_of_r[0] == 1`).
/// `c_of_r[r]`: hyperedges with at least one endpoint inside B(s, r) and
/// at least one outside (straddling k-ary edges count once).
pub struct BallProfile {
    pub n_of_r: Vec<u32>,
    pub c_of_r: Vec<u32>,
}

/// Compute ball profiles for the given sources in one BFS pass per source.
pub fn ball_profiles(
    snapshot: &GraphSnapshot<'_>,
    sources: &[NodeId],
    r_max: usize,
) -> Vec<BallProfile> {
    sources
        .iter()
        .map(|&s| ball_profile(snapshot, s, r_max))
        .collect()
}

fn ball_profile(snapshot: &GraphSnapshot<'_>, source: NodeId, r_max: usize) -> BallProfile {
    // Layered BFS, ported verbatim from the GeometryObserver's ball_growth
    // (d_eff values must not drift: fixture-parity is a standing test).
    let mut dist: HashMap<NodeId, usize> = HashMap::new();
    dist.insert(source, 0);
    let mut queue: VecDeque<NodeId> = VecDeque::new();
    queue.push_back(source);

    let mut counts = vec![0u32; r_max + 1];
    counts[0] = 1;

    let mut visited_edges: HashSet<EdgeId> = HashSet::new();
    while let Some(v) = queue.pop_front() {
        let d = dist[&v];
        if d >= r_max {
            continue;
        }
        for edge_id in snapshot.edges_incident_to(v) {
            // Each hyperedge is a clique among its endpoints: once BFS has
            // expanded it from any endpoint, revisits add nothing.
            if !visited_edges.insert(*edge_id) {
                continue;
            }
            if let Some(edge) = snapshot.get_edge(*edge_id) {
                for &nb in &edge.nodes {
                    if !dist.contains_key(&nb) && snapshot.get_node(nb).is_some() {
                        dist.insert(nb, d + 1);
                        counts[d + 1] += 1;
                        queue.push_back(nb);
                    }
                }
            }
        }
    }

    // Cumulative: N(r) = nodes within distance r.
    for r in 1..=r_max {
        counts[r] += counts[r - 1];
    }

    // Cut counting on the same distance map: an edge straddles B(s, r)
    // iff min-endpoint-distance <= r < max-endpoint-distance (unreached
    // endpoints have distance infinity). Difference array keeps this O(E).
    let mut cut_delta = vec![0i64; r_max + 2];
    for (_, edge) in snapshot.edges() {
        let mut dmin = usize::MAX;
        let mut unreached = false;
        let mut dmax = 0usize;
        for &n in &edge.nodes {
            match dist.get(&n) {
                Some(&d) => {
                    dmin = dmin.min(d);
                    dmax = dmax.max(d);
                }
                None => unreached = true,
            }
        }
        if dmin == usize::MAX || dmin > r_max {
            continue; // no endpoint inside any ball we report
        }
        let hi = if unreached { r_max } else { dmax.saturating_sub(1).min(r_max) };
        if dmin <= hi {
            cut_delta[dmin] += 1;
            cut_delta[hi + 1] -= 1;
        }
    }
    let mut c_of_r = vec![0u32; r_max + 1];
    let mut acc = 0i64;
    for (r, c) in c_of_r.iter_mut().enumerate() {
        acc += cut_delta[r];
        *c = acc as u32;
    }

    BallProfile { n_of_r: counts, c_of_r }
}

/// Mean N(r) and mean C(r) across sources, in source order (summation
/// order matters for float determinism; keep it stable).
pub fn mean_profiles(profiles: &[BallProfile], r_max: usize) -> (Vec<f64>, Vec<f64>) {
    let n_sources = profiles.len().max(1) as f64;
    let mut mean_n = vec![0f64; r_max + 1];
    let mut mean_c = vec![0f64; r_max + 1];
    for p in profiles {
        for r in 0..=r_max {
            mean_n[r] += p.n_of_r[r] as f64;
            mean_c[r] += p.c_of_r[r] as f64;
        }
    }
    for r in 0..=r_max {
        mean_n[r] /= n_sources;
        mean_c[r] /= n_sources;
    }
    (mean_n, mean_c)
}

/// The non-degenerate radius band used by the d_eff fit — and, by the LK
/// plan's mandate, by the cut-scaling fit too: r >= 1 (log 0 undefined),
/// stop at saturation (ball has swallowed `saturation_frac` of the graph)
/// or when growth stalls (past every source's reach).
pub fn growth_band(mean_n: &[f64], n_total: usize, saturation_frac: f64) -> Vec<usize> {
    let mut band = Vec::new();
    let mut prev_mean = 1.0;
    for (r, &mean) in mean_n.iter().enumerate().skip(1) {
        if mean >= saturation_frac * n_total as f64 {
            break;
        }
        if mean <= prev_mean {
            break;
        }
        band.push(r);
        prev_mean = mean;
    }
    band
}

/// Least-squares fit y = a + slope*x with r² goodness. `None` when fewer
/// than `min_points` points or x has no variance.
pub fn fit_loglog(pts: &[(f64, f64)], min_points: usize) -> Option<(f64, f64)> {
    if pts.len() < min_points {
        return None;
    }
    let n = pts.len() as f64;
    let mean_x = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let sxy: f64 = pts.iter().map(|p| (p.0 - mean_x) * (p.1 - mean_y)).sum();
    let sxx: f64 = pts.iter().map(|p| (p.0 - mean_x).powi(2)).sum();
    let syy: f64 = pts.iter().map(|p| (p.1 - mean_y).powi(2)).sum();
    if sxx <= f64::EPSILON {
        return None;
    }
    let slope = sxy / sxx;
    let r2 = if syy <= f64::EPSILON {
        1.0 // y constant and fitted exactly: a flat, perfect fit
    } else {
        (sxy * sxy) / (sxx * syy)
    };
    Some((slope, r2))
}
