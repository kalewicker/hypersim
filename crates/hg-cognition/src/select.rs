//! Shared selection mechanics: the creation/match partition, Gumbel-top-k,
//! and the policy RNG derivation (`docs/policy_config.md` §3, §5).

use hg_observe::Binding;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Split candidate indices into `(creation, pattern_match)`.
///
/// The predicate is `Binding::is_creation()` — `bound_nodes` **and**
/// `matched_edges` both empty — not `bound_nodes.is_empty()` alone. A rule
/// whose LHS matches edges but binds no node variables is an ordinary
/// pattern match, and the weaker predicate would exempt it from the budget.
/// This is also exactly the predicate the engine uses for its own
/// creations/ordinary split, so the two partitions agree by construction
/// rather than by coincidence.
pub fn partition_candidates(candidates: &[Binding]) -> (Vec<usize>, Vec<usize>) {
    let mut creation = Vec::new();
    let mut matches = Vec::new();
    for (i, b) in candidates.iter().enumerate() {
        if b.is_creation() {
            creation.push(i);
        } else {
            matches.push(i);
        }
    }
    (creation, matches)
}

/// Sample `k` items without replacement, proportional to `softmax(scores)`,
/// by perturbing each score with a Gumbel draw and taking the top `k`.
///
/// The noise is not decoration. Under deterministic top-k a small parameter
/// perturbation usually does not change which `k` candidates win, so ES
/// fitness is locally flat and the gradient estimate is mostly zero. The
/// Gumbel perturbation makes selection smooth in expectation, which is what
/// gives ES a usable search signal at all.
///
/// `scored` is `(index, score)`; the returned vector holds the winning
/// indices, longest-first by perturbed score. `k == None` means "take
/// everything" (still perturbed and sorted, so the ordering contract does
/// not change with the budget).
pub fn gumbel_top_k(
    mut scored: Vec<(usize, f32)>,
    k: Option<usize>,
    rng: &mut ChaCha8Rng,
) -> Vec<usize> {
    for (_, s) in scored.iter_mut() {
        // u ∈ (0, 1): the lower bound keeps ln(u) finite.
        let u: f32 = rng.gen_range(1e-9f32..1.0);
        *s += -(-u.ln()).ln();
    }
    // Ties broken by index so the result is a function of (scores, draws)
    // alone and never of the input vector's incidental order.
    scored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let take = k.unwrap_or(scored.len());
    scored.into_iter().take(take).map(|(i, _)| i).collect()
}

/// Derive a policy RNG seed (`docs/policy_config.md` §5).
///
/// `rollout_idx` is the index of the sim seed within `seeds_per_member`, so
/// the two rollouts of one member on two sim seeds get different policy
/// noise. Without it, antithetic pairs would share a Gumbel draw and the
/// common-random-numbers argument would do less work than it appears to.
///
/// FxHash-style mixing, written out rather than imported, so the derivation
/// is stable against any future dependency bump: a champion's reproduction
/// depends on this number.
pub fn derive_policy_seed(
    campaign_seed: u64,
    generation: u32,
    member_idx: u32,
    rollout_idx: u32,
    rng_seed_salt: u64,
) -> u64 {
    const K: u64 = 0x517c_c1b7_2722_0a95;
    let mut h: u64 = campaign_seed;
    for x in [
        generation as u64,
        member_idx as u64,
        rollout_idx as u64,
        rng_seed_salt,
    ] {
        h = (h ^ x).wrapping_mul(K);
        h ^= h >> 32;
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{creation, matched, node_ids, ring_store};
    use rand::SeedableRng;
    use smallvec::SmallVec;

    /// A binding with matched edges but no bound nodes is an ordinary
    /// pattern match, not a creation. This is the case the plan's sketched
    /// `bound_nodes.is_empty()` predicate would have gotten wrong.
    #[test]
    fn edge_only_binding_is_not_creation() {
        let store = ring_store(4, 0);
        let e = store.edges().next().expect("ring has edges").0;

        let mut edges: SmallVec<[hg_core::EdgeId; 8]> = SmallVec::new();
        edges.push(e);
        let edge_only = Binding::with_edges(0, 0, SmallVec::new(), edges);

        assert!(edge_only.bound_nodes.is_empty());
        assert!(!edge_only.is_creation());

        let (creation_ix, match_ix) = partition_candidates(&[creation(0, 0), edge_only]);
        assert_eq!(creation_ix, vec![0]);
        assert_eq!(match_ix, vec![1], "edge-only binding must be budgeted");
    }

    #[test]
    fn partition_preserves_indices() {
        let store = ring_store(4, 0);
        let ids = node_ids(&store);
        let cands = vec![
            matched(0, 0, &ids[0..1]),
            creation(1, 0),
            matched(2, 0, &ids[1..2]),
            creation(3, 0),
        ];
        let (c, m) = partition_candidates(&cands);
        assert_eq!(c, vec![1, 3]);
        assert_eq!(m, vec![0, 2]);
    }

    #[test]
    fn gumbel_top_k_is_deterministic_in_the_seed() {
        let scored: Vec<(usize, f32)> = (0..64).map(|i| (i, i as f32 * 0.1)).collect();
        let a = gumbel_top_k(scored.clone(), Some(8), &mut ChaCha8Rng::seed_from_u64(7));
        let b = gumbel_top_k(scored, Some(8), &mut ChaCha8Rng::seed_from_u64(7));
        assert_eq!(a, b);
    }

    #[test]
    fn gumbel_top_k_respects_budget_and_favours_high_scores() {
        // Scores far enough apart that noise (Gumbel sd ≈ 1.28) cannot
        // reorder them: the top 5 must be the five highest-scoring.
        let scored: Vec<(usize, f32)> = (0..20).map(|i| (i, i as f32 * 100.0)).collect();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let picked = gumbel_top_k(scored, Some(5), &mut rng);
        assert_eq!(picked.len(), 5);
        let mut sorted = picked.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![15, 16, 17, 18, 19]);
    }

    #[test]
    fn gumbel_top_k_none_budget_keeps_everything() {
        let scored: Vec<(usize, f32)> = (0..12).map(|i| (i, 0.0)).collect();
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        assert_eq!(gumbel_top_k(scored, None, &mut rng).len(), 12);
    }

    #[test]
    fn policy_seed_varies_in_every_component() {
        let base = derive_policy_seed(1, 2, 3, 4, 5);
        assert_ne!(base, derive_policy_seed(9, 2, 3, 4, 5));
        assert_ne!(base, derive_policy_seed(1, 9, 3, 4, 5));
        assert_ne!(base, derive_policy_seed(1, 2, 9, 4, 5));
        assert_ne!(base, derive_policy_seed(1, 2, 3, 9, 5));
        assert_ne!(base, derive_policy_seed(1, 2, 3, 4, 9));
        assert_eq!(base, derive_policy_seed(1, 2, 3, 4, 5));
    }
}
