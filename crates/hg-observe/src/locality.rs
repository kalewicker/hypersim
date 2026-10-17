//! `LocalityObserver` (LK1) and the two structural leakage instruments
//! (hypersim_leakage_plan_v1.md §1.1–1.2, normative):
//!
//! **Edge detour distance**: `detour(e) = max over unordered pairs
//! {u,v} ⊆ V(e) of d_{G−e}(u,v)`, truncated BFS capped at `r_max`;
//! any pair beyond the cap (or disconnected) makes `e` a wormhole
//! (detour = ∞). `max`, not `min`: an edge is geometric only if ALL of
//! its endpoints are locally reachable without it.
//!
//! **Boundary-cut scaling**: fit of log C vs log N over the same
//! non-degenerate radius band the d_eff fit uses, from the shared
//! `balls::ball_profiles` pass. `area_law_defect = cut_exponent −
//! (d_eff − 1)/d_eff`; clean d-dimensional geometry → ≈ 0,
//! expander/tree → cut_exponent → 1.
//!
//! Known degeneracy (documented, not a bug): a bare ring has detour
//! N−1 on every edge — under the BFS cap those read as wormholes. High
//! girth without path redundancy is indistinguishable from non-locality
//! by the detour instrument ALONE; instruments are interpreted jointly
//! (the ring's cut_exponent ≈ 0 disambiguates it from an expander).
//!
//! Sampling is seeded (ChaCha8), never thread_rng, per
//! docs/reproducibility_policy.md. Observers are read-only by
//! construction (`GraphSnapshot`).

use crate::balls::{ball_profiles, fit_loglog, growth_band, mean_profiles, BallProfile};
use crate::observer::Observer;
use hg_core::{EdgeId, GraphSnapshot, NodeId};
use hg_schema::{ClaimType, LeakageRow};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, VecDeque};

/// Log-spaced detour histogram bins (closure-defect micro features, LK4):
/// detour in [1,2], (2,4], (4,8], (8,12], (12,r_max], ∞.
pub const DETOUR_HIST_BINS: usize = 6;

/// `[leakage]` knobs consumed by the observer (defaults per Appendix B).
#[derive(Clone, Copy, Debug)]
pub struct LeakageSettings {
    pub cadence_steps: u64,
    pub detour_sample_edges: usize,
    pub detour_r_max: usize,
    pub ball_sources: usize,
}

impl Default for LeakageSettings {
    fn default() -> Self {
        Self {
            cadence_steps: 500,
            detour_sample_edges: 256,
            detour_r_max: 12,
            ball_sources: 32,
        }
    }
}

/// Result of one detour census. Detours are `f32::INFINITY` for wormholes.
#[derive(Clone, Debug, Default)]
pub struct DetourCensus {
    /// Sampled detour values, sorted ascending (∞ last).
    pub detours: Vec<f32>,
    pub p50: f32,
    pub p95: f32,
    pub wormhole_frac: f32,
    /// The sampled edges whose detour exceeded the cap (v2 §1.1: the
    /// wormhole-edge subgraph these induce is the handle census input).
    pub wormhole_edges: Vec<EdgeId>,
    /// Normalized log-spaced histogram ([`DETOUR_HIST_BINS`] fractions).
    pub histogram: [f64; DETOUR_HIST_BINS],
}

/// Detour distance of one hyperedge: max over endpoint pairs of the
/// graph distance with `e` removed, truncated at `r_max` (∞ beyond).
pub fn edge_detour(snapshot: &GraphSnapshot<'_>, e: EdgeId, r_max: usize) -> f32 {
    let Some(edge) = snapshot.get_edge(e) else {
        return f32::INFINITY;
    };
    // Distinct endpoints (repeated-endpoint edges contribute no pair for
    // the repeat; a pair {u,u} has distance 0).
    let mut endpoints: Vec<NodeId> = edge.nodes.to_vec();
    endpoints.sort_unstable();
    endpoints.dedup();
    if endpoints.len() < 2 {
        return 0.0; // no pair to detour between: vacuously geometric
    }

    let mut worst = 0usize;
    // BFS from each endpoint but the last covers all unordered pairs.
    for (i, &u) in endpoints.iter().enumerate().take(endpoints.len() - 1) {
        let targets = &endpoints[i + 1..];
        let mut remaining: Vec<NodeId> = targets.to_vec();
        let mut dist: HashMap<NodeId, usize> = HashMap::new();
        dist.insert(u, 0);
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        queue.push_back(u);
        while let Some(v) = queue.pop_front() {
            let d = dist[&v];
            if d >= r_max || remaining.is_empty() {
                continue;
            }
            for edge_id in snapshot.edges_incident_to(v) {
                if *edge_id == e {
                    continue; // the detour is measured in G − e
                }
                if let Some(adj) = snapshot.get_edge(*edge_id) {
                    for &nb in &adj.nodes {
                        if !dist.contains_key(&nb) && snapshot.get_node(nb).is_some() {
                            dist.insert(nb, d + 1);
                            remaining.retain(|&t| t != nb);
                            queue.push_back(nb);
                        }
                    }
                }
            }
        }
        for t in targets {
            match dist.get(t) {
                Some(&d) => worst = worst.max(d),
                None => return f32::INFINITY, // beyond r_max or disconnected
            }
        }
    }
    worst as f32
}

/// Sampled detour census over `k_e` edges (seeded sampler, without
/// replacement; arity-<2 edges are excluded as pairless).
pub fn detour_census(
    snapshot: &GraphSnapshot<'_>,
    seed: u64,
    k_e: usize,
    r_max: usize,
) -> DetourCensus {
    // Deterministic candidate order (slotmap iteration is stable for a
    // given construction sequence — same argument as the d_eff sampler).
    let candidates: Vec<EdgeId> = snapshot
        .edges()
        .filter(|(_, e)| {
            let mut nodes = e.nodes.to_vec();
            nodes.sort_unstable();
            nodes.dedup();
            nodes.len() >= 2
        })
        .map(|(id, _)| id)
        .collect();
    if candidates.is_empty() {
        return DetourCensus::default();
    }
    let n_sample = k_e.min(candidates.len());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let picked = rand::seq::index::sample(&mut rng, candidates.len(), n_sample);

    let mut wormhole_edges: Vec<EdgeId> = Vec::new();
    let mut detours: Vec<f32> = picked
        .iter()
        .map(|ix| {
            let e = candidates[ix];
            let d = edge_detour(snapshot, e, r_max);
            if d.is_infinite() {
                wormhole_edges.push(e);
            }
            d
        })
        .collect();
    detours.sort_by(|a, b| a.total_cmp(b));

    let n = detours.len();
    let pct = |q: f64| -> f32 {
        // Nearest-rank percentile.
        let rank = ((q * n as f64).ceil() as usize).clamp(1, n);
        detours[rank - 1]
    };
    let wormholes = detours.iter().filter(|d| d.is_infinite()).count();

    let mut histogram = [0f64; DETOUR_HIST_BINS];
    for &d in &detours {
        let bin = if d.is_infinite() {
            5
        } else if d <= 2.0 {
            0
        } else if d <= 4.0 {
            1
        } else if d <= 8.0 {
            2
        } else if d <= 12.0 {
            3
        } else {
            4
        };
        histogram[bin] += 1.0;
    }
    for h in histogram.iter_mut() {
        *h /= n as f64;
    }

    DetourCensus {
        p50: pct(0.50),
        p95: pct(0.95),
        wormhole_frac: wormholes as f32 / n as f32,
        wormhole_edges,
        detours,
        histogram,
    }
}

/// Connected components of the wormhole-edge subgraph (v2 §1.1): edges
/// sharing an endpoint are one handle. Edge count measures refinement of
/// a handle; component count measures handles.
pub fn wormhole_components(snapshot: &GraphSnapshot<'_>, wormholes: &[EdgeId]) -> u32 {
    let mut node_root: HashMap<NodeId, usize> = HashMap::new();
    let mut parent: Vec<usize> = (0..wormholes.len()).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for (ix, e) in wormholes.iter().enumerate() {
        let Some(edge) = snapshot.get_edge(*e) else { continue };
        for &n in &edge.nodes {
            match node_root.get(&n) {
                Some(&other) => {
                    let ra = find(&mut parent, ix);
                    let rb = find(&mut parent, other);
                    parent[rb] = ra;
                }
                None => {
                    node_root.insert(n, ix);
                }
            }
        }
    }
    let mut roots: Vec<usize> = (0..wormholes.len())
        .map(|i| find(&mut parent, i))
        .collect();
    roots.sort_unstable();
    roots.dedup();
    roots.len() as u32
}

/// Calibrated phase classification (v2 §1.5). Thresholds come from the
/// LK0R p_peak calibration sweep (docs/fixture_calibration.md). The
/// detour channel is valid ONLY in Sparse. Two INDEPENDENT dissolution
/// signatures are needed because the cut channel is demoted (also
/// non-monotone across the crossover): tree/expander backgrounds show
/// HIGH defect, while a crossed-over small-world shows LOW defect but
/// INFLATED d_bfs (measured: sw(48², 5e-2) → defect 0.074, d_bfs 2.81
/// vs clean-2d 0.093/1.70 and clean-3d 0.015/2.44). Missing inputs are
/// conservatively Crossover — never Sparse by default.
///
/// Calibrated for 2d/3d lattice substrates; a substrate family with
/// genuine d_eff > 2.5 requires re-calibration (documented limitation).
/// v3 §1.1 provenance requirement: the calibration document version the
/// thresholds below were frozen against. Cited in every run manifest of
/// a leakage-enabled run; a threshold change here must bump this string.
pub const CLASSIFIER_CALIBRATION: &str = "fixture_calibration.md LK0R (2026-07-12)";

pub fn classify_phase(
    wormhole_frac: f32,
    area_law_defect: Option<f32>,
    d_bfs: Option<f32>,
) -> hg_schema::LeakagePhase {
    use hg_schema::LeakagePhase;
    let _ = wormhole_frac; // both low- and high-wf quadrants are Sparse-valid
    const DEFECT_DENSE: f32 = 0.28;
    const DEFECT_CROSSOVER: f32 = 0.20;
    const DBFS_DENSE: f32 = 2.7;
    const DBFS_CROSSOVER: f32 = 2.5;
    let (Some(defect), Some(d)) = (area_law_defect, d_bfs) else {
        return LeakagePhase::Crossover;
    };
    if defect >= DEFECT_DENSE || d >= DBFS_DENSE {
        LeakagePhase::Dense
    } else if defect >= DEFECT_CROSSOVER || d >= DBFS_CROSSOVER {
        LeakagePhase::Crossover
    } else {
        LeakagePhase::Sparse
    }
}

/// Cut-scaling fit from shared ball profiles: slope and r² of log C(r)
/// vs log N(r) over the d_eff radius band (points with C = 0 excluded —
/// log undefined; a disconnected dust ball has no boundary).
pub fn cut_fit_from_profiles(
    profiles: &[BallProfile],
    n_total: usize,
    r_max: usize,
    saturation_frac: f64,
    min_points: usize,
) -> Option<(f64, f64)> {
    let (mean_n, mean_c) = mean_profiles(profiles, r_max);
    let band = growth_band(&mean_n, n_total, saturation_frac);
    let pts: Vec<(f64, f64)> = band
        .iter()
        .filter(|&&r| mean_c[r] > 0.0)
        .map(|&r| (mean_n[r].ln(), mean_c[r].ln()))
        .collect();
    fit_loglog(&pts, min_points)
}

/// Everything one leakage emission computes; `row` is what lands in
/// leakage.csv, the histogram feeds the closure-defect micro vector.
#[derive(Clone, Debug)]
pub struct LeakageComputation {
    pub row: LeakageRow,
    pub detour_histogram: [f64; DETOUR_HIST_BINS],
}

/// One full leakage measurement over a snapshot. `sources` and their
/// `profiles` are supplied by the caller so the BFS pass is shared with
/// the d_eff estimator (LK1 cost budget: the ball BFS runs ONCE).
/// `d_eff` must be from the same window or `None` — never stale.
#[allow(clippy::too_many_arguments)]
pub fn compute_leakage(
    snapshot: &GraphSnapshot<'_>,
    step: u64,
    seed: u64,
    settings: &LeakageSettings,
    profiles: &[BallProfile],
    ball_r_max: usize,
    saturation_frac: f64,
    min_fit_points: usize,
    d_eff: Option<f64>,
) -> LeakageComputation {
    let census = detour_census(
        snapshot,
        // Distinct stream from the ball-source sampler.
        seed ^ 0x00DE_7011,
        settings.detour_sample_edges,
        settings.detour_r_max,
    );
    let cut = cut_fit_from_profiles(
        profiles,
        snapshot.node_count(),
        ball_r_max,
        saturation_frac,
        min_fit_points,
    );
    let cut_exponent = cut.map(|(s, _)| s as f32);
    let cut_fit_r2 = cut.map(|(_, r2)| r2 as f32);
    let area_law_defect = match (cut_exponent, d_eff) {
        (Some(ce), Some(d)) if d > 0.0 => Some(ce - ((d - 1.0) / d) as f32),
        _ => None,
    };
    let d_bfs = d_eff.map(|d| d as f32);
    let phase = classify_phase(census.wormhole_frac, area_law_defect, d_bfs);
    let cone_status = if phase == hg_schema::LeakagePhase::Sparse {
        hg_schema::ConeStatus::InPhase
    } else {
        hg_schema::ConeStatus::OutOfPhase
    };
    let wormhole_count_est =
        (census.wormhole_frac as f64 * snapshot.edge_count() as f64).round() as u32;
    let components = wormhole_components(snapshot, &census.wormhole_edges);
    LeakageComputation {
        row: LeakageRow {
            step,
            detour_p50: census.p50,
            detour_p95: census.p95,
            wormhole_frac: census.wormhole_frac,
            cut_exponent,
            cut_fit_r2,
            area_law_defect,
            d_bfs,
            d_spec: None,      // spectral-dimension observer: reserved
            dim_mismatch: None, // requires d_spec
            wormhole_count_est: Some(wormhole_count_est),
            wormhole_components: Some(components),
            phase,
            cone_status,
            // LK-R audit fields are filled by the runner when the metered
            // rule family is active; the instrument itself never sees them.
            e_total_now: None,
            n_subdivisions: None,
            e_fused_total: None,
            churn_events: None,
            audit_residual: None,
            claim_type: ClaimType::Estimated,
        },
        detour_histogram: census.histogram,
    }
}

/// Live leakage instrumentation (LK1). The runner drives it explicitly at
/// window boundaries (like the evaluator and causal observers) so d_eff
/// and the ball BFS are shared; the `Observer` impl exists for chains
/// that only have the trait seam.
pub struct LocalityObserver {
    pub settings: LeakageSettings,
    run_seed: u64,
    next_emit_at: u64,
    /// Rows accumulated this run (the runner drains them into the CSV).
    pub rows: Vec<LeakageRow>,
    /// Detour histogram per emitted row (closure-defect input), aligned
    /// with `rows`.
    pub histograms: Vec<[f64; DETOUR_HIST_BINS]>,
}

impl LocalityObserver {
    pub fn new(settings: LeakageSettings, run_seed: u64) -> Self {
        Self {
            settings,
            run_seed,
            next_emit_at: settings.cadence_steps,
            rows: Vec::new(),
            histograms: Vec::new(),
        }
    }

    /// Per-window seed, same derivation shape as the runner's d_eff seed
    /// (reproducible at any thread count).
    pub fn seed_for(&self, step: u64) -> u64 {
        self.run_seed ^ step.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x4C4B_0000
    }

    /// Cadence gate: true when `steps_observed` has crossed the next
    /// emission boundary (call once per window; advances the boundary).
    pub fn due(&mut self, steps_observed: u64) -> bool {
        if steps_observed >= self.next_emit_at {
            let c = self.settings.cadence_steps.max(1);
            self.next_emit_at = (steps_observed / c + 1) * c;
            true
        } else {
            false
        }
    }

    /// Record one emission computed by the runner (shared-BFS path).
    pub fn record(&mut self, computation: LeakageComputation) {
        self.histograms.push(computation.detour_histogram);
        self.rows.push(computation.row);
    }

    /// Self-contained emission (trait path / tests): samples its own ball
    /// sources with the d_eff protocol and computes everything, without a
    /// same-window d_eff (area_law_defect needs the caller-supplied one).
    pub fn observe_window(&mut self, snapshot: &GraphSnapshot<'_>, step: u64, d_eff: Option<f64>) {
        const BALL_R_MAX: usize = 10; // = geometry::DEFF_R_MAX (shared pass)
        const SATURATION: f64 = 0.9;
        const MIN_FIT: usize = 3;
        let seed = self.seed_for(step);
        let ids: Vec<NodeId> = snapshot.node_ids().collect();
        let n_sources = self.settings.ball_sources.min(ids.len());
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let sources: Vec<NodeId> = rand::seq::index::sample(&mut rng, ids.len(), n_sources)
            .iter()
            .map(|ix| ids[ix])
            .collect();
        let profiles = ball_profiles(snapshot, &sources, BALL_R_MAX);
        let comp = compute_leakage(
            snapshot, step, seed, &self.settings, &profiles, BALL_R_MAX, SATURATION, MIN_FIT,
            d_eff,
        );
        self.record(comp);
    }
}

impl Observer for LocalityObserver {
    fn on_window_complete(&mut self, snapshot: &GraphSnapshot<'_>, window: u64) {
        // Trait seam: no step count available, so the window index is the
        // cadence unit (cadence_steps treated as cadence_windows here).
        if self.due(window.saturating_add(1)) {
            self.observe_window(snapshot, window, None);
        }
    }
}
