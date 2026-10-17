//! Hand-coded, zero-parameter heuristics — the Stage-0 probes
//! (NN plan §5.3).
//!
//! Their job is to fail cheaply. If no hand heuristic can shift a
//! gate-relevant metric against budget-matched random on a rule set, then
//! selection has no causal power on that rule set and Stages 2–3 are
//! pointless there — which is itself a substrate result worth writing up.
//! Running them before any training is the whole cheap-gates-first
//! discipline.

use crate::config::DegreeDirection;
use crate::select::partition_candidates;
use hg_core::GraphSnapshot;
use hg_observe::{Binding, RuleSelectionPolicy};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Prefer bindings whose bound nodes have high (or low) mean degree.
///
/// Deterministic top-k, ties broken by candidate index. No RNG: this is a
/// probe, not a search target, and determinism makes its effect on the
/// dynamics unambiguous.
pub struct DegreeGreedy {
    budget: Option<usize>,
    direction: DegreeDirection,
}

impl DegreeGreedy {
    pub fn new(budget: Option<usize>, direction: DegreeDirection) -> Self {
        Self { budget, direction }
    }
}

impl RuleSelectionPolicy for DegreeGreedy {
    fn select(&mut self, snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let (creation, matches) = partition_candidates(candidates);
        let Some(k) = self.budget else {
            return creation.into_iter().chain(matches).collect();
        };

        let mut scored: Vec<(usize, f32)> = matches
            .into_iter()
            .map(|i| {
                let b = &candidates[i];
                let mean = if b.bound_nodes.is_empty() {
                    0.0
                } else {
                    b.bound_nodes
                        .iter()
                        .map(|&n| snapshot.edges_incident_to(n).len() as f32)
                        .sum::<f32>()
                        / b.bound_nodes.len() as f32
                };
                let s = match self.direction {
                    DegreeDirection::High => mean,
                    DegreeDirection::Low => -mean,
                };
                (i, s)
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        creation
            .into_iter()
            .chain(scored.into_iter().take(k).map(|(i, _)| i))
            .collect()
    }
}

/// Sample the budget from the candidate pool with per-rule weights.
///
/// A *bias*, not a filter: weights shift the sampling distribution rather
/// than imposing a hard order, so a low-weighted rule still fires
/// occasionally. A deterministic weight-sorted top-k would collapse to
/// "always the highest-weighted rule's first k candidates", which is a
/// different and much blunter intervention than the plan's "prefer rule 3
/// to rule 1".
///
/// A weight of exactly `0.0` does exclude a rule outright — that is the
/// intended way to spell "never this rule".
pub struct RuleBias {
    weights: Vec<f32>,
    budget: Option<usize>,
    rng: ChaCha8Rng,
}

impl RuleBias {
    pub fn new(weights: Vec<f32>, budget: Option<usize>, seed: u64) -> Self {
        Self {
            weights,
            budget,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }
}

impl RuleSelectionPolicy for RuleBias {
    fn select(&mut self, _snapshot: &GraphSnapshot<'_>, candidates: &[Binding]) -> Vec<usize> {
        let (creation, matches) = partition_candidates(candidates);
        let Some(k) = self.budget else {
            return creation.into_iter().chain(matches).collect();
        };

        // Gumbel-top-k over log-weights: sampling k without replacement
        // with probability proportional to the weights.
        let mut scored: Vec<(usize, f32)> = matches
            .into_iter()
            .map(|i| {
                let w = self
                    .weights
                    .get(candidates[i].rule_id)
                    .copied()
                    .unwrap_or(1.0)
                    .max(0.0);
                let logw = if w > 0.0 { w.ln() } else { f32::NEG_INFINITY };
                let u: f32 = self.rng.gen_range(1e-9f32..1.0);
                (i, logw + -(-u.ln()).ln())
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        creation
            .into_iter()
            .chain(
                scored
                    .into_iter()
                    // A zero-weighted rule scores -inf and is dropped even
                    // when that leaves the budget unspent.
                    .filter(|(_, s)| s.is_finite())
                    .take(k)
                    .map(|(i, _)| i),
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{candidate_pool, creation, matched, node_ids, ring_store};

    #[test]
    fn degree_greedy_high_picks_the_hub() {
        // Node 0 carries the ring's 2 edges plus 16 hub edges.
        let store = ring_store(32, 16);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        let cands: Vec<Binding> = (0..8).map(|i| matched(0, 0, &ids[i..i + 1])).collect();

        let mut high = DegreeGreedy::new(Some(1), DegreeDirection::High);
        assert_eq!(high.select(&snap, &cands), vec![0], "hub is index 0");

        let mut low = DegreeGreedy::new(Some(1), DegreeDirection::Low);
        assert_ne!(low.select(&snap, &cands), vec![0]);
    }

    #[test]
    fn degree_greedy_exempts_creations_and_holds_the_budget() {
        let store = ring_store(32, 4);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 3, 30, 2, 2);
        let mut p = DegreeGreedy::new(Some(5), DegreeDirection::High);
        let picked = p.select(&snap, &cands);
        let n_creation = picked.iter().filter(|&&i| cands[i].is_creation()).count();
        assert_eq!(n_creation, 3);
        assert_eq!(picked.len() - n_creation, 5);
    }

    #[test]
    fn degree_greedy_is_deterministic() {
        let store = ring_store(48, 6);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 1, 40, 3, 2);
        let a = DegreeGreedy::new(Some(7), DegreeDirection::High).select(&snap, &cands);
        let b = DegreeGreedy::new(Some(7), DegreeDirection::High).select(&snap, &cands);
        assert_eq!(a, b);
    }

    #[test]
    fn rule_bias_shifts_the_mix_toward_the_heavy_rule() {
        let store = ring_store(64, 4);
        let snap = store.snapshot();
        // 200 candidates split evenly over rules 0 and 1.
        let cands = candidate_pool(&store, 0, 200, 2, 1);

        let mut heavy_one = RuleBias::new(vec![1.0, 20.0], Some(20), 7);
        let picked = heavy_one.select(&snap, &cands);
        let rule1 = picked.iter().filter(|&&i| cands[i].rule_id == 1).count();
        assert!(
            rule1 >= 15,
            "rule 1 weighted 20x should dominate a 20-slot budget, got {rule1}"
        );

        let mut even = RuleBias::new(vec![1.0, 1.0], Some(20), 7);
        let picked = even.select(&snap, &cands);
        let rule1_even = picked.iter().filter(|&&i| cands[i].rule_id == 1).count();
        assert!(rule1_even < rule1, "equal weights should not favour rule 1");
    }

    #[test]
    fn rule_bias_zero_weight_excludes_a_rule_entirely() {
        let store = ring_store(32, 0);
        let snap = store.snapshot();
        let cands = candidate_pool(&store, 0, 40, 2, 1);
        let mut p = RuleBias::new(vec![0.0, 1.0], Some(30), 3);
        let picked = p.select(&snap, &cands);
        assert!(picked.iter().all(|&i| cands[i].rule_id == 1));
        // Budget deliberately left unspent rather than backfilled.
        assert_eq!(picked.len(), 20);
    }

    #[test]
    fn rule_bias_exempts_creations_even_at_zero_weight() {
        let store = ring_store(16, 0);
        let snap = store.snapshot();
        let ids = node_ids(&store);
        let cands = vec![creation(0, 0), matched(0, 0, &ids[0..1])];
        let mut p = RuleBias::new(vec![0.0], Some(4), 1);
        let picked = p.select(&snap, &cands);
        assert_eq!(picked, vec![0], "creation survives a zero-weighted rule");
    }
}
