//! Graph geometry: the effective-dimension estimator (metrics `d_eff`).
//!
//! Ball-growth method: for a d-dimensional structure the number of nodes
//! within graph distance r of a source scales as N(r) ~ r^d, so d_eff is
//! the least-squares slope of log N(r) vs log r, averaged over sampled
//! sources. BFS treats an N-ary hyperedge as making all its endpoints
//! pairwise distance 1 — the same adjacency the WL ball walk uses.
//!
//! The BFS itself lives in `hg_observe::balls::ball_profiles` (LK1 shared
//! refactor): one pass yields both the growth profile consumed here and
//! the boundary-cut profile consumed by the `LocalityObserver`. This
//! module keeps the sampling protocol and the fit unchanged — d_eff
//! values are pinned by the fixture tests.
//!
//! Sampling is seeded (ChaCha8 from the run seed + window), never
//! thread_rng, per docs/reproducibility_policy.md.

use hg_core::{GraphSnapshot, NodeId};
use hg_observe::balls::{ball_profiles, fit_loglog, growth_band, mean_profiles, BallProfile};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Sources sampled per window. All-pairs BFS every window would be far
/// too expensive on large graphs; a few dozen sources is plenty for a
/// slope estimate.
pub const DEFF_SOURCES: usize = 32;
/// BFS radius cap: bounds cost, and the log-log fit uses the small-r
/// growth regime anyway.
pub const DEFF_R_MAX: usize = 10;
/// Points where the ball has swallowed this fraction of the graph are
/// saturated (slope artificially flattens toward 0) and excluded.
pub const DEFF_SATURATION_FRAC: f64 = 0.9;
/// Minimum usable (log r, log N) points for a fit.
pub const DEFF_MIN_FIT_POINTS: usize = 3;

/// The d_eff source-sampling protocol (reused verbatim by the leakage
/// track): seeded index sample over the snapshot's stable node order.
pub fn sample_sources(snapshot: &GraphSnapshot<'_>, seed: u64, n: usize) -> Vec<NodeId> {
    // Deterministic candidate order (slotmap iteration is stable for a
    // given construction sequence, which the engine guarantees).
    let ids: Vec<NodeId> = snapshot.node_ids().collect();
    let n_sources = n.min(ids.len());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    rand::seq::index::sample(&mut rng, ids.len(), n_sources)
        .iter()
        .map(|ix| ids[ix])
        .collect()
}

/// The d_eff fit over already-computed ball profiles: mean N(r) across
/// sources, non-degenerate band (r >= 1, stop at saturation or stalled
/// growth), least-squares slope of log N vs log r.
pub fn dimension_from_profiles(profiles: &[BallProfile], n_total: usize) -> Option<f64> {
    if profiles.is_empty() {
        return None;
    }
    let (mean_n, _) = mean_profiles(profiles, DEFF_R_MAX);
    let band = growth_band(&mean_n, n_total, DEFF_SATURATION_FRAC);
    let pts: Vec<(f64, f64)> = band
        .iter()
        .map(|&r| ((r as f64).ln(), mean_n[r].ln()))
        .collect();
    fit_loglog(&pts, DEFF_MIN_FIT_POINTS).map(|(slope, _)| slope)
}

/// Estimate the effective dimension of the snapshot's graph.
///
/// `Some(slope)` when the log-log fit succeeds on at least
/// [`DEFF_MIN_FIT_POINTS`] unsaturated radii; `None` only for graphs too
/// small or degenerate to have a meaningful dimension (first windows of
/// a tiny seed, isolated dust, a ball that saturates immediately).
pub fn effective_dimension(snapshot: &GraphSnapshot<'_>, seed: u64) -> Option<f64> {
    let n_total = snapshot.node_count();
    if n_total < 8 {
        return None;
    }
    let sources = sample_sources(snapshot, seed, DEFF_SOURCES);
    let profiles = ball_profiles(snapshot, &sources, DEFF_R_MAX);
    dimension_from_profiles(&profiles, n_total)
}
