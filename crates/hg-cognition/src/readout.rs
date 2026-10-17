//! Probe readouts — `readout_set` r1 and r2 (`docs/policy_config.md` §4B).
//!
//! **Not** the policy feature set. v1 conflated the two under one
//! `feature_set = "v1"` key and they are different objects with opposite
//! requirements:
//!
//! | | policy feature (`features.rs`) | probe readout (here) |
//! |---|---|---|
//! | computed per | candidate binding | window |
//! | requirement | **cheap** | **complete** |
//! | consumer | `MlpScorer` scoring | Stage 0b separation, Stage 1 memory |
//!
//! A policy feature that is expensive makes every campaign unaffordable; a
//! probe feature that is *incomplete* makes every separation and memory
//! claim a joint statement about the substrate and the projection. Which is
//! what happened: `glider`'s r1 readout is exactly constant across its
//! whole run, so Stage 0b classified it `Undetermined` and Stage 0a found
//! no budget cell above 2σ — neither of which is a statement about
//! `glider`'s dynamics. **`glider` is invisible, not inert.**
//!
//! No policy ever sees a probe row, so `readout_set` is not part of the
//! genome contract: a champion is meaningless under a different
//! `feature_set` and completely indifferent to this.

use hg_core::GraphSnapshot;

/// Degree normalization, shared with `feature_set = "v1"` feature 4 so a
/// reader comparing the two is comparing the same scale.
pub const DEG_SCALE: f64 = 64.0;
/// Node/edge count normalization for r2 block 6.
pub const SIZE_SCALE: f64 = 65536.0;

/// Which readout a probe uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadoutSet {
    /// The 16 WL-bucket histogram used throughout C0–C5. Retained
    /// **unchanged** so every C0–C5 number stays reproducible and any
    /// r1/r2 disagreement is attributable to the readout rather than to a
    /// silent revision of the baseline.
    #[default]
    R1,
    /// r1 plus five blocks that are not invariant under a conservative
    /// relabel (§4B.3).
    R2,
}

impl ReadoutSet {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::R1 => "r1",
            Self::R2 => "r2",
        }
    }
}

/// `[cognition.probe]`.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ProbeConfig {
    pub readout_set: ReadoutSet,
    /// r2 block 2's depth. Ignored under r1.
    pub top_n_wl: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            readout_set: ReadoutSet::R1,
            top_n_wl: 16,
        }
    }
}

/// r1 — the WL-bucket histogram.
///
/// Byte-identical to the C0–C5 `wl_bucket_probe`, including the `% buckets`
/// fold and the `1.0 / n` accumulation order. Changing anything here
/// invalidates every published Stage 0a and Stage 0b number, so it is
/// pinned by `r1_reproduces_the_c0_c5_readout_exactly`.
pub fn r1(snap: &GraphSnapshot<'_>, buckets: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; buckets];
    let classes = hg_core::compute_wl_class_map_snapshot(snap, 1);
    let n = classes.len().max(1) as f32;
    for h in classes.values() {
        v[(h.0 % buckets as u64) as usize] += 1.0 / n;
    }
    v
}

/// r2 — r1 plus five blocks chosen to break the invariance r1 has.
///
/// Layout, in order: r1 (`buckets`), raw WL multiset (`top_n_wl`), degree
/// moments (4), edge count by label (`n_labels`), components (2), raw size
/// (2). Total `buckets + top_n_wl + n_labels + 8`.
///
/// `n_labels` is passed in rather than discovered per window because the
/// readout's dimension must be constant across a run — a probe whose width
/// changes when a new label first appears produces a ragged matrix, and
/// `kernel_held`'s SVD and the memory-capacity ridge both reject it.
pub fn r2(
    snap: &GraphSnapshot<'_>,
    buckets: usize,
    top_n_wl: usize,
    n_labels: usize,
) -> Vec<f32> {
    let mut v = r1(snap, buckets);

    // --- block 2: the raw WL multiset, ranked.
    //
    // This is the block that unblinds `glider`. r1 folds hashes into 16
    // buckets and normalizes, which is invariant under a relabel that
    // permutes classes within the multiset. The raw hashes, ranked by
    // frequency, are not.
    let classes = hg_core::compute_wl_class_map_snapshot(snap, 1);
    let total = classes.len().max(1) as f32;
    let mut counts: std::collections::BTreeMap<u64, u32> = Default::default();
    for h in classes.values() {
        *counts.entry(h.0).or_default() += 1;
    }
    let mut ranked: Vec<(u32, u64)> = counts.into_iter().map(|(h, c)| (c, h)).collect();
    // (count desc, hash asc). The tiebreak is specified because ties are
    // common on small graphs, and an unspecified order would make the
    // readout process-dependent — the same order hazard the V-track's C3
    // found in `HashMap::values()` summation.
    ranked.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for i in 0..top_n_wl {
        v.push(ranked.get(i).map(|(c, _)| *c as f32 / total).unwrap_or(0.0));
    }

    // --- block 3: degree moments over live nodes.
    let degrees: Vec<f64> = snap
        .node_ids()
        .map(|id| snap.edges_incident_to(id).len() as f64)
        .collect();
    let (mean, var, skew, max) = moments(&degrees);
    let scale = |x: f64| (((1.0 + x).ln() / (1.0 + DEG_SCALE).ln()).clamp(0.0, 1.0)) as f32;
    v.push(scale(mean));
    v.push(scale(var));
    // Skew is signed, so it takes a squashing rather than a log scale.
    v.push(skew.tanh() as f32);
    v.push(scale(max));

    // --- block 4: edge count by label.
    let e_total = snap.edge_count().max(1) as f32;
    let mut by_label = vec![0.0f32; n_labels];
    for (_, e) in snap.edges() {
        if (e.label.0 as usize) < n_labels {
            by_label[e.label.0 as usize] += 1.0 / e_total;
        }
    }
    v.extend(by_label);

    // --- block 5: components.
    //
    // Computed here in `hg-cognition`, from a snapshot, for a probe row
    // that no evaluator sees and no `metrics.csv` carries.
    // `docs/fitness_formulas.md` §5 declines to add a component census as a
    // held-out *evaluator* metric because adding an evaluator metric to
    // serve the learning layer is the contamination the firewall forbids.
    // This is on the other side of that firewall: `hg-eval` is untouched.
    let (n_components, largest) = components(snap);
    let nodes = snap.node_count().max(1) as f64;
    v.push((((1.0 + n_components as f64).ln() / (1.0 + nodes).ln()).clamp(0.0, 1.0)) as f32);
    v.push((largest as f64 / nodes) as f32);

    // --- block 6: raw size, not a ratio.
    let sz = |x: usize| {
        (((1.0 + x as f64).ln() / (1.0 + SIZE_SCALE).ln()).clamp(0.0, 1.0)) as f32
    };
    v.push(sz(snap.node_count()));
    v.push(sz(snap.edge_count()));

    v
}

/// The dimension `r2` produces, so a caller can size a matrix before the
/// first window rather than discovering it from row 0.
pub fn r2_dim(buckets: usize, top_n_wl: usize, n_labels: usize) -> usize {
    buckets + top_n_wl + n_labels + 8
}

/// `(mean, variance, skew, max)`. Population moments; skew is `0.0` when
/// the variance is, rather than `NaN`.
fn moments(xs: &[f64]) -> (f64, f64, f64, f64) {
    if xs.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let sd = var.sqrt();
    let skew = if sd > 0.0 {
        xs.iter().map(|x| ((x - mean) / sd).powi(3)).sum::<f64>() / n
    } else {
        0.0
    };
    let max = xs.iter().cloned().fold(0.0f64, f64::max);
    (mean, var, skew, max)
}

/// `(component count, largest component size)` over the node set, treating
/// each hyperedge as connecting all of its nodes.
///
/// Union–find rather than a traversal: O(E α(V)) against a window of at
/// least several steps of rewriting, and it needs no adjacency index the
/// snapshot does not already have.
fn components(snap: &GraphSnapshot<'_>) -> (usize, usize) {
    let ids: Vec<_> = snap.node_ids().collect();
    if ids.is_empty() {
        return (0, 0);
    }
    let index: std::collections::HashMap<_, usize> =
        ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let mut parent: Vec<usize> = (0..ids.len()).collect();

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }

    for (_, e) in snap.edges() {
        let mut it = e.nodes.iter().filter_map(|n| index.get(n).copied());
        if let Some(first) = it.next() {
            for other in it {
                let (a, b) = (find(&mut parent, first), find(&mut parent, other));
                if a != b {
                    parent[a] = b;
                }
            }
        }
    }

    let mut sizes: std::collections::HashMap<usize, usize> = Default::default();
    for i in 0..ids.len() {
        *sizes.entry(find(&mut parent, i)).or_default() += 1;
    }
    let largest = sizes.values().cloned().max().unwrap_or(0);
    (sizes.len(), largest)
}

/// A probe closure for `RolloutRequest::probe`.
pub fn probe(
    set: ReadoutSet,
    buckets: usize,
    top_n_wl: usize,
    n_labels: usize,
) -> impl FnMut(&GraphSnapshot<'_>, u64) -> Vec<f32> + Send {
    move |snap, _window| match set {
        ReadoutSet::R1 => r1(snap, buckets),
        ReadoutSet::R2 => r2(snap, buckets, top_n_wl, n_labels),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_dimension_is_stable_and_documented() {
        // 16 + 16 + 3 + 8 = 43, the worked example in
        // `docs/policy_config.md` §4B.3.
        assert_eq!(r2_dim(16, 16, 3), 43);
    }

    #[test]
    fn moments_of_a_constant_are_not_nan() {
        // sd = 0 makes the standard skew formula 0/0. A NaN in one probe
        // column poisons the whole SVD in `kernel_held` and every ridge
        // solve in Stage 1, and it would look like a substrate property.
        let (mean, var, skew, max) = moments(&[3.0; 8]);
        assert_eq!((mean, var, skew, max), (3.0, 0.0, 0.0, 3.0));
    }

    #[test]
    fn moments_recover_a_known_distribution() {
        let (mean, var, _, max) = moments(&[1.0, 2.0, 3.0, 4.0]);
        assert!((mean - 2.5).abs() < 1e-12);
        assert!((var - 1.25).abs() < 1e-12);
        assert_eq!(max, 4.0);
    }

    #[test]
    fn skew_is_signed_and_squashed_into_range() {
        // Right-tailed: positive. Left-tailed: negative. Both bounded, so
        // one outlier cannot dominate the readout's scale.
        let (_, _, right, _) = moments(&[1.0, 1.0, 1.0, 1.0, 9.0]);
        let (_, _, left, _) = moments(&[9.0, 9.0, 9.0, 9.0, 1.0]);
        assert!(right > 0.0 && left < 0.0);
        assert!(right.tanh().abs() <= 1.0 && left.tanh().abs() <= 1.0);
    }

    #[test]
    fn the_ranked_multiset_tiebreak_is_deterministic() {
        // Equal counts must order by hash ascending, not by whatever a hash
        // map yields. The V-track's C3 found exactly this class of order
        // hazard in `HashMap::values()` summation, where fields existed on
        // which two processes agreed bit-for-bit and only a third diverged.
        let mut ranked: Vec<(u32, u64)> = vec![(2, 90), (2, 10), (5, 50), (2, 50)];
        ranked.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        assert_eq!(ranked, vec![(5, 50), (2, 10), (2, 50), (2, 90)]);
    }
}
