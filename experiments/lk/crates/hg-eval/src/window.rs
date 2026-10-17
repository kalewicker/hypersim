//! Per-window feature computation (docs/metric_formulas.md).
//!
//! Every metric derives from the type histogram: WL(v, r=1) per live node.

use crate::thresholds::*;
use ahash::{AHashMap, AHashSet};
use hg_core::{compute_wl_class_map_snapshot, GraphSnapshot, NodeId, WlHash};

/// Snapshot-derived features for one window.
#[derive(Clone, Debug, Default)]
pub struct WindowFeatures {
    pub window: u64,
    pub n_nodes: usize,
    pub n_edges: usize,
    pub mean_degree: Option<f64>,
    /// Node-id churn: |sym-diff(live ids)| / |union| vs previous window.
    /// 0.0 = identical snapshots; the fixture-mode "activity".
    pub activity: f64,
    pub growth_ratio: f64,
    pub type_histogram_entropy: f64,
    pub dominant_type_fraction: f64,
    pub type_histogram_jsd: f64,
    pub entropy_rate: f64,
    pub new_type_bins: usize,
    pub new_type_bin_rate: f64,
    pub pi_normalized: Option<f64>,
    pub pi_sample_size: usize,
    pub pi_sample_strategy: &'static str,
    pub active_persistent_structures: usize,
    pub static_persistent_motifs: usize,
    pub static_repeated_motifs: usize,
    pub n_type_bins: usize,
    /// Type bins first seen this window (novelty-archive signatures).
    pub new_signatures: Vec<u64>,
    /// True from the second window onward (t-1 exists).
    pub has_prev: bool,
}

/// Stateful computer: holds window t-1's histogram and member sets.
pub struct WindowComputer {
    window_len: u64,
    prev: Option<PrevWindow>,
    seen_bins: AHashSet<WlHash>,
    window_ix: u64,
}

struct PrevWindow {
    hist: AHashMap<WlHash, usize>,
    members: AHashMap<WlHash, AHashSet<NodeId>>,
    live_ids: AHashSet<NodeId>,
    entropy: f64,
    /// Histogram mass = number of SAMPLED nodes (≤ pi_sample_cap) — the
    /// denominator for distribution math only.
    sample_total: usize,
    /// Full live node count — the denominator for growth_ratio. Distinct
    /// from sample_total: above the sampling cap they diverge, and using
    /// the sampled total here inflated growth into false Exploding
    /// classifications on graphs > 4096 nodes.
    n_nodes_full: usize,
}

impl WindowComputer {
    pub fn new(window_len: u64) -> Self {
        Self {
            window_len: window_len.max(1),
            prev: None,
            seen_bins: AHashSet::new(),
            window_ix: 0,
        }
    }

    /// Compute features for the current snapshot and roll state forward,
    /// computing the WL class map internally (fixtures and standalone use).
    pub fn observe(&mut self, snapshot: &GraphSnapshot<'_>) -> WindowFeatures {
        let classes = compute_wl_class_map_snapshot(snapshot, WL_RADIUS);
        self.observe_with_classes(snapshot, &classes)
    }

    /// Compute features using a precomputed per-node WL class map — the
    /// shared once-per-window computation (see hg_core::compute_wl_class_map).
    pub fn observe_with_classes(
        &mut self,
        snapshot: &GraphSnapshot<'_>,
        classes: &AHashMap<NodeId, WlHash>,
    ) -> WindowFeatures {
        let n_nodes = snapshot.node_count();
        let n_edges = snapshot.edge_count();

        // Deterministic node order: canonical iteration via sorted ids is
        // unnecessary — slotmap iteration is deterministic for a given
        // construction history, and all downstream aggregates are
        // order-insensitive (histograms, sets).
        let all_ids: Vec<NodeId> = snapshot.node_ids().collect();

        // PI sampling: full below the cap, deterministic stride above.
        let (sampled, strategy): (Vec<NodeId>, &'static str) = if all_ids.len() <= PI_SAMPLE_CAP {
            (all_ids.clone(), "full")
        } else {
            let stride = all_ids.len().div_ceil(PI_SAMPLE_CAP);
            (
                all_ids.iter().copied().step_by(stride).collect(),
                "stride",
            )
        };

        // Type histogram + member sets.
        let mut hist: AHashMap<WlHash, usize> = AHashMap::new();
        let mut members: AHashMap<WlHash, AHashSet<NodeId>> = AHashMap::new();
        for &id in &sampled {
            let h = classes.get(&id).copied().unwrap_or(WlHash(0));
            *hist.entry(h).or_insert(0) += 1;
            members.entry(h).or_default().insert(id);
        }
        let total: usize = hist.values().sum();

        let entropy = shannon_entropy(&hist, total);
        let dominant = hist
            .values()
            .map(|&c| c as f64 / total.max(1) as f64)
            .fold(0.0, f64::max);

        // Degree.
        let mean_degree = if n_nodes > 0 {
            let deg_sum: usize = all_ids
                .iter()
                .map(|&id| snapshot.edges_incident_to(id).len())
                .sum();
            Some(deg_sum as f64 / n_nodes as f64)
        } else {
            None
        };

        // Novelty: bins never seen in any earlier window of this run.
        let mut new_signatures = Vec::new();
        for h in hist.keys() {
            if !self.seen_bins.contains(h) {
                new_signatures.push(h.0);
            }
        }
        let new_type_bins = new_signatures.len();

        let live_ids: AHashSet<NodeId> = all_ids.iter().copied().collect();

        let (activity, jsd, entropy_rate, pi, growth_ratio, active_ps, static_ps) =
            match &self.prev {
                None => (0.0, 0.0, 0.0, None, 1.0, 0, 0),
                Some(prev) => {
                    // Node-id churn.
                    let inter = live_ids.intersection(&prev.live_ids).count();
                    let union = live_ids.union(&prev.live_ids).count();
                    let churn = if union == 0 {
                        0.0
                    } else {
                        (union - inter) as f64 / union as f64
                    };

                    let jsd = jensen_shannon(&prev.hist, prev.sample_total, &hist, total);
                    let er = (entropy - prev.entropy).abs();

                    // PI proxy: persisting type-mass overlap (sorted
                    // iteration: see sorted_entries).
                    let overlap: f64 = sorted_entries(&hist)
                        .iter()
                        .map(|&(h, c)| {
                            let p_now = c as f64 / total.max(1) as f64;
                            let p_prev = prev
                                .hist
                                .get(&h)
                                .map(|&pc| pc as f64 / prev.sample_total.max(1) as f64)
                                .unwrap_or(0.0);
                            p_now.min(p_prev)
                        })
                        .sum();

                    let growth = n_nodes as f64 / prev.n_nodes_full.max(1) as f64;

                    // Persistence: bins present in both windows.
                    let mut active = 0;
                    let mut statics = 0;
                    for (h, m_now) in &members {
                        if let Some(m_prev) = prev.members.get(h) {
                            if m_now == m_prev {
                                statics += 1;
                            } else {
                                active += 1;
                            }
                        }
                    }
                    (churn, jsd, er, Some(overlap), growth, active, statics)
                }
            };

        let features = WindowFeatures {
            window: self.window_ix,
            n_nodes,
            n_edges,
            mean_degree,
            activity,
            growth_ratio,
            type_histogram_entropy: entropy,
            dominant_type_fraction: dominant,
            type_histogram_jsd: jsd,
            entropy_rate,
            new_type_bins,
            new_type_bin_rate: new_type_bins as f64 / self.window_len as f64,
            pi_normalized: pi,
            pi_sample_size: sampled.len(),
            pi_sample_strategy: strategy,
            active_persistent_structures: active_ps,
            static_persistent_motifs: static_ps,
            static_repeated_motifs: hist.values().filter(|&&c| c > 1).count(),
            n_type_bins: hist.len(),
            new_signatures,
            has_prev: self.prev.is_some(),
        };

        // Roll state.
        for h in hist.keys() {
            self.seen_bins.insert(*h);
        }
        self.prev = Some(PrevWindow {
            hist,
            members,
            live_ids,
            entropy,
            sample_total: total,
            n_nodes_full: n_nodes,
        });
        self.window_ix += 1;

        features
    }
}

/// Float accumulations iterate in sorted-WlHash order: hash-map iteration
/// order varies across processes, and float addition is not associative —
/// unsorted sums differ in the last ulp between runs, breaking the
/// cross-thread-count CSV byte comparison (docs/reproducibility_policy.md).
fn sorted_entries(hist: &AHashMap<WlHash, usize>) -> Vec<(WlHash, usize)> {
    let mut v: Vec<(WlHash, usize)> = hist.iter().map(|(h, c)| (*h, *c)).collect();
    v.sort_unstable_by_key(|(h, _)| *h);
    v
}

fn shannon_entropy(hist: &AHashMap<WlHash, usize>, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    sorted_entries(hist)
        .iter()
        .map(|&(_, c)| {
            let p = c as f64 / total as f64;
            if p > 0.0 {
                -p * p.log2()
            } else {
                0.0
            }
        })
        .sum()
}

/// JSD (log2, in [0,1]) between two count histograms.
fn jensen_shannon(
    a: &AHashMap<WlHash, usize>,
    a_total: usize,
    b: &AHashMap<WlHash, usize>,
    b_total: usize,
) -> f64 {
    if a_total == 0 || b_total == 0 {
        return 0.0;
    }
    let keys: AHashSet<&WlHash> = a.keys().chain(b.keys()).collect();
    let mut keys: Vec<&WlHash> = keys.into_iter().collect();
    keys.sort_unstable();
    let mut jsd = 0.0;
    for k in keys {
        let p = a.get(k).map(|&c| c as f64 / a_total as f64).unwrap_or(0.0);
        let q = b.get(k).map(|&c| c as f64 / b_total as f64).unwrap_or(0.0);
        let m = 0.5 * (p + q);
        if p > 0.0 {
            jsd += 0.5 * p * (p / m).log2();
        }
        if q > 0.0 {
            jsd += 0.5 * q * (q / m).log2();
        }
    }
    jsd.clamp(0.0, 1.0)
}
