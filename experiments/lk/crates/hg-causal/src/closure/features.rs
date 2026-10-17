//! Frozen macro/micro feature vectors for the closure-defect estimator
//! (the LK4 spec's "exact field list", plan §1.4). Changing either layout
//! is a spec revision: update the field lists, the extraction, and
//! docs/leakage_closure_spec.md together.
//!
//! macro(w): emergent-layer summaries — what the geometric description
//! keeps. micro(w): substrate detail the macro omits.

use ahash::AHashMap;
use hg_core::{GraphSnapshot, NodeId, WlHash};
use hg_schema::Regime;

/// Frozen macro layout (RE-FROZEN by plan v2 §1.4: gains the phase-plane
/// coordinates wormhole_components / area_law_defect / dim_mismatch —
/// changing it again requires a plan revision): 8 scalars + 11-way
/// regime one-hot.
pub const MACRO_FIELDS: [&str; 9] = [
    "d_eff",
    "cut_exponent",
    "wormhole_frac",
    "wormhole_components",
    "area_law_defect",
    "dim_mismatch",
    "activity",
    "wl_class_count",
    "regime_onehot[11]",
];

/// Frozen micro layout.
pub const MICRO_FIELDS: [&str; 4] = [
    "detour_histogram[6]",
    "top16_wl_class_freq[16]",
    "degree_moments[mean,var,skew]",
    "component_size_top3_frac",
];

const REGIMES: [Regime; 11] = [
    Regime::Frozen,
    Regime::StaticOrdered,
    Regime::GrowthTreadmill,
    Regime::WhiteNoiseSoup,
    Regime::SterileChaos,
    Regime::Exploding,
    Regime::FragmentedIsolation,
    Regime::Monoculture,
    Regime::ActivePersistentStructure,
    Regime::GenerativeCandidate,
    Regime::Unknown,
];

/// macro(w). Missing measurements impute 0.0 (documented; the estimator
/// standardizes per fold, so a constant imputed column is inert).
#[allow(clippy::too_many_arguments)]
pub fn macro_features(
    d_eff: Option<f64>,
    cut_exponent: Option<f64>,
    wormhole_frac: f64,
    wormhole_components: Option<u32>,
    area_law_defect: Option<f64>,
    dim_mismatch: Option<f64>,
    activity: Option<f64>,
    wl_class_count: Option<usize>,
    regime: Regime,
) -> Vec<f64> {
    let mut v = vec![
        d_eff.unwrap_or(0.0),
        cut_exponent.unwrap_or(0.0),
        wormhole_frac,
        wormhole_components.map(|c| c as f64).unwrap_or(0.0),
        area_law_defect.unwrap_or(0.0),
        dim_mismatch.unwrap_or(0.0),
        activity.unwrap_or(0.0),
        wl_class_count.map(|c| c as f64).unwrap_or(0.0),
    ];
    for r in REGIMES {
        v.push(if r == regime { 1.0 } else { 0.0 });
    }
    v
}

/// micro(w): detour histogram (6 log-spaced fractions, from the
/// LocalityObserver), top-16 WL-class frequency vector (fractions,
/// descending, zero-padded), degree moments, component-size top-3 (as
/// fractions of n).
pub fn micro_features(
    snapshot: &GraphSnapshot<'_>,
    classes: &AHashMap<NodeId, WlHash>,
    detour_histogram: &[f64; 6],
) -> Vec<f64> {
    let n = snapshot.node_count().max(1) as f64;
    let mut v: Vec<f64> = detour_histogram.to_vec();

    // Top-16 WL class frequencies.
    let mut counts: AHashMap<WlHash, usize> = AHashMap::new();
    for (id, _) in snapshot.nodes() {
        let h = classes.get(&id).copied().unwrap_or(WlHash(0));
        *counts.entry(h).or_insert(0) += 1;
    }
    let mut freqs: Vec<f64> = counts.values().map(|&c| c as f64 / n).collect();
    freqs.sort_by(|a, b| b.total_cmp(a));
    freqs.resize(16, 0.0);
    v.extend_from_slice(&freqs[..16]);

    // Degree moments (degree = distinct neighbors under clique adjacency).
    let mut degree: AHashMap<NodeId, ahash::AHashSet<NodeId>> = AHashMap::new();
    for (_, edge) in snapshot.edges() {
        for &a in &edge.nodes {
            for &b in &edge.nodes {
                if a != b {
                    degree.entry(a).or_default().insert(b);
                }
            }
        }
    }
    let degs: Vec<f64> = snapshot
        .node_ids()
        .map(|id| degree.get(&id).map(|s| s.len()).unwrap_or(0) as f64)
        .collect();
    let dn = degs.len().max(1) as f64;
    let mean = degs.iter().sum::<f64>() / dn;
    let var = degs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / dn;
    let skew = if var > 1e-12 {
        degs.iter().map(|d| (d - mean).powi(3)).sum::<f64>() / dn / var.powf(1.5)
    } else {
        0.0
    };
    v.push(mean);
    v.push(var);
    v.push(skew);

    // Component sizes, top 3, as fractions of n (union-find over edges).
    let ids: Vec<NodeId> = snapshot.node_ids().collect();
    let index: AHashMap<NodeId, usize> = ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    let mut parent: Vec<usize> = (0..ids.len()).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for (_, edge) in snapshot.edges() {
        let mut it = edge.nodes.iter().filter_map(|n| index.get(n).copied());
        if let Some(first) = it.next() {
            let ra = find(&mut parent, first);
            for b in it {
                let rb = find(&mut parent, b);
                parent[rb] = ra;
            }
        }
    }
    let mut sizes: AHashMap<usize, usize> = AHashMap::new();
    for i in 0..ids.len() {
        let r = find(&mut parent, i);
        *sizes.entry(r).or_insert(0) += 1;
    }
    let mut sizes: Vec<f64> = sizes.values().map(|&s| s as f64 / n).collect();
    sizes.sort_by(|a, b| b.total_cmp(a));
    sizes.resize(3, 0.0);
    v.extend_from_slice(&sizes[..3]);

    v
}
