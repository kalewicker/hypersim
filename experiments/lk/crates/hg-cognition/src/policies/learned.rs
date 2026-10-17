//! The learned policies: `StaticBias` (Stage 2, CMA-ES) and `MlpScorer`
//! (Stage 3, OpenAI-ES). NN plan §5.4, §5.6.

use super::{categorical, masked_softmax};
use crate::features::{FeatureExtractor, CAND_SCORE_CAP};
use crate::genome::Parameterized;
use crate::select::{gumbel_top_k, partition_candidates};
use hg_core::GraphSnapshot;
use hg_observe::{Binding, RuleSelectionPolicy};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// One logit per rule — the smallest possible learnable policy.
///
/// It can express "prefer rule 3 to rule 1" and nothing else. That is the
/// point: if CMA-ES over `StaticBias` cannot beat `RandomSubset` on an
/// environment *constructed* so that rule identity matters (fixture F-L1),
/// the training loop is broken, not the hypothesis.
///
/// Selection follows the plan literally: the budget is allocated across
/// rules by softmax over the rule logits, and candidates are drawn
/// uniformly within a rule's group. Crucially this is **group-size
/// independent** — a rule offering 3 candidates and a rule offering 300
/// compete on their logits alone, so the policy can express a strong
/// preference for a rare rule. Scoring each candidate by its rule's logit
/// and taking a Gumbel-top-k would instead weight by group size and quietly
/// make that preference inexpressible.
pub struct StaticBias {
    logits: Vec<f32>,
    budget: Option<usize>,
    rng: ChaCha8Rng,
    // Scratch, reused across steps.
    groups: Vec<Vec<usize>>,
    mask: Vec<bool>,
    probs: Vec<f32>,
}

impl StaticBias {
    pub fn new(n_rules: usize, budget: Option<usize>, seed: u64) -> Self {
        Self {
            logits: vec![0.0; n_rules],
            budget,
            rng: ChaCha8Rng::seed_from_u64(seed),
            groups: vec![Vec::new(); n_rules],
            mask: vec![false; n_rules],
            probs: Vec::new(),
        }
    }

    pub fn logits(&self) -> &[f32] {
        &self.logits
    }
}

impl Parameterized for StaticBias {
    fn n_params(&self) -> usize {
        self.logits.len()
    }

    fn set_params(&mut self, theta: &[f32]) {
        assert_eq!(
            theta.len(),
            self.logits.len(),
            "StaticBias genome length mismatch: a truncated or padded genome would run, \
             score, and mean nothing"
        );
        self.logits.copy_from_slice(theta);
    }

    fn get_params(&self) -> Vec<f32> {
        self.logits.clone()
    }
}

impl RuleSelectionPolicy for StaticBias {
    fn select(&mut self, _snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let (creation, matches) = partition_candidates(candidates);
        let Some(k) = self.budget else {
            return creation.into_iter().chain(matches).collect();
        };

        for g in self.groups.iter_mut() {
            g.clear();
        }
        for i in matches {
            if let Some(g) = self.groups.get_mut(candidates[i].rule_id) {
                g.push(i);
            }
        }
        for (r, m) in self.mask.iter_mut().enumerate() {
            *m = !self.groups[r].is_empty();
        }

        let mut out = creation;
        for _ in 0..k {
            masked_softmax(&self.logits, &self.mask, &mut self.probs);
            let u: f32 = self.rng.gen_range(0.0f32..1.0);
            let Some(rule) = categorical(&self.probs, u) else {
                break; // every group exhausted
            };
            // Uniform within the group; swap_remove keeps it O(1) and is
            // deterministic given the RNG stream.
            let g = &mut self.groups[rule];
            let pick = self.rng.gen_range(0..g.len());
            out.push(g.swap_remove(pick));
            if g.is_empty() {
                self.mask[rule] = false;
            }
        }
        // Selection order is not meaningful to the engine (it canonically
        // sorts and dedups), but a sorted return keeps the policy's own
        // output comparable across runs in tests and logs.
        out.sort_unstable();
        out
    }
}

/// Per-candidate MLP scorer over `feature_set = "v1"` — the Stage-3 policy.
///
/// One hidden ReLU layer to a scalar score, then Gumbel-top-k. At defaults
/// (`feat_dim ≈ 32`, `hidden = 16`) that is ~545 parameters, well inside
/// ES's comfortable regime.
///
/// The `&mut self` in `RuleSelectionPolicy::select` permits recurrent state
/// later — a policy carrying a compressed summary of its own past
/// selections. v1 policies are deliberately stateless beyond the RNG and
/// the per-step feature cache.
/// How raw scorer outputs are treated before `gumbel_top_k` perturbs them.
///
/// # Why this exists (plan v3 §4, N-1)
///
/// `gumbel_top_k` adds `Gumbel(0, 1)` noise — sd ≈ 1.28 — to raw scores.
/// An MLP's output scale is **quadratic** in its genome, so with `Raw`
/// scoring the policy's behaviour is a function of `|theta|`:
///
/// | `|theta|` | score spread | behaviour |
/// |---|---|---|
/// | 0.54 | 0.0055 | budget-matched random, whatever the weights say |
/// | 7 | 0.921 | responsive |
/// | 41.6 | 32.5 | deterministic argmax, the Gumbel draw irrelevant |
///
/// v1 §5.6 chose Gumbel over deterministic top-k *because* deterministic
/// selection makes fitness locally flat under ES perturbation. That
/// diagnosis was right and the fix **relocated** the flatness rather than
/// removing it: there is now a flat region at small `|theta|` and the
/// original flat region returns at large `|theta|`, with a trainable window
/// between them that nothing keeps the search inside.
///
/// `PoolStandard` removes the window instead of navigating it. Selection
/// then depends only on the *relative ordering and spacing* the network
/// produces, never on its absolute output scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScoreNorm {
    /// N-1. Standardize across the pool being selected from before adding
    /// Gumbel noise. **The adopted architecture.**
    #[default]
    PoolStandard,
    /// The pre-E1 behaviour. Retained *only* so the regression test can
    /// demonstrate the basin it removes; not a supported configuration.
    Raw,
}

/// N-1's epsilon. Guards the degenerate pool (every score equal, `sd = 0`),
/// which correctly reduces to uniform selection.
pub const SCORE_NORM_EPS: f32 = 1e-6;

/// Standardize scores across the pool: `(s - mean) / (sd + eps)`.
pub fn pool_standardize(scores: &mut [f32]) {
    let n = scores.len();
    if n == 0 {
        return;
    }
    let mean = scores.iter().sum::<f32>() / n as f32;
    let var = scores.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / n as f32;
    let denom = var.sqrt() + SCORE_NORM_EPS;
    for s in scores.iter_mut() {
        *s = (*s - mean) / denom;
    }
}

pub struct MlpScorer {
    norm: ScoreNorm,
    w1: Vec<f32>, // [hidden × feat_dim], row-major
    b1: Vec<f32>, // [hidden]
    w2: Vec<f32>, // [hidden]
    b2: f32,
    hidden: usize,
    feat_dim: usize,
    budget: Option<usize>,
    rng: ChaCha8Rng,
    feat: FeatureExtractor,
    /// Steps on which the 512-candidate cap bound. Reported per rollout:
    /// a rule set where it binds on most steps is one where the policy is
    /// choosing from a sample rather than from the pool, and that must be
    /// said out loud rather than discovered later.
    ///
    /// Shared rather than plain, because the policy is moved into the engine
    /// and dropped when the rollout ends — the campaign runner needs a
    /// handle it still owns afterwards, or the column would have to be
    /// silently reported as zero.
    cap_bound_steps: Arc<AtomicU64>,
}

impl MlpScorer {
    /// v1 features. `with_features` selects the encoding explicitly.
    pub fn new(n_rules: usize, wl_buckets: usize, hidden: usize, budget: Option<usize>, seed: u64) -> Self {
        Self::with_features(
            n_rules,
            wl_buckets,
            hidden,
            budget,
            seed,
            crate::features::FeatureSet::V1,
        )
    }

    pub fn with_features(
        n_rules: usize,
        wl_buckets: usize,
        hidden: usize,
        budget: Option<usize>,
        seed: u64,
        set: crate::features::FeatureSet,
    ) -> Self {
        Self::with_features_and_norm(n_rules, wl_buckets, hidden, budget, seed, set, ScoreNorm::default())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_features_and_norm(
        n_rules: usize,
        wl_buckets: usize,
        hidden: usize,
        budget: Option<usize>,
        seed: u64,
        set: crate::features::FeatureSet,
        norm: ScoreNorm,
    ) -> Self {
        let feat = FeatureExtractor::with_set(n_rules, wl_buckets, set);
        let feat_dim = feat.dim();
        Self {
            norm,
            w1: vec![0.0; hidden * feat_dim],
            b1: vec![0.0; hidden],
            w2: vec![0.0; hidden],
            b2: 0.0,
            hidden,
            feat_dim,
            budget,
            rng: ChaCha8Rng::seed_from_u64(seed),
            feat,
            cap_bound_steps: Arc::new(AtomicU64::new(0)),
        }
    }

    /// A handle to the cap counter that outlives the policy. Take this
    /// before handing the policy to the engine.
    pub fn cap_counter(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.cap_bound_steps)
    }

    pub fn feat_dim(&self) -> usize {
        self.feat_dim
    }

    /// Hidden width — written into the champion record
    /// (`docs/campaign_output.md` §6) so a saved genome carries its layout.
    pub fn hidden(&self) -> usize {
        self.hidden
    }

    pub fn cap_bound_steps(&self) -> u64 {
        self.cap_bound_steps.load(Ordering::Relaxed)
    }

    pub fn score(&self, x: &[f32]) -> f32 {
        forward(&self.w1, &self.b1, &self.w2, self.b2, self.feat_dim, x)
    }
}

/// One hidden ReLU layer to a scalar. A free function rather than a method
/// so that `select` can hold `&mut self.feat` (the feature scratch buffer)
/// and `&self.w1` at the same time — disjoint field borrows are fine, a
/// whole-`self` method call is not.
fn forward(w1: &[f32], b1: &[f32], w2: &[f32], b2: f32, feat_dim: usize, x: &[f32]) -> f32 {
    let mut s = b2;
    for (h, (bias, out_w)) in b1.iter().zip(w2).enumerate() {
        let row = &w1[h * feat_dim..(h + 1) * feat_dim];
        let mut a = *bias;
        for (w, xi) in row.iter().zip(x) {
            a += w * xi;
        }
        s += out_w * a.max(0.0);
    }
    s
}

impl Parameterized for MlpScorer {
    fn n_params(&self) -> usize {
        self.w1.len() + self.b1.len() + self.w2.len() + 1
    }

    fn set_params(&mut self, theta: &[f32]) {
        assert_eq!(
            theta.len(),
            self.n_params(),
            "MlpScorer genome length mismatch: a truncated or padded genome would run, \
             score, and mean nothing"
        );
        let (a, rest) = theta.split_at(self.w1.len());
        let (b, rest) = rest.split_at(self.b1.len());
        let (c, rest) = rest.split_at(self.w2.len());
        self.w1.copy_from_slice(a);
        self.b1.copy_from_slice(b);
        self.w2.copy_from_slice(c);
        self.b2 = rest[0];
    }

    fn get_params(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.n_params());
        out.extend_from_slice(&self.w1);
        out.extend_from_slice(&self.b1);
        out.extend_from_slice(&self.w2);
        out.push(self.b2);
        out
    }
}

impl RuleSelectionPolicy for MlpScorer {
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let (creation, matches) = partition_candidates(candidates);

        // Cap scoring cost on pathological candidate counts.
        let pool = if matches.len() > CAND_SCORE_CAP {
            self.cap_bound_steps.fetch_add(1, Ordering::Relaxed);
            let mut m = matches;
            // Take the slice `partial_shuffle` returns, not a prefix — see
            // `RandomSubset::select` for the measurement. Truncating to the
            // prefix here would have capped the MLP's view to the first 512
            // bindings in engine order on every dense step.
            let (chosen, _rest) = m.partial_shuffle(&mut self.rng, CAND_SCORE_CAP);
            let mut m = chosen.to_vec();
            // The subsample is a set, not a sequence: sorting it keeps the
            // scoring order a function of the candidate pool rather than of
            // the shuffle, so only *which* candidates are scored depends on
            // the RNG, never the order the Gumbel draws are consumed in.
            m.sort_unstable();
            m
        } else {
            matches
        };

        // Features 6 and 7 are properties of the whole pool, so the step
        // context is built from the full candidate slice — not from the
        // capped subsample, which would misreport the candidate count.
        self.feat.begin_step(snapshot, candidates);
        // Primed from the CAPPED pool, not the full one: the WL work is
        // only needed for candidates that will actually be scored, and
        // priming from the full pool would trigger the whole-graph pass on
        // steps where the scorer touches a handful of nodes.
        self.feat
            .prime_wl(snapshot, pool.iter().map(|&i| &candidates[i]));

        // Disjoint field borrows: `feat` mutable (it owns the scratch
        // buffer), the weights immutable.
        let (w1, b1, w2, b2, feat_dim, feat) = (
            &self.w1,
            &self.b1,
            &self.w2,
            self.b2,
            self.feat_dim,
            &mut self.feat,
        );
        let scored: Vec<(usize, f32)> = pool
            .into_iter()
            .map(|i| {
                let x = feat.features(snapshot, &candidates[i]);
                (i, forward(w1, b1, w2, b2, feat_dim, x))
            })
            .collect();

        // N-1: standardize across the pool before the Gumbel draw, so the
        // policy depends on the network's relative ordering and spacing
        // rather than on its absolute output scale (`ScoreNorm`).
        let mut scored = scored;
        if self.norm == ScoreNorm::PoolStandard {
            let mut vals: Vec<f32> = scored.iter().map(|(_, s)| *s).collect();
            pool_standardize(&mut vals);
            for ((_, s), v) in scored.iter_mut().zip(vals) {
                *s = v;
            }
        }

        let mut picked = gumbel_top_k(scored, self.budget, &mut self.rng);
        picked.sort_unstable();
        creation.into_iter().chain(picked).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{candidate_pool, creation, matched, node_ids, ring_store};

    fn seeded_theta(n: usize, seed: u64) -> Vec<f32> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..n).map(|_| rng.gen_range(-1.0f32..1.0)).collect()
    }

    // ---- StaticBias ----

    #[test]
    fn static_bias_param_round_trip() {
        let mut p = StaticBias::new(5, Some(4), 1);
        assert_eq!(p.n_params(), 5);
        let theta = seeded_theta(5, 42);
        p.set_params(&theta);
        assert_eq!(p.get_params(), theta);
    }

    #[test]
    #[should_panic(expected = "genome length mismatch")]
    fn static_bias_rejects_a_wrong_length_genome() {
        StaticBias::new(5, Some(4), 1).set_params(&[0.0; 4]);
    }

    #[test]
    fn static_bias_favours_the_high_logit_rule() {
        let store = ring_store(64, 2);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 0, 100, 2, 1);
        let mut p = StaticBias::new(2, Some(20), 5);
        p.set_params(&[-3.0, 3.0]);
        let picked = p.select(&snap, &cands);
        let rule1 = picked.iter().filter(|&&i| cands[i].rule_id == 1).count();
        assert!(rule1 >= 18, "expected rule 1 to dominate, got {rule1}/20");
    }

    /// **`StaticBias` is inexpressible on a single-rule set**, and every
    /// builtin rule set ships exactly one rule.
    ///
    /// A masked softmax over one logit is 1.0 for every value of that
    /// logit, so the allocation step has no choice to make and the draw
    /// falls through to uniform-within-group — which is precisely
    /// `RandomSubset`. Two genomes at opposite extremes must therefore
    /// select the *same indices* given the same RNG stream. Stage 2 guards
    /// on this in `campaign::run_campaign`; without the guard a campaign
    /// runs to completion and reports a margin that is pure policy-RNG
    /// noise.
    #[test]
    fn a_single_rule_genome_cannot_change_selection() {
        let store = ring_store(64, 1);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 0, 100, 1, 1);

        let mut low = StaticBias::new(1, Some(16), 77);
        low.set_params(&[-9.0]);
        let mut high = StaticBias::new(1, Some(16), 77);
        high.set_params(&[9.0]);

        assert_eq!(
            low.select(&snap, &cands),
            high.select(&snap, &cands),
            "a one-rule genome changed the selection — the Stage 2 guard would be wrong"
        );

        // And what it selects is a uniform sample, i.e. the same
        // *distribution* as budget-matched random — not the same index
        // list, since the two policies consume their RNG streams
        // differently.
        let mut counts = vec![0usize; cands.len()];
        for seed in 0..400u64 {
            let mut p = StaticBias::new(1, Some(10), seed);
            p.set_params(&[3.0]);
            for i in p.select(&snap, &cands) {
                counts[i] += 1;
            }
        }
        assert_eq!(
            counts.iter().filter(|c| **c == 0).count(),
            0,
            "a one-rule StaticBias must still spread its budget over the whole pool"
        );
    }

    /// The group-size-independence property that motivates the softmax
    /// allocation: a rare rule with a high logit still wins its share.
    #[test]
    fn static_bias_allocation_ignores_group_size() {
        let store = ring_store(64, 2);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        // Rule 0: 200 candidates. Rule 1: 4 candidates.
        let mut cands: Vec<Binding> = (0..200)
            .map(|i| matched(0, 0, &ids[i % ids.len()..i % ids.len() + 1]))
            .collect();
        cands.extend((0..4).map(|i| matched(1, 0, &ids[i..i + 1])));

        let mut p = StaticBias::new(2, Some(8), 9);
        p.set_params(&[0.0, 4.0]);
        let picked = p.select(&snap, &cands);
        let rule1 = picked.iter().filter(|&&i| cands[i].rule_id == 1).count();
        assert_eq!(
            rule1, 4,
            "all 4 rare-rule candidates should be taken despite the 50:1 group ratio"
        );
    }

    #[test]
    fn static_bias_exempts_creations_and_holds_the_budget() {
        let store = ring_store(32, 2);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 3, 40, 3, 2);
        let mut p = StaticBias::new(3, Some(7), 2);
        let picked = p.select(&snap, &cands);
        let n_creation = picked.iter().filter(|&&i| cands[i].is_creation()).count();
        assert_eq!(n_creation, 3);
        assert_eq!(picked.len() - n_creation, 7);
    }

    #[test]
    fn static_bias_never_returns_a_duplicate() {
        let store = ring_store(16, 0);
        let snap = store.snapshot();
        // Budget exceeds the pool: the group-exhaustion path must not
        // re-draw an already-taken candidate.
        let cands = candidate_pool(&store, 1, 5, 2, 1);
        let mut p = StaticBias::new(2, Some(50), 4);
        let picked = p.select(&snap, &cands);
        let mut dedup = picked.clone();
        dedup.dedup();
        assert_eq!(picked, dedup);
        assert_eq!(picked.len(), 6, "1 creation + all 5 matches");
    }

    // ---- ScoreNorm / N-1 (plan v3 §4) ----

    /// The exit criterion for E1's N-1 half, as plan v3 §4 states it:
    ///
    /// > score spread at `‖θ‖ ∈ {0.5, 7, 42}` produces selection-divergence
    /// > within noise of each other under N-1, where the pre-fix
    /// > architecture spans random-equivalent to argmax across that range.
    ///
    /// Measured on the *scores after normalization*, which is where the
    /// claim lives: under `Raw` the spread grows quadratically in `‖θ‖` and
    /// so crosses the Gumbel scale of ~1.28 somewhere in the middle of that
    /// range; under `PoolStandard` it is invariant.
    #[test]
    fn n1_removes_the_gumbel_basin_across_three_orders_of_theta() {
        let mut p = MlpScorer::new(2, 16, 16, Some(4), 1);
        let n = p.n_params();
        let d = p.feat_dim();
        let xs: Vec<Vec<f32>> = (0..24)
            .map(|c| (0..d).map(|i| (((i * 7 + c * 13) % 11) as f32) / 10.0).collect())
            .collect();

        // One direction, three magnitudes.
        let dir = seeded_theta(n, 99);
        let dn = dir.iter().map(|v| v * v).sum::<f32>().sqrt();

        let spread_of = |p: &mut MlpScorer, norm: ScoreNorm, l2: f32| -> f32 {
            let g: Vec<f32> = dir.iter().map(|v| v * l2 / dn).collect();
            p.set_params(&g);
            let mut s: Vec<f32> = xs.iter().map(|x| p.score(x)).collect();
            if norm == ScoreNorm::PoolStandard {
                pool_standardize(&mut s);
            }
            s.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
                - s.iter().cloned().fold(f32::INFINITY, f32::min)
        };

        let mags = [0.5f32, 7.0, 42.0];
        let raw: Vec<f32> = mags.iter().map(|m| spread_of(&mut p, ScoreNorm::Raw, *m)).collect();
        let n1: Vec<f32> = mags
            .iter()
            .map(|m| spread_of(&mut p, ScoreNorm::PoolStandard, *m))
            .collect();

        // The defect, still real: raw spread crosses the Gumbel scale.
        assert!(
            raw[0] < 0.1,
            "|theta| = 0.5 must be far below the Gumbel sd of 1.28, got {}",
            raw[0]
        );
        assert!(
            raw[2] > 10.0,
            "|theta| = 42 must be far above it, got {}",
            raw[2]
        );
        assert!(
            raw[2] / raw[0].max(f32::MIN_POSITIVE) > 100.0,
            "the pre-fix architecture must span orders of magnitude: {raw:?}"
        );

        // N-1: invariant to the genome's scale. Only the direction matters,
        // and the direction is identical across the three.
        for w in n1.windows(2) {
            let rel = (w[1] - w[0]).abs() / w[0].max(f32::MIN_POSITIVE);
            assert!(
                rel < 1e-3,
                "N-1 spread must not depend on |theta|: {n1:?} (rel {rel})"
            );
        }
        assert!(
            n1[0] > 0.5,
            "a standardized pool must still separate candidates, got {}",
            n1[0]
        );
    }

    /// The degenerate pool, which is the one case N-1's epsilon exists for.
    #[test]
    fn n1_reduces_to_uniform_when_every_score_ties() {
        let mut all_equal = vec![3.5f32; 8];
        pool_standardize(&mut all_equal);
        assert!(
            all_equal.iter().all(|v| v.abs() < 1e-3),
            "a tied pool must standardize to zeros, not to NaN or infinity: {all_equal:?}"
        );

        // theta = 0 is that case, so N-1's neutral point is still
        // random-equivalent — but now by degenerate normalization rather
        // than by sub-threshold spread, and with no basin around it
        // (plan v3 §4).
        let mut p = MlpScorer::new(2, 16, 16, Some(4), 1);
        let n = p.n_params();
        p.set_params(&vec![0.0; n]);
        let d = p.feat_dim();
        let mut s: Vec<f32> = (0..8)
            .map(|c| p.score(&(0..d).map(|i| ((i + c) % 5) as f32 / 4.0).collect::<Vec<f32>>()))
            .collect();
        assert!(s.iter().all(|v| *v == 0.0));
        pool_standardize(&mut s);
        assert!(s.iter().all(|v| v.abs() < 1e-3));
    }

    /// Single scores and empty pools must not panic or produce NaN.
    #[test]
    fn n1_handles_degenerate_pool_sizes() {
        let mut empty: Vec<f32> = vec![];
        pool_standardize(&mut empty);
        let mut one = vec![42.0f32];
        pool_standardize(&mut one);
        assert!(one[0].abs() < 1e-3, "a lone score standardizes to 0, got {}", one[0]);
    }

    // ---- MlpScorer ----

    #[test]
    fn mlp_param_count_matches_the_layout() {
        let p = MlpScorer::new(4, 16, 16, Some(8), 1);
        assert_eq!(p.feat_dim(), 4 + 16 + 7);
        assert_eq!(p.n_params(), 16 * 27 + 16 + 16 + 1);
    }

    #[test]
    fn mlp_param_round_trip_is_exact() {
        let mut p = MlpScorer::new(4, 16, 16, Some(8), 1);
        let theta = seeded_theta(p.n_params(), 7);
        p.set_params(&theta);
        let back = p.get_params();
        for (a, b) in theta.iter().zip(&back) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    #[test]
    #[should_panic(expected = "genome length mismatch")]
    fn mlp_rejects_a_wrong_length_genome() {
        let mut p = MlpScorer::new(4, 16, 16, Some(8), 1);
        let n = p.n_params();
        p.set_params(&vec![0.0; n - 1]);
    }

    /// The Phase C1 exit criterion: identical (snapshot, candidates,
    /// params, rng seed) => identical selection.
    #[test]
    fn mlp_selection_is_reproducible() {
        let store = ring_store(64, 8);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 2, 120, 4, 3);
        let theta = seeded_theta(MlpScorer::new(4, 16, 16, Some(12), 0).n_params(), 3);

        let mut a = MlpScorer::new(4, 16, 16, Some(12), 77);
        a.set_params(&theta);
        let mut b = MlpScorer::new(4, 16, 16, Some(12), 77);
        b.set_params(&theta);
        let mut c = MlpScorer::new(4, 16, 16, Some(12), 78);
        c.set_params(&theta);

        let sa = a.select(&snap, &cands);
        assert_eq!(sa, b.select(&snap, &cands));
        assert_ne!(sa, c.select(&snap, &cands), "a different RNG seed must differ");
    }

    #[test]
    fn mlp_exempts_creations_and_holds_the_budget() {
        let store = ring_store(32, 4);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 5, 60, 3, 2);
        let mut p = MlpScorer::new(3, 16, 8, Some(9), 1);
        p.set_params(&seeded_theta(p.n_params(), 2));
        let picked = p.select(&snap, &cands);
        let n_creation = picked.iter().filter(|&&i| cands[i].is_creation()).count();
        assert_eq!(n_creation, 5);
        assert_eq!(picked.len() - n_creation, 9);
    }

    #[test]
    fn mlp_zero_genome_still_selects_the_budget() {
        // All-zero params score every candidate identically; selection is
        // then pure Gumbel noise, which must still fill the budget.
        let store = ring_store(32, 0);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 0, 40, 2, 1);
        let mut p = MlpScorer::new(2, 8, 4, Some(10), 1);
        assert_eq!(p.select(&snap, &cands).len(), 10);
    }

    #[test]
    fn mlp_cap_binds_and_is_counted() {
        let store = ring_store(1024, 16);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 0, CAND_SCORE_CAP + 200, 2, 1);
        let mut p = MlpScorer::new(2, 8, 4, Some(16), 1);
        p.set_params(&seeded_theta(p.n_params(), 5));
        let picked = p.select(&snap, &cands);
        assert_eq!(picked.len(), 16);
        assert_eq!(p.cap_bound_steps(), 1);

        // Below the cap it must not fire.
        let small = candidate_pool(&store, 0, 10, 2, 1);
        p.select(&snap, &small);
        assert_eq!(p.cap_bound_steps(), 1);
    }

    #[test]
    fn mlp_with_no_budget_keeps_every_candidate() {
        let store = ring_store(32, 2);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 3, 20, 2, 2);
        let mut p = MlpScorer::new(2, 8, 4, None, 1);
        p.set_params(&seeded_theta(p.n_params(), 1));
        assert_eq!(p.select(&snap, &cands).len(), cands.len());
    }

    #[test]
    fn mlp_learns_to_prefer_a_rule_when_told_to() {
        // Hand-build a genome whose first hidden unit reads rule 1's
        // one-hot and passes it to the output: a check that the feature
        // layout and the forward pass agree on where a rule lives.
        let store = ring_store(64, 2);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 0, 100, 2, 1);

        let hidden = 4;
        let mut p = MlpScorer::new(2, 8, hidden, Some(20), 3);
        let feat_dim = p.feat_dim();
        let w1_len = hidden * feat_dim;
        let w2_off = w1_len + hidden; // w1, then b1, then w2, then b2

        let mut theta = vec![0.0f32; p.n_params()];
        theta[1] = 10.0; // w1[hidden 0][feature 1] — rule 1's one-hot slot
        theta[w2_off] = 10.0; // w2[0]
        p.set_params(&theta);

        let picked = p.select(&snap, &cands);
        let rule1 = picked.iter().filter(|&&i| cands[i].rule_id == 1).count();
        // 18/20 under `Raw`, 17/20 under N-1 — and the change is the point,
        // not a regression. This genome scores rule 1 at 10 x 10 = 100 and
        // everything else at 0, a gap that swamps `Gumbel(0, 1)` entirely and
        // makes selection a deterministic argmax. N-1 standardizes the pool,
        // so the same genome now expresses a strong *preference* the Gumbel
        // draw can still occasionally overrule — which is the whole purpose
        // of the fix (`ScoreNorm`): no genome, however large, buys its way
        // back to argmax.
        //
        // The bar is therefore set against chance rather than against argmax.
        // Rule 1 is roughly half the pool, so ~10/20 is what no preference
        // looks like, and this test's claim is that the feature layout and
        // the forward pass agree on where a rule lives.
        assert!(
            rule1 >= 15,
            "expected rule 1 to dominate well above the ~10/20 chance rate, got {rule1}/20"
        );
    }

    #[test]
    fn mlp_creation_bindings_are_never_scored_away() {
        let store = ring_store(16, 0);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        let cands = vec![
            creation(0, 0),
            matched(0, 0, &ids[0..1]),
            matched(1, 0, &ids[1..2]),
        ];
        // Budget of 1: one match survives, and the creation regardless.
        let mut p = MlpScorer::new(2, 8, 4, Some(1), 1);
        p.set_params(&seeded_theta(p.n_params(), 8));
        let picked = p.select(&snap, &cands);
        assert!(picked.contains(&0));
        assert_eq!(picked.len(), 2);
    }
}
