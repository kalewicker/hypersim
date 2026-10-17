//! `PassThrough` and `RandomSubset` — the two zero-parameter controls
//! (NN plan §5.1–5.2).

use crate::select::partition_candidates;
use hg_core::GraphSnapshot;
use hg_observe::{Binding, RuleSelectionPolicy};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Returns every index. Reproduces engine-default `MaxParallel` behaviour
/// exactly.
///
/// Exists so that "policy machinery installed but inert" is testable: a
/// `PassThrough` rollout must produce the same `final_canonical_hash` as
/// the same config with `[cognition]` absent. That equality is the leak
/// detector for the entire integration — if it fails, the harness is
/// changing the physics before any learning has happened, and every
/// downstream comparison is meaningless.
#[derive(Debug, Default, Clone, Copy)]
pub struct PassThrough;

impl RuleSelectionPolicy for PassThrough {
    fn select(&mut self, _snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        (0..candidates.len()).collect()
    }
}

/// Uniform sample of `k` pattern-match bindings; creation bindings exempt.
///
/// **The universal control.** Every learned policy's fitness is reported as
/// an improvement over budget-matched `RandomSubset` and against nothing
/// else — comparing a budgeted learned policy against unconstrained
/// `MaxParallel` would confound learned preferences with throttled
/// dynamics.
pub struct RandomSubset {
    budget: Option<usize>,
    rng: ChaCha8Rng,
}

impl RandomSubset {
    pub fn new(budget: Option<usize>, seed: u64) -> Self {
        Self {
            budget,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn budget(&self) -> Option<usize> {
        self.budget
    }
}

impl RuleSelectionPolicy for RandomSubset {
    fn select(&mut self, _snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let (creation, mut matches) = partition_candidates(candidates);
        if let Some(k) = self.budget {
            if matches.len() > k {
                // Take the slice `partial_shuffle` HANDS BACK, never a
                // prefix of the original.
                //
                // This read `partial_shuffle(rng, k); truncate(k)` and that
                // was wrong in a way no test caught. In rand 0.8
                // `partial_shuffle` permutes into the *tail* and returns
                // `(tail, head)` — so truncating to the prefix kept the k
                // candidates the shuffle had mostly left alone. Measured on
                // 100 candidates at k = 16 it returned exactly
                // `[0, 1, ..., 15]`: the universal control was taking the
                // first k bindings in engine order, near-deterministically,
                // rather than sampling. Using the returned slice is correct
                // whichever end the implementation fills.
                let (chosen, _rest) = matches.partial_shuffle(&mut self.rng, k);
                matches = chosen.to_vec();
            }
        }
        creation.into_iter().chain(matches).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{candidate_pool, ring_store};

    #[test]
    fn pass_through_returns_everything_in_order() {
        let store = ring_store(16, 2);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 3, 20, 2, 2);
        let picked = PassThrough.select(&snap, &cands);
        assert_eq!(picked, (0..cands.len()).collect::<Vec<_>>());
    }

    #[test]
    fn random_subset_respects_budget_and_exempts_creations() {
        let store = ring_store(32, 2);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 4, 50, 3, 2);
        let mut p = RandomSubset::new(Some(6), 11);
        let picked = p.select(&snap, &cands);

        let n_creation = picked.iter().filter(|&&i| cands[i].is_creation()).count();
        let n_match = picked.len() - n_creation;
        assert_eq!(n_creation, 4, "every creation binding must survive");
        assert_eq!(n_match, 6, "pattern matches truncated to the budget");
    }

    /// The universal control has to actually be uniform, and the tests that
    /// existed could not tell.
    ///
    /// `random_subset_respects_budget_and_exempts_creations` checks the
    /// count and `random_subset_is_deterministic_in_its_seed` checks that
    /// two seeds differ — both pass for a policy that returns the first k
    /// bindings in engine order, which is what this one did: rand 0.8's
    /// `partial_shuffle` permutes into the tail and returns `(tail, head)`,
    /// so `truncate(k)` kept the prefix the shuffle had barely touched. On
    /// 100 candidates at k = 16 it returned exactly `[0, 1, ..., 15]`.
    ///
    /// This test measures the property directly: over many seeds, every
    /// candidate must be selected about equally often. A first-k policy
    /// puts the entire mass on the first k and zero everywhere else.
    #[test]
    fn random_subset_samples_uniformly_not_just_reproducibly() {
        let store = ring_store(64, 1);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 0, 100, 1, 1);
        const K: usize = 10;
        const TRIALS: u64 = 400;

        let mut counts = vec![0usize; cands.len()];
        for seed in 0..TRIALS {
            for i in RandomSubset::new(Some(K), seed).select(&snap, &cands) {
                counts[i] += 1;
            }
        }

        let expected = TRIALS as f64 * K as f64 / cands.len() as f64; // 40
        let never = counts.iter().filter(|c| **c == 0).count();
        assert_eq!(never, 0, "{never} candidates were never selected in {TRIALS} draws");
        let (lo, hi) = (
            *counts.iter().min().unwrap() as f64,
            *counts.iter().max().unwrap() as f64,
        );
        assert!(
            lo > expected * 0.4 && hi < expected * 1.6,
            "selection counts span {lo}..{hi} around an expected {expected} — not uniform"
        );
    }

    #[test]
    fn random_subset_with_no_budget_keeps_everything() {
        let store = ring_store(16, 0);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 2, 9, 2, 1);
        let mut p = RandomSubset::new(None, 5);
        assert_eq!(p.select(&snap, &cands).len(), cands.len());
    }

    #[test]
    fn random_subset_budget_above_pool_is_not_an_error() {
        let store = ring_store(16, 0);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 1, 3, 2, 1);
        let mut p = RandomSubset::new(Some(999), 5);
        assert_eq!(p.select(&snap, &cands).len(), 4);
    }

    #[test]
    fn random_subset_is_deterministic_in_its_seed() {
        let store = ring_store(32, 2);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 2, 40, 3, 2);
        let a = RandomSubset::new(Some(8), 99).select(&snap, &cands);
        let b = RandomSubset::new(Some(8), 99).select(&snap, &cands);
        let c = RandomSubset::new(Some(8), 100).select(&snap, &cands);
        assert_eq!(a, b);
        assert_ne!(a, c, "a different seed should sample differently");
    }
}

