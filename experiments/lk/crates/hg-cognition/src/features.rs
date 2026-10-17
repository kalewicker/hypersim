//! Per-candidate policy features. `feature_set = "v1"` (frozen) and
//! `"v2"` (`docs/policy_config.md` §4, `docs/cognition_e0_quantizer_audit.md`).
//!
//! Per-candidate features from `GraphSnapshot` only: no causal-layer calls,
//! because these run on every candidate of every step of every rollout of
//! every generation.
//!
//! | # | Feature | Dim |
//! |---|---|---|
//! | 1 | rule one-hot | `n_rules` |
//! | 2 | priority (tanh-squashed) | 1 |
//! | 3 | arity (normalized) | 1 |
//! | 4 | bound-node degree mean/max/min (log-scaled) | 3 |
//! | 5 | WL bucket histogram over bound nodes | `wl_buckets` |
//! | 6 | log candidate count | 1 |
//! | 7 | fraction of candidates sharing this rule | 1 |
//!
//! Total `n_rules + wl_buckets + 7`.
//!
//! The normalization constants below are **part of the frozen feature
//! set**, not tuning knobs: a champion genome trained under one
//! normalization is meaningless under another. Changing one is
//! `feature_set = "v2"`.
//!
//! # v2 — the two saturating scales become scale-free
//!
//! E0's quantizer audit measured all four of v1's fixed scales against the
//! substrates they run on. Two **saturate**, and saturation destroys
//! distinctions the substrate actually makes:
//!
//! | observable | v1 scale | measured | verdict |
//! |---|---|---|---|
//! | candidate count | `log / 4096` | median **9198**, max 16974 on `tri_two` | pinned at 1.0 on **78.9%** of steps |
//! | node degree | `log / 64` | max **69** / **130** | clips 0.3% / 6.0% of values |
//! | arity | `/ 8` | 2 – 3 | compressed 8×, **no information lost** |
//! | priority | `tanh(/8)` | `{0}` on every owned substrate | constant — a substrate fact, not a scale fault |
//!
//! v2 changes **only the two that saturate**, and replaces their fixed
//! denominators with ratios against a quantity that moves with the
//! substrate:
//!
//! * degree → `rel = deg / mean_degree`, then `rel / (1 + rel)`
//! * candidate count → `rel = n_candidates / node_count`, then `rel / (1 + rel)`
//!
//! `x / (1 + x)` is monotone on `[0, ∞)` into `[0, 1)` and **cannot clip at
//! either end**, so no substrate can push either feature into a constant.
//! `mean_degree` is `edge_count / node_count` — O(1) from the snapshot, no
//! extra pass over the candidate pool.
//!
//! **What v2 deliberately does not do.** It does not widen `arity` or
//! `priority`, and it is not tuned to increase the fraction of `[0, 1]` the
//! features occupy. Under-using a range loses no information; saturating one
//! destroys it. Choosing constants to maximize observed spread would be
//! fitting the instrument to the observations, which
//! `instrument_registry.md` §1.1 rejects — the audit's `used` column is a
//! diagnostic, never a target.
//!
//! `feat_dim` is **unchanged** at `n_rules + wl_buckets + 7`, so genome
//! length and every campaign mechanism carry over; only the values differ.

use hg_core::{
    compute_wl_class_map_snapshot, compute_wl_hash_snapshot, GraphSnapshot, NodeId, WlHash,
};
use hg_observe::Binding;
use rustc_hash::{FxHashMap, FxHashSet};

pub const FEATURE_SET_V1: &str = "v1";
/// Scale-free degree and candidate-count encodings; see the module header.
pub const FEATURE_SET_V2: &str = "v2";

/// Which encoding a `FeatureExtractor` applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FeatureSet {
    #[default]
    V1,
    V2,
}

impl FeatureSet {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            FEATURE_SET_V1 => Some(Self::V1),
            FEATURE_SET_V2 => Some(Self::V2),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => FEATURE_SET_V1,
            Self::V2 => FEATURE_SET_V2,
        }
    }
}

/// v2's saturation-free squash: monotone `[0, ∞) -> [0, 1)`, no clipping at
/// either end, so no substrate can drive the feature to a constant.
#[inline]
fn ratio_squash(x: f32) -> f32 {
    if x <= 0.0 {
        0.0
    } else {
        x / (1.0 + x)
    }
}

/// Degree log-scale denominator: `ln(1 + deg) / ln(1 + DEG_SCALE)`.
pub const DEG_SCALE: f32 = 64.0;
/// Candidate-count log-scale denominator.
pub const CAND_SCALE: f32 = 4096.0;
/// Arity normalizer (the `SmallVec<[NodeId; 8]>` inline capacity).
pub const ARITY_SCALE: f32 = 8.0;
/// Priority squash: `tanh(priority / PRIORITY_SCALE)`.
pub const PRIORITY_SCALE: f32 = 8.0;
/// WL radius for feature 5.
pub const WL_RADIUS: u8 = 1;
/// Max pattern-match candidates `MlpScorer` scores per step; above this it
/// subsamples with the policy RNG (`docs/policy_config.md` §4.4).
pub const CAND_SCORE_CAP: usize = 512;

/// Crossover for the WL strategy in [`FeatureExtractor::prime_wl`]: use the
/// whole-graph refinement pass when
/// `distinct_bound_nodes · WL_CLASSMAP_RATIO > n_nodes`.
///
/// `compute_wl_class_map_snapshot` is documented as bit-identical to
/// per-node `compute_wl_hash`, so this switch changes cost and nothing
/// else. The ratio is calibrated from `benches/scoring.rs`
/// (`wl_strategy`): on a 4096-node substrate a per-node radius-1 ball walk
/// costs ~0.48 µs and the whole-graph pass ~0.066 µs per graph node, so the
/// pass wins above roughly `n_nodes / 7.3` distinct nodes. Rounded to 8 —
/// the exact crossover is machine- and topology-dependent, and being
/// slightly late to switch costs far less than being early on a large,
/// sparsely-touched graph.
pub const WL_CLASSMAP_RATIO: usize = 8;

/// `n_rules + wl_buckets + 7`.
pub fn feat_dim(n_rules: usize, wl_buckets: usize) -> usize {
    n_rules + wl_buckets + 7
}

/// Per-step context shared by every candidate in one `select` call:
/// the candidate count and the per-rule candidate counts. Computed once by
/// [`FeatureExtractor::begin_step`], not per candidate.
#[derive(Clone, Debug, Default)]
struct StepContext {
    step: u64,
    n_candidates: usize,
    per_rule: Vec<usize>,
}

pub struct FeatureExtractor {
    n_rules: usize,
    wl_buckets: usize,
    set: FeatureSet,
    dim: usize,
    ctx: StepContext,
    /// WL hashes for this step only, cleared when the step advances.
    /// Measured worth: 22× on a 4096-node substrate (36 µs for 512
    /// bindings over shared nodes vs 802 µs over distinct ones,
    /// `benches/scoring.rs::wl_hash_cache`). Filled by [`Self::prime_wl`],
    /// or lazily on demand if that was not called.
    wl_cache: FxHashMap<NodeId, WlHash>,
    cache_step: Option<u64>,
    /// Scratch buffer reused across candidates so `features()` does not
    /// allocate per candidate.
    scratch: Vec<f32>,
    /// Reused across steps so `prime_wl` does not allocate per step.
    distinct_scratch: FxHashSet<NodeId>,
    /// True when the current step's cache came from a whole-graph pass, so
    /// a cache miss means "this node is not in the graph" rather than "not
    /// computed yet".
    primed_whole_graph: bool,
}

impl FeatureExtractor {
    /// v1 encoding. Retained so every existing call site keeps its meaning
    /// rather than silently changing feature semantics.
    pub fn new(n_rules: usize, wl_buckets: usize) -> Self {
        Self::with_set(n_rules, wl_buckets, FeatureSet::V1)
    }

    pub fn with_set(n_rules: usize, wl_buckets: usize, set: FeatureSet) -> Self {
        let dim = feat_dim(n_rules, wl_buckets.max(1));
        Self {
            n_rules,
            wl_buckets: wl_buckets.max(1),
            set,
            dim,
            ctx: StepContext::default(),
            wl_cache: FxHashMap::default(),
            cache_step: None,
            scratch: vec![0.0; dim],
            distinct_scratch: FxHashSet::default(),
            primed_whole_graph: false,
        }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn n_rules(&self) -> usize {
        self.n_rules
    }

    /// Compute the per-step context and invalidate the WL cache if the step
    /// advanced. Must be called once per `select`, before any `features`
    /// call, with the full candidate slice — features 6 and 7 are
    /// properties of the whole pool, not of one binding.
    pub fn begin_step(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) {
        let step = snapshot.step();
        if self.cache_step != Some(step) {
            self.wl_cache.clear();
            self.primed_whole_graph = false;
            self.cache_step = Some(step);
        }
        let mut per_rule = vec![0usize; self.n_rules];
        for b in candidates {
            if let Some(slot) = per_rule.get_mut(b.rule_id) {
                *slot += 1;
            }
        }
        self.ctx = StepContext {
            step,
            n_candidates: candidates.len(),
            per_rule,
        };
    }

    /// Fill the WL cache for the bindings that will actually be scored,
    /// choosing between per-node ball walks and one whole-graph refinement
    /// pass by [`WL_CLASSMAP_RATIO`].
    ///
    /// Separate from [`Self::begin_step`] because the two take different
    /// candidate sets: pool-level features (candidate count, rule share)
    /// describe the *whole* step, while the WL work is only needed for the
    /// subset a policy will score after its 512-candidate cap. Priming from
    /// the full pool would trigger the whole-graph pass on steps where the
    /// scorer touches a handful of nodes.
    ///
    /// Optional: `features()` falls back to computing a missing hash on
    /// demand, so a policy that skips this call is slower but never wrong.
    pub fn prime_wl<'b>(
        &mut self,
        snapshot: &GraphSnapshot<'_>,
        bindings: impl Iterator<Item = &'b Binding>,
    ) {
        self.distinct_scratch.clear();
        for b in bindings {
            for &n in b.bound_nodes.iter() {
                if !self.wl_cache.contains_key(&n) {
                    self.distinct_scratch.insert(n);
                }
            }
        }
        if self.distinct_scratch.is_empty() {
            return;
        }

        let n_nodes = snapshot.node_count();
        if self.distinct_scratch.len().saturating_mul(WL_CLASSMAP_RATIO) > n_nodes {
            // One O(radius · E) pass for the whole graph. Bit-identical to
            // the per-node walk, per `compute_wl_class_map_snapshot`'s
            // contract (it is property-tested against it in hg-core).
            let map = compute_wl_class_map_snapshot(snapshot, WL_RADIUS);
            self.wl_cache.extend(map);
            self.primed_whole_graph = true;
        } else {
            // `take` so the borrow of `self.distinct_scratch` ends before
            // `self.wl_cache` is mutated; the allocation is handed straight
            // back afterwards.
            let wanted = std::mem::take(&mut self.distinct_scratch);
            for &n in wanted.iter() {
                let h = compute_wl_hash_snapshot(snapshot, n, WL_RADIUS);
                self.wl_cache.insert(n, h);
            }
            self.distinct_scratch = wanted;
        }
    }

    /// True when this step's cache was filled by a whole-graph pass
    /// (diagnostic; the bench and Stage 0a reporting use it).
    pub fn primed_whole_graph(&self) -> bool {
        self.primed_whole_graph
    }

    fn wl_bucket(&mut self, snapshot: &GraphSnapshot<'_>, node: NodeId) -> usize {
        let hash = match self.wl_cache.get(&node) {
            Some(h) => *h,
            None => {
                let h = compute_wl_hash_snapshot(snapshot, node, WL_RADIUS);
                self.wl_cache.insert(node, h);
                h
            }
        };
        (hash.0 % self.wl_buckets as u64) as usize
    }

    /// Feature vector for one candidate. The returned slice is a scratch
    /// buffer owned by the extractor: valid until the next `features` call.
    pub fn features(&mut self, snapshot: &GraphSnapshot<'_>, binding: &Binding) -> &[f32] {
        debug_assert_eq!(
            self.cache_step,
            Some(snapshot.step()),
            "begin_step must run before features on each step"
        );
        for v in self.scratch.iter_mut() {
            *v = 0.0;
        }

        // 1. rule one-hot.
        if binding.rule_id < self.n_rules {
            self.scratch[binding.rule_id] = 1.0;
        }
        let mut i = self.n_rules;

        // 2. priority, squashed: priorities are unbounded i32 and a raw
        // value would swamp every other feature at the first hidden layer.
        self.scratch[i] = (binding.priority as f32 / PRIORITY_SCALE).tanh();
        i += 1;

        // 3. arity.
        let arity = binding.bound_nodes.len();
        self.scratch[i] = (arity as f32 / ARITY_SCALE).clamp(0.0, 1.0);
        i += 1;

        // 4. degree mean / max / min. All zero for a node-free binding —
        // there is no degree to report and zero is the honest encoding of
        // "this feature does not apply", consistent with the one-hot
        // convention above.
        //
        // v2: `deg / mean_degree`, squashed. `mean_degree` is O(1) from the
        // snapshot, so this costs no extra pass over the pool — and unlike
        // v1's `log / DEG_SCALE` it cannot clip, which v1 did on 6.0% of
        // `wn_three` values (observed max degree 130 against a scale of 64).
        let mean_deg = if self.set == FeatureSet::V2 {
            let n = snapshot.node_count().max(1) as f32;
            (snapshot.edge_count() as f32 / n).max(f32::MIN_POSITIVE)
        } else {
            0.0
        };
        let (mut sum, mut max, mut min) = (0.0f32, 0.0f32, f32::MAX);
        for &n in binding.bound_nodes.iter() {
            let deg = snapshot.edges_incident_to(n).len() as f32;
            let scaled = match self.set {
                FeatureSet::V1 => ((1.0 + deg).ln() / (1.0 + DEG_SCALE).ln()).clamp(0.0, 1.0),
                FeatureSet::V2 => ratio_squash(deg / mean_deg),
            };
            sum += scaled;
            max = max.max(scaled);
            min = min.min(scaled);
        }
        if arity > 0 {
            self.scratch[i] = sum / arity as f32;
            self.scratch[i + 1] = max;
            self.scratch[i + 2] = min;
        }
        i += 3;

        // 5. WL bucket histogram over bound nodes, normalized by arity so
        // the block sums to 1 for any binding with nodes and to 0 for one
        // without — the same "does not apply" encoding as feature 4.
        if arity > 0 {
            let inv = 1.0 / arity as f32;
            // Collected first: `wl_bucket` needs &mut self and the loop
            // would otherwise hold a borrow of `binding` through it.
            let nodes: Vec<NodeId> = binding.bound_nodes.iter().copied().collect();
            for n in nodes {
                let b = self.wl_bucket(snapshot, n);
                self.scratch[i + b] += inv;
            }
        }
        i += self.wl_buckets;

        // 6. candidate count.
        //
        // v1 divides `ln(1 + n)` by `ln(1 + 4096)`, which pins the feature at
        // 1.0 on 78.9% of `tri_two` steps (median pool 9198). v2 measures the
        // pool against the graph it came from — candidate counts scale with
        // graph size, so the ratio is the substrate-independent quantity and
        // it cannot saturate.
        //
        // This is the feature carrying *scarcity*, and C-1 established that
        // scarcity is the precondition for selection to have causal power at
        // all, so pinning it is the costliest of the two saturations.
        self.scratch[i] = match self.set {
            FeatureSet::V1 => {
                ((1.0 + self.ctx.n_candidates as f32).ln() / (1.0 + CAND_SCALE).ln())
                    .clamp(0.0, 1.0)
            }
            FeatureSet::V2 => ratio_squash(
                self.ctx.n_candidates as f32 / snapshot.node_count().max(1) as f32,
            ),
        };
        i += 1;

        // 7. share of this step's candidates carrying the same rule.
        let same = self
            .ctx
            .per_rule
            .get(binding.rule_id)
            .copied()
            .unwrap_or(0) as f32;
        self.scratch[i] = if self.ctx.n_candidates > 0 {
            same / self.ctx.n_candidates as f32
        } else {
            0.0
        };

        &self.scratch
    }

    /// Number of WL hashes computed this step (bench/diagnostic use: the
    /// cache's whole justification is that this stays near the distinct
    /// bound-node count, not the candidate count).
    pub fn wl_cache_len(&self) -> usize {
        self.wl_cache.len()
    }

    pub fn current_step(&self) -> u64 {
        self.ctx.step
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{candidate_pool, creation, matched, node_ids, ring_store};

    #[test]
    fn dim_is_n_rules_plus_buckets_plus_seven() {
        let fx = FeatureExtractor::new(6, 16);
        assert_eq!(fx.dim(), 6 + 16 + 7);
        assert_eq!(feat_dim(3, 8), 18);
    }

    #[test]
    fn every_component_stays_in_range() {
        let store = ring_store(64, 12);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 2, 40, 4, 3);
        let mut fx = FeatureExtractor::new(4, 16);
        fx.begin_step(&snap, &cands);
        for b in &cands {
            for (j, v) in fx.features(&snap, b).iter().enumerate() {
                assert!(v.is_finite(), "feature {j} not finite");
                assert!((-1.0..=1.0).contains(v), "feature {j} out of range: {v}");
            }
        }
    }

    #[test]
    fn creation_binding_has_zero_degree_and_wl_blocks() {
        let store = ring_store(16, 2);
        let snap = store.snapshot();
        let cands = vec![creation(1, 0)];
        let mut fx = FeatureExtractor::new(3, 8);
        fx.begin_step(&snap, &cands);
        let f = fx.features(&snap, &cands[0]).to_vec();

        assert_eq!(f[1], 1.0, "rule one-hot");
        // degree block (3) and WL block (8) both all-zero.
        let deg_start = 3 + 2;
        assert!(f[deg_start..deg_start + 3].iter().all(|v| *v == 0.0));
        let wl_start = deg_start + 3;
        assert!(f[wl_start..wl_start + 8].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn wl_block_sums_to_one_for_node_bearing_bindings() {
        let store = ring_store(32, 4);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        let cands = vec![matched(0, 0, &ids[0..4])];
        let mut fx = FeatureExtractor::new(2, 8);
        fx.begin_step(&snap, &cands);
        let f = fx.features(&snap, &cands[0]).to_vec();
        let wl_start = 2 + 2 + 3;
        let sum: f32 = f[wl_start..wl_start + 8].iter().sum();
        assert!((sum - 1.0).abs() < 1e-5, "WL block summed to {sum}");
    }

    #[test]
    fn degree_feature_separates_a_hub_from_a_ring_node() {
        // Node 0 carries the ring's two edges plus every hub edge.
        let store = ring_store(32, 16);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        let hub = matched(0, 0, &ids[0..1]);
        let plain = matched(0, 0, &ids[8..9]);
        let cands = vec![hub.clone(), plain.clone()];
        let mut fx = FeatureExtractor::new(1, 4);
        fx.begin_step(&snap, &cands);
        let deg_ix = 1 + 2;
        let hub_deg = fx.features(&snap, &hub)[deg_ix];
        let plain_deg = fx.features(&snap, &plain)[deg_ix];
        assert!(
            hub_deg > plain_deg,
            "hub {hub_deg} should out-rank plain {plain_deg}"
        );
    }

    #[test]
    fn rule_share_reflects_the_pool() {
        let store = ring_store(16, 0);
        let snap = store.snapshot();
        // 8 matches over 2 rules => each rule holds exactly half.
        let cands = candidate_pool(&store, 0, 8, 2, 1);
        let mut fx = FeatureExtractor::new(2, 4);
        fx.begin_step(&snap, &cands);
        let share_ix = fx.dim() - 1;
        let share = fx.features(&snap, &cands[0])[share_ix];
        assert!((share - 0.5).abs() < 1e-6, "share was {share}");
    }

    #[test]
    fn wl_hashes_are_computed_per_node_not_per_candidate() {
        let store = ring_store(8, 0);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        // 64 candidates all bound to the same 2 nodes.
        let cands: Vec<_> = (0..64).map(|_| matched(0, 0, &ids[0..2])).collect();
        let mut fx = FeatureExtractor::new(1, 8);
        fx.begin_step(&snap, &cands);
        for b in &cands {
            let _ = fx.features(&snap, b);
        }
        assert_eq!(
            fx.wl_cache_len(),
            2,
            "cache should hold one entry per distinct bound node"
        );
    }

    /// The whole point of the strategy switch: the two WL paths must
    /// produce byte-identical features. If they ever diverged, a champion
    /// would score differently depending on how many nodes its step
    /// happened to touch — a determinism break that no fitness comparison
    /// would survive and no test elsewhere would catch.
    #[test]
    fn both_wl_strategies_give_identical_features() {
        let store = ring_store(64, 6);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        // 40 bindings over 40 distinct nodes on a 64-node graph:
        // 40 · 8 > 64, so priming takes the whole-graph pass.
        let cands: Vec<_> = (0..40).map(|i| matched(0, 0, &ids[i..i + 1])).collect();

        let mut primed = FeatureExtractor::new(1, 16);
        primed.begin_step(&snap, &cands);
        primed.prime_wl(&snap, cands.iter());
        assert!(primed.primed_whole_graph(), "expected the class-map path");
        let a: Vec<Vec<f32>> = cands
            .iter()
            .map(|b| primed.features(&snap, b).to_vec())
            .collect();

        // Unprimed: every hash computed on demand by ball walk.
        let mut lazy = FeatureExtractor::new(1, 16);
        lazy.begin_step(&snap, &cands);
        let b: Vec<Vec<f32>> = cands
            .iter()
            .map(|c| lazy.features(&snap, c).to_vec())
            .collect();

        for (ra, rb) in a.iter().zip(&b) {
            for (x, y) in ra.iter().zip(rb) {
                assert_eq!(x.to_bits(), y.to_bits(), "WL strategies diverged");
            }
        }
    }

    #[test]
    fn prime_wl_takes_the_per_node_path_when_few_nodes_are_touched() {
        let store = ring_store(512, 4);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        // 4 nodes out of 512: 4 · 8 = 32 <= 512, so ball walks.
        let cands: Vec<_> = (0..4).map(|i| matched(0, 0, &ids[i..i + 1])).collect();
        let mut fx = FeatureExtractor::new(1, 8);
        fx.begin_step(&snap, &cands);
        fx.prime_wl(&snap, cands.iter());
        assert!(!fx.primed_whole_graph());
        assert_eq!(fx.wl_cache_len(), 4, "only the touched nodes are hashed");
    }

    #[test]
    fn prime_wl_on_creation_only_bindings_is_a_no_op() {
        let store = ring_store(32, 0);
        let snap = store.snapshot();
        let cands = vec![creation(0, 0), creation(1, 0)];
        let mut fx = FeatureExtractor::new(2, 8);
        fx.begin_step(&snap, &cands);
        fx.prime_wl(&snap, cands.iter());
        assert_eq!(fx.wl_cache_len(), 0);
        assert!(!fx.primed_whole_graph());
    }

    #[test]
    fn extraction_is_a_pure_function_of_snapshot_and_binding() {
        let store = ring_store(48, 6);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 1, 20, 3, 2);

        let mut a = FeatureExtractor::new(3, 16);
        a.begin_step(&snap, &cands);
        let out_a: Vec<Vec<f32>> = cands.iter().map(|b| a.features(&snap, b).to_vec()).collect();

        let mut b = FeatureExtractor::new(3, 16);
        b.begin_step(&snap, &cands);
        let out_b: Vec<Vec<f32>> = cands.iter().map(|c| b.features(&snap, c).to_vec()).collect();

        assert_eq!(out_a, out_b);
    }
    /// E0's fix, pinned. The two v1 scales that **saturate** must become
    /// unable to, at any observable magnitude — and the two that merely
    /// compress must be left alone, because under-using a range loses no
    /// information while saturating one destroys distinctions.
    #[test]
    fn v2_cannot_saturate_the_two_scales_v1_pins() {
        // The measured extremes, from the E0 audit, plus an order of
        // magnitude past them in each direction so the claim is about the
        // encoding rather than about these substrates.
        for deg_ratio in [0.0f32, 0.5, 1.0, 2.0, 10.0, 130.0, 1.0e4] {
            let v = ratio_squash(deg_ratio);
            assert!(
                (0.0..1.0).contains(&v),
                "ratio_squash({deg_ratio}) = {v} left [0, 1)"
            );
            if deg_ratio > 0.0 {
                assert!(v > 0.0, "only a zero observable may map to zero");
                assert!(v < 1.0, "nothing may reach the ceiling: {deg_ratio} -> {v}");
            }
        }
        // Strictly monotone, so no two distinct observables collide — which
        // is exactly what v1's clamp does above its scale.
        let mut prev = -1.0f32;
        for x in [0.1f32, 1.0, 5.0, 64.0, 65.0, 130.0, 500.0] {
            let v = ratio_squash(x);
            assert!(v > prev, "must be strictly increasing at {x}");
            prev = v;
        }

        // And v1's clamp really does collide, on the measured values: degree
        // 69 and 130 are indistinguishable at DEG_SCALE = 64.
        let v1 = |deg: f32| ((1.0 + deg).ln() / (1.0 + DEG_SCALE).ln()).clamp(0.0, 1.0);
        assert_eq!(
            v1(69.0),
            v1(130.0),
            "the v1 defect this test protects against must still be real"
        );
        assert!(ratio_squash(69.0) < ratio_squash(130.0));
    }


}